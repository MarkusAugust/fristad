import { attributes, createGuard } from "../shared.js";
export const SPINNER_CLASS = "lt-spinner";
export const spinnerSizes = ["small", "medium", "large"];
/**
 * The attributes for a loading indicator.
 *
 * Without `label` the ring is only a shape, and screen readers announce
 * nothing. With `label` it becomes a `role="status"` that says what is going
 * on. If you already have a visible text next to it, let that carry the
 * message and drop `label`, so the same thing is not announced twice.
 *
 * ```ts
 * <span {...spinner({ label: "Loading applications" })} />
 * ```
 */
export const spinner = Object.assign(({ size = "medium", label } = {}) => attributes({
    class: SPINNER_CLASS,
    "data-size": size === "medium" ? undefined : size,
    role: label ? "status" : undefined,
    "aria-label": label,
}), {
    sizes: spinnerSizes,
    isSize: createGuard(spinnerSizes),
});
//# sourceMappingURL=spinner.js.map