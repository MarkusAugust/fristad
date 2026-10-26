import { attributes } from "../shared.js";
export const ICON_CLASS = "lt-icon";
/**
 * The attributes for an icon. The class goes on an `<svg>`, and the svg comes
 * from the app's icon set: LO-TEK ships no icons.
 *
 * The icon is as large as the text around it, and is drawn in the colour the
 * svg says, which should be `currentColor`. Without `label` it is decoration,
 * and if it stands alone in a button, the button must get its name somewhere
 * else: `aria-label` on the button, or a text with `lt-sr-only`. The core
 * reports a button that lacks one.
 *
 * ```ts
 * <button {...button()} aria-label="Delete the application">
 *   <svg {...icon()} viewBox="0 0 24 24">…</svg>
 * </button>
 * ```
 */
export const icon = ({ label } = {}) => attributes({
    class: ICON_CLASS,
    role: label ? "img" : undefined,
    "aria-label": label || undefined,
    "aria-hidden": label ? undefined : "true",
});
//# sourceMappingURL=icon.js.map