// Tester grafikkvalget og musa i den ferdige nettsiden (som browser-test.mjs):
//   node tools/input-test.mjs KATALOG [webgl|webgpu]
// Bytter grafikk med G og måler bildefrekvensen i begge, klikker i bildet
// (med og uten pekerlås) og ser at et hugg sendes, og åpner en butikk og
// lukker den med «Tilbake» for å se at musa fanges igjen.
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join } from "node:path";

const [dir = "dist", mode = "webgl"] = process.argv.slice(2);
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg", ".mp3": "audio/mpeg" };
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
const page = await browser.newPage({ viewport: { width: 960, height: 540 } });
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
page.on("console", (m) => m.type() === "error" && !m.text().includes("404") && errors.push(m.text()));
let failed = 0;
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FEIL"} ${what}`);
  if (!ok) failed++;
};
await page.goto(`http://127.0.0.1:${port}/${mode === "webgl" ? "?webgl" : ""}`);
await page.waitForFunction(() => window.fristad?.engine, null, { timeout: 180000 });
// Musekallene til motoren logges.
await page.evaluate(() => {
  const e = window.fristad.engine;
  const mouse = e.mouse.bind(e);
  window.__mouse = [];
  e.mouse = (b, d) => {
    window.__mouse.push([b, d]);
    return mouse(b, d);
  };
});
await page.waitForSelector("#enter:not([hidden])", { timeout: 180000 });
await page.click("#enter", { timeout: 120000 });
await page.waitForTimeout(1500);
const fps = async () => {
  await page.waitForTimeout(6000);
  return Number((await page.textContent("#perf")).split(" ")[0]);
};
// Grafikken.
check((await page.textContent("#graphics-label")) === "standard", "starter i standard grafikk");
const fpsStandard = await fps();
await page.keyboard.press("g");
check((await page.textContent("#graphics-label")) === "avansert", "G bytter til avansert");
check((await page.evaluate(() => localStorage.getItem("fristad.graphics"))) === "avansert", "valget huskes");
const fpsAdvanced = await fps();
console.log(`     bilder per sekund: standard ${fpsStandard}, avansert ${fpsAdvanced} (SwiftShader)`);
check(fpsStandard >= fpsAdvanced, "standard er ikke tregere");
await page.keyboard.press("g");
// Musa: et klikk i bildet sender et hugg (med eller uten pekerlås).
await page.evaluate(() => (window.__mouse = []));
await page.mouse.click(480, 270, { delay: 30 });
await page.waitForTimeout(300);
const lockedAfterClick = await page.evaluate(() => document.pointerLockElement?.id ?? null);
await page.mouse.click(480, 270, { delay: 30 });
await page.waitForTimeout(300);
const calls = await page.evaluate(() => window.__mouse);
console.log(`     pekerlås: ${lockedAfterClick ?? "nei"}, musekall: ${JSON.stringify(calls)}`);
check(calls.some(([b, d]) => b === 0 && d), "venstre klikk sender et hugg");
// Høyre knapp blokker.
await page.evaluate(() => (window.__mouse = []));
await page.mouse.click(480, 270, { button: "right" });
await page.waitForTimeout(200);
check((await page.evaluate(() => window.__mouse)).some(([b, d]) => b === 2 && d), "høyre klikk blokker");
// Butikken: «Tilbake» fanger musa igjen og spillet går.
await page.evaluate(() => window.fristad.openShop("Buttonsmeden", []));
await page.waitForTimeout(300);
check(await page.isVisible("#overlay"), "butikken er åpen");
await page.click("#overlay-close", { force: true, timeout: 120000 });
await page.waitForTimeout(600);
const after = await page.evaluate(() => ({ locked: document.pointerLockElement?.id ?? null, gate: !document.querySelector("#gate").hidden }));
console.log(`     etter «Tilbake»: pekerlås ${after.locked ?? "nei"}, menyen ${after.gate ? "synlig" : "skjult"}`);
check(!after.gate, "spillet går videre etter «Tilbake»");
check(after.locked === "world" || lockedAfterClick === null, "musa er fanget igjen (der nettleseren gir pekerlås)");
check(errors.length === 0, `ingen feil i konsollen ${errors.length ? JSON.stringify(errors.slice(0, 3)) : ""}`);
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
