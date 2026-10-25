// Kartet ovenfra i nivåredigeringen (`?nivaa`, web/nivaa.js): tegner
// kartet motoren gir (`engine.level_map()`, `edit::level::map`) i et
// Canvas 2D, etasje for etasje, og finner det som ligger under pekeren.
// Alt er i nivåets rom i meter: cellene er 4 × 4 m med midten i (4i, 4j),
// nord (−z) opp.

export const CELL = 4;
const SIDE = [
  [0, -1],
  [1, 0],
  [0, 1],
  [-1, 0],
];

// Fargene etter slaget modulen er og slaget åpningen ble.
export const KIND_FILL = { rom: "#5b4a34", gang: "#3c4855", trapp: "#4a5838", ferdig: "#5e3b3b" };
export const EDGE_COLOR = { door: "#e8a33c", gate: "#ff5a4a", secret: "#b06cff", gang: "#4fa3ff", arch: "#7fd18b" };
const EDGE_WIDTH = { door: 1.6, gate: 2.8, secret: 2.0, gang: 3.2, arch: 3.6 };
// Valgene for en åpning (eierens), som farge.
export const CHOSEN_COLOR = { dor: "#e8a33c", port: "#ff5a4a", falsk: "#b06cff", gang: "#4fa3ff", bue: "#7fd18b", snarvei: "#ffd84a", vegg: "#9a9a9a", auto: "#d9d4c7" };
const THING_STYLE = {
  start: ["S", "#57d36a"],
  up: ["↑", "#6fc3ff"],
  down: ["↓", "#6fc3ff"],
  rest: ["✚", "#ffcf6a"],
  mob: ["F", "#ff5f5f"],
  elite: ["E", "#ff9a3c"],
  boss: ["B", "#d1202a"],
  chest: ["K", "#e6c15a"],
  rare: ["K", "#c48bff"],
  lore: ["L", "#e7dcc0"],
  bonus: ["+", "#7fe0c0"],
  lever: ["⌐", "#4fd3e0"],
  trap: ["T", "#ff7ad1"],
  hint: ["·", "#b06cff"],
  gate: ["", "#ff5a4a"],
  secret_wall: ["", "#b06cff"],
  shortcut: ["", "#ffd84a"],
};

/// Skjermen ↔ nivået.
export function makeView(canvas) {
  return {
    cx: 8,
    cz: 8,
    s: 14,
    w: canvas.clientWidth || 400,
    h: canvas.clientHeight || 300,
    toScreen(x, z) {
      return [(x - this.cx) * this.s + this.w / 2, (z - this.cz) * this.s + this.h / 2];
    },
    toWorld(px, py) {
      return [(px - this.w / 2) / this.s + this.cx, (py - this.h / 2) / this.s + this.cz];
    },
  };
}

/// Rutene og åpningene til en modul i paletten dreid `rot` kvart
/// omdreininger med klokka og speilet langs x (først), med midtcella i `at`.
export function placeFootprint(pm, at, rot, mirror) {
  const turn = ([i, j]) => {
    let p = [mirror ? -i : i, j];
    for (let k = 0; k < (rot & 3); k++) p = [-p[1], p[0]];
    return p;
  };
  const side = (l) => {
    let s = mirror ? [0, 3, 2, 1][l] : l;
    return (s + rot) & 3;
  };
  const cells = pm.cells.map((c) => {
    const [i, j] = turn([c[0], c[2]]);
    return [at[0] + i, at[1] + c[1], at[2] + j];
  });
  const openings = pm.openings.map((o) => {
    const [i, j] = turn([o.at[0], o.at[1]]);
    return { ...o, at: [at[0] + i, at[2] + j], y: at[1] + o.y, side: side(sideFromPlaced(pm, o)) };
  });
  return { cells, openings };
}

