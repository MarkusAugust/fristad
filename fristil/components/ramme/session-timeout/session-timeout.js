import { attributes, idOrFallback } from "../../css/shared.js";
export const SESSION_TIMEOUT_CLASS = "fs-session-timeout";
export const SESSION_TIMEOUT_DIALOG_CLASS = "fs-session-timeout__dialog";
export const SESSION_TIMEOUT_TITLE_CLASS = "fs-session-timeout__title";
export const SESSION_TIMEOUT_TEXT_CLASS = "fs-session-timeout__text";
export const SESSION_TIMEOUT_COUNT_CLASS = "fs-session-timeout__count";
export const SESSION_TIMEOUT_ACTIONS_CLASS = "fs-session-timeout__actions";
/** `value` på knappen som forlenger økten. */
export const SESSION_EXTEND = "extend";
/** `value` på knappen som logger ut. */
export const SESSION_LOGOUT = "logout";
/**
 * Attributtene for varselet om at innlogget økt går ut.
 *
 * Den som rendrer, skriver dialogen og hele teksten, på sitt eget språk og
 * med sitt eget oversettelsesverktøy. Komponenten tar bare tiden: den åpner
 * og lukker dialogen, fyller inn tallet og leser opp nedtellingen.
 *
 * Knappene er vanlige knapper i et `<form method="dialog">`. Nettleseren
 * lukker dialogen og setter `returnValue` til knappens `value`, og
 * komponenten leser det. `data-ignore-morph` ber Datastars morfing la
 * varselet være. Uten den tar morfingen `open` og tallet mens dialogen står i
 * topplaget. Komponenten setter `open` tilbake og tallet kommer igjen ved
 * neste tikk. Med attributtet står tallet hele tiden.
 *
 * ```ts
 * const okt = fs.sessionTimeout({ titleId: "okt-tittel" })
 * ```
 * ```html
 * <fs-session-timeout {...okt.host}>
 *   <dialog {...okt.dialog}>
 *     <h2 {...okt.title}>Du blir snart logget ut</h2>
 *     <p {...okt.text}>
 *       Vi logger deg ut om <span {...okt.count}></span>.
 *     </p>
 *     <span {...okt.live}></span>
 *     <form {...okt.actions}>
 *       <button {...okt.extend}>Fortsett å være innlogget</button>
 *       <button {...okt.logout}>Logg ut nå</button>
 *     </form>
 *   </dialog>
 * </fs-session-timeout>
 * ```
 */
export const sessionTimeout = Object.assign(({ titleId: givenId, warnAt = 25 * 60, expiresAt = 30 * 60, }) => {
    // Reserven gjelder bare den som ikke har en typesjekk.
    const titleId = idOrFallback("fs.sessionTimeout()", givenId, "titleId");
    return {
        host: attributes({
            class: SESSION_TIMEOUT_CLASS,
            "warn-at": warnAt === 25 * 60 ? undefined : String(warnAt),
            "expires-at": expiresAt === 30 * 60 ? undefined : String(expiresAt),
            "data-ignore-morph": "",
        }),
        dialog: attributes({
            class: SESSION_TIMEOUT_DIALOG_CLASS,
            // alertdialog og ikke dialog: varselet er en avbrytelse brukeren ikke
            // ba om, og da skal skjermleseren lese hele innholdet når det åpnes.
            role: "alertdialog",
            "aria-labelledby": titleId,
        }),
        title: attributes({ class: SESSION_TIMEOUT_TITLE_CLASS, id: titleId }),
        text: attributes({ class: SESSION_TIMEOUT_TEXT_CLASS }),
        count: attributes({
            class: SESSION_TIMEOUT_COUNT_CLASS,
            "aria-hidden": "true",
        }),
        live: attributes({
            class: "fs-sr-only",
            role: "status",
        }),
        actions: attributes({
            class: SESSION_TIMEOUT_ACTIONS_CLASS,
            method: "dialog",
        }),
        extend: attributes({
            class: "fs-button",
            value: SESSION_EXTEND,
        }),
        logout: attributes({
            class: "fs-button",
            "data-variant": "secondary",
            value: SESSION_LOGOUT,
        }),
    };
}, {
    /** Klassen på selve `<dialog>`. */
    dialog: SESSION_TIMEOUT_DIALOG_CLASS,
    /** Klassen på overskriften. */
    title: SESSION_TIMEOUT_TITLE_CLASS,
    /** Klassen på avsnittet med tallet i. */
    text: SESSION_TIMEOUT_TEXT_CLASS,
    /** Klassen på tallet komponenten fyller inn. */
    count: SESSION_TIMEOUT_COUNT_CLASS,
    /** Klassen på skjemaet med knappene. */
    actions: SESSION_TIMEOUT_ACTIONS_CLASS,
});
//# sourceMappingURL=session-timeout.js.map