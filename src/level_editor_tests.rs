//! Nivåredigeringen, fase 3 (`docs/nivaaredigering.md` del 12 og 23):
//! `LevelEditor` i kjernen over nivådokumentet: modulene settes ut,
//! flyttes, dreies, speiles og slettes, åpningene får slag, innholdet
//! kobles, alt kan angres, og reglene (kompilatorens og nåbarheten,
//! koblingene og takhøyden) sier fra med sted. Økta viser nivået i 3D som
//! bevegelige instanser, og prøveturen kan ikke drepe deg.
//!
//! Her står det som trenger økta, modellene eller lagringen i fast form.
//! Resten testes i kjernen (`crates/fristad-domain/tests/regler/level_editor.rs`).

use crate::adapters::data::{kit, level_doc_json, level_doc_path};
use crate::domain::edit::level::{LevelEditor, LevelOp};
use crate::domain::world::crypt::compose::Severity;
use crate::domain::world::crypt::doc::{LevelDoc, LevelId};

/// En renderer som ikke tegner noe (økta lager bildet selv).
struct Blind;

impl crate::ports::Renderer for Blind {
    fn render(&mut self, _frame: &crate::domain::scene::Frame) {}
    fn resize(&mut self, _width: u32, _height: u32) {}
    fn set_quality(&mut self, _quality: crate::ports::Quality) {}
    fn triangles(&self) -> u32 {
        0
    }
}

fn blank() -> LevelEditor {
    let k = kit();
    LevelEditor::new(LevelEditor::blank("Prøve", 3, "grav", &k), k)
}

fn op(e: &mut LevelEditor, json: &str) -> Vec<String> {
    let o: LevelOp = serde_json::from_str(json).unwrap_or_else(|err| panic!("{json}: {err}"));
    e.apply(o).unwrap_or_else(|err| panic!("{json}: {err}"))
}

fn errors(e: &LevelEditor) -> Vec<String> {
    e.issues().iter().filter(|i| i.severity == Severity::Error).map(|i| format!("{}: {}", i.rule, i.msg)).collect()
}

