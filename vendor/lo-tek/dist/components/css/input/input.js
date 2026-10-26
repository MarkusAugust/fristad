import { attributes, createGuard, fieldStates, isFieldState, } from "../shared.js";
export const INPUT_CLASS = "lt-input";
export const inputTypes = [
    "text",
    "password",
    "email",
    "number",
    "date",
    "datetime-local",
    "week",
    "month",
    "tel",
    "url",
    "search",
    "time",
];
/** The types that get an icon in the field, and that open the browser's own picker. */
const TYPES_WITH_ICON = ["date", "time", "datetime-local"];
/**
 * The attributes for a text field.
 *
 * `type` sets both the HTML type and the icon variant, so the two cannot get
 * out of step. Write them by hand, and it is easy to get a date field that
 * looks like a date field, but where nothing happens when the user presses
 * the icon.
 *
 * ```ts
 * <input {...input({ type: "date", state: "invalid" })} />
 * ```
 */
export const input = Object.assign(({ type = "text", state = "default" } = {}) => attributes({
    class: INPUT_CLASS,
    type,
    "data-variant": TYPES_WITH_ICON.includes(type)
        ? type
        : undefined,
    "data-state": state === "default" ? undefined : state,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    types: inputTypes,
    isType: createGuard(inputTypes),
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=input.js.map