// JavaScript-laget i Fristad: løkka, input, pekerlås, overleggene og lyden.
// Byen, fysikken og grafikken bor i Rust (pkg/fristad.js og .wasm), bak
// den drivende porten.

import init, { start, asset_paths, model_paths } from "./pkg/fristad.js";
import { createMusic } from "./music.js";
import { createAmbience } from "./ambience.js";
import { createSounds } from "./sounds.js";
import { defineFsToast } from "./fristil/components/frittstaende/toast/fs-toast.js";
import { defineFsTabs } from "./fristil/components/ramme/tabs/fs-tabs.js";
import { defineFsDialog } from "./fristil/components/ramme/dialog/fs-dialog.js";
import { defineFsConnectionStatus } from "./fristil/components/frittstaende/connection-status/fs-connection-status.js";

defineFsToast();
// Boka viser eksempelet fra dokumentasjonen levende, så komponentene med
// oppførsel må være registrert, som i Fristil-docs.
defineFsTabs();
defineFsDialog();
defineFsConnectionStatus();

const $ = (id) => document.getElementById(id);
const gateEl = $("gate");
const hud = $("hud");
const panel = $("panel");
const aimEl = $("aim");
const status = $("status");
const enterBtn = $("enter");
const toast = document.querySelector("fs-toast");

let canvas = $("world");
// Grafikken: avansert (alt på) eller redusert (to skyggekaskader, ingen SSAO
// eller glød, færre lys, grovere detalj, lavere oppløsning) for vanlige
// jobb-PC-er. Valget huskes; uten valg byttes det til redusert av seg selv
// om bildefrekvensen blir liggende lavt.
let reducedGfx = false;
let gfxChosen = false;
try {
  const g = localStorage.getItem("fristad.graphics");
  gfxChosen = g === "redusert" || g === "avansert";
  reducedGfx = g === "redusert";
} catch {
  /* ingen lagring */
}
let engine = null;
let docs = {};
let panelOpen = false;
let current = { component: "" };

