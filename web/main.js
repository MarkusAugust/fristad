// JavaScript-laget i Fristad: oppstarten, løkka, hendelsene fra motoren,
// input, pekerlås og pause. Byen, fysikken og grafikken bor i Rust
// (pkg/fristad.js og .wasm), bak den drivende porten. Det modulene deler,
// står i shared.js; lastingen i loading.js; grafikkvalget i graphics.js og
// dialogene for det i settings.js; det som står over bildet i hud.js;
// menyene i menus.js, inventory.js, shop.js og workshop.js; boka i book.js,
// kartet i map.js og målingen i bench.js.

import init, { start } from "./pkg/fristad.js";
import { createMusic } from "./music.js";
import { createAmbience } from "./ambience.js";
import { createFilm } from "./film/player.js";
import { createEditor } from "./editor.js";
import { defineLtToast } from "./lotek/components/standalone/toast/lt-toast.js";
import { defineLtTabs } from "./lotek/components/frame/tabs/lt-tabs.js";
import { defineLtDialog } from "./lotek/components/frame/dialog/lt-dialog.js";
import { defineLtConnectionStatus } from "./lotek/components/standalone/connection-status/lt-connection-status.js";
import { closePanel, openPanel, panelOpen, readDoc, setPanelOpen } from "./book.js";
import { invOpen, renderInventory, toggleInventory } from "./inventory.js";
import { drawMaps, setMap, setWhere, toggleBigMap } from "./map.js";
import { closeOverlay, overlayOpen } from "./menus.js";
import { $, beginBtn, canvas, clickHint, dragging, engine, enterBtn, gateEl, hud, lastHud, locked, onEnter, panel, playing } from "./shared.js";
import { EDIT, NIVAA, SLAA, setCanvas, setDocs, setDragging, setEngine, setLastHud, setPlaying, sfx, status, toast } from "./shared.js";
import { openMead, openShop, updateShop } from "./shop.js";
import { openWorkshop } from "./workshop.js";
import { benchRunning, countFrame, runBench } from "./bench.js";
import { adapt, fit, QUALITIES, quality, setGraphics, setQuality } from "./graphics.js";
import { fade, hurtArc, lootOpen, openLoot, openLore, setBars, setFoe, setGoal, setLootMore, toggleJournal, updateBubbles, updateMarks, warp } from "./hud.js";
import { checkGpu, fetchLazy, fetchLevel, gpuLost, gpuName, lazyManifest, loadAssets, loadModels, loseGpu, softwareGpu, stage, watchGpuLoss } from "./loading.js";
import { gamma, graphicsOpen, mouseSens, openGraphics, openSettings, settingsOpen } from "./settings.js";

defineLtToast();
// Boka viser eksempelet fra dokumentasjonen levende, så komponentene med
// oppførsel må være registrert, som i LO-TEK-docs.
defineLtTabs();
defineLtDialog();
defineLtConnectionStatus();

let editor = null;
let editorStarted = false;
// Fristad er laget for datamaskin med mus og tastatur; mobil og nettbrett er
// ikke et mål (eieren 19. oktober). Der lastes ingenting, og startkortet ber
// deg prøve på en datamaskin (`?mobil` laster likevel).
function onPhone() {
  const touchOnly = matchMedia("(pointer: coarse)").matches && !matchMedia("(pointer: fine)").matches;
  const ua = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);
  return (touchOnly || ua) && !new URLSearchParams(location.search).has("mobil");
}

