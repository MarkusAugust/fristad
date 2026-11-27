import { attributes } from "../shared.js";
export const PAGINATION_CLASS = "lt-pagination";
/** The gap in the row of numbers, as in "1 2 … 9 10". */
export const PAGINATION_GAP_CLASS = "lt-pagination__gap";
/**
 * The attributes for pagination.
 *
 * The class goes on a `<ul>` inside a `<nav aria-label="Pagination">`. Each
 * page is a link with its own address, not a button: the user can then open
 * page three in a new tab, and come back with the back button.
 *
 * The current page is marked with `aria-current="page"`, which both gives
 * the colour and is what the screen reader announces.
 *
 * ```ts
 * <nav aria-label="Pagination">
 *   <ul {...pagination()}>…</ul>
 * </nav>
 * ```
 */
export const pagination = Object.assign(() => attributes({ class: PAGINATION_CLASS }), {
    /** The class on the gap in the row of numbers. */
    gap: PAGINATION_GAP_CLASS,
});
//# sourceMappingURL=pagination.js.map