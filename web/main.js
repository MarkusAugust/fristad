// JavaScript-laget i Fristad: løkka, input, pekerlås og panelene.
// Verden, fysikk og grafikk bor i Rust (pkg/fristad.js og .wasm).

import init, { start } from "./pkg/fristad.js";

const $ = (id) => document.getElementById(id);
const gateEl = $("gate");
const hud = $("hud");
const panel = $("panel");
const promptEl = $("prompt");
const status = $("status");
const enterBtn = $("enter");

let canvas = $("world");
let engine = null;
let docs = {};
let panelOpen = false;
let current = { component: "button", variant: "primary" };

const VARIANTS = [
  { name: "primary", label: "Send søknad", note: "Standard. Trenger ingen data-variant." },
  { name: "secondary", label: "Lagre utkast", note: "Omriss, gjennomsiktig bakgrunn." },
  { name: "ghost", label: "Legg til vedlegg", note: "Ingen kant, ingen bakgrunn." },
  { name: "danger", label: "Slett søknad", note: "For handlinger som ikke kan angres." },
];

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
  try {
    engine = await start(canvas, true);
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    canvas = fresh;
    fit();
    engine = await start(canvas, false);
  }
  console.info(`Fristad kjører på ${engine.backend()}, Fristil ${engine.fristil_version()}`);
  $("version").textContent = engine.fristil_version();
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

let last = performance.now();
function loop(now) {
  const dt = Math.min((now - last) / 1000, 0.1);
  last = now;
  const events = JSON.parse(engine.frame(dt));
  for (const e of events) handle(e);
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

  $("close").addEventListener("click", () => closePanel(true));
}

let helpTimer = 0;
function fadeHelp() {
  const help = $("help");
  help.classList.remove("faded");
  clearTimeout(helpTimer);
  helpTimer = setTimeout(() => help.classList.add("faded"), 7000);
}

function openPanel() {
  const d = docs[current.component] ?? {};
  panelOpen = true;
  engine.set_paused(true);
  document.exitPointerLock();
  hud.hidden = true;

  $("panel-title").textContent = d.title ?? current.component;
  $("panel-class").textContent = `.fs-${current.component}`;
  $("panel-description").textContent = d.description ?? "";
  $("panel-intro").innerHTML = inlineCode(d.intro ?? "");
  $("panel-code").textContent = d.code ?? "";
  $("panel-link").href = d.url ?? "https://fristil.sobernetics.no/";
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

// Ekte Fristil-knapper. Velger du en, bytter knappen i byen variant.
function renderVariants() {
  const list = $("variants");
  list.replaceChildren();
  for (const v of VARIANTS) {
    const row = document.createElement("div");
    row.className = "variant";
    row.dataset.active = String(v.name === current.variant);

    const btn = document.createElement("button");
    btn.className = "fs-button";
    if (v.name !== "primary") btn.dataset.variant = v.name;
    btn.textContent = v.label;
    btn.setAttribute("aria-pressed", String(v.name === current.variant));
    btn.addEventListener("click", () => {
      current.variant = v.name;
      engine.set_variant(v.name);
      renderVariants();
    });

    const info = document.createElement("div");
    const code = document.createElement("code");
    code.textContent = v.name === "primary" ? "uten data-variant" : `data-variant="${v.name}"`;
    const note = document.createElement("div");
    note.className = "fs-paragraph";
    note.dataset.size = "small";
    note.textContent = v.note;
    info.append(code, note);

    row.append(btn, info);
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
