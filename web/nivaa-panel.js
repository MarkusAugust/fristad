// Egenskapspanelet i nivåredigeringen: nivået, modulen, tingen eller lyset
// som er valgt, bruddene på reglene med «Rett», og tallene.

import { copy, docHead, docLight, docModule, docThing, paste, select, send, turnSelection } from "./nivaa-doc.js";
import { armModule, armThing, saveModule, saveTemplate, setMode, startLink } from "./nivaa-palette.js";
import { CELL } from "./nivaa-plan.js";
import { $, esc, field, palette, setStatus, st, THEME_NAMES, toast } from "./nivaa-shared.js";
import { goTo } from "./nivaa-view.js";

function options(list, value) {
  return list.map(([v, n]) => `<option value="${esc(v)}"${String(v) === String(value ?? "") ? " selected" : ""}>${esc(n)}</option>`).join("");
}

function stoneOptions(value) {
  const v = typeof value === "string" ? value : "";
  return options([["", "Som rommet rundt (standard)"], ...palette.stones], v);
}

export function showSelection() {
  const box = $("sel");
  const map = st.map;
  if (st.selOpening) {
    const m = map.modules.find((m) => m.id === st.selOpening.module);
    const o = m?.openings.find((o) => o.id === st.selOpening.opening);
    if (!o) return (box.innerHTML = "");
    const ov = m.overrides.find((v) => v.opening === o.id);
    box.innerHTML = `<h2>Åpningen</h2><p class="muted">${esc(m.name)} · ${esc(o.id)} · modulen vil ha «${esc(o.kind)}»</p>
      ${field("Slag", `<select id="op-kind" class="lt-select">${options(palette.openings, ov?.kind || "auto")}</select>`)}
      ${ov?.id ? `<p class="muted">Id: <code>${esc(ov.id)}</code> (en spak kan kobles hit med K)</p>` : ""}
      <p class="muted">Automatisk: dør mellom rom, gangåpning mot en gang, mur der ingenting møter.</p>`;
    box.querySelector("#op-kind").onchange = (e) => send({ op: "opening", module: m.id, opening: o.id, kind: e.target.value });
    return;
  }
  if (st.sel.length > 1) {
    box.innerHTML = `<h2>${st.sel.length} valgt</h2><div class="row">
      <button class="lt-button" data-variant="secondary" id="s-turn">Drei (R)</button>
      <button class="lt-button" data-variant="secondary" id="s-mirror">Speil (M)</button>
      <button class="lt-button" data-variant="secondary" id="s-copy">Kopier ved siden av</button>
      <button class="lt-button" data-variant="secondary" id="s-tpl" title="Ctrl+G">Lagre som mal</button>
      <button class="lt-button" data-variant="secondary" id="s-mod" title="Ctrl+Shift+G">Lagre som modul</button>
      <button class="lt-button" data-variant="danger" id="s-del">Slett</button></div>`;
    box.querySelector("#s-tpl").onclick = saveTemplate;
    box.querySelector("#s-mod").onclick = saveModule;
    box.querySelector("#s-turn").onclick = () => turnSelection(false);
    box.querySelector("#s-mirror").onclick = () => send({ op: "mirror", ids: st.sel });
    box.querySelector("#s-copy").onclick = () => (copy(), paste());
    box.querySelector("#s-del").onclick = () => send({ op: "delete", ids: st.sel });
    return;
  }
  const id = st.sel[0];
  const mm = id && map.modules.find((m) => m.id === id);
  if (mm) return showModule(box, mm);
  const tt = id && map.things.find((t) => t.id === id);
  if (tt) return showThing(box, tt);
  const ll = id && map.lights.find((l) => l.id === id && l.own);
  if (ll) return showLight(box, ll);
  showDoc(box);
}

