/**
 * Registrerer alle web-komponentene i ett kall.
 *
 * Hver komponent har sin egen `defineFs*`, og en app som bare bruker feltet
 * kan nøye seg med `defineFsField()` og holde resten utenfor det som bygges. Men de
 * fleste appene bruker flere, og spilldemoen sto med sju kall i en
 * `useEffect`, altså etter første tegning. Da er elementene i markupen
 * vanlige `HTMLElement` i det React tegner dem, og det første kallet på
 * `reportSuccess()` kom før effekten. Ett kall, ved import, lukker gapet.
 *
 * Funksjonen gjør ingenting på en server, som de enkelte gjør, så den kan stå
 * øverst i en modul som kjøres begge steder.
 */
import { defineFsConnectionStatus } from "./components/frittstaende/connection-status/fs-connection-status.js";
import { defineFsToast } from "./components/frittstaende/toast/fs-toast.js";
import { defineFsDialog } from "./components/ramme/dialog/fs-dialog.js";
import { defineFsErrorSummary } from "./components/ramme/error-summary/fs-error-summary.js";
import { defineFsField } from "./components/ramme/field/fs-field.js";
import { defineFsPopover } from "./components/ramme/popover/fs-popover.js";
import { defineFsSessionTimeout } from "./components/ramme/session-timeout/fs-session-timeout.js";
import { defineFsSuggestion } from "./components/ramme/suggestion/fs-suggestion.js";
import { defineFsTabs } from "./components/ramme/tabs/fs-tabs.js";
export function defineFs() {
    defineFsField();
    defineFsTabs();
    defineFsErrorSummary();
    defineFsPopover();
    defineFsSuggestion();
    defineFsDialog();
    defineFsToast();
    defineFsSessionTimeout();
    defineFsConnectionStatus();
}
//# sourceMappingURL=register.js.map