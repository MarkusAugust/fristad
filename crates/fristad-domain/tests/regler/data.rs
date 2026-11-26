//! Byggesettet og nivådokumentene fra `assets/`, lest fra disken når testene
//! kjører. I spillet gjør `adapters::data` det samme med filene bygd inn;
//! begge setter byggesettet sammen med `KitDef::assemble`.

use crate::domain::world::crypt::doc::{LevelDoc, ModuleDef};
use crate::domain::world::crypt::kit::{KitDef, KitFile};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

/// Json-filene i en mappe, sortert etter navn.
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect()).unwrap_or_default();
    out.sort();
    out
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Byggesettet for nivåredigeringen: alle manifestene slått sammen.
pub fn kit() -> KitDef {
    static KIT: OnceLock<KitDef> = OnceLock::new();
    KIT.get_or_init(|| {
        let files = json_files(&assets().join("models/crypt/kit")).iter().map(|p| read::<KitFile>(p)).collect::<Vec<_>>();
        let dir = assets().join("levels/moduler");
        let mut themes: Vec<PathBuf> = std::fs::read_dir(&dir).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
        themes.sort();
        let mut modules = vec![];
        for theme in themes {
            for p in json_files(&theme) {
                let name = p.strip_prefix(&dir).unwrap().with_extension("").to_string_lossy().replace('\\', "/");
                modules.push((name, read::<ModuleDef>(&p)));
            }
        }
        KitDef::assemble(files, modules, read(&assets().join("levels/steiner.json")))
    })
    .clone()
}

/// Nivådokumentet `assets/levels/nivaa_<navn>.json`.
pub fn level_doc(name: &str) -> Option<LevelDoc> {
    let path = assets().join(format!("levels/nivaa_{name}.json"));
    path.exists().then(|| read(&path))
}
