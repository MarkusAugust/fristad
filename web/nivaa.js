// Nivåredigeringen (`?nivaa`, docs/nivaaredigering.md del 12): eieren
// bygger kryptnivåene av moduler på et kart ovenfra, med motoren som 3D ved
// siden av. Dokumentet og reglene bor i kjernen (`edit::level`,
// `LevelEditor`); siden sender endringene som json (`engine.level`) og
// tegner kartet den får tilbake (web/nivaa-plan.js). Utkastet huskes i
// nettleseren, Lagre legger fila på grenen `redigering` (web/github.js) eller
// laster ned en zip, og «Spill» bygger verden med nivået og lar deg gå
// gjennom det uten å kunne dø (Esc tilbake).

import init, {
  start_level_editor,
  asset_paths,
  model_paths,
  model_url,
  level_doc_names,
  level_doc_source,
  level_blank,
  level_palette,
} from "./pkg/fristad.js";
import { draw, hit, makeView, placeFootprint, CELL, CHOSEN_COLOR } from "./nivaa-plan.js";
import { commit, token, setToken, branchFiles, SAVE_BRANCH } from "./github.js";
import { makeZip } from "./editor.js";

const $ = (id) => document.getElementById(id);
const plan = $("plan");
let canvas = $("world");
const params = new URLSearchParams(location.search);
const DRAFT_PREFIX = "fristad.nivaa.utkast:";
const LAST = "fristad.nivaa.siste";

let engine = null;
let palette = null;
const st = {
  map: null,
  floor: 0,
  view: makeView(plan),
  sel: [],
  selOpening: null,
  ghost: null,
  thingGhost: null,
  box: null,
  linkFrom: null,
  mouse: null,
  camera: null,
  flash: null,
};
// Det som er «tatt» fra paletten: { type: "module", id, rot, mirror } eller { type: "thing", kind, label }.
let arm = null;
let mode = null; // "link" | "patrol"
let clipboard = null;
let viewMode = "split";
let playing = false;
let docPath = "";
let lastMouse = [0, 0];
let drag = null;
// Paletten viser temaet nivået har (settes når nivået er åpnet).
let paletteFilter = () => {};
const THEME_NAMES = { grav: "Gravkjelleren", kloakk: "Kloakken", gruve: "Gruva", borg: "Borgen", kjerne: "Kjernen" };

const store = {
  get(k) {
    try {
      return localStorage.getItem(k);
    } catch {
      return null;
    }
  },
  set(k, v) {
    try {
      localStorage.setItem(k, v);
    } catch {
      /* ingen lagring */
    }
  },
  del(k) {
    try {
      localStorage.removeItem(k);
    } catch {
      /* ingen lagring */
    }
  },
};

function toast(text, ms = 3500) {
  const t = $("toast");
  t.textContent = text;
  t.hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => (t.hidden = true), ms);
}

function setStatus(text) {
  $("status").textContent = text;
}

// ---------------------------------------------------------------- lasting

async function loadAssets() {
  try {
    const res = await fetch("assets/manifest.json", { cache: "no-cache" });
    if (!res.ok) return { manifest: undefined, names: [], blobs: [] };
    const manifest = await res.text();
    const paths = asset_paths(manifest);
    const blobs = new Array(paths.length);
    let next = 0;
    let done = 0;
    const worker = async () => {
      while (next < paths.length) {
        const i = next++;
        try {
          const r = await fetch(`assets/${paths[i]}`);
          if (r.ok) blobs[i] = new Uint8Array(await r.arrayBuffer());
        } catch {
          /* laget bakes i motoren */
        }
        done++;
        progress(`Henter teksturer (${done} av ${paths.length}) …`, done / paths.length);
      }
    };
    await Promise.all(Array.from({ length: 6 }, worker));
    const got = paths.map((p, i) => [p, blobs[i]]).filter(([, b]) => b);
    return { manifest, names: got.map(([p]) => p), blobs: got.map(([, b]) => b) };
  } catch {
    return { manifest: undefined, names: [], blobs: [] };
  }
}

async function fetchModel(path) {
  const r = await fetch(`models/${model_url(path)}.gz`);
  if (!r.ok) return undefined;
  let bytes = new Uint8Array(await r.arrayBuffer());
  if (bytes[0] === 0x1f && bytes[1] === 0x8b) {
    const plain = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
    bytes = new Uint8Array(await new Response(plain).arrayBuffer());
  }
  return bytes;
}

async function loadModels() {
  const all = model_paths();
  const out = new Array(all.length);
  let next = 0;
  let done = 0;
  const worker = async () => {
    while (next < all.length) {
      const i = next++;
      out[i] = await fetchModel(all[i]).catch(() => undefined);
      done++;
      progress(`Henter modellene (${done} av ${all.length}) …`, done / all.length);
    }
  };
  await Promise.all(Array.from({ length: 6 }, worker));
  return out;
}

function progress(text, frac) {
  $("loading").textContent = text;
  $("bar").value = Math.round(frac * 100);
}

// Dokumentet siden åpner med: `?nivaa=<navn>` eller `&fil=<navn>` fra
// repoet (et nyere utkast går foran), ellers det siste du hadde oppe, ellers
// et nytt nivå 1.
function firstDoc() {
  const name = params.get("fil") || params.get("nivaa") || "";
  const path = name ? `assets/levels/nivaa_${name}.json` : store.get(LAST) || "";
  const draft = path && store.get(DRAFT_PREFIX + path);
  if (draft) {
    try {
      const d = JSON.parse(draft);
      if (d.text) return { text: d.text, note: `Fortsetter fra utkastet (${new Date(d.at).toLocaleString("nb")}).` };
    } catch {
      /* ødelagt utkast */
    }
  }
  const short = path.replace(/^assets\/levels\/nivaa_/, "").replace(/\.json$/, "");
  const text = short ? level_doc_source(short) : "";
  if (text) return { text, note: "" };
  return { text: level_blank("Nytt nivå", 1, "grav"), note: "Nytt nivå." };
}

