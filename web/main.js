// JavaScript-laget i Fristad: løkka, input, pekerlås, overleggene og lyden.
// Byen, fysikken og grafikken bor i Rust (pkg/fristad.js og .wasm), bak
// den drivende porten.

import init, { start, asset_paths, model_paths, start_paths, lazy_paths, level_paths, model_url } from "./pkg/fristad.js";
import { createMusic } from "./music.js";
import { createAmbience } from "./ambience.js";
import { createFilm } from "./film/player.js";
import { createSounds } from "./sounds.js";
import { createEditor } from "./editor.js";
import { defineLtToast } from "./lotek/components/standalone/toast/lt-toast.js";
import { defineLtTabs } from "./lotek/components/frame/tabs/lt-tabs.js";
import { defineLtDialog } from "./lotek/components/frame/dialog/lt-dialog.js";
import { defineLtConnectionStatus } from "./lotek/components/standalone/connection-status/lt-connection-status.js";

defineLtToast();
// Boka viser eksempelet fra dokumentasjonen levende, så komponentene med
// oppførsel må være registrert, som i LO-TEK-docs.
defineLtTabs();
defineLtDialog();
defineLtConnectionStatus();

const $ = (id) => document.getElementById(id);
const gateEl = $("gate");
const hud = $("hud");
const panel = $("panel");
const status = $("status");
const enterBtn = $("enter");
const beginBtn = $("begin");
const toast = document.querySelector("lt-toast");

let canvas = $("world");
// Grafikken: forhåndsvalgene enkel (de svakeste maskinene og WebGL2: uten
// kantutjevning, solskygge og glød, høyst 900p), standard (myke skygger,
// solstråler, SMAA, opptil 1,5 piksler per CSS-piksel, høyst 1080p) og
// avansert (lengre sikt, skygge langt unna, skygge i krokene, alle lys,
// opptil 1440p) setter bryterne i `ports::Quality`; hver bryter kan endres
// for seg i grafikkdialogen (`openGraphics`). Valget huskes, og grafikken
// byttes aldri av seg selv.
const QUALITIES = ["enkel", "standard", "avansert"];
const QUALITY_INFO = {
  enkel: { cap: 900, dense: 1, note: "uten solskygge, glød og kantutjevning; for svake maskiner og WebGL2." },
  standard: { cap: 1080, dense: 1.5, note: "myke skygger, solstråler og SMAA, kortere sikt. Laget for en vanlig maskin." },
  avansert: { cap: 1440, dense: 1.5, note: "lengre sikt, skygge langt unna og i krokene, alle lys, høyere oppløsning." },
};
let quality = null;
// Har spilleren endret en bryter i grafikkdialogen, står bryterne her
// (`fristad.graphics.custom`, med `scale` for oppløsningsskalaen), og
// forhåndsvalget over er bare utgangspunktet og taket for oppløsningen.
let custom = null;
try {
  const q = localStorage.getItem("fristad.graphics");
  if (QUALITIES.includes(q)) quality = q;
  const c = JSON.parse(localStorage.getItem("fristad.graphics.custom") || "null");
  if (quality && c && typeof c === "object") custom = c;
} catch {
  /* ingen lagring */
}
let engine = null;
// Redigeringsmodusen (`?edit`, bare for eieren; web/editor.js).
const EDIT = new URLSearchParams(location.search).has("edit");
// `?nivaa=test` (eller `brett`): nivådokumentet som eneste kryptnivå, og med
// `&slaa` bitene slått sammen per rute på 16 m (målingen i fase 0 av
// nivåredigeringen, `docs/nivaaredigering.md`; `?nivaa=test&bench`).
const NIVAA = new URLSearchParams(location.search).get("nivaa") || undefined;
const SLAA = new URLSearchParams(location.search).has("slaa");
let editor = null;
let editorStarted = false;
let docs = {};
let panelOpen = false;
let current = { component: "" };

// Oppløsningen tegnes i faste trinn (høyden i piksler; bredden følger
// vinduet), så et stort vindu på en svak maskin ikke gir en oppløsning
// den ikke kan håndtere. «Automatisk» tar det største trinnet som får
// plass på skjermen, men aldri over 1080p i standard grafikk og 1440p i
// avansert. Valget huskes som `fristad.resolution` (høyden eller «auto»).
const RES_STEPS = [720, 900, 1080, 1152, 1440, 1620, 1800, 2160];
// Musefølsomheten (ganger bevegelsen) og lysstyrken (gamma, 1 er som malt;
// over 1 løfter de mørke tonene for mørke skjermer), fra Innstillinger (O).
// Huskes som `fristad.mouse` og `fristad.gamma`.
const readNum = (key, lo, hi, def) => {
  try {
    const v = Number(localStorage.getItem(key));
    return Number.isFinite(v) && v >= lo && v <= hi && localStorage.getItem(key) !== null ? v : def;
  } catch {
    return def;
  }
};
let mouseSens = readNum("fristad.mouse", 0.2, 3, 1);
let gamma = readNum("fristad.gamma", 0.6, 2, 1);
// Tekststørrelsen i hele grensesnittet (eieren 18. oktober: «et hakk
// større»): 1 er 18 px brødtekst, og LO-TEKs tokens og alt i style.css
// følger rot-fontstørrelsen (rem), mens canvaset står som det er. Huskes
// som `fristad.textsize`.
let textSize = readNum("fristad.textsize", 0.9, 1.4, 1);
// Kolonnen til høyre slutter over siktekortet når de ellers ville møttes
// (smalt vindu eller stor tekst).
function fitSide() {
  const rem = parseFloat(getComputedStyle(document.documentElement).fontSize) || 18;
  const card = Math.min(25 * rem, innerWidth - 32);
  const side = Math.min(19 * rem, innerWidth - 32);
  document.documentElement.classList.toggle("side-short", innerWidth / 2 + card / 2 + 12 > innerWidth - 16 - side);
}
const setTextSize = (v) => {
  document.documentElement.style.setProperty("--ui-text", String(v));
  fitSide();
};
setTextSize(textSize);
addEventListener("resize", fitSide);
const remember = (key, v) => {
  try {
    localStorage.setItem(key, String(v));
  } catch {
    /* ingen lagring */
  }
};
const RES_NAMES = { 720: "720p", 900: "900p", 1080: "1080p (FHD)", 1152: "QWXGA", 1440: "1440p (QHD)", 1620: "1620p (3K)", 1800: "1800p (QHD+)", 2160: "2160p (4K)" };
let resolution = "auto";
try {
  const r = localStorage.getItem("fristad.resolution");
  if (r && (r === "auto" || RES_STEPS.includes(Number(r)))) resolution = r === "auto" ? r : Number(r);
} catch {
  /* ingen lagring */
}
// Høyden i piksler som tegnes for vinduet slik det er nå.
function renderHeight() {
  const dpr = window.devicePixelRatio || 1;
  const ph = Math.max(1, Math.round(canvas.clientHeight * dpr));
  if (resolution === "auto") {
    // Standard og avansert tegner opptil 1,5 piksler per CSS-piksel på
    // skjermer med høy pikseltetthet, og aldri over trinnet (1080p og 1440p).
    const info = QUALITY_INFO[quality ?? "standard"];
    const dense = Math.round(canvas.clientHeight * Math.min(dpr, info.dense));
    const cap = Math.min(dense, info.cap);
    const fit = RES_STEPS.filter((h) => h <= cap);
    return fit.length ? fit[fit.length - 1] : Math.min(ph, cap);
  }
  // Et valgt trinn, men aldri flere piksler enn skjermen viser.
  return Math.min(resolution, ph);
}
// Målingen (`?bench`) kan tegne i en brøkdel av oppløsningen.
let benchScale = 1;
function fit() {
  const h = Math.max(1, Math.round(renderHeight() * benchScale * (custom?.scale ?? 1)));
  const w = Math.max(1, Math.round((h * canvas.clientWidth) / Math.max(1, canvas.clientHeight)));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  engine?.resize(w, h);
}
function setResolution(r, announce = true) {
  resolution = r;
  try {
    localStorage.setItem("fristad.resolution", String(r));
  } catch {
    /* ingen lagring */
  }
  fit();
  if (announce) toast?.show?.(r === "auto" ? "Oppløsning: automatisk." : `Oppløsning: ${canvas.width} × ${canvas.height}.`);
}

// Fristad er laget for datamaskin med mus og tastatur; mobil og nettbrett er
// ikke et mål (eieren 19. oktober). Der lastes ingenting, og startkortet ber
// deg prøve på en datamaskin (`?mobil` laster likevel).
function onPhone() {
  const touchOnly = matchMedia("(pointer: coarse)").matches && !matchMedia("(pointer: fine)").matches;
  const ua = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);
  return (touchOnly || ua) && !new URLSearchParams(location.search).has("mobil");
}

async function boot() {
  if (onPhone()) {
    status.hidden = true;
    $("loading").hidden = true;
    $("touch-note").hidden = false;
    return;
  }
  fit();
  stage("Laster motoren …");
  await init();
  try {
    docs = await (await fetch("docs.json")).json();
  } catch {
    docs = {};
  }
  // Teksturlagene: egne bilder og fotoskannede CC0-materialer, samlet av
  // tools/fetch-assets.ts under byggingen. Mangler de, bakes lagene i motoren.
  const assets = await loadAssets();
  // Modellene fra Blender ligger ved siden av motoren, gzip-pakket.
  const models = await loadModels();
  stage("Bygger byen og baker materialene …");
  // Gi nettleseren ett bilde til å tegne teksten før wasm tar tråden.
  await new Promise((r) => setTimeout(r, 30));
  watchGpuLoss();
  // `?webgl` i adressen tvinger WebGL2, til testing.
  const forceGl = new URLSearchParams(location.search).has("webgl");
  try {
    if (forceGl) throw new Error("WebGL2 valgt i adressen");
    engine = await start(canvas, true, assets.manifest, assets.names, assets.blobs, models, EDIT, NIVAA, SLAA, lazyManifest);
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    canvas = fresh;
    fit();
    engine = await start(canvas, false, assets.manifest, assets.names, assets.blobs, models, EDIT, NIVAA, SLAA, lazyManifest);
  }
  console.info(`Fristad kjører på ${engine.backend()}, LO-TEK ${engine.lotek_version()}`);
  if (gpuLost !== null) loseGpu(gpuLost);
  engine.set_gamma(gamma);
  // Med `?nivaa=`: rett ned i nivået.
  if (NIVAA) engine.descend();
  // Til feilsøking i konsollen og til testene.
  window.fristad = { engine, openPanel, closePanel, openShop, readDoc, setGraphics, setQuality, bench: runBench };
  // Uten eget valg får WebGL2 enkel grafikk og WebGPU standard.
  setQuality(quality ?? (engine.backend() === "Gl" ? "enkel" : "standard"), false, false);
  if (EDIT && engine.can_edit()) {
    editor = createEditor({
      engine,
      canvas,
      toast,
      // P: musa fanges med en gang (mens tastetrykket teller), og når
      // spillet er bygget på nytt, settes grafikknivået igjen.
      onLock: () => lock(),
      onPlay: () => setQuality(quality ?? (engine.backend() === "Gl" ? "enkel" : "standard"), false, false),
    });
    window.fristad.editor = editor;
  }
  $("version").textContent = engine.lotek_version();
  $("weather").textContent = engine.weather();
  new ResizeObserver(fit).observe(canvas);
  fit();
  bindInput();
  requestAnimationFrame(loop);

  status.hidden = true;
  $("loading").hidden = true;
  // Når alt er lastet, velger du: se introfilmen (som toner over i byen ved
  // sørporten) eller gå rett til spillet (eieren 18. oktober). Verden står
  // ikke bak kortet før du har valgt (`.gate-first`).
  if (FILM) {
    beginBtn.addEventListener("click", startFilm, { once: true });
    beginBtn.hidden = false;
    enterBtn.textContent = "Gå rett til spillet";
    enterBtn.dataset.variant = "secondary";
    enterBtn.hidden = false;
    beginBtn.focus();
  } else {
    enterBtn.hidden = false;
    enterBtn.focus();
  }
  checkGpu();
  if (new URLSearchParams(location.search).has("bench")) runBench();
}

// Tegner nettleseren uten grafikkortet (maskinvareakselerasjon slått av,
// eller driveren svartelistet), faller den tilbake til programvare som
// SwiftShader eller Microsoft Basic Render Driver, og alt går svært tregt.
// Da sier startkortet fra og forklarer hvordan det slås på; grafikken
// byttes bare om spilleren velger det.
let gpuName = "";
let softwareGpu = false;
let gpuToastShown = false;
const SOFTWARE = /swiftshader|llvmpipe|softpipe|lavapipe|basic render driver|software rasterizer|microsoft basic/i;
async function checkGpu() {
  let name = "";
  let fallback = false;
  try {
    const gl = document.createElement("canvas").getContext("webgl2");
    const info = gl?.getExtension("WEBGL_debug_renderer_info");
    name = (info && gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) || gl?.getParameter(gl.RENDERER) || "";
    gl?.getExtension("WEBGL_lose_context")?.loseContext();
  } catch {
    /* ingen WebGL2 */
  }
  if (engine.backend() !== "Gl" && navigator.gpu) {
    try {
      const adapter = await navigator.gpu.requestAdapter();
      fallback = !!(adapter?.info?.isFallbackAdapter ?? adapter?.isFallbackAdapter);
      const d = adapter?.info?.description || adapter?.info?.device || "";
      if (d) name = d;
      else if (adapter?.info?.vendor && !name) name = adapter.info.vendor;
    } catch {
      /* ingen adapter */
    }
  }
  gpuName = name;
  softwareGpu = fallback || SOFTWARE.test(name);
  console.info(`Grafikk: ${name || "ukjent"}${softwareGpu ? " (programvare, uten grafikkort)" : ""}`);
  if (!softwareGpu) return;
  const short = name.match(/swiftshader|llvmpipe|softpipe|lavapipe|microsoft basic render driver|basic render driver/i)?.[0];
  $("gpu-name").textContent = short ? `tegnes med ${short}` : "tegnes med programvare";
  $("gpu-warning").hidden = false;
  $("gpu-reduced").hidden = quality === "enkel";
  $("gpu-reduced").addEventListener("click", () => {
    setQuality("enkel");
    $("gpu-reduced").hidden = true;
  });
}

async function loadAssets() {
  const none = { manifest: undefined, names: [], blobs: [] };
  let manifest;
  try {
    // Manifestet hentes alltid ferskt: et gammelt et i hurtigbufferet viser
    // til filer som ikke finnes lenger.
    const res = await fetch("assets/manifest.json", { cache: "no-cache" });
    if (!res.ok) return none;
    manifest = await res.text();
  } catch {
    return none;
  }
  const paths = asset_paths(manifest);
  const bar = $("loading");
  bar.hidden = false;
  let done = 0;
  const tick = () => {
    done++;
    bar.value = Math.round((done / paths.length) * 100);
    setStatus(`Henter teksturer (${done} av ${paths.length}) …`);
  };
  setStatus(`Henter teksturer (0 av ${paths.length}) …`);
  // Seks om gangen, så nettleseren ikke åpner hundre forbindelser. En fil
  // som mangler gjør bare at det laget bakes i motoren, ikke alle.
  const blobs = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      try {
        const r = await fetch(`assets/${paths[i]}`);
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        blobs[i] = new Uint8Array(await r.arrayBuffer());
      } catch (e) {
        console.warn(`Teksturen ${paths[i]} kunne ikke hentes (${e.message}); laget bakes i stedet.`);
      }
      tick();
    }
  };
  await Promise.all(Array.from({ length: 6 }, worker));
  const got = paths.map((p, i) => [p, blobs[i]]).filter(([, b]) => b);
  return { manifest, names: got.map(([p]) => p), blobs: got.map(([, b]) => b) };
}

// Henter én modellfil (models/<sti>.gz, stien fra `model_url`) og pakker
// den ut; `count` får bytene mens de kommer.
async function fetchModel(path, count = () => {}) {
  const r = await fetch(`models/${model_url(path)}.gz`);
  if (!r.ok) {
    console.warn(`Modellen ${path} mangler: HTTP ${r.status}`);
    return undefined;
  }
  const counted = r.body.pipeThrough(
    new TransformStream({
      transform(chunk, ctl) {
        count(chunk.byteLength);
        ctl.enqueue(chunk);
      },
    }),
  );
  let bytes = new Uint8Array(await new Response(counted).arrayBuffer());
  // Pakket ut allerede hvis serveren sendte den med Content-Encoding.
  if (bytes[0] === 0x1f && bytes[1] === 0x8b) {
    const plain = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
    bytes = new Uint8Array(await new Response(plain).arrayBuffer());
  }
  return bytes;
}

// Mister nettleseren grafikkortet (grafikkprosessen startes på nytt, eller
// SwiftShader i Chromium uten skjerm), må motoren slutte å bruke enheten:
// ellers kaster neste createBuffer, og motoren stopper. wgpu 25 kan ikke
// melde det selv (lukningen den gir `device.lost` er sluppet før løftet
// løses), så vi lytter på enheten nettleseren gir motoren.
let gpuLost = null;
function watchGpuLoss() {
  const proto = globalThis.GPUAdapter?.prototype;
  if (!proto || proto.requestDevice.fristad) return;
  const request = proto.requestDevice;
  const wrapped = async function (...args) {
    const device = await request.apply(this, args);
    device.lost.then((info) => {
      if (info.reason === "destroyed") return;
      loseGpu(info.message || "ukjent grunn");
    });
    return device;
  };
  wrapped.fristad = true;
  proto.requestDevice = wrapped;
}
function loseGpu(message) {
  gpuLost = message;
  if (!engine || engine.backend() === "Gl") return;
  engine.gpu_lost(message);
  toast?.show?.("Grafikkortet forsvant. Last siden på nytt for å se byen igjen.");
}

