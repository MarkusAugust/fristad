// Boka: dokumentasjonen fra LO-TEK i panelet til høyre, med eksempelet
// levende i en shadow root.

import { closeOverlay } from "./menus.js";
import { $, clickHint, docs, engine, enter, enterBtn, gateEl, hud, locked, panel, setDragging, setPlaying, sfx } from "./shared.js";

export let panelOpen = false;
let current = { component: "" };

// Boka: dokumentasjonen for en komponent du har vunnet.
export function readDoc(component) {
  sfx.play("page");
  engine.read_doc(component);
  if (docs[component]) {
    current = { component };
    closeOverlay(false);
    openPanel();
  }
}

export function openPanel() {
  const d = docs[current.component] ?? {};
  panelOpen = true;
  setPlaying(false);
  setDragging(false);
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

export function closePanel(relock) {
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

export const setPanelOpen = (on) => {
  panelOpen = on;
};
