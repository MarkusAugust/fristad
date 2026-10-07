import { attributes, createGuard } from "../shared.js";
export const ACCORDION_CLASS = "fs-accordion";
/** Innholdet som vises når panelet er åpent. */
export const ACCORDION_CONTENT_CLASS = "fs-accordion__content";
export const accordionVariants = ["default", "plain"];
/**
 * Attributtene for et panel som kan foldes ut.
 *
 * Klassen hører på et `<details>` med et `<summary>` først. Nettleseren tar
 * seg av åpning, lukking, tastatur og at skjermlesere melder «utvidet» eller
 * «sammenfoldet». Bygger du det samme av knapper og `aria-expanded`, må alt
 * det skrives på nytt, og nettleserens søk i siden finner ikke lenger tekst
 * inne i et lukket panel.
 *
 * ```ts
 * <details {...accordion()}>
 *   <summary>Hvem kan søke?</summary>
 *   <div class={accordion.content}>…</div>
 * </details>
 * ```
 */
export const accordion = Object.assign(({ variant = "default" } = {}) => attributes({
    class: ACCORDION_CLASS,
    "data-variant": variant === "default" ? undefined : variant,
}), {
    /** Klassen på innholdet under overskriften. */
    content: ACCORDION_CONTENT_CLASS,
    variants: accordionVariants,
    isVariant: createGuard(accordionVariants),
});
//# sourceMappingURL=accordion.js.map