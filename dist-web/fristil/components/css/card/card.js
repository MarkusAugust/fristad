import { attributes, createGuard } from "../shared.js";
export const CARD_CLASS = "fs-card";
/** Overskriften inne i kortet. */
export const CARD_TITLE_CLASS = "fs-card__title";
export const cardVariants = ["outline", "filled"];
/**
 * Attributtene for et kort.
 *
 * `interactive` gir hover og fokusmarkering, men gjør ingenting klikkbart av
 * seg selv. Skal hele kortet kunne trykkes på, må elementet være en `<a>`
 * eller en `<button>`. Et `<div>` med en klikklytter kan hverken nås med
 * tastatur eller meldes som noe å trykke på.
 *
 * ```ts
 * <a {...card({ interactive: true })} href="/sak/481">…</a>
 * ```
 */
export const card = Object.assign(({ variant = "outline", interactive } = {}) => attributes({
    class: CARD_CLASS,
    "data-variant": variant === "outline" ? undefined : variant,
    "data-interactive": interactive ? "" : undefined,
}), {
    /** Klassen på overskriften inne i kortet. */
    title: CARD_TITLE_CLASS,
    variants: cardVariants,
    isVariant: createGuard(cardVariants),
});
//# sourceMappingURL=card.js.map