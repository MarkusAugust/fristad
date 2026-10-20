import { attributes } from "../shared.js";
export const PAGINATION_CLASS = "fs-pagination";
/** Hoppet i nummerrekken, som «1 2 … 9 10». */
export const PAGINATION_GAP_CLASS = "fs-pagination__gap";
/**
 * Attributtene for sidenavigering.
 *
 * Klassen hører på en `<ul>` inne i en `<nav aria-label="Sidenavigering">`.
 * Hver side er en lenke med egen adresse, ikke en knapp: da kan brukeren
 * åpne side tre i ny fane, og komme tilbake med tilbakeknappen.
 *
 * Gjeldende side markeres med `aria-current="page"`, som både gir fargen og
 * er det skjermleseren leser opp.
 *
 * ```ts
 * <nav aria-label="Sidenavigering">
 *   <ul {...pagination()}>…</ul>
 * </nav>
 * ```
 */
export const pagination = Object.assign(() => attributes({ class: PAGINATION_CLASS }), {
    /** Klassen på hoppet i nummerrekken. */
    gap: PAGINATION_GAP_CLASS,
});
//# sourceMappingURL=pagination.js.map