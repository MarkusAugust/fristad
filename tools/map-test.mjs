// Tester minikartet, det store kartet og stolpene i den ferdige nettsiden
// (som browser-test.mjs):
//   node tools/map-test.mjs KATALOG [webgl|webgpu] [UT]
// Går ned i kjelleren med engine.descend(), ser at minikartet kommer og har
// tegnet startrommet, åpner og lukker det store kartet med K, og tar
// skjermbilder (UT-minikart.png og UT-kart.png).
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join } from "node:path";

const [dir = "dist", mode = "webgl", out = "kart"] = process.argv.slice(2);
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
await page.waitForSelector("#enter:not([hidden])", { timeout: 180000 });
await page.click("#enter", { timeout: 120000 });
await page.waitForTimeout(1500);
// Stolpene over knappelinja.
check(await page.isVisible("#bars"), "liv og energi står over knappelinja");
check(/\d+ \/ \d+/.test(await page.textContent("#hp-text")), `livet med tall: ${await page.textContent("#hp-text")}`);
check(await page.isHidden("#minimap"), "oppe i byen er det ikke noe minikart");
// Ned i kjelleren: minikartet kommer.
// SwiftShader tegner sakte: god tid (standard grafikk er valgt fra start).
await page.evaluate(() => window.fristad.engine.descend());
await page.waitForSelector("#minimap:not([hidden])", { timeout: 180000 });
await page.waitForTimeout(8000);
check(await page.isVisible("#minimap"), "minikartet står i hjørnet nede i kjelleren");
check((await page.textContent("#map-name")).startsWith("Nivå 1"), `navnet på nivået: ${await page.textContent("#map-name")}`);
const lit = await page.evaluate(() => {
  const c = document.getElementById("minimap-canvas");
  const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
  let n = 0;
  for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 0) n++;
  return n / (d.length / 4);
});
console.log(`     tegnet på minikartet: ${Math.round(lit * 100)} %`);
check(lit > 0.05, "minikartet har tegnet startrommet");
await page.screenshot({ path: `${out}-minikart.png` });
await page.keyboard.press("k");
await page.waitForTimeout(3000);
check(await page.isVisible("#bigmap"), "K åpner det store kartet");
await page.screenshot({ path: `${out}-kart.png` });
await page.keyboard.press("k");
await page.waitForTimeout(300);
check(await page.isHidden("#bigmap"), "K lukker det igjen");
check(errors.length === 0, `ingen feil i konsollen${errors.length ? ": " + errors.join(" | ") : ""}`);
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
