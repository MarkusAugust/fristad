import { attributes } from "../../css/shared.js";
export const ERROR_SUMMARY_CLASS = "fs-error-summary";
export const ERROR_SUMMARY_TITLE_CLASS = "fs-error-summary__title";
/**
 * Attributtene for en feiloppsummering.
 *
 * Serveren skriver hele boksen, også overskriften og lista. `<fs-error-summary>`
 * flytter bare fokus og håndterer klikk på lenkene. Tidligere lagde komponenten
 * overskriften selv, og da forsvant den ved hver morfing i Datastar.
 *
 * ```ts
 * const feil = fs.errorSummary({ count: errors.length, id: "feil" })
 * ```
 * ```html
 * <fs-error-summary {...feil.host}>
 *   <h2 {...feil.title}>Du må rette 2 feil</h2>
 *   <ul><li><a href="#epost">E-posten mangler @</a></li></ul>
 * </fs-error-summary>
 * ```
 */
export const errorSummary = ({ count = 0, id, } = {}) => ({
    host: attributes({
        class: ERROR_SUMMARY_CLASS,
        role: "alert",
        tabindex: "-1",
        id,
        hidden: count === 0 ? true : undefined,
    }),
    title: attributes({ class: ERROR_SUMMARY_TITLE_CLASS }),
});
//# sourceMappingURL=error-summary.js.map