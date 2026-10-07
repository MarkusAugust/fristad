import { LABEL_CLASS } from "../../css/label/label.js";
import { DEFAULT_TEXTS } from "../../default-texts.js";
import { addClass, defineElement, derivedParts, HostElement, isServerControlled, languageOf, SERVER_CONTROLLED, setAttr, setFlag, setText, uniqueId, warnAboutMarkup, } from "../../host-element.js";
import { SUGGESTION_EMPTY_CLASS, SUGGESTION_LIST_CLASS, SUGGESTION_OPTION_CLASS, } from "./suggestion.js";
/**
 * Tekstene for antall treff, ett attributt per form. `count-none` er null
 * treff, og resten er formene `Intl.PluralRules` kjenner.
 */
const COUNT_ATTRIBUTES = [
    "count-none",
    "count-zero",
    "count-one",
    "count-two",
    "count-few",
    "count-many",
    "count-other",
];
export const FS_SUGGESTION_TAG = "fs-suggestion";
/** Feltet: rollen når den står der, ellers det første inndatafeltet. */
const CONTROL_SELECTOR = "[role='combobox'], input:not([type='hidden'])";
/** Lista: rollen når den står der, ellers klassen. */
const LIST_SELECTOR = `[role='listbox'], .${SUGGESTION_LIST_CLASS}`;
/** Et alternativ: rollen når den står der, ellers en `<li>` uten rolle i lista. */
const OPTION_SELECTOR = `[role='option'], .${SUGGESTION_LIST_CLASS} > li:not([role])`;
/**
 * Tastaturet og filtreringen i et felt med forslagsliste.
 *
 * Serveren skriver feltet og lista. Lages markupen med JavaScript, skriver
 * `fs.suggestion()` rollene, id-ene og koblingen. Kommer den fra en mal uten
 * JavaScript, holder det med en `<label>`, et `<input>`, en
 * `.fs-suggestion__list` med `<li>` og et `[role="status"]`: komponenten
 * setter rollene, lager id-ene, kobler ledeteksten og lista til feltet, og
 * lukker lista til brukeren rører feltet. Bare det som mangler, så det
 * serveren skrev står. Komponenten rendrer ingenting. Den lagde tidligere
 * hele feltet selv, og da fantes det verken ledetekst eller inndatafelt før
 * skriptet hadde kjørt, og ingenting ble med i innsendingen.
 *
 * Det komponenten gjør er det nettleseren ikke gjør: filtrerer alternativene
 * mens brukeren skriver, flytter markeringen med piltastene, og holder
 * `aria-activedescendant` i synk slik at skjermleseren leser opp alternativet
 * uten at fokus forlater feltet.
 *
 * Har noen andre alt filtrert, en server eller en React-komponent som
 * rendrer bare treffene, sier `prefiltered` fra, og komponenten lar
 * alternativene være i fred.
 *
 * ```html
 * <fs-suggestion>
 *   <label class="fs-label" for="kommune">Kommune</label>
 *   <div class="fs-suggestion__field">
 *     <input class="fs-input" id="kommune" role="combobox" aria-controls="kommune-list"
 *            aria-expanded="false" aria-autocomplete="list" autocomplete="off"
 *            aria-describedby="kommune-status">
 *     <ul class="fs-suggestion__list" id="kommune-list" role="listbox" hidden>
 *       <li class="fs-suggestion__option" id="kommune-option-0" role="option">Bergen</li>
 *     </ul>
 *     <p class="fs-suggestion__empty" hidden>Ingen treff</p>
 *     <span class="fs-sr-only" id="kommune-status" role="status" aria-live="polite"
 *           data-ignore-morph></span>
 *   </div>
 * </fs-suggestion>
 * ```
 */
