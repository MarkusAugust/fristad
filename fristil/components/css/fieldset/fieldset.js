import { attributes, createGuard, fieldStates, isFieldState, requiredMarkers, } from "../shared.js";
export const FIELDSET_CLASS = "fs-fieldset";
export const LEGEND_CLASS = "fs-legend";
/**
 * Attributtene for en gruppe med kontroller som hører sammen.
 *
 * `<fieldset>` med `<legend>` er den ene måten å gi et sett radioknapper eller
 * avkryssingsbokser en felles ledetekst som skjermlesere faktisk leser opp
 * før hvert valg.
 *
 * ```ts
 * <fieldset {...fieldset({ state: "invalid" })}>
 *   <legend {...legend({ required: "symbol" })}>Leveringsmåte</legend>
 * </fieldset>
 * ```
 */
export const fieldset = Object.assign(({ state = "default", disabled } = {}) => attributes({
    class: FIELDSET_CLASS,
    "data-state": state === "default" ? undefined : state,
    disabled: disabled ? true : undefined,
}), {
    states: fieldStates,
    isState: isFieldState,
});
/**
 * Attributtene for ledeteksten på en gruppe.
 *
 * Samme markering som `label()`, men på et `<legend>`.
 */
export const legend = Object.assign(({ required, optional, disabled } = {}) => attributes({
    class: LEGEND_CLASS,
    "data-required": required,
    "data-optional": optional && !required ? "" : undefined,
    "aria-disabled": disabled ? "true" : undefined,
}), {
    markers: requiredMarkers,
    isMarker: createGuard(requiredMarkers),
});
//# sourceMappingURL=fieldset.js.map