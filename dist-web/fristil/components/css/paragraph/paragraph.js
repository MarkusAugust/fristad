import { attributes, createGuard } from "../shared.js";
export const PARAGRAPH_CLASS = "fs-paragraph";
export const paragraphSizes = ["small", "medium", "large"];
export const paragraphVariants = ["default", "lead"];
/**
 * Attributtene for et avsnitt.
 *
 * Bredden er begrenset til 70 tegn. Lengre linjer gjør det vanskelig å finne
 * tilbake til starten av neste, og det er den vanligste grunnen til at en
 * tekst er tung å lese selv når skriften er stor nok.
 *
 * ```ts
 * <p {...paragraph({ variant: "lead" })}>Kort om hva siden handler om.</p>
 * ```
 */
export const paragraph = Object.assign(({ size = "medium", variant = "default", } = {}) => attributes({
    class: PARAGRAPH_CLASS,
    "data-size": size === "medium" ? undefined : size,
    "data-variant": variant === "default" ? undefined : variant,
}), {
    sizes: paragraphSizes,
    isSize: createGuard(paragraphSizes),
    variants: paragraphVariants,
    isVariant: createGuard(paragraphVariants),
});
//# sourceMappingURL=paragraph.js.map