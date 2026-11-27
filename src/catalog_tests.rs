//! Katalogen i redigeringen av byen (`?edit`, `data::category`): hva som
//! vises, og at alt som vises, har et bilde (eieren 26. november).

use crate::adapters::data::{buildings, category, waiting};
use crate::adapters::gltf::model_files;

/// Hvert kort i katalogen har et bilde (`assets/catalog/thumbs/<navn>.jpg`,
/// `python3.13 tools/qa/thumbs.py`): en ny modell uten bilde stopper her.
#[test]
fn alt_i_katalogen_har_bilde() {
    let root = env!("CARGO_MANIFEST_DIR");
    let houses = buildings();
    let pictured = |name: &str| std::path::Path::new(&format!("{root}/assets/catalog/thumbs/{name}.jpg")).exists();
    let missing: Vec<&str> = model_files().filter(|(name, path)| category(name, path, &houses).is_some()).map(|(name, _)| *name).filter(|name| !pictured(name)).collect();
    assert!(missing.is_empty(), "uten bilde i katalogen (python3.13 tools/qa/thumbs.py): {missing:?}");
}

/// De løse bitene og mekanismene i krypten, armene, fiendene og innbyggerne
/// står ikke i katalogen: de settes ikke ut som ting i byen.
#[test]
fn katalogen_skjuler_det_som_ikke_settes_ut_i_byen() {
    let houses = buildings();
    for (name, path) in model_files() {
        let hidden = ["crypt/", "arms/", "mobs/", "npcs/"].iter().any(|f| path.starts_with(f));
        assert!(!hidden || category(name, path, &houses).is_none(), "{name} ({path}) skal ikke stå i katalogen");
    }
    assert_eq!(category("barrel", "town/barrel.glb", &houses), Some("Gjenstander"));
    // Krypten på kirkegården er et bygg i byen, ikke en bit av nivåene.
    assert_eq!(category("crypt", "town/crypt.glb", &houses), Some("Hus"));
}

/// Lista over det som venter på dom (`assets/catalog/venter.json`) kan leses.
#[test]
fn lista_over_det_som_venter_kan_leses() {
    let _ = (waiting("models"), waiting("modules"));
}

/// Tingene og fiendene i paletten i `?nivaa` viser et bilde når eieren peker
/// på dem (`Palette::pictures`): bildet finnes for hver.
#[test]
fn tingene_og_fiendene_i_paletten_har_bilde() {
    let root = env!("CARGO_MANIFEST_DIR");
    let things = crate::domain::edit::level::verdicts::THING_PICTURES.iter().map(|(_, m)| *m);
    let mobs = crate::domain::mob_kind::Kind::ALL.iter().map(|k| k.model());
    let missing: Vec<&str> = things.chain(mobs).filter(|m| !std::path::Path::new(&format!("{root}/assets/catalog/thumbs/{m}.jpg")).exists()).collect();
    assert!(missing.is_empty(), "uten bilde (python3.13 tools/qa/thumbs.py <navn>): {missing:?}");
}

/// Avvikene nivåredigeringen får (`data::level_verdicts`), følger dommene:
/// det som er godkjent og uendret, står ikke i lista.
#[test]
fn avvikene_i_nivaaredigeringen_folger_dommene() {
    let approvals: serde_json::Value = serde_json::from_str(crate::adapters::data::APPROVALS).expect("approvals.json");
    let kit = crate::adapters::data::kit();
    let v = crate::adapters::data::level_verdicts(&kit);
    let waits = waiting("modules");
    for m in &kit.modules {
        let key = m.id.replace('/', "_");
        let approved = approvals["modules"][&key]["verdict"] == "godkjent" && !waits.contains(&key);
        assert_eq!(v.modules.contains_key(&m.id), !approved, "{}", m.id);
    }
}
