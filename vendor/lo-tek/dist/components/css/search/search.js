import { INPUT_CLASS, input } from "../input/input.js";
import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const SEARCH_CLASS = "lt-search";
/** The row with the search field and the button next to it. */
export const SEARCH_ROW_CLASS = "lt-search-row";
/**
 * The attributes for a search field.
 *
 * The field is an `lt-input` with room for the magnifying glass, so both
 * classes are set. `type="search"` gives the browser's own cross for
 * clearing the field, and on mobile a keyboard with "Search" instead of
 * "Enter".
 *
 * ```ts
 * <input {...search()} name="q" />
 * ```
 */
export const search = Object.assign(({ state = "default" } = {}) => {
    const base = input({ type: "search", state });
    return attributes({
        ...base,
        class: `${INPUT_CLASS} ${SEARCH_CLASS}`,
        type: "search",
        "data-variant": undefined,
    }, base);
}, {
    /** The class on the row with the field and the button. */
    row: SEARCH_ROW_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=search.js.map