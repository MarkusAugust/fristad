// Redigeringen i nettleseren (`?edit`, docs/redigering.md steg 4), med den
// ferdige nettsiden (grenen web-dist, `sh tools/pull-dist.sh`) i Chromium
// uten skjerm: panelet og katalogen kommer, et klikk velger tønna på torget,
// piltasten flytter den, Ctrl+Z angrer, musa drar den og Alt+dra heiser den,
// egenskapene og historikken, dybden (klikk igjen og Tab), festet, rammen og
// bokmerkene (runde 1), gruppene, ruta til en innbygger og lysene om natta
// (runde 2); P bygger spillet på nytt og P igjen går tilbake i
// redigeringen; nede i krypten redigerer P nivået du står i.
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

// Runde 1: egenskapene, historikken, dybden, festet, rammen og bokmerkene.
const stateIs = (t) => until((t) => document.querySelector("#ed-state").textContent === t, t);
const changed = () => until(() => /endret|brudd/.test(document.querySelector("#ed-state").textContent));
const undoAll = async () => {
  for (let k = 0; k < 12; k++) {
    if ((await text("#ed-state")) === "Grønt") return true;
    await page.keyboard.down("Control");
    await page.keyboard.press("KeyZ");
    await page.keyboard.up("Control");
    await page.waitForTimeout(1500);
  }
  return stateIs("Grønt");
};
await page.mouse.click(box.x, box.y);
await until(() => document.querySelector("#ed-selected").textContent.includes("barrel"));
check(await until(() => !document.querySelector("#ed-props-box").hidden && document.querySelector('#ed-props [name="x"]')), "egenskapene til tønna vises");
const x0 = Number(await page.evaluate(() => document.querySelector('#ed-props [name="x"]').value));
await page.evaluate((x) => {
  const f = document.querySelector('#ed-props [name="x"]');
  f.value = String(x);
  f.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
}, x0 + 0.5);
check(await changed(), `x satt i panelet (${x0} + 0,5): ${await text("#ed-state")}`);
check(await until(() => document.querySelectorAll("#ed-history .editor-step").length >= 2), "historikken har steget");
await page.evaluate(() => document.querySelector('#ed-history [data-n="0"]').click());
check(await stateIs("Grønt"), `«Start» i historikken angret: ${await text("#ed-state")}`);
// Klikk igjen på samme sted: lista over det som lå under pekeren, og Tab.
await page.mouse.click(box.x, box.y);
await page.waitForTimeout(1500);
await page.mouse.click(box.x, box.y);
await page.waitForTimeout(1500);
const under = await page.evaluate(() => document.querySelectorAll("#ed-under .editor-under-item").length);
console.log(`under pekeren: ${under}`);
await page.keyboard.press("Tab");
await page.waitForTimeout(1500);
check((await text("#ed-selected")).length > 0, `Tab valgte: ${await text("#ed-selected")}`);
// Festet: bryteren i panelet.
await page.evaluate(() => document.querySelector("#ed-snap").click());
check(await until(() => document.querySelector("#ed-snap").checked), "festet er på");
await page.evaluate(() => document.querySelector("#ed-snap").click());
// Rammen: Shift+dra rundt tønna velger den.
await page.keyboard.down("Shift");
await page.mouse.move(box.x - 40, box.y - 40);
await page.mouse.down();
await page.mouse.move(box.x + 40, box.y + 40, { steps: 5 });
await page.mouse.up();
await page.keyboard.up("Shift");
check(await until(() => document.querySelector("#ed-selected").textContent.includes("barrel")), `rammen valgte: ${await text("#ed-selected")}`);
// Bokmerkene: Ctrl+1 husker kameraet, 1 går dit igjen.
await page.keyboard.down("Control");
await page.keyboard.press("Digit1");
await page.keyboard.up("Control");
await page.evaluate(() => window.fristad.engine.place(0, 20, 0, 0, -45));
await page.keyboard.press("Digit1");
await page.waitForTimeout(1500);
const back1 = await page.evaluate(() => window.fristad.engine.view());
check(Math.abs(back1[0] - view0[0]) < 0.5 && Math.abs(back1[2] - view0[2]) < 0.5, `bokmerke 1 tilbake (${back1.slice(0, 3).map((v) => v.toFixed(1))})`);

