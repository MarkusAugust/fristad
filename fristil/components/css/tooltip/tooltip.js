import { attributes } from "../shared.js";
export const TOOLTIP_CLASS = "fs-tooltip";
/** Selve boblen med teksten. */
export const TOOLTIP_BUBBLE_CLASS = "fs-tooltip__bubble";
/**
 * Attributtene for omslaget rundt en utløser og en hjelpeboble.
 *
 * Boblen vises på hover og på fokus. Fokusdelen er ikke valgfri: en boble som
 * bare kommer med musa finnes ikke for den som bruker tastatur.
 *
 * Teksten i boblen må kobles til utløseren med `aria-describedby`. Uten den
 * er boblen bare noe som dukker opp på skjermen, og leses aldri opp.
 *
 * Bruk den til et tillegg, aldri til noe brukeren må ha. På en berøringsskjerm
 * finnes hverken hover eller fokus før noe trykkes.
 *
 * ```ts
 * <span {...tooltip()}>
 *   <button aria-describedby="hint">Arkiver</button>
 *   <span class={tooltip.bubble} role="tooltip" id="hint">Flyttes til arkivet</span>
 * </span>
 * ```
 */
export const tooltip = Object.assign(() => attributes({ class: TOOLTIP_CLASS }), {
    /** Klassen på boblen med teksten. */
    bubble: TOOLTIP_BUBBLE_CLASS,
});
//# sourceMappingURL=tooltip.js.map