/// Åpningene i nivået som ikke møter noe (og ikke er gjort til mur), utenom
/// modulene i `skip`: der en ny modul kan kobles på.
export function freeOpenings(map, skip = []) {
  const joined = new Set(map.edges.flatMap((e) => [`${e.at[0]},${e.at[2]},${e.side}`, `${e.at[0] + SIDE[e.side][0]},${e.at[2] + SIDE[e.side][1]},${(e.side + 2) & 3}`]));
  const out = [];
  for (const m of map.modules) {
    if (skip.includes(m.id)) continue;
    for (const o of m.openings) if (o.chosen !== "vegg" && !joined.has(`${o.at[0]},${o.at[1]},${o.side}`)) out.push(o);
  }
  return out;
}

/// Magneten: der modulen `pm` (dreid `rots`, speilet `mirror`) kan stå med en
/// åpning mot en ledig åpning i nivået, nærmest `want` (ruta pekeren peker på)
/// og innen `reach` ruter. `taken` er rutene som er opptatt ("i,y,j"). Gir
/// { at, rot, links } (links: midten av kantene som kobles, i meter) eller null.
export function snapPlacement(map, pm, want, rots, mirror, taken, skip = [], reach = 1.6) {
  const free = freeOpenings(map, skip).filter((f) => Math.abs(f.y - want[1]) < 2.5 && Math.hypot(f.at[0] - want[0], f.at[1] - want[2]) < reach + 6);
  if (!free.length) return null;
  const freeKey = new Map(free.map((f) => [`${f.at[0] + SIDE[f.side][0]},${f.y},${f.at[1] + SIDE[f.side][1]},${(f.side + 2) & 3}`, f]));
  let best = null;
  rots.forEach((rot, k) => {
    const fp0 = placeFootprint(pm, [0, 0, 0], rot, mirror);
    for (const f of free) {
      const c = [f.at[0] + SIDE[f.side][0], f.at[1] + SIDE[f.side][1]];
      for (const g of fp0.openings) {
        if (g.side !== ((f.side + 2) & 3)) continue;
        const at = [c[0] - g.at[0], f.y - g.y, c[1] - g.at[1]];
        const d = Math.hypot(at[0] - want[0], at[2] - want[2]);
        if (d > reach) continue;
        const fp = placeFootprint(pm, at, rot, mirror);
        if (fp.cells.some((q) => taken.has(q.join()))) continue;
        const links = fp.openings.filter((o) => freeKey.has(`${o.at[0]},${o.y},${o.at[1]},${o.side}`));
        const score = d - 0.35 * (links.length - 1) + 0.05 * k;
        if (!best || score < best.score) {
          best = { score, at, rot, links: links.map((o) => [o.at[0] * CELL + (SIDE[o.side][0] * CELL) / 2, o.y, o.at[1] * CELL + (SIDE[o.side][1] * CELL) / 2]) };
        }
      }
    }
  });
  return best;
}

// I paletten er åpningene allerede satt ut med rot 0 (siden i nivået er
// den lokale).
function sideFromPlaced(_pm, o) {
  return o.side;
}

