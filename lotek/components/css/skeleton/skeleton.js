import { attributes, createGuard } from "../shared.js";
export const SKELETON_CLASS = "lt-skeleton";
export const skeletonVariants = ["block", "text", "circle"];
/**
 * The attributes for a placeholder while the content loads.
 *
 * The shapes are always hidden from screen readers. They say nothing about
 * what is coming, and a row of empty boxes being announced is worse than
 * silence. Announce the wait in one place instead, with `role="status"`.
 *
 * ```ts
 * <div {...skeleton({ variant: "text" })} />
 * ```
 */
export const skeleton = Object.assign(({ variant = "block" } = {}) => attributes({
    class: SKELETON_CLASS,
    "data-variant": variant === "block" ? undefined : variant,
    "aria-hidden": "true",
}), {
    variants: skeletonVariants,
    isVariant: createGuard(skeletonVariants),
});
//# sourceMappingURL=skeleton.js.map