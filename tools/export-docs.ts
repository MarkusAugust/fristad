// Henter dokumentasjonen for komponentene i Fristad fra Fristil sine
// MDX-sider, og skriver det panelet trenger til web/docs.json.
//
//   bun tools/export-docs.ts                       henter fra GitHub
//   bun tools/export-docs.ts ../fristil            leser fra en lokal klone
//
// Panelet viser beskrivelsen, ingressen og eksempelkoden fra siden, så
// teksten i byen er den samme som på dokumentasjonssiden.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** Komponentene som finnes i Fristad så langt. */
const COMPONENTS = ["button", "alert", "tag", "badge", "switch", "spinner", "breadcrumbs", "progress", "toast", "skeleton", "connection-status", "dialog", "tabs", "card", "accordion", "input", "select", "checkbox", "radio", "textarea", "table", "tooltip", "link", "avatar", "divider"];

/// Veiledningssidene i roten av dokumentasjonen, som Verkstedet viser.
const PAGES = ["kom-i-gang", "typesikker-bruk", "tilgjengelighet"];

const local = process.argv[2];
const RAW = "https://raw.githubusercontent.com/MarkusAugust/fristil/master";
const PATH = "documentation/src/content/docs/components";

async function source(name: string, dir = PATH): Promise<string> {
  if (local) return readFileSync(join(local, dir, `${name}.mdx`), "utf8");
  const res = await fetch(`${RAW}/${dir}/${name}.mdx`);
  if (!res.ok) throw new Error(`${name}.mdx: HTTP ${res.status}`);
  return res.text();
}

function parse(name: string, mdx: string, page = false) {
  const front = mdx.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? "";
  const field = (k: string) =>
    front.match(new RegExp(`^${k}:\\s*(.+)$`, "m"))?.[1].trim() ?? "";
  const body = mdx.slice(mdx.indexOf("---", 3) + 3);

  // Ingressen er første vanlige avsnitt før første overskrift.
  const intro =
    body
      .split(/\n## /)[0]
      .split(/\n\s*\n/)
      .map((p) => p.trim())
      .find((p) => p && !p.startsWith("import") && !p.startsWith("<")) ?? "";

  // Eksempelkoden står i `kode={`...`}` i første <Eksempel>. Rammekomponentene
  // (tabs, dialog) viser den i stedet i en ```html-blokk med markupen; skript
  // og kommentarer tas bort, siden panelet selv registrerer komponentene.
  let code = body.match(/kode=\{`\n?([\s\S]*?)`\}/)?.[1].trim() ?? "";
  if (!code && page) {
    // Veiledningssidene: de første kodeblokkene, uansett språk.
    code = [...body.matchAll(/```[a-z]*\n([\s\S]*?)```/g)]
      .slice(0, 2)
      .map((m) => m[1].trim())
      .join("\n\n");
  }
  if (!code) {
    const fences = [...body.matchAll(/```html\n([\s\S]*?)```/g)].map((m) => m[1]);
    const markup = fences.find((f) => /class="fs-|<fs-/.test(f) && !/<link /.test(f)) ?? "";
    code = markup
      .replace(/<script[\s\S]*?<\/script>/g, "")
      .replace(/<!--[\s\S]*?-->/g, "")
      .split("\n")
      .map((l) => l.replace(/^ {4}/, ""))
      .join("\n")
      .replace(/\n{3,}/g, "\n\n")
      .trim();
  }

  return {
    name,
    title: field("title"),
    description: field("description"),
    intro,
    code,
    url: page ? `https://fristil.sobernetics.no/${name}/` : `https://fristil.sobernetics.no/components/${name}/`,
  };
}

const docs: Record<string, ReturnType<typeof parse>> = {};
for (const name of COMPONENTS) docs[name] = parse(name, await source(name));
for (const name of PAGES) docs[name] = parse(name, await source(name, "documentation/src/content/docs"), true);

writeFileSync("web/docs.json", JSON.stringify(docs, null, 2) + "\n");
console.log(`Dokumentasjon for ${COMPONENTS.length} komponent(er) og ${PAGES.length} side(r) skrevet til web/docs.json`);
