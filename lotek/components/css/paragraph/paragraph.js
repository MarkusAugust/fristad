import { attributes, createGuard } from "../shared.js";
export const PARAGRAPH_CLASS = "lt-paragraph";
export const paragraphSizes = ["small", "medium", "large"];
export const paragraphVariants = ["default", "lead"];
/**
 * The attributes for a paragraph.
 *
 * The width is limited to 70 characters. Longer lines make it hard to find
 * the start of the next one, and that is the most common reason a text is
 * hard to read even when the type is large enough.
 *
 * ```ts
 * <p {...paragraph({ variant: "lead" })}>A short summary of what the page is about.</p>
 * ```
 */
export const paragraph = Object.assign(({ size = "medium", variant = "default", } = {}) => attributes({
    class: PARAGRAPH_CLASS,
    "data-size": size === "medium" ? undefined : size,
    "data-variant": variant === "default" ? undefined : variant,
}), {
    sizes: paragraphSizes,
    isSize: createGuard(paragraphSizes),
    variants: paragraphVariants,
    isVariant: createGuard(paragraphVariants),
});
//# sourceMappingURL=paragraph.js.map