function showDoc(box) {
  const h = docHead();
  box.innerHTML = `<h2>Nivået</h2>
    ${field("Navnet", `<input id="d-name" class="lt-input" value="${esc(h.name)}">`)}
    <div class="row">
      ${field(
        "Nummer",
        // Et tall fra 1 (uten øvre grense), eller et navn som «test».
        `<input id="d-level" class="lt-input" value="${esc(String(h.level ?? ""))}" style="width:5em">`,
      )}
      ${field("Tema", `<select id="d-theme" class="lt-select">${options([["", "Som modulene"], ...palette.themes.map((t) => [t, THEME_NAMES[t] || t])], h.theme)}</select>`)}
    </div>
    ${field("Steinen i hele nivået", `<select id="d-stone" class="lt-select">${stoneOptions(h.stone)}</select>`)}
    <p class="muted">Velg en modul, en åpning eller en ting på kartet for å endre den. Et nivå merket «klar» erstatter det gamle nivået med samme nummer.</p>
    <h2>Innholdet</h2>${contentList()}`;
  box.querySelectorAll("[data-arm]").forEach((b) => b.addEventListener("click", () => (armThing(b.dataset.arm, null), toast("Klikk på kartet der den skal stå"))));
  const put = (patch) => send({ op: "set_head", head: { ...docHead(), ...patch } });
  box.querySelector("#d-name").onchange = (e) => put({ name: e.target.value });
  box.querySelector("#d-level").onchange = (e) => put({ level: /^\d+$/.test(e.target.value) ? Number(e.target.value) : e.target.value });
  box.querySelector("#d-theme").onchange = (e) => put({ theme: e.target.value });
  box.querySelector("#d-stone").onchange = (e) => put({ stone: e.target.value || undefined });
}

/// Det nivået har: det som må være med (med «Sett ut» om det mangler) og
/// hvor mange fiender, kister og lapper.
function contentList() {
  const count = (k) => st.map.things.filter((t) => t.kind === k).reduce((a, t) => a + (["mob", "elite"].includes(k) ? t.n || 1 : 1), 0);
  const name = (k) => palette.things.find(([x]) => x === k)?.[1] || k;
  const must = ["start", "up", "rest", "down"].map((k) => {
    const n = count(k);
    return `<li>${n === 1 ? "✓" : "✗"} ${esc(name(k))}${n > 1 ? ` (${n}, skal være én)` : ""}${n === 0 ? ` <button class="chip fix" data-arm="${k}">Sett ut</button>` : ""}</li>`;
  });
  const more = [["mob", "fiender"], ["elite", "eliter"], ["boss", "boss"], ["chest", "kister"], ["rare", "sjeldne kister"], ["lore", "lapper"], ["lever", "spaker"]]
    .map(([k, n]) => [count(k), n])
    .filter(([c]) => c)
    .map(([c, n]) => `${c} ${n}`);
  return `<ul class="checklist">${must.join("")}</ul><p class="muted">${more.length ? more.join(" · ") : "Ingen fiender eller kister ennå."}</p>`;
}

function showModule(box, m) {
  const pm = palette.modules.find((p) => p.id === m.module);
  box.innerHTML = `<h2>Modulen</h2><p class="muted">${esc(pm?.name || m.module)} · dreid ${m.rot * 90}°${m.mirror ? " · speilet" : ""}</p>
    ${field("Navnet på rommet", `<input id="m-name" class="lt-input" value="${esc(m.name)}">`)}
    ${field(
      "Hører til rommet (samme rom som en annen modul)",
      `<select id="m-room" class="lt-select">${options([["", "Sitt eget"], ...st.map.modules.filter((x) => x.id !== m.id).map((x) => [x.room || x.id, x.name])], m.room || "")}</select>`,
    )}
    ${field("Steinen", `<select id="m-stone" class="lt-select">${stoneOptions(m.stone)}</select>`)}
    ${moodFields(m.mood)}
    <div class="row"><button class="lt-button" data-variant="secondary" id="m-turn">Drei (R)</button>
    <button class="lt-button" data-variant="secondary" id="m-mirror">Speil (M)</button>
    <button class="lt-button" data-variant="danger" id="m-del">Slett</button></div>
    <h2>Åpningene</h2><p class="muted">Klikk en pil på kartet for å velge dør, port, falsk vegg, gang, bue, snarvei eller mur.</p>
    <ul class="muted">${m.openings.map((o) => `<li>${esc(o.id)}: ${esc(o.chosen ? palette.openings.find(([k]) => k === o.chosen)?.[1] || o.chosen : "automatisk")}</li>`).join("")}</ul>
    ${
      m.removed?.length
        ? `<h2>Fjernet fra malen</h2><ul class="muted">${m.removed.map((r) => `<li>${esc(r)} <button class="chip" data-restore="${esc(r)}">sett tilbake</button></li>`).join("")}</ul>`
        : ""
    }`;
  box.querySelectorAll("[data-restore]").forEach((b) => b.addEventListener("click", () => send({ op: "reattach", module: m.id, thing: b.dataset.restore })));
  const put = (patch) => {
    const d = docModule(m.id);
    if (d) send({ op: "set_module", module: { ...d, ...patch } });
  };
  box.querySelector("#m-name").onchange = (e) => put({ name: e.target.value || undefined });
  box.querySelector("#m-room").onchange = (e) => put({ room: e.target.value || undefined });
  bindMood(box, (patch) => put({ mood: { ...m.mood, ...patch } }));
  box.querySelector("#mood-reset")?.addEventListener("click", () => put({ mood: undefined }));
  if (!m.own_mood) box.querySelector("#mood-reset")?.remove();
  box.querySelector("#m-stone").onchange = (e) => put({ stone: e.target.value || undefined });
  box.querySelector("#m-turn").onclick = () => send({ op: "turn", ids: [m.id] });
  box.querySelector("#m-mirror").onclick = () => send({ op: "mirror", ids: [m.id] });
  box.querySelector("#m-del").onclick = () => send({ op: "delete", ids: [m.id] });
}

