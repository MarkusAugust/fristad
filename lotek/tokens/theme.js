import { defaultCore } from "../diagnostics/default-core.js";
/**
 * Builds the theme, and checks every promise while it is built.
 *
 * The arithmetic happens in the core, written in Rust, which `lotek theme` and
 * LO-TEK for Kotlin use. The typography and shape values are written straight
 * into a CSS rule, so a value that could break out of it, such as
 * `4px; } html {`, an unclosed parenthesis or `</style>`, is rejected with an
 * explanation that names the field in the recipe.
 */
export function buildTheme(input) {
    return defaultCore().buildTheme(input);
}
//# sourceMappingURL=theme.js.map