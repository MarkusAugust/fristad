import { attributes } from "../shared.js";
export const BREADCRUMBS_CLASS = "fs-breadcrumbs";
/**
 * Attributtene for en brødsmulesti.
 *
 * Klassen hører på en `<ol>` inne i en `<nav>` med en tekst som sier hva
 * navigasjonen er, for eksempel `aria-label="Du er her"`. Rekkefølgen betyr
 * noe, så listen må være nummerert selv om tallene ikke vises.
 *
 * Skilletegnet mellom stegene legges inn med CSS. Sto det i markupen, ville
 * skjermlesere lest «skråstrek» mellom hvert steg.
 *
 * ```ts
 * <nav aria-label="Du er her">
 *   <ol {...breadcrumbs()}>
 *     <li><a href="/">Forsiden</a></li>
 *     <li><a href="/saker" aria-current="page">Mine saker</a></li>
 *   </ol>
 * </nav>
 * ```
 */
export const breadcrumbs = () => attributes({ class: BREADCRUMBS_CLASS });
//# sourceMappingURL=breadcrumbs.js.map