// Fargene eieren velger mellom (lineære, som lysene i motoren).
const LIGHT_COLORS = [
  ["", "Ild (standard)", null],
  ["kald", "Kaldt måneskinn", [0.55, 0.7, 1.0]],
  ["gift", "Giftgrønt", [0.45, 1.0, 0.4]],
  ["magi", "Fiolett magi", [0.75, 0.4, 1.0]],
  ["blek", "Blekt hvitt", [0.95, 0.92, 0.85]],
  ["blod", "Blodrødt", [1.0, 0.25, 0.18]],
];

function colorHex(c) {
  if (!c) return "#ff9e4d";
  const h = (v) => Math.round(Math.max(0, Math.min(1, v)) * 255).toString(16).padStart(2, "0");
  return `#${h(c[0])}${h(c[1])}${h(c[2])}`;
}

function hexColor(s) {
  return [1, 3, 5].map((k) => Math.round((parseInt(s.slice(k, k + 2), 16) / 255) * 1000) / 1000);
}

/// Stemningen i et rom (`Mood`): faklene, lysstyrken, fargen, tåka og lyden.
function moodFields(mood) {
  const m = mood || {};
  const preset = LIGHT_COLORS.find(([, , c]) => (c && m.color ? c.every((v, k) => Math.abs(v - m.color[k]) < 0.01) : !c && !m.color))?.[0] ?? "egen";
  return `<h2>Stemningen</h2>
    ${field(`Fakler (${Math.round((m.torches ?? 0) * 100)} % av festene)`, `<input id="mood-torch" type="range" min="0" max="1" step="0.05" value="${m.torches ?? 0}">`)}
    ${field(`Lysstyrke (${Math.round((m.light ?? 1) * 100)} %)`, `<input id="mood-light" type="range" min="0.1" max="2.5" step="0.05" value="${m.light ?? 1}">`)}
    <div class="row">${field("Fargen på lysene", `<select id="mood-color" class="lt-select">${options([...LIGHT_COLORS.map(([k, n]) => [k, n]), ["egen", "Egen farge"]], preset)}</select>`)}
      <input id="mood-pick" type="color" value="${colorHex(m.color)}" title="Egen farge" aria-label="Egen farge"></div>
    ${field(`Tåke (${Math.round((m.fog ?? 0) * 100)} %)`, `<input id="mood-fog" type="range" min="0" max="1" step="0.05" value="${m.fog ?? 0}">`)}
    ${field("Lyden", `<select id="mood-sound" class="lt-select">${options(palette.sounds || [["", "Drypp"]], m.sound || "")}</select>`)}
    <button class="lt-button" data-variant="secondary" id="mood-reset" type="button">Som malen</button>`;
}

