/**
 * Koblingen på en hel side: at hver id det pekes på finnes, og at hvert felt
 * og hver tekst er koblet.
 *
 * `diagnoseMarkup` sjekker ordforrådet: elementer, klasser og verdier. Den
 * sjekker ikke om `for` og `aria-describedby` peker på noe, og kan ikke: i en
 * mal som er delt i biter kan id-en stå i en annen fil. På en side slik
 * serveren sender den, er alt med, og da kan det kreves. Det er koblingen en
 * skjermleser lever av, og den en kodeagent oftest bommer på.
 *
 * Markup inni `<fs-field>` meldes ikke for manglende kobling: web-komponenten
 * setter den i nettleseren, og bar markup der er riktig. `<fs-suggestion>`
 * setter `for` på ledeteksten sin på samme måte, så et felt inni den meldes
 * heller ikke uten ledetekst. En id som er skrevet og peker feil, meldes
 * uansett hvor den står.
 */
import { readAttributes, tagEnd, } from "./diagnostics.js";
const LINK = "https://fristil.sobernetics.no/components/field/";
const SUMMARY_LINK = "https://fristil.sobernetics.no/components/error-summary/";
const HOW = "Lag koblingen med fs.field({ id }) der koden kan kalle en JavaScript-funksjon, " +
    "eller legg feltet i <fs-field>, som setter den i nettleseren.";
/** Attributtene som peker på id-er, og hva skjermleseren mister når de bommer. */
const REFERENCES = {
    for: "ledeteksten",
    "aria-describedby": "beskrivelsen",
    "aria-labelledby": "navnet",
    "aria-controls": "koblingen til det elementet styrer",
};
/** Kontroller som ikke trenger en ledetekst. */
const UNLABELLED_TYPES = new Set([
    "hidden",
    "submit",
    "button",
    "reset",
    "image",
]);
/** Elementene en `<label for>` kan peke på, etter HTML-standarden. */
const LABELABLE = new Set([
    "button",
    "input",
    "meter",
    "output",
    "progress",
    "select",
    "textarea",
]);
/** Elementer uten lukketagg. */
const VOID = new Set([
    "area",
    "base",
    "br",
    "col",
    "embed",
    "hr",
    "img",
    "input",
    "link",
    "meta",
    "param",
    "source",
    "track",
    "wbr",
]);
const value = (node, name) => node.attributes.find((a) => a.name === name);
const hasClass = (node, test) => (value(node, "class")?.value ?? "").split(/\s+/).some(test);
/** Noden selv eller en forfar som oppfyller vilkåret. */
function within(node, test) {
    for (let n = node; n; n = n.parent)
        if (test(n))
            return true;
    return false;
}
/** Om nettleseren skjuler noden, selv eller gjennom en forfar. */
const isHidden = (node) => within(node, (n) => value(n, "hidden") !== undefined ||
    (n.name === "dialog" && value(n, "open") === undefined) ||
    (n.name === "details" &&
        n !== node &&
        value(n, "open") === undefined &&
        node.name !== "summary"));
const ENTITIES = {
    amp: "&",
    lt: "<",
    gt: ">",
    quot: '"',
    apos: "'",
    nbsp: " ",
};
/** Et tegn fra et tall i en entitet, eller entiteten som den sto når tallet ikke er et tegn. */
const fromCode = (code, hit) => Number.isInteger(code) && code > 0 && code <= 0x10ffff
    ? String.fromCodePoint(code)
    : hit;
/** En attributtverdi slik nettleseren leser den, med entitetene dekodet. */
function decode(text) {
    return text.replace(/&(#x[0-9a-f]+|#[0-9]+|[a-z]+);/gi, (hit, code) => {
        const lower = code.toLowerCase();
        if (lower.startsWith("#x"))
            return fromCode(Number.parseInt(lower.slice(2), 16), hit);
        if (lower.startsWith("#"))
            return fromCode(Number.parseInt(lower.slice(1), 10), hit);
        return ENTITIES[lower] ?? hit;
    });
}
/**
 * Id-en en lenke som `href="#f%C3%B8dselsdato"` går til. Nettleseren
 * dekoder prosentkodingen før den leter etter id-en.
 */
function fragment(href) {
    try {
        return decode(decodeURIComponent(href));
    }
    catch {
        return decode(href);
    }
}
/** Bytter hvert tegn med mellomrom, unntatt linjeskift, så posisjonene står. */
const blank = (hit) => hit.replace(/[^\n]/g, " ");
const TAG = /<(\/?)([a-z][a-z0-9-]*)(?=[\s/>])/gi;
/**
 * Hver tagg på siden, i rekkefølge. Lesingen hopper til der taggen slutter, så
 * en `<` inne i en attributtverdi ikke leses som en ny tagg. Det er et ekstra
 * vern: `pageSource` har alt blanket ut slike verdier, og ingen test feller
 * om hoppet fjernes.
 */
