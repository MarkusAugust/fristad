import { attributes, createGuard, fieldStates, isFieldState, requiredMarkers, } from "../shared.js";
export const FIELDSET_CLASS = "lt-fieldset";
export const LEGEND_CLASS = "lt-legend";
/**
 * The attributes for a group of controls that belong together.
 *
 * `<fieldset>` with `<legend>` is the one way to give a set of radio buttons
 * or checkboxes a shared label that screen readers actually announce before
 * each choice.
 *
 * ```ts
 * <fieldset {...fieldset({ state: "invalid" })}>
 *   <legend {...legend({ required: "symbol" })}>Delivery method</legend>
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
 * The attributes for the label of a group.
 *
 * The same marker as `label()`, but on a `<legend>`.
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