// Runde 2: grupper. En stor ramme velger flere ting, Ctrl+G lager en
// gruppe, et klikk på tønna velger hele gruppen, og Ctrl+Shift+G løser opp.
await page.keyboard.down("Shift");
await page.mouse.move(box.x - 200, box.y - 120);
await page.mouse.down();
await page.mouse.move(box.x + 200, box.y + 120, { steps: 5 });
await page.mouse.up();
await page.keyboard.up("Shift");
await page.waitForTimeout(1500);
const many = (await text("#ed-selected")).split(",").length;
check(many >= 2, `rammen valgte ${many} ting`);
await page.keyboard.down("Control");
await page.keyboard.press("KeyG");
await page.keyboard.up("Control");
check(await until(() => /Gruppen «gruppe 1»/.test(document.querySelector("#ed-grp").textContent)), `gruppen er laget: ${await text("#ed-grp")}`);
check(await changed(), "gruppen er en endring");
await page.mouse.click(box.x, box.y - 300);
await page.mouse.click(box.x, box.y);
check(await until((n) => document.querySelector("#ed-selected").textContent.split(",").length === n, many), `klikk på tønna valgte gruppen: ${await text("#ed-selected")}`);
await page.keyboard.down("Control");
await page.keyboard.down("Shift");
await page.keyboard.press("KeyG");
await page.keyboard.up("Shift");
await page.keyboard.up("Control");
check(await until(() => document.querySelector("#ed-grp").hidden), "gruppen er løst opp");
check(await undoAll(), `gruppene angret: ${await text("#ed-state")}`);

// Runde 2: ruta til en innbygger. Ovenfra mot Buttonsmeden: et klikk velger
// hen, ruta vises, og ventetiden ved første punkt settes i panelet.
await page.evaluate(() => window.fristad.engine.place(10.7, 7, 6.5, 0, -45));
await page.waitForTimeout(3000);
await page.mouse.click(box.x, box.y);
const npc = await until(() => !document.querySelector("#ed-route").hidden, null, 60000);
check(npc, `ruta vises for ${await text("#ed-selected")}: ${await text("#ed-route")}`);
if (npc) {
  await shot("ruta");
  await page.evaluate(() => {
    const f = document.querySelector('#ed-props [name="route.0.wait"]');
    f.value = "9";
    f.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  });
  check(await changed(), "ventetiden er satt");
  check(await undoAll(), "ventetiden angret");
}

// Runde 2: lysene og natta. T bytter til natt også i redigeringen, og
// lyset i lyktestolpen på torget (Ref med lys) endres i panelet.
await page.keyboard.press("KeyT");
check(await until(() => document.documentElement.dataset.theme === "dark"), "T: natt i redigeringen");
await page.evaluate(() => window.fristad.engine.place(-10.32, 2.6, -4.5, 0, -15));
await page.waitForTimeout(3000);
await shot("natt");
await page.mouse.click(box.x, box.y);
await page.waitForTimeout(1500);
for (let k = 0; k < 6 && !/lykt|ild/.test(await text("#ed-selected")); k++) {
  await page.keyboard.press("Tab");
  await page.waitForTimeout(1500);
}
const lit = /lykt|ild/.test(await text("#ed-selected"));
console.log(`lyset: ${await text("#ed-selected")}`);
if (lit) {
  await page.evaluate(() => {
    const f = document.querySelector('#ed-props [name="range"]');
    f.value = "14";
    f.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  });
  check(await changed(), "rekkevidden til lyset er satt");
  await shot("lyset");
  check(await undoAll(), "lyset angret");
} else console.log("(lyset lå ikke under pekeren her; testene i Rust dekker det)");
await page.keyboard.press("KeyT");
await until(() => document.documentElement.dataset.theme !== "dark");
await page.evaluate((v) => window.fristad.engine.place(v[0], v[1], v[2], (v[3] * 180) / Math.PI, (v[4] * 180) / Math.PI), view0);
await page.waitForTimeout(2000);

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
