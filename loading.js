// Lastingen: teksturene og modellene ved start, kryptnivåene og modellene
// som hentes etter hvert, framdriften på startkortet, og grafikkortet
// (hvilket det er, og om det blir borte).

import { asset_paths, lazy_paths, level_paths, model_paths, model_url, start_paths } from "./pkg/fristad.js";
import { quality, setQuality } from "./graphics.js";
import { $, EDIT, engine, status, toast } from "./shared.js";

// Tegner nettleseren uten grafikkortet (maskinvareakselerasjon slått av,
// eller driveren svartelistet), faller den tilbake til programvare som
// SwiftShader eller Microsoft Basic Render Driver, og alt går svært tregt.
// Da sier startkortet fra og forklarer hvordan det slås på; grafikken
// byttes bare om spilleren velger det.
export let gpuName = "";
export let softwareGpu = false;
const SOFTWARE = /swiftshader|llvmpipe|softpipe|lavapipe|basic render driver|software rasterizer|microsoft basic/i;
export async function checkGpu() {
  let name = "";
  let fallback = false;
  try {
    const gl = document.createElement("canvas").getContext("webgl2");
    const info = gl?.getExtension("WEBGL_debug_renderer_info");
    name = (info && gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) || gl?.getParameter(gl.RENDERER) || "";
    gl?.getExtension("WEBGL_lose_context")?.loseContext();
  } catch {
    /* ingen WebGL2 */
  }
  if (engine.backend() !== "Gl" && navigator.gpu) {
    try {
      const adapter = await navigator.gpu.requestAdapter();
      fallback = !!(adapter?.info?.isFallbackAdapter ?? adapter?.isFallbackAdapter);
      const d = adapter?.info?.description || adapter?.info?.device || "";
      if (d) name = d;
      else if (adapter?.info?.vendor && !name) name = adapter.info.vendor;
    } catch {
      /* ingen adapter */
    }
  }
  gpuName = name;
  softwareGpu = fallback || SOFTWARE.test(name);
  console.info(`Grafikk: ${name || "ukjent"}${softwareGpu ? " (programvare, uten grafikkort)" : ""}`);
  if (!softwareGpu) return;
  const short = name.match(/swiftshader|llvmpipe|softpipe|lavapipe|microsoft basic render driver|basic render driver/i)?.[0];
  $("gpu-name").textContent = short ? `tegnes med ${short}` : "tegnes med programvare";
  $("gpu-warning").hidden = false;
  $("gpu-reduced").hidden = quality === "enkel";
  $("gpu-reduced").addEventListener("click", () => {
    setQuality("enkel");
    $("gpu-reduced").hidden = true;
  });
}

export async function loadAssets() {
  const none = { manifest: undefined, names: [], blobs: [] };
  let manifest;
  try {
    // Manifestet hentes alltid ferskt: et gammelt et i hurtigbufferet viser
    // til filer som ikke finnes lenger.
    const res = await fetch("assets/manifest.json", { cache: "no-cache" });
    if (!res.ok) return none;
    manifest = await res.text();
  } catch {
    return none;
  }
  const paths = asset_paths(manifest);
  const bar = $("loading");
  bar.hidden = false;
  let done = 0;
  const tick = () => {
    done++;
    bar.value = Math.round((done / paths.length) * 100);
    setStatus(`Henter teksturer (${done} av ${paths.length}) …`);
  };
  setStatus(`Henter teksturer (0 av ${paths.length}) …`);
  // Seks om gangen, så nettleseren ikke åpner hundre forbindelser. En fil
  // som mangler gjør bare at det laget bakes i motoren, ikke alle.
  const blobs = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      try {
        const r = await fetch(`assets/${paths[i]}`);
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        blobs[i] = new Uint8Array(await r.arrayBuffer());
      } catch (e) {
        console.warn(`Teksturen ${paths[i]} kunne ikke hentes (${e.message}); laget bakes i stedet.`);
      }
      tick();
    }
  };
  await Promise.all(Array.from({ length: 6 }, worker));
  const got = paths.map((p, i) => [p, blobs[i]]).filter(([, b]) => b);
  return { manifest, names: got.map(([p]) => p), blobs: got.map(([, b]) => b) };
}

// Henter én modellfil (models/<sti>.gz, stien fra `model_url`) og pakker
// den ut; `count` får bytene mens de kommer.
async function fetchModel(path, count = () => {}) {
  const r = await fetch(`models/${model_url(path)}.gz`);
  if (!r.ok) {
    console.warn(`Modellen ${path} mangler: HTTP ${r.status}`);
    return undefined;
  }
  const counted = r.body.pipeThrough(
    new TransformStream({
      transform(chunk, ctl) {
        count(chunk.byteLength);
        ctl.enqueue(chunk);
      },
    }),
  );
  let bytes = new Uint8Array(await new Response(counted).arrayBuffer());
  // Pakket ut allerede hvis serveren sendte den med Content-Encoding.
  if (bytes[0] === 0x1f && bytes[1] === 0x8b) {
    const plain = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
    bytes = new Uint8Array(await new Response(plain).arrayBuffer());
  }
  return bytes;
}

