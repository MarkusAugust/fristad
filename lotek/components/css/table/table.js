import { attributes, createGuard } from "../shared.js";
export const TABLE_CLASS = "lt-table";
/** The container that lets a wide table scroll sideways. */
export const TABLE_SCROLL_CLASS = "lt-table-scroll";
export const tableVariants = ["default", "striped"];
/**
 * The attributes for a table.
 *
 * The class goes on a real `<table>` with `<thead>` and `<th>`. The header
 * cells are what let screen readers say "Amount, 1,240 kroner" when the user
 * is in a cell. Build the table from `<div>` elements, and that connection
 * does not exist.
 *
 * ```ts
 * <table {...table({ variant: "striped" })}>…</table>
 * ```
 */
export const table = Object.assign(({ variant = "default", hoverable } = {}) => attributes({
    class: TABLE_CLASS,
    "data-variant": variant === "default" ? undefined : variant,
    "data-hoverable": hoverable ? "" : undefined,
}), {
    /** The class on the container around a wide table. */
    scroll: TABLE_SCROLL_CLASS,
    variants: tableVariants,
    isVariant: createGuard(tableVariants),
});
/** The class on the button in a sortable column header. */
export const TABLE_SORT_CLASS = "lt-table__sort";
export const tableSortDirections = ["ascending", "descending"];
/**
 * The attributes for a sortable column header.
 *
 * `aria-sort` is on the `<th>`, not on the button, because the cell is what
 * describes the column. Only the sorted column has the attribute. The arrow
 * is drawn from the same attribute, so what the screen reader says and what
 * is shown cannot drift apart. The sorting itself is the app's job.
 *
 * ```ts
 * const date = lt.tableSort({ direction: "descending" })
 * ```
 * ```html
 * <th scope="col" {...date.header}>
 *   <button {...date.button}>Date</button>
 * </th>
 * ```
 */
export const tableSort = Object.assign(({ direction } = {}) => ({
    header: attributes({ "aria-sort": direction }),
    button: attributes({ class: TABLE_SORT_CLASS, type: "button" }),
}), {
    // The list sits next to it, as `tableSortDirections`, and not on the
    // function. The guard for the lists requires the attribute and the class
    // to be on the same element, and here `aria-sort` is on the cell and the
    // class on the button. The table test requires each direction to give its
    // own arrow.
    isDirection: createGuard(tableSortDirections),
});
//# sourceMappingURL=table.js.map