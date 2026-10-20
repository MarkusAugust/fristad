import { accordion } from "./components/css/accordion/accordion.js";
import { alert } from "./components/css/alert/alert.js";
import { avatar } from "./components/css/avatar/avatar.js";
import { badge } from "./components/css/badge/badge.js";
import { breadcrumbs } from "./components/css/breadcrumbs/breadcrumbs.js";
import { button } from "./components/css/button/button.js";
import { card } from "./components/css/card/card.js";
import { checkbox } from "./components/css/checkbox/checkbox.js";
import { divider } from "./components/css/divider/divider.js";
import { errorText } from "./components/css/error-text/error-text.js";
import { fieldset, legend } from "./components/css/fieldset/fieldset.js";
import { fileUpload } from "./components/css/file-upload/file-upload.js";
import { heading } from "./components/css/heading/heading.js";
import { helpText } from "./components/css/help-text/help-text.js";
import { input } from "./components/css/input/input.js";
import { label } from "./components/css/label/label.js";
import { link } from "./components/css/link/link.js";
import { list } from "./components/css/list/list.js";
import { pagination } from "./components/css/pagination/pagination.js";
import { paragraph } from "./components/css/paragraph/paragraph.js";
import { progress } from "./components/css/progress/progress.js";
import { radio } from "./components/css/radio/radio.js";
import { search } from "./components/css/search/search.js";
import { select } from "./components/css/select/select.js";
import { fieldStates, isFieldState, isRequiredMarker, requiredMarkers, } from "./components/css/shared.js";
import { skeleton } from "./components/css/skeleton/skeleton.js";
import { skipLink } from "./components/css/skip-link/skip-link.js";
import { spinner } from "./components/css/spinner/spinner.js";
import { srOnly } from "./components/css/sr-only/sr-only.js";
import { switchControl } from "./components/css/switch/switch.js";
import { table, tableSort } from "./components/css/table/table.js";
import { tag } from "./components/css/tag/tag.js";
import { textarea } from "./components/css/textarea/textarea.js";
import { toggleGroup } from "./components/css/toggle-group/toggle-group.js";
import { tooltip } from "./components/css/tooltip/tooltip.js";
import { connectionStatus } from "./components/frittstaende/connection-status/connection-status.js";
import { toast } from "./components/frittstaende/toast/toast.js";
import { dialog } from "./components/ramme/dialog/dialog.js";
import { errorSummary } from "./components/ramme/error-summary/error-summary.js";
import { computeFieldAttributes } from "./components/ramme/field/field-core.js";
import { popover } from "./components/ramme/popover/popover.js";
import { sessionTimeout } from "./components/ramme/session-timeout/session-timeout.js";
import { suggestion } from "./components/ramme/suggestion/suggestion.js";
import { tabs } from "./components/ramme/tabs/tabs.js";
import { setAttributes } from "./dom.js";
/**
 * Hele komponent-API-et på ett sted.
 *
 * Hver komponent er en funksjon som tar et objekt med valg og returnerer
 * attributtene du sprer inn i elementet:
 *
 * ```ts
 * import { fs } from "@fristil/designsystem"
 *
 * <button {...fs.button({ variant: "secondary" })}>Lagre utkast</button>
 * <input {...fs.input({ type: "email", state: "invalid" })} />
 * <span {...fs.badge({ color: "success" })}>Innvilget</span>
 * ```
 *
 * Formen er den samme for alle, så du lærer den én gang. Skriv `fs.` i
 * editoren for å se hva som finnes; skriv `fs.button.` for å se de lovlige
 * variantene og vakten som validerer verdier utenfra.
 *
 * Returverdiene er vanlige objekter. Det er grunnen til at det samme API-et
 * virker i React, Astro, Datastar og ren HTML.
 */
export const fs = {
    accordion,
    alert,
    avatar,
    badge,
    breadcrumbs,
    button,
    card,
    checkbox,
    /**
     * Linja som sier fra at forbindelsen til serveren er borte. Serveren kan
     * ikke melde det selv, så `<fs-connection-status>` eier innholdet.
     */
    connectionStatus,
    dialog,
    divider,
    errorText,
    /**
     * Feiloppsummeringen over et skjema. Serveren skriver hele boksen, også
     * overskriften, og `<fs-error-summary>` flytter bare fokus.
     */
    errorSummary,
    /**
     * Kobler sammen ledetekst, felt, hjelpetekst og feilmelding.
     *
     * ```ts
     * const felt = fs.field({ id: "epost", required: "text", help: true, error: true, invalid })
     *
     * <label {...felt.label}>E-postadresse</label>
     * <input {...fs.input({ type: "email", state: felt.state })} {...felt.control} />
     * <p {...fs.helpText()} {...felt.help}>Vi sender kvittering hit.</p>
     * <p {...fs.errorText()} {...felt.error}>Skriv en gyldig adresse.</p>
     * ```
     *
     * Samme utregning som `<fs-field>` gjør, men som data, for markup som
     * lages med JavaScript. Da trengs ingen web component.
     */
    field: computeFieldAttributes,
    /** Gruppe av kontroller som hører sammen. Ledeteksten er `fs.legend`. */
    fieldset,
    heading,
    fileUpload,
    helpText,
    input,
    label,
    legend,
    link,
    list,
    pagination,
    paragraph,
    /**
     * Knappen og panelet i et sprettoppvindu. `<fs-popover>` regner ut
     * posisjonen, serveren skriver koblingen.
     */
    popover,
    radio,
    search,
    select,
    /**
     * Varselet om at innlogget økt går ut. Nedtellingen er klientens klokke,
     * så `<fs-session-timeout>` eier innholdet.
     */
    sessionTimeout,
    skeleton,
    skipLink,
    progress,
    spinner,
    /** Tekst bare skjermlesere skal få. Krever `sr-only.css`. */
    srOnly,
    /**
     * Feltet og forslagslista. `<fs-suggestion>` tar tastaturet og
     * filtreringen, serveren skriver markupen.
     */
    suggestion,
    /** Av og på. Heter `switchControl` når du importerer den direkte. */
    switch: switchControl,
    /**
     * Fanerad med paneler. Serveren skriver rollene og skjuler panelene som
     * ikke er valgt, `<fs-tabs>` tar piltastene.
     */
    tabs,
    /**
     * Varselregionen. Setter `data-ignore-morph`, fordi `<fs-toast>` lager og
     * fjerner sine egne meldinger inni den.
     */
    toast,
    toggleGroup,
    tooltip,
    table,
    /**
     * Overskriften og knappen i en kolonne som kan sorteres. `aria-sort` på
     * `<th>` er tilstanden, og pila tegnes fra den. Sorteringen er appens.
     */
    tableSort,
    tag,
    textarea,
    /**
     * Bruker et attributtsett på et element i vanlig DOM.
     *
     * I JSX sprer du objektet. Uten JSX gjør denne jobben det, og rydder bort
     * det byggefunksjonen forvalter fra forrige tilstand. Flere sett sendes
     * hver for seg: `fs.setAttributes(felt, fs.input(), kobling.control)`.
     */
    setAttributes,
    /** Valideringstilstandene som deles av input, textarea og select. */
    states: fieldStates,
    isState: isFieldState,
    /** Måtene et påkrevd felt kan markeres på. */
    markers: requiredMarkers,
    isMarker: isRequiredMarker,
};
//# sourceMappingURL=fs.js.map