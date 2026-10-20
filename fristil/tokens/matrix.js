import { oklchToRgb, parseHex, rgbToOklch, toHex } from "./color.js";
import { buildFamily, checkPromises, NEUTRAL_LAYERS, ROLES, } from "./contract.js";
/**
 * Matrisen: familie ganger rolle, som ferdige tokens.
 *
 * Ingen palett, ingen trinn, ingen pinner: rollen *er* fargen. Navnene følger
 * navnerommene til Tailwind, daisyUI og Digdir, altså systemprefiks, slag og
 * sti. Slaget skiller farge fra `--fs-radius-*` og fra komponentvariabler som
 * `--fs-button-padding`, og lar Tailwind-temaet bli generert.
 */
/** Familiene Fristil leverer selv. En konsument kan ha andre. */
export const FAMILIES = [
    "accent",
    "visited",
    "brand1",
    "brand2",
    "brand3",
    "neutral",
    "danger",
    "warning",
    "success",
];
/** Rollenavnet slik det skrives i CSS. `textSubtle` blir `text-subtle`. */
export function roleToCss(role) {
    return role.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}
/** Tokennavnet for én celle i matrisen. */
export function tokenName(family, role) {
    return `--fs-color-${family}-${role}`;
}
/**
 * Bygger hele matrisen for ett utseende.
 *
 * `neutral` får `canvas` og `raised` i tillegg til rollene sine. De hører bare
 * dit: en rød side er ikke en tilstand systemet har.
 */
export function buildMatrix(brands, appearance) {
    if (!brands.neutral) {
        throw new Error("Matrisen trenger en nøytral familie. Den bærer siden, teksten og " +
            "kantene, og de andre familiene måles mot den.");
    }
    const families = {};
    for (const [name, brand] of Object.entries(brands))
        families[name] = buildFamily(brand, appearance);
    // Lagene regnes av den nøytrale kuløren, med enda mindre metning enn
    // flatene: en side med kulør i seg ser malt ut, ikke nøytral. Både lysheten
    // og metningen står i `NEUTRAL_LAYERS`, som resten av kontrakten.
    const { c, h } = rgbToOklch(parseHex(brands.neutral));
    const layer = (name) => toHex(oklchToRgb({
        l: NEUTRAL_LAYERS[name].lightness[appearance],
        c: c * NEUTRAL_LAYERS[name].chroma,
        h,
    }));
    const layers = {
        canvas: layer("canvas"),
        surface: families.neutral.surface,
        raised: layer("raised"),
    };
    const tokens = {};
    for (const [name, family] of Object.entries(families))
        for (const role of Object.keys(ROLES))
            tokens[tokenName(name, roleToCss(role))] = family[role];
    tokens[tokenName("neutral", "canvas")] = layers.canvas;
    tokens[tokenName("neutral", "raised")] = layers.raised;
    return { tokens, violations: checkPromises(families, layers) };
}
/**
 * Fristils egne merkefarger.
 *
 * Kulørene for det interaktive og for status er systemets egne. De tre
 * merkefargene er nye, med tydelig avstand i kulør, slik at en kategori kan
 * skilles fra en annen uten at fargen betyr noe i seg selv.
 */
export const FRISTIL_BRANDS = {
    accent: "#1362ae",
    visited: "#5a77a8",
    brand1: "#0d7a5f",
    brand2: "#5b3fa0",
    brand3: "#b04c5c",
    neutral: "#24272b",
    danger: "#a82e39",
    warning: "#896508",
    success: "#316f2a",
};
//# sourceMappingURL=matrix.js.map