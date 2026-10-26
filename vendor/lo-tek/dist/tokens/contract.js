/**
 * The colour contract, as `@lo-tek/interfaces/contract`: what each role is,
 * and what it promises.
 *
 * One colour layer, not two. A colour is a point in a matrix of **family**
 * (what it means) and **role** (what it does), and every family has the same
 * roles. The hue comes from the consumer, the lightness from the role, and so
 * the contrast is guaranteed by construction rather than by a check
 * afterwards.
 *
 * The numbers are in `color-contract.json`, which the core reads, and the
 * arithmetic happens there. Here is the data, for anyone who wants to read it.
 * A theme is built with `buildTheme` from `@lo-tek/interfaces/theme`, and a
 * hand-written theme is checked with `inspectTheme` from
 * `@lo-tek/interfaces/theme-check`.
 */
export { NEUTRAL_LAYERS, REQUIREMENT, ROLES, } from "./matrix-cells.js";
//# sourceMappingURL=contract.js.map