import { attributes, createGuard } from "../shared.js";
export const ALERT_CLASS = "fs-alert";
/** Overskriften inne i meldingen. */
export const ALERT_TITLE_CLASS = "fs-alert__title";
export const alertColors = ["info", "success", "warning", "danger"];
/**
 * Attributtene for en melding som gjelder hele siden eller et helt skjema.
 *
 * Rollen settes ikke her, fordi den avhenger av når meldingen kommer.
 * Står den i markupen fra start, skal den ikke ha noen rolle. Dukker den opp
 * mens siden står stille, trenger den `role="status"`, eller `role="alert"`
 * hvis brukeren må stoppe opp.
 *
 * ```ts
 * <div {...alert({ color: "warning" })}>
 *   <p {...alert.title}>Søknaden er ikke sendt</p>
 * </div>
 * ```
 */
export const alert = Object.assign(({ color } = {}) => attributes({
    class: ALERT_CLASS,
    "data-color": color,
}), {
    /** Klassen på overskriften inne i meldingen. */
    title: ALERT_TITLE_CLASS,
    colors: alertColors,
    isColor: createGuard(alertColors),
});
//# sourceMappingURL=alert.js.map