/**
 * Sjekker markup mot Fristil, utenfor nettleseren.
 *
 * `diagnoseMarkup(html)` er den samme sjekken som editorutvidelsen kjører
 * mens du skriver, og som `fristil sjekk` kjører på kommandolinjen. Den
 * finnes her fordi markup som blir til uten JavaScript, i en Go-mal, en
 * Kotlin-streng eller en Razor-visning, ikke har noen kompilator som ser
 * på attributtene. En test i appen kan da kjøre den over HTML-en serveren
 * faktisk sender:
 *
 * ```ts
 * import { diagnoseMarkup } from "@fristil/designsystem/diagnostics"
 *
 * const funn = diagnoseMarkup(html)
 * expect(funn).toEqual([])
 * ```
 *
 * Elementene og klassene den sjekker mot genereres fra komponentene, og
 * følger pakken. `diagnose(text, elements, classes)` tar egne lister.
 */
export { classes } from "./classes.js";
export { closest, diagnose, tagEnd, withoutHidden, } from "./diagnostics.js";
export { elements } from "./elements.js";
export { checkReferences, pageSource } from "./references.js";
import { classes } from "./classes.js";
import { diagnose, } from "./diagnostics.js";
import { elements } from "./elements.js";
import { checkReferences, pageSource } from "./references.js";
/** Alle funn i teksten, mot pakkens egne elementer og klasser. */
export function diagnoseMarkup(text) {
    return diagnose(text, elements, classes);
}
/**
 * Alle funn på en hel side: det `diagnoseMarkup` finner, og i tillegg at
 * hver `for`, `aria-describedby`, `aria-labelledby` og `aria-controls` peker
 * på en id som finnes, at ingen id står to ganger, og at hvert Fristil-felt
 * og hver hjelpetekst og feilmelding er koblet.
 *
 * Bruk den på HTML-en serveren sender, ikke på en mal: i en mal som er delt
 * i biter, kan id-en stå i en annen fil.
 *
 * ```ts
 * const html = await (await fetch("http://localhost:8080/skjema")).text()
 * expect(diagnosePage(html)).toEqual([])
 * ```
 */
export function diagnosePage(text, elementList = elements, classList = classes) {
    // Ordforrådet sjekkes på den samme teksten som koblingen: uten innholdet i
    // `<template>` og uten markup i attributtverdier. Se `pageSource`.
    const page = pageSource(text);
    return [
        ...diagnose(page, elementList, classList),
        ...checkReferences(page),
    ].sort((a, b) => a.start - b.start);
}
//# sourceMappingURL=index.js.map