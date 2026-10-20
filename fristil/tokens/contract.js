import { contrastRatio, oklchToRgb, parseHex, rgbToOklch, toHex, } from "./color.js";
/**
 * Lyshet og metningsandel per rolle, i hvert utseende.
 *
 * Metningen er en andel, så en dus merkefarge gir en dus matrise. Flatene
 * tåler minst: en lys flate med mye metning ser skitten ut.
 *
 * Tre valg som ikke er åpenbare:
 *
 * - `fill` styres av at `content` skal kunne leses oppå den, ikke av avstanden
 *   til siden. Velges den etter siden alene, blir den for lys til å bære tekst.
 * - Tekst og kant har tre trinn hver. Brødtekst skal ikke ligge på
 *   minstekravet: satt til `text` ble den `#4a4d51` framfor `#1a1a1a`, og det
 *   er et ekte tap i lesbarhet selv om kravet holdt.
 * - `textStrong` har lav metning, siden en sterk tekstfarge nesten er sort
 *   eller hvit uansett kulør.
 */
export const ROLES = {
    surface: { lightness: { light: 0.96, dark: 0.26 }, chroma: 0.22 },
    borderSubtle: { lightness: { light: 0.88, dark: 0.34 }, chroma: 0.35 },
    border: { lightness: { light: 0.575, dark: 0.625 }, chroma: 0.55 },
    borderStrong: { lightness: { light: 0.48, dark: 0.74 }, chroma: 0.6 },
    fill: { lightness: { light: 0.53, dark: 0.65 }, chroma: 1 },
    text: { lightness: { light: 0.42, dark: 0.82 }, chroma: 0.85 },
    textStrong: { lightness: { light: 0.28, dark: 0.93 }, chroma: 0.5 },
    textSubtle: { lightness: { light: 0.485, dark: 0.73 }, chroma: 0.7 },
    content: { lightness: { light: 0.99, dark: 0.16 }, chroma: 0.04 },
};
/**
 * Siden, kortet og den hevede flaten, som bare den nøytrale familien har.
 *
 * Tekst og kant må holde mot alle tre, ikke bare mot siden. `raised` er det
 * vanskeligste laget, mørkest i lyst tema og lysest i mørkt.
 */
export const NEUTRAL_LAYERS = {
    canvas: { lightness: { light: 1, dark: 0.18 }, chroma: 0.08 },
    raised: { lightness: { light: 0.92, dark: 0.32 }, chroma: 0.12 },
};
/** Kontrastkravene, med margin over WCAG. Tekst er 4,5 og grafikk er 3. */
export const REQUIREMENT = { text: 4.6, graphic: 3.1 };
/** Bygger én familie fra én merkefarge. */
export function buildFamily(brand, appearance) {
    const { c, h } = rgbToOklch(parseHex(brand));
    const out = {};
    for (const [role, spec] of Object.entries(ROLES)) {
        out[role] = toHex(oklchToRgb({ l: spec.lightness[appearance], c: c * spec.chroma, h }));
    }
    return out;
}
/**
 * Løftene, kontrollert mot de flatene fargen faktisk havner på.
 *
 * Lista er selve kontrakten. Legger du til en rolle som skal stå inne for noe,
 * hører løftet hjemme her.
 *
 * `borderSubtle` står uten løfte med vilje: en dekorativ skillelinje er ikke
 * nødvendig for å forstå siden, og WCAG 1.4.11 gjelder den ikke. Trenger
 * kanten å bety noe, er `border` den riktige.
 */
export function promisesFor(f, layers) {
    const ratio = (a, b) => contrastRatio(parseHex(a), parseHex(b));
    return [
        // Tekst leses på siden, på et kort, på en hevet flate og på egen flate.
        ["text mot canvas", ratio(f.text, layers.canvas), REQUIREMENT.text],
        ["text mot surface", ratio(f.text, layers.surface), REQUIREMENT.text],
        ["text mot raised", ratio(f.text, layers.raised), REQUIREMENT.text],
        ["text mot egen surface", ratio(f.text, f.surface), REQUIREMENT.text],
        // `textStrong` er brødteksten og lenka ved hover, så den har de samme
        // tre løftene som `text`. Den ligger lenger fra flaten enn `text` gjør,
        // så løftene følger av dem over, men de skal stå skrevet: en endring i
        // lysheten skal felle sjekken og ikke bare bestå i stillhet.
        [
            "textStrong mot canvas",
            ratio(f.textStrong, layers.canvas),
            REQUIREMENT.text,
        ],
        [
            "textStrong mot surface",
            ratio(f.textStrong, layers.surface),
            REQUIREMENT.text,
        ],
        [
            "textStrong mot raised",
            ratio(f.textStrong, layers.raised),
            REQUIREMENT.text,
        ],
        [
            "textSubtle mot canvas",
            ratio(f.textSubtle, layers.canvas),
            REQUIREMENT.text,
        ],
        [
            "textSubtle mot surface",
            ratio(f.textSubtle, layers.surface),
            REQUIREMENT.text,
        ],
        [
            "textSubtle mot raised",
            ratio(f.textSubtle, layers.raised),
            REQUIREMENT.text,
        ],
        // Kanten er grafikk, ikke tekst, og har det lavere kravet.
        ["border mot canvas", ratio(f.border, layers.canvas), REQUIREMENT.graphic],
        ["border mot raised", ratio(f.border, layers.raised), REQUIREMENT.graphic],
        [
            "border mot egen surface",
            ratio(f.border, f.surface),
            REQUIREMENT.graphic,
        ],
        ["fill mot canvas", ratio(f.fill, layers.canvas), REQUIREMENT.graphic],
        ["fill mot raised", ratio(f.fill, layers.raised), REQUIREMENT.graphic],
        // Og teksten oppå den fylte flaten er det strengeste kravet av alle.
        ["content oppå fill", ratio(f.content, f.fill), REQUIREMENT.text],
        /*
         * Knappen og gjeldende side i pagineringen bytter flate til `text` under
         * musa, og `content` blir stående. Det følger av lyshetene, siden `text`
         * ligger lenger fra `content` enn `fill` gjør i begge utseender, men det
         * skal stå som et løfte: uten det var hover den eneste fylte flaten i
         * systemet kontrakten ikke sa noe om.
         */
        ["content oppå text", ratio(f.content, f.text), REQUIREMENT.text],
    ];
}
/** Alle løfter i alle familier, med dem som ikke holder. */
export function checkPromises(families, layers) {
    const violations = [];
    for (const [family, f] of Object.entries(families))
        for (const [promise, ratio, required] of promisesFor(f, layers))
            if (ratio < required)
                violations.push({ family, promise, ratio, required });
    return violations;
}
//# sourceMappingURL=contract.js.map