async function boot() {
  if (onPhone()) {
    status.hidden = true;
    $("loading").hidden = true;
    $("touch-note").hidden = false;
    return;
  }
  fit();
  stage("Laster motoren …");
  await init();
  try {
    setDocs(await (await fetch("docs.json")).json());
  } catch {
    setDocs({});
  }
  // Teksturlagene: egne bilder og fotoskannede CC0-materialer, samlet av
  // tools/fetch-assets.ts under byggingen. Mangler de, bakes lagene i motoren.
  const assets = await loadAssets();
  // Modellene fra Blender ligger ved siden av motoren, gzip-pakket.
  const models = await loadModels();
  stage("Bygger byen og baker materialene …");
  // Gi nettleseren ett bilde til å tegne teksten før wasm tar tråden.
  await new Promise((r) => setTimeout(r, 30));
  watchGpuLoss();
  // `?webgl` i adressen tvinger WebGL2, til testing.
  const forceGl = new URLSearchParams(location.search).has("webgl");
  try {
    if (forceGl) throw new Error("WebGL2 valgt i adressen");
    setEngine(await start(canvas, true, assets.manifest, assets.names, assets.blobs, models, EDIT, NIVAA, SLAA, lazyManifest));
  } catch (e) {
    console.warn("WebGPU feilet, prøver WebGL2:", e);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    setCanvas(fresh);
    fit();
    setEngine(await start(canvas, false, assets.manifest, assets.names, assets.blobs, models, EDIT, NIVAA, SLAA, lazyManifest));
  }
  console.info(`Fristad kjører på ${engine.backend()}, LO-TEK ${engine.lotek_version()}`);
  if (gpuLost !== null) loseGpu(gpuLost);
  engine.set_gamma(gamma);
  // Med `?nivaa=`: rett ned i nivået.
  if (NIVAA) engine.descend();
  // Til feilsøking i konsollen og til testene.
  window.fristad = { engine, openPanel, closePanel, openShop, readDoc, setGraphics, setQuality, bench: runBench };
  // Uten eget valg får WebGL2 enkel grafikk og WebGPU standard.
  setQuality(quality ?? (engine.backend() === "Gl" ? "enkel" : "standard"), false, false);
  if (EDIT && engine.can_edit()) {
    editor = createEditor({
      engine,
      canvas,
      toast,
      // P: musa fanges med en gang (mens tastetrykket teller), og når
      // spillet er bygget på nytt, settes grafikknivået igjen.
      onLock: () => lock(),
      onPlay: () => setQuality(quality ?? (engine.backend() === "Gl" ? "enkel" : "standard"), false, false),
    });
    window.fristad.editor = editor;
  }
  $("version").textContent = engine.lotek_version();
  $("weather").textContent = engine.weather();
  new ResizeObserver(fit).observe(canvas);
  fit();
  bindInput();
  requestAnimationFrame(loop);

  status.hidden = true;
  $("loading").hidden = true;
  // Når alt er lastet, velger du: se introfilmen (som toner over i byen ved
  // sørporten) eller gå rett til spillet (eieren 18. oktober). Verden står
  // ikke bak kortet før du har valgt (`.gate-first`).
  if (FILM) {
    beginBtn.addEventListener("click", startFilm, { once: true });
    beginBtn.hidden = false;
    enterBtn.textContent = "Gå rett til spillet";
    enterBtn.dataset.variant = "secondary";
    enterBtn.hidden = false;
    beginBtn.focus();
  } else {
    enterBtn.hidden = false;
    enterBtn.focus();
  }
  checkGpu();
  if (new URLSearchParams(location.search).has("bench")) runBench();
}

