/**
 * The base the web components inherit from, and their registration.
 *
 * `HTMLElement` and `customElements` only exist in the browser. A
 * `class X extends HTMLElement` is evaluated when the module loads, so a
 * server that only wanted `lt.button()` from the package stopped with
 * "HTMLElement is not defined" before it got that far. That hit nine of the
 * entry points, the main entry included, and made the package useless in
 * exactly the situation it is built for: the server writing the markup.
 *
 * The fallback is an empty class. It is never used for anything, because
 * without `customElements` nothing is registered, and without registration
 * the browser never creates an instance. It only exists so the module can
 * load.
 */
export const HostElement = typeof HTMLElement === "undefined"
    ? class {
    }
    : HTMLElement;
/**
 * Registers a web component, once, and only in the browser.
 *
 * When `defineLt*` is called from a module that also runs on the server, as
 * in TanStack Start or a server-rendered React app, nothing happens. There is
 * no `customElements` there to register in.
 *
 * A constructor can only be in the registry once. If the class is already
 * there under another name, a subclass is registered:
 * `defineLtToast("app-toast")` followed by `defineLt()` otherwise threw
 * NotSupportedError on `lt-toast`, and the components after it in the list
 * were never registered. With the subclass both names work, and `instanceof`
 * holds for both.
 */
export function defineElement(tagName, element) {
    if (typeof customElements === "undefined")
        return;
    if (customElements.get(tagName))
        return;
    if (isRegistered(element)) {
        customElements.define(tagName, class extends element {
        });
        return;
    }
    try {
        customElements.define(tagName, element);
    }
    catch (error) {
        // The fallback for browsers without `getName`: the name is free, so a
        // NotSupportedError here means the constructor is already in use.
        if (!(error instanceof DOMException && error.name === "NotSupportedError"))
            throw error;
        customElements.define(tagName, class extends element {
        });
    }
}
/**
 * Whether the constructor is already in the registry, under any name.
 * `getName` arrived in Chrome 117, Firefox 116 and Safari 17; without it the
 * answer is `false`, and `define` reports the problem itself.
 */
function isRegistered(element) {
    if (typeof customElements.getName !== "function")
        return false;
    return customElements.getName(element) !== null;
}
/**
 * Sets an attribute, but only when the value actually differs.
 *
 * The components observe the attributes they set themselves, so they can set
 * them back after a patch. Writing a value that is already there counts as a
 * change, so the observer calls itself, writes again, and the microtask queue
 * never empties. The page freezes, and a test run hangs without an error: no
 * stack trace, no assertion, just silence. That is why no test can catch it,
 * and why writes go through one function all the components use.
 *
 * `null` removes the attribute.
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
 * Turns a boolean attribute on or off, without writing when it is already so.
 *
 * `el.hidden = true` on something already hidden writes the attribute again,
 * and that counts as a change. `toggleAttribute` does not: if the attribute
 * is already as it should be, nothing happens. So it is the only permitted
 * way to `hidden`, `disabled` and the other boolean attributes in a component
 * that observes the attributes it writes.
 *
 * `Boolean(on)` and not `on` alone: with `undefined` as the second argument
 * `toggleAttribute` toggles instead of removing. React 19 writes
 * `el.open = undefined` when a prop goes from `true` to absent, and the
 * setter then put the attribute back: a dialog the user had closed opened
 * again as the app followed along.
 */
export function setFlag(element, name, on) {
    element.toggleAttribute(name, Boolean(on));
}
/**
 * The language nearest the element, or `undefined` when the page does not
 * say.
 *
 * `undefined` gives `Intl` the browser's language. A `lang` that is not a
 * valid language tag makes `Intl` throw, and then the browser's language is
 * better than no text.
 */
