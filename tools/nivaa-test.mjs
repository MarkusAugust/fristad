// Nivåredigeringen i nettleseren (`?nivaa`, web/nivaa.js): kjører den ferdige
// nettsiden i Chromium uten skjerm og sjekker at kartet, paletten og 3D kommer
// opp, at en modul kan settes ut med musa (fra paletten til kartet), at angre
// virker, at reglene vises, at prøveturen starter og Esc går tilbake, og at
// Lagre uten nøkkel gir en zip (utkastet huskes). Skjermbildene skrives i UT.
//   node tools/nivaa-test.mjs KATALOG [webgl|webgpu] [UT-mappe]
// Nettsiden hentes fra grenen web-dist: `sh tools/pull-dist.sh`.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { readFile, stat, mkdir } from "node:fs/promises";
import { extname, join } from "node:path";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
const [dir = "dist", mode = "webgl", out = "target/nivaa-test"] = process.argv.slice(2);
await mkdir(out, { recursive: true });
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
const args =
  mode === "webgpu"
    ? ["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--use-angle=swiftshader", "--use-vulkan=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"]
    : ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"];
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM ?? "/opt/pw-browsers/chromium", args, headless: true });
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, acceptDownloads: true });
const errors = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(m.text());
  console.log(`[${m.type()}] ${m.text().slice(0, 300)}`);
});
page.on("pageerror", (e) => {
  errors.push(e.message);
  console.log(`[pageerror] ${e.message}`);
});
page.on("response", (r) => r.status() >= 400 && !r.url().endsWith(".jpg") && console.log(`[http ${r.status()}] ${r.url()}`));

let failed = 0;
const check = (ok, what) => {
  console.log(`${ok ? "OK  " : "FEIL"} ${what}`);
  if (!ok) failed++;
};
const shot = async (name) => {
  await page.screenshot({ path: `${out}/${name}.png`, timeout: 300000 });
  console.log(`[test] ${out}/${name}.png`);
};

// `?nivaa` på forsiden sender videre til nivaa.html.
await page.goto(`http://127.0.0.1:${port}/?nivaa${mode === "webgl" ? "&webgl" : ""}&fil=test`, { waitUntil: "load" });
const t0 = Date.now();
await page.waitForFunction(() => window.fristad?.engine && document.querySelector("#overlay")?.hidden, null, { timeout: 600000 });
console.log(`[test] klar etter ${Math.round((Date.now() - t0) / 1000)} s på ${await page.evaluate(() => window.fristad.engine.backend())}`);
check(page.url().includes("nivaa.html"), "?nivaa sender til nivaa.html");
await page.waitForTimeout(3000);
const map0 = await page.evaluate(() => window.fristad.state.map);
check(map0 && map0.modules.length >= 10, `prøvenivået er åpnet (${map0?.modules.length} moduler)`);
check(await page.evaluate(() => document.querySelectorAll("#modlist .tile").length) >= 10, "paletten har modulene");
check(await page.evaluate(() => [...document.querySelectorAll("#modlist img")].some((i) => i.complete && i.naturalWidth > 0)), "bildene av modulene kommer");
// Kartet er tegnet (ikke bare bakgrunn).
const painted = await page.evaluate(() => {
  const c = document.querySelector("#plan");
  const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
  let n = 0;
  for (let i = 0; i < d.length; i += 16) if (d[i] > 60) n++;
  return n;
});
check(painted > 500, `kartet er tegnet (${painted} lyse punkter)`);
// 3D: noe er tegnet (WebGL: les tilbake pikslene fra et kopi-canvas).
const litNow = () =>
  page.evaluate(async () => {
    const c = document.querySelector("#world");
    const tmp = document.createElement("canvas");
    tmp.width = 160;
    tmp.height = 90;
    // Uten preserveDrawingBuffer kan bildet være tømt: les i neste bilde,
    // og prøv noen ganger.
    let best = 0;
    for (let k = 0; k < 5 && best < 1000; k++) {
      await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
      tmp.getContext("2d").clearRect(0, 0, 160, 90);
      tmp.getContext("2d").drawImage(c, 0, 0, 160, 90);
      const d = tmp.getContext("2d").getImageData(0, 0, 160, 90).data;
      let n = 0;
      for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 60) n++;
      best = Math.max(best, n);
    }
    return best;
  });
