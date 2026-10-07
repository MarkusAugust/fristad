import { attributes, createGuard } from "../shared.js";
export const BADGE_CLASS = "fs-badge";
export const badgeColors = [
    "info",
    "success",
    "warning",
    "danger",
    "neutral",
];
/**
 * Attributtene for et statusmerke.
 *
 * ```ts
 * <span {...badge({ color: "success" })}>Innvilget</span>
 * ```
 */
export const badge = Object.assign(({ color = "info" } = {}) => attributes({
    class: BADGE_CLASS,
    "data-color": color === "info" ? undefined : color,
}), {
    colors: badgeColors,
    isColor: createGuard(badgeColors),
});
//# sourceMappingURL=badge.js.map