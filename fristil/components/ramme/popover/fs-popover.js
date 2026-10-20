import { addClass, defineElement, HostElement, isServerControlled, SERVER_CONTROLLED, setAttr, setFlag, uniqueId, warnAboutMarkup, } from "../../host-element.js";
import { isPopoverPlacement, POPOVER_CLASS, } from "./popover.js";
export const FS_POPOVER_TAG = "fs-popover";
/**
 * Et panel som henger under en knapp, og som lukker seg selv.
 *
 * Panelet bruker nettleserens egen `popover`, så det havner i topplaget og
 * legger seg over alt annet uten `z-index`. `fs.popover()` skriver
 * `popover="manual"`, så Escape og klikk utenfor er komponentens: med `auto`
 * rakk nettleseren å lukke panelet før knappen fikk klikket sitt, og knappen
 * kunne ikke brukes til å lukke igjen. Håndskrevet markup med bare `popover`
 * er `auto`, og da følger komponenten med når nettleseren lukker det.
 *
 * Det topplaget ikke gjør er å plassere panelet. `position-anchor` finnes
 * ennå ikke i alle nettlesere, så posisjonen regnes ut her.
 *
 * Lages markupen med JavaScript, skriver `fs.popover()` koblingen:
 * `aria-controls` på knappen, klassen, id-en og `popover` på panelet.
 * Kommer markupen fra en mal uten JavaScript, holder det med en knapp og et
 * panel med klassen `fs-popover`: komponenten setter `popover="manual"`,
 * lager id-en og skriver `aria-controls` på knappen. Bare det som mangler,
 * så det serveren skrev står. Komponenten setter `open` på verten,
 * `aria-expanded` på knappen og posisjonen på panelet, og setter alt den
 * har skrevet tilbake når en patch river det bort. Malen trenger ingen
 * `data-preserve-attr`.
 *
 * Reparasjonen gjelder én vei: har noen bedt om at vinduet er åpent, blir det
 * stående gjennom en patch. Sender serveren `open`, åpnes det, for det er noe
 * serveren faktisk sa.
 *
 * Skal serveren eie tilstanden, settes `server-controlled` på verten. Det er
 * også svaret når siden styrer `open` med et attributt utenfra, som med
 * Datastars `data-attr:open`: et fjernet attributt er ikke til å skille fra en
 * morfing, mens `meny.open = false` er en beskjed komponenten kan se.
 *
 * ```html
 * <fs-popover placement="bottom-end">
 *   <button class="fs-button" aria-controls="meny" aria-expanded="false">Handlinger</button>
 *   <ul id="meny" class="fs-popover" popover="manual">…</ul>
 * </fs-popover>
 * ```
 */