function* scan(source) {
    const pattern = new RegExp(TAG);
    for (let hit = pattern.exec(source); hit; hit = pattern.exec(source)) {
        const start = hit.index;
        const end = tagEnd(source, start + hit[0].length);
        if (end < 0)
            return;
        yield { closing: hit[1] === "/", name: hit[2].toLowerCase(), start, end };
        pattern.lastIndex = end + 1;
    }
}
/** Elementer der innholdet er tekst, ikke elementer på siden. */
const RAW_TEXT = new Set(["script", "style", "textarea", "template"]);
/**
 * Siden slik den leses som en hel side, med samme lengde: uten kommentarer,
 * uten innholdet i `<script>`, `<style>`, `<textarea>` og `<template>`, og
 * uten markup som står i en attributtverdi. Alt dette er tekst for
 * nettleseren, ikke elementer på siden.
 *
 * Innholdet i en `<template>` er ikke i dokumentet. Er den en shadow root
 * (`shadowrootmode`), har den sine egne id-er, og en id der kan verken
 * kollidere med eller pekes på fra siden utenfor. Den sjekkes derfor ikke.
 *
 * Siden leses én gang, fra start til slutt, tagg for tagg, og ingenting
 * blankes med et regulært uttrykk over hele teksten. To ganger løp et slikt
 * uttrykk over taggrenser og slukte ekte markup: fra en attributtverdi som
 * sluttet på `=`, og fra en `data-code="<textarea>"` til neste ekte
 * `</textarea>`. Her er en `<` i en attributtverdi aldri starten på noe.
 */
export function pageSource(text) {
    const ranges = [];
    const tagAt = /<(\/?)([a-z][a-z0-9-]*)(?=[\s/>])/iy;
    let i = 0;
    while (i < text.length) {
        const lt = text.indexOf("<", i);
        if (lt < 0)
            break;
        if (text.startsWith("<!--", lt)) {
            const close = text.indexOf("-->", lt + 4);
            const end = close < 0 ? text.length : close + 3;
            ranges.push([lt, end]);
            i = end;
            continue;
        }
        tagAt.lastIndex = lt;
        const hit = tagAt.exec(text);
        if (!hit) {
            i = lt + 1;
            continue;
        }
        const end = tagEnd(text, lt + hit[0].length);
        if (end < 0)
            break;
        i = end + 1;
        if (hit[1] === "/")
            continue;
        const nameEnd = lt + hit[0].length;
        for (const attribute of readAttributes(text.slice(nameEnd, end), nameEnd))
            if (attribute.value?.includes("<"))
                ranges.push([
                    attribute.valueStart,
                    attribute.valueStart + attribute.value.length,
                ]);
        const name = hit[2].toLowerCase();
        if (RAW_TEXT.has(name)) {
            const closer = new RegExp(`</${name}\\s*>`, "gi");
            closer.lastIndex = end + 1;
            const close = closer.exec(text)?.index ?? text.length;
            ranges.push([end + 1, close]);
            i = close;
        }
    }
    let result = "";
    let from = 0;
    for (const [start, end] of ranges) {
        result += text.slice(from, start) + blank(text.slice(start, end));
        from = end;
    }
    return result + text.slice(from);
}
/**
 * Om taggen avsluttes med `/>`. En `/` til slutt i en verdi uten
 * anførselstegn, som `href=/sok/`, hører til verdien.
 */
