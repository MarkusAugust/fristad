import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const CHECKBOX_CLASS = "fs-checkbox";
/** Raden som holder boksen og teksten ved siden av hverandre. */
export const CHECKBOX_ROW_CLASS = "fs-checkbox-row";
/**
 * Attributtene for en avkryssingsboks.
 *
 * `type` settes her, slik at boksen ikke kan bli et tekstfelt med en boks sin
 * stil. Delvis avkrysset settes med `element.indeterminate = true` i
 * JavaScript, ikke som et attributt, fordi HTML ikke har noe slikt attributt.
 *
 * ```ts
 * <input {...checkbox({ state: "invalid" })} />
 * ```
 */
export const checkbox = Object.assign(({ state = "default" } = {}) => attributes({
    class: CHECKBOX_CLASS,
    type: "checkbox",
    "data-state": state === "default" ? undefined : state,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    /** Klassen på raden rundt boksen og teksten. */
    row: CHECKBOX_ROW_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=checkbox.js.map