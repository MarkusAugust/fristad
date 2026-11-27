// Lagringen i nivåredigeringen: hvilket dokument siden åpner med, Åpne-
// dialogen, og Lagre til grenen `redigering` (web/github.js) med
// konfliktsjekken, eller som zip uten nøkkel.

import { level_blank, level_doc_names, level_doc_source } from "./pkg/fristad.js";
import { makeZip } from "./zip.js";
import { blobSha, branchFile, branchFiles, commit, SAVE_AT, SAVE_FROM, setToken, token } from "./github.js";
import { docPath, refresh, send } from "./nivaa-doc.js";
import { clearPendingModules, clearPendingTemplates, pendingModules, pendingTemplates } from "./nivaa-palette.js";
import { $, BASE, DRAFT_PREFIX, engine, esc, field, LAST, palette, params, setStatus, st, store, THEME_NAMES, toast } from "./nivaa-shared.js";
import { centerOn, frameFloor } from "./nivaa-view.js";

// Dokumentet siden åpner med: `?nivaa=<navn>` eller `&fil=<navn>` fra
// repoet (et nyere utkast går foran), ellers det siste du hadde oppe, ellers
// et nytt nivå 1.
export function firstDoc() {
  const name = params.get("fil") || params.get("nivaa") || "";
  const path = name ? `assets/levels/nivaa_${name}.json` : store.get(LAST) || "";
  const draft = path && store.get(DRAFT_PREFIX + path);
  if (draft) {
    try {
      const d = JSON.parse(draft);
      if (d.text) return { text: d.text, note: `Fortsetter fra utkastet (${new Date(d.at).toLocaleString("nb")}).` };
    } catch {
      /* ødelagt utkast */
    }
  }
  const short = path.replace(/^assets\/levels\/nivaa_/, "").replace(/\.json$/, "");
  const text = short ? level_doc_source(short) : "";
  if (text) {
    if (!store.get(BASE + path)) rememberBase(path, text);
    return { text, note: "" };
  }
  return { text: level_blank("Nytt nivå", 1, "grav"), note: "Nytt nivå." };
}

export async function save() {
  const d = JSON.parse(engine.level_doc());
  store.set(DRAFT_PREFIX + d.path, JSON.stringify({ at: Date.now(), text: d.text }));
  if (!token()) {
    const key = prompt("Nøkkel til GitHub (github_pat_…, «Contents: Read and write» på fristad-repoet). La stå tom for å laste ned en zip i stedet.");
    if (key) setToken(key.trim());
  }
  if (!token()) {
    const url = URL.createObjectURL(makeZip([d, ...pendingTemplates(), ...pendingModules()]));
    const a = document.createElement("a");
    a.href = url;
    a.download = d.path.split("/").pop().replace(".json", ".zip");
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 2000);
    send({ cmd: "saved" });
    toast(`Lastet ned ${a.download} (utkastet er lagret i nettleseren).`);
    return;
  }
  setStatus("Lagrer …");
  try {
    const conflict = await checkConflict(d);
    if (conflict === "avbryt") return setStatus("");
    const tpls = pendingTemplates();
    const mods = pendingModules();
    const r = await commit([d, ...tpls, ...mods], `Nivåredigeringen: ${st.map.name || d.path}`);
    clearPendingTemplates();
    clearPendingModules();
    rememberBase(d.path, d.text);
    send({ cmd: "saved" });
    toast(r.unchanged ? "Ingenting nytt å lagre." : `Lagret ${SAVE_AT}.`);
  } catch (e) {
    toast(`Lagringen feilet: ${e.message}. Utkastet ligger i nettleseren.`, 7000);
  }
  setStatus("");
}

/// Husk hva redigeringen av `path` bygger på (teksten fra repoet, grenen
/// eller det som ble lagret).
async function rememberBase(path, text) {
  try {
    store.set(BASE + path, await blobSha(text));
  } catch {
    /* uten crypto.subtle (http): ingen konfliktsjekk */
  }
}

/// Før lagring: har fila på grenen endret seg siden redigeringen begynte
/// (lagret fra en annen fane eller maskin)? Da velger eieren å skrive over,
/// åpne den som ligger der, eller avbryte. Gir "ok", "over" eller "avbryt".
async function checkConflict(d) {
  const there = await branchFile(d.path).catch(() => null);
  if (!there) return "ok";
  const mine = await blobSha(d.text).catch(() => "");
  const base = store.get(BASE + d.path);
  if (there.sha === mine || there.sha === base) return "ok";
  return new Promise((done) => {
    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML = `<h2 class="lt-heading" data-size="s">Fila er endret ${SAVE_AT}</h2>
      <p class="lt-paragraph">${esc(d.path)} ${SAVE_AT} er endret et annet sted siden du begynte her (en annen fane, en annen maskin eller rett i fila). Lagrer du, skrives den over.</p>
      <div class="row"><button class="lt-button" data-variant="danger" id="c-over">Skriv over</button>
      <button class="lt-button" data-variant="secondary" id="c-open">Åpne den som ligger der</button>
      <button class="lt-button" data-variant="secondary" id="c-cancel">Avbryt</button></div>`;
    const ov = $("overlay");
    ov.replaceChildren(card);
    ov.hidden = false;
    const close = (v) => {
      ov.hidden = true;
      done(v);
    };
    card.querySelector("#c-over").onclick = () => close("over");
    card.querySelector("#c-cancel").onclick = () => close("avbryt");
    card.querySelector("#c-open").onclick = () => {
      openText(there.text, `Åpnet ${d.path} fra ${SAVE_FROM}; utkastet ditt ligger i nettleseren.`);
      rememberBase(d.path, there.text);
      close("avbryt");
    };
  });
}

export function openDialog() {
  const card = document.createElement("div");
  card.className = "card";
  const names = level_doc_names().filter((n) => n !== "brett");
  const drafts = [];
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(DRAFT_PREFIX)) drafts.push(k.slice(DRAFT_PREFIX.length));
  }
  card.innerHTML = `
    <h2 class="lt-heading" data-size="s">Åpne et nivå</h2>
    <p class="muted">Utkastene i denne nettleseren og nivåene i repoet. Et nivå merket «klar» erstatter det gamle nivået med samme nummer.</p>
    <div class="doclist">${drafts.map((p) => `<button class="lt-button" data-variant="secondary" data-draft="${p}">Utkast: ${p.replace("assets/levels/", "")}</button>`).join("")}
    ${names.map((n) => `<button class="lt-button" data-variant="secondary" data-repo="${n}">I repoet: nivaa_${n}.json</button>`).join("")}
    <button class="lt-button" data-variant="secondary" data-branch="1">Hent dette nivået fra ${SAVE_FROM}</button></div>
    <h2 class="lt-heading" data-size="s">Nytt nivå</h2>
    <div class="row" style="align-items:flex-end">
      ${field("Navnet", `<input id="new-name" class="lt-input" value="Nytt nivå">`).replace('class="field"', 'class="field" style="flex:1"')}
      ${field("Nivå", `<input id="new-n" class="lt-input" type="number" min="1" step="1" value="1" style="width:5em">`)}
      ${field(
        "Temaet",
        `<select id="new-theme" class="lt-select">${(palette.themes.length ? palette.themes : ["grav"]).map((t) => `<option value="${t}">${THEME_NAMES[t] || t}</option>`).join("")}</select>`,
      )}
      <button id="new-make" class="lt-button" data-variant="primary" style="margin-bottom:8px">Lag</button>
    </div>
    <p class="muted">Det nye nivået begynner med kapellet med døra opp og startstedet. Dra rom og ganger fra paletten inn til åpningene.</p>
    <div class="row" style="margin-top:12px"><button id="open-cancel" class="lt-button" data-variant="secondary">Avbryt</button></div>`;
  const ov = $("overlay");
  ov.replaceChildren(card);
  ov.hidden = false;
  const close = () => (ov.hidden = true);
  card.querySelector("#open-cancel").onclick = close;
  card.querySelectorAll("[data-draft]").forEach((b) =>
    b.addEventListener("click", () => {
      const d = JSON.parse(store.get(DRAFT_PREFIX + b.dataset.draft) || "{}");
      if (d.text) openText(d.text, "Utkastet er åpnet.");
      close();
    }),
  );
  card.querySelectorAll("[data-repo]").forEach((b) =>
    b.addEventListener("click", () => {
      const text = level_doc_source(b.dataset.repo);
      openText(text, `nivaa_${b.dataset.repo}.json fra repoet.`);
      rememberBase(`assets/levels/nivaa_${b.dataset.repo}.json`, text);
      close();
    }),
  );
  card.querySelector("[data-branch]").onclick = async () => {
    try {
      const files = await branchFiles([docPath]);
      if (!files.length) toast(token() ? `${docPath} finnes ikke ${SAVE_AT}.` : "Trenger nøkkelen til GitHub (Lagre spør etter den).");
      else {
        openText(files[0].text, `Hentet ${docPath} fra ${SAVE_FROM}.`);
        rememberBase(docPath, files[0].text);
      }
    } catch (e) {
      toast(e.message);
    }
    close();
  };
  card.querySelector("#new-make").onclick = () => {
    const n = Math.max(1, Math.floor(Number(card.querySelector("#new-n").value)) || 1);
    openText(level_blank(card.querySelector("#new-name").value || `Nivå ${n}`, n, card.querySelector("#new-theme").value), `Nytt nivå ${n}.`);
    close();
  };
}

function openText(text, note) {
  const out = JSON.parse(engine.level_open(text));
  if (out.error) return toast(out.error);
  st.sel = [];
  st.selOpening = null;
  refresh(out);
  centerOn();
  frameFloor();
  if (note) toast(note);
}
