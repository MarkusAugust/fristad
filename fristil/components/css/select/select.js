import { attributes, createGuard, fieldStates, isFieldState, } from "../shared.js";
export const SELECT_CLASS = "fs-select";
/**
 * Hvem som tegner nedtrekkslista.
 *
 * `auto` er nettleserens egen liste, et vindu fra operativsystemet som ikke
 * kan styles. `styled` ber nettleseren tegne den inne i siden i stedet, slik
 * at den får designsystemets farger, avstander og skygge.
 */
export const selectPickers = ["auto", "styled"];
export const isSelectPicker = createGuard(selectPickers);
/**
 * Attributtene for en nedtrekksliste.
 *
 * ```ts
 * <select {...select({ state: "invalid" })} />
 * <select {...select({ picker: "styled" })} />
 * ```
 */
export const select = Object.assign(({ state = "default", picker = "auto", } = {}) => attributes({
    class: SELECT_CLASS,
    "data-state": state === "default" ? undefined : state,
    "data-picker": picker === "auto" ? undefined : picker,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    states: fieldStates,
    isState: isFieldState,
    pickers: selectPickers,
    isPicker: isSelectPicker,
});
//# sourceMappingURL=select.js.map