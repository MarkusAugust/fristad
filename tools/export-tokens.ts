// Leser LO-TEK sine tokens fra pakken og skriver dem som JSON for motoren.
//
// Motoren er skrevet i Rust og kan ikke lese CSS. Dette skriptet er broen:
// det henter fargene for lyst og mørkt tema og spacing-skalaen, og legger
// dem i assets/tokens.json, som Rust-koden bygger inn ved kompilering.
// Oppdateres LO-TEK, kjøres skriptet på nytt og byen får ny palett.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";

const css = readFileSync(
  "node_modules/@lo-tek/interfaces/src/tokens/tokens.css",
  "utf8",
);

/** Henter innholdet i blokken som starter med `selector {`. */
function block(selectorStart: string): string {
  const at = css.indexOf(selectorStart);
  if (at < 0) throw new Error(`Fant ikke ${selectorStart} i tokens.css`);
  const open = css.indexOf("{", at);
  let depth = 0;
  for (let i = open; i < css.length; i++) {
    if (css[i] === "{") depth++;
    if (css[i] === "}") depth--;
    if (depth === 0) return css.slice(open + 1, i);
  }
  throw new Error("Ubalanserte klammer");
}

function vars(body: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/--lt-([a-z0-9-]+):\s*([^;]+);/g)) {
    out[m[1]] = m[2].trim();
  }
  return out;
}

const root = vars(block(":root {"));
const light = { ...root, ...vars(block(`
  [data-theme="light"],`)) };
const dark = { ...root, ...vars(block(`
  [data-theme="dark"],`)) };

const colors = (t: Record<string, string>) =>
  Object.fromEntries(
    Object.entries(t).filter(
      ([k, v]) => k.startsWith("color-") && /^#[0-9a-f]{6}$/i.test(v),
    ),
  );

const remToPx = (v: string) =>
  v.endsWith("rem") ? parseFloat(v) * 16 : v.endsWith("px") ? parseFloat(v) : NaN;

const spacing = Object.fromEntries(
  Object.entries(root)
    .filter(([k]) => k.startsWith("spacing-"))
    .map(([k, v]) => [k.slice("spacing-".length), remToPx(v)]),
);

const fontSize = Object.fromEntries(
  Object.entries(root)
    .filter(([k]) => k.startsWith("font-size-") && k !== "font-size-reference")
    .map(([k, v]) => [k.slice("font-size-".length), remToPx(v)]),
);
const lineHeight = Object.fromEntries(
  Object.entries(root)
    .filter(([k]) => k.startsWith("line-height-"))
    .map(([k, v]) => [k.slice("line-height-".length), parseFloat(v)]),
);

const pkg = JSON.parse(
  readFileSync("node_modules/@lo-tek/interfaces/package.json", "utf8"),
);

const out = {
  lotek: pkg.version,
  light: colors(light),
  dark: colors(dark),
  spacing,
  font_size: fontSize,
  line_height: lineHeight,
};

mkdirSync("assets", { recursive: true });
writeFileSync("assets/tokens.json", JSON.stringify(out, null, 2) + "\n");
console.log(
  `LO-TEK ${pkg.version}: ${Object.keys(out.light).length} farger (lys), ` +
    `${Object.keys(out.dark).length} (mørk), ${Object.keys(spacing).length} spacing-trinn, ${Object.keys(fontSize).length} skriftstørrelser`,
);
