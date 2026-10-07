/**
 * Grunnlaget web-komponentene arver fra, og registreringen av dem.
 *
 * `HTMLElement` og `customElements` finnes bare i nettleseren. En
 * `class X extends HTMLElement` blir evaluert i det modulen lastes, så en
 * server som bare ville hente `fs.button()` fra pakken stoppet med
 * «HTMLElement is not defined» før den kom så langt. Det rammet ni av
 * inngangspunktene, hovedinngangen inkludert, og gjorde pakken ubrukelig i
 * nettopp den situasjonen den er bygd for: at serveren skriver markupen.
 *
 * Reserven er en tom klasse. Den blir aldri brukt til noe, for uten
 * `customElements` blir ingenting registrert, og uten registrering lager
 * nettleseren aldri en forekomst. Den finnes bare for at modulen skal kunne
 * lastes.
 */
export const HostElement = typeof HTMLElement === "undefined"
    ? class {
    }
    : HTMLElement;
/**
 * Registrerer en web component, én gang, og bare i nettleseren.
 *
 * Kalles `defineFs*` fra en modul som også kjøres på serveren, som i
 * TanStack Start eller en React-app med server-rendring, skal det ikke skje
 * noe. Der finnes det ingen `customElements` å registrere i.
 */
export function defineElement(tagName, element) {
    if (typeof customElements === "undefined")
        return;
    if (!customElements.get(tagName))
        customElements.define(tagName, element);
}
/**
 * Setter et attributt, men bare når verdien faktisk er en annen.
 *
 * Komponentene observerer de attributtene de selv setter, for å kunne sette
 * dem tilbake etter en patch. En skriving av en verdi som alt står der teller
 * som en endring, så observatøren kaller seg selv, skriver på nytt, og
 * mikrotaskkøen tømmes aldri. Siden fryser, og en testkjøring henger uten
 * feilmelding: ingen stakksporing, ingen påstand, bare stillhet. Det er
 * derfor ingen test kan fange det, og hvorfor skrivingen går gjennom én
 * funksjon som alle komponentene bruker.
 *
 * `null` fjerner attributtet.
 */
export function setAttr(element, name, value) {
    if (value === null || value === undefined) {
        if (element.hasAttribute(name))
            element.removeAttribute(name);
    }
    else if (element.getAttribute(name) !== value) {
        element.setAttribute(name, value);
    }
}
/**
 * Slår et boolsk attributt av eller på, uten å skrive når det står slik alt.
 *
 * `el.hidden = true` på noe som alt er skjult skriver attributtet på nytt, og
 * det teller som en endring. `toggleAttribute` gjør ikke det: står
 * attributtet slik det skal, skjer ingenting. Den er derfor den eneste
 * lovlige veien til `hidden`, `disabled` og de andre boolske attributtene i en
 * komponent som observerer sine egne.
 *
 * `Boolean(on)` og ikke `on` alene: med `undefined` som andre argument
 * veksler `toggleAttribute` i stedet for å fjerne. React 19 skriver
 * `el.open = undefined` når en prop går fra `true` til borte, og setteren
 * satte da attributtet tilbake: en dialog brukeren hadde lukket, åpnet seg
 * igjen i det appen fulgte etter.
 */
export function setFlag(element, name, on) {
    element.toggleAttribute(name, Boolean(on));
}
/**
 * Språket nærmest elementet, eller `undefined` når siden ikke sier noe.
 *
 * `undefined` gir `Intl` nettleserens eget språk. Et `lang` som ikke er en
 * gyldig språkkode får `Intl` til å kaste, og da er nettleserens språk
 * bedre enn ingen tekst.
 */
export function languageOf(element) {
    /*
     * `closest()` stopper ved en skyggerot. Står elementet i en, som hver
     * forhåndsvisning i dokumentasjonen, fortsetter letingen fra verten, ellers
     * fant den aldri `lang` på `<html>`.
     */
    let node = element;
    while (node) {
        const owner = node.closest("[lang]");
        // `lang=""` betyr «ukjent språk» og stopper arven fra foreldrene.
        if (owner) {
            const lang = owner.getAttribute("lang")?.trim();
            if (!lang)
                return undefined;
            try {
                Intl.getCanonicalLocales(lang);
                return lang;
            }
            catch {
                return undefined;
            }
        }
        const root = node.getRootNode();
        node = root instanceof ShadowRoot ? root.host : null;
    }
    return undefined;
}
/**
 * Skriver tekst i et element, men bare når teksten er en annen.
 *
 * `textContent` er en `childList`-endring, og komponentene observerer barna
 * sine. Å skrive den samme teksten på nytt teller som en endring, og da
 * kaller observatøren seg selv.
 *
 * `textContent` er all tekst i undertreet slått sammen, så sammenligningen
 * ser ikke forskjell på `<b>3</b> treff` og `3 treff`. Funksjonen er ment for
 * elementer komponenten fyller med ren tekst, som meldingen om antall treff i
 * forslagsfeltet.
 */
