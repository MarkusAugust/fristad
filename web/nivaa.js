// Nivåredigeringen (`?nivaa`, docs/nivaaredigering.md del 12): eieren
// bygger kryptnivåene av moduler på et kart ovenfra, med motoren som 3D ved
// siden av. Dokumentet og reglene bor i kjernen (`edit::level`,
// `LevelEditor`); siden sender endringene som json (`engine.level`) og
// tegner kartet den får tilbake (web/nivaa-plan.js). Utkastet huskes i
// nettleseren, Lagre legger fila på grenen `redigering` (web/github.js) eller
// laster ned en zip, og «Spill» bygger verden med nivået og lar deg gå
// gjennom det uten å kunne dø (Esc tilbake).

import init, { asset_paths, level_blank, level_palette, model_paths, model_url, start_level_editor } from "./pkg/fristad.js";
import { bind3d } from "./nivaa-3d.js";
import { copy, docHead, onShow, paste, placeModule, refresh, select, send, turnSelection } from "./nivaa-doc.js";
import { bindMap } from "./nivaa-map.js";
import {
  arm,
  buildPalette,
  disarm,
  ghostFor,
  loadOwnModules,
  loadTemplates,
  saveModule,
  saveTemplate,
  startLink,
  templateGhost,
} from "./nivaa-palette.js";
import { showIssues, showSelection, showStats } from "./nivaa-panel.js";
import { firstDoc, openDialog, save } from "./nivaa-save.js";
import {
  $,
  canvas,
  engine,
  params,
  plan,
  playing,
  redraw,
  setCanvas,
  setEngine,
  setPalette,
  setPlaying,
  setStatus,
  st,
  store,
  toast,
  typing,
} from "./nivaa-shared.js";
import { centerOn, fit, focusSelection, frameFloor, gotoMark, markCamera, sendCut, setFloor, setView, viewMode } from "./nivaa-view.js";

const HELP_SEEN = "fristad.nivaa.hjelp";

async function loadAssets() {
  try {
    const res = await fetch("assets/manifest.json", { cache: "no-cache" });
    if (!res.ok) return { manifest: undefined, names: [], blobs: [] };
    const manifest = await res.text();
    const paths = asset_paths(manifest);
    const blobs = new Array(paths.length);
    let next = 0;
    let done = 0;
    const worker = async () => {
      while (next < paths.length) {
        const i = next++;
        try {
          const r = await fetch(`assets/${paths[i]}`);
          if (r.ok) blobs[i] = new Uint8Array(await r.arrayBuffer());
        } catch {
          /* laget bakes i motoren */
        }
        done++;
        progress(`Henter teksturer (${done} av ${paths.length}) …`, done / paths.length);
      }
    };
    await Promise.all(Array.from({ length: 6 }, worker));
    const got = paths.map((p, i) => [p, blobs[i]]).filter(([, b]) => b);
    return { manifest, names: got.map(([p]) => p), blobs: got.map(([, b]) => b) };
  } catch {
    return { manifest: undefined, names: [], blobs: [] };
  }
}

async function fetchModel(path) {
  const r = await fetch(`models/${model_url(path)}.gz`);
  if (!r.ok) return undefined;
  let bytes = new Uint8Array(await r.arrayBuffer());
  if (bytes[0] === 0x1f && bytes[1] === 0x8b) {
    const plain = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
    bytes = new Uint8Array(await new Response(plain).arrayBuffer());
  }
  return bytes;
}

// Det redigeringen og prøveturen ikke trenger: byfolket (prøveturen går
// rett ned i nivået; en innbygger uten figur er bare usynlig).
const SKIP_MODELS = ["npcs/"];

