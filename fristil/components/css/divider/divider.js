import { attributes, createGuard } from "../shared.js";
export const DIVIDER_CLASS = "fs-divider";
export const dividerVariants = ["default", "subtle", "strong"];
/**
 * Attributtene for en skillelinje.
 *
 * Klassen hører på et `<hr>`. Skiller streken to deler av innholdet, er den
 * en opplysning skjermlesere skal få. Er den bare pynt mellom to bokser som
 * allerede er atskilt, sett `aria-hidden="true"` på den.
 *
 * ```ts
 * <hr {...divider({ variant: "subtle" })} />
 * ```
 */
export const divider = Object.assign(({ variant = "default" } = {}) => attributes({
    class: DIVIDER_CLASS,
    "data-variant": variant === "default" ? undefined : variant,
}), {
    variants: dividerVariants,
    isVariant: createGuard(dividerVariants),
});
//# sourceMappingURL=divider.js.map