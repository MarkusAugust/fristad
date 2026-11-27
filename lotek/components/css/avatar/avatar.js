import { attributes, createGuard } from "../shared.js";
export const AVATAR_CLASS = "lt-avatar";
/** Several profile pictures that overlap. */
export const AVATAR_STACK_CLASS = "lt-avatar-stack";
export const avatarSizes = ["small", "medium", "large"];
export const avatarVariants = ["circle", "square"];
/**
 * The attributes for a profile picture or initials.
 *
 * If the name is already next to it, the picture is decoration: set
 * `aria-hidden="true"` on it, or the name is announced twice. If the
 * picture stands alone, it must have a text that says who it is.
 *
 * ```ts
 * <span {...avatar({ size: "small" })} aria-hidden="true">ON</span>
 * ```
 */
export const avatar = Object.assign(({ size = "medium", variant = "circle", } = {}) => attributes({
    class: AVATAR_CLASS,
    "data-size": size === "medium" ? undefined : size,
    "data-variant": variant === "circle" ? undefined : variant,
}), {
    /** The class on the list of overlapping profile pictures. */
    stack: AVATAR_STACK_CLASS,
    sizes: avatarSizes,
    isSize: createGuard(avatarSizes),
    variants: avatarVariants,
    isVariant: createGuard(avatarVariants),
});
//# sourceMappingURL=avatar.js.map