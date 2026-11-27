// Det modulene i nivåredigeringen deler: elementene, motoren, byggesettet
// (`palette`), tilstanden til kartet (`st`), lagringen i nettleseren og de
// små hjelperne. Bare eieren setter en verdi; de andre bruker `set…`.

import { CELL, draw, makeView } from "./nivaa-plan.js";

export const $ = (id) => document.getElementById(id);
export const plan = $("plan");
export let canvas = $("world");
export const params = new URLSearchParams(location.search);
export const DRAFT_PREFIX = "fristad.nivaa.utkast:";
export const LAST = "fristad.nivaa.siste";
// Malene eieren har lagret i denne nettleseren (navn → tekst), og de som
// ikke er lagret på grenen ennå.
export const TEMPLATES = "fristad.nivaa.maler";
// Eierens egne moduler (Ctrl+Shift+G): id → { path, text, pending }.
export const MODULES = "fristad.nivaa.moduler";
export const MARKS = "fristad.nivaa.kamera:";
// Blob-sha-en til fila redigeringen begynte fra (repoet, grenen eller sist
// lagret), per sti: konfliktsjekken før lagring.
export const BASE = "fristad.nivaa.grunnlag:";
export let engine = null;
export let palette = null;
export const st = {
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
export let playing = false;
export let drag = null;
// Paletten viser temaet nivået har (settes når nivået er åpnet).
export let paletteFilter = () => {};
export const THEME_NAMES = { grav: "Gravkjelleren", kloakk: "Kloakken", gruve: "Gruva", borg: "Borgen", kjerne: "Kjernen" };

export const store = {
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

export function toast(text, ms = 3500) {
  const t = $("toast");
  t.textContent = text;
  t.hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => (t.hidden = true), ms);
}

export function setStatus(text) {
  $("status").textContent = text;
}

export function redraw() {
  draw(plan, st);
}

export function slug(name) {
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

export function chip(text, fn, key) {
  const b = document.createElement("button");
  b.className = "chip";
  b.type = "button";
  b.textContent = text;
  b.dataset.key = key;
  b.addEventListener("click", fn);
  return b;
}

export function worldAt(e) {
  const r = plan.getBoundingClientRect();
  return st.view.toWorld(e.clientX - r.left, e.clientY - r.top);
}

export function cellAt(x, z) {
  return [Math.round(x / CELL), st.floor, Math.round(z / CELL)];
}

// Alt holdt nede: ingen magnet (modulen står der pekeren er).
export let altDown = false;

export function snap(v) {
  return Math.round(v * 2) / 2;
}

export function thingY(id) {
  return st.map.things.find((t) => t.id === id)?.pos[1] ?? st.floor;
}

export function typing() {
  const a = document.activeElement;
  return a && (a.tagName === "INPUT" || a.tagName === "TEXTAREA" || a.tagName === "SELECT");
}

export function esc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

export function field(label, html) {
  return `<div class="field"><span class="lt-label">${label}</span>${html}</div>`;
}

export const setEngine = (v) => {
  engine = v;
};
export const setCanvas = (v) => {
  canvas = v;
};
export const setPalette = (v) => {
  palette = v;
};
export const setPlaying = (v) => {
  playing = v;
};
export const setDrag = (v) => {
  drag = v;
};
export const setAltDown = (v) => {
  altDown = v;
};
export const setPaletteFilter = (v) => {
  paletteFilter = v;
};
