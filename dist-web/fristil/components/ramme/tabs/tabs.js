import { attributes, idOrFallback } from "../../css/shared.js";
export const TABS_LIST_CLASS = "fs-tabs__list";
export const TABS_PANEL_CLASS = "fs-tabs__panel";
/**
 * Attributtene for en fanerad.
 *
 * Serveren skriver rollene og skjuler panelene som ikke er valgt. Gjorde
 * komponenten det, ville alle panelene vises til skriptet hadde kjørt, og
 * innholdet hoppe når det skjulte seg selv.
 *
 * Ingen bevaringsliste. Fanevalget er brukerens, og `<fs-tabs>` setter det
 * tilbake når en patch river det bort. Skal serveren kunne flytte fanen,
 * settes `server-controlled` på `<fs-tabs>`.
 *
 * ```ts
 * const faner = fs.tabs({ id: "sak", count: 2, selected: 1, label: "Velg visning" })
 * ```
 */
export const tabs = ({ id: givenId, count, selected = 0, label, }) => {
    // Reserven gjelder bare den som ikke har en typesjekk.
    const id = idOrFallback("fs.tabs()", givenId);
    // Er `selected` utenfor rekkevidde, rettes den her. Ellers ville markupen
    // sagt noe annet enn det brukeren ser.
    const valid = Math.min(Math.max(selected, 0), Math.max(count - 1, 0));
    const tabId = (index) => `${id}-tab-${index}`;
    const panelId = (index) => `${id}-panel-${index}`;
    // `Array.from` kaster på en negativ lengde. Null faner er en tom rad.
    const indices = Array.from({ length: Math.max(count, 0) }, (_, index) => index);
    return {
        list: attributes({
            class: TABS_LIST_CLASS,
            role: "tablist",
            "aria-label": label,
        }),
        tabs: indices.map((index) => attributes({
            id: tabId(index),
            role: "tab",
            type: "button",
            "aria-selected": (index === valid ? "true" : "false"),
            "aria-controls": panelId(index),
            // Rullerende tabindex: bare den valgte fanen er en tabbestopp, så Tab
            // går fra raden og rett inn i panelet.
            tabindex: (index === valid ? "0" : "-1"),
        })),
        panels: indices.map((index) => attributes({
            id: panelId(index),
            class: TABS_PANEL_CLASS,
            role: "tabpanel",
            "aria-labelledby": tabId(index),
            // Panelet får fokus når det ikke har noe å fokusere på selv, ellers
            // hopper Tab rett forbi innholdet som nettopp ble vist.
            tabindex: "0",
            hidden: index === valid ? undefined : true,
        })),
    };
};
//# sourceMappingURL=tabs.js.map