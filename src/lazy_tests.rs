//! Byens modeller som hentes etter start i nettleseren (plan del 10a):
//! manifestet, verden bygget fra det, og når økta ber om modellene.

use crate::adapters::delivery::{lazy, later_of, level_of, manifest, Manifest, WithManifest};
use crate::adapters::gltf::{model_files, Embedded, Files};
use crate::adapters::rapier::RapierPhysics;
use crate::application::app::Pieces;
use crate::application::{prepare, App, Prepared};
use crate::ports::driving::{Event, Play};
use crate::ports::ModelSource;
use crate::tests::{world, Counting};

fn read(p: &str) -> Vec<u8> {
    std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).expect(p)
}

/// Filene nettleseren henter ved start når manifestet finnes.
fn start_files() -> Files {
    Files(model_files().filter(|(n, p)| level_of(p).is_none() && !lazy(n, p)).map(|(_, p)| (p.to_string(), read(p))).collect())
}

/// Manifestet slik nettleseren får det: skrevet til json og lest igjen.
fn shipped() -> Manifest {
    let json = serde_json::to_string(&manifest(&Embedded)).unwrap();
    serde_json::from_str(&json).unwrap()
}

/// Manifestet sier det samme som glb-filene: målene, kolliderne og
/// skjelettet med klippene, også etter turen gjennom json. Og det dekker
/// det `delivery::lazy` sier hentes senere: innsiden, møblene, galleriet,
/// fiendene og utstyret.
#[test]
fn manifestet_er_likt_glb_filene() {
    let m = shipped();
    let lazy_names: Vec<&str> = model_files().filter(|(n, p)| lazy(n, p)).map(|(n, _)| *n).collect();
    assert_eq!(m.len(), lazy_names.len(), "en linje per modell som hentes senere");
    for want in ["tavern_interior", "gallery_card", "mutant_button", "framework", "arms_mail", "sword_rare", "token"] {
        assert!(m.contains_key(want), "{want} mangler i manifestet");
    }
    for want in ["tavern", "well", "npc_smith", "town_wall"] {
        assert!(!m.contains_key(want), "{want} trengs til første bilde");
    }
    assert!(model_files().any(|(_, p)| p.starts_with("furniture/") && m.contains_key(&p[10..p.len() - 4])), "møblene hentes senere");
    for (name, entry) in &m {
        let glb = later_of(&Embedded.load(name).unwrap());
        assert_eq!(format!("{:?}", entry.info), format!("{:?}", glb.info), "{name}: målene eller kolliderne");
        assert_eq!(format!("{:?}", entry.rig), format!("{:?}", glb.rig), "{name}: skjelettet");
    }
    assert!(m["mutant_button"].rig.as_ref().is_some_and(|r| r.clip("walk").is_some()), "fienden har klippene sine");
}

/// Verden bygget fra filene ved start og manifestet er den samme som
/// verden bygget fra alle glb-filene: kolliderne, målene, figurene,
/// instansene og skjelettene.
#[test]
fn verden_fra_manifestet_er_lik_verden_fra_glb() {
    let native = world();
    let start = start_files();
    let lazy = prepare(&WithManifest(&start, shipped()), 7);
    let w = (&lazy.world, &native.world);
    assert_eq!(format!("{:?}", w.0.colliders), format!("{:?}", w.1.colliders), "kolliderne");
    assert_eq!(format!("{:?}", w.0.targets), format!("{:?}", w.1.targets), "siktemålene");
    let placed = |p: &Prepared| p.world.placements.iter().map(|pl| (pl.model, pl.transforms.clone(), pl.reserve)).collect::<Vec<_>>();
    assert_eq!(placed(&lazy), placed(native), "instansene");
    let heights = |p: &Prepared| p.world.npcs.iter().map(|n| (n.model, n.height)).collect::<Vec<_>>();
    assert_eq!(heights(&lazy), heights(native), "figurenes høyde");
    let rigs = |p: &Prepared| {
        let mut v: Vec<_> = p.rigs.iter().map(|(n, r)| (n.clone(), format!("{r:?}"))).collect();
        v.sort();
        v
    };
    assert_eq!(rigs(&lazy), rigs(native), "skjelettene");
    // Det som ikke ble hentet, er ikke lastet, men venter.
    assert!(lazy.models.iter().all(|m| !lazy.later.contains(&m.name)));
    assert!(lazy.later.iter().any(|n| n == "tavern_interior") && lazy.later.iter().any(|n| n == "mutant_button"));
    assert!(native.later.is_empty(), "utenom nettleseren er alt lastet");
}

