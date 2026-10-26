import { attributes, createGuard } from "../shared.js";
export const PROGRESS_CLASS = "lt-progress";
export const progressColors = ["success", "warning", "danger"];
/**
 * The attributes for a progress bar on a `<progress>`.
 *
 * The element has the semantics built in: the browser exposes it as
 * `role="progressbar"` and announces how far it has come. You need neither a
 * role, `aria-valuenow` nor `aria-valuemax`.
 *
 * Leave out `value`, and the bar becomes indeterminate: "something is going
 * on, we do not know for how long". That is the same as `lt.spinner()` says,
 * so choose one of them.
 *
 * ```ts
 * <progress {...progress({ value: 3, max: 7, label: "Step 3 of 7" })} />
 * ```
 *
 * `max` defaults to 100 and is always written out, against the rule that a
 * default gives no attribute. The reason is that the rule is about
 * appearance, where the CSS already has the answer. `max` is semantics, and
 * HTML's own default is `1`: a `<progress value="40">` without `max` shows a
 * full bar, without anything saying so.
 */
export const progress = Object.assign(({ value, max = 100, color, label, valueText, } = {}) => attributes({
    class: PROGRESS_CLASS,
    // The numbers are sent out as strings, like the rest of the return
    // values: that is the HTML form, and it is what `setAttributes` accepts.
    value: value === undefined ? undefined : String(value),
    max: String(max),
    "data-color": color,
    "aria-label": label,
    "aria-valuetext": valueText,
}), {
    colors: progressColors,
    isColor: createGuard(progressColors),
});
//# sourceMappingURL=progress.js.map