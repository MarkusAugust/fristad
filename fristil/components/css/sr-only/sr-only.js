import { attributes } from "../shared.js";
export const SR_ONLY_CLASS = "fs-sr-only";
/**
 * Attributtene for tekst som bare skjermlesere skal få.
 *
 * ```ts
 * <span {...srOnly()}>Åpner i ny fane</span>
 * ```
 *
 * Bruk den når en ledetekst eller forklaring trengs for den som hører siden,
 * men ville vært støy for den som ser den. Skal innholdet bort for alle, hører
 * `hidden` hjemme i stedet.
 */
export const srOnly = () => attributes({ class: SR_ONLY_CLASS });
//# sourceMappingURL=sr-only.js.map