// JavaScript-laget i Fristad: løkka, input, pekerlås og panelene.
// Byen, fysikken og grafikken bor i Rust (pkg/fristad.js og .wasm).
// Meldingene og forbindelsesstatusen er Fristils egne web-komponenter.

import init, { start, asset_paths } from "./pkg/fristad.js";
import { defineFsToast } from "./fristil/components/frittstaende/toast/fs-toast.js";
import { defineFsConnectionStatus } from "./fristil/components/frittstaende/connection-status/fs-connection-status.js";

defineFsToast();
defineFsConnectionStatus();

const $ = (id) => document.getElementById(id);
const gateEl = $("gate");
const hud = $("hud");
const panel = $("panel");
const promptEl = $("prompt");
const status = $("status");
const enterBtn = $("enter");
const toast = document.querySelector("fs-toast");

let canvas = $("world");
let engine = null;
let docs = {};
let panelOpen = false;
let current = { component: "button", variant: "primary" };

// Variantene per komponent, i samme rekkefølge som R blar gjennom dem i
// byen, med en tekst til panelet.
const VARIANTS = {
  button: [
    { name: "primary", label: "Send søknad", note: "Standard. Trenger ingen data-variant." },
    { name: "secondary", label: "Lagre utkast", note: "Omriss, gjennomsiktig bakgrunn." },
    { name: "ghost", label: "Legg til vedlegg", note: "Ingen kant, ingen bakgrunn." },
    { name: "danger", label: "Slett søknad", note: "For handlinger som ikke kan angres." },
  ],
  alert: [
    { name: "neutral", note: "Standard, nøytral." },
    { name: "info", note: 'data-color="info"' },
    { name: "success", note: 'data-color="success"' },
    { name: "warning", note: 'data-color="warning"' },
    { name: "danger", note: 'data-color="danger"' },
  ],
  tag: [
    { name: "default", note: "Standard, med kant." },
    { name: "filled", note: 'data-variant="filled"' },
    { name: "disabled", note: 'aria-disabled="true"' },
  ],
  badge: [
    { name: "accent", note: "Standard." },
    { name: "success", note: 'data-color="success"' },
    { name: "warning", note: 'data-color="warning"' },
    { name: "danger", note: 'data-color="danger"' },
    { name: "neutral", note: 'data-color="neutral"' },
  ],
  progress: [
    { name: "accent", note: "Standard." },
    { name: "success", note: 'data-color="success"' },
  ],
  spinner: [
    { name: "small", note: 'data-size="small"' },
    { name: "medium", note: "Standard." },
    { name: "large", note: 'data-size="large"' },
  ],
  skeleton: [
    { name: "text", note: 'data-variant="text"' },
    { name: "circle", note: 'data-variant="circle"' },
    { name: "block", note: "Standard, en blokk." },
  ],
};

// Lerretet tegnes i en litt lavere oppløsning enn skjermen på skjermer med
// høy pikseltetthet. Forskjellen synes knapt, men GPU-en får mye mindre å gjøre.
function fit() {
  const scale = Math.min(window.devicePixelRatio || 1, 1.5);
  const w = Math.max(1, Math.round(canvas.clientWidth * scale));
  const h = Math.max(1, Math.round(canvas.clientHeight * scale));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  engine?.resize(w, h);
}

async function boot() {
  fit();
  await init();
  try {
    docs = await (await fetch("docs.json")).json();
  } catch {
    docs = {};
  }
  // Fotoskannede materialer og modeller (CC0 fra Poly Haven), hentet av
  // tools/fetch-assets.ts under byggingen. Mangler de, bakes alt i motoren.
  const assets = await loadAssets();
  try {
    engine = await start(canvas, true, assets.manifest, assets.names, assets.blobs);
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    canvas = fresh;
    fit();
    engine = await start(canvas, false, assets.manifest, assets.names, assets.blobs);
  }
  console.info(`Fristad kjører på ${engine.backend()}, Fristil ${engine.fristil_version()}`);
  // Til feilsøking i konsollen og til testene.
  window.fristad = { engine, openPanel, closePanel };
  $("version").textContent = engine.fristil_version();
  $("weather").textContent = engine.weather();
  new ResizeObserver(fit).observe(canvas);
  fit();
  bindInput();
  requestAnimationFrame(loop);

  status.hidden = true;
  enterBtn.hidden = false;
  enterBtn.focus();
  if (matchMedia("(pointer: coarse)").matches && !matchMedia("(pointer: fine)").matches) {
    $("touch-note").hidden = false;
  }
}