export class FsPopover extends HostElement {
    constructor() {
        super(...arguments);
        /**
         * Hva komponenten sist ble bedt om, gjennom `open`-egenskapen.
         *
         * `open` bor på verten, og en morfing river bort alt som ikke står i
         * serverens HTML. Uten noe mer lukket hver eneste patch et vindu brukeren
         * nettopp hadde åpnet.
         *
         * Et fjernet attributt ser likt ut uansett hvem som fjernet det, så
         * komponenten kan ikke se forskjell på en morfing og en app. Skillet går i
         * stedet på **hvordan** appen sier fra: går den gjennom egenskapen, altså
         * `meny.open = false`, `hide()` eller `toggle()`, er det en beskjed, og den
         * følges. Setter noe attributtet direkte, som Datastars `data-attr:open`,
         * er det ikke til å skille fra en morfing, og da skal siden si
         * `server-controlled` og la serveren eie tilstanden.
         */
        this.wantsOpen = false;
        this.handleTriggerClick = () => {
            this.toggle();
        };
        this.handlePanelToggle = (event) => {
            const state = event.newState;
            // Komponentens egne `showPopover()` og `hidePopover()` gir også
            // hendelsen, men da stemmer `open` alt, og ingenting skjer.
            if (state === "closed" && this.open)
                this.hide();
            else if (state === "open" && !this.open)
                this.show();
        };
        this.handleOutsideClick = (event) => {
            // `composedPath()` og ikke `target`: står komponenten i en skyggerot,
            // er `target` omdirigert til skyggeverten, og et klikk i selve panelet
            // så ut som et klikk utenfor.
            const target = event.composedPath()[0];
            if (this.contains(target) || this.panel?.contains(target))
                return;
            this.hide();
        };
        this.handleKeydown = (event) => {
            if (event.key !== "Escape" || !this.open)
                return;
            // Kom tastetrykket fra knappen eller panelet, er det brukt. Uten dette
            // lukket det også en modal dialog vinduet sto i: ett trykk tok begge,
            // og brukeren mistet dialogen. Kom det fra et annet sted, som en dialog
            // åpnet oppå vinduet, er det ikke vinduets å stoppe.
            const target = event.composedPath()[0];
            if (this.contains(target) || this.panel?.contains(target)) {
                event.preventDefault();
            }
            this.hide();
            // Fokus tilbake til knappen. Uten dette står fokus på et panel som ikke
            // lenger finnes, og neste tastetrykk starter på toppen av siden.
            this.triggerElement?.focus();
        };
        /** Regner ut hvor panelet skal stå, mot knappens plass på skjermen. */
        this.reposition = () => {
            if (!this.open || !this.panel || !this.triggerElement)
                return;
            const anchor = this.triggerElement.getBoundingClientRect();
            const panel = this.panel.getBoundingClientRect();
            const space = 4;
            const below = this.placement.startsWith("bottom");
            // `start` og `end` følger leseretningen: i et dokument som leses fra
            // høyre er `start` knappens høyre kant. Uten dette lå panelet på feil
            // side av knappen i RTL. `:dir(rtl)` og ikke `direction` fra
            // `getComputedStyle`: stilarket snur verdien med `:dir(rtl)`, så begge
            // leser HTML-retningen fra `dir`. En side som setter `direction: rtl` i
            // CSS uten `dir` snus ikke, verken her eller i stilarket, som for
            // ikonene i bakgrunnen.
            const rtl = this.panel.matches(":dir(rtl)");
            const alignRight = this.placement.endsWith("end") !== rtl;
            let top = below ? anchor.bottom + space : anchor.top - panel.height - space;
            let left = alignRight ? anchor.right - panel.width : anchor.left;
            // Panelet skal ikke havne utenfor skjermen. Går det ut på siden, flyttes
            // det inn. Er det ikke plass på den siden av knappen plasseringen ber
            // om, legges det på den andre, og klemmes så inn i vinduet uansett.
            // `top-*` manglet den siste delen, og et panel ved toppen av siden lå
            // helt utenfor skjermen, uten å kunne rulles fram.
            left = Math.max(space, Math.min(left, window.innerWidth - panel.width - space));
            if (below && top + panel.height > window.innerHeight) {
                top = anchor.top - panel.height - space;
            }
            else if (!below && top < space) {
                top = anchor.bottom + space;
            }
            top = Math.max(space, Math.min(top, window.innerHeight - panel.height - space));
            this.panel.style.setProperty("--fs-popover-top", `${Math.round(top)}px`);
            this.panel.style.setProperty("--fs-popover-left", `${Math.round(left)}px`);
        };
    }
    /** Om panelet er åpent. Speiles, så CSS kan treffe tilstanden. */
    get open() {
        return this.hasAttribute("open");
    }
    set open(value) {
        // Beskjeden noteres her, og ikke i `show()` og `hide()`. De går begge
        // gjennom setteren, men det gjør også `meny.open = false` fra en app, og
        // uten dette satte komponenten attributtet rett tilbake igjen.
        //
        // Ingen hukommelse når serveren eier tilstanden. Uten den sperren spratt
        // vinduet opp av seg selv i det `server-controlled` ble tatt av igjen.
        this.wantsOpen = isServerControlled(this) ? false : value;
        setFlag(this, "open", value);
    }
    /** Hvilken kant panelet henger fra. Standard: `bottom-start`. */
    get placement() {
        const value = this.getAttribute("placement");
        return isPopoverPlacement(value) ? value : "bottom-start";
    }
    set placement(value) {
        setAttr(this, "placement", value);
    }
    connectedCallback() {
        window.addEventListener("resize", this.reposition);
        window.addEventListener("scroll", this.reposition, true);
        /*
         * Attributtene er med, ikke bare barna. `aria-expanded` på knappen og
         * plasseringen på panelet er noe komponenten regner ut, så river en patch
         * dem bort, skal de tilbake. Hver skriving i `sync()` sammenligner først,
         * ellers ville observatøren utløst seg selv.
         */
        this.observer = new MutationObserver(() => this.sync());
        this.observer.observe(this, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: [
                "aria-expanded",
                "style",
                // Koblingen komponenten fyller inn når markupen kom uten den.
                "popover",
                "id",
                "aria-controls",
                "class",
            ],
        });
        this.sync();
    }
    disconnectedCallback() {
        window.removeEventListener("resize", this.reposition);
        window.removeEventListener("scroll", this.reposition, true);
        document.removeEventListener("click", this.handleOutsideClick, true);
        document.removeEventListener("keydown", this.handleKeydown);
        this.observer?.disconnect();
        this.observer = undefined;
        this.triggerElement?.removeEventListener("click", this.handleTriggerClick);
        this.panel?.removeEventListener("toggle", this.handlePanelToggle);
        // Referansen må nullstilles, ellers ser `sync()` at knappen er den samme
        // når elementet settes inn igjen, og hopper over å feste lytteren på
        // nytt. Da lar panelet seg ikke åpne lenger.
        this.triggerElement = undefined;
        this.panel = undefined;
    }
    attributeChangedCallback(name) {
        // `server-controlled` slått på midt i: komponenten slipper taket, og neste
        // patch bestemmer.
        if (name === SERVER_CONTROLLED && isServerControlled(this)) {
            this.wantsOpen = false;
        }
        if (this.isConnected)
            this.sync();
    }
    sync() {
        /*
         * Setter `open` tilbake når en patch tok det.
         *
         * Bare én vei: står `open` der, er vinduet åpent, og da har enten
         * brukeren eller serveren sagt det. Er det borte mens brukeren åpnet det,
         * er det morfingen som tok det, og da kommer det tilbake.
         */
        if (this.wantsOpen && !this.open && !isServerControlled(this)) {
            /*
             * Vent til hele patchen har landet før vinduet åpnes igjen.
             *
             * En morfing setter ett attributt om gangen, og `open` kommer før
             * `server-controlled` i dokumentrekkefølgen. Reparerte komponenten med
             * en gang, satte den `open` tilbake mens serveren var midt i å si at
             * den overtar tilstanden, og vinduet ble stående åpent etterpå.
             * `queueMicrotask` kjører etter at hele patchen er ferdig, og vilkåret
             * sjekkes på nytt der.
             */
            queueMicrotask(() => {
                if (!this.isConnected || isServerControlled(this))
                    return;
                if (this.wantsOpen && !this.open)
                    setFlag(this, "open", true);
            });
        }
        // Delene kjennes igjen på koblingen som må være der uansett: panelet er
        // det som har `popover`, og knappen er den som peker på panelet med
        // `aria-controls`. Før sto det `slot="trigger"` på knappen, et levn fra
        // den gangen komponenten hadde shadow DOM. Uten en skyggerot gjør `slot`
        // ingenting i HTML, så attributtet var en merkelapp som så ut som noe
        // annet enn det var.
        const panel = this.findPanel();
        if (panel) {
            // Bare det som mangler. En mal kan ha skrevet `popover` uten verdi, og
            // det er `auto`; da følger komponenten med når nettleseren lukker.
            addClass(panel, POPOVER_CLASS);
            if (!panel.hasAttribute("popover"))
                setAttr(panel, "popover", "manual");
            if (!panel.id) {
                this.panelId ??= uniqueId("fs-popover");
                setAttr(panel, "id", this.panelId);
            }
        }
        const trigger = panel ? this.findTrigger(panel) : null;
        if (panel && trigger && !trigger.hasAttribute("aria-controls")) {
            setAttr(trigger, "aria-controls", panel.id);
        }
        if (!trigger || !panel) {
            /*
             * To ulike feil, og hver sin beskjed, så utvikleren leter på riktig
             * sted. Et tomt element er et område serveren ikke har fylt ennå, og
             * det er ikke en feil i markupen.
             */
            const isEmpty = () => this.childElementCount === 0;
            if (!panel) {
                warnAboutMarkup(this, "fant ingen panel: et element med popover eller klassen " +
                    "fs-popover. Uten det kan ingenting åpnes eller plasseres.", () => !isEmpty() && this.findPanel() === null);
            }
            else {
                warnAboutMarkup(this, "fant ingen knapp: verken en med aria-controls som peker på " +
                    "panelet, eller en <button> utenfor panelet. Uten den vet " +
                    "komponenten ikke hva som åpner vinduet.", () => {
                    const found = this.findPanel();
                    return !isEmpty() && found !== null && !this.findTrigger(found);
                });
            }
            /*
             * Slipp taket i det vi hadde. River en patch panelet bort, holdt
             * komponenten ellers på en løsrevet node, og `reposition()` fortsatte
             * å regne ut plasseringen for noe som ikke står i siden.
             *
             * Lytterne på `document` må med. Uten dem ble de liggende i fangstfasen
             * så lenge markupen var ødelagt, og et klikk hvor som helst på siden ga
             * appen en `popover-toggle` den ikke hadde bedt om.
             */
            this.triggerElement?.removeEventListener("click", this.handleTriggerClick);
            this.triggerElement = undefined;
            this.panel?.removeEventListener("toggle", this.handlePanelToggle);
            this.panel = undefined;
            document.removeEventListener("click", this.handleOutsideClick, true);
            document.removeEventListener("keydown", this.handleKeydown);
            return;
        }
        if (this.triggerElement !== trigger) {
            this.triggerElement?.removeEventListener("click", this.handleTriggerClick);
            trigger.addEventListener("click", this.handleTriggerClick);
            this.triggerElement = trigger;
        }
        if (this.panel !== panel) {
            /*
             * Tilstanden står i markupen som `:popover-open`, og skal leses derfra.
             * Lukker nettleseren et `popover="auto"` selv, fordi et annet åpnes
             * eller brukeren klikker utenfor, sier `toggle` fra. Uten lytteren sto
             * verten med `open` og knappen med `aria-expanded="true"` over et
             * lukket panel, og neste klikk gjorde ingenting synlig.
             */
            this.panel?.removeEventListener("toggle", this.handlePanelToggle);
            panel.addEventListener("toggle", this.handlePanelToggle);
            this.panel = panel;
        }
        // Komponentens eget, og satt på nytt hvis en patch tok det.
        setAttr(trigger, "aria-expanded", String(this.open));
        if (this.open) {
            if (!panel.matches(":popover-open"))
                panel.showPopover();
            this.reposition();
            document.addEventListener("click", this.handleOutsideClick, true);
            document.addEventListener("keydown", this.handleKeydown);
        }
        else {
            if (panel.matches(":popover-open"))
                panel.hidePopover();
            document.removeEventListener("click", this.handleOutsideClick, true);
            document.removeEventListener("keydown", this.handleKeydown);
        }
    }
    /** Panelet: det som har `popover`, ellers det med klassen. */
    findPanel() {
        return this.querySelector(`[popover], .${POPOVER_CLASS}`);
    }
    /**
     * Knappen: den som peker på panelet med `aria-controls`, ellers den første
     * knappen utenfor panelet som ikke peker på noe annet. En knapp med en
     * `aria-controls` som peker et annet sted er ikke en reserve: da har
     * serveren sagt noe annet, og det skal ikke gjettes rundt.
     */
    findTrigger(panel) {
        if (panel.id) {
            const pointing = this.querySelector(`[aria-controls="${CSS.escape(panel.id)}"]`);
            if (pointing)
                return pointing;
        }
        return ([...this.querySelectorAll("button, [role='button']")].find((button) => !panel.contains(button) && !button.hasAttribute("aria-controls")) ?? null);
    }
    emit(open) {
        this.dispatchEvent(new CustomEvent("popover-toggle", {
            detail: { open },
            bubbles: true,
            composed: true,
        }));
    }
    /** Åpner panelet. */
    show() {
        if (this.open)
            return;
        this.open = true;
        this.emit(true);
    }
    /** Lukker panelet. */
    hide() {
        if (!this.open)
            return;
        this.open = false;
        this.emit(false);
    }
    /** Åpner eller lukker. */
    toggle() {
        if (this.open)
            this.hide();
        else
            this.show();
    }
}
FsPopover.observedAttributes = ["open", "placement", SERVER_CONTROLLED];
export function defineFsPopover(tagName = FS_POPOVER_TAG) {
    defineElement(tagName, FsPopover);
}
//# sourceMappingURL=fs-popover.js.map