let gpuToastShown = false;
let last = performance.now();
let frames = 0;
let perfAt = performance.now();
function loop(now) {
  const dt = Math.min((now - last) / 1000, 0.1);
  last = now;
  // Åpningsfilmen er sitt eget malte bilde over spillet. Mens den dekker
  // hele skjermen, tegnes ikke spillet; når den toner bort til slutt, står
  // du ved sørporten bak den.
  if (film) {
    const cover = film.tick(dt);
    if (cover < 0.999 && !filmReveal) {
      filmReveal = true;
      if (!filmReplay) {
        engine.set_theme(false);
        engine.respawn();
      }
    }
    if (cover >= 0.999) {
      requestAnimationFrame(loop);
      return;
    }
  }
  updateLook();
  // Trubaduren i Vertshuset: munnen følger sangen der den er (0 når den ikke spiller).
  engine.set_voice?.(music.voice());
  const events = JSON.parse(engine.frame(dt));
  // En feil i én hendelse skal aldri stoppe løkka (og fryse spillet): den
  // logges, og resten av hendelsene og neste bilde går som før.
  for (const e of events) {
    try {
      handle(e);
    } catch (err) {
      console.error(`Hendelsen ${e?.type ?? "?"} feilet:`, err);
    }
  }
  // Bilder per sekund, så eieren kan si hvordan det flyter.
  frames++;
  countFrame();
  if (now - perfAt > 1000) {
    const fps = Math.round((frames * 1000) / (now - perfAt));
    if (!benchRunning) adapt(fps);
    const tris = Math.round(engine.triangles() / 1000);
    const res = `${canvas.width}×${canvas.height}`;
    // Tiden på grafikkortet per del (bare der nettleseren kan måle den):
    // summen i telleren, delene når du holder musa over.
    const parts = engine.gpu_times ? engine.gpu_times() : "";
    // (Programvaregrafikk kan gi bare nuller; da vises ingenting.)
    const gpuMs = parts ? parts.split(" · ").reduce((sum, p) => sum + parseFloat(p.split(" ").pop()), 0) : 0;
    // Delene står rett i telleren (musa er fanget, så ingen kan holde den over).
    const partText = parts ? parts.replace(/skygge/i, "sk").replace(/verden/i, "v").replace(/etterbehandling/i, "e").replace(/ ms/g, "") : "";
    const api = `${engine.backend() === "Gl" ? "WebGL2" : "WebGPU"}${softwareGpu ? " · uten grafikkort" : ""}`;
    $("perf").textContent = `${fps} fps · ${api} · ${res} · ${tris}k △${gpuMs > 0 ? ` · GPU ${gpuMs.toFixed(1)} ms (${partText})` : ""}`;
    $("perf").title = `Bilder per sekund, grafikk-API og oppløsning${gpuName ? `. Grafikk: ${gpuName}` : ""}${gpuMs > 0 ? `. GPU-tid (ms): ${parts}` : ""}`;
    frames = 0;
    perfAt = now;
  }
  requestAnimationFrame(loop);
}

// Et lite klikk når du trykker på en knapp i dialogene og panelet.
document.addEventListener("click", (ev) => {
  if (ev.target.closest?.("dialog button, .lt-button, #panel button")) sfx.play("click");
});

function handle(e) {
  switch (e.type) {
    case "prompt":
      if (e.label) {
        $("prompt-name").textContent = e.label;
        $("prompt-info").textContent = e.info || "";
        $("prompt-info").hidden = !e.info;
        $("key-use").hidden = !e.verb;
        $("key-use-text").textContent = e.verb || "";
        $("key-trade").hidden = !e.trade;
        $("key-trade-text").textContent = e.trade || "";
        $("target").hidden = false;
      } else {
        $("target").hidden = true;
      }
      break;
    case "fade":
      fade();
      break;
    case "fetch_level":
      fetchLevel(e.level);
      break;
    case "fetch_models":
      fetchLazy(e.names, e.paths);
      break;
    case "warp":
      warp();
      break;
    case "hurt":
      hurtArc(e.angle);
      break;
    case "map":
      setMap(e.map);
      break;
    case "where":
      setWhere(e);
      drawMaps();
      break;
    case "theme":
      document.documentElement.dataset.theme = e.theme;
      music.night(e.theme === "dark");
      break;
    case "toast":
      // Lappene i krypten er opptil rundt 400 tegn: la varselet stå lenge nok
      // til å lese dem (rundt 15 tegn i sekundet, minst seks sekunder).
      toast?.show?.(e.text, { duration: Math.max(6000, e.text.length * 70) });
      break;
    case "goal":
      setGoal(e);
      break;
    case "lore":
      openLore(e.title, e.text);
      break;
    case "foe":
      setFoe(e.foe);
      break;
    case "weather":
      $("weather").textContent = e.weather;
      break;
    case "hud":
      // Pusten går når utholdenheten er brukt opp, og hjertet slår når
      // livet er lavt.
      if (playing && e.stamina === 0 && lastHud && lastHud.stamina > 0) sfx.play("breath");
      if (playing) sfx.heart(e.hp);
      setLastHud(e);
      if (playing) music.set(e.cellar ? "cellar" : "town");
      music.hud(e);
      setBars(e.hp, e.stamina);
      // Lykta: stolpen står alltid (i byen ser du hva du må fylle hos
      // Apotekeren; oljen brenner bare nede), og mørket lukker seg nede når
      // oljen går tom.
      $("light-bar").hidden = false;
      $("light").value = e.light;
      $("light-text").textContent = `${e.light} %`;
      $("light-bar").title = e.cellar ? "Oljen i lykta brenner" : "Lykta brenner bare i krypten. Olje får du hos Apotekeren.";
      document.documentElement.style.setProperty("--dark", e.cellar ? String(Math.max(0, 1 - e.light / 25)) : "0");
      $("tokens").textContent = `${e.tokens} tokens`;
      if (overlayOpen) updateShop();
      $("keys").textContent = e.keys === 1 ? "1 nøkkel" : `${e.keys} nøkler`;
      $("keys").hidden = !e.keys;
      $("boss").hidden = !e.boss;
      if (e.boss) {
        $("boss-name").textContent = e.boss.name;
        $("boss-hp").value = e.boss.hp;
      }
      // Liv og tokens vises nede i kjelleren, og oppe når du har noe.
      $("vitals").hidden = !(e.cellar || e.tokens > 0);
      $("fight-keys").hidden = !e.cellar;
      if (!e.cellar) setFoe(null);
      break;
    case "sound":
      sfx.play(e.name, e);
      break;
    case "marks":
      updateMarks(e.marks);
      break;
    case "speech":
      updateBubbles(e.bubbles);
      break;
    case "inventory":
      renderInventory(e);
      setBars(lastHud.hp, lastHud.stamina);
      try {
        localStorage.setItem("fristad.save", engine.save_state());
      } catch {
        /* lagring er en bekvemmelighet */
      }
      break;
    case "haul":
      setLootMore(e.components ?? []);
      break;
    case "loot":
      openLoot(e.component);
      break;
    case "edit":
      editor?.update(e.edit);
      break;
    case "open_doc":
      // F på en fylt ramme på galleriveggen: boka i panelet til høyre.
      readDoc(e.component);
      break;
    case "ambience":
      lastAmbience = e;
      if (playing) {
        ambience.set(e);
        sfx.ambience(e);
        music.ambience(e);
      }
      break;
    case "shop":
      openShop(e.npc, e.wares, e.swaps ?? []);
      break;
    case "mead":
      openMead(e);
      break;
    case "song":
      music.song(e.on, () => engine.song_over?.());
      break;
    case "workshop":
      openWorkshop(e.recipes);
      break;
  }
}