// Bitene til et kryptnivå hentes først når økta ber om nivået
// (`fetch_level`, plan del 10a), og gis motoren med `add_level`.
const levelsAsked = new Set();
async function fetchLevel(n) {
  if (levelsAsked.has(n)) return;
  levelsAsked.add(n);
  const paths = level_paths(n);
  const out = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      out[i] = await fetchModel(paths[i]).catch((e) => console.warn(`Modellen ${paths[i]}:`, e));
    }
  };
  await Promise.all(Array.from({ length: 4 }, worker));
  levelsAsked.delete(n);
  engine?.add_level(n, out);
}

// Byens modeller som ikke trengs til første bilde (innsiden, møblene,
// galleriet, fiendene, utstyret du ikke har på; `lazy_paths`, plan del 10a)
// hentes når økta ber om dem (`fetch_models`) og gis motoren med
// `add_models`. Verden bygges før de kommer, fra manifestet
// (models/manifest.json, mål, kollidere og skjelett). Mangler manifestet,
// hentes alt ved start som før.
let lazyManifest;
async function fetchLazy(names, paths) {
  const out = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      out[i] = await fetchModel(paths[i]).catch((e) => console.warn(`Modellen ${paths[i]}:`, e));
    }
  };
  await Promise.all(Array.from({ length: 4 }, worker));
  engine?.add_models(names, out);
}

// Henter modellfilene byen trenger (alle utenom bitene til kryptnivåene og
// det som hentes senere; med ?edit alle) og pakker dem ut. Framdriften
// regnes i byte, fra størrelsene i models/sizes.json.
async function loadModels() {
  const all = model_paths();
  if (!EDIT) {
    const bytes = await fetchModel("manifest.json").catch(() => undefined);
    if (bytes) lazyManifest = new TextDecoder().decode(bytes);
  }
  const later = new Set(lazyManifest ? lazy_paths() : []);
  const wanted = new Set(EDIT ? all : start_paths().filter((p) => !later.has(p)));
  let sizes = {};
  try {
    sizes = await (await fetch("models/sizes.json")).json();
  } catch {
    sizes = {};
  }
  for (const level of Object.values(sizes.levels || {})) Object.assign(sizes, level);
  const paths = all.filter((p) => wanted.has(p));
  const total = paths.reduce((a, p) => a + (sizes[p] || 1), 0);
  let got = 0;
  const bar = $("loading");
  bar.hidden = false;
  const show = () => {
    bar.value = Math.round((got / total) * 100);
    setStatus(`Henter byen (${(got / 1e6).toFixed(0)} av ${(total / 1e6).toFixed(0)} MB) …`);
  };
  show();
  // Samme rekkefølge som model_paths(); det som ikke hentes, er tomt.
  const out = new Array(all.length);
  let next = 0;
  let last = 0;
  const worker = async () => {
    while (next < paths.length) {
      const path = paths[next++];
      out[all.indexOf(path)] = await fetchModel(path, (n) => {
        got += n;
        if (got - last > 256 * 1024) {
          last = got;
          show();
        }
      });
      show();
    }
  };
  await Promise.all(Array.from({ length: 6 }, worker));
  return out;
}

function setStatus(text) {
  const span = status.querySelector("span:last-child");
  if (span) span.textContent = text;
}

// Et trinn uten kjent framdrift: stolpen vises uten verdi (ubestemt).
function stage(text) {
  setStatus(text);
  const bar = $("loading");
  bar.hidden = false;
  bar.removeAttribute("value");
}

let last = performance.now();
let frames = 0;
let perfAt = performance.now();
// Byttes grafikken: motoren, oppløsningen og knappene. Et forhåndsvalg
// setter alle bryterne (de egendefinerte glemmes).
function setQuality(name, chosen = true, announce = true) {
  quality = name;
  if (chosen) {
    custom = null;
    try {
      localStorage.setItem("fristad.graphics", name);
      localStorage.removeItem("fristad.graphics.custom");
    } catch {
      /* ingen lagring */
    }
  }
  sendGraphics();
  if (announce) toast?.show?.(`Grafikk: ${name} – ${QUALITY_INFO[name].note} G bytter nivå.`);
}
// Bryterne i et forhåndsvalg, fra motoren (verdiene står i `ports::Quality`).
const presetCache = {};
function presetOf(name) {
  if (!presetCache[name] && engine?.quality_preset) {
    try {
      presetCache[name] = { ...JSON.parse(engine.quality_preset(name)), scale: 1 };
    } catch {
      /* eldre motor */
    }
  }
  return presetCache[name];
}
// Bryterne slik de står nå (forhåndsvalget eller de egendefinerte).
function currentGraphics() {
  return custom ?? presetOf(quality ?? "standard");
}
// Egendefinerte brytere fra dialogen. Svarer de til et forhåndsvalg, er
// det det forhåndsvalget igjen.
function setCustom(values) {
  const same = QUALITIES.find((q) => {
    const p = presetOf(q);
    return p && Object.keys(p).every((k) => Number(p[k]) === Number(values[k]));
  });
  if (same) {
    setQuality(same, true, false);
    return;
  }
  custom = values;
  try {
    localStorage.setItem("fristad.graphics", quality ?? "standard");
    localStorage.setItem("fristad.graphics.custom", JSON.stringify(values));
  } catch {
    /* ingen lagring */
  }
  sendGraphics();
}
function graphicsName() {
  return custom ? "egendefinert" : (quality ?? "standard");
}
function sendGraphics() {
  if (engine) {
    if (custom) {
      const { scale, ...switches } = custom;
      engine.set_quality(JSON.stringify({ preset: quality ?? "standard", ...switches }));
    } else engine.set_quality(quality ?? "standard");
  }
  slowSeconds = 0;
  fit();
  const name = graphicsName();
  $("graphics").textContent = `Grafikk: ${name}`;
  $("graphics-label").textContent = name;
}
// Til nettlesertestene og eldre lenker: standard (`true`) eller avansert.
function setGraphics(reduced, chosen = true, announce = true) {
  setQuality(reduced ? "standard" : "avansert", chosen, announce);
}
// Går det tregt over tid, foreslås et lavere nivå én gang. Ingenting
// byttes uten at spilleren ber om det.
let slowSeconds = 0;
let suggested = false;
function adapt(fps) {
  if (!playing || quality === "enkel" || suggested || benchRunning) return;
  slowSeconds = fps < 25 ? slowSeconds + 1 : 0;
  if (slowSeconds >= 8) {
    suggested = true;
    toast?.show?.("Spillet går tregt. Velg et lavere grafikknivå i Innstillinger (O) eller med G.");
  }
}
function loop(now) {
  const dt = Math.min((now - last) / 1000, 0.1);
  last = now;
  // Åpningsfilmen er sitt eget malte bilde over spillet. Mens den dekker
  // hele skjermen, tegnes ikke spillet; når den toner bort til slutt, står
  // du ved sørporten bak den.
  if (film) {
    const cover = film.tick(dt);
    if (cover < 0.999 && !filmReveal) {
      filmReveal = true;
      if (!filmReplay) {
        engine.set_theme(false);
        engine.respawn();
      }
    }
    if (cover >= 0.999) {
      requestAnimationFrame(loop);
      return;
    }
  }
  updateLook();
  // Trubaduren i Vertshuset: munnen følger sangen der den er (0 når den ikke spiller).
  engine.set_voice?.(music.voice());
  const events = JSON.parse(engine.frame(dt));
  // En feil i én hendelse skal aldri stoppe løkka (og fryse spillet): den
  // logges, og resten av hendelsene og neste bilde går som før.
  for (const e of events) {
    try {
      handle(e);
    } catch (err) {
      console.error(`Hendelsen ${e?.type ?? "?"} feilet:`, err);
    }
  }
  // Bilder per sekund, så eieren kan si hvordan det flyter.
  frames++;
  benchFrames++;
  if (now - perfAt > 1000) {
    const fps = Math.round((frames * 1000) / (now - perfAt));
    adapt(fps);
    const tris = Math.round(engine.triangles() / 1000);
    const res = `${canvas.width}×${canvas.height}`;
    // Tiden på grafikkortet per del (bare der nettleseren kan måle den):
    // summen i telleren, delene når du holder musa over.
    const parts = engine.gpu_times ? engine.gpu_times() : "";
    // (Programvaregrafikk kan gi bare nuller; da vises ingenting.)
    const gpuMs = parts ? parts.split(" · ").reduce((sum, p) => sum + parseFloat(p.split(" ").pop()), 0) : 0;
    // Delene står rett i telleren (musa er fanget, så ingen kan holde den over).
    const partText = parts ? parts.replace(/skygge/i, "sk").replace(/verden/i, "v").replace(/etterbehandling/i, "e").replace(/ ms/g, "") : "";
    $("perf").textContent = `${fps} fps · ${engine.backend() === "Gl" ? "WebGL2" : "WebGPU"}${softwareGpu ? " · uten grafikkort" : ""} · ${res} · ${tris}k △${gpuMs > 0 ? ` · GPU ${gpuMs.toFixed(1)} ms (${partText})` : ""}`;
    $("perf").title = `Bilder per sekund, grafikk-API og oppløsning${gpuName ? `. Grafikk: ${gpuName}` : ""}${gpuMs > 0 ? `. GPU-tid (ms): ${parts}` : ""}`;
    frames = 0;
    perfAt = now;
  }
  requestAnimationFrame(loop);
}

// Et lite klikk når du trykker på en knapp i dialogene og panelet.
document.addEventListener("click", (ev) => {
  if (ev.target.closest?.("dialog button, .lt-button, #panel button")) sfx.play("click");
});

// Treffet: en rød bue i den kanten av bildet slaget eller skuddet kom fra
// (`Event::Hurt`: vinkelen fra blikket, 0 foran = øverst, positiv til høyre,
// ±π bak = nederst). Hver bue toner ut for seg (`.hurt-arc` i style.css).
function hurtArc(angle) {
  const arcs = document.querySelectorAll(".hurt-arc");
  if (arcs.length >= 4) arcs[0].remove();
  const el = document.createElement("div");
  el.className = "hurt-arc";
  el.setAttribute("aria-hidden", "true");
  el.style.setProperty("--from", `${(angle * 180) / Math.PI}deg`);
  el.addEventListener("animationend", () => el.remove());
  document.body.appendChild(el);
}

function handle(e) {
  switch (e.type) {
    case "prompt":
      if (e.label) {
        $("prompt-name").textContent = e.label;
        $("prompt-info").textContent = e.info || "";
        $("prompt-info").hidden = !e.info;
        $("key-use").hidden = !e.verb;
        $("key-use-text").textContent = e.verb || "";
        $("key-trade").hidden = !e.trade;
        $("key-trade-text").textContent = e.trade || "";
        $("target").hidden = false;
      } else {
        $("target").hidden = true;
      }
      break;
    case "fade":
      fade();
      break;
    case "fetch_level":
      fetchLevel(e.level);
      break;
    case "fetch_models":
      fetchLazy(e.names, e.paths);
      break;
    case "warp":
      warp();
      break;
    case "hurt":
      hurtArc(e.angle);
      break;
    case "map":
      setMap(e.map);
      break;
    case "where":
      mapWhere = e;
      drawMaps();
      break;
    case "theme":
      document.documentElement.dataset.theme = e.theme;
      music.night(e.theme === "dark");
      break;
    case "toast":
      // Lappene i krypten er opptil rundt 400 tegn: la varselet stå lenge nok
      // til å lese dem (rundt 15 tegn i sekundet, minst seks sekunder).
      toast?.show?.(e.text, { duration: Math.max(6000, e.text.length * 70) });
      break;
    case "goal":
      setGoal(e);
      break;
    case "lore":
      openLore(e.title, e.text);
      break;
    case "foe":
      setFoe(e.foe);
      break;
    case "weather":
      $("weather").textContent = e.weather;
      break;
    case "hud":
      // Pusten går når utholdenheten er brukt opp, og hjertet slår når
      // livet er lavt.
      if (playing && e.stamina === 0 && lastHud && lastHud.stamina > 0) sfx.play("breath");
      if (playing) sfx.heart(e.hp);
      lastHud = e;
      if (playing) music.set(e.cellar ? "cellar" : "town");
      music.hud(e);
      setBars(e.hp, e.stamina);
      // Lykta: stolpen står alltid (i byen ser du hva du må fylle hos
      // Apotekeren; oljen brenner bare nede), og mørket lukker seg nede når
      // oljen går tom.
      $("light-bar").hidden = false;
      $("light").value = e.light;
      $("light-text").textContent = `${e.light} %`;
      $("light-bar").title = e.cellar ? "Oljen i lykta brenner" : "Lykta brenner bare i krypten. Olje får du hos Apotekeren.";
      document.documentElement.style.setProperty("--dark", e.cellar ? String(Math.max(0, 1 - e.light / 25)) : "0");
      $("tokens").textContent = `${e.tokens} tokens`;
      if (overlayOpen) updateShop();
      $("keys").textContent = e.keys === 1 ? "1 nøkkel" : `${e.keys} nøkler`;
      $("keys").hidden = !e.keys;
      $("boss").hidden = !e.boss;
      if (e.boss) {
        $("boss-name").textContent = e.boss.name;
        $("boss-hp").value = e.boss.hp;
      }
      // Liv og tokens vises nede i kjelleren, og oppe når du har noe.
      $("vitals").hidden = !(e.cellar || e.tokens > 0);
      $("fight-keys").hidden = !e.cellar;
      if (!e.cellar) setFoe(null);
      break;
    case "sound":
      sfx.play(e.name, e);
      break;
    case "marks":
      updateMarks(e.marks);
      break;
    case "speech":
      updateBubbles(e.bubbles);
      break;
    case "inventory":
      renderInventory(e);
      setBars(lastHud.hp, lastHud.stamina);
      try {
        localStorage.setItem("fristad.save", engine.save_state());
      } catch {
        /* lagring er en bekvemmelighet */
      }
      break;
    case "haul":
      lootMore = e.components ?? [];
      break;
    case "loot":
      openLoot(e.component);
      break;
    case "edit":
      editor?.update(e.edit);
      break;
    case "open_doc":
      // F på en fylt ramme på galleriveggen: boka i panelet til høyre.
      readDoc(e.component);
      break;
    case "ambience":
      lastAmbience = e;
      if (playing) {
        ambience.set(e);
        sfx.ambience(e);
        music.ambience(e);
      }
      break;
    case "shop":
      openShop(e.npc, e.wares, e.swaps ?? []);
      break;
    case "mead":
      openMead(e);
      break;
    case "song":
      music.song(e.on, () => engine.song_over?.());
      break;
    case "workshop":
      openWorkshop(e.recipes);
      break;
  }
}

// Lageret og utstyret i hjørnet: utstyret som linjer, komponentene som
// chips du kan lese om i boka (det gir liv tilbake, én gang per tur).
let lastHud = { hp: 100, stamina: 100, tokens: 0, keys: 0 };

// Stolpene over knappelinja: liv og energi med tall (maks fra utstyret).
function setBars(hp, stamina) {
  const maxHp = lastInv?.max_hp ?? 100;
  const maxSt = lastInv?.max_stamina ?? 100;
  $("hp").max = maxHp;
  $("hp").value = hp;
  $("hp-text").textContent = `${hp} / ${maxHp}`;
  $("stamina").max = maxSt;
  $("stamina").value = stamina;
  $("stamina-text").textContent = `${stamina} / ${maxSt}`;
}
let lastInv = null;
function renderInventory(e) {
  lastInv = e;
  if (invOpen) renderInvGear(e);
  $("gear").innerHTML = `<div><b>Våpen:</b> ${esc(e.weapon)} <i>${esc(e.weapon_special)}</i></div><div><b>Skjold:</b> ${esc(e.shield)}</div><div><b>Rustning:</b> ${esc(e.armor)}${e.lantern ? " · Søkelykt" : ""}</div>`;
  const inv = $("inventory");
  inv.replaceChildren();
  const chip = (name, n, carried) => {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "chip" + (carried ? " carried" : "") + (e.read.includes(name) ? " read" : "");
    b.textContent = `${name} ×${n}`;
    b.title = carried ? "Bæres: trygt når du er oppe igjen. Klikk for å lese i boka." : "Klikk for å lese i boka (+20 liv første gang per tur).";
    b.addEventListener("click", () => readDoc(name));
    inv.appendChild(b);
  };
  for (const [name, n] of e.stash) chip(name, n, false);
  for (const [name, n] of e.carried) chip(name, n, true);
  $("vitals").hidden = false;
}

function esc(t) {
  const d = document.createElement("div");
  d.textContent = t ?? "";
  return d.innerHTML;
}

// Boka: dokumentasjonen for en komponent du har vunnet.
function readDoc(component) {
  sfx.play("page");
  engine.read_doc(component);
  if (docs[component]) {
    current = { component };
    closeOverlay(false);
    openPanel();
  }
}

// Ned i krypten og opp igjen: skjermen går i svart et øyeblikk.
function fade() {
  const el = $("fade");
  el.classList.remove("on");
  void el.offsetWidth;
  el.classList.add("on");
}

