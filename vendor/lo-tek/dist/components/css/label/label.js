import { attributes, createGuard, requiredMarkers, } from "../shared.js";
export const LABEL_CLASS = "lt-label";
export { requiredMarkers };
/**
 * The attributes for a label.
 *
 * The marker is only visual, so set `required` on the field itself as well,
 * or the screen reader is not told that it must be filled in.
 *
 * ```ts
 * <label {...label({ required: "symbol" })} htmlFor="email">Email address</label>
 * ```
 */
export const label = Object.assign(({ required, optional, disabled } = {}) => attributes({
    class: LABEL_CLASS,
    "data-required": required,
    "data-optional": optional && !required ? "" : undefined,
    "aria-disabled": disabled ? "true" : undefined,
}), {
    markers: requiredMarkers,
    isMarker: createGuard(requiredMarkers),
});
//# sourceMappingURL=label.js.map