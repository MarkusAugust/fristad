import { attributes, createGuard } from "../shared.js";
export const LIST_CLASS = "lt-list";
export const listVariants = ["default", "plain", "divided"];
/**
 * The attributes for a list.
 *
 * The class goes on `<ul>` or `<ol>`. `plain` and `divided` remove the
 * bullets visually, but the list stays a list, so the screen reader still
 * says how many items it has. That is why the variant exists: without it
 * people would use `<div>` to get rid of the bullets. The exception is
 * Safari, where a `<ul>` without visible bullets can need `role="list"`. See
 * the component page.
 *
 * ```ts
 * <ul {...list({ variant: "divided" })}>…</ul>
 * ```
 */
export const list = Object.assign(({ variant = "default" } = {}) => attributes({
    class: LIST_CLASS,
    "data-variant": variant === "default" ? undefined : variant,
}), {
    variants: listVariants,
    isVariant: createGuard(listVariants),
});
//# sourceMappingURL=list.js.map