// Lerretet tegnes i en litt lavere oppløsning enn skjermen på skjermer med
// høy pikseltetthet. Forskjellen synes knapt, men GPU-en får mye mindre å gjøre.
// I redusert grafikk tegnes det aldri tettere enn én piksel per CSS-piksel.
function fit() {
  const scale = Math.min(window.devicePixelRatio || 1, reducedGfx ? 1 : 1.5);
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
  window.fristad = { engine, openPanel, closePanel, setScale, setQuality, setGraphics };
  setGraphics(reducedGfx, false, false);
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
// Byttes grafikken: motoren, kvaliteten, oppløsningen og knappene.
function setGraphics(reduced, chosen = true, announce = true) {
  reducedGfx = reduced;
  if (chosen) {
    gfxChosen = true;
    try {
      localStorage.setItem("fristad.graphics", reduced ? "redusert" : "avansert");
    } catch {
      /* ingen lagring */
    }
  }
  if (engine) {
    engine.set_reduced(reduced);
    quality = reduced ? 0 : 1;
    engine.set_quality(quality);
    scale = reduced ? 0.75 : 1;
    engine.set_scale(scale);
  }
  slowSeconds = 0;
  fastSeconds = 0;
  lowStreak = 0;
  fit();
  const name = reduced ? "redusert" : "avansert";
  $("graphics").textContent = `Grafikk: ${name}`;
  $("graphics").setAttribute("aria-pressed", String(reduced));
  $("graphics-label").textContent = name;
  if (announce) toast?.show?.(reduced ? "Redusert grafikk: lettere å kjøre, litt enklere bilde. G bytter tilbake." : "Avansert grafikk: alt på. G bytter til redusert.");
}
let lowStreak = 0;
let suggested = false;
function adapt(fps) {
  if (!playing) return;
  // Går det tregt selv med lav kvalitet og oppløsning i avansert grafikk,
  // byttes det til redusert (eller foreslås, om avansert er valgt).
  if (!reducedGfx && quality === 0 && scale <= 0.6 && fps < 28) {
    lowStreak++;
    if (lowStreak >= 4) {
      lowStreak = 0;
      if (!gfxChosen) {
        setGraphics(true, false, false);
        toast?.show?.("Spillet gikk tregt, så grafikken er satt til redusert. G bytter.");
      } else if (!suggested) {
        suggested = true;
        toast?.show?.("Spillet går tregt. Trykk G for redusert grafikk.");
      }
    }
  } else {
    lowStreak = 0;
  }
  if (!autoScale) return;
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
    if (quality > 0 && !reducedGfx) {
      quality = Math.max(0, quality - 0.5);
      engine.set_quality(quality);
    } else if (scale > 0.5) {
      scale = Math.round((scale - 0.1) * 10) / 10;
      engine.set_scale(scale);
    }
    slowSeconds = 0;
  } else if (fastSeconds >= 5) {
    if (scale < (reducedGfx ? 0.85 : 1)) {
      scale = Math.round((scale + 0.1) * 10) / 10;
      engine.set_scale(scale);
    } else if (quality < 1 && !reducedGfx) {
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

// Et lite klikk når du trykker på en knapp i dialogene og panelet.
document.addEventListener("click", (ev) => {
  if (ev.target.closest?.("dialog button, .fs-button, #panel button")) sfx.play("click");
});

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
      // Pusten går når utholdenheten er brukt opp, og hjertet slår når
      // livet er lavt.
      if (playing && e.stamina === 0 && lastHud && lastHud.stamina > 0) sfx.play("breath");
      if (playing) sfx.heart(e.hp);
      lastHud = e;
      if (playing) music.set(e.cellar ? "cellar" : "town");
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
      try {
        localStorage.setItem("fristad.save", engine.save_state());
      } catch {
        /* lagring er en bekvemmelighet */
      }
      break;
    case "loot":
      openLoot(e.component);
      break;
    case "ambience":
      lastAmbience = e;
      if (playing) {
        ambience.set(e);
        sfx.ambience(e);
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
let lastHud = { hp: 100, stamina: 100, tokens: 0, keys: 0 };
let lastInv = null;
function renderInventory(e) {
  lastInv = e;
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

// Kista: en Fristil-dialog som sier hva du fant, og lar deg lese om
// komponenten i boka (panelet til høyre). Spillet står på pause så lenge.
let lootOpen = false;
function openLoot(component) {
  lootOpen = true;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const d = docs[component] ?? {};
  const title = d.title ?? component;
  const dlg = document.createElement("dialog");
  dlg.className = "fs-dialog loot";
  dlg.setAttribute("aria-labelledby", "loot-title");
  const found = component
    ? `<p>Under lokket, mellom råtne filler og knokler, ligger <strong>${esc(title)}</strong>: en bit av Fristil, slik den så ut før den ble korrupt.</p>
       <p class="fs-help-text">${esc(d.description ?? "")}</p>
       <p>Bær den opp til byen, så er den din. Dør du før det, blir den liggende her nede.</p>`
    : `<p>Kista er tom. Noen har vært her før deg.</p>`;
  dlg.innerHTML = `
    <h2 class="fs-dialog__title" id="loot-title">${component ? "Du fant noe i kista" : "Ingenting"}</h2>
    <div class="fs-dialog__body">${found}</div>
    <div class="fs-dialog__footer">
      <button class="fs-button" data-variant="secondary" type="button" data-act="take">${component ? "Ta med og gå videre" : "Gå videre"}</button>
      ${component && docs[component] ? `<button class="fs-button" data-variant="primary" type="button" data-act="read">Les om ${esc(title)}</button>` : ""}
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

// Inventaret (I): utstyret du har på deg og komponentene du har, bygd med
// Fristils dialog, faner, kort, tabell, merker og framdriftsstolper.
let invOpen = null;
function toggleInventory() {
  if (invOpen) {
    closeInventory(true);
    return;
  }
  const e = lastInv ?? { stash: [], carried: [], weapon: "Sverd", weapon_special: "", shield: "Rundskjold", armor: "Gambeson", lantern: false, max_hp: 100, max_stamina: 100, tokens: 0, read: [] };
  const h = lastHud;
  playing = false;
  dragging = false;
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const title = (c) => docs[c]?.title ?? c;
  const counts = new Map();
  for (const [c, n] of e.stash) counts.set(c, { stash: n, carried: 0 });
  for (const [c, n] of e.carried) counts.set(c, { stash: counts.get(c)?.stash ?? 0, carried: n });
  const rows = [...counts.entries()]
    .sort((a, b) => title(a[0]).localeCompare(title(b[0]), "nb"))
    .map(([c, k]) => `<tr>
        <th scope="row">${esc(title(c))}</th>
        <td data-align="end">${k.stash}</td>
        <td data-align="end">${k.carried ? `<span class="fs-badge" data-color="warning">${k.carried}</span>` : "0"}</td>
        <td>${e.read.includes(c) ? '<span class="fs-badge" data-color="success">Lest</span>' : '<span class="fs-badge" data-color="neutral">Ulest</span>'}</td>
        <td data-align="end">${docs[c] ? `<button class="fs-button" data-variant="secondary" type="button" data-read="${esc(c)}">Les</button>` : ""}</td>
      </tr>`)
    .join("");
  const slot = (label, name, note, tag) => `<div class="fs-card" data-variant="filled">
      <p class="eyebrow">${label}</p>
      <h3 class="fs-card__title">${esc(name)}</h3>
      ${note ? `<p class="fs-help-text">${esc(note)}</p>` : ""}
      ${tag ? `<span class="fs-tag" data-variant="filled">${esc(tag)}</span>` : ""}
    </div>`;
  const dlg = document.createElement("dialog");
  dlg.className = "fs-dialog inventory-dialog";
  dlg.setAttribute("aria-labelledby", "inv-title");
  dlg.innerHTML = `
    <h2 class="fs-dialog__title" id="inv-title">Utstyr og lager</h2>
    <div class="fs-dialog__body">
      <div class="inv-stats">
        <label class="fs-label">Liv ${h.hp} av ${e.max_hp}<progress class="fs-progress" data-color="danger" value="${h.hp}" max="${e.max_hp}"></progress></label>
        <label class="fs-label">Utholdenhet ${h.stamina} av ${e.max_stamina}<progress class="fs-progress" value="${h.stamina}" max="${e.max_stamina}"></progress></label>
        <div class="inv-badges">
          <span class="fs-badge" data-color="warning">${e.tokens} tokens</span>
          <span class="fs-badge" data-color="neutral">${h.keys === 1 ? "1 nøkkel" : `${h.keys} nøkler`}</span>
        </div>
      </div>
      <fs-tabs>
        <div class="fs-tabs__list" aria-label="Inventaret">
          <button>Utstyr</button>
          <button>Komponenter (${counts.size})</button>
        </div>
        <div class="fs-tabs__panel">
          <div class="inv-slots">
            ${slot("Våpen", e.weapon, e.weapon_special, "I høyre hånd")}
            ${slot("Skjold", e.shield, "", "I venstre hånd")}
            ${slot("Rustning", e.armor, "", "På kroppen")}
            ${slot("Lys", e.lantern ? "Søkelykt" : "Fakkel", e.lantern ? "Lyser lenger inn i mørket." : "", "")}
          </div>
        </div>
        <div class="fs-tabs__panel">
          ${counts.size
            ? `<table class="fs-table" data-variant="striped">
                <caption>Komponentene du har vunnet. Det du bærer, er ditt først når du er oppe igjen.</caption>
                <thead><tr><th scope="col">Komponent</th><th scope="col" data-align="end">I lageret</th><th scope="col" data-align="end">Bærer</th><th scope="col">Boka</th><th scope="col"></th></tr></thead>
                <tbody>${rows}</tbody>
              </table>`
            : `<p class="fs-paragraph">Du har ingen komponenter ennå. De ligger i kistene og hos fiendene i krypten.</p>`}
        </div>
      </fs-tabs>
    </div>
    <div class="fs-dialog__footer">
      <button class="fs-button" data-variant="primary" type="button" data-act="close">Lukk (I)</button>
    </div>`;
  document.body.appendChild(dlg);
  invOpen = dlg;
  dlg.querySelector('[data-act="close"]').addEventListener("click", () => closeInventory(true));
  for (const b of dlg.querySelectorAll("[data-read]")) {
    b.addEventListener("click", () => {
      const c = b.dataset.read;
      closeInventory(false);
      readDoc(c);
    });
  }
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    closeInventory(true);
  });
  sfx.play("bag");
  dlg.showModal();
}

function closeInventory(resume) {
  if (!invOpen) return;
  invOpen.close();
  invOpen.remove();
  invOpen = null;
  if (resume) enter();
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

// Lukkes med knappen eller E (en brukerhandling), fanges musa igjen med en
// gang, så du kan se deg rundt uten å klikke. Med Esc gir ikke nettleseren
// oss musa; da sier en melding at et klikk i bildet tar deg tilbake.
function closeOverlay(resume = true, relock = true) {
  if (!overlayOpen) return;
  overlayOpen = false;
  $("overlay").hidden = true;
  if (resume) enter(relock);
  if (resume && !relock) clickHint();
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
      wrap.innerHTML = `<label class="fs-label">${esc(c)}</label><input class="fs-input" readonly value="${need} stk (har ${have})"><p class="fs-error-text" ${have >= need ? "hidden" : ""}>Mangler ${Math.max(0, need - have)}. Finnes i krypten og hos Knappesmeden.</p>`;
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

// Lydene (web/sounds.js): opptak og syntetiske lyder, med retning og etterklang.
const sfx = createSounds();

// Musikken (web/music.js): fem stykker etter tur i byen, uhygge i krypten.
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
  const p = canvas.requestPointerLock?.({ unadjustedMovement: true });
  if (p?.catch) p.catch(() => canvas.requestPointerLock?.());
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
  if (!soundsLoading) {
    soundsLoading = true;
    sfx.preload();
  }
  music.set(lastHud.cellar ? "cellar" : "town");
  if (lastAmbience) {
    ambience.set(lastAmbience);
    sfx.ambience(lastAmbience);
  }
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
  if (playing) sfx.play("pause");
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
  // Ga ikke nettleseren oss musa, går det an å spille uten: dra for å se,
  // klikk for å hugge.
  document.addEventListener("pointerlockerror", () => {
    if (playing) clickHint();
  });

  // Musetastene er hugg og blokk. Når musa er fanget, går hvert trykk rett
  // til spillet. Uten pekerlås (noen nettlesere og jobb-PC-er gir den ikke)
  // drar venstre tast kameraet, og et kort klikk uten å dra er et hugg;
  // høyre tast blokker uansett. Trykk i HUD-en over bildet teller òg, men
  // ikke i dialogene.
  let dragDist = 0;
  const inGame = (t) => t === canvas || (t instanceof Element && !t.closest("dialog, #overlay, #panel, #gate, button, a, input"));
  document.addEventListener("mousedown", (e) => {
    if (!playing || !inGame(e.target)) return;
    if (locked()) {
      e.preventDefault();
      engine.mouse(e.button, true);
    } else if (e.button === 0) {
      dragging = true;
      dragDist = 0;
    } else {
      e.preventDefault();
      engine.mouse(e.button, true);
    }
  });
  window.addEventListener("mouseup", (e) => {
    const wasDrag = dragging;
    dragging = false;
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
    if (dragging) dragDist += Math.abs(e.movementX) + Math.abs(e.movementY);
    if (locked() || dragging) engine.look(e.movementX, e.movementY);
  });

  const keys = new Set(["Space", "Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);
  document.addEventListener("keydown", (e) => {
    // Esc med panelet oppe lukker panelet og tar deg rett tilbake i spillet,
    // uten å be om pekerlås (Esc gir ingen brukerhandling å låse på, og et
    // avslått forsøk åpnet pausemenyen). Klikk i bildet låser igjen; neste
    // Esc åpner menyen.
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
    if (e.code === "KeyG" && !e.repeat && (playing || !gateEl.hidden)) {
      setGraphics(!reducedGfx);
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
  $("graphics").addEventListener("click", () => setGraphics(!reducedGfx));
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

// Eksempelet fra dokumentasjonen, levende, i en flate slik Fristil-docs
// viser den (Preview.astro): i en shadow root med bare Fristils CSS, så
// ingenting fra spillets stilark treffer komponentene.
const FLATE = `
  :host { display: block; }
  .flate {
    display: flex;
    gap: var(--fs-spacing-3);
    padding: var(--fs-spacing-6);
    max-width: var(--fs-preview-max-width, none);
    background: var(--fs-color-neutral-canvas);
    color: var(--fs-color-neutral-text-strong);
    border: 1px solid var(--fs-color-neutral-border-subtle);
    border-radius: var(--fs-spacing-1);
    font-family: inherit;
    font-size: var(--fs-font-size-m);
    line-height: 1.6;
  }
  .flate[data-fast-inni] { contain: layout; }
  .flate[data-layout="rad"] { flex-wrap: wrap; align-items: center; }
  .flate[data-layout="stabel"] { flex-direction: column; align-items: stretch; }
  /* Docs-flaten gir <hr> nettleserens margin-inline: auto, og da blir
     skillelinja null bred i stabelen. I boka skal den synes. */
  .flate > .fs-divider { margin-inline: 0; }
`;

// Fristils stilark hentes som tekst: kopierte regler (cssRules) mister
// `background: var(...)` når lengre ned i regelen setter enkeltdelene.
const demoSheets = [new CSSStyleSheet(), new CSSStyleSheet()];
demoSheets[1].replaceSync(FLATE);
fetch("fristil.css")
  .then((r) => r.text())
  .then((css) => demoSheets[0].replace(css))
  .catch((e) => console.warn("fristil.css til demoen:", e));

// Det skriptet ved siden av demoen i docs gjør (ToastDemo.astro osv.).
const DEMO_ACTIONS = {
  toast: (root) =>
    root.querySelector(".fs-button")?.addEventListener("click", () =>
      root.querySelector("fs-toast")?.show("Søknaden er lagret", { color: "success" })),
  "connection-status": (root) => {
    const status = root.querySelector("fs-connection-status");
    root.getElementById("demo-samband-av")?.addEventListener("click", () => status?.reportFailure());
    root.getElementById("demo-samband-pa")?.addEventListener("click", () => status?.reportSuccess());
  },
  dialog: (root) =>
    root.querySelector("[data-apne]")?.addEventListener("click", () =>
      root.querySelector("fs-dialog")?.setAttribute("open", "")),
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
  root.adoptedStyleSheets = demoSheets;
  const tpl = document.createElement("template");
  tpl.innerHTML = demo.html;
  // Lenker i eksemplene skal ikke føre noen steder.
  for (const a of tpl.content.querySelectorAll("a[href]")) a.setAttribute("href", "#");
  for (const s of tpl.content.querySelectorAll("script")) s.remove();
  const flate = document.createElement("div");
  flate.className = "flate";
  flate.dataset.layout = demo.layout || "rad";
  if (demo.fixed) flate.dataset.fastInni = "";
  if (demo.maxWidth) flate.style.setProperty("--fs-preview-max-width", demo.maxWidth);
  flate.append(tpl.content);
  const style = document.createElement("style");
  style.textContent = demo.css ?? "";
  root.replaceChildren(style, flate);
  DEMO_ACTIONS[d.name]?.(root);
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