function bindMood(box, put) {
  const on = (id, ev, fn) => box.querySelector(id)?.addEventListener(ev, fn);
  on("#mood-torch", "change", (e) => put({ torches: Number(e.target.value) }));
  on("#mood-light", "change", (e) => put({ light: Number(e.target.value) }));
  on("#mood-fog", "change", (e) => put({ fog: Number(e.target.value) }));
  on("#mood-sound", "change", (e) => put({ sound: e.target.value }));
  on("#mood-color", "change", (e) => {
    const c = LIGHT_COLORS.find(([k]) => k === e.target.value);
    if (c) put({ color: c[2] || undefined });
    else put({ color: hexColor(box.querySelector("#mood-pick").value) });
  });
  on("#mood-pick", "change", (e) => put({ color: hexColor(e.target.value) }));
}

/// Et lys eieren har satt ut: farge, rekkevidde, styrke, høyde og flamme.
function showLight(box, ml) {
  const l = docLight(ml.id);
  if (!l) return (box.innerHTML = "");
  const put = (patch) => send({ op: "set_light", light: { ...docLight(ml.id), ...patch } });
  box.innerHTML = `<h2>Lyset</h2><p class="muted">${esc(l.id)} · (${l.pos.map((v) => v.toFixed(1)).join(", ")}) · dra det på kartet</p>
    ${field("Fargen", `<input id="l-color" type="color" value="${colorHex(l.color)}">`)}
    ${field(`Rekkevidde (${l.range.toFixed(1)} m)`, `<input id="l-range" type="range" min="2" max="20" step="0.5" value="${l.range}">`)}
    ${field(`Styrke (${(l.intensity ?? 2).toFixed(1)})`, `<input id="l-int" type="range" min="0.2" max="6" step="0.1" value="${l.intensity ?? 2}">`)}
    ${field(`Høyde over gulvet (${(l.pos[1] - st.floor).toFixed(1)} m)`, `<input id="l-h" type="range" min="0.3" max="8" step="0.1" value="${l.pos[1] - st.floor}">`)}
    <label class="row"><input id="l-flicker" type="checkbox"${l.flicker ? " checked" : ""}> Flamme (blafrer)</label>
    <div class="row" style="margin-top:8px"><button class="lt-button" data-variant="danger" id="l-del">Slett</button></div>`;
  const on = (id, ev, fn) => box.querySelector(id)?.addEventListener(ev, fn);
  on("#l-color", "change", (e) => put({ color: hexColor(e.target.value) }));
  on("#l-range", "change", (e) => put({ range: Number(e.target.value) }));
  on("#l-int", "change", (e) => put({ intensity: Number(e.target.value) }));
  on("#l-h", "change", (e) => put({ pos: [l.pos[0], st.floor + Number(e.target.value), l.pos[2]] }));
  on("#l-flicker", "change", (e) => put({ flicker: e.target.checked }));
  on("#l-del", "click", () => send({ op: "delete", ids: [l.id] }));
}