// Musikken (web/music.js): sporene fra filer (byen etter døgnet, krypten
// etter temaet, kampen når en fiende er nær), ellers partituret.
const music = createMusic(() => sfx.context());
// Lydene i byen (web/ambience.js) etter hvor du står.
const ambience = createAmbience(() => sfx.context(), sfx);
let lastAmbience = null;

function lock() {
  // Avslår nettleseren (uten brukerhandling, eller uten rå bevegelse), er
  // det ikke en feil: et klikk i bildet låser igjen.
  try {
    const p = canvas.requestPointerLock?.({ unadjustedMovement: true });
    if (p?.catch) {
      p.catch(() => {
        try {
          canvas.requestPointerLock?.()?.catch?.(() => {});
        } catch {
          /* ingen pekerlås */
        }
      });
    }
  } catch {
    /* ingen pekerlås */
  }
}

let loadedSave = false;
let soundsLoading = false;
function enter(relock = true) {
  pausedMenu = false;
  gateEl.classList.remove("gate-first");
  beginBtn.hidden = true;
  delete enterBtn.dataset.variant;
  if (!loadedSave) {
    loadedSave = true;
    try {
      const saved = localStorage.getItem("fristad.save");
      if (saved) engine.load_state(saved);
    } catch {
      /* ingen lagring */
    }
  }
  if (soundsLoading && !panelOpen) sfx.play("resume");
  setPlaying(true);
  if (softwareGpu && !gpuToastShown) {
    gpuToastShown = true;
    toast?.show?.("Nettleseren tegner uten grafikkortet, så spillet går tregt. Se startkortet (Esc) for hvordan du slår på maskinvareakselerasjon.");
  }
  if (!soundsLoading) {
    soundsLoading = true;
    sfx.preload();
  }
  music.set(lastHud.cellar ? "cellar" : "town");
  if (lastAmbience) {
    ambience.set(lastAmbience);
    sfx.ambience(lastAmbience);
    music.ambience(lastAmbience);
  }
  gateEl.hidden = true;
  hud.hidden = false;
  engine.set_paused(false);
  canvas.focus();
  fadeHelp();
  // Med `?edit` begynner du i redigeringen, med fri mus.
  if (editor && !editorStarted) {
    editorStarted = true;
    if (editor.begin()) return;
  }
  if (relock && !editor?.active) lock();
  setTimeout(updateLook, 400);
}

