import { attributes, idOrFallback } from "../../css/shared.js";
export const POPOVER_CLASS = "fs-popover";
/** Hvilken kant av knappen panelet henger fra. `start` og `end` følger leseretningen. */
export const popoverPlacements = [
    "bottom-start",
    "bottom-end",
    "top-start",
    "top-end",
];
export function isPopoverPlacement(value) {
    return popoverPlacements.includes(value);
}
/**
 * Attributtene for et sprettoppvindu.
 *
 * Serveren skriver både knappen og panelet med koblingen mellom dem.
 * `<fs-popover>` regner ut posisjonen og lukker ved klikk utenfor.
 *
 * ```ts
 * const boks = fs.popover({ id: "hjelp" })
 * ```
 * ```html
 * <fs-popover>
 *   <button {...boks.trigger}>Hva betyr dette?</button>
 *   <div {...boks.panel}>Vi bruker fødselsdatoen til å finne riktig sats.</div>
 * </fs-popover>
 * ```
 */
export const popover = Object.assign(({ id: givenId, open = false, placement = "bottom-start", }) => {
    // Reserven gjelder bare den som ikke har en typesjekk.
    const id = idOrFallback("fs.popover()", givenId);
    return {
        host: attributes({
            // `open` må stå på verten, ikke bare i knappens aria-expanded. Uten det
            // sa markupen at panelet var åpent mens komponenten mente det var lukket.
            // Ingen bevaringsliste: river en patch attributtet bort, setter
            // komponenten det tilbake, og `server-controlled` slår av reparasjonen
            // når serveren skal eie tilstanden.
            open: open ? true : undefined,
            placement: placement === "bottom-start" ? undefined : placement,
        }),
        trigger: attributes({
            "aria-expanded": (open ? "true" : "false"),
            "aria-controls": id,
        }),
        panel: attributes({
            class: POPOVER_CLASS,
            id,
            // `manual` og ikke `auto`: komponenten lukker selv, slik at knappen kan
            // brukes til å lukke igjen uten at nettleseren rekker å lukke først.
            popover: "manual",
        }),
    };
}, {
    // Vakten henger på funksjonen, som ellers i `fs`. Lista ligger ved siden
    // av, som `popoverPlacements`, og ikke på funksjonen: `placement` styrer
    // hvor JavaScript legger panelet og ikke CSS, og vaktposten for
    // klassene krever at hver liste på en byggefunksjon gjør noe i CSS-en.
    isPlacement: isPopoverPlacement,
});
//# sourceMappingURL=popover.js.map