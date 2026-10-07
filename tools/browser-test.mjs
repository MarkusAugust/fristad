// Kjører den ferdige nettsiden (dist/ eller grenen web-dist) i Chromium uten
// skjerm, og skriver konsollen og et skjermbilde. Brukes slik:
//   node tools/browser-test.mjs KATALOG [webgpu|webgl] [UT.png] [ventetid ms] [BxH]
// Claude henter nettsiden fra grenen web-dist: `sh tools/pull-dist.sh`.
import { createRequire } from "node:module";
// Playwright ligger globalt i skymiljøet (/opt/node-tools); lokalt holder `npm i -D playwright`.
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join } from "node:path";

const [dir = "dist", mode = "webgl", out = "browser.png", wait = "4000", size = "1280x720"] = process.argv.slice(2);
const [vw, vh] = size.split("x").map(Number);
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg", ".gltf": "model/gltf+json", ".bin": "application/octet-stream" };
const server = createServer(async (req, res) => {
  const path = join(dir, decodeURIComponent(req.url.split("?")[0]).replace(/\/$/, "/index.html"));
  try {
    await stat(path);
    res.writeHead(200, { "content-type": types[extname(path)] ?? "application/octet-stream" });
    res.end(await readFile(path));
  } catch {
    res.writeHead(404);
    res.end();
  }
});
await new Promise((r) => server.listen(0, r));
const port = server.address().port;

const args = mode === "webgpu"
  ? ["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--use-angle=swiftshader", "--use-vulkan=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"]
  : ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"];
const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args, headless: true });
const page = await browser.newPage({ viewport: { width: vw, height: vh } });
page.on("console", (m) => console.log(`[${m.type()}] ${m.text().slice(0, 400)}`));
page.on("pageerror", (e) => console.log(`[pageerror] ${e.message}`));
await page.goto(`http://127.0.0.1:${port}/${mode === "webgl" ? "?webgl" : ""}`, { waitUntil: "load" });
// Vent til motoren er klar (knappen på førstesiden), og gå inn.
try {
  await page.waitForFunction(() => window.fristad?.engine, null, { timeout: 120000 });
  console.log("[test] motoren er klar, backend:", await page.evaluate(() => window.fristad.engine.backend()));
  // SwiftShader er treg: lås laveste kvalitet og oppløsning, så det første
  // bildet kommer innen rimelig tid.
  await page.evaluate(() => { window.fristad.setQuality(0); window.fristad.setScale(0.4); });
  await page.evaluate(() => document.querySelector("#enter")?.click());
  await page.waitForTimeout(Number(wait));
  console.log("[test] perf:", await page.evaluate(() => document.querySelector("#perf")?.textContent));
} catch (e) {
  console.log("[test] motoren ble ikke klar:", e.message);
}
await page.screenshot({ path: out, timeout: 300000 });
console.log("[test] skjermbilde:", out);
await browser.close();
server.close();