// Kartet over etasjen i kjelleren: minikartet dreier med deg (fram er
// opp), det store kartet (K) har nord opp. Bare det du har sett tegnes.
let mapData = null;
let mapWhere = null;
let bigMapOpen = false;
function setMap(m) {
  mapData = m;
  $("minimap").hidden = !mapData;
  if (!mapData) toggleBigMap(false);
  else $("map-name").textContent = `Nivå ${m.level} · ${m.name}`;
  drawMaps();
}
// Målet nå (én linje i panelet) og journalen (J) med det du har lest.
// Målet endres når du går ned, slår bossen, finner noe eller leser en lapp,
// så spillet lagres da også.
function setGoal(g) {
  const goal = $("goal");
  goal.textContent = g.text;
  goal.hidden = !g.text;
  $("journal-goal").textContent = g.text;
  const lore = $("journal-lore");
  lore.replaceChildren();
  if (!g.journal.length) {
    const p = document.createElement("p");
    p.textContent = "Du har ikke lest noe ennå. Lappene og bøkene i krypten havner her.";
    lore.append(p);
  }
  for (const j of g.journal) {
    const h = document.createElement("h3");
    h.className = "lt-heading";
    h.dataset.size = "xs";
    h.textContent = j.title;
    const p = document.createElement("p");
    p.className = "lt-paragraph";
    p.textContent = j.text;
    lore.append(h, p);
  }
  try {
    localStorage.setItem("fristad.save", engine.save_state());
  } catch {
    /* lagring er en bekvemmelighet */
  }
}
function toggleJournal(on = $("journal").hidden) {
  $("journal").hidden = !on;
}
function toggleBigMap(on = !bigMapOpen) {
  bigMapOpen = on && !!mapData;
  $("bigmap").hidden = !bigMapOpen;
  drawMaps();
}
const MAP_COLORS = { floor: "#6b5a44", wall: "#e8d9b8", player: "#ffd75e", exit: "#4cc3ff", gate: "#c9a24a", gate_open: "#7d6a3c", chest: "#e8b84a", ascend: "#9be08a", boss: "#e0483a", shortcut: "#b08a5a", lever: "#d9d0c0", lever_pulled: "#8a8070", rest: "#f0c060", rest_lit: "#ffe08a" };
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
function drawMaps() {
  if (!mapData) return;
  drawMap($("minimap-canvas"), true, 30);
  if (bigMapOpen) drawMap($("bigmap-canvas"), false, 0);
}

// Gjennom portalen: en virvel over skjermen (synsfeltet strekkes i motoren).
function warp() {
  const el = $("warp");
  el.classList.remove("on");
  void el.offsetWidth;
  el.classList.add("on");
}

// Kista: en LO-TEK-dialog som sier hva du fant, og lar deg lese om
// komponenten i boka (panelet til høyre). Spillet står på pause så lenge.
let lootOpen = false;
// Det som fulgte med i kista utenom kistas egen komponent (`haul`, plan 14).
let lootMore = [];
function openLoot(component) {
  lootOpen = true;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const d = docs[component] ?? {};
  const title = d.title ?? component;
  const more = lootMore;
  lootMore = [];
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog loot";
  dlg.setAttribute("aria-labelledby", "loot-title");
  const found = component
    ? `<p>Under lokket, mellom råtne filler og knokler, ligger <strong>${esc(title)}</strong>: en bit av LO-TEK, slik den så ut før den ble korrupt.</p>
       <p class="lt-help-text">${esc(d.description ?? "")}</p>
       ${more.length ? `<p>Under den ligger også ${more.map((c) => `<strong>${esc(compTitle(c))}</strong>`).join(more.length > 2 ? ", " : " og ").replace(/, (?=[^,]*$)/, " og ")}.</p>` : ""}
       <p>Bær ${more.length ? "dem" : "den"} opp til byen, så er de${more.length ? "" : "n"} din${more.length ? "e" : ""}. Dør du før det, blir ${more.length ? "de" : "den"} liggende her nede.</p>`
    : `<p>Kista er tom. Noen har vært her før deg.</p>`;
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="loot-title">${component ? "Du fant noe i kista" : "Ingenting"}</h2>
    <div class="lt-dialog__body">${found}</div>
    <div class="lt-dialog__footer">
      <button class="lt-button" data-variant="secondary" type="button" data-act="take">${component ? "Ta med og gå videre" : "Gå videre"}</button>
      ${component && docs[component] ? `<button class="lt-button" data-variant="primary" type="button" data-act="read">Les om ${esc(title)}</button>` : ""}
    </div>`;
  document.body.appendChild(dlg);
  const close = (then) => {
    lootOpen = false;
    dlg.close();
    dlg.remove();
    then();
  };
  dlg.querySelector('[data-act="take"]').addEventListener("click", () => close(() => enter()));
  dlg.querySelector('[data-act="read"]')?.addEventListener("click", () => close(() => readDoc(component)));
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    close(() => {
      enter(false);
      clickHint();
    });
  });
  dlg.showModal();
  dlg.querySelector('[data-act="read"], [data-act="take"]')?.focus();
}

// Lappene og bøkene i krypten (F på en lapp): et kort midt på skjermen med
// tittel og tekst, og spillet står stille mens du leser. F, E, Esc, mellomrom
// eller et klikk lukker det.
function openLore(title, text) {
  if (lootOpen) return;
  lootOpen = true;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog lore";
  dlg.setAttribute("aria-labelledby", "lore-title");
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="lore-title"></h2>
    <div class="lt-dialog__body"><p class="lt-paragraph lore-text"></p></div>
    <div class="lt-dialog__footer">
      <span class="lt-help-text lore-keys"><kbd>F</kbd> <kbd>E</kbd> <kbd>Esc</kbd> eller klikk lukker</span>
      <button class="lt-button" data-variant="primary" type="button" data-act="close">Legg fra deg</button>
    </div>`;
  dlg.querySelector("#lore-title").textContent = title;
  const body = dlg.querySelector(".lore-text");
  body.textContent = text ? `«${text}»` : "";
  body.hidden = !text;
  document.body.appendChild(dlg);
  let closed = false;
  const close = (relock) => {
    if (closed) return;
    closed = true;
    lootOpen = false;
    dlg.close();
    dlg.remove();
    sfx.play("page");
    enter(relock);
    if (!relock) clickHint();
  };
  dlg.addEventListener("click", () => close(true));
  dlg.addEventListener("keydown", (ev) => {
    if (ev.repeat) return;
    if (["KeyF", "KeyE", "Space", "Enter"].includes(ev.code)) {
      ev.preventDefault();
      ev.stopPropagation();
      close(true);
    }
  });
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    close(false);
  });
  dlg.showModal();
  dlg.querySelector('[data-act="close"]').focus();
}

// Fiendekortet midt på skjermen om fienden du sikter på: navnet, om den er
// elite eller boss, om den skyter, hva den er (kanon og slaget) og livet som
// følger skaden. Det står på samme plass som siktekortet, aldri samtidig.
function setFoe(f) {
  const el = $("foe");
  el.hidden = !f;
  if (!f) return;
  $("foe-name").textContent = f.name;
  $("foe-info").textContent = f.info;
  $("foe-rank").textContent = f.rank;
  $("foe-rank").hidden = !f.rank;
  // Slagene som skyter (inputen, Toast-frosken), plan 13e.
  $("foe-ranged").hidden = !f.ranged;
  $("foe-hp").max = f.max_hp;
  $("foe-hp").value = f.hp;
  $("foe-hp-text").textContent = `${f.hp} / ${f.max_hp}`;
}

// Tastepanelet oppe til høyre foldes sammen og ut med F1 eller et klikk på
// overskriften, og huskes. På smale skjermer er det sammenfoldet fra start.
function initKeysPanel() {
  const el = $("keys-panel");
  let open = innerWidth >= 700;
  try {
    const saved = localStorage.getItem("fristad.keys");
    if (saved) open = saved === "open";
  } catch {
    /* ingen lagring */
  }
  el.open = open;
  el.addEventListener("toggle", () => {
    try {
      localStorage.setItem("fristad.keys", el.open ? "open" : "closed");
    } catch {
      /* ingen lagring */
    }
  });
}
initKeysPanel();

// Inventaret (I), plan 13a/13b: plassene for det du har på (Våpen, Skjold,
// Rustning, Lys) til venstre er filteret, lista over utstyret ruller for
// seg i en dialog med fast høyde, hver rad er en oppsummering (ikon, navn,
// sjeldenhet), og den valgte står med beskrivelse og sammenligning mot det
// du har på, med «Bruk <navn>» rett under tabellen. Dobbeltklikk eller
// Enter tar på, og piltastene går i lista. Fanen «Komponenter» er lageret.
let invOpen = null;
let invSelected = null;
let invFilter = remembered("fristad.inventory.slot", "weapon");
let invDetail = false;
function toggleInventory() {
  if (invOpen) {
    closeInventory(true);
    return;
  }
  const e = lastInv ?? { stash: [], carried: [], weapon: "Rustent sverd", weapon_special: "", shield: "Gammelt rundskjold", armor: "ingen", lantern: false, max_hp: 100, max_stamina: 100, tokens: 0, read: [], gear: [] };
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog menu-dialog inventory-dialog";
  dlg.setAttribute("aria-labelledby", "inv-title");
  dlg.innerHTML = `
    <div class="lt-dialog__header menu-head">
      <h2 class="lt-dialog__title" id="inv-title">Utstyr og lager</h2>
      <div class="inv-stats"></div>
    </div>
    <div class="lt-dialog__body menu-body">
      <lt-tabs>
        <ul class="lt-tabs__list" aria-label="Inventaret">
          <li><a href="#inv-gear">Utstyr</a></li>
          <li><a href="#inv-comps" class="inv-comp-tab">Komponenter</a></li>
        </ul>
        <div class="lt-tabs__panel inv-gear-panel" id="inv-gear"></div>
        <div class="lt-tabs__panel inv-comps" id="inv-comps"></div>
      </lt-tabs>
    </div>
    <div class="lt-dialog__footer menu-foot">
      <button class="lt-button" data-variant="secondary" type="button" data-act="close">Lukk (I)</button>
    </div>`;
  document.body.appendChild(dlg);
  invOpen = dlg;
  invSelected = null;
  invDetail = false;
  renderInvGear(e);
  dlg.querySelector('[data-act="close"]').addEventListener("click", () => closeInventory(true));
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    if (invDetail && narrow()) {
      invDetail = false;
      renderInvGear(lastInv ?? e);
      return;
    }
    closeInventory(true);
  });
  sfx.play("bag");
  dlg.showModal();
  dlg.querySelector(".pick-row[aria-selected='true']")?.focus();
}

// Smal skjerm: lista alene, og detaljene i stedet for lista med «Tilbake».
const narrow = () => innerWidth <= 720;

// Lykta som en ting i plassen «Lys»: oljen og hvor lenge den brenner
// (`lantern::FULL` er 8 minutter).
function lanternItem(e) {
  const light = lastHud?.light ?? 100;
  const min = Math.round((light / 100) * 8);
  return {
    id: e.lantern ? "search_lantern" : "",
    name: e.lantern ? "Søkelykt" : "Lykta i beltet",
    slot: "lantern",
    rarity: e.lantern ? "sjelden" : "vanlig",
    special: "",
    about: e.lantern ? "Lykt med slipt glass som kaster lyset dobbelt så langt inn i mørket." : "Lykta du bærer i beltet. Oljen brenner bare nede i krypten; fyll den hos Apotekeren eller på hvilestedene.",
    stats: [{ label: "Olje", value: light / 100, max: 1, text: `${light} %`, up: true }, { label: "Brenner", value: min, max: 8, text: min ? `ca. ${min} min` : "tom", up: true }, { label: "Bærelyset", value: e.lantern ? 2.2 : 1, max: 2.2, text: e.lantern ? "×2,2" : "×1,0", up: true }],
    equipped: true,
    owned: true,
    light,
  };
}

function renderInvGear(e) {
  const dlg = invOpen;
  if (!dlg) return;
  const h = lastHud;
  dlg.querySelector(".inv-stats").innerHTML = `
    <label class="lt-label">Liv <span class="bar-num">${h.hp} / ${e.max_hp}</span><progress class="lt-progress" data-color="danger" value="${h.hp}" max="${e.max_hp}"></progress></label>
    <label class="lt-label">Energi <span class="bar-num">${h.stamina} / ${e.max_stamina}</span><progress class="lt-progress" value="${h.stamina}" max="${e.max_stamina}"></progress></label>
    <span class="lt-badge" data-color="warning">${e.tokens} tokens</span>
    <span class="lt-badge" data-color="neutral">${h.keys === 1 ? "1 nøkkel" : `${h.keys ?? 0} nøkler`}</span>`;
  const gear = e.gear ?? [];
  const lantern = lanternItem(e);
  const worn = (slot) => (slot === "lantern" ? lantern : gear.find((g) => g.slot === slot && g.equipped) ?? (slot === "armor" ? NO_ARMOR : null));
  const slotOf = { weapon: 0, shield: 1, armor: 2, lantern: 3 };
  const all = [...gear, lantern]
    .filter((g) => g.slot === invFilter)
    .sort((a, b) => slotOf[a.slot] - slotOf[b.slot] || Number(b.equipped) - Number(a.equipped) || rarityRank(b.rarity) - rarityRank(a.rarity) || a.name.localeCompare(b.name, "nb"));
  // Først det første du ikke har på, så det står mot det du har på med en
  // gang; ellers det du har på.
  if (!all.some((g) => itemKey(g) === invSelected)) invSelected = itemKey(all.find((g) => !g.equipped) ?? all[0] ?? {});
  const sel = all.find((g) => itemKey(g) === invSelected);
  // Plassene: det du har på, og filteret.
  const slotCard = (slot, label, item, empty) => {
    const [name, r] = splitRarity(item?.name ?? "");
    const rarity = item?.rarity || r;
    const real = item && item.id !== "" || slot === "lantern";
    return `<button type="button" class="slot-card" data-slot="${slot}" data-rarity="${esc(rarity || "vanlig")}" aria-pressed="${invFilter === slot}" title="${esc(label)}: ${esc(real ? name : empty)}" ${real ? "" : 'data-empty=""'}>
        ${real ? gearIcon(item, "small") : `<span class="gear-ico small silhouette" aria-hidden="true">${SILHOUETTES[slot]}</span>`}
        <span class="slot-text">
          <span class="eyebrow">${label}</span>
          <span class="slot-name">${esc(real ? name : empty)}</span>
          ${slot === "lantern" ? `<progress class="lt-progress mini" data-color="warning" max="100" value="${lantern.light}" aria-label="Olje"></progress>` : ""}
        </span>
      </button>`;
  };
  const wornName = (slot) => worn(slot) ?? null;
  const counts = new Map();
  for (const [c, n] of e.stash) counts.set(c, { stash: n, carried: 0 });
  for (const [c, n] of e.carried) counts.set(c, { stash: counts.get(c)?.stash ?? 0, carried: n });
  const panel = dlg.querySelector(".inv-gear-panel");
  const hadFocus = panel.contains(document.activeElement) && document.activeElement.classList.contains("pick-row");
  panel.innerHTML = `<div class="inv-layout picker${invDetail ? " show-detail" : ""}">
      <div class="inv-slots" role="group" aria-label="Det du har på (velg for å vise bare det slaget)">
        ${slotCard("weapon", "Våpen", wornName("weapon"), "")}
        ${slotCard("shield", "Skjold", wornName("shield"), "")}
        ${slotCard("armor", "Rustning", worn("armor") === NO_ARMOR ? null : worn("armor"), "Finn en rustning, eller få den smidd hos Smeden")}
        ${slotCard("lantern", "Lys", lantern, "")}
      </div>
      <div class="pick-side">
        <p class="pick-count">${SLOT_NAMES[invFilter]} <span class="tab-count">${all.length}</span></p>
        <ul class="pick-list" role="listbox" aria-label="Utstyret ditt">
          ${all.map((g) => pickRow(itemKey(g), g, g.equipped ? '<span class="lt-badge" data-color="success">På deg</span>' : "", invSelected === itemKey(g))).join("")}
        </ul>
      </div>
      <section class="detail" aria-live="polite">${sel ? invDetailHtml(sel, worn(sel.slot), [...gear, NO_ARMOR]) : ""}</section>
    </div>`;
  const tab = dlg.querySelector(".inv-comp-tab");
  tab.innerHTML = `Komponenter <span class="tab-count">${counts.size}</span>`;
  dlg.querySelector(".inv-comps").innerHTML = counts.size
    ? `<div class="menu-scroll"><table class="lt-table" data-variant="striped">
        <caption>Komponentene du har vunnet. Det du bærer, er ditt først når du er oppe igjen.</caption>
        <thead><tr><th scope="col">Komponent</th><th scope="col" data-align="end">I lageret</th><th scope="col" data-align="end">Bærer</th><th scope="col">Boka</th><th scope="col"></th></tr></thead>
        <tbody>${[...counts.entries()]
          .sort((a, b) => compTitle(a[0]).localeCompare(compTitle(b[0]), "nb"))
          .map(([c, k]) => `<tr>
            <th scope="row"><span class="comp-name"><img class="comp-ico" src="components/${esc(c)}.jpg" alt="" loading="lazy" onerror="this.hidden=true">${esc(compTitle(c))}</span></th>
            <td data-align="end">${k.stash}</td>
            <td data-align="end">${k.carried ? `<span class="lt-badge" data-color="warning">${k.carried}</span>` : "0"}</td>
            <td>${e.read.includes(c) ? '<span class="lt-badge" data-color="success">Lest</span>' : '<span class="lt-badge" data-color="neutral">Ulest</span>'}</td>
            <td data-align="end">${docs[c] ? `<button class="lt-button" data-variant="secondary" type="button" data-read="${esc(c)}">Les</button>` : ""}</td>
          </tr>`)
          .join("")}</tbody>
      </table></div>`
    : `<p class="lt-paragraph">Du har ingen komponenter ennå. De ligger i kistene og hos fiendene i krypten.</p>`;
  for (const b of dlg.querySelectorAll("[data-read]")) {
    b.addEventListener("click", () => {
      const c = b.dataset.read;
      closeInventory(false);
      readDoc(c);
    });
  }
  for (const b of panel.querySelectorAll("[data-slot]")) {
    b.addEventListener("click", () => {
      invFilter = b.dataset.slot;
      remember("fristad.inventory.slot", invFilter);
      invSelected = null;
      invDetail = false;
      sfx.play("click");
      renderInvGear(lastInv ?? e);
    });
  }
  const equip = (key) => {
    const g = all.find((x) => itemKey(x) === key);
    if (!g || g.equipped || g.slot === "lantern") return;
    if (engine.equip(g.id)) sfx.play("click");
  };
  bindPickList(panel, {
    select(key, open) {
      invSelected = key;
      if (open) invDetail = true;
      sfx.play("click");
      renderInvGear(lastInv ?? e);
    },
    activate: equip,
  });
  panel.querySelector("[data-equip]")?.addEventListener("click", (ev) => equip(ev.currentTarget.dataset.equip));
  panel.querySelector("[data-back]")?.addEventListener("click", () => {
    invDetail = false;
    renderInvGear(lastInv ?? e);
  });
  const row = panel.querySelector(".pick-row[aria-selected='true']");
  row?.scrollIntoView({ block: "nearest" });
  if (hadFocus) row?.focus();
}

