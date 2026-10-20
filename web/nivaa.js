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
import { draw, hit, makeView, miniMap, placeFootprint, CELL, CHOSEN_COLOR } from "./nivaa-plan.js";
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
const MARKS = "fristad.nivaa.kamera:";
// Blob-sha-en til fila redigeringen begynte fra (repoet, grenen eller sist
// lagret), per sti: konfliktsjekken før lagring.
const BASE = "fristad.nivaa.grunnlag:";

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
  cut: false,
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
  window.fristad = { engine, level: send, state: st, setFloor, place: placeModule, select, blank: level_blank };
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
    const url = URL.createObjectURL(makeZip([d, ...pendingTemplates()]));
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
    const r = await commit([d, ...tpls], `Nivåredigeringen: ${st.map.name || d.path}`);
    clearPendingTemplates();
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
    card.innerHTML = `<h2 class="fs-heading" data-size="s">Fila er endret på grenen</h2>
      <p class="fs-paragraph">${esc(d.path)} på grenen ${SAVE_BRANCH} er lagret et annet sted siden du begynte her (en annen fane eller maskin). Lagrer du, skrives den over.</p>
      <div class="row"><button class="fs-button" data-variant="danger" id="c-over">Skriv over</button>
      <button class="fs-button" data-variant="secondary" id="c-open">Åpne den på grenen</button>
      <button class="fs-button" data-variant="secondary" id="c-cancel">Avbryt</button></div>`;
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
    const mini = miniMap(st.map, plan.clientWidth, plan.clientHeight);
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
    if (arm?.type === "template") st.ghost = templateGhost(x, z);
    if (arm?.type === "thing" || arm?.type === "light") st.thingGhost = [snap(x), st.floor, snap(z)];
    if (drag?.type === "modules") {
      const c = cellAt(x, z);
      drag.by = [c[0] - drag.from[0], 0, c[2] - drag.from[2]];
      // Fantomet: de valgte modulene flyttet.
      const moved = st.map.modules.filter((m) => st.sel.includes(m.id));
      st.ghost = drag.by[0] || drag.by[2] ? { cells: moved.flatMap((m) => m.cells.map((q) => [q[0] + drag.by[0], q[1], q[2] + drag.by[2]])), openings: [], ok: true } : null;
    }
    if (drag?.type === "thing" || drag?.type === "light") {
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
    if (d?.type === "pan" || d?.type === "mini") return;
    if (arm?.type === "module" && (e.button === 0 || d?.type === "palette")) {
      const g = ghostFor(x, z);
      if (g) placeModule(arm.id, g.at, arm.rot, arm.mirror);
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
      if (d.by[0] || d.by[2]) send({ op: "move", ids: st.sel, by: d.by });
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
    redraw();
  };
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
  if (ctrl && e.code === "KeyG") {
    e.preventDefault();
    return saveTemplate();
  }
  const digit = /^Digit([1-9])$/.exec(e.code)?.[1];
  if (digit && (ctrl || e.shiftKey)) {
    e.preventDefault();
    return markCamera(digit);
  }
  if (digit && !e.altKey) return gotoMark(digit);
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
      if (arm?.type === "module" || arm?.type === "template") {
        arm.rot = (arm.rot + (e.shiftKey ? 3 : 1)) & 3;
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
    case "PageUp":
    case "PageDown": {
      e.preventDefault();
      const f = st.floors || st.map.floors;
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
  engine.level(JSON.stringify({ cmd: "place", x: p[0], y: p[1] + 1.7, z: p[2], yaw: st.camera?.length ? st.camera[3] : 0, pitch: -10 }));
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
  const ll = id && map.lights.find((l) => l.id === id && l.own);
  if (ll) return showLight(box, ll);
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
    ${field("Steinen", `<select id="m-stone" class="fs-select">${stoneOptions(m.stone)}</select>`)}
    ${moodFields(m.mood)}
    <div class="row"><button class="fs-button" data-variant="secondary" id="m-turn">Drei (R)</button>
    <button class="fs-button" data-variant="secondary" id="m-mirror">Speil (M)</button>
    <button class="fs-button" data-variant="danger" id="m-del">Slett</button></div>
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
    <div class="row">${field("Fargen på lysene", `<select id="mood-color" class="fs-select">${options([...LIGHT_COLORS.map(([k, n]) => [k, n]), ["egen", "Egen farge"]], preset)}</select>`)}
      <input id="mood-pick" type="color" value="${colorHex(m.color)}" title="Egen farge" aria-label="Egen farge"></div>
    ${field(`Tåke (${Math.round((m.fog ?? 0) * 100)} %)`, `<input id="mood-fog" type="range" min="0" max="1" step="0.05" value="${m.fog ?? 0}">`)}
    ${field("Lyden", `<select id="mood-sound" class="fs-select">${options(palette.sounds || [["", "Drypp"]], m.sound || "")}</select>`)}
    <button class="fs-button" data-variant="secondary" id="mood-reset" type="button">Som malen</button>`;
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
    <div class="row" style="margin-top:8px"><button class="fs-button" data-variant="danger" id="l-del">Slett</button></div>`;
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
      <div class="row"><button class="fs-button" data-variant="secondary" id="t-detach">Endre her</button>
      <button class="fs-button" data-variant="danger" id="t-del">Fjern fra modulen</button></div>`;
    box.querySelector("#t-detach").onclick = () => send({ op: "detach", id: t.id }) && showSelection();
    box.querySelector("#t-del").onclick = () => send({ op: "delete", ids: [t.id] });
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
  html += `<div class="row" style="margin-top:8px"><button class="fs-button" data-variant="secondary" id="t-turn">Drei (R)</button><button class="fs-button" data-variant="danger" id="t-del">Slett</button>
    ${t.template ? `<button class="fs-button" data-variant="secondary" id="t-back" title="Slik modulmalen har det">Tilbake til malen</button>` : ""}</div>`;
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
  engine.level(JSON.stringify({ cmd: "place", x, y: y + 1.7, z, yaw: st.camera?.length ? st.camera[3] : 0, pitch: -20 }));
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
