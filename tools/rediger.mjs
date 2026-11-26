// Redigeringen lokalt (eieren 26. november): tjeneren viser den bygde
// nettsiden i dist/ på http://localhost:1350, og Lagre (Ctrl+S) i `?edit` og
// `?nivaa` skriver filene rett i repoet på maskinen i stedet for til grenen
// `redigering` på GitHub (web/github.js, `LOCAL`). Ingen nøkkel og ingen zip:
// endringene står i arbeidskopien, der Claude kjører kontrollene før push.
//
//   node tools/rediger.mjs           viser dist/ (bygg først med --bygg)
//   node tools/rediger.mjs --bygg    kjører `sh tools/build-web.sh` først
//
// Godkjenningssiden (bygd med `sh tools/dom.sh`) vises på /dom/, og hver dom
// der føres rett inn i assets/catalog/approvals.json.
//
// Tjeneren lytter bare på denne maskinen, og skriver bare json-filer under
// assets/layout/ og assets/levels/, og dommene.

import { execFileSync } from "node:child_process";
import { createReadStream, existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PORT = 1350; // web/github.js kjenner tjeneren på porten
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const DIST = join(ROOT, "dist");
// Godkjenningssiden, bygd med `sh tools/dom.sh` (tools/qa/review.py, `LOCAL`).
const DOM = join(ROOT, "target", "qa", "dom");
const WRITABLE = /^assets\/(layout|levels)\/[A-Za-z0-9_\-/æøåÆØÅ]+\.json$/;
const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".mp4": "video/mp4",
  ".mp3": "audio/mpeg",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".ttf": "font/ttf",
  ".gz": "application/gzip",
};

if (process.argv.includes("--bygg")) execFileSync("sh", ["tools/build-web.sh"], { cwd: ROOT, stdio: "inherit" });
if (!existsSync(join(DIST, "pkg"))) {
  console.error("dist/ mangler. Kjør `node tools/rediger.mjs --bygg` (bygger nettversjonen først, noen minutter).");
  process.exit(1);
}

// En fil i repoet som redigeringen får lese og skrive, eller null.
function repoFile(path) {
  if (typeof path !== "string" || !WRITABLE.test(path) || path.includes("..")) return null;
  return join(ROOT, path);
}

async function body(req) {
  let text = "";
  for await (const chunk of req) text += chunk;
  return JSON.parse(text);
}

// Dommene fra godkjenningssiden: GET gir svarene som er gitt siden siden ble
// bygd (id → svar), POST fører ett svar rett inn i approvals.json med de
// samme reglene som før (`python3 -m tools.qa approvals svar`).
function verdicts(req, res, input) {
  const file = join(DOM, "svar.json");
  if (req.method === "GET") {
    const saved = existsSync(file) ? JSON.parse(readFileSync(file, "utf8")) : {};
    const out = Object.fromEntries(Object.entries(saved).map(([id, v]) => [id, v.data]));
    return res.writeHead(200, { "Content-Type": TYPES[".json"], "Cache-Control": "no-store" }).end(JSON.stringify(out));
  }
  if (!/^[tmu]_[A-Za-z0-9_]+$/.test(input.id ?? "") || typeof input.data !== "object") return res.writeHead(400).end("Ugyldig svar.");
  const venv = join(ROOT, "target", "qa", "venv", "bin", "python");
  const out = execFileSync(existsSync(venv) ? venv : "python3", ["-m", "tools.qa", "approvals", "svar"], { cwd: ROOT, input: JSON.stringify(input) });
  console.log(`Dom over ${input.id}: ${input.data.verdict || "punkt uten dom"} (${String(out).trim()})`);
  res.writeHead(200, { "Content-Type": TYPES[".json"] }).end("{}");
}

