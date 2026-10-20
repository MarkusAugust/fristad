import { addClass, defineElement, HostElement, setAttr, warnAboutMarkup, } from "../../host-element.js";
import { ERROR_SUMMARY_CLASS, ERROR_SUMMARY_TITLE_CLASS, } from "./error-summary.js";
export const FS_ERROR_SUMMARY_TAG = "fs-error-summary";
/**
 * Sender brukeren fra en feil i oppsummeringen til feltet som feilet.
 *
 * Serveren skriver innholdet: overskriften og lista med lenker. Lages
 * markupen med JavaScript, skriver `fs.errorSummary()` også klassen,
 * `role="alert"` og `tabindex="-1"` på boksen, og komponenten lar det stå.
 * Kommer markupen fra en mal uten JavaScript, fyller komponenten inn de
 * tre, og klassen på overskriften, og setter dem tilbake når en patch
 * river dem bort. Komponenten lager ingen noder: den gjorde det før, og da
 * forsvant overskriften ved første morfing i Datastar uten å komme tilbake.
 *
 * Det som er igjen er to ting nettleseren ikke gjør selv:
 *
 * 1. Fokus flyttes til boksen når den kommer til syne, så den som hører siden
 *    får vite at innsendingen stoppet.
 * 2. Fokus gis til selve kontrollen når en lenke følges. En vanlig ankerlenke
 *    ruller bare dit, og neste tastetrykk fortsetter der fokus sto før.
 *
 * ```html
 * <fs-error-summary>
 *   <h2>Skjemaet har to feil</h2>
 *   <ul class="fs-list">
 *     <li><a href="#epost">Skriv en gyldig e-postadresse</a></li>
 *   </ul>
 * </fs-error-summary>
 * ```
 */
