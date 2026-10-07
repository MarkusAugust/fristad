import { INPUT_CLASS, input } from "../input/input.js";
import { attributes, fieldStates, isFieldState, } from "../shared.js";
export const SEARCH_CLASS = "fs-search";
/** Raden med søkefeltet og knappen ved siden av. */
export const SEARCH_ROW_CLASS = "fs-search-row";
/**
 * Attributtene for et søkefelt.
 *
 * Feltet er en `fs-input` med plass til forstørrelsesglasset, så begge
 * klassene settes. `type="search"` gir nettleserens eget kryss for å tømme
 * feltet, og på mobil et tastatur med «Søk» i stedet for «Enter».
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
    /** Klassen på raden med feltet og knappen. */
    row: SEARCH_ROW_CLASS,
    states: fieldStates,
    isState: isFieldState,
});
//# sourceMappingURL=search.js.map