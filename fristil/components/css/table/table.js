import { attributes, createGuard } from "../shared.js";
export const TABLE_CLASS = "fs-table";
/** Beholderen som lar en bred tabell rulles sidelengs. */
export const TABLE_SCROLL_CLASS = "fs-table-scroll";
export const tableVariants = ["default", "striped"];
/**
 * Attributtene for en tabell.
 *
 * Klassen hører på et ekte `<table>` med `<thead>` og `<th>`. Det er
 * overskriftscellene som gjør at skjermlesere kan si «Beløp, 1 240 kroner»
 * når brukeren står i en celle. Bygger du tabellen av `<div>`-er, finnes ikke
 * den koblingen.
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
    /** Klassen på beholderen rundt en bred tabell. */
    scroll: TABLE_SCROLL_CLASS,
    variants: tableVariants,
    isVariant: createGuard(tableVariants),
});
/** Klassen på knappen i en kolonneoverskrift som kan sorteres. */
export const TABLE_SORT_CLASS = "fs-table__sort";
export const tableSortDirections = ["ascending", "descending"];
/**
 * Attributtene for en kolonneoverskrift som kan sorteres.
 *
 * `aria-sort` står på `<th>`, ikke på knappen, for det er cellen som
 * beskriver kolonnen. Bare den sorterte kolonnen har attributtet. Pila
 * tegnes fra det samme attributtet, så det skjermleseren sier og det som
 * vises kan ikke gli fra hverandre. Selve sorteringen er appens.
 *
 * ```ts
 * const dato = fs.tableSort({ direction: "descending" })
 * ```
 * ```html
 * <th scope="col" {...dato.header}>
 *   <button {...dato.button}>Dato</button>
 * </th>
 * ```
 */
export const tableSort = Object.assign(({ direction } = {}) => ({
    header: attributes({ "aria-sort": direction }),
    button: attributes({ class: TABLE_SORT_CLASS, type: "button" }),
}), {
    // Lista ligger ved siden av, som `tableSortDirections`, og ikke på
    // funksjonen. Vaktposten for listene krever at attributtet og klassen
    // står på samme element, og her står `aria-sort` på cellen og klassen på
    // knappen. At hver retning gir sin egen pil, krever tabelltesten.
    isDirection: createGuard(tableSortDirections),
});
//# sourceMappingURL=table.js.map