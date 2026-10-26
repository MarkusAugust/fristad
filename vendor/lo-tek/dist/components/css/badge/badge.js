import { attributes, createGuard } from "../shared.js";
export const BADGE_CLASS = "lt-badge";
export const badgeColors = [
    "info",
    "success",
    "warning",
    "danger",
    "neutral",
];
/**
 * The attributes for a status badge.
 *
 * ```ts
 * <span {...badge({ color: "success" })}>Approved</span>
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