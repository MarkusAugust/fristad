// JavaScript-laget i Fristad: løkka, input, pekerlås, overleggene og lyden.
// Byen, fysikken og grafikken bor i Rust (pkg/fristad.js og .wasm), bak
// den drivende porten.

import init, { start, asset_paths, model_paths } from "./pkg/fristad.js";
import { defineFsToast } from "./fristil/components/frittstaende/toast/fs-toast.js";
import { defineFsTabs } from "./fristil/components/ramme/tabs/fs-tabs.js";

defineFsToast();
// Boka viser eksempelet fra dokumentasjonen levende, så fanene må være
// registrert for at <fs-tabs> skal koble fanene til panelene.
defineFsTabs();

const $ = (id) => document.getElementById(id);
const gateEl = $("gate");
const hud = $("hud");
const panel = $("panel");
const aimEl = $("aim");
const status = $("status");
const enterBtn = $("enter");
const toast = document.querySelector("fs-toast");

let canvas = $("world");
let engine = null;
let docs = {};
let panelOpen = false;
let current = { component: "" };

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
  // `?webgl` i adressen tvinger WebGL2, til testing.
  const forceGl = new URLSearchParams(location.search).has("webgl");
  try {
    if (forceGl) throw new Error("WebGL2 valgt i adressen");
    engine = await start(canvas, true, assets.manifest, assets.names, assets.blobs, models);
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    canvas = fresh;
    fit();
    engine = await start(canvas, false, assets.manifest, assets.names, assets.blobs, models);
  }
  console.info(`Fristad kjører på ${engine.backend()}, Fristil ${engine.fristil_version()}`);
  // Til feilsøking i konsollen og til testene.
  window.fristad = { engine, openPanel, closePanel, setScale, setQuality };
  $("version").textContent = engine.fristil_version();
  $("weather").textContent = engine.weather();
  new ResizeObserver(fit).observe(canvas);
  fit();
  bindInput();
  requestAnimationFrame(loop);

  status.hidden = true;
  $("loading").hidden = true;
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
  const bar = $("loading");
  bar.hidden = false;
  let done = 0;
  const tick = () => {
    done++;
    bar.value = Math.round((done / paths.length) * 100);
    setStatus(`Henter teksturer (${done} av ${paths.length}) …`);
  };
  setStatus(`Henter teksturer (0 av ${paths.length}) …`);
  // Seks om gangen, så nettleseren ikke åpner hundre forbindelser.
  const blobs = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      const r = await fetch(`assets/${paths[i]}`);
      if (!r.ok) throw new Error(`${paths[i]}: HTTP ${r.status}`);
      blobs[i] = new Uint8Array(await r.arrayBuffer());
      tick();
    }
  };
  const ok = await Promise.all(Array.from({ length: 6 }, worker)).then(
    () => true,
    (e) => {
      console.warn("Teksturene kunne ikke hentes, bruker de bakte:", e);
      return false;
    },
  );
  if (!ok) return none;
  return { manifest, names: paths, blobs };
}

