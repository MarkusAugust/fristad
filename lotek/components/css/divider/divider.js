import { attributes, createGuard } from "../shared.js";
export const DIVIDER_CLASS = "lt-divider";
export const dividerVariants = ["default", "subtle", "strong"];
/**
 * The attributes for a divider.
 *
 * The class goes on an `<hr>`. If the line separates two parts of the
 * content, it is information screen readers should get. If it is only
 * decoration between two boxes that are already apart, set
 * `aria-hidden="true"` on it.
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