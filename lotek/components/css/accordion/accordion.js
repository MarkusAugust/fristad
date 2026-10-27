import { attributes, createGuard } from "../shared.js";
export const ACCORDION_CLASS = "lt-accordion";
/** The content shown when the panel is open. */
export const ACCORDION_CONTENT_CLASS = "lt-accordion__content";
export const accordionVariants = ["default", "plain"];
/**
 * The attributes for a panel that can be expanded.
 *
 * The class goes on a `<details>` with a `<summary>` first. The browser
 * handles opening, closing, the keyboard and screen readers announcing
 * "expanded" or "collapsed". Build the same thing from buttons and
 * `aria-expanded`, and all of that must be written again, and the browser's
 * find in page no longer finds text inside a closed panel.
 *
 * ```ts
 * <details {...accordion()}>
 *   <summary>Who can apply?</summary>
 *   <div class={accordion.content}>…</div>
 * </details>
 * ```
 */
export const accordion = Object.assign(({ variant = "default" } = {}) => attributes({
    class: ACCORDION_CLASS,
    "data-variant": variant === "default" ? undefined : variant,
}), {
    /** The class on the content below the summary. */
    content: ACCORDION_CONTENT_CLASS,
    variants: accordionVariants,
    isVariant: createGuard(accordionVariants),
});
//# sourceMappingURL=accordion.js.map