// «Klikk for å se deg rundt med musa»: står midt i bildet så lenge du spiller
// uten fanget mus (etter filmen kan ikke nettleseren fange den uten et klikk).
// Sa nettleseren nei (`pointerlockerror`), spilles det uten: dra for å se.
let noLock = false;
const lookEl = $("look");
function updateLook() {
  const hide = !(playing && !locked() && !noLock && !editor?.active && !panelOpen && !overlayOpen);
  if (lookEl.hidden !== hide) lookEl.hidden = hide;
}

// Knappene som bare hører til pausemenyen. Startkortet viser bare framdriften
// mens siden laster (eieren 18. oktober: knappene sto der uten å virke).
const PAUSE_ONLY = ["respawn", "replay", "quit", "graphics", "settings"];
// Pausemenyen står (Esc i spillet): Esc igjen eller et klikk utenfor kortet
// tar deg tilbake i spillet (eieren 19. oktober).
let pausedMenu = false;
let pausedAt = 0;

function quit() {
  pausedMenu = false;
  setPlaying(false);
  setDragging(false);
  setPanelOpen(false);
  panel.hidden = true;
  engine.set_paused(true);
  engine.respawn();
  if (locked()) document.exitPointerLock?.();
  enterBtn.textContent = "Gå inn i Fristad";
  for (const id of PAUSE_ONLY) $(id).hidden = true;
  hud.hidden = true;
  gateEl.hidden = false;
  enterBtn.focus();
}

function pause() {
  if (playing) sfx.play("pause");
  setPlaying(false);
  setDragging(false);
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  if (!panelOpen) {
    pausedMenu = true;
    pausedAt = performance.now();
    enterBtn.textContent = "Fortsett";
    for (const id of PAUSE_ONLY) $(id).hidden = id === "replay" && !FILM;
    gateEl.hidden = false;
    enterBtn.focus();
  }
}

// Åpningsfilmen (den ferdige videoen film/intro.mp4, klippet av
// tools/film/cut.py; manuset i docs/film/manus.md, kanon i docs/kanon.md)
// spilles hver gang med ett klikk («Klikk for å begynne», som også gir lyd)
// når byen er bygd, og toner over i spillet ved sørporten. `?nofilm` hopper
// over den.
const FILM = !EDIT && !new URLSearchParams(location.search).has("nofilm");
let film = null;
let filmReveal = false;
// Filmen sett på nytt fra pausemenyen: du blir stående der du var, i den
// tida på døgnet det var, og spillet står stille under den.
let filmReplay = false;

async function startFilm(replay = false) {
  filmReplay = replay === true;
  pausedMenu = false;
  beginBtn.hidden = true;
  gateEl.classList.remove("gate-first");
  gateEl.hidden = true;
  hud.hidden = true;
  if (!soundsLoading) {
    soundsLoading = true;
    sfx.preload();
  }
  music.set("silence");
  if (!filmReplay) {
    engine.set_theme(false);
    engine.set_paused(false);
  }
  filmReveal = false;
  try {
    film = createFilm();
  } catch (e) {
    console.warn("Filmen kunne ikke vises:", e);
    enter();
    return;
  }
  const { skipped, gesture } = await film.done;
  film = null;
  // Hoppet over: rett til sørporten om dagen (ikke når den ble sett på nytt).
  if (!filmReplay && (skipped || !filmReveal)) {
    engine.set_theme(false);
    engine.respawn();
  }
  // Hoppet du over med et klikk eller mellomrom, kan musa fanges med en gang;
  // ellers står «Klikk for å se deg rundt» midt i bildet.
  enter(!!gesture);
}