export function setText(element, text) {
    if (element.textContent !== text)
        element.textContent = text;
}
/** Legger på en klasse, men bare når den ikke står der fra før. */
export function addClass(element, name) {
    if (!element.classList.contains(name))
        element.classList.add(name);
}
/** En id komponenten lager selv, når markupen ikke hadde noen. */
export function uniqueId(prefix) {
    return `${prefix}-${Math.random().toString(36).slice(2, 9)}`;
}
/**
 * Delene med en rolle, og de uten bare når komponenten satte rollene selv.
 *
 * Komponentene fyller inn roller der markupen kom uten: en knapp i raden
 * blir en fane, en `<li>` i lista blir et alternativ. Men har serveren
 * skrevet rollene, er et element uten rolle noe annet, som en «lukk»-knapp
 * i raden, og skal ikke få rolle, id og `tabindex="-1"` av komponenten.
 * `roledByHost` er rollene komponenten selv satte: er hver rolle som finnes
 * komponentens, er markupen bar, og et element uten rolle som serveren
 * sender inn senere er en ny del. Delene avgrenses til verten, siden
 * komponenter i komponenter er vanlig markup.
 */
export function derivedParts(host, selector, role, roledByHost) {
    const own = [...host.querySelectorAll(selector)].filter((part) => part.closest(host.localName) === host);
    const bare = own
        .filter((part) => part.getAttribute("role") === role)
        .every((part) => roledByHost.has(part));
    return bare ? own : own.filter((part) => part.getAttribute("role") === role);
}
/**
 * Attributtet som gir serveren tilstanden tilbake.
 *
 * Komponentene reparerer som standard det brukeren har gjort: valgte fanen,
 * det åpne sprettoppvinduet, den utvidede forslagslista. En morfing river det
 * bort, siden ingenting av det sto i HTML-en serveren sendte, og komponenten
 * setter det tilbake. Da trenger ingen mal å kjenne til attributtene.
 *
 * Noen ganger er det serveren som skal bestemme: «gå videre til steg 2» er en
 * ekte ting en server vil kunne gjøre. Står `server-controlled` på verten,
 * reparerer komponenten ingenting, og hver patch bestemmer.
 *
 * Dette erstatter `data-preserve-attr`, som krevde at malen skrev av navnene
 * på hvert attributt komponenten kom til å røre. Ett attributt å huske i
 * stedet for ni, og standardvalget er det som er riktig nesten alltid.
 *
 * Fredningen er dessuten morferens egen mekanisme: `data-preserve-attr` er et
 * `data-*`-attributt, og et slikt attributt betyr bare noe for koden som leser
 * det. Reparasjon leser markupen, og virker uansett hvem som morfer.
 */
export const SERVER_CONTROLLED = "server-controlled";
/** Om serveren eier tilstanden i dette elementet. */
export function isServerControlled(element) {
    return element.hasAttribute(SERVER_CONTROLLED);
}
/**
 * Sier fra når markupen en komponent fikk, ikke henger sammen.
 *
 * En komponent som ikke finner delene sine kan ikke gjøre jobben, og det
 * eneste alternativet til en beskjed er stillhet. Det var stillhet lenge:
 * en mal uten en kontroll i `<fs-field>` ga et felt uten kobling, og
 * ingenting sa fra før noen leste siden med skjermleser.
 *
 * Beskjeden kommer én gang per element og melding. En komponent
 * synkroniserer seg ved hver patch, og en advarsel per patch ville fylt
 * konsollen så fort at den ble ubrukelig. Meldingen må derfor være den
 * samme hver gang: interpolerer du et tall inn i den, er hver runde en ny
 * melding, og dedupliseringen er borte.
 *
 * `stillBroken` er grunnen til at advarselen kan stoles på. Den kalles på
 * nytt når siden har falt til ro, og bare da skrives noe ut. En komponent
 * ser markupen sin i det den kobles til, og der er den ofte halvferdig:
 * HTML som strømmer fra en Go- eller Kotlin-server leveres i pakker, og et
 * brudd midt mellom ledeteksten og feltet er helt vanlig. Uten den siste
 * sjekken advarte komponenten om markup som var i orden et øyeblikk senere,
 * og en advarsel som også kommer på riktig markup blir slått av.
 *
 * Den kommer i alle miljøer, ikke bare under utvikling. Pakken har ingen
 * byggetrinn som kunne fjernet den, og en advarsel som bare utløses av
 * markup som faktisk er gal, hører hjemme der den skjer.
 */
