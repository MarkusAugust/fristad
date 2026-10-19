//! Finner kryptnivåene og modellene deres før kompileringen.
//!
//! - `crypt_levels.rs`: `LEVEL_FILES`, json-fila for hvert nivå n som finnes,
//!   fra 1 og oppover uten hull. Det ekte nivået (`assets/models/crypt/
//!   crypt_NN.json`, fra Blender) brukes når det finnes, ellers testnivået i
//!   `assets/crypt_test/`. Kjernen (`domain/world/crypt`) tar dem inn med
//!   `include_str!`, som byen gjør med `<navn>_inside.json`.
//! - `crypt_models.rs`: `CRYPT_FILES` (navn → sti under `assets/models/`) og
//!   `CRYPT_MODELS` (bygd inn utenom nettleseren) for alle `.glb` i
//!   `assets/models/crypt/`, til glTF-adapteren. Bitene og byggesettet trenger
//!   derfor ingen linje i `models!`.
//! - `level_docs.rs`: `KIT_FILES` (manifestene til byggesettet,
//!   `assets/models/crypt/kit/<tema>.json`) og `LEVEL_DOC_FILES` (nivå-
//!   dokumentene `assets/levels/nivaa_<navn>.json`), til `adapters/data.rs`.
//!   Bitene i `assets/models/crypt/kit/<tema>/` er med i `crypt_models.rs`.
//! - `inside_files.rs`: `INSIDE_FILES`, alle `<navn>_inside.json` i
//!   `assets/models/town/` (det husene har med seg fra Blender), til
//!   `adapters/data.rs`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let real = root.join("assets/models/crypt");
    let test = root.join("assets/crypt_test");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", real.display());
    println!("cargo:rerun-if-changed={}", test.display());

    let mut levels = String::from("/// Nivåene som finnes: nummeret og json-fila.\npub const LEVEL_FILES: &[(u32, &str)] = &[\n");
    for n in 1..=10u32 {
        let name = format!("crypt_{n:02}.json");
        let Some(p) = [real.join(&name), test.join(&name)].into_iter().find(|p| p.exists()) else {
            break;
        };
        println!("cargo:rerun-if-changed={}", p.display());
        writeln!(levels, "    ({n}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    levels.push_str("];\n");
    std::fs::write(out.join("crypt_levels.rs"), levels).unwrap();

    // Kryptmodellene og møblene (`assets/models/furniture/`, skilt ut av
    // husene) tas inn uten linje i `models!`.
    let furniture = root.join("assets/models/furniture");
    println!("cargo:rerun-if-changed={}", furniture.display());
    // Byggesettet for nivåredigeringen: bitene per tema og manifestene.
    let kit = real.join("kit");
    println!("cargo:rerun-if-changed={}", kit.display());
    let mut kit_dirs: Vec<PathBuf> = vec![];
    let mut kit_files: Vec<PathBuf> = vec![];
    if let Ok(dir) = std::fs::read_dir(&kit) {
        for e in dir.flatten() {
            let p = e.path();
            if p.is_dir() {
                println!("cargo:rerun-if-changed={}", p.display());
                kit_dirs.push(p);
            } else if p.extension().is_some_and(|x| x == "json") {
                println!("cargo:rerun-if-changed={}", p.display());
                kit_files.push(p);
            }
        }
    }
    kit_dirs.sort();
    kit_files.sort();
    let mut glbs: Vec<(String, PathBuf)> = vec![];
    for d in [&real, &furniture].into_iter().chain(kit_dirs.iter()) {
        let Ok(dir) = std::fs::read_dir(d) else {
            continue;
        };
        for e in dir.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "glb") {
                println!("cargo:rerun-if-changed={}", p.display());
                glbs.push((p.file_stem().unwrap().to_string_lossy().to_string(), p));
            }
        }
    }
    glbs.sort();
    let models_dir = root.join("assets/models");
    let rel = |p: &Path| p.strip_prefix(&models_dir).unwrap().to_string_lossy().to_string();
    let mut models = String::from("/// Kryptmodellene: navn og fila under `assets/models/`.\npub const CRYPT_FILES: &[(&str, &str)] = &[\n");
    for (n, p) in &glbs {
        writeln!(models, "    ({n:?}, {:?}),", rel(p)).unwrap();
    }
    models.push_str("];\n#[cfg(not(target_arch = \"wasm32\"))]\nconst CRYPT_MODELS: &[(&str, &[u8])] = &[\n");
    for (n, p) in &glbs {
        writeln!(models, "    ({n:?}, include_bytes!({:?}) as &[u8]),", p.display().to_string()).unwrap();
    }
    models.push_str("];\n");
    std::fs::write(out.join("crypt_models.rs"), models).unwrap();

    // Nivådokumentene (`assets/levels/nivaa_<navn>.json`).
    let docs_dir = root.join("assets/levels");
    println!("cargo:rerun-if-changed={}", docs_dir.display());
    let mut docs: Vec<(String, PathBuf)> = vec![];
    if let Ok(dir) = std::fs::read_dir(&docs_dir) {
        for e in dir.flatten() {
            let p = e.path();
            let stem = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if let Some(name) = stem.strip_prefix("nivaa_").filter(|_| p.extension().is_some_and(|x| x == "json")) {
                println!("cargo:rerun-if-changed={}", p.display());
                docs.push((name.to_string(), p));
            }
        }
    }
    docs.sort();
    let mut text = String::from("/// Manifestene til byggesettet: temaet og json-fila.\nconst KIT_FILES: &[(&str, &str)] = &[\n");
    for p in &kit_files {
        writeln!(text, "    ({:?}, include_str!({:?})),", p.file_stem().unwrap().to_string_lossy(), p.display().to_string()).unwrap();
    }
    text.push_str("];\n/// Nivådokumentene: navnet (`01`, `test`) og json-fila.\nconst LEVEL_DOC_FILES: &[(&str, &str)] = &[\n");
    for (n, p) in &docs {
        writeln!(text, "    ({n:?}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    text.push_str("];\n");
    std::fs::write(out.join("level_docs.rs"), text).unwrap();

    let town = root.join("assets/models/town");
    println!("cargo:rerun-if-changed={}", town.display());
    let mut insides: Vec<(String, PathBuf)> = vec![];
    for d in [&town, &furniture] {
        let Ok(dir) = std::fs::read_dir(d) else {
            continue;
        };
        for e in dir.flatten() {
            let p = e.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if let Some(stem) = name.strip_suffix("_inside.json") {
                println!("cargo:rerun-if-changed={}", p.display());
                insides.push((stem.to_string(), p));
            }
        }
    }
    insides.sort();
    let mut text = String::from("/// Det husene har med seg: navnet og json-fila.\nconst INSIDE_FILES: &[(&str, &str)] = &[\n");
    for (n, p) in &insides {
        writeln!(text, "    ({n:?}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    text.push_str("];\n");
    std::fs::write(out.join("inside_files.rs"), text).unwrap();

    // Det som står inne i husene, i husets rom (`assets/layout/inside/`).
    let inside = root.join("assets/layout/inside");
    println!("cargo:rerun-if-changed={}", inside.display());
    let mut contents: Vec<(String, PathBuf)> = vec![];
    if let Ok(dir) = std::fs::read_dir(&inside) {
        for e in dir.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "json") {
                println!("cargo:rerun-if-changed={}", p.display());
                contents.push((p.file_stem().unwrap().to_string_lossy().to_string(), p));
            }
        }
    }
    contents.sort();
    let mut text = String::from("/// Det som står inne i husene: huset og json-fila.\nconst CONTENT_FILES: &[(&str, &str)] = &[\n");
    for (n, p) in &contents {
        writeln!(text, "    ({n:?}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    text.push_str("];\n");
    std::fs::write(out.join("content_files.rs"), text).unwrap();

    // Det som står i kryptnivåene (`assets/layout/crypt_NN.json`: merkene og
    // lysene), som redigeringen endrer.
    let layout_dir = root.join("assets/layout");
    println!("cargo:rerun-if-changed={}", layout_dir.display());
    let mut crypts: Vec<(u32, PathBuf)> = vec![];
    if let Ok(dir) = std::fs::read_dir(&layout_dir) {
        for e in dir.flatten() {
            let p = e.path();
            let stem = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if let Some(n) = stem.strip_prefix("crypt_").and_then(|n| n.parse::<u32>().ok()).filter(|_| p.extension().is_some_and(|x| x == "json")) {
                println!("cargo:rerun-if-changed={}", p.display());
                crypts.push((n, p));
            }
        }
    }
    crypts.sort();
    let mut text = String::from("/// Det som står i kryptnivåene: nivået og json-fila.\nconst CRYPT_CONTENT_FILES: &[(u32, &str)] = &[\n");
    for (n, p) in &crypts {
        writeln!(text, "    ({n}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    text.push_str("];\n");
    std::fs::write(out.join("crypt_content_files.rs"), text).unwrap();

    // Byen som data (`assets/layout/town.json`), om den er skrevet ut.
    let layout = root.join("assets/layout/town.json");
    println!("cargo:rerun-if-changed={}", layout.display());
    let text =
        if layout.exists() { format!("const LAYOUT_FILE: Option<&str> = Some(include_str!({:?}));\n", layout.display().to_string()) } else { "const LAYOUT_FILE: Option<&str> = None;\n".to_string() };
    std::fs::write(out.join("layout_file.rs"), text).unwrap();
}