async function loadModels() {
  const all = model_paths().map((p) => (SKIP_MODELS.some((s) => p.startsWith(s)) ? null : p));
  const out = new Array(all.length);
  let next = 0;
  let done = 0;
  const worker = async () => {
    while (next < all.length) {
      const i = next++;
      out[i] = all[i] ? await fetchModel(all[i]).catch(() => undefined) : undefined;
      done++;
      progress(`Henter modellene (${done} av ${all.length}) …`, done / all.length);
    }
  };
  await Promise.all(Array.from({ length: 6 }, worker));
  return out;
}

function progress(text, frac) {
  $("loading").textContent = text;
  $("bar").value = Math.round(frac * 100);
}

async function boot() {
  progress("Laster motoren …", 0);
  await init();
  setPalette(JSON.parse(level_palette()));
  buildPalette();
  loadTemplates();
  const assets = await loadAssets();
  const models = await loadModels();
  progress("Bygger byen og nivået …", 1);
  await new Promise((r) => setTimeout(r, 30));
  const first = firstDoc();
  const webgl = params.has("webgl");
  try {
    if (webgl) throw new Error("WebGL2 valgt");
    setEngine(await start_level_editor(canvas, true, assets.manifest, assets.names, assets.blobs, models, first.text));
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    setCanvas(fresh);
    setEngine(await start_level_editor(canvas, false, assets.manifest, assets.names, assets.blobs, models, first.text));
  }
  loadOwnModules();
  window.fristad = { engine, level: send, state: st, setFloor, place: placeModule, select, blank: level_blank, saveModule };
  engine.set_quality(engine.backend() === "Gl" ? "enkel" : "standard");
  fit();
  new ResizeObserver(fit).observe(canvas);
  new ResizeObserver(() => redraw()).observe(plan);
  refresh(JSON.parse(engine.level_map()));
  centerOn();
  bindUi();
  bindMap();
  bind3d();
  sendCut();
  frameFloor();
  macKeys(document.body);
  $("overlay").hidden = true;
  if (first.note) toast(first.note);
  // Første gang: hjelpen med de fire stegene.
  if (!store.get(HELP_SEEN)) {
    store.set(HELP_SEEN, "1");
    showHelp(true);
  }
  requestAnimationFrame(loop);
}

let lastT = performance.now();
let camTick = 0;
function loop(t) {
  const dt = Math.min(0.1, (t - lastT) / 1000);
  lastT = t;
  if (engine && (viewMode !== "map" || playing)) {
    try {
      const events = JSON.parse(engine.frame(dt));
      for (const e of events) if (e.type === "toast" && playing) toast(e.text);
    } catch (e) {
      console.error(e);
    }
    if (!playing && ++camTick % 6 === 0) {
      st.camera = engine.level_camera();
      redraw();
    }
  }
  requestAnimationFrame(loop);
}

const FLY_KEYS = ["KeyW", "KeyA", "KeyS", "KeyD", "Space", "KeyC", "ShiftLeft", "ShiftRight"];
function bindUi() {
  $("undo").onclick = () => send({ cmd: "undo" });
  $("redo").onclick = () => send({ cmd: "redo" });
  $("save").onclick = save;
  $("open").onclick = openDialog;
  $("play").onclick = () => play(false);
  $("play-here").onclick = () => play(true);
  $("ready").onclick = () => {
    const head = docHead();
    head.status = "klar";
    if (send({ op: "set_head", head })) toast("Merket klart. Lagre, så tar Claude det inn i spillet.");
  };
  $("floor").onchange = () => {
    const v = $("floor").value;
    if (v.startsWith("new:")) {
      const y = Number(v.slice(4));
      st.extraFloors.push(y);
      $("floor").dataset.list = "";
      refresh(st.map);
      setFloor(y);
      toast(`Etasjen ${y > 0 ? "+" : ""}${y} m: sett ut moduler her, og koble den til med en trapp, stige eller heis. Etasjen under står gjennomsiktig.`, 6000);
    } else setFloor(Number(v));
  };
  $("cut").onclick = () => {
    st.cut = !st.cut;
    $("cut").setAttribute("aria-pressed", st.cut);
    sendCut();
    redraw();
  };
  $("helpbtn").onclick = () => showHelp(true);
  $("help-close").onclick = () => showHelp(false);
  $("help").onclick = (e) => e.target === $("help") && showHelp(false);
  $("v-map").onclick = () => setView("map");
  $("v-split").onclick = () => setView("split");
  $("v-3d").onclick = () => setView("3d");
  $("light").onclick = () => {
    // Knappen viser lyset som er på: arbeidslyset (alt synes) eller
    // spillets (faklene og lysene som følger rommene, tåka).
    const on = $("light").dataset.mode === "game";
    $("light").dataset.mode = on ? "work" : "game";
    $("light").textContent = on ? "Lys: arbeid" : "Lys: spillet";
    engine.level(JSON.stringify({ cmd: "light", work: on }));
  };
  addEventListener("keydown", onKey);
  addEventListener("keyup", (e) => {
    if (playing || (document.activeElement === canvas && FLY_KEYS.includes(e.code))) engine.key(e.code, false);
  });
  addEventListener("blur", () => engine?.release_all());
}

