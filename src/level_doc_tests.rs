//! Nivåredigeringen, fase 1 (`docs/nivaaredigering.md`): kompilatoren i
//! kjernen (`crypt::compose`) over byggesettet (`assets/models/crypt/kit/`),
//! prøvenivået (`assets/levels/nivaa_test.json`) og prøvebrettet (hver bit
//! mot hver bit i alle dreininger). Prøvenivået går også gjennom de gamle
//! testene over krypten (`provenivaaet_kan_gas_gjennom`,
//! `ingen_usynlige_vegger_i_provenivaaet`, `fiendene_finner_fram_i_provenivaaet`,
//! `lyset_folger_rommene_i_provenivaaet`).
//!
//! Her står bare det som trenger modellene. Kompilatorens regler testes i
//! kjernen (`crates/fristad-domain/tests/regler/level_doc.rs`).

use crate::adapters::data::{kit, level_doc};
use crate::domain::world::crypt::compose;
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::kit::PieceKind;

fn test_doc() -> LevelDoc {
    level_doc("test").expect("assets/levels/nivaa_test.json")
}

/// Modellene en bit kan ha (modellen og variantene).
fn family(k: &crate::domain::world::crypt::kit::KitDef, id: &str) -> Vec<String> {
    let p = k.piece(id).unwrap_or_else(|| panic!("{id} finnes ikke"));
    let mut out: Vec<String> = std::iter::once(p.model.clone()).chain(p.variants.iter().map(|v| v.model.clone())).collect();
    out.sort();
    out.dedup();
    out
}

/// Byggesettet finnes, og hver bit har modellen sin.
#[test]
fn byggesettet_har_bitene() {
    let k = kit();
    assert!(k.pieces.iter().filter(|p| p.kind == PieceKind::Cell).count() >= 6, "for få cellebiter: {:?}", k.pieces.iter().map(|p| &p.id).collect::<Vec<_>>());
    let models: Vec<&str> = crate::adapters::gltf::model_files().map(|(n, _)| *n).collect();
    for m in k.models() {
        assert!(models.contains(&m), "byggesettet bruker {m}, som ikke finnes i assets/models/crypt/kit/");
    }
    for opening in ["wall", "gang", "door", "gate", "secret", "arch", "overmur", "rail"] {
        assert!(k.edge("grav", opening).is_some(), "ingen halvkant for «{opening}»");
    }
    assert!(k.of_kind("grav", PieceKind::Corner).is_some() && k.of_kind("grav", PieceKind::Pillar).is_some());
    assert!(k.of_kind("grav", PieceKind::Leaf).is_some() && k.of_kind("grav", PieceKind::Prop).is_some(), "den falske veggen og fakkelen mangler");
    for class in ["inner", "wall", "pilaster", "outer"] {
        assert!(k.corner("grav", class).is_some_and(|p| p.fits.iter().any(|f| f == class)), "ingen kvarthjørne for «{class}»");
    }
    // Pilotsettet (del 4.4): rundt 20 biter, de vanligste i minst to utseender.
    assert!(k.models().len() >= 20, "pilotsettet har {} modeller", k.models().len());
    for id in ["grav/romfelt", "grav/gang", "grav/vegg"] {
        assert!(family(&k, id).len() >= 2, "{id} har bare ett utseende");
    }
    assert!(k.listed().any(|p| !p.footprint.is_empty()), "ingen sammensatt bit (rotunden)");
}

