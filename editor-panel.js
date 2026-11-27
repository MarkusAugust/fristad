// Panelet i redigeringen (`?edit`, web/editor.js): det som står i det, og
// hvordan katalogkortene, feltene for egenskapene og historikken tegnes.
// Tastene, musa og det som sendes til motoren, står i editor.js.

import { LOCAL } from "./github.js";

export function esc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

/// Panelet til høyre, lagt i dokumentet (skjult til redigeringen begynner).
export function makePanel() {
  const panel = document.createElement("aside");
  panel.id = "editor";
  panel.className = "editor";
  panel.hidden = true;
  panel.innerHTML = `
    <div class="editor-head">
      <h2 class="lt-heading" data-size="s">Redigering</h2>
      <span id="ed-state" class="lt-badge">Grønt</span>
    </div>
    <div class="editor-actions">
      <button id="ed-undo" class="lt-button" data-variant="secondary" type="button" title="Ctrl+Z">Angre</button>
      <button id="ed-redo" class="lt-button" data-variant="secondary" type="button" title="Ctrl+Shift+Z">Gjør om</button>
      <button id="ed-play" class="lt-button" data-variant="primary" type="button" title="P">Spill (P)</button>
      <button id="ed-save" class="lt-button" data-variant="secondary" type="button" disabled title="Lagre på grenen redigering (Ctrl+S)">Lagre</button>
      <label class="editor-snap lt-label" title="Fest til rutenettet (0,5 m) og vinkelen (15°). Ctrl holdt gjør det samme mens du drar.">
        <input id="ed-snap" type="checkbox"> Fest
      </label>
    </div>
    <p id="ed-saved" class="lt-help-text" role="status"></p>
    <section class="editor-section">
      <h3 class="lt-heading" data-size="xs">Valgt</h3>
      <p id="ed-selected" class="lt-paragraph" data-size="small">Ingenting. Klikk på noe i bildet.</p>
      <div id="ed-under-box" hidden>
        <p class="lt-help-text">Under pekeren, nærmest først. Klikk for å velge (<kbd>Shift</kbd> legger til), <kbd>Tab</kbd>/<kbd>Shift+Tab</kbd> blar,
        <kbd>Alt</kbd>+klikk i bildet tar det bakerste.</p>
        <ol id="ed-under" class="editor-under"></ol>
      </div>
      <div class="editor-grouping">
        <input id="ed-grp-name" class="lt-input" type="text" placeholder="Navn på gruppen" aria-label="Navn på gruppen">
        <button id="ed-grp-make" class="lt-button" data-variant="secondary" type="button" title="Ctrl+G">Lag gruppe</button>
        <button id="ed-grp-free" class="lt-button" data-variant="secondary" type="button" title="Ctrl+Shift+G" disabled>Løs opp</button>
      </div>
      <p id="ed-grp" class="lt-help-text" hidden></p>
      <p id="ed-route" class="lt-help-text" hidden></p>
      <div id="ed-wall" hidden>
        <label class="lt-label" for="ed-wall-h">Høyde på veggen (m)</label>
        <input id="ed-wall-h" class="lt-input" type="number" min="0.5" max="20" step="0.5" value="3">
      </div>
    </section>
    <section class="editor-section" id="ed-props-box" hidden>
      <h3 class="lt-heading" data-size="xs">Egenskaper</h3>
      <form id="ed-props" class="editor-props" autocomplete="off"></form>
    </section>
    <section class="editor-section">
      <h3 class="lt-heading" data-size="xs">Reglene</h3>
      <ul id="ed-breaches" class="editor-breaches"></ul>
    </section>
    <section class="editor-section editor-catalog">
      <h3 class="lt-heading" data-size="xs">Katalogen</h3>
      <div class="editor-filters">
        <input id="ed-search" class="lt-input" type="search" placeholder="Søk" aria-label="Søk i katalogen">
        <select id="ed-group" class="lt-select" aria-label="Kategori"><option value="">Alle kategorier</option></select>
      </div>
      <div class="lt-switch-row editor-odd">
        <input class="lt-switch" type="checkbox" role="switch" id="ed-odd">
        <label class="lt-label" for="ed-odd">Bare avvik</label>
        ${LOCAL ? '<a class="lt-link" href="/dom/" target="_blank" rel="noopener">Godkjenningssiden</a>' : ""}
      </div>
      <p id="ed-placing" class="lt-paragraph" data-size="small" hidden></p>
      <div id="ed-grid" class="editor-grid"></div>
    </section>
    <section class="editor-section">
      <h3 class="lt-heading" data-size="xs">Historikk</h3>
      <ol id="ed-history" class="editor-history"></ol>
    </section>
    <section class="editor-section">
      <h3 class="lt-heading" data-size="xs">Linje</h3>
      <p class="lt-paragraph" data-size="small"><kbd>L</kbd> begynner: klikk punkter på bakken,
      <kbd>Enter</kbd> gir en rad av modellen du har valgt i katalogen (gjerde), <kbd>V</kbd> en usynlig vegg, <kbd>Esc</kbd> avbryter.</p>
    </section>
    <section class="editor-section editor-keys">
      <p class="lt-paragraph" data-size="small">
        <strong>Musa:</strong> dra en ting for å flytte den langs bakken (den settes ned igjen der du slipper), <kbd>Alt</kbd>+dra heiser opp og ned,
        <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+dra dreier (flere valgt dreies samlet), <kbd>Shift</kbd>+klikk velger flere, <kbd>Shift</kbd>+dra velger alt innenfor rammen,
        <kbd>Alt</kbd>+klikk det bakerste.
        Dra en modell fra katalogen ut i bildet for å sette den ut (<kbd>R</kbd> dreier 90°, <kbd>Ctrl</kbd> fester). <kbd>Shift+1</kbd>–<kbd>9</kbd> husker kameraet,
        <kbd>1</kbd>–<kbd>9</kbd> går dit.
        <kbd>WASD</kbd> fly, <kbd>Mellomrom</kbd>/<kbd>C</kbd> opp og ned, <kbd>Shift</kbd> fort, høyre knapp ser rundt.
        Piltastene flytter det valgte langs bakken, <kbd>+</kbd>/<kbd>−</kbd> (eller <kbd>PgUp</kbd>/<kbd>PgDn</kbd>) opp og ned (<kbd>Alt</kbd> små steg, <kbd>Shift</kbd> store),
        <kbd>Ctrl+G</kbd> lager en gruppe av det valgte (velges, flyttes, dreies og kopieres samlet), <kbd>Ctrl+Shift+G</kbd> løser den opp.
        En valgt innbygger viser ruta: dra stolpene, klikk på streken for et nytt punkt, <kbd>Delete</kbd> eller <kbd>⌫</kbd> sletter punktet du tok.
        <kbd>T</kbd> bytter dag og natt, så lysene (de små korsene) synes; klikk et lys for å endre det.
        <kbd>Q</kbd>/<kbd>E</kbd> dreier, <kbd>R</kbd> en kvart runde, <kbd>Tab</kbd> velger det neste under pekeren (kirka, så gjerdet inni boksen dens),
        <kbd>Shift+Tab</kbd> det forrige, <kbd>G</kbd> setter ned på det som er under, <kbd>Ctrl+D</kbd> kopierer, <kbd>Delete</kbd> eller <kbd>⌫</kbd> sletter.
      </p>
    </section>`;
  document.body.append(panel);
  return panel;
}

