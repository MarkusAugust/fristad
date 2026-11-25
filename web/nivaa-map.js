// Kartet i nivåredigeringen: musa på kartet (sette ut, velge, flytte, ramme,
// koble og patrulje), menyene og forklaringen under pekeren.

import { placeIn3d } from "./nivaa-3d.js";
import { clipboard, copy, docLight, docThing, paste, placeModule, select, send, turnSelection } from "./nivaa-doc.js";
import { arm, disarm, ghostFor, mode, setMode, startLink, templateGhost } from "./nivaa-palette.js";
import { showSelection } from "./nivaa-panel.js";
import { CELL, hit, miniMap, snapPlacement } from "./nivaa-plan.js";
import {
  $,
  altDown,
  canvas,
  cellAt,
  drag,
  engine,
  esc,
  palette,
  plan,
  redraw,
  setAltDown,
  setDrag,
  setStatus,
  snap,
  st,
  thingY,
  toast,
  worldAt,
} from "./nivaa-shared.js";
import { centerOn, focusSelection, frameFloor, setView, viewMode } from "./nivaa-view.js";

let lastMouse = [0, 0];
/// Fantomet når modulene flyttes: én modul får magneten (uten å dreies).
function moveGhost(x, z) {
  const moved = st.map.modules.filter((m) => st.sel.includes(m.id));
  const c = cellAt(x, z);
  let by = [c[0] - drag.from[0], 0, c[2] - drag.from[2]];
  let links = [];
  if (moved.length === 1 && !altDown) {
    const m = moved[0];
    const pm = palette.modules.find((p) => p.id === m.module);
    const taken = new Set(st.map.modules.filter((o) => o.id !== m.id).flatMap((o) => o.cells.map((q) => q.join())));
    const want = [m.at[0] + by[0], m.at[1], m.at[2] + by[2]];
    const snapped = pm && snapPlacement(st.map, pm, want, [m.rot], m.mirror, taken, [m.id]);
    if (snapped) {
      by = [snapped.at[0] - m.at[0], snapped.at[1] - m.at[1], snapped.at[2] - m.at[2]];
      links = snapped.links;
    }
  }
  drag.by = by;
  if (!by[0] && !by[1] && !by[2]) return null;
  const taken = new Set(st.map.modules.filter((o) => !st.sel.includes(o.id)).flatMap((o) => o.cells.map((q) => q.join())));
  const cells = moved.flatMap((m) => m.cells.map((q) => [q[0] + by[0], q[1] + by[1], q[2] + by[2]]));
  return { cells, openings: [], links, ok: !cells.some((q) => taken.has(q.join())) };
}

