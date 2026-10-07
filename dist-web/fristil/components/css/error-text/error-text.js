import { attributes, createGuard } from "../shared.js";
export const ERROR_TEXT_CLASS = "fs-error-text";
export const errorTextVariants = ["error", "warning"];
/**
 * Attributtene for en feilmelding under et felt.
 *
 * ```ts
 * <p {...errorText({ variant: "warning" })}>…</p>
 * ```
 */
export const errorText = Object.assign(({ variant = "error" } = {}) => attributes({
    class: ERROR_TEXT_CLASS,
    "data-variant": variant === "error" ? undefined : variant,
}), {
    variants: errorTextVariants,
    isVariant: createGuard(errorTextVariants),
});
//# sourceMappingURL=error-text.js.map