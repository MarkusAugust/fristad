import { attributes, createFieldId, idOrFallback } from "../../css/shared.js";
// Videreeksportert her, der den hørte hjemme før, så konsumenter som
// importerer fra `./field-core` ikke merker flyttingen.
export { createFieldId };
/**
 * Setter sammen `aria-describedby` av hjelpetekst, feilmelding og det
 * kalleren har lagt til selv.
 *
 * Feilmeldingen tas bare med når feltet faktisk er ugyldig. Ellers ville
 * `aria-describedby` pekt på et skjult element, og skjermlesere melder da
 * enten ingenting eller noe som ikke står på skjermen.
 */
export function joinDescribedBy(ids) {
    const unique = new Set();
    for (const id of ids) {
        if (!id)
            continue;
        for (const part of id.split(/\s+/)) {
            if (part)
                unique.add(part);
        }
    }
    return unique.size > 0 ? [...unique].join(" ") : undefined;
}
export function computeFieldAttributes(options) {
    const { id: givenId, help = false, error = false, required, optional, invalid = false, disabled = false, describedBy = [], } = options;
    // Reserven gjelder bare den som ikke har en typesjekk. Se `idOrFallback`.
    const id = idOrFallback("fs.field()", givenId);
    const helpId = options.helpId ?? `${id}-help`;
    const errorId = options.errorId ?? `${id}-error`;
    return {
        label: attributes({
            class: "fs-label",
            for: id,
            "data-required": required,
            "data-optional": optional && !required ? "" : undefined,
            "aria-disabled": disabled ? "true" : undefined,
        }),
        control: attributes({
            id,
            "aria-describedby": joinDescribedBy([
                help && helpId,
                invalid && error && errorId,
                ...describedBy,
            ]),
            "aria-invalid": invalid ? "true" : undefined,
            // Fargen følger med her, ikke fra input()/textarea()/select(). Ellers
            // ville begge satt aria-invalid, og i maler som skriver ut attributtene
            // bokstavelig, som Astro og ren HTML, ble de stående dobbelt.
            "data-state": invalid ? "invalid" : undefined,
            disabled: disabled ? true : undefined,
        }),
        help: attributes({ id: helpId }),
        error: attributes({
            id: errorId,
            hidden: invalid ? undefined : true,
        }),
        state: invalid ? "invalid" : "default",
    };
}
//# sourceMappingURL=field-core.js.map