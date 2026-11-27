// Utsnittet i nivåredigeringen: kart, delt eller 3D, etasjen og snittet,
// banekameraet i 3D og det å ramme inn noe, og bokmerkene.

import { docPath, select } from "./nivaa-doc.js";
import { CELL, hit } from "./nivaa-plan.js";
import { $, canvas, engine, MARKS, plan, redraw, st, store, toast } from "./nivaa-shared.js";

export let viewMode = "split";
export function fit() {
  const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
  const w = Math.max(1, Math.round(canvas.clientWidth * dpr));
  const h = Math.max(1, Math.round(canvas.clientHeight * dpr));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  engine?.resize(w, h);
}

function marks() {
  try {
    return JSON.parse(store.get(MARKS + docPath) || "{}");
  } catch {
    return {};
  }
}

/// Ctrl+1–9 (Shift+1–9 der nettleseren tar Ctrl): husk kameraet i 3D,
/// utsnittet i kartet og etasjen.
export function markCamera(n) {
  const all = marks();
  all[n] = { camera: engine.level_camera(), view: [st.view.cx, st.view.cz, st.view.s], floor: st.floor };
  store.set(MARKS + docPath, JSON.stringify(all));
  toast(`Bokmerke ${n} husket.`, 1500);
}

/// 1–9: tilbake til bokmerket.
export function gotoMark(n) {
  const m = marks()[n];
  if (!m) return toast(`Ingen bokmerke ${n} ennå (Ctrl+${n} eller Shift+${n} husker kameraet).`, 2000);
  setFloor(m.floor);
  [st.view.cx, st.view.cz, st.view.s] = m.view;
  if (m.camera?.length) {
    const [x, y, z, yaw, pitch] = m.camera;
    engine.level(JSON.stringify({ cmd: "place", x, y, z, yaw, pitch }));
  }
  redraw();
}

export function centerOn() {
  const cells = st.map.modules.flatMap((m) => m.cells);
  if (!cells.length) return redraw();
  const xs = cells.map((c) => c[0] * CELL);
  const zs = cells.map((c) => c[2] * CELL);
  const [x0, x1, z0, z1] = [Math.min(...xs) - 4, Math.max(...xs) + 4, Math.min(...zs) - 4, Math.max(...zs) + 4];
  st.view.cx = (x0 + x1) / 2;
  st.view.cz = (z0 + z1) / 2;
  const w = plan.clientWidth || 400;
  const h = plan.clientHeight || 300;
  st.view.s = Math.max(3, Math.min(40, Math.min(w / (x1 - x0), h / (z1 - z0)) * 0.9));
  redraw();
}

export function setFloor(y) {
  st.floor = y;
  $("floor").value = String(y);
  engine.level(JSON.stringify({ cmd: "floor", y }));
  sendCut();
  redraw();
}

/// Snittet i 3D (S): alt over etasjen kartet viser kuttes bort
/// (`LevelCommand::Cut`, `fs_clip`), så du ser inn i den under en annen.
export function sendCut() {
  engine.level(JSON.stringify({ cmd: "cut", y: st.cut ? st.floor : null }));
}

// Banekameraet i 3D: kameraet går rundt et punkt (`pivot`) i en avstand og
// retning (grader; pitch negativ ser ned). Venstre dra går rundt, hjulet
// zoomer, midtknappen flytter, høyre knapp + WASD flyr som før.
export const orbit = { pivot: null, dist: 18, yaw: 35, pitch: -45 };

function camForward(yaw, pitch) {
  const a = (yaw * Math.PI) / 180;
  const p = (pitch * Math.PI) / 180;
  return [-Math.sin(a) * Math.cos(p), Math.sin(p), -Math.cos(a) * Math.cos(p)];
}

export function applyOrbit() {
  if (!orbit.pivot) return;
  const f = camForward(orbit.yaw, orbit.pitch);
  const [x, y, z] = orbit.pivot;
  engine.level(JSON.stringify({ cmd: "place", x: x - f[0] * orbit.dist, y: y - f[1] * orbit.dist, z: z - f[2] * orbit.dist, yaw: orbit.yaw, pitch: orbit.pitch }));
  st.camera = engine.level_camera();
}

/// Punktet kameraet går rundt, fra kameraet slik det står nå (etter flyging
/// eller et hopp i kartet): gulvet midt i bildet, ellers et stykke fram.
export function syncOrbit() {
  const c = engine.level_camera();
  if (!c.length) return;
  orbit.yaw = c[3];
  orbit.pitch = c[4];
  const f = camForward(c[3], c[4]);
  const p = engine.level_pick(0.5, 0.5);
  const d = p.length ? Math.hypot(p[0] - c[0], p[1] - c[1], p[2] - c[2]) : 0;
  orbit.dist = d > 0.5 && d < 120 ? d : Math.max(4, Math.min(orbit.dist, 30));
  orbit.pivot = [c[0] + f[0] * orbit.dist, c[1] + f[1] * orbit.dist, c[2] + f[2] * orbit.dist];
}

/// Vis rutene (i, y, j) skrått ovenfra, så alt får plass i 3D.
function frameCells(cells, keepYaw = true) {
  if (!cells.length) return;
  const xs = cells.map((c) => c[0] * CELL);
  const zs = cells.map((c) => c[2] * CELL);
  const ys = cells.map((c) => c[1]);
  const span = Math.max(Math.max(...xs) - Math.min(...xs), Math.max(...zs) - Math.min(...zs)) + CELL;
  orbit.pivot = [(Math.min(...xs) + Math.max(...xs)) / 2, Math.min(...ys), (Math.min(...zs) + Math.max(...zs)) / 2];
  orbit.dist = span * 0.85 + 6;
  orbit.pitch = -50;
  if (!keepYaw || !st.camera?.length) orbit.yaw = 35;
  else orbit.yaw = st.camera[3];
  applyOrbit();
}

/// Hele etasjen i 3D.
export function frameFloor() {
  const cells = st.map.modules.flatMap((m) => m.cells).filter((c) => c[1] >= st.floor - 0.5 && c[1] < st.floor + 2.5);
  frameCells(cells.length ? cells : st.map.modules.flatMap((m) => m.cells), false);
}

export function focusSelection() {
  const mods = st.map.modules.filter((m) => st.sel.includes(m.id));
  const things = st.map.things.filter((t) => st.sel.includes(t.id));
  const cells = [...mods.flatMap((m) => m.cells), ...things.map((t) => [Math.round(t.pos[0] / CELL), t.pos[1], Math.round(t.pos[2] / CELL)])];
  if (!cells.length) return frameFloor();
  st.view.cx = (Math.min(...cells.map((c) => c[0])) + Math.max(...cells.map((c) => c[0]))) * CELL / 2;
  st.view.cz = (Math.min(...cells.map((c) => c[2])) + Math.max(...cells.map((c) => c[2]))) * CELL / 2;
  frameCells(cells);
  redraw();
}

export function setView(m) {
  viewMode = m;
  const v = $("views");
  v.classList.toggle("only-map", m === "map");
  v.classList.toggle("only-3d", m === "3d");
  $("v-map").setAttribute("aria-pressed", m === "map");
  $("v-split").setAttribute("aria-pressed", m === "split");
  $("v-3d").setAttribute("aria-pressed", m === "3d");
  requestAnimationFrame(() => {
    fit();
    redraw();
  });
}

/// Gå til et sted i kartet og 3D: etasjen, utsnittet, valget og et blink.
export function goTo(at, pick = true) {
  const [x, y, z] = at;
  const f = st.map.floors.reduce((a, b) => (Math.abs(b - y) < Math.abs(a - y) ? b : a), st.floor);
  setFloor(f);
  st.view.cx = x;
  st.view.cz = z;
  st.flash = at;
  setTimeout(() => {
    st.flash = null;
    redraw();
  }, 1500);
  if (pick) {
    const h = hit(st.map, f, x, z, 1);
    if (h?.type === "module" || h?.type === "thing") select([h.id]);
  }
  orbit.pivot = [x, y, z];
  orbit.dist = 12;
  orbit.pitch = -50;
  if (st.camera?.length) orbit.yaw = st.camera[3];
  applyOrbit();
  redraw();
}
