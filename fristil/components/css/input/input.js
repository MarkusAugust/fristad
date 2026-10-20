import { attributes, createGuard, fieldStates, isFieldState, } from "../shared.js";
export const INPUT_CLASS = "fs-input";
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
/** Typene som får et ikon i feltet, og som åpner nettleserens egen velger. */
const TYPER_MED_IKON = ["date", "time", "datetime-local"];
/**
 * Attributtene for et tekstfelt.
 *
 * `type` setter både HTML-typen og ikonvarianten, så de to kan ikke komme i
 * utakt. Skriver du dem for hånd, er det lett å få et datofelt som ser ut
 * som et datofelt, men der ingenting skjer når brukeren trykker på ikonet.
 *
 * ```ts
 * <input {...input({ type: "date", state: "invalid" })} />
 * ```
 */
export const input = Object.assign(({ type = "text", state = "default" } = {}) => attributes({
    class: INPUT_CLASS,
    type,
    "data-variant": TYPER_MED_IKON.includes(type)
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