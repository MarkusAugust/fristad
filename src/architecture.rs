//! Arkitekturen som test. Kjernen, portene og applikasjonen er egne pakker
//! (`crates/`), så kompilatoren hindrer dem i å bruke en pakke de ikke har
//! som avhengighet. Testen passer på at ingen legger til en avhengighet som
//! ikke hører hjemme, og at ingen går rundt pakkesystemet med det standard-
//! biblioteket alltid har (filer, miljø, klokke, tråder) eller ved å lese
//! filer når den kompileres.
//!
//! - `fristad-domain`: bare matematikk (`glam`, `bytemuck`), `serde` og `log`.
//! - `fristad-ports`: kjernen og `glam`.
//! - `fristad-app`: kjernen, portene, `glam` og `log`.
//! - Hovedpakken (adapterne og vertene) får bruke alt.

use std::path::{Path, PathBuf};

/// Pakkene og avhengighetene de har lov til.
const RULES: &[(&str, &[&str])] =
    &[("fristad-domain", &["glam", "bytemuck", "serde", "log"]), ("fristad-ports", &["fristad-domain", "glam"]), ("fristad-app", &["fristad-domain", "fristad-ports", "glam", "log"])];

/// Det de indre pakkene ikke får bruke fra standardbiblioteket, og ikke
/// får gjøre når de kompileres.
const FORBIDDEN: &[&str] = &["fs", "env", "process", "net", "thread", "time"];
const MACROS: &[&str] = &["include_str!", "include_bytes!", "include!", "env!", "option_env!"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Navnene i `[dependencies]` i en `Cargo.toml` (også målrettede tabeller
/// som `[target.….dependencies]`), men ikke `[dev-dependencies]`.
fn dependencies(manifest: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(manifest).unwrap();
    let mut out = vec![];
    let mut inside = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            inside = l.ends_with("dependencies]") && !l.contains("dev-dependencies") && !l.contains("build-dependencies");
            continue;
        }
        if inside && !l.is_empty() && !l.starts_with('#') {
            if let Some((name, _)) = l.split_once('=') {
                out.push(name.trim().to_string());
            }
        }
    }
    out
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Kildeteksten uten kommentarer (linjekommentarer og blokkommentarer), med
/// linjene beholdt så linjenumrene stemmer.
fn code(path: &Path) -> String {
    let src = std::fs::read_to_string(path).unwrap();
    let mut out = String::with_capacity(src.len());
    let b = src.as_bytes();
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            in_str = c != '"';
            i += 1;
        } else if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
        } else if src[i..].starts_with("//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if src[i..].starts_with("/*") {
            while i < b.len() && !src[i..].starts_with("*/") {
                if b[i] == b'\n' {
                    out.push('\n');
                }
                i += 1;
            }
            i += 2;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

#[test]
fn pakkene_har_bare_lovlige_avhengigheter() {
    for (krate, allowed) in RULES {
        let deps = dependencies(&root().join("crates").join(krate).join("Cargo.toml"));
        for d in &deps {
            assert!(allowed.contains(&d.as_str()), "{krate} har avhengigheten {d}, som den ikke får ha (lov: {allowed:?})");
        }
    }
}

#[test]
fn kjernen_bruker_ikke_filer_miljo_eller_klokke() {
    let mut bad = vec![];
    for (krate, _) in RULES {
        let mut all = vec![];
        files(&root().join("crates").join(krate).join("src"), &mut all);
        for path in all {
            let text = code(&path);
            for (n, line) in text.lines().enumerate() {
                let compact: String = line.split_whitespace().collect();
                for m in FORBIDDEN {
                    // `std::fs`, `std::{fs, …}` og `std::{…, fs}`.
                    let direct = compact.contains(&format!("std::{m}::")) || compact.contains(&format!("std::{m};")) || compact.contains(&format!("std::{m}}}"));
                    let grouped =
                        compact.contains("std::{") && compact.split("std::{").skip(1).any(|g| g.split('}').next().unwrap_or("").split(',').any(|x| x == *m || x.starts_with(&format!("{m}::"))));
                    if direct || grouped {
                        bad.push(format!("{}:{}: std::{m}", path.display(), n + 1));
                    }
                }
                for m in MACROS {
                    if compact.contains(m) && !compact.contains(&format!("\"{m}")) {
                        bad.push(format!("{}:{}: {m}", path.display(), n + 1));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "Kjernen, portene og applikasjonen skal ikke lese filer, miljø eller klokke:\n{}", bad.join("\n"));
}

/// Største lengde (linjer) på en fil i kjernen, portene og applikasjonen.
const MAX_LINES: usize = 800;

/// Filene som var lengre da grensen kom (11. oktober), med lengden da som
/// tak. De kan bare krympe; når en er under grensen, skal den strykes her.
const TOO_LONG: &[(&str, usize)] = &[("fristad-domain/src/domain/mobs.rs", 1121), ("fristad-domain/src/domain/geometry.rs", 827)];

#[test]
fn filene_er_ikke_for_lange() {
    let mut bad = vec![];
    for (krate, _) in RULES {
        let mut all = vec![];
        files(&root().join("crates").join(krate).join("src"), &mut all);
        for path in all {
            let lines = std::fs::read_to_string(&path).unwrap().lines().count();
            let rel = path.strip_prefix(root().join("crates")).unwrap().to_string_lossy().replace('\\', "/");
            match TOO_LONG.iter().find(|(f, _)| *f == rel) {
                Some((_, cap)) if lines > *cap => bad.push(format!("{rel}: {lines} linjer, mer enn taket {cap}")),
                Some(_) if lines <= MAX_LINES => bad.push(format!("{rel}: {lines} linjer, under grensen; stryk den fra TOO_LONG")),
                Some(_) => {}
                None if lines > MAX_LINES => bad.push(format!("{rel}: {lines} linjer, mer enn {MAX_LINES}; del den etter oppgave")),
                None => {}
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Glb-lasteren gjetter ikke ut fra navn: rollene til materialene og
/// modellene står som data i fila (`extras.fristad`, `tools/models/roles.py`).
#[test]
fn lasteren_leser_rollene_og_gjetter_ikke() {
    let text = code(&root().join("src/adapters/gltf.rs"));
    let bad: Vec<_> = text
        .lines()
        .enumerate()
        .filter(|(_, l)| ["starts_with(\"", "ends_with(\"", "strip_prefix(\"", "contains(\""].iter().any(|m| l.contains(m)))
        .map(|(n, l)| format!("{}: {}", n + 1, l.trim()))
        .collect();
    assert!(bad.is_empty(), "gltf.rs sammenligner navn; legg regelen i tools/models/roles.py i stedet:\n{}", bad.join("\n"));
}
