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
page.on("response", (r) => r.status() >= 400 && console.log(`[http ${r.status()}] ${r.url()}`));
// Minnet wasm-modulen har tatt (haugen vokser og krymper aldri): fanges
// ved instansieringen, så det kan måles også på eldre bygg.
await page.addInitScript(() => {
  const keep = (r) => {
    const exports = r.instance?.exports ?? r.exports;
    if (exports?.memory) window.__wasmMemory = exports.memory;
    return r;
  };
  for (const name of ["instantiate", "instantiateStreaming"]) {
    const orig = WebAssembly[name];
    WebAssembly[name] = (...a) => orig.apply(WebAssembly, a).then(keep);
  }
});
await page.goto(`http://127.0.0.1:${port}/${mode === "webgl" ? "?webgl" : ""}`, { waitUntil: "load" });
// Vent til motoren er klar (knappen på førstesiden), og gå inn.
try {
  const t0 = Date.now();
  await page.waitForFunction(() => window.fristad?.engine, null, { timeout: 400000 });
  console.log(`[test] klar etter ${Math.round((Date.now() - t0) / 1000)} s`);
  console.log("[test] motoren er klar, backend:", await page.evaluate(() => window.fristad.engine.backend()));
  await page.evaluate(() => document.querySelector("#enter")?.click());
  await page.waitForTimeout(Number(wait));
  console.log("[test] perf:", await page.evaluate(() => document.querySelector("#perf")?.textContent));
  console.log("[test] wasm-minne:", await page.evaluate(() => `${Math.round((window.__wasmMemory?.buffer.byteLength ?? 0) / 1048576)} MB`));
} catch (e) {
  console.log("[test] motoren ble ikke klar:", e.message);
  console.log("[test] wasm-minne:", await page.evaluate(() => `${Math.round((window.__wasmMemory?.buffer.byteLength ?? 0) / 1048576)} MB`).catch(() => "?"));
}
await page.screenshot({ path: out, timeout: 300000 });
console.log("[test] skjermbilde:", out);
await browser.close();
server.close();
