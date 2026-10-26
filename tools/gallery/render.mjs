// Rendrer LO-TEK-komponentene i galleriet (craft::GALLERY) slik boka viser
// dem: `demo`-HTML-en fra web/docs.json med LO-TEKs stilark, i lyst tema,
// skåret ned til standardvarianten, og tar et skarpt skjermbilde av hver
// (gjennomsiktig bakgrunn, beskåret til innholdet). Skaleringen velges per
// komponent (`scaleFor`: minst 4, høyst 12), så den lengste sida blir minst
// `TARGET` piksler og platen på 2048 aldri må forstørre bildet (en liten
// Badge rendres i 12×, et bredt Card i 5×). Så lager tools/gallery/pack.py
// kvadratiske 2048×2048-teksturer og kontaktarket. Kjør:
//   sh tools/pull-dist.sh      (lotek.css og komponent-JS-en i dist-web/)
//   PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright node tools/gallery/render.mjs [dist-web] [UTKATALOG]
//   python3 tools/gallery/pack.py UTKATALOG assets/textures/components shots/galleriet/kontaktark.png
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import { extname, join } from "node:path";

const [dir = "dist-web", out = "target/gallery"] = process.argv.slice(2);
const docs = JSON.parse(await readFile("web/docs.json", "utf8"));

// Samme rekkefølge og navn som `craft::GALLERY`. `keep` er hva av demoen som
// er standardvarianten (CSS-velgere på barna av flaten, i rekkefølge); tom
// liste er hele demoen. `prep` kjøres i siden før bildet tas.
const GALLERY = {
  "button": { keep: [".lt-button:not([data-variant])"] },
  "tag": { keep: [".lt-tag:not([data-variant]):not([aria-disabled])"] },
  "badge": { keep: [".lt-badge:not([data-color])"] },
  "divider": { keep: ["p:nth-of-type(1)", ".lt-divider:not([data-variant])", "p:nth-of-type(2)"] },
  "card": { keep: [".lt-card:not([data-variant])"] },
  "alert": { keep: [".lt-alert:not([data-color])"] },
  "progress": { keep: [".lt-progress:not([data-color])"] },
  "label": { keep: ["label[for=navn]", "#navn"] },
  "help-text": { keep: [".lt-help-text:not([data-variant])"] },
  "tooltip": { keep: [], css: ".lt-tooltip__bubble { visibility: visible !important; opacity: 1 !important; }", padTop: 56 },
  "fieldset": { keep: [] },
  "file-upload": { keep: [] },
  "search": { keep: [] },
  "dialog": { keep: ["lt-dialog"], modal: "dialog.lt-dialog", css: ".lt-dialog { max-inline-size: 20rem; }" },
  "tabs": { keep: [] },
};

// Smalere enn i boka (demoene er opptil 34rem), så teksten blir stor nok til
// å leses på en plate på 43 cm på en meters hold (15rem, eieren 18. oktober:
// «uklare»); standardvarianten er den samme.
const WIDTH = "15rem";

const FLATE = `
  html, body { margin: 0; background: transparent; }
  body { font-family: "Inter", system-ui, sans-serif; padding: 40px; }
  .flate { display: flex; gap: var(--lt-spacing-3); padding: var(--lt-spacing-6);
    max-width: var(--lt-preview-max-width, none); width: max-content;
    background: transparent; color: var(--lt-color-neutral-text-strong);
    font-size: var(--lt-font-size-m); line-height: 1.6; }
  .flate[data-layout="rad"] { flex-wrap: wrap; align-items: center; }
  .flate[data-layout="stabel"] { flex-direction: column; align-items: stretch; width: auto; }
  .flate > .lt-divider { margin-inline: 0; }
`;