// Mister nettleseren grafikkortet (grafikkprosessen startes på nytt, eller
// SwiftShader i Chromium uten skjerm), må motoren slutte å bruke enheten:
// ellers kaster neste createBuffer, og motoren stopper. wgpu 25 kan ikke
// melde det selv (lukningen den gir `device.lost` er sluppet før løftet
// løses), så vi lytter på enheten nettleseren gir motoren.
export let gpuLost = null;
export function watchGpuLoss() {
  const proto = globalThis.GPUAdapter?.prototype;
  if (!proto || proto.requestDevice.fristad) return;
  const request = proto.requestDevice;
  const wrapped = async function (...args) {
    const device = await request.apply(this, args);
    device.lost.then((info) => {
      if (info.reason === "destroyed") return;
      loseGpu(info.message || "ukjent grunn");
    });
    return device;
  };
  wrapped.fristad = true;
  proto.requestDevice = wrapped;
}
export function loseGpu(message) {
  gpuLost = message;
  if (!engine || engine.backend() === "Gl") return;
  engine.gpu_lost(message);
  toast?.show?.("Grafikkortet forsvant. Last siden på nytt for å se byen igjen.");
}

// Det motoren trenger hentet nå (`engine.fetches()`, plan del 10a): bitene
// til et kryptnivå (`level`) eller modeller som hentes etter start
// (`names` og `paths`). Spørres etter hvert bilde; en feil her stopper ikke
// løkka.
export function deliver() {
  const wanted = engine?.fetches();
  if (!wanted) return;
  for (const f of JSON.parse(wanted)) {
    const done = f.level ? fetchLevel(f.level) : fetchLazy(f.names, f.paths);
    done.catch((err) => console.error("Hentingen feilet:", err));
  }
}

// Bitene til et kryptnivå hentes først når økta ber om nivået, og gis
// motoren med `add_level`.
const levelsAsked = new Set();
async function fetchLevel(n) {
  if (levelsAsked.has(n)) return;
  levelsAsked.add(n);
  const paths = level_paths(n);
  const out = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      out[i] = await fetchModel(paths[i]).catch((e) => console.warn(`Modellen ${paths[i]}:`, e));
    }
  };
  await Promise.all(Array.from({ length: 4 }, worker));
  levelsAsked.delete(n);
  engine?.add_level(n, out);
}

// Byens modeller som ikke trengs til første bilde (innsiden, møblene,
// galleriet, fiendene, utstyret du ikke har på; `lazy_paths`, plan del 10a)
// hentes når økta ber om dem og gis motoren med
// `add_models`. Verden bygges før de kommer, fra manifestet
// (models/manifest.json, mål, kollidere og skjelett). Mangler manifestet,
// hentes alt ved start som før.
export let lazyManifest;
async function fetchLazy(names, paths) {
  const out = new Array(paths.length);
  let next = 0;
  const worker = async () => {
    while (next < paths.length) {
      const i = next++;
      out[i] = await fetchModel(paths[i]).catch((e) => console.warn(`Modellen ${paths[i]}:`, e));
    }
  };
  await Promise.all(Array.from({ length: 4 }, worker));
  engine?.add_models(names, out);
}

// Henter modellfilene byen trenger (alle utenom bitene til kryptnivåene og
// det som hentes senere; med ?edit alle) og pakker dem ut. Framdriften
// regnes i byte, fra størrelsene i models/sizes.json.
export async function loadModels() {
  const all = model_paths();
  if (!EDIT) {
    const bytes = await fetchModel("manifest.json").catch(() => undefined);
    if (bytes) lazyManifest = new TextDecoder().decode(bytes);
  }
  const later = new Set(lazyManifest ? lazy_paths() : []);
  const wanted = new Set(EDIT ? all : start_paths().filter((p) => !later.has(p)));
  let sizes = {};
  try {
    sizes = await (await fetch("models/sizes.json")).json();
  } catch {
    sizes = {};
  }
  for (const level of Object.values(sizes.levels || {})) Object.assign(sizes, level);
  const paths = all.filter((p) => wanted.has(p));
  const total = paths.reduce((a, p) => a + (sizes[p] || 1), 0);
  let got = 0;
  const bar = $("loading");
  bar.hidden = false;
  const show = () => {
    bar.value = Math.round((got / total) * 100);
    setStatus(`Henter byen (${(got / 1e6).toFixed(0)} av ${(total / 1e6).toFixed(0)} MB) …`);
  };
  show();
  // Samme rekkefølge som model_paths(); det som ikke hentes, er tomt.
  const out = new Array(all.length);
  let next = 0;
  let last = 0;
  const worker = async () => {
    while (next < paths.length) {
      const path = paths[next++];
      out[all.indexOf(path)] = await fetchModel(path, (n) => {
        got += n;
        if (got - last > 256 * 1024) {
          last = got;
          show();
        }
      });
      show();
    }
  };
  await Promise.all(Array.from({ length: 6 }, worker));
  return out;
}

function setStatus(text) {
  const span = status.querySelector("span:last-child");
  if (span) span.textContent = text;
}

// Et trinn uten kjent framdrift: stolpen vises uten verdi (ubestemt).
export function stage(text) {
  setStatus(text);
  const bar = $("loading");
  bar.hidden = false;
  bar.removeAttribute("value");
}
