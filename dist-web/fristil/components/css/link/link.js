import { attributes } from "../shared.js";
export const LINK_CLASS = "fs-link";
/**
 * Attributtene for en lenke.
 *
 * ```ts
 * <a {...link()} href="/kvittering">Se kvittering</a>
 * ```
 */
export const link = ({ disabled = false } = {}) => attributes({
    class: LINK_CLASS,
    "aria-disabled": disabled ? "true" : undefined,
});
//# sourceMappingURL=link.js.map