export function bindMap() {
  let space = false;
  addEventListener("keydown", (e) => {
    if (e.code === "Space" && document.activeElement === plan) space = true;
  });
  addEventListener("keyup", (e) => {
    if (e.code === "Space") space = false;
  });
  plan.addEventListener("contextmenu", (e) => e.preventDefault());
  plan.addEventListener("wheel", (e) => {
    e.preventDefault();
    const r = plan.getBoundingClientRect();
    const [px, py] = [e.clientX - r.left, e.clientY - r.top];
    const before = st.view.toWorld(px, py);
    st.view.s = Math.min(80, Math.max(2, st.view.s * Math.exp(-e.deltaY * 0.0015)));
    const after = st.view.toWorld(px, py);
    st.view.cx += before[0] - after[0];
    st.view.cz += before[1] - after[1];
    redraw();
  });
  plan.addEventListener("pointerdown", (e) => {
    $("opmenu").hidden = true;
    plan.focus();
    plan.setPointerCapture(e.pointerId);
    const [x, z] = worldAt(e);
    lastMouse = [e.clientX, e.clientY];
    if (e.button === 2 || e.button === 1 || space) {
      setDrag({ type: "pan", button: e.button, at: [e.clientX, e.clientY], moved: false });
      return;
    }
    if (e.button !== 0) return;
    const mini = miniMap(st.map, plan.clientWidth, plan.clientHeight, st.view);
    const pr = plan.getBoundingClientRect();
    if (mini?.inside(e.clientX - pr.left, e.clientY - pr.top)) {
      [st.view.cx, st.view.cz] = mini.toWorld(e.clientX - pr.left, e.clientY - pr.top);
      setDrag({ type: "mini" });
      redraw();
      return;
    }
    if (mode === "patrol") return addPatrol(x, z);
    if (mode === "link") return finishLink(x, z);
    if (arm) return;
    const h = hit(st.map, st.floor, x, z, Math.max(0.6, 10 / st.view.s));
    if (h?.type === "thing") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      setDrag({ type: "thing", id: h.id, own: h.own, from: [x, z], moved: false });
      return;
    }
    if (h?.type === "light") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      setDrag({ type: "light", id: h.id, from: [x, z], moved: false });
      return;
    }
    if (h?.type === "opening") {
      st.sel = [];
      st.selOpening = { module: h.module, opening: h.opening };
      engine.level(JSON.stringify({ cmd: "select", ids: [h.module] }));
      showSelection();
      openingMenu(h, e);
      redraw();
      return;
    }
    if (h?.type === "module") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      setDrag({ type: "modules", from: cellAt(x, z), by: [0, 0, 0] });
      return;
    }
    if (!e.shiftKey) select([]);
    const r = plan.getBoundingClientRect();
    setDrag({ type: "box", a: [e.clientX - r.left, e.clientY - r.top], add: e.shiftKey });
  });
  plan.addEventListener("pointermove", (e) => {
    setAltDown(e.altKey);
    const [x, z] = worldAt(e);
    const r = plan.getBoundingClientRect();
    st.mouse = [e.clientX - r.left, e.clientY - r.top];
    if (drag?.type === "pan") {
      drag.moved = drag.moved || Math.hypot(e.clientX - drag.at[0], e.clientY - drag.at[1]) > 4;
      st.view.cx -= (e.clientX - lastMouse[0]) / st.view.s;
      st.view.cz -= (e.clientY - lastMouse[1]) / st.view.s;
      lastMouse = [e.clientX, e.clientY];
      return redraw();
    }
    lastMouse = [e.clientX, e.clientY];
    if (arm?.type === "module") st.ghost = ghostFor(x, z);
    if (arm?.type === "template") st.ghost = templateGhost(x, z);
    if (arm?.type === "thing" || arm?.type === "light") st.thingGhost = [snap(x), st.floor, snap(z), arm.type === "light" ? "light" : arm.kind];
    // Fantomet: de valgte modulene flyttet.
    if (drag?.type === "modules") st.ghost = moveGhost(x, z);
    if (drag?.type === "thing" || drag?.type === "light") {
      drag.moved = drag.moved || Math.hypot(x - drag.from[0], z - drag.from[1]) > 0.3;
      if (drag.moved) st.thingGhost = [snap(x), st.floor, snap(z)];
    }
    if (drag?.type === "box") st.box = [...drag.a, ...st.mouse];
    showTip(drag || arm || mode ? null : hit(st.map, st.floor, x, z, Math.max(0.6, 10 / st.view.s)));
    redraw();
  });
  plan.addEventListener("pointerup", (e) => {
    const [x, z] = worldAt(e);
    const d = drag;
    setDrag(null);
    // Høyre klikk uten å dra: menyen for det som står der.
    if (d?.type === "pan" && d.button === 2 && !d.moved) return contextMenu(e, x, z);
    if (d?.type === "pan" || d?.type === "mini") return;
    if (arm?.type === "module" && (e.button === 0 || d?.type === "palette")) {
      const g = ghostFor(x, z);
      if (g) placeModule(arm.id, g.at, g.rot, arm.mirror);
      if (!e.shiftKey) disarm();
      return;
    }
    if (arm?.type === "template" && e.button === 0) {
      const g = templateGhost(x, z);
      if (g) {
        const out = send({ op: "stamp", template: arm.tpl, at: g.at, rot: arm.rot, mirror: arm.mirror });
        if (out?.made?.length) select(out.made);
      }
      if (!e.shiftKey) disarm();
      return;
    }
    if (arm?.type === "light" && e.button === 0) {
      const out = send({ op: "add_light", at: [snap(x), st.floor + 2.4, snap(z)] });
      if (out?.made?.length) select(out.made);
      if (!e.shiftKey) disarm();
      return;
    }
    if (arm?.type === "thing" && e.button === 0) {
      const out = send({ op: "add_thing", kind: arm.kind, at: [snap(x), st.floor, snap(z)], yaw: arm.yaw, label: arm.label });
      if (out?.made?.length) select(out.made);
      if (!e.shiftKey) disarm();
      return;
    }
    if (d?.type === "modules") {
      st.ghost = null;
      if (d.by[0] || d.by[1] || d.by[2]) send({ op: "move", ids: st.sel, by: d.by });
      redraw();
      return;
    }
    if (d?.type === "thing") {
      st.thingGhost = null;
      if (d.moved) {
        // Innhold fra en modulmal løsnes av kjernen i samme steg (modulen
        // husker det i `hidden`, og dokumentet får det på det nye stedet).
        send({ op: "move_thing", id: d.id, to: [snap(x), thingY(d.id), snap(z)] });
      }
      redraw();
      return;
    }
    if (d?.type === "light") {
      st.thingGhost = null;
      const l = docLight(d.id);
      if (d.moved && l) send({ op: "set_light", light: { ...l, pos: [snap(x), l.pos[1], snap(z)] } });
      redraw();
      return;
    }
    if (d?.type === "box") {
      st.box = null;
      const [ax, az] = st.view.toWorld(...d.a);
      const [lo, hi] = [
        [Math.min(ax, x), Math.min(az, z)],
        [Math.max(ax, x), Math.max(az, z)],
      ];
      const inBox = (px, pz) => px >= lo[0] && px <= hi[0] && pz >= lo[1] && pz <= hi[1];
      const ids = st.map.modules.filter((m) => m.cells.some((c) => c[1] >= st.floor - 0.5 && c[1] < st.floor + 2.5 && inBox(c[0] * CELL, c[2] * CELL))).map((m) => m.id);
      ids.push(...st.map.things.filter((t) => t.own && inBox(t.pos[0], t.pos[2])).map((t) => t.id));
      select(ids, d.add);
    }
  });
  // Dra fra paletten og slipp på kartet, eller i 3D (på gulvet der).
  addEventListener("pointerup", (e) => {
    if (drag?.type !== "palette") return;
    setDrag(null);
    const r = plan.getBoundingClientRect();
    if (viewMode !== "3d" && e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom && arm?.type === "module") {
      const [x, z] = st.view.toWorld(e.clientX - r.left, e.clientY - r.top);
      const g = ghostFor(x, z);
      if (g) placeModule(arm.id, g.at, g.rot, arm.mirror);
      disarm();
      return;
    }
    const w = canvas.getBoundingClientRect();
    if (viewMode !== "map" && e.clientX >= w.left && e.clientX <= w.right && e.clientY >= w.top && e.clientY <= w.bottom) placeIn3d(e);
  });
  addEventListener("pointermove", (e) => {
    if (drag?.type !== "palette" || !arm) return;
    setAltDown(e.altKey);
    const r = plan.getBoundingClientRect();
    if (e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom) {
      const [x, z] = st.view.toWorld(e.clientX - r.left, e.clientY - r.top);
      st.ghost = ghostFor(x, z);
      redraw();
      return;
    }
    // Over 3D: fantomet på kartet der det havner.
    const w = canvas.getBoundingClientRect();
    if (arm.type === "module" && e.clientX >= w.left && e.clientX <= w.right && e.clientY >= w.top && e.clientY <= w.bottom) {
      const p = engine.level_pick((e.clientX - w.left) / w.width, (e.clientY - w.top) / w.height);
      st.ghost = p.length ? ghostFor(p[0], p[2]) : null;
      redraw();
    }
  });
  plan.addEventListener("pointerleave", () => {
    showTip(null);
    st.ghost = arm?.type === "module" ? null : st.ghost;
    st.thingGhost = null;
    redraw();
  });
  // Dobbeltklikk: gå inn her i 3D.
  plan.addEventListener("dblclick", (e) => {
    const [x, z] = worldAt(e);
    const yaw = st.camera?.length ? st.camera[3] : 0;
    engine.level(JSON.stringify({ cmd: "place", x, y: st.floor + 1.7, z, yaw, pitch: -5 }));
    if (viewMode === "map") setView("split");
  });
}

