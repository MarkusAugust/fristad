import { attributes, createGuard } from "../shared.js";
export const LIST_CLASS = "fs-list";
export const listVariants = ["default", "plain", "divided"];
/**
 * Attributtene for en liste.
 *
 * Klassen hører på `<ul>` eller `<ol>`. `plain` og `divided` fjerner punktene
 * visuelt, men listen blir værende en liste, så skjermleseren sier fortsatt
 * hvor mange elementer den har. Det er grunnen til at varianten finnes: uten
 * den ville folk brukt `<div>` for å bli kvitt punktene.
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