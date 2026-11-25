// Grafikkvalget: forhåndsvalgene og de egendefinerte bryterne, oppløsningen
// bildet tegnes i, og det som sendes til motoren.

import { $, canvas, engine, playing, toast } from "./shared.js";

// Grafikken: forhåndsvalgene enkel (de svakeste maskinene og WebGL2: uten
// kantutjevning, solskygge og glød, høyst 900p), standard (myke skygger,
// solstråler, SMAA, opptil 1,5 piksler per CSS-piksel, høyst 1080p) og
// avansert (lengre sikt, skygge langt unna, skygge i krokene, alle lys,
// opptil 1440p) setter bryterne i `ports::Quality`; hver bryter kan endres
// for seg i grafikkdialogen (`openGraphics`). Valget huskes, og grafikken
// byttes aldri av seg selv.
export const QUALITIES = ["enkel", "standard", "avansert"];
export const QUALITY_INFO = {
  enkel: { cap: 900, dense: 1, note: "uten solskygge, glød og kantutjevning; for svake maskiner og WebGL2." },
  standard: { cap: 1080, dense: 1.5, note: "myke skygger, solstråler og SMAA, kortere sikt. Laget for en vanlig maskin." },
  avansert: { cap: 1440, dense: 1.5, note: "lengre sikt, skygge langt unna og i krokene, alle lys, høyere oppløsning." },
};
export let quality = null;
// Har spilleren endret en bryter i grafikkdialogen, står bryterne her
// (`fristad.graphics.custom`, med `scale` for oppløsningsskalaen), og
// forhåndsvalget over er bare utgangspunktet og taket for oppløsningen.
export let custom = null;
try {
  const q = localStorage.getItem("fristad.graphics");
  if (QUALITIES.includes(q)) quality = q;
  const c = JSON.parse(localStorage.getItem("fristad.graphics.custom") || "null");
  if (quality && c && typeof c === "object") custom = c;
} catch {
  /* ingen lagring */
}
// Oppløsningen tegnes i faste trinn (høyden i piksler; bredden følger
// vinduet), så et stort vindu på en svak maskin ikke gir en oppløsning
// den ikke kan håndtere. «Automatisk» tar det største trinnet som får
// plass på skjermen, men aldri over 1080p i standard grafikk og 1440p i
// avansert. Valget huskes som `fristad.resolution` (høyden eller «auto»).
export const RES_STEPS = [720, 900, 1080, 1152, 1440, 1620, 1800, 2160];
export const RES_NAMES = { 720: "720p", 900: "900p", 1080: "1080p (FHD)", 1152: "QWXGA", 1440: "1440p (QHD)", 1620: "1620p (3K)", 1800: "1800p (QHD+)", 2160: "2160p (4K)" };
export let resolution = "auto";
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
export function fit() {
  const h = Math.max(1, Math.round(renderHeight() * benchScale * (custom?.scale ?? 1)));
  const w = Math.max(1, Math.round((h * canvas.clientWidth) / Math.max(1, canvas.clientHeight)));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  engine?.resize(w, h);
}
export function setResolution(r, announce = true) {
  resolution = r;
  try {
    localStorage.setItem("fristad.resolution", String(r));
  } catch {
    /* ingen lagring */
  }
  fit();
  if (announce) toast?.show?.(r === "auto" ? "Oppløsning: automatisk." : `Oppløsning: ${canvas.width} × ${canvas.height}.`);
}

// Byttes grafikken: motoren, oppløsningen og knappene. Et forhåndsvalg
// setter alle bryterne (de egendefinerte glemmes).
export function setQuality(name, chosen = true, announce = true) {
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
export function currentGraphics() {
  return custom ?? presetOf(quality ?? "standard");
}
// Egendefinerte brytere fra dialogen. Svarer de til et forhåndsvalg, er
// det det forhåndsvalget igjen.
export function setCustom(values) {
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
export function setGraphics(reduced, chosen = true, announce = true) {
  setQuality(reduced ? "standard" : "avansert", chosen, announce);
}
// Går det tregt over tid, foreslås et lavere nivå én gang (ikke mens
// målingen går, det passer løkka på). Ingenting byttes uten at spilleren
// ber om det.
let slowSeconds = 0;
let suggested = false;
export function adapt(fps) {
  if (!playing || quality === "enkel" || suggested) return;
  slowSeconds = fps < 25 ? slowSeconds + 1 : 0;
  if (slowSeconds >= 8) {
    suggested = true;
    toast?.show?.("Spillet går tregt. Velg et lavere grafikknivå i Innstillinger (O) eller med G.");
  }
}

// Målingen (bench.js) setter bryterne og skalaen rett, uten å huske dem.
export const putCustom = (c) => {
  custom = c;
};
export const setBenchScale = (s) => {
  benchScale = s;
};
