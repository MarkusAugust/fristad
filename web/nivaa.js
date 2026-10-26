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
import { draw, hit, makeView, miniMap, placeFootprint, snapPlacement, CELL, CHOSEN_COLOR } from "./nivaa-plan.js";
import { commit, token, setToken, branchFiles, branchFile, blobSha, SAVE_BRANCH } from "./github.js";
import { makeZip } from "./editor.js";

const $ = (id) => document.getElementById(id);
const plan = $("plan");
let canvas = $("world");
const params = new URLSearchParams(location.search);
const DRAFT_PREFIX = "fristad.nivaa.utkast:";
const LAST = "fristad.nivaa.siste";
// Malene eieren har lagret i denne nettleseren (navn → tekst), og de som
// ikke er lagret på grenen ennå.
const TEMPLATES = "fristad.nivaa.maler";
// Eierens egne moduler (Ctrl+Shift+G): id → { path, text, pending }.
const MODULES = "fristad.nivaa.moduler";
const MARKS = "fristad.nivaa.kamera:";
// Blob-sha-en til fila redigeringen begynte fra (repoet, grenen eller sist
// lagret), per sti: konfliktsjekken før lagring.
const BASE = "fristad.nivaa.grunnlag:";
const HELP_SEEN = "fristad.nivaa.hjelp";

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
  // Etasjene eieren har lagt til, og snittet (bare denne og den under).
  extraFloors: [],
  floors: [0],
  // Snittet står på fra start: i 3D ser du rommene ovenfra som et dukkehus.
  cut: true,
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

// Det redigeringen og prøveturen ikke trenger: byfolket (prøveturen går
// rett ned i nivået; en innbygger uten figur er bare usynlig).
const SKIP_MODELS = ["npcs/"];