async function boot() {
  progress("Laster motoren …", 0);
  await init();
  palette = JSON.parse(level_palette());
  buildPalette();
  const assets = await loadAssets();
  const models = await loadModels();
  progress("Bygger byen og nivået …", 1);
  await new Promise((r) => setTimeout(r, 30));
  const first = firstDoc();
  const webgl = params.has("webgl");
  try {
    if (webgl) throw new Error("WebGL2 valgt");
    engine = await start_level_editor(canvas, true, assets.manifest, assets.names, assets.blobs, models, first.text);
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    canvas = fresh;
    engine = await start_level_editor(canvas, false, assets.manifest, assets.names, assets.blobs, models, first.text);
  }
  window.fristad = { engine, level: send, state: st, setFloor, place: placeModule };
  engine.set_quality(engine.backend() === "Gl" ? "enkel" : "standard");
  fit();
  new ResizeObserver(fit).observe(canvas);
  new ResizeObserver(() => redraw()).observe(plan);
  refresh(JSON.parse(engine.level_map()));
  centerOn();
  bindUi();
  bindMap();
  bind3d();
  $("overlay").hidden = true;
  if (first.note) toast(first.note);
  requestAnimationFrame(loop);
}

// ---------------------------------------------------------------- motoren

function fit() {
  const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
  const w = Math.max(1, Math.round(canvas.clientWidth * dpr));
  const h = Math.max(1, Math.round(canvas.clientHeight * dpr));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  engine?.resize(w, h);
}

let lastT = performance.now();
let camTick = 0;
function loop(t) {
  const dt = Math.min(0.1, (t - lastT) / 1000);
  lastT = t;
  if (engine && (viewMode !== "map" || playing)) {
    try {
      const events = JSON.parse(engine.frame(dt));
      for (const e of events) if (e.type === "toast" && playing) toast(e.text);
    } catch (e) {
      console.error(e);
    }
    if (!playing && ++camTick % 6 === 0) {
      st.camera = engine.level_camera();
      redraw();
    }
  }
  requestAnimationFrame(loop);
}

/// Send en kommando eller endring; kartet kommer tilbake.
function send(cmd) {
  const out = JSON.parse(engine.level(JSON.stringify(cmd)));
  if (out.error) {
    toast(out.error);
    return null;
  }
  refresh(out);
  return out;
}

function refresh(map) {
  st.map = map;
  // Etasjene.
  const floors = map.floors.length ? map.floors : [0];
  const sel = $("floor");
  const want = floors.map(String).join();
  if (sel.dataset.list !== want) {
    sel.innerHTML = floors.map((f) => `<option value="${f}">${f === 0 ? "Hovedplanet (0 m)" : `${f > 0 ? "+" : ""}${f} m`}</option>`).join("");
    sel.dataset.list = want;
  }
  if (!floors.includes(st.floor)) st.floor = floors.reduce((a, f) => (Math.abs(f - st.floor) < Math.abs(a - st.floor) ? f : a), floors[0]);
  sel.value = String(st.floor);
  // Det valgte som ikke finnes lenger.
  const ids = new Set([...map.modules.map((m) => m.id), ...map.things.map((t) => t.id), ...map.lights.map((l) => l.id)]);
  st.sel = st.sel.filter((id) => ids.has(id));
  $("docname").textContent = `${map.name || "Uten navn"} · ${/^\d+$/.test(map.level) ? `nivå ${map.level}` : map.level}`;
  $("docstatus").textContent = map.status === "klar" ? "Klar" : "Utkast";
  $("docstatus").dataset.variant = map.status === "klar" ? "success" : "warning";
  $("undo").disabled = !map.can_undo;
  $("redo").disabled = !map.can_redo;
  $("save").textContent = map.unsaved ? `Lagre (${map.unsaved})` : "Lagre";
  $("ready").disabled = !map.clean || map.status === "klar";
  $("ready").title = map.clean ? "Merk nivået klart: det erstatter det gamle nivået med samme nummer" : "Rett feilene i reglene først";
  showIssues();
  showSelection();
  showStats();
  redraw();
  docPath = JSON.parse(engine.level_doc()).path;
  if (map.theme !== refresh.theme) {
    refresh.theme = map.theme;
    paletteFilter(map.theme || map.modules[0]?.module.split("/")[0]);
  }
  store.set(LAST, docPath);
  scheduleDraft();
}

function redraw() {
  draw(plan, st);
}

// ---------------------------------------------------------------- utkast og lagring

let draftTimer = 0;
function scheduleDraft() {
  if (!st.map || !st.map.unsaved) return;
  clearTimeout(draftTimer);
  draftTimer = setTimeout(() => {
    const d = JSON.parse(engine.level_doc());
    store.set(DRAFT_PREFIX + d.path, JSON.stringify({ at: Date.now(), text: d.text }));
  }, 800);
}

async function save() {
  const d = JSON.parse(engine.level_doc());
  store.set(DRAFT_PREFIX + d.path, JSON.stringify({ at: Date.now(), text: d.text }));
  if (!token()) {
    const key = prompt("Nøkkel til GitHub (github_pat_…, «Contents: Read and write» på fristad-repoet). La stå tom for å laste ned en zip i stedet.");
    if (key) setToken(key.trim());
  }
  if (!token()) {
    const url = URL.createObjectURL(makeZip([d]));
    const a = document.createElement("a");
    a.href = url;
    a.download = d.path.split("/").pop().replace(".json", ".zip");
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 2000);
    send({ cmd: "saved" });
    toast(`Lastet ned ${a.download} (utkastet er lagret i nettleseren).`);
    return;
  }
  setStatus("Lagrer …");
  try {
    const r = await commit([d], `Nivåredigeringen: ${st.map.name || d.path}`);
    send({ cmd: "saved" });
    toast(r.unchanged ? "Ingenting nytt å lagre." : `Lagret på grenen ${SAVE_BRANCH}.`);
  } catch (e) {
    toast(`Lagringen feilet: ${e.message}. Utkastet ligger i nettleseren.`, 7000);
  }
  setStatus("");
}

