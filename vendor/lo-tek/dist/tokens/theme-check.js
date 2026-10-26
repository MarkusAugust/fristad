import { defaultCore } from "../diagnostics/default-core.js";
/** Reads a theme and checks every promise in every block. */
export function inspectTheme(css) {
    return defaultCore().inspectTheme(css);
}
/** Only the problems, for anyone who does not need the counts. */
export function checkTheme(css) {
    return inspectTheme(css).problems;
}
//# sourceMappingURL=theme-check.js.map