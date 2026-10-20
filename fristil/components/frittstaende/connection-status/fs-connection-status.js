import { defineElement, HostElement, whenUpgraded } from "../../host-element.js";
import { CONNECTION_STATUS_BAR_CLASS, CONNECTION_STATUS_CLASS, } from "./connection-status.js";
export const FS_CONNECTION_STATUS_TAG = "fs-connection-status";
import { DEFAULT_TEXTS } from "../../default-texts.js";
const DEFAULT_OFFLINE = DEFAULT_TEXTS.connectionOffline;
const DEFAULT_ONLINE = DEFAULT_TEXTS.connectionOnline;
/** Hvor lenge «tilbake på nett» blir stående. */
const RECEIPT_MS = 4000;
/**
 * Sier fra når forbindelsen til serveren er borte.
 *
 * Serveren kan per definisjon ikke fortelle deg dette. Er den utilgjengelig,
 * kommer det ingenting derfra. Derfor er dette en frittstående komponent som
 * eier alt innholdet sitt.
 *
 * Den er ikke pynt i en app som henter innhold fra serveren underveis. En
 * side som mister strømmen sin ser helt normal ut, men er frosset, og en
 * bruker som fyller ut et skjema inn i en død forbindelse mister alt.
 *
 * `navigator.onLine` sier bare om maskinen har et nettverk, ikke om serveren
 * svarer. Derfor finnes `reportFailure()` og `reportSuccess()`, som appen
 * kaller når et kall feiler eller lykkes.
 *
 * ```ts
 * const status = document.querySelector("fs-connection-status")
 * try {
 *   await fetch("/api/lagre", { method: "POST", body })
 *   status.reportSuccess()
 * } catch {
 *   status.reportFailure()
 * }
 * ```
 */