async function loadAssets() {
  const none = { manifest: undefined, names: [], blobs: [] };
  let manifest;
  try {
    const res = await fetch("assets/manifest.json");
    if (!res.ok) return none;
    manifest = await res.text();
  } catch {
    return none;
  }
  const paths = asset_paths(manifest);
  setStatus(`Henter materialer og modeller (${paths.length} filer) …`);
  const blobs = await Promise.all(
    paths.map(async (p) => {
      const r = await fetch(`assets/${p}`);
      if (!r.ok) throw new Error(`${p}: HTTP ${r.status}`);
      return new Uint8Array(await r.arrayBuffer());
    }),
  ).catch((e) => {
    console.warn("Ressursene kunne ikke hentes, bruker de bakte:", e);
    return null;
  });
  if (!blobs) return none;
  setStatus("Bygger byen …");
  return { manifest, names: paths, blobs };
}

function setStatus(text) {
  const span = status.querySelector("span:last-child");
  if (span) span.textContent = text;
}

let last = performance.now();
let frames = 0;
let perfAt = performance.now();
function loop(now) {
  const dt = Math.min((now - last) / 1000, 0.1);
  last = now;
  const events = JSON.parse(engine.frame(dt));
  for (const e of events) handle(e);
  // Bilder per sekund, så eieren kan si hvordan det flyter.
  frames++;
  if (now - perfAt > 1000) {
    $("perf").textContent = `${Math.round((frames * 1000) / (now - perfAt))} fps · ${engine.backend() === "Gl" ? "WebGL2" : "WebGPU"} · ${canvas.width}×${canvas.height}`;
    frames = 0;
    perfAt = now;
  }
  requestAnimationFrame(loop);
}

function handle(e) {
  switch (e.type) {
    case "prompt":
      if (e.component) {
        current = { component: e.component, variant: e.variant };
        $("prompt-name").textContent = docs[e.component]?.title ?? e.component;
        $("prompt-class").textContent = e.class;
        $("prompt-variant").textContent = e.variant;
        $("prompt-variant").hidden = !e.variant;
        $("key-variant").hidden = !e.variants;
        $("key-use").hidden = !e.usable;
        promptEl.hidden = false;
      } else {
        promptEl.hidden = true;
      }
      break;
    case "variant":
      current.variant = e.variant;
      break;
    case "inspect":
      current = { component: e.component, variant: e.variant };
      openPanel();
      break;
    case "theme":
      document.documentElement.dataset.theme = e.theme;
      break;
    case "toast":
      toast?.show?.(e.text);
      break;
    case "weather":
      $("weather").textContent = e.weather;
      break;
  }
}

// Pekerlås: musa fanges når du er i byen, og slippes med Esc.
function lock() {
  const p = canvas.requestPointerLock?.({ unadjustedMovement: true });
  if (p?.catch) p.catch(() => canvas.requestPointerLock());
}

function locked() {
  return document.pointerLockElement === canvas;
}

