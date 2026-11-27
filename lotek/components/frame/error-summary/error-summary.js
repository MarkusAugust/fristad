import { attributes } from "../../css/shared.js";
export const ERROR_SUMMARY_CLASS = "lt-error-summary";
export const ERROR_SUMMARY_TITLE_CLASS = "lt-error-summary__title";
/**
 * The attributes for an error summary.
 *
 * The server writes the whole box, including the heading and the list.
 * `<lt-error-summary>` only moves focus and handles clicks on the links. The
 * component used to create the heading itself, and then it disappeared on
 * every morph in Datastar.
 *
 * ```ts
 * const summary = lt.errorSummary({ count: errors.length, id: "errors" })
 * ```
 * ```html
 * <lt-error-summary {...summary.host}>
 *   <h2 {...summary.title}>You need to fix 2 errors</h2>
 *   <ul><li><a href="#email">The email address is missing an @</a></li></ul>
 * </lt-error-summary>
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