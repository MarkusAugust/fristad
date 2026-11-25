// Dokumentet i nivåredigeringen: endringene som sendes til kjernen
// (`send`), det som følger når kartet kommer tilbake (`refresh`), utvalget,
// utklippet og utkastet som huskes i nettleseren.

import { $, DRAFT_PREFIX, engine, LAST, paletteFilter, redraw, st, store, toast } from "./nivaa-shared.js";

// Panelet (nivaa-panel.js) viser utvalget, bruddene og tallene. Det gis hit
// med `onShow` (fra nivaa.js), så dokumentet ikke kjenner panelet.
let show = { issues() {}, selection() {}, stats() {} };
export const onShow = (fns) => {
  show = fns;
};

export let clipboard = null;
export let docPath = "";

/// Send en kommando eller endring; kartet kommer tilbake.
export function send(cmd) {
  const out = JSON.parse(engine.level(JSON.stringify(cmd)));
  if (out.error) {
    toast(out.error);
    return null;
  }
  refresh(out);
  return out;
}

export function refresh(map) {
  st.map = map;
  // Etasjene: gulvene som finnes, og de nye eieren har lagt til (tomme til
  // noe står der). En ny etasje står 6 m over eller under (`docs/nivaaredigering.md` del 25).
  const floors = [...new Set([...(map.floors.length ? map.floors : [0]), ...st.extraFloors])].sort((a, b) => a - b);
  const sel = $("floor");
  const want = floors.map(String).join();
  if (sel.dataset.list !== want) {
    const top = Math.max(...floors);
    const low = Math.min(...floors);
    sel.innerHTML =
      floors.map((f) => `<option value="${f}">${f === 0 ? "Hovedplanet (0 m)" : `${f > 0 ? "+" : ""}${f} m`}${map.floors.includes(f) ? "" : " (tom)"}</option>`).join("") +
      `<option value="new:${top + 6}">Ny etasje over (+${top + 6} m)</option><option value="new:${low - 6}">Ny etasje under (${low - 6} m)</option>`;
    sel.dataset.list = want;
  }
  if (!floors.includes(st.floor)) st.floor = floors.reduce((a, f) => (Math.abs(f - st.floor) < Math.abs(a - st.floor) ? f : a), floors[0]);
  sel.value = String(st.floor);
  st.floors = floors;
  // Det valgte som ikke finnes lenger.
  const ids = new Set([...map.modules.map((m) => m.id), ...map.things.map((t) => t.id), ...map.lights.map((l) => l.id)]);
  st.sel = st.sel.filter((id) => ids.has(id));
  $("docname").textContent = `${map.name || "Uten navn"} · ${/^\d+$/.test(map.level) ? `nivå ${map.level}` : map.level}`;
  $("docstatus").textContent = map.status === "klar" ? "Klar" : "Utkast";
  $("docstatus").dataset.variant = map.status === "klar" ? "success" : "warning";
  $("undo").disabled = !map.can_undo;
  $("redo").disabled = !map.can_redo;
  $("save").textContent = map.unsaved ? `Lagre (${map.unsaved})` : "Lagre";
  $("ready").disabled = !map.clean || map.status === "klar";
  $("ready").title = map.clean ? "Merk nivået klart: det erstatter det gamle nivået med samme nummer" : "Rett feilene i reglene først";
  show.issues();
  show.selection();
  show.stats();
  redraw();
  docPath = JSON.parse(engine.level_doc()).path;
  if (map.theme !== refresh.theme) {
    refresh.theme = map.theme;
    paletteFilter(map.theme || map.modules[0]?.module.split("/")[0]);
  }
  store.set(LAST, docPath);
  scheduleDraft();
}

let draftTimer = 0;
function scheduleDraft() {
  if (!st.map || !st.map.unsaved) return;
  clearTimeout(draftTimer);
  draftTimer = setTimeout(() => {
    const d = JSON.parse(engine.level_doc());
    store.set(DRAFT_PREFIX + d.path, JSON.stringify({ at: Date.now(), text: d.text }));
  }, 800);
}

/// Sett ut en modul (`id`) med midtcella i `at` (til testene og konsollen).
export function placeModule(id, at, rot = 0, mirror = false) {
  const out = send({ op: "add_module", module: id, at, rot, mirror });
  if (out?.made?.length) select(out.made);
  return out;
}

export function select(ids, add = false) {
  st.sel = add ? [...new Set([...st.sel, ...ids])] : ids;
  st.selOpening = null;
  engine.level(JSON.stringify({ cmd: "select", ids: st.sel }));
  show.selection();
  redraw();
}

export function turnSelection(back) {
  const things = st.sel.filter((id) => !st.map.modules.some((m) => m.id === id));
  if (things.length === st.sel.length && things.length === 1) {
    const t = docThing(things[0]);
    if (t) {
      t.yaw = ((t.yaw || 0) + (back ? 90 : -90) + 360) % 360;
      return send({ op: "set_thing", thing: t });
    }
  }
  send({ op: "turn", ids: st.sel, back });
}

export function copy() {
  const doc = JSON.parse(JSON.parse(engine.level_doc()).text);
  clipboard = { modules: doc.modules?.filter((m) => st.sel.includes(m.id)) || [], things: doc.things?.filter((t) => st.sel.includes(t.id)) || [] };
  toast(`Kopierte ${clipboard.modules.length} moduler og ${clipboard.things.length} ting.`, 1500);
}

export function paste() {
  if (!clipboard) return;
  // Ved siden av: like langt til høyre som utvalget er bredt.
  const cells = st.map.modules.filter((m) => clipboard.modules.some((c) => c.id === m.id)).flatMap((m) => m.cells);
  const w = cells.length ? Math.max(...cells.map((c) => c[0])) - Math.min(...cells.map((c) => c[0])) + 1 : 1;
  const out = send({ op: "paste", modules: clipboard.modules, things: clipboard.things, by: [w, 0, 0] });
  if (out?.made?.length) select(out.made);
}

function docJson() {
  return JSON.parse(JSON.parse(engine.level_doc()).text);
}

export function docHead() {
  const d = docJson();
  return { name: d.name || "", level: d.level, status: d.status || "utkast", theme: d.theme || "", seed: d.seed || 0, stone: d.stone };
}

export function docThing(id) {
  return docJson().things?.find((t) => t.id === id) || null;
}

export function docLight(id) {
  return docJson().lights?.find((l) => l.id === id) || null;
}

export function docModule(id) {
  return docJson().modules?.find((m) => m.id === id) || null;
}
