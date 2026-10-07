import { buildMatrix, FRISTIL_BRANDS } from "./matrix.js";
export const Breakpoints = {
    sm: "640px",
    md: "768px",
    lg: "1024px",
    xl: "1280px",
    "2xl": "1536px",
};
export const Containers = {
    xs: "100%",
    sm: "576px",
    md: "960px",
    lg: "1152px",
    xl: "1536px",
    aside: "384px",
    asideNarrow: "288px",
    wideContent: "768px",
};
/**
 * Alt som ikke er farge.
 *
 * Fargene står ikke her. De regnes av kontrakten i `contract.ts` og bygges av
 * `buildMatrix`, så systemets egne farger er den samme kontrakten anvendt på
 * systemets egne kulører. Se `colorTokens` under.
 */
export const cssTokens = {
    // Tekststørrelser
    "--fs-font-size-reference": "16px",
    "--fs-font-size-xxs": "0.625rem",
    "--fs-font-size-xs": "0.75rem",
    "--fs-font-size-s": "0.875rem",
    "--fs-font-size-m": "1rem",
    "--fs-font-size-l": "1.125rem",
    "--fs-font-size-xl": "1.375rem",
    "--fs-font-size-xxl": "1.875rem",
    "--fs-font-size-mega": "2.625rem",
    /*
     * Skriftvekter og linjeavstander står her, ikke i hver komponent, fordi en
     * organisasjon som tar systemet i bruk gjerne har sin egen typografi. Et
     * annet norsk designsystem har fete knapper med linjeavstand 1,666, våre er
     * halvfete med 1,5, og forskjellen skal settes ett sted.
     */
    "--fs-font-weight-regular": "400",
    "--fs-font-weight-medium": "500",
    "--fs-font-weight-semibold": "600",
    "--fs-font-weight-bold": "700",
    /** Kontroller og knapper. */
    "--fs-line-height-default": "1.5",
    /** Overskrifter. */
    "--fs-line-height-heading": "1.2",
    /** Brødtekst i en artikkel. */
    "--fs-line-height-article": "1.6",
    /** Tettere tekst i små flater: merkelapper, hjelpebobler. */
    "--fs-line-height-compact": "1.4",
    /*
     * Avstandsskalaen, med 4 piksler som grunnenhet.
     *
     * Én skala, ikke to. Tailwind har ett navnerom for både avstand og
     * størrelse, `--spacing-*`, og regner `p-4` som `calc(var(--spacing) * 4)`.
     * To skalaer ville betydd at bare den ene kunne kobles dit.
     */
    "--fs-spacing-px": "1px",
    "--fs-spacing-0-5": "0.125rem",
    "--fs-spacing-1": "0.25rem",
    "--fs-spacing-2": "0.5rem",
    "--fs-spacing-3": "0.75rem",
    "--fs-spacing-4": "1rem",
    "--fs-spacing-5": "1.25rem",
    "--fs-spacing-6": "1.5rem",
    "--fs-spacing-7": "1.75rem",
    "--fs-spacing-8": "2rem",
    "--fs-spacing-10": "2.5rem",
    "--fs-spacing-12": "3rem",
    "--fs-spacing-16": "4rem",
    /*
     * Ikonene er bilder med streken malt inn, så de følger ikke matrisen.
     *
     * Et ikon tegnet inn i en bakgrunn kan ikke lese en CSS-variabel. De har
     * derfor en mørk og en lys utgave, og snur med temaet.
     */
    "--fs-icon-search": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%234d4d4d' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='11' cy='11' r='7'/%3E%3Cline x1='16.5' y1='16.5' x2='21' y2='21'/%3E%3C/svg%3E\")",
    "--fs-icon-check": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23ffffff' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='20 6 9 17 4 12'/%3E%3C/svg%3E\")",
    "--fs-icon-dash": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23ffffff' stroke-width='3' stroke-linecap='round'%3E%3Cline x1='6' y1='12' x2='18' y2='12'/%3E%3C/svg%3E\")",
    "--fs-icon-calendar": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%234d4d4d' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Crect x='3' y='4' width='18' height='18' rx='2' ry='2'/%3E%3Cline x1='16' y1='2' x2='16' y2='6'/%3E%3Cline x1='8' y1='2' x2='8' y2='6'/%3E%3Cline x1='3' y1='10' x2='21' y2='10'/%3E%3C/svg%3E\")",
    "--fs-icon-clock": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%234d4d4d' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='12' cy='12' r='9'/%3E%3Cpolyline points='12 7 12 12 15 15'/%3E%3C/svg%3E\")",
};
/** Ikonene i mørkt tema, med lysere strek. */
export const darkTokens = {
    "--fs-icon-search": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23b2b2b2' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='11' cy='11' r='7'/%3E%3Cline x1='16.5' y1='16.5' x2='21' y2='21'/%3E%3C/svg%3E\")",
    "--fs-icon-check": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%231a1a1a' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='20 6 9 17 4 12'/%3E%3C/svg%3E\")",
    "--fs-icon-dash": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%231a1a1a' stroke-width='3' stroke-linecap='round'%3E%3Cline x1='6' y1='12' x2='18' y2='12'/%3E%3C/svg%3E\")",
    "--fs-icon-calendar": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23b2b2b2' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Crect x='3' y='4' width='18' height='18' rx='2' ry='2'/%3E%3Cline x1='16' y1='2' x2='16' y2='6'/%3E%3Cline x1='8' y1='2' x2='8' y2='6'/%3E%3Cline x1='3' y1='10' x2='21' y2='10'/%3E%3C/svg%3E\")",
    "--fs-icon-clock": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23b2b2b2' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='12' cy='12' r='9'/%3E%3Cpolyline points='12 7 12 12 15 15'/%3E%3C/svg%3E\")",
};
/**
 * Systemfarger som ikke er en celle i matrisen.
 *
 * Avslått tilstand er unntatt kontrastkravet i WCAG 1.4.3, og verdien er med
 * vilje lav: en kontroll som er slått av skal se av. 3,44:1 mot flaten er
 * fortsatt lesbart, mens en lysere grå ga 1,83 og var det ikke.
 *
 * Flaten bak en modal og skyggen under et panel er gjennomsiktig sort i begge
 * temaer, altså ikke en farge fra en skala.
 */
const SYSTEM_COLORS = {
    "--fs-color-disabled-surface": "var(--fs-color-neutral-raised)",
    "--fs-color-disabled-text": "var(--fs-color-neutral-border)",
    "--fs-focus-ring": "2px solid var(--fs-color-accent-border-strong)",
};
/*
 * `buildMatrix` tar merkefarger fra en konsument, så nøklene den gir er
 * `string`. Her er merkene systemets egne, og da er de nøyaktig `MatrixToken`.
 * At de virkelig er det, er en påstand `contract.browser.test.ts` gjør: «har
 * hver familie med hver rolle» kaller matrisen og krever hver celle.
 */
const lightCells = buildMatrix(FRISTIL_BRANDS, "light").tokens;
const darkCells = buildMatrix(FRISTIL_BRANDS, "dark").tokens;
export const colorTokens = {
    ...lightCells,
    ...SYSTEM_COLORS,
    "--fs-color-overlay": "#1a1a1a80",
    "--fs-shadow-overlay": "0 16px 40px rgba(0, 0, 0, 0.12)",
};
export const darkColorTokens = {
    ...darkCells,
    "--fs-color-overlay": "rgba(0, 0, 0, 0.72)",
    "--fs-shadow-overlay": "0 16px 40px rgba(0, 0, 0, 0.6)",
};
//# sourceMappingURL=tokens.js.map