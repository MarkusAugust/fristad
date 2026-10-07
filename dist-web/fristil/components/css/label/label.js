import { attributes, createGuard, requiredMarkers, } from "../shared.js";
export const LABEL_CLASS = "fs-label";
export { requiredMarkers };
/**
 * Attributtene for en ledetekst.
 *
 * Markeringen er bare visuell, så sett `required` på selve feltet i tillegg,
 * ellers får skjermleseren ikke vite at det må fylles ut.
 *
 * ```ts
 * <label {...label({ required: "symbol" })} htmlFor="epost">E-postadresse</label>
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