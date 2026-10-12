// Validerer shaderne mot Dawn (WebGPU i Chromium), som er strengere enn
// naga: blant annet må textureSample stå i uniform kontrollflyt. Modulene
// settes sammen slik rendereren gjør det. Kjør: node tools/wgsl-check.mjs
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
const dir = new URL("../src/adapters/wgpu/shaders/", import.meta.url);
const src = (n) => readFileSync(new URL(n + ".wgsl", dir), "utf8");
const modules = {
  world: ["noise", "common", "shadow", "world"],
  sky: ["noise", "common", "sky"],
  particles: ["noise", "common", "particles"],
  lines: ["noise", "common", "lines"],
  post: ["noise", "common", "shadow", "post"],
  bake: ["noise", "bake"],
  smaa: ["smaa"],
};

const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM ?? "/opt/pw-browsers/chromium",
  args: ["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--use-angle=swiftshader", "--use-vulkan=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"],
});
// WebGPU krever sikker kontekst, så sida serveres fra 127.0.0.1.
const { createServer } = await import("node:http");
const server = createServer((_req, res) => {
  res.writeHead(200, { "content-type": "text/html" });
  res.end("<!doctype html><title>wgsl</title>");
});
await new Promise((r) => server.listen(0, r));
const page = await browser.newPage();
await page.goto(`http://127.0.0.1:${server.address().port}/`);
let failed = 0;
for (const [name, parts] of Object.entries(modules)) {
  const code = parts.map(src).join("\n");
  const messages = await page.evaluate(async (code) => {
    const adapter = await navigator.gpu?.requestAdapter();
    if (!adapter) return [{ type: "error", message: "ingen WebGPU-adapter" }];
    const device = await adapter.requestDevice();
    const mod = device.createShaderModule({ code });
    const info = await mod.getCompilationInfo();
    return info.messages.map((m) => ({ type: m.type, line: m.lineNum, message: m.message }));
  }, code);
  const errors = messages.filter((m) => m.type === "error");
  if (errors.length) {
    failed++;
    // Linjenummeret er i den sammensatte kilden; regn om til fil og linje.
    const starts = [];
    let at = 1;
    for (const p of parts) {
      starts.push([p, at]);
      at += src(p).split("\n").length;
    }
    for (const e of errors) {
      const [file, start] = [...starts].reverse().find(([, s]) => e.line >= s) ?? ["?", 1];
      console.log(`${name}: ${file}.wgsl:${e.line - start + 1}: ${e.message}`);
    }
  } else {
    console.log(`${name}: ok (${messages.length} meldinger)`);
  }
}
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