const reported = new WeakMap();
const queued = new WeakMap();
export function warnAboutMarkup(element, message, stillBroken) {
    if (typeof console === "undefined")
        return;
    if (reported.get(element)?.has(message))
        return;
    let pending = queued.get(element);
    if (!pending) {
        pending = new Set();
        queued.set(element, pending);
    }
    if (pending.has(message))
        return;
    pending.add(message);
    whenSettled(() => {
        pending.delete(message);
        if (!element.isConnected || !stillBroken())
            return;
        let said = reported.get(element);
        if (!said) {
            said = new Set();
            reported.set(element, said);
        }
        if (said.has(message))
            return;
        said.add(message);
        console.warn(`${element.tagName.toLowerCase()}: ${message}`, element);
    });
}
/**
 * Kjører etter at siden har falt til ro.
 *
 * Først må parseren være ferdig, ellers ser vi bare den delen av markupen
 * som har rukket å komme. Så to tegninger, slik at en morfing, en
 * React-hydrering eller et skript som fyller elementet i neste oppgave
 * rekker å bli ferdig først.
 */
function whenSettled(run) {
    let done = false;
    const once = () => {
        if (done)
            return;
        done = true;
        run();
    };
    const afterTwoFrames = () => {
        if (typeof requestAnimationFrame === "undefined") {
            once();
            return;
        }
        /*
         * En reserve, fordi `requestAnimationFrame` ikke alltid fyrer i en ramme
         * som ikke tegnes. Testet i en `<iframe>` med `display: none`: Chromium
         * og WebKit fyrer likevel, Firefox gjør det ikke. Uten reserven ble
         * meldingen stående i køen for alltid der, og sperret en senere, ekte
         * advarsel om det samme.
         *
         * Den avlyses ikke. Et forsøk på å avlyse den i den første rammen var
         * verre enn problemet: fyrer første ramme og fanen blir skjult før den
         * andre, er både reserven borte og den andre rammen borte, og da kjører
         * sjekken aldri. `once` er engangs, så det gjør ingenting at begge veier
         * er i gang samtidig, og den som kommer først vinner.
         *
         * Et sekund er valgt så to rammer vinner i alt annet enn en side som står
         * helt stille. Skjer det, er en advarsel om markup som fortsatt er
         * halvferdig et lite problem ved siden av at siden er frosset.
         */
        setTimeout(once, 1000);
        requestAnimationFrame(() => requestAnimationFrame(once));
    };
    if (typeof document !== "undefined" && document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", afterTwoFrames, {
            once: true,
        });
        return;
    }
    afterTwoFrames();
}
/**
 * Venter til elementet er oppgradert til komponenten sin, og gir det tilbake
 * med riktig type.
 *
 * De frittstående komponentene har et imperativt API: `show()`,
 * `reportFailure()`, `extend()`. Kalles en av dem før `defineFs*` har kjørt,
 * er elementet fortsatt et vanlig `HTMLElement`, og kallet feiler med
 * «reportSuccess is not a function». I drift skjedde det i gapet mellom at
 * serverens HTML sto der og at registreringen var lastet: den første
 * meldingen fra hendelsesstrømmen kom inn i det gapet, og appen la en
 * `typeof`-vakt rundt hvert kall for å komme rundt det.
 *
 * Hjelperne rundt metodene venter i stedet på `whenDefined` for elementets
 * eget tagnavn, så et kall som kommer for tidlig blir gjort i det
 * registreringen er der. `upgrade()` tar med et element som ennå ikke står i
 * dokumentet; et som står der, er alt oppgradert når `whenDefined` løses.
 *
 * Et element ingen registrerer ville ellers ventet i stillhet for alltid.
 * Etter tre sekunder sier hjelperen fra i konsollen, én gang per tagnavn,
 * og venter videre: kommer registreringen sent, skal kallet fortsatt gjøres.
 */
const UPGRADE_WARNING_MS = 3000;
const warnedTags = new Set();
export async function whenUpgraded(element) {
    const tag = element.localName;
    if (!customElements.get(tag)) {
        const timer = setTimeout(() => {
            if (customElements.get(tag) || warnedTags.has(tag))
                return;
            warnedTags.add(tag);
            console.warn(`<${tag}> er ikke registrert etter ${UPGRADE_WARNING_MS / 1000} sekunder. ` +
                `Kall defineFs() fra @fristil/designsystem/register ved oppstart, ikke i en effekt.`, element);
        }, UPGRADE_WARNING_MS);
        // `finally`: `whenDefined` avviser et navn uten bindestrek, og uten
        // opprydning ville timeren advart om «ikke registrert» tre sekunder
        // etter at kallet alt hadde feilet med den riktige feilen.
        try {
            await customElements.whenDefined(tag);
        }
        finally {
            clearTimeout(timer);
        }
    }
    customElements.upgrade(element);
    return element;
}
//# sourceMappingURL=host-element.js.map