/// Tegn kartet. `st`: { map, floor, view, sel (id-er), selOpening, ghost, box, hover, link, issueAt }.
export function draw(canvas, st) {
  const ctx = canvas.getContext("2d");
  const dpr = window.devicePixelRatio || 1;
  const W = canvas.clientWidth;
  const H = canvas.clientHeight;
  if (canvas.width !== Math.round(W * dpr) || canvas.height !== Math.round(H * dpr)) {
    canvas.width = Math.round(W * dpr);
    canvas.height = Math.round(H * dpr);
  }
  const v = st.view;
  v.w = W;
  v.h = H;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = "#0e0f11";
  ctx.fillRect(0, 0, W, H);
  const map = st.map;
  if (!map) return;
  const s = v.s;
  // Rutenettet.
  const [x0, z0] = v.toWorld(0, 0);
  const [x1, z1] = v.toWorld(W, H);
  ctx.lineWidth = 1;
  for (let i = Math.floor(x0 / CELL) - 1; i <= Math.ceil(x1 / CELL) + 1; i++) {
    const [px] = v.toScreen(i * CELL - CELL / 2, 0);
    ctx.strokeStyle = i % 5 === 0 ? "#26292e" : "#1a1c20";
    ctx.beginPath();
    ctx.moveTo(px + 0.5, 0);
    ctx.lineTo(px + 0.5, H);
    ctx.stroke();
  }
  for (let j = Math.floor(z0 / CELL) - 1; j <= Math.ceil(z1 / CELL) + 1; j++) {
    const [, py] = v.toScreen(0, j * CELL - CELL / 2);
    ctx.strokeStyle = j % 5 === 0 ? "#26292e" : "#1a1c20";
    ctx.beginPath();
    ctx.moveTo(0, py + 0.5);
    ctx.lineTo(W, py + 0.5);
    ctx.stroke();
  }
  const floor = st.floor;
  const onFloor = (y) => y >= floor - 0.5 && y < floor + 2.5;
  // Etasjen under (den nærmeste under denne): gjennomsiktig, så du ser hva
  // trappene, stigene og heisene skal ned til. Med snittet (S) står bare den
  // og denne; ellers er de andre stiplet.
  const below = (map.floors || []).filter((f) => f < floor - 2.5).reduce((a, f) => (a === null || f > a ? f : a), null);
  const onBelow = (y) => below !== null && y >= below - 0.5 && y < below + 2.5;
  const rect = (i, j, inset) => {
    const [a, b] = v.toScreen(i * CELL - CELL / 2 + inset, j * CELL - CELL / 2 + inset);
    const size = (CELL - 2 * inset) * s;
    return [a, b, size, size];
  };
  // Modulene: andre etasjer svakt, så denne.
  for (const pass of [0, 1]) {
    for (const m of map.modules) {
      const here = m.cells.filter((c) => onFloor(c[1]));
      if (pass === 0) {
        const under = m.cells.filter((c) => onBelow(c[1]) && !onFloor(c[1]));
        if (under.length) {
          ctx.globalAlpha = 0.22;
          ctx.fillStyle = KIND_FILL[m.kind] || "#4a4a4a";
          for (const c of under) ctx.fillRect(...rect(c[0], c[2], 0.25));
          ctx.globalAlpha = 1;
        }
        if (here.length || st.cut) continue;
        ctx.strokeStyle = "#2c3036";
        ctx.setLineDash([3, 3]);
        for (const c of m.cells) if (!onBelow(c[1])) ctx.strokeRect(...rect(c[0], c[2], 0.3));
        ctx.setLineDash([]);
        continue;
      }
      if (!here.length) continue;
      const selected = st.sel.includes(m.id);
      ctx.fillStyle = KIND_FILL[m.kind] || "#4a4a4a";
      for (const c of here) {
        ctx.globalAlpha = c[1] === floor ? 1 : 0.75;
        ctx.fillRect(...rect(c[0], c[2], 0.25));
      }
      ctx.globalAlpha = 1;
      // Trinnene: en strek per meter opp.
      ctx.strokeStyle = "rgba(255,255,255,.12)";
      for (const c of here.filter((c) => c[1] !== floor)) {
        const [a, b, w] = rect(c[0], c[2], 0.6);
        ctx.strokeRect(a, b, w, w);
      }
      // Omrisset: sidene der naboen ikke er samme modul.
      const key = (i, j) => `${i},${j}`;
      const mine = new Set(m.cells.map((c) => key(c[0], c[2])));
      ctx.strokeStyle = selected ? "#ffd36a" : "#a99a7c";
      ctx.lineWidth = selected ? 3 : 1.5;
      for (const c of here) {
        for (let w = 0; w < 4; w++) {
          const [di, dj] = SIDE[w];
          if (mine.has(key(c[0] + di, c[2] + dj))) continue;
          sideLine(ctx, v, c[0], c[2], w);
        }
      }
      ctx.lineWidth = 1;
      // Navnet: øverst til venstre i modulen, kortet av så det aldri går
      // ut over kanten (og inn i naboens).
      if (s > 6) {
        const i0 = Math.min(...here.map((c) => c[0]));
        const i1 = Math.max(...here.map((c) => c[0]));
        const j0 = Math.min(...here.map((c) => c[2]));
        const [lx, ly] = v.toScreen(i0 * CELL - CELL / 2, j0 * CELL - CELL / 2);
        const room = (i1 - i0 + 1) * CELL * s - 8;
        ctx.font = `600 ${Math.max(10, Math.min(13, s * 0.7))}px Inter, sans-serif`;
        const text = fitText(ctx, m.name, room);
        if (text) {
          ctx.textAlign = "left";
          ctx.textBaseline = "top";
          const w = ctx.measureText(text).width;
          ctx.fillStyle = "rgba(10,11,13,.6)";
          ctx.fillRect(lx + 3, ly + 3, w + 6, Math.max(10, Math.min(13, s * 0.7)) + 5);
          ctx.fillStyle = selected ? "#ffe7a8" : "#e7e1d2";
          ctx.fillText(text, lx + 6, ly + 5);
        }
      }
    }
  }
  // Åpningene etter sammenføyningen; buene og gangene inne i en modul er
  // bare modulens eget rom og tegnes ikke.
  const owner = new Map();
  for (const m of map.modules) for (const c of m.cells) if (onFloor(c[1])) owner.set(`${c[0]},${c[2]}`, m.id);
  for (const e of map.edges) {
    if (!onFloor(e.at[1])) continue;
    const a = owner.get(`${e.at[0]},${e.at[2]}`);
    if ((e.kind === "arch" || e.kind === "gang") && a && a === owner.get(`${e.at[0] + SIDE[e.side][0]},${e.at[2] + SIDE[e.side][1]}`)) continue;
    const [di, dj] = SIDE[e.side];
    const mx = e.at[0] * CELL + di * CELL / 2;
    const mz = e.at[2] * CELL + dj * CELL / 2;
    const half = (EDGE_WIDTH[e.kind] || 2) / 2;
    const [ax, az] = [mx - Math.abs(dj) * half, mz - Math.abs(di) * half];
    const [bx, bz] = [mx + Math.abs(dj) * half, mz + Math.abs(di) * half];
    ctx.strokeStyle = EDGE_COLOR[e.kind] || "#ddd";
    ctx.lineWidth = Math.max(3, s * 0.35);
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(...v.toScreen(ax, az));
    ctx.lineTo(...v.toScreen(bx, bz));
    ctx.stroke();
    ctx.lineCap = "butt";
    ctx.lineWidth = 1;
  }
  // Åpningene som ikke møter noe (piler ut), og eierens valg.
  const joined = new Set(map.edges.flatMap((e) => [`${e.at[0]},${e.at[2]},${e.side}`, `${e.at[0] + SIDE[e.side][0]},${e.at[2] + SIDE[e.side][1]},${(e.side + 2) & 3}`]));
  for (const m of map.modules) {
    for (const o of m.openings) {
      if (!onFloor(o.y)) continue;
      const isSel = st.selOpening && st.selOpening.module === m.id && st.selOpening.opening === o.id;
      const free = !joined.has(`${o.at[0]},${o.at[1]},${o.side}`);
      if (!free && !o.chosen && !isSel) continue;
      // Med en modul i hånda lyser de ledige åpningene: der den kan kobles på.
      const want = st.ghost && free && o.chosen !== "vegg";
      arrow(ctx, v, o.at[0], o.at[1], o.side, isSel ? "#ffd36a" : want ? "#7be38f" : o.chosen ? CHOSEN_COLOR[o.chosen] || "#ddd" : "rgba(220,214,200,.45)", isSel ? 1.3 : want ? 1.25 : 1);
    }
  }
  // Stigene og heisene: enden i en annen etasje (⇅ med høyden).
  for (const t of map.things) {
    if (!t.other_end) continue;
    for (const [p, q] of [
      [t.pos, t.other_end],
      [t.other_end, t.pos],
    ]) {
      if (!onFloor(p[1]) || onFloor(q[1])) continue;
      const [px, py] = v.toScreen(p[0], p[2]);
      ctx.fillStyle = "#4fd3e0";
      ctx.font = "600 12px Inter, sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(`⇅ ${q[1] > p[1] ? "+" : ""}${Math.round(q[1] - p[1])} m`, px, py - 14);
    }
  }
  // Koblingene og patruljene.
  const pos = new Map(map.things.map((t) => [t.id, t.pos]));
  ctx.strokeStyle = "#5aa7ff";
  ctx.lineWidth = 2;
  for (const l of map.links) {
    const a = pos.get(l.from);
    const b = pos.get(l.to);
    if (!a || !b || !(onFloor(a[1]) || onFloor(b[1]))) continue;
    ctx.setLineDash(l.own ? [] : [4, 4]);
    linkArrow(ctx, v.toScreen(a[0], a[2]), v.toScreen(b[0], b[2]));
  }
  ctx.setLineDash([]);
  for (const t of map.things) {
    if (!t.patrol.length || !onFloor(t.pos[1])) continue;
    ctx.strokeStyle = "rgba(255,140,60,.8)";
    ctx.setLineDash([5, 4]);
    ctx.beginPath();
    ctx.moveTo(...v.toScreen(t.pos[0], t.pos[2]));
    for (const p of t.patrol) ctx.lineTo(...v.toScreen(p[0], p[2]));
    ctx.stroke();
    ctx.setLineDash([]);
    for (const p of t.patrol) {
      const [px, py] = v.toScreen(p[0], p[2]);
      ctx.fillStyle = "rgba(255,140,60,.9)";
      ctx.fillRect(px - 3, py - 3, 6, 6);
    }
  }
  ctx.lineWidth = 1;
  // Lysene.
  for (const l of map.lights) {
    if (!onFloor(l.pos[1] - 1.5)) continue;
    const [px, py] = v.toScreen(l.pos[0], l.pos[2]);
    const c = l.color.map((x) => Math.round(Math.min(1, x) * 255));
    if (!l.own) {
      // Faklene og lysene fra modulene: små prikker.
      ctx.fillStyle = `rgba(${c[0]},${c[1]},${c[2]},.45)`;
      ctx.fillRect(px - 1.5, py - 1.5, 3, 3);
      continue;
    }
    const selected = st.sel.includes(l.id);
    if (selected) {
      ctx.strokeStyle = `rgba(${c[0]},${c[1]},${c[2]},.5)`;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.arc(px, py, l.range * s, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);
    }
    ctx.fillStyle = `rgb(${c[0]},${c[1]},${c[2]})`;
    ctx.beginPath();
    ctx.arc(px, py, selected ? 7 : 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = selected ? "#ffd36a" : "#101114";
    ctx.lineWidth = selected ? 2.5 : 1.5;
    ctx.stroke();
    ctx.lineWidth = 1;
  }
  // Innholdet.
  const r = Math.max(6, Math.min(11, s * 0.5));
  for (const t of map.things) {
    if (!onFloor(t.pos[1])) continue;
    const [px, py] = v.toScreen(t.pos[0], t.pos[2]);
    if (t.kind === "prop") {
      ctx.fillStyle = "rgba(160,150,130,.5)";
      ctx.fillRect(px - 2, py - 2, 4, 4);
      continue;
    }
    const [glyph, color] = THING_STYLE[t.kind] || ["?", "#ccc"];
    if (!glyph) continue;
    const selected = st.sel.includes(t.id);
    ctx.globalAlpha = t.own ? 1 : 0.7;
    ctx.fillStyle = "#101114";
    ctx.beginPath();
    ctx.arc(px, py, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.lineWidth = selected ? 3 : 1.5;
    ctx.strokeStyle = selected ? "#ffd36a" : color;
    ctx.stroke();
    // Retningen.
    const yaw = (t.yaw * Math.PI) / 180;
    ctx.beginPath();
    ctx.moveTo(px + Math.sin(yaw) * r, py + Math.cos(yaw) * r);
    ctx.lineTo(px + Math.sin(yaw) * (r + 5), py + Math.cos(yaw) * (r + 5));
    ctx.stroke();
    ctx.fillStyle = color;
    ctx.font = `600 ${Math.round(r * 1.1)}px Inter, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(glyph, px, py + 0.5);
    ctx.globalAlpha = 1;
    ctx.lineWidth = 1;
  }
  // Bruddene.
  for (const i of map.issues) {
    if (!i.at || !onFloor(i.at[1])) continue;
    const [px, py] = v.toScreen(i.at[0], i.at[2]);
    ctx.strokeStyle = i.severity === "feil" ? "rgba(255,80,70,.95)" : "rgba(240,200,80,.8)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(px, py, r + 5, 0, Math.PI * 2);
    ctx.stroke();
    ctx.lineWidth = 1;
  }
  if (st.flash) {
    const [px, py] = v.toScreen(st.flash[0], st.flash[2]);
    ctx.strokeStyle = "#ffd36a";
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(px, py, r + 12, 0, Math.PI * 2);
    ctx.stroke();
    ctx.lineWidth = 1;
  }
  // Fantomet.
  if (st.ghost) {
    const g = st.ghost;
    ctx.globalAlpha = 0.55;
    ctx.fillStyle = g.ok ? "#4fbf6a" : "#e0483c";
    for (const c of g.cells) if (onFloor(c[1])) ctx.fillRect(...rect(c[0], c[2], 0.25));
    ctx.globalAlpha = 1;
    for (const o of g.openings) if (onFloor(o.y)) arrow(ctx, v, o.at[0], o.at[1], o.side, "#e8ffe8", 1);
    // Magneten: der fantomet kobles til en åpning i nivået.
    for (const [x, , z] of g.links || []) {
      const [px, py] = v.toScreen(x, z);
      ctx.fillStyle = "#57d36a";
      ctx.strokeStyle = "#0e0f11";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(px, py, Math.max(5, Math.min(9, s * 0.4)), 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      ctx.lineWidth = 1;
    }
  }
  if (st.thingGhost) {
    const [px, py] = v.toScreen(st.thingGhost[0], st.thingGhost[2]);
    // Utenfor modulene (i fjellet) er fantomet rødt.
    const inside = owner.has(`${Math.round(st.thingGhost[0] / CELL)},${Math.round(st.thingGhost[2] / CELL)}`);
    let [glyph, color] = THING_STYLE[st.thingGhost[3]] || (st.thingGhost[3] === "light" ? ["☀", "#ffcf6a"] : ["", "#e8ffe8"]);
    if (!inside) color = "#e0483c";
    ctx.fillStyle = "rgba(16,17,20,.7)";
    ctx.beginPath();
    ctx.arc(px, py, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = color;
    ctx.setLineDash([3, 3]);
    ctx.stroke();
    ctx.setLineDash([]);
    if (glyph) {
      ctx.fillStyle = color;
      ctx.font = `600 ${Math.round(r * 1.1)}px Inter, sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(glyph, px, py + 0.5);
    }
  }
  if (st.box) {
    const [a, b, c, d] = st.box;
    ctx.strokeStyle = "#ffd36a";
    ctx.setLineDash([4, 3]);
    ctx.strokeRect(Math.min(a, c), Math.min(b, d), Math.abs(c - a), Math.abs(d - b));
    ctx.setLineDash([]);
  }
  if (st.linkFrom) {
    const a = pos.get(st.linkFrom);
    if (a && st.mouse) {
      ctx.strokeStyle = "#5aa7ff";
      ctx.setLineDash([6, 4]);
      linkArrow(ctx, v.toScreen(a[0], a[2]), st.mouse);
      ctx.setLineDash([]);
    }
  }
  drawMini(ctx, st, W, H);
  // Kameraet i 3D.
  if (st.camera && st.camera.length) {
    const [x, , z, yaw] = st.camera;
    const [px, py] = v.toScreen(x, z);
    const a = (yaw * Math.PI) / 180;
    const fx = -Math.sin(a);
    const fz = -Math.cos(a);
    ctx.fillStyle = "rgba(120,200,255,.25)";
    ctx.beginPath();
    ctx.moveTo(px, py);
    const span = 0.6;
    for (const d of [-span, span]) {
      const c = Math.cos(d);
      const sn = Math.sin(d);
      ctx.lineTo(px + (fx * c - fz * sn) * 40, py + (fx * sn + fz * c) * 40);
    }
    ctx.closePath();
    ctx.fill();
    ctx.fillStyle = "#8fd0ff";
    ctx.beginPath();
    ctx.arc(px, py, 4, 0, Math.PI * 2);
    ctx.fill();
  }
}

/// Minikartet nede til høyre: hele nivået med alle etasjene (denne
/// sterkest), utsnittet kartet viser og 3D-kameraet. Gir ruta det står i
/// og hvordan et punkt der blir et sted i nivået (`toWorld`), eller null.
export function miniMap(map, W, H, view) {
  const cells = map.modules.flatMap((m) => m.cells);
  if (!cells.length || W < 360 || H < 260) return null;
  const xs = cells.map((c) => c[0] * CELL);
  const zs = cells.map((c) => c[2] * CELL);
  const [x0, x1, z0, z1] = [Math.min(...xs) - 6, Math.max(...xs) + 6, Math.min(...zs) - 6, Math.max(...zs) + 6];
  // Bare når nivået ikke får plass i utsnittet.
  if (view) {
    const [a, b] = view.toScreen(x0 + 4, z0 + 4);
    const [c, d] = view.toScreen(x1 - 4, z1 - 4);
    if (a >= 0 && b >= 0 && c <= W && d <= H) return null;
  }
  const w = 170;
  const h = 130;
  const k = Math.min(w / (x1 - x0), h / (z1 - z0));
  const rect = [W - w - 12, 30, w, h];
  const ox = rect[0] + (w - (x1 - x0) * k) / 2;
  const oz = rect[1] + (h - (z1 - z0) * k) / 2;
  return {
    rect,
    k,
    toScreen: (x, z) => [ox + (x - x0) * k, oz + (z - z0) * k],
    toWorld: (px, py) => [(px - ox) / k + x0, (py - oz) / k + z0],
    inside: (px, py) => px >= rect[0] && px <= rect[0] + w && py >= rect[1] && py <= rect[1] + h,
  };
}

function drawMini(ctx, st, W, H) {
  const mini = miniMap(st.map, W, H, st.view);
  if (!mini) return;
  const [rx, ry, rw, rh] = mini.rect;
  ctx.fillStyle = "rgba(10,11,13,.88)";
  ctx.fillRect(rx - 4, ry - 4, rw + 8, rh + 8);
  ctx.strokeStyle = "#30343a";
  ctx.strokeRect(rx - 4.5, ry - 4.5, rw + 9, rh + 9);
  const size = CELL * mini.k;
  for (const m of st.map.modules) {
    for (const c of m.cells) {
      const here = c[1] >= st.floor - 0.5 && c[1] < st.floor + 2.5;
      ctx.fillStyle = here ? KIND_FILL[m.kind] || "#4a4a4a" : "rgba(120,120,120,.35)";
      const [px, py] = mini.toScreen(c[0] * CELL - CELL / 2, c[2] * CELL - CELL / 2);
      ctx.fillRect(px, py, Math.max(1, size - 0.5), Math.max(1, size - 0.5));
    }
  }
  // Utsnittet kartet viser.
  const v = st.view;
  const [a, b] = mini.toScreen(...v.toWorld(0, 0));
  const [c, d] = mini.toScreen(...v.toWorld(W, H));
  ctx.strokeStyle = "#ffd36a";
  ctx.strokeRect(Math.max(rx, a), Math.max(ry, b), Math.min(rx + rw, c) - Math.max(rx, a), Math.min(ry + rh, d) - Math.max(ry, b));
  if (st.camera && st.camera.length) {
    const [px, py] = mini.toScreen(st.camera[0], st.camera[2]);
    ctx.fillStyle = "#8fd0ff";
    ctx.beginPath();
    ctx.arc(px, py, 3, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.fillStyle = "#a7a49c";
  ctx.font = "11px Inter, sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "bottom";
  ctx.fillText("Oversikt (klikk for å gå dit)", rx, ry - 6);
}

/// Teksten kortet av med «…» så den får plass i `room` piksler (tom om ikke
/// engang tre tegn får plass).
function fitText(ctx, text, room) {
  if (ctx.measureText(text).width <= room) return text;
  for (let n = text.length - 1; n >= 3; n--) {
    const t = text.slice(0, n).trimEnd() + "…";
    if (ctx.measureText(t).width <= room) return t;
  }
  return "";
}

function sideLine(ctx, v, i, j, w) {
  const h = CELL / 2;
  const cx = i * CELL;
  const cz = j * CELL;
  const pts = [
    [cx - h, cz - h, cx + h, cz - h],
    [cx + h, cz - h, cx + h, cz + h],
    [cx - h, cz + h, cx + h, cz + h],
    [cx - h, cz - h, cx - h, cz + h],
  ][w];
  ctx.beginPath();
  ctx.moveTo(...v.toScreen(pts[0], pts[1]));
  ctx.lineTo(...v.toScreen(pts[2], pts[3]));
  ctx.stroke();
}

function arrow(ctx, v, i, j, side, color, k) {
  const [di, dj] = SIDE[side];
  const mx = i * CELL + (di * CELL) / 2;
  const mz = j * CELL + (dj * CELL) / 2;
  const [px, py] = v.toScreen(mx, mz);
  const size = Math.max(5, Math.min(12, v.s * 0.55)) * k;
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(px + di * size, py + dj * size);
  ctx.lineTo(px - dj * size * 0.8, py + di * size * 0.8);
  ctx.lineTo(px + dj * size * 0.8, py - di * size * 0.8);
  ctx.closePath();
  ctx.fill();
}

function linkArrow(ctx, a, b) {
  ctx.beginPath();
  ctx.moveTo(a[0], a[1]);
  ctx.lineTo(b[0], b[1]);
  ctx.stroke();
  const ang = Math.atan2(b[1] - a[1], b[0] - a[0]);
  ctx.beginPath();
  ctx.moveTo(b[0], b[1]);
  ctx.lineTo(b[0] - Math.cos(ang - 0.4) * 9, b[1] - Math.sin(ang - 0.4) * 9);
  ctx.moveTo(b[0], b[1]);
  ctx.lineTo(b[0] - Math.cos(ang + 0.4) * 9, b[1] - Math.sin(ang + 0.4) * 9);
  ctx.stroke();
}

/// Det som ligger under punktet (x, z) i etasjen: innhold (nærmest innen
/// `reach` m), en åpning (i en modul) eller en kant, eller en modul.
export function hit(map, floor, x, z, reach) {
  const onFloor = (y) => y >= floor - 0.5 && y < floor + 2.5;
  let best = null;
  for (const t of map.things) {
    if (!onFloor(t.pos[1]) || t.kind === "prop") continue;
    const d = Math.hypot(t.pos[0] - x, t.pos[2] - z);
    if (d < reach && (!best || d < best.d)) best = { type: "thing", id: t.id, own: t.own, d };
  }
  if (best) return best;
  // Lysene eieren har satt ut.
  for (const l of map.lights) {
    if (!l.own || !onFloor(l.pos[1] - 1.5)) continue;
    const d = Math.hypot(l.pos[0] - x, l.pos[2] - z);
    if (d < reach && (!best || d < best.d)) best = { type: "light", id: l.id, own: true, d };
  }
  if (best) return best;
  // Åpningene: nær midten av en ytre side.
  const i = Math.round(x / CELL);
  const j = Math.round(z / CELL);
  const lx = x - i * CELL;
  const lz = z - j * CELL;
  const near = Math.max(Math.abs(lx), Math.abs(lz)) > CELL / 2 - Math.max(0.7, reach * 0.8) && Math.min(Math.abs(lx), Math.abs(lz)) < 1.4;
  if (near) {
    const side = Math.abs(lx) > Math.abs(lz) ? (lx > 0 ? 1 : 3) : lz > 0 ? 2 : 0;
    for (const m of map.modules) {
      const o = m.openings.find((o) => o.at[0] === i && o.at[1] === j && o.side === side && onFloor(o.y));
      if (o) return { type: "opening", module: m.id, opening: o.id, side, at: [i, j] };
    }
    const e = map.edges.find((e) => onFloor(e.at[1]) && ((e.at[0] === i && e.at[2] === j && e.side === side) || (e.at[0] + SIDE[e.side][0] === i && e.at[2] + SIDE[e.side][1] === j && ((e.side + 2) & 3) === side)));
    if (e && e.switch) return { type: "edge", switch: e.switch, kind: e.kind };
  }
  for (const m of map.modules) {
    if (m.cells.some((c) => c[0] === i && c[2] === j && onFloor(c[1]))) return { type: "module", id: m.id };
  }
  return null;
}
