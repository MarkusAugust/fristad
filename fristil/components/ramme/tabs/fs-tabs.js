import { defineElement, derivedParts, HostElement, isServerControlled, SERVER_CONTROLLED, setAttr, setFlag, uniqueId, warnAboutMarkup, } from "../../host-element.js";
export const FS_TABS_TAG = "fs-tabs";
/** Raden: den serveren har gitt rollen, eller klassen fra `tabs.css`. */
const LIST_SELECTOR = "[role='tablist'], .fs-tabs__list";
/** En fane: rollen når den står der, ellers en knapp uten rolle i raden. */
const TAB_SELECTOR = "[role='tab'], .fs-tabs__list > button:not([role])";
/** Et panel: rollen når den står der, ellers klassen uten rolle. */
const PANEL_SELECTOR = "[role='tabpanel'], .fs-tabs__panel:not([role])";
/**
 * Tastaturet i en fanerad, og koblingen når markupen kommer uten.
 *
 * Lages markupen med JavaScript, skriver `fs.tabs()` rollene, id-ene,
 * `aria-controls` og `hidden`, og komponenten lar alt det stå. Blir markupen
 * til uten JavaScript, i en Go- eller Kotlin-mal, holder det med strukturen:
 * en `.fs-tabs__list` med knapper og ett `.fs-tabs__panel` per knapp.
 * Komponenten setter da rollene, lager id-ene og kobler hver fane til sitt
 * panel. En Kotlin-mal i spilldemoen skrev sju attributter per fane for hånd
 * før dette, og ingen kompilator så på dem.
 *
 * Det serveren har skrevet står. Komponenten fyller bare inn det som
 * mangler, og setter det tilbake når en patch river det bort, med de samme
 * id-ene som sist. Hvilken fane som er valgt leses fra `aria-selected`, og
 * mangler den, fra hvilket panel som ikke er `hidden`; er ingenting sagt,
 * er det den første. Før komponenten er registrert viser `tabs.css` bare
 * det første synlige panelet, så innholdet ikke hopper når resten skjules.
 *
 * Det komponenten gjør etterpå er det nettleseren ikke gjør selv: Tab hopper
 * forbi hele raden og inn i panelet, mens piltastene flytter mellom fanene.
 * Et sett knapper uten dette gir én tabbestopp per fane, og skjermleseren
 * sier «knapp» der den skulle sagt «fane, 2 av 3, valgt».
 *
 * Fanevalget er brukerens, og komponenten setter det tilbake når en patch
 * river det bort. Malen trenger derfor ingen `data-preserve-attr`. Skal
 * serveren kunne flytte fanen, som i «gå videre til steg 2», settes
 * `server-controlled` på verten, og da bestemmer hver patch.
 *
 * ```html
 * <fs-tabs>
 *   <div class="fs-tabs__list" aria-label="Deler av saken">
 *     <button>Søknaden</button>
 *     <button>Vedlegg</button>
 *   </div>
 *   <div class="fs-tabs__panel">…</div>
 *   <div class="fs-tabs__panel">…</div>
 * </fs-tabs>
 * ```
 */
