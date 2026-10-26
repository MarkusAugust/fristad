import { LABEL_CLASS } from "../../css/label/label.js";
import { DEFAULT_TEXTS } from "../../default-texts.js";
import { addClass, defineElement, derivedParts, HostElement, isServerControlled, languageOf, SERVER_CONTROLLED, setAttr, setFlag, setText, uniqueId, warnAboutMarkup, } from "../../host-element.js";
import { SUGGESTION_EMPTY_CLASS, SUGGESTION_LIST_CLASS, SUGGESTION_OPTION_CLASS, SUGGESTION_VALUE_CLASS, } from "./suggestion.js";
/**
 * The texts for the number of matches, one attribute per form. `count-none`
 * is zero matches, and the rest are the forms `Intl.PluralRules` knows.
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
export const LT_SUGGESTION_TAG = "lt-suggestion";
/**
 * The parts the suggestion field finds, as selectors relative to the element.
 * The web component finds its parts with these, and the manifest describes
 * them for whoever writes the markup by hand, so the two cannot disagree.
 */
export const LT_SUGGESTION_PARTS = {
    /** The field: the role when present, otherwise the first input. */
    control: "[role='combobox'], input:not([type='hidden'])",
    /** The list: the role when present, otherwise the class. */
    list: `[role='listbox'], .${SUGGESTION_LIST_CLASS}`,
    /** An option: the role when present, otherwise an `<li>` without a role in the list. */
    option: `[role='option'], .${SUGGESTION_LIST_CLASS} > li:not([role])`,
    empty: `.${SUGGESTION_EMPTY_CLASS}`,
    status: "[role='status']",
    /** The hidden field that receives the code of the option the user chose. */
    value: `input.${SUGGESTION_VALUE_CLASS}`,
};
const CONTROL_SELECTOR = LT_SUGGESTION_PARTS.control;
const LIST_SELECTOR = LT_SUGGESTION_PARTS.list;
const OPTION_SELECTOR = LT_SUGGESTION_PARTS.option;
/**
 * The keyboard and the filtering in a field with a suggestion list.
 *
 * The server writes the field and the list. When the markup is made with
 * JavaScript, `lt.suggestion()` writes the roles, the ids and the wiring.
 * When it comes from a template without JavaScript, the template writes
 * `for` on the label and `id` on the field, or wraps the field in the label,
 * which HTML requires for the field to have a name without JavaScript. The
 * rest is enough: a `.lt-suggestion__list` with `<li>` and a
 * `[role="status"]`. The web component sets the roles, gives the list and the
 * options ids, connects the list to the field, and closes the list until the
 * user touches the field. Only what is missing, so what the server wrote
 * stays.
 *
 * The web component never writes an id on the field. Datastar and idiomorph
 * recognise a node by its id, and a field with an id the server did not send
 * is replaced on the next update, along with focus and what the user has
 * typed.
 *
 * The field always shows the option's text. If the server has written a
 * hidden `.lt-suggestion__value`, it receives the code from `data-value`, or
 * the text when the option has none. Without JavaScript the text is
 * submitted, and the server looks it up. The web component renders nothing.
 * It used to create the whole field itself, and then there was neither a
 * label nor an input before the script had run, and nothing was included in
 * the submission.
 *
 * What the web component does is what the browser does not: it filters the
 * options while the user types, moves the highlight with the arrow keys, and
 * keeps `aria-activedescendant` in sync so the screen reader announces the
 * option without focus leaving the field.
 *
 * If something else has already filtered, a server or a React component that
 * renders only the matches, `prefiltered` says so, and the web component
 * leaves the options alone.
 *
 * ```html
 * <lt-suggestion>
 *   <label class="lt-label" for="city">City</label>
 *   <input class="lt-input" id="city" role="combobox" aria-controls="city-list"
 *          aria-expanded="false" aria-autocomplete="list" autocomplete="off"
 *          aria-describedby="city-status">
 *   <ul class="lt-suggestion__list" id="city-list" role="listbox" hidden>
 *     <li class="lt-suggestion__option" id="city-option-0" role="option">Bergen</li>
 *   </ul>
 *   <p class="lt-suggestion__empty">No matches</p>
 *   <span class="lt-sr-only" id="city-status" role="status"></span>
 * </lt-suggestion>
 * ```
 */
