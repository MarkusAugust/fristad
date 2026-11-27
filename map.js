// Kartet over krypten: minikartet som dreier med deg, og det store kartet
// med nord opp (K).

import { $ } from "./shared.js";

// Kartet over etasjen i kjelleren: minikartet dreier med deg (fram er
// opp), det store kartet (K) har nord opp. Bare det du har sett tegnes.
let mapData = null;
let mapWhere = null;
let bigMapOpen = false;
export function setMap(m) {
  mapData = m;
  $("minimap").hidden = !mapData;
  if (!mapData) toggleBigMap(false);
  else $("map-name").textContent = `Nivå ${m.level} · ${m.name}`;
  drawMaps();
}
export function toggleBigMap(on = !bigMapOpen) {
  bigMapOpen = on && !!mapData;
  $("bigmap").hidden = !bigMapOpen;
  drawMaps();
}
const MAP_COLORS = {
  floor: "#6b5a44", wall: "#e8d9b8", player: "#ffd75e", exit: "#4cc3ff", gate: "#c9a24a", gate_open: "#7d6a3c", chest: "#e8b84a",
  ascend: "#9be08a", boss: "#e0483a", shortcut: "#b08a5a", lever: "#d9d0c0", lever_pulled: "#8a8070", rest: "#f0c060", rest_lit: "#ffe08a",
};
// Kartet er rommene du har sett (omrisset ovenfra, x og z i meter).
function drawMap(canvas, rotate, metres) {
  const dpr = window.devicePixelRatio || 1;
  const css = canvas.getBoundingClientRect();
  if (!css.width) return;
  const w = Math.round(css.width * dpr);
  if (canvas.width !== w) canvas.width = canvas.height = w;
  const ctx = canvas.getContext("2d");
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, w, w);
  const m = mapData;
  if (!m) return;
  let x0 = Infinity, x1 = -Infinity, z0 = Infinity, z1 = -Infinity;
  for (const poly of m.rooms) for (const [x, z] of poly) {
    x0 = Math.min(x0, x); x1 = Math.max(x1, x); z0 = Math.min(z0, z); z1 = Math.max(z1, z);
  }
  if (x0 === Infinity) { x0 = -10; x1 = 10; z0 = -10; z1 = 10; }
  const me = mapWhere ?? { x: (x0 + x1) / 2, z: (z0 + z1) / 2, yaw: 0 };
  // Verden (x, z) til skjermen: dreid så fram er opp, eller nord opp med
  // hele det oppdagede kartet.
  let a, b, c, d, e, f;
  if (rotate) {
    const s = w / 2 / metres;
    const cy = Math.cos(me.yaw), sy = Math.sin(me.yaw);
    a = cy * s; b = sy * s; c = -sy * s; d = cy * s;
    e = w / 2 - (a * me.x + c * me.z);
    f = w / 2 - (b * me.x + d * me.z);
  } else {
    const span = Math.max(x1 - x0, z1 - z0, 20) * 1.1;
    const s = w / span;
    a = s; b = 0; c = 0; d = s;
    e = w / 2 - s * ((x0 + x1) / 2);
    f = w / 2 - s * ((z0 + z1) / 2);
  }
  const at = (x, z) => [a * x + c * z + e, b * x + d * z + f];
  ctx.save();
  if (rotate) {
    ctx.beginPath();
    ctx.arc(w / 2, w / 2, w / 2, 0, Math.PI * 2);
    ctx.clip();
  }
  ctx.setTransform(a, b, c, d, e, f);
  ctx.fillStyle = MAP_COLORS.floor;
  ctx.strokeStyle = MAP_COLORS.wall;
  ctx.lineWidth = rotate ? 0.3 : Math.max(0.2, 1.4 / Math.abs(a));
  ctx.lineJoin = "round";
  for (const poly of m.rooms) {
    if (poly.length < 3) continue;
    ctx.beginPath();
    poly.forEach(([x, z], i) => (i ? ctx.lineTo(x, z) : ctx.moveTo(x, z)));
    ctx.closePath();
    ctx.fill();
  }
  for (const poly of m.rooms) {
    if (poly.length < 3) continue;
    ctx.beginPath();
    poly.forEach(([x, z], i) => (i ? ctx.lineTo(x, z) : ctx.moveTo(x, z)));
    ctx.closePath();
    ctx.stroke();
  }
  // Merkene og deg selv står rett opp, med fast størrelse.
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  const r = Math.max(4, w / (rotate ? 36 : 70));
  for (const k of m.marks) {
    const [x, y] = at(k.x, k.z);
    ctx.fillStyle = MAP_COLORS[k.kind] ?? "#fff";
    ctx.beginPath();
    switch (k.kind) {
      case "exit": ctx.moveTo(x - r, y - r * 0.7); ctx.lineTo(x + r, y - r * 0.7); ctx.lineTo(x, y + r); break;
      case "ascend": ctx.moveTo(x - r, y + r * 0.7); ctx.lineTo(x + r, y + r * 0.7); ctx.lineTo(x, y - r); break;
      case "chest": ctx.rect(x - r * 0.7, y - r * 0.55, r * 1.4, r * 1.1); break;
      case "gate": case "gate_open": case "shortcut": ctx.rect(x - r * 0.35, y - r, r * 0.7, r * 2); break;
      default: ctx.arc(x, y, r * 0.8, 0, Math.PI * 2);
    }
    ctx.fill();
  }
  const [px, py] = at(me.x, me.z);
  // Pila peker dit du ser: rett opp på minikartet.
  const dir = rotate ? -Math.PI / 2 : Math.atan2(-Math.cos(me.yaw), -Math.sin(me.yaw));
  ctx.translate(px, py);
  ctx.rotate(dir + Math.PI / 2);
  ctx.fillStyle = MAP_COLORS.player;
  ctx.strokeStyle = "#1c1a17";
  ctx.lineWidth = Math.max(1, r * 0.25);
  ctx.beginPath();
  ctx.moveTo(0, -r * 1.4);
  ctx.lineTo(r * 0.9, r);
  ctx.lineTo(0, r * 0.45);
  ctx.lineTo(-r * 0.9, r);
  ctx.closePath();
  ctx.stroke();
  ctx.fill();
  ctx.restore();
}
export function drawMaps() {
  if (!mapData) return;
  drawMap($("minimap-canvas"), true, 30);
  if (bigMapOpen) drawMap($("bigmap-canvas"), false, 0);
}

export const setWhere = (w) => {
  mapWhere = w;
};
