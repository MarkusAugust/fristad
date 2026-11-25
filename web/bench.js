// Målingen (`?bench` i adressen, eller `fristad.bench()` i konsollen): går
// gjennom faste steder og grafikkvalg, måler tiden på grafikkortet per del
// og bildene per sekund, og viser alt som én tekst eieren kan kopiere.
// Bryterne (MSAA, alpha to coverage, solskygge, flat farge) skiller hva som
// koster: flat farge måler bare rasterisering og overtegning.

import { custom, fit, putCustom, quality, setBenchScale, setQuality } from "./graphics.js";
import { gpuName, softwareGpu } from "./loading.js";
import { canvas, engine, gateEl, NIVAA, SLAA, toast } from "./shared.js";

export let benchFrames = 0;
export let benchRunning = false;
const BENCH_PLACES = [
  ["torget", 2, 1.7, 9, 0],
  ["Storgata", 4.7, 1.7, 28, 0],
  ["sørporten", 4.7, 1.7, 50, 0],
];
// I prøvenivået (`?nivaa=test`): stedene i nivåets eget rom (nivå 1).
const BENCH_LEVEL_PLACES = [
  ["kapellet", 8, 1.7, 9.5, 0],
  ["hvelvet", 19, 2.7, 15, -135],
  ["gangen", 20, 1.7, 7.5, 180],
];
// Hver kjøring: nivået, en kort forklaring og bryterne til `engine.set_bench`
// (det som mangler, står som i vanlig spill). `scale` ganger oppløsningen.
const BENCH_RUNS = [
  { q: "standard", what: "som i spillet (SMAA)" },
  { q: "standard", what: "uten kantutjevning", smaa: false },
  { q: "standard", what: "MSAA 4× i stedet for SMAA", msaa: 4 },
  { q: "standard", what: "flat farge", flat: true },
  { q: "standard", what: "halv oppløsning", scale: 0.5 },
  // Grafikkbryterne (`gfx`, som i grafikkdialogen) én om gangen.
  { q: "standard", what: "uten myke skygger", gfx: { soft_shadow: 0 } },
  { q: "standard", what: "uten solstråler", gfx: { rays: 0 } },
  { q: "standard", what: "med skygge i krokene", gfx: { ao: 1 } },
  { q: "standard", what: "med skygge langt unna", gfx: { far_shadow: 1 } },
  { q: "standard", what: "uten malt gradering", gfx: { grade: 0 } },
  { q: "enkel", what: "som i spillet" },
  { q: "avansert", what: "som i spillet (SMAA)" },
  { q: "avansert", what: "MSAA 8× i stedet for SMAA", msaa: 8 },
];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function measure(seconds) {
  const parts = new Map();
  let samples = 0;
  benchFrames = 0;
  const t0 = performance.now();
  while (performance.now() - t0 < seconds * 1000) {
    await sleep(100);
    const text = engine.gpu_times ? engine.gpu_times() : "";
    if (!text) continue;
    samples++;
    for (const p of text.split(" · ")) {
      const k = p.slice(0, p.lastIndexOf(" "));
      parts.set(k, (parts.get(k) ?? 0) + parseFloat(p.split(" ").pop()));
    }
  }
  const fps = (benchFrames * 1000) / (performance.now() - t0);
  const avg = [...parts].map(([k, v]) => [k, v / Math.max(samples, 1)]);
  const total = avg.reduce((a, [, v]) => a + v, 0);
  return { fps, total, avg };
}
export async function runBench() {
  if (benchRunning || !engine) return;
  benchRunning = true;
  // Målingen bruker forhåndsvalgene; de egendefinerte bryterne settes til side.
  const customBefore = custom;
  putCustom(null);
  const before = quality;
  gateEl.hidden = true;
  engine.set_paused(false);
  const lines = [
    `Fristad ${engine.lotek_version()} · ${engine.backend() === "Gl" ? "WebGL2" : "WebGPU"} · ${gpuName || "ukjent grafikk"}${softwareGpu ? " (programvare)" : ""}`,
    `${navigator.userAgent}`,
    `Lerret ${canvas.width}×${canvas.height} · vindu ${innerWidth}×${innerHeight} · dpr ${devicePixelRatio}`,
    "sted · nivå · kjøring → GPU ms (delene) · fps · trekanter og kall i hovedpasset · lerret",
  ];
  toast?.show?.("Måler grafikken … (rundt ett minutt)");
  if (NIVAA) lines.push(`Nivådokumentet «${NIVAA}» ${SLAA ? "med bitene slått sammen per 16 m" : "med bitene som instanser"}`);
  const places = NIVAA ? BENCH_LEVEL_PLACES : BENCH_PLACES;
  for (const [name, x, y, z, yaw] of places) {
    for (const run of BENCH_RUNS) {
      if (name !== places[0][0] && run !== BENCH_RUNS[0] && run.q === "standard") continue;
      setQuality(run.q, false, false);
      const { q, what, scale, gfx, ...flags } = run;
      if (gfx) engine.set_quality(JSON.stringify({ preset: q, ...gfx }));
      engine.set_bench(JSON.stringify(flags));
      setBenchScale(scale ?? 1);
      fit();
      if (NIVAA) engine.place_in_level(1, x, y, z, yaw, 0);
      else engine.place(x, y, z, yaw, 0);
      await sleep(600);
      const m = await measure(2);
      const partText = m.avg.map(([k, v]) => `${k} ${v.toFixed(1)}`).join(" · ");
      const tris = `${Math.round(engine.triangles() / 1000)}k △ · ${engine.calls()} kall`;
      const gpu = m.total > 0 ? `${m.total.toFixed(1)} ms (${partText})` : "GPU-tid ikke tilgjengelig";
      lines.push(`${name} · ${q} · ${what} → ${gpu} · ${m.fps.toFixed(0)} fps · ${tris} · ${canvas.width}×${canvas.height}`);
    }
  }
  engine.set_bench("{}");
  setBenchScale(1);
  fit();
  putCustom(customBefore);
  setQuality(before ?? "standard", false, false);
  benchRunning = false;
  showBench(lines.join("\n"));
}
function showBench(text) {
  console.info(text);
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog settings-dialog";
  dlg.setAttribute("aria-labelledby", "bench-title");
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="bench-title">Målingen</h2>
    <div class="lt-dialog__body">
      <p class="lt-help-text">Kopier teksten og send den til Claude.</p>
      <textarea class="lt-textarea" rows="16" readonly style="width:100%;font-family:'IBM Plex Mono',monospace;font-size:12px"></textarea>
    </div>
    <div class="lt-dialog__footer">
      <button class="lt-button" data-variant="secondary" type="button" data-act="close">Lukk</button>
      <button class="lt-button" data-variant="primary" type="button" data-act="copy">Kopier</button>
    </div>`;
  document.body.appendChild(dlg);
  dlg.querySelector("textarea").value = text;
  dlg.querySelector('[data-act="copy"]').addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(text);
      toast?.show?.("Kopiert.");
    } catch {
      dlg.querySelector("textarea").select();
    }
  });
  dlg.querySelector('[data-act="close"]').addEventListener("click", () => {
    dlg.close();
    dlg.remove();
    gateEl.hidden = false;
  });
  dlg.showModal();
}

// Løkka teller hvert bilde som er tegnet.
export const countFrame = () => {
  benchFrames++;
};