const selfClosing = (body) => /(^|[\s"'])\/$/.test(body);
/** Elementene på siden som et tre, i den rekkefølgen de står. */
function tree(source) {
    const nodes = [];
    const stack = [];
    for (const tag of scan(source)) {
        if (tag.closing) {
            // Lukker det nærmeste åpne elementet med samme navn, og alt som står
            // åpent inni det, slik nettleseren gjør med en tagg som mangler.
            let at = stack.length - 1;
            while (at >= 0 && stack[at].name !== tag.name)
                at--;
            if (at >= 0)
                stack.length = at;
            continue;
        }
        const nameStart = tag.start + 1;
        const nameEnd = nameStart + tag.name.length;
        const body = source.slice(nameEnd, tag.end);
        const closes = selfClosing(body);
        const node = {
            name: tag.name,
            nameStart,
            nameEnd,
            attributes: readAttributes(closes ? body.slice(0, -1) : body, nameEnd),
            parent: stack[stack.length - 1],
        };
        nodes.push(node);
        // I HTML lukker `/>` bare i SVG og MathML. `<div/>` åpner et element.
        const foreign = tag.name === "svg" ||
            tag.name === "math" ||
            stack.some((n) => n.name === "svg" || n.name === "math");
        if (!VOID.has(tag.name) && !(closes && foreign))
            stack.push(node);
    }
    return nodes;
}
/** Alle funn om koblingen på siden, i den rekkefølgen de står. */
export function checkReferences(text) {
    const nodes = tree(pageSource(text));
    const findings = [];
    const ids = new Map();
    const labelFor = new Set();
    const describedBy = new Set();
    for (const node of nodes) {
        const id = value(node, "id");
        if (id?.value) {
            const decoded = decode(id.value);
            if (ids.has(decoded)) {
                findings.push({
                    start: id.valueStart,
                    end: id.valueStart + id.value.length,
                    severity: "error",
                    link: LINK,
                    message: `id="${id.value}" står mer enn én gang på siden. for og ` +
                        "aria-describedby peker da på det første elementet, og koblingen " +
                        "til dette er brutt. Hver id må være unik.",
                });
            }
            else
                ids.set(decoded, node);
        }
        if (node.name === "label") {
            const target = value(node, "for")?.value;
            if (target)
                labelFor.add(decode(target));
        }
        for (const part of decode(value(node, "aria-describedby")?.value ?? "").split(/\s+/))
            if (part)
                describedBy.add(part);
    }
    for (const node of nodes) {
        // Hver id det pekes på, må finnes.
        for (const [name, loses] of Object.entries(REFERENCES)) {
            const attribute = value(node, name);
            if (!attribute?.value)
                continue;
            for (const token of attribute.value.matchAll(/\S+/g)) {
                const target = token[0];
                const at = attribute.valueStart + (token.index ?? 0);
                const found = ids.get(decode(target));
                if (!found) {
                    findings.push({
                        start: at,
                        end: at + target.length,
                        severity: "error",
                        link: LINK,
                        message: `${name}="${attribute.value}" peker på id-en «${target}», som ` +
                            `ikke finnes på siden. Skjermleseren mister ${loses}. ` +
                            (name === "aria-controls" ? "" : HOW),
                    });
                }
                else if (name === "for" &&
                    node.name === "label" &&
                    !LABELABLE.has(found.name) &&
                    !found.name.includes("-")) {
                    // En web component kan være knyttet til skjema og ha en
                    // ledetekst; det kan ikke sjekkes her, så det får passere.
                    findings.push({
                        start: at,
                        end: at + target.length,
                        severity: "error",
                        link: LINK,
                        message: `for="${attribute.value}" peker på et <${found.name}>, som ikke ` +
                            "kan ha en ledetekst. for må peke på feltet selv: <input>, " +
                            `<select>, <textarea> eller en knapp. ${HOW}`,
                    });
                }
            }
        }
        // Lenkene i feiloppsummeringen skal gå til et felt som finnes.
        if (node.name === "a" &&
            within(node, (n) => n.name === "fs-error-summary" ||
                hasClass(n, (c) => c === "fs-error-summary"))) {
            const href = value(node, "href");
            const target = href?.value?.startsWith("#") ? href.value.slice(1) : "";
            if (href && target && !ids.has(fragment(target))) {
                findings.push({
                    start: href.valueStart + 1,
                    end: href.valueStart + target.length + 1,
                    severity: "error",
                    link: SUMMARY_LINK,
                    message: `Lenken i feiloppsummeringen går til «#${target}», som ikke finnes ` +
                        "på siden. Den skal gå til feltet som feilet, slik at brukeren " +
                        "havner der feilen kan rettes.",
                });
            }
        }
        if (within(node, (n) => n.name === "fs-field"))
            continue;
        // Et Fristil-felt uten ledetekst.
        if ((node.name === "input" ||
            node.name === "textarea" ||
            node.name === "select") &&
            hasClass(node, (c) => c.startsWith("fs-")) &&
            !UNLABELLED_TYPES.has((value(node, "type")?.value ?? "").toLowerCase())) {
            const id = value(node, "id")?.value;
            const named = within(node, (n) => n.name === "fs-suggestion" || n.name === "label") ||
                (id && labelFor.has(decode(id))) ||
                value(node, "aria-label") ||
                value(node, "aria-labelledby");
            if (!named) {
                findings.push({
                    start: node.nameStart,
                    end: node.nameEnd,
                    severity: "warning",
                    link: LINK,
                    message: `<${node.name}> har ingen ledetekst: ingen <label for> som peker på ` +
                        "det, ingen <label> rundt, og verken aria-label eller " +
                        `aria-labelledby. En skjermleser leser feltet opp uten navn. ${HOW}`,
                });
            }
        }
        // En synlig hjelpetekst eller feilmelding som ikke er koblet til noe felt.
        const kind = hasClass(node, (c) => c === "fs-error-text")
            ? "Feilmeldingen"
            : hasClass(node, (c) => c === "fs-help-text")
                ? "Hjelpeteksten"
                : undefined;
        // Et statusfelt, med `role="status"`, `role="alert"` eller `aria-live`,
        // leses opp av seg selv når det endres, og trenger ingen kobling.
        const live = ["status", "alert"].includes(value(node, "role")?.value ?? "") ||
            value(node, "aria-live") !== undefined;
        if (kind && !live && !isHidden(node)) {
            const id = value(node, "id")?.value;
            if (!id || !describedBy.has(decode(id))) {
                findings.push({
                    start: node.nameStart,
                    end: node.nameEnd,
                    severity: "warning",
                    link: LINK,
                    message: `${kind} er ikke koblet til noe felt: ` +
                        (id
                            ? `id-en «${id}» står ikke i aria-describedby på noe felt. `
                            : "den har ingen id, så ingen aria-describedby kan peke på den. ") +
                        `En skjermleser leser den ikke opp sammen med feltet. ${HOW}`,
                });
            }
        }
    }
    return findings.sort((a, b) => a.start - b.start);
}
//# sourceMappingURL=references.js.map