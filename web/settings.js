// Innstillingene (O) og grafikkdialogen: musefølsomheten, lysstyrken,
// tekststørrelsen, oppløsningen og hver grafikkbryter for seg.

import { currentGraphics, custom, QUALITIES, quality, QUALITY_INFO, RES_NAMES, RES_STEPS, resolution, setCustom, setQuality, setResolution } from "./graphics.js";
import { canvas, engine, enter, enterBtn, gateEl, locked, playing, remember, setDragging, setPlaying, sfx } from "./shared.js";

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
export let mouseSens = readNum("fristad.mouse", 0.2, 3, 1);
export let gamma = readNum("fristad.gamma", 0.6, 2, 1);
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

export let settingsOpen = null;
export function openSettings() {
  if (settingsOpen) return;
  const wasPlaying = playing;
  setPlaying(false);
  setDragging(false);
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
export let graphicsOpen = null;
export function openGraphics(resume = playing) {
  if (graphicsOpen) return;
  const cur = currentGraphics();
  // Før motoren er lastet finnes ikke bryterne ennå: da de vanlige innstillingene.
  if (!cur) {
    openSettings();
    return;
  }
  const wasPlaying = resume;
  setPlaying(false);
  setDragging(false);
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
