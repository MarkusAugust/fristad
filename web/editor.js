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
// Det du gjør, huskes som et utkast i nettleseren (`fristad.edit-draft`),
// og Lagre legger det på grenen `redigering`, én commit per økt. Neste gang
// fortsetter du fra utkastet, eller fra det du lagret sist.

import { branchFiles, commit, setToken, token, SAVE_BRANCH } from "./github.js";

const DRAFT = "fristad.edit-draft";

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
    </div>
    <p id="ed-saved" class="fs-help-text" role="status"></p>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Valgt</h3>
      <p id="ed-selected" class="fs-paragraph" data-size="small">Ingenting. Klikk på noe i bildet.</p>
      <p id="ed-under" class="fs-paragraph" data-size="small" hidden></p>
      <div id="ed-wall" hidden>
        <label class="fs-label" for="ed-wall-h">Høyde på veggen (m)</label>
        <input id="ed-wall-h" class="fs-input" type="number" min="0.5" max="20" step="0.5" value="3">
      </div>
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
      <h3 class="fs-heading" data-size="xs">Linje</h3>
      <p class="fs-paragraph" data-size="small"><kbd>L</kbd> begynner: klikk punkter på bakken, <kbd>Enter</kbd> gir en rad av modellen du har valgt i katalogen (gjerde), <kbd>V</kbd> en usynlig vegg, <kbd>Esc</kbd> avbryter.</p>
    </section>
    <section class="editor-section editor-keys">
      <p class="fs-paragraph" data-size="small">
        <strong>Musa:</strong> dra en ting for å flytte den langs bakken, <kbd>Alt</kbd>+dra heiser opp og ned, <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+dra dreier (flere valgt dreies samlet), <kbd>Shift</kbd>+klikk velger flere.
        <kbd>WASD</kbd> fly, <kbd>Mellomrom</kbd>/<kbd>C</kbd> opp og ned, <kbd>Shift</kbd> fort, høyre knapp ser rundt.
        Piltastene flytter det valgte langs bakken, <kbd>PgUp</kbd>/<kbd>PgDn</kbd> opp og ned (<kbd>Alt</kbd> små steg, <kbd>Shift</kbd> store),
        <kbd>Q</kbd>/<kbd>E</kbd> dreier, <kbd>Tab</kbd> velger det neste under pekeren (kirka, så gjerdet inni boksen dens), <kbd>G</kbd> setter ned på det som er under, <kbd>Ctrl+D</kbd> kopierer, <kbd>Delete</kbd> sletter.
      </p>
    </section>`;
  document.body.append(panel);
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
  $("ed-grid").addEventListener("click", (e) => {
    const b = e.target.closest("[data-model]");
    if (!b) return;
    setPlacing(placing === b.dataset.model ? null : b.dataset.model);
  });

  function setPlacing(model) {
    placing = model;
    const p = $("ed-placing");
    p.hidden = !model;
    p.textContent = model ? `Klikk i bildet for å sette ut ${model} (Shift: flere). Esc avbryter.` : "";
    renderCatalog();
  }

  function send(cmd) {
    return engine.edit(JSON.stringify(cmd));
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

  function openSave() {
    if (!view?.dirty) return;
    if (!view.clean) {
      toast?.show?.(`${view.breaches.length} regler er brutt. Rett dem før du lagrer.`);
      return;
    }
    if (token()) {
      save();
      return;
    }
    d$("#ed-token-row").hidden = false;
    d$("[data-act=forget]").hidden = true;
    d$("#ed-msg").value = message;
    d$("#ed-save-status").textContent = "";
    dialog.showModal();
  }

  d$("[data-act=close]").addEventListener("click", () => dialog.close());
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
  $("ed-save").addEventListener("click", openSave);

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
    draftTimer = setTimeout(storeDraft, 1500);
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

  $("ed-undo").addEventListener("click", () => send({ cmd: "undo" }));
  $("ed-redo").addEventListener("click", () => send({ cmd: "redo" }));
  $("ed-play").addEventListener("click", () => play());
  $("ed-wall-h").addEventListener("change", (e) => send({ cmd: "wall_height", h: Number(e.target.value) || 3 }));

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
    // Det som lå under pekeren: klikk igjen eller Tab for det neste.
    const under = v.under ?? [];
    $("ed-under").hidden = under.length < 2;
    $("ed-under").innerHTML = `Under pekeren: ${under.map((u) => (u.selected ? `<strong>${esc(u.label)}</strong>` : esc(u.label))).join(" › ")}. Klikk igjen eller <kbd>Tab</kbd> for neste.`;
    $("ed-wall").hidden = !sel.some((s) => s.label === "usynlig vegg");
    const list = $("ed-breaches");
    list.innerHTML = v.breaches.length
      ? v.breaches
          .slice(0, 30)
          .map((b) => `<li class="fs-paragraph" data-size="small">${esc(b.why)}</li>`)
          .join("") + (v.breaches.length > 30 ? `<li class="fs-paragraph" data-size="small">… og ${v.breaches.length - 30} til</li>` : "")
      : `<li class="fs-paragraph" data-size="small">Alt er grønt.</li>`;
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

  function mouseMove(e) {
    if (!active) return false;
    if (looking) engine.look(e.movementX, e.movementY);
    else if (pressed) send({ cmd: "drag", ...at(e) });
    else hint(e);
    return true;
  }

  function mouseUp(e) {
    if (!active) return false;
    if (e.button === 2 && looking) looking = false;
    else if (pressed) {
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
        hint(e);
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
          openSave();
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
        if (lining || placing) {
          e.preventDefault();
          lining = false;
          send({ cmd: "line_cancel" });
          setPlacing(null);
          return true;
        }
        return false;
      case "Tab":
        e.preventDefault();
        if (!e.repeat) send({ cmd: "cycle" });
        return true;
      case "KeyP":
        if (!e.repeat) play();
        return true;
      default:
        return false;
    }
  }

  document.addEventListener("keyup", (e) => hint(e));

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
