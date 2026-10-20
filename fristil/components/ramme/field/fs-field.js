import { addClass, defineElement, HostElement, setAttr, setFlag, warnAboutMarkup, } from "../../host-element.js";
import { computeFieldAttributes } from "./field-core.js";
export const FS_FIELD_TAG = "fs-field";
function uniqueId(prefix) {
    return `${prefix}-${Math.random().toString(36).slice(2, 9)}`;
}
const CONTROL_SELECTOR = "input:not([type='hidden']), textarea, select";
function readMarker(value) {
    return value === "none" || value === "symbol" || value === "text"
        ? value
        : undefined;
}
/**
 * Attributtene `<fs-field>` regner ut selv, og setter tilbake om de blir
 * borte.
 *
 * Det samme sto en gang i `FIELD_PRESERVED_ATTRIBUTES`, som malen måtte
 * skrive av inn i `data-preserve-attr`. Den lista finnes ikke lenger:
 * komponenten ser at attributtene er borte og setter dem tilbake.
 */
const DERIVED_ATTRIBUTES = [
    "class",
    "for",
    "id",
    "aria-describedby",
    "aria-invalid",
    "aria-disabled",
    "data-state",
    "data-required",
    "data-optional",
    "disabled",
    "hidden",
];
function readControlWord(control) {
    return {
        invalid: control.getAttribute("aria-invalid"),
        describedBy: control.getAttribute("aria-describedby"),
        disabled: control.hasAttribute("disabled"),
    };
}
/**
 * Kobler ledetekst, kontroll, hjelpetekst og feilmelding i vanlig DOM.
 *
 * Komponenten er for markup som blir til uten JavaScript: en Go-mal, en
 * PHP-fil, en Razor-visning eller håndskrevet HTML. Lages markupen med
 * JavaScript, uansett hvor koden kjører, skal `fs.field()` skrive
 * attributtene i stedet, og da trengs ikke dette elementet.
 *
 * Komponenten rendrer ingenting. Den satte tidligere et `<slot>`-element inn i
 * vanlig DOM, og siden serveren ikke visste om det, fjernet Datastars morfing
 * det ved hver patch.
 *
 * Malen trenger ingenting ekstra. River en morfing bort koblingen, ser
 * komponenten det og setter den tilbake. Feltet har ingen tilstand brukeren
 * lager selv, så det finnes heller ingenting `server-controlled` kunne slått
 * av her: alt komponenten skriver er utledet av markupen serveren sendte.
 */
