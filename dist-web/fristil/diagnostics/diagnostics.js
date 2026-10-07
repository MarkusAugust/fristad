/**
 * Sier fra om markup som ikke stemmer med Fristil.
 *
 * Fullføringen og forklaringen kommer fra VS Codes egen HTML-tjeneste, og
 * den validerer ingenting: et element som ikke finnes er lovlig HTML, og et
 * attributt komponenten aldri leser står der like stille. Dette er den
 * andre halvdelen. Det leser hver `<fs-…>`-tagg i dokumentet og sjekker den
 * mot `elements.ts` ved siden av, som genereres fra den samme `metadata.ts`
 * i editorutvidelsen som fullføringen, så de to kan ikke sprike.
 *
 * Fila ligger i pakken og har ingen avhengighet til VS Code eller til DOM.
 * Utvidelsen importerer den fra `@fristil/designsystem/diagnostics`, det
 * samme gjør `fristil sjekk` på kommandolinjen, og en app kan kjøre den i
 * sine egne tester over HTML-en serveren faktisk sender. Da kan
 * `editor/scripts/sjekk-diagnostikk.ts` kjøre den med bun over kjente feil
 * og se at hver av dem felles.
 *
 * Det som sjekkes, i rekkefølgen det meldes:
 *
 *   1. Elementet finnes. `<fs-dialog-header>` finnes ikke, og nettleseren
 *      sier ingenting om det.
 *   2. Attributtet finnes på elementet. Et navn som bare skiller seg fra et
 *      kjent i bindestreker, understreker eller store bokstaver, som
 *      `onlinetext`, meldes som skrivefeil, også i en tagg med mal i.
 *      Ellers slipper globale HTML-attributter, `data-*`, `aria-*`,
 *      hendelser og rammeverkenes egne gjennom: `hx-`, `x-`, `v-`,
 *      hyperscripts `_`, Angulars `i18n`, og alt med `:`, `@`, `*`,
 *      klammer, parenteser eller `%$#?` i navnet.
 *   3. Verdien er lovlig: i den lukkede lista der det finnes en, et tall der
 *      det skal være et tall, og ikke `="false"` på et boolsk attributt, som
 *      betyr på.
 *   4. `<fs-field>` har en kontroll og en ledetekst, med de samme
 *      unntakene og den samme teksten som komponenten selv bruker i
 *      nettleseren.
 *   5. Klassene finnes. `class="fs-buton"` meldes, med den nærmeste kjente
 *      som forslag. Klassene leses fra `classes.ts`, som genereres fra
 *      pakkens CSS.
 *   6. Verdiene på det klassen tar er lovlige: `data-variant="ghots"` på
 *      `fs-button` meldes, med lista fra byggefunksjonen. Standardverdien,
 *      den som ikke gir noe attributt, nevnes også.
 *
 * Et funn kan ha en rettelse: bytt navnet til det som var ment, ta bort et
 * boolsk attributt med `="false"`, sett inn en ledetekst. Rettelsen er
 * bare tekst og posisjoner, så den kan sjekkes her, og editoren gjør den om
 * til en lyspære.
 *
 * Markup som blir til på en server er ofte en mal, og en mal er ikke hel:
 * `{{ if .Feil }}invalid{{ end }}` i en tagg, `{{ template "input" . }}`
 * der kontrollen skulle stått. Der det står malsyntaks, Go, Jinja, PHP, ASP,
 * JS-maler eller Razor, holder diagnostikken seg unna: ukjente attributtnavn
 * meldes ikke i en tagg med mal i, en verdi med mal i sjekkes ikke, og et
 * felt med mal i regnes som ufylt. En verdi i klammer, `open={isOpen}` i Astro og
 * Svelte, er et uttrykk og sjekkes ikke. Elementnavnet sjekkes alltid: det
 * står aldri i en mal.
 *
 * Posisjonene er tegnindekser i den opprinnelige teksten. Kommentarer,
 * skript og stilark blankes ut med like mange tegn før lesingen, så en
 * `<fs-…>` i en kommentar ikke meldes og ingen posisjon forskyves.
 */
