// Redigeringen i nettleseren (`?edit`, docs/redigering.md steg 4), med den
// ferdige nettsiden (grenen web-dist, `sh tools/pull-dist.sh`) i Chromium
// uten skjerm: panelet og katalogen kommer, et klikk velger tønna på torget,
// piltasten flytter den, Ctrl+Z angrer, musa drar den og Alt+dra heiser den, P bygger spillet på nytt og P igjen
// går tilbake i redigeringen; nede i krypten redigerer P nivået du står i.
// Skriver konsollen, det som feiler, og bilder.
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
await page.waitForFunction(() => window.fristad?.engine, null, { timeout: 600000 });
// SwiftShader tegner sakte (0–1 bilde i sekundet): hendelsene fra motoren
// kommer med neste bilde, så testen venter på hver tilstand.
const until = async (fn, arg, ms = 180000) => {
  try {
    await page.waitForFunction(fn, arg, { timeout: ms, polling: 500 });
    return true;
  } catch {
    return false;
  }
};
const text = (sel) => page.evaluate((s) => document.querySelector(s)?.textContent ?? "", sel);
const shot = (name) => page.screenshot({ path: `${out}_${name}.png`, timeout: 300000 });

await page.evaluate(() => document.querySelector("#enter")?.click());
check(await until(() => document.querySelector("#editor") && !document.querySelector("#editor").hidden), "panelet er oppe");
const items = await page.evaluate(() => document.querySelectorAll("#ed-grid .editor-item").length);
check(items > 100, `katalogen har ${items} modeller`);
check(await until(() => [...document.querySelectorAll("#ed-grid img")].slice(0, 6).every((i) => i.complete && i.naturalWidth > 0)), "bildene i katalogen er lastet");

// Fra torget mot tønna øst for lyktestolpen; klikk midt i bildet (i lerretet).
await page.evaluate(() => window.fristad.engine.place(4.6, 1.7, 9.4, -90, -12));
const box = await page.evaluate(() => {
  const r = document.querySelector("#world").getBoundingClientRect();
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
});
await page.mouse.click(box.x, box.y);
check(await until(() => document.querySelector("#ed-selected").textContent.includes("barrel")), `klikket valgte: ${await text("#ed-selected")}`);
await shot("valgt");
await page.keyboard.press("ArrowLeft");
await page.keyboard.down("Shift");
await page.keyboard.press("ArrowLeft");
await page.keyboard.up("Shift");
check(await until(() => /endret|brudd/.test(document.querySelector("#ed-state").textContent)), `etter flytting: ${await text("#ed-state")}`);
await shot("flyttet");
await page.keyboard.down("Control");
await page.keyboard.press("KeyZ");
await page.keyboard.press("KeyZ");
await page.keyboard.up("Control");
check(await until(() => document.querySelector("#ed-state").textContent === "Grønt"), `angret: ${await text("#ed-state")}`);

// Musa: dra tønna selv (den følger pekeren), Alt+dra heiser den, og hvert
// drag angres for seg.
const view0 = await page.evaluate(() => window.fristad.engine.view());
await page.mouse.click(box.x, box.y);
await until(() => document.querySelector("#ed-selected").textContent.includes("barrel"));
await page.mouse.move(box.x, box.y);
await page.mouse.down();
await page.mouse.move(box.x + 60, box.y, { steps: 6 });
await page.mouse.up();
check(await until(() => /endret|brudd/.test(document.querySelector("#ed-state").textContent)), `dratt med musa: ${await text("#ed-state")}`);
await shot("dratt");
await page.keyboard.down("Alt");
await page.mouse.move(box.x + 60, box.y);
await page.mouse.down();
await page.mouse.move(box.x + 60, box.y - 40, { steps: 6 });
await page.mouse.up();
await page.keyboard.up("Alt");
check(await until(() => /brudd/.test(document.querySelector("#ed-state").textContent)), `Alt+dra heiste (svever): ${await text("#ed-state")}`);
await page.keyboard.down("Control");
await page.keyboard.press("KeyZ");
await page.keyboard.press("KeyZ");
await page.keyboard.up("Control");
check(await until(() => document.querySelector("#ed-state").textContent === "Grønt"), `angret dragene: ${await text("#ed-state")}`);
check((await page.evaluate(() => window.fristad.engine.view()))[0] === view0[0], "kameraet sto stille mens du dro");

// P: spill (byen bygges på nytt), og P igjen: tilbake i redigeringen, med
// endringen i dataene (den er ikke lagret, så den står som endret).
await page.mouse.click(box.x, box.y);
await until(() => document.querySelector("#ed-selected").textContent.includes("barrel"));
await page.keyboard.down("Shift");
await page.keyboard.press("ArrowLeft");
await page.keyboard.up("Shift");
check(await until(() => /endret|brudd/.test(document.querySelector("#ed-state").textContent)), "flyttet før P");
await page.keyboard.press("KeyP");
check(await until(() => document.querySelector("#editor").hidden, null, 600000), "P: spiller");
await page.waitForTimeout(5000);
await shot("spill");
await page.keyboard.press("KeyP");
check(await until(() => !document.querySelector("#editor").hidden), "P igjen: redigerer");
check(await until(() => /endret|brudd/.test(document.querySelector("#ed-state").textContent)), `P igjen: endringen er med (${await text("#ed-state")})`);
await shot("igjen");

// Krypten: P (spill), ned trappa, og P igjen redigerer nivået du står i
// (redigeringen løfter deg ikke opp til byen).
await page.keyboard.press("KeyP");
check(await until(() => document.querySelector("#editor").hidden, null, 600000), "P: spiller igjen");
await page.evaluate(() => window.fristad.engine.descend());
check(await until(() => window.fristad.engine.view()[1] < -20, null, 300000), `ned i krypten (y ${(await page.evaluate(() => window.fristad.engine.view()[1])).toFixed(1)})`);
await page.waitForTimeout(3000);
await page.keyboard.press("KeyP");
check(await until(() => !document.querySelector("#editor").hidden), "P i krypten: redigerer");
await page.waitForTimeout(3000);
const y = await page.evaluate(() => window.fristad.engine.view()[1]);
check(y < -20, `redigeringen står i krypten (y ${y.toFixed(1)})`);
await page.mouse.click(box.x, box.y);
await page.waitForTimeout(3000);
console.log(`klikk i krypten valgte: ${await text("#ed-selected")}`);
await shot("krypten");
check(errors.filter((e) => !e.includes("404")).length === 0, `ingen feil i konsollen (${errors.length}, 404 for favicon telles ikke)`);
await browser.close();
server.close();
console.log(fails.length ? `\n${fails.length} feil` : "\nalt ok");
process.exit(fails.length ? 1 : 0);
