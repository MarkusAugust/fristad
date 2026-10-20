import { attributes, createGuard } from "../shared.js";
export const HELP_TEXT_CLASS = "fs-help-text";
export const helpTextVariants = [
    "muted",
    "strong",
    "success",
    "warning",
];
/**
 * Attributtene for hjelpetekst under et felt.
 *
 * ```ts
 * <p {...helpText({ variant: "warning" })}>…</p>
 * ```
 */
export const helpText = Object.assign(({ variant = "muted" } = {}) => attributes({
    class: HELP_TEXT_CLASS,
    "data-variant": variant === "muted" ? undefined : variant,
}), {
    variants: helpTextVariants,
    isVariant: createGuard(helpTextVariants),
});
//# sourceMappingURL=help-text.js.map