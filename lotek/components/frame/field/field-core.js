import { attributes, createFieldId, idOrFallback } from "../../css/shared.js";
// Re-exported here, where it used to live, so consumers that import from
// `./field-core` do not notice the move.
export { createFieldId };
/**
 * Joins `aria-describedby` from the help text, the error text and whatever
 * the caller added.
 *
 * The error text is only included when the field is actually invalid.
 * Otherwise `aria-describedby` would point to a hidden element, and screen
 * readers then announce either nothing or something that is not on the
 * screen.
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
    // The fallback is only for callers without a type check. See `idOrFallback`.
    const id = idOrFallback("lt.field()", givenId);
    const helpId = options.helpId ?? `${id}-help`;
    const errorId = options.errorId ?? `${id}-error`;
    return {
        label: attributes({
            class: "lt-label",
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
            // The colour comes from here, not from input()/textarea()/select().
            // Otherwise both would set aria-invalid, and in templates that write
            // the attributes out literally, such as Astro and plain HTML, it
            // appeared twice.
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