/// Valget for en åpning rett ved pekeren: ett klikk gjør den til dør, port,
/// falsk vegg … (det samme som i panelet).
function openingMenu(h, e) {
  const m = st.map.modules.find((m) => m.id === h.module);
  const o = m?.openings.find((o) => o.id === h.opening);
  if (!o) return;
  const box = $("opmenu");
  box.classList.remove("list");
  const now = o.chosen || "auto";
  box.innerHTML = palette.openings.map(([k, n]) => `<button type="button" class="chip${k === now ? " armed" : ""}" data-kind="${esc(k)}">${esc(n)}</button>`).join("");
  box.querySelectorAll("button").forEach((b) =>
    b.addEventListener("click", (ev) => {
      ev.stopPropagation();
      send({ op: "opening", module: m.id, opening: o.id, kind: b.dataset.kind });
      box.hidden = true;
    }),
  );
  const r = plan.getBoundingClientRect();
  box.hidden = false;
  box.style.left = `${Math.max(4, Math.min(e.clientX - r.left - box.offsetWidth / 2, plan.clientWidth - box.offsetWidth - 4))}px`;
  box.style.top = `${Math.min(e.clientY - r.top + 14, plan.clientHeight - box.offsetHeight - 4)}px`;
  showTip(null);
}

/// Høyreklikkmenyen på kartet: det du kan gjøre med det som står der.
function contextMenu(e, x, z) {
  disarm();
  const h = hit(st.map, st.floor, x, z, Math.max(0.6, 10 / st.view.s));
  if (h?.type === "opening") return openingMenu(h, e);
  const items = [];
  if (h?.type === "module" || h?.type === "thing" || h?.type === "light") {
    if (!st.sel.includes(h.id)) select([h.id]);
    const t = h.type === "thing" ? st.map.things.find((t) => t.id === h.id) : null;
    items.push(["Drei (R)", () => turnSelection(false)]);
    if (h.type === "module") items.push(["Speil (M)", () => send({ op: "mirror", ids: st.sel })], ["Kopier ved siden av (Ctrl+D)", () => (copy(), paste())]);
    if (t && ["lever", "elite", "hint"].includes(t.kind)) items.push(["Koble til … (K)", startLink]);
    if (t && ["mob", "elite", "boss"].includes(t.kind) && t.own) items.push(["Patrulje: klikk punktene", () => (setMode("patrol"), setStatus("Klikk punktene på kartet · Esc er ferdig"))]);
    items.push(["Vis i 3D (F)", focusSelection], ["Slett (Delete)", () => send({ op: "delete", ids: st.sel })]);
  } else {
    items.push(
      ["Vis hele etasjen (Home)", () => (centerOn(), frameFloor())],
      ["Gå inn her i 3D", () => engine.level(JSON.stringify({ cmd: "place", x, y: st.floor + 1.7, z, yaw: st.camera?.length ? st.camera[3] : 0, pitch: -5 }))],
    );
    if (clipboard) items.unshift(["Lim inn (Ctrl+V)", paste]);
  }
  const box = $("opmenu");
  box.classList.add("list");
  box.innerHTML = items.map(([n], k) => `<button type="button" class="chip" data-k="${k}">${esc(n)}</button>`).join("");
  box.querySelectorAll("button").forEach((b) =>
    b.addEventListener("click", (ev) => {
      ev.stopPropagation();
      box.hidden = true;
      items[Number(b.dataset.k)][1]();
    }),
  );
  const r = plan.getBoundingClientRect();
  box.hidden = false;
  box.style.left = `${Math.max(4, Math.min(e.clientX - r.left, plan.clientWidth - box.offsetWidth - 4))}px`;
  box.style.top = `${Math.min(e.clientY - r.top + 6, plan.clientHeight - box.offsetHeight - 4)}px`;
  showTip(null);
}