// `/lokal/les` gir filene som finnes, `/lokal/lagre` skriver dem som er endret.
async function local(req, res) {
  // Bare siden selv får lese og skrive, ikke en annen nettside som er åpen.
  const own = [`http://localhost:${PORT}`, `http://127.0.0.1:${PORT}`];
  if (req.url === "/lokal/dom" && req.method === "GET" && req.headers["sec-fetch-site"] !== "cross-site") return verdicts(req, res);
  if (req.method !== "POST" || !own.includes(req.headers.origin) || !(req.headers["content-type"] || "").startsWith("application/json")) {
    return res.writeHead(403).end("Bare redigeringen på denne maskinen.");
  }
  const input = await body(req);
  let out;
  if (req.url === "/lokal/dom") return verdicts(req, res, input);
  if (req.url === "/lokal/les") {
    const files = [];
    for (const path of input.paths ?? []) {
      const file = repoFile(path);
      if (file && existsSync(file)) files.push({ path, text: readFileSync(file, "utf8") });
    }
    out = { files };
  } else if (req.url === "/lokal/lagre") {
    const bad = (input.files ?? []).filter((f) => !repoFile(f.path) || typeof f.text !== "string");
    if (bad.length) return res.writeHead(400).end(`Kan ikke skrive ${bad.map((f) => f.path).join(", ")}`);
    let changed = 0;
    for (const f of input.files ?? []) {
      const file = repoFile(f.path);
      if (existsSync(file) && readFileSync(file, "utf8") === f.text) continue;
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, f.text);
      console.log(`Lagret ${f.path}`);
      changed++;
    }
    out = { changed };
  } else return res.writeHead(404).end();
  res.writeHead(200, { "Content-Type": TYPES[".json"] }).end(JSON.stringify(out));
}

const server = createServer(async (req, res) => {
  try {
    if (req.url.startsWith("/lokal/")) return await local(req, res);
    let path = normalize(decodeURIComponent(req.url.split("?")[0]));
    if (path === "/dom") return res.writeHead(302, { Location: "/dom/" }).end();
    if (path.endsWith("/")) path += "index.html";
    // Godkjenningssiden ligger utenfor dist/ (den er ikke en del av spillet).
    const [base, rel] = path.startsWith("/dom/") ? [DOM, path.slice(4)] : [DIST, path];
    const file = join(base, rel);
    if (!file.startsWith(base) || !existsSync(file) || !statSync(file).isFile()) {
      return res.writeHead(404).end(base === DOM ? "Godkjenningssiden er ikke bygd. Kjør `sh tools/dom.sh`." : "");
    }
    const size = statSync(file).size;
    const head = { "Content-Type": TYPES[extname(file)] || "application/octet-stream", "Accept-Ranges": "bytes", "Cache-Control": "no-cache" };
    // Filmen og lyden hentes i biter, så det går an å hoppe i dem.
    const range = /bytes=(\d+)-(\d*)/.exec(req.headers.range || "");
    if (range) {
      const start = Number(range[1]);
      const end = range[2] ? Math.min(Number(range[2]), size - 1) : size - 1;
      res.writeHead(206, { ...head, "Content-Range": `bytes ${start}-${end}/${size}`, "Content-Length": end - start + 1 });
      return createReadStream(file, { start, end }).pipe(res);
    }
    res.writeHead(200, { ...head, "Content-Length": size });
    createReadStream(file).pipe(res);
  } catch (e) {
    console.error(e);
    if (!res.headersSent) res.writeHead(500);
    res.end(String(e.message));
  }
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`Fristad lokalt (Ctrl+C stopper):
  Nivåene i krypten:  http://localhost:${PORT}/?nivaa
  Byen og husene:     http://localhost:${PORT}/?edit
  Spillet:            http://localhost:${PORT}/
  Godkjenningssiden:  http://localhost:${PORT}/dom/
Lagre (Ctrl+S) skriver rett i assets/layout/ og assets/levels/ i repoet,
og dommene på godkjenningssiden rett i assets/catalog/approvals.json.`);
});
