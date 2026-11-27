import { defineElement, HostElement, whenUpgraded } from "../../host-element.js";
import { CONNECTION_STATUS_BAR_CLASS, CONNECTION_STATUS_CLASS, } from "./connection-status.js";
export const LT_CONNECTION_STATUS_TAG = "lt-connection-status";
import { DEFAULT_TEXTS } from "../../default-texts.js";
const DEFAULT_OFFLINE = DEFAULT_TEXTS.connectionOffline;
const DEFAULT_ONLINE = DEFAULT_TEXTS.connectionOnline;
/** How long "back online" stays visible. */
const RECEIPT_MS = 4000;
/**
 * Tells the user when the connection to the server is lost.
 *
 * By definition the server cannot tell you this. When it is unreachable,
 * nothing comes from it. So this is a standalone component that creates all
 * of its content itself.
 *
 * It is not decoration in an app that fetches content from the server as it
 * goes. A page that loses its stream looks perfectly normal but is frozen,
 * and a user filling in a form over a dead connection loses everything.
 *
 * `navigator.onLine` only says whether the machine has a network, not whether
 * the server answers. That is why `reportFailure()` and `reportSuccess()`
 * exist: the app calls them when a request fails or succeeds.
 *
 * ```ts
 * const status = document.querySelector("lt-connection-status")
 * try {
 *   const response = await fetch("/api/save", { method: "POST", body })
 *   if (!response.ok) throw new Error(response.statusText)
 *   status.reportSuccess()
 * } catch {
 *   status.reportFailure()
 * }
 * ```
 */
export class LtConnectionStatus extends HostElement {
    constructor() {
        super(...arguments);
        /** True when the app has reported a failed request, regardless of navigator. */
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
        // If the text changes while the bar is showing, as when the language
        // changes, the bar says the new one. `textContent` is only written when
        // the text differs.
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
         * The element may have been out of the document, and then both the
         * events and the timer are lost. If the receipt was showing, it stayed
         * for good, and if the bar said "offline" while the network came back,
         * it kept saying so until the next time the network dropped. `failing`
         * is the app's word and not the network's, so it stays.
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
     * The bar, created empty first and filled afterwards.
     *
     * A live region must exist before its content arrives, or assistive
     * technology does not announce the first change. When the bar was already
     * filled as it was inserted, the region appeared complete to the screen
     * reader, and "No connection", the most important message, could be lost.
     * So the text is set in the next frame when the bar is new. This cannot be
     * checked without a screen reader; the rule is known from NVDA, JAWS and
     * VoiceOver.
     *
     * The bar is created only when it is needed, not on connection: when it
     * was in the DOM before React hydrated, React found an element it had not
     * rendered.
     */
    ensureBar() {
        if (this.bar)
            return { bar: this.bar, fresh: false };
        const bar = document.createElement("div");
        bar.className = CONNECTION_STATUS_BAR_CLASS;
        // polite and not assertive: the message matters, but should not
        // interrupt mid-word. It stays until the connection is back.
        bar.setAttribute("role", "status");
        bar.setAttribute("aria-live", "polite");
        this.append(bar);
        this.bar = bar;
        return { bar, fresh: true };
    }
    write(bar, fresh, state) {
        // The text is read when it is written, not when it is scheduled. If
        // `offline-text` changed in the frame between, the bar wrote the old one.
        const text = () => state === "offline" ? this.offlineText : this.onlineText;
        if (!fresh || typeof requestAnimationFrame === "undefined") {
            bar.textContent = text();
            return;
        }
        requestAnimationFrame(() => {
            // The guard is the state, not the bar: if `reportSuccess()` came in
            // the same frame, the bar already says "online", and the offline text
            // must not be written over the receipt.
            if (this.bar === bar && bar.dataset.state === state) {
                bar.textContent = text();
            }
        });
    }
    showOffline() {
        // Browsers often fire several `offline` events in a row, and the app
        // can report a failure on top of a network that is already gone. If the
        // bar already says "offline", there is nothing new to write or report:
        // each write was announced again, and each report gave the app a new
        // `connection-lost`.
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
    /** Reports that a request to the server failed. */
    reportFailure() {
        if (this.failing)
            return;
        this.failing = true;
        this.showOffline();
    }
    /** Reports that a request to the server succeeded again. */
    reportSuccess() {
        if (!this.failing)
            return;
        this.failing = false;
        this.showOnline();
    }
}
LtConnectionStatus.observedAttributes = ["offline-text", "online-text"];
export function defineLtConnectionStatus(tagName = LT_CONNECTION_STATUS_TAG) {
    defineElement(tagName, LtConnectionStatus);
}
/**
 * `element.reportFailure()`, but waits for the registration first.
 *
 * This is where the gap showed up in production: the first message from the
 * event stream came before `defineLtConnectionStatus()`, and the app got
 * "reportSuccess is not a function". See `whenUpgraded`.
 */
export async function reportFailure(element) {
    const status = await whenUpgraded(element);
    status.reportFailure();
}
/** `element.reportSuccess()`, but waits for the registration first. */
export async function reportSuccess(element) {
    const status = await whenUpgraded(element);
    status.reportSuccess();
}
//# sourceMappingURL=lt-connection-status.js.map