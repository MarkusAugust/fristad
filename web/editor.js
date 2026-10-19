// Redigeringsmodusen i nettleseren (docs/redigering.md, steg 4), bare for
// eieren: `?edit` i adressen. Motoren gjør jobben (velger, flytter, sjekker
// reglene, angrer); her er panelet med katalogen, det valgte og reglene, og
// tastene og musa.
//
// Musa er fri i redigeringen: venstre klikk velger det under pekeren, og
// dra flytter det langs bakken; med Alt holdt inne heiser draget det valgte
// opp og ned, med Ctrl (Cmd på Mac) dreier det (flere valgt dreies samlet).
// Pilene og ringen rundt det valgte virker fortsatt. Høyre knapp holdt nede
// ser rundt. WASD, mellomrom og C flyr.
//
// Dra en modell fra katalogen ut i bildet: et fantom følger pekeren (grønn
// boks når alt er i orden, rød når en regel brytes), R dreier det 90°, og
// slipp setter det ned på det som er under. «Fest» (eller Ctrl holdt) setter
// på rutenettet (0,5 m) og vinkelen (15°). Det som ligger under pekeren
// står som en liste å klikke i (Tab og Shift+Tab blar, Alt+klikk tar det
// bakerste), Shift+dra i bildet velger alt innenfor rammen, panelet viser
// egenskapene til det valgte å skrive i, og historikken er en liste å
// klikke seg tilbake i. Ctrl+1–9 husker hvor kameraet står, 1–9 går dit.
//
// Det du gjør, huskes som et utkast i nettleseren (`fristad.edit-draft`)
// ved hver endring og hvert 30. sekund, og Lagre (Ctrl+S) legger det på
// grenen `redigering`, én commit per økt; uten nøkkel lastes filene ned som
// zip. Neste gang fortsetter du fra utkastet, eller fra det du lagret sist.

import { branchFiles, commit, setToken, token, SAVE_BRANCH } from "./github.js";

const DRAFT = "fristad.edit-draft";
const MARKS = "fristad.edit-marks";
const AUTOSAVE_MS = 30000;
// Så langt (px) musa må gå før et trykk blir et drag (katalogen, rammen).
const SLOP = 5;

