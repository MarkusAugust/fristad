import { attributes, createGuard } from "../shared.js";
export const BUTTON_CLASS = "lt-button";
export const buttonVariants = [
    "primary",
    "secondary",
    "ghost",
    "danger",
];
/**
 * The attributes for a button.
 *
 * ```ts
 * <button {...button({ variant: "secondary" })}>Save draft</button>
 * ```
 */
export const button = Object.assign(({ variant = "primary" } = {}) => attributes({
    class: BUTTON_CLASS,
    "data-variant": variant === "primary" ? undefined : variant,
}), {
    /** The valid variants, for lookup and branching. */
    variants: buttonVariants,
    /** Checks whether a string from outside is a valid variant. */
    isVariant: createGuard(buttonVariants),
});
//# sourceMappingURL=button.js.map