export function languageOf(element) {
    /*
     * `closest()` stops at a shadow root. If the element is in one, as in
     * every preview in the documentation, the search continues from the host;
     * otherwise it never found `lang` on `<html>`.
     */
    let node = element;
    while (node) {
        const owner = node.closest("[lang]");
        // `lang=""` means "unknown language" and stops inheritance from the ancestors.
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
 * Writes text in an element, but only when the text differs.
 *
 * `textContent` is a `childList` change, and the components observe their
 * children. Writing the same text again counts as a change, and then the
 * observer calls itself.
 *
 * `textContent` is all the text in the subtree joined, so the comparison does
 * not tell `<b>3</b> matches` from `3 matches`. The function is meant for
 * elements the component fills with plain text, such as the match count in
 * the suggestion field.
 */
export function setText(element, text) {
    if (element.textContent !== text)
        element.textContent = text;
}
/** Adds a class, but only when it is not already there. */
export function addClass(element, name) {
    if (!element.classList.contains(name))
        element.classList.add(name);
}
/** An id the component creates itself, when the markup had none. */
export function uniqueId(prefix) {
    return `${prefix}-${Math.random().toString(36).slice(2, 9)}`;
}
/**
 * The parts with a role, and those without only when the component set the
 * roles itself.
 *
 * The components fill in roles where the markup came without them: a button
 * in the row becomes a tab, an `<li>` in the list becomes an option. But if
 * the server wrote the roles, an element without a role is something else,
 * such as a "close" button in the row, and must not get a role, an id and
 * `tabindex="-1"` from the component. `roledByHost` holds the roles the
 * component set itself: if every role present was set by the component, the
 * markup is bare, and an element without a role that the server sends later
 * is a new part. The parts are limited to the host, since components inside
 * components are ordinary markup.
 */
export function derivedParts(host, selector, role, roledByHost) {
    const own = [...host.querySelectorAll(selector)].filter((part) => part.closest(host.localName) === host);
    const bare = own
        .filter((part) => part.getAttribute("role") === role)
        .every((part) => roledByHost.has(part));
    return bare ? own : own.filter((part) => part.getAttribute("role") === role);
}
/**
 * The attribute that gives the state back to the server.
 *
 * By default the components repair what the user has done: the selected tab,
 * the open popover, the expanded suggestion list. A morph tears it away,
 * since none of it was in the HTML the server sent, and the component sets it
 * back. Then no template needs to know the attributes.
 *
 * Sometimes the server should decide: "go on to step 2" is a real thing a
 * server wants to be able to do. With `server-controlled` on the host, the
 * component does not set back what the user did, and each patch decides. It
 * still does the wiring and the keyboard, and what the attribute means for
 * each component is documented in the component.
 *
 * This replaces `data-preserve-attr`, which required the template to copy the
 * name of every attribute the component would touch. One attribute to
 * remember instead of nine, and the default is the right choice almost
 * always.
 *
 * Preserving is also a mechanism of one particular morpher:
 * `data-preserve-attr` is a `data-*` attribute, and such an attribute only
 * means something to the code that reads it. Repair reads the markup, and
 * works whoever does the morphing.
 */
export const SERVER_CONTROLLED = "server-controlled";
/** Whether the server decides the state in this element. */
export function isServerControlled(element) {
    return element.hasAttribute(SERVER_CONTROLLED);
}
/**
 * Warns when the markup a component received does not fit together.
 *
 * A component that cannot find its parts cannot do its job, and the only
 * alternative to a message is silence. It was silent for a long time: a
 * template without a control in `<lt-field>` gave a field without wiring,
 * and nothing said so until someone read the page with a screen reader.
 *
 * The message comes once per element and message. A component syncs on every
 * patch, and a warning per patch would fill the console so fast it became
 * useless. So the message must be the same every time: interpolate a number
 * into it and every round is a new message, and the deduplication is gone.
 *
 * `stillBroken` is why the warning can be trusted. It is called again when
 * the page has settled, and only then is anything written. A component sees
 * its markup as it connects, and the markup is often unfinished then: HTML
 * streamed from a Go or Kotlin server arrives in chunks, and a break between
 * the label and the field is perfectly normal. Without the final check the
 * component warned about markup that was fine a moment later, and a warning
 * that also fires on correct markup gets switched off.
 *
 * It comes in every environment, not only during development. The package
 * has no build step that could remove it, and a warning that only fires on
 * markup that really is wrong belongs where it happens.
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
 * Runs once the page has settled.
 *
 * First the parser must be done, or we only see the part of the markup that
 * has arrived so far. Then two frames, so a morph, a React hydration or a
 * script that fills the element in the next task has time to finish first.
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
         * A fallback, because `requestAnimationFrame` does not always fire in a
         * frame that is not painted. Tested in an `<iframe>` with
         * `display: none`: Chromium and WebKit fire anyway, Firefox does not.
         * Without the fallback the message stayed in the queue there for ever,
         * and blocked a later, real warning about the same thing.
         *
         * It is not cancelled. An attempt to cancel it in the first frame was
         * worse than the problem: if the first frame fires and the tab is hidden
         * before the second, both the fallback and the second frame are gone,
         * and the check never runs. `once` runs only once, so it does no harm
         * that both paths are running, and whichever comes first wins.
         *
         * One second is chosen so two frames win in anything but a page that is
         * completely stalled. If that happens, a warning about markup that is
         * still unfinished is a small problem next to a frozen page.
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
 * Waits until the element has been upgraded to its component, and returns it
 * with the right type.
 *
 * The components with an imperative API have `show()`, `reportFailure()` and
 * `extend()`. If one of them is called before `defineLt*` has run, the
 * element is still a plain `HTMLElement`, and the call fails with
 * "reportSuccess is not a function". In production it happened in the gap
 * between the server's HTML being there and the registration being loaded:
 * the first message from the event stream arrived in that gap, and the app
 * put a `typeof` guard around every call to get around it.
 *
 * The helpers around the methods instead wait for `whenDefined` on the
 * element's actual tag name, so a call that comes too early is made as soon
 * as the registration is there. `upgrade()` covers an element not yet in the
 * document; one that is in it is already upgraded when `whenDefined`
 * resolves.
 *
 * An element nobody registers would otherwise wait silently for ever. After
 * three seconds the helper says so in the console, once per tag name, and
 * keeps waiting: if the registration comes late, the call should still be
 * made.
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
            console.warn(`<${tag}> is not registered after ${UPGRADE_WARNING_MS / 1000} seconds. ` +
                `Call defineLt() from @lo-tek/interfaces/register at startup, not in an effect.`, element);
        }, UPGRADE_WARNING_MS);
        // `finally`: `whenDefined` rejects a name without a hyphen, and without
        // the cleanup the timer would have warned "not registered" three seconds
        // after the call had already failed with the right error.
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