export class FsConnectionStatus extends HostElement {
    constructor() {
        super(...arguments);
        /** Sant når appen har meldt at et kall feilet, uavhengig av navigator. */
        this.failing = false;
        this.handleOffline = () => {
            this.showOffline();
        };
        this.handleOnline = () => {
            if (this.failing)
                return;
            this.showOnline();
        };
    }
    get offlineText() {
        return this.getAttribute("offline-text") ?? DEFAULT_OFFLINE;
    }
    set offlineText(value) {
        this.setAttribute("offline-text", value);
    }
    get onlineText() {
        return this.getAttribute("online-text") ?? DEFAULT_ONLINE;
    }
    set onlineText(value) {
        this.setAttribute("online-text", value);
    }
    attributeChangedCallback() {
        // Byttes teksten mens linja står, som når språket endres, skal linja
        // si det nye. `textContent` skrives bare når teksten er en annen.
        const bar = this.bar;
        if (!bar || bar.textContent === "")
            return;
        const text = bar.dataset.state === "offline" ? this.offlineText : this.onlineText;
        if (bar.textContent !== text)
            bar.textContent = text;
    }
    connectedCallback() {
        this.classList.add(CONNECTION_STATUS_CLASS);
        window.addEventListener("offline", this.handleOffline);
        window.addEventListener("online", this.handleOnline);
        if (!navigator.onLine) {
            this.showOffline();
            return;
        }
        /*
         * Elementet kan ha vært ute av dokumentet, og da er både hendelsene og
         * tidsuret tapt. Sto kvitteringen der, ble den stående for godt, og sto
         * linja som «offline» mens nettet kom tilbake, sa den det til neste
         * gang nettet falt. `failing` er appens ord og ikke nettverkets, så det
         * står.
         */
        if (this.bar?.dataset.state === "online")
            this.scheduleRemoval(this.bar);
        else if (this.bar?.dataset.state === "offline" && !this.failing) {
            this.showOnline();
        }
    }
    disconnectedCallback() {
        window.removeEventListener("offline", this.handleOffline);
        window.removeEventListener("online", this.handleOnline);
        if (this.timer)
            window.clearTimeout(this.timer);
        this.timer = undefined;
    }
    /**
     * Linja, laget tom først og fylt etterpå.
     *
     * Et live-område må finnes før innholdet kommer, ellers annonserer
     * hjelpemidlene ikke den første endringen. Sto linja ferdig fylt i det
     * den ble satt inn, oppsto regionen ferdig for skjermleseren, og «Ingen
     * forbindelse», den viktigste meldingen, kunne gå tapt. Derfor settes
     * teksten i neste tegning når linja er ny. Det lar seg ikke etterprøve
     * uten en skjermleser; regelen er kjent fra NVDA, JAWS og VoiceOver.
     *
     * Linja lages først når den trengs, ikke ved tilkobling: sto den i DOM-en
     * før React hydrerte, fant React et element den ikke hadde rendret.
     */
    ensureBar() {
        if (this.bar)
            return { bar: this.bar, fresh: false };
        const bar = document.createElement("div");
        bar.className = CONNECTION_STATUS_BAR_CLASS;
        // polite og ikke assertive: beskjeden er viktig, men skal ikke avbryte
        // midt i et ord. Den blir stående til forbindelsen er tilbake.
        bar.setAttribute("role", "status");
        bar.setAttribute("aria-live", "polite");
        this.append(bar);
        this.bar = bar;
        return { bar, fresh: true };
    }
    write(bar, fresh, state) {
        // Teksten leses i det den skrives, ikke i det den bestilles. Ble
        // `offline-text` byttet i tegningen imellom, skrev linja den gamle.
        const text = () => state === "offline" ? this.offlineText : this.onlineText;
        if (!fresh || typeof requestAnimationFrame === "undefined") {
            bar.textContent = text();
            return;
        }
        requestAnimationFrame(() => {
            // Vakten er tilstanden, ikke linja: kom `reportSuccess()` i samme
            // tegning, står linja alt som «online», og offline-teksten skal ikke
            // skrives oppå kvitteringen.
            if (this.bar === bar && bar.dataset.state === state) {
                bar.textContent = text();
            }
        });
    }
    showOffline() {
        // Nettlesere fyrer gjerne flere `offline` på rad, og appen kan melde
        // en feil oppå et nettverk som alt er borte. Sier linja alt «offline»,
        // er det ingenting nytt å skrive eller melde: hver skriving ble lest opp
        // på nytt, og hver melding ga appen en ny `connection-lost`.
        if (this.bar?.dataset.state === "offline")
            return;
        if (this.timer)
            window.clearTimeout(this.timer);
        const { bar, fresh } = this.ensureBar();
        bar.dataset.state = "offline";
        this.write(bar, fresh, "offline");
        this.emit("connection-lost");
    }
    showOnline() {
        if (!this.bar || this.bar.dataset.state === "online")
            return;
        const bar = this.bar;
        bar.dataset.state = "online";
        this.write(bar, false, "online");
        this.emit("connection-restored");
        this.scheduleRemoval(bar);
    }
    scheduleRemoval(bar) {
        if (this.timer)
            window.clearTimeout(this.timer);
        this.timer = window.setTimeout(() => {
            bar.remove();
            this.bar = undefined;
            this.timer = undefined;
        }, RECEIPT_MS);
    }
    emit(name) {
        this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }));
    }
    /** Meld fra at et kall til serveren feilet. */
    reportFailure() {
        if (this.failing)
            return;
        this.failing = true;
        this.showOffline();
    }
    /** Meld fra at et kall til serveren lyktes igjen. */
    reportSuccess() {
        if (!this.failing)
            return;
        this.failing = false;
        this.showOnline();
    }
}
FsConnectionStatus.observedAttributes = ["offline-text", "online-text"];
export function defineFsConnectionStatus(tagName = FS_CONNECTION_STATUS_TAG) {
    defineElement(tagName, FsConnectionStatus);
}
/**
 * `element.reportFailure()`, men venter først på registreringen.
 *
 * Det var her gapet viste seg i drift: den første meldingen fra
 * hendelsesstrømmen kom før `defineFsConnectionStatus()`, og appen fikk
 * «reportSuccess is not a function». Se `whenUpgraded`.
 */
export async function reportFailure(element) {
    const status = await whenUpgraded(element);
    status.reportFailure();
}
/** `element.reportSuccess()`, men venter først på registreringen. */
export async function reportSuccess(element) {
    const status = await whenUpgraded(element);
    status.reportSuccess();
}
//# sourceMappingURL=fs-connection-status.js.map