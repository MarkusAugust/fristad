import { checkPromises, promisesFor, ROLES, } from "./contract.js";
import { buildMatrix, FRISTIL_BRANDS, roleToCss, tokenName } from "./matrix.js";
// Bare det `parseHex` kan lese. Slipper porten gjennom `#rrggbbaa`, kaster
// den lenger inne og hele kommandoen dør med stakkspor.
const HEX = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i;
/**
 * Deler CSS-teksten i blokker, med en ramme per nivå.
 *
 * Hver ramme holder sin egen tekst, ikke bare selektoren. Uten det ble alt som
 * sto før en nøstet regel lest som en del av selektoren, og deklarasjonen
 * forsvant uten et ord.
 */
export function parseBlocks(css) {
    const withoutComments = css.replace(/\/\*[\s\S]*?\*\//g, "");
    const blocks = [];
    const stack = [];
    let buffer = "";
    for (const char of withoutComments) {
        if (char === "{") {
            const split = buffer.lastIndexOf(";");
            if (stack.length > 0)
                stack[stack.length - 1].text += buffer.slice(0, split + 1);
            stack.push({ selector: buffer.slice(split + 1).trim(), text: "" });
            buffer = "";
            continue;
        }
        if (char === "}") {
            const frame = stack.pop();
            if (!frame)
                continue;
            frame.text += buffer;
            const declarations = readDeclarations(frame.text);
            if (Object.keys(declarations).length > 0) {
                const path = [...stack.map((f) => f.selector), frame.selector];
                blocks.push({
                    selector: path.filter(Boolean).join(" "),
                    appearance: readAppearance(path.join(" "), frame.text),
                    declarations,
                });
            }
            buffer = "";
            continue;
        }
        buffer += char;
    }
    return blocks;
}
function readDeclarations(text) {
    const out = {};
    for (const part of text.split(";")) {
        const separator = part.indexOf(":");
        if (separator < 0)
            continue;
        const name = part.slice(0, separator).trim();
        if (!name.startsWith("--fs-color-"))
            continue;
        // `!important` er lovlig og plausibelt i et håndskrevet tema, og sier
        // ingenting om fargen.
        out[name] = part
            .slice(separator + 1)
            .replace(/\s*!important\s*$/i, "")
            .trim();
    }
    return out;
}
/** Mørkt eller lyst, lest av `color-scheme` først og av selektoren ellers. */
function readAppearance(selector, body) {
    // Siste deklarasjon vinner, som i kaskaden.
    const found = [...body.matchAll(/(?:^|[\s;{])color-scheme\s*:([^;}]*)/gi)];
    const declared = found[found.length - 1];
    if (declared) {
        /*
         * Verdien leses, den mønstermatches ikke.
         *
         * Tre ganger ble den samme linja lappet for den samme feilklassen: først
         * manglet avslutningen `$`, så `only dark`, så `dark only`. Grammatikken
         * er `normal | [ light | dark | <custom-ident> ]+ && only?`, og `&&`
         * betyr at `only` kan stå på begge sider. Et regulært uttrykk som skal
         * dekke hver rekkefølge blir en ny lapp hver gang noen skriver den
         * lovlige formen vi ikke tenkte på.
         *
         * Spørsmålet som betyr noe er hvilke av de to nøkkelordene som står der.
         * Står begge, sier blokka at den virker i begge, ikke at den er ett av
         * dem, og da er selektoren det beste svaret. Står ingen, likeså.
         */
        const words = declared[1]
            .toLowerCase()
            .replace(/!important/g, "")
            .split(/\s+/)
            .filter(Boolean);
        const light = words.includes("light");
        const dark = words.includes("dark");
        if (light !== dark)
            return dark ? "dark" : "light";
    }
    // `dark` må stå som eget ord: `.darkmode-toggle` er ikke et mørkt tema.
    return /(?<![a-z])dark(?![a-z])/i.test(selector) ? "dark" : "light";
}
/** Rollenavnene slik de skrives i CSS, lengste først. */
const ROLE_NAMES = Object.keys(ROLES)
    .map(roleToCss)
    .sort((a, b) => b.length - a.length);
/** De to lagene bare den nøytrale familien har. De er ikke roller. */
const LAYER_NAMES = ["canvas", "raised"];
/**
 * Deler `--fs-color-min-merkevare-text` i familie og rolle.
 *
 * Rollen leses som **endelsen**. Med familien som begynnelsen falt et
 * familienavn med bindestrek utenfor uten et ord. `null` som rolle betyr et
 * kjent lag, og de hører bare til den nøytrale familien.
 */
function splitToken(token) {
    const rest = token.startsWith("--fs-color-") ? token.slice(11) : undefined;
    if (!rest)
        return undefined;
    for (const layer of LAYER_NAMES) {
        if (!rest.endsWith(`-${layer}`))
            continue;
        const family = rest.slice(0, -layer.length - 1);
        return family === "neutral" ? { family, role: null } : undefined;
    }
    for (const name of ROLE_NAMES) {
        if (!rest.endsWith(`-${name}`))
            continue;
        const family = rest.slice(0, -name.length - 1);
        if (!family)
            return undefined;
        const role = Object.keys(ROLES).find((r) => roleToCss(r) === name);
        return role ? { family, role } : undefined;
    }
    return undefined;
}
/**
 * Leser et tema og kontrollerer hvert løfte i hver blokk.
 *
 * Verdier som mangler fylles fra Fristils eget tema, slik at en konsument som
 * bare har overstyrt én celle får den kontrollert mot resten av systemet sitt
 * framfor mot ingenting.
 */
export function inspectTheme(css) {
    const problems = [];
    const blocks = parseBlocks(css);
    let checkedPromises = 0;
    let understood = 0;
    // Klammene telles for seg: en ulukket blokk blir ikke lest, og uten dette
    // forsvant hele blokka uten et ord.
    const withoutCss = css.replace(/\/\*[\s\S]*?\*\//g, "");
    const opened = (withoutCss.match(/{/g) ?? []).length;
    const closed = (withoutCss.match(/}/g) ?? []).length;
    if (opened !== closed)
        problems.push({
            selector: "(hele fila)",
            message: `Fila har ${opened} «{» og ${closed} «}». En blokk som ikke er lukket blir ikke lest, så deler av temaet kan være ukontrollert.`,
        });
    if (blocks.length === 0) {
        // Klammemeldingen skal med her også, ellers står konsumenten igjen med
        // «er dette et Fristil-tema?» om en fil som bare er avkuttet.
        problems.push({
            selector: "(hele fila)",
            message: "Fant ingen --fs-color-*-verdier. Er dette et Fristil-tema, og " +
                "står verdiene i en blokk?",
        });
        return { blocks: 0, declarations: 0, promises: 0, problems };
    }
    for (const block of blocks) {
        const defaults = buildMatrix(FRISTIL_BRANDS, block.appearance).tokens;
        const values = { ...defaults };
        for (const [name, value] of Object.entries(block.declarations)) {
            if (!HEX.test(value)) {
                problems.push({
                    selector: block.selector,
                    message: `${name} er «${value}». Kontrasten kan bare regnes på en heksfarge, så denne er ikke kontrollert.`,
                });
                continue;
            }
            values[name] = value;
            if (splitToken(name))
                understood++;
        }
        const families = {};
        for (const name of Object.keys(values)) {
            const split = splitToken(name);
            if (!split) {
                // Bare navn konsumenten selv skrev meldes. Standardverdiene er våre
                // egne, og de deles alltid.
                if (name in block.declarations)
                    problems.push({
                        selector: block.selector,
                        message: `${name} er ikke et token i systemet. Navnet er --fs-color-<familie>-<rolle>, og rollene er ${ROLE_NAMES.join(", ")}.`,
                    });
                continue;
            }
            if (split.role === null)
                continue;
            families[split.family] ??= {};
            families[split.family][split.role] = values[name];
        }
        // En familie konsumenten fant på selv fylles ikke av standarden og kan ha
        // hull. Den meldes som ufullstendig framfor å kastes på `undefined`.
        const allRoles = Object.keys(ROLES);
        for (const [name, family] of Object.entries(families)) {
            const missing = allRoles.filter((r) => !family[r]);
            if (missing.length === 0)
                continue;
            delete families[name];
            problems.push({
                selector: block.selector,
                message: `${name} mangler ${missing.map(roleToCss).join(", ")}. En familie må ha alle rollene for at løftene skal kunne kontrolleres.`,
            });
        }
        const layers = {
            canvas: values[tokenName("neutral", "canvas")],
            surface: values[tokenName("neutral", "surface")],
            raised: values[tokenName("neutral", "raised")],
        };
        for (const family of Object.values(families))
            checkedPromises += promisesFor(family, layers).length;
        for (const violation of checkPromises(families, layers)) {
            problems.push({
                selector: block.selector,
                message: `${violation.family}: ${violation.promise} er ${violation.ratio.toFixed(2)}:1, kravet er ${violation.required}:1.`,
            });
        }
    }
    return {
        problems,
        blocks: blocks.length,
        declarations: understood,
        promises: checkedPromises,
    };
}
/** Bare problemene, for den som ikke trenger tellingen. */
export function checkTheme(css) {
    return inspectTheme(css).problems;
}
//# sourceMappingURL=theme-check.js.map