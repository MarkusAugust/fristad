//! Steinbyttet (`docs/nivaaredigering.md` del 22, `crypt::stone`): murene
//! kan skifte stein per nivå, rom, modul, celle og vegg, med gulv, mur og
//! hvelv hver for seg, uten nye modeller.

use crate::adapters::data::{kit, level_doc};
use crate::domain::geometry::Layer;
use crate::domain::model::Look;
use crate::domain::world::crypt::compose::{self, Compiler, *};
use crate::domain::world::crypt::doc::{DocEdge, DocModule, DocRoom, LevelDoc};
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

/// Muren (laget) instansen får, eller `-` uten bytte.
fn wall(i: &InstanceDef) -> String {
    i.stone.as_ref().map(|s| s.to[1].clone()).unwrap_or_else(|| "-".into())
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

/// Dokumentet med stein på nivået, rommet, modulen og kanten skrives og
/// leses likt, og begge formene leses fra json.
#[test]
fn dokumentet_med_stein_leses_og_skrives() {
    let mut doc = two_rooms();
    doc.theme = "kloakk".into();
    doc.stone = named("tegl");
    doc.rooms = vec![DocRoom { id: "a".into(), stone: Some(StoneRef::Parts(StoneParts { vault: Some("crypt_vault".into()), ..Default::default() })), ..Default::default() }];
    doc.modules[1].stone = named("kvader");
    doc.edges = vec![DocEdge { cell: "a/c00".into(), side: "E".into(), stone: named("berg"), ..Default::default() }];
    let text = serde_json::to_string(&doc).unwrap();
    let back: LevelDoc = serde_json::from_str(&text).unwrap();
    assert_eq!(back, doc);
    let json = r#"{"stone": "tegl", "rooms": [{"id": "r", "stone": {"wall": "sewer_brick", "floor": "kvader"}}], "edges": [{"cell": "c", "side": "N", "stone": "berg"}]}"#;
    let d: LevelDoc = serde_json::from_str(json).unwrap();
    assert_eq!(d.stone, named("tegl"));
    assert_eq!(d.rooms[0].stone, Some(StoneRef::Parts(StoneParts { floor: Some("kvader".into()), wall: Some("sewer_brick".into()), vault: None, trim: None })));
    assert_eq!(d.edges[0].stone, named("berg"));
}

/// Kompilatoren setter steinen på hver instans: ingen bytte uten valg,
/// nivåets stein overalt, rommets over nivåets flate for flate, modulens
/// over rommets, kantens bare på den ene halvkanten, og overgangen går midt
/// i åpningen (hver halvkant og hvert hjørne har sin egen celles stein).
#[test]
fn kompilatoren_gir_steinen_per_instans() {
    let k = kit();
    let plain = compose::compile(&two_rooms(), &k).0;
    assert!(plain.instances.iter().all(|i| i.stone.is_none()), "gravkjelleren uten valg skal ikke bytte noe");

    let mut doc = two_rooms();
    doc.stone = named("tegl");
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", r.errors().collect::<Vec<_>>());
    for i in f.instances.iter().filter(|i| i.model != "grav_fakkel") {
        let s = i.stone.as_ref().unwrap_or_else(|| panic!("{} har ingen stein", i.model));
        assert_eq!(s.from, ["crypt_floor".to_string(), "crypt_wall".into(), "crypt_dressed".into()]);
        assert_eq!(s.to, ["sewer_floor".to_string(), "sewer_brick".into(), "sewer_brick".into(), "crypt_dressed".into()]);
    }
    assert!(f.instances.iter().filter(|i| i.model == "grav_fakkel").all(|i| i.stone.is_none()));

    // Nivåets tema gir standardsteinen.
    let mut themed = two_rooms();
    themed.theme = "borg".into();
    assert!(compose::compile(&themed, &k).0.instances.iter().filter(|i| i.model != "grav_fakkel").all(|i| wall(i) == "keep_ashlar"));

    // Rommet a får teglhvelv, modulen b kvader, og kanten øst i a berg.
    doc.rooms = vec![DocRoom { id: "a".into(), theme: "grav".into(), stone: Some(StoneRef::Parts(StoneParts { vault: Some("gravtegl".into()), ..Default::default() })), ..Default::default() }];
    doc.modules[1].stone = named("kvader");
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", r.errors().collect::<Vec<_>>());
    let cell_a = at(&f, 0.0, 0.0, None);
    assert_eq!(cell_a.len(), 1);
    assert_eq!(cell_a[0].stone.as_ref().unwrap().to, ["sewer_floor".to_string(), "sewer_brick".into(), "crypt_vault".into(), "crypt_dressed".into()], "rommet over nivået, flate for flate");
    assert_eq!(wall(at(&f, 4.0, 0.0, None)[0]), "keep_ashlar", "modulens stein");
    // Døra mellom dem: a sin halvdel tegl, b sin kvader.
    let (east, west) = (side_yaw(1), side_yaw(3));
    assert_eq!(at(&f, 2.0, 0.0, Some(east)).iter().map(|i| wall(i)).collect::<Vec<_>>(), ["sewer_brick"], "a sin halvdel av døra");
    assert_eq!(at(&f, 2.0, 0.0, Some(west)).iter().map(|i| wall(i)).collect::<Vec<_>>(), ["keep_ashlar"], "b sin halvdel av døra");
    // Hjørnene i (2, ±2): ett fra hver celle, hver med sin stein.
    let mut corners: Vec<String> = at(&f, 2.0, -2.0, None).iter().map(|i| wall(i)).collect();
    corners.sort();
    assert_eq!(corners, ["keep_ashlar", "sewer_brick"], "hjørnene der rommene møtes");

    // Kanten: bare a sin halvdel av muren øst blir berg.
    doc.edges = vec![DocEdge { cell: "a/c00".into(), side: "E".into(), stone: named("berg"), ..Default::default() }];
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", r.errors().collect::<Vec<_>>());
    assert_eq!(at(&f, 2.0, 0.0, Some(east)).iter().map(|i| wall(i)).collect::<Vec<_>>(), ["mine_rock"]);
    assert_eq!(at(&f, 2.0, 0.0, Some(west)).iter().map(|i| wall(i)).collect::<Vec<_>>(), ["keep_ashlar"]);
    assert_eq!(at(&f, 0.0, -2.0, None).iter().map(|i| wall(i)).collect::<Vec<_>>(), ["sewer_brick"], "muren nord i a er som før");
}

/// En ukjent stein eller et ukjent lag er en feil med sted.
#[test]
fn ukjent_stein_er_feil() {
    let k = kit();
    let mut doc = two_rooms();
    doc.modules[0].stone = named("marmor");
    doc.modules[1].stone = Some(StoneRef::Parts(StoneParts { wall: Some("gull".into()), ..Default::default() }));
    let (_, r) = compose::compile(&doc, &k);
    let bad: Vec<_> = r.errors().filter(|i| i.rule == "stein").collect();
    assert!(bad.iter().any(|i| i.msg.contains("marmor")) && bad.iter().any(|i| i.msg.contains("gull")), "{bad:?}");
    assert!(bad.iter().all(|i| i.at.is_some()));
}

/// Samme dokument gir samme stein byte for byte, og å bytte steinen i én
/// modul regner bare nabolaget på nytt og gir det samme som fra bunnen.
#[test]
fn steinen_er_fast_og_regnes_pa_nytt() {
    let k = kit();
    let mut doc = level_doc("test").expect("prøvenivået");
    doc.stone = named("kvader");
    let a = serde_json::to_string(&compose::compile(&doc, &k).0).unwrap();
    let b = serde_json::to_string(&compose::compile(&doc, &k).0).unwrap();
    assert_eq!(a, b);
    let mut c = Compiler::default();
    c.compile(&doc, &k);
    let m = doc.modules.iter().position(|m| m.module.ends_with("skattkammer")).expect("skattkammeret");
    doc.modules[m].stone = named("tegl");
    let (inc, r) = c.compile(&doc, &k);
    let fresh = compose::compile(&doc, &k).0;
    assert_eq!(serde_json::to_string(&inc).unwrap(), serde_json::to_string(&fresh).unwrap());
    assert!(r.recomputed < r.cells, "alt ble regnet på nytt ({} av {})", r.recomputed, r.cells);
    assert!(fresh.instances.iter().any(|i| wall(i) == "sewer_brick") && fresh.instances.iter().any(|i| wall(i) == "keep_ashlar"));
}

/// Den falske veggen får steinen til muren den står i, så den ikke synes.
#[test]
fn den_falske_veggen_far_murens_stein() {
    let k = kit();
    let mut doc = level_doc("test").expect("prøvenivået");
    let (f, _) = compose::compile(&doc, &k);
    assert!(f.marks.iter().filter(|m| m.kind == "secret_wall").all(|m| m.stone.is_none()));
    doc.stone = named("tegl");
    let (f, _) = compose::compile(&doc, &k);
    let walls: Vec<_> = f.marks.iter().filter(|m| m.kind == "secret_wall").collect();
    assert!(!walls.is_empty());
    for m in walls {
        assert_eq!(m.stone.as_ref().map(|s| s.to[1].as_str()), Some("sewer_brick"), "{:?}", m.id);
    }
}

/// Byttet i kjernen (sammenslåingen) og i rendereren følger samme regel:
/// gulvlaget blir gulv, murlaget mur, og murlaget som vender ned hvelv.
#[test]
fn steinbyttet_folger_flatene() {
    let k = kit();
    let mut doc = two_rooms();
    doc.stone = named("kjernestein");
    let (f, _) = compose::compile(&doc, &k);
    let look: Look = f.instances.iter().find_map(|i| i.stone.as_ref()).and_then(|s| s.look()).expect("utseendet");
    let (floor, wall, trim, plank) = (Layer::CryptFloor as u8, Layer::CryptStone as u8, Layer::CryptDressed as u8, Layer::Planks as u8);
    assert_eq!(look.layer(floor, 1.0), Layer::CoreStone as u8);
    assert_eq!(look.layer(wall, 0.0), Layer::CoreStone as u8, "loddrett mur");
    assert_eq!(look.layer(wall, 0.6), Layer::CoreStone as u8, "trinn og sokkel");
    assert_eq!(look.layer(wall, -0.8), Layer::CorePlastic as u8, "hvelvet");
    assert_eq!(look.layer(trim, 0.0), Layer::CoreStone as u8, "søyla");
    assert_eq!(look.layer(trim, -0.8), Layer::CoreStone as u8, "ribba under hvelvet er hogd stein, ikke hvelv");
    assert_eq!(look.layer(plank, -1.0), plank, "andre lag er som før");
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

/// Prøvenivået med steinene (`assets/levels/nivaa_steiner.json`: rom 2 × 2
/// og ganger i ti steiner, med overganger i dørene og gangene) bygges uten
/// feil og bruker hver stein.
#[test]
fn steinnivaaet_bygges() {
    let k = kit();
    let doc = level_doc("steiner").expect("assets/levels/nivaa_steiner.json");
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", r.errors().collect::<Vec<_>>());
    for s in k.stones.stones.iter().filter(|s| s.id != "gravmur") {
        assert!(f.instances.iter().any(|i| i.stone.as_ref().is_some_and(|x| x.to == [s.floor.clone(), s.wall.clone(), s.vault.clone(), s.trim.clone()])), "steinen {} er ikke brukt", s.id);
    }
}

/// Hvelvene er i lyset (eieren 20. oktober: hvelvene i bosshallen og
/// rotunden var svarte): rommet når opp til toppen av bitene (`RoomDef.rise`),
/// også over den frie høyden, og faklene og lysene i mørk stein (tegl, berg,
/// kvader) lyser sterkere enn i gravmuren (`StoneFile::light`).
#[test]
fn hvelvene_er_i_lyset_og_mork_stein_far_sterkere_lys() {
    let k = kit();
    for (id, top) in [("grav/bosshall", 10.0), ("grav/rotunde", 7.5), ("grav/rom_2x2", 5.0)] {
        let doc = compose::modules::showcase(&k, id).unwrap();
        let (f, _) = compose::compile(&doc, &k);
        let reach = f.rooms.iter().map(|r| r.h + r.rise).fold(0.0f32, f32::max);
        assert!(reach >= top, "{id}: rommet når {reach} m opp i lyset, hvelvet er {top} m");
    }
    // Faklene: samme modul i gravmur og i tegl.
    let torch = |id: &str| {
        let (f, _) = compose::compile(&compose::modules::showcase(&k, id).unwrap(), &k);
        f.lights.iter().filter(|l| l.flicker).map(|l| l.intensity.unwrap_or(2.0)).fold(0.0f32, f32::max)
    };
    let (grav, tegl) = (torch("grav/rom_2x2"), torch("kloakk/rom_2x2"));
    assert!(tegl > grav * 1.5, "faklene i tegl ({tegl}) skal lyse sterkere enn i gravmur ({grav})");
    for l in ["sewer_brick", "mine_rock", "keep_ashlar"] {
        assert!(k.stones.light.get(l).is_some_and(|v| *v > 1.0), "{l} mangler lysfaktor");
    }
}
