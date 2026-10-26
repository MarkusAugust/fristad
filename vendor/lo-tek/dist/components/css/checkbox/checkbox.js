import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const CHECKBOX_CLASS = "lt-checkbox";
/** The row that holds the box and the text next to each other. */
export const CHECKBOX_ROW_CLASS = "lt-checkbox-row";
/**
 * The attributes for a checkbox.
 *
 * `type` is set here, so the box cannot become a text field with a box's
 * styling. The indeterminate state is set with `element.indeterminate = true`
 * in JavaScript, not as an attribute, because HTML has no such attribute.
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
    /** The class on the row around the box and the text. */
    row: CHECKBOX_ROW_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=checkbox.js.map