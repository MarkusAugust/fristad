import { DEFAULT_TEXTS } from "../../default-texts.js";
import { defineElement, HostElement, whenUpgraded } from "../../host-element.js";
import { TOAST_CLASS, TOAST_CLOSE_CLASS, TOAST_MESSAGE_CLASS } from "./toast.js";
export const FS_TOAST_TAG = "fs-toast";
export const toastColors = ["neutral", "success", "warning", "danger"];
export function isToastColor(value) {
    return toastColors.includes(value);
}
const DEFAULT_DURATION = 6000;
const DEFAULT_LABEL = DEFAULT_TEXTS.toastRegion;
const DEFAULT_CLOSE_LABEL = DEFAULT_TEXTS.toastClose;
/**
 * Køen av korte meldinger i hjørnet av skjermen.
 *
 * Dette er en av de tre frittstående komponentene, og en del av pakken du
 * kaller i stedet for å skrive. Elementet er beholderen, ikke meldingen: du
 * legger det inn én gang i appen og kaller `show()` når noe skal meldes.
 * Meldingene finnes ikke før en hendelse på klienten skaper dem, så det er
 * ingenting for serveren å rendre.
 *
 * Serveren skriver beholderen med `fs.toast()`, og den setter
 * `data-ignore-morph`. Uten det river Datastars morfing meldingene bort ved
 * neste patch, fordi serverens utgave av regionen er tom.
 *
 * Beholderen er en `role="status"`-region, ikke `role="alert"`. En melding
 * som dukker opp i hjørnet skal ikke avbryte det skjermleseren holder på med;
 * er beskjeden så viktig at den må avbryte, hører den hjemme i en
 * [Alert](../../css/alert/alert.js) i selve siden. Regionen har
 * `aria-atomic="false"`: `status` er atomisk som standard, og da ble hele
 * stabelen lest opp på nytt for hver ny melding.
 *
 * Meldinger som forsvinner av seg selv er en tilgjengelighetsfelle: den som
 * leser sakte eller bruker forstørrelse rekker ikke lese dem. Derfor er
 * lukkeknappen alltid der, og tiden stopper mens musa eller fokus er i
 * meldingen.
 *
 * ```ts
 * document.querySelector("fs-toast").show("Søknaden er sendt", {
 *   color: "success",
 * })
 * ```
 */
