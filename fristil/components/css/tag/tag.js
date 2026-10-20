import { attributes, createGuard } from "../shared.js";
export const TAG_CLASS = "fs-tag";
export const tagVariants = ["outline", "filled"];
/**
 * Attributtene for en merkelapp.
 *
 * En merkelapp merker innhold: emneord på en artikkel, valgte filtre over en
 * liste. Sier den hvilken tilstand noe er i, er [Badge](../badge/badge.js)
 * riktig i stedet.
 *
 * `selectable` gir hover og fokusmarkering, men gjør ingenting trykkbart.
 * Elementet må være en `<button>`, og du setter `aria-pressed` selv, siden
 * bare du vet om lappen er valgt.
 *
 * ```ts
 * <button {...tag({ selectable: true })} aria-pressed="true">Bostøtte</button>
 * ```
 */
export const tag = Object.assign(({ variant = "outline", selectable } = {}) => attributes({
    class: TAG_CLASS,
    "data-variant": variant === "outline" ? undefined : variant,
    "data-selectable": selectable ? "" : undefined,
}), {
    variants: tagVariants,
    isVariant: createGuard(tagVariants),
});
//# sourceMappingURL=tag.js.map