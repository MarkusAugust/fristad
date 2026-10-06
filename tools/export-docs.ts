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
const COMPONENTS = ["button"];

const local = process.argv[2];
const RAW = "https://raw.githubusercontent.com/MarkusAugust/fristil/main";
const PATH = "documentation/src/content/docs/components";

async function source(name: string): Promise<string> {
  if (local) return readFileSync(join(local, PATH, `${name}.mdx`), "utf8");
  const res = await fetch(`${RAW}/${PATH}/${name}.mdx`);
  if (!res.ok) throw new Error(`${name}.mdx: HTTP ${res.status}`);
  return res.text();
}

function parse(name: string, mdx: string) {
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

  // Eksempelkoden står i `kode={`...`}` i første <Eksempel>.
  const code = body.match(/kode=\{`\n?([\s\S]*?)`\}/)?.[1].trim() ?? "";

  return {
    name,
    title: field("title"),
    description: field("description"),
    intro,
    code,
    url: `https://fristil.sobernetics.no/components/${name}/`,
  };
}

const docs: Record<string, ReturnType<typeof parse>> = {};
for (const name of COMPONENTS) docs[name] = parse(name, await source(name));

writeFileSync("web/docs.json", JSON.stringify(docs, null, 2) + "\n");
console.log(`Dokumentasjon for ${COMPONENTS.length} komponent(er) skrevet til web/docs.json`);