export class FsToast extends HostElement {
    /**
     * Hva lukkeknappen heter for skjermleseren, i hver melding som vises.
     * `closeLabel` i `show()` overstyrer den for én melding.
     */
    get closeLabel() {
        return this.getAttribute("close-label") || DEFAULT_CLOSE_LABEL;
    }
    set closeLabel(value) {
        this.setAttribute("close-label", value);
    }
    /** Standard levetid i millisekunder. `0` lar meldingene bli stående. */
    get duration() {
        // `0` er en gyldig verdi og betyr at meldingene blir stående. Uten
        // sjekken mot null her forsvant de likevel etter seks sekunder, og bare
        // `show(..., { duration: 0 })` virket. Et tomt attributt er ikke null:
        // `Number("")` er 0, og `<fs-toast duration>` lot meldingene stå.
        const raw = this.getAttribute("duration");
        if (raw === null || raw.trim() === "")
            return DEFAULT_DURATION;
        const value = Number(raw);
        return Number.isFinite(value) && value >= 0 ? value : DEFAULT_DURATION;
    }
    set duration(value) {
        this.setAttribute("duration", String(value));
    }
    connectedCallback() {
        // Skrev serveren regionen med fs.toast(), står alt dette allerede. Her
        // settes det bare når det mangler, så en ren HTML-side uten byggefunksjon også
        // får en region skjermleseren forstår.
        // Klassen bærer plasseringen. Den settes her også, så en region uten
        // byggefunksjon, eller registrert under et annet navn, havner i hjørnet.
        this.classList.add(TOAST_CLASS);
        if (!this.hasAttribute("role"))
            this.setAttribute("role", "status");
        if (!this.hasAttribute("aria-live")) {
            this.setAttribute("aria-live", "polite");
        }
        if (!this.hasAttribute("aria-atomic")) {
            this.setAttribute("aria-atomic", "false");
        }
        if (!this.hasAttribute("aria-label")) {
            this.setAttribute("aria-label", this.getAttribute("label") ?? DEFAULT_LABEL);
        }
    }
    attributeChangedCallback(name, _old, value) {
        // `label` kan settes etter at elementet står i DOM-en, for eksempel av et
        // rammeverk som fyller inn attributtene i et senere steg.
        if (name === "label" && value)
            this.setAttribute("aria-label", value);
    }
    /** Viser en melding, og returnerer elementet den ble lagt i. */
    show(message, options = {}) {
        const { color = "neutral", duration = this.duration, closeLabel = this.closeLabel, } = options;
        const toast = document.createElement("div");
        toast.className = TOAST_MESSAGE_CLASS;
        // En ukjent farge gir ingen kant, framfor et `data-color` uten regel.
        if (isToastColor(color) && color !== "neutral")
            toast.dataset.color = color;
        const text = document.createElement("span");
        text.textContent = message;
        toast.append(text);
        const close = document.createElement("button");
        close.type = "button";
        close.className = TOAST_CLOSE_CLASS;
        close.setAttribute("aria-label", closeLabel);
        close.textContent = "×";
        close.addEventListener("click", () => this.dismiss(toast));
        toast.append(close);
        this.prepend(toast);
        if (duration > 0) {
            let timer = window.setTimeout(() => this.dismiss(toast), duration);
            /*
             * Tiden stopper mens brukeren leser eller er på vei til lukkeknappen,
             * og går først når verken musa eller fokus er i meldingen. Pausen var
             * to uavhengige par, og musa som gikk ut startet klokka igjen mens
             * fokus sto i meldingen: den forsvant under fingrene på brukeren.
             */
            const holds = new Set();
            const pause = (reason) => {
                holds.add(reason);
                window.clearTimeout(timer);
            };
            const resume = (reason) => {
                holds.delete(reason);
                if (holds.size > 0)
                    return;
                window.clearTimeout(timer);
                timer = window.setTimeout(() => this.dismiss(toast), duration);
            };
            toast.addEventListener("mouseenter", () => pause("pointer"));
            toast.addEventListener("mouseleave", () => resume("pointer"));
            toast.addEventListener("focusin", () => pause("focus"));
            toast.addEventListener("focusout", (event) => {
                // Innenfor meldingen, fra teksten til knappen: fortsatt pause.
                const next = event.relatedTarget;
                if (next instanceof Node && toast.contains(next))
                    return;
                resume("focus");
            });
        }
        return toast;
    }
    /** Fjerner en melding. */
    dismiss(toast) {
        if (!this.contains(toast))
            return;
        /*
         * Sto fokus i meldingen, som på lukkeknappen etter Enter, faller det
         * ellers til `body`, og neste Tab starter øverst på siden. Meldingen ved
         * siden av er det nærmeste stedet å fortsette fra: den under, ellers den
         * over.
         */
        const hadFocus = toast.contains(document.activeElement);
        const neighbour = toast.nextElementSibling ?? toast.previousElementSibling;
        toast.remove();
        if (hadFocus) {
            neighbour?.querySelector(`.${TOAST_CLOSE_CLASS}`)?.focus();
        }
        this.dispatchEvent(new CustomEvent("toast-dismiss", { bubbles: true, composed: true }));
    }
    /** Fjerner alle meldingene. */
    clear() {
        for (const toast of this.querySelectorAll(`.${TOAST_MESSAGE_CLASS}`)) {
            toast.remove();
        }
    }
}
FsToast.observedAttributes = ["duration", "label", "close-label"];
export function defineFsToast(tagName = FS_TOAST_TAG) {
    defineElement(tagName, FsToast);
}
/**
 * `element.show()`, men venter først på at `defineFsToast()` har kjørt.
 *
 * Se `whenUpgraded` for hvorfor: et kall før registreringen traff et vanlig
 * `HTMLElement` uten `show`.
 */
export async function showToast(element, message, options = {}) {
    const toast = await whenUpgraded(element);
    return toast.show(message, options);
}
//# sourceMappingURL=fs-toast.js.map