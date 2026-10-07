import { attributes } from "../shared.js";
export const SKIP_LINK_CLASS = "fs-skip-link";
/**
 * Attributtene for en hopp-til-innhold-lenke.
 *
 * Lenken skal være det første elementet i `<body>`, og peke på id-en til
 * hovedinnholdet. Den ligger utenfor skjermen til den får fokus, og kan
 * derfor ikke skjules med `display: none`: da kan tastaturet heller ikke nå
 * den, og hele poenget forsvinner.
 *
 * ```ts
 * <a {...skipLink()} href="#hovedinnhold">Hopp til hovedinnhold</a>
 * ```
 */
export const skipLink = () => attributes({ class: SKIP_LINK_CLASS });
//# sourceMappingURL=skip-link.js.map