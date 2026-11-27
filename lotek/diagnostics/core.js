/**
 * The core: LO-TEK's check and theme generator, written in Rust and built to
 * a single WebAssembly module.
 *
 * The same module runs here, in VS Code, on the JVM (Chicory) and in
 * IntelliJ, and gives the same answer everywhere. The module has no imports,
 * so it needs no glue: the loader writes the text in as UTF-8 and reads the
 * answer out as JSON, as every host language does.
 *
 * The core carries no data. The vocabulary and the colour contract are given
 * to it with `loadManifest` and `loadContract`, from the files in the package.
 *
 * `diagnoseMarkup` and `diagnosePage` in `index.ts` load the module and the
 * files from the package themselves. `loadCore` is for anyone who has the
 * module some other way, such as an extension that bundles it, or a browser:
 *
 * ```ts
 * const core = loadCore(await WebAssembly.compileStreaming(fetch(url)))
 * core.loadManifest(await (await fetch(manifestUrl)).text())
 * core.diagnosePage(html)
 * ```
 *
 * Synchronous, because Node, Bun and VS Code allow it. In the browser the
 * module must be compiled with `WebAssembly.compile` first, and the compiled
 * module passed here.
 */
/** Loads the core from the module's bytes or an already compiled module. */
export function loadCore(source) {
    const module = source instanceof WebAssembly.Module
        ? source
        : new WebAssembly.Module(source);
    const e = new WebAssembly.Instance(module, {}).exports;
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    /** Writes the text into the module's memory and returns the pointer and length. */
    const write = (text) => {
        const bytes = encoder.encode(text);
        const pointer = e.alloc(bytes.length);
        // The memory can grow on every call, so the view is created after `alloc`.
        new Uint8Array(e.memory.buffer, pointer, bytes.length).set(bytes);
        return [pointer, bytes.length];
    };
    const read = () => JSON.parse(decoder.decode(new Uint8Array(e.memory.buffer, e.result_ptr(), e.result_len())));
    /** The answer from a call that can be rejected: the value, or the core's explanation as an error. */
    const answer = (status) => {
        if (status !== 0)
            throw new Error(read().error);
        return read();
    };
    return {
        diagnoseMarkup: (html) => answer(e.diagnose_markup_raw(...write(html))),
        diagnosePage: (html) => answer(e.diagnose_page_raw(...write(html))),
        loadManifest: (json) => {
            answer(e.load_manifest_raw(...write(json)));
        },
        loadContract: (json) => {
            answer(e.load_contract_raw(...write(json)));
        },
        version: () => {
            e.version_raw();
            return read();
        },
        buildTheme: (recipe) => answer(e.build_theme_raw(...write(JSON.stringify(recipe)))),
        inspectTheme: (css) => answer(e.inspect_theme_raw(...write(css))),
        inspectStyles: (css) => {
            e.inspect_styles_raw(...write(css));
            return read();
        },
        diagnoseStyled: (html, css, page) => answer(e.diagnose_styled_raw(...write(JSON.stringify({ html, css, page })))),
    };
}
//# sourceMappingURL=core.js.map