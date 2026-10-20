import { attributes } from "../shared.js";
export const SWITCH_CLASS = "fs-switch";
/** Raden som holder bryteren og teksten ved siden av hverandre. */
export const SWITCH_ROW_CLASS = "fs-switch-row";
/**
 * Attributtene for en av- og på-bryter.
 *
 * Bryteren er en avkryssingsboks med `role="switch"`. Da leser skjermlesere
 * den opp som «på» og «av» framfor «avkrysset», mens tastaturet og
 * `FormData` virker som for en vanlig boks.
 *
 * Bryteren har ingen valideringstilstand. Et valg som er på eller av kan ikke
 * være ugyldig i seg selv; er det påkrevd å slå den på, hører kravet hjemme i
 * en avkryssingsboks med `required`.
 *
 * ```ts
 * <input {...switchControl()} />
 * ```
 */
export const switchControl = Object.assign(({ disabled } = {}) => attributes({
    class: SWITCH_CLASS,
    type: "checkbox",
    role: "switch",
    disabled: disabled ? true : undefined,
}), {
    /** Klassen på raden rundt bryteren og teksten. */
    row: SWITCH_ROW_CLASS,
});
//# sourceMappingURL=switch.js.map