// Detaljene for en ting i inventaret: beskrivelsen, sammenligningen og
// «Bruk <navn>» rett under tabellen.
function invDetailHtml(sel, worn, pool) {
  const [name] = splitRarity(sel.name);
  const same = !worn || itemKey(worn) === itemKey(sel);
  const action =
    sel.slot === "lantern"
      ? `<p class="lt-help-text">${sel.id ? "Søkelykta henger i beltet." : "En søkelykt smis hos Smeden."} Olje får du hos Apotekeren og på hvilestedene.</p>`
      : `<div class="detail-actions">
          ${sel.equipped
            ? '<span class="lt-help-text">Du har den på deg. Velg noe annet i lista for å se det mot denne.</span>'
            : `<button type="button" class="lt-button" data-variant="primary" data-equip="${esc(sel.id)}">Bruk ${esc(name)}</button>
               <span class="lt-help-text">Bytter ut ${esc(splitRarity(worn?.name ?? "")[0])}. Dobbeltklikk eller Enter i lista gjør det samme.</span>`}
        </div>`;
  return `<div class="detail-scroll">
      <button type="button" class="lt-button detail-back" data-variant="ghost" data-back>← Tilbake til lista</button>
      ${detailHead(sel)}
      <h4 class="lt-heading detail-sub" data-size="xs">${same ? "Verdiene" : `Mot ${esc(splitRarity(worn?.name ?? "")[0])}, som du har på`}</h4>
      ${compareTable(sel, same ? null : worn, pool)}
      ${action}
    </div>`;
}

function closeInventory(resume) {
  if (!invOpen) return;
  invOpen.close();
  invOpen.remove();
  invOpen = null;
  if (resume) enter();
}

// Innstillingene (O, eller knappen på startkortet): grafikk og oppløsning
// som LO-TEK-radioknapper i en dialog. Ingenting byttes uten at
// spilleren velger det.
// Innstillingene og grafikken kan alltid lukkes: med knappen, krysset i
// hjørnet, et klikk utenfor vinduet eller Esc (eieren 18. oktober: vinduet
// lot seg ikke lukke).
function dismissable(dlg, close) {
  const x = document.createElement("button");
  x.type = "button";
  x.className = "lt-button dialog-x";
  x.dataset.variant = "ghost";
  x.setAttribute("aria-label", "Lukk");
  x.textContent = "×";
  x.addEventListener("click", close);
  dlg.prepend(x);
  dlg.addEventListener("click", (ev) => {
    if (ev.target !== dlg) return;
    const r = dlg.getBoundingClientRect();
    if (ev.clientX < r.left || ev.clientX > r.right || ev.clientY < r.top || ev.clientY > r.bottom) close();
  });
}

let settingsOpen = null;
function openSettings() {
  if (settingsOpen) return;
  const wasPlaying = playing;
  playing = false;
  dragging = false;
  engine?.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const dpr = window.devicePixelRatio || 1;
  const ph = Math.round(canvas.clientHeight * dpr);
  const radio = (name, value, label, note, checked, disabled = false) => `<label class="lt-radio-row">
      <input class="lt-radio" type="radio" name="${name}" value="${value}" ${checked ? "checked" : ""} ${disabled ? "disabled" : ""}>
      <span>${label}${note ? ` <span class="lt-help-text">${note}</span>` : ""}</span>
    </label>`;
  const steps = RES_STEPS.map((h) => {
    const w = Math.round((h * canvas.clientWidth) / Math.max(1, canvas.clientHeight));
    const px = ((w * h) / 1e6).toFixed(1);
    return radio("res", h, `${w} × ${h} (${RES_NAMES[h]})`, `${px} Mpx${h > ph ? ", større enn skjermen" : ""}`, resolution === h, h > ph);
  }).join("");
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog settings-dialog";
  dlg.setAttribute("aria-labelledby", "settings-title");
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="settings-title">Innstillinger</h2>
    <div class="lt-dialog__body">
      <fieldset class="lt-fieldset">
        <legend class="lt-legend">Grafikk</legend>
        ${QUALITIES.map((q) => radio("gfx", q, q[0].toUpperCase() + q.slice(1), QUALITY_INFO[q].note, !custom && quality === q)).join("")}
        ${custom ? radio("gfx", "custom", "Egendefinert", `med ${quality} som utgangspunkt.`, true) : ""}
        <p><button class="lt-button" data-variant="secondary" type="button" data-act="graphics">Avanserte grafikkinnstillinger …</button></p>
      </fieldset>
      <fieldset class="lt-fieldset">
        <legend class="lt-legend">Oppløsning</legend>
        ${radio("res", "auto", "Automatisk", `største trinn som får plass, høyst ${QUALITY_INFO[quality ?? "standard"].cap}p.`, resolution === "auto")}
        ${steps}
      </fieldset>
      <p class="lt-help-text" id="set-res-info">Vinduet er ${Math.round(canvas.clientWidth * dpr)} × ${ph} piksler. Spillet tegnes nå i ${canvas.width} × ${canvas.height}.</p>
      <fieldset class="lt-fieldset">
        <legend class="lt-legend">Mus, lysstyrke og tekst</legend>
        <label class="lt-label range-row" for="set-mouse">Musefølsomhet <output id="set-mouse-out">${mouseSens.toFixed(2)}×</output></label>
        <input class="range" id="set-mouse" name="mouse" type="range" min="0.2" max="3" step="0.05" value="${mouseSens}">
        <label class="lt-label range-row" for="set-gamma">Lysstyrke <output id="set-gamma-out">${gamma.toFixed(2)}</output></label>
        <input class="range" id="set-gamma" name="gamma" type="range" min="0.6" max="2" step="0.05" value="${gamma}">
        <p class="lt-help-text">Er skjermen mørk, så du ikke ser noe i krypten, skru lysstyrken opp. 1,00 er slik bildet er malt.</p>
        <label class="lt-label range-row" for="set-text">Tekststørrelse <output id="set-text-out">${Math.round(textSize * 100)} %</output></label>
        <input class="range" id="set-text" name="textsize" type="range" min="0.9" max="1.4" step="0.05" value="${textSize}">
        <p><button class="lt-button" data-variant="secondary" data-size="small" type="button" data-act="reset-input">Tilbakestill</button></p>
      </fieldset>
    </div>
    <div class="lt-dialog__footer">
      <button class="lt-button" data-variant="primary" type="button" data-act="close">Lukk (O)</button>
    </div>`;
  document.body.appendChild(dlg);
  settingsOpen = dlg;
  const setSens = (v) => {
    mouseSens = v;
    remember("fristad.mouse", v);
    dlg.querySelector("#set-mouse").value = v;
    dlg.querySelector("#set-mouse-out").textContent = `${v.toFixed(2)}×`;
  };
  const setText = (v) => {
    textSize = v;
    remember("fristad.textsize", v);
    setTextSize(v);
    dlg.querySelector("#set-text").value = v;
    dlg.querySelector("#set-text-out").textContent = `${Math.round(v * 100)} %`;
  };
  const setGamma = (v) => {
    gamma = v;
    remember("fristad.gamma", v);
    engine?.set_gamma(v);
    dlg.querySelector("#set-gamma").value = v;
    dlg.querySelector("#set-gamma-out").textContent = v.toFixed(2);
  };
  dlg.addEventListener("input", (ev) => {
    if (ev.target.name === "mouse") setSens(Number(ev.target.value));
    if (ev.target.name === "gamma") setGamma(Number(ev.target.value));
    if (ev.target.name === "textsize") setText(Number(ev.target.value));
  });
  dlg.querySelector('[data-act="reset-input"]').addEventListener("click", () => {
    setSens(1);
    setGamma(1);
    setText(1);
    sfx.play("click");
  });
  dlg.addEventListener("change", (ev) => {
    const t = ev.target;
    if (t.name === "mouse" || t.name === "gamma" || t.name === "textsize") return sfx.play("click");
    if (t.name === "gfx" && t.value !== "custom") setQuality(t.value);
    if (t.name === "res") setResolution(t.value === "auto" ? "auto" : Number(t.value));
    dlg.querySelector("#set-res-info").textContent = `Vinduet er ${Math.round(canvas.clientWidth * dpr)} × ${ph} piksler. Spillet tegnes nå i ${canvas.width} × ${canvas.height}.`;
    sfx.play("click");
  });
  const close = () => {
    if (settingsOpen !== dlg) return;
    settingsOpen.close();
    settingsOpen.remove();
    settingsOpen = null;
    if (wasPlaying) enter();
    else if (!gateEl.hidden) enterBtn.focus();
  };
  dlg.querySelector('[data-act="close"]').addEventListener("click", close);
  dismissable(dlg, close);
  dlg.querySelector('[data-act="graphics"]').addEventListener("click", () => {
    settingsOpen.close();
    settingsOpen.remove();
    settingsOpen = null;
    openGraphics(wasPlaying);
  });
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    close();
  });
  dlg.showModal();
}

// De avanserte grafikkinnstillingene (knappen «Grafikk» i pausemenyen, eller
// fra Innstillinger): hver bryter for seg, med hva den koster. Forhåndsvalgene
// setter bryterne; endres én, står det «Egendefinert». Ingenting byttes uten
// at spilleren velger det. Kostnadene er målt på grafikkortet
// (docs/kvalitet/ytelse.md).
const GRAPHICS_ROWS = [
  { key: "scale", label: "Oppløsningsskala", note: "andelen av oppløsningen som tegnes; resten skaleres opp.", options: [[0.5, "50 %"], [0.67, "67 %"], [0.75, "75 %"], [0.85, "85 %"], [1, "100 %"]] },
  { key: "shadow", label: "Solskygge", note: "skyggene fra sola og månen." },
  { key: "shadow_res", label: "Skyggeoppløsning", note: "skarpere skygger med større kart.", options: [[1024, "1024"], [2048, "2048"], [4096, "4096"]] },
  { key: "soft_shadow", label: "Myke skygger", note: "tolv faste prøver langs kanten i stedet for fire, uten støy." },
  { key: "far_shadow", label: "Skygge langt unna", note: "en kaskade til, ut til 110 m." },
  { key: "ao", label: "Skygge i krokene", note: "mørkere i hjørner og under ting (bare WebGPU).", webgpu: true },
  { key: "bloom", label: "Glød", note: "lys, ild og vinduer gløder." },
  { key: "rays", label: "Solstråler", note: "lyset står i stråler rundt sola." },
  { key: "smaa", label: "Kantutjevning (SMAA)", note: "glatte kanter uten uskarphet." },
  { key: "grade", label: "Malt fargegradering", note: "varm by, kald kjeller og myk overgang mot hvitt." },
  { key: "fog", label: "Avstandståke", note: "lufta mellom deg og det som er langt unna." },
  { key: "far", label: "Sikt", note: "hvor langt unna modellene tegnes.", options: [[120, "120 m"], [160, "160 m"], [250, "250 m"], [0, "Hele verden"]] },
  { key: "detail", label: "Småting langt unna", note: "småting og fin detalj også på avstand." },
  { key: "lights", label: "Lys samtidig", note: "lykter, ild og vinduer som lyser opp rundt seg.", options: [[4, "4"], [8, "8"], [16, "16"], [48, "48"]] },
  { key: "textures", label: "Full teksturtetthet", note: "av: ett mipnivå ned, halv tetthet." },
  { key: "ground", label: "Blandet bakke", note: "brostein, jord og gress glir ujevnt over i hverandre." },
  { key: "particles", label: "Røyk og nedbør", note: "røyk fra pipene, regn og snø." },
];
let graphicsOpen = null;
function openGraphics(resume = playing) {
  if (graphicsOpen) return;
  const cur = currentGraphics();
  // Før motoren er lastet finnes ikke bryterne ennå: da de vanlige innstillingene.
  if (!cur) {
    openSettings();
    return;
  }
  const wasPlaying = resume;
  playing = false;
  dragging = false;
  engine?.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const gl = engine?.backend() === "Gl";
  const row = (r) => {
    const id = `gfx-${r.key}`;
    const off = r.webgpu && gl;
    const note = `<span class="lt-help-text">${r.note}${off ? " Ikke i WebGL2." : ""}</span>`;
    if (r.options) {
      return `<div class="gfx-row"><label class="lt-label" for="${id}">${r.label}</label>
        <select class="lt-select" id="${id}" data-key="${r.key}">${r.options.map(([v, t]) => `<option value="${v}">${t}</option>`).join("")}</select>${note}</div>`;
    }
    return `<div class="lt-switch-row gfx-row"><input class="lt-switch" type="checkbox" role="switch" id="${id}" data-key="${r.key}" ${off ? "disabled" : ""}>
      <label class="lt-label" for="${id}">${r.label} ${note}</label></div>`;
  };
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog settings-dialog graphics-dialog";
  dlg.setAttribute("aria-labelledby", "graphics-title");
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="graphics-title">Grafikk</h2>
    <div class="lt-dialog__body">
      <fieldset class="lt-fieldset">
        <legend class="lt-legend">Forhåndsvalg</legend>
        <div class="gfx-presets">
          ${QUALITIES.map((q) => `<label class="lt-radio-row"><input class="lt-radio" type="radio" name="gfx-preset" value="${q}"><span>${q[0].toUpperCase() + q.slice(1)}</span></label>`).join("")}
          <label class="lt-radio-row"><input class="lt-radio" type="radio" name="gfx-preset" value="custom" disabled><span>Egendefinert</span></label>
        </div>
      </fieldset>
      <fieldset class="lt-fieldset gfx-switches">
        <legend class="lt-legend">Brytere</legend>
        ${GRAPHICS_ROWS.map(row).join("")}
      </fieldset>
      <p class="lt-help-text gfx-now"></p>
    </div>
    <div class="lt-dialog__footer">
      <button class="lt-button" data-variant="primary" type="button" data-act="close">Lukk</button>
    </div>`;
  document.body.appendChild(dlg);
  graphicsOpen = dlg;
  const show = () => {
    const v = currentGraphics();
    for (const el of dlg.querySelectorAll("[data-key]")) {
      const k = el.dataset.key;
      if (el.type === "checkbox") el.checked = !!v[k] && !(el.disabled && k === "ao");
      else el.value = String(v[k]);
    }
    for (const el of dlg.querySelectorAll('[name="gfx-preset"]')) el.checked = el.value === (custom ? "custom" : (quality ?? "standard"));
    dlg.querySelector(".gfx-now").textContent = `Spillet tegnes nå i ${canvas.width} × ${canvas.height}.`;
  };
  show();
  dlg.addEventListener("change", (ev) => {
    const t = ev.target;
    if (t.name === "gfx-preset") {
      if (t.value !== "custom") setQuality(t.value, true, false);
    } else if (t.dataset.key) {
      const v = { ...currentGraphics() };
      // En bryter som ikke kan brukes her (skygge i krokene i WebGL2) beholder verdien.
      for (const el of dlg.querySelectorAll("[data-key]")) if (!el.disabled) v[el.dataset.key] = el.type === "checkbox" ? el.checked : Number(el.value);
      setCustom(v);
    }
    show();
    sfx.play("click");
  });
  const close = () => {
    if (graphicsOpen !== dlg) return;
    graphicsOpen.close();
    graphicsOpen.remove();
    graphicsOpen = null;
    if (wasPlaying) enter();
    else if (!gateEl.hidden) enterBtn.focus();
  };
  dlg.querySelector('[data-act="close"]').addEventListener("click", close);
  dismissable(dlg, close);
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    close();
  });
  dlg.showModal();
}