export class FsSuggestion extends HostElement {
    constructor() {
        super(...arguments);
        /** Alternativene som alt har fått lytteren sin. */
        this.bound = new WeakSet();
        /** Id-ene komponenten selv har laget, per node, så en patch får de samme tilbake. */
        this.ids = new WeakMap();
        /** Rollene komponenten selv har satt. Se `derivedParts`. */
        this.roledByMe = new WeakSet();
        /** Sant mens komponenten selv sender hendelser, så den ikke svarer seg selv. */
        this.choosing = false;
        this.handleInput = () => {
            if (this.choosing)
                return;
            this.filter();
            this.setOpen(true);
        };
        this.handleFocus = () => {
            this.filter();
            this.setOpen(true);
        };
        this.handleFocusOut = (event) => {
            // Går fokus til noe inne i komponenten, som en knapp ved siden av
            // feltet, står lista.
            const next = event.relatedTarget;
            if (next instanceof Node && this.contains(next))
                return;
            this.setOpen(false);
        };
        this.handleKeydown = (event) => {
            const open = this.listElement?.hidden === false;
            if (event.key === "ArrowDown") {
                event.preventDefault();
                if (!open) {
                    this.filter();
                    this.setOpen(true);
                }
                this.markActive(this.activeIndex + 1);
            }
            else if (event.key === "ArrowUp") {
                event.preventDefault();
                // Lista må åpnes først, som med pil ned. Uten det pekte
                // `aria-activedescendant` på et alternativ i en liste feltet samtidig
                // meldte som lukket, og skjermleseren leste opp noe som ikke sto på
                // skjermen.
                if (!open) {
                    this.filter();
                    this.setOpen(true);
                }
                // Er ingenting markert, går pil opp til det siste. `activeIndex - 1`
                // ville gitt det nest siste, siden ingenting markert er -1.
                this.markActive(this.activeIndex < 0 ? -1 : this.activeIndex - 1);
            }
            else if (event.key === "Enter" && open && this.activeIndex >= 0) {
                event.preventDefault();
                const chosen = this.visible[this.activeIndex];
                if (chosen)
                    this.choose(chosen);
            }
            else if (event.key === "Escape" && open) {
                event.preventDefault();
                this.setOpen(false);
            }
        };
        this.handleListMouseDown = (event) => {
            event.preventDefault();
        };
        this.handleOptionMouseDown = (event) => {
            // mousedown og ikke click: feltet mister ellers fokus før valget rekker
            // å skje, og lista lukker seg først.
            event.preventDefault();
            this.choose(event.currentTarget);
        };
    }
    connectedCallback() {
        /*
         * Attributtene er med, ikke bare barna. Hver skriving under sammenligner
         * først, ellers ville observatøren utløst seg selv i det uendelige.
         */
        this.observer = new MutationObserver(() => this.sync());
        this.observer.observe(this, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: [
                "hidden",
                "aria-expanded",
                "aria-selected",
                "aria-activedescendant",
                // Koblingen komponenten fyller inn når markupen kom uten den.
                "role",
                "id",
                "for",
                "aria-controls",
                "autocomplete",
                "aria-autocomplete",
                "class",
            ],
        });
        this.sync();
        // Lista lukkes når fokus forlater komponenten, med Tab som med alt annet.
        // På verten og ikke på feltet: går fokus via en knapp inni komponenten og
        // så videre ut, skal lista lukkes da også. Uten dette ble den stående
        // over neste felt, med `aria-expanded` sann på et felt uten fokus.
        this.addEventListener("focusout", this.handleFocusOut);
    }
    attributeChangedCallback(name) {
        if (name === SERVER_CONTROLLED && isServerControlled(this)) {
            this.wantOpen = undefined;
            this.active = undefined;
        }
        /*
         * Attributtet kan komme og gå mens siden lever, og da skal lista rettes
         * opp med en gang framfor ved neste tastetrykk. `this.observer` er
         * beskjeden om at komponenten står i dokumentet: tilbakekallet kommer
         * også under oppgraderingen, før barna finnes.
         */
        if (name === "prefiltered" && this.observer) {
            // Antallet som står der i det attributtet settes er utgangspunktet, og
            // ikke noe å lese opp. Uten dette ble den første endringen etterpå
            // regnet som utgangspunktet, og gikk tapt.
            this.announcedCount = this.prefiltered ? this.visible.length : undefined;
            this.filter();
        }
    }
    sync() {
        this.wire();
        this.bind();
        this.repair();
        this.announceFiltered();
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
     * Fyller inn koblingen der markupen kom uten den. Bare det som mangler.
     *
     * Lista lukkes når markupen ikke sier noe om den, altså når verken
     * `aria-expanded` på feltet eller `hidden` på lista står der. Sier
     * markupen én av delene, er det serverens ord om begge, som med
     * `fs.suggestion({ open: true })`.
     */
    wire() {
        const control = this.querySelector(CONTROL_SELECTOR);
        if (!control)
            return;
        if (!control.hasAttribute("role"))
            setAttr(control, "role", "combobox");
        if (!control.hasAttribute("autocomplete")) {
            setAttr(control, "autocomplete", "off");
        }
        if (!control.hasAttribute("aria-autocomplete")) {
            setAttr(control, "aria-autocomplete", "list");
        }
        const label = this.querySelector("label");
        const root = this.getRootNode();
        /*
         * `for` og `id` er den samme opplysningen. Byttet en patch ut feltet,
         * kom det nye uten id mens ledeteksten beholdt sin `for`, og en ny,
         * tilfeldig id på feltet rev koblingen. Peker `for` på ingenting, er det
         * den id-en feltet skal ha.
         */
        if (!control.id) {
            const wanted = label?.getAttribute("for");
            const free = wanted && !root.getElementById?.(wanted);
            // Husket på noden, så en senere morfing som river både `id` og `for`
            // gir den samme tilbake.
            if (free)
                this.ids.set(control, wanted);
            setAttr(control, "id", this.rememberedId(control, "fs-suggestion"));
        }
        if (label) {
            addClass(label, LABEL_CLASS);
            if (!label.hasAttribute("for"))
                setAttr(label, "for", control.id);
        }
        const list = this.listElement;
        if (list) {
            addClass(list, SUGGESTION_LIST_CLASS);
            if (!list.hasAttribute("role"))
                setAttr(list, "role", "listbox");
            if (!list.id) {
                // Samme regel som for feltet: peker `aria-controls` på ingenting,
                // er det den id-en den nye lista skal ha.
                const wanted = control.getAttribute("aria-controls");
                const free = wanted && !root.getElementById?.(wanted);
                if (free)
                    this.ids.set(list, wanted);
                setAttr(list, "id", this.rememberedId(list, "fs-suggestion-list"));
            }
            if (!control.hasAttribute("aria-controls")) {
                setAttr(control, "aria-controls", list.id);
            }
            if (!control.hasAttribute("aria-expanded")) {
                if (!list.hasAttribute("hidden"))
                    setFlag(list, "hidden", true);
                setAttr(control, "aria-expanded", "false");
            }
        }
        for (const option of this.options) {
            addClass(option, SUGGESTION_OPTION_CLASS);
            if (!option.hasAttribute("role")) {
                setAttr(option, "role", "option");
                this.roledByMe.add(option);
            }
            if (!option.id) {
                setAttr(option, "id", this.rememberedId(option, "fs-suggestion-option"));
            }
        }
    }
    /**
     * Setter brukerens tilstand tilbake etter en patch.
     *
     * Filtreringen regnes ut på nytt av det som står i feltet, så den trenger
     * ingen hukommelse. Det lista viser og hva som er markert gjør det.
     */
    repair() {
        if (isServerControlled(this) || !this.control)
            return;
        if (this.wantOpen !== undefined) {
            /*
             * Begge delene sjekkes hver for seg. En patch som bare rører feltet lot
             * `aria-expanded="false"` stå mens alternativene var synlige, og
             * skjermleseren meldte at lista var lukket mens den sto på skjermen.
             */
            const list = this.listElement;
            const expanded = this.control.getAttribute("aria-expanded");
            if ((list && list.hidden === this.wantOpen) ||
                expanded !== String(this.wantOpen)) {
                this.applyOpen(this.wantOpen);
            }
            if (this.wantOpen)
                this.filter();
        }
        if (this.active) {
            /*
             * Teksten må stemme, ikke bare id-en, og alternativet må være synlig.
             * Ellers markerte komponenten noe serveren nettopp hadde filtrert bort,
             * og `aria-activedescendant` pekte på et skjult element.
             */
            const option = this.visible.find((o) => o.id === this.active?.id &&
                (o.textContent ?? "").trim() === this.active?.label);
            if (!option) {
                // Pekeren må bort sammen med markeringen. Uten dette pekte
                // `aria-activedescendant` på et alternativ som nå heter noe annet, og
                // skjermleseren leste opp en kommune brukeren aldri navigerte til.
                this.active = undefined;
                this.control.removeAttribute("aria-activedescendant");
            }
            else if (option.getAttribute("aria-selected") !== "true" ||
                this.control.getAttribute("aria-activedescendant") !== option.id) {
                // Begge sidene av koblingen sjekkes. Rev patchen bare
                // `aria-activedescendant`, mens markeringen sto igjen, mistet
                // skjermleseren lesepunktet sitt uten at noe annet så galt ut.
                this.markOption(option);
            }
        }
    }
    disconnectedCallback() {
        this.removeEventListener("focusout", this.handleFocusOut);
        this.observer?.disconnect();
        this.observer = undefined;
        this.unbind();
    }
    /**
     * Noen andre har alt filtrert, så komponenten skal la være.
     *
     * Navnet het `server-filtered` før, og det var misvisende: det handler ikke
     * om servere. En React-app som rendrer bare treffene har filtrert like
     * fullt, uten at noen server er involvert.
     *
     * Komponenten skjuler et alternativ når teksten ikke inneholder det som
     * står i feltet. Filtrerer du på noe annet, uten diakritikk, på en kode som
     * ikke vises eller uskarpt, blir de to uenige, og da er det ditt filter som
     * skal gjelde. Attributtet slår av både skjulingen og tommeldingen, altså
     * alt som handler om hva lista viser. Opplesningen av antall treff blir
     * igjen, siden den ikke er markup.
     */
    get prefiltered() {
        return this.hasAttribute("prefiltered");
    }
    set prefiltered(on) {
        /*
         * Setteren er ikke pynt. React 19 skriver egenskapen framfor attributtet
         * når en web component har en med det navnet, og en getter alene
         * kaster «Cannot set property prefiltered». Attributtet landet aldri, og
         * komponenten skjulte det React nettopp hadde rendret. Bindestreken i det
         * gamle navnet skjulte problemet: `server-filtered` kan ikke være et
         * egenskapsnavn, så React sendte det som attributt.
         */
        setFlag(this, "prefiltered", on);
    }
    get listElement() {
        return this.querySelector(LIST_SELECTOR);
    }
    get emptyElement() {
        return this.querySelector(`.${SUGGESTION_EMPTY_CLASS}`);
    }
    /**
     * Alternativene, kjent igjen på rollen og ikke på klassen.
     *
     * Rollen må stå der uansett, for skjermleseren. En mal som skriver
     * `<li role="option">` med egen styling fikk verken filtrering eller
     * piltaster da komponenten så etter klassen, og ingen sa fra.
     */
    get options() {
        return derivedParts(this, OPTION_SELECTOR, "option", this.roledByMe);
    }
    /**
     * Hvilket synlig alternativ som er markert nå.
     *
     * Den leses fra `aria-selected` i markupen, ikke fra en teller inni
     * komponenten. Serveren kan sette markeringen selv med
     * `fs.suggestion({ activeIndex })`, og en parallell teller ville hoppet
     * til toppen av lista ved første piltast i stedet for til alternativet
     * etter. Filtreringen flytter også på alternativene, og da svarer
     * oppslaget riktig av seg selv.
     */
    get activeIndex() {
        return this.visible.findIndex((option) => option.getAttribute("aria-selected") === "true");
    }
    /** Alternativene som er synlige nå. */
    get visible() {
        return this.options.filter((option) => !option.hidden);
    }
    unbind() {
        const control = this.control;
        if (control) {
            control.removeEventListener("input", this.handleInput);
            control.removeEventListener("keydown", this.handleKeydown);
            control.removeEventListener("focus", this.handleFocus);
            this.control = undefined;
        }
        this.list?.removeEventListener("mousedown", this.handleListMouseDown);
        this.list = undefined;
        this.empty?.removeEventListener("mousedown", this.handleListMouseDown);
        this.empty = undefined;
    }
    bind() {
        const control = this.querySelector(CONTROL_SELECTOR);
        if (!control) {
            warnAboutMarkup(this, 'fant ingen felt: verken et <input> eller en [role="combobox"]. ' +
                "Uten det vet komponenten ikke hvilket felt den skal lytte på, " +
                "og verken filtrering eller piltaster virker.", 
            // Et tomt element er et område serveren ikke har fylt ennå, og det
            // er ikke en feil i markupen.
            () => this.childElementCount > 0 &&
                this.querySelector(CONTROL_SELECTOR) === null);
            return;
        }
        warnAboutMarkup(this, 'fant ingen liste: verken en [role="listbox"] eller en ' +
            ".fs-suggestion__list. Alternativene kan da verken vises, " +
            "filtreres eller velges med tastaturet.", () => this.querySelector(CONTROL_SELECTOR) !== null && !this.listElement);
        // Antall treff leses opp i et statuselement serveren sender. Uten det
        // får den som ikke ser skjermen ingen beskjed om at lista snevret seg
        // inn, og før sto komponenten da bare stille.
        warnAboutMarkup(this, 'fant ingen [role="status"]. Antall treff leses da ikke opp for den ' +
            "som ikke ser skjermen. `fs.suggestion()` skriver elementet, med " +
            "data-ignore-morph så teksten overlever en patch.", () => this.querySelector(CONTROL_SELECTOR) !== null &&
            this.listElement !== null &&
            this.querySelector("[role='status']") === null);
        if (control !== this.control) {
            this.control?.removeEventListener("input", this.handleInput);
            this.control?.removeEventListener("keydown", this.handleKeydown);
            this.control?.removeEventListener("focus", this.handleFocus);
            control.addEventListener("input", this.handleInput);
            control.addEventListener("keydown", this.handleKeydown);
            control.addEventListener("focus", this.handleFocus);
            this.control = control;
        }
        /*
         * Et trykk i lista eller på tommeldingen skal ikke ta fokus fra feltet,
         * heller ikke på rullefeltet. Ellers lukket `focusout` lista før valget
         * rakk å skje. Begge kan byttes ut av en patch, så lytterne følger
         * elementene.
         */
        const list = this.listElement;
        if (list !== this.list) {
            this.list?.removeEventListener("mousedown", this.handleListMouseDown);
            list?.addEventListener("mousedown", this.handleListMouseDown);
            this.list = list ?? undefined;
        }
        const empty = this.emptyElement;
        if (empty !== this.empty) {
            this.empty?.removeEventListener("mousedown", this.handleListMouseDown);
            empty?.addEventListener("mousedown", this.handleListMouseDown);
            this.empty = empty ?? undefined;
        }
        // Alternativene byttes ut uavhengig av feltet: i en Datastar-app sender
        // serveren en ny liste mens brukeren skriver. Lå dette bak sjekken over,
        // fikk de nye alternativene aldri lytteren sin, og valg med mus sluttet
        // å virke etter første oppdatering.
        for (const option of this.options) {
            if (this.bound.has(option))
                continue;
            option.addEventListener("mousedown", this.handleOptionMouseDown);
            this.bound.add(option);
        }
    }
    setOpen(open) {
        /*
         * En liste uten alternativer under et tomt felt har ingenting å vise,
         * og åpnes ikke. Uten dette åpnet fokus en tom liste, og et felt som
         * venter på sitt første søk viste «Ingen treff» før brukeren hadde
         * skrevet et tegn: tommeldingen følger lista, og lista var åpen.
         */
        const show = open && this.hasSomethingToShow();
        this.wantOpen = show;
        this.applyOpen(show);
    }
    hasSomethingToShow() {
        return this.options.length > 0 || (this.control?.value ?? "").trim() !== "";
    }
    /** Skriver tilstanden ut i markupen, uten å endre hva brukeren ville. */
    applyOpen(open) {
        const list = this.listElement;
        if (!list || !this.control)
            return;
        setFlag(list, "hidden", !open);
        setAttr(this.control, "aria-expanded", String(open));
        this.syncEmpty();
        if (!open) {
            this.active = undefined;
            this.control.removeAttribute("aria-activedescendant");
            for (const option of this.options) {
                setAttr(option, "aria-selected", "false");
            }
        }
    }
    /**
     * Tommeldingen følger lista: synlig bare når lista er åpen og tom.
     *
     * Den er søsken til lista og ikke barn, så `hidden` på lista skjulte den
     * ikke, og «Ingen treff» ble stående under et lukket felt etter Escape.
     * Har noen andre filtrert, eier de også tommeldingen.
     */
    syncEmpty() {
        if (this.prefiltered)
            return;
        const empty = this.emptyElement;
        const list = this.listElement;
        if (!empty || !list)
            return;
        setFlag(empty, "hidden", Boolean(list.hidden) || this.visible.length > 0);
    }
    /**
     * Skjuler det som ikke passer, og melder hvor mange som er igjen.
     *
     * Kjøres når brukeren skriver eller setter fokus i feltet, og på nytt
     * etter en patch mens lista er åpen, siden patchen kan ha byttet ut
     * alternativene.
     */
    filter() {
        /*
         * Har noen andre filtrert, eier de hva lista viser, og tommeldingen er
         * en del av det. Komponenten rører derfor ingenting her, og opplesningen
         * kommer fra `announceFiltered` i stedet.
         *
         * To utgaver var feil før denne. Den første returnerte med en gang, og
         * da satt en Datastar-app igjen uten opplesning av antall treff i det
         * hele tatt. Den andre telte her: da meldte et tomt felt «Ingen treff»
         * allerede ved fokus, før brukeren hadde skrevet et tegn, og et
         * asynkront søk meldte det på nytt ved hvert tastetrykk mens svaret
         * fortsatt var underveis. Komponenten satte samtidig tommeldingen synlig
         * igjen etter at appen hadde skjult den, og det fantes ingen vei utenom.
         */
        if (this.prefiltered)
            return;
        const query = (this.control?.value ?? "").trim().toLowerCase();
        for (const option of this.options) {
            const label = (option.textContent ?? "").trim().toLowerCase();
            setFlag(option, "hidden", query !== "" && !label.includes(query));
        }
        this.dropHiddenMark();
        this.syncEmpty();
        this.announce(this.visible.length);
    }
    /**
     * Tar bort markeringen fra et alternativ filtreringen nettopp skjulte.
     *
     * Ryddingen hører her, der skjulingen skjer, og ikke bare i `repair()`:
     * den returnerer med en gang under `server-controlled`, og da pekte
     * `aria-activedescendant` på et skjult alternativ, som skjermleseren leste
     * opp likevel.
     */
    dropHiddenMark() {
        const control = this.control;
        if (!control)
            return;
        const marked = this.options.find((option) => option.getAttribute("aria-selected") === "true");
        if (marked?.hidden) {
            setAttr(marked, "aria-selected", "false");
            if (this.active?.id === marked.id)
                this.active = undefined;
        }
        const pointed = control.getAttribute("aria-activedescendant");
        if (pointed) {
            const target = this.options.find((option) => option.id === pointed);
            if (!target || target.hidden) {
                control.removeAttribute("aria-activedescendant");
            }
        }
    }
    /**
     * Leser opp antallet når noen andre har filtrert.
     *
     * Lista endrer seg da ikke av et tastetrykk, men av at appen rendrer på
     * nytt, og det er nettopp den endringen observatøren ser. Beskjeden hører
     * hjemme her og ikke i `filter()`, for mellom tastetrykket og det nye
     * svaret kan det gå et halvt sekund over nettverket.
     *
     * Bare endringer leses opp. Det første antallet er utgangspunktet og ikke
     * en nyhet, og det står ofte 0 der fordi appen ikke har rendret lista
     * ennå.
     */
    announceFiltered() {
        if (!this.prefiltered)
            return;
        const hits = this.visible.length;
        if (hits === this.announcedCount)
            return;
        const first = this.announcedCount === undefined;
        this.announcedCount = hits;
        if (!first)
            this.announce(hits);
    }
    /**
     * Melder antall treff.
     *
     * Uten dette får den som ikke ser skjermen ingen beskjed om at lista
     * snevret seg inn mens hun skrev. Teksten er klientgenerert, så elementet
     * har `data-ignore-morph` fra byggefunksjonen. Mangler elementet, har `bind()`
     * alt sagt fra.
     */
    announce(hits) {
        const status = this.querySelector("[role='status']");
        if (!status)
            return;
        setText(status, this.countText(hits));
    }
    /**
     * Teksten for et antall, på sidens språk.
     *
     * `count-none` gjelder null treff, siden «Ingen treff» er en egen setning
     * og ikke en bøyning. Ellers velger `Intl.PluralRules` formen språket
     * krever, og `count-<form>` gir teksten, med `{n}` for tallet. Mangler
     * formen, gjelder `count-other`. Mangler den også, er teksten norsk.
     */
    countText(hits) {
        const lang = languageOf(this);
        const written = (name) => this.getAttribute(name);
        const form = new Intl.PluralRules(lang).select(hits);
        const template = (hits === 0 ? written("count-none") : null) ??
            written(`count-${form}`) ??
            written("count-other");
        const number = new Intl.NumberFormat(lang).format(hits);
        if (template !== null)
            return template.replaceAll("{n}", number);
        const fallback = hits === 0
            ? DEFAULT_TEXTS.suggestionNone
            : hits === 1
                ? DEFAULT_TEXTS.suggestionOne
                : DEFAULT_TEXTS.suggestionOther;
        return fallback.replaceAll("{n}", number);
    }
    markActive(index) {
        const shown = this.visible;
        if (shown.length === 0 || !this.control)
            return;
        const next = (index + shown.length) % shown.length;
        this.markOption(shown[next]);
        shown[next].scrollIntoView({ block: "nearest" });
    }
    /** Markerer ett alternativ, og husker hvilket. */
    markOption(option) {
        if (!this.control)
            return;
        for (const other of this.options) {
            setAttr(other, "aria-selected", other === option ? "true" : "false");
        }
        this.active = { id: option.id, label: (option.textContent ?? "").trim() };
        // aria-activedescendant flytter skjermleserens lesepunkt uten at fokus
        // forlater feltet. Uten den leses alternativet aldri opp.
        setAttr(this.control, "aria-activedescendant", option.id);
    }
    choose(option) {
        if (!this.control)
            return;
        const value = option.getAttribute("data-value") ?? (option.textContent ?? "").trim();
        this.control.value = value;
        // Flagget må stå før hendelsene sendes. `input` er den samme hendelsen
        // komponenten selv lytter på, så uten dette åpner lista seg igjen i det
        // øyeblikket brukeren har valgt noe.
        this.choosing = true;
        this.control.dispatchEvent(new Event("input", { bubbles: true }));
        this.control.dispatchEvent(new Event("change", { bubbles: true }));
        this.choosing = false;
        this.setOpen(false);
        this.dispatchEvent(new CustomEvent("suggestion-select", {
            detail: { value },
            bubbles: true,
            composed: true,
        }));
    }
}
FsSuggestion.observedAttributes = [
    "prefiltered",
    SERVER_CONTROLLED,
    ...COUNT_ATTRIBUTES,
];
export function defineFsSuggestion(tagName = FS_SUGGESTION_TAG) {
    defineElement(tagName, FsSuggestion);
}
//# sourceMappingURL=fs-suggestion.js.map