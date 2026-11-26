//! Steinbyttet (`docs/nivaaredigering.md` del 22, `crypt::stone`): murene
//! kan skifte stein per nivå, rom, modul, celle og vegg, med gulv, mur og
//! hvelv hver for seg, uten nye modeller.
//!
//! Her står bare det som trenger teksturlagene modellene bruker. Resten
//! testes i kjernen (`crates/fristad-domain/tests/regler/stone.rs`).

use crate::adapters::data::kit;
use crate::domain::world::crypt::compose;
use crate::domain::world::crypt::doc::{DocModule, LevelDoc};
use crate::domain::world::crypt::level::{InstanceDef, LevelFile};
use crate::domain::world::crypt::stone::{StoneParts, StoneRef};

fn named(s: &str) -> Option<StoneRef> {
    Some(StoneRef::Named(s.into()))
}

fn put(id: &str, module: &str, at: [i32; 3]) -> DocModule {
    DocModule { id: id.into(), module: module.into(), at, ..Default::default() }
}

/// To små rom side om side (a i x = 0, b i x = 4), med døra mellom dem.
fn two_rooms() -> LevelDoc {
    LevelDoc { format: 1, modules: vec![put("a", "grav/rom_1x1", [0, 0, 0]), put("b", "grav/rom_1x1", [1, 0, 0])], ..Default::default() }
}

/// Instansene som står i (x, z) og vender `yaw` (halvkantene: siden de hører til).
fn at<'a>(f: &'a LevelFile, x: f32, z: f32, yaw: Option<f32>) -> Vec<&'a InstanceDef> {
    f.instances.iter().filter(|i| (i.pos[0] - x).abs() < 0.01 && (i.pos[2] - z).abs() < 0.01 && yaw.is_none_or(|y| (i.yaw - y).abs() < 0.01) && i.model != "grav_fakkel").collect()
}

/// Steinene peker på lag som finnes og er malt, hvert tema har en stein, og
/// gravkjellerens standardstein er lagene bitene faktisk er laget med.
#[test]
fn steinene_peker_pa_lag_som_finnes() {
    let k = kit();
    assert!(k.stones.stones.len() >= 5, "for få steiner: {:?}", k.stones.stones);
    assert!(k.stones.check().is_empty(), "{:#?}", k.stones.check());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for s in &k.stones.stones {
        for l in [&s.floor, &s.wall, &s.vault, &s.trim] {
            assert!(root.join("assets/textures").join(l).join("meta.json").exists(), "steinen {}: laget {l} har ikke noe malt bilde i assets/textures/", s.id);
        }
    }
    for t in ["grav", "kloakk", "gruve", "borg", "kjerne"] {
        assert!(k.stones.base(t).is_some(), "temaet {t} har ingen standardstein");
    }
    let base = k.stones.base("grav").unwrap();
    let used = crate::adapters::gltf::used_layers();
    assert!(used.contains(&base.floor) && used.contains(&base.wall) && used.contains(&base.trim), "byggesettet er ikke laget med {base:?}");
    // Den hogde steinen er hogd i hver stein: aldri tømmer eller plast.
    for s in &k.stones.stones {
        assert!(!["mine_timber", "core_plastic"].contains(&s.trim.as_str()), "steinen {}: søylene og buene i {}", s.id, s.trim);
    }
}

/// Den hogde steinen (søyler, ribber, buer, karmer) er en egen flate: bitene
/// er laget med den, hver stein har sin, og den overstyres som de andre.
#[test]
fn den_hogde_steinen_er_egen_flate() {
    let k = kit();
    let used = crate::adapters::gltf::used_layers();
    assert!(used.iter().any(|l| l == "crypt_dressed"), "byggesettet har ingen hogd stein");
    let mut doc = two_rooms();
    doc.stone = named("berg");
    let (f, _) = compose::compile(&doc, &k);
    let s = f.instances.iter().find_map(|i| i.stone.as_ref()).unwrap();
    assert_eq!((s.to[1].as_str(), s.to[3].as_str()), ("mine_rock", "keep_ashlar"), "berget har kvader i søylene");
    doc.modules[1].stone = Some(StoneRef::Parts(StoneParts { trim: Some("kjernestein".into()), ..Default::default() }));
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", r.errors().collect::<Vec<_>>());
    let b = at(&f, 4.0, 0.0, None)[0].stone.as_ref().unwrap();
    assert_eq!((b.to[1].as_str(), b.to[3].as_str()), ("mine_rock", "core_stone"), "trim fra en stein, muren arvet");
    doc.modules[1].stone = Some(StoneRef::Parts(StoneParts { trim: Some("ashlar".into()), ..Default::default() }));
    assert_eq!(at(&compose::compile(&doc, &k).0, 4.0, 0.0, None)[0].stone.as_ref().unwrap().to[3], "ashlar", "trim som lagnavn");
    let d: LevelDoc = serde_json::from_str(r#"{"stone": {"trim": "kvader"}}"#).unwrap();
    assert_eq!(d.stone, Some(StoneRef::Parts(StoneParts { trim: Some("kvader".into()), ..Default::default() })));
}