// Butikk og verksted: en LO-TEK-dialog midt på skjermen med spillet på
// pause mens den står. Den ligger i HUD-en, så den tar imot musa selv.
let overlayOpen = false;
function openOverlay(title, subtitle = "") {
  overlayOpen = true;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  $("overlay-title").textContent = title;
  $("overlay-sub").textContent = subtitle;
  $("overlay-sub").hidden = !subtitle;
  $("overlay").querySelector(".overlay-inner").classList.remove("menu");
  $("overlay-body").replaceChildren();
  $("overlay-actions").replaceChildren();
  $("overlay").hidden = false;
  return $("overlay-body");
}

// Lukkes med knappen eller E (en brukerhandling), fanges musa igjen med en
// gang, så du kan se deg rundt uten å klikke. Med Esc gir ikke nettleseren
// oss musa; da sier en melding at et klikk i bildet tar deg tilbake.
function closeOverlay(resume = true, relock = true) {
  if (!overlayOpen) return;
  forgeSkip?.();
  overlayOpen = false;
  $("overlay").hidden = true;
  if (resume) enter(relock);
  if (resume && !relock) clickHint();
}

// Butikken: én rad per vare (navnet, prisen og «Kjøp»). Lista ruller inne i
// dialogen når den er lang (plan 13c). Skriptoriet har også byttene
// (plan 14): da står varene og byttene i hver sin fane (`openSmith`).
function openShop(npc, wares, swaps = []) {
  if (swaps.length) return openSmith(npc, wares, swaps);
  const body = openOverlay(npc, "");
  body.appendChild(wareList(wares));
  updateShop();
}

function wareList(wares) {
  const list = document.createElement("ul");
  list.className = "lt-list wares";
  list.dataset.variant = "divided";
  for (const w of wares) {
    const row = document.createElement("li");
    row.className = "ware";
    const text = document.createElement("div");
    text.className = "ware-text";
    const name = document.createElement("span");
    name.className = "ware-name";
    name.textContent = w.name;
    const price = document.createElement("span");
    price.className = "ware-price";
    price.textContent = `${w.price} tokens`;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "lt-button";
    btn.dataset.variant = "secondary";
    btn.dataset.price = String(w.price);
    btn.textContent = "Kjøp";
    btn.setAttribute("aria-label", `Kjøp ${w.name} for ${w.price} tokens`);
    const why = document.createElement("span");
    why.className = "lt-help-text ware-why";
    why.id = `ware-why-${w.id}`;
    btn.setAttribute("aria-describedby", why.id);
    btn.addEventListener("click", () => {
      if (btn.disabled) return;
      engine.buy(w.id);
    });
    text.append(name, why);
    // Varen med ikon (`icons/<id>.png`, helsedrikkene); uten ikon står bare teksten.
    const main = document.createElement("div");
    main.className = "ware-main";
    const ico = document.createElement("img");
    ico.className = "ware-ico";
    ico.src = `icons/${w.id}.png`;
    ico.alt = "";
    ico.loading = "lazy";
    ico.onerror = () => ico.remove();
    main.append(ico, text);
    row.append(main, price, btn);
    list.appendChild(row);
  }
  return list;
}

// Skriptoriet (plan 14; fra Buttonsmeden 26. oktober): fanene «Kjøp» og
// «Bytte», som Smia. I
// «Bytte» står ett bytte per rad (det du får, sjeldenheten og hva du gir),
// de du kan gjøre nå øverst; detaljene viser hva du gir og har, avgiften og
// hvor komponenten ellers finnes, og «Bytt» står fast i bunnlinja. Etter et
// bytte sender motoren butikken på nytt, så tallene er de nye.
let shopTab = remembered("fristad.shop.tab", "swap");
let swapSel = null;
function openSmith(npc, wares, swaps) {
  const body = openOverlay(npc, `Du har ${lastHud?.tokens ?? 0} tokens.`);
  $("overlay").querySelector(".overlay-inner").classList.add("menu");
  let detail = false;
  const lack = (s) => Math.max(0, s.n - s.have);
  const sorted = [...swaps].sort((a, b) => Number(b.ok) - Number(a.ok) || lack(a) - lack(b) || rarityRank(a.get_rarity) - rarityRank(b.get_rarity));
  const tabs = [["buy", "Kjøp", wares.length], ["swap", "Bytte", swaps.length]];
  if (!tabs.some(([k]) => k === shopTab)) shopTab = "swap";
  const badge = (s) => (s.ok ? '<span class="lt-badge" data-color="success">Kan byttes</span>' : lack(s) ? `<span class="lt-badge" data-color="warning">Mangler ${lack(s)}</span>` : '<span class="lt-badge" data-color="warning">Mangler tokens</span>');
  const compIco = (c, size, r) => `<span class="gear-ico ${size} comp-face" data-rarity="${esc(r)}"><img src="components/${esc(c)}.jpg" alt="" loading="lazy" onerror="this.hidden=true"></span>`;
  const wrap = document.createElement("div");
  wrap.className = "smith picker";
  body.appendChild(wrap);
  const actions = $("overlay-actions");
  const swapDetail = (s) => {
    const tokens = lastHud?.tokens ?? 0;
    const why = s.ok ? `Du gir ${s.n} ${compTitle(s.give)} og ${s.fee} tokens.` : lack(s) ? `Mangler ${lack(s)} ${compTitle(s.give)} i lageret.` : `Mangler ${s.fee - tokens} tokens til avgiften.`;
    actions.innerHTML = narrow() && !detail ? "" : `<button type="button" class="lt-button" data-variant="primary" data-swap="${esc(s.id)}" ${s.ok ? "" : "disabled"} aria-describedby="swap-why"><span>Bytt<span class="craft-name"> til ${esc(compTitle(s.get))}</span></span></button>
        <span class="lt-help-text" id="swap-why">${esc(why)}</span>`;
    const giveOk = s.have >= s.n;
    const feeOk = tokens >= s.fee;
    return `<div class="detail-scroll">
        <button type="button" class="lt-button detail-back" data-variant="ghost" data-back>← Tilbake til lista</button>
        <header class="detail-head" data-rarity="${esc(s.get_rarity)}">
          ${compIco(s.get, "big", s.get_rarity)}
          <div class="detail-title">
            <h3 class="lt-heading" data-size="m">${esc(compTitle(s.get))}</h3>
            <div class="gear-tags">${rarityTag(s.get_rarity)}<span class="lt-tag">Komponent</span>${badge(s)}</div>
          </div>
        </header>
        <p class="lt-paragraph detail-about">${esc(docs[s.get]?.description ?? "")}</p>
        <h4 class="lt-heading detail-sub" data-size="xs">Du gir</h4>
        <ul class="comp-list">
          <li class="comp ${giveOk ? "have" : "lack"}">
            <img class="comp-ico" src="components/${esc(s.give)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
            <div class="comp-main">
              <div class="comp-top"><span class="comp-title">${s.n} × ${esc(compTitle(s.give))}</span>${rarityTag(s.give_rarity)}<span class="comp-count">har ${s.have} av ${s.n}</span></div>
              <progress class="lt-progress mini" data-color="${giveOk ? "success" : "warning"}" max="${s.n}" value="${Math.min(s.have, s.n)}" aria-hidden="true"></progress>
            </div>
          </li>
          <li class="comp ${feeOk ? "have" : "lack"}">
            <span class="comp-ico token-ico" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 2.5l8.2 4.75v9.5L12 21.5l-8.2-4.75v-9.5z"/></svg></span>
            <div class="comp-main">
              <div class="comp-top"><span class="comp-title">Avgift: ${s.fee} tokens</span><span class="comp-count">har ${tokens}</span></div>
            </div>
          </li>
        </ul>
        <h4 class="lt-heading detail-sub" data-size="xs">Du får</h4>
        <ul class="comp-list">
          <li class="comp have">
            <img class="comp-ico" src="components/${esc(s.get)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
            <div class="comp-main">
              <div class="comp-top"><span class="comp-title">1 × ${esc(compTitle(s.get))}</span><span class="comp-count">har ${s.get_have} nå</span></div>
              <p class="comp-where">Ellers: ${esc(s.source)}</p>
            </div>
          </li>
        </ul>
        <p class="lt-help-text swap-rule">Byttet går bare én vei, og smeden kjøper ingenting tilbake.</p>
      </div>`;
  };
  const render = () => {
    const hadFocus = wrap.contains(document.activeElement) && document.activeElement.classList.contains("pick-row");
    actions.innerHTML = "";
    if (shopTab === "buy") {
      wrap.classList.remove("show-detail");
      wrap.innerHTML = `${menuTabs(tabs, shopTab, "Butikken")}<div class="shop-buy"></div>`;
      wrap.querySelector(".shop-buy").appendChild(wareList(wares));
      updateShop();
    } else {
      if (!sorted.some((s) => s.id === swapSel)) swapSel = sorted[0]?.id ?? null;
      const sel = sorted.find((s) => s.id === swapSel);
      wrap.classList.toggle("show-detail", detail);
      const detailHtml = sel ? swapDetail(sel) : '<p class="lt-paragraph">Ingen bytter her.</p>';
      wrap.innerHTML = `${menuTabs(tabs, shopTab, "Butikken")}
        <div class="pick-side">
          <ul class="pick-list" role="listbox" aria-label="Byttene">
            ${sorted
              .map(
                (s) => `<li class="pick-row" role="option" data-key="${esc(s.id)}" data-rarity="${esc(s.get_rarity)}" aria-selected="${s.id === swapSel}" tabindex="${s.id === swapSel ? 0 : -1}">
                ${compIco(s.get, "small", s.get_rarity)}
                <span class="pick-text"><span class="pick-name">${esc(compTitle(s.get))}</span><span class="pick-meta">${rarityTag(s.get_rarity)}<span class="pick-sub">${s.n} × ${esc(compTitle(s.give))} + ${s.fee} t</span>${badge(s)}</span></span>
              </li>`,
              )
              .join("")}
          </ul>
        </div>
        <section class="detail" aria-live="polite">${detailHtml}</section>`;
    }
    bindTabs(wrap, (k) => {
      shopTab = k;
      remember("fristad.shop.tab", k);
      detail = false;
      sfx.play("click");
      render();
      wrap.querySelector(`[data-tab="${k}"]`)?.focus();
    });
    bindPickList(wrap, {
      select(key, open) {
        if (key === swapSel && !open) return;
        swapSel = key;
        if (open) detail = true;
        sfx.play("click");
        render();
      },
    });
    wrap.querySelector("[data-back]")?.addEventListener("click", () => {
      detail = false;
      render();
      wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
    });
    actions.querySelector("[data-swap]")?.addEventListener("click", (ev) => {
      const b = ev.currentTarget;
      if (b.disabled) return;
      // Går byttet, kommer butikken på nytt med de nye tallene.
      if (!engine.swap?.(b.dataset.swap)) sfx.play("deny");
    });
    const row = wrap.querySelector(".pick-row[aria-selected='true']");
    row?.scrollIntoView({ block: "nearest" });
    if (hadFocus) row?.focus();
  };
  render();
  if (!narrow()) wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
}

// Det du ikke har råd til, er en deaktivert LO-TEK-knapp med forklaring.
// Kalles når butikken åpnes og hver gang tokenene endres (etter et kjøp).
function updateShop() {
  const tokens = lastHud?.tokens ?? 0;
  if (!$("overlay").querySelector(".overlay-inner").classList.contains("menu") || $("overlay-body").querySelector(".smith")) {
    $("overlay-sub").textContent = `Du har ${tokens} tokens.`;
    $("overlay-sub").hidden = false;
  }
  for (const btn of document.querySelectorAll("#overlay-body .ware .lt-button[data-price]")) {
    const price = Number(btn.dataset.price);
    const short = price > tokens;
    btn.disabled = short;
    const why = document.getElementById(btn.getAttribute("aria-describedby"));
    if (why) why.textContent = short ? `Har ikke råd (mangler ${price - tokens} tokens)` : "";
  }
}

// Trubaduren (E): det han sier og ett kjøp, et krus mjød, så synger han
// kvadet om Gorvek. Har du ikke råd, er knappen av som i butikkene
// (`updateShop`); synger han allerede, er den av med en forklaring.
function openMead(e) {
  const body = openOverlay(e.npc, "");
  const say = document.createElement("p");
  say.className = "lt-paragraph mead-text";
  say.textContent = `«${e.text}»`;
  const row = document.createElement("div");
  row.className = "ware mead";
  const text = document.createElement("div");
  text.className = "ware-text";
  const why = document.createElement("span");
  why.className = "lt-help-text ware-why";
  why.id = "ware-why-mead";
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "lt-button";
  btn.dataset.variant = "primary";
  btn.textContent = `Kjøp et krus mjød (${e.price} token)`;
  btn.setAttribute("aria-describedby", why.id);
  if (e.singing) {
    btn.disabled = true;
    why.textContent = "Han synger allerede for kruset ditt.";
  } else btn.dataset.price = String(e.price);
  btn.addEventListener("click", () => {
    if (btn.disabled) return;
    if (engine.buy_mead?.()) closeOverlay();
  });
  text.append(why);
  row.append(text, btn);
  body.append(say, row);
  updateShop();
}

// Felles for inventaret og Smia (plan 13a): sjeldenheten i farge
// (LO-TEKs tokens: vanlig grå, sjelden blå, episk lilla, legendarisk gull),
// ikonet (`icons/<id>.png`), raden med oppsummeringen og
// sammenligningstabellen.
const SLOT_NAMES = { weapon: "Våpen", shield: "Skjold", armor: "Rustning", lantern: "Lys" };
const RARITIES = ["vanlig", "sjelden", "episk", "legendarisk"];
const rarityRank = (r) => Math.max(0, RARITIES.indexOf(r));
const cap = (s) => (s ? s[0].toUpperCase() + s.slice(1) : "");
// «Gammelt rundskjold (vanlig)» → ["Gammelt rundskjold", "vanlig"].
function splitRarity(name) {
  const m = /^(.*?)\s*\((vanlig|sjelden|episk|legendarisk)\)$/.exec(name ?? "");
  return m ? [m[1], m[2]] : [name ?? "", ""];
}
const rarityTag = (r) => (r ? `<span class="lt-tag rarity-tag" data-rarity="${esc(r)}">${cap(esc(r))}</span>` : "");
const itemKey = (g) => `${g?.slot ?? ""}:${g?.id ?? ""}`;
const compTitle = (c) => docs[c]?.title ?? c;
const SLOTS = ["weapon", "shield", "armor", "lantern"];
// Det sist valgte slaget huskes mellom gangene (en bekvemmelighet).
function remembered(key, fallback) {
  try {
    return localStorage.getItem(key) || fallback;
  } catch {
    return fallback;
  }
}
// Fanerekken over lista (LO-TEKs `lt-tabs__list`, lenker i en liste): én
// rad som aldri brytes, med antallet som en liten badge. Lista byttes ut
// når en fane velges, så det er ingen paneler å peke på, og klikket skriver
// ikke i adressen. [[nøkkel, navn, antall]].
function menuTabs(tabs, current, label) {
  return `<ul class="lt-tabs__list menu-tabs" role="tablist" aria-label="${esc(label)}">
      ${tabs.map(([k, n, count]) => `<li role="presentation"><a href="#${k}" role="tab" data-tab="${k}" aria-selected="${k === current}" tabindex="${k === current ? 0 : -1}">${esc(n)}<span class="tab-count">${count}</span></a></li>`).join("")}
    </ul>`;
}
function bindTabs(root, pick) {
  const tabs = [...root.querySelectorAll(".menu-tabs [data-tab]")];
  for (const t of tabs)
    t.addEventListener("click", (ev) => {
      ev.preventDefault();
      if (t.getAttribute("aria-selected") !== "true") pick(t.dataset.tab);
    });
  root.querySelector(".menu-tabs")?.addEventListener("keydown", (ev) => {
    const i = tabs.indexOf(document.activeElement);
    if (i < 0 || (ev.key !== "ArrowRight" && ev.key !== "ArrowLeft")) return;
    ev.preventDefault();
    pick(tabs[(i + (ev.key === "ArrowRight" ? 1 : tabs.length - 1)) % tabs.length].dataset.tab);
  });
  root.querySelector('.menu-tabs [aria-selected="true"]')?.scrollIntoView({ block: "nearest", inline: "nearest" });
}
const SILHOUETTES = {
  weapon: '<svg viewBox="0 0 24 24"><path d="M14 3h7v7L10 21l-3-3L18 7M5 15l4 4M3 21l3-3"/></svg>',
  shield: '<svg viewBox="0 0 24 24"><path d="M12 3l8 3v6c0 5-4 8-8 9-4-1-8-4-8-9V6z"/></svg>',
  armor: '<svg viewBox="0 0 24 24"><path d="M8 3l4 2 4-2 4 3-2 4v11H6V10L4 6z"/></svg>',
  lantern: '<svg viewBox="0 0 24 24"><path d="M9 3h6M10 3v3h4V3M7 6h10v13H7zM9 21h6M12 10v5"/></svg>',
};
function gearIcon(item, size) {
  const r = item?.rarity || splitRarity(item?.name)[1] || "vanlig";
  if (!item?.id) return `<span class="gear-ico ${size} silhouette" data-rarity="${esc(r)}" aria-hidden="true">${SILHOUETTES[item?.slot] ?? ""}</span>`;
  return `<span class="gear-ico ${size}" data-rarity="${esc(r)}"><img src="icons/${esc(item.id)}.png" alt="" loading="lazy" onerror="this.hidden=true"></span>`;
}
const NO_ARMOR = { id: "", name: "Ingen rustning", slot: "armor", rarity: "vanlig", special: "", about: "Bare skjorta og kappa du kom til porten i.", equipped: true, owned: true, stats: [
  { label: "Skade du tar", value: 1, text: "100 %", up: false },
  { label: "Utholdenhet tilbake", value: 1, text: "×1.00", up: true },
  { label: "Unnamanøver koster", value: 1, text: "×1.00", up: false },
] };

