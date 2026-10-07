import { addClass, defineElement, HostElement, isServerControlled, SERVER_CONTROLLED, setAttr, setFlag, uniqueId, warnAboutMarkup, } from "../../host-element.js";
import { DIALOG_CLASS, DIALOG_TITLE_CLASS } from "./dialog.js";
export const FS_DIALOG_TAG = "fs-dialog";
/**
 * En modal dialog en server kan åpne.
 *
 * Å åpne en `<dialog>` er et kall og ikke et attributt. `showModal()` flytter
 * fokus inn, holder fokus inne i dialogen, lukker på Escape og gjør resten av
 * siden utilgjengelig. `<dialog open>` gjør ingen av delene: det er en boks
 * på siden.
 *
 * En server som bare sender HTML kan ikke kalle noe. Uten denne komponenten
 * kunne en app i Datastar, htmx eller en Go-mal altså ikke åpne en dialog i
 * det hele tatt, uten å skrive sitt eget skript ved siden av. Her sier
 * serveren i stedet at dialogen er åpen, og komponenten gjør kallet.
 *
 * Lukker brukeren dialogen, med Escape eller med en knapp i en
 * `<form method="dialog">`, fjernes `open` fra verten igjen, slik at
 * markupen sier det samme som skjermen. Komponenten melder fra med
 * `dialog-toggle`, som bærer `open` og `returnValue`. Et klikk på flaten bak
 * lukker den ikke: det gjør heller ikke en vanlig `<dialog>`, og komponenten
 * legger ingenting til.
 *
 * `open` på **verten** er serverens. Sender serveren området på nytt med
 * `open` fortsatt satt, åpnes dialogen igjen. `open` på selve `<dialog>` er
 * nettleserens, satt av `showModal()`, og det setter komponenten tilbake når
 * en patch river det bort. Malen trenger ingen `data-preserve-attr`.
 *
 * Skriv `open` på selve `<dialog>` også når dialogen skal vises. Uten
 * JavaScript er en `<dialog>` uten `open` skjult, og da finnes ikke
 * innholdet i det hele tatt. `fs.dialog({ open: true })` gir begge.
 *
 * `server-controlled` på verten slår av reparasjonen av `open` på
 * `<dialog>`. Da bestemmer hver patch om dialogen vises, også når den står i
 * topplaget.
 *
 * ```html
 * <fs-dialog open>
 *   <dialog class="fs-dialog" aria-labelledby="tittel" open>
 *     <h2 class="fs-dialog__title" id="tittel">Vedtaket er registrert</h2>
 *     <div class="fs-dialog__body">Saken er ferdigbehandlet.</div>
 *     <form method="dialog" class="fs-dialog__footer">
 *       <button class="fs-button" value="lukk">Lukk</button>
 *     </form>
 *   </dialog>
 * </fs-dialog>
 * ```
 */