// En zip uten komprimering (bare lagret), med CRC-32 per fil: nok til å
// laste ned filene når det ikke er noen nøkkel til GitHub.
const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
function crc32(bytes) {
  let c = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) c = CRC[(c ^ bytes[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
export function makeZip(files) {
  const enc = new TextEncoder();
  const parts = [];
  const central = [];
  let offset = 0;
  const now = new Date();
  const time = (now.getHours() << 11) | (now.getMinutes() << 5) | (now.getSeconds() >> 1);
  const date = ((now.getFullYear() - 1980) << 9) | ((now.getMonth() + 1) << 5) | now.getDate();
  for (const f of files) {
    const name = enc.encode(f.path);
    const data = enc.encode(f.text);
    const crc = crc32(data);
    const local = new DataView(new ArrayBuffer(30));
    local.setUint32(0, 0x04034b50, true);
    local.setUint16(4, 20, true);
    local.setUint16(6, 0x0800, true); // navnene er UTF-8
    local.setUint16(10, time, true);
    local.setUint16(12, date, true);
    local.setUint32(14, crc, true);
    local.setUint32(18, data.length, true);
    local.setUint32(22, data.length, true);
    local.setUint16(26, name.length, true);
    parts.push(new Uint8Array(local.buffer), name, data);
    const cd = new DataView(new ArrayBuffer(46));
    cd.setUint32(0, 0x02014b50, true);
    cd.setUint16(4, 20, true);
    cd.setUint16(6, 20, true);
    cd.setUint16(8, 0x0800, true);
    cd.setUint16(12, time, true);
    cd.setUint16(14, date, true);
    cd.setUint32(16, crc, true);
    cd.setUint32(20, data.length, true);
    cd.setUint32(24, data.length, true);
    cd.setUint16(28, name.length, true);
    cd.setUint32(42, offset, true);
    central.push(new Uint8Array(cd.buffer), name);
    offset += 30 + name.length + data.length;
  }
  const size = central.reduce((n, b) => n + b.length, 0);
  const end = new DataView(new ArrayBuffer(22));
  end.setUint32(0, 0x06054b50, true);
  end.setUint16(8, files.length, true);
  end.setUint16(10, files.length, true);
  end.setUint32(12, size, true);
  end.setUint32(16, offset, true);
  return new Blob([...parts, ...central, new Uint8Array(end.buffer)], { type: "application/zip" });
}

const STEP = 0.1;
const TURN = (15 * Math.PI) / 180;

const VERDICTS = { godkjent: "Godkjent", underkjent: "Underkjent", usikker: "Usikker" };

function esc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

export function createEditor({ engine, canvas, toast, onPlay, onLock, onEdit }) {
  // Redigerer nå (`active`), modellen som skal settes ut, og linjeverktøyet.
  let active = false;
  let placing = null;
  let lining = false;
  let looking = false;
  let pressed = false;
  let view = null;
  // Fest til rutenett og vinkel: bryteren i panelet, eller Ctrl holdt.
  let snapToggle = false;
  let ctrlHeld = false;
  let snapSent = null;
  // Dra fra katalogen: modellen og der trykket begynte.
  let dragModel = null;
  let ghostShown = false;
  let ghostFrame = 0;
  let ghostAt = null;
  // Shift+trykk i bildet: et klikk (legg til eller ta bort) eller en ramme.
  let boxFrom = null;
  let boxing = false;
  const catalog = JSON.parse(engine.catalog());

  const panel = document.createElement("aside");
  panel.id = "editor";
  panel.className = "editor";
  panel.hidden = true;
  panel.innerHTML = `
    <div class="editor-head">
      <h2 class="fs-heading" data-size="s">Redigering</h2>
      <span id="ed-state" class="fs-badge">Grønt</span>
    </div>
    <div class="editor-actions">
      <button id="ed-undo" class="fs-button" data-variant="secondary" type="button" title="Ctrl+Z">Angre</button>
      <button id="ed-redo" class="fs-button" data-variant="secondary" type="button" title="Ctrl+Shift+Z">Gjør om</button>
      <button id="ed-play" class="fs-button" data-variant="primary" type="button" title="P">Spill (P)</button>
      <button id="ed-save" class="fs-button" data-variant="secondary" type="button" disabled title="Lagre på grenen redigering (Ctrl+S)">Lagre</button>
      <label class="editor-snap fs-label" title="Fest til rutenettet (0,5 m) og vinkelen (15°). Ctrl holdt gjør det samme mens du drar.">
        <input id="ed-snap" type="checkbox"> Fest
      </label>
    </div>
    <p id="ed-saved" class="fs-help-text" role="status"></p>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Valgt</h3>
      <p id="ed-selected" class="fs-paragraph" data-size="small">Ingenting. Klikk på noe i bildet.</p>
      <div id="ed-under-box" hidden>
        <p class="fs-help-text">Under pekeren, nærmest først. Klikk for å velge (<kbd>Shift</kbd> legger til), <kbd>Tab</kbd>/<kbd>Shift+Tab</kbd> blar, <kbd>Alt</kbd>+klikk i bildet tar det bakerste.</p>
        <ol id="ed-under" class="editor-under"></ol>
      </div>
      <div class="editor-grouping">
        <input id="ed-grp-name" class="fs-input" type="text" placeholder="Navn på gruppen" aria-label="Navn på gruppen">
        <button id="ed-grp-make" class="fs-button" data-variant="secondary" type="button" title="Ctrl+G">Lag gruppe</button>
        <button id="ed-grp-free" class="fs-button" data-variant="secondary" type="button" title="Ctrl+Shift+G" disabled>Løs opp</button>
      </div>
      <p id="ed-grp" class="fs-help-text" hidden></p>
      <p id="ed-route" class="fs-help-text" hidden></p>
      <div id="ed-wall" hidden>
        <label class="fs-label" for="ed-wall-h">Høyde på veggen (m)</label>
        <input id="ed-wall-h" class="fs-input" type="number" min="0.5" max="20" step="0.5" value="3">
      </div>
    </section>
    <section class="editor-section" id="ed-props-box" hidden>
      <h3 class="fs-heading" data-size="xs">Egenskaper</h3>
      <form id="ed-props" class="editor-props" autocomplete="off"></form>
    </section>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Reglene</h3>
      <ul id="ed-breaches" class="editor-breaches"></ul>
    </section>
    <section class="editor-section editor-catalog">
      <h3 class="fs-heading" data-size="xs">Katalogen</h3>
      <div class="editor-filters">
        <input id="ed-search" class="fs-input" type="search" placeholder="Søk" aria-label="Søk i katalogen">
        <select id="ed-verdict" class="fs-select" aria-label="Dom">
          <option value="">Alle</option>
          <option value="godkjent">Godkjent</option>
          <option value="underkjent">Underkjent</option>
          <option value="usikker">Usikker</option>
          <option value="none">Ikke vurdert</option>
        </select>
        <select id="ed-group" class="fs-select" aria-label="Kategori"><option value="">Alle kategorier</option></select>
      </div>
      <p id="ed-placing" class="fs-paragraph" data-size="small" hidden></p>
      <div id="ed-grid" class="editor-grid"></div>
    </section>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Historikk</h3>
      <ol id="ed-history" class="editor-history"></ol>
    </section>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Linje</h3>
      <p class="fs-paragraph" data-size="small"><kbd>L</kbd> begynner: klikk punkter på bakken, <kbd>Enter</kbd> gir en rad av modellen du har valgt i katalogen (gjerde), <kbd>V</kbd> en usynlig vegg, <kbd>Esc</kbd> avbryter.</p>
    </section>
    <section class="editor-section editor-keys">
      <p class="fs-paragraph" data-size="small">
        <strong>Musa:</strong> dra en ting for å flytte den langs bakken (den settes ned igjen der du slipper), <kbd>Alt</kbd>+dra heiser opp og ned, <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+dra dreier (flere valgt dreies samlet), <kbd>Shift</kbd>+klikk velger flere, <kbd>Shift</kbd>+dra velger alt innenfor rammen, <kbd>Alt</kbd>+klikk det bakerste.
        Dra en modell fra katalogen ut i bildet for å sette den ut (<kbd>R</kbd> dreier 90°, <kbd>Ctrl</kbd> fester). <kbd>Ctrl+1</kbd>–<kbd>9</kbd> husker kameraet, <kbd>1</kbd>–<kbd>9</kbd> går dit.
        <kbd>WASD</kbd> fly, <kbd>Mellomrom</kbd>/<kbd>C</kbd> opp og ned, <kbd>Shift</kbd> fort, høyre knapp ser rundt.
        Piltastene flytter det valgte langs bakken, <kbd>PgUp</kbd>/<kbd>PgDn</kbd> opp og ned (<kbd>Alt</kbd> små steg, <kbd>Shift</kbd> store),
        <kbd>Ctrl+G</kbd> lager en gruppe av det valgte (velges, flyttes, dreies og kopieres samlet), <kbd>Ctrl+Shift+G</kbd> løser den opp.
        En valgt innbygger viser ruta: dra stolpene, klikk på streken for et nytt punkt, <kbd>Delete</kbd> sletter punktet du tok.
        <kbd>T</kbd> bytter dag og natt, så lysene (de små korsene) synes; klikk et lys for å endre det.
        <kbd>Q</kbd>/<kbd>E</kbd> dreier, <kbd>R</kbd> en kvart runde, <kbd>Tab</kbd> velger det neste under pekeren (kirka, så gjerdet inni boksen dens), <kbd>Shift+Tab</kbd> det forrige, <kbd>G</kbd> setter ned på det som er under, <kbd>Ctrl+D</kbd> kopierer, <kbd>Delete</kbd> sletter.
      </p>
    </section>`;
  document.body.append(panel);
  // Rammen når Shift+dra velger.
  const box = document.createElement("div");
  box.className = "editor-box";
  box.hidden = true;
  document.body.append(box);
  // I spillet: et lite merke om at P tar deg tilbake i redigeringen.
  const back = document.createElement("p");
  back.className = "editor-back fs-tag";
  back.hidden = true;
  back.innerHTML = "<kbd>P</kbd> tilbake til redigeringen";
  document.body.append(back);
  const $ = (id) => panel.querySelector(`#${id}`);

  // Gruppene i katalogen (mappene modellene ligger i).
  // Kategoriene i fast rekkefølge (husene først).
  const ORDER = ["Hus", "Gjenstander", "Møbler", "Gjerder", "Mur og porter", "Krypten"];
  const rank = (g) => (ORDER.indexOf(g) + 1 || 99);
  catalog.sort((a, b) => rank(a.group) - rank(b.group) || a.name.localeCompare(b.name));
  for (const g of [...new Set(catalog.map((m) => m.group))].sort((a, b) => rank(a) - rank(b))) {
    const o = document.createElement("option");
    o.value = g;
    o.textContent = g;
    $("ed-group").append(o);
  }

  function renderCatalog() {
    const q = $("ed-search").value.trim().toLowerCase();
    const v = $("ed-verdict").value;
    const g = $("ed-group").value;
    const shown = catalog.filter((m) => (!q || m.name.includes(q)) && (!g || m.group === g) && (!v || (v === "none" ? !m.verdict : m.verdict === v)));
    $("ed-grid").innerHTML = shown
      .map(
        (m) => `<button type="button" class="editor-item${placing === m.name ? " is-active" : ""}" data-model="${esc(m.name)}" title="${esc(m.why || m.name)}">
          <img src="catalog/${esc(m.name)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
          <span class="editor-item-name">${esc(m.name)}</span>
          <span class="fs-badge" data-variant="${m.verdict === "godkjent" ? "success" : m.verdict === "underkjent" ? "danger" : "warning"}">${esc(VERDICTS[m.verdict] ?? "Ikke vurdert")}</span>
        </button>`,
      )
      .join("");
  }
  for (const id of ["ed-search", "ed-verdict", "ed-group"]) $(id).addEventListener("input", renderCatalog);
  // Et klikk velger modellen å sette ut; et drag tar den rett ut i bildet.
  $("ed-grid").addEventListener("mousedown", (e) => {
    const b = e.target.closest("[data-model]");
    if (!b || e.button !== 0) return;
    e.preventDefault();
    dragModel = { name: b.dataset.model, x: e.clientX, y: e.clientY, moved: false };
  });

  function setPlacing(model) {
    placing = model;
    const p = $("ed-placing");
    p.hidden = !model;
    p.textContent = model ? `Klikk i bildet for å sette ut ${model} (Shift: flere, R dreier). Esc avbryter.` : "";
    if (!model) hideGhost();
    renderCatalog();
  }

  function send(cmd) {
    return engine.edit(JSON.stringify(cmd));
  }

  // Fantomet følger pekeren (høyst én gang per bilde).
  function ghost(model, p) {
    ghostAt = { model, ...p };
    if (ghostFrame) return;
    ghostFrame = requestAnimationFrame(() => {
      ghostFrame = 0;
      if (!ghostAt) return;
      send({ cmd: "ghost", model: ghostAt.model, x: ghostAt.x, y: ghostAt.y });
      ghostShown = true;
    });
  }
  function hideGhost() {
    ghostAt = null;
    if (ghostShown) send({ cmd: "ghost" });
    ghostShown = false;
  }
  const ghostModel = () => (dragModel?.moved ? dragModel.name : placing);

  // Festet: bryteren, eller Ctrl holdt (motoren får det bare når det endres).
  function sendSnap() {
    const on = snapToggle !== ctrlHeld;
    if (on === snapSent) return;
    snapSent = on;
    send({ cmd: "snap", on });
    if (ghostAt) ghost(ghostAt.model, ghostAt);
  }
  $("ed-snap").addEventListener("change", (e) => {
    snapToggle = e.target.checked;
    sendSnap();
  });

  // Kamerabokmerkene (Ctrl+1–9 husker, 1–9 går dit), i nettleseren.
  function readMarks() {
    try {
      return JSON.parse(localStorage.getItem(MARKS) || "{}");
    } catch {
      return {};
    }
  }
  function bookmark(n, store) {
    const marks = readMarks();
    if (store) {
      marks[n] = engine.view();
      try {
        localStorage.setItem(MARKS, JSON.stringify(marks));
      } catch {
        /* uten lagring huskes det ikke */
      }
      toast?.show?.(`Kameraet er husket på ${n}.`);
      return;
    }
    const v = marks[n];
    if (!v) {
      toast?.show?.(`Ingenting på ${n} ennå. Ctrl+${n} husker der du står.`);
      return;
    }
    engine.place(v[0], v[1], v[2], (v[3] * 180) / Math.PI, (v[4] * 180) / Math.PI);
  }

  // Lagre: filene på grenen `redigering` (web/github.js), én commit per
  // økt. Første gang limer du inn en nøkkel; den huskes bare i denne
  // nettleseren, og så lagrer Lagre og Ctrl+S med en gang, uten spørsmål.
  const dialog = document.createElement("dialog");
  dialog.className = "fs-dialog editor-save";
  dialog.innerHTML = `
    <h2 class="fs-dialog__title fs-heading" data-size="s">Lagre endringene</h2>
    <div class="fs-dialog__body">
      <p class="fs-paragraph" data-size="small">Endringene legges på grenen <code>${SAVE_BRANCH}</code> i fristad-repoet, én commit for hele økta (lagrer du igjen, oppdateres den). Neste gang du redigerer, fortsetter du derfra. Claude kjører testene og slår dem inn i spillet.</p>
      <label class="fs-label" for="ed-msg">Hva du har endret (kan stå tomt)</label>
      <input id="ed-msg" class="fs-input" type="text">
      <div id="ed-token-row">
        <label class="fs-label" for="ed-token">GitHub-nøkkel</label>
        <input id="ed-token" class="fs-input" type="password" autocomplete="off" placeholder="github_pat_…">
        <p class="fs-help-text">En finkornet nøkkel (github.com → Settings → Developer settings → Fine-grained tokens) med tilgang bare til fristad-repoet og «Contents: Read and write». Den huskes bare i denne nettleseren.</p>
      </div>
      <p id="ed-save-status" class="fs-paragraph" data-size="small" role="status"></p>
    </div>
    <div class="fs-dialog__footer">
      <button type="button" class="fs-button" data-variant="ghost" data-act="forget">Glem nøkkelen</button>
      <button type="button" class="fs-button" data-variant="secondary" data-act="close">Avbryt</button>
      <button type="button" class="fs-button" data-variant="secondary" data-act="zip">Last ned zip</button>
      <button type="button" class="fs-button" data-variant="primary" data-act="save">Lagre</button>
    </div>`;
  document.body.append(dialog);
  const d$ = (sel) => dialog.querySelector(sel);
  const hhmm = (t = Date.now()) => new Date(t).toLocaleTimeString("nb-NO", { hour: "2-digit", minute: "2-digit" });
  const setSaved = (text) => ($("ed-saved").textContent = text);
  let message = "";
  let saving = false;
  let lastSaved = "";

  // Lagre det som er redigert. Gir om det gikk.
  async function save() {
    if (saving) return false;
    const out = JSON.parse(engine.save_files());
    if (out.error) {
      setSaved(out.error);
      return false;
    }
    saving = true;
    setSaved("Lagrer …");
    try {
      const r = await commit(out.files, message || `Redigering ${new Date().toLocaleString("nb-NO")}`);
      lastSaved = JSON.stringify(out.files);
      storeDraft();
      const text = r.unchanged ? "Ingenting nytt å lagre." : `Lagret kl. ${hhmm()} på grenen ${SAVE_BRANCH}.`;
      setSaved(text);
      toast?.show?.(text);
      return true;
    } catch (e) {
      setSaved(e.message);
      toast?.show?.(e.message);
      return false;
    } finally {
      saving = false;
    }
  }

  function openSave(quick = false) {
    if (!view?.dirty) return;
    if (!view.clean) {
      toast?.show?.(`${view.breaches.length} regler er brutt. Rett dem før du lagrer.`);
      return;
    }
    if (token()) {
      save();
      return;
    }
    if (quick) {
      download();
      return;
    }
    d$("#ed-token-row").hidden = false;
    d$("[data-act=forget]").hidden = true;
    d$("#ed-msg").value = message;
    d$("#ed-save-status").textContent = "";
    dialog.showModal();
  }

  d$("[data-act=close]").addEventListener("click", () => dialog.close());
  d$("[data-act=zip]").addEventListener("click", () => {
    if (download()) dialog.close();
  });
  d$("[data-act=forget]").addEventListener("click", () => {
    setToken("");
    d$("#ed-token-row").hidden = false;
    d$("[data-act=forget]").hidden = true;
  });
  d$("[data-act=save]").addEventListener("click", async () => {
    const t = d$("#ed-token").value.trim();
    if (t) setToken(t);
    if (!token()) {
      d$("#ed-save-status").textContent = "Lim inn nøkkelen først.";
      return;
    }
    message = d$("#ed-msg").value.trim();
    const btn = d$("[data-act=save]");
    btn.disabled = true;
    d$("#ed-save-status").textContent = "Lagrer …";
    const ok = await save();
    btn.disabled = false;
    d$("#ed-token").value = "";
    if (ok) dialog.close();
    else d$("#ed-save-status").textContent = $("ed-saved").textContent;
  });
  $("ed-save").addEventListener("click", () => openSave(false));

  // Utkastet: det som er redigert, i nettleseren, så ingenting går tapt om
  // fanen lukkes før du lagrer. Det hører til byen slik den var da du begynte
  // (`base`); er spillet bygd på nytt siden, legges det bort.
  let shipped = "";
  let draftTimer = 0;
  const hash = (text) => {
    let h = 5381;
    for (let i = 0; i < text.length; i++) h = (h * 33 + text.charCodeAt(i)) | 0;
    return String(h >>> 0);
  };
  function readDraft() {
    try {
      return JSON.parse(localStorage.getItem(DRAFT) || "null");
    } catch {
      return null;
    }
  }
  function storeDraft() {
    try {
      if (!view?.dirty) {
        localStorage.removeItem(DRAFT);
        return;
      }
      const out = JSON.parse(engine.save_files());
      if (!out.files) return;
      const files = JSON.stringify(out.files);
      localStorage.setItem(DRAFT, JSON.stringify({ at: Date.now(), base: shipped, saved: files === lastSaved, files: out.files }));
    } catch {
      /* uten lagring i nettleseren er det ikke noe utkast */
    }
  }
  function draftLater() {
    clearTimeout(draftTimer);
    draftTimer = setTimeout(storeDraft, 400);
  }
  // Og hvert 30. sekund mens du redigerer, uansett.
  setInterval(() => {
    if (active) storeDraft();
  }, AUTOSAVE_MS);

  // Uten nøkkel: filene lastes ned som zip (stiene som i repoet).
  function download() {
    const out = JSON.parse(engine.save_files());
    if (out.error) {
      setSaved(out.error);
      toast?.show?.(out.error);
      return false;
    }
    const url = URL.createObjectURL(makeZip(out.files));
    const a = document.createElement("a");
    a.href = url;
    a.download = `fristad-redigering-${new Date().toISOString().slice(0, 16).replace(/[:T]/g, "-")}.zip`;
    document.body.append(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 5000);
    const text = `Lastet ned ${out.files.length} filer kl. ${hhmm()}. Lim inn en nøkkel under Lagre for å lagre på grenen.`;
    setSaved(text);
    toast?.show?.(text);
    return true;
  }

  // Første gang du går inn i redigeringen: fortsett fra utkastet, ellers fra
  // det du lagret sist på grenen.
  let restored = false;
  async function restore() {
    restored = true;
    const now = JSON.parse(engine.save_files());
    if (!now.files) return;
    const cur = new Map(now.files.map((f) => [f.path, f.text]));
    shipped = hash(now.files.map((f) => f.text).join("\n"));
    let files = [];
    let why = "";
    const draft = readDraft();
    if (draft && draft.base === shipped) {
      files = draft.files ?? [];
      why = draft.saved ? "Fortsetter der du slapp." : `Fortsetter med utkastet fra kl. ${hhmm(draft.at)}, som ikke er lagret ennå.`;
      if (draft.saved) lastSaved = JSON.stringify(draft.files);
    } else {
      if (draft) toast?.show?.("Utkastet var fra en eldre utgave av byen og er lagt bort.");
      try {
        files = await branchFiles([...cur.keys()]);
        why = `Fortsetter fra det du lagret sist (grenen ${SAVE_BRANCH}).`;
      } catch (e) {
        setSaved(`Fikk ikke hentet det du lagret: ${e.message}`);
      }
    }
    const changed = files.filter((f) => cur.has(f.path) && cur.get(f.path) !== f.text);
    if (!changed.length) return;
    toast?.show?.("Henter det du gjorde sist …");
    setTimeout(() => {
      if (engine.open_files(JSON.stringify({ files: changed }))) {
        toast?.show?.(why);
        setSaved(why);
      } else toast?.show?.("Kunne ikke åpne det du gjorde sist.");
    }, 30);
  }

  const makeGroup = () => send({ cmd: "group", name: $("ed-grp-name").value.trim() });
  const freeGroup = () => send({ cmd: "ungroup" });
  $("ed-grp-make").addEventListener("click", makeGroup);
  $("ed-grp-free").addEventListener("click", freeGroup);
  $("ed-undo").addEventListener("click", () => send({ cmd: "undo" }));
  $("ed-redo").addEventListener("click", () => send({ cmd: "redo" }));
  $("ed-play").addEventListener("click", () => play());
  $("ed-wall-h").addEventListener("change", (e) => send({ cmd: "wall_height", h: Number(e.target.value) || 3 }));
  $("ed-under").addEventListener("click", (e) => {
    const b = e.target.closest("[data-k]");
    if (b) send({ cmd: "select_under", k: Number(b.dataset.k), add: e.shiftKey });
  });
  $("ed-history").addEventListener("click", (e) => {
    const b = e.target.closest("[data-n]");
    if (b) send({ cmd: "undo_to", n: Number(b.dataset.n) });
  });
  // Egenskapene: Enter eller å gå ut av feltet setter verdien (ett steg å
  // angre). Fargevelgeren skriver i tekstfeltet ved siden av.
  function setProp(input) {
    const key = input.name;
    const value = input.type === "checkbox" ? (input.checked ? "ja" : "nei") : input.value;
    if (input.dataset.was === value) return;
    input.dataset.was = value;
    send({ cmd: "set_prop", key, value });
  }
  $("ed-props").addEventListener("submit", (e) => e.preventDefault());
  $("ed-props").addEventListener("change", (e) => {
    const t = e.target;
    if (t.dataset.pick) {
      const text = $("ed-props").querySelector(`[name="${t.dataset.pick}"]`);
      text.value = t.value;
      setProp(text);
    } else if (t.name) setProp(t);
  });
  $("ed-props").addEventListener("keydown", (e) => {
    if (e.key === "Enter" && e.target.tagName === "INPUT") {
      e.preventDefault();
      setProp(e.target);
    }
  });

  // Hendelsen fra motoren etter hver endring.
  function update(v) {
    view = v;
    if (!v) return;
    const st = $("ed-state");
    st.textContent = v.clean ? (v.dirty ? "Grønt, endret" : "Grønt") : `${v.breaches.length} brudd`;
    st.dataset.variant = v.clean ? "success" : "danger";
    $("ed-undo").disabled = !v.can_undo;
    $("ed-redo").disabled = !v.can_redo;
    $("ed-save").disabled = !v.dirty;
    $("ed-save").title = v.clean ? "Lagre på grenen redigering (Ctrl+S)" : "Rett reglene som er brutt før du lagrer";
    draftLater();
    const sel = v.selected;
    $("ed-selected").textContent = sel.length ? sel.map((s) => s.label).join(", ") : "Ingenting. Klikk på noe i bildet.";
    // Det som lå under pekeren, nærmest først: klikk for å velge.
    const under = v.under ?? [];
    $("ed-under-box").hidden = under.length < 2;
    $("ed-under").innerHTML = under
      .map((u, k) => `<li><button type="button" class="editor-under-item${u.selected ? " is-active" : ""}" data-k="${k}">${esc(u.label)}</button></li>`)
      .join("");
    renderProps(v.props ?? []);
    renderHistory(v.history ?? [], v.step ?? 0);
    if (v.error) toast?.show?.(v.error);
    if (ghostModel()) {
      const p = $("ed-placing");
      p.hidden = false;
      const g = v.ghost;
      p.textContent = !g ? `Dra ${ghostModel()} ut i bildet.` : g.length ? `${ghostModel()}: ${g.join(", ")}.` : `${ghostModel()}: grønt. Slipp for å sette ut (R dreier).`;
      p.dataset.state = g && g.length ? "bad" : "ok";
    }
    if (typeof v.snap === "boolean") snapSent = v.snap;
    $("ed-wall").hidden = !sel.some((s) => s.label === "usynlig vegg");
    // Gruppen: navnet når én gruppe er valgt, og knappene.
    const grp = $("ed-grp");
    grp.hidden = !v.group;
    grp.textContent = v.group ? `Gruppen «${v.group}» (${sel.length} ting).` : "";
    $("ed-grp-make").disabled = sel.length < 2;
    $("ed-grp-free").disabled = !(v.props ?? []).some((p) => p.key === "group" && p.value) && !v.group;
    // Ruta til en valgt innbygger.
    const route = (v.props ?? []).filter((p) => p.key.startsWith("route.")).length;
    const rt = $("ed-route");
    rt.hidden = !route;
    rt.textContent =
      v.route_point != null
        ? `Punkt ${v.route_point + 1} av ${route} på ruta er tatt: dra det langs bakken, Delete sletter det.`
        : `Ruta har ${route} punkter (stolpene): dra dem, klikk på streken for et nytt punkt.`;
    const list = $("ed-breaches");
    list.innerHTML = v.breaches.length
      ? v.breaches
          .slice(0, 30)
          .map((b) => `<li class="fs-paragraph" data-size="small">${esc(b.why)}</li>`)
          .join("") + (v.breaches.length > 30 ? `<li class="fs-paragraph" data-size="small">… og ${v.breaches.length - 30} til</li>` : "")
      : `<li class="fs-paragraph" data-size="small">Alt er grønt.</li>`;
  }

  // Egenskapene til det valgte: et felt per egenskap (tekst, tall, farge).
  // Panelet tegnes bare på nytt når noe er endret, så feltet du skriver i
  // ikke mister fokus.
  let propsKey = "";
  function renderProps(props) {
    const key = JSON.stringify(props);
    if (key === propsKey) return;
    propsKey = key;
    $("ed-props-box").hidden = !props.length;
    const field = (p) => {
      const name = esc(p.key);
      const value = esc(p.value);
      if (p.key === "kind") return `<input class="fs-input" name="${name}" value="${value}" data-was="${value}" readonly>`;
      if (p.key === "text" || p.key === "lines") return `<textarea class="fs-input" name="${name}" rows="3" data-was="${value}">${value}</textarea>`;
      if ((p.key === "flicker" && (p.value === "ja" || p.value === "nei")) || p.key === "always")
        return `<input type="checkbox" name="${name}" ${p.value === "ja" ? "checked" : ""} data-was="${value}">`;
      if (p.key === "tint" || p.key === "color")
        return `<span class="editor-color"><input type="color" data-pick="${name}" value="${/^#[0-9a-f]{6}$/i.test(p.value) ? value : "#ffffff"}" aria-label="Velg farge"><input class="fs-input" name="${name}" value="${value}" data-was="${value}" placeholder="#rrggbb"></span>`;
      // Valgene (fiendeslagene): en liste å velge fra, fri tekst går også.
      if (p.options?.length)
        return `<input class="fs-input" name="${name}" value="${value}" data-was="${value}" list="ed-opt-${name}"><datalist id="ed-opt-${name}">${p.options.map((o) => `<option value="${esc(o)}"></option>`).join("")}</datalist>`;
      const num = ["x", "y", "z", "yaw", "range", "intensity", "flicker", "width", "height", "n"].includes(p.key) || p.key.startsWith("route.");
      return `<input class="fs-input" name="${name}" value="${value}" data-was="${value}" ${num ? 'inputmode="decimal"' : ""}>`;
    };
    $("ed-props").innerHTML = props.map((p) => `<label class="editor-prop"><span class="fs-label">${esc(p.label)}</span>${field(p)}</label>`).join("");
    // Teksten i et tekstfelt settes når du går ut av det.
    for (const t of $("ed-props").querySelectorAll("textarea")) t.addEventListener("blur", () => setProp(t));
  }

  // Historikken: «Start» og hvert steg; det du står på er markert, og det
  // som kan gjøres om, er dempet. Klikk går dit.
  function renderHistory(steps, at) {
    const items = [`<li><button type="button" class="editor-step${at === 0 ? " is-active" : ""}" data-n="0">Start</button></li>`];
    steps.forEach((s, k) => {
      const n = k + 1;
      items.push(`<li><button type="button" class="editor-step${n === at ? " is-active" : ""}${n > at ? " is-undone" : ""}" data-n="${n}">${esc(s)}</button></li>`);
    });
    const list = $("ed-history");
    list.innerHTML = items.slice(-40).join("");
    list.querySelector(".is-active")?.scrollIntoView?.({ block: "nearest" });
  }

  function begin() {
    if (!send({ cmd: "enter" })) return false;
    active = true;
    panel.hidden = false;
    back.hidden = true;
    document.body.classList.add("editing");
    if (document.pointerLockElement) document.exitPointerLock?.();
    renderCatalog();
    onEdit?.(true);
    if (!restored) restore();
    return true;
  }

  // P: spill det du har laget (spillet bygges på nytt fra dataene).
  function play() {
    if (!active) return;
    // Musa fanges mens tastetrykket ennå teller som en handling fra deg.
    onLock?.();
    toast?.show?.("Bygger byen på nytt …");
    setTimeout(() => {
      if (!engine.play()) {
        toast?.show?.("Kunne ikke bygge byen på nytt.");
        return;
      }
      active = false;
      lining = false;
      setPlacing(null);
      panel.hidden = true;
      back.hidden = false;
      document.body.classList.remove("editing");
      onPlay?.();
    }, 30);
  }

  // Pekeren i 0..1 fra øverst til venstre i bildet.
  function at(e) {
    const r = canvas.getBoundingClientRect();
    return { x: (e.clientX - r.left) / r.width, y: (e.clientY - r.top) / r.height };
  }

  function onCanvas(e) {
    return e.target === canvas;
  }

  // Musa: gir true når redigeringen tok den.
  function mouseDown(e) {
    if (!active) return false;
    if (!onCanvas(e)) return true;
    e.preventDefault();
    // Ctrl+klikk på Mac kommer som høyre knapp: det er dreiingen.
    const turn = e.ctrlKey || e.metaKey;
    if (e.button === 2 && !turn) {
      looking = true;
      return true;
    }
    if (e.button !== 0 && e.button !== 2) return true;
    const p = at(e);
    if (lining) {
      send({ cmd: "line_point", ...p });
      return true;
    }
    if (placing) {
      send({ cmd: "place", model: placing, ...p });
      if (!e.shiftKey) setPlacing(null);
      else ghost(placing, p);
      return true;
    }
    // Shift: et klikk legger til eller tar bort, et drag er en ramme.
    if (e.shiftKey && e.button === 0 && !turn && !e.altKey) {
      boxFrom = { x: e.clientX, y: e.clientY, p };
      boxing = false;
      return true;
    }
    pressed = true;
    send({ cmd: "press", ...p, add: e.shiftKey, lift: e.altKey && !turn, turn });
    canvas.style.cursor = turn ? "ew-resize" : e.altKey ? "ns-resize" : "grabbing";
    return true;
  }

  // Pekeren sier hva et drag vil gjøre: Alt heiser, Ctrl eller Cmd dreier.
  function hint(e) {
    if (!active || pressed) return;
    canvas.style.cursor = e.ctrlKey || e.metaKey ? "ew-resize" : e.altKey ? "ns-resize" : "";
  }

  function drawBox(e) {
    const x0 = Math.min(boxFrom.x, e.clientX);
    const y0 = Math.min(boxFrom.y, e.clientY);
    Object.assign(box.style, { left: `${x0}px`, top: `${y0}px`, width: `${Math.abs(e.clientX - boxFrom.x)}px`, height: `${Math.abs(e.clientY - boxFrom.y)}px` });
    box.hidden = false;
  }

  function mouseMove(e) {
    if (!active) return false;
    if (dragModel && !dragModel.moved && Math.hypot(e.clientX - dragModel.x, e.clientY - dragModel.y) > SLOP) {
      dragModel.moved = true;
      document.body.classList.add("editor-dragging");
    }
    if (looking) engine.look(e.movementX, e.movementY);
    else if (boxFrom) {
      if (!boxing && Math.hypot(e.clientX - boxFrom.x, e.clientY - boxFrom.y) > SLOP) boxing = true;
      if (boxing) drawBox(e);
    } else if (pressed) send({ cmd: "drag", ...at(e) });
    else {
      const model = ghostModel();
      if (model && onCanvas(e)) ghost(model, at(e));
      else if (model) hideGhost();
      hint(e);
    }
    return true;
  }

  function mouseUp(e) {
    if (!active) return false;
    if (dragModel) {
      const d = dragModel;
      dragModel = null;
      document.body.classList.remove("editor-dragging");
      if (!d.moved) {
        setPlacing(placing === d.name ? null : d.name);
      } else if (onCanvas(e)) {
        // Slipp: settes ned på det som er under, der fantomet sto.
        send({ cmd: "place", model: d.name, ...at(e) });
        hideGhost();
        setPlacing(placing);
      } else {
        hideGhost();
        setPlacing(placing);
      }
      return true;
    }
    if (e.button === 2 && looking) looking = false;
    else if (boxFrom) {
      const from = boxFrom;
      boxFrom = null;
      box.hidden = true;
      if (boxing) {
        const to = at(e);
        send({ cmd: "select_box", x0: from.p.x, y0: from.p.y, x1: to.x, y1: to.y, add: true });
      } else {
        send({ cmd: "press", ...from.p, add: true, lift: false, turn: false });
        send({ cmd: "release" });
      }
      boxing = false;
    } else if (pressed) {
      pressed = false;
      send({ cmd: "release" });
      hint(e);
    }
    return true;
  }

  // Det valgte langs bakken etter hvor du ser (piltastene).
  function along(forward, right) {
    const yaw = engine.view()[3];
    const f = { x: -Math.sin(yaw), z: -Math.cos(yaw) };
    const r = { x: -f.z, z: f.x };
    return { x: f.x * forward + r.x * right, z: f.z * forward + r.z * right };
  }

  // Tastene: gir true når redigeringen tok tasten (resten går til motoren:
  // flyging med WASD, mellomrom og C).
  function key(e) {
    if (!active) {
      if (e.code === "KeyP" && !e.repeat && engine.can_edit()) {
        e.preventDefault();
        begin();
        return true;
      }
      return false;
    }
    if (e.target instanceof Element && e.target.closest("input, select, textarea")) return true;
    const step = e.altKey ? STEP / 10 : e.shiftKey ? STEP * 10 : STEP;
    const turn = e.altKey ? TURN / 15 : TURN;
    const ctrl = e.ctrlKey || e.metaKey;
    const move = (x, y, z) => send({ cmd: "move", x, y, z });
    switch (e.code) {
      case "AltLeft":
      case "AltRight":
      case "ControlLeft":
      case "ControlRight":
      case "MetaLeft":
      case "MetaRight":
        // Alt flytter ellers fokus til menylinja i noen nettlesere.
        if (e.code.startsWith("Alt")) e.preventDefault();
        else if (!ctrlHeld) {
          ctrlHeld = true;
          sendSnap();
        }
        hint(e);
        return true;
      case "KeyR":
        if (e.repeat || ctrl) return !ctrl;
        // En kvart runde: fantomet før det settes ut, ellers det valgte.
        if (ghostModel()) send({ cmd: "ghost_turn", a: e.shiftKey ? Math.PI / 2 : -Math.PI / 2 });
        else send({ cmd: "rotate", a: e.shiftKey ? Math.PI / 2 : -Math.PI / 2 });
        return true;
      case "Digit1":
      case "Digit2":
      case "Digit3":
      case "Digit4":
      case "Digit5":
      case "Digit6":
      case "Digit7":
      case "Digit8":
      case "Digit9":
        e.preventDefault();
        if (!e.repeat) bookmark(e.code.slice(5), ctrl);
        return true;
      case "ArrowUp":
      case "ArrowDown":
      case "ArrowLeft":
      case "ArrowRight": {
        e.preventDefault();
        const f = e.code === "ArrowUp" ? 1 : e.code === "ArrowDown" ? -1 : 0;
        const r = e.code === "ArrowRight" ? 1 : e.code === "ArrowLeft" ? -1 : 0;
        const d = along(f * step, r * step);
        move(d.x, 0, d.z);
        return true;
      }
      case "PageUp":
      case "PageDown":
        e.preventDefault();
        move(0, e.code === "PageUp" ? step : -step, 0);
        return true;
      case "KeyQ":
      case "KeyE":
        if (e.repeat && !e.altKey) return true;
        send({ cmd: "rotate", a: e.code === "KeyQ" ? turn : -turn });
        return true;
      case "KeyG":
        if (ctrl) {
          e.preventDefault();
          if (!e.repeat) e.shiftKey ? freeGroup() : makeGroup();
          return true;
        }
        if (!e.repeat) send({ cmd: "drop" });
        return true;
      case "KeyD":
        if (ctrl) {
          e.preventDefault();
          if (!e.repeat) send({ cmd: "copy" });
          return true;
        }
        return false;
      case "Delete":
      case "Backspace":
        e.preventDefault();
        if (!e.repeat) send({ cmd: "delete" });
        return true;
      case "KeyZ":
        if (ctrl) {
          e.preventDefault();
          send({ cmd: e.shiftKey ? "redo" : "undo" });
          return true;
        }
        return false;
      case "KeyS":
        if (ctrl) {
          e.preventDefault();
          openSave(true);
          return true;
        }
        return false;
      case "KeyY":
        if (ctrl) {
          e.preventDefault();
          send({ cmd: "redo" });
          return true;
        }
        return false;
      case "KeyL":
        if (!e.repeat) {
          lining = true;
          send({ cmd: "line_cancel" });
          toast?.show?.("Linje: klikk punkter på bakken. Enter gir en rad av modellen du har valgt i katalogen, V en usynlig vegg, Esc avbryter.");
        }
        return true;
      case "Enter":
        if (lining) {
          e.preventDefault();
          if (!placing) {
            toast?.show?.("Velg en modell i katalogen (et gjerde) først, eller trykk V for en usynlig vegg.");
            return true;
          }
          send({ cmd: "line_finish", kind: "row", model: placing });
          lining = false;
          setPlacing(null);
        }
        return true;
      case "KeyV":
        if (lining) {
          send({ cmd: "line_finish", kind: "wall", height: Number($("ed-wall-h").value) || 3 });
          lining = false;
          return true;
        }
        return false;
      case "Escape":
        if (lining || placing || dragModel) {
          e.preventDefault();
          dragModel = null;
          document.body.classList.remove("editor-dragging");
          lining = false;
          send({ cmd: "line_cancel" });
          setPlacing(null);
          return true;
        }
        return false;
      case "Tab":
        e.preventDefault();
        if (!e.repeat) send({ cmd: e.shiftKey ? "cycle_back" : "cycle" });
        return true;
      case "KeyP":
        if (!e.repeat) play();
        return true;
      default:
        return false;
    }
  }

  document.addEventListener("keyup", (e) => {
    if (active && ctrlHeld && !e.ctrlKey && !e.metaKey) {
      ctrlHeld = false;
      sendSnap();
    }
    hint(e);
  });
  // Fanen mister fokus med Ctrl nede: slipp den.
  window.addEventListener("blur", () => {
    if (ctrlHeld) {
      ctrlHeld = false;
      if (active) sendSnap();
    }
  });

  return {
    get active() {
      return active;
    },
    begin,
    update,
    key,
    mouseDown,
    mouseMove,
    mouseUp,
  };
}
