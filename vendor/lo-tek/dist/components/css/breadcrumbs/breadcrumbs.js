import { attributes } from "../shared.js";
export const BREADCRUMBS_CLASS = "lt-breadcrumbs";
/**
 * The attributes for a breadcrumb trail.
 *
 * The class goes on an `<ol>` inside a `<nav>` with a text that says what
 * the navigation is, for example `aria-label="You are here"`. The order
 * matters, so the list must be numbered even though the numbers are not
 * shown.
 *
 * The separator between the steps is added with CSS. Had it been in the
 * markup, screen readers would have read "slash" between every step.
 *
 * ```ts
 * <nav aria-label="You are here">
 *   <ol {...breadcrumbs()}>
 *     <li><a href="/">Home</a></li>
 *     <li><a href="/cases" aria-current="page">My cases</a></li>
 *   </ol>
 * </nav>
 * ```
 */
export const breadcrumbs = () => attributes({ class: BREADCRUMBS_CLASS });
//# sourceMappingURL=breadcrumbs.js.map