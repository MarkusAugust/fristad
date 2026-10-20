import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const RADIO_CLASS = "fs-radio";
/** Raden som holder knappen og teksten ved siden av hverandre. */
export const RADIO_ROW_CLASS = "fs-radio-row";
/**
 * Attributtene for en radioknapp.
 *
 * Alle knappene i en gruppe må ha samme `name`. Det er den koblingen som gjør
 * at bare én kan være valgt, og at piltastene flytter mellom dem. `name`
 * setter du selv, siden bare du vet hva gruppen heter.
 *
 * ```ts
 * <input {...radio()} name="leveringsmate" value="post" />
 * ```
 */
export const radio = Object.assign(({ state = "default" } = {}) => attributes({
    class: RADIO_CLASS,
    type: "radio",
    "data-state": state === "default" ? undefined : state,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    /** Klassen på raden rundt knappen og teksten. */
    row: RADIO_ROW_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=radio.js.map