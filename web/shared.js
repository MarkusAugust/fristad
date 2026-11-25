// Det modulene i nettlaget deler: elementene på siden, motoren, lyden, om
// du er inne i spillet, og små hjelpere. Bare den som eier en verdi, setter
// den; de andre leser den og bruker `set…`.

import { createSounds } from "./sounds.js";

export const $ = (id) => document.getElementById(id);
export const gateEl = $("gate");
export const hud = $("hud");
export const panel = $("panel");
export const status = $("status");
export const enterBtn = $("enter");
export const beginBtn = $("begin");
export const toast = document.querySelector("lt-toast");

export let canvas = $("world");
export let engine = null;
export let docs = {};
export const remember = (key, v) => {
  try {
    localStorage.setItem(key, String(v));
  } catch {
    /* ingen lagring */
  }
};
// Lageret og utstyret i hjørnet: utstyret som linjer, komponentene som
// chips du kan lese om i boka (det gir liv tilbake, én gang per tur).
export let lastHud = { hp: 100, stamina: 100, tokens: 0, keys: 0 };

export function esc(t) {
  const d = document.createElement("div");
  d.textContent = t ?? "";
  return d.innerHTML;
}

// Det sist valgte slaget huskes mellom gangene (en bekvemmelighet).
export function remembered(key, fallback) {
  try {
    return localStorage.getItem(key) || fallback;
  } catch {
    return fallback;
  }
}
// Lydene (web/sounds.js): opptak og syntetiske lyder, med retning og etterklang.
export const sfx = createSounds();

// Du er «inne» når startkortet er borte. Musa fanges med pekerlås når
// nettleseren gir oss den; ellers ser du deg rundt ved å dra med musa, så
// knappen virker også i innebygde nettlesere uten pekerlås.
export let playing = false;
export let dragging = false;

export function locked() {
  return document.pointerLockElement === canvas;
}

// Sier fra (én gang i blant) at et klikk i bildet fanger musa igjen.
let hintAt = 0;
export function clickHint() {
  const now = performance.now();
  if (now - hintAt < 8000) return;
  hintAt = now;
  toast?.show?.("Klikk i bildet for å se deg rundt med musa igjen. Uten fanget mus: dra for å se, klikk for å hugge.");
}

export const setCanvas = (c) => {
  canvas = c;
};
export const setEngine = (e) => {
  engine = e;
};
export const setDocs = (d) => {
  docs = d;
};
export const setLastHud = (h) => {
  lastHud = h;
};
export const setPlaying = (on) => {
  playing = on;
};
export const setDragging = (on) => {
  dragging = on;
};

// Tilbake i spillet: `enter` i main.js, som gir den hit med `onEnter`, så
// menyene kan lukke seg uten å kjenne hovedmodulen.
export let enter = () => {};
export const onEnter = (f) => {
  enter = f;
};
