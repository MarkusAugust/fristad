// Henter dokumentasjonen for komponentene i Fristad fra Fristil sine
// MDX-sider, og skriver det panelet trenger til web/docs.json.
//
//   bun tools/export-docs.ts                       henter fra GitHub
//   bun tools/export-docs.ts ../fristil            leser fra en lokal klone
//
// Panelet viser beskrivelsen, ingressen og eksempelkoden fra siden, så
// teksten i byen er den samme som på dokumentasjonssiden. Forhåndsvisningen
// (`demo`) er den første på siden, med flaten slik Fristil-docs viser den:
// markupen, `layout`, `maxWidth` og CSS-en som bare gjelder visningen.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fs } from "@fristil/designsystem";

/** Komponentene som finnes i Fristad så langt. */
const COMPONENTS = ["button", "alert", "tag", "badge", "switch", "spinner", "breadcrumbs", "progress", "toast", "skeleton", "connection-status", "dialog", "tabs", "card", "accordion", "input", "select", "checkbox", "radio", "textarea", "table", "tooltip", "link", "avatar", "divider", "heading", "label", "fieldset", "help-text", "error-text", "file-upload", "search"];

/// Veiledningssidene i roten av dokumentasjonen, som Verkstedet viser.
const PAGES = ["kom-i-gang", "typesikker-bruk", "tilgjengelighet"];

const local = process.argv[2];
const RAW = "https://raw.githubusercontent.com/MarkusAugust/fristil/master";
const PATH = "documentation/src/content/docs/components";
const DEMOS = "documentation/src/components";

async function read(path: string): Promise<string> {
  if (local) return readFileSync(join(local, path), "utf8");
  const res = await fetch(`${RAW}/${path}`);
  if (!res.ok) throw new Error(`${path}: HTTP ${res.status}`);
  return res.text();
}

const source = (name: string, dir = PATH) => read(`${dir}/${name}.mdx`);

