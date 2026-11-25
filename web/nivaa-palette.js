// Paletten i nivåredigeringen og det du har tatt fra den (`arm`): en modul,
// en ting, et lys eller en mal, med fantomet som viser hvor den havner. Her
// står også malene (Ctrl+G) og eierens egne moduler (Ctrl+Shift+G).

import { refresh } from "./nivaa-doc.js";
import { placeFootprint, snapPlacement } from "./nivaa-plan.js";
import {
  $,
  altDown,
  cellAt,
  chip,
  engine,
  MODULES,
  palette,
  redraw,
  setDrag,
  setPalette,
  setPaletteFilter,
  setStatus,
  slug,
  st,
  store,
  TEMPLATES,
  THEME_NAMES,
  toast,
} from "./nivaa-shared.js";

// Det som er «tatt» fra paletten: { type: "module", id, rot, mirror } eller { type: "thing", kind, label }.
export let arm = null;
export let mode = null; // "link" | "patrol"
function localTemplates() {
  try {
    return JSON.parse(store.get(TEMPLATES) || "{}");
  } catch {
    return {};
  }
}

/// Malene som ikke er lagret på grenen ennå, som filer.
export function pendingTemplates() {
  return Object.entries(localTemplates())
    .filter(([, t]) => t.pending)
    .map(([key, t]) => ({ path: `assets/levels/maler/${key}.json`, text: t.text }));
}

export function clearPendingTemplates() {
  const all = localTemplates();
  for (const t of Object.values(all)) delete t.pending;
  store.set(TEMPLATES, JSON.stringify(all));
}

/// Ctrl+G: det valgte som mal, husket i nettleseren og lagret på grenen
/// neste gang du lagrer (`assets/levels/maler/<navn>.json`).
export function saveTemplate() {
  if (!st.sel.length) return toast("Velg modulene (og innholdet) som skal bli en mal først.");
  const name = prompt("Navnet på malen", "Ny mal");
  if (!name) return;
  const text = engine.level_template(JSON.stringify(st.sel), name);
  if (!text) return;
  const all = localTemplates();
  all[slug(name)] = { name, text, pending: true };
  store.set(TEMPLATES, JSON.stringify(all));
  buildTemplates();
  toast(`Malen «${name}» er lagret. Lagre legger den på grenen sammen med nivået.`);
}

function localModules() {
  try {
    return JSON.parse(store.get(MODULES) || "{}");
  } catch {
    return {};
  }
}

/// Ctrl+Shift+G: det valgte som en ny modul (`LevelEditor::as_module`): den
/// står i paletten med en gang (bildet mangler til `module_shots.py` har tatt
/// det), huskes i nettleseren og legges i
/// `assets/levels/moduler/<tema>/<navn>.json` neste gang du lagrer.
export function saveModule() {
  if (!st.sel.some((id) => st.map.modules.some((m) => m.id === id))) return toast("Velg modulene som skal bli en ny modul først.");
  const name = prompt("Navnet på modulen", "Ny modul");
  if (!name) return;
  const out = JSON.parse(engine.level_module(JSON.stringify(st.sel), name, slug(name)));
  if (out.error) return toast(out.error, 6000);
  const all = localModules();
  all[out.id] = { path: out.path, text: out.text, pending: true };
  store.set(MODULES, JSON.stringify(all));
  setPalette(JSON.parse(out.palette));
  buildPalette();
  refresh(JSON.parse(engine.level_map()));
  armModule(out.id);
  toast(`Modulen «${name}» (${out.id}) står i paletten. Bildet mangler til module_shots.py har tatt det; Lagre legger fila på grenen.`, 7000);
  return out;
}

/// Modulene eieren har laget før, inn i byggesettet (de som ikke er i repoet ennå).
export function loadOwnModules() {
  let changed = false;
  for (const [id, m] of Object.entries(localModules())) {
    if (palette.modules.some((p) => p.id === id) && !m.pending) continue;
    const out = JSON.parse(engine.level_add_module(m.text));
    if (out.error) console.warn(id, out.error);
    else {
      setPalette(out);
      changed = true;
    }
  }
  if (changed) buildPalette();
}

export function pendingModules() {
  return Object.values(localModules())
    .filter((m) => m.pending)
    .map((m) => ({ path: m.path, text: m.text }));
}

export function clearPendingModules() {
  const all = localModules();
  for (const m of Object.values(all)) delete m.pending;
  store.set(MODULES, JSON.stringify(all));
}

let repoTemplates = {};
export async function loadTemplates() {
  try {
    const r = await fetch("maler/index.json", { cache: "no-cache" });
    if (r.ok) {
      for (const f of await r.json()) {
        const t = await fetch(`maler/${f}`).then((x) => (x.ok ? x.text() : ""));
        if (t) repoTemplates[f.replace(/\.json$/, "")] = { name: JSON.parse(t).name || f, text: t };
      }
    }
  } catch {
    /* ingen maler i repoet */
  }
  buildTemplates();
}

function buildTemplates() {
  const list = $("tpllist");
  if (!list) return;
  list.innerHTML = "";
  const all = { ...repoTemplates, ...localTemplates() };
  for (const [key, t] of Object.entries(all)) {
    const b = chip(t.name + (t.pending ? " *" : ""), () => armTemplate(key, t), `tpl:${key}`);
    b.title = t.pending ? "Lagret i nettleseren; Lagre legger den på grenen" : `assets/levels/maler/${key}.json`;
    list.append(b);
  }
  if (!list.children.length) list.innerHTML = `<span class="muted">Ingen ennå.</span>`;
}

function armTemplate(key, t) {
  let tpl;
  try {
    tpl = JSON.parse(t.text);
  } catch {
    return toast("Malen kan ikke leses.");
  }
  arm = { type: "template", key, tpl, rot: 0, mirror: false };
  mode = null;
  markArmed();
  setStatus(`Klikk på kartet for å sette ut «${t.name}». R dreier, M speiler, Esc avbryter.`);
}

/// Fantomet for malen med den første modulen i ruta under pekeren: hver
/// modul flyttet, dreid og speilet som kjernen gjør (`LevelOp::Stamp`).
export function templateGhost(x, z) {
  const at = cellAt(x, z);
  const flip = (i, j) => {
    let p = [arm.mirror ? -i : i, j];
    for (let k = 0; k < (arm.rot & 3); k++) p = [-p[1], p[0]];
    return p;
  };
  const cells = [];
  const openings = [];
  for (const m of arm.tpl.modules || []) {
    const pm = palette.modules.find((p) => p.id === m.module);
    if (!pm) continue;
    const [i, j] = flip(m.at[0], m.at[2]);
    const r = (m.rot || 0) & 3;
    const rot = arm.mirror ? (arm.rot + 4 - r) & 3 : (arm.rot + r) & 3;
    const mirror = arm.mirror ? !m.mirror : !!m.mirror;
    const g = placeFootprint(pm, [at[0] + i, at[1] + (m.at[1] || 0), at[2] + j], rot, mirror);
    cells.push(...g.cells);
    openings.push(...g.openings);
  }
  const taken = new Set(st.map.modules.flatMap((m) => m.cells.map((c) => c.join())));
  return { cells, openings, at, ok: !cells.some((c) => taken.has(c.join())) };
}

export function buildPalette() {
  // Gruppene fra kjernen (`map::GROUPS`): rom, ganger, trapper, ferdigrom,
  // mekanismer og bosshallen.
  const groups = palette.groups || [["rom", "Rom"], ["gang", "Ganger"], ["trapp", "Trapper"], ["ferdig", "Ferdigrom"]];
  const list = $("modlist");
  list.innerHTML = "";
  for (const [group, title] of groups) {
    const mods = palette.modules.filter((m) => (m.group || m.kind) === group);
    if (!mods.length) continue;
    const h = document.createElement("h2");
    h.textContent = title;
    h.dataset.group = group;
    list.append(h);
    const tiles = document.createElement("div");
    tiles.className = "tiles";
    for (const m of mods) {
      const b = document.createElement("button");
      b.className = "tile";
      b.type = "button";
      b.dataset.module = m.id;
      b.title = m.about || m.name;
      const [theme, name] = m.id.split("/");
      b.dataset.theme = theme;
      b.innerHTML = `<img alt="" loading="lazy" draggable="false" src="moduler/${theme}/${name}.jpg"><span>${m.name}</span>`;
      b.addEventListener("dragstart", (e) => e.preventDefault());
      // Bildet fra `module_shots.py`; eierens nye moduler har ikke noe ennå.
      b.querySelector("img").onerror = (e) => e.target.replaceWith(Object.assign(document.createElement("div"), { className: "noimg", textContent: "Bilde mangler" }));
      b.addEventListener("pointerdown", (e) => {
        e.preventDefault();
        armModule(m.id);
        setDrag({ type: "palette" });
      });
      tiles.append(b);
    }
    list.append(tiles);
  }
  // Bygd før (en ny modul er lagt til): bare lista og filteret på nytt.
  if ($("theme")) return $("theme").dispatchEvent(new Event("change"));
  // Temaet: modulene i nivåets tema (eller alle), så samme navn i flere
  // temaer ikke står om hverandre.
  const themes = [...new Set(palette.modules.map((m) => m.theme || m.id.split("/")[0]))];
  const pick = document.createElement("select");
  pick.id = "theme";
  pick.className = "lt-select";
  pick.setAttribute("aria-label", "Temaet i paletten");
  pick.innerHTML = themes.map((t) => `<option value="${t}">${THEME_NAMES[t] || t}</option>`).join("") + `<option value="">Alle temaene</option>`;
  pick.value = themes.includes("grav") ? "grav" : themes[0] || "";
  $("search").after(pick);
  const filter = () => {
    const q = $("search").value.toLowerCase();
    const t = pick.value;
    list.querySelectorAll(".tile").forEach((el) => (el.hidden = (t && el.dataset.theme !== t) || (q && !el.textContent.toLowerCase().includes(q) && !el.dataset.module.includes(q))));
    list.querySelectorAll("h2").forEach((h) => (h.hidden = ![...h.nextElementSibling.children].some((c) => !c.hidden)));
  };
  $("search").addEventListener("input", filter);
  pick.addEventListener("change", filter);
  setPaletteFilter((theme) => {
    if (theme && themes.includes(theme) && pick.value !== theme) pick.value = theme;
    filter();
  });
  filter();
  const things = $("thinglist");
  for (const [kind, name] of palette.things.filter(([k]) => !["mob", "elite", "boss"].includes(k))) {
    things.append(chip(name, () => armThing(kind, null), kind));
  }
  const mobs = $("moblist");
  for (const [key, name] of palette.mobs) mobs.append(chip(name.replace(/^Korrupt /, ""), () => armThing("mob", key), `mob:${key}`));
  mobs.append(chip("Elite …", () => armThing("elite", "button"), "elite"));
  mobs.append(chip("Boss …", () => armThing("boss", "modal"), "boss"));
  // Lys du setter selv (dra dem etterpå; farge, styrke og flamme i panelet).
  things.append(chip("Lys", armLight, "light"));
}

function armLight() {
  arm = { type: "light" };
  mode = null;
  markArmed();
  setStatus("Klikk på kartet for å sette ut et lys. Esc avbryter.");
}

export function armModule(id) {
  arm = { type: "module", id, rot: arm?.type === "module" && arm.id === id ? arm.rot : 0, mirror: false };
  mode = null;
  markArmed();
  setStatus("Slipp den nær en ledig åpning, så snapper den på plass · R dreier · M speiler · Alt fritt · Esc avbryter");
}

export function armThing(kind, label) {
  arm = { type: "thing", kind, label, yaw: 0 };
  mode = null;
  markArmed();
  setStatus("Klikk på kartet for å sette ut. R dreier, Esc avbryter.");
}

export function disarm() {
  arm = null;
  st.ghost = null;
  st.thingGhost = null;
  mode = null;
  st.linkFrom = null;
  markArmed();
  setStatus("");
  redraw();
}

function markArmed() {
  document.querySelectorAll(".tile, .chip").forEach((el) => {
    const on =
      arm &&
      ((arm.type === "module" && el.dataset.module === arm.id) ||
        (arm.type === "light" && el.dataset.key === "light") ||
        (arm.type === "template" && el.dataset.key === `tpl:${arm.key}`) ||
        (arm.type === "thing" && (el.dataset.key === arm.kind || el.dataset.key === `mob:${arm.label}`) && (arm.kind !== "mob" || el.dataset.key === `mob:${arm.label}`)));
    el.classList.toggle("armed", !!on);
  });
}

export function ghostFor(x, z) {
  const pm = palette.modules.find((m) => m.id === arm.id);
  if (!pm) return null;
  const want = cellAt(x, z);
  const taken = new Set(st.map.modules.flatMap((m) => m.cells.map((c) => c.join())));
  // Magneten: en åpning mot en ledig åpning i nærheten; uten R dreies
  // modulen selv så den passer.
  const rots = arm.turned ? [arm.rot] : [0, 1, 2, 3].map((k) => (arm.rot + k) & 3);
  const snapped = altDown ? null : snapPlacement(st.map, pm, want, rots, arm.mirror, taken);
  const rot = snapped ? snapped.rot : arm.rot;
  const at = snapped ? snapped.at : want;
  const g = placeFootprint(pm, at, rot, arm.mirror);
  g.ok = !g.cells.some((c) => taken.has(c.join()));
  g.at = at;
  g.rot = rot;
  g.links = snapped?.links || [];
  return g;
}

export function startLink() {
  const id = st.sel[0];
  const t = st.map.things.find((x) => x.id === id);
  if (!t) return toast("Velg en spak, en elite eller et hint, så K, og klikk på porten eller den falske veggen.");
  mode = "link";
  st.linkFrom = id;
  setStatus("Klikk på målet: en port, en bro … eller en åpning (den blir en port) · Esc avbryter");
}

export const setMode = (v) => {
  mode = v;
};
