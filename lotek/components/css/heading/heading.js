import { attributes, createGuard } from "../shared.js";
export const HEADING_CLASS = "lt-heading";
export const headingSizes = ["xs", "s", "m", "l", "xl", "mega"];
/**
 * The attributes for a heading.
 *
 * Size and level are two different things. The level is `<h1>` to `<h6>`
 * and sets the structure screen readers navigate by; `size` only sets how
 * large the text looks. An `<h2>` that should look small is correct; an
 * `<h4>` chosen because it was small enough skips a level in the structure.
 *
 * ```ts
 * <h2 {...heading({ size: "m" })}>Attachments</h2>
 * ```
 */
export const heading = Object.assign(({ size = "l" } = {}) => attributes({
    class: HEADING_CLASS,
    "data-size": size === "l" ? undefined : size,
}), {
    sizes: headingSizes,
    isSize: createGuard(headingSizes),
});
//# sourceMappingURL=heading.js.map