/// Lappen ved pekeren på kartet: hva det er og hva du kan gjøre med det.
function showTip(h) {
  const tip = $("maptip");
  if (!h || !st.mouse) return (tip.hidden = true);
  let html = "";
  if (h.type === "thing") {
    const t = st.map.things.find((t) => t.id === h.id);
    if (!t) return (tip.hidden = true);
    const kind = palette.things.find(([k]) => k === t.kind)?.[1] || t.kind;
    const mob = ["mob", "elite", "boss"].includes(t.kind) ? palette.mobs.find(([k]) => k === t.label)?.[1] : null;
    const extra = mob
      ? `${mob}${t.n > 1 ? ` × ${t.n}` : ""}${t.ambush ? " · i bakhold" : t.patrol.length ? " · patruljerer" : ""}`
      : t.kind === "lore"
        ? t.label || "uten tittel"
        : t.kind === "chest"
          ? t.loot || "tilfeldig fra nivået"
          : "";
    const from = st.map.modules.find((m) => m.id === t.module)?.name || t.module || "";
    const how = t.own ? "Dra for å flytte · R dreier · Delete sletter" : `Fra modulen ${esc(from)} · dra for å flytte her`;
    html = `<b>${esc(kind)}</b>${extra ? ` · ${esc(extra)}` : ""}<div class="muted">${how}</div>`;
  } else if (h.type === "light") {
    html = `<b>Lys</b><div class="muted">Dra for å flytte · farge og styrke i panelet</div>`;
  } else if (h.type === "opening") {
    const m = st.map.modules.find((m) => m.id === h.module);
    const o = m?.openings.find((o) => o.id === h.opening);
    const chosen = o?.chosen ? palette.openings.find(([k]) => k === o.chosen)?.[1] || o.chosen : "automatisk";
    html = `<b>Åpning</b> · ${esc(chosen)}<div class="muted">Klikk for å gjøre den til dør, port, falsk vegg, mur …</div>`;
  } else if (h.type === "edge") {
    html = `<b>${esc({ gate: "Port", secret: "Falsk vegg", door: "Dør" }[h.kind] || "Åpning")}</b> <code>${esc(h.switch)}</code><div class="muted">En spak eller nøkkel kan kobles hit (K)</div>`;
  } else if (h.type === "module") {
    const m = st.map.modules.find((m) => m.id === h.id);
    const pm = m && palette.modules.find((p) => p.id === m.module);
    if (!m) return (tip.hidden = true);
    html = `<b>${esc(m.name)}</b>${pm && pm.name !== m.name ? ` · ${esc(pm.name)}` : ""}<div class="muted">Dra for å flytte · R dreier · M speiler · F viser i 3D</div>`;
  }
  if (!html) return (tip.hidden = true);
  tip.innerHTML = html;
  tip.hidden = false;
  const [mx, my] = st.mouse;
  const W = plan.clientWidth;
  tip.style.left = `${Math.min(mx + 16, W - tip.offsetWidth - 6)}px`;
  tip.style.top = `${my + 18}px`;
}

