// Redigeringen i nettleseren (`?edit`, docs/redigering.md steg 4), med den
// ferdige nettsiden (grenen web-dist, `sh tools/pull-dist.sh`) i Chromium
// uten skjerm: panelet og katalogen kommer, et klikk velger tønna på torget,
// piltasten flytter den, Ctrl+Z angrer, P bygger spillet på nytt og P igjen
// går tilbake i redigeringen. Skriver konsollen, det som feiler, og bilder.
//   PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright node tools/edit-test.mjs dist-web [webgl|webgpu] UT_PREFIKS
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join } from "node:path";

const [dir = "dist", mode = "webgl", out = "edit"] = process.argv.slice(2);
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg" };
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
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
const errors = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(m.text());
  console.log(`[${m.type()}] ${m.text().slice(0, 300)}`);
});
page.on("pageerror", (e) => {
  errors.push(e.message);
  console.log(`[pageerror] ${e.message}`);
});
const fails = [];
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FEIL"} ${what}`);
  if (!ok) fails.push(what);
};

await page.goto(`http://127.0.0.1:${port}/?edit${mode === "webgl" ? "&webgl" : ""}`, { waitUntil: "load" });
await page.waitForFunction(() => window.fristad?.engine, null, { timeout: 180000 });
await page.evaluate(() => document.querySelector("#enter")?.click());
await page.waitForTimeout(1500);
check(await page.evaluate(() => !document.querySelector("#editor")?.hidden), "panelet er oppe");
const items = await page.evaluate(() => document.querySelectorAll("#ed-grid .editor-item").length);
check(items > 100, `katalogen har ${items} modeller`);
const imgs = await page.evaluate(() => [...document.querySelectorAll("#ed-grid img")].slice(0, 20).filter((i) => i.complete && i.naturalWidth > 0).length);
console.log(`bilder lastet (av de 20 første): ${imgs}`);

// Fra torget mot tønna øst for lyktestolpen; klikk midt i bildet.
await page.evaluate(() => window.fristad.engine.place(4.6, 1.7, 9.4, -90, -12));
await page.waitForTimeout(500);
await page.mouse.click(640, 360);
await page.waitForTimeout(500);
const selected = await page.evaluate(() => document.querySelector("#ed-selected").textContent);
check(selected.includes("barrel"), `klikket valgte: ${selected}`);
await page.screenshot({ path: `${out}_valgt.png` });
await page.keyboard.press("ArrowLeft");
await page.keyboard.down("Shift");
await page.keyboard.press("ArrowLeft");
await page.keyboard.up("Shift");
await page.waitForTimeout(500);
const state = await page.evaluate(() => document.querySelector("#ed-state").textContent);
check(/endret|brudd/.test(state), `etter flytting: ${state}`);
await page.screenshot({ path: `${out}_flyttet.png` });
await page.keyboard.down("Control");
await page.keyboard.press("KeyZ");
await page.keyboard.press("KeyZ");
await page.keyboard.up("Control");
await page.waitForTimeout(500);
check((await page.evaluate(() => document.querySelector("#ed-state").textContent)) === "Grønt", "angret");

// P: spill (byen bygges på nytt), og P igjen: tilbake i redigeringen.
await page.keyboard.press("ArrowLeft");
await page.keyboard.press("KeyP");
await page.waitForTimeout(8000);
check(await page.evaluate(() => document.querySelector("#editor").hidden), "P: spiller");
await page.screenshot({ path: `${out}_spill.png` });
await page.keyboard.press("KeyP");
await page.waitForTimeout(1500);
check(await page.evaluate(() => !document.querySelector("#editor").hidden), "P igjen: redigerer");
check(errors.length === 0, `ingen feil i konsollen (${errors.length})`);
await browser.close();
server.close();
console.log(fails.length ? `\n${fails.length} feil` : "\nalt ok");
process.exit(fails.length ? 1 : 0);