function showThing(box, t) {
  const name = palette.things.find(([k]) => k === t.kind)?.[1] || t.kind;
  if (!t.own) {
    const about = `Fra modulen ${esc(t.module || "")} (${esc(t.id)}). Det følger modulen. Dra det på kartet for å flytte det i dette nivået, eller endre det her; malen er den samme.`;
    box.innerHTML = `<h2>${esc(name)}</h2><p class="muted">${about}</p>
      <div class="row"><button class="lt-button" data-variant="secondary" id="t-detach">Endre her</button>
      <button class="lt-button" data-variant="danger" id="t-del">Fjern fra modulen</button></div>`;
    box.querySelector("#t-detach").onclick = () => send({ op: "detach", id: t.id }) && showSelection();
    box.querySelector("#t-del").onclick = () => send({ op: "delete", ids: [t.id] });
    return;
  }
  const d = docThing(t.id) || {};
  const put = (patch) => send({ op: "set_thing", thing: { ...docThing(t.id), ...patch } });
  let html = `<h2>${esc(name)}</h2><p class="muted">${esc(t.id)} · (${t.pos.map((v) => v.toFixed(1)).join(", ")}) · ${Math.round(d.yaw || 0)}°</p>`;
  const mob = ["mob", "elite", "boss"].includes(t.kind);
  if (mob) {
    html += field("Slaget", `<select id="t-label" class="lt-select">${options(palette.mobs, d.label || "button")}</select>`);
    html += `<div class="row">${field("Antall", `<input id="t-n" class="lt-input" type="number" min="1" max="8" value="${d.n || 1}">`)}
      ${field(
        "Bakhold",
        `<select id="t-amb" class="lt-select">${options(
          [["", "Nei (patruljerer eller står)"], ["4", "Innen 4 m"], ["8", "Innen 8 m"], ["rom", "Når du kommer inn i rommet"]],
          ambushValue(d.ambush),
        )}</select>`,
      )}</div>`;
    html += `<div class="row"><button class="lt-button" data-variant="secondary" id="t-patrol">Patrulje: klikk punkter</button><button class="lt-button" data-variant="secondary"
      id="t-clear">Tøm patruljen (${(d.patrol || []).length})</button></div>`;
  }
  if (t.kind === "chest") {
    const loot = options([["", "Fra nivåets liste (tilfeldig)"], ...palette.loot.map((c) => [c, c])], d.loot || "");
    html += field("Det kista gir", `<select id="t-loot" class="lt-select">${loot}</select>`);
  }
  if (t.kind === "rare") html += field("Den sjeldne komponenten", `<select id="t-label" class="lt-select">${options(palette.loot.map((c) => [c, c]), d.label || "")}</select>`);
  if (t.kind === "lore") {
    html += field("Tittelen", `<input id="t-title" class="lt-input" value="${esc(d.label || "")}">`);
    html += field("Teksten (ingen moderne ord: kanon gjelder)", `<textarea id="t-text" class="lt-textarea" rows="7">${esc(d.text || "")}</textarea>`);
  }
  if (t.kind === "bonus") html += field("Bonusen", `<select id="t-label" class="lt-select">${options([["liv", "Liv"], ["utholdenhet", "Utholdenhet"]], d.label || "liv")}</select>`);
  if (t.kind === "trap") {
    const traps = options([["arrows", "Piler"], ["spikes", "Pigger"], ["rocks", "Steiner"], ["pendulum", "Pendel"]], d.label || "arrows");
    html += field("Fella", `<select id="t-label" class="lt-select">${traps}</select>`);
  }
  if (t.kind === "hint") html += field("Hintet", `<select id="t-label" class="lt-select">${options([["crack", "Sprekk"], ["draft", "Trekk"], ["sound", "Lyd"]], d.label || "crack")}</select>`);
  const links = st.map.links.filter((l) => l.from === t.id || l.to === t.id);
  if (["lever", "elite", "hint"].includes(t.kind) || links.length) {
    const kind = (l) => palette.links.find(([x]) => x === l.kind)?.[1] || l.kind;
    html += `<h2>Koblingene</h2><ul class="muted">${
      links
        .map((l, k) => `<li>${esc(l.from)} → ${esc(l.to)} (${esc(kind(l))}) ${l.own ? `<button class="chip" data-unlink="${k}">fjern</button>` : ""}</li>`)
        .join("") || "<li>Ingen</li>"
    }</ul>
      <button class="lt-button" data-variant="secondary" id="t-link">Koble (K): klikk så på målet</button>`;
  }
  html += `<div class="row" style="margin-top:8px"><button class="lt-button" data-variant="secondary" id="t-turn">Drei (R)</button><button class="lt-button" data-variant="danger"
      id="t-del">Slett</button>
    ${t.template ? `<button class="lt-button" data-variant="secondary" id="t-back" title="Slik modulmalen har det">Tilbake til malen</button>` : ""}</div>`;
  if (t.template) html = `<p class="muted">Endret i dette nivået; malen i modulen ${esc(t.template)} er den samme.</p>` + html;
  box.innerHTML = html;
  const on = (id, ev, fn) => box.querySelector(id)?.addEventListener(ev, fn);
  on("#t-label", "change", (e) => put({ label: e.target.value }));
  on("#t-n", "change", (e) => put({ n: Math.max(1, Number(e.target.value) || 1) }));
  on("#t-amb", "change", (e) => put({ ambush: e.target.value === "" ? undefined : e.target.value === "rom" ? roomAt(t.pos) || 4 : Number(e.target.value) }));
  on("#t-patrol", "click", () => {
    setMode("patrol");
    setStatus("Klikk punktene på kartet. Esc er ferdig.");
  });
  on("#t-clear", "click", () => put({ patrol: [] }));
  on("#t-loot", "change", (e) => put({ loot: e.target.value || undefined }));
  on("#t-title", "change", (e) => put({ label: e.target.value }));
  on("#t-text", "change", (e) => put({ text: e.target.value }));
  on("#t-link", "click", startLink);
  on("#t-turn", "click", () => turnSelection(false));
  on("#t-del", "click", () => send({ op: "delete", ids: [t.id] }));
  on("#t-back", "click", () => send({ op: "reattach", module: t.template, thing: t.id.slice(t.template.length + 1) }));
  box.querySelectorAll("[data-unlink]").forEach((b) =>
    b.addEventListener("click", () => {
      const l = links[Number(b.dataset.unlink)];
      send({ op: "unlink", from: l.from, to: l.to });
    }),
  );
}

