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
        let Some(p) = [real.join(&name), test.join(&name)].into_iter().find(|p| p.exists()) else { break };
        println!("cargo:rerun-if-changed={}", p.display());
        writeln!(levels, "    ({n}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    levels.push_str("];\n");
    std::fs::write(out.join("crypt_levels.rs"), levels).unwrap();

    let mut glbs: Vec<(String, PathBuf)> = vec![];
    if let Ok(dir) = std::fs::read_dir(&real) {
        for e in dir.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "glb") {
                println!("cargo:rerun-if-changed={}", p.display());
                glbs.push((p.file_stem().unwrap().to_string_lossy().to_string(), p));
            }
        }
    }
    glbs.sort();
    let rel = |p: &Path| format!("crypt/{}", p.file_name().unwrap().to_string_lossy());
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

    let town = root.join("assets/models/town");
    println!("cargo:rerun-if-changed={}", town.display());
    let mut insides: Vec<(String, PathBuf)> = vec![];
    if let Ok(dir) = std::fs::read_dir(&town) {
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
}
