import { attributes } from "../shared.js";
export const TOGGLE_GROUP_CLASS = "fs-toggle-group";
/** Ledeteksten rundt hvert alternativ. */
export const TOGGLE_GROUP_OPTION_CLASS = "fs-toggle-group__option";
/**
 * Attributtene for en gruppe der ett alternativ er valgt om gangen.
 *
 * Gruppen er radioknapper i forkledning. Hvert alternativ er en `<label>` med
 * en `<input type="radio">` inni, krympet bort visuelt. Da beholder du
 * piltastene, `FormData` og at skjermleseren sier «2 av 3» uten at noe av det
 * må skrives på nytt.
 *
 * Er valgene handlinger framfor en innstilling, som «fet, kursiv,
 * understreket», hører knapper med `aria-pressed` hjemme i stedet.
 *
 * ```ts
 * <fieldset {...toggleGroup()}>
 *   <legend class="fs-sr-only">Visning</legend>
 *   <label class={toggleGroup.option}>
 *     <input type="radio" name="visning" value="liste" checked /> Liste
 *   </label>
 * </fieldset>
 * ```
 */
export const toggleGroup = Object.assign(() => attributes({ class: TOGGLE_GROUP_CLASS }), {
    /** Klassen på ledeteksten rundt hvert alternativ. */
    option: TOGGLE_GROUP_OPTION_CLASS,
});
//# sourceMappingURL=toggle-group.js.map