// Rommet (modulens) et punkt står i: bakholdet vekkes når du går inn der.
function roomAt(p) {
  const i = Math.round(p[0] / CELL);
  const j = Math.round(p[2] / CELL);
  const m = st.map.modules.find((m) => m.cells.some((c) => c[0] === i && c[2] === j && Math.abs(c[1] - p[1]) < 2.5));
  return m ? m.room || m.id : null;
}

function ambushValue(a) {
  if (a === undefined || a === null || a === false) return "";
  if (a === true) return "4";
  if (typeof a === "number") return a <= 5 ? "4" : "8";
  return "rom";
}

export function showIssues() {
  const list = $("issues");
  const issues = st.map.issues;
  const errs = issues.filter((i) => i.severity === "feil").length;
  $("issuecount").textContent = errs ? `${errs} feil` : issues.length ? `${issues.length} advarsler` : "grønt";
  $("issuecount").dataset.variant = errs ? "danger" : issues.length ? "warning" : "success";
  list.innerHTML = "";
  for (const i of issues) {
    const li = document.createElement("li");
    li.className = i.severity;
    li.textContent = i.msg;
    li.title = `${i.rule}${i.at ? " · klikk for å gå dit" : ""}`;
    if (i.at) li.onclick = () => goTo(i.at);
    // «Rett»: rettingen når den er entydig (`edit/level/fixes.rs`).
    if (i.fix) {
      const b = document.createElement("button");
      b.className = "chip fix";
      b.textContent = i.fix.label;
      b.title = "Rett bruddet (Ctrl+Z angrer)";
      b.onclick = (e) => {
        e.stopPropagation();
        applyFix(i);
      };
      li.append(" ", b);
    }
    list.append(li);
  }
}

/// Gjør rettingen til et brudd (`MapIssue.fix`).
function applyFix(i) {
  const f = i.fix;
  if (f.do === "op") {
    if (send(f.op)) toast(`Rettet: ${f.label.toLowerCase()} (Ctrl+Z angrer)`);
  } else if (f.do === "focus") {
    const t = st.map.things.find((t) => t.id === f.thing);
    if (t && !t.own) send({ op: "detach", id: f.thing });
    if (i.at) goTo(i.at, false);
    select([f.thing]);
    const el = $("props").querySelector(f.field === "text" ? "#t-text" : `#t-${f.field}`);
    el?.focus();
  } else if (f.do === "palette") {
    armModule(f.module);
    document.querySelector(`.tile[data-module="${CSS.escape(f.module)}"]`)?.scrollIntoView({ block: "nearest" });
    toast("Bosshallen er valgt i paletten: sett den ut og flytt fienden dit");
  } else if (f.do === "arm") {
    armThing(f.kind, null);
    toast("Klikk på kartet der den skal stå");
  } else if (f.do === "show") {
    goTo(f.at, false);
    select([f.module]);
    toast("Den nærmeste åpningen: åpne den mot noe man kommer til");
  }
}

export function showStats() {
  const s = st.map.stats;
  const built = `${st.map.modules.length} moduler · ${s.cells} celler (${s.recomputed} regnet nå)`;
  $("stats").textContent = `${built}\n${s.instances} biter · ${s.colliders} kollidere · ${s.polys} polygoner · ${s.rooms} rom · ${s.lights} lys`;
  $("stats").style.whiteSpace = "pre-line";
}