const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".woff2": "font/woff2" };
let current = "";
const server = createServer(async (req, res) => {
  if (req.url.startsWith("/__gallery")) { res.writeHead(200, { "content-type": "text/html" }); res.end(current); return; }
  const p = join(dir, decodeURIComponent(req.url.split("?")[0]));
  let body;
  try { body = await readFile(p); } catch { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": types[extname(p)] ?? "application/octet-stream" });
  res.end(body);
});
await new Promise((r) => server.listen(0, r));
const base = `http://127.0.0.1:${server.address().port}`;
await mkdir(out, { recursive: true });
const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", headless: true, args: ["--lang=nb"] });
// Den lengste sida i bildet (piksler): innholdet fyller 80 % av platen på
// 2048 i pack.py, altså rundt 1640, og litt til for kanten.
const TARGET = 1800;
const scaleFor = (w, h) => Math.min(12, Math.max(4, Math.ceil(TARGET / Math.max(w, h))));
const pages = new Map();
async function pageAt(scale) {
  if (!pages.has(scale)) {
    const page = await browser.newPage({ viewport: { width: 900, height: 700 }, deviceScaleFactor: scale, colorScheme: "light", locale: "nb-NO" });
    page.on("pageerror", (e) => console.error("side:", e.message));
    page.on("console", (m) => { if (m.type() === "error") console.error("konsoll:", m.text()); });
    pages.set(scale, page);
  }
  return pages.get(scale);
}
const meta = {};
for (const [name, spec] of Object.entries(GALLERY)) {
  const demo = docs[name]?.demo;
  if (!demo?.html) { console.error(`${name}: ingen demo i web/docs.json`); process.exitCode = 1; continue; }
  current = `<!doctype html><html lang="nb" data-theme="light"><head><meta charset="utf-8">
    <link rel="stylesheet" href="/lotek.css"><style>${FLATE}${demo.css ?? ""}${spec.css ?? ""}</style></head>
    <body><div class="flate" data-layout="${demo.layout || "rad"}" style="--lt-preview-max-width:${WIDTH}">${demo.html}</div>
    <script type="module">
      import { defineLtTabs } from "/lotek/components/frame/tabs/lt-tabs.js";
      import { defineLtDialog } from "/lotek/components/frame/dialog/lt-dialog.js";
      defineLtTabs(); defineLtDialog(); window.__klar = true;
    </script></body></html>`;
  const shoot = async (page) => {
    await page.goto(`${base}/__gallery?${name}`, { waitUntil: "load" });
    await page.waitForFunction(() => window.__klar === true);
    await page.evaluate(({ keep }) => {
      const flate = document.querySelector(".flate");
      if (keep.length) {
        const chosen = keep.map((s) => flate.querySelector(`:scope > ${s}`)).filter(Boolean);
        for (const el of [...flate.children]) if (!chosen.includes(el)) el.remove();
      }
    }, { keep: spec.keep });
    if (spec.modal) await page.evaluate((s) => { document.querySelector(s).showModal(); document.activeElement?.blur(); }, spec.modal);
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(250);
    // Innholdets rammer (uten flatens polstring), til beskjæringen.
    const box = await page.evaluate(({ modal }) => {
      const els = modal ? [document.querySelector(modal)] : [...document.querySelector(".flate").children, ...document.querySelectorAll(".lt-tooltip__bubble")];
      let x0 = 1e9, y0 = 1e9, x1 = -1e9, y1 = -1e9;
      for (const el of els) {
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.height === 0) continue;
        x0 = Math.min(x0, r.left); y0 = Math.min(y0, r.top); x1 = Math.max(x1, r.right); y1 = Math.max(y1, r.bottom);
      }
      const bg = getComputedStyle(document.querySelector(".flate")).backgroundColor;
      return { x: x0, y: y0, w: x1 - x0, h: y1 - y0, bg };
    }, { modal: spec.modal });
    const m = spec.modal ? 2 : 10;
    const clip = { x: Math.max(0, box.x - m), y: Math.max(0, box.y - m), width: box.w + 2 * m, height: box.h + 2 * m };
    return { box, clip };
  };
  // Først i 4× for å måle, så på nytt i skaleringen som gir TARGET.
  let { box, clip } = await shoot(await pageAt(4));
  const scale = scaleFor(clip.width, clip.height);
  const page = await pageAt(scale);
  if (scale !== 4) ({ box, clip } = await shoot(page));
  // Gjennomsiktig bakgrunn: pack.py legger komponenten på pergament.
  await page.screenshot({ path: join(out, `${name}.png`), clip, omitBackground: true });
  meta[name] = { ...box, clip, scale, padTop: spec.padTop ?? 0 };
  console.log(`${name}: ${Math.round(box.w)}×${Math.round(box.h)} px i ${scale}× (bakgrunn ${box.bg})`);
}
await writeFile(join(out, "meta.json"), JSON.stringify(meta, null, 1));
await browser.close();
server.close();
