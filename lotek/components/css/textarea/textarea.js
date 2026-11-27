import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const TEXTAREA_CLASS = "lt-textarea";
/**
 * The attributes for a multi-line text field.
 *
 * ```ts
 * <textarea {...textarea({ state: "invalid" })} />
 * ```
 */
export const textarea = Object.assign(({ state = "default" } = {}) => attributes({
    class: TEXTAREA_CLASS,
    "data-state": state === "default" ? undefined : state,
    "aria-invalid": state === "invalid" ? "true" : undefined,
}), {
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=textarea.js.map