/// Avviket på en modell, eller null: bare det som ikke er godkjent, får
/// merke (eieren 26. november). En modell som er ny eller endret etter
/// dommen, venter på dom.
export function deviation(m) {
  if (m.waiting) return { text: "Venter på dom", variant: "warning" };
  if (m.verdict === "underkjent") return { text: "Underkjent", variant: "danger" };
  if (m.verdict === "usikker") return { text: "Usikker", variant: "warning" };
  return null;
}

/// Kortet til en modell i katalogen (`active`: den som er valgt å sette ut).
export function catalogItem(m, active) {
  const d = deviation(m);
  return `<button type="button" class="editor-item${active ? " is-active" : ""}" data-model="${esc(m.name)}" title="${esc(m.why || m.name)}">
    <img src="catalog/${esc(m.name)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
    <span class="editor-item-name">${esc(m.name)}</span>
    ${d ? `<span class="lt-badge" data-variant="${d.variant}">${d.text}</span>` : ""}
  </button>`;
}

/// Feltet til en egenskap: tekst, tall, avkryssing, farge eller en liste å
/// velge fra.
export function propField(p) {
  const name = esc(p.key);
  const value = esc(p.value);
  if (p.key === "kind") return `<input class="lt-input" name="${name}" value="${value}" data-was="${value}" readonly>`;
  if (p.key === "text" || p.key === "lines") return `<textarea class="lt-input" name="${name}" rows="3" data-was="${value}">${value}</textarea>`;
  if ((p.key === "flicker" && (p.value === "ja" || p.value === "nei")) || p.key === "always")
    return `<input type="checkbox" name="${name}" ${p.value === "ja" ? "checked" : ""} data-was="${value}">`;
  if (p.key === "tint" || p.key === "color") {
    const hex = /^#[0-9a-f]{6}$/i.test(p.value) ? value : "#ffffff";
    const pick = `<input type="color" data-pick="${name}" value="${hex}" aria-label="Velg farge">`;
    return `<span class="editor-color">${pick}<input class="lt-input" name="${name}" value="${value}" data-was="${value}" placeholder="#rrggbb"></span>`;
  }
  // Valgene (fiendeslagene): en liste å velge fra, fri tekst går også.
  if (p.options?.length) {
    const options = p.options.map((o) => `<option value="${esc(o)}"></option>`).join("");
    return `<input class="lt-input" name="${name}" value="${value}" data-was="${value}" list="ed-opt-${name}"><datalist id="ed-opt-${name}">${options}</datalist>`;
  }
  const num = ["x", "y", "z", "yaw", "range", "intensity", "flicker", "width", "height", "n"].includes(p.key) || p.key.startsWith("route.");
  return `<input class="lt-input" name="${name}" value="${value}" data-was="${value}" ${num ? 'inputmode="decimal"' : ""}>`;
}

/// Historikken som punkter: «Start» og hvert steg; det du står på (`at`) er
/// markert, og det som kan gjøres om, er dempet.
export function historyItems(steps, at) {
  const items = [`<li><button type="button" class="editor-step${at === 0 ? " is-active" : ""}" data-n="0">Start</button></li>`];
  steps.forEach((s, k) => {
    const n = k + 1;
    items.push(`<li><button type="button" class="editor-step${n === at ? " is-active" : ""}${n > at ? " is-undone" : ""}" data-n="${n}">${esc(s)}</button></li>`);
  });
  return items.slice(-40).join("");
}
