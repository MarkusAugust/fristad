import { attributes, createGuard } from "../shared.js";
export const CARD_CLASS = "lt-card";
/** The heading inside the card. */
export const CARD_TITLE_CLASS = "lt-card__title";
export const cardVariants = ["outline", "filled"];
/**
 * The attributes for a card.
 *
 * `interactive` gives hover and a focus ring, but makes nothing clickable by
 * itself. If the whole card should be pressable, the element must be an
 * `<a>` or a `<button>`. A `<div>` with a click listener can neither be
 * reached with the keyboard nor be announced as something to press.
 *
 * ```ts
 * <a {...card({ interactive: true })} href="/case/481">…</a>
 * ```
 */
export const card = Object.assign(({ variant = "outline", interactive } = {}) => attributes({
    class: CARD_CLASS,
    "data-variant": variant === "outline" ? undefined : variant,
    "data-interactive": interactive ? "" : undefined,
}), {
    /** The class on the heading inside the card. */
    title: CARD_TITLE_CLASS,
    variants: cardVariants,
    isVariant: createGuard(cardVariants),
});
//# sourceMappingURL=card.js.map