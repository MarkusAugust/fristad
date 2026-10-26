import { attributes, idOrFallback } from "../../css/shared.js";
export const SESSION_TIMEOUT_CLASS = "lt-session-timeout";
export const SESSION_TIMEOUT_DIALOG_CLASS = "lt-session-timeout__dialog";
export const SESSION_TIMEOUT_TITLE_CLASS = "lt-session-timeout__title";
export const SESSION_TIMEOUT_TEXT_CLASS = "lt-session-timeout__text";
export const SESSION_TIMEOUT_COUNT_CLASS = "lt-session-timeout__count";
export const SESSION_TIMEOUT_ACTIONS_CLASS = "lt-session-timeout__actions";
/** `value` on the button that extends the session. */
export const SESSION_EXTEND = "extend";
/** `value` on the button that logs out. */
export const SESSION_LOGOUT = "logout";
/**
 * The attributes for the warning that a signed-in session is about to expire.
 *
 * The renderer writes the dialog and all of the text, in its own language and
 * with its own translation tool. The component only keeps time: it opens and
 * closes the dialog, fills in the number and announces the countdown.
 *
 * The buttons are plain buttons in a `<form method="dialog">`. The browser
 * closes the dialog and sets `returnValue` to the button's `value`, and the
 * component reads it. `data-ignore-morph` asks Datastar's morph to leave the
 * warning alone. Without it the morph takes `open` and the number while the
 * dialog is in the top layer. The component puts `open` back, and the number
 * comes back on the next tick. With the attribute the number stays the whole
 * time.
 *
 * ```ts
 * const session = lt.sessionTimeout({ titleId: "session-title" })
 * ```
 * ```html
 * <lt-session-timeout {...session.host}>
 *   <dialog {...session.dialog}>
 *     <h2 {...session.title}>You will soon be signed out</h2>
 *     <p {...session.text}>
 *       We will sign you out in <span {...session.count}></span>.
 *     </p>
 *     <span {...session.live}></span>
 *     <form {...session.actions}>
 *       <button {...session.extend}>Stay signed in</button>
 *       <button {...session.logout}>Sign out now</button>
 *     </form>
 *   </dialog>
 * </lt-session-timeout>
 * ```
 */
export const sessionTimeout = Object.assign(({ titleId: givenId, warnAt = 25 * 60, expiresAt = 30 * 60, activityInterval = 60, }) => {
    // The fallback only matters to someone without a type check.
    const titleId = idOrFallback("lt.sessionTimeout()", givenId, "titleId");
    return {
        host: attributes({
            class: SESSION_TIMEOUT_CLASS,
            "warn-at": warnAt === 25 * 60 ? undefined : String(warnAt),
            "expires-at": expiresAt === 30 * 60 ? undefined : String(expiresAt),
            "activity-interval": activityInterval === 60 ? undefined : String(activityInterval),
            "data-ignore-morph": "",
        }),
        dialog: attributes({
            class: SESSION_TIMEOUT_DIALOG_CLASS,
            // alertdialog and not dialog: the warning is an interruption the user did
            // not ask for, and then the screen reader should read all of the content
            // when it opens.
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
            class: "lt-sr-only",
            role: "status",
        }),
        actions: attributes({
            class: SESSION_TIMEOUT_ACTIONS_CLASS,
            method: "dialog",
        }),
        extend: attributes({
            class: "lt-button",
            value: SESSION_EXTEND,
        }),
        logout: attributes({
            class: "lt-button",
            "data-variant": "secondary",
            value: SESSION_LOGOUT,
        }),
    };
}, {
    /** The class on the `<dialog>` itself. */
    dialog: SESSION_TIMEOUT_DIALOG_CLASS,
    /** The class on the heading. */
    title: SESSION_TIMEOUT_TITLE_CLASS,
    /** The class on the paragraph holding the number. */
    text: SESSION_TIMEOUT_TEXT_CLASS,
    /** The class on the number the component fills in. */
    count: SESSION_TIMEOUT_COUNT_CLASS,
    /** The class on the form with the buttons. */
    actions: SESSION_TIMEOUT_ACTIONS_CLASS,
});
//# sourceMappingURL=session-timeout.js.map