function bindInput() {
  enterBtn.addEventListener("click", lock);
  canvas.addEventListener("click", () => {
    if (!locked() && !panelOpen) lock();
  });

  document.addEventListener("pointerlockchange", () => {
    if (locked()) {
      gateEl.hidden = true;
      hud.hidden = false;
      engine.set_paused(false);
      canvas.focus();
      fadeHelp();
    } else {
      engine.set_paused(true);
      if (!panelOpen) {
        enterBtn.textContent = "Fortsett";
        gateEl.hidden = false;
        enterBtn.focus();
      }
    }
  });

  document.addEventListener("mousemove", (e) => {
    if (locked()) engine.look(e.movementX, e.movementY);
  });

  const keys = new Set(["Space", "Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);
  document.addEventListener("keydown", (e) => {
    if (panelOpen && e.code === "Escape") {
      closePanel(false);
      return;
    }
    if (!locked()) return;
    if (keys.has(e.code)) e.preventDefault();
    if (e.repeat) return;
    engine.key(e.code, true);
  });
  document.addEventListener("keyup", (e) => {
    if (locked()) engine.key(e.code, false);
  });
  window.addEventListener("blur", () => engine.release_all());

  // Forbindelsen: `.fs-connection-status` viser linja, byen får uvær.
  engine.set_online(navigator.onLine);
  window.addEventListener("offline", () => engine.set_online(false));
  window.addEventListener("online", () => engine.set_online(true));

  $("close").addEventListener("click", () => closePanel(true));
}

let helpTimer = 0;
function fadeHelp() {
  const help = $("help");
  help.classList.remove("faded");
  clearTimeout(helpTimer);
  helpTimer = setTimeout(() => help.classList.add("faded"), 9000);
}

function openPanel() {
  const d = docs[current.component] ?? {};
  panelOpen = true;
  engine.set_paused(true);
  document.exitPointerLock();
  hud.hidden = true;

  $("panel-title").textContent = d.title ?? current.component;
  $("panel-class").textContent = `.fs-${current.component}`;
  $("panel-kind").textContent = d.description?.includes("Frittstående") ? "Frittstående komponent" : "CSS-komponent";
  $("panel-description").textContent = d.description ?? "";
  $("panel-intro").innerHTML = inlineCode(d.intro ?? "");
  $("panel-code").textContent = d.code ?? "";
  $("panel-link").href = d.url ?? "https://fristil.sobernetics.no/";
  renderDemo(d.code ?? "");
  renderVariants();
  panel.hidden = false;
  $("close").focus();
}

function closePanel(relock) {
  panelOpen = false;
  panel.hidden = true;
  if (relock) {
    lock();
  } else {
    enterBtn.textContent = "Fortsett";
    gateEl.hidden = false;
    enterBtn.focus();
  }
}

// Eksempelet fra dokumentasjonen, levende: samme HTML som i Fristil-docs.
function renderDemo(code) {
  const demo = $("demo");
  demo.replaceChildren();
  if (!code) {
    demo.hidden = true;
    return;
  }
  demo.hidden = false;
  const tpl = document.createElement("template");
  tpl.innerHTML = code;
  // Lenker i eksemplene skal ikke føre noen steder.
  for (const a of tpl.content.querySelectorAll("a[href]")) a.setAttribute("href", "#");
  for (const s of tpl.content.querySelectorAll("script")) s.remove();
  demo.append(tpl.content);
}

// Variantene som chips. Velger du en, bytter komponenten i byen.
function renderVariants() {
  const list = $("variants");
  const section = $("variants-section");
  list.replaceChildren();
  const variants = VARIANTS[current.component];
  if (!variants) {
    section.hidden = true;
    return;
  }
  section.hidden = false;
  for (const v of variants) {
    const row = document.createElement("div");
    row.className = "variant";
    row.dataset.active = String(v.name === current.variant);

    const btn = document.createElement("button");
    btn.className = "fs-button";
    if (current.component === "button") {
      if (v.name !== "primary") btn.dataset.variant = v.name;
      btn.textContent = v.label;
    } else {
      btn.dataset.variant = v.name === current.variant ? "primary" : "secondary";
      btn.textContent = v.name;
    }
    btn.setAttribute("aria-pressed", String(v.name === current.variant));
    btn.addEventListener("click", () => {
      current.variant = v.name;
      engine.set_variant(current.component, v.name);
      renderVariants();
    });

    const note = document.createElement("div");
    note.className = "fs-paragraph";
    note.dataset.size = "small";
    note.textContent = v.note;

    row.append(btn, note);
    list.append(row);
  }
}

function inlineCode(text) {
  const esc = text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  return esc.replace(/`([^`]+)`/g, "<code>$1</code>");
}

boot().catch((e) => {
  console.error(e);
  status.classList.add("error");
  status.textContent =
    "Fristad trenger WebGPU eller WebGL2, og fikk ikke startet her. Prøv en oppdatert Chrome, Edge, Safari eller Firefox.";
});