export class FsField extends HostElement {
    constructor() {
        super(...arguments);
        this.managedIds = new Set();
        /**
         * Hva komponenten skrev på hver kontroll, nøklet på selve kontrollen.
         *
         * Et `WeakMap` og ikke et felt med «forrige kontroll». Minnet skal
         * overleve at verten kobles fra og til igjen, som når React flytter et
         * felt, og at kontrollen forsvinner og kommer tilbake i en patch. Et felt
         * som ble nullstilt i `disconnectedCallback` gjorde at komponenten leste
         * sitt eget `aria-invalid="true"` som serverens ord etter en flytting, og
         * feltet kunne aldri bli gyldig igjen. Samtidig holder et `WeakMap` ingen
         * løsrevet node i live: er kontrollen borte for godt, er minnet det også.
         */
        this.memory = new WeakMap();
        this.labelMemory = new WeakMap();
        /** Kontrollene der `data-state` er komponentens egen. */
        this.stateByMe = new WeakSet();
    }
    /**
     * Egenskapene speiler attributtene.
     *
     * Tilstanden bor i attributtet og ikke i et felt på klassen, slik at det
     * serveren sendte og det komponenten mener alltid er det samme. En egen
     * `invalid`-variabel ville kunne si noe annet enn markupen etter en morfing.
     */
    get invalid() {
        return this.hasAttribute("invalid");
    }
    set invalid(value) {
        setFlag(this, "invalid", value);
    }
    get disabled() {
        return this.hasAttribute("disabled");
    }
    set disabled(value) {
        setFlag(this, "disabled", value);
    }
    get optional() {
        return this.hasAttribute("optional");
    }
    set optional(value) {
        setFlag(this, "optional", value);
    }
    /**
     * Markeringen ledeteksten faktisk har.
     *
     * Leses fra markupen: først attributtet på verten, så `data-required` på
     * ledeteksten, som serveren skriver med `fs.field()`. En getter som bare så
     * på sitt eget attributt svarte «none» mens ledeteksten viste en stjerne.
     *
     * `none` er en egen verdi og ikke fraværet av en: står den på verten,
     * overstyrer den det serveren skrev på ledeteksten. Setteren skriver den
     * derfor bokstavelig. En setter som oversatte `none` til «fjern
     * attributtet» ga `felt.requiredMarker = "none"` en annen betydning enn
     * `required-marker="none"`, og markeringen serveren skrev kom tilbake.
     */
    get requiredMarker() {
        const control = this.querySelector(CONTROL_SELECTOR);
        return (readMarker(this.getAttribute("required-marker")) ??
            readMarker(this.resolveLabel(control)?.getAttribute("data-required")) ??
            "none");
    }
    set requiredMarker(value) {
        setAttr(this, "required-marker", value);
    }
    get controlId() {
        return this.getAttribute("control-id") ?? undefined;
    }
    set controlId(value) {
        setAttr(this, "control-id", value ?? null);
    }
    get describedBy() {
        return this.getAttribute("described-by") ?? undefined;
    }
    set describedBy(value) {
        setAttr(this, "described-by", value ?? null);
    }
    connectedCallback() {
        /*
         * `slotchange` melder ikke fra i vanlig DOM, og innholdet byttes ut mens
         * brukeren fyller ut skjemaet.
         *
         * Attributtene er med, ikke bare barna. En morfing river bort det som
         * ikke står i serverens HTML, og koblingen mellom ledetekst, felt og
         * hjelpetekst er nettopp det: noe komponenten regnet ut, ikke noe
         * serveren sendte. Før måtte malen liste opp attributtene i
         * `data-preserve-attr` for at de skulle overleve. Nå ser komponenten at
         * de er borte, og setter dem tilbake.
         *
         * Lista er avgrenset til det komponenten selv utleder. De andre
         * komponentene gjør det samme med sin egen tilstand, hver med sin liste.
         *
         * Hver skriving i `sync()` sammenligner først. Uten det ville
         * observatøren utløst seg selv i det uendelige.
         */
        this.observer = new MutationObserver(() => this.sync());
        this.observer.observe(this, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: DERIVED_ATTRIBUTES,
        });
        this.sync();
    }
    disconnectedCallback() {
        this.observer?.disconnect();
        this.observer = undefined;
    }
    attributeChangedCallback() {
        if (this.isConnected)
            this.sync();
    }
    /**
     * Id-en kontrollen skal ha, lest fra markupen når den står der.
     *
     * Ledetekstens `for` er med som kilde, og uten den mistet feltet koblingen
     * ved første patch: morfingen kan erstatte kontrollen med serverens node,
     * som ikke har noen id, mens ledeteksten beholder sin `for`. Da fant
     * komponenten ingen id, fant opp en ny, og skrev den bare på kontrollen, og
     * `for` pekte etter det på et element som ikke fantes. Funnet i
     * spilldemoen, i appen som sender HTML-biter fra en Kotlin-server.
     *
     * Lager komponenten id-en selv, huskes den. En ny id per patch ville gitt
     * en skjermleser en peker som skiftet under opplesningen.
     */
    resolveControlId(control, label) {
        const fromMarkup = this.getAttribute("control-id") || control.id || label?.htmlFor;
        if (!fromMarkup && !this.lastId) {
            this.generatedControlId ??= uniqueId("fs-field-control");
        }
        this.lastId = fromMarkup || this.lastId || this.generatedControlId;
        return this.lastId;
    }
    /**
     * Ledeteksten feltet hører sammen med.
     *
     * Vanligvis står den inni elementet. Står den utenfor, med `for` som peker
     * på kontrollen, er feltet like godt navngitt, og komponenten kobler den
     * på samme måte. Én forskjell er verdt å vite: en ledetekst utenfor ligger
     * ikke i det komponenten observerer, så river en patch klassen av den,
     * kommer den ikke tilbake av seg selv.
     */
    resolveLabel(control) {
        const inside = this.querySelector("label");
        if (inside || !control?.id)
            return inside;
        const root = this.getRootNode();
        return (root.querySelector?.(`label[for="${CSS.escape(control.id)}"]`) ?? null);
    }
    /**
     * Serverens ord om kontrollen, utledet av hva som står der nå mot hva
     * komponenten selv skrev sist.
     *
     * En kontroll komponenten aldri har skrevet på, sier bare serverens ord.
     * Ellers er hvert attributt serverens hvis det er et annet enn det
     * komponenten skrev, og uendret hvis det er det samme.
     */
    serverLabelWord(label) {
        if (!label)
            return { required: null, optional: false };
        const now = {
            required: label.getAttribute("data-required"),
            optional: label.hasAttribute("data-optional"),
        };
        const known = this.labelMemory.get(label);
        if (!known)
            return now;
        return {
            required: now.required !== known.written.required
                ? now.required
                : known.server.required,
            optional: now.optional !== known.written.optional
                ? now.optional
                : known.server.optional,
        };
    }
    serverWord(control, now) {
        const known = this.memory.get(control);
        if (!known)
            return now;
        return {
            invalid: now.invalid !== known.written.invalid
                ? now.invalid
                : known.server.invalid,
            describedBy: now.describedBy !== known.written.describedBy
                ? now.describedBy
                : known.server.describedBy,
            disabled: now.disabled !== known.written.disabled
                ? now.disabled
                : known.server.disabled,
        };
    }
    sync() {
        const control = this.querySelector(CONTROL_SELECTOR);
        const help = this.querySelector(".fs-help-text");
        const error = this.querySelector(".fs-error-text");
        if (!control) {
            warnAboutMarkup(this, "fant ingen kontroll å koble til. Ledeteksten, hjelpeteksten og " +
                "feilmeldingen står uten et felt, og koblingen kan ikke lages. " +
                "Sett inn et <input>, <textarea> eller <select>.", 
            // Et tomt element er et område serveren ikke har fylt ennå, og det
            // er ikke en feil i markupen.
            () => this.childElementCount > 0 && !this.querySelector(CONTROL_SELECTOR));
            return;
        }
        // Ledeteksten kan stå utenfor elementet. Oppslaget må skje etter at
        // kontrollen er funnet, siden det går via id-en hennes.
        const label = this.resolveLabel(control);
        warnAboutMarkup(this, "fant ingen <label>. Feltet får da ingen ledetekst, og en " +
            "skjermleser leser det opp uten navn.", 
        /*
         * Tre lovlige måter å gi feltet et navn på, og ingen av dem skal gi en
         * advarsel: en `<label>` inni, en `<label for>` utenfor, eller
         * `aria-label` og `aria-labelledby` på kontrollen, som i et søkefelt
         * med bare et ikon.
         */
        () => {
            const named = this.querySelector(CONTROL_SELECTOR);
            if (!named || this.resolveLabel(named))
                return false;
            return (!named.hasAttribute("aria-label") &&
                !named.hasAttribute("aria-labelledby"));
        });
        /*
         * Id-ene hjelpeteksten og feilmeldingen sist hadde, husket mellom rundene.
         *
         * Markupen er kilden så lenge den har dem. River en morfing dem bort,
         * ville en ny id blitt laget for hver eneste patch, og en skjermleser som
         * står midt i en opplesning ville fulgt en peker som skiftet under den.
         * Minnet er ikke en parallell utgave av tilstanden: står id-en i
         * markupen, er det den som gjelder.
         */
        if (help) {
            if (help.id)
                this.lastHelpId = help.id;
            else {
                this.lastHelpId ??= uniqueId("fs-field-help");
                setAttr(help, "id", this.lastHelpId);
            }
            this.managedIds.add(help.id);
        }
        if (error) {
            if (error.id)
                this.lastErrorId = error.id;
            else {
                this.lastErrorId ??= uniqueId("fs-field-error");
                setAttr(error, "id", this.lastErrorId);
            }
            this.managedIds.add(error.id);
        }
        // Markeringene leses også fra markupen. Skrev serveren dem med
        // `fs.field()`, står de på ledeteksten, og en komponent som bare så på
        // sine egne attributter ville fjernet dem igjen. I React ga det en
        // hydreringsfeil: serveren sendte `data-required="symbol"`, komponenten
        // tok det bort, og så mente React at HTML-en ikke stemte.
        const serverLabel = this.serverLabelWord(label);
        const marker = readMarker(this.getAttribute("required-marker")) ??
            readMarker(serverLabel.required);
        // Tilstanden leses fra markupen, ikke bare fra et attributt på verten.
        // Skrev serveren feltet med `fs.field()`, står svaret allerede på
        // kontrollen, og en komponent som regnet ut sitt eget ville fjernet det
        // igjen. Da kranglet de to halvdelene av API-et med hverandre. Se
        // `ControlMemory` for hvordan serverens ord skilles fra komponentens eget.
        const server = this.serverWord(control, readControlWord(control));
        // `aria-invalid` har fire lovlige verdier, og både `grammar` og
        // `spelling` betyr ugyldig. Fravær, `false` og tom streng betyr gyldig;
        // ARIA sier at tom streng skal leses som `false`, og hjelpemidlene gjør
        // det, så komponenten kan ikke vise rød ramme på den.
        const serverInvalid = server.invalid !== null &&
            server.invalid !== "false" &&
            server.invalid !== "";
        const invalid = this.hasAttribute("invalid") || serverInvalid;
        const disabled = this.hasAttribute("disabled") || server.disabled;
        /*
         * Id-ene serveren selv la i `aria-describedby`, utenom dem komponenten
         * forvalter. `fs.field({ describedBy })` skriver dem rett på kontrollen,
         * og de skal med videre. De forvaltede strykes her og legges til igjen
         * etter dagens tilstand, ellers ble feilmeldingens id stående etter at
         * feilen var borte.
         *
         * Forvaltet er dagens hjelpetekst og feilmelding, og en id komponenten
         * har forvaltet før som ikke lenger peker på noe. Det siste er for et
         * skript som fjerner hjelpeteksten uten å røre kontrollen. Eierskapet
         * avgjøres av DOM-en og ikke av historikken alene: peker id-en fortsatt
         * på et element, som når serveren flytter hjelpeteksten ut av feltet og
         * beholder id-en, er den serverens.
         */
        const root = this.getRootNode();
        const managedNow = (id) => id === help?.id ||
            id === error?.id ||
            (this.managedIds.has(id) && !root.getElementById?.(id));
        const serverExtras = (server.describedBy ?? "")
            .split(/\s+/)
            .filter((id) => id && !managedNow(id));
        const computed = computeFieldAttributes({
            id: this.resolveControlId(control, label),
            help: Boolean(help),
            error: Boolean(error),
            helpId: help?.id,
            errorId: error?.id,
            required: marker === "symbol" || marker === "text" ? marker : undefined,
            optional: this.hasAttribute("optional") || serverLabel.optional,
            invalid,
            disabled,
            describedBy: [...serverExtras, this.getAttribute("described-by") ?? ""],
        });
        setAttr(control, "id", computed.control.id);
        if (label) {
            addClass(label, computed.label.class);
            // Alltid, ikke bare når den mangler: `for` og `id` er den samme
            // opplysningen, og de to kan ikke få lov til å si hver sin ting.
            setAttr(label, "for", computed.label.for);
            setAttr(label, "data-required", computed.label["data-required"]);
            setAttr(label, "data-optional", computed.label["data-optional"]);
            this.labelMemory.set(label, {
                written: {
                    required: computed.label["data-required"] ?? null,
                    optional: computed.label["data-optional"] !== undefined,
                },
                server: serverLabel,
            });
            setAttr(label, "aria-disabled", computed.label["aria-disabled"]);
        }
        if (error) {
            // Bare `hidden`. Et skjult element er allerede ute av
            // tilgjengelighetstreet, så `aria-hidden` var overflødig, og ga en
            // hydreringsfeil i React fordi serveren ikke skriver det.
            setFlag(error, "hidden", Boolean(computed.error.hidden));
        }
        // Serverens ord står ordrett når det er serveren som sier feltet er
        // ugyldig, også `spelling` og `grammar`. Ellers skriver komponenten
        // `true` når verten sier det, og lar et `false` serveren skrev stå:
        // det er gyldig og vanlig i håndskrevet HTML, og ble strøket ved hver
        // patch.
        const ariaInvalid = serverInvalid
            ? server.invalid
            : invalid
                ? "true"
                : server.invalid;
        setAttr(control, "aria-describedby", computed.control["aria-describedby"]);
        setAttr(control, "aria-invalid", ariaInvalid);
        // `setFlag` og ikke `setAttr`: en mal kan ha skrevet
        // `disabled="disabled"`, og den skal stå som den er. `setAttr` ville
        // normalisert verdien til den tomme strengen, og siden `disabled` er
        // blant attributtene komponenten observerer, ville serveren og
        // komponenten skrevet hver sin verdi ved hver patch. Ingen `aria-disabled`
        // ved siden av: et ekte `disabled` er alt synlig for hjelpemidlene, og
        // `computeFieldAttributes` skriver det bare på ledeteksten.
        setFlag(control, "disabled", disabled);
        this.memory.set(control, {
            written: {
                invalid: ariaInvalid,
                describedBy: computed.control["aria-describedby"] ?? null,
                disabled,
            },
            server,
        });
        // `data-state` settes bare på systemets egne kontroller, og bare når
        // konsumenten ikke har satt den selv. Den fjernes bare når komponenten
        // selv satte den og feltet ikke lenger er ugyldig. En `data-state` en
        // mal har skrevet for hånd, er malens.
        const isSystemField = control.classList.contains("fs-input") ||
            control.classList.contains("fs-textarea") ||
            control.classList.contains("fs-select");
        const state = computed.control["data-state"];
        if (state && isSystemField && !control.hasAttribute("data-state")) {
            setAttr(control, "data-state", state);
            this.stateByMe.add(control);
        }
        else if (!state &&
            this.stateByMe.has(control) &&
            control.getAttribute("data-state") === "invalid") {
            control.removeAttribute("data-state");
            this.stateByMe.delete(control);
        }
    }
}
FsField.observedAttributes = [
    "invalid",
    "disabled",
    "optional",
    "required-marker",
    "control-id",
    "described-by",
];
export function defineFsField(tagName = FS_FIELD_TAG) {
    defineElement(tagName, FsField);
}
//# sourceMappingURL=fs-field.js.map