/// Økta ber om modellene når den trenger dem: armsettet, våpenet og
/// skjoldet du har på straks (først), innsiden og møblene når du står nær
/// huset, galleriet hjemme, fiendene med nivået de går i. Det som er
/// kommet, gis rendereren.
#[test]
fn modellene_hentes_naar_okta_trenger_dem() {
    let start = start_files();
    let prepared = prepare(&WithManifest(&start, shipped()), 7);
    let tavern = prepared.world.placements.iter().find(|p| p.model == "tavern_interior").and_then(|p| p.transforms.first()).map(|t| glam::Vec3::from(t.translation)).expect("Vertshuset");
    let levels: Vec<glam::Vec3> = prepared.world.crypt.levels.iter().map(|l| l.start.0).collect();
    let crypt = prepared.world.crypt.clone();
    let mut app = App::new(prepared, Counting::default(), RapierPhysics::new());
    let asked = |app: &mut App<Counting, RapierPhysics>| -> Vec<String> {
        let mut out = vec![];
        for _ in 0..3 {
            out.extend(app.tick(1.0 / 30.0).into_iter().filter_map(|e| if let Event::FetchModels(n) = e { Some(n) } else { None }).flatten());
        }
        out
    };
    let first = asked(&mut app);
    assert_eq!(first.first().map(String::as_str), Some("arms"), "armsettet først: {first:?}");
    assert!(first.iter().any(|n| n == "sword") && first.iter().any(|n| n == "shield_round"), "våpenet og skjoldet straks: {first:?}");
    assert!(!first.iter().any(|n| n == "arms_plate" || n == "sword_legend"), "ikke utstyret du ikke har på");
    let level1: Vec<&str> = crypt.spawns.iter().filter(|s| s.level == 0).map(|s| crate::domain::mob_kind::Kind::from_label(&s.label).model()).collect();
    assert!(!level1.is_empty());
    assert!(level1.iter().all(|m| first.iter().any(|n| n == m)), "fiendene i nivå 1 kommer med nivået: {first:?}");
    let deep: Vec<&str> = crypt.spawns.iter().filter(|s| s.level >= 3).map(|s| crate::domain::mob_kind::Kind::from_label(&s.label).model()).filter(|m| !level1.contains(m)).collect();
    assert!(deep.iter().all(|m| !first.iter().any(|n| n == m)), "fiendene dypere venter");
    // Innsiden når du står ved huset.
    app.session.place_feet(tavern + glam::Vec3::new(0.0, 0.0, 12.0));
    let near = asked(&mut app);
    assert!(near.iter().any(|n| n == "tavern_interior"), "innsiden ved Vertshuset: {near:?}");
    // Nede i nivå 4: fiendene der.
    app.session.place_feet(levels[3]);
    let down = asked(&mut app);
    let four: Vec<&str> = crypt.spawns.iter().filter(|s| s.level == 3).map(|s| crate::domain::mob_kind::Kind::from_label(&s.label).model()).collect();
    assert!(four.iter().all(|m| down.iter().chain(first.iter()).any(|n| n == m)), "fiendene i nivå 4: {down:?}");
    // Det som kommer, får rendereren, og det bes ikke om igjen.
    let models: Vec<_> = near.iter().filter_map(|n| Embedded.load(n)).collect();
    app.add_models(&models);
    assert!(app.renderer.models.iter().any(|n| n == "tavern_interior"));
    assert!(app.lazy_models().iter().filter(|l| near.contains(&l.name)).all(|l| l.state == Pieces::Loaded));
    assert!(asked(&mut app).iter().all(|n| !near.contains(n)));

    // En fiende som intet nivå setter ut, hentes aldri (de nye slagene
    // ventet før med nivå 1).
    let spawned: Vec<&str> = crypt.spawns.iter().map(|s| crate::domain::mob_kind::Kind::from_label(&s.label).model()).collect();
    let mobs: Vec<&str> = crate::domain::mob_kind::Kind::ALL.iter().map(|k| k.model()).collect();
    let all: Vec<&String> = first.iter().chain(&near).chain(&down).collect();
    assert!(all.iter().all(|n| !mobs.contains(&n.as_str()) || spawned.contains(&n.as_str())), "bare fiender nivåene har: {all:?}");
    if !spawned.contains(&"navbar") {
        assert!(!all.iter().any(|n| *n == "navbar"), "Navbaren venter til et nivå har den");
    }

    // Utenom nettleseren bes det ikke om noe.
    let mut native = App::new(prepare(&Embedded, 7), Counting::default(), RapierPhysics::new());
    assert!(asked(&mut native).is_empty());
}

