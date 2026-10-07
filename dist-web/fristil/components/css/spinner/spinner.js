import { attributes, createGuard } from "../shared.js";
export const SPINNER_CLASS = "fs-spinner";
export const spinnerSizes = ["small", "medium", "large"];
/**
 * Attributtene for en venteindikator.
 *
 * Uten `label` er ringen bare en form, og skjermlesere melder ingenting. Med
 * `label` blir den en `role="status"` som sier hva som pågår. Har du allerede
 * en synlig tekst ved siden av, la den bære beskjeden og dropp `label`, så
 * det samme ikke leses opp to ganger.
 *
 * ```ts
 * <span {...spinner({ label: "Laster søknader" })} />
 * ```
 */
export const spinner = Object.assign(({ size = "medium", label } = {}) => attributes({
    class: SPINNER_CLASS,
    "data-size": size === "medium" ? undefined : size,
    role: label ? "status" : undefined,
    "aria-label": label,
}), {
    sizes: spinnerSizes,
    isSize: createGuard(spinnerSizes),
});
//# sourceMappingURL=spinner.js.map