function addPatrol(x, z) {
  const t = docThing(st.sel[0]);
  if (!t) return disarm();
  t.patrol = [...(t.patrol || []), [snap(x), t.pos?.[1] ?? st.floor, snap(z)]];
  send({ op: "set_thing", thing: t });
}

function finishLink(x, z) {
  const from = st.linkFrom;
  const h = hit(st.map, st.floor, x, z, Math.max(0.8, 12 / st.view.s));
  let to = null;
  if (h?.type === "thing") to = h.id;
  if (h?.type === "edge") to = h.switch;
  const fromT = st.map.things.find((t) => t.id === from);
  if (h?.type === "opening") {
    const m = st.map.modules.find((m) => m.id === h.module);
    let ov = m?.overrides.find((o) => o.opening === h.opening);
    // En vanlig åpning blir det koblingen trenger: en port (spaken, eliten
    // med nøkkelen) eller en falsk vegg (hintet).
    if (!ov?.id && m) {
      const kind = fromT?.kind === "hint" ? "falsk" : "port";
      const out = send({ op: "opening", module: m.id, opening: h.opening, kind });
      ov = out?.modules.find((x) => x.id === m.id)?.overrides.find((o) => o.opening === h.opening);
      if (ov?.id) toast(`Åpningen er nå ${kind === "port" ? "en port" : "en falsk vegg"}`, 2000);
    }
    to = ov?.id || null;
  }
  disarm();
  if (!to) return toast("Det er ikke noe å koble til der: klikk på en åpning, en port eller en falsk vegg.");
  const kind = fromT?.kind === "elite" ? "key" : fromT?.kind === "hint" ? "for" : "opens";
  send({ op: "link", from, to, kind });
}
