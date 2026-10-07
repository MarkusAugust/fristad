import { attributes } from "../../css/shared.js";
import { DEFAULT_TEXTS } from "../../default-texts.js";
/**
 * Klassen på verten, altså regionen meldingene ligger i.
 *
 * Fram til 0.28.0 var dette klassen på hver melding, og regionen ble stylet
 * med elementnavnet. Registrert under et annet navn, med
 * `defineFsToast("min-toast")`, mistet den da plasseringen sin.
 */
export const TOAST_CLASS = "fs-toast";
export const TOAST_MESSAGE_CLASS = "fs-toast__message";
export const TOAST_CLOSE_CLASS = "fs-toast__close";
/**
 * Attributtene for varselregionen.
 *
 * Regionen er `role="status"` og ikke `role="alert"`. En melding som kommer
 * fordi noe gikk bra skal ikke avbryte det skjermleseren holder på med.
 *
 * `data-ignore-morph` er ikke valgfri. `<fs-toast>` lager og fjerner sine egne
 * elementer inni regionen, og uten attributtet river Datastars morfing dem bort
 * ved neste patch, fordi serverens utgave av regionen er tom. Attributtet må
 * stå i HTML-en serveren sender: komponenten kan ikke sette det på seg selv,
 * siden morfingen leser det fra serverens node.
 */
export const toast = ({ label = DEFAULT_TEXTS.toastRegion, closeLabel, } = {}) => ({
    host: attributes({
        class: TOAST_CLASS,
        role: "status",
        "aria-live": "polite",
        // `status` er atomisk som standard, og da leses hele stabelen opp på
        // nytt for hver melding som kommer til.
        "aria-atomic": "false",
        "aria-label": label,
        "close-label": closeLabel,
        "data-ignore-morph": "",
    }),
    message: attributes({ class: TOAST_MESSAGE_CLASS }),
    close: attributes({ class: TOAST_CLOSE_CLASS }),
});
//# sourceMappingURL=toast.js.map