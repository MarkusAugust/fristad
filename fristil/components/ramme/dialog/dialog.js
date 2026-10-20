import { attributes, createGuard, idOrFallback } from "../../css/shared.js";
export const DIALOG_CLASS = "fs-dialog";
export const DIALOG_HEADER_CLASS = "fs-dialog__header";
export const DIALOG_TITLE_CLASS = "fs-dialog__title";
export const DIALOG_SUBTITLE_CLASS = "fs-dialog__subtitle";
export const DIALOG_BODY_CLASS = "fs-dialog__body";
export const DIALOG_FOOTER_CLASS = "fs-dialog__footer";
/**
 * Fargen på toppen, og hva den betyr.
 *
 * `neutral` er standard og gir ingen farge. De fire statusfargene er de
 * samme som varsleren bruker, og `brand` er merkefargen, for en topp som
 * sier hvem som snakker framfor hvordan det gikk.
 */
export const dialogColors = [
    "neutral",
    "brand",
    "info",
    "success",
    "warning",
    "danger",
];
/**
 * Attributtene for en dialog.
 *
 * Å åpne en dialog er et kall, ikke et attributt: bare `showModal()` flytter
 * fokus inn, holder fokus inne i dialogen, lukker på Escape og gjør resten
 * av siden utilgjengelig. Et `<dialog open>` i markupen er en boks på siden.
 *
 * Det er et problem for en server som bare sender HTML, for den kan ikke
 * kalle noe i nettleseren. Derfor sier serveren at dialogen er åpen, og
 * `<fs-dialog>` gjør kallet.
 *
 * De to `open`-ene er ikke det samme, og det er verdt å holde dem fra hverandre:
 *
 * - `open` på **verten** er serverens beskjed om at dialogen skal vises. Den
 *   er ikke fredet. Hadde den vært det, kunne serveren aldri åpnet dialogen
 *   igjen etter at brukeren hadde lukket den én gang. Sender serveren
 *   området på nytt med `open` fortsatt satt, åpnes dialogen altså igjen.
 *   Skal en avvisning vare, må appen si fra til serveren.
 * - `open` på **`<dialog>`** setter nettleseren selv når `showModal()`
 *   kalles. Det står ikke i serverens HTML, så en morfing river det bort, og
 *   komponenten setter det tilbake. Serveren skriver det i tillegg når den
 *   vet at dialogen skal vises: uten JavaScript er en `<dialog>` uten `open`
 *   skjult, og innholdet finnes da ikke for leseren. Med attributtet står
 *   det som en boks på siden til komponenten gjør den om til en modal.
 *
 * ```ts
 * const boks = fs.dialog({ titleId: "slett-tittel", open: true })
 * ```
 * ```html
 * <fs-dialog {...boks.host}>
 *   <dialog {...boks.dialog}>
 *     <h2 {...boks.title}>Slette søknaden?</h2>
 *     <div {...boks.body}>Søknaden og vedleggene blir borte.</div>
 *     <form method="dialog" {...boks.footer}>
 *       <button class="fs-button" value="avbryt">Avbryt</button>
 *     </form>
 *   </dialog>
 * </fs-dialog>
 * ```
 */
export const dialog = Object.assign(({ titleId: givenId, open = false, color = "neutral", }) => {
    // Reserven gjelder bare den som ikke har en typesjekk.
    const titleId = idOrFallback("fs.dialog()", givenId, "titleId");
    return {
        host: attributes({
            open: open ? true : undefined,
        }),
        dialog: attributes({
            class: DIALOG_CLASS,
            "aria-labelledby": titleId,
            /*
             * `open` står begge steder når dialogen skal vises, og det er med
             * vilje.
             *
             * Uten JavaScript er `<dialog>` uten `open` skjult, så innholdet
             * serveren ville vise fantes ikke for leseren. Med `open` står det
             * der som en boks på siden, og komponenten gjør den om til en ekte
             * modal med `showModal()` når den får kjøre.
             *
             * I React er det dessuten det eneste som stemmer: komponenten setter
             * `open` på `<dialog>` før React hydrerer, og sto det ikke i serverens
             * HTML, meldte React avvik ved hvert eneste oppslag.
             */
            open: open ? true : undefined,
            "data-color": color === "neutral" ? undefined : color,
        }),
        header: attributes({ class: DIALOG_HEADER_CLASS }),
        title: attributes({ class: DIALOG_TITLE_CLASS, id: titleId }),
        subtitle: attributes({ class: DIALOG_SUBTITLE_CLASS }),
        body: attributes({ class: DIALOG_BODY_CLASS }),
        footer: attributes({ class: DIALOG_FOOTER_CLASS }),
    };
}, {
    /** Klassen på selve `<dialog>`. */
    dialog: DIALOG_CLASS,
    /** Klassen på toppen, som kan farges med `color`. */
    header: DIALOG_HEADER_CLASS,
    /** Klassen på overskriften i dialogen. */
    title: DIALOG_TITLE_CLASS,
    /** Klassen på underteksten i toppen. */
    subtitle: DIALOG_SUBTITLE_CLASS,
    /** Klassen på innholdet, som ruller når dialogen blir for høy. */
    body: DIALOG_BODY_CLASS,
    /** Klassen på raden med knapper nederst. */
    footer: DIALOG_FOOTER_CLASS,
    colors: dialogColors,
    isColor: createGuard(dialogColors),
});
//# sourceMappingURL=dialog.js.map