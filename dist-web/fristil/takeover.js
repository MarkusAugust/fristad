/**
 * Regnestykket bak `fristil overta <komponent>`.
 *
 * Kommandoen kopierer kildekoden til én komponent inn i prosjektet ditt.
 * Laget og `--fs-*`-variablene dekker utseendet, og siden ingen komponent
 * har shadow DOM, når CSS-en din fram overalt. Det de ikke dekker er «jeg
 * vil at denne komponenten skal oppføre seg annerledes». Da er alternativet
 * enten å leve med det eller å skrive komponenten på nytt fra bunnen, og
 * denne kommandoen er veien imellom.
 *
 * Etter omleggingen er det en overkommelig vei: en komponent er et stilark
 * og en fil på mellom hundre og tre hundre linjer, uten avhengigheter, som
 * fester oppførsel på markup serveren har sendt. Det er noe man kan overta
 * og vedlikeholde selv.
 *
 * Fila her gjør ingenting med filsystemet, bare med tekst, slik at
 * omskrivingen kan etterprøves i en vanlig test. `cli.ts` leser og skriver
 * filene.
 *
 * Det som må skrives om er henvisningene ut av mappa. En komponent peker på
 * naboene sine med relative stier, og de stiene finnes ikke lenger når mappa
 * står i et annet prosjekt. `../shared.js` blir til
 * `@fristil/designsystem/shared`, som fortsatt virker, og som gjør det synlig
 * hva kopien fremdeles henter fra pakken.
 */
/**
 * Bygger oppslaget fra en fil i pakken til inngangspunktet som peker på den.
 *
 * `exports` peker på `dist` for JavaScript og på `src` for komponentenes
 * CSS. Begge føres tilbake til kilden, siden det er kilden som kopieres.
 * `fristil.css`, den samlede fila, ligger i `dist` og importeres aldri fra en
 * komponent, så oppslaget den gir treffer ingenting.
 */
export function buildEntryPoints(name, exports) {
    const entries = new Map();
    for (const [subpath, target] of Object.entries(exports)) {
        const file = typeof target === "string" ? target : target.import;
        if (!file)
            continue;
        const source = file
            .replace(/^\.\//, "")
            .replace(/^dist\//, "src/")
            .replace(/\.js$/, ".ts");
        entries.set(source, `${name}${subpath.replace(/^\./, "")}`);
    }
    return entries;
}
/** Slår sammen en relativ sti med mappa den står i. */
export function resolvePath(fromDirectory, relative) {
    const parts = fromDirectory.split("/").filter(Boolean);
    for (const part of relative.split("/")) {
        if (part === "." || part === "")
            continue;
        if (part === "..")
            parts.pop();
        else
            parts.push(part);
    }
    return parts.join("/");
}
const IMPORT_PATTERN = /(from\s+|@import\s+)(["'])([^"']+)\2/g;
/**
 * Skriver om henvisningene i én fil.
 *
 * Bare stier som peker ut av mappa røres. En nabo i samme mappe blir med i
 * kopien, og skal fortsatt finnes der.
 */
export function rewriteReferences(file, entryPoints, 
/** Stiene til filene som blir med i kopien, som de står i pakken. */
ownFiles) {
    const directory = file.path.slice(0, file.path.lastIndexOf("/"));
    const rewrites = [];
    const content = file.content.replace(IMPORT_PATTERN, (match, lead, quote, specifier) => {
        if (!specifier.startsWith("."))
            return match;
        const target = resolvePath(directory, specifier);
        // En fil som blir med i kopien skal stå urørt. Sammenligningen går på
        // hele stien: filnavnet alene ville latt `../shared.js` stå så snart
        // mappa selv hadde en `shared.ts`, og kopien hadde pekt ut av seg selv.
        // Importen i koden peker dessuten på «.js», altså filen etter bygging,
        // mens kilden heter «.ts».
        const asSource = target.replace(/\.js$/, ".ts");
        if (ownFiles.has(target) || ownFiles.has(asSource))
            return match;
        const entry = entryPoints.get(target) ??
            entryPoints.get(target.replace(/\.js$/, ".ts"));
        if (!entry)
            return match;
        rewrites.push({ from: specifier, to: entry });
        return `${lead}${quote}${entry}${quote}`;
    });
    return { name: file.path.split("/").pop() ?? file.path, content, rewrites };
}
/**
 * Setter sammen hele planen for én komponent.
 *
 * `files` er alle filene i komponentmappa, uten testene: de hører til
 * pakkens eget oppsett og sier ingenting i et annet prosjekt.
 */
export function planTakeover(files, entryPoints) {
    const ownFiles = new Set(files.map((file) => file.path));
    const planned = files.map((file) => rewriteReferences(file, entryPoints, ownFiles));
    const dependencies = new Set();
    const keptImports = new Set();
    for (const file of planned) {
        for (const match of file.content.matchAll(IMPORT_PATTERN)) {
            const specifier = match[3];
            if (specifier.startsWith("."))
                continue;
            if (specifier.startsWith("@fristil/")) {
                keptImports.add(specifier);
                continue;
            }
            const parts = specifier.split("/");
            dependencies.add(specifier.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0]);
        }
    }
    const replacedEntries = files
        .map((file) => entryPoints.get(file.path))
        .filter((entry) => entry !== undefined);
    return {
        files: planned,
        dependencies: [...dependencies].sort(),
        keptImports: [...keptImports].sort(),
        replacedEntries: replacedEntries.sort(),
    };
}
//# sourceMappingURL=takeover.js.map