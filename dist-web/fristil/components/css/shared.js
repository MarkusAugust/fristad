/**
 * Felles byggeklosser for komponent-API-et.
 *
 * Hver komponent eksporterer én funksjon som tar et objekt med valg og returnerer
 * attributtene du sprer inn i elementet. Formen er lik for alle, slik at du
 * bare trenger å lære den én gang.
 */
/**
 * Attributtene hvert sett forvalter, også dem som er av akkurat nå.
 *
 * `attributes()` stryker nøklene uten verdi, og da er det borte hvilke
 * attributter byggefunksjonen har et ord om. `setAttributes` trenger nettopp
 * det: `fs.switch()` forvalter `disabled` og skal kunne fjerne det, mens
 * `fs.button()` aldri har skrevet det og skal la det stå. Lista ligger i et
 * kart ved siden av objektet, ikke på det, så spredning, JSON og React ser
 * det samme som før.
 */
const managed = new WeakMap();
/** Navnene settet forvalter, eller `undefined` for et sett vi ikke har laget. */
export function managedNames(set) {
    return managed.get(set);
}
/**
 * Fjerner attributter uten verdi, så `{...spredning}` ikke setter tomme felt.
 *
 * Bygger settet på et annet, som `fs.search()` på `fs.input()`, sendes det
 * andre inn som `based`. Spredningen har alt mistet nøklene som var av, og
 * uten dette forvaltet søkefeltet `data-state` bare når den tilfeldigvis var
 * satt: den kunne settes, men ikke fjernes.
 */
export function attributes(values, ...based) {
    const result = {};
    for (const [name, value] of Object.entries(values)) {
        if (value !== undefined) {
            ;
            result[name] = value;
        }
    }
    const names = new Set(Object.keys(values));
    for (const set of based) {
        for (const name of managed.get(set) ?? [])
            names.add(name);
    }
    managed.set(result, [...names]);
    return result;
}
/** Lager en typevakt for en liste av lovlige strenger. */
export function createGuard(allowed) {
    return (value) => allowed.includes(value);
}
/**
 * Valideringstilstand for skjemakontroller.
 *
 * Input, textarea og select hadde hver sin identiske type. Nå er det én, så
 * en verdi kan sendes mellom dem uten konvertering.
 */
export const fieldStates = ["default", "invalid", "success"];
export const isFieldState = createGuard(fieldStates);
/** Hvordan et påkrevd felt markeres i ledeteksten. */
export const requiredMarkers = ["symbol", "text"];
export const isRequiredMarker = createGuard(requiredMarkers);
/**
 * Sier fra én gang per melding, i konsollen.
 *
 * Byggefunksjonene er rene funksjoner uten et element å henge beskjeden på,
 * så de kan ikke bruke `warnAboutMarkup`. Dedupliseringen trengs like fullt:
 * en app som rendrer en liste med felt ville ellers sagt det samme per felt og
 * per rendring.
 */
const said = new Set();
function warnOnce(message) {
    if (typeof console === "undefined" || said.has(message))
        return;
    said.add(message);
    console.warn(message);
}
let idCounter = 0;
/**
 * Lager en id som er unik innenfor dokumentet.
 *
 * Trygg bare når markupen rendres én gang. Rendres det samme feltet både på en
 * server og i nettleseren, gir de to kjøringene to ulike id-er, og da er
 * koblingen brutt til rammeverket har rettet den opp.
 */
export function createFieldId() {
    idCounter += 1;
    return `fs-field-${idCounter}-${Math.random().toString(36).slice(2, 8)}`;
}
/**
 * Id-en byggefunksjonen fikk, eller en reserve med en beskjed.
 *
 * Hver byggefunksjon som tar en id krever den i typen. Det holder ikke alene: en
 * konsument uten TypeScript ser ingen type, og ren HTML med
 * `<script type="module">` er en førsteklasses måte å bruke Fristil på. Uten
 * reserven ble id-ene til strenger som `undefined-list`, `aria-controls` pekte
 * dit, og koblingen var brutt på en måte som så gyldig ut.
 *
 * Den tomme strengen teller som ingen id. `fs.field({ id: "" })` ga `for=""`
 * og `help.id="-help"`, altså det samme problemet uten at noe sa fra.
 *
 * Navnet på byggefunksjonen står i meldingen. Uten det sa forslagsfeltet «fs.field()»
 * og sendte utvikleren til feil sted.
 */
export function idOrFallback(builder, id, option = "id") {
    if (typeof id === "string" && id.trim() !== "")
        return id;
    warnOnce(`${builder}: ingen ${option} oppgitt, så det lages en tilfeldig. To kjøringer gir ` +
        "da to ulike, og rendres markupen både på en server og i nettleseren, " +
        `peker koblingen på noe som ikke finnes. Oppgi ${option}, i React fra useId().`);
    return createFieldId();
}
//# sourceMappingURL=shared.js.map