/// Dokumentet lagres i fast form og leses tilbake likt; stien følger nummeret.
#[test]
fn nivaaredigeringen_lagrer_fast_form() {
    let mut e = blank();
    op(&mut e, r#"{"op":"add_module","module":"grav/gang_1","at":[2,0,0],"rot":1}"#);
    op(&mut e, r#"{"op":"add_thing","kind":"mob","at":[8.0,0,0.0],"label":"tooltip"}"#);
    let text = level_doc_json(e.doc());
    assert!(text.lines().any(|l| l.trim_start().starts_with("{\"at\":[2,0,0]") || l.contains("\"module\":\"grav/gang_1\"")), "én modul per linje:\n{text}");
    let back: LevelDoc = serde_json::from_str(&text).unwrap();
    assert_eq!(&back, e.doc());
    assert_eq!(level_doc_path(e.doc()), "assets/levels/nivaa_03.json");
    let named = LevelDoc { level: LevelId::Name("test".into()), ..LevelDoc::default() };
    assert_eq!(level_doc_path(&named), "assets/levels/nivaa_test.json");
}

/// Økta i nivåredigeringen: nivået tegnes som bevegelige instanser (med
/// plass satt av), de faste i krypten skjules, og en endring synes straks;
/// prøveturen begynner ved startstedet, og du kan ikke dø.
#[test]
fn nivaaredigeringen_viser_nivaaet_og_proveturen_dreper_ikke() {
    use crate::application::{prepare_level_editor, App};
    use crate::ports::driving::{Command, LevelCommand, Play};
    let k = kit();
    let editor = LevelEditor::new(LevelEditor::blank("Prøve", 1, "grav", &k), k);
    let prepared = prepare_level_editor(&crate::adapters::gltf::Embedded, 7, editor);
    let reserve = |name: &str| prepared.world.placements.iter().find(|p| p.model == name).map(|p| p.reserve).unwrap_or(0);
    assert!(reserve("grav_vegg") >= 1000, "plass til murene i 3D");
    let mut app = App::new(prepared, Blind, crate::adapters::rapier::RapierPhysics::new());
    let f = app.session.frame(16.0 / 9.0);
    let mut f2 = f;
    app.session.level_frame(&mut f2);
    let walls = f2.instances.iter().filter(|i| i.model == "grav_vegg").count();
    assert!(walls > 4, "kapellet har murer i 3D: {walls}");
    assert!(!f2.hidden.is_empty(), "de faste instansene i krypten skjules");
    assert!(app.command(Command::Level(LevelCommand::Op(LevelOp::AddModule { module: "grav/rom_2x2".into(), at: [2, 0, 0], rot: 0, mirror: false }))));
    let mut f3 = app.session.frame(16.0 / 9.0);
    app.session.level_frame(&mut f3);
    assert!(f3.instances.len() > f2.instances.len(), "det nye rommet synes straks");
    // Prøveturen: ned i nivået, uten å kunne dø.
    let back = app.session.view_pose();
    assert!(app.session.level_play(back, false));
    assert!(app.session.immortal);
    let below = |app: &App<Blind, crate::adapters::rapier::RapierPhysics>| app.session.view_pose().0.y < crate::domain::world::crypt::TOP;
    assert!(below(&app), "prøveturen begynner nede i nivået");
    let _ = app.session.die_now();
    assert!(below(&app), "du dør ikke på prøvetur");
    assert!(app.session.level_back());
    assert!(!app.session.immortal);
    let _ = app.tick(0.016);
}

/// 3D i redigeringen har rom til rendereren, så himmelen er svart (ingen
/// skyer når kameraet står utenfor nivået); med spillets lys følger lysene
/// rommene som i krypten.
#[test]
fn nivaaredigeringen_har_ingen_himmel() {
    use crate::application::{prepare_level_editor, App};
    use crate::ports::driving::{Command, LevelCommand, Play};
    let k = kit();
    let editor = LevelEditor::new(LevelEditor::blank("Prøve", 1, "grav", &k), k);
    let prepared = prepare_level_editor(&crate::adapters::gltf::Embedded, 7, editor);
    let mut app = App::new(prepared, Blind, crate::adapters::rapier::RapierPhysics::new());
    let mut f = app.session.frame(16.0 / 9.0);
    app.session.level_frame(&mut f);
    let r = f.rooms.as_ref().expect("rom til rendereren med arbeidslyset");
    assert!(!r.boxes.is_empty() && r.masks.iter().all(|m| *m == u64::MAX), "arbeidslyset lyser overalt");
    assert!(app.command(Command::Level(LevelCommand::WorkLight(false))));
    let mut f = app.session.frame(16.0 / 9.0);
    app.session.level_frame(&mut f);
    let r = f.rooms.as_ref().expect("rom til rendereren med spillets lys");
    let rooms = app.session.level_editing().unwrap().editor.file().rooms.len();
    assert_eq!(r.boxes.len(), rooms, "rommene i nivået");
    assert_eq!(r.masks.len(), f.lights.len());
    // Snittet (S): alt over etasjen kuttes bort, og det som ligger helt over tegnes ikke.
    assert!(f.clip.is_none(), "uten snitt kuttes ingenting");
    assert!(app.command(Command::Level(LevelCommand::Cut(Some(6.0)))));
    let mut f = app.session.frame(16.0 / 9.0);
    app.session.level_frame(&mut f);
    let origin = app.session.level_editing().unwrap().origin;
    let clip = f.clip.expect("snittet er på");
    assert!((clip - (origin.y + 6.0 + crate::application::level_editing::CUT_HEIGHT)).abs() < 1e-3, "{clip}");
    assert!(f.band.1 <= clip);
    assert!(app.command(Command::Level(LevelCommand::Cut(None))));
    let mut f = app.session.frame(16.0 / 9.0);
    app.session.level_frame(&mut f);
    assert!(f.clip.is_none());
}

/// Paletten har ett sett moduler (eieren 20. oktober: «det holder med en
/// modell, og så må jeg kunne kle flatene i den teksturen jeg vil ha»),
/// gruppert som rom, ganger, trapper, ferdigrom, mekanismer og bosshall,
/// hver form én gang og med bildet sitt (`module_shots.py`,
/// `assets/levels/moduler/grav/<navn>.jpg`; eierens egne moduler, `OWN`,
/// kan stå uten til bildet er tatt), og temaene nivået kan ha er
/// de som har en standardstein.
#[test]
fn paletten_har_alle_temaene_gruppert() {
    use crate::domain::edit::level::map;
    let mut k = kit();
    // En modul eieren har lagret selv (Ctrl+Shift+G), uten bilde ennå.
    let mut own = k.modules.iter().find(|m| m.id == "grav/rom_1x1").unwrap().clone();
    own.id = "grav/egen_provemodul".into();
    own.about = format!("{} Rom 1 × 1.", crate::domain::edit::level::OWN);
    k.modules.push(own);
    let p = map::palette(&k, vec![], vec![]);
    assert!(p.modules.iter().any(|m| m.id == "grav/egen_provemodul"), "eierens modul står i paletten");
    assert!(p.modules.len() >= 30, "bare {} moduler", p.modules.len());
    for g in ["rom", "gang", "trapp", "ferdig", "mekanisme", "bosshall"] {
        assert!(p.modules.iter().any(|m| m.group == g), "ingen i gruppen {g}");
    }
    let mut names: Vec<&str> = p.modules.iter().map(|m| m.id.split('/').nth(1).unwrap()).collect();
    names.sort();
    assert!(names.windows(2).all(|w| w[0] != w[1]), "samme form står to ganger: {names:?}");
    for m in &p.modules {
        assert!(m.id.starts_with("grav/"), "{} står utenfor settet", m.id);
        // Eierens egne moduler (Ctrl+Shift+G) står med «Bilde mangler» til
        // `module_shots.py` har tatt bildet.
        if m.about.starts_with(crate::domain::edit::level::OWN) {
            continue;
        }
        let name = m.id.split('/').nth(1).unwrap();
        let img = format!("{}/assets/levels/moduler/grav/{name}.jpg", env!("CARGO_MANIFEST_DIR"));
        assert!(std::path::Path::new(&img).exists(), "{} mangler bildet i paletten ({img})", m.id);
    }
    for t in ["grav", "kloakk", "gruve", "borg", "kjerne"] {
        assert!(p.themes.iter().any(|x| x == t), "temaet {t} mangler: {:?}", p.themes);
    }
    for id in ["grav/tronsal", "grav/slusekammer", "grav/plastsal", "grav/gruvesal", "grav/kanal"] {
        assert!(p.modules.iter().any(|m| m.id == id), "ferdigrommet {id} mangler");
    }
    // De gamle kopiene per tema leses som modulen i settet.
    assert_eq!(k.module("kloakk/rom_2x2").map(|m| m.id.as_str()), Some("grav/rom_2x2"));
    assert_eq!(k.module("borg/tronsal").map(|m| m.id.as_str()), Some("grav/tronsal"));
    let g = |id: &str| p.modules.iter().find(|m| m.id == id).map(|m| m.group);
    assert_eq!(g("grav/bosshall"), Some("bosshall"));
    assert_eq!(g("grav/porthall"), Some("mekanisme"));
    assert_eq!(g("grav/kapell"), Some("ferdig"));
    assert_eq!(g("grav/halvtrapp"), Some("trapp"));
    // Rekkefølgen følger gruppene.
    let order: Vec<usize> = p.modules.iter().map(|m| map::GROUPS.iter().position(|(x, _)| *x == m.group).unwrap()).collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]), "paletten står i gruppene sine");
}

