import { attributes, createGuard } from "../shared.js";
export const AVATAR_CLASS = "fs-avatar";
/** Flere profilbilder som overlapper. */
export const AVATAR_STACK_CLASS = "fs-avatar-stack";
export const avatarSizes = ["small", "medium", "large"];
export const avatarVariants = ["circle", "square"];
/**
 * Attributtene for et profilbilde eller initialer.
 *
 * Står navnet allerede ved siden av, er bildet pynt: sett `aria-hidden="true"`
 * på det, ellers leses navnet opp to ganger. Står bildet alene, må det ha en
 * tekst som sier hvem det er.
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
    /** Klassen på lista med profilbilder som overlapper. */
    stack: AVATAR_STACK_CLASS,
    sizes: avatarSizes,
    isSize: createGuard(avatarSizes),
    variants: avatarVariants,
    isVariant: createGuard(avatarVariants),
});
//# sourceMappingURL=avatar.js.map