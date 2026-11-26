// Lagringen i redigeringen (`?edit`, web/editor.js): Lagre legger filene på
// grenen `redigering` (web/github.js), én commit per økt, eller laster dem
// ned som zip uten nøkkel. Det du gjør, huskes som et utkast i nettleseren
// (`fristad.edit-draft`) ved hver endring og hvert 30. sekund, og neste gang
// fortsetter du fra utkastet, eller fra det du lagret sist.

import { branchFiles, commit, SAVE_BRANCH, setToken, token } from "./github.js";
import { makeZip } from "./zip.js";

const DRAFT = "fristad.edit-draft";
const AUTOSAVE_MS = 30000;

/// `status` er linja i panelet som sier hva som er lagret, `getView` det
/// motoren sist sendte (`dirty`, `clean`, `breaches`), og `isActive` om du
/// redigerer nå.
export function createSaving({ engine, toast, status, getView, isActive }) {
  const dialog = document.createElement("dialog");
  dialog.className = "lt-dialog editor-save";
  dialog.innerHTML = `
    <h2 class="lt-dialog__title lt-heading" data-size="s">Lagre endringene</h2>
    <div class="lt-dialog__body">
      <p class="lt-paragraph" data-size="small">Endringene legges på grenen <code>${SAVE_BRANCH}</code> i fristad-repoet, én commit for hele økta (lagrer du igjen,
      oppdateres den). Neste gang du redigerer, fortsetter du derfra. Claude kjører testene og slår dem inn i spillet.</p>
      <label class="lt-label" for="ed-msg">Hva du har endret (kan stå tomt)</label>
      <input id="ed-msg" class="lt-input" type="text">
      <div id="ed-token-row">
        <label class="lt-label" for="ed-token">GitHub-nøkkel</label>
        <input id="ed-token" class="lt-input" type="password" autocomplete="off" placeholder="github_pat_…">
        <p class="lt-help-text">En finkornet nøkkel (github.com → Settings → Developer settings → Fine-grained tokens) med tilgang bare til fristad-repoet og «Contents:
        Read and write». Den huskes bare i denne nettleseren.</p>
      </div>
      <p id="ed-save-status" class="lt-paragraph" data-size="small" role="status"></p>
    </div>
    <div class="lt-dialog__footer">
      <button type="button" class="lt-button" data-variant="ghost" data-act="forget">Glem nøkkelen</button>
      <button type="button" class="lt-button" data-variant="secondary" data-act="close">Avbryt</button>
      <button type="button" class="lt-button" data-variant="secondary" data-act="zip">Last ned zip</button>
      <button type="button" class="lt-button" data-variant="primary" data-act="save">Lagre</button>
    </div>`;
  document.body.append(dialog);
  const d$ = (sel) => dialog.querySelector(sel);
  const hhmm = (t = Date.now()) => new Date(t).toLocaleTimeString("nb-NO", { hour: "2-digit", minute: "2-digit" });
  const setSaved = (text) => (status.textContent = text);
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
    const view = getView();
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
    else d$("#ed-save-status").textContent = status.textContent;
  });

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
      if (!getView()?.dirty) {
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
    if (isActive()) storeDraft();
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

  // Første gang du går inn i redigeringen (senere kall gjør ingenting):
  // fortsett fra utkastet, ellers fra det du lagret sist på grenen.
  let restored = false;
  async function restore() {
    if (restored) return;
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

  return { openSave, draftLater, restore };
}