// Henter modellfilene (models/<sti>.gz) og pakker dem ut. Framdriften
// regnes i byte, fra størrelsene i models/sizes.json.
async function loadModels() {
  const paths = model_paths();
  let sizes = {};
  try {
    sizes = await (await fetch("models/sizes.json")).json();
  } catch {
    sizes = {};
  }
  const total = paths.reduce((a, p) => a + (sizes[p] || 1), 0);
  let got = 0;
  const bar = $("loading");
  bar.hidden = false;
  const show = () => {
    bar.value = Math.round((got / total) * 100);
    setStatus(`Henter byen (${(got / 1e6).toFixed(0)} av ${(total / 1e6).toFixed(0)} MB) …`);
  };
  show();
  const out = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      const r = await fetch(`models/${paths[i]}.gz`);
      if (!r.ok) {
        console.warn(`Modellen ${paths[i]} mangler: HTTP ${r.status}`);
        continue;
      }
      // Telle bytene mens de kommer.
      let last = 0;
      const counted = r.body.pipeThrough(
        new TransformStream({
          transform(chunk, ctl) {
            got += chunk.byteLength;
            if (got - last > 256 * 1024) {
              last = got;
              show();
            }
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
      out[i] = bytes;
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
// Intern oppløsning: senkes automatisk når bildefrekvensen faller, og heves
// igjen når det flyter. `setScale(tall)` låser den, `setScale(null)` slipper.
// Først senkes kvaliteten (færre prøver i skygger, SSAO og tåke: 1 → 0.5 → 0),
// så oppløsningen. Opp igjen i motsatt rekkefølge.
let scale = 1;
let quality = 1;
let autoScale = true;
let slowSeconds = 0;
let fastSeconds = 0;
function setScale(s) {
  autoScale = s === null || s === undefined;
  if (!autoScale) {
    scale = Math.min(1, Math.max(0.4, s));
    engine.set_scale(scale);
  }
}
function setQuality(q) {
  autoScale = q === null || q === undefined;
  if (!autoScale) {
    quality = Math.min(1, Math.max(0, q));
    engine.set_quality(quality);
  }
}
function adapt(fps) {
  if (!autoScale || !playing) return;
  if (fps < 40) {
    slowSeconds++;
    fastSeconds = 0;
  } else if (fps > 57) {
    fastSeconds++;
    slowSeconds = 0;
  } else {
    slowSeconds = 0;
    fastSeconds = 0;
  }
  if (slowSeconds >= 2) {
    if (quality > 0) {
      quality = Math.max(0, quality - 0.5);
      engine.set_quality(quality);
    } else if (scale > 0.5) {
      scale = Math.round((scale - 0.1) * 10) / 10;
      engine.set_scale(scale);
    }
    slowSeconds = 0;
  } else if (fastSeconds >= 5) {
    if (scale < 1) {
      scale = Math.round((scale + 0.1) * 10) / 10;
      engine.set_scale(scale);
    } else if (quality < 1) {
      quality = Math.min(1, quality + 0.5);
      engine.set_quality(quality);
    }
    fastSeconds = 0;
  }
}
function loop(now) {
  const dt = Math.min((now - last) / 1000, 0.1);
  last = now;
  const events = JSON.parse(engine.frame(dt));
  for (const e of events) handle(e);
  // Bilder per sekund, så eieren kan si hvordan det flyter.
  frames++;
  if (now - perfAt > 1000) {
    const fps = Math.round((frames * 1000) / (now - perfAt));
    adapt(fps);
    const tris = Math.round(engine.triangles() / 1000);
    const res = `${Math.round(canvas.width * scale)}×${Math.round(canvas.height * scale)}`;
    const q = quality >= 1 ? "" : quality > 0 ? " · middels" : " · lav";
    $("perf").textContent = `${fps} fps · ${engine.backend() === "Gl" ? "WebGL2" : "WebGPU"} · ${res}${scale < 1 ? ` (${Math.round(scale * 100)} %)` : ""}${q} · ${tris}k △`;
    frames = 0;
    perfAt = now;
  }
  requestAnimationFrame(loop);
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
        aimEl.classList.add("is-open");
      } else {
        aimEl.classList.remove("is-open");
      }
      break;
    case "fade":
      fade();
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
    case "hud":
      $("hp").value = e.hp;
      $("stamina").value = e.stamina;
      $("tokens").textContent = `${e.tokens} tokens`;
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
      break;
    case "sound":
      sfx.play(e.name);
      break;
    case "marks":
      updateMarks(e.marks);
      break;
    case "speech":
      updateBubbles(e.bubbles);
      break;
    case "inventory":
      renderInventory(e);
      try {
        localStorage.setItem("fristad.save", engine.save_state());
      } catch {
        /* lagring er en bekvemmelighet */
      }
      break;
    case "shop":
      openShop(e.npc, e.wares);
      break;
    case "workshop":
      openWorkshop(e.recipes);
      break;
  }
}

// Lageret og utstyret i hjørnet: utstyret som linjer, komponentene som
// chips du kan lese om i boka (det gir liv tilbake, én gang per tur).
function renderInventory(e) {
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
  engine.read_doc(component);
  if (docs[component]) {
    current = { component };
    closeOverlay(false);
    openPanel();
  }
}

// Ned i krypta og opp igjen: skjermen går i svart et øyeblikk.
function fade() {
  const el = $("fade");
  el.classList.remove("on");
  void el.offsetWidth;
  el.classList.add("on");
}

// Butikk og verksted: en enkel overleggsboks (ikke designsystemet), med
// spillet på pause mens den står.
let overlayOpen = false;
function openOverlay(title) {
  overlayOpen = true;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  $("overlay-title").textContent = title;
  $("overlay-body").replaceChildren();
  $("overlay").hidden = false;
  return $("overlay-body");
}

function closeOverlay(resume = true) {
  if (!overlayOpen) return;
  overlayOpen = false;
  $("overlay").hidden = true;
  if (resume) enter(false);
}

function openShop(npc, wares) {
  const body = openOverlay(npc);
  const list = document.createElement("div");
  list.className = "wares";
  for (const w of wares) {
    const row = document.createElement("div");
    row.className = "ware";
    const name = document.createElement("span");
    name.textContent = w.name;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "fs-button";
    btn.dataset.variant = "secondary";
    btn.textContent = `Kjøp (${w.price} tokens)`;
    btn.addEventListener("click", () => {
      engine.buy(w.id);
    });
    row.append(name, btn);
    list.appendChild(row);
  }
  body.appendChild(list);
}

// Skjemakontoret: oppskriften er et Fristil-skjema. Feltene er
// komponentene som trengs, feilteksten sier hva som mangler.
function openWorkshop(recipes) {
  const body = openOverlay("Skriptoriet");
  const form = document.createElement("form");
  form.className = "workshop";
  form.innerHTML = `
    <p class="fs-paragraph" data-size="small">Søknad om smiing. Velg oppskrift, så fylles feltene ut fra lageret ditt.</p>
    <label class="fs-label" for="recipe">Oppskrift</label>
    <select id="recipe" class="fs-select"></select>
    <p id="recipe-result" class="fs-help-text"></p>
    <div id="recipe-fields"></div>
    <div class="panel-actions"><button type="submit" class="fs-button" data-variant="primary">Send søknad</button></div>`;
  body.appendChild(form);
  const sel = form.querySelector("#recipe");
  for (const r of recipes) {
    const o = document.createElement("option");
    o.value = r.id;
    o.textContent = `${r.name} (${r.rarity})${r.ok ? "" : " · mangler"}`;
    sel.appendChild(o);
  }
  const fields = form.querySelector("#recipe-fields");
  const render = () => {
    const r = recipes.find((x) => x.id === sel.value) ?? recipes[0];
    form.querySelector("#recipe-result").textContent = r.result;
    fields.replaceChildren();
    for (const [c, need, have] of r.needs) {
      const wrap = document.createElement("div");
      wrap.className = "field";
      wrap.innerHTML = `<label class="fs-label">${esc(c)}</label><input class="fs-input" readonly value="${need} stk (har ${have})"><p class="fs-error-text" ${have >= need ? "hidden" : ""}>Mangler ${Math.max(0, need - have)}. Finnes i krypta og hos Knappesmeden.</p>`;
      fields.appendChild(wrap);
    }
    if (r.requires.length) {
      const fs = document.createElement("fieldset");
      fs.className = "fs-fieldset";
      fs.innerHTML = `<legend>Avanserte felt (må være i lageret)</legend>`;
      for (const [c, ok] of r.requires) {
        const p = document.createElement("p");
        p.className = ok ? "fs-help-text" : "fs-error-text";
        p.textContent = ok ? `${c}: i lageret` : `${c}: mangler`;
        fs.appendChild(p);
      }
      fields.appendChild(fs);
    }
  };
  sel.addEventListener("change", render);
  render();
  form.addEventListener("submit", (ev) => {
    ev.preventDefault();
    if (engine.craft(sel.value)) closeOverlay(true);
  });
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

// Lyd: syntetisert med WebAudio, ingen filer. Konteksten lages ved
// første lyd (etter at spilleren har klikket seg inn).
const sfx = (() => {
  let ctx = null;
  const get = () => {
    if (!ctx) {
      try {
        ctx = new (window.AudioContext || window.webkitAudioContext)();
      } catch {
        return null;
      }
    }
    if (ctx.state === "suspended") ctx.resume().catch(() => {});
    return ctx;
  };
  const noise = (c, dur) => {
    const b = c.createBuffer(1, Math.ceil(c.sampleRate * dur), c.sampleRate);
    const d = b.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
    const s = c.createBufferSource();
    s.buffer = b;
    return s;
  };
  const env = (c, peak, attack, decay, at = 0) => {
    const g = c.createGain();
    const t = c.currentTime + at;
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(peak, t + attack);
    g.gain.exponentialRampToValueAtTime(0.0001, t + attack + decay);
    g.connect(c.destination);
    return g;
  };
  const tone = (c, type, f0, f1, peak, attack, decay, at = 0) => {
    const o = c.createOscillator();
    o.type = type;
    const t = c.currentTime + at;
    o.frequency.setValueAtTime(f0, t);
    o.frequency.exponentialRampToValueAtTime(Math.max(1, f1), t + attack + decay);
    o.connect(env(c, peak, attack, decay, at));
    o.start(t);
    o.stop(t + attack + decay + 0.05);
  };
  const whoosh = (c, f0, f1, peak, dur, at = 0) => {
    const s = noise(c, dur + 0.1);
    const f = c.createBiquadFilter();
    f.type = "bandpass";
    f.Q.value = 1.4;
    const t = c.currentTime + at;
    f.frequency.setValueAtTime(f0, t);
    f.frequency.exponentialRampToValueAtTime(f1, t + dur * 0.4);
    f.frequency.exponentialRampToValueAtTime(f0 * 0.7, t + dur);
    s.connect(f).connect(env(c, peak, dur * 0.25, dur * 0.75, at));
    s.start(t);
  };
  const crunch = (c, peak, dur, cutoff, at = 0) => {
    const s = noise(c, dur + 0.05);
    const f = c.createBiquadFilter();
    f.type = "lowpass";
    f.frequency.value = cutoff;
    s.connect(f).connect(env(c, peak, 0.005, dur, at));
    s.start(c.currentTime + at);
  };
  const play = {
    swing: (c) => whoosh(c, 500, 2600, 0.35, 0.28),
    swing_heavy: (c) => whoosh(c, 220, 1400, 0.5, 0.5),
    hit: (c) => {
      tone(c, "sine", 140, 45, 0.7, 0.005, 0.18);
      crunch(c, 0.45, 0.12, 2500);
    },
    hit_heavy: (c) => {
      tone(c, "sine", 110, 35, 0.9, 0.005, 0.3);
      crunch(c, 0.6, 0.2, 1800);
      tone(c, "square", 400, 90, 0.08, 0.005, 0.1);
    },
    kill: (c) => {
      tone(c, "sawtooth", 180, 30, 0.5, 0.01, 0.6);
      crunch(c, 0.6, 0.45, 1200);
      tone(c, "square", 900, 200, 0.12, 0.01, 0.35, 0.05);
    },
    block: (c) => {
      tone(c, "square", 700, 500, 0.25, 0.003, 0.09);
      crunch(c, 0.3, 0.08, 4000);
    },
    parry: (c) => {
      tone(c, "sine", 1900, 1700, 0.35, 0.003, 0.5);
      tone(c, "sine", 2850, 2600, 0.2, 0.003, 0.4);
      crunch(c, 0.25, 0.05, 6000);
    },
    hurt: (c) => {
      tone(c, "sine", 90, 40, 0.8, 0.005, 0.25);
      crunch(c, 0.35, 0.15, 900);
    },
    dodge: (c) => whoosh(c, 180, 700, 0.3, 0.3),
    clang: (c) => {
      tone(c, "square", 1200, 900, 0.2, 0.003, 0.12);
      tone(c, "sine", 2400, 2300, 0.12, 0.003, 0.25);
      crunch(c, 0.2, 0.06, 5000);
    },
    spit: (c) => {
      whoosh(c, 900, 3200, 0.3, 0.22);
      tone(c, "sawtooth", 600, 1400, 0.12, 0.01, 0.15);
    },
    slam: (c) => {
      tone(c, "sine", 60, 25, 1.0, 0.005, 0.7);
      crunch(c, 0.7, 0.5, 600);
      crunch(c, 0.3, 0.9, 200, 0.05);
    },
    roar: (c) => {
      const o = c.createOscillator();
      o.type = "sawtooth";
      const t = c.currentTime;
      o.frequency.setValueAtTime(45, t);
      o.frequency.linearRampToValueAtTime(70, t + 0.5);
      o.frequency.linearRampToValueAtTime(40, t + 1.3);
      const v = c.createOscillator();
      v.frequency.value = 7;
      const vg = c.createGain();
      vg.gain.value = 12;
      v.connect(vg).connect(o.frequency);
      const f = c.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = 500;
      o.connect(f).connect(env(c, 0.5, 0.1, 1.3));
      o.start(t);
      v.start(t);
      o.stop(t + 1.5);
      v.stop(t + 1.5);
      crunch(c, 0.3, 1.0, 400, 0.1);
    },
    growl: (c) => {
      const o = c.createOscillator();
      o.type = "sawtooth";
      const t = c.currentTime;
      o.frequency.setValueAtTime(70, t);
      o.frequency.linearRampToValueAtTime(95, t + 0.4);
      const v = c.createOscillator();
      v.frequency.value = 11;
      const vg = c.createGain();
      vg.gain.value = 9;
      v.connect(vg).connect(o.frequency);
      const f = c.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = 700;
      o.connect(f).connect(env(c, 0.3, 0.05, 0.5));
      o.start(t);
      v.start(t);
      o.stop(t + 0.6);
      v.stop(t + 0.6);
    },
    talk: (c) => {
      // Et lite mumlende «hm-hm».
      tone(c, "triangle", 220, 180, 0.12, 0.02, 0.12);
      tone(c, "triangle", 260, 200, 0.1, 0.02, 0.14, 0.14);
    },
    pickup: (c) => {
      tone(c, "sine", 880, 880, 0.2, 0.005, 0.12);
      tone(c, "sine", 1320, 1320, 0.2, 0.005, 0.25, 0.08);
    },
    death: (c) => {
      tone(c, "sawtooth", 160, 30, 0.5, 0.02, 1.4);
      crunch(c, 0.4, 0.8, 600);
    },
    door: (c) => {
      // Tung tredør med jernbeslag: knirk og et dunk.
      tone(c, "sawtooth", 140, 210, 0.08, 0.05, 0.5);
      crunch(c, 0.5, 0.25, 500, 0.45);
      tone(c, "sine", 70, 40, 0.5, 0.005, 0.3, 0.45);
    },
    well: (c) => {
      // Brønnen: valsen surrer når bøtta slippes, plask i vannet, og så
      // knirker sveiven i rykk mens bøtta heises opp.
      tone(c, "triangle", 320, 120, 0.05, 0.02, 0.95);
      crunch(c, 0.35, 0.45, 900, 0.95);
      tone(c, "sine", 180, 90, 0.12, 0.005, 0.25, 0.97);
      for (let k = 0; k < 10; k++) {
        const at = 1.45 + k * 0.24;
        tone(c, "sawtooth", 115 + (k % 2) * 35, 95, 0.045, 0.01, 0.12, at);
        crunch(c, 0.06, 0.05, 1800, at + 0.06);
      }
    },
  };
  return {
    play(name) {
      const c = get();
      if (!c || !play[name]) return;
      try {
        play[name](c);
      } catch {
        /* lyden er pynt */
      }
    },
  };
})();

// Du er «inne» når startkortet er borte. Musa fanges med pekerlås når
// nettleseren gir oss den; ellers ser du deg rundt ved å dra med musa, så
// knappen virker også i innebygde nettlesere uten pekerlås.
let playing = false;
let dragging = false;

function lock() {
  const p = canvas.requestPointerLock?.({ unadjustedMovement: true });
  if (p?.catch) p.catch(() => canvas.requestPointerLock?.());
}

function locked() {
  return document.pointerLockElement === canvas;
}

let loadedSave = false;
function enter(relock = true) {
  if (!loadedSave) {
    loadedSave = true;
    try {
      const saved = localStorage.getItem("fristad.save");
      if (saved) engine.load_state(saved);
    } catch {
      /* ingen lagring */
    }
  }
  playing = true;
  gateEl.hidden = true;
  hud.hidden = false;
  engine.set_paused(false);
  canvas.focus();
  fadeHelp();
  if (relock) lock();
}

function quit() {
  playing = false;
  dragging = false;
  panelOpen = false;
  panel.hidden = true;
  engine.set_paused(true);
  engine.respawn();
  if (locked()) document.exitPointerLock?.();
  enterBtn.textContent = "Gå inn i Fristad";
  $("respawn").hidden = true;
  $("quit").hidden = true;
  hud.hidden = true;
  gateEl.hidden = false;
  enterBtn.focus();
}

function pause() {
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  if (!panelOpen) {
    enterBtn.textContent = "Fortsett";
    $("respawn").hidden = false;
    $("quit").hidden = false;
    gateEl.hidden = false;
    enterBtn.focus();
  }
}

function bindInput() {
  enterBtn.addEventListener("click", enter);
  // Sitter du fast, tar knappen i pausemenyen (og H) deg tilbake til sørporten.
  $("respawn").addEventListener("click", () => {
    engine.respawn();
    enter();
  });
  // Avslutt: tilbake til startskjermen, med spilleren ved sørporten igjen.
  $("quit").addEventListener("click", quit);
  canvas.addEventListener("click", () => {
    if (playing && !locked()) lock();
  });

  document.addEventListener("pointerlockchange", () => {
    // Slippes låsen mens du spiller (Esc), er det pause.
    if (!locked() && playing && !panelOpen) pause();
  });

  // Musetastene er hugg og blokk når musa er fanget; uten pekerlås drar
  // venstre tast kameraet.
  canvas.addEventListener("mousedown", (e) => {
    if (!playing) return;
    if (locked()) {
      e.preventDefault();
      engine.mouse(e.button, true);
    } else if (e.button === 0) {
      dragging = true;
    }
  });
  window.addEventListener("mouseup", (e) => {
    dragging = false;
    if (playing) engine.mouse(e.button, false);
  });
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener("mousemove", (e) => {
    if (!playing) return;
    if (locked() || dragging) engine.look(e.movementX, e.movementY);
  });

  const keys = new Set(["Space", "Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);
  document.addEventListener("keydown", (e) => {
    // Esc med panelet oppe lukker panelet og tar deg rett tilbake i spillet,
    // uten å be om pekerlås (Esc gir ingen brukerhandling å låse på, og et
    // avslått forsøk åpnet pausemenyen). Klikk i bildet låser igjen; neste
    // Esc åpner menyen.
    if (panelOpen && e.code === "Escape") {
      e.preventDefault();
      closePanel("resume");
      return;
    }
    if (overlayOpen) {
      if (e.code === "Escape") {
        e.preventDefault();
        closeOverlay(true);
      }
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
  $("overlay-close").addEventListener("click", () => closeOverlay(true));
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
  $("panel-class").textContent = `.fs-${current.component}`;
  $("panel-kind").textContent = d.description?.includes("Frittstående") ? "Frittstående komponent" : "CSS-komponent";
  $("panel-description").textContent = d.description ?? "";
  $("panel-intro").innerHTML = inlineCode(d.intro ?? "");
  $("panel-code").textContent = d.code ?? "";
  $("panel-link").href = d.url ?? "https://fristil.sobernetics.no/";
  renderDemo(d.code ?? "");
  panel.hidden = false;
  $("close").focus();
}

function closePanel(relock) {
  panelOpen = false;
  panel.hidden = true;
  if (relock === "resume") {
    enter(false);
  } else if (relock) {
    enter();
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