/// Et nivå med en av de nye fiendene (Navbaren): redigeringen henter den
/// straks merket står i dokumentet, og prøveturen (verden bygd på nytt med
/// dokumentet som nivå 1, `Engine::level_play`) henter den med nivået.
#[test]
fn ny_fiende_i_et_nivaa_hentes_med_nivaaet() {
    use crate::application::prepare_level_editor;
    use crate::domain::edit::level::{LevelEditor, LevelOp};
    use crate::ports::driving::{Command, LevelCommand};
    let start = start_files();
    let editor = || {
        let k = crate::adapters::data::kit();
        LevelEditor::new(LevelEditor::blank("Prøve", 1, "grav", &k), k)
    };
    let asked = |app: &mut App<Counting, RapierPhysics>| -> Vec<String> {
        let mut out = vec![];
        for _ in 0..3 {
            out.extend(app.tick(1.0 / 30.0).into_iter().filter_map(|e| if let Event::FetchModels(n) = e { Some(n) } else { None }).flatten());
        }
        out
    };
    // I redigeringen: ingen Navbar før merket står der.
    let mut app = App::new(prepare_level_editor(&WithManifest(&start, shipped()), 7, editor()), Counting::default(), RapierPhysics::new());
    let before = asked(&mut app);
    assert!(!before.iter().any(|n| n == "navbar" || n == "dropdown"), "ingen nye fiender uten merke: {before:?}");
    let add: LevelOp = serde_json::from_str(r#"{"op":"add_thing","kind":"boss","at":[0.0,0,0.0],"label":"navbar"}"#).unwrap();
    assert!(app.command(Command::Level(LevelCommand::Op(add.clone()))));
    let after = asked(&mut app);
    assert!(after.iter().any(|n| n == "navbar"), "Navbaren hentes når merket settes ut: {after:?}");
    assert!(!after.iter().any(|n| n == "dropdown"));
    // Prøveturen: verden bygd på nytt med dokumentet, så nivå 1 har Navbaren.
    let mut e = editor();
    e.apply(add).unwrap();
    let mut play = App::new(prepare_level_editor(&WithManifest(&start, shipped()), 7, e), Counting::default(), RapierPhysics::new());
    let back = play.session.view_pose();
    assert!(play.session.level_play(back, false));
    let down = asked(&mut play);
    assert!(down.iter().any(|n| n == "navbar"), "prøveturen henter Navbaren med nivået: {down:?}");
    assert!(!down.iter().any(|n| n == "dropdown" || n == "spinner"), "bare fiendene nivået har: {down:?}");
}