/// Prøvenivået i spillet: bitene står som instanser, og nivået har det et
/// nivå skal ha (start, opp, hvilested, ned, porten med spaken).
#[test]
fn provenivaaet_bygges_av_bitene() {
    let p = crate::tests::doc_world();
    let c = &p.world.crypt;
    assert_eq!(c.levels.len(), 1);
    let l = &c.levels[0];
    assert!(l.down.is_some() && l.up.is_some() && c.rests.len() == 1 && c.gates.len() == 1 && c.levers.len() == 1);
    // Den falske veggen (`grav_falsk`) settes ut av mekanismen, ikke som bit.
    let placed: usize = p.world.placements.iter().filter(|pl| pl.model.starts_with("grav_") && pl.model != "grav_falsk").map(|pl| pl.transforms.len()).sum();
    let (f, _) = compose::compile(&test_doc(), &kit());
    assert_eq!(placed, f.instances.len(), "ikke alle bitene står i verden");
    assert!(p.world.missing.iter().all(|m| !m.starts_with("grav_")), "modeller mangler: {:?}", p.world.missing);
    eprintln!("prøvenivået: {} instanser, {} kollidere, {} polygoner, {} rom", f.instances.len(), f.colliders.len(), f.nav.polys.len(), f.rooms.len());
}

/// Manifestet og glb-filene sier det samme: hver modell i byggesettet har
/// manifestet for bitene som bruker den i `extras.kit` (skrevet av
/// `tools/crypt/kit_bake.py` ved eksporten, likt `assets/models/crypt/kit/grav.json`).
#[test]
fn manifestet_er_likt_glb_filene() {
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string("assets/models/crypt/kit/grav.json").unwrap()).unwrap();
    let pieces = manifest["pieces"].as_array().unwrap();
    let files: std::collections::HashMap<&str, &str> = crate::adapters::gltf::model_files().map(|(n, p)| (*n, *p)).collect();
    let k = kit();
    for model in k.models() {
        let want: Vec<&serde_json::Value> = pieces
            .iter()
            .filter(|p| p["model"] == model || p["variants"].as_array().is_some_and(|vs| vs.iter().any(|v| v["model"] == model)))
            .collect();
        assert!(!want.is_empty(), "{model}: ingen bit i manifestet bruker modellen");
        let path = format!("assets/models/{}", files.get(model).unwrap_or_else(|| panic!("{model} finnes ikke")));
        let bytes = std::fs::read(&path).unwrap();
        let doc = gltf::Gltf::from_slice(&bytes).unwrap();
        let extras: serde_json::Value = serde_json::from_str(doc.as_json().extras.as_ref().map(|r| r.get()).unwrap_or("{}")).unwrap();
        let got = extras["kit"].as_array().unwrap_or_else(|| panic!("{path}: mangler extras.kit (eksporter med kit_cells.py)"));
        assert_eq!(got.iter().collect::<Vec<_>>(), want, "{path}: extras.kit er ikke likt manifestet");
    }
}

/// Hver bit har en fjernversjon (noden `far`) som er lettere enn den nære,
/// og budsjettet per bit holder (del 14: rundt 2 500 trekanter per celle med
/// mur, hjørner, gulv og hvelv).
#[test]
fn bitene_har_fjernversjon_og_holder_budsjettet() {
    let files: std::collections::HashMap<&str, &str> = crate::adapters::gltf::model_files().map(|(n, p)| (*n, *p)).collect();
    let k = kit();
    for model in k.models() {
        let bytes = std::fs::read(format!("assets/models/{}", files[model])).unwrap();
        let m = crate::adapters::gltf::load(model, &[("model.glb", &bytes)], -1.0, 0.0).unwrap();
        let near = m.mesh.indices.len() / 3;
        let far = m.far_mesh.as_ref().map(|f| f.indices.len() / 3).unwrap_or_else(|| panic!("{model}: ingen fjernversjon"));
        assert!(far > 0 && far <= near, "{model}: fjernversjonen har {far} trekanter, den nære {near}");
        let cells = k.listed().filter(|p| p.model == model || p.variants.iter().any(|v| v.model == model)).map(|p| p.footprint.len().max(1)).max().unwrap_or(1);
        assert!(near <= 1_200 * cells, "{model}: {near} trekanter (budsjettet er 1 200 per celle for én bit)");
    }
}
