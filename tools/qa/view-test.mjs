// Prøver 3D-visningen i dommen i Chromium uten skjerm: åpner siden i UT,
// velger fanen, trykker «Se i 3D» på kortet og skriver et skjermbilde.
// three.js hentes fra npm-pakken i THREE_DIR (CDN-en er sperret i skyen).
//
//   PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright THREE_DIR=…/package \
//     node tools/qa/view-test.mjs UT fane kort bilde.png [--whole]
import http from "node:http";
import fs from "node:fs";
import path from "node:path";

const { chromium } = await import(path.join(process.env.PLAYWRIGHT_DIR, "index.mjs"));
const [dir, tab, id, out] = process.argv.slice(2);
const three = process.env.THREE_DIR;
const types = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".json": "application/json; charset=utf-8", ".glb": "model/gltf-binary", ".jpg": "image/jpeg" };
const server = http.createServer((req, res) => {
  const p = path.join(dir, decodeURIComponent(req.url.split("?")[0]).replace(/^\/$/, "/index.html"));
  if (!fs.existsSync(p)) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": types[path.extname(p)] || "application/octet-stream" });
  fs.createReadStream(p).pipe(res);
}).listen(0);
const port = server.address().port;
const browser = await chromium.launch({ args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
const page = await browser.newPage({ viewport: { width: 1400, height: 1000 } });
const errors = [];
page.on("console", m => { if (m.type() === "error") errors.push(m.text()); });
page.on("pageerror", e => errors.push(String(e)));
await page.route("https://cdn.jsdelivr.net/npm/three@0.160.0/**", route => {
  const rel = route.request().url().split("three@0.160.0/")[1];
  route.fulfill({ path: path.join(three, rel), contentType: "text/javascript" });
});
await page.goto(`http://127.0.0.1:${port}/`);
await page.waitForSelector(".chip");
await page.locator(".chip", { hasText: tab }).first().click();
await page.locator(`#${id} .btn3d`).click();
await page.waitForFunction(() => document.getElementById("view-status").textContent === "", null, { timeout: 120000 });
if (process.argv.includes("--whole") === false) await page.uncheck("#view-whole");
await page.waitForTimeout(1500);
await page.locator("#view").screenshot({ path: out });
console.log(errors.length ? "FEIL:\n" + errors.join("\n") : "ingen feil i konsollen");
await browser.close();
server.close();
