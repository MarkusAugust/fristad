/// <reference types="node" />
/**
 * The host for the command line in Node: runs the WASI module `lotek.wasm`
 * with Node's own `node:wasi`. Used by `cli.ts`, and by the VS Code
 * extension, which starts the language server, `lotek lsp`, this way.
 *
 * The commands are written once, in Rust (`cli/`). The same code is the
 * executable from `cargo install` and what `java -jar` runs, so the answers
 * are the same everywhere. The host does only what WASI cannot:
 *
 * - opens the folders the module reads and writes in, the working directory
 *   and the rest of the file system, so both relative and absolute paths work;
 * - fetches the addresses `lotek check` gets, since WASI has no network, and
 *   gives the page to the module as a file (see `cli/src/fetch.rs`);
 * - says when standard input is a terminal, which WASI cannot see.
 */
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync, } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { isatty } from "node:tty";
import { fileURLToPath } from "node:url";
const windows = process.platform === "win32";
/**
 * A path as the WASI module sees it.
 *
 * On Windows every drive is opened as its own folder, `C:\` as `/c`, and
 * the backslashes become slashes. Elsewhere the path is the same.
 */
function forWasi(path) {
    if (!windows)
        return path;
    const absolute = /^([A-Za-z]):[\\/](.*)$/.exec(path);
    if (absolute)
        return `/${absolute[1].toLowerCase()}/${absolute[2].replace(/\\/g, "/")}`;
    return path.replace(/\\/g, "/");
}
/** An argument for the WASI module: the path in a file or in a `--flag=value`. */
function toModule(part) {
    const flag = /^(--[^=]+=)(.*)$/s.exec(part);
    return flag ? flag[1] + forWasi(flag[2]) : forWasi(part);
}
/**
 * Runs one command, like `["check", "form.html"]`, and gives the exit code.
 * `module` is the path to `lotek.wasm`, and `packageDir` the folder with the
 * package's files: the manifest, the colour contract, the rulebooks and the
 * components. The command line carries none of them, and gets the folder as
 * `LOTEK_PACKAGE`.
 */
export async function runCommandLine(module, args, packageDir) {
    /*
     * The whole file system is opened, and the module is told where the working
     * directory is and goes there itself. A WASI module otherwise starts in `/`,
     * and a relative path was read from the root.
     */
    const preopens = {};
    if (windows) {
        // Only the drives that exist. A folder that cannot be opened stops
        // `node:wasi` before the module starts, and most machines have only C:.
        for (const letter of "abcdefghijklmnopqrstuvwxyz") {
            const drive = `${letter.toUpperCase()}:\\`;
            if (existsSync(drive))
                preopens[`/${letter}`] = drive;
        }
    }
    else
        preopens["/"] = "/";
    /*
     * The addresses are fetched here and given to the module as files.
     *
     * The file has the address, the status code (0 when nobody answered), the
     * content type or the reason, and the page, in that order. The module
     * judges the answer: whether it is a page and whether the status code is
     * fine.
     */
    let fetchedDir;
    const warn = process.emitWarning;
    try {
        const address = /^https?:\/\//i;
        const moduleArgs = [];
        for (const [index, part] of args.entries()) {
            if (args[0] !== "check" || index === 0 || !address.test(part)) {
                moduleArgs.push(toModule(part));
                continue;
            }
            fetchedDir ??= mkdtempSync(join(tmpdir(), "lotek-"));
            let answer;
            try {
                // As in the executable: a server that never answers must not hang a
                // CI job until it is killed.
                const response = await fetch(part, {
                    signal: AbortSignal.timeout(30_000),
                });
                const type = response.headers.get("content-type") ?? "";
                answer = `${part}\n${response.status}\n${type}\n${await response.text()}`;
            }
            catch (error) {
                const reason = error instanceof Error ? error.message : String(error);
                answer = `${part}\n0\n${reason.replace(/\n/g, " ")}\n`;
            }
            const file = join(fetchedDir, `${index}.txt`);
            writeFileSync(file, answer);
            moduleArgs.push(`--fetched=${forWasi(file)}`);
        }
        // `node:wasi` is marked experimental, and says so in the console. That is
        // nothing the user of the command line can do anything about.
        process.emitWarning = ((warning, ...rest) => {
            if (String(warning).includes("WASI"))
                return;
            warn.call(process, warning, ...rest);
        });
        const { WASI } = await import("node:wasi");
        const wasi = new WASI({
            version: "preview1",
            args: ["lotek", ...moduleArgs],
            env: {
                LOTEK_WORKDIR: forWasi(process.cwd()),
                LOTEK_PACKAGE: forWasi(packageDir instanceof URL ? fileURLToPath(packageDir) : packageDir),
                // So the messages show the paths as `C:\\…`, not as `/c/…`.
                ...(windows ? { LOTEK_WINDOWS: "1" } : {}),
                // `isatty(0)`, not `process.stdin.isTTY`: touching `process.stdin`
                // puts a pipe in non-blocking mode, and the module then gave up
                // reading before the writer was done, as in `curl … | lotek check`.
                ...(isatty(0) ? { LOTEK_TERMINAL: "1" } : {}),
            },
            preopens,
            returnOnExit: true,
        });
        const compiled = await WebAssembly.compile(readFileSync(module));
        const instance = await WebAssembly.instantiate(compiled, wasi.getImportObject());
        return wasi.start(instance);
    }
    finally {
        // Also when the module crashes: the fetched pages must not be left
        // behind, and the warnings come back for the rest of the process.
        process.emitWarning = warn;
        if (fetchedDir)
            rmSync(fetchedDir, { recursive: true, force: true });
    }
}
//# sourceMappingURL=wasi-host.js.map