/// Stemningen i et rom (fase 4): lysstyrken og fargen går til faklene og
/// lysene i rommet, tåka og lyden til rommet i nivåfila, og i spillet hører
/// du vannet og står i tåka der. Lys eieren setter ut, kan endres og flyttes.
#[test]
fn stemningen_og_lysene_folger_rommet() {
    use crate::application::{prepare_level_editor, App};
    use crate::domain::world::crypt::doc::Mood;
    use crate::ports::driving::{Event, Play};
    let mut e = blank();
    let id = op(&mut e, r#"{"op":"add_module","module":"grav/rom_2x2","at":[2,0,0]}"#)[0].clone();
    let room = |e: &LevelEditor| e.file().lights.iter().filter(|l| l.room.as_deref() == Some(id.as_str())).cloned().collect::<Vec<_>>();
    let mut m = e.doc().modules.iter().find(|m| m.id == id).unwrap().clone();
    m.mood = Some(Mood { torches: 1.0, ..Mood::default() });
    e.apply(LevelOp::SetModule { module: m.clone() }).unwrap();
    let plain = room(&e);
    assert!(!plain.is_empty(), "faklene i rommet");
    m.mood = Some(Mood { torches: 1.0, light: 0.5, color: Some([0.5, 0.7, 1.0]), fog: 0.6, sound: "vann".into() });
    e.apply(LevelOp::SetModule { module: m.clone() }).unwrap();
    let dim = room(&e);
    assert_eq!(dim.len(), plain.len());
    for (a, b) in plain.iter().zip(&dim) {
        assert!((b.intensity.unwrap_or(2.0) - a.intensity.unwrap_or(2.0) * 0.5).abs() < 0.01, "halv lysstyrke: {a:?} → {b:?}");
        assert_eq!(b.color, [0.5, 0.7, 1.0]);
    }
    let r = e.file().rooms.iter().find(|r| r.id == id).unwrap();
    assert_eq!((r.fog, r.sound.as_str()), (0.6, "vann"));
    // Kartet viser stemningen som gjelder (malens om eieren ikke har endret den).
    let map = e.map();
    assert!(map.modules.iter().find(|x| x.id == id).unwrap().own_mood);
    assert!(!map.modules.iter().find(|x| x.module.ends_with("kapell")).unwrap().own_mood);
    // Et lys eieren setter ut: farge, styrke, flamme, og det flyttes.
    let l = op(&mut e, r#"{"op":"add_light","at":[9.0,2.4,1.0]}"#)[0].clone();
    let mut light = e.doc().lights.iter().find(|x| x.id == l).unwrap().clone();
    light.color = [0.4, 1.0, 0.4];
    light.intensity = Some(4.0);
    light.flicker = false;
    light.pos = [10.0, 2.4, 2.0];
    e.apply(LevelOp::SetLight { light }).unwrap();
    let ml = e.map().lights.into_iter().find(|x| x.id == l).unwrap();
    assert!(ml.own && !ml.flicker && ml.pos == [10.0, 2.4, 2.0] && ml.color == [0.4, 1.0, 0.4], "eierens lys har sin egen farge: {ml:?}");
    // 0,5 lysstyrke i rommet halverer også eierens lys der.
    assert!((ml.intensity - 2.0).abs() < 0.01, "styrken ganget med rommets: {}", ml.intensity);
    // I spillet: stå i rommet og hør vannet; tåka kommer.
    let k = kit();
    let doc = e.doc().clone();
    let prepared = prepare_level_editor(&crate::adapters::gltf::Embedded, 7, LevelEditor::new(doc, k));
    let mut app = App::new(prepared, Blind, crate::adapters::rapier::RapierPhysics::new());
    let back = app.session.view_pose();
    assert!(app.session.level_play(back, false));
    let origin = app.session.level_editing().unwrap().origin;
    app.session.place_feet(origin + glam::Vec3::new(10.0, 0.1, 2.0));
    let mut amb = None;
    for _ in 0..90 {
        for ev in app.tick(1.0 / 30.0) {
            if let Event::Ambience(a) = ev {
                amb = Some(a);
            }
        }
    }
    let a = amb.expect("lydbildet");
    assert!(a.cellar && a.water > 0.9, "vannet i rommet: {a:?}");
    let f = app.session.frame(16.0 / 9.0);
    assert!(f.fog_density > 0.03, "tåka i rommet: {}", f.fog_density);
}

/// Malens innhold kan endres per nivå (fase 4): flyttes, endres og fjernes,
/// lagret i modulen (`hidden`) og dokumentet, koblingene i malen følger
/// med, og «tilbake til malen» gir det som var.
#[test]
fn malens_innhold_endres_per_nivaa() {
    let mut e = blank();
    let m = op(&mut e, r#"{"op":"add_module","module":"grav/porthall","at":[3,0,0]}"#)[0].clone();
    let before = e.doc().clone();
    let map = e.map();
    let lever = map.things.iter().find(|t| t.kind == "lever" && t.module.as_deref() == Some(m.as_str())).expect("spaken i porthallen").clone();
    let chest = map.things.iter().find(|t| t.kind == "chest" && t.module.as_deref() == Some(m.as_str())).expect("kista i porthallen").clone();
    assert!(!lever.own);
    // Flytt spaken: den blir eierens med samme id, og koblingen til porten står.
    let to = [lever.pos[0] + 1.0, lever.pos[1], lever.pos[2]];
    e.apply(LevelOp::MoveThing { id: lever.id.clone(), to }).unwrap();
    let local = lever.id.split_once('/').unwrap().1.to_string();
    assert!(e.doc().modules.iter().find(|x| x.id == m).unwrap().hidden.contains(&local));
    let mark = e.file().marks.iter().find(|x| x.id.as_deref() == Some(lever.id.as_str())).unwrap();
    assert_eq!(mark.pos.unwrap(), to, "spaken står der den ble flyttet");
    assert!(!mark.opens.is_empty(), "spaken åpner fortsatt porten");
    assert_eq!(e.file().marks.iter().filter(|x| x.kind == "lever").count(), 1, "bare én spak");
    let t = e.map().things.into_iter().find(|x| x.id == lever.id).unwrap();
    assert!(t.own && t.template.as_deref() == Some(m.as_str()));
    assert!(!errors(&e).iter().any(|x| x.contains("kobling")), "{:?}", errors(&e));
    // Endre kista: hva den gir.
    let mut c = e.flat().things.into_iter().find(|x| x.id == chest.id).unwrap();
    c.loot = Some("button".into());
    c.anchor = "fri".into();
    c.pos = Some(chest.pos);
    e.apply(LevelOp::SetThing { thing: c }).unwrap();
    assert_eq!(e.file().marks.iter().find(|x| x.id.as_deref() == Some(chest.id.as_str())).unwrap().loot.as_deref(), Some("button"));
    // Fjern kista fra modulen i dette nivået.
    e.apply(LevelOp::Reattach { module: m.clone(), thing: chest.id.split_once('/').unwrap().1.into() }).unwrap();
    e.apply(LevelOp::Delete { ids: vec![chest.id.clone()] }).unwrap();
    assert!(!e.file().marks.iter().any(|x| x.id.as_deref() == Some(chest.id.as_str())), "kista er borte");
    assert_eq!(e.map().modules.iter().find(|x| x.id == m).unwrap().removed, vec![chest.id.split_once('/').unwrap().1.to_string()]);
    // Flytt modulen: den løsnede spaken følger med.
    e.apply(LevelOp::Move { ids: vec![m.clone()], by: [0, 0, 2] }).unwrap();
    assert_eq!(e.doc().things.iter().find(|x| x.id == lever.id).unwrap().pos.unwrap(), [to[0], to[1], to[2] + 8.0]);
    // Tilbake til malen: begge, og dokumentet er som før.
    e.apply(LevelOp::Move { ids: vec![m.clone()], by: [0, 0, -2] }).unwrap();
    e.apply(LevelOp::Reattach { module: m.clone(), thing: local }).unwrap();
    e.apply(LevelOp::Reattach { module: m.clone(), thing: chest.id.split_once('/').unwrap().1.into() }).unwrap();
    assert_eq!(e.doc(), &before, "tilbake til malen gir dokumentet som før");
    // Det lagres og leses likt.
    e.apply(LevelOp::MoveThing { id: lever.id.clone(), to }).unwrap();
    let text = level_doc_json(e.doc());
    assert!(text.contains("\"hidden\""), "{text}");
    let back: LevelDoc = serde_json::from_str(&text).unwrap();
    assert_eq!(&back, e.doc());
}

/// Malene (fase 4): et utvalg lagres som mal med innholdet og koblingene,
/// og settes ut igjen dreid og speilet med nye id-er som henger sammen.
#[test]
fn malene_settes_ut_med_koblingene() {
    let mut e = blank();
    // Rommet med en gang vest for seg og porten mellom dem, langt unna kapellet.
    let r = op(&mut e, r#"{"op":"add_module","module":"grav/rom_3x2","at":[10,0,0]}"#)[0].clone();
    let map = e.map();
    let m = map.modules.iter().find(|m| m.id == r).unwrap();
    let o = m.openings.iter().filter(|o| o.side == 3).min_by_key(|o| o.at[1].abs()).unwrap().clone();
    let g = op(&mut e, &format!(r#"{{"op":"add_module","module":"grav/rom_1x1","at":[{},0,{}]}}"#, o.at[0] - 1, o.at[1]))[0].clone();
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{}","kind":"port"}}"#, o.id));
    let gate = e.doc().modules.iter().find(|m| m.id == r).unwrap().overrides[0].id.clone().unwrap();
    assert!(e.map().edges.iter().any(|x| x.kind == "gate"), "porten mellom gangen og rommet");
    let lever = op(&mut e, r#"{"op":"add_thing","kind":"lever","at":[41.0,0,1.0]}"#)[0].clone();
    op(&mut e, &format!(r#"{{"op":"link","from":"{lever}","to":"{gate}","kind":"opens"}}"#));
    let light = op(&mut e, r#"{"op":"add_light","at":[40.0,2.4,0.0]}"#)[0].clone();
    let tpl = e.template(&[r.clone(), g.clone()], "Porten med spaken");
    assert_eq!(tpl.modules.len(), 2);
    assert_eq!(tpl.modules[0].at, [0, 0, 0], "malen begynner i origo");
    assert!(tpl.things.iter().any(|t| t.id == lever), "spaken i rommet er med");
    assert!(tpl.lights.iter().any(|l| l.id == light), "lyset i rommet er med");
    assert_eq!(tpl.links.len(), 1, "koblingen er med");
    // Malen leses og skrives som et nivådokument.
    let text = level_doc_json(&tpl);
    let tpl: LevelDoc = serde_json::from_str(&text).unwrap();
    // Sett den ut tre steder: som den er, dreid og speilet.
    let before = e.doc().clone();
    for (at, rot, mirror) in [([10, 0, 8], 0u8, false), ([24, 0, 0], 1, false), ([10, 0, -10], 2, true)] {
        let made = e.apply(LevelOp::Stamp { template: Box::new(tpl.clone()), at, rot, mirror }).unwrap();
        assert_eq!(made.len(), 4, "modulene, spaken og lyset: {made:?}");
        let (m2, l2) = (made[0].clone(), made[2].clone());
        assert_ne!(m2, r);
        let doc = e.doc();
        let gate2 = doc.modules.iter().find(|m| m.id == m2).unwrap().overrides[0].id.clone().unwrap();
        assert_ne!(gate2, gate, "porten får ny id");
        assert!(doc.links.iter().any(|l| l.from == l2 && l.to == gate2), "spaken åpner sin egen port");
        // Spaken står i den nye modulen.
        let foot = e.footprint(doc.modules.iter().find(|m| m.id == m2).unwrap());
        let p = doc.things.iter().find(|t| t.id == l2).unwrap().pos.unwrap();
        let cell = [(p[0] / 4.0).round() as i32, (p[2] / 4.0).round() as i32];
        assert!(foot.iter().any(|c| [c[0], c[2]] == cell), "spaken {p:?} i modulen {foot:?}");
        let mm = doc.modules.iter().find(|m| m.id == m2).unwrap();
        assert_eq!((mm.rot, mm.mirror), (rot, mirror));
    }
    assert!(!errors(&e).iter().any(|x| x.starts_with("kobling")), "{:?}", errors(&e));
    // Ett angresteg per mal.
    for _ in 0..3 {
        assert!(e.undo());
    }
    assert_eq!(e.doc(), &before);
}

/// «Merk klart» (fase 4): et lite nivå 1 merket klart erstatter det gamle
/// nivå 1 i spillet (`data::ready_docs`, `Data::compose_levels`), mens et
/// utkast og et prøvenivå uten nummer blir stående utenfor.
#[test]
fn et_klart_nivaa_erstatter_det_gamle() {
    use crate::adapters::data::ready_docs;
    use crate::ports::ModelSource;
    let mut e = blank();
    let head = |e: &LevelEditor, n: u32, status: &str| crate::domain::edit::level::Head { level: LevelId::N(n), status: status.into(), ..e.head() };
    op(&mut e, r#"{"op":"add_module","module":"grav/nedgang","at":[2,0,0],"rot":1}"#);
    op(&mut e, r#"{"op":"add_thing","kind":"rest","at":[0.0,0,2.0]}"#);
    e.apply(LevelOp::SetHead { head: head(&e, 1, "utkast") }).unwrap();
    assert!(errors(&e).is_empty(), "det lille nivået: {:?}", errors(&e));
    let draft = e.doc().clone();
    e.apply(LevelOp::SetHead { head: head(&e, 1, "klar") }).unwrap();
    let ready = e.doc().clone();
    let test = LevelDoc { status: "klar".into(), level: LevelId::Name("test".into()), ..ready.clone() };
    let picked = ready_docs([draft.clone(), ready.clone(), test].into_iter());
    assert_eq!(picked.len(), 1, "bare det klare med nummer");
    assert_eq!(picked[0].0, 1);
    // Ingen øvre grense (eieren 26. november): et klart nivå 11 er med, og 0 er ikke et nivå.
    let deep = |n: u32| LevelDoc { level: LevelId::N(n), ..ready.clone() };
    assert_eq!(ready_docs([deep(11), deep(0)].into_iter()).iter().map(|(n, _)| *n).collect::<Vec<_>>(), [11], "nivå 11 er et nivå");
    // Verden bygd med det: nivå 1 er det nye, de andre står.
    let mut data = crate::adapters::gltf::Embedded.world_data();
    let old: Vec<u32> = data.levels.iter().map(|(n, _)| *n).collect();
    let old1 = data.levels.iter().find(|(n, _)| *n == 1).map(|(_, f)| f.rooms.len());
    data.docs = picked;
    let prepared = crate::application::prepare_with(&crate::adapters::gltf::Embedded, 7, data, false);
    let c = &prepared.world.crypt;
    assert_eq!(c.levels.iter().map(|l| l.n).collect::<Vec<_>>(), old, "de samme nivåene");
    let l1 = c.levels.iter().find(|l| l.n == 1).unwrap();
    let want = e.file().rooms.len();
    assert_eq!(l1.rooms.len(), want, "nivå 1 er det nye ({want} rom, det gamle hadde {old1:?})");
    assert!(l1.rooms.iter().any(|r| r.id == "kapell"), "kapellet fra det nye nivået");
    assert!(l1.down.is_some(), "trappa ned");
}

/// Et utvalg som ny modul (del 27): modulene utvidet til cellene, med
/// kantene der de møttes, innholdet og koblingene, i fast form; tatt inn i
/// byggesettet står den i paletten og bygger det samme som utvalget.
#[test]
fn utvalget_blir_en_ny_modul() {
    use crate::adapters::data::{module_json, module_path};
    use crate::domain::world::crypt::doc::ModuleDef;
    let mut e = blank();
    // Porthallen og et rom ved siden av, med en dør imellom.
    let a = op(&mut e, r#"{"op":"add_module","module":"grav/porthall","at":[3,0,0]}"#)[0].clone();
    let b = op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[4,0,0]}"#)[0].clone();
    let ids = vec![a.clone(), b.clone()];
    assert!(e.as_module(&ids, "Porthall med kott", "Porthall").is_err(), "filnavnet med store bokstaver");
    assert!(e.as_module(&ids, "Kapell", "kapell").is_err(), "byggesettets egne moduler skrives ikke over");
    let def = e.as_module(&ids, "Porthall med kott", "porthall_kott").expect("modulen");
    assert_eq!(def.id, "grav/porthall_kott");
    assert_eq!(module_path(&def.id), "assets/levels/moduler/grav/porthall_kott.json");
    let cells: usize = e.doc().modules.iter().filter(|m| ids.contains(&m.id)).map(|m| e.footprint(m).len()).sum();
    assert_eq!(def.cells.len(), cells, "cellene til begge modulene");
    assert!(def.things.iter().any(|t| t.kind == "lever") && def.things.iter().any(|t| t.kind == "gate"), "spaken og porten følger med");
    assert!(def.links.iter().any(|l| l.kind == "opens"), "koblingen følger med");
    // Fast form og tilbake.
    let text = module_json(&def);
    assert!(text.lines().count() > def.cells.len(), "én celle per linje");
    let back: ModuleDef = serde_json::from_str(&text).unwrap();
    assert_eq!(back.cells.len(), def.cells.len());
    // I byggesettet: i paletten, og satt ut alene bygger den det samme.
    e.add_module(back);
    let k = e.kit().clone();
    let palette = crate::domain::edit::level::map::palette(&k, vec![], vec![]);
    let pm = palette.modules.iter().find(|m| m.id == def.id).expect("i paletten");
    assert_eq!(pm.cells.len(), cells);
    assert!(!pm.openings.is_empty(), "den nye modulen har åpninger");
    let before = e.file().instances.len();
    let errs = errors(&e).len();
    op(&mut e, &format!(r#"{{"op":"delete","ids":["{a}","{b}"]}}"#));
    let n = op(&mut e, r#"{"op":"add_module","module":"grav/porthall_kott","at":[3,0,0]}"#)[0].clone();
    assert!(errors(&e).iter().all(|x| !x.starts_with("modul") && !x.starts_with("kant") && !x.starts_with("kobling")), "{:?}", errors(&e));
    assert_eq!(errors(&e).len(), errs, "de samme bruddene som utvalget: {:?}", errors(&e));
    let after = e.file().instances.len();
    assert!((after as i64 - before as i64).abs() <= 2, "samme bygg: {before} før, {after} etter");
    assert!(
        e.file()
            .marks
            .iter()
            .any(|m| m.kind == "lever" && m.opens.iter().any(|o| o.starts_with(&format!("{n}/")) || e.file().marks.iter().any(|g| g.kind == "gate" && g.id.as_deref() == Some(o.as_str())))),
        "spaken åpner porten i den nye modulen"
    );
}