export class LtSuggestion extends HostElement {
    constructor() {
        super(...arguments);
        /** The options that already have their listener. */
        this.bound = new WeakSet();
        /** The ids the web component created, per node, so a patch gets the same ones back. */
        this.ids = new WeakMap();
        /** The roles the web component set. See `derivedParts`. */
        this.roledByMe = new WeakSet();
        /** True while the web component dispatches events, so it does not answer itself. */
        this.choosing = false;
        /** The `for` the web component wrote on the label. */
        this.forByMe = new WeakMap();
        /**
         * The name the web component gave the list, per list: `aria-labelledby`
         * pointing to the label, or `aria-label` with its text. If something else
         * is there, the server wrote it, and it stays.
         */
        this.listNameByMe = new WeakMap();
        /**
         * Labels where the server wrote a `for` pointing to another id. Reported
         * once; the memory is not reset, since the rewritten `for` cannot be told
         * apart from a server that corrected itself.
         */
        this.wrongFor = new WeakSet();
        /**
         * Places the list right below the field, as wide as it. If there is no
         * room below, it goes above, and it is squeezed into the viewport either
         * way, like the panel in `<lt-popover>`. The position is physical,
         * computed from the field's box.
         */
        this.reposition = () => {
            const list = this.listElement;
            const control = this.querySelector(CONTROL_SELECTOR);
            if (!list || !control || !list.matches(":popover-open"))
                return;
            const field = control.getBoundingClientRect();
            const height = list.getBoundingClientRect().height;
            const space = 4;
            let top = field.bottom + space;
            if (top + height > window.innerHeight && field.top - height - space >= 0) {
                top = field.top - height - space;
            }
            top = Math.max(space, Math.min(top, window.innerHeight - height - space));
            list.style.setProperty("--lt-suggestion-top", `${Math.round(top)}px`);
            list.style.setProperty("--lt-suggestion-left", `${Math.round(field.left)}px`);
            list.style.setProperty("--lt-suggestion-width", `${Math.round(field.width)}px`);
        };
        /**
         * The form is reset. The browser sets the text back to the server's after
         * the event has been dispatched, so the code is set in the next turn, after
         * the reset is done. The text there then decides: if it is the server's,
         * the server's code applies; if it is the chosen one, the code stays;
         * otherwise the code of the option with that text, otherwise none. The
         * server's comes first because two options can have the same name: if the
         * user chose the second Våler, the reset goes back to the server's. The
         * server may also have sent the chosen one back, and then it is as much the
         * server's word as the user's.
         */
        this.handleReset = () => {
            setTimeout(() => {
                const valueField = this.valueField;
                if (!valueField || !this.control)
                    return;
                const text = this.control.value;
                const option = this.options.find((o) => (o.textContent ?? "").trim() === text);
                const code = this.serverChoice && text === this.serverChoice.label
                    ? this.serverChoice.value
                    : this.chosen && text === this.chosen.label
                        ? this.chosen.value
                        : option
                            ? (option.getAttribute("data-value") ?? text)
                            : "";
                this.writeCode(valueField, code);
                this.chosen = code !== "" ? { label: text, value: code } : undefined;
            }, 0);
        };
        this.handleInput = () => {
            if (this.choosing)
                return;
            // The user changed the text after a choice. The code no longer applies.
            // If the user types the chosen text again, after an update emptied both,
            // the code comes back at once.
            const valueField = this.valueField;
            if (valueField && this.chosen) {
                if (this.control?.value !== this.chosen.label) {
                    this.writeCode(valueField, "");
                    this.chosen = undefined;
                }
                else if (valueField.value === "") {
                    this.writeCode(valueField, this.chosen.value);
                }
            }
            if (this.locked)
                return;
            this.filter();
            this.setOpen(true);
        };
        this.handleFocus = () => {
            if (this.locked)
                return;
            this.filter();
            this.setOpen(true);
        };
        this.handleFocusOut = (event) => {
            // If focus goes to something inside the web component, such as a button
            // next to the field, the list stays.
            const next = event.relatedTarget;
            if (next instanceof Node && this.contains(next))
                return;
            this.setOpen(false);
        };
        this.handleKeydown = (event) => {
            // During IME composition the keys belong to the composition. Enter
            // confirms the characters, and must not choose the highlighted option.
            if (event.isComposing || event.keyCode === 229)
                return;
            if (this.locked)
                return;
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
                // The list must open first, as with arrow down. Without that
                // `aria-activedescendant` pointed to an option in a list the field at the
                // same time reported as closed, and the screen reader announced something
                // that was not on screen.
                if (!open) {
                    this.filter();
                    this.setOpen(true);
                }
                // If nothing is highlighted, arrow up goes to the last one.
                // `activeIndex - 1` would have given the second to last, since nothing
                // highlighted is -1.
                this.markActive(this.activeIndex < 0 ? -1 : this.activeIndex - 1);
            }
            else if ((event.key === "PageUp" || event.key === "PageDown") &&
                open &&
                this.selectable.length > 0) {
                // Home and End move the caret in the text, and are left alone. PgUp and
                // PgDn go to the first and last option.
                event.preventDefault();
                this.markActive(event.key === "PageUp" ? 0 : this.selectable.length - 1);
            }
            else if (event.key === "Enter" && open && this.activeIndex >= 0) {
                event.preventDefault();
                const chosen = this.selectable[this.activeIndex];
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
            // Keeps focus in the field. Without this the field lost focus before
            // `click`, and `focusout` closed the list before the choice could happen.
            event.preventDefault();
        };
        this.handleOptionClick = (event) => {
            // The choice happens on `click`, which assistive technology also sends
            // when it activates an element. It used to happen on `mousedown`, and a
            // `click` alone chose nothing.
            this.choose(event.currentTarget);
        };
    }
    connectedCallback() {
        /*
         * The attributes are included, not just the children. Every write below
         * compares first, or the observer would trigger itself forever.
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
                // The wiring the web component fills in when the markup arrived without it.
                "role",
                "id",
                "for",
                "aria-controls",
                "autocomplete",
                "aria-autocomplete",
                "class",
                // The value field's `value` is an attribute, and a morph can write
                // the server's empty value over the code the user chose.
                "value",
                // `form` on the field picks which form the reset belongs to.
                "form",
                // A field that becomes `readonly` or `disabled` must close the list.
                "readonly",
                "disabled",
                "aria-disabled",
                // The list sits in the top layer. A morph that removes `popover`
                // closes the popover, and the web component sets it back.
                "popover",
                // The list's name, from the label.
                "aria-labelledby",
                "aria-label",
            ],
        });
        this.sync();
        window.addEventListener("resize", this.reposition);
        window.addEventListener("scroll", this.reposition, true);
        // The list closes when focus leaves the web component, with Tab as with
        // anything else. On the host and not on the field: if focus goes through
        // a button inside the web component and then out, the list must close
        // then too. Without this it stayed over the next field, with
        // `aria-expanded` true on a field without focus.
        this.addEventListener("focusout", this.handleFocusOut);
    }
    attributeChangedCallback(name) {
        if (name === SERVER_CONTROLLED && isServerControlled(this)) {
            this.wantOpen = undefined;
            this.active = undefined;
        }
        /*
         * The attribute can come and go while the page is alive, and then the
         * list must be corrected at once rather than on the next keystroke.
         * `this.observer` signals that the web component is in the document: the
         * callback also fires during the upgrade, before the children exist.
         */
        if (name === "prefiltered" && this.observer) {
            // The count present when the attribute is set is the starting point,
            // and nothing to announce. Without this the first change afterwards
            // was taken as the starting point, and was lost.
            this.announcedCount = this.prefiltered ? this.visible.length : undefined;
            this.filter();
        }
    }
    sync() {
        this.wire();
        this.bind();
        this.repair();
        this.announceFiltered();
        this.mirrorPopover();
    }
    /**
     * Opens and closes the popover the list sits in, following `hidden`.
     *
     * `hidden` still says whether the list is open, from the server as from
     * the web component, and the popover follows. The list sits in the top
     * layer, so a parent with `overflow`, such as the body of a dialog, does
     * not clip it.
     */
    mirrorPopover() {
        const list = this.listElement;
        if (!list?.isConnected || !list.hasAttribute("popover"))
            return;
        const shown = list.matches(":popover-open");
        try {
            if (!list.hidden && !shown)
                list.showPopover();
            else if (list.hidden && shown)
                list.hidePopover();
        }
        catch {
            // An element on its way out of the tree cannot be shown. The next
            // round sees it again.
            return;
        }
        if (!list.hidden)
            this.reposition();
    }
    /**
     * Names the list from the label, when the markup arrived without a name.
     *
     * With `aria-labelledby` when the label is next to the field; the web
     * component gives it an id when it has none, since it holds nothing the
     * user can lose. If the label wraps the field, it gets no id: a morph would
     * then replace the label, and the field with it. In that case the list
     * gets the label's text as `aria-label`, without the field's value.
     */
    nameList(list, control) {
        const mine = this.listNameByMe.get(list);
        const labelledby = list.getAttribute("aria-labelledby");
        const label = list.getAttribute("aria-label");
        const fromServer = (labelledby !== null &&
            !(mine?.name === "aria-labelledby" && mine.value === labelledby)) ||
            (label !== null && !(mine?.name === "aria-label" && mine.value === label));
        if (fromServer)
            return;
        const labelElement = control.labels?.[0] ?? this.querySelector("label");
        if (!labelElement)
            return;
        if (labelElement.contains(control)) {
            let text = "";
            for (const node of labelElement.childNodes)
                if (node !== control &&
                    !(node instanceof Element && node.contains(control)))
                    text += node.textContent ?? "";
            text = text.replace(/\s+/g, " ").trim();
            if (labelledby !== null)
                setAttr(list, "aria-labelledby", null);
            if (text === "") {
                if (label !== null)
                    setAttr(list, "aria-label", null);
                this.listNameByMe.delete(list);
                return;
            }
            setAttr(list, "aria-label", text);
            this.listNameByMe.set(list, { name: "aria-label", value: text });
            return;
        }
        if (!labelElement.id) {
            setAttr(labelElement, "id", this.rememberedId(labelElement, "lt-suggestion-label"));
        }
        if (label !== null)
            setAttr(list, "aria-label", null);
        setAttr(list, "aria-labelledby", labelElement.id);
        this.listNameByMe.set(list, {
            name: "aria-labelledby",
            value: labelElement.id,
        });
    }
    /** The id an element should have: the one it has, otherwise the one the web component created last. */
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
     * Fills in the wiring where the markup arrived without it. Only what is
     * missing.
     *
     * The list is closed when the markup says nothing about it, that is when
     * neither `aria-expanded` on the field nor `hidden` on the list is there.
     * If the markup states one of them, that is the server's word on both, as
     * with `lt.suggestion({ open: true })`.
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
        if (label) {
            addClass(label, LABEL_CLASS);
            // `for` and `id` are the same information, and are written from the
            // field's id, never the other way round: the web component does not
            // write an id on the field. If the label wraps the field, the wiring
            // is implicit. Same rule as in `<lt-field>`.
            if (control.id && !label.contains(control)) {
                const current = label.getAttribute("for");
                if (current !== null &&
                    current !== control.id &&
                    current !== this.forByMe.get(label)) {
                    this.wrongFor.add(label);
                }
                setAttr(label, "for", control.id);
                this.forByMe.set(label, control.id);
            }
            warnAboutMarkup(this, "for on the label pointed to a different id than the field's. " +
                "The component rewrote it, but without JavaScript the field has " +
                "no label and a screen reader announces it without a name, so " +
                "write the field's id in for in the template.", () => {
                const now = this.querySelector(CONTROL_SELECTOR);
                const labelNow = this.querySelector("label");
                return (now !== null && labelNow !== null && this.wrongFor.has(labelNow));
            });
            warnAboutMarkup(this, "<label> is not connected to the field: the field has no id, " +
                "and the label does not wrap it, so without JavaScript a screen " +
                "reader announces the field without a name. Write an id on the " +
                "field and the same value in for on the label, or wrap the " +
                "field in the <label>; the component does not write the id " +
                "itself, because an id the server did not send makes a server " +
                "update replace the field.", () => {
                const now = this.querySelector(CONTROL_SELECTOR);
                const labelNow = this.querySelector("label");
                return (now !== null &&
                    labelNow !== null &&
                    !now.id &&
                    !labelNow.contains(now) &&
                    !now.hasAttribute("aria-label") &&
                    !now.hasAttribute("aria-labelledby"));
            });
            // `for` wins over the label wrapping the field. If it points to another
            // id, the field has no label, with JavaScript as well.
            warnAboutMarkup(this, "<label> wraps the field, but its for points to a different id " +
                "than the field's, and for wins, so a screen reader announces " +
                "the field without a name. Remove for, or give the field the " +
                "same id.", () => {
                const now = this.querySelector(CONTROL_SELECTOR);
                const labelNow = this.querySelector("label");
                if (!now || !labelNow?.contains(now))
                    return false;
                const target = labelNow.getAttribute("for");
                return target !== null && target !== now.id;
            });
        }
        else {
            // With no label inside, it can be outside, with `for` set to the field's
            // id. It gets the class, as in `<lt-field>`; a patch of this region does
            // not touch it, so it needs no repair.
            if (control.id) {
                const outside = this.getRootNode().querySelector?.(`label[for="${CSS.escape(control.id)}"]`);
                if (outside)
                    addClass(outside, LABEL_CLASS);
            }
            warnAboutMarkup(this, "found no <label>, so the field has no label and a screen " +
                "reader announces it without a name. If the label is outside " +
                "<lt-suggestion>, its for must point to the field's id.", () => {
                const now = this.querySelector(CONTROL_SELECTOR);
                if (!now || this.querySelector("label"))
                    return false;
                // A <label> around the whole <lt-suggestion> also names the field.
                if ((now.labels?.length ?? 0) > 0)
                    return false;
                if (now.hasAttribute("aria-label") ||
                    now.hasAttribute("aria-labelledby"))
                    return false;
                const rootNow = this.getRootNode();
                return !(now.id &&
                    rootNow.querySelector?.(`label[for="${CSS.escape(now.id)}"]`));
            });
        }
        const list = this.listElement;
        if (list) {
            addClass(list, SUGGESTION_LIST_CLASS);
            if (!list.hasAttribute("role"))
                setAttr(list, "role", "listbox");
            if (!list.id) {
                // Same rule as for the field: if `aria-controls` points to nothing,
                // that is the id the new list should have.
                const wanted = control.getAttribute("aria-controls");
                const free = wanted && !root.getElementById?.(wanted);
                if (free)
                    this.ids.set(list, wanted);
                setAttr(list, "id", this.rememberedId(list, "lt-suggestion-list"));
            }
            if (!control.hasAttribute("aria-controls")) {
                setAttr(control, "aria-controls", list.id);
            }
            if (!list.hasAttribute("popover"))
                setAttr(list, "popover", "manual");
            this.nameList(list, control);
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
                setAttr(option, "id", this.rememberedId(option, "lt-suggestion-option"));
            }
            // From the start, not only when the list closes: otherwise the options
            // the server sent were without it until then, while the rest of the list
            // had it.
            if (!option.hasAttribute("aria-selected")) {
                setAttr(option, "aria-selected", "false");
            }
        }
    }
    /**
     * Sets the user's state back after a patch.
     *
     * The filtering is recomputed from what is in the field, so it needs no
     * memory. What the list shows and what is highlighted do.
     */
    repair() {
        if (!this.control)
            return;
        // The code is set back under `server-controlled` too. The attribute is
        // about whether the list is open and what is highlighted, not what the user
        // chose.
        const valueField = this.valueField;
        if (valueField &&
            this.chosen &&
            this.control.value === this.chosen.label &&
            valueField.value === "") {
            this.writeCode(valueField, this.chosen.value);
        }
        // An emptied status element gets its text back while the list is open,
        // where the count is what the user is working on. Not otherwise: every
        // patch of the region would announce the old count again. And not when
        // something else filters: then a patch is the answer to a keystroke, and
        // `announceFiltered` announces the new count right after.
        const status = this.querySelector(LT_SUGGESTION_PARTS.status);
        if (status &&
            this.wantOpen === true &&
            !this.prefiltered &&
            this.announcedText !== undefined &&
            (status.textContent ?? "").trim() === "") {
            setText(status, this.announcedText);
        }
        if (isServerControlled(this))
            return;
        // A field that became `readonly` or `disabled` while the list was open
        // closes it. Otherwise the repair kept it open, and Escape does nothing in
        // a locked field.
        if (this.locked && this.wantOpen)
            this.setOpen(false);
        if (this.wantOpen !== undefined) {
            /*
             * Each half is checked separately. A patch that touched only the field left
             * `aria-expanded="false"` while the options were visible, and the screen
             * reader said the list was closed while it was on screen.
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
             * The text is what counts, and the option must be visible. Otherwise the
             * web component highlighted something the server had just filtered out, and
             * `aria-activedescendant` pointed to a hidden element. The id is a
             * fallback: if the server sends the list again, the options are replaced
             * and get new ids, and the highlight is found again by its text. The code
             * counts too, since two places can have the same name.
             */
            const sameText = (o) => (o.textContent ?? "").trim() === this.active?.label &&
                o.getAttribute("data-value") === this.active?.value;
            const option = this.visible.find((o) => o.id === this.active?.id && sameText(o)) ??
                this.visible.find(sameText);
            if (!option) {
                // The pointer must go together with the highlight. Without this
                // `aria-activedescendant` pointed to an option that is now called
                // something else, and the screen reader announced a place the user never
                // navigated to.
                this.active = undefined;
                this.control.removeAttribute("aria-activedescendant");
            }
            else if (option.getAttribute("aria-selected") !== "true" ||
                this.control.getAttribute("aria-activedescendant") !== option.id) {
                // Both sides of the wiring are checked. If the patch stripped only
                // `aria-activedescendant` while the highlight remained, the screen reader
                // lost its reading position without anything else looking wrong.
                this.markOption(option);
            }
        }
    }
    disconnectedCallback() {
        window.removeEventListener("resize", this.reposition);
        window.removeEventListener("scroll", this.reposition, true);
        this.removeEventListener("focusout", this.handleFocusOut);
        this.observer?.disconnect();
        this.observer = undefined;
        this.unbind();
    }
    /**
     * Something else has already filtered, so the web component leaves the
     * options alone.
     *
     * The name used to be `server-filtered`, and that was misleading: it is not
     * about servers. A React app that renders only the matches has filtered all
     * the same, without any server involved.
     *
     * The web component hides an option when its text does not contain what is
     * in the field. If you filter on something else, without diacritics, on a
     * code that is not shown, or fuzzily, the two disagree, and then your filter
     * should win. The attribute turns off both the hiding and the empty
     * message, that is everything about what the list shows. The announcement
     * of the number of matches remains, since it is not markup.
     */
    get prefiltered() {
        return this.hasAttribute("prefiltered");
    }
    set prefiltered(on) {
        /*
         * The setter is not decoration. React 19 writes the property rather than
         * the attribute when a web component has one with that name, and a getter
         * alone throws "Cannot set property prefiltered". The attribute never
         * landed, and the web component hid what React had just rendered. The
         * hyphen in the old name hid the problem: `server-filtered` cannot be a
         * property name, so React sent it as an attribute.
         */
        setFlag(this, "prefiltered", on);
    }
    get listElement() {
        return this.querySelector(LIST_SELECTOR);
    }
    get emptyElement() {
        return this.querySelector(LT_SUGGESTION_PARTS.empty);
    }
    get valueField() {
        return this.querySelector(LT_SUGGESTION_PARTS.value);
    }
    /** The field accepts no input: `readonly` or `disabled`. */
    get locked() {
        return Boolean(this.control?.readOnly || this.control?.disabled);
    }
    /**
     * The options, recognised by the role and not by the class.
     *
     * The role must be there anyway, for the screen reader. A template that
     * wrote `<li role="option">` with its own styling got neither filtering nor
     * arrow keys when the web component looked for the class, and nothing said
     * so.
     */
    get options() {
        return derivedParts(this, OPTION_SELECTOR, "option", this.roledByMe);
    }
    /**
     * Which visible option is highlighted now.
     *
     * It is read from `aria-selected` in the markup, not from a counter inside
     * the web component. The server can set the highlight itself with
     * `lt.suggestion({ activeIndex })`, and a parallel counter would have jumped
     * to the top of the list on the first arrow key instead of to the next
     * option. The filtering also moves the options around, and then the lookup
     * gives the right answer by itself.
     */
    get activeIndex() {
        return this.selectable.findIndex((option) => option.getAttribute("aria-selected") === "true");
    }
    /** The options that are visible now. */
    get visible() {
        return this.options.filter((option) => !option.hidden);
    }
    /**
     * The visible options that can be chosen. An option with
     * `aria-disabled="true"` is shown, but the arrow keys skip it and it cannot
     * be chosen, with neither keyboard nor mouse.
     */
    get selectable() {
        return this.visible.filter((option) => option.getAttribute("aria-disabled") !== "true");
    }
    unbind() {
        const control = this.control;
        if (control) {
            control.removeEventListener("input", this.handleInput);
            control.removeEventListener("keydown", this.handleKeydown);
            control.removeEventListener("focus", this.handleFocus);
            this.control = undefined;
        }
        this.boundForm?.removeEventListener("reset", this.handleReset);
        this.boundForm = undefined;
        this.list?.removeEventListener("mousedown", this.handleListMouseDown);
        this.list = undefined;
        this.empty?.removeEventListener("mousedown", this.handleListMouseDown);
        this.empty = undefined;
    }
    /**
     * Reports when an update replaced the field because the id from the server
     * changed.
     *
     * The message comes only when the user had something to lose: the old field
     * is out of the tree, both had an id from the server, the ids differ, `name`
     * is the same, and the old field held something other than the new one. A
     * server that sends the text back has not lost it.
     */
    warnAboutReplacedControl(lost, control) {
        if (lost.isConnected || !lost.id || !control.id || lost.id === control.id)
            return;
        if (lost.getAttribute("name") !== control.getAttribute("name"))
            return;
        if (lost.value === control.value)
            return;
        warnAboutMarkup(this, "a server update replaced the field and lost what the user had " +
            "typed, along with focus and the list, because the field's id " +
            "was different from last time and the update did not recognise " +
            "it. Build the id from something that is the same every time, " +
            "such as the field name or a key in the data, not from a random " +
            "number or a counter.", 
        // The replacement has happened and cannot undo itself, unless another
        // update has already replaced the field again.
        () => !lost.isConnected &&
            this.querySelector(CONTROL_SELECTOR) === control);
    }
    bind() {
        const control = this.querySelector(CONTROL_SELECTOR);
        if (!control) {
            warnAboutMarkup(this, 'found no field: neither an <input> nor a [role="combobox"]. ' +
                "Without one the component does not know which field to listen " +
                "to, and neither filtering nor the arrow keys work.", 
            // An empty element is a region the server has not filled yet, and not
            // an error in the markup.
            () => this.childElementCount > 0 &&
                this.querySelector(CONTROL_SELECTOR) === null);
            // The field is gone. Firefox sends no `focusout` when the focused node is
            // removed, so the list would otherwise stay open without a field.
            this.wantOpen = undefined;
            this.active = undefined;
            const orphan = this.listElement;
            if (orphan)
                setFlag(orphan, "hidden", true);
            return;
        }
        warnAboutMarkup(this, 'found no list: neither a [role="listbox"] nor a ' +
            ".lt-suggestion__list. The options cannot be shown, filtered or " +
            "chosen with the keyboard.", () => this.querySelector(CONTROL_SELECTOR) !== null && !this.listElement);
        // The number of matches is announced in a status element the server
        // sends. Without it, people who cannot see the screen get no word that the
        // list narrowed, and the web component used to stay silent then.
        warnAboutMarkup(this, 'found no [role="status"], so the number of matches is not ' +
            "announced to people who cannot see the screen. Write an empty " +
            "element with that role, as `lt.suggestion()` does.", () => this.querySelector(CONTROL_SELECTOR) !== null &&
            this.listElement !== null &&
            this.querySelector(LT_SUGGESTION_PARTS.status) === null);
        if (control !== this.control) {
            if (this.control)
                this.warnAboutReplacedControl(this.control, control);
            // The new field has no focus. The open list and the highlight belonged to
            // the old one.
            this.wantOpen = undefined;
            this.active = undefined;
            // A value field the server filled applies to the text in the field now.
            const valueField = this.valueField;
            this.serverChoice =
                valueField && valueField.value !== ""
                    ? { label: control.value, value: valueField.value }
                    : undefined;
            this.chosen = this.serverChoice;
            this.control?.removeEventListener("input", this.handleInput);
            this.control?.removeEventListener("keydown", this.handleKeydown);
            this.control?.removeEventListener("focus", this.handleFocus);
            control.addEventListener("input", this.handleInput);
            control.addEventListener("keydown", this.handleKeydown);
            control.addEventListener("focus", this.handleFocus);
            this.control = control;
        }
        // The form is read every time: `form` on the field can point to another
        // one without anything moving.
        const form = control.form;
        if (form !== this.boundForm) {
            this.boundForm?.removeEventListener("reset", this.handleReset);
            form?.addEventListener("reset", this.handleReset);
            this.boundForm = form;
        }
        const code = this.valueField?.value ?? "";
        if (code !== "" &&
            code !== this.writtenCode &&
            code !== this.chosen?.value) {
            // Something other than the web component wrote the code: the server's word.
            this.serverChoice = { label: control.value, value: code };
            this.chosen = this.serverChoice;
        }
        /*
         * A press in the list or on the empty message must not take focus from the
         * field, not on the scrollbar either. Otherwise `focusout` closed the list
         * before the choice could happen. Both can be replaced by a patch, so the
         * listeners follow the elements.
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
        // The options are replaced independently of the field: in a Datastar app
        // the server sends a new list while the user types. Had this been behind
        // the check above, the new options would never have got their listener,
        // and choosing with the mouse would have stopped working after the first
        // update.
        for (const option of this.options) {
            if (this.bound.has(option))
                continue;
            option.addEventListener("mousedown", this.handleOptionMouseDown);
            option.addEventListener("click", this.handleOptionClick);
            this.bound.add(option);
        }
    }
    setOpen(open) {
        /*
         * A list without options below an empty field has nothing to show, and is
         * not opened. Without this, focus opened an empty list, and a field waiting
         * for its first search showed "No matches" before the user had typed a
         * character: the empty message follows the list, and the list was open.
         */
        const show = open && this.hasSomethingToShow();
        this.wantOpen = show;
        this.applyOpen(show);
    }
    hasSomethingToShow() {
        return this.options.length > 0 || (this.control?.value ?? "").trim() !== "";
    }
    /** Writes the state into the markup, without changing what the user wanted. */
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
     * The empty message follows the list: visible only when the list is open and
     * empty.
     *
     * It is a sibling of the list and not a child, so `hidden` on the list did
     * not hide it, and "No matches" stayed below a closed field after Escape.
     * If something else has filtered, it also controls the empty message.
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
     * Hides what does not match, and announces how many are left.
     *
     * Runs when the user types or focuses the field, and again after a patch
     * while the list is open, since the patch may have replaced the options.
     */
    filter() {
        /*
         * If something else has filtered, it decides what the list shows, and the
         * empty message is part of that. The web component therefore touches
         * nothing here, and the announcement comes from `announceFiltered`
         * instead.
         *
         * Two versions before this one were wrong. The first returned at once, and
         * then a Datastar app had no announcement of the number of matches at all.
         * The second counted here: then an empty field announced "No matches"
         * already on focus, before the user had typed a character, and an
         * asynchronous search announced it again on every keystroke while the
         * answer was still on its way. The web component also made the empty
         * message visible again after the app had hidden it, and there was no way
         * around it.
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
     * Removes the highlight from an option the filtering just hid.
     *
     * The cleanup belongs here, where the hiding happens, and not only in
     * `repair()`: that returns at once under `server-controlled`, and then
     * `aria-activedescendant` pointed to a hidden option, which the screen
     * reader announced anyway.
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
     * Announces the count when something else has filtered.
     *
     * The list then changes not from a keystroke but from the app rendering
     * again, and that is exactly the change the observer sees. The message
     * belongs here and not in `filter()`, because half a second over the
     * network can pass between the keystroke and the new answer.
     *
     * Only changes are announced. The first count is the starting point and not
     * news, and it is often 0 because the app has not rendered the list yet.
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
     * Announces the number of matches.
     *
     * Without this, people who cannot see the screen get no word that the list
     * narrowed while they typed. The text is generated on the client, and
     * `repair()` writes it back when an update empties the element. If the
     * element is missing, `bind()` has already reported it.
     */
    announce(hits) {
        const status = this.querySelector(LT_SUGGESTION_PARTS.status);
        if (!status)
            return;
        this.announcedText = this.countText(hits);
        setText(status, this.announcedText);
    }
    /**
     * The text for a count, in the page's language.
     *
     * `count-none` applies to zero matches, since "No matches" is a sentence of
     * its own and not an inflection. Otherwise `Intl.PluralRules` picks the form
     * the language requires, and `count-<form>` gives the text, with `{n}` for
     * the number. If the form is missing, `count-other` applies. If that is
     * missing too, the default text from `DEFAULT_TEXTS` applies.
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
        const shown = this.selectable;
        if (shown.length === 0 || !this.control)
            return;
        const next = (index + shown.length) % shown.length;
        this.markOption(shown[next]);
        shown[next].scrollIntoView({ block: "nearest" });
    }
    /** Highlights one option, and remembers which. */
    markOption(option) {
        if (!this.control)
            return;
        for (const other of this.options) {
            setAttr(other, "aria-selected", other === option ? "true" : "false");
        }
        this.active = {
            id: option.id,
            label: (option.textContent ?? "").trim(),
            value: option.getAttribute("data-value"),
        };
        // aria-activedescendant moves the screen reader's reading position without
        // focus leaving the field. Without it the option is never announced.
        setAttr(this.control, "aria-activedescendant", option.id);
    }
    choose(option) {
        if (!this.control || this.locked)
            return;
        if (option.getAttribute("aria-disabled") === "true")
            return;
        const label = (option.textContent ?? "").trim();
        const value = option.getAttribute("data-value") ?? label;
        /*
         * The setter from the prototype chain, past a `value` React has put on the
         * node itself to track it. Set directly on the node, React saw no change,
         * `onChange` never came, and on the next render React wrote the old text
         * back into the field. The chain and not just the nearest prototype, so
         * `<textarea>` and a subclass of `<input>` work too.
         */
        let setter;
        for (let proto = Object.getPrototypeOf(this.control); proto && !setter; proto = Object.getPrototypeOf(proto)) {
            setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
        }
        // The field shows the text. The code goes to the value field, when the
        // server has written one, and in the event. `data-value` used to end up in
        // the field, and the user who chose Bergen saw "4601".
        if (setter)
            setter.call(this.control, label);
        else
            this.control.value = label;
        const valueField = this.valueField;
        if (valueField) {
            this.writeCode(valueField, value);
            this.chosen = { label, value };
        }
        // The flag must be set before the events are dispatched. `input` is the
        // same event the web component listens to, so without this the list opens
        // again the moment the user has chosen something.
        this.choosing = true;
        this.control.dispatchEvent(new Event("input", { bubbles: true }));
        this.control.dispatchEvent(new Event("change", { bubbles: true }));
        this.choosing = false;
        this.setOpen(false);
        this.dispatchEvent(new CustomEvent("suggestion-select", {
            detail: { value, label },
            bubbles: true,
            composed: true,
        }));
    }
    /**
     * Writes the code in the value field and dispatches `input` and `change` on
     * it, so `data-bind` in Datastar sees the code. The events do not bubble: a
     * `data-on:input` on the host, which the recipe uses to fetch suggestions,
     * must not trigger a search for the code every time the user chooses.
     */
    writeCode(valueField, code) {
        if (valueField.value === code)
            return;
        this.writtenCode = code;
        setAttr(valueField, "value", code);
        // A hidden field reads the attribute. If the field has another type and a
        // morph has already set the property, the attribute no longer counts.
        if (valueField.value !== code)
            valueField.value = code;
        valueField.dispatchEvent(new Event("input"));
        valueField.dispatchEvent(new Event("change"));
    }
}
LtSuggestion.observedAttributes = [
    "prefiltered",
    SERVER_CONTROLLED,
    ...COUNT_ATTRIBUTES,
];
export function defineLtSuggestion(tagName = LT_SUGGESTION_TAG) {
    defineElement(tagName, LtSuggestion);
}
//# sourceMappingURL=lt-suggestion.js.map