// Egenskaper der lavt er bra, vises slik at høyt er bra der det går
// («Tåler 75 %» i stedet for «Slipper gjennom 25 %»); ellers følger fargen.
const FLIP = { "Slipper gjennom": "Tåler", "Skade du tar": "Demper skade" };
function shownStat(st) {
  if (FLIP[st.label]) {
    const v = 1 - st.value;
    return { label: FLIP[st.label], value: v, text: `${Math.round(v * 100)} %`, up: true };
  }
  return { ...st, text: String(st.text).replace(/(\d)\.(\d)/g, "$1,$2") };
}
function fmtDelta(st, d) {
  const sign = d > 0 ? "+" : "−";
  const a = Math.abs(d);
  const n = (x, k) => x.toFixed(k).replace(".", ",");
  if (/%$/.test(st.text)) return `${sign}${Math.round(a * 100)} %`;
  if (/ m$/.test(st.text)) return `${sign}${n(a, 1)} m`;
  if (/ s$/.test(st.text)) return `${sign}${n(a, 2)} s`;
  if (/min$/.test(st.text)) return `${sign}${Math.round(a)} min`;
  return `${sign}${n(a, 2)}`;
}
// Hvor god en verdi er mot de andre tingene i samme meny (0–100): lengre
// stolpe er bedre, også der lavt er bra.
function goodness(st, label, pool) {
  if (st.max) return (100 * st.value) / st.max;
  const vals = pool.flatMap((g) => g?.stats ?? []).filter((x) => x.label === label).map((x) => shownStat(x).value);
  if (!vals.length) return 100;
  return st.up ? (100 * st.value) / Math.max(...vals, 1e-6) : (100 * Math.min(...vals)) / Math.max(st.value, 1e-6);
}
function compareTable(sel, other, pool) {
  const cell = (st, label) => `<span class="stat-val">${esc(st.text)}</span><progress class="lt-progress mini" max="100" value="${Math.round(goodness(st, label, pool))}" aria-hidden="true"></progress>`;
  const rows = sel.stats
    .map((raw) => {
      const st = shownStat(raw);
      const oraw = other?.stats.find((x) => x.label === raw.label);
      const o = oraw && shownStat(oraw);
      let delta = "";
      if (o) {
        const d = st.value - o.value;
        if (Math.abs(d) > 1e-4) {
          const good = st.up ? d > 0 : d < 0;
          delta = `<span class="delta ${good ? "better" : "worse"}" aria-label="${good ? "bedre" : "dårligere"}">${fmtDelta(st, d)}</span>`;
        }
      }
      return `<tr><th scope="row">${esc(st.label)}${st.up ? "" : ' <span class="low-good" title="Lavt er bra">(lavt er bra)</span>'}</th>${other ? `<td data-align="end">${o ? cell(o, raw.label) : "–"}</td>` : ""}<td data-align="end">${cell(st, raw.label)}</td>${other ? `<td data-align="end" class="delta-cell">${delta}</td>` : ""}</tr>`;
    })
    .join("");
  const otherName = splitRarity(other?.name)[0];
  return `<table class="lt-table compare">
      <caption class="lt-sr-only">${other ? `${esc(splitRarity(sel.name)[0])} mot ${esc(otherName)}` : "Verdiene"}</caption>
      <thead><tr><th scope="col">Egenskap</th>${other ? `<th scope="col" data-align="end">Har på</th>` : ""}<th scope="col" data-align="end">${other ? "Denne" : "Verdi"}</th>${other ? '<th scope="col" data-align="end">Forskjell</th>' : ""}</tr></thead>
      <tbody>${rows}</tbody>
    </table>`;
}
function detailHead(item, badge = "") {
  const [name, r] = splitRarity(item.name);
  const rarity = item.rarity || r;
  return `<header class="detail-head" data-rarity="${esc(rarity)}">
      ${gearIcon(item, "big")}
      <div class="detail-title">
        <h3 class="lt-heading" data-size="m">${esc(name)}</h3>
        <div class="gear-tags">${rarityTag(rarity)}<span class="lt-tag">${SLOT_NAMES[item.slot] ?? ""}</span>${item.equipped && item.slot !== "lantern" ? '<span class="lt-badge" data-color="success">På deg</span>' : ""}${badge}</div>
        ${item.special ? `<p class="detail-special"><span class="lt-tag" data-variant="filled">${esc(cap(item.special))}</span></p>` : ""}
      </div>
    </header>
    ${item.about ? `<p class="lt-paragraph detail-about">${esc(item.about)}</p>` : ""}`;
}
// Én rad i lista: ikonet, navnet, sjeldenheten og et merke.
function pickRow(key, item, badge, selected, sub = "") {
  const [name, r] = splitRarity(item.name);
  const rarity = item.rarity || r;
  return `<li class="pick-row" role="option" data-key="${esc(key)}" data-rarity="${esc(rarity)}" aria-selected="${selected}" tabindex="${selected ? 0 : -1}">
      ${gearIcon(item, "small")}
      <span class="pick-text"><span class="pick-name">${esc(name)}</span><span class="pick-meta">${rarityTag(rarity)}${sub ? `<span class="pick-sub">${esc(sub)}</span>` : ""}${badge}</span></span>
    </li>`;
}
// Musa og tastene i lista: klikk velger (på smal skjerm åpner det
// detaljene), to klikk eller Enter bruker, piltastene flytter.
let lastPick = { key: "", at: 0 };
function bindPickList(root, { select, activate }) {
  const list = root.querySelector(".pick-list");
  if (!list) return;
  const rows = [...list.querySelectorAll(".pick-row")];
  for (const row of rows) {
    row.addEventListener("click", () => {
      const key = row.dataset.key;
      const now = performance.now();
      if (activate && lastPick.key === key && now - lastPick.at < 450) {
        lastPick = { key: "", at: 0 };
        activate(key);
        return;
      }
      lastPick = { key, at: now };
      select(key, narrow());
    });
  }
  list.addEventListener("keydown", (ev) => {
    const i = rows.indexOf(document.activeElement);
    if (i < 0) return;
    if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
      ev.preventDefault();
      const next = rows[Math.min(rows.length - 1, Math.max(0, i + (ev.key === "ArrowDown" ? 1 : -1)))];
      next.focus();
      select(next.dataset.key, false);
    } else if (ev.key === "Enter") {
      ev.preventDefault();
      if (activate) activate(rows[i].dataset.key);
      else select(rows[i].dataset.key, true);
    } else if (ev.key === " ") {
      ev.preventDefault();
      select(rows[i].dataset.key, true);
    }
  });
}

// Smie-animasjonen (plan 13c): komponentene flyr fra lista inn mot midten,
// lyser i LO-TEKs fyllfarger og smelter sammen i en glød, hammeren slår
// to ganger (lyden `craft`), og ikonet for det nye kommer fram med en kort
// blink. Rundt 1,8 s; klikk, Esc, E, Enter eller mellomrom hopper over.
// Med `prefers-reduced-motion` toner bare ikonet inn. `done` kalles én
// gang, også når den hoppes over. Gir funksjonen som hopper over.
const FORGE_GLOWS = ["accent", "brand1", "brand2", "success", "warning", "danger", "brand3"];
let forgeSkip = null;
// Esc, E, Enter eller mellomrom hopper over (før de andre tastene, så Esc
// ikke også lukker Smia).
document.addEventListener("keydown", (e) => {
  if (!forgeSkip || !["Escape", "KeyE", "Enter", "Space"].includes(e.code)) return;
  e.preventDefault();
  e.stopImmediatePropagation();
  forgeSkip();
}, true);
function forgeAnimation(box, item, done) {
  const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const timers = [];
  const anims = [];
  const lit = [...box.querySelectorAll(".comp.have")];
  const stage = document.createElement("div");
  stage.className = `forge-stage${calm ? " calm" : ""}`;
  stage.setAttribute("role", "status");
  stage.innerHTML = `<div class="forge-core" aria-hidden="true"></div><div class="forge-sparks" aria-hidden="true"></div>
    <div class="forge-item">${gearIcon(item, "big")}<p class="forge-text">${esc(splitRarity(item.name)[0])} er smidd</p></div>
    <div class="forge-blink" aria-hidden="true"></div><p class="forge-skip">Klikk eller Esc for å hoppe over</p>`;
  box.classList.add("forge-host");
  box.appendChild(stage);
  const at = (ms, f) => timers.push(setTimeout(f, ms));
  const anim = (el, frames, opts) => {
    const a = el.animate(frames, { fill: "both", ...opts });
    anims.push(a);
    return a;
  };
  let over = false;
  const finish = () => {
    if (over) return;
    over = true;
    forgeSkip = null;
    timers.forEach(clearTimeout);
    anims.forEach((a) => a.cancel());
    stage.remove();
    box.classList.remove("forge-host");
    lit.forEach((c) => c.classList.remove("lit"));
    done();
  };
  forgeSkip = finish;
  stage.addEventListener("click", finish);
  const shown = stage.querySelector(".forge-item");
  if (calm) {
    anim(shown, [{ opacity: 0 }, { opacity: 1 }], { duration: 300, easing: "ease-out" });
    at(700, finish);
    return finish;
  }
  // Komponentlista rulles fram, så de flyr fra der du ser dem.
  box.querySelector(".comp-list")?.scrollIntoView({ block: "nearest" });
  const s = stage.getBoundingClientRect();
  const cx = s.width / 2;
  const cy = s.height * 0.42;
  stage.style.setProperty("--forge-x", `${cx}px`);
  stage.style.setProperty("--forge-y", `${cy}px`);
  // Ingrediensene lyser opp i hver sin farge og flyr inn mot midten.
  lit.forEach((c, i) => {
    const glow = `var(--lt-color-${FORGE_GLOWS[i % FORGE_GLOWS.length]}-fill)`;
    c.style.setProperty("--glow", glow);
    c.classList.add("lit");
    const ico = c.querySelector(".comp-ico");
    const r = ico?.getBoundingClientRect();
    if (!r || !r.width) return;
    const fly = document.createElement("img");
    fly.className = "forge-fly";
    fly.src = ico.src;
    fly.alt = "";
    fly.style.setProperty("--glow", glow);
    fly.style.left = `${r.left - s.left}px`;
    fly.style.top = `${r.top - s.top}px`;
    fly.style.width = `${r.width}px`;
    fly.style.height = `${r.height}px`;
    stage.appendChild(fly);
    const a = (i / lit.length) * Math.PI * 2;
    const dx = cx + Math.cos(a) * 28 - (r.left - s.left + r.width / 2);
    const dy = cy + Math.sin(a) * 28 - (r.top - s.top + r.height / 2);
    const mx = cx - (r.left - s.left + r.width / 2);
    const my = cy - (r.top - s.top + r.height / 2);
    anim(fly, [
      { transform: "translate(0, 0) scale(1)", opacity: 1, offset: 0 },
      { transform: `translate(${dx * 0.5}px, ${dy * 0.5 - 40}px) scale(1.25)`, opacity: 1, offset: 0.35 },
      { transform: `translate(${dx}px, ${dy}px) scale(0.8)`, opacity: 1, offset: 0.62 },
      { transform: `translate(${mx}px, ${my}px) scale(0.15)`, opacity: 0, offset: 1 },
    ], { duration: 1000, delay: i * 70, easing: "cubic-bezier(.45,0,.25,1)" });
  });
  // Gløden der de smelter sammen, som slår ut ved hvert hammerslag.
  const core = stage.querySelector(".forge-core");
  anim(core, [
    { transform: "translate(-50%, -50%) scale(0.2)", opacity: 0, offset: 0 },
    { transform: "translate(-50%, -50%) scale(0.5)", opacity: 0.7, offset: 0.4 },
    { transform: "translate(-50%, -50%) scale(1)", opacity: 1, offset: 0.55 },
    { transform: "translate(-50%, -50%) scale(0.85)", opacity: 1, offset: 0.62 },
    { transform: "translate(-50%, -50%) scale(1.15)", opacity: 1, offset: 0.72 },
    { transform: "translate(-50%, -50%) scale(1.6)", opacity: 0, offset: 0.85 },
    { transform: "translate(-50%, -50%) scale(1.6)", opacity: 0, offset: 1 },
  ], { duration: 1500, easing: "ease-out" });
  const sparks = stage.querySelector(".forge-sparks");
  for (const t of [820, 1080]) {
    at(t, () => {
      sfx.play("craft");
      anim(sparks, [{ transform: "translate(-50%, -50%) scale(0.3) rotate(0deg)", opacity: 1 }, { transform: `translate(-50%, -50%) scale(2.6) rotate(${t === 820 ? 25 : -40}deg)`, opacity: 0 }], { duration: 450, easing: "ease-out", fill: "none" });
      anim(box, [{ transform: "translate(0, 0)" }, { transform: "translate(0, 3px)" }, { transform: "translate(0, -1px)" }, { transform: "translate(0, 0)" }], { duration: 160, fill: "none" });
    });
  }
  // Det nye kommer fram med en kort blink.
  at(1250, () => {
    anim(stage.querySelector(".forge-blink"), [{ opacity: 0.85 }, { opacity: 0 }], { duration: 260, easing: "ease-out" });
    anim(shown, [
      { transform: "scale(0.6)", opacity: 0, filter: "brightness(3)" },
      { transform: "scale(1.08)", opacity: 1, filter: "brightness(1.6)", offset: 0.45 },
      { transform: "scale(1)", opacity: 1, filter: "brightness(1)" },
    ], { duration: 420, easing: "ease-out" });
  });
  at(1900, finish);
  return finish;
}