const lit = await litNow();
check(lit > 200, `3D-vinduet viser nivået (${lit} lyse punkter av 14400)`);
await shot("1_prøvenivået");

// Sett ut en modul: dra «Rom 2 × 2» fra paletten og slipp på kartet øst for nivået.
const n0 = map0.modules.length;
const tile = page.locator('#modlist .tile[data-module="grav/rom_2x2"]');
const target = await page.evaluate(() => {
  const st = window.fristad.state;
  // Under nivået (sør), ved den venstre kanten: fritt og innenfor kartet.
  const cells = st.map.modules.flatMap((m) => m.cells);
  const i = Math.min(...cells.map((c) => c[0])) + 1;
  const j = Math.max(...cells.map((c) => c[2])) + 2;
  const r = document.querySelector("#plan").getBoundingClientRect();
  const [px, py] = st.view.toScreen(i * 4, j * 4);
  return [r.left + px, r.top + py, i];
});
const tb = await tile.boundingBox();
await page.mouse.move(tb.x + tb.width / 2, tb.y + tb.height / 2);
await page.mouse.down();
await page.mouse.move(target[0] - 30, target[1], { steps: 8 });
await page.mouse.move(target[0], target[1], { steps: 4 });
await page.mouse.up();
await page.waitForTimeout(500);
let map1 = await page.evaluate(() => window.fristad.state.map);
check(map1.modules.length === n0 + 1, `en modul er satt ut med musa (${n0} → ${map1.modules.length})`);
const added = map1.modules.find((m) => m.module === "grav/rom_2x2" && !map0.modules.some((o) => o.id === m.id));
check(!!added && added.at[0] === target[2], `modulen står der den ble sluppet (${added?.at})`);
await page.waitForTimeout(1500);
await shot("2_modul_satt_ut");
// R dreier, angre og gjør om.
await page.keyboard.press("KeyR");
await page.waitForTimeout(300);
map1 = await page.evaluate(() => window.fristad.state.map);
check(map1.modules.find((m) => m.id === added?.id)?.rot === 1, "R dreier det valgte");
await page.keyboard.press("Control+KeyZ");
await page.keyboard.press("Control+KeyZ");
await page.waitForTimeout(300);
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === n0, "Ctrl+Z angrer");
await page.keyboard.press("Control+KeyY");
await page.waitForTimeout(300);
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === n0 + 1, "Ctrl+Y gjør om");
// Reglene: et rom i lufta øst for alt annet har ingen vei inn: «kan ikke nås»
// meldes for innhold der. Sett en kiste i det.
await page.evaluate((i) => window.fristad.level({ op: "add_thing", kind: "chest", at: [i * 4 + 40, 0, 40] }), target[2]);
const issues = await page.evaluate(() => [...document.querySelectorAll("#issues li")].map((l) => l.textContent));
check(issues.some((t) => t.includes("gulvet") || t.includes("nås")), `reglene sier fra (${issues.length} i lista)`);
await page.locator("#issues li.feil").first().click();
await page.waitForTimeout(400);
await shot("3_reglene");
await page.keyboard.press("Control+KeyZ");
// Åpningen: klikk en pil og velg port.
await page.locator("#plan").click({ position: { x: 5, y: 5 }, button: "right" });
await page.keyboard.press("Home");
await page.waitForTimeout(300);
const opening = await page.evaluate(() => {
  const st = window.fristad.state;
  const m = st.map.modules.find((m) => m.module === "grav/rom_2x2") || st.map.modules[0];
  const o = m.openings[0];
  const d = [[0, -1], [1, 0], [0, 1], [-1, 0]][o.side];
  const r = document.querySelector("#plan").getBoundingClientRect();
  const [px, py] = st.view.toScreen(o.at[0] * 4 + d[0] * 1.8, o.at[1] * 4 + d[1] * 1.8);
  return [r.left + px, r.top + py, m.id, o.id];
});
await page.mouse.click(opening[0], opening[1]);
await page.waitForTimeout(300);
check(await page.evaluate(() => !!document.querySelector("#op-kind")), "klikk på en åpning gir valget i panelet");
if (await page.evaluate(() => !!document.querySelector("#op-kind"))) {
  await page.selectOption("#op-kind", "port");
  await page.waitForTimeout(400);
  const ov = await page.evaluate((m) => window.fristad.state.map.modules.find((x) => x.id === m)?.overrides, opening[2]);
  check(ov?.some((o) => o.kind === "port"), "åpningen er en port");
}
// Delt skjerm og full størrelse.
await page.keyboard.press("Escape");
// Kameraet tilbake i kapellet (det forrige «gå dit» sto ved kista i fjellet).
await page.evaluate(() => {
  const m = window.fristad.state.map.modules.find((m) => m.module === "grav/kapell") || window.fristad.state.map.modules[0];
  const c = m.cells[Math.floor(m.cells.length / 2)];
  window.fristad.level({ cmd: "place", x: c[0] * 4, y: c[1] + 1.7, z: c[2] * 4 + 1, yaw: 0, pitch: -5 });
});
await page.click("#v-3d");
await page.waitForTimeout(1500);
const lit3d = await litNow();
check(lit3d > 1000, `bare 3D fyller vinduet (${lit3d} lyse punkter)`);
await shot("4_bare_3d");
await page.click("#v-map");
await page.waitForTimeout(400);
await shot("5_bare_kartet");
await page.click("#v-split");
await page.waitForTimeout(800);
// Prøveturen og Esc tilbake.
await page.click("#play");
await page.waitForFunction(() => document.body.classList.contains("playing"), null, { timeout: 600000 }).catch(() => {});
const playing = await page.evaluate(() => window.fristad.engine.level_playing());
check(playing, "Spill fra start bygger verden og starter prøveturen");
await page.waitForTimeout(3000);
const litPlay = await litNow();
check(litPlay > 500, `prøveturen viser nivået (${litPlay} lyse punkter)`);
await shot("6_proveturen");
await page.keyboard.press("Escape");
await page.waitForTimeout(800);
check(!(await page.evaluate(() => window.fristad.engine.level_playing())), "Esc går tilbake til redigeringen");
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === n0 + 1, "redigeringen er som den var etter prøveturen");
// Lagre uten nøkkel: zip, og utkastet er i nettleseren.
page.once("dialog", (d) => d.accept(""));
const dl = page.waitForEvent("download", { timeout: 90000 }).catch(() => null);
await page.click("#save", { force: true });
const file = await dl;
check(!!file && file.suggestedFilename().endsWith(".zip"), `Lagre uten nøkkel laster ned ${file?.suggestedFilename()}`);
const draft = await page.evaluate(() => Object.keys(localStorage).filter((k) => k.startsWith("fristad.nivaa.utkast:")));
check(draft.length > 0, `utkastet er i nettleseren (${draft.join(", ")})`);
// Last siden på nytt: utkastet åpnes.
await page.reload({ waitUntil: "load" });
await page.waitForFunction(() => window.fristad?.engine && document.querySelector("#overlay")?.hidden, null, { timeout: 600000 });
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === n0 + 1, "utkastet åpnes igjen");
await page.waitForTimeout(2000);
await shot("7_etter_omlasting");
check(!errors.some((e) => /panic|unreachable|RuntimeError/.test(e)), `ingen krasj i konsollen (${errors.length} feil)`);
console.log(failed ? `[test] ${failed} sjekker feilet` : "[test] alt i orden");
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
