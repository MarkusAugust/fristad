import { attributes, createGuard } from "../shared.js";
export const SKELETON_CLASS = "fs-skeleton";
export const skeletonVariants = ["block", "text", "circle"];
/**
 * Attributtene for en plassholder mens innholdet lastes.
 *
 * Formene er alltid skjult for skjermlesere. De sier ingenting om hva som
 * kommer, og en rekke tomme bokser som leses opp er verre enn stillhet. Si
 * fra om ventingen ett sted i stedet, med `role="status"`.
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