function openDialog() {
  const card = document.createElement("div");
  card.className = "card";
  const names = level_doc_names().filter((n) => n !== "brett");
  const drafts = [];
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(DRAFT_PREFIX)) drafts.push(k.slice(DRAFT_PREFIX.length));
  }
  card.innerHTML = `
    <h2 class="fs-heading" data-size="s">Åpne et nivå</h2>
    <p class="muted">Utkastene i denne nettleseren og nivåene i repoet. Et nivå merket «klar» erstatter det gamle nivået med samme nummer.</p>
    <div class="doclist">${drafts.map((p) => `<button class="fs-button" data-variant="secondary" data-draft="${p}">Utkast: ${p.replace("assets/levels/", "")}</button>`).join("")}
    ${names.map((n) => `<button class="fs-button" data-variant="secondary" data-repo="${n}">I repoet: nivaa_${n}.json</button>`).join("")}
    <button class="fs-button" data-variant="secondary" data-branch="1">Hent dette nivået fra grenen ${SAVE_BRANCH}</button></div>
    <h2 class="fs-heading" data-size="s">Nytt nivå</h2>
    <div class="row">
      <input id="new-name" class="fs-input" placeholder="Navnet" value="Nytt nivå" style="flex:1">
      <select id="new-n" class="fs-select">${[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => `<option>${n}</option>`).join("")}</select>
      <select id="new-theme" class="fs-select">${(palette.themes.length ? palette.themes : ["grav"]).map((t) => `<option>${t}</option>`).join("")}</select>
      <button id="new-make" class="fs-button" data-variant="primary">Lag</button>
    </div>
    <div class="row" style="margin-top:12px"><button id="open-cancel" class="fs-button" data-variant="secondary">Avbryt</button></div>`;
  const ov = $("overlay");
  ov.replaceChildren(card);
  ov.hidden = false;
  const close = () => (ov.hidden = true);
  card.querySelector("#open-cancel").onclick = close;
  card.querySelectorAll("[data-draft]").forEach((b) =>
    b.addEventListener("click", () => {
      const d = JSON.parse(store.get(DRAFT_PREFIX + b.dataset.draft) || "{}");
      if (d.text) openText(d.text, "Utkastet er åpnet.");
      close();
    }),
  );
  card.querySelectorAll("[data-repo]").forEach((b) =>
    b.addEventListener("click", () => {
      openText(level_doc_source(b.dataset.repo), `nivaa_${b.dataset.repo}.json fra repoet.`);
      close();
    }),
  );
  card.querySelector("[data-branch]").onclick = async () => {
    try {
      const files = await branchFiles([docPath]);
      if (!files.length) toast(token() ? `${docPath} finnes ikke på grenen ${SAVE_BRANCH}.` : "Trenger nøkkelen til GitHub (Lagre spør etter den).");
      else openText(files[0].text, `Hentet ${docPath} fra grenen ${SAVE_BRANCH}.`);
    } catch (e) {
      toast(e.message);
    }
    close();
  };
  card.querySelector("#new-make").onclick = () => {
    const n = Number(card.querySelector("#new-n").value);
    openText(level_blank(card.querySelector("#new-name").value || `Nivå ${n}`, n, card.querySelector("#new-theme").value), `Nytt nivå ${n}.`);
    close();
  };
}

function openText(text, note) {
  const out = JSON.parse(engine.level_open(text));
  if (out.error) return toast(out.error);
  st.sel = [];
  st.selOpening = null;
  refresh(out);
  centerOn();
  if (note) toast(note);
}

// ---------------------------------------------------------------- paletten

function buildPalette() {
  const groups = { rom: "Rom", gang: "Ganger", trapp: "Trapper", ferdig: "Ferdige rom" };
  const list = $("modlist");
  list.innerHTML = "";
  for (const [kind, title] of Object.entries(groups)) {
    const mods = palette.modules.filter((m) => m.kind === kind);
    if (!mods.length) continue;
    const h = document.createElement("h2");
    h.textContent = title;
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
      b.querySelector("img").onerror = (e) => (e.target.style.visibility = "hidden");
      b.addEventListener("pointerdown", (e) => {
        e.preventDefault();
        armModule(m.id);
        drag = { type: "palette" };
      });
      tiles.append(b);
    }
    list.append(tiles);
  }
  // Temaet: modulene i nivåets tema (eller alle), så samme navn i flere
  // temaer ikke står om hverandre.
  const themes = [...new Set(palette.modules.map((m) => m.theme || m.id.split("/")[0]))];
  const pick = document.createElement("select");
  pick.id = "theme";
  pick.className = "fs-select";
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
  paletteFilter = (theme) => {
    if (theme && themes.includes(theme) && pick.value !== theme) pick.value = theme;
    filter();
  };
  filter();
  const things = $("thinglist");
  for (const [kind, name] of palette.things.filter(([k]) => !["mob", "elite", "boss"].includes(k))) {
    things.append(chip(name, () => armThing(kind, null), kind));
  }
  const mobs = $("moblist");
  for (const [key, name] of palette.mobs) mobs.append(chip(name.replace(/^Korrupt /, ""), () => armThing("mob", key), `mob:${key}`));
  mobs.append(chip("Elite …", () => armThing("elite", "button"), "elite"));
  mobs.append(chip("Boss …", () => armThing("boss", "modal"), "boss"));
}

function chip(text, fn, key) {
  const b = document.createElement("button");
  b.className = "chip";
  b.type = "button";
  b.textContent = text;
  b.dataset.key = key;
  b.addEventListener("click", fn);
  return b;
}

function armModule(id) {
  arm = { type: "module", id, rot: arm?.type === "module" && arm.id === id ? arm.rot : 0, mirror: false };
  mode = null;
  markArmed();
  setStatus("Klikk på kartet for å sette ut. R dreier, M speiler, Esc avbryter.");
}