// Smia (plan 13c, eierens revisjon 18. oktober; fra Skriptoriet 26.
// oktober): en dialog med fast
// høyde. Til venstre filtrene med antall og lista som ruller for seg, med
// én oppsummering per oppskrift (ikon, navn, sjeldenhet og «Kan smis»,
// «Mangler N» eller «Har du»); de du kan smi nå øverst, det du har nederst
// og dempet. Til høyre detaljene for den valgte: beskrivelsen,
// sammenligningen med det du har på, komponentene med «har 2 av 3», og
// «Smi» fast nederst, av med grunnen. På smal skjerm står detaljene i
// stedet for lista, med «Tilbake». Fanene er slagene (eieren 18. oktober:
// ingen «Alt», du velger alltid et slag), og den sist valgte huskes.
let workshopFilter = remembered("fristad.workshop.tab", "weapon");
let workshopSel = null;
function openWorkshop(recipes) {
  const body = openOverlay("Smia", "Søknad om smiing: har du komponentene i lageret, godkjenner smeden skjemaet og smir tingen.");
  $("overlay").querySelector(".overlay-inner").classList.add("menu");
  let detail = false;
  const lack = (r) => r.needs.reduce((s, [, n, have]) => s + Math.max(0, n - have), 0) + r.requires.filter(([, ok]) => !ok).length;
  const state = (r) => (r.item.owned ? 2 : r.ok ? 0 : 1);
  const sorted = [...recipes].sort((a, b) => state(a) - state(b) || lack(a) - lack(b) || rarityRank(a.item.rarity) - rarityRank(b.item.rarity) || a.item.name.localeCompare(b.item.name, "nb"));
  const pool = recipes.flatMap((r) => [r.item, r.current]).filter(Boolean);
  const filters = SLOTS.filter((k) => recipes.some((r) => r.item.slot === k)).map((k) => [k, SLOT_NAMES[k], recipes.filter((r) => r.item.slot === k).length]);
  if (!filters.some(([k]) => k === workshopFilter)) workshopFilter = filters[0]?.[0] ?? "weapon";
  // Etter smiinga: tingen er din og på deg, og komponentene er brukt.
  const crafted = (r) => {
    const made = { ...r.item, owned: true, equipped: true };
    for (const x of recipes) {
      for (const need of x.needs) {
        const used = r.needs.find(([c]) => c === need[0]);
        if (used) need[2] = Math.max(0, need[2] - used[1]);
      }
      if (x.id === r.id) x.item = made;
      else if (x.item.slot === made.slot && made.slot !== "lantern") x.current = made;
      x.ok = !x.item.owned && x.needs.every(([, n, have]) => have >= n) && x.requires.every(([, ok]) => ok);
    }
  };
  const badge = (r) => (r.item.owned ? '<span class="lt-badge" data-color="neutral">Har du</span>' : r.ok ? '<span class="lt-badge" data-color="success">Kan smis</span>' : `<span class="lt-badge" data-color="warning">Mangler ${lack(r)}</span>`);
  const wrap = document.createElement("div");
  wrap.className = "workshop picker";
  body.appendChild(wrap);
  // «Smi» står fast i bunnlinja til venstre for «Tilbake».
  const actions = $("overlay-actions");
  const recipeDetail = (r) => {
    const where = new Map((r.sources ?? []).map(([c, text, cryptOnly]) => [c, { text, cryptOnly }]));
    const missing = [];
    const comp = (c, have, n, keep) => {
      const ok = have >= n;
      if (!ok) missing.push(compTitle(c));
      const w = where.get(c);
      return `<li class="comp ${ok ? "have" : "lack"}">
          <img class="comp-ico" src="components/${esc(c)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
          <div class="comp-main">
            <div class="comp-top"><span class="comp-title">${esc(compTitle(c))}</span><span class="comp-count">${keep ? (ok ? "ligger i lageret" : "må ligge i lageret") : `har ${have} av ${n}`}</span>${w?.cryptOnly ? '<span class="lt-badge" data-color="warning">Bare i krypten</span>' : ""}</div>
            <progress class="lt-progress mini" data-color="${ok ? "success" : "warning"}" max="${n}" value="${Math.min(have, n)}" aria-hidden="true"></progress>
            ${ok || !w ? "" : `<p class="comp-where">${esc(w.text)}</p>`}
          </div>
        </li>`;
    };
    const comps = r.needs.map(([c, n, have]) => comp(c, have, n, false)).concat(r.requires.map(([c, ok]) => comp(c, ok ? 1 : 0, 1, true)));
    const [name] = splitRarity(r.item.name);
    const owned = r.item.owned;
    const can = r.ok && !owned;
    const list = (xs) => (xs.length > 1 ? `${xs.slice(0, -1).join(", ")} og ${xs.at(-1)}` : xs[0] ?? "");
    const why = owned ? "Du har den allerede." : can ? "Komponentene trekkes fra lageret." : `Mangler ${list(missing)}.`;
    const other = r.current && r.current.id !== r.item.id ? r.current : null;
    actions.innerHTML = narrow() && !detail ? "" : `<button type="button" class="lt-button" data-variant="primary" data-craft="${esc(r.id)}" ${can ? "" : "disabled"} aria-describedby="craft-why">Smi<span class="craft-name"> ${esc(name)}</span></button>
        <span class="lt-help-text" id="craft-why">${esc(why)}</span>`;
    return `<div class="detail-scroll">
        <button type="button" class="lt-button detail-back" data-variant="ghost" data-back>← Tilbake til lista</button>
        ${detailHead(r.item, badge(r))}
        <h4 class="lt-heading detail-sub" data-size="xs">${other ? `Mot ${esc(splitRarity(other.name)[0])}, som du har på` : "Verdiene"}</h4>
        ${compareTable(r.item, other, pool)}
        <h4 class="lt-heading detail-sub" data-size="xs">Komponentene</h4>
        <ul class="comp-list">${comps.join("")}</ul>
      </div>`;
  };
  const render = () => {
    const shown = sorted.filter((r) => r.item.slot === workshopFilter);
    if (!shown.some((r) => r.id === workshopSel)) workshopSel = shown[0]?.id ?? null;
    const sel = shown.find((r) => r.id === workshopSel);
    const hadFocus = wrap.contains(document.activeElement) && document.activeElement.classList.contains("pick-row");
    wrap.classList.toggle("show-detail", detail);
    actions.innerHTML = "";
    const detailHtml = sel ? recipeDetail(sel) : '<p class="lt-paragraph">Ingen oppskrifter her.</p>';
    wrap.innerHTML = `${menuTabs(filters, workshopFilter, "Slagene")}
      <div class="pick-side">
        <ul class="pick-list" role="listbox" aria-label="Oppskriftene">
          ${shown.map((r) => pickRow(r.id, r.item, badge(r), r.id === workshopSel).replace('class="pick-row"', `class="pick-row${r.item.owned ? " dim" : ""}" data-recipe="${esc(r.id)}"`)).join("")}
        </ul>
      </div>
      <section class="detail" aria-live="polite">${detailHtml}</section>`;
    bindTabs(wrap, (k) => {
      workshopFilter = k;
      remember("fristad.workshop.tab", k);
      workshopSel = null;
      detail = false;
      sfx.play("click");
      render();
      wrap.querySelector(`[data-tab="${k}"]`)?.focus();
    });
    bindPickList(wrap, {
      select(key, open) {
        if (key === workshopSel && !open) return;
        workshopSel = key;
        if (open) detail = true;
        sfx.play("click");
        render();
      },
    });
    wrap.querySelector("[data-back]")?.addEventListener("click", () => {
      detail = false;
      render();
      wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
    });
    actions.querySelector("[data-craft]")?.addEventListener("click", (ev) => {
      const b = ev.currentTarget;
      if (b.disabled) return;
      if (engine.craft(b.dataset.craft)) {
        // Smie-animasjonen, så står raden med «Har du». Lageret er trukket
        // i motoren; her trekkes det samme fra oppskriftene som vises.
        b.disabled = true;
        const r = recipes.find((x) => x.id === b.dataset.craft);
        if (!r) return;
        forgeAnimation($("overlay").querySelector(".overlay-inner"), r.item, () => {
          crafted(r);
          render();
          wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
        });
      } else {
        sfx.play("deny");
      }
    });
    const row = wrap.querySelector(".pick-row[aria-selected='true']");
    row?.scrollIntoView({ block: "nearest" });
    if (hadFocus) row?.focus();
  };
  render();
  if (!narrow()) wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
}

// Snakkeboblene over innbyggerne: vanlige bobler, ikke designsystemet.
const bubbleEls = new Map();
function updateBubbles(bubbles) {
  const root = $("bubbles");
  const seen = new Set();
  for (const b of bubbles) {
    seen.add(b.id);
    let el = bubbleEls.get(b.id);
    if (!el) {
      el = document.createElement("div");
      el.className = "bubble";
      el.innerHTML = '<div class="bubble-name"></div><div class="bubble-text"></div>';
      root.appendChild(el);
      bubbleEls.set(b.id, el);
    }
    el.style.left = `${b.x * 100}%`;
    el.style.top = `${b.y * 100}%`;
    el.firstChild.textContent = b.name;
    el.lastChild.textContent = b.text;
  }
  for (const [id, el] of bubbleEls) {
    if (!seen.has(id)) {
      el.remove();
      bubbleEls.delete(id);
    }
  }
}

// Merkene i kampen: ett element per id, flyttet dit spillet sier.
const markEls = new Map();
function updateMarks(marks) {
  const root = $("marks");
  const seen = new Set();
  for (const m of marks) {
    seen.add(m.id);
    let el = markEls.get(m.id);
    if (!el) {
      el = document.createElement("div");
      el.className = "mark";
      if (m.kind === "bar") {
        el.innerHTML = '<div class="mark-bar"><i></i></div>';
      } else {
        el.innerHTML = `<span class="mark-text${Number(m.text) >= 50 ? " heavy" : ""}"></span>`;
      }
      root.appendChild(el);
      markEls.set(m.id, el);
    }
    el.style.left = `${m.x * 100}%`;
    el.style.top = `${m.y * 100}%`;
    if (m.kind === "bar") {
      el.firstChild.firstChild.style.width = `${Math.round(m.value * 100)}%`;
    } else {
      el.firstChild.textContent = m.text;
      el.style.opacity = String(Math.max(0, Math.min(1, m.value * 1.5)));
    }
  }
  for (const [id, el] of markEls) {
    if (!seen.has(id)) {
      el.remove();
      markEls.delete(id);
    }
  }
}

// Lydene (web/sounds.js): opptak og syntetiske lyder, med retning og etterklang.
const sfx = createSounds();

// Musikken (web/music.js): sporene fra filer (byen etter døgnet, krypten
// etter temaet, kampen når en fiende er nær), ellers partituret.
const music = createMusic(() => sfx.context());
// Lydene i byen (web/ambience.js) etter hvor du står.
const ambience = createAmbience(() => sfx.context(), sfx);
let lastAmbience = null;

// Du er «inne» når startkortet er borte. Musa fanges med pekerlås når
// nettleseren gir oss den; ellers ser du deg rundt ved å dra med musa, så
// knappen virker også i innebygde nettlesere uten pekerlås.
let playing = false;
let dragging = false;

function lock() {
  // Avslår nettleseren (uten brukerhandling, eller uten rå bevegelse), er
  // det ikke en feil: et klikk i bildet låser igjen.
  try {
    const p = canvas.requestPointerLock?.({ unadjustedMovement: true });
    if (p?.catch) {
      p.catch(() => {
        try {
          canvas.requestPointerLock?.()?.catch?.(() => {});
        } catch {
          /* ingen pekerlås */
        }
      });
    }
  } catch {
    /* ingen pekerlås */
  }
}

function locked() {
  return document.pointerLockElement === canvas;
}

// Sier fra (én gang i blant) at et klikk i bildet fanger musa igjen.
let hintAt = 0;
function clickHint() {
  const now = performance.now();
  if (now - hintAt < 8000) return;
  hintAt = now;
  toast?.show?.("Klikk i bildet for å se deg rundt med musa igjen. Uten fanget mus: dra for å se, klikk for å hugge.");
}

let loadedSave = false;
let soundsLoading = false;
function enter(relock = true) {
  pausedMenu = false;
  gateEl.classList.remove("gate-first");
  beginBtn.hidden = true;
  delete enterBtn.dataset.variant;
  if (!loadedSave) {
    loadedSave = true;
    try {
      const saved = localStorage.getItem("fristad.save");
      if (saved) engine.load_state(saved);
    } catch {
      /* ingen lagring */
    }
  }
  if (soundsLoading && !panelOpen) sfx.play("resume");
  playing = true;
  if (softwareGpu && !gpuToastShown) {
    gpuToastShown = true;
    toast?.show?.("Nettleseren tegner uten grafikkortet, så spillet går tregt. Se startkortet (Esc) for hvordan du slår på maskinvareakselerasjon.");
  }
  if (!soundsLoading) {
    soundsLoading = true;
    sfx.preload();
  }
  music.set(lastHud.cellar ? "cellar" : "town");
  if (lastAmbience) {
    ambience.set(lastAmbience);
    sfx.ambience(lastAmbience);
    music.ambience(lastAmbience);
  }
  gateEl.hidden = true;
  hud.hidden = false;
  engine.set_paused(false);
  canvas.focus();
  fadeHelp();
  // Med `?edit` begynner du i redigeringen, med fri mus.
  if (editor && !editorStarted) {
    editorStarted = true;
    if (editor.begin()) return;
  }
  if (relock && !editor?.active) lock();
  setTimeout(updateLook, 400);
}

// «Klikk for å se deg rundt med musa»: står midt i bildet så lenge du spiller
// uten fanget mus (etter filmen kan ikke nettleseren fange den uten et klikk).
// Sa nettleseren nei (`pointerlockerror`), spilles det uten: dra for å se.
let noLock = false;
const lookEl = $("look");
function updateLook() {
  const hide = !(playing && !locked() && !noLock && !editor?.active && !panelOpen && !overlayOpen);
  if (lookEl.hidden !== hide) lookEl.hidden = hide;
}

// Knappene som bare hører til pausemenyen. Startkortet viser bare framdriften
// mens siden laster (eieren 18. oktober: knappene sto der uten å virke).
const PAUSE_ONLY = ["respawn", "replay", "quit", "graphics", "settings"];
// Pausemenyen står (Esc i spillet): Esc igjen eller et klikk utenfor kortet
// tar deg tilbake i spillet (eieren 19. oktober).
let pausedMenu = false;
let pausedAt = 0;

function quit() {
  pausedMenu = false;
  playing = false;
  dragging = false;
  panelOpen = false;
  panel.hidden = true;
  engine.set_paused(true);
  engine.respawn();
  if (locked()) document.exitPointerLock?.();
  enterBtn.textContent = "Gå inn i Fristad";
  for (const id of PAUSE_ONLY) $(id).hidden = true;
  hud.hidden = true;
  gateEl.hidden = false;
  enterBtn.focus();
}

function pause() {
  if (playing) sfx.play("pause");
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  if (!panelOpen) {
    pausedMenu = true;
    pausedAt = performance.now();
    enterBtn.textContent = "Fortsett";
    for (const id of PAUSE_ONLY) $(id).hidden = id === "replay" && !FILM;
    gateEl.hidden = false;
    enterBtn.focus();
  }
}

// Åpningsfilmen (den ferdige videoen film/intro.mp4, klippet av
// tools/film/cut.py; manuset i docs/film/manus.md, kanon i docs/kanon.md)
// spilles hver gang med ett klikk («Klikk for å begynne», som også gir lyd)
// når byen er bygd, og toner over i spillet ved sørporten. `?nofilm` hopper
// over den.
const FILM = !EDIT && !new URLSearchParams(location.search).has("nofilm");
let film = null;
let filmReveal = false;
// Filmen sett på nytt fra pausemenyen: du blir stående der du var, i den
// tida på døgnet det var, og spillet står stille under den.
let filmReplay = false;

async function startFilm(replay = false) {
  filmReplay = replay === true;
  pausedMenu = false;
  beginBtn.hidden = true;
  gateEl.classList.remove("gate-first");
  gateEl.hidden = true;
  hud.hidden = true;
  if (!soundsLoading) {
    soundsLoading = true;
    sfx.preload();
  }
  music.set("silence");
  if (!filmReplay) {
    engine.set_theme(false);
    engine.set_paused(false);
  }
  filmReveal = false;
  try {
    film = createFilm();
  } catch (e) {
    console.warn("Filmen kunne ikke vises:", e);
    enter();
    return;
  }
  const { skipped, gesture } = await film.done;
  film = null;
  // Hoppet over: rett til sørporten om dagen (ikke når den ble sett på nytt).
  if (!filmReplay && (skipped || !filmReveal)) {
    engine.set_theme(false);
    engine.respawn();
  }
  // Hoppet du over med et klikk eller mellomrom, kan musa fanges med en gang;
  // ellers står «Klikk for å se deg rundt» midt i bildet.
  enter(!!gesture);
}

