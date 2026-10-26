import { attributes, createGuard } from "../shared.js";
export const ALERT_CLASS = "lt-alert";
/** The heading inside the message. */
export const ALERT_TITLE_CLASS = "lt-alert__title";
export const alertColors = ["info", "success", "warning", "danger"];
/**
 * The attributes for a message about the whole page or a whole form.
 *
 * The role is not set here, because it depends on when the message comes.
 * If it is in the markup from the start, it should have no role. If it
 * appears while the page is idle, it needs `role="status"`, or
 * `role="alert"` if the user must stop.
 *
 * ```ts
 * <div {...alert({ color: "warning" })}>
 *   <p class={alert.title}>The application has not been sent</p>
 * </div>
 * ```
 */
export const alert = Object.assign(({ color } = {}) => attributes({
    class: ALERT_CLASS,
    "data-color": color,
}), {
    /** The class on the heading inside the message. */
    title: ALERT_TITLE_CLASS,
    colors: alertColors,
    isColor: createGuard(alertColors),
});
//# sourceMappingURL=alert.js.map