export class FsDialog extends HostElement {
    constructor() {
        super(...arguments);
        this.handleClose = () => {
            // `close` er køet, ikke synkron. Rekker serveren å lukke og åpne igjen i
            // samme oppgave, gjelder ikke denne hendelsen lenger, og uten sperren
            // lukket komponenten dialogen den nettopp hadde åpnet.
            if (this.dialog?.matches(":modal"))
                return;
            // Lukket serveren den, er verten alt i takt, og `sync()` har meldt fra.
            if (!this.open)
                return;
            this.removeAttribute("open");
            this.notify(false, this.dialog?.returnValue ?? "");
        };
    }
    /** Om dialogen er åpen. Speiler `open`-attributtet. */
    get open() {
        return this.hasAttribute("open");
    }
    set open(value) {
        setFlag(this, "open", value);
    }
    connectedCallback() {
        /*
         * Serveren kan sende dialogen inn i et område som allerede står i siden,
         * og da finnes ikke `<dialog>` ennå når komponenten kobles til. Bare en
         * endring blant vertens egne barn kjører hele `sync()`: dialogen er
         * alltid et direkte barn, og innholdet inni den endrer seg ved hver
         * patch uten at det sier noe om hvilken dialog som er komponentens.
         *
         * `open` på selve `<dialog>` er med. Nettleseren setter det når
         * `showModal()` kalles, og en morfing river det bort igjen, siden
         * serveren ikke sendte det. Uten dette forsvant en åpen dialog i det noe
         * i området rundt ble patchet, og malen måtte skrive
         * `data-preserve-attr="open"` for å hindre det.
         */
        /*
         * Attributtene er med for koblingen komponenten fyller inn når markupen
         * kom uten den: klassen, overskriftens id og `aria-labelledby`. Men en
         * attributtpost kjører bare koblingen, ikke hele `sync()`. `close()`
         * fjerner `open` synkront og køer `close`-hendelsen som en egen oppgave,
         * og observatøren kjører før den. Endret et klikk en klasse på knappen
         * i samme oppgave som det lukket dialogen, så `sync()` en vert med `open`
         * og en dialog som ikke var modal, og åpnet den igjen før lukkingen rakk
         * å bli håndtert. Hver skriving sammenligner først, ellers ville
         * observatøren utløst seg selv.
         */
        this.observer = new MutationObserver((records) => {
            const swapped = records.some((record) => record.type === "childList" && record.target === this);
            if (swapped)
                this.sync();
            else
                this.rewire();
        });
        this.observer.observe(this, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ["class", "id", "aria-labelledby"],
        });
        this.sync();
    }
    disconnectedCallback() {
        this.observer?.disconnect();
        this.observer = undefined;
        this.openObserver?.disconnect();
        this.openObserver = undefined;
        this.dialogElement?.removeEventListener("close", this.handleClose);
        this.dialogElement = undefined;
    }
    attributeChangedCallback() {
        this.sync();
    }
    /**
     * Dialogen komponenten styrer.
     *
     * Bare et direkte barn. En `<dialog>` lenger ned i treet kan tilhøre noe
     * annet, og skal ikke åpnes av denne komponenten.
     */
    get dialog() {
        return this.querySelector(":scope > dialog");
    }
    /** Koblingen alene, for en attributtpost. Se observatøren. */
    rewire() {
        const dialog = this.dialog;
        if (dialog?.isConnected)
            this.wire(dialog);
    }
    /**
     * Fyller inn det en mal uten JavaScript ikke skrev: klassen på dialogen,
     * og tittelen. Tittelen er det `aria-labelledby` peker på når serveren
     * skrev det, ellers den første overskriften, og bare når dialogen ikke alt
     * har et navn fra `aria-label`. Klassen og id-en skrives bare på tittelen:
     * en overskrift i kroppen til en dialog med `aria-label` er ikke tittelen,
     * og skal ikke stiles som den.
     */
    wire(dialog) {
        addClass(dialog, DIALOG_CLASS);
        const named = dialog.getAttribute("aria-labelledby");
        if (named) {
            // Attributtet kan liste flere id-er. Tittelen er den første som står
            // inne i dialogen.
            const root = this.getRootNode();
            const title = named
                .split(/\s+/)
                .map((id) => root.getElementById?.(id) ?? null)
                .find((element) => element !== null && dialog.contains(element));
            if (title)
                addClass(title, DIALOG_TITLE_CLASS);
            return;
        }
        if (dialog.hasAttribute("aria-label"))
            return;
        const title = dialog.querySelector("h1, h2, h3, h4, h5, h6");
        if (!title)
            return;
        addClass(title, DIALOG_TITLE_CLASS);
        if (!title.id) {
            this.titleId ??= uniqueId("fs-dialog-title");
            setAttr(title, "id", this.titleId);
        }
        setAttr(dialog, "aria-labelledby", title.id);
    }
    notify(open, returnValue = "") {
        this.dispatchEvent(new CustomEvent("dialog-toggle", {
            bubbles: true,
            // Uten `composed` stopper hendelsen i en skyggerot, og en lytter
            // utenfor får aldri vite at dialogen ble lukket.
            composed: true,
            detail: { open, returnValue },
        }));
    }
    /**
     * Setter `open` tilbake på en dialog som fortsatt står i topplaget.
     *
     * `showModal()` setter attributtet selv, og serveren sendte det ikke, så
     * morfingen tar det. Nettleserens egen stil skjuler da en `<dialog>` uten
     * `open`, og dialogen forsvant for brukeren midt i noe hun holdt på med.
     * Før måtte malen skrive `data-preserve-attr="open"` for å hindre det.
     *
     * `:modal` og ikke `this.open` er vilkåret, og det er presist: lukker
     * brukeren dialogen, forlater den topplaget, og da er det manglende
     * attributtet ekte. Bare en dialog som fortsatt er modal uten å si det, er
     * en dialog noen har tatt attributtet fra.
     */
    repairOpen() {
        const dialog = this.dialogElement;
        if (!dialog?.isConnected || !dialog.matches(":modal") || dialog.open) {
            return;
        }
        // Attributtet må tilbake uansett: uten det gjør `close()` ingenting, og
        // dialogen ble stående i topplaget, usynlig, med resten av siden inert.
        setAttr(dialog, "open", "");
        if (!isServerControlled(this))
            return;
        /*
         * Serveren eier tilstanden, og patchen tok `open` fra en modal dialog.
         * Da er beskjeden at den skal lukkes, ikke settes tilbake. Uten dette
         * sto dialogen med `:modal` og `display: none`, og ingenting på siden
         * kunne klikkes. Verten følger med her, så `handleClose` har ingenting
         * igjen å gjøre og hendelsen kommer én gang.
         */
        dialog.close();
        // Hendelsen kommer uansett hvem som fjernet `open` på verten. Har React
        // alt tatt det, i sin rekkefølge med barn før forelder, er dette den ene
        // gangen noen kan melde fra: `sync()` og `handleClose` ser etterpå en
        // dialog som alt er lukket, og tier.
        if (this.open)
            this.removeAttribute("open");
        this.notify(false, dialog.returnValue);
    }
    sync() {
        const dialog = this.dialog;
        // `showModal()` kaster «The element is not in a Document» hvis dialogen
        // ikke står i siden. Det er ikke teoretisk: en morfer bygger serverens
        // utgave i et løsrevet tre før den sammenlignes, og en web
        // component tas i bruk der også. Uten denne linja kastet komponenten ved
        // hver eneste patch som rørte dialogen. Står den løsrevet nå, kjøres
        // `sync()` uansett på nytt når den kobles til.
        if (!dialog?.isConnected) {
            // Slipp taket i en dialog som er borte. Ellers ble observatøren og
            // lytteren hengende på en løsrevet node så lenge verten levde.
            if (!dialog) {
                this.openObserver?.disconnect();
                this.openObserver = undefined;
                this.dialogElement?.removeEventListener("close", this.handleClose);
                this.dialogElement = undefined;
            }
            warnAboutMarkup(this, "fant ingen <dialog> som direkte barn. Uten den kan ingenting " +
                "åpnes modalt, og innholdet står som en vanlig boks på siden.", 
            // Er komponenten løsrevet, byggefunksjon en morfer serverens utgave i et
            // eget tre, og da er det ingenting å si fra om. Et tomt element er
            // et område serveren ikke har fylt ennå.
            () => this.childElementCount > 0 && this.dialog === null);
            return;
        }
        this.wire(dialog);
        const first = dialog !== this.dialogElement;
        if (first) {
            this.dialogElement?.removeEventListener("close", this.handleClose);
            this.dialogElement = dialog;
            dialog.addEventListener("close", this.handleClose);
            this.openObserver?.disconnect();
            this.openObserver = new MutationObserver(() => this.repairOpen());
            this.openObserver.observe(dialog, { attributeFilter: ["open"] });
        }
        this.repairOpen();
        /*
         * Lukket brukeren dialogen før komponenten fikk kjøre?
         *
         * Serveren skriver `open` begge steder, så innholdet finnes uten
         * JavaScript, og `<form method="dialog">` lukker boksen uten JavaScript
         * også. Skjer det før skriptet er lastet, finnes det ingen lytter, og
         * første `sync()` ville sett en vert som sier «åpen» og en dialog som er
         * lukket, og åpnet den igjen. Dialogen spratt altså opp igjen rett etter
         * at brukeren hadde lukket den, og på mobil var vinduet stort nok til at
         * det skjedde hver gang.
         *
         * `returnValue` skiller de to tilfellene. Nettleseren setter den til
         * verdien på knappen som lukket dialogen, så en dialog som aldri har
         * vært åpnet har den tom. En mal som bare skrev `open` på verten, som
         * dokumentasjonen viste lenge, har den også tom, og skal åpnes som før.
         *
         * Det holder bare når knappen har en `value`. Lukkes dialogen med en
         * knapp uten verdi, eller med `close()` uten argument, etterlater
         * nettleseren ingenting å se etter, og dialogen åpner seg igjen slik den
         * gjorde før. Derfor har hver `<form method="dialog">` i dokumentasjonen
         * en `value`, og det er verdt å holde på.
         */
        if (first && this.open && !dialog.open && dialog.returnValue !== "") {
            this.removeAttribute("open");
            return;
        }
        // `:modal` og ikke `open`. De to er ikke det samme: et `<dialog open>`
        // i markupen, og en dialog som har vært flyttet i DOM-en mens den var
        // åpen, har `open` uten å være modal. Da er den en boks på siden, uten
        // fokusfelle og uten Escape, altså nøyaktig det komponenten finnes for å
        // hindre.
        const modal = dialog.matches(":modal");
        if (this.open && !modal) {
            /*
             * `showModal()` kaster `InvalidStateError` når `open` står der fra før
             * uten at dialogen er modal. Attributtet må altså bort først.
             *
             * Men ikke med `close()`. Den sender en ekte `close`-hendelse, og
             * siden serveren nå skriver `open` på `<dialog>` selv, ville den
             * hendelsen kommet ved hver eneste lasting av en dialog som er åpen.
             * En app som lytter på `close` rett på `<dialog>`, slik det komplette
             * eksempelet i dokumentasjonen gjør, fikk da en spøkelseslukking før
             * brukeren hadde sett dialogen. `removeAttribute` gir tillatelsen uten
             * hendelsen.
             */
            dialog.removeAttribute("open");
            dialog.showModal();
            this.notify(true);
        }
        else if (!this.open && (modal || dialog.open)) {
            /*
             * `modal` og ikke bare `dialog.open`: attributtet kan være borte mens
             * dialogen fortsatt står i topplaget.
             *
             * React er grunnen. Serveren skriver `open` på `<dialog>`, så React
             * eier attributtet, og React oppdaterer barn før forelder. Lukker
             * appen dialogen, fjerner React først `open` fra `<dialog>` og så fra
             * `<fs-dialog>`. Så vi kommer hit med `dialog.open` alt usann.
             *
             * `close()` gjør ingenting uten attributtet, så uten dette ble
             * dialogen stående i topplaget, usynlig, med resten av siden inert.
             * Brukeren satt igjen med en side der ingenting kunne klikkes, og
             * ingenting synlig som forklarte hvorfor.
             */
            setAttr(dialog, "open", "");
            dialog.close();
            this.notify(false, dialog.returnValue);
        }
    }
}
FsDialog.observedAttributes = ["open", SERVER_CONTROLLED];
export function defineFsDialog(tagName = FS_DIALOG_TAG) {
    defineElement(tagName, FsDialog);
}
//# sourceMappingURL=fs-dialog.js.map