/** Dekoder et fragment, og gir det tilbake urørt hvis det ikke er gyldig. */
function decodeFragment(fragment) {
    try {
        return decodeURIComponent(fragment);
    }
    catch {
        return fragment;
    }
}
export class FsErrorSummary extends HostElement {
    constructor() {
        super(...arguments);
        this.hasFocused = false;
        /**
         * Om komponenten har prøvd å flytte fokus siden boksen sist ble synlig.
         *
         * Første forsøk skjer uansett hvor fokus står: en ny innsending har
         * feilet, og det er beskjeden. Landet det ikke, fordi boksen sto i et
         * skjult panel, prøver komponenten igjen ved neste endring i lista, men
         * bare når ingen står i et felt. Uten det skillet rev gjenforsøket fokus
         * ut av feltet brukeren rettet i, i det live-valideringen patchet lista.
         */
        this.focusAttempted = false;
        this.links = new Set();
        this.handleLinkClick = (event) => {
            const link = event.currentTarget;
            const id = link.getAttribute("href")?.slice(1);
            if (!id)
                return;
            // Vakten står igjen for kappløpet: målet kan ha forsvunnet i en patch
            // mellom synkroniseringen og klikket. `sync()` har alt meldt fra om en
            // lenke som aldri har hatt et mål.
            const target = this.resolveTarget(id);
            if (!target)
                return;
            event.preventDefault();
            // Er lenken til en ledetekst, skal fokus til det den hører til, enten
            // den peker med `for` eller omslutter kontrollen. `control` dekker begge
            // for ekte kontroller; peker `for` på noe annet, som en gruppe, følges
            // id-en dit.
            const focusable = target instanceof HTMLLabelElement
                ? (target.control ??
                    (target.htmlFor ? this.resolveTarget(target.htmlFor) : null) ??
                    target)
                : target;
            // Et mål som ikke kan få fokus, som en overskrift, får en `tabindex`
            // for dette ene hoppet. Det er en skriving på markup serveren eier, og
            // en patch kan ta den igjen; da har hoppet alt skjedd.
            if (!focusable.hasAttribute("tabindex") && !isFocusable(focusable)) {
                setAttr(focusable, "tabindex", "-1");
            }
            focusable.focus();
            // Uten `behavior`: sidens egen `scroll-behavior` bestemmer, og da
            // gjelder `prefers-reduced-motion` av seg selv.
            focusable.scrollIntoView({ block: "center" });
        };
    }
    connectedCallback() {
        // Feilene kommer og går mens brukeren retter, og `slotchange` melder ikke
        // fra i vanlig DOM. Attributtene er de komponenten selv fyller inn, så
        // en patch som river dem bort får dem tilbake. Hver skriving
        // sammenligner først, ellers ville observatøren utløst seg selv.
        this.observer = new MutationObserver(() => this.sync());
        this.observer.observe(this, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ["class", "role", "tabindex"],
        });
        this.sync();
    }
    attributeChangedCallback() {
        // Serveren kan sende boksen skjult og senere bare ta bort `hidden` på
        // den samme noden. Uten dette får en oppsummering som nettopp ble synlig
        // aldri fokus, og det er hele grunnen til at komponenten finnes.
        if (this.isConnected)
            this.sync();
    }
    disconnectedCallback() {
        this.observer?.disconnect();
        this.observer = undefined;
        for (const link of this.links) {
            link.removeEventListener("click", this.handleLinkClick);
        }
        this.links.clear();
    }
    /**
     * Om boksen skal ta fokus når den blir synlig. Standard: ja.
     *
     * Attributtet heter `data-autofocus` og ikke `autofocus`, selv om det
     * siste leser bedre. `autofocus` er en boolsk egenskap på `HTMLElement`,
     * og React 19 setter egenskaper framfor attributter på web
     * components. `autofocus="false"` ble da til `el.autofocus = "false"`, som
     * er sant, mens attributtet aldri kom i markupen, og avslaget virket ikke.
     * `data-*` sendes videre som attributt i alle React-versjoner.
     */
    get shouldFocus() {
        return this.dataset.autofocus !== "false";
    }
    /**
     * Fyller inn det en mal uten JavaScript ikke skrev. Bare det som mangler:
     * en rolle serveren har valgt selv står, og det gjør en `tabindex` også.
     */
    wire() {
        addClass(this, ERROR_SUMMARY_CLASS);
        if (!this.hasAttribute("role"))
            setAttr(this, "role", "alert");
        // Uten `tabindex` gjør `focus()` ingenting, og hele grunnen til
        // komponenten forsvinner i stillhet.
        if (!this.hasAttribute("tabindex"))
            setAttr(this, "tabindex", "-1");
        const title = this.querySelector("h1, h2, h3, h4, h5, h6");
        if (title)
            addClass(title, ERROR_SUMMARY_TITLE_CLASS);
    }
    sync() {
        this.wire();
        const links = [
            ...this.querySelectorAll("li a[href^='#']"),
        ];
        // Skjuler serveren boksen igjen, er den innsendingen over. Uten denne
        // nullstillingen tok boksen fokus bare første gang: mønsteret i en
        // Datastar-app er at lista står med de samme lenkene hele veien og bare
        // `hidden` slås av og på, så «har jeg flyttet fokus hit før» er ikke et
        // svar på om dette er en ny innsending. Den står før returen for en tom
        // liste: React fjerner punktene før den setter `hidden` på verten, og da
        // er lista alt tom i den runden boksen skjules.
        if (this.hidden) {
            this.hasFocused = false;
            this.focusAttempted = false;
        }
        if (links.length === 0) {
            // Flaggene nullstilles ikke av en tom liste, bare av `hidden` over. En
            // synlig liste som tømmes og fylles mens brukeren retter, rykket
            // ellers fokus ut av feltet han sto i.
            warnAboutMarkup(this, 'fant ingen lenker til feltene. Hvert punkt trenger en <a href="#id"> ' +
                "som peker på kontrollen eller ledeteksten, ellers kommer brukeren " +
                "seg ikke fra feilen til feltet.", 
            // En tom boks er ikke en feil: mønsteret er at serveren lar den stå
            // med `hidden` og fyller den når innsendingen feiler. En boks med
            // punkter, men uten lenker til feltene, er noe annet.
            () => this.querySelector("li") !== null &&
                this.querySelector("li a[href^='#']") === null);
            return;
        }
        for (const link of links) {
            /*
             * En lenke som ikke fører noe sted, meldt her og ikke først ved et
             * klikk.
             *
             * Sjekken sto bare i klikkhåndtereren, og da var en boks med en lenke
             * til et felt som ikke finnes helt taus til noen faktisk fulgte den.
             * En feiloppsummering leses av den som nettopp mislyktes med et skjema,
             * og en lenke som ikke virker er akkurat det som gjør boksen verdiløs.
             * Id-en står i meldingen, og hver lenke har sin egen, så en boks med to
             * ødelagte lenker sier fra om begge.
             */
            const id = link.getAttribute("href")?.slice(1);
            if (id) {
                warnAboutMarkup(this, `lenken peker på #${id}, men det finnes ikke noe element med den ` +
                    "id-en. Lenken ruller ingen steder, og fokus blir stående.", () => this.resolveTarget(id) === null);
            }
            if (this.links.has(link))
                continue;
            link.addEventListener("click", this.handleLinkClick);
            this.links.add(link);
        }
        if (this.hidden)
            return;
        // Er boksen synlig nå, og vi ikke har flyttet fokus hit ennå, er det
        // denne innsendingen som feilet. Flagget settes bare når fokus faktisk
        // landet: står boksen i et skjult panel eller en lukket dialog, gjør
        // `focus()` ingenting, og da skal neste forsøk få lov, så sant ingen
        // står i et felt.
        if (this.shouldFocus && !this.hasFocused) {
            // `document.activeElement` og ikke rotas: står boksen i en skyggerot
            // og feltet i vanlig DOM, er rotas `activeElement` null, og brukeren
            // ville blitt regnet som «ingen». Dokumentets peker på skyggeverten når
            // fokus står i et skyggetre, og er aldri null for et felt i siden.
            const active = document.activeElement;
            const nobodyTyping = !active || active === document.body;
            if (this.focusAttempted && !nobodyTyping)
                return;
            const root = this.getRootNode();
            this.focusAttempted = true;
            this.focus();
            this.hasFocused = root.activeElement === this;
        }
    }
    /**
     * Elementet en lenke peker på.
     *
     * Oppslaget går mot rota komponenten selv står i, ikke mot `document`.
     * Ligger skjemaet i en skyggerot, som i en forhåndsvisning eller inne i en
     * annen komponent, finner `document.getElementById` ingenting, og lenken
     * blir en vanlig ankerlenke uten fokusflytting.
     */
    resolveTarget(raw) {
        const root = this.getRootNode();
        // En lenke til et felt med æ, ø eller å i id-en er gjerne prosentkodet,
        // `#f%C3%B8dselsdato`. Nettleseren dekoder den når lenken følges, så
        // oppslaget må gjøre det samme, ellers meldes en lenke som virker.
        for (const id of new Set([raw, decodeFragment(raw)])) {
            const target = root.getElementById?.(id) ?? document.getElementById(id);
            if (target)
                return target;
        }
        return null;
    }
}
FsErrorSummary.observedAttributes = ["data-autofocus", "hidden"];
function isFocusable(element) {
    return (element instanceof HTMLInputElement ||
        element instanceof HTMLSelectElement ||
        element instanceof HTMLTextAreaElement ||
        element instanceof HTMLButtonElement ||
        element instanceof HTMLAnchorElement);
}
export function defineFsErrorSummary(tagName = FS_ERROR_SUMMARY_TAG) {
    defineElement(tagName, FsErrorSummary);
}
//# sourceMappingURL=fs-error-summary.js.map