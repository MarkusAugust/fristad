/**
 * Å bruke attributtene på et element i vanlig DOM.
 *
 * I JSX sprer du objektet rett inn i elementet. Uten JSX måtte du løkke
 * gjennom det selv, og passe på å fjerne attributter fra forrige tilstand.
 * Går et felt fra ugyldig til gyldig, utelater byggefunksjonen `data-state`, og et
 * `setAttribute` alene ville latt den gamle verdien bli stående.
 */
import { managedNames } from "./components/css/shared.js";
/**
 * Det et sett uten en byggefunksjon bak seg får ryddet.
 *
 * Et håndskrevet eller spredd sett står ikke i kartet over hva det
 * forvalter, og da er `data-*`-navnene det eneste funksjonen trygt kan ta:
 * de er systemets egne. Et sett rett fra en byggefunksjon bruker ikke denne
 * lista. Det rydder nøyaktig det byggefunksjonen forvalter, så
 * `fs.popover().trigger` på en sekundærknapp lar `data-variant` stå.
 */
const SAFE_FOR_UNKNOWN = [
    "data-variant",
    "data-state",
    "data-color",
    "data-size",
    "data-picker",
    "data-required",
    "data-optional",
    "data-interactive",
    "data-hoverable",
    "data-selectable",
    "data-autofocus",
];
/**
 * Navn som aldri fjernes, heller ikke når byggefunksjonen forvalter dem.
 *
 * `id` og `for` er koblinger andre elementer peker på. `fs.errorSummary()`
 * tar en valgfri `id`, og et kall uten den strøk id-en konsumenten selv
 * hadde skrevet på boksen: hver lenke og hver `aria-*` som pekte dit, røk.
 */
const NEVER_REMOVED = ["id", "for"];
let warnedAboutUnknownSet = false;
/** Klassene systemet eier. Konsumentens egne klasser beholdes. */
const IS_SYSTEM_CLASS = /^fs-/;
/**
 * Setter attributtene fra en `fs`-funksjon på et element.
 *
 * ```ts
 * import { fs } from "@fristil/designsystem"
 *
 * const felt = document.querySelector("input")
 * fs.setAttributes(felt, fs.input({ type: "email", state: "invalid" }))
 * ```
 *
 * Kall den på nytt for å endre tilstand. Det byggefunksjonen forvalter og
 * som ikke er med i det nye settet, fjernes, og `fs-`-klassene byttes. En
 * byggefunksjon forvalter attributtene den har et valg for: `fs.spinner()`
 * har `label`, og eier dermed `role` og `aria-label`. Det den ikke har et
 * valg for, står som det står: `fs.button()` har aldri skrevet `disabled`,
 * så en deaktivert knapp er fortsatt deaktivert etterpå. `id` og `for`
 * fjernes aldri.
 *
 * Skal to sett på samme element, sendes de hver for seg, ikke spredd sammen:
 *
 * ```ts
 * fs.setAttributes(felt, fs.input({ type: "email" }), kobling.control)
 * ```
 *
 * Et spredd objekt er et nytt objekt, og da vet ikke funksjonen hvilken
 * byggefunksjon det kom fra. Det settes riktig, men bare `data-*` ryddes.
 * Et uttrykkelig `false` i et håndskrevet sett fjerner attributtet.
 *
 * Den tar `null`, og gjør da ingenting. `document.querySelector()` gir
 * `Element | null`, så uten det måtte hvert eneste kallsted i en `strict`-app
 * skrive en vakt eller et utropstegn rundt et oppslag som nesten alltid
 * treffer. Det er den samme avveiningen som `element?.classList`.
 */
export function setAttributes(element, ...sets) {
    if (!element || sets.length === 0)
        return;
    const merged = {};
    const removable = new Set();
    const classes = [];
    for (const set of sets) {
        const names = managedNames(set);
        if (names) {
            for (const name of names)
                removable.add(name);
        }
        else {
            for (const name of SAFE_FOR_UNKNOWN)
                removable.add(name);
            warnAboutUnknownSet(set);
        }
        for (const [name, value] of Object.entries(set)) {
            if (name === "class") {
                if (typeof value === "string")
                    classes.push(...value.split(/\s+/));
            }
            else if (value === false) {
                // Et uttrykkelig `false` i et håndskrevet sett er en bestilling.
                merged[name] = false;
                removable.add(name);
            }
            else if (value !== undefined) {
                // `undefined` overstyrer ikke et tidligere sett: byggefunksjonene
                // stryker slike nøkler selv, og et håndskrevet sett skal ikke kunne
                // ta `aria-invalid` fra `fs.input()` ved å nevne den uten verdi.
                merged[name] = value;
            }
        }
    }
    for (const name of NEVER_REMOVED)
        removable.delete(name);
    if (classes.length > 0) {
        const ownClasses = [...element.classList].filter((klasse) => !IS_SYSTEM_CLASS.test(klasse));
        element.className = [...new Set([...classes, ...ownClasses])]
            .filter(Boolean)
            .join(" ");
    }
    for (const name of removable) {
        if (name === "class")
            continue;
        setOrRemove(element, name, merged[name]);
    }
    for (const [name, value] of Object.entries(merged)) {
        if (removable.has(name))
            continue;
        if (value === undefined || value === false)
            continue;
        element.setAttribute(name, value === true ? "" : value);
    }
}
/**
 * Sier fra, én gang, om et sett som ser ut som en spredning av to.
 *
 * `{ ...fs.input(), ...felt.control }` settes riktig, men er et nytt objekt
 * uten en byggefunksjon bak seg. Neste kall fjerner da ikke `aria-invalid`
 * når feltet blir gyldig, og ingenting annet ville sagt det.
 */
function warnAboutUnknownSet(set) {
    if (warnedAboutUnknownSet)
        return;
    if (typeof set.class !== "string" || !/(^|\s)fs-/.test(set.class))
        return;
    warnedAboutUnknownSet = true;
    console.warn("fs.setAttributes(): settet kommer ikke rett fra en byggefunksjon, så " +
        "bare data-attributtene ryddes. Send settene hver for seg framfor å " +
        "spre dem: fs.setAttributes(element, fs.input(), felt.control).");
}
function setOrRemove(element, name, value) {
    if (value === undefined || value === false) {
        element.removeAttribute(name);
    }
    else {
        element.setAttribute(name, value === true ? "" : value);
    }
}
//# sourceMappingURL=dom.js.map