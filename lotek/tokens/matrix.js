/**
 * The matrix, as `@lo-tek/interfaces/matrix`: family times role, as finished
 * tokens.
 *
 * No palette, no steps, no pins: the role *is* the colour. The names follow
 * the namespaces of Tailwind and daisyUI: system prefix, kind and path.
 *
 * The colours are computed by the core. Here are LO-TEK's colours, and the
 * names. A matrix for other brand colours is `light` and `dark` in the theme
 * `buildTheme` returns.
 */
export { darkCells, FAMILIES, LOTEK_BRANDS, lightCells, } from "./matrix-cells.js";
/** The role name as it is written in CSS. `textSubtle` becomes `text-subtle`. */
export function roleToCss(role) {
    return role.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}
/** The token name for one cell in the matrix. */
export function tokenName(family, role) {
    return `--lt-color-${family}-${role}`;
}
//# sourceMappingURL=matrix.js.map