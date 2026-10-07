// Rendrer komponentene i tools/sheets.mjs med Fristils egen CSS i Chromium
// uten skjerm, pakker dem i ett atlas per tema og skriver
// assets/sheets/light.png, dark.png og index.json. Kjør:
//   PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright node tools/render-sheets.mjs
// Skalaen er 2 atlaspiksler per CSS-piksel; i motoren er 1 CSS-px = 1 cm.
import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { SHEETS } from "./sheets.mjs";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
const SCALE = 2;
const SIZE = 4096;
const PAD = 2;
const css = readFileSync("node_modules/@fristil/designsystem/dist/fristil.css", "utf8");

const page_html = (theme) => `<!doctype html><html lang="nb" data-theme="${theme}"><head><meta charset="utf-8"><style>${css}
html, body { margin: 0; background: transparent; }
body { font-family: Inter, system-ui, sans-serif; }
.sheet { position: absolute; display: inline-block; }
.sheet > * { margin: 0; }
.fs-link--visited { color: var(--fs-color-visited-text); }
.fs-table[data-hoverable] tbody tr[data-hover] { background: var(--fs-color-accent-surface); }
</style></head><body></body></html>`;

const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", headless: true });
const page = await browser.newPage({ viewport: { width: Math.floor(SIZE / SCALE), height: Math.floor(SIZE / SCALE) }, deviceScaleFactor: SCALE });
await page.setContent(page_html("light"));
await page.evaluate((sheets) => {
  for (const s of sheets) {
    const d = document.createElement("div");
    d.className = "sheet";
    d.dataset.id = s.id;
    d.innerHTML = s.html;
    if (s.zoom) d.style.zoom = String(s.zoom);
    if (s.indeterminate) d.querySelector("input").indeterminate = true;
    document.body.append(d);
  }
}, SHEETS);
await page.evaluate(() => document.fonts.ready);
// Målene i CSS-px, så pakking på rader.
const sizes = await page.evaluate(() =>
  [...document.querySelectorAll(".sheet")].map((d) => {
    const r = d.getBoundingClientRect();
    // Et ark som CSS-en skjuler (tooltip-bobla uten hover) ville blitt en
    // tom plate i byen; stopp her i stedet.
    const el = d.firstElementChild;
    const cs = el && getComputedStyle(el);
    // En transform flytter arket ut av ruta si i atlaset uten å endre målene.
    // Rotasjon (spinneren) er greit; forskyvning (tx, ty i matrisen) er ikke.
    const m = cs && cs.transform !== "none" ? cs.transform.match(/matrix\(([^)]*)\)/) : null;
    const shifted = m ? m[1].split(",").slice(4).some((v) => Math.abs(Number(v)) > 0.5) : false;
    const hidden = !cs || cs.visibility === "hidden" || cs.display === "none" || Number(cs.opacity) === 0 || r.width < 1 || r.height < 1 || shifted;
    return { id: d.dataset.id, w: Math.ceil(r.width), h: Math.ceil(r.height), hidden };
  }));
const invisible = sizes.filter((s) => s.hidden).map((s) => s.id);
if (invisible.length) throw new Error(`ark som er usynlige eller forskjøvet i CSS-en: ${invisible.join(", ")}`);
const order = sizes.slice().sort((a, b) => b.h - a.h);
const W = Math.floor(SIZE / SCALE);
let x = PAD, y = PAD, row = 0;
const place = {};
for (const s of order) {
  if (x + s.w + PAD > W) {
    x = PAD;
    y += row + PAD;
    row = 0;
  }
  place[s.id] = { x, y, w: s.w, h: s.h };
  x += s.w + PAD;
  row = Math.max(row, s.h);
}
const used = y + row + PAD;
if (used > W) throw new Error(`atlaset er fullt: ${used} > ${W} CSS-px`);
const index = { scale: SCALE, size: SIZE, sheets: {} };
for (const [id, p] of Object.entries(place)) {
  index.sheets[id] = { x: p.x * SCALE, y: p.y * SCALE, w: p.w * SCALE, h: p.h * SCALE };
}
for (const theme of ["light", "dark"]) {
  await page.setContent(page_html(theme));
  await page.evaluate(({ sheets, place }) => {
    for (const s of sheets) {
      const d = document.createElement("div");
      d.className = "sheet";
      d.innerHTML = s.html;
      if (s.zoom) d.style.zoom = String(s.zoom);
      if (s.indeterminate) d.querySelector("input").indeterminate = true;
      // `zoom` skalerer også left/top, så posisjonen gis i zoomede enheter.
      const p = place[s.id];
      const z = s.zoom ?? 1;
      d.style.left = `${p.x / z}px`;
      d.style.top = `${p.y / z}px`;
      document.body.append(d);
    }
  }, { sheets: SHEETS, place });
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(200);
  await page.screenshot({ path: `assets/sheets/${theme}.png`, omitBackground: true, clip: { x: 0, y: 0, width: W, height: W } });
  console.log(`assets/sheets/${theme}.png: ${sizes.length} komponenter, ${Math.round(used * SCALE)} av ${SIZE} px brukt`);
}
writeFileSync("assets/sheets/index.json", JSON.stringify(index));
await browser.close();
