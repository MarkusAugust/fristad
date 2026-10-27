import { attributes } from "../shared.js";
export const LINK_CLASS = "lt-link";
/**
 * The attributes for a link.
 *
 * ```ts
 * <a {...link()} href="/receipt">See receipt</a>
 * <a {...link({ disabled: true })}>See annual statement</a>
 * ```
 *
 * An `<a>` without `href` is no link to a screen reader and is not in the
 * tab order. `role` and `tabindex` give it both back while it is disabled.
 */
export const link = ({ disabled = false } = {}) => attributes({
    class: LINK_CLASS,
    role: disabled ? "link" : undefined,
    tabindex: disabled ? "0" : undefined,
    "aria-disabled": disabled ? "true" : undefined,
});
//# sourceMappingURL=link.js.map