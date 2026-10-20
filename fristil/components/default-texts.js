/**
 * Hver tekst pakken selv skriver inn i siden, når ingen har gitt den en
 * annen.
 *
 * Fristil har nesten ingen tekst: den som rendrer skriver teksten, på sitt
 * eget språk. Det som står igjen her er standardverdier for tekst en
 * komponent lager selv, og hver av dem kan byttes ut. Hvordan står ved
 * siden av, og det samme står i tabellen på dokumentasjonssiden om
 * oversettelse.
 *
 * Teksten står her og ikke i komponentene, så den er ett sted å lese.
 * `scripts/sjekk-tekster.ts` feller bygget på en tekst for brukeren som står
 * i en komponent og ikke her, og krever at hver av dem står i tabellen på
 * siden om oversettelse.
 *
 * Tekstene i CSS, som « (påkrevd)», kan ikke importere noe. De leses fra en
 * `--fs-`-variabel med teksten som reserve, og `sjekk-tekster.ts` krever det.
 */
export const DEFAULT_TEXTS = {
    /** `aria-label` på regionen til `<fs-toast>`. Byttes med `label`. */
    toastRegion: "Varsler",
    /** Lukkeknappen i hver melding. Byttes med `close-label`. */
    toastClose: "Lukk melding",
    /** `<fs-connection-status>` når forbindelsen er borte. Byttes med `offline-text`. */
    connectionOffline: "Ingen forbindelse. Det du skriver blir ikke lagret.",
    /** `<fs-connection-status>` når den kommer tilbake. Byttes med `online-text`. */
    connectionOnline: "Forbindelsen er tilbake.",
    /** Antall treff i `<fs-suggestion>` ved null treff. Byttes med `count-none`. */
    suggestionNone: "Ingen treff",
    /** Antall treff ved ett treff. Byttes med `count-one`. */
    suggestionOne: "Ett treff",
    /** Antall treff ellers. `{n}` er tallet. Byttes med `count-other`. */
    suggestionOther: "{n} treff",
};
//# sourceMappingURL=default-texts.js.map