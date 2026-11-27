/**
 * Colour arithmetic: sRGB, OKLCH and contrast, as `@lo-tek/interfaces/color`.
 *
 * The theme is computed in the core, written in Rust (`core/src/theme/color.rs`).
 * These five calculations are the same, for anyone who wants to compute colours
 * in JavaScript, including in the browser. The tests that check the colours a
 * browser actually paints use them too, and so check the core against an
 * independent implementation of the formula.
 *
 * The arithmetic is done in OKLCH, not HSL. In HSL, lightness means something
 * different for each hue: `hsl(60 100% 50%)` is bright yellow and
 * `hsl(240 100% 50%)` is almost black, with the same numbers. In OKLCH the
 * lightness is the same perceived lightness whatever the hue, so the contract
 * can set one lightness per role and apply it to every colour.
 *
 * Contrast is computed with the WCAG 2.1 formula, the same one the tests use,
 * so the contract and the check agree.
 */
/** Reads `#rrggbb` or `#rgb`. */
export function parseHex(hex) {
    const clean = hex.trim().replace(/^#/, "");
    const full = clean.length === 3
        ? clean
            .split("")
            .map((char) => char + char)
            .join("")
        : clean;
    if (!/^[0-9a-fA-F]{6}$/.test(full)) {
        throw new Error(`"${hex}" is not a valid hexadecimal colour`);
    }
    return {
        r: Number.parseInt(full.slice(0, 2), 16),
        g: Number.parseInt(full.slice(2, 4), 16),
        b: Number.parseInt(full.slice(4, 6), 16),
    };
}
export function toHex({ r, g, b }) {
    const channel = (value) => Math.round(Math.min(255, Math.max(0, value)))
        .toString(16)
        .padStart(2, "0");
    return `#${channel(r)}${channel(g)}${channel(b)}`;
}
function toLinear(channel) {
    const c = channel / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}
function fromLinear(channel) {
    const c = channel <= 0.0031308
        ? channel * 12.92
        : 1.055 * channel ** (1 / 2.4) - 0.055;
    return c * 255;
}
export function rgbToOklch({ r, g, b }) {
    const lr = toLinear(r);
    const lg = toLinear(g);
    const lb = toLinear(b);
    const l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    const m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
    const s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;
    const l_ = Math.cbrt(l);
    const m_ = Math.cbrt(m);
    const s_ = Math.cbrt(s);
    const lightness = 0.2104542553 * l_ + 0.793617785 * m_ - 0.0040720468 * s_;
    const a = 1.9779984951 * l_ - 2.428592205 * m_ + 0.4505937099 * s_;
    const bb = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.808675766 * s_;
    return {
        l: lightness,
        c: Math.hypot(a, bb),
        h: (Math.atan2(bb, a) * 180) / Math.PI,
    };
}
function oklchToRgbRaw({ l, c, h }) {
    const rad = (h * Math.PI) / 180;
    const a = c * Math.cos(rad);
    const b = c * Math.sin(rad);
    const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = l - 0.0894841775 * a - 1.291485548 * b;
    const lr = l_ ** 3;
    const mr = m_ ** 3;
    const sr = s_ ** 3;
    return {
        r: fromLinear(4.0767416621 * lr - 3.3077115913 * mr + 0.2309699292 * sr),
        g: fromLinear(-1.2684380046 * lr + 2.6097574011 * mr - 0.3413193965 * sr),
        b: fromLinear(-0.0041960863 * lr - 0.7034186147 * mr + 1.707614701 * sr),
    };
}
function outside({ r, g, b }) {
    const margin = 0.5;
    return [r, g, b].some((channel) => channel < -margin || channel > 255 + margin);
}
/**
 * Converts an OKLCH colour to an rgb colour a screen can show.
 *
 * Not every combination of lightness and chroma can be shown in sRGB. If the
 * colour falls outside, the chroma is reduced until it is inside, instead of
 * clipping each channel separately. Clipping would change the hue.
 */
export function oklchToRgb(color) {
    const attempt = oklchToRgbRaw(color);
    if (!outside(attempt))
        return attempt;
    let low = 0;
    let high = color.c;
    for (let i = 0; i < 24; i += 1) {
        const middle = (low + high) / 2;
        if (outside(oklchToRgbRaw({ ...color, c: middle })))
            high = middle;
        else
            low = middle;
    }
    return oklchToRgbRaw({ ...color, c: low });
}
function luminance({ r, g, b }) {
    return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}
/** The contrast ratio between two colours, as WCAG 2.1 computes it. */
export function contrastRatio(a, b) {
    const la = luminance(a);
    const lb = luminance(b);
    return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}
//# sourceMappingURL=color.js.map