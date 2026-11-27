import { attributes, checkedKeys, idOrFallback } from "../../css/shared.js";
import { computeFieldAttributes, } from "../field/field-core.js";
export const SUGGESTION_CLASS = "lt-suggestion";
export const SUGGESTION_LIST_CLASS = "lt-suggestion__list";
export const SUGGESTION_OPTION_CLASS = "lt-suggestion__option";
export const SUGGESTION_EMPTY_CLASS = "lt-suggestion__empty";
export const SUGGESTION_VALUE_CLASS = "lt-suggestion__value";
export const SUGGESTION_STATUS_CLASS = "lt-sr-only";
/**
 * The attributes for a field with a suggestion list.
 *
 * The server writes both the field and the list. The web component filters,
 * handles the arrow keys and keeps `aria-activedescendant` in sync. It
 * renders nothing: it used to create the whole field itself, and then there
 * was nothing to fill in before the script had run.
 *
 * No preserve list. The attributes that change while the user types are set
 * by the web component, and it sets them back when a patch strips them. If
 * something else has already filtered, set `prefiltered` on
 * `<lt-suggestion>`, and the web component leaves both the options and the
 * empty message alone. If the server should also decide whether the list is
 * expanded and which option is highlighted, set `server-controlled` as well.
 */
export const suggestion = ({ id: givenId, keys: givenKeys = [], activeIndex = -1, open = false, pending = false, ...field }) => {
    // The fallback only matters for callers without a type check.
    const id = idOrFallback("lt.suggestion()", givenId);
    const keys = checkedKeys("lt.suggestion()", givenKeys);
    const count = keys.length;
    const computed = computeFieldAttributes({ ...field, id });
    const listId = `${id}-list`;
    const labelId = `${id}-label`;
    const statusId = `${id}-status`;
    const optionId = (index) => `${id}-option-${keys[index]}`;
    // The highlight is written only when the list is open, both halves
    // together. The web component removes both as the list closes, and a
    // closed list with a highlighted option would point
    // `aria-activedescendant` into something hidden.
    const marked = open &&
        Number.isInteger(activeIndex) &&
        activeIndex >= 0 &&
        activeIndex < count;
    return {
        // The label also names the list, through `aria-labelledby`.
        label: attributes({ ...computed.label, id: labelId }, computed.label),
        control: attributes({
            ...computed.control,
            class: "lt-input",
            type: "text",
            role: "combobox",
            autocomplete: "off",
            "aria-expanded": (open ? "true" : "false"),
            "aria-controls": listId,
            "aria-describedby": [computed.control["aria-describedby"], statusId]
                .filter(Boolean)
                .join(" "),
            "aria-autocomplete": "list",
            "aria-activedescendant": marked ? optionId(activeIndex) : undefined,
        }, computed.control),
        /**
         * The list. It sits in the top layer, `popover="manual"`, so a parent
         * with `overflow` does not clip it. `hidden` still says whether it is
         * open; the web component opens and closes the popover from that.
         */
        list: attributes({
            class: SUGGESTION_LIST_CLASS,
            id: listId,
            role: "listbox",
            "aria-labelledby": labelId,
            popover: "manual",
            hidden: open ? undefined : true,
            "aria-busy": pending ? "true" : undefined,
        }),
        options: keys.map((_, index) => attributes({
            class: SUGGESTION_OPTION_CLASS,
            id: optionId(index),
            role: "option",
            "aria-selected": (marked && index === activeIndex ? "true" : "false"),
        })),
        /**
         * The text shown when nothing matches.
         *
         * It follows the list: visible only when the list is open and empty. The
         * stylesheet handles a closed list, with a rule that hides the empty
         * message after a list with `hidden`. The builder function therefore
         * only says whether there are matches, and whether the answer has
         * arrived (`pending`). It used to read `open`, and with `prefiltered` the
         * app controls the empty message without knowing whether the list is
         * open: that state belongs to the web component, so the message could
         * never be shown.
         */
        empty: attributes({
            class: SUGGESTION_EMPTY_CLASS,
            hidden: count === 0 && !pending ? undefined : true,
        }),
        /**
         * The region that announces the number of matches to screen readers.
         *
         * The web component writes the text while the user types, so it is not
         * in the server's version. If an update empties the element, the web
         * component writes the text again. The element must still be in the
         * markup the server sends, or there is nothing to announce into before
         * the script has run. `role="status"` implies `aria-live="polite"`.
         */
        status: attributes({
            class: SUGGESTION_STATUS_CLASS,
            id: statusId,
            role: "status",
        }),
        /**
         * The hidden field that receives the code of the option the user chose.
         *
         * The field shows the text, and without JavaScript the text is what is
         * submitted. With JavaScript the web component writes `data-value` from
         * the option here, or the text when the option has no `data-value`, and
         * empties it when the user changes the text. You write `name` yourself;
         * it belongs to your form.
         */
        value: attributes({
            class: SUGGESTION_VALUE_CLASS,
            type: "hidden",
        }),
        help: computed.help,
        error: computed.error,
        state: computed.state,
    };
};
//# sourceMappingURL=suggestion.js.map