export class FsTabs extends HostElement {
    constructor() {
        super(...arguments);
        this.bound = new Set();
        /**
         * Id-ene komponenten selv har laget, per node. En morfing river dem bort
         * og beholder noden, og da skal den samme id-en tilbake: en skjermleser
         * midt i en opplesning skal ikke følge en peker som skifter under den.
         */
        this.ids = new WeakMap();
        /**
         * Roller komponenten selv har satt. Skiller en bar rad, der hver knapp er
         * en fane, fra en rad serveren har skrevet rollene i, der en knapp uten
         * rolle er noe annet, som en «lukk»-knapp.
         */
        this.roledByMe = new WeakSet();
        this.selectedMemory = new WeakMap();
        this.hiddenMemory = new WeakMap();
        /**
         * Id-en komponenten selv skrev i `aria-controls` på en fane.
         *
         * `aria-controls` og panelets `id` er den samme opplysningen. Byttet en
         * patch ut panelet med en ny node, sto fanens halvdel igjen og pekte på en
         * id som ikke fantes: det nye panelet fikk verken id, rolle eller `hidden`.
         * Er pekeren komponentens egen, hører panelet på samme plass til fanen, og
         * får den samme id-en tilbake.
         */
        this.controlsByMe = new WeakMap();
        this.handleClick = (event) => {
            const index = this.tabs.indexOf(event.currentTarget);
            if (index >= 0)
                this.select(index);
        };
        this.handleKeydown = (event) => {
            const tabs = this.tabs;
            const current = tabs.indexOf(event.currentTarget);
            if (current < 0)
                return;
            // Alt+venstrepil er «tilbake» i nettleseren, og Cmd+pil flytter i
            // historikken på macOS. Med en modifikator er tasten ikke fanenes.
            if (event.altKey || event.ctrlKey || event.metaKey)
                return;
            // I en side som leses fra høyre står neste fane til venstre. Uten dette
            // flyttet høyrepil fokus visuelt bakover.
            const rtl = getComputedStyle(this).direction === "rtl";
            const steps = {
                ArrowRight: rtl ? -1 : 1,
                ArrowLeft: rtl ? 1 : -1,
                ArrowDown: 1,
                ArrowUp: -1,
            };
            let next;
            if (event.key in steps) {
                next = this.nextEnabled(current, steps[event.key]);
            }
            else if (event.key === "Home") {
                next = this.nextEnabled(-1, 1);
            }
            else if (event.key === "End") {
                next = this.nextEnabled(tabs.length, -1);
            }
            if (next === undefined)
                return;
            event.preventDefault();
            this.select(next);
            tabs[next].focus();
        };
    }
    connectedCallback() {
        /*
         * Attributtene er med, ikke bare barna. En morfing river bort det som
         * ikke står i serverens HTML, og fanevalget er nettopp det: noe brukeren
         * gjorde etter at siden kom. Hver skriving sammenligner først, ellers
         * ville observatøren utløst seg selv.
         */
        this.observer = new MutationObserver(() => this.sync());
        this.observer.observe(this, {
            childList: true,
            subtree: true,
            attributes: true,
            // `disabled` er med fordi komponenten leser det: deaktiverer en patch
            // fanen brukeren valgte, skal tabbestoppet flyttes til en som kan få
            // fokus. Har brukeren ikke valgt noe, er markupen serverens, og
            // komponenten rører den ikke.
            attributeFilter: [
                "aria-selected",
                "tabindex",
                "hidden",
                "disabled",
                "aria-disabled",
                // Koblingen komponenten setter når markupen kom uten den, og setter
                // tilbake når en patch tar den.
                "role",
                "id",
                "aria-controls",
                "aria-labelledby",
                "type",
            ],
        });
        this.sync();
    }
    attributeChangedCallback(name) {
        // `server-controlled` slått på midt i: da skal komponenten slippe taket,
        // og neste patch bestemmer.
        if (name === SERVER_CONTROLLED && isServerControlled(this)) {
            this.chosenTab = undefined;
            this.pendingTab = undefined;
            // Det som står der nå er utgangspunktet. Uten dette hoppet raden
            // tilbake til fanen serveren sa sist, før brukeren valgte en annen.
            this.selectedMemory = new WeakMap();
            this.hiddenMemory = new WeakMap();
        }
    }
    disconnectedCallback() {
        this.observer?.disconnect();
        this.observer = undefined;
        for (const tab of this.bound) {
            tab.removeEventListener("click", this.handleClick);
            tab.removeEventListener("keydown", this.handleKeydown);
        }
        this.bound.clear();
    }
    /**
     * Fanene som hører til denne raden, og bare dem.
     *
     * Faner i faner er vanlig markup. Uten avgrensningen ble en indre rad en
     * del av den ytre: et klikk på en indre fane skjulte det ytre panelet den
     * sto i, og begge komponentene festet lyttere på de samme knappene.
     */
    get tabs() {
        return derivedParts(this, TAB_SELECTOR, "tab", this.roledByMe);
    }
    /** Panelene som står inni denne raden. Reserven når koblingen mangler. */
    get ownPanels() {
        return derivedParts(this, PANEL_SELECTOR, "tabpanel", this.roledByMe);
    }
    /** Serverens ord om et attributt, mot hva komponenten selv skrev sist. */
    serverWord(memory, element, now) {
        const known = memory.get(element);
        if (!known)
            return now;
        return now !== known.written ? now : known.server;
    }
    /** Står det noe annet på en fane eller et panel enn komponenten skrev? */
    touchedByOthers() {
        return this.tabs.some((tab, index) => {
            const selected = this.selectedMemory.get(tab);
            if (!selected || tab.getAttribute("aria-selected") !== selected.written) {
                return true;
            }
            const panel = this.panelFor(tab, index);
            if (!panel)
                return false;
            const hidden = this.hiddenMemory.get(panel);
            const now = panel.hasAttribute("hidden") ? "" : null;
            return !hidden || now !== hidden.written;
        });
    }
    /**
     * Slipper klikket, og gjør det som står i markupen nå til serverens ord.
     *
     * Minnet har serverens ord fra før klikket, og de er foreldet: serveren
     * har svart. Et attributt serveren skrev likt med komponenten, kan ikke
     * skilles fra et den ikke rørte, så de gamle ordene vant, og raden hoppet
     * tilbake til fanen fra før klikket idet serveren bekreftet den nye.
     *
     * `hidden` på panelene leses derfor som det står. `aria-selected` leses
     * som det står når serveren har rørt minst én fane, altså når serveren
     * snakker i `aria-selected`. Har den ikke det, er verdiene komponentens
     * egne fra klikket, og da sier panelene hvilken fane som gjelder.
     */
    releasePending() {
        this.pendingTab = undefined;
        const tabs = this.tabs;
        const serverSpoke = tabs.some((tab) => {
            const now = tab.getAttribute("aria-selected");
            const known = this.selectedMemory.get(tab);
            // En ny, bar knapp har ikke sagt noe. Telte den, ble klikkets
            // `aria-selected` lest som serverens, og en senere flytting med bare
            // `hidden` ble satt tilbake.
            return known ? now !== known.written : now !== null;
        });
        this.hiddenMemory = new WeakMap();
        if (serverSpoke) {
            this.selectedMemory = new WeakMap();
            return;
        }
        for (const tab of tabs) {
            this.selectedMemory.set(tab, {
                written: tab.getAttribute("aria-selected"),
                server: null,
            });
        }
    }
    serverSelected(tab) {
        return this.serverWord(this.selectedMemory, tab, tab.getAttribute("aria-selected"));
    }
    serverHidden(panel) {
        return (this.serverWord(this.hiddenMemory, panel, panel.hasAttribute("hidden") ? "" : null) !== null);
    }
    /** Raden knappene står i, når den er denne komponentens. */
    get list() {
        const list = this.querySelector(LIST_SELECTOR);
        return list?.closest(this.localName) === this ? list : null;
    }
    /** Id-en et element skal ha: sin egen, ellers den komponenten laget sist. */
    rememberedId(element, prefix) {
        if (element.id)
            return element.id;
        let id = this.ids.get(element);
        if (!id) {
            id = uniqueId(prefix);
            this.ids.set(element, id);
        }
        return id;
    }
    /**
     * Panelet en fane styrer.
     *
     * Koblingen står i markupen som `aria-controls`, og leses derfra. Da kan
     * panelene stå i en annen rekkefølge enn fanene, og utenfor verten. Et
     * panel utenfor ligger likevel ikke i det komponenten observerer, så river
     * en patch `hidden` av det, kommer det ikke tilbake av seg selv.
     *
     * Rekkefølgen er reserve for markup **uten** `aria-controls`. Står
     * attributtet der og peker på ingenting, er svaret ingenting, og
     * komponenten sier fra. Falt den tilbake på rekkefølgen også da, kunne to
     * faner få det samme panelet, og de to skrev motsatt `hidden` på det i
     * hver eneste runde: observatøren kalte seg selv, og siden frøs.
     *
     * Ett unntak: pekeren er komponentens egen, og panelet på samme plass er
     * ledig, altså uten at noen annen fane peker på det. Da er panelet byttet
     * ut med en ny node. Er det fjernet, har de andre panelene rykket fram,
     * og plassen er en annen fanes: da er svaret fortsatt ingenting.
     */
    panelFor(tab, index) {
        const id = tab.getAttribute("aria-controls");
        if (id === null)
            return this.ownPanels[index] ?? null;
        const root = this.getRootNode();
        const panel = root.getElementById?.(id) ?? null;
        // Peker komponentens egen `aria-controls` på ingenting, er panelet
        // byttet ut. Da gjelder rekkefølgen, som da koblingen ble laget.
        if (!panel && this.controlsByMe.get(tab) === id) {
            const candidate = this.ownPanels[index] ?? null;
            const taken = candidate !== null &&
                candidate.id !== "" &&
                this.tabs.some((other) => other !== tab &&
                    other.getAttribute("aria-controls") === candidate.id);
            return taken ? null : candidate;
        }
        return panel;
    }
    /** En fane som ikke kan velges, verken med mus eller tastatur. */
    isDisabled(tab) {
        return (tab.hasAttribute("disabled") ||
            tab.getAttribute("aria-disabled") === "true");
    }
    /**
     * Indeksen på fanen som er markert, eller `-1` når ingen er det.
     *
     * `selected` svarer 0 i det tilfellet, fordi den er offentlig API og en
     * fanerad i praksis alltid har en valgt fane. Men den løgnen kan ikke
     * brukes internt: sa den 0 når ingenting var markert, returnerte
     * `select(0)` med en gang, og raden svarte ikke på et klikk i det hele
     * tatt.
     */
    get markedIndex() {
        return this.tabs.findIndex((tab) => tab.getAttribute("aria-selected") === "true");
    }
    /** Indeksen på fanen som er valgt. */
    get selected() {
        const index = this.markedIndex;
        return index < 0 ? 0 : index;
    }
    set selected(index) {
        // React 19 skriver egenskapen når den finnes, og dokumentasjonen viser
        // `faner.selected` som API. En getter alene kastet ved tilordning.
        this.select(index);
    }
    sync() {
        this.wire();
        this.bind();
        this.repair();
    }
    /**
     * Fyller inn koblingen der markupen kom uten den.
     *
     * Bare det som mangler. Et attributt serveren har skrevet er serverens
     * ord, også når det sier noe annet enn komponenten ville sagt: en
     * `tabindex="-1"` på et panel eller en egen id på en fane står. Panelet
     * finnes gjennom `aria-controls` når den står der, ellers etter
     * rekkefølgen, og koblingen skrives så inn i markupen, slik at den også
     * står der for hjelpemidlene.
     */
    wire() {
        const list = this.list;
        if (list && !list.hasAttribute("role"))
            setAttr(list, "role", "tablist");
        this.tabs.forEach((tab, index) => {
            if (!tab.hasAttribute("role")) {
                setAttr(tab, "role", "tab");
                this.roledByMe.add(tab);
            }
            // En knapp uten `type` sender skjemaet den står i. Bare på knapper:
            // en fane kan være noe annet, og da betyr `type` noe annet.
            if (tab.localName === "button" && !tab.hasAttribute("type")) {
                setAttr(tab, "type", "button");
            }
            if (!tab.id)
                setAttr(tab, "id", this.rememberedId(tab, "fs-tabs-tab"));
            const panel = this.panelFor(tab, index);
            if (!panel)
                return;
            if (!panel.hasAttribute("role")) {
                setAttr(panel, "role", "tabpanel");
                this.roledByMe.add(panel);
            }
            const mine = this.controlsByMe.get(tab);
            const stale = mine !== undefined && tab.getAttribute("aria-controls") === mine;
            if (!panel.id) {
                // Det nye panelet får id-en fanen alt peker på, når pekeren er
                // komponentens egen. Da skifter den ikke under en skjermleser.
                // Husket på noden, så en senere morfing som river begge halvdelene
                // også gir den samme tilbake.
                if (stale)
                    this.ids.set(panel, mine);
                setAttr(panel, "id", this.rememberedId(panel, "fs-tabs-panel"));
            }
            else if (stale && panel.id !== mine) {
                // Serveren ga det nye panelet en id. Da er det den som gjelder.
                setAttr(tab, "aria-controls", panel.id);
                this.controlsByMe.set(tab, panel.id);
            }
            // Panelet får fokus når det ikke har noe å fokusere på selv, ellers
            // hopper Tab rett forbi innholdet som nettopp ble vist.
            if (!panel.hasAttribute("tabindex"))
                setAttr(panel, "tabindex", "0");
            if (!tab.hasAttribute("aria-controls")) {
                setAttr(tab, "aria-controls", panel.id);
                this.controlsByMe.set(tab, panel.id);
            }
            if (!panel.hasAttribute("aria-labelledby")) {
                setAttr(panel, "aria-labelledby", tab.id);
            }
        });
    }
    /**
     * Fanen serveren sier er valgt, når brukeren ikke har valgt noe.
     *
     * Serverens ord, ikke det som står der: det komponenten selv skrev
     * teller ikke, se `Word`. `aria-selected` først. Mangler den, som i markup
     * skrevet uten JavaScript, sier `hidden` på panelene det samme: det
     * første panelet serveren viser hører til den valgte fanen. Sier markupen
     * ingenting, eller skjuler den alle panelene, er det den første, som er
     * det en fanerad uansett starter på.
     */
    get derivedIndex() {
        const tabs = this.tabs;
        const marked = tabs.findIndex((tab) => this.serverSelected(tab) === "true");
        if (marked >= 0)
            return marked;
        const shown = tabs.findIndex((tab, index) => {
            const panel = this.panelFor(tab, index);
            return panel !== null && !this.serverHidden(panel);
        });
        return shown < 0 ? 0 : shown;
    }
    /**
     * Setter brukerens valg tilbake etter en patch.
     *
     * Uten dette måtte malen skrive `data-preserve-attr="aria-selected tabindex"`
     * på hver fane og `hidden` på hvert panel, altså liste opp attributtene
     * komponenten kom til å røre. Ingen kompilator så på den lista.
     *
     * `apply()` kalles uansett om fanene ser riktige ut, for en patch kan ha
     * rørt bare panelene. Det er den smale patchen dokumentasjonen selv
     * anbefaler, og en sjekk på `aria-selected` alene så den ikke: fanen sa én
     * ting og skjermen en annen.
     */
    repair() {
        if (this.pendingTab) {
            const index = this.tabs.indexOf(this.pendingTab);
            if (isServerControlled(this) && index >= 0 && !this.touchedByOthers()) {
                this.apply(index);
                return;
            }
            this.releasePending();
        }
        if (!this.chosenTab || isServerControlled(this)) {
            /*
             * Ingen brukervalg å sette tilbake, så serverens ord bestemmer, og det
             * skrives helt ut: `aria-selected`, tabbestoppet og `hidden` i takt.
             * Bar struktur uten `aria-selected` får dermed den første fanen valgt
             * og resten av panelene skjult, og en patch som bare rørte panelene
             * flytter også markeringen. Det gjelder også under
             * `server-controlled`: komponenten setter ikke brukerens valg tilbake
             * der, men skriver fortsatt ut det serverens markup sier. Stemmer alt
             * fra før, skriver `apply()` ingenting.
             */
            if (this.tabs.length > 0)
                this.apply(this.derivedIndex);
            return;
        }
        const index = this.tabs.indexOf(this.chosenTab);
        if (index < 0) {
            /*
             * Fanen finnes ikke lenger. Serveren har sendt noe annet, og da er det
             * serverens markup som gjelder.
             *
             * Med ett unntak, og det er verdt å vite om: markupen må henge sammen
             * etterpå. Fjernet patchen fanen som var markert, står raden igjen uten
             * en eneste `aria-selected="true"`, alle panelene er skjult, og et
             * klikk gjør ingenting. Da velger komponenten den første, som er det en
             * fanerad uansett starter på. Sendte serveren med vilje en rad uten
             * markering, blir den altså overkjørt, men bare når brukeren hadde
             * valgt noe fra før.
             *
             * Valget meldes, for dette er komponentens eget og ikke brukerens.
             * Uten hendelsen ville en app som laster innholdet i panelet eller
             * skriver valget i adressen aldri fått vite at det flyttet seg.
             */
            this.chosenTab = undefined;
            if (this.markedIndex < 0 && this.tabs.length > 0) {
                this.apply(0);
                this.notify(0);
            }
            return;
        }
        this.apply(index);
    }
    bind() {
        const tabs = this.tabs;
        if (tabs.length === 0) {
            warnAboutMarkup(this, "fant ingen faner. Raden trenger en .fs-tabs__list med <button>-er, " +
                'eller knapper med role="tab". Uten dem sier skjermleseren «knapp» ' +
                "der den skulle sagt «fane, 2 av 3, valgt», og piltastene gjør " +
                "ingenting.", 
            // En tom `<fs-tabs>` er et område serveren ikke har fylt ennå, og
            // det er ikke en feil i markupen.
            () => this.childElementCount > 0 && this.tabs.length === 0);
            return;
        }
        // Uten tallene i meldingen. Interpolerer den et tall, er hver runde en
        // ny melding, og advarselen kommer på nytt for hvert panel som dukker
        // opp framfor én gang.
        warnAboutMarkup(this, "har faner uten et panel: enten mangler et element med " +
            'role="tabpanel", eller aria-controls peker på en id som ikke ' +
            "finnes. Fanene uten et panel kan velges uten at noe vises.", () => this.tabs.some((tab, i) => this.panelFor(tab, i) === null));
        for (const tab of tabs) {
            if (this.bound.has(tab))
                continue;
            tab.addEventListener("click", this.handleClick);
            tab.addEventListener("keydown", this.handleKeydown);
            this.bound.add(tab);
        }
    }
    /**
     * Neste fane som kan velges, i en retning, rundt om nødvendig.
     *
     * En deaktivert fane hoppes over. Ble den valgt, gjorde `focus()` på en
     * deaktivert knapp ingenting, og raden sto uten en eneste fane som kunne
     * få fokus. Tastaturbrukeren var låst ute av raden.
     */
    nextEnabled(from, step) {
        const tabs = this.tabs;
        for (let i = 1; i <= tabs.length; i++) {
            const index = (from + step * i + tabs.length * i) % tabs.length;
            if (!this.isDisabled(tabs[index]))
                return index;
        }
        return undefined;
    }
    /** Velger en fane og melder fra. */
    select(index) {
        const tabs = this.tabs;
        if (index < 0 || index >= tabs.length)
            return;
        if (this.isDisabled(tabs[index]))
            return;
        // `markedIndex` og ikke `selected`: den siste svarer 0 også når ingenting
        // er markert, og da lot den første fanen seg aldri velge.
        if (index === this.markedIndex)
            return;
        // Ingen hukommelse når serveren eier valget. Uten dette ville et valg
        // gjort mens attributtet sto der blitt satt tilbake i det noen fjernet
        // det igjen.
        if (!isServerControlled(this))
            this.chosenTab = tabs[index];
        this.apply(index);
        if (isServerControlled(this))
            this.pendingTab = tabs[index];
        this.notify(index);
    }
    /** Sier fra om hvilken fane som er valgt nå. */
    notify(index) {
        this.dispatchEvent(new CustomEvent("tab-select", {
            detail: { index },
            bubbles: true,
            composed: true,
        }));
    }
    /**
     * Skriver valget ut i markupen.
     *
     * Hver skriving sammenligner først. Komponenten observerer nå de samme
     * attributtene den setter, så en skriving uten sammenligning ville utløst
     * observatøren, som ville skrevet på nytt, i det uendelige.
     */
    apply(index) {
        const tabs = this.tabs;
        /*
         * Tabbestoppet er den valgte fanen, med mindre en patch har deaktivert
         * den. Da får den neste fanen som kan velges det, ellers hopper Tab
         * forbi hele raden, og `keydown` fyrer ikke på en deaktivert knapp, så
         * piltastene hjelper heller ikke.
         */
        const stop = this.isDisabled(tabs[index])
            ? (this.nextEnabled(index, 1) ?? index)
            : index;
        /*
         * Et panel er synlig når noen fane som peker på det er valgt, og skrives
         * én gang. Peker to faner på det samme, ved en feil i markupen, ville de
         * ellers skrevet motsatt `hidden` på det i hver runde, og siden frosset.
         */
        const panels = new Map();
        tabs.forEach((tab, i) => {
            const chosen = i === index;
            // Serverens ord noteres før skrivingen, ellers ble skrivingen lest
            // som serverens i neste runde.
            const server = this.serverSelected(tab);
            setAttr(tab, "aria-selected", String(chosen));
            this.selectedMemory.set(tab, { written: String(chosen), server });
            setAttr(tab, "tabindex", i === stop ? "0" : "-1");
            const panel = this.panelFor(tab, i);
            if (panel)
                panels.set(panel, (panels.get(panel) ?? false) || chosen);
        });
        for (const [panel, shown] of panels) {
            const server = this.serverHidden(panel) ? "" : null;
            setFlag(panel, "hidden", !shown);
            this.hiddenMemory.set(panel, { written: shown ? null : "", server });
        }
    }
}
/**
 * Bare `server-controlled`. Hvilken fane som er valgt står i markupen
 * serveren sendte, som `aria-selected` på fanen og `hidden` på panelene, og
 * leses derfra. Et eget `selected` ville vært en parallell utgave av det
 * samme.
 */
FsTabs.observedAttributes = [SERVER_CONTROLLED];
export function defineFsTabs(tagName = FS_TABS_TAG) {
    defineElement(tagName, FsTabs);
}
//# sourceMappingURL=fs-tabs.js.map