const DOCS = "https://fristil.sobernetics.no/components/";
/*
 * Attributter ethvert element kan ha, uten at komponenten leser dem: de
 * globale attributtene i HTML. Prefiksene under er `data-*`, `aria-*`,
 * hendelsene, HTMX, Alpine og Vue, og hyperscripts `_` og Angulars `i18n`.
 * Svelte, Angular, Alpines korte former og malspråkenes rester har `:`,
 * `@`, `*`, klammer, parenteser eller `%$#?` i navnet, og det har aldri et
 * Fristil-attributt. Et navn utenfor alt dette
 * som komponenten ikke kjenner, er nesten alltid en skrivefeil.
 */
const GLOBAL = new Set([
    "accesskey",
    "autocapitalize",
    "autofocus",
    "class",
    "contenteditable",
    "dir",
    "draggable",
    "enterkeyhint",
    "exportparts",
    "hidden",
    "id",
    "inert",
    "inputmode",
    "is",
    "itemid",
    "itemprop",
    "itemref",
    "itemscope",
    "itemtype",
    "lang",
    "nonce",
    "part",
    "popover",
    "role",
    "slot",
    "spellcheck",
    "style",
    "tabindex",
    "title",
    "translate",
    "xmlns",
]);
const isGlobal = (name) => GLOBAL.has(name) ||
    name === "_" ||
    /^(data-|aria-|on|hx-|x-|v-|i18n)/.test(name) ||
    /[:@*[\](){}%$#?]/.test(name);
/*
 * Go, Jinja, PHP, ASP, JS-maler og Razor. Razors `@Navn` regnes bare i en
 * verdi og i innhold, ikke i en tagg: der er `@click` Alpine, et attributt
 * som skal slippe gjennom uten å gjøre resten av taggen til en mal.
 */
const TEMPLATE = /\{\{|\{%|\{#|<\?|<%|\$\{|@\(/;
const isTemplated = (text) => TEMPLATE.test(text);
// En verdi i klammer er Astro eller Svelte, og et uttrykk, ikke en verdi.
const isTemplatedValue = (text) => isTemplated(text) || /^\s*[@{]/.test(text);
// Razor i innholdet, utenfor taggene: `<input @input="…">` er Alpine på en
// kontroll, og skal ikke gjøre feltet til en mal.
const isTemplatedContent = (text) => isTemplated(text) || /(^|\s)@[A-Za-z]/.test(text.replace(/<[^>]*>/g, " "));
/** Navnet uten bindestreker og store bokstaver, for å kjenne igjen skrivefeil. */
const normalized = (name) => name.toLowerCase().replace(/[-_]/g, "");
/*
 * Redigeringsavstanden mellom to navn: tegn som må byttes, settes inn, tas
 * bort eller bytte plass. At to nabotegn har byttet plass teller som én,
 * for `ghots` er `ghost`. Tre rader er nok, ikke en tabell.
 */
function distance(a, b) {
    let before = new Array(b.length + 1);
    let previous = Array.from({ length: b.length + 1 }, (_, j) => j);
    for (let i = 1; i <= a.length; i++) {
        const current = [i];
        for (let j = 1; j <= b.length; j++) {
            let d = Math.min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
            if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1])
                d = Math.min(d, before[j - 2] + 1);
            current[j] = d;
        }
        before = previous;
        previous = current;
    }
    return previous[b.length];
}
/**
 * Det nærmeste kjente navnet, når det er nært nok til å være en skrivefeil.
 * `sure` er sant når navnet har de samme bokstavene uten bindestreker og
 * understreker, som `onlinetext`. Ellers må navnet være høyst ett tegn unna,
 * og to når det er langt nok til at to feil er sannsynlig: `fs-buton` er
 * `fs-button`, mens `fs-tabs` ikke er `fs-table`. Svarene huskes per kjøring: den samme ukjente klassen kan stå på
 * hver rad i en tabell, og avstanden skal regnes én gang.
 */
export function closest(name, candidates, cache) {
    if (cache?.has(name))
        return cache.get(name);
    const result = find(name, candidates);
    cache?.set(name, result);
    return result;
}
function commonPrefix(a, b) {
    let i = 0;
    while (i < a.length && i < b.length && a[i] === b[i])
        i++;
    return i;
}
function find(name, candidates) {
    const wanted = normalized(name);
    const same = candidates.find((c) => normalized(c) === wanted);
    if (same)
        return { name: same, sure: true };
    const lower = name.toLowerCase();
    const max = name.length >= 8 ? 2 : 1;
    let best;
    let bestDistance = max + 1;
    for (const candidate of candidates) {
        if (Math.abs(candidate.length - lower.length) > max)
            continue;
        const d = distance(lower, candidate);
        // Ved lik avstand vinner den som deler lengst begynnelse: `fs-tabel` er
        // `fs-table`, ikke `fs-label`.
        if (d < bestDistance ||
            (d === bestDistance &&
                best &&
                commonPrefix(lower, candidate) > commonPrefix(lower, best))) {
            bestDistance = d;
            best = candidate;
        }
    }
    return best ? { name: best, sure: false } : undefined;
}
/** Bytter hvert treff med like mange mellomrom, så posisjonene står. */
const blankOut = (text, pattern) => text.replace(pattern, (hit) => hit.replace(/[^\n]/g, " "));
/** Teksten uten kommentarer, skript og stilark, med samme lengde. */
export function withoutHidden(text) {
    let out = blankOut(text, /<!--[\s\S]*?(?:-->|$)/g);
    out = blankOut(out, /<script\b[\s\S]*?(?:<\/script\s*>|$)/gi);
    out = blankOut(out, /<style\b[\s\S]*?(?:<\/style\s*>|$)/gi);
    return out;
}
/*
 * Der taggen som begynner på `from` slutter: indeksen til `>`, eller -1.
 * `<?…?>` og `<%…%>` inne i taggen hoppes over, så PHP og ASP ikke
 * avslutter den før tiden.
 */
export function tagEnd(text, from) {
    let quote = null;
    for (let i = from; i < text.length; i++) {
        const char = text[i];
        if (quote) {
            if (char === quote)
                quote = null;
            continue;
        }
        if (char === '"' || char === "'")
            quote = char;
        else if (char === "<" && (text[i + 1] === "?" || text[i + 1] === "%")) {
            const closer = text.indexOf(`${text[i + 1]}>`, i + 2);
            if (closer < 0)
                return -1;
            i = closer + 1;
        }
        else if (char === ">")
            return i;
    }
    return -1;
}
/*
 * Attributtene i en tagg, lest fra teksten mellom navnet og `>`. En verdi
 * står i anførselstegn, uten, eller i klammer: `open={isOpen}` er Astro og
 * Svelte, og klammene kan ha klammer i seg. Den leses som én verdi, så det
 * som står inni ikke blir attributtnavn.
 */
export function readAttributes(body, offset) {
    const out = [];
    let i = 0;
    const skipSpace = () => {
        while (i < body.length && /\s/.test(body[i]))
            i++;
    };
    while (i < body.length) {
        const spaceStart = i;
        skipSpace();
        const nameStart = i;
        while (i < body.length && !/[\s"'=<>/]/.test(body[i]))
            i++;
        if (i === nameStart) {
            i++;
            continue;
        }
        const name = body.slice(nameStart, i);
        let value;
        let valueStart = i;
        const afterName = i;
        skipSpace();
        if (body[i] === "=") {
            i++;
            skipSpace();
            const open = body[i];
            if (open === '"' || open === "'") {
                const close = body.indexOf(open, i + 1);
                valueStart = i + 1;
                value = body.slice(valueStart, close < 0 ? body.length : close);
                i = close < 0 ? body.length : close + 1;
            }
            else if (open === "{") {
                valueStart = i;
                i = bracesEnd(body, i);
                value = body.slice(valueStart, i);
            }
            else {
                valueStart = i;
                while (i < body.length && !/[\s"'=<>`]/.test(body[i]))
                    i++;
                value = body.slice(valueStart, i);
            }
        }
        else
            i = afterName;
        out.push({
            name: name.toLowerCase(),
            value,
            start: offset + nameStart,
            end: offset + nameStart + name.length,
            valueEnd: offset + (value === undefined ? afterName : i),
            spaceStart: offset + spaceStart,
            valueStart: offset + (value === undefined ? afterName : valueStart),
        });
    }
    return out;
}
/** Der en klammeverdi som begynner på `from` slutter, etter den matchende `}`. */
function bracesEnd(text, from) {
    let depth = 0;
    let quote = null;
    for (let i = from; i < text.length; i++) {
        const char = text[i];
        if (quote) {
            if (char === quote)
                quote = null;
            continue;
        }
        if (char === '"' || char === "'" || char === "`")
            quote = char;
        else if (char === "{")
            depth++;
        else if (char === "}" && --depth === 0)
            return i + 1;
    }
    return text.length;
}
const list = (names) => names.join(", ");
/** Slik komponentene leser tallet: `Number(verdi)`, og tomt er ikke et tall. */
const isNumber = (value) => value.trim() !== "" && !Number.isNaN(Number(value));
function checkAttribute(tag, element, attribute, templatedTag) {
    const { name, value } = attribute;
    const base = {
        start: attribute.start,
        end: attribute.end,
        link: element.link,
    };
    const known = element.attributes[name];
    if (!known) {
        // `onlinetext` ville sluppet gjennom som en hendelse, og `requiredmarker`
        // som en skrivefeil uten hjelp. Et kjent navn med samme bokstaver er
        // sterkere enn begge deler.
        const meant = Object.keys(element.attributes).find((candidate) => normalized(candidate) === normalized(name));
        if (meant)
            return {
                ...base,
                severity: "warning",
                message: `<${tag}> har ikke attributtet «${name}». Mente du ${meant}?`,
                fix: {
                    title: `Bytt til ${meant}`,
                    start: attribute.start,
                    end: attribute.end,
                    text: meant,
                    preferred: true,
                },
            };
        // I en tagg med mal i er navnene ikke til å stole på: `{{ if }}` blir
        // til «if» og «end», og ingen av dem er en skrivefeil.
        if (templatedTag || isGlobal(name))
            return undefined;
        return {
            ...base,
            severity: "warning",
            message: `<${tag}> har ikke attributtet «${name}», og komponenten leser ` +
                `det ikke. Attributtene er ${list(Object.keys(element.attributes))}.`,
        };
    }
    if (value !== undefined && isTemplatedValue(value))
        return undefined;
    if (known.type === "flag") {
        // `hidden="until-found"` er HTML, ikke en feil.
        if (name === "hidden" && value === "until-found")
            return undefined;
        if (value !== undefined && value !== "" && value.toLowerCase() !== name)
            return {
                ...base,
                severity: "warning",
                message: `${name} er et boolsk attributt: det står der eller ikke. ` +
                    `${name}="${value}" betyr det samme som ${name}. Ta det bort ` +
                    "for å slå det av.",
                fix: {
                    title: `Ta bort ${name}`,
                    start: attribute.spaceStart,
                    end: attribute.valueEnd,
                    text: "",
                    preferred: true,
                },
            };
        return undefined;
    }
    if (known.type === "values") {
        if (value === undefined || !known.values.includes(value))
            return {
                ...base,
                severity: "error",
                message: `${name} kan ikke være «${value ?? ""}». Lovlige verdier: ` +
                    `${list(known.values)}.`,
            };
        return undefined;
    }
    if (known.type === "number" && value !== undefined && !isNumber(value))
        return {
            ...base,
            severity: "error",
            message: `${name} skal være et tall, ikke «${value}».`,
        };
    return undefined;
}
/*
 * Det `<fs-field>` selv sier fra om i nettleseren, med den samme teksten.
 * Komponenten godtar tre måter å gi feltet et navn: en `<label>` inni, en
 * `<label for>` utenfor som peker på kontrollen, eller `aria-label` og
 * `aria-labelledby` på kontrollen. Et tomt element er et område serveren
 * ikke har fylt ennå, og meldes ikke. Det samme gjelder et element med mal
 * i: kontrollen kan stå i en partial.
 */
const CONTROL = /<(input|textarea|select)(?=[\s/>])/gi;
/** Den første kontrollen i innholdet, slik komponenten teller dem: ikke `type="hidden"`. */
function findControl(content) {
    for (const hit of content.matchAll(CONTROL)) {
        const from = (hit.index ?? 0) + hit[0].length;
        const to = tagEnd(content, from);
        // En kontroll uten `>`, mens noen skriver, er ingen kontroll ennå.
        if (to < 0)
            continue;
        const attributes = readAttributes(content.slice(from, to), 0);
        const type = attributes.find((a) => a.name === "type")?.value;
        if (hit[1].toLowerCase() === "input" && type?.toLowerCase() === "hidden")
            continue;
        return attributes;
    }
    return undefined;
}
/*
 * Verdiene i hver `<label for>` i dokumentet, lest én gang: oppslaget per
 * felt skal ikke gå gjennom hele dokumentet, for det kjøres ved hvert
 * tastetrykk, og tusen felt ganger hele fila ble sekunder.
 */
function labelTargets(text) {
    const out = new Set();
    const pattern = /<label\b[^>]*?\sfor\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))/gi;
    for (const hit of text.matchAll(pattern))
        out.add(hit[1] ?? hit[2] ?? hit[3]);
    return out;
}
function checkField(labels, tag, nameStart, nameEnd, content, contentStart, attributes, link) {
    if (!/<[a-z]/i.test(content) || isTemplatedContent(content))
        return [];
    const base = { start: nameStart, end: nameEnd, link };
    // Innrykket før første tagg, så en innsatt ledetekst får sin egen linje.
    const leading = content.match(/^\s*/)?.[0] ?? "";
    const control = findControl(content);
    if (!control)
        return [
            {
                ...base,
                severity: "warning",
                message: `<${tag}> fant ingen kontroll å koble til. Ledeteksten, ` +
                    "hjelpeteksten og feilmeldingen står uten et felt, og koblingen " +
                    "kan ikke lages. Sett inn et <input>, <textarea> eller <select>.",
            },
        ];
    if (/<label(?=[\s/>])/i.test(content))
        return [];
    const has = (name) => control.find((a) => a.name === name);
    if (has("aria-label") || has("aria-labelledby"))
        return [];
    // Komponenten finner ledeteksten via kontrollens id ved første
    // synkronisering, og via control-id siden. Begge teller.
    const ids = [
        attributes.find((a) => a.name === "control-id")?.value,
        has("id")?.value,
    ];
    if (ids.some((id) => id && labels.has(id)))
        return [];
    return [
        {
            ...base,
            severity: "warning",
            message: `<${tag}> fant ingen <label>. Feltet får da ingen ledetekst, og en ` +
                "skjermleser leser det opp uten navn.",
            fix: {
                title: "Sett inn en ledetekst",
                start: contentStart + leading.length,
                end: contentStart + leading.length,
                text: `<label>Ledetekst</label>${leading.includes("\n") ? leading : ""}`,
                preferred: true,
            },
        },
    ];
}
/*
 * Det `<fs-session-timeout>` selv sier fra om i nettleseren, med den samme
 * teksten: varselet er en dialog den som rendrer skriver, og uten den kommer
 * det ingen advarsel. Et tomt element meldes her, i motsetning til i
 * nettleseren, der det kan være et område som ikke er fylt ennå: i en fil er
 * et tomt `<fs-session-timeout>` den gamle bruken. Et element med mal i
 * meldes ikke, siden dialogen kan stå i en partial.
 */
function checkSessionTimeout(tag, nameStart, nameEnd, content, link) {
    if (isTemplatedContent(content))
        return [];
    if (/<dialog(?=[\s/>])/i.test(content))
        return [];
    return [
        {
            start: nameStart,
            end: nameEnd,
            link,
            severity: "warning",
            message: `<${tag}> fant ingen <dialog>. Varselet kan ikke vises, og økten går ` +
                "ut uten advarsel.",
        },
    ];
}
/*
 * Klassene og det de tar. Kjøres over hver tagg, ikke bare `<fs-…>`: en
 * `fs-button` står på en `<button>`. Et klassenavn som begynner på `fs-` og
 * ikke finnes, meldes med det nærmeste kjente som forslag. Har taggen en
 * kjent klasse, sjekkes verdiene på attributtene klassen tar, som
 * `data-variant` på `fs-button` og `data-state` på `fs-input`.
 */
function checkClasses(attributes, classes, cache) {
    const findings = [];
    const classAttribute = attributes.find((a) => a.name === "class");
    if (!classAttribute?.value || isTemplatedValue(classAttribute.value))
        return findings;
    const names = Object.keys(classes);
    const present = [];
    let offset = classAttribute.valueStart;
    for (const token of classAttribute.value.split(/(\s+)/)) {
        const start = offset;
        offset += token.length;
        // `fs-{b}` i Svelte er et uttrykk, og har ingen klasse å sjekke ennå.
        if (!token.trim() || !token.startsWith("fs-") || /[{}]/.test(token))
            continue;
        const info = classes[token];
        if (info) {
            present.push(info);
            continue;
        }
        const meant = closest(token, names, cache);
        findings.push({
            start,
            end: start + token.length,
            severity: "warning",
            link: meant ? classes[meant.name].link : DOCS,
            message: `Klassen «${token}» finnes ikke i Fristil.` +
                (meant ? ` Mente du ${meant.name}?` : ""),
            ...(meant
                ? {
                    fix: {
                        title: `Bytt til ${meant.name}`,
                        start,
                        end: start + token.length,
                        text: meant.name,
                        preferred: meant.sure,
                    },
                }
                : {}),
        });
    }
    // Verdiene sjekkes også i en tagg med mal i: et rent attributt med en ren
    // verdi er til å stole på, det er bare navnene rundt malen som ikke er det.
    for (const attribute of attributes) {
        // Tom verdi er ingen verdi, og sjekkes ikke.
        if (!attribute.value || isTemplatedValue(attribute.value))
            continue;
        for (const info of present) {
            const takes = info.attributes[attribute.name];
            if (!takes)
                continue;
            if (takes.values.includes(attribute.value) ||
                attribute.value === takes.default)
                continue;
            const meant = closest(attribute.value, [
                ...takes.values,
                ...(takes.default ? [takes.default] : []),
            ]);
            const shown = attribute.value.replace(/\s+/g, " ");
            findings.push({
                start: attribute.start,
                end: attribute.valueEnd,
                severity: "warning",
                link: info.link,
                message: `${attribute.name} kan ikke være «${shown}» på ${info.title.toLowerCase()}. ` +
                    `Lovlige verdier: ${list(takes.values)}` +
                    (takes.default ? `, og ${takes.default} uten attributt.` : "."),
                ...(meant
                    ? {
                        fix: {
                            title: `Bytt til ${meant.name}`,
                            start: attribute.valueStart,
                            end: attribute.valueStart + attribute.value.length,
                            text: meant.name,
                            preferred: meant.sure,
                        },
                    }
                    : {}),
            });
            break;
        }
    }
    return findings;
}
/** Alle funn i teksten, i den rekkefølgen de står. */
export function diagnose(text, elements, classes = {}) {
    const source = withoutHidden(text);
    const findings = [];
    const known = Object.keys(elements);
    let labels;
    const nearest = new Map();
    // Klassene, på alle tagger.
    for (const hit of source.matchAll(/<([a-z][a-z0-9-]*)(?=[\s/>])/gi)) {
        const nameEnd = (hit.index ?? 0) + hit[0].length;
        const end = tagEnd(source, nameEnd);
        if (end < 0)
            continue;
        const body = source.slice(nameEnd, end).replace(/\/$/, "");
        if (!/\bclass\s*=/i.test(body))
            continue;
        findings.push(...checkClasses(readAttributes(body, nameEnd), classes, nearest));
    }
    for (const hit of source.matchAll(/<(fs-[a-z0-9-]*)(?=[\s/>])/gi)) {
        const tag = hit[1].toLowerCase();
        const nameStart = (hit.index ?? 0) + 1;
        const nameEnd = nameStart + tag.length;
        const element = elements[tag];
        if (!element) {
            findings.push({
                start: nameStart,
                end: nameEnd,
                severity: "error",
                link: DOCS,
                message: `<${tag}> finnes ikke i Fristil. Elementene er ${list(known)}.`,
            });
            continue;
        }
        const end = tagEnd(source, nameEnd);
        if (end < 0)
            continue;
        const body = source.slice(nameEnd, end).replace(/\/$/, "");
        const templatedTag = isTemplated(body);
        const attributes = readAttributes(body, nameEnd);
        for (const attribute of attributes) {
            const finding = checkAttribute(tag, element, attribute, templatedTag);
            if (finding)
                findings.push(finding);
        }
        if (tag === "fs-session-timeout") {
            const closer = /<\/fs-session-timeout\b/gi;
            closer.lastIndex = end;
            const close = closer.exec(source)?.index ?? -1;
            // Uten lukketagg er markupen halvskrevet, og resten av fila hører ikke
            // til elementet. En <dialog> lenger ned ville ellers tatt meldingen.
            if (close >= 0) {
                findings.push(...checkSessionTimeout(tag, nameStart, nameEnd, source.slice(end + 1, close), element.link));
            }
        }
        if (tag === "fs-field") {
            // Lukketaggen kan ha store bokstaver, som åpningstaggen.
            const closer = /<\/fs-field\b/gi;
            closer.lastIndex = end;
            const close = closer.exec(source)?.index ?? -1;
            const content = source.slice(end + 1, close < 0 ? source.length : close);
            labels ??= labelTargets(source);
            findings.push(...checkField(labels, tag, nameStart, nameEnd, content, end + 1, attributes, element.link));
        }
    }
    return findings.sort((a, b) => a.start - b.start);
}
//# sourceMappingURL=diagnostics.js.map