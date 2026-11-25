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
        } else if p.extension().is_some_and(|x| x == "rs" || x == "js") {
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

/// Største lengde (linjer) på en kildefil: Rust i alle pakkene og
/// nettlaget i `web/`.
const MAX_LINES: usize = 800;

/// Største bredde (tegn) på en linje, den samme som `rustfmt` får
/// (`max_width=200`). Linjetaket alene kunne tas ut i bredden.
const MAX_WIDTH: usize = 200;

/// Filene som var for lange da grensen kom (kjernen 11. oktober, resten 25.
/// november), med lengden da som tak. De kan bare krympe; når en er under
/// grensen, skal den strykes her.
#[rustfmt::skip]
const TOO_LONG: &[(&str, usize)] = &[
    ("crates/fristad-domain/src/domain/geometry.rs", 802),
    ("crates/fristad-domain/src/domain/mobs.rs", 887),
    ("src/edit_tests.rs", 1002),
    ("src/tests.rs", 3619),
    ("web/editor.js", 1035),
    ("web/main.js", 3121),
    ("web/nivaa.js", 2163),
];

/// Filene som hadde linjer over `MAX_WIDTH` da grensen kom (25. november),
/// med antallet. Det kan bare synke: blir det færre, skal tallet senkes, og
/// uten noen strykes fila her.
#[rustfmt::skip]
const TOO_WIDE: &[(&str, usize)] = &[
    ("crates/fristad-app/src/application/app.rs", 1),
    ("crates/fristad-app/src/application/audio.rs", 1),
    ("crates/fristad-app/src/application/fight.rs", 1),
    ("crates/fristad-app/src/application/usage.rs", 3),
    ("crates/fristad-app/src/application/view.rs", 2),
    ("crates/fristad-domain/src/domain/combat.rs", 1),
    ("crates/fristad-domain/src/domain/craft.rs", 16),
    ("crates/fristad-domain/src/domain/drops.rs", 2),
    ("crates/fristad-domain/src/domain/geometry.rs", 1),
    ("crates/fristad-domain/src/domain/herd.rs", 1),
    ("crates/fristad-domain/src/domain/mob_bolts.rs", 3),
    ("crates/fristad-domain/src/domain/mob_kind.rs", 37),
    ("crates/fristad-domain/src/domain/mobs.rs", 4),
    ("crates/fristad-domain/src/domain/npcs.rs", 3),
    ("crates/fristad-domain/src/domain/world/cast.rs", 1),
    ("crates/fristad-domain/src/domain/world/crypt/compose/check.rs", 4),
    ("crates/fristad-domain/src/domain/world/crypt/compose/look.rs", 1),
    ("crates/fristad-domain/src/domain/world/crypt/compose/mod.rs", 2),
    ("crates/fristad-domain/src/domain/world/crypt/compose/rooms.rs", 1),
    ("crates/fristad-domain/src/domain/world/crypt/mod.rs", 3),
    ("crates/fristad-domain/src/domain/world/layout.rs", 1),
    ("crates/fristad-domain/src/domain/world/town.rs", 1),
    ("src/adapters/native.rs", 1),
    ("src/adapters/web.rs", 1),
    ("src/adapters/wgpu/frame.rs", 1),
    ("src/adapters/wgpu/mod.rs", 2),
    ("src/adapters/wgpu/models.rs", 1),
    ("src/adapters/wgpu/passes.rs", 2),
    ("src/adapters/wgpu/pipes.rs", 2),
    ("src/adapters/wgpu/setup.rs", 1),
    ("src/adapters/wgpu/texture.rs", 4),
    ("src/adapters/wgpu/timer.rs", 3),
    ("src/adapters/wire.rs", 6),
    ("src/bin/snapshot.rs", 4),
    ("src/crypt_tests.rs", 3),
    ("src/level_doc_tests.rs", 3),
    ("src/mechanism_tests.rs", 4),
    ("src/tests.rs", 8),
    ("web/editor.js", 9),
    ("web/main.js", 20),
    ("web/nivaa-plan.js", 1),
    ("web/nivaa.js", 16),
];

/// Kildefilene reglene gjelder: Rust i `crates/`, `src/` og `build.rs`, og
/// JavaScript i `web/`.
fn sources() -> Vec<PathBuf> {
    let mut all = vec![root().join("build.rs")];
    files(&root().join("crates"), &mut all);
    files(&root().join("src"), &mut all);
    files(&root().join("web"), &mut all);
    all.retain(|p| !p.components().any(|c| c.as_os_str() == "target"));
    all.sort();
    all
}

fn relative(path: &Path) -> String {
    path.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/")
}

#[test]
fn filene_er_ikke_for_lange() {
    let mut bad = vec![];
    for path in sources() {
        let lines = std::fs::read_to_string(&path).unwrap().lines().count();
        let rel = relative(&path);
        match TOO_LONG.iter().find(|(f, _)| *f == rel) {
            Some((_, cap)) if lines > *cap => bad.push(format!("{rel}: {lines} linjer, mer enn taket {cap}")),
            Some(_) if lines <= MAX_LINES => bad.push(format!("{rel}: {lines} linjer, under grensen; stryk den fra TOO_LONG")),
            Some(_) => {}
            None if lines > MAX_LINES => bad.push(format!("{rel}: {lines} linjer, mer enn {MAX_LINES}; del den etter oppgave")),
            None => {}
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Ingen nye linjer over `MAX_WIDTH`: en fil kan ikke holde seg under
/// linjetaket ved å vokse i bredden.
#[test]
fn linjene_er_ikke_for_brede() {
    let mut bad = vec![];
    for path in sources() {
        // Denne fila har listene over de andre.
        if path.ends_with("src/architecture.rs") {
            continue;
        }
        let wide = std::fs::read_to_string(&path).unwrap().lines().filter(|l| l.chars().count() > MAX_WIDTH).count();
        let rel = relative(&path);
        match TOO_WIDE.iter().find(|(f, _)| *f == rel).map(|(_, n)| *n) {
            Some(cap) if wide > cap => bad.push(format!("{rel}: {wide} linjer over {MAX_WIDTH} tegn, mer enn taket {cap}; del linjene")),
            Some(cap) if wide < cap => bad.push(format!("{rel}: {wide} linjer over {MAX_WIDTH} tegn, færre enn taket {cap}; senk det i TOO_WIDE{}", if wide == 0 { " (stryk fila)" } else { "" })),
            Some(_) => {}
            None if wide > 0 => bad.push(format!("{rel}: {wide} linjer over {MAX_WIDTH} tegn; del dem")),
            None => {}
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