async function loadModels() {
  const all = model_paths().map((p) => (SKIP_MODELS.some((s) => p.startsWith(s)) ? null : p));
  const out = new Array(all.length);
  let next = 0;
  let done = 0;
  const worker = async () => {
    while (next < all.length) {
      const i = next++;
      out[i] = all[i] ? await fetchModel(all[i]).catch(() => undefined) : undefined;
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
  if (text) {
    if (!store.get(BASE + path)) rememberBase(path, text);
    return { text, note: "" };
  }
  return { text: level_blank("Nytt nivå", 1, "grav"), note: "Nytt nivå." };
}

async function boot() {
  progress("Laster motoren …", 0);
  await init();
  palette = JSON.parse(level_palette());
  buildPalette();
  loadTemplates();
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
  loadOwnModules();
  window.fristad = { engine, level: send, state: st, setFloor, place: placeModule, select, blank: level_blank, saveModule };
  engine.set_quality(engine.backend() === "Gl" ? "enkel" : "standard");
  fit();
  new ResizeObserver(fit).observe(canvas);
  new ResizeObserver(() => redraw()).observe(plan);
  refresh(JSON.parse(engine.level_map()));
  centerOn();
  bindUi();
  bindMap();
  bind3d();
  sendCut();
  frameFloor();
  macKeys(document.body);
  $("overlay").hidden = true;
  if (first.note) toast(first.note);
  // Første gang: hjelpen med de fire stegene.
  if (!store.get(HELP_SEEN)) {
    store.set(HELP_SEEN, "1");
    showHelp(true);
  }
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
  // Etasjene: gulvene som finnes, og de nye eieren har lagt til (tomme til
  // noe står der). En ny etasje står 6 m over eller under (`docs/nivaaredigering.md` del 25).
  const floors = [...new Set([...(map.floors.length ? map.floors : [0]), ...st.extraFloors])].sort((a, b) => a - b);
  const sel = $("floor");
  const want = floors.map(String).join();
  if (sel.dataset.list !== want) {
    const top = Math.max(...floors);
    const low = Math.min(...floors);
    sel.innerHTML =
      floors.map((f) => `<option value="${f}">${f === 0 ? "Hovedplanet (0 m)" : `${f > 0 ? "+" : ""}${f} m`}${map.floors.includes(f) ? "" : " (tom)"}</option>`).join("") +
      `<option value="new:${top + 6}">Ny etasje over (+${top + 6} m)</option><option value="new:${low - 6}">Ny etasje under (${low - 6} m)</option>`;
    sel.dataset.list = want;
  }
  if (!floors.includes(st.floor)) st.floor = floors.reduce((a, f) => (Math.abs(f - st.floor) < Math.abs(a - st.floor) ? f : a), floors[0]);
  sel.value = String(st.floor);
  st.floors = floors;
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
    const url = URL.createObjectURL(makeZip([d, ...pendingTemplates(), ...pendingModules()]));
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
    const conflict = await checkConflict(d);
    if (conflict === "avbryt") return setStatus("");
    const tpls = pendingTemplates();
    const mods = pendingModules();
    const r = await commit([d, ...tpls, ...mods], `Nivåredigeringen: ${st.map.name || d.path}`);
    clearPendingTemplates();
    clearPendingModules();
    rememberBase(d.path, d.text);
    send({ cmd: "saved" });
    toast(r.unchanged ? "Ingenting nytt å lagre." : `Lagret på grenen ${SAVE_BRANCH}.`);
  } catch (e) {
    toast(`Lagringen feilet: ${e.message}. Utkastet ligger i nettleseren.`, 7000);
  }
  setStatus("");
}

/// Husk hva redigeringen av `path` bygger på (teksten fra repoet, grenen
/// eller det som ble lagret).
async function rememberBase(path, text) {
  try {
    store.set(BASE + path, await blobSha(text));
  } catch {
    /* uten crypto.subtle (http): ingen konfliktsjekk */
  }
}

/// Før lagring: har fila på grenen endret seg siden redigeringen begynte
/// (lagret fra en annen fane eller maskin)? Da velger eieren å skrive over,
/// åpne den som ligger der, eller avbryte. Gir "ok", "over" eller "avbryt".
async function checkConflict(d) {
  const there = await branchFile(d.path).catch(() => null);
  if (!there) return "ok";
  const mine = await blobSha(d.text).catch(() => "");
  const base = store.get(BASE + d.path);
  if (there.sha === mine || there.sha === base) return "ok";
  return new Promise((done) => {
    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML = `<h2 class="lt-heading" data-size="s">Fila er endret på grenen</h2>
      <p class="lt-paragraph">${esc(d.path)} på grenen ${SAVE_BRANCH} er lagret et annet sted siden du begynte her (en annen fane eller maskin). Lagrer du, skrives den over.</p>
      <div class="row"><button class="lt-button" data-variant="danger" id="c-over">Skriv over</button>
      <button class="lt-button" data-variant="secondary" id="c-open">Åpne den på grenen</button>
      <button class="lt-button" data-variant="secondary" id="c-cancel">Avbryt</button></div>`;
    const ov = $("overlay");
    ov.replaceChildren(card);
    ov.hidden = false;
    const close = (v) => {
      ov.hidden = true;
      done(v);
    };
    card.querySelector("#c-over").onclick = () => close("over");
    card.querySelector("#c-cancel").onclick = () => close("avbryt");
    card.querySelector("#c-open").onclick = () => {
      openText(there.text, `Åpnet ${d.path} fra grenen ${SAVE_BRANCH}; utkastet ditt ligger i nettleseren.`);
      rememberBase(d.path, there.text);
      close("avbryt");
    };
  });
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
    <h2 class="lt-heading" data-size="s">Åpne et nivå</h2>
    <p class="muted">Utkastene i denne nettleseren og nivåene i repoet. Et nivå merket «klar» erstatter det gamle nivået med samme nummer.</p>
    <div class="doclist">${drafts.map((p) => `<button class="lt-button" data-variant="secondary" data-draft="${p}">Utkast: ${p.replace("assets/levels/", "")}</button>`).join("")}
    ${names.map((n) => `<button class="lt-button" data-variant="secondary" data-repo="${n}">I repoet: nivaa_${n}.json</button>`).join("")}
    <button class="lt-button" data-variant="secondary" data-branch="1">Hent dette nivået fra grenen ${SAVE_BRANCH}</button></div>
    <h2 class="lt-heading" data-size="s">Nytt nivå</h2>
    <div class="row" style="align-items:flex-end">
      ${field("Navnet", `<input id="new-name" class="lt-input" value="Nytt nivå">`).replace('class="field"', 'class="field" style="flex:1"')}
      ${field("Nivå", `<select id="new-n" class="lt-select">${[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => `<option>${n}</option>`).join("")}</select>`)}
      ${field("Temaet", `<select id="new-theme" class="lt-select">${(palette.themes.length ? palette.themes : ["grav"]).map((t) => `<option value="${t}">${THEME_NAMES[t] || t}</option>`).join("")}</select>`)}
      <button id="new-make" class="lt-button" data-variant="primary" style="margin-bottom:8px">Lag</button>
    </div>
    <p class="muted">Det nye nivået begynner med kapellet med døra opp og startstedet. Dra rom og ganger fra paletten inn til åpningene.</p>
    <div class="row" style="margin-top:12px"><button id="open-cancel" class="lt-button" data-variant="secondary">Avbryt</button></div>`;
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
      const text = level_doc_source(b.dataset.repo);
      openText(text, `nivaa_${b.dataset.repo}.json fra repoet.`);
      rememberBase(`assets/levels/nivaa_${b.dataset.repo}.json`, text);
      close();
    }),
  );
  card.querySelector("[data-branch]").onclick = async () => {
    try {
      const files = await branchFiles([docPath]);
      if (!files.length) toast(token() ? `${docPath} finnes ikke på grenen ${SAVE_BRANCH}.` : "Trenger nøkkelen til GitHub (Lagre spør etter den).");
      else {
        openText(files[0].text, `Hentet ${docPath} fra grenen ${SAVE_BRANCH}.`);
        rememberBase(docPath, files[0].text);
      }
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
  frameFloor();
  if (note) toast(note);
}

// ---------------------------------------------------------------- maler

function localTemplates() {
  try {
    return JSON.parse(store.get(TEMPLATES) || "{}");
  } catch {
    return {};
  }
}

function slug(name) {
  return (
    name
      .toLowerCase()
      .replace(/æ/g, "ae")
      .replace(/ø/g, "o")
      .replace(/å/g, "aa")
      .replace(/[^a-z0-9]+/g, "_")
      .replace(/^_|_$/g, "") || "mal"
  );
}

/// Malene som ikke er lagret på grenen ennå, som filer.
function pendingTemplates() {
  return Object.entries(localTemplates())
    .filter(([, t]) => t.pending)
    .map(([key, t]) => ({ path: `assets/levels/maler/${key}.json`, text: t.text }));
}

function clearPendingTemplates() {
  const all = localTemplates();
  for (const t of Object.values(all)) delete t.pending;
  store.set(TEMPLATES, JSON.stringify(all));
}

/// Ctrl+G: det valgte som mal, husket i nettleseren og lagret på grenen
/// neste gang du lagrer (`assets/levels/maler/<navn>.json`).
function saveTemplate() {
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

// ---------------------------------------------------------------- eierens moduler

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
function saveModule() {
  if (!st.sel.some((id) => st.map.modules.some((m) => m.id === id))) return toast("Velg modulene som skal bli en ny modul først.");
  const name = prompt("Navnet på modulen", "Ny modul");
  if (!name) return;
  const out = JSON.parse(engine.level_module(JSON.stringify(st.sel), name, slug(name)));
  if (out.error) return toast(out.error, 6000);
  const all = localModules();
  all[out.id] = { path: out.path, text: out.text, pending: true };
  store.set(MODULES, JSON.stringify(all));
  palette = JSON.parse(out.palette);
  buildPalette();
  refresh(JSON.parse(engine.level_map()));
  armModule(out.id);
  toast(`Modulen «${name}» (${out.id}) står i paletten. Bildet mangler til module_shots.py har tatt det; Lagre legger fila på grenen.`, 7000);
  return out;
}

/// Modulene eieren har laget før, inn i byggesettet (de som ikke er i repoet ennå).
function loadOwnModules() {
  let changed = false;
  for (const [id, m] of Object.entries(localModules())) {
    if (palette.modules.some((p) => p.id === id) && !m.pending) continue;
    const out = JSON.parse(engine.level_add_module(m.text));
    if (out.error) console.warn(id, out.error);
    else {
      palette = out;
      changed = true;
    }
  }
  if (changed) buildPalette();
}

function pendingModules() {
  return Object.values(localModules())
    .filter((m) => m.pending)
    .map((m) => ({ path: m.path, text: m.text }));
}

function clearPendingModules() {
  const all = localModules();
  for (const m of Object.values(all)) delete m.pending;
  store.set(MODULES, JSON.stringify(all));
}

let repoTemplates = {};
async function loadTemplates() {
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
function templateGhost(x, z) {
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

// ---------------------------------------------------------------- bokmerker

function marks() {
  try {
    return JSON.parse(store.get(MARKS + docPath) || "{}");
  } catch {
    return {};
  }
}

/// Ctrl+1–9 (Shift+1–9 der nettleseren tar Ctrl): husk kameraet i 3D,
/// utsnittet i kartet og etasjen.
function markCamera(n) {
  const all = marks();
  all[n] = { camera: engine.level_camera(), view: [st.view.cx, st.view.cz, st.view.s], floor: st.floor };
  store.set(MARKS + docPath, JSON.stringify(all));
  toast(`Bokmerke ${n} husket.`, 1500);
}

/// 1–9: tilbake til bokmerket.
function gotoMark(n) {
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

// ---------------------------------------------------------------- paletten

function buildPalette() {
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
        drag = { type: "palette" };
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
  // Lys du setter selv (dra dem etterpå; farge, styrke og flamme i panelet).
  things.append(chip("Lys", armLight, "light"));
}

function armLight() {
  arm = { type: "light" };
  mode = null;
  markArmed();
  setStatus("Klikk på kartet for å sette ut et lys. Esc avbryter.");
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
  setStatus("Slipp den nær en ledig åpning, så snapper den på plass · R dreier · M speiler · Alt fritt · Esc avbryter");
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
    const on = arm && ((arm.type === "module" && el.dataset.module === arm.id) || (arm.type === "light" && el.dataset.key === "light") || (arm.type === "template" && el.dataset.key === `tpl:${arm.key}`) || (arm.type === "thing" && (el.dataset.key === arm.kind || el.dataset.key === `mob:${arm.label}`) && (arm.kind !== "mob" || el.dataset.key === `mob:${arm.label}`)));
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

// Alt holdt nede: ingen magnet (modulen står der pekeren er).
let altDown = false;

function ghostFor(x, z) {
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
    $("opmenu").hidden = true;
    plan.focus();
    plan.setPointerCapture(e.pointerId);
    const [x, z] = worldAt(e);
    lastMouse = [e.clientX, e.clientY];
    if (e.button === 2 || e.button === 1 || space) {
      drag = { type: "pan", button: e.button, at: [e.clientX, e.clientY], moved: false };
      return;
    }
    if (e.button !== 0) return;
    const mini = miniMap(st.map, plan.clientWidth, plan.clientHeight, st.view);
    const pr = plan.getBoundingClientRect();
    if (mini?.inside(e.clientX - pr.left, e.clientY - pr.top)) {
      [st.view.cx, st.view.cz] = mini.toWorld(e.clientX - pr.left, e.clientY - pr.top);
      drag = { type: "mini" };
      redraw();
      return;
    }
    if (mode === "patrol") return addPatrol(x, z);
    if (mode === "link") return finishLink(x, z);
    if (arm) return;
    const h = hit(st.map, st.floor, x, z, Math.max(0.6, 10 / st.view.s));
    if (h?.type === "thing") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      drag = { type: "thing", id: h.id, own: h.own, from: [x, z], moved: false };
      return;
    }
    if (h?.type === "light") {
      if (!st.sel.includes(h.id) || e.shiftKey) select([h.id], e.shiftKey);
      drag = { type: "light", id: h.id, from: [x, z], moved: false };
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
      drag = { type: "modules", from: cellAt(x, z), by: [0, 0, 0] };
      return;
    }
    if (!e.shiftKey) select([]);
    const r = plan.getBoundingClientRect();
    drag = { type: "box", a: [e.clientX - r.left, e.clientY - r.top], add: e.shiftKey };
  });
  plan.addEventListener("pointermove", (e) => {
    altDown = e.altKey;
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
    drag = null;
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
    drag = null;
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
    altDown = e.altKey;
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
    if (t && ["mob", "elite", "boss"].includes(t.kind) && t.own) items.push(["Patrulje: klikk punktene", () => ((mode = "patrol"), setStatus("Klikk punktene på kartet · Esc er ferdig"))]);
    items.push(["Vis i 3D (F)", focusSelection], ["Slett (Delete)", () => send({ op: "delete", ids: st.sel })]);
  } else {
    items.push(["Vis hele etasjen (Home)", () => (centerOn(), frameFloor())], ["Gå inn her i 3D", () => engine.level(JSON.stringify({ cmd: "place", x, y: st.floor + 1.7, z, yaw: st.camera?.length ? st.camera[3] : 0, pitch: -5 }))]);
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
    const extra = mob ? `${mob}${t.n > 1 ? ` × ${t.n}` : ""}${t.ambush ? " · i bakhold" : t.patrol.length ? " · patruljerer" : ""}` : t.kind === "lore" ? t.label || "uten tittel" : t.kind === "chest" ? t.loot || "tilfeldig fra nivået" : "";
    html = `<b>${esc(kind)}</b>${extra ? ` · ${esc(extra)}` : ""}<div class="muted">${t.own ? "Dra for å flytte · R dreier · Delete sletter" : `Fra modulen ${esc(st.map.modules.find((m) => m.id === t.module)?.name || t.module || "")} · dra for å flytte her`}</div>`;
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
  setStatus("Klikk på målet: en port, en bro … eller en åpning (den blir en port) · Esc avbryter");
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
  sendCut();
  redraw();
}

/// Snittet i 3D (S): alt over etasjen kartet viser kuttes bort
/// (`LevelCommand::Cut`, `fs_clip`), så du ser inn i den under en annen.
function sendCut() {
  engine.level(JSON.stringify({ cmd: "cut", y: st.cut ? st.floor : null }));
}

// ---------------------------------------------------------------- 3D

let looking = false;
// Banekameraet i 3D: kameraet går rundt et punkt (`pivot`) i en avstand og
// retning (grader; pitch negativ ser ned). Venstre dra går rundt, hjulet
// zoomer, midtknappen flytter, høyre knapp + WASD flyr som før.
const orbit = { pivot: null, dist: 18, yaw: 35, pitch: -45 };

function camForward(yaw, pitch) {
  const a = (yaw * Math.PI) / 180;
  const p = (pitch * Math.PI) / 180;
  return [-Math.sin(a) * Math.cos(p), Math.sin(p), -Math.cos(a) * Math.cos(p)];
}

function applyOrbit() {
  if (!orbit.pivot) return;
  const f = camForward(orbit.yaw, orbit.pitch);
  const [x, y, z] = orbit.pivot;
  engine.level(JSON.stringify({ cmd: "place", x: x - f[0] * orbit.dist, y: y - f[1] * orbit.dist, z: z - f[2] * orbit.dist, yaw: orbit.yaw, pitch: orbit.pitch }));
  st.camera = engine.level_camera();
}

/// Punktet kameraet går rundt, fra kameraet slik det står nå (etter flyging
/// eller et hopp i kartet): gulvet midt i bildet, ellers et stykke fram.
function syncOrbit() {
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
function frameFloor() {
  const cells = st.map.modules.flatMap((m) => m.cells).filter((c) => c[1] >= st.floor - 0.5 && c[1] < st.floor + 2.5);
  frameCells(cells.length ? cells : st.map.modules.flatMap((m) => m.cells), false);
}

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
function placeIn3d(e) {
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

function bind3d() {
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
  $("floor").onchange = () => {
    const v = $("floor").value;
    if (v.startsWith("new:")) {
      const y = Number(v.slice(4));
      st.extraFloors.push(y);
      $("floor").dataset.list = "";
      refresh(st.map);
      setFloor(y);
      toast(`Etasjen ${y > 0 ? "+" : ""}${y} m: sett ut moduler her, og koble den til med en trapp, stige eller heis. Etasjen under står gjennomsiktig.`, 6000);
    } else setFloor(Number(v));
  };
  $("cut").onclick = () => {
    st.cut = !st.cut;
    $("cut").setAttribute("aria-pressed", st.cut);
    sendCut();
    redraw();
  };
  $("helpbtn").onclick = () => showHelp(true);
  $("help-close").onclick = () => showHelp(false);
  $("help").onclick = (e) => e.target === $("help") && showHelp(false);
  $("v-map").onclick = () => setView("map");
  $("v-split").onclick = () => setView("split");
  $("v-3d").onclick = () => setView("3d");
  $("light").onclick = () => {
    // Knappen viser lyset som er på: arbeidslyset (alt synes) eller
    // spillets (faklene og lysene som følger rommene, tåka).
    const on = $("light").dataset.mode === "game";
    $("light").dataset.mode = on ? "work" : "game";
    $("light").textContent = on ? "Lys: arbeid" : "Lys: spillet";
    engine.level(JSON.stringify({ cmd: "light", work: on }));
  };
  addEventListener("keydown", onKey);
  addEventListener("keyup", (e) => {
    if (playing || (document.activeElement === canvas && FLY_KEYS.includes(e.code))) engine.key(e.code, false);
  });
  addEventListener("blur", () => engine?.release_all());
}

// På Mac står ⌘ der det står Ctrl (begge virker), og ⌥ for Alt.
const MAC = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

function macKeys(root) {
  if (!MAC) return;
  const walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let n = walk.nextNode(); n; n = walk.nextNode()) n.nodeValue = n.nodeValue.replace(/Ctrl\+/g, "⌘").replace(/\bAlt\b/g, "⌥ Alt");
  root.querySelectorAll("[title]").forEach((el) => (el.title = el.title.replace(/Ctrl\+/g, "⌘").replace(/\bAlt\b/g, "⌥ Alt")));
}

function showHelp(on) {
  $("help").hidden = !on;
  if (on) $("help-close").focus();
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
  if (!$("help").hidden) {
    if (e.code === "Escape" || e.code === "F1" || e.key === "?") {
      e.preventDefault();
      showHelp(false);
    }
    return;
  }
  if (e.code === "F1" || (e.key === "?" && !typing())) {
    e.preventDefault();
    return showHelp(true);
  }
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
  if (ctrl && e.code === "KeyG") {
    e.preventDefault();
    return e.shiftKey ? saveModule() : saveTemplate();
  }
  const digit = /^Digit([1-9])$/.exec(e.code)?.[1];
  if (digit && (ctrl || e.shiftKey)) {
    e.preventDefault();
    return markCamera(digit);
  }
  if (digit && !e.altKey) return gotoMark(digit);
  if (ctrl && e.code === "KeyA") {
    // Alt i etasjen: modulene og innholdet ditt.
    e.preventDefault();
    const on = (y) => y >= st.floor - 0.5 && y < st.floor + 2.5;
    return select([...st.map.modules.filter((m) => m.cells.some((c) => on(c[1]))).map((m) => m.id), ...st.map.things.filter((t) => t.own && on(t.pos[1])).map((t) => t.id)]);
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
      $("opmenu").hidden = true;
      disarm();
      select([]);
      break;
    case "KeyR":
      if (arm?.type === "module" || arm?.type === "template") {
        // Fra dreiningen magneten valgte; etter R dreies den ikke av seg selv.
        arm.rot = ((st.ghost?.rot ?? arm.rot) + (e.shiftKey ? 3 : 1)) & 3;
        arm.turned = true;
        if (st.mouse) st.ghost = arm.type === "module" ? ghostFor(...st.view.toWorld(...st.mouse)) : templateGhost(...st.view.toWorld(...st.mouse));
        redraw();
      } else if (arm?.type === "thing") arm.yaw = (arm.yaw + 90) % 360;
      else if (st.sel.length) turnSelection(e.shiftKey);
      break;
    case "KeyM":
    case "KeyX":
      if (arm?.type === "module" || arm?.type === "template") {
        arm.mirror = !arm.mirror;
        if (st.mouse) st.ghost = arm.type === "module" ? ghostFor(...st.view.toWorld(...st.mouse)) : templateGhost(...st.view.toWorld(...st.mouse));
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
    case "KeyS":
      if (!ctrl) $("cut").click();
      break;
    case "KeyP":
      play(e.shiftKey);
      break;
    // Etasjen: PageUp/PageDown, eller Shift+↑/↓ (tastaturer uten dem).
    case "PageUp":
    case "PageDown":
    case "ArrowUp":
    case "ArrowDown": {
      if (e.code.startsWith("Arrow") && !e.shiftKey) break;
      e.preventDefault();
      const f = st.floors || st.map.floors;
      const i = f.indexOf(st.floor) + (e.code === "PageUp" || e.code === "ArrowUp" ? 1 : -1);
      if (i >= 0 && i < f.length) setFloor(f[i]);
      break;
    }
    case "Tab":
      e.preventDefault();
      setView({ split: "map", map: "3d", "3d": "split" }[viewMode]);
      break;
    // Hele etasjen: Home eller H.
    case "Home":
    case "KeyH":
      centerOn();
      frameFloor();
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
  const mods = st.map.modules.filter((m) => st.sel.includes(m.id));
  const things = st.map.things.filter((t) => st.sel.includes(t.id));
  const cells = [...mods.flatMap((m) => m.cells), ...things.map((t) => [Math.round(t.pos[0] / CELL), t.pos[1], Math.round(t.pos[2] / CELL)])];
  if (!cells.length) return frameFloor();
  st.view.cx = (Math.min(...cells.map((c) => c[0])) + Math.max(...cells.map((c) => c[0]))) * CELL / 2;
  st.view.cz = (Math.min(...cells.map((c) => c[2])) + Math.max(...cells.map((c) => c[2]))) * CELL / 2;
  frameCells(cells);
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
  $("overlay").replaceChildren(Object.assign(document.createElement("div"), { className: "card", innerHTML: "<p class='lt-paragraph'>Bygger verden med nivået …</p>" }));
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

function docLight(id) {
  return docJson().lights?.find((l) => l.id === id) || null;
}

function docModule(id) {
  return docJson().modules?.find((m) => m.id === id) || null;
}

function esc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

function field(label, html) {
  return `<div class="field"><span class="lt-label">${label}</span>${html}</div>`;
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
      ${field("Nummer", `<select id="d-level" class="lt-select">${options([...Array.from({ length: 10 }, (_, k) => [k + 1, String(k + 1)]), ...(typeof h.level === "string" ? [[h.level, h.level]] : [])], h.level)}</select>`)}
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
    ${field("Hører til rommet (samme rom som en annen modul)", `<select id="m-room" class="lt-select">${options([["", "Sitt eget"], ...st.map.modules.filter((x) => x.id !== m.id).map((x) => [x.room || x.id, x.name])], m.room || "")}</select>`)}
    ${field("Steinen", `<select id="m-stone" class="lt-select">${stoneOptions(m.stone)}</select>`)}
    ${moodFields(m.mood)}
    <div class="row"><button class="lt-button" data-variant="secondary" id="m-turn">Drei (R)</button>
    <button class="lt-button" data-variant="secondary" id="m-mirror">Speil (M)</button>
    <button class="lt-button" data-variant="danger" id="m-del">Slett</button></div>
    <h2>Åpningene</h2><p class="muted">Klikk en pil på kartet for å velge dør, port, falsk vegg, gang, bue, snarvei eller mur.</p>
    <ul class="muted">${m.openings.map((o) => `<li>${esc(o.id)}: ${esc(o.chosen ? palette.openings.find(([k]) => k === o.chosen)?.[1] || o.chosen : "automatisk")}</li>`).join("")}</ul>
    ${m.removed?.length ? `<h2>Fjernet fra malen</h2><ul class="muted">${m.removed.map((r) => `<li>${esc(r)} <button class="chip" data-restore="${esc(r)}">sett tilbake</button></li>`).join("")}</ul>` : ""}`;
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
    box.innerHTML = `<h2>${esc(name)}</h2><p class="muted">Fra modulen ${esc(t.module || "")} (${esc(t.id)}). Det følger modulen. Dra det på kartet for å flytte det i dette nivået, eller endre det her; malen er den samme.</p>
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
      ${field("Bakhold", `<select id="t-amb" class="lt-select">${options([["", "Nei (patruljerer eller står)"], ["4", "Innen 4 m"], ["8", "Innen 8 m"], ["rom", "Når du kommer inn i rommet"]], ambushValue(d.ambush))}</select>`)}</div>`;
    html += `<div class="row"><button class="lt-button" data-variant="secondary" id="t-patrol">Patrulje: klikk punkter</button><button class="lt-button" data-variant="secondary" id="t-clear">Tøm patruljen (${(d.patrol || []).length})</button></div>`;
  }
  if (t.kind === "chest") html += field("Det kista gir", `<select id="t-loot" class="lt-select">${options([["", "Fra nivåets liste (tilfeldig)"], ...palette.loot.map((c) => [c, c])], d.loot || "")}</select>`);
  if (t.kind === "rare") html += field("Den sjeldne komponenten", `<select id="t-label" class="lt-select">${options(palette.loot.map((c) => [c, c]), d.label || "")}</select>`);
  if (t.kind === "lore") {
    html += field("Tittelen", `<input id="t-title" class="lt-input" value="${esc(d.label || "")}">`);
    html += field("Teksten (ingen moderne ord: kanon gjelder)", `<textarea id="t-text" class="lt-textarea" rows="7">${esc(d.text || "")}</textarea>`);
  }
  if (t.kind === "bonus") html += field("Bonusen", `<select id="t-label" class="lt-select">${options([["liv", "Liv"], ["utholdenhet", "Utholdenhet"]], d.label || "liv")}</select>`);
  if (t.kind === "trap") html += field("Fella", `<select id="t-label" class="lt-select">${options([["arrows", "Piler"], ["spikes", "Pigger"], ["rocks", "Steiner"], ["pendulum", "Pendel"]], d.label || "arrows")}</select>`);
  if (t.kind === "hint") html += field("Hintet", `<select id="t-label" class="lt-select">${options([["crack", "Sprekk"], ["draft", "Trekk"], ["sound", "Lyd"]], d.label || "crack")}</select>`);
  const links = st.map.links.filter((l) => l.from === t.id || l.to === t.id);
  if (["lever", "elite", "hint"].includes(t.kind) || links.length) {
    html += `<h2>Koblingene</h2><ul class="muted">${links.map((l, k) => `<li>${esc(l.from)} → ${esc(l.to)} (${esc(palette.links.find(([x]) => x === l.kind)?.[1] || l.kind)}) ${l.own ? `<button class="chip" data-unlink="${k}">fjern</button>` : ""}</li>`).join("") || "<li>Ingen</li>"}</ul>
      <button class="lt-button" data-variant="secondary" id="t-link">Koble (K): klikk så på målet</button>`;
  }
  html += `<div class="row" style="margin-top:8px"><button class="lt-button" data-variant="secondary" id="t-turn">Drei (R)</button><button class="lt-button" data-variant="danger" id="t-del">Slett</button>
    ${t.template ? `<button class="lt-button" data-variant="secondary" id="t-back" title="Slik modulmalen har det">Tilbake til malen</button>` : ""}</div>`;
  if (t.template) html = `<p class="muted">Endret i dette nivået; malen i modulen ${esc(t.template)} er den samme.</p>` + html;
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

/// Gå til et sted i kartet og 3D: etasjen, utsnittet, valget og et blink.
function goTo(at, pick = true) {
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