function bindInput() {
  enterBtn.addEventListener("click", () => enter());
  // Sitter du fast, tar knappen i pausemenyen (og H) deg tilbake til sørporten.
  $("respawn").addEventListener("click", () => {
    engine.respawn();
    enter();
  });
  // Avslutt: tilbake til startskjermen, med spilleren ved sørporten igjen.
  $("quit").addEventListener("click", quit);
  // Filmen på nytt fra pausemenyen.
  $("replay").addEventListener("click", () => startFilm(true));
  // Et klikk utenfor kortet i pausemenyen er det samme som «Fortsett».
  gateEl.addEventListener("click", (e) => {
    if (pausedMenu && e.target === gateEl) enter();
  });
  canvas.addEventListener("click", () => {
    if (playing && !locked() && !editor?.active) lock();
  });
  $("look").addEventListener("click", () => {
    if (playing && !locked()) lock();
  });

  document.addEventListener("pointerlockchange", () => {
    updateLook();
    // Slippes låsen mens du spiller (Esc), er det pause.
    if (!locked() && playing && !panelOpen && !editor?.active) pause();
  });
  // Ga ikke nettleseren oss musa, går det an å spille uten: dra for å se,
  // klikk for å hugge.
  document.addEventListener("pointerlockerror", () => {
    noLock = true;
    updateLook();
    if (playing) clickHint();
  });

  // Musetastene er hugg og blokk. Når musa er fanget, går hvert trykk rett
  // til spillet. Uten pekerlås (noen nettlesere og jobb-PC-er gir den ikke)
  // drar venstre tast kameraet, og et kort klikk uten å dra er et hugg;
  // høyre tast blokker uansett. Trykk i HUD-en over bildet teller òg, men
  // ikke i dialogene.
  let dragDist = 0;
  // Uten pekerlås: står venstre tast stille i `HOLD_MS`, er det et hugg som
  // holdes (det tunge hugget lades til tasten slippes); drar du først, er
  // det å se deg rundt.
  const HOLD_MS = 140;
  let holdTimer = 0;
  let holding = false;
  const inGame = (t) => t === canvas || (t instanceof Element && !t.closest("dialog, #overlay, #panel, #gate, button, a, input, details"));
  document.addEventListener("mousedown", (e) => {
    if (playing && editor?.mouseDown(e)) return;
    if (!playing || !inGame(e.target)) return;
    if (locked()) {
      e.preventDefault();
      engine.mouse(e.button, true);
    } else if (e.button === 0) {
      dragging = true;
      dragDist = 0;
      clearTimeout(holdTimer);
      holdTimer = setTimeout(() => {
        if (dragging && dragDist < 6 && playing) {
          holding = true;
          engine.mouse(0, true);
        }
      }, HOLD_MS);
    } else {
      e.preventDefault();
      engine.mouse(e.button, true);
    }
  });
  window.addEventListener("mouseup", (e) => {
    if (editor?.mouseUp(e)) return;
    const wasDrag = dragging;
    dragging = false;
    clearTimeout(holdTimer);
    if (e.button === 0 && holding) {
      holding = false;
      engine.mouse(0, false);
      return;
    }
    if (!playing) return;
    if (!locked() && e.button === 0 && wasDrag && dragDist < 6 && inGame(e.target)) {
      // Et kort klikk uten pekerlås: et hugg.
      engine.mouse(0, true);
      engine.mouse(0, false);
      return;
    }
    engine.mouse(e.button, false);
  });
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener("contextmenu", (e) => {
    if (playing && inGame(e.target)) e.preventDefault();
  });
  document.addEventListener("mousemove", (e) => {
    if (!playing) return;
    if (editor?.mouseMove(e)) return;
    if (dragging) dragDist += Math.abs(e.movementX) + Math.abs(e.movementY);
    if (locked() || dragging) engine.look(e.movementX * mouseSens, e.movementY * mouseSens);
  });

  const keys = new Set(["Space", "Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);
  document.addEventListener("keydown", (e) => {
    // Esc med panelet oppe lukker panelet og tar deg rett tilbake i spillet,
    // uten å be om pekerlås (Esc gir ingen brukerhandling å låse på, og et
    // avslått forsøk åpnet pausemenyen). Klikk i bildet låser igjen; neste
    // Esc åpner menyen.
    // Redigeringen tar sine taster først (og P for å gå inn i den igjen).
    if ((settingsOpen || graphicsOpen) && e.code === "Escape") {
      e.preventDefault();
      (graphicsOpen || settingsOpen).querySelector('[data-act="close"]').click();
      return;
    }
    if (editor && playing && !panelOpen && !invOpen && !overlayOpen && !settingsOpen && editor.key(e)) return;
    if (panelOpen && e.code === "Escape") {
      // Er dialogen i demoen åpen, er det den Esc lukker.
      if ($("demo").shadowRoot?.querySelector("dialog:modal")) return;
      e.preventDefault();
      closePanel("resume");
      return;
    }
    if (lootOpen) return;
    if (e.code === "KeyI" && !e.repeat && (playing || invOpen)) {
      e.preventDefault();
      toggleInventory();
      return;
    }
    if (invOpen) return;
    if (e.code === "KeyM" && !e.repeat && playing) {
      const on = music.toggle();
      toast?.show?.(on ? "Musikken er på." : "Musikken er av.");
      return;
    }
    if (overlayOpen) {
      if (e.code === "Escape") {
        e.preventDefault();
        closeOverlay(true, false);
      } else if (e.code === "KeyE" && !e.repeat) {
        e.preventDefault();
        closeOverlay(true, true);
      }
      return;
    }
    if (e.code === "KeyJ" && !e.repeat && playing) {
      e.preventDefault();
      toggleJournal();
      return;
    }
    // Tastene: F1, eller ? (F-tastene trenger fn på små tastaturer).
    const field = e.composedPath()[0];
    const writing = field instanceof HTMLElement && (field.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(field.tagName));
    if ((e.code === "F1" || (e.key === "?" && !writing)) && !e.repeat) {
      e.preventDefault();
      $("keys-panel").open = !$("keys-panel").open;
      return;
    }
    if (e.code === "KeyK" && !e.repeat && playing) {
      e.preventDefault();
      toggleBigMap();
      return;
    }
    if (e.code === "KeyG" && !e.repeat && !graphicsOpen && (playing || !gateEl.hidden)) {
      setQuality(QUALITIES[(QUALITIES.indexOf(quality) + 1) % QUALITIES.length]);
      return;
    }
    if (e.code === "KeyO" && !e.repeat && (playing || !gateEl.hidden || settingsOpen)) {
      e.preventDefault();
      if (settingsOpen) settingsOpen.querySelector('[data-act="close"]').click();
      else openSettings();
      return;
    }
    if (settingsOpen || graphicsOpen) return;
    // Esc i pausemenyen: tilbake i spillet. Musa fanges ikke med Esc (det er
    // ingen brukerhandling å låse på); «Klikk for å se deg rundt» står da i bildet.
    // (Ikke den samme Esc-en som slapp musa og åpnet menyen.)
    if (pausedMenu && e.code === "Escape" && !e.repeat && performance.now() - pausedAt > 300) {
      e.preventDefault();
      enter(false);
      return;
    }
    if (!playing) return;
    if (e.code === "Escape" && !locked()) {
      pause();
      return;
    }
    if (keys.has(e.code)) e.preventDefault();
    if (e.repeat) return;
    engine.key(e.code, true);
  });
  document.addEventListener("keyup", (e) => {
    if (playing) engine.key(e.code, false);
  });
  window.addEventListener("blur", () => engine.release_all());

  $("close").addEventListener("click", () => closePanel(true));
  $("overlay-close").addEventListener("click", () => closeOverlay(true, true));
  $("graphics").addEventListener("click", () => openGraphics());
  $("settings").addEventListener("click", () => openSettings());
}

// Tastekartet nederst står hele tida, så det alltid går an å se hva tastene gjør.
function fadeHelp() {}

function openPanel() {
  const d = docs[current.component] ?? {};
  panelOpen = true;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  hud.hidden = true;

  $("panel-title").textContent = d.title ?? current.component;
  $("panel-class").textContent = `.lt-${current.component}`;
  // Slaget står i beskrivelsen fra docs («Standalone component.» osv.).
  $("panel-kind").textContent = d.description?.match(/\b(Standalone|Frame) component\b/)?.[0] ?? "CSS component";
  $("panel-description").textContent = d.description ?? "";
  $("panel-intro").innerHTML = inlineCode(d.intro ?? "");
  $("panel-code").textContent = d.code ?? "";
  $("panel-link").href = d.url ?? "https://lo-tek.sobernetics.no/";
  renderDemo(d);
  panel.hidden = false;
  $("close").focus();
}

function closePanel(relock) {
  panelOpen = false;
  panel.hidden = true;
  renderDemo({});
  if (relock === "resume") {
    enter(false);
    clickHint();
  } else if (relock) {
    enter();
  } else {
    enterBtn.textContent = "Fortsett";
    gateEl.hidden = false;
    enterBtn.focus();
  }
}

// Eksempelet fra dokumentasjonen, levende, i en flate slik LO-TEK-docs
// viser den (Preview.astro): i en shadow root med bare LO-TEKs CSS, så
// ingenting fra spillets stilark treffer komponentene. Flatens stilark
// (`docs-flate.css`) skriver tools/export-docs.ts rett fra Preview.astro, så
// panelet kjenner de samme layoutene som docs (Divider bruker `flow`); dette
// er reserven til det er hentet.
const FLATE = `
  :host { display: block; }
  .surface {
    display: flex;
    gap: var(--lt-spacing-3);
    padding: var(--lt-spacing-6);
    max-width: var(--lt-preview-max-width, none);
    background: var(--lt-color-neutral-canvas);
    color: var(--lt-color-neutral-text-strong);
    border: 1px solid var(--lt-color-neutral-border-subtle);
    border-radius: var(--lt-spacing-1);
    font-family: inherit;
    font-size: var(--lt-font-size-m);
    line-height: 1.6;
  }
  .surface[data-fixed-inside] { contain: layout; }
  .surface[data-layout="row"] { flex-wrap: wrap; align-items: center; }
  .surface[data-layout="stack"] { flex-direction: column; align-items: stretch; }
  .surface[data-layout="flow"] { display: block; }
`;
// Det panelet legger til over docs-flaten.
const FLATE_EXTRA = `
  /* Spillet setter de norske tekstene på :root (style.css), og de arves inn
     i shadow root. Boka er LO-TEK-docs på engelsk, så her gjelder LO-TEKs. */
  :host { --lt-label-required-text: initial; --lt-label-optional-text: initial; }
  /* Docs-flaten gir <hr> nettleserens margin-inline: auto, og da blir
     skillelinja null bred i stabelen. I boka skal den synes. */
  .surface > .lt-divider { margin-inline: 0; }
`;

// LO-TEKs stilark hentes som tekst: kopierte regler (cssRules) mister
// `background: var(...)` når lengre ned i regelen setter enkeltdelene.
const demoSheets = [new CSSStyleSheet(), new CSSStyleSheet()];
demoSheets[1].replaceSync(FLATE + FLATE_EXTRA);
fetch("docs-flate.css")
  .then((r) => (r.ok ? r.text() : Promise.reject(new Error(`HTTP ${r.status}`))))
  .then((css) => demoSheets[1].replace(css + FLATE_EXTRA))
  .catch((e) => console.warn("docs-flate.css til demoen:", e));
fetch("lotek.css")
  .then((r) => r.text())
  .then((css) => demoSheets[0].replace(css))
  .catch((e) => console.warn("lotek.css til demoen:", e));

// Det skriptet ved siden av demoen i docs gjør (ToastDemo.astro osv.).
const DEMO_ACTIONS = {
  toast: (root) =>
    root.querySelector(".lt-button")?.addEventListener("click", () =>
      root.querySelector("lt-toast")?.show("The application has been saved", { color: "success" })),
  "connection-status": (root) => {
    const status = root.querySelector("lt-connection-status");
    root.getElementById("demo-connection-off")?.addEventListener("click", () => status?.reportFailure());
    root.getElementById("demo-connection-on")?.addEventListener("click", () => status?.reportSuccess());
  },
  dialog: (root) =>
    root.querySelector("[data-open]")?.addEventListener("click", () =>
      root.querySelector("lt-dialog")?.setAttribute("open", "")),
};

function renderDemo(d) {
  const host = $("demo");
  const demo = d.demo;
  if (!demo?.html) {
    host.hidden = true;
    host.shadowRoot?.replaceChildren();
    return;
  }
  host.hidden = false;
  const root = host.shadowRoot ?? host.attachShadow({ mode: "open" });
  // Eksempelets egen CSS (`extraCss`/`previewCss` i docs) sist, som i
  // Preview.astro: i et <style> tapte den mot flaten i adoptedStyleSheets,
  // som kommer etter i kaskaden (linja i Connection Status dekket knappen).
  const own = new CSSStyleSheet();
  own.replaceSync(demo.css ?? "");
  root.adoptedStyleSheets = [...demoSheets, own];
  const tpl = document.createElement("template");
  tpl.innerHTML = demo.html;
  // Lenker i eksemplene skal ikke føre noen steder, men lenkene til et
  // anker i eksempelet står: fanene i LO-TEK er lenker til panelene sine.
  for (const a of tpl.content.querySelectorAll("a[href]")) if (!a.getAttribute("href").startsWith("#")) a.setAttribute("href", "#");
  for (const s of tpl.content.querySelectorAll("script")) s.remove();
  const surface = document.createElement("div");
  surface.className = "surface";
  surface.dataset.layout = demo.layout || "row";
  if (demo.fixed) surface.dataset.fixedInside = "";
  if (demo.maxWidth) surface.style.setProperty("--lt-preview-max-width", demo.maxWidth);
  surface.append(tpl.content);
  root.replaceChildren(surface);
  DEMO_ACTIONS[d.name]?.(root);
}

function inlineCode(text) {
  const esc = text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  return esc.replace(/`([^`]+)`/g, "<code>$1</code>");
}

// `?uitest`: tools/ui-test.mjs viser menyene med falske data uten motoren
// (hendelsene går gjennom `handle` som fra Rust, og `engine` er en attrapp).
if (new URLSearchParams(location.search).has("uitest")) {
  window.fristadUi = {
    handle,
    toggleInventory,
    set(o) {
      if (o.engine) engine = o.engine;
      if (o.docs) docs = o.docs;
      if (o.hud) lastHud = o.hud;
    },
  };
}

boot().catch((e) => {
  console.error(e);
  status.classList.add("error");
  status.textContent =
    "Fristad trenger WebGPU eller WebGL2, og fikk ikke startet her. Prøv en oppdatert Chrome, Edge, Safari eller Firefox.";
});


// Målingen (`?bench` i adressen, eller `fristad.bench()` i konsollen): går
// gjennom faste steder og grafikkvalg, måler tiden på grafikkortet per del
// og bildene per sekund, og viser alt som én tekst eieren kan kopiere.
// Bryterne (MSAA, alpha to coverage, solskygge, flat farge) skiller hva som
// koster: flat farge måler bare rasterisering og overtegning.
let benchFrames = 0;
let benchRunning = false;
const BENCH_PLACES = [
  ["torget", 2, 1.7, 9, 0],
  ["Storgata", 4.7, 1.7, 28, 0],
  ["sørporten", 4.7, 1.7, 50, 0],
];
// I prøvenivået (`?nivaa=test`): stedene i nivåets eget rom (nivå 1).
const BENCH_LEVEL_PLACES = [
  ["kapellet", 8, 1.7, 9.5, 0],
  ["hvelvet", 19, 2.7, 15, -135],
  ["gangen", 20, 1.7, 7.5, 180],
];
// Hver kjøring: nivået, en kort forklaring og bryterne til `engine.set_bench`
// (det som mangler, står som i vanlig spill). `scale` ganger oppløsningen.
const BENCH_RUNS = [
  { q: "standard", what: "som i spillet (SMAA)" },
  { q: "standard", what: "uten kantutjevning", smaa: false },
  { q: "standard", what: "MSAA 4× i stedet for SMAA", msaa: 4 },
  { q: "standard", what: "flat farge", flat: true },
  { q: "standard", what: "halv oppløsning", scale: 0.5 },
  // Grafikkbryterne (`gfx`, som i grafikkdialogen) én om gangen.
  { q: "standard", what: "uten myke skygger", gfx: { soft_shadow: 0 } },
  { q: "standard", what: "uten solstråler", gfx: { rays: 0 } },
  { q: "standard", what: "med skygge i krokene", gfx: { ao: 1 } },
  { q: "standard", what: "med skygge langt unna", gfx: { far_shadow: 1 } },
  { q: "standard", what: "uten malt gradering", gfx: { grade: 0 } },
  { q: "enkel", what: "som i spillet" },
  { q: "avansert", what: "som i spillet (SMAA)" },
  { q: "avansert", what: "MSAA 8× i stedet for SMAA", msaa: 8 },
];


const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function measure(seconds) {
  const parts = new Map();
  let samples = 0;
  benchFrames = 0;
  const t0 = performance.now();
  while (performance.now() - t0 < seconds * 1000) {
    await sleep(100);
    const text = engine.gpu_times ? engine.gpu_times() : "";
    if (!text) continue;
    samples++;
    for (const p of text.split(" · ")) {
      const k = p.slice(0, p.lastIndexOf(" "));
      parts.set(k, (parts.get(k) ?? 0) + parseFloat(p.split(" ").pop()));
    }
  }
  const fps = (benchFrames * 1000) / (performance.now() - t0);
  const avg = [...parts].map(([k, v]) => [k, v / Math.max(samples, 1)]);
  const total = avg.reduce((a, [, v]) => a + v, 0);
  return { fps, total, avg };
}
async function runBench() {
  if (benchRunning || !engine) return;
  benchRunning = true;
  // Målingen bruker forhåndsvalgene; de egendefinerte bryterne settes til side.
  const customBefore = custom;
  custom = null;
  const before = quality;
  gateEl.hidden = true;
  engine.set_paused(false);
  const lines = [
    `Fristad ${engine.lotek_version()} · ${engine.backend() === "Gl" ? "WebGL2" : "WebGPU"} · ${gpuName || "ukjent grafikk"}${softwareGpu ? " (programvare)" : ""}`,
    `${navigator.userAgent}`,
    `Lerret ${canvas.width}×${canvas.height} · vindu ${innerWidth}×${innerHeight} · dpr ${devicePixelRatio}`,
    "sted · nivå · kjøring → GPU ms (delene) · fps · trekanter og kall i hovedpasset · lerret",
  ];
  toast?.show?.("Måler grafikken … (rundt ett minutt)");
  if (NIVAA) lines.push(`Nivådokumentet «${NIVAA}» ${SLAA ? "med bitene slått sammen per 16 m" : "med bitene som instanser"}`);
  const places = NIVAA ? BENCH_LEVEL_PLACES : BENCH_PLACES;
  for (const [name, x, y, z, yaw] of places) {
    for (const run of BENCH_RUNS) {
      if (name !== places[0][0] && run !== BENCH_RUNS[0] && run.q === "standard") continue;
      setQuality(run.q, false, false);
      const { q, what, scale, gfx, ...flags } = run;
      if (gfx) engine.set_quality(JSON.stringify({ preset: q, ...gfx }));
      engine.set_bench(JSON.stringify(flags));
      benchScale = scale ?? 1;
      fit();
      if (NIVAA) engine.place_in_level(1, x, y, z, yaw, 0);
      else engine.place(x, y, z, yaw, 0);
      await sleep(600);
      const m = await measure(2);
      const partText = m.avg.map(([k, v]) => `${k} ${v.toFixed(1)}`).join(" · ");
      const tris = `${Math.round(engine.triangles() / 1000)}k △ · ${engine.calls()} kall`;
      lines.push(`${name} · ${q} · ${what} → ${m.total > 0 ? `${m.total.toFixed(1)} ms (${partText})` : "GPU-tid ikke tilgjengelig"} · ${m.fps.toFixed(0)} fps · ${tris} · ${canvas.width}×${canvas.height}`);
    }
  }
  engine.set_bench("{}");
  benchScale = 1;
  fit();
  custom = customBefore;
  setQuality(before ?? "standard", false, false);
  benchRunning = false;
  showBench(lines.join("\n"));
}
function showBench(text) {
  console.info(text);
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog settings-dialog";
  dlg.setAttribute("aria-labelledby", "bench-title");
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="bench-title">Målingen</h2>
    <div class="lt-dialog__body">
      <p class="lt-help-text">Kopier teksten og send den til Claude.</p>
      <textarea class="lt-textarea" rows="16" readonly style="width:100%;font-family:'IBM Plex Mono',monospace;font-size:12px"></textarea>
    </div>
    <div class="lt-dialog__footer">
      <button class="lt-button" data-variant="secondary" type="button" data-act="close">Lukk</button>
      <button class="lt-button" data-variant="primary" type="button" data-act="copy">Kopier</button>
    </div>`;
  document.body.appendChild(dlg);
  dlg.querySelector("textarea").value = text;
  dlg.querySelector('[data-act="copy"]').addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(text);
      toast?.show?.("Kopiert.");
    } catch {
      dlg.querySelector("textarea").select();
    }
  });
  dlg.querySelector('[data-act="close"]').addEventListener("click", () => {
    dlg.close();
    dlg.remove();
    gateEl.hidden = false;
  });
  dlg.showModal();
}