function bindInput() {
  enterBtn.addEventListener("click", () => enter());
  // Sitter du fast, tar knappen i pausemenyen (og H) deg tilbake til sørporten.
  $("respawn").addEventListener("click", () => {
    engine.respawn();
    enter();
  });
  // Avslutt: tilbake til startskjermen, med spilleren ved sørporten igjen.
  $("quit").addEventListener("click", quit);
  // Filmen på nytt fra pausemenyen.
  $("replay").addEventListener("click", () => startFilm(true));
  // Et klikk utenfor kortet i pausemenyen er det samme som «Fortsett».
  gateEl.addEventListener("click", (e) => {
    if (pausedMenu && e.target === gateEl) enter();
  });
  canvas.addEventListener("click", () => {
    if (playing && !locked() && !editor?.active) lock();
  });
  $("look").addEventListener("click", () => {
    if (playing && !locked()) lock();
  });

  document.addEventListener("pointerlockchange", () => {
    updateLook();
    // Slippes låsen mens du spiller (Esc), er det pause.
    if (!locked() && playing && !panelOpen && !editor?.active) pause();
  });
  // Ga ikke nettleseren oss musa, går det an å spille uten: dra for å se,
  // klikk for å hugge.
  document.addEventListener("pointerlockerror", () => {
    noLock = true;
    updateLook();
    if (playing) clickHint();
  });

  // Musetastene er hugg og blokk. Når musa er fanget, går hvert trykk rett
  // til spillet. Uten pekerlås (noen nettlesere og jobb-PC-er gir den ikke)
  // drar venstre tast kameraet, og et kort klikk uten å dra er et hugg;
  // høyre tast blokker uansett. Trykk i HUD-en over bildet teller òg, men
  // ikke i dialogene.
  let dragDist = 0;
  // Uten pekerlås: står venstre tast stille i `HOLD_MS`, er det et hugg som
  // holdes (det tunge hugget lades til tasten slippes); drar du først, er
  // det å se deg rundt.
  const HOLD_MS = 140;
  let holdTimer = 0;
  let holding = false;
  const inGame = (t) => t === canvas || (t instanceof Element && !t.closest("dialog, #overlay, #panel, #gate, button, a, input, details"));
  document.addEventListener("mousedown", (e) => {
    if (playing && editor?.mouseDown(e)) return;
    if (!playing || !inGame(e.target)) return;
    if (locked()) {
      e.preventDefault();
      engine.mouse(e.button, true);
    } else if (e.button === 0) {
      setDragging(true);
      dragDist = 0;
      clearTimeout(holdTimer);
      holdTimer = setTimeout(() => {
        if (dragging && dragDist < 6 && playing) {
          holding = true;
          engine.mouse(0, true);
        }
      }, HOLD_MS);
    } else {
      e.preventDefault();
      engine.mouse(e.button, true);
    }
  });
  window.addEventListener("mouseup", (e) => {
    if (editor?.mouseUp(e)) return;
    const wasDrag = dragging;
    setDragging(false);
    clearTimeout(holdTimer);
    if (e.button === 0 && holding) {
      holding = false;
      engine.mouse(0, false);
      return;
    }
    if (!playing) return;
    if (!locked() && e.button === 0 && wasDrag && dragDist < 6 && inGame(e.target)) {
      // Et kort klikk uten pekerlås: et hugg.
      engine.mouse(0, true);
      engine.mouse(0, false);
      return;
    }
    engine.mouse(e.button, false);
  });
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener("contextmenu", (e) => {
    if (playing && inGame(e.target)) e.preventDefault();
  });
  document.addEventListener("mousemove", (e) => {
    if (!playing) return;
    if (editor?.mouseMove(e)) return;
    if (dragging) dragDist += Math.abs(e.movementX) + Math.abs(e.movementY);
    if (locked() || dragging) engine.look(e.movementX * mouseSens, e.movementY * mouseSens);
  });

  const keys = new Set(["Space", "Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);
  document.addEventListener("keydown", (e) => {
    // Esc med panelet oppe lukker panelet og tar deg rett tilbake i spillet,
    // uten å be om pekerlås (Esc gir ingen brukerhandling å låse på, og et
    // avslått forsøk åpnet pausemenyen). Klikk i bildet låser igjen; neste
    // Esc åpner menyen.
    // Redigeringen tar sine taster først (og P for å gå inn i den igjen).
    if ((settingsOpen || graphicsOpen) && e.code === "Escape") {
      e.preventDefault();
      (graphicsOpen || settingsOpen).querySelector('[data-act="close"]').click();
      return;
    }
    if (editor && playing && !panelOpen && !invOpen && !overlayOpen && !settingsOpen && editor.key(e)) return;
    if (panelOpen && e.code === "Escape") {
      // Er dialogen i demoen åpen, er det den Esc lukker.
      if ($("demo").shadowRoot?.querySelector("dialog:modal")) return;
      e.preventDefault();
      closePanel("resume");
      return;
    }
    if (lootOpen) return;
    if (e.code === "KeyI" && !e.repeat && (playing || invOpen)) {
      e.preventDefault();
      toggleInventory();
      return;
    }
    if (invOpen) return;
    if (e.code === "KeyM" && !e.repeat && playing) {
      const on = music.toggle();
      toast?.show?.(on ? "Musikken er på." : "Musikken er av.");
      return;
    }
    if (overlayOpen) {
      if (e.code === "Escape") {
        e.preventDefault();
        closeOverlay(true, false);
      } else if (e.code === "KeyE" && !e.repeat) {
        e.preventDefault();
        closeOverlay(true, true);
      }
      return;
    }
    if (e.code === "KeyJ" && !e.repeat && playing) {
      e.preventDefault();
      toggleJournal();
      return;
    }
    // Tastene: F1, eller ? (F-tastene trenger fn på små tastaturer).
    const field = e.composedPath()[0];
    const writing = field instanceof HTMLElement && (field.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(field.tagName));
    if ((e.code === "F1" || (e.key === "?" && !writing)) && !e.repeat) {
      e.preventDefault();
      $("keys-panel").open = !$("keys-panel").open;
      return;
    }
    if (e.code === "KeyK" && !e.repeat && playing) {
      e.preventDefault();
      toggleBigMap();
      return;
    }
    if (e.code === "KeyG" && !e.repeat && !graphicsOpen && (playing || !gateEl.hidden)) {
      setQuality(QUALITIES[(QUALITIES.indexOf(quality) + 1) % QUALITIES.length]);
      return;
    }
    if (e.code === "KeyO" && !e.repeat && (playing || !gateEl.hidden || settingsOpen)) {
      e.preventDefault();
      if (settingsOpen) settingsOpen.querySelector('[data-act="close"]').click();
      else openSettings();
      return;
    }
    if (settingsOpen || graphicsOpen) return;
    // Esc i pausemenyen: tilbake i spillet. Musa fanges ikke med Esc (det er
    // ingen brukerhandling å låse på); «Klikk for å se deg rundt» står da i bildet.
    // (Ikke den samme Esc-en som slapp musa og åpnet menyen.)
    if (pausedMenu && e.code === "Escape" && !e.repeat && performance.now() - pausedAt > 300) {
      e.preventDefault();
      enter(false);
      return;
    }
    if (!playing) return;
    if (e.code === "Escape" && !locked()) {
      pause();
      return;
    }
    if (keys.has(e.code)) e.preventDefault();
    if (e.repeat) return;
    engine.key(e.code, true);
  });
  document.addEventListener("keyup", (e) => {
    if (playing) engine.key(e.code, false);
  });
  window.addEventListener("blur", () => engine.release_all());

  $("close").addEventListener("click", () => closePanel(true));
  $("overlay-close").addEventListener("click", () => closeOverlay(true, true));
  $("graphics").addEventListener("click", () => openGraphics());
  $("settings").addEventListener("click", () => openSettings());
}

// Tastekartet nederst står hele tida, så det alltid går an å se hva tastene gjør.
function fadeHelp() {}

// `?uitest`: tools/ui-test.mjs viser menyene med falske data uten motoren
// (hendelsene går gjennom `handle` som fra Rust, og `engine` er en attrapp).
if (new URLSearchParams(location.search).has("uitest")) {
  window.fristadUi = {
    handle,
    toggleInventory,
    set(o) {
      if (o.engine) setEngine(o.engine);
      if (o.docs) setDocs(o.docs);
      if (o.hud) setLastHud(o.hud);
    },
  };
}

onEnter(enter);
boot().catch((e) => {
  console.error(e);
  status.classList.add("error");
  status.textContent =
    "Fristad trenger WebGPU eller WebGL2, og fikk ikke startet her. Prøv en oppdatert Chrome, Edge, Safari eller Firefox.";
});
