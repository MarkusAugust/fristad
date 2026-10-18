// Sjekker at liv-, energi- og lysstolpene nederst, panelene til høyre
// (minikartet, fienden du sikter på og tastepanelet, åpent) og
// bildefrekvensen ikke overlapper hverandre, og at de står innenfor kantene,
// i åtte skjermstørrelser fra 2560 til 390 px bred (uten motoren, bare HUD-en):
//   node tools/layout-test.mjs KATALOG [UT]   (skjermbilder UT-BREDDE.png)
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join } from "node:path";
const [dir = "dist", out = ""] = process.argv.slice(2);
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };
const server = createServer(async (req, res) => {
  const p = join(dir, decodeURIComponent(req.url.split("?")[0]).replace(/\/$/, "/index.html"));
  // Les fila først: feiler lesingen etter writeHead, krasjer serveren.
  let body;
  try { body = await readFile(p); } catch { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": types[extname(p)] ?? "application/octet-stream" }); res.end(body);
});
await new Promise((r) => server.listen(0, r));
const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", headless: true });
const overlap = (a, b) => a && b && !(a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y);
let bad = 0;
for (const [w, h] of [[2560, 1440], [1920, 1080], [1366, 768], [1280, 720], [1024, 640], [800, 600], [600, 800], [390, 844]]) {
  const page = await browser.newPage({ viewport: { width: w, height: h } });
  await page.route("**/*.wasm", (r) => r.abort());
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.evaluate(() => {
    for (const id of ["hud", "minimap", "foe", "light-bar", "fight-keys"]) document.getElementById(id).hidden = false;
    document.getElementById("gate").hidden = true;
    document.getElementById("keys-panel").open = true;
    document.getElementById("foe-info").textContent = "Kongens rammeverk, levende, med seglet og krona. Gorvek delte det, og nå har det funnet delene sine igjen. Det kan ikke dø, men det kan deles.";
  });
  await page.waitForTimeout(200);
  const box = (s) => page.locator(s).boundingBox();
  // Liv, energi og lys.
  const bars = [await box("#bars .bar:nth-child(1)"), await box("#bars .bar:nth-child(2)"), await box("#bars .bar:nth-child(3)")];
  const side = { kart: await box("#minimap"), fiende: await box("#foe"), taster: await box("#keys-panel") };
  const perf = await box("#perf");
  const hits = [];
  for (const b of bars) for (const [n, k] of Object.entries(side)) if (overlap(b, k)) hits.push(`stolpe/${n}`);
  for (const b of bars) if (overlap(b, perf)) hits.push("stolpe/perf");
  for (const [n, k] of Object.entries(side)) if (overlap(k, perf)) hits.push(`${n}/perf`);
  const names = Object.keys(side);
  for (let i = 0; i < names.length; i++) for (let j = i + 1; j < names.length; j++) if (overlap(side[names[i]], side[names[j]])) hits.push(`${names[i]}/${names[j]}`);
  const off = [...bars, ...Object.values(side)].some((b) => b.x < 0 || b.x + b.width > w || b.y < 0 || b.y + b.height > h);
  console.log(`${w}x${h}: ${hits.length || off ? "FEIL " + [...new Set(hits)].join(", ") + (off ? " utenfor" : "") : "ok"}`);
  if (hits.length || off) bad++;
  if (out) await page.screenshot({ path: `${out}-${w}.png` });
  await page.close();
}
await browser.close(); server.close(); process.exit(bad ? 1 : 0);
