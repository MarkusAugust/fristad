import { attributes, createGuard } from "../shared.js";
export const TAG_CLASS = "lt-tag";
export const tagVariants = ["outline", "filled"];
/**
 * The attributes for a tag.
 *
 * A tag labels content: keywords on an article, selected filters above a
 * list. If it says what state something is in, [Badge](../badge/badge.js)
 * is right instead.
 *
 * `selectable` gives hover and a focus ring, but makes nothing pressable.
 * The element must be a `<button>`, and you set `aria-pressed` yourself,
 * since only you know whether the tag is selected.
 *
 * ```ts
 * <button {...tag({ selectable: true })} aria-pressed="true">Housing benefit</button>
 * ```
 */
export const tag = Object.assign(({ variant = "outline", selectable } = {}) => attributes({
    class: TAG_CLASS,
    "data-variant": variant === "outline" ? undefined : variant,
    "data-selectable": selectable ? "" : undefined,
}), {
    variants: tagVariants,
    isVariant: createGuard(tagVariants),
});
//# sourceMappingURL=tag.js.map