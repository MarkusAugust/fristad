// 3D-bildet i nivåredigeringen: klikk velger, dra flytter ting og lys, og
// det du har tatt fra paletten kan slippes rett i bildet.

import { docLight, placeModule, select, send } from "./nivaa-doc.js";
import { arm, disarm, ghostFor, templateGhost } from "./nivaa-palette.js";
import { hit } from "./nivaa-plan.js";
import { canvas, engine, plan, playing, redraw, snap, st, thingY } from "./nivaa-shared.js";
import { applyOrbit, orbit, syncOrbit } from "./nivaa-view.js";

let looking = false;
/// Gulvet under pekeren i 3D (etasjen kartet viser), eller null.
function pickAt(e) {
  const r = canvas.getBoundingClientRect();
  const p = engine.level_pick((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height);
  return p.length ? p : null;
}

/// Tingen eller lyset dras i 3D: fantomet står på kartet til det slippes.
function dragGrab(g, e) {
  const p = pickAt(e);
  if (!p) return;
  const kind = g.h.type === "light" ? "light" : st.map.things.find((t) => t.id === g.h.id)?.kind;
  st.thingGhost = [snap(p[0]), st.floor, snap(p[2]), kind];
  redraw();
}

function dropGrab(g, e) {
  const p = pickAt(e);
  st.thingGhost = null;
  if (p && g.h.type === "thing") send({ op: "move_thing", id: g.h.id, to: [snap(p[0]), thingY(g.h.id), snap(p[2])] });
  else if (p && g.h.type === "light") {
    const l = docLight(g.h.id);
    if (l) send({ op: "set_light", light: { ...l, pos: [snap(p[0]), l.pos[1], snap(p[2])] } });
  }
  redraw();
}

/// Sett ut det som er tatt fra paletten der pekeren treffer gulvet i 3D
/// (med magneten, som på kartet). Gir om noe ble satt ut.
export function placeIn3d(e) {
  if (!arm) return false;
  const r = canvas.getBoundingClientRect();
  const p = engine.level_pick((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height);
  if (!p.length) return false;
  const [x, z] = [p[0], p[2]];
  let out = null;
  if (arm.type === "module") {
    const g = ghostFor(x, z);
    if (g) out = placeModule(arm.id, g.at, g.rot, arm.mirror);
  } else if (arm.type === "thing") out = send({ op: "add_thing", kind: arm.kind, at: [snap(x), st.floor, snap(z)], yaw: arm.yaw, label: arm.label });
  else if (arm.type === "light") out = send({ op: "add_light", at: [snap(x), st.floor + 2.4, snap(z)] });
  else if (arm.type === "template") {
    const g = templateGhost(x, z);
    if (g) out = send({ op: "stamp", template: arm.tpl, at: g.at, rot: arm.rot, mirror: arm.mirror });
  }
  if (out?.made?.length) select(out.made);
  if (!e.shiftKey) disarm();
  return true;
}

export function bind3d() {
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  // Midtknappen flytter; ikke nettleserens autorulling.
  for (const el of [canvas, plan]) el.addEventListener("mousedown", (e) => e.button === 1 && e.preventDefault());
  let press = null;
  canvas.addEventListener("pointerdown", (e) => {
    canvas.focus();
    if (playing) {
      canvas.requestPointerLock?.();
      engine.mouse(e.button, true);
      return;
    }
    canvas.setPointerCapture(e.pointerId);
    if (e.button === 2) {
      looking = true;
      return;
    }
    press = { button: e.button, x: e.clientX, y: e.clientY, last: [e.clientX, e.clientY], moved: false };
    // Dra i en valgt ting eller et lys: flytt det (ellers går kameraet
    // rundt, også over en valgt modul, som flyttes på kartet).
    if (e.button === 0 && !arm && st.sel.length) {
      const p = pickAt(e);
      const h = p && hit(st.map, st.floor, p[0], p[2], 0.8);
      if (h && st.sel.includes(h.id) && ["thing", "light"].includes(h.type)) press.grab = { h, from: p };
    }
  });
  canvas.addEventListener("pointerup", (e) => {
    if (playing) return engine.mouse(e.button, false);
    if (e.button === 2) {
      looking = false;
      // Etter flygingen går kameraet rundt det det ser på.
      syncOrbit();
      return;
    }
    const p0 = press;
    press = null;
    if (p0?.grab && p0.moved) return dropGrab(p0.grab, e);
    if (!p0 || p0.moved || e.button !== 0) return;
    // Noe tatt fra paletten: sett det ut der.
    if (arm && placeIn3d(e)) return;
    // Et klikk uten å dra velger det som står der.
    const r = canvas.getBoundingClientRect();
    const p = engine.level_pick((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height);
    if (p.length) {
      const h = hit(st.map, st.floor, p[0], p[2], 0.8);
      if (h?.type === "module" || h?.type === "thing") select([h.id], e.shiftKey);
      else if (!e.shiftKey) select([]);
    }
  });
  canvas.addEventListener("pointermove", (e) => {
    if (looking || (playing && document.pointerLockElement === canvas)) return engine.look(e.movementX, e.movementY);
    if (!press) return;
    const dx = e.clientX - press.last[0];
    const dy = e.clientY - press.last[1];
    press.last = [e.clientX, e.clientY];
    if (!press.moved) {
      if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < 4) return;
      press.moved = true;
      if (!press.grab) syncOrbit();
    }
    if (press.grab) return dragGrab(press.grab, e);
    if (press.button === 0 && !e.shiftKey) {
      // Gå rundt punktet.
      orbit.yaw -= dx * 0.4;
      orbit.pitch = Math.max(-89, Math.min(20, orbit.pitch - dy * 0.3));
    } else if ((press.button === 1 || (press.button === 0 && e.shiftKey)) && orbit.pivot) {
      // Flytt punktet langs bildet.
      const a = (orbit.yaw * Math.PI) / 180;
      const k = orbit.dist * 0.0018;
      orbit.pivot = [orbit.pivot[0] - Math.cos(a) * dx * k, orbit.pivot[1] + dy * k, orbit.pivot[2] + Math.sin(a) * dx * k];
    }
    applyOrbit();
  });
  canvas.addEventListener(
    "wheel",
    (e) => {
      if (playing) return;
      e.preventDefault();
      syncOrbit();
      if (!orbit.pivot) return;
      orbit.dist = Math.max(1.5, Math.min(160, orbit.dist * Math.exp(e.deltaY * 0.0012)));
      applyOrbit();
    },
    { passive: false },
  );
}