// Et lite JSX-oppsett som gjør elementer om til HTML-tekst. Med det kan
// <Eksempel>-taggen og demokomponentene (<ToastDemo /> osv.) leses slik
// Astro leser dem, også `{...fs.tabs(...)}` og `.map(...)`.
type Html = { html: string };
const VOID = new Set(["area", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"]);
const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);
const text = (c: unknown): string =>
  c == null || c === false || c === true ? "" : Array.isArray(c) ? c.map(text).join("") : typeof c === "object" ? (c as Html).html : esc(String(c));

function h(tag: unknown, props: Record<string, unknown> | null, ...children: unknown[]): Html {
  if (typeof tag === "function") return tag({ ...props, children });
  const attrs = Object.entries(props ?? {})
    .filter(([, v]) => v != null && v !== false)
    .map(([k, v]) => (v === true ? ` ${k}` : ` ${k}="${esc(String(v))}"`))
    .join("");
  const name = String(tag);
  return { html: VOID.has(name) ? `<${name}${attrs}>` : `<${name}${attrs}>${text(children)}</${name}>` };
}
const Fragment = (p: { children: unknown[] }): Html => ({ html: text(p.children) });

/** Slutten på åpningstaggen og på hele JSX-elementet som begynner i `start`. */
function elementEnd(src: string, start: number, name: string): [number, number] {
  let depth = 0;
  for (let i = start; i < src.length; i++) {
    const c = src[i];
    if (depth > 0 && (c === '"' || c === "'" || c === "`")) {
      for (i++; i < src.length && src[i] !== c; i++) if (src[i] === "\\") i++;
    } else if (depth === 0 && c === '"') {
      i = src.indexOf('"', i + 1);
    } else if (c === "{") depth++;
    else if (c === "}") depth--;
    else if (depth === 0 && src.startsWith("/>", i)) return [i + 2, i + 2];
    else if (depth === 0 && c === ">") {
      const close = src.indexOf(`</${name}>`, i);
      return [i + 1, close < 0 ? i + 1 : close + name.length + 3];
    }
  }
  throw new Error(`fant ikke slutten på <${name}>`);
}

/** Kjører JSX-en med `scope` (frontmatter fra en .astro-fil) foran. */
function render(jsx: string, components: Record<string, Function>, scope = ""): Html {
  const body = `export function demo() {\n${scope}\nreturn (<>${jsx}</>);\n}`;
  const js = new Bun.Transpiler({
    loader: "tsx",
    tsconfig: { compilerOptions: { jsx: "react", jsxFactory: "h", jsxFragmentFactory: "Fragment" } },
  })
    .transformSync(body)
    .replace(/^export /m, "");
  const names = Object.keys(components);
  return new Function("h", "Fragment", "fs", ...names, `${js}\nreturn demo();`)(h, Fragment, fs, ...Object.values(components));
}

type Demo = { html: string; layout: string; maxWidth: string; css: string; fixed: boolean };

/** Den første forhåndsvisningen på siden: et <Eksempel> eller en <XDemo />. */
async function demo(body: string): Promise<Demo | null> {
  const eks = body.search(/^\s*<Eksempel\b/m);
  const own = body.match(/^\s*<([A-Z]\w*Demo)\s*\/>/m);
  if (eks < 0 && !own) return null;
  let found: Demo | null = null;
  if (own && (eks < 0 || (own.index ?? 0) < eks)) {
    // Demokomponenten: frontmatteren gir variablene, <Preview> flaten.
    const astro = await read(`${DEMOS}/${own[1]}.astro`);
    const [, front = "", tpl = ""] = astro.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/) ?? [];
    const scope = front.replace(/^import .*$/gm, "");
    const start = tpl.indexOf("<Preview");
    const jsx = tpl.slice(start, elementEnd(tpl, start, "Preview")[1]).replace(/<!--[\s\S]*?-->/g, "")
      // Astro slår mellomrom mellom taggene sammen til ett; JSX fjerner dem.
      .replace(/>\s*\n\s*</g, '>{" "}<');
    const Preview = (p: Record<string, any>) => {
      found = { html: text(p.children).trim(), layout: p.layout ?? "rad", maxWidth: p.maxWidth ?? "", css: (p.ekstraCss ?? "").trim(), fixed: !!p.fastInni };
      return { html: "" };
    };
    render(jsx, { Preview }, scope);
  } else {
    const start = body.indexOf("<Eksempel", eks);
    // Bare åpningstaggen: barna er MDX (fanene med TypeScript), ikke JSX.
    const open = elementEnd(body, start, "Eksempel")[0];
    const jsx = body.slice(start, open).replace(/\/?>$/, "/>");
    const Eksempel = (p: Record<string, any>) => {
      const css = [p.egenCss, p.visningsCss].filter(Boolean).join("\n").trim();
      found = { html: String(p.kode ?? "").trim(), layout: p.layout ?? "rad", maxWidth: p.maxWidth ?? "", css, fixed: false };
      return { html: "" };
    };
    render(jsx, { Eksempel });
  }
  return found;
}

async function parse(name: string, mdx: string, page = false) {
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
  const own = body.search(/^\s*<[A-Z]\w*Demo\s*\/>/m);
  if (!code || (own >= 0 && own < body.search(/^\s*<Eksempel\b/m))) {
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
    demo: page ? null : await demo(body),
    url: page ? `https://fristil.sobernetics.no/${name}/` : `https://fristil.sobernetics.no/components/${name}/`,
  };
}

const docs: Record<string, Awaited<ReturnType<typeof parse>>> = {};
for (const name of COMPONENTS) docs[name] = await parse(name, await source(name));
for (const name of PAGES) docs[name] = await parse(name, await source(name, "documentation/src/content/docs"), true);

writeFileSync("web/docs.json", JSON.stringify(docs, null, 2) + "\n");

// Flaten eksemplene vises på, rett fra Fristil-docs (Preview.astro): de
// samme layoutene (`rad`, `stabel`, `flyt` …), så panelet aldri mangler en
// som docs har fått (eieren 18. oktober: Divider sto i fire kolonner fordi
// panelet ikke kjente `flyt`).
const preview = await read(`${DEMOS}/Preview.astro`);
const flate = preview.match(/const stilark = \[\s*`([\s\S]*?)`/)?.[1];
if (!flate || !flate.includes(".flate")) throw new Error("Preview.astro: fant ikke stilarket til flaten");
writeFileSync("web/docs-flate.css", flate.replace(/^\t/gm, "").trim() + "\n");
console.log(`Dokumentasjon for ${COMPONENTS.length} komponent(er) og ${PAGES.length} side(r) skrevet til web/docs.json`);
