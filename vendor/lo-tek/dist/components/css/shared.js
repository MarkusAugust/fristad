/**
 * Shared building blocks for the component API.
 *
 * Each component exports one function that takes an object of options and
 * returns the attributes you spread onto the element. The shape is the same
 * for all of them, so you only need to learn it once.
 */
/**
 * The attributes each set manages, including those that are off right now.
 *
 * `attributes()` drops the keys without a value, and with them the record of
 * which attributes the builder function has a say in. `setAttributes` needs
 * exactly that: `lt.switch()` manages `disabled` and must be able to remove
 * it, while `lt.button()` has never written it and must leave it alone. The
 * list is kept in a map beside the object, not on it, so spreading, JSON and
 * React see the same as before.
 */
const managed = new WeakMap();
/** The names the set manages, or `undefined` for a set we did not create. */
export function managedNames(set) {
    return managed.get(set);
}
/**
 * Gives a set rewritten from another the same managed names.
 *
 * `/react` renames `class` to `className` in a new object, and the new object
 * was not in the map. `lt.setAttributes` from there then only cleaned up
 * `data-*`, and `aria-invalid` stayed after the field was corrected. The
 * names are given as they are spelled in the new set.
 *
 * For `/react` and other entry points that rewrite the sets, not for apps: a
 * set from `lt.*` is already managed.
 */
export function rememberManaged(set, names) {
    managed.set(set, names);
}
/**
 * Removes attributes without a value, so `{...spread}` does not set empty
 * fields.
 *
 * When the set builds on another, as `lt.search()` does on `lt.input()`, the
 * other is passed in as `based`. The spread has already lost the keys that
 * were off, and without this the search field only managed `data-state` when
 * it happened to be set: it could be set, but not removed.
 */
export function attributes(values, ...based) {
    const result = {};
    for (const [name, value] of Object.entries(values)) {
        if (value !== undefined) {
            ;
            result[name] = value;
        }
    }
    const names = new Set(Object.keys(values));
    for (const set of based) {
        for (const name of managed.get(set) ?? [])
            names.add(name);
    }
    managed.set(result, [...names]);
    return result;
}
/** Creates a type guard for a list of allowed strings. */
export function createGuard(allowed) {
    return (value) => allowed.includes(value);
}
/**
 * Validation state for form controls.
 *
 * Input, textarea and select each had an identical type of their own. Now
 * there is one, so a value can pass between them without conversion.
 */
export const fieldStates = ["default", "invalid", "success"];
export const isFieldState = createGuard(fieldStates);
/** How a required field is marked in its label. */
export const requiredMarkers = ["symbol", "text"];
export const isRequiredMarker = createGuard(requiredMarkers);
/**
 * Warns once per message, in the console.
 *
 * The builder functions are pure functions without an element to attach the
 * message to, so they cannot use `warnAboutMarkup`. The deduplication is
 * needed all the same: an app that renders a list of fields would otherwise
 * say the same thing per field and per render.
 */
const said = new Set();
export function warnOnce(message) {
    if (typeof console === "undefined" || said.has(message))
        return;
    said.add(message);
    console.warn(message);
}
let idCounter = 0;
/**
 * Creates an id that is unique within the document.
 *
 * Safe only when the markup is rendered once. When the same field is rendered
 * both on a server and in the browser, the two runs give two different ids,
 * and the wiring is broken until the framework has corrected it.
 */
export function createFieldId() {
    idCounter += 1;
    return `lt-field-${idCounter}-${Math.random().toString(36).slice(2, 8)}`;
}
/**
 * The keys a builder function creates ids from, checked.
 *
 * A key is the same as `key` in React: a fixed value from the data, such as
 * a record id. The id is the same on every render, and a server update
 * recognises the element. An index is not a key: if the server inserts a row
 * first, the same index points to something else.
 *
 * Throws, instead of warning and creating a fallback as `idOrFallback` does:
 * an empty key, one with whitespace or two identical ones give ids that are
 * empty, invalid or identical, and there is no fallback that is right.
 * Whitespace is what JavaScript counts as whitespace (`\s`), which the
 * Kotlin version spells out.
 */
export function checkedKeys(builder, keys) {
    if (!Array.isArray(keys)) {
        throw new Error(`${builder}: keys is missing. Pass a list with one key per element, taken from the data, such as a record id.`);
    }
    const seen = new Set();
    for (const key of keys) {
        if (typeof key !== "string") {
            throw new Error(`${builder}: a key in keys is not a string but ${typeof key}. The keys become ids; turn numbers into strings, as in String(recordId).`);
        }
        if (key === "") {
            throw new Error(`${builder}: a key in keys is empty. The keys become ids, and each must be a fixed value from the data.`);
        }
        if (/\s/.test(key)) {
            throw new Error(`${builder}: the key "${key}" contains whitespace. The keys become ids, and an id cannot contain whitespace.`);
        }
        if (seen.has(key)) {
            throw new Error(`${builder}: the key "${key}" appears twice in keys. The keys become ids, and two equal ids point to the same element.`);
        }
        seen.add(key);
    }
    return keys;
}
/**
 * The id the builder function received, or a fallback with a warning.
 *
 * Every builder function that takes an id requires it in the type. That is
 * not enough on its own: a consumer without TypeScript sees no type, and
 * plain HTML with `<script type="module">` is a first-class way to use
 * LO-TEK. Without the fallback the ids became strings like `undefined-list`,
 * `aria-controls` pointed there, and the wiring was broken in a way that
 * looked valid.
 *
 * The empty string counts as no id. `lt.field({ id: "" })` gave `for=""` and
 * `help.id="-help"`, the same problem without anything saying so.
 *
 * The name of the builder function is in the message. Without it the
 * suggestion field said "lt.field()" and sent the developer to the wrong
 * place.
 */
export function idOrFallback(builder, id, option = "id") {
    if (typeof id === "string" && id.trim() !== "")
        return id;
    warnOnce(`${builder}: no ${option} given, so a random one is created, and two runs ` +
        "create two different ones. Rendered on a server and in the browser, the " +
        "wiring points to something that does not exist, and a server that sends " +
        "the region again with a new id makes the morph replace the elements and " +
        `lose what the user did in them. Pass ${option}, in React from useId().`);
    return createFieldId();
}
//# sourceMappingURL=shared.js.map