function armThing(kind, label) {
  arm = { type: "thing", kind, label, yaw: 0 };
  mode = null;
  markArmed();
  setStatus("Klikk på kartet for å sette ut. R dreier, Esc avbryter.");
}

function disarm() {
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
    const on = arm && ((arm.type === "module" && el.dataset.module === arm.id) || (arm.type === "thing" && (el.dataset.key === arm.kind || el.dataset.key === `mob:${arm.label}`) && (arm.kind !== "mob" || el.dataset.key === `mob:${arm.label}`)));
    el.classList.toggle("armed", !!on);
  });
}

// ---------------------------------------------------------------- kartet

function worldAt(e) {
  const r = plan.getBoundingClientRect();
  return st.view.toWorld(e.clientX - r.left, e.clientY - r.top);
}

function cellAt(x, z) {
  return [Math.round(x / CELL), st.floor, Math.round(z / CELL)];
}

function ghostFor(x, z) {
  const pm = palette.modules.find((m) => m.id === arm.id);
  if (!pm) return null;
  const at = cellAt(x, z);
  const g = placeFootprint(pm, at, arm.rot, arm.mirror);
  const taken = new Set(st.map.modules.flatMap((m) => m.cells.map((c) => c.join())));
  g.ok = !g.cells.some((c) => taken.has(c.join()));
  g.at = at;
  return g;
}

/// Sett ut en modul (`id`) med midtcella i `at` (til testene og konsollen).
function placeModule(id, at, rot = 0, mirror = false) {
  const out = send({ op: "add_module", module: id, at, rot, mirror });
  if (out?.made?.length) select(out.made);
  return out;
}

function select(ids, add = false) {
  st.sel = add ? [...new Set([...st.sel, ...ids])] : ids;
  st.selOpening = null;
  engine.level(JSON.stringify({ cmd: "select", ids: st.sel }));
  showSelection();
  redraw();
}

function bindMap() {
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
    plan.focus();
    plan.setPointerCapture(e.pointerId);
    const [x, z] = worldAt(e);
    lastMouse = [e.clientX, e.clientY];
    if (e.button === 2 || e.button === 1 || space) {
      drag = { type: "pan" };
      return;
    }
    if (e.button !== 0) return;
    if (mode === "patrol") return addPatrol(x, z);
    if (mode === "link") return finishLink(x, z);
    if (arm) return;
    const h = hit(st.map, st.floor, x, z, Math.max(0.6, 10 / st.view.s));
    if (h?.type === "thing") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      drag = h.own ? { type: "thing", id: h.id, from: [x, z], moved: false } : null;
      return;
    }
    if (h?.type === "opening") {
      st.sel = [];
      st.selOpening = { module: h.module, opening: h.opening };
      engine.level(JSON.stringify({ cmd: "select", ids: [h.module] }));
      showSelection();
      redraw();
      return;
    }
    if (h?.type === "module") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      drag = { type: "modules", from: cellAt(x, z), by: [0, 0, 0] };
      return;
    }
    if (!e.shiftKey) select([]);
    const r = plan.getBoundingClientRect();
    drag = { type: "box", a: [e.clientX - r.left, e.clientY - r.top], add: e.shiftKey };
  });
  plan.addEventListener("pointermove", (e) => {
    const [x, z] = worldAt(e);
    const r = plan.getBoundingClientRect();
    st.mouse = [e.clientX - r.left, e.clientY - r.top];
    if (drag?.type === "pan") {
      st.view.cx -= (e.clientX - lastMouse[0]) / st.view.s;
      st.view.cz -= (e.clientY - lastMouse[1]) / st.view.s;
      lastMouse = [e.clientX, e.clientY];
      return redraw();
    }
    lastMouse = [e.clientX, e.clientY];
    if (arm?.type === "module") st.ghost = ghostFor(x, z);
    if (arm?.type === "thing") st.thingGhost = [snap(x), st.floor, snap(z)];
    if (drag?.type === "modules") {
      const c = cellAt(x, z);
      drag.by = [c[0] - drag.from[0], 0, c[2] - drag.from[2]];
      // Fantomet: de valgte modulene flyttet.
      const moved = st.map.modules.filter((m) => st.sel.includes(m.id));
      st.ghost = drag.by[0] || drag.by[2] ? { cells: moved.flatMap((m) => m.cells.map((q) => [q[0] + drag.by[0], q[1], q[2] + drag.by[2]])), openings: [], ok: true } : null;
    }
    if (drag?.type === "thing") {
      drag.moved = drag.moved || Math.hypot(x - drag.from[0], z - drag.from[1]) > 0.3;
      if (drag.moved) st.thingGhost = [snap(x), st.floor, snap(z)];
    }
    if (drag?.type === "box") st.box = [...drag.a, ...st.mouse];
    redraw();
  });
  plan.addEventListener("pointerup", (e) => {
    const [x, z] = worldAt(e);
    const d = drag;
    drag = null;
    if (d?.type === "pan") return;
    if (arm?.type === "module" && (e.button === 0 || d?.type === "palette")) {
      const g = ghostFor(x, z);
      if (g) placeModule(arm.id, g.at, arm.rot, arm.mirror);
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
      if (d.by[0] || d.by[2]) send({ op: "move", ids: st.sel, by: d.by });
      redraw();
      return;
    }
    if (d?.type === "thing") {
      st.thingGhost = null;
      if (d.moved) send({ op: "move_thing", id: d.id, to: [snap(x), thingY(d.id), snap(z)] });
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
  // Dra fra paletten og slipp på kartet.
  addEventListener("pointerup", (e) => {
    if (drag?.type !== "palette") return;
    drag = null;
    const r = plan.getBoundingClientRect();
    if (e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom && arm?.type === "module") {
      const [x, z] = st.view.toWorld(e.clientX - r.left, e.clientY - r.top);
      const g = ghostFor(x, z);
      if (g) placeModule(arm.id, g.at, arm.rot, arm.mirror);
      disarm();
    }
  });
  addEventListener("pointermove", (e) => {
    if (drag?.type !== "palette" || !arm) return;
    const r = plan.getBoundingClientRect();
    if (e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom) {
      const [x, z] = st.view.toWorld(e.clientX - r.left, e.clientY - r.top);
      st.ghost = ghostFor(x, z);
      redraw();
    }
  });
  plan.addEventListener("pointerleave", () => {
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

function snap(v) {
  return Math.round(v * 2) / 2;
}

function thingY(id) {
  return st.map.things.find((t) => t.id === id)?.pos[1] ?? st.floor;
}

function addPatrol(x, z) {
  const t = docThing(st.sel[0]);
  if (!t) return disarm();
  t.patrol = [...(t.patrol || []), [snap(x), t.pos?.[1] ?? st.floor, snap(z)]];
  send({ op: "set_thing", thing: t });
}

function startLink() {
  const id = st.sel[0];
  const t = st.map.things.find((x) => x.id === id);
  if (!t) return toast("Velg en spak, en elite eller et hint, så K, og klikk på porten eller den falske veggen.");
  mode = "link";
  st.linkFrom = id;
  setStatus("Klikk på målet (en port, falsk vegg, bro …). Esc avbryter.");
}

function finishLink(x, z) {
  const from = st.linkFrom;
  const h = hit(st.map, st.floor, x, z, Math.max(0.8, 12 / st.view.s));
  let to = null;
  if (h?.type === "thing") to = h.id;
  if (h?.type === "edge") to = h.switch;
  if (h?.type === "opening") {
    const m = st.map.modules.find((m) => m.id === h.module);
    const ov = m?.overrides.find((o) => o.opening === h.opening);
    to = ov?.id || null;
  }
  disarm();
  if (!to) return toast("Det er ikke noe å koble til der. Gjør åpningen til en port, snarvei eller falsk vegg først.");
  const fromT = st.map.things.find((t) => t.id === from);
  const kind = fromT?.kind === "elite" ? "key" : fromT?.kind === "hint" ? "for" : "opens";
  send({ op: "link", from, to, kind });
}

function centerOn() {
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

function setFloor(y) {
  st.floor = y;
  $("floor").value = String(y);
  engine.level(JSON.stringify({ cmd: "floor", y }));
  redraw();
}

// ---------------------------------------------------------------- 3D

let looking = false;
function bind3d() {
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  canvas.addEventListener("pointerdown", (e) => {
    canvas.focus();
    if (playing) {
      canvas.requestPointerLock?.();
      engine.mouse(e.button, true);
      return;
    }
    if (e.button === 2) {
      looking = true;
      canvas.setPointerCapture(e.pointerId);
      return;
    }
    if (e.button === 0) {
      const r = canvas.getBoundingClientRect();
      const p = engine.level_pick((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height);
      if (p.length) {
        const h = hit(st.map, st.floor, p[0], p[2], 0.8);
        if (h?.type === "module" || h?.type === "thing") select([h.id], e.shiftKey);
        else if (!e.shiftKey) select([]);
      }
    }
  });
  canvas.addEventListener("pointerup", (e) => {
    if (playing) return engine.mouse(e.button, false);
    if (e.button === 2) looking = false;
  });
  canvas.addEventListener("pointermove", (e) => {
    if (looking || (playing && document.pointerLockElement === canvas)) engine.look(e.movementX, e.movementY);
  });
}

const FLY_KEYS = ["KeyW", "KeyA", "KeyS", "KeyD", "Space", "KeyC", "ShiftLeft", "ShiftRight"];
function bindUi() {
  $("undo").onclick = () => send({ cmd: "undo" });
  $("redo").onclick = () => send({ cmd: "redo" });
  $("save").onclick = save;
  $("open").onclick = openDialog;
  $("play").onclick = () => play(false);
  $("play-here").onclick = () => play(true);
  $("ready").onclick = () => {
    const head = docHead();
    head.status = "klar";
    if (send({ op: "set_head", head })) toast("Merket klart. Lagre, så tar Claude det inn i spillet.");
  };
  $("floor").onchange = () => setFloor(Number($("floor").value));
  $("v-map").onclick = () => setView("map");
  $("v-split").onclick = () => setView("split");
  $("v-3d").onclick = () => setView("3d");
  $("light").onclick = () => {
    const on = $("light").textContent !== "Arbeidslys";
    $("light").textContent = on ? "Arbeidslys" : "Spillets lys";
    engine.level(JSON.stringify({ cmd: "light", work: on }));
  };
  addEventListener("keydown", onKey);
  addEventListener("keyup", (e) => {
    if (playing || (document.activeElement === canvas && FLY_KEYS.includes(e.code))) engine.key(e.code, false);
  });
  addEventListener("blur", () => engine?.release_all());
}

function typing() {
  const a = document.activeElement;
  return a && (a.tagName === "INPUT" || a.tagName === "TEXTAREA" || a.tagName === "SELECT");
}

function onKey(e) {
  if (playing) {
    if (e.code === "Escape") {
      e.preventDefault();
      return back();
    }
    engine.key(e.code, true);
    return;
  }
  const ctrl = e.ctrlKey || e.metaKey;
  if (ctrl && e.code === "KeyS") {
    e.preventDefault();
    return save();
  }
  if (typing()) return;
  if (document.activeElement === canvas && FLY_KEYS.includes(e.code) && !ctrl) {
    e.preventDefault();
    engine.key(e.code, true);
    return;
  }
  if (ctrl && e.code === "KeyZ") {
    e.preventDefault();
    return send({ cmd: e.shiftKey ? "redo" : "undo" });
  }
  if (ctrl && e.code === "KeyY") {
    e.preventDefault();
    return send({ cmd: "redo" });
  }
  if (ctrl && e.code === "KeyC") return copy();
  if (ctrl && e.code === "KeyV") return paste();
  if (ctrl && e.code === "KeyD") {
    e.preventDefault();
    copy();
    return paste();
  }
  switch (e.code) {
    case "Escape":
      disarm();
      select([]);
      break;
    case "KeyR":
      if (arm?.type === "module") {
        arm.rot = (arm.rot + (e.shiftKey ? 3 : 1)) & 3;
        if (st.mouse) st.ghost = ghostFor(...st.view.toWorld(...st.mouse));
        redraw();
      } else if (arm?.type === "thing") arm.yaw = (arm.yaw + 90) % 360;
      else if (st.sel.length) turnSelection(e.shiftKey);
      break;
    case "KeyM":
    case "KeyX":
      if (arm?.type === "module") {
        arm.mirror = !arm.mirror;
        if (st.mouse) st.ghost = ghostFor(...st.view.toWorld(...st.mouse));
        redraw();
      } else if (st.sel.length) send({ op: "mirror", ids: st.sel });
      break;
    case "Delete":
    case "Backspace":
      if (st.sel.length) send({ op: "delete", ids: st.sel });
      break;
    case "KeyK":
      startLink();
      break;
    case "KeyL":
      $("light").click();
      break;
    case "KeyF":
      focusSelection();
      break;
    case "KeyP":
      play(e.shiftKey);
      break;
    case "PageUp":
    case "PageDown": {
      e.preventDefault();
      const f = st.map.floors;
      const i = f.indexOf(st.floor) + (e.code === "PageUp" ? 1 : -1);
      if (i >= 0 && i < f.length) setFloor(f[i]);
      break;
    }
    case "Tab":
      e.preventDefault();
      setView({ split: "map", map: "3d", "3d": "split" }[viewMode]);
      break;
    case "Home":
      centerOn();
      break;
  }
}

function turnSelection(back) {
  const things = st.sel.filter((id) => !st.map.modules.some((m) => m.id === id));
  if (things.length === st.sel.length && things.length === 1) {
    const t = docThing(things[0]);
    if (t) {
      t.yaw = ((t.yaw || 0) + (back ? 90 : -90) + 360) % 360;
      return send({ op: "set_thing", thing: t });
    }
  }
  send({ op: "turn", ids: st.sel, back });
}

function focusSelection() {
  const m = st.map.modules.find((m) => st.sel.includes(m.id));
  const t = st.map.things.find((t) => st.sel.includes(t.id));
  let p = null;
  if (m) p = [m.cells.reduce((a, c) => a + c[0], 0) / m.cells.length * CELL, m.cells[0][1], m.cells.reduce((a, c) => a + c[2], 0) / m.cells.length * CELL];
  else if (t) p = t.pos;
  if (!p) return;
  st.view.cx = p[0];
  st.view.cz = p[2];
  engine.level(JSON.stringify({ cmd: "place", x: p[0], y: p[1] + 5, z: p[2] + 7, yaw: 0, pitch: -35 }));
  redraw();
}

function setView(m) {
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

function copy() {
  const doc = JSON.parse(JSON.parse(engine.level_doc()).text);
  clipboard = { modules: doc.modules?.filter((m) => st.sel.includes(m.id)) || [], things: doc.things?.filter((t) => st.sel.includes(t.id)) || [] };
  toast(`Kopierte ${clipboard.modules.length} moduler og ${clipboard.things.length} ting.`, 1500);
}

function paste() {
  if (!clipboard) return;
  // Ved siden av: like langt til høyre som utvalget er bredt.
  const cells = st.map.modules.filter((m) => clipboard.modules.some((c) => c.id === m.id)).flatMap((m) => m.cells);
  const w = cells.length ? Math.max(...cells.map((c) => c[0])) - Math.min(...cells.map((c) => c[0])) + 1 : 1;
  const out = send({ op: "paste", modules: clipboard.modules, things: clipboard.things, by: [w, 0, 0] });
  if (out?.made?.length) select(out.made);
}

// ---------------------------------------------------------------- prøveturen

function play(here) {
  if (playing) return;
  setStatus("Bygger nivået …");
  $("overlay").replaceChildren(Object.assign(document.createElement("div"), { className: "card", innerHTML: "<p class='fs-paragraph'>Bygger verden med nivået …</p>" }));
  $("overlay").hidden = false;
  setTimeout(() => {
    const ok = engine.level_play(here);
    $("overlay").hidden = true;
    setStatus("");
    if (!ok) return toast("Prøveturen kunne ikke starte.");
    playing = true;
    document.body.classList.add("playing");
    $("playbar").hidden = false;
    setView("3d");
    canvas.focus();
    canvas.requestPointerLock?.();
  }, 30);
}

function back() {
  playing = false;
  document.exitPointerLock?.();
  engine.level_back();
  document.body.classList.remove("playing");
  $("playbar").hidden = true;
  setView("split");
  refresh(JSON.parse(engine.level_map()));
}

document.addEventListener("pointerlockchange", () => {
  if (playing && !document.pointerLockElement) setStatus("Klikk i bildet for å fange musa, Esc for å gå tilbake.");
});

// ---------------------------------------------------------------- panelet

function docJson() {
  return JSON.parse(JSON.parse(engine.level_doc()).text);
}

function docHead() {
  const d = docJson();
  return { name: d.name || "", level: d.level, status: d.status || "utkast", theme: d.theme || "", seed: d.seed || 0, stone: d.stone };
}

function docThing(id) {
  return docJson().things?.find((t) => t.id === id) || null;
}

function docModule(id) {
  return docJson().modules?.find((m) => m.id === id) || null;
}

function esc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

function field(label, html) {
  return `<div class="field"><span class="fs-label">${label}</span>${html}</div>`;
}

function options(list, value) {
  return list.map(([v, n]) => `<option value="${esc(v)}"${String(v) === String(value ?? "") ? " selected" : ""}>${esc(n)}</option>`).join("");
}

function stoneOptions(value) {
  const v = typeof value === "string" ? value : "";
  return options([["", "Som rommet rundt (standard)"], ...palette.stones], v);
}

function showSelection() {
  const box = $("sel");
  const map = st.map;
  if (st.selOpening) {
    const m = map.modules.find((m) => m.id === st.selOpening.module);
    const o = m?.openings.find((o) => o.id === st.selOpening.opening);
    if (!o) return (box.innerHTML = "");
    const ov = m.overrides.find((v) => v.opening === o.id);
    box.innerHTML = `<h2>Åpningen</h2><p class="muted">${esc(m.name)} · ${esc(o.id)} · modulen vil ha «${esc(o.kind)}»</p>
      ${field("Slag", `<select id="op-kind" class="fs-select">${options(palette.openings, ov?.kind || "auto")}</select>`)}
      ${ov?.id ? `<p class="muted">Id: <code>${esc(ov.id)}</code> (en spak kan kobles hit med K)</p>` : ""}
      <p class="muted">Automatisk: dør mellom rom, gangåpning mot en gang, mur der ingenting møter.</p>`;
    box.querySelector("#op-kind").onchange = (e) => send({ op: "opening", module: m.id, opening: o.id, kind: e.target.value });
    return;
  }
  if (st.sel.length > 1) {
    box.innerHTML = `<h2>${st.sel.length} valgt</h2><div class="row">
      <button class="fs-button" data-variant="secondary" id="s-turn">Drei (R)</button>
      <button class="fs-button" data-variant="secondary" id="s-mirror">Speil (M)</button>
      <button class="fs-button" data-variant="secondary" id="s-copy">Kopier ved siden av</button>
      <button class="fs-button" data-variant="danger" id="s-del">Slett</button></div>`;
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
  showDoc(box);
}

function showDoc(box) {
  const h = docHead();
  box.innerHTML = `<h2>Nivået</h2>
    ${field("Navnet", `<input id="d-name" class="fs-input" value="${esc(h.name)}">`)}
    <div class="row">
      ${field("Nummer", `<select id="d-level" class="fs-select">${options([...Array.from({ length: 10 }, (_, k) => [k + 1, String(k + 1)]), ...(typeof h.level === "string" ? [[h.level, h.level]] : [])], h.level)}</select>`)}
      ${field("Tema", `<select id="d-theme" class="fs-select">${options([["", "Som modulene"], ...palette.themes.map((t) => [t, t])], h.theme)}</select>`)}
    </div>
    ${field("Steinen i hele nivået", `<select id="d-stone" class="fs-select">${stoneOptions(h.stone)}</select>`)}
    <p class="muted">Velg en modul, en åpning eller en ting på kartet for å endre den. Et nivå merket «klar» erstatter det gamle nivået med samme nummer.</p>`;
  const put = (patch) => send({ op: "set_head", head: { ...docHead(), ...patch } });
  box.querySelector("#d-name").onchange = (e) => put({ name: e.target.value });
  box.querySelector("#d-level").onchange = (e) => put({ level: /^\d+$/.test(e.target.value) ? Number(e.target.value) : e.target.value });
  box.querySelector("#d-theme").onchange = (e) => put({ theme: e.target.value });
  box.querySelector("#d-stone").onchange = (e) => put({ stone: e.target.value || undefined });
}

function showModule(box, m) {
  const pm = palette.modules.find((p) => p.id === m.module);
  box.innerHTML = `<h2>Modulen</h2><p class="muted">${esc(pm?.name || m.module)} · dreid ${m.rot * 90}°${m.mirror ? " · speilet" : ""}</p>
    ${field("Navnet på rommet", `<input id="m-name" class="fs-input" value="${esc(m.name)}">`)}
    ${field("Hører til rommet (samme rom som en annen modul)", `<select id="m-room" class="fs-select">${options([["", "Sitt eget"], ...st.map.modules.filter((x) => x.id !== m.id).map((x) => [x.room || x.id, x.name])], m.room || "")}</select>`)}
    ${field(`Fakler (${Math.round((m.mood?.torches ?? 0.5) * 100)} %)`, `<input id="m-torch" type="range" min="0" max="1" step="0.05" value="${m.mood?.torches ?? 0.5}">`)}
    ${field("Steinen", `<select id="m-stone" class="fs-select">${stoneOptions(m.stone)}</select>`)}
    <div class="row"><button class="fs-button" data-variant="secondary" id="m-turn">Drei (R)</button>
    <button class="fs-button" data-variant="secondary" id="m-mirror">Speil (M)</button>
    <button class="fs-button" data-variant="danger" id="m-del">Slett</button></div>
    <h2>Åpningene</h2><p class="muted">Klikk en pil på kartet for å velge dør, port, falsk vegg, gang, bue, snarvei eller mur.</p>
    <ul class="muted">${m.openings.map((o) => `<li>${esc(o.id)}: ${esc(o.chosen ? palette.openings.find(([k]) => k === o.chosen)?.[1] || o.chosen : "automatisk")}</li>`).join("")}</ul>`;
  const put = (patch) => {
    const d = docModule(m.id);
    if (d) send({ op: "set_module", module: { ...d, ...patch } });
  };
  box.querySelector("#m-name").onchange = (e) => put({ name: e.target.value || undefined });
  box.querySelector("#m-room").onchange = (e) => put({ room: e.target.value || undefined });
  box.querySelector("#m-torch").onchange = (e) => put({ mood: { torches: Number(e.target.value) } });
  box.querySelector("#m-stone").onchange = (e) => put({ stone: e.target.value || undefined });
  box.querySelector("#m-turn").onclick = () => send({ op: "turn", ids: [m.id] });
  box.querySelector("#m-mirror").onclick = () => send({ op: "mirror", ids: [m.id] });
  box.querySelector("#m-del").onclick = () => send({ op: "delete", ids: [m.id] });
}

function showThing(box, t) {
  const name = palette.things.find(([k]) => k === t.kind)?.[1] || t.kind;
  if (!t.own) {
    box.innerHTML = `<h2>${esc(name)}</h2><p class="muted">Fra modulen ${esc(t.module || "")} (${esc(t.id)}). Det følger modulen; endre modulen eller sett ut ditt eget.</p>`;
    return;
  }
  const d = docThing(t.id) || {};
  const put = (patch) => send({ op: "set_thing", thing: { ...docThing(t.id), ...patch } });
  let html = `<h2>${esc(name)}</h2><p class="muted">${esc(t.id)} · (${t.pos.map((v) => v.toFixed(1)).join(", ")}) · ${Math.round(d.yaw || 0)}°</p>`;
  const mob = ["mob", "elite", "boss"].includes(t.kind);
  if (mob) {
    html += field("Slaget", `<select id="t-label" class="fs-select">${options(palette.mobs, d.label || "button")}</select>`);
    html += `<div class="row">${field("Antall", `<input id="t-n" class="fs-input" type="number" min="1" max="8" value="${d.n || 1}">`)}
      ${field("Bakhold", `<select id="t-amb" class="fs-select">${options([["", "Nei (patruljerer eller står)"], ["4", "Innen 4 m"], ["8", "Innen 8 m"], ["rom", "Når du kommer inn i rommet"]], ambushValue(d.ambush))}</select>`)}</div>`;
    html += `<div class="row"><button class="fs-button" data-variant="secondary" id="t-patrol">Patrulje: klikk punkter</button><button class="fs-button" data-variant="secondary" id="t-clear">Tøm patruljen (${(d.patrol || []).length})</button></div>`;
  }
  if (t.kind === "chest") html += field("Det kista gir", `<select id="t-loot" class="fs-select">${options([["", "Fra nivåets liste (tilfeldig)"], ...palette.loot.map((c) => [c, c])], d.loot || "")}</select>`);
  if (t.kind === "rare") html += field("Den sjeldne komponenten", `<select id="t-label" class="fs-select">${options(palette.loot.map((c) => [c, c]), d.label || "")}</select>`);
  if (t.kind === "lore") {
    html += field("Tittelen", `<input id="t-title" class="fs-input" value="${esc(d.label || "")}">`);
    html += field("Teksten (ingen moderne ord: kanon gjelder)", `<textarea id="t-text" class="fs-textarea" rows="7">${esc(d.text || "")}</textarea>`);
  }
  if (t.kind === "bonus") html += field("Bonusen", `<select id="t-label" class="fs-select">${options([["liv", "Liv"], ["utholdenhet", "Utholdenhet"]], d.label || "liv")}</select>`);
  if (t.kind === "trap") html += field("Fella", `<select id="t-label" class="fs-select">${options([["arrows", "Piler"], ["spikes", "Pigger"], ["rocks", "Steiner"], ["pendulum", "Pendel"]], d.label || "arrows")}</select>`);
  if (t.kind === "hint") html += field("Hintet", `<select id="t-label" class="fs-select">${options([["crack", "Sprekk"], ["draft", "Trekk"], ["sound", "Lyd"]], d.label || "crack")}</select>`);
  const links = st.map.links.filter((l) => l.from === t.id || l.to === t.id);
  if (["lever", "elite", "hint"].includes(t.kind) || links.length) {
    html += `<h2>Koblingene</h2><ul class="muted">${links.map((l, k) => `<li>${esc(l.from)} → ${esc(l.to)} (${esc(palette.links.find(([x]) => x === l.kind)?.[1] || l.kind)}) ${l.own ? `<button class="chip" data-unlink="${k}">fjern</button>` : ""}</li>`).join("") || "<li>Ingen</li>"}</ul>
      <button class="fs-button" data-variant="secondary" id="t-link">Koble (K): klikk så på målet</button>`;
  }
  html += `<div class="row" style="margin-top:8px"><button class="fs-button" data-variant="secondary" id="t-turn">Drei (R)</button><button class="fs-button" data-variant="danger" id="t-del">Slett</button></div>`;
  box.innerHTML = html;
  const on = (id, ev, fn) => box.querySelector(id)?.addEventListener(ev, fn);
  on("#t-label", "change", (e) => put({ label: e.target.value }));
  on("#t-n", "change", (e) => put({ n: Math.max(1, Number(e.target.value) || 1) }));
  on("#t-amb", "change", (e) => put({ ambush: e.target.value === "" ? undefined : e.target.value === "rom" ? roomAt(t.pos) || 4 : Number(e.target.value) }));
  on("#t-patrol", "click", () => {
    mode = "patrol";
    setStatus("Klikk punktene på kartet. Esc er ferdig.");
  });
  on("#t-clear", "click", () => put({ patrol: [] }));
  on("#t-loot", "change", (e) => put({ loot: e.target.value || undefined }));
  on("#t-title", "change", (e) => put({ label: e.target.value }));
  on("#t-text", "change", (e) => put({ text: e.target.value }));
  on("#t-link", "click", startLink);
  on("#t-turn", "click", () => turnSelection(false));
  on("#t-del", "click", () => send({ op: "delete", ids: [t.id] }));
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

function showIssues() {
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
    if (i.at) {
      li.onclick = () => {
        const [x, y, z] = i.at;
        const f = st.map.floors.reduce((a, b) => (Math.abs(b - y) < Math.abs(a - y) ? b : a), st.floor);
        setFloor(f);
        st.view.cx = x;
        st.view.cz = z;
        st.flash = i.at;
        setTimeout(() => {
          st.flash = null;
          redraw();
        }, 1500);
        const h = hit(st.map, f, x, z, 1);
        if (h?.type === "module" || h?.type === "thing") select([h.id]);
        engine.level(JSON.stringify({ cmd: "place", x, y: y + 4, z: z + 6, yaw: 0, pitch: -30 }));
        redraw();
      };
    }
    list.append(li);
  }
}

function showStats() {
  const s = st.map.stats;
  $("stats").textContent = `${st.map.modules.length} moduler · ${s.cells} celler (${s.recomputed} regnet nå)\n${s.instances} biter · ${s.colliders} kollidere · ${s.polys} polygoner · ${s.rooms} rom · ${s.lights} lys`;
  $("stats").style.whiteSpace = "pre-line";
}

// Fargene for åpningene i forklaringen (brukes av kartet).
void CHOSEN_COLOR;

boot().catch((e) => {
  console.error(e);
  progress(`Noe gikk galt: ${e.message || e}`, 0);
});
