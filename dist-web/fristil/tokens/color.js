/**
 * Fargeregning: sRGB, OKLCH og kontrast.
 *
 * Regningen skjer i OKLCH og ikke i HSL. I HSL betyr lyshet noe annet for hver
 * kulør: `hsl(60 100% 50%)` er knallgul og `hsl(240 100% 50%)` er nesten sort,
 * med samme tall. I OKLCH er lysheten den samme opplevde lysheten uansett
 * kulør, så kontrakten kan sette én lyshet per rolle og la den gjelde for alle
 * farger.
 *
 * Kontrasten regnes med WCAG 2.1 sin formel, altså den samme som testene
 * bruker, slik at kontrakten og kontrollen er enige.
 */
/** Leser `#rrggbb` eller `#rgb`. */
export function parseHex(hex) {
    const rent = hex.trim().replace(/^#/, "");
    const full = rent.length === 3
        ? rent
            .split("")
            .map((tegn) => tegn + tegn)
            .join("")
        : rent;
    if (!/^[0-9a-fA-F]{6}$/.test(full)) {
        throw new Error(`«${hex}» er ikke en gyldig heksadesimal farge`);
    }
    return {
        r: Number.parseInt(full.slice(0, 2), 16),
        g: Number.parseInt(full.slice(2, 4), 16),
        b: Number.parseInt(full.slice(4, 6), 16),
    };
}
export function toHex({ r, g, b }) {
    const tall = (value) => Math.round(Math.min(255, Math.max(0, value)))
        .toString(16)
        .padStart(2, "0");
    return `#${tall(r)}${tall(g)}${tall(b)}`;
}
function toLinear(kanal) {
    const c = kanal / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}
function fromLinear(kanal) {
    const c = kanal <= 0.0031308 ? kanal * 12.92 : 1.055 * kanal ** (1 / 2.4) - 0.055;
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
function utenfor({ r, g, b }) {
    const slingringsmonn = 0.5;
    return [r, g, b].some((kanal) => kanal < -slingringsmonn || kanal > 255 + slingringsmonn);
}
/**
 * Gjør en OKLCH-farge om til rgb som finnes på en skjerm.
 *
 * Ikke alle kombinasjoner av lyshet og metning kan vises i sRGB. Havner
 * fargen utenfor, dempes metningen til den er innenfor, framfor å klippe
 * kanalene hver for seg. Det siste ville endret kuløren.
 */
export function oklchToRgb(farge) {
    const forsok = oklchToRgbRaw(farge);
    if (!utenfor(forsok))
        return forsok;
    let lav = 0;
    let hoy = farge.c;
    for (let i = 0; i < 24; i += 1) {
        const midt = (lav + hoy) / 2;
        if (utenfor(oklchToRgbRaw({ ...farge, c: midt })))
            hoy = midt;
        else
            lav = midt;
    }
    return oklchToRgbRaw({ ...farge, c: lav });
}
function luminans({ r, g, b }) {
    return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}
/** Kontrastforholdet mellom to farger, slik WCAG 2.1 regner det. */
export function contrastRatio(a, b) {
    const la = luminans(a);
    const lb = luminans(b);
    return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}
//# sourceMappingURL=color.js.map