import { attributes, createGuard } from "../shared.js";
export const BUTTON_CLASS = "fs-button";
export const buttonVariants = [
    "primary",
    "secondary",
    "ghost",
    "danger",
];
/**
 * Attributtene for en knapp.
 *
 * ```ts
 * <button {...button({ variant: "secondary" })}>Lagre utkast</button>
 * ```
 */
export const button = Object.assign(({ variant = "primary" } = {}) => attributes({
    class: BUTTON_CLASS,
    "data-variant": variant === "primary" ? undefined : variant,
}), {
    /** De lovlige variantene, for oppslag og forgrening. */
    variants: buttonVariants,
    /** Sjekker om en streng utenfra er en gyldig variant. */
    isVariant: createGuard(buttonVariants),
});
//# sourceMappingURL=button.js.map