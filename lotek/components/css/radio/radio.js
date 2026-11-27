import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const RADIO_CLASS = "lt-radio";
/** The row that holds the button and the text next to each other. */
export const RADIO_ROW_CLASS = "lt-radio-row";
/**
 * The attributes for a radio button.
 *
 * All the buttons in a group must have the same `name`. That connection is
 * what lets only one be selected, and lets the arrow keys move between them.
 * You set `name` yourself, since only you know what the group is called.
 *
 * ```ts
 * <input {...radio()} name="delivery" value="post" />
 * ```
 */
export const radio = Object.assign(({ state = "default" } = {}) => attributes({
    class: RADIO_CLASS,
    type: "radio",
    "data-state": state === "default" ? undefined : state,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    /** The class on the row around the button and the text. */
    row: RADIO_ROW_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=radio.js.map