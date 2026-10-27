import { attributes, createGuard, fieldStates, isFieldState, } from "../shared.js";
export const SELECT_CLASS = "lt-select";
/**
 * Who draws the dropdown list.
 *
 * `auto` is the browser's own list, a window from the operating system that
 * cannot be styled. `styled` asks the browser to draw it inside the page
 * instead, so it gets the design system's colours, spacing and shadow.
 */
export const selectPickers = ["auto", "styled"];
export const isSelectPicker = createGuard(selectPickers);
/**
 * The attributes for a dropdown list.
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