// På Mac står ⌘ der det står Ctrl (begge virker), og ⌥ for Alt.
const MAC = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

function macKeys(root) {
  if (!MAC) return;
  const walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let n = walk.nextNode(); n; n = walk.nextNode()) n.nodeValue = n.nodeValue.replace(/Ctrl\+/g, "⌘").replace(/\bAlt\b/g, "⌥ Alt");
  root.querySelectorAll("[title]").forEach((el) => (el.title = el.title.replace(/Ctrl\+/g, "⌘").replace(/\bAlt\b/g, "⌥ Alt")));
}

function showHelp(on) {
  $("help").hidden = !on;
  if (on) $("help-close").focus();
}

function onKey(e) {
  if (playing) {
    if (e.code === "Escape") {
      e.preventDefault();
      return back();
    }
    engine.key(e.code, true);
    return;
  }
  const ctrl = e.ctrlKey || e.metaKey;
  if (!$("help").hidden) {
    if (e.code === "Escape" || e.code === "F1" || e.key === "?") {
      e.preventDefault();
      showHelp(false);
    }
    return;
  }
  if (e.code === "F1" || (e.key === "?" && !typing())) {
    e.preventDefault();
    return showHelp(true);
  }
  if (ctrl && e.code === "KeyS") {
    e.preventDefault();
    return save();
  }
  if (typing()) return;
  if (document.activeElement === canvas && FLY_KEYS.includes(e.code) && !ctrl) {
    e.preventDefault();
    engine.key(e.code, true);
    return;
  }
  if (ctrl && e.code === "KeyZ") {
    e.preventDefault();
    return send({ cmd: e.shiftKey ? "redo" : "undo" });
  }
  if (ctrl && e.code === "KeyY") {
    e.preventDefault();
    return send({ cmd: "redo" });
  }
  if (ctrl && e.code === "KeyG") {
    e.preventDefault();
    return e.shiftKey ? saveModule() : saveTemplate();
  }
  const digit = /^Digit([1-9])$/.exec(e.code)?.[1];
  if (digit && (ctrl || e.shiftKey)) {
    e.preventDefault();
    return markCamera(digit);
  }
  if (digit && !e.altKey) return gotoMark(digit);
  if (ctrl && e.code === "KeyA") {
    // Alt i etasjen: modulene og innholdet ditt.
    e.preventDefault();
    const on = (y) => y >= st.floor - 0.5 && y < st.floor + 2.5;
    return select([...st.map.modules.filter((m) => m.cells.some((c) => on(c[1]))).map((m) => m.id), ...st.map.things.filter((t) => t.own && on(t.pos[1])).map((t) => t.id)]);
  }
  if (ctrl && e.code === "KeyC") return copy();
  if (ctrl && e.code === "KeyV") return paste();
  if (ctrl && e.code === "KeyD") {
    e.preventDefault();
    copy();
    return paste();
  }
  switch (e.code) {
    case "Escape":
      $("opmenu").hidden = true;
      disarm();
      select([]);
      break;
    case "KeyR":
      if (arm?.type === "module" || arm?.type === "template") {
        // Fra dreiningen magneten valgte; etter R dreies den ikke av seg selv.
        arm.rot = ((st.ghost?.rot ?? arm.rot) + (e.shiftKey ? 3 : 1)) & 3;
        arm.turned = true;
        if (st.mouse) st.ghost = arm.type === "module" ? ghostFor(...st.view.toWorld(...st.mouse)) : templateGhost(...st.view.toWorld(...st.mouse));
        redraw();
      } else if (arm?.type === "thing") arm.yaw = (arm.yaw + 90) % 360;
      else if (st.sel.length) turnSelection(e.shiftKey);
      break;
    case "KeyM":
    case "KeyX":
      if (arm?.type === "module" || arm?.type === "template") {
        arm.mirror = !arm.mirror;
        if (st.mouse) st.ghost = arm.type === "module" ? ghostFor(...st.view.toWorld(...st.mouse)) : templateGhost(...st.view.toWorld(...st.mouse));
        redraw();
      } else if (st.sel.length) send({ op: "mirror", ids: st.sel });
      break;
    case "Delete":
    case "Backspace":
      if (st.sel.length) send({ op: "delete", ids: st.sel });
      break;
    case "KeyK":
      startLink();
      break;
    case "KeyL":
      $("light").click();
      break;
    case "KeyF":
      focusSelection();
      break;
    case "KeyS":
      if (!ctrl) $("cut").click();
      break;
    case "KeyP":
      play(e.shiftKey);
      break;
    // Etasjen: PageUp/PageDown, eller Shift+↑/↓ (tastaturer uten dem).
    case "PageUp":
    case "PageDown":
    case "ArrowUp":
    case "ArrowDown": {
      if (e.code.startsWith("Arrow") && !e.shiftKey) break;
      e.preventDefault();
      const f = st.floors || st.map.floors;
      const i = f.indexOf(st.floor) + (e.code === "PageUp" || e.code === "ArrowUp" ? 1 : -1);
      if (i >= 0 && i < f.length) setFloor(f[i]);
      break;
    }
    case "Tab":
      e.preventDefault();
      setView({ split: "map", map: "3d", "3d": "split" }[viewMode]);
      break;
    // Hele etasjen: Home eller H.
    case "Home":
    case "KeyH":
      centerOn();
      frameFloor();
      break;
  }
}

function play(here) {
  if (playing) return;
  setStatus("Bygger nivået …");
  $("overlay").replaceChildren(Object.assign(document.createElement("div"), { className: "card", innerHTML: "<p class='lt-paragraph'>Bygger verden med nivået …</p>" }));
  $("overlay").hidden = false;
  setTimeout(() => {
    const ok = engine.level_play(here);
    $("overlay").hidden = true;
    setStatus("");
    if (!ok) return toast("Prøveturen kunne ikke starte.");
    setPlaying(true);
    document.body.classList.add("playing");
    $("playbar").hidden = false;
    setView("3d");
    canvas.focus();
    canvas.requestPointerLock?.();
  }, 30);
}

function back() {
  setPlaying(false);
  document.exitPointerLock?.();
  engine.level_back();
  document.body.classList.remove("playing");
  $("playbar").hidden = true;
  setView("split");
  refresh(JSON.parse(engine.level_map()));
}

document.addEventListener("pointerlockchange", () => {
  if (playing && !document.pointerLockElement) setStatus("Klikk i bildet for å fange musa, Esc for å gå tilbake.");
});

// Panelet viser utvalget, bruddene og tallene hver gang dokumentet endres.
onShow({ issues: showIssues, selection: showSelection, stats: showStats });

boot().catch((e) => {
  console.error(e);
  progress(`Noe gikk galt: ${e.message || e}`, 0);
});
