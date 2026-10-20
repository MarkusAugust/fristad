//! Nivåredigeringen, fase 3 (`docs/nivaaredigering.md` del 12 og 23):
//! `LevelEditor` i kjernen over nivådokumentet: modulene settes ut,
//! flyttes, dreies, speiles og slettes, åpningene får slag, innholdet
//! kobles, alt kan angres, og reglene (kompilatorens og nåbarheten,
//! koblingene og takhøyden) sier fra med sted. Økta viser nivået i 3D som
//! bevegelige instanser, og prøveturen kan ikke drepe deg.

use crate::adapters::data::{kit, level_doc, level_doc_json, level_doc_path};
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

/// Sett ut, flytt, slett, angre og gjør om: dokumentet går fram og tilbake
/// nøyaktig, og kartet følger.
#[test]
fn nivaaredigeringen_bygger_og_angrer() {
    let mut e = blank();
    let start = e.doc().clone();
    assert_eq!(e.doc().modules.len(), 1, "et nytt nivå begynner med kapellet");
    assert!(errors(&e).is_empty(), "kapellet alene har feil: {:?}", errors(&e));
    let made = op(&mut e, r#"{"op":"add_module","module":"grav/rom_2x2","at":[2,0,0]}"#);
    assert_eq!(made.len(), 1);
    let id = made[0].clone();
    assert_eq!(e.doc().modules.len(), 2);
    // Kapellet og rommet møtes: det blir en åpning mellom dem.
    let map = e.map();
    assert!(map.edges.iter().any(|x| x.at[0] == 1 && x.side == 1 && x.kind == "door"), "ingen dør mellom kapellet og rommet: {:?}", map.edges);
    assert!(map.modules.iter().any(|m| m.id == id && m.cells.len() == 4));
    op(&mut e, &format!(r#"{{"op":"move","ids":["{id}"],"by":[0,0,1]}}"#));
    assert_eq!(e.doc().modules[1].at, [2, 0, 1]);
    op(&mut e, &format!(r#"{{"op":"delete","ids":["{id}"]}}"#));
    assert_eq!(e.doc().modules.len(), 1);
    assert!(e.undo() && e.undo());
    assert_eq!(e.doc().modules[1].at, [2, 0, 0]);
    assert!(e.undo());
    assert_eq!(e.doc(), &start, "tre angresteg gir dokumentet tilbake");
    assert!(!e.can_undo());
    assert!(e.redo() && e.redo() && e.redo());
    assert_eq!(e.doc().modules.len(), 1);
    assert!(!e.can_redo());
    // Det som er galt, gjøres ikke.
    assert!(e.apply(LevelOp::AddModule { module: "grav/finnes_ikke".into(), at: [0, 0, 5], rot: 0, mirror: false }).is_err());
}

/// Dreiing og speiling av én modul og av en gruppe: fire kvarte dreininger
/// og to speilinger gir det samme tilbake, og gruppen holder sammen.
#[test]
fn nivaaredigeringen_dreier_og_speiler() {
    let mut e = blank();
    let a = op(&mut e, r#"{"op":"add_module","module":"grav/gang_2","at":[2,0,0]}"#)[0].clone();
    let b = op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[3,0,0]}"#)[0].clone();
    let before = e.doc().clone();
    let cells = |e: &LevelEditor, id: &str| {
        let mut v = e.footprint(e.doc().modules.iter().find(|m| m.id == id).unwrap());
        v.sort();
        v
    };
    let foot = cells(&e, &a);
    op(&mut e, &format!(r#"{{"op":"turn","ids":["{a}"]}}"#));
    assert_eq!(e.doc().modules[1].rot, 1);
    // Gangen på 2 × 1 står nå på tvers (N–S i stedet for Ø–V, eller omvendt).
    let turned = cells(&e, &a);
    let span = |v: &[[i32; 3]]| (v.iter().map(|c| c[0]).max().unwrap() - v.iter().map(|c| c[0]).min().unwrap(), v.iter().map(|c| c[2]).max().unwrap() - v.iter().map(|c| c[2]).min().unwrap());
    let (s0, s1) = (span(&foot), span(&turned));
    assert_eq!((s0.0, s0.1), (s1.1, s1.0), "dreid 90° skal bytte bredde og dybde: {foot:?} → {turned:?}");
    for _ in 0..3 {
        op(&mut e, &format!(r#"{{"op":"turn","ids":["{a}"]}}"#));
    }
    assert_eq!(e.doc(), &before, "fire kvarte dreininger gir det samme");
    // Gruppen dreies om den første modulen: rommet ved siden av flytter seg rundt.
    op(&mut e, &format!(r#"{{"op":"turn","ids":["{a}","{b}"]}}"#));
    let ma = e.doc().modules.iter().find(|m| m.id == a).unwrap().at;
    let mb = e.doc().modules.iter().find(|m| m.id == b).unwrap().at;
    assert_eq!((mb[0] - ma[0], mb[2] - ma[2]), (0, 1), "øst for gangen blir sør etter en kvart dreining med klokka");
    op(&mut e, &format!(r#"{{"op":"turn","ids":["{a}","{b}"],"back":true}}"#));
    assert_eq!(e.doc(), &before, "med og mot klokka opphever hverandre");
    op(&mut e, &format!(r#"{{"op":"mirror","ids":["{a}","{b}"]}}"#));
    let mb = e.doc().modules.iter().find(|m| m.id == b).unwrap();
    assert_eq!(mb.at, [1, 0, 0], "speilet langs x om gangen");
    assert!(mb.mirror);
    op(&mut e, &format!(r#"{{"op":"mirror","ids":["{a}","{b}"]}}"#));
    assert_eq!(e.doc(), &before, "to speilinger gir det samme");
    // Innhold i en modul følger med når den flyttes og dreies.
    let t = op(&mut e, r#"{"op":"add_thing","kind":"chest","at":[12.5,0,0.5],"yaw":0}"#)[0].clone();
    op(&mut e, &format!(r#"{{"op":"move","ids":["{b}"],"by":[0,0,2]}}"#));
    let p = e.doc().things.iter().find(|x| x.id == t).unwrap().pos.unwrap();
    assert_eq!(p, [12.5, 0.0, 8.5], "kista følger rommet");
}

/// Koblingene: en port på en åpning og en spak er feil hver for seg, og
/// riktige når de er koblet. Åpningene kan overstyres.
#[test]
fn nivaaredigeringen_kobler() {
    let mut e = blank();
    let r = op(&mut e, r#"{"op":"add_module","module":"grav/rom_3x2","at":[3,0,0]}"#)[0].clone();
    let map = e.map();
    let m = map.modules.iter().find(|m| m.id == r).unwrap();
    // Åpningen mot kapellet (vest for rommet, ruta i = 2).
    let opening = m.openings.iter().find(|o| o.side == 3 && o.at == [2, 0]).unwrap_or_else(|| panic!("ingen åpning mot vest: {:?}", m.openings)).id.clone();
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{opening}","kind":"port"}}"#));
    let gate = e.doc().modules.iter().find(|m| m.id == r).unwrap().overrides[0].id.clone().expect("porten får en id");
    assert!(e.map().edges.iter().any(|x| x.kind == "gate"), "porten står i kanten");
    assert!(errors(&e).iter().any(|x| x.contains(&format!("porten {gate} kan ikke åpnes"))), "porten uten spak: {:?}", errors(&e));
    let lever = op(&mut e, r#"{"op":"add_thing","kind":"lever","at":[1.0,0,1.0]}"#)[0].clone();
    assert!(errors(&e).iter().any(|x| x.contains(&format!("spaken {lever} åpner ingenting"))), "{:?}", errors(&e));
    op(&mut e, &format!(r#"{{"op":"link","from":"{lever}","to":"{gate}","kind":"opens"}}"#));
    assert!(!errors(&e).iter().any(|x| x.contains("kan ikke åpnes") || x.contains("åpner ingenting")), "koblet: {:?}", errors(&e));
    // Slettes spaken, går koblingen med.
    op(&mut e, &format!(r#"{{"op":"delete","ids":["{lever}"]}}"#));
    assert!(e.doc().links.is_empty());
    assert!(e.undo());
    assert_eq!(e.doc().links.len(), 1, "angre gir spaken og koblingen tilbake");
    // Mur i åpningen: porten er borte, og rommet kan ikke nås.
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{opening}","kind":"vegg"}}"#));
    assert!(!e.map().edges.iter().any(|x| x.kind == "gate"));
    let chest = op(&mut e, r#"{"op":"add_thing","kind":"chest","at":[16.0,0,2.0]}"#)[0].clone();
    assert!(errors(&e).iter().any(|x| x.contains(&chest) && x.contains("kan ikke nås")), "kista bak muren: {:?}", errors(&e));
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{opening}","kind":"auto"}}"#));
    assert!(!errors(&e).iter().any(|x| x.contains(&chest)), "åpent igjen: {:?}", errors(&e));
}

/// Reglene: overlapp, innhold i fjellet, lapp uten tekst, og «klar» bare
/// når alt er grønt.
#[test]
fn nivaaredigeringen_sier_fra_om_reglene() {
    let mut e = blank();
    // Mangler trappa ned og hvilestedet: advarsler, ikke feil.
    assert!(e.issues().iter().any(|i| i.rule == "innhold" && i.msg.contains("down")));
    op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[0,0,0]}"#);
    assert!(errors(&e).iter().any(|x| x.starts_with("overlapp")), "to moduler i samme rute: {:?}", errors(&e));
    let bad = e.issues().iter().find(|i| i.rule == "overlapp").unwrap();
    assert!(bad.at.is_some(), "bruddet har et sted å gå til");
    assert!(e.apply(LevelOp::SetHead { head: crate::domain::edit::level::Head { status: "klar".into(), ..e.head() } }).is_err(), "klar med feil");
    assert!(e.undo());
    let mob = op(&mut e, r#"{"op":"add_thing","kind":"mob","at":[40.0,0,40.0],"label":"button"}"#)[0].clone();
    assert!(errors(&e).iter().any(|x| x.contains(&mob) && x.contains("gulvet")), "fienden i fjellet: {:?}", errors(&e));
    op(&mut e, &format!(r#"{{"op":"move_thing","id":"{mob}","to":[0.0,0,2.0]}}"#));
    assert!(!errors(&e).iter().any(|x| x.contains(&mob)), "{:?}", errors(&e));
    let lore = op(&mut e, r#"{"op":"add_thing","kind":"lore","at":[2.0,0,2.0]}"#)[0].clone();
    assert!(errors(&e).iter().any(|x| x.contains(&format!("lappen {lore} er tom"))));
    let mut t = e.doc().things.iter().find(|t| t.id == lore).unwrap().clone();
    t.text = Some("Mester Ansgar var her.".into());
    e.apply(LevelOp::SetThing { thing: t }).unwrap();
    assert!(errors(&e).is_empty(), "{:?}", errors(&e));
    let head = crate::domain::edit::level::Head { status: "klar".into(), ..e.head() };
    e.apply(LevelOp::SetHead { head }).expect("uten feil kan nivået merkes klart");
    assert!(e.doc().ready());
}

/// Prøvenivået (bygd for hånd og gått gjennom med boten) har ingen feil i
/// reglene redigeringen bruker: de sier ikke fra om noe som virker.
#[test]
fn nivaaredigeringen_godtar_provenivaaet() {
    let e = LevelEditor::new(level_doc("test").expect("nivaa_test.json"), kit());
    assert!(errors(&e).is_empty(), "prøvenivået: {:?}", errors(&e));
    let map = e.map();
    assert_eq!(map.modules.len(), e.doc().modules.len());
    assert!(map.things.iter().any(|t| t.kind == "start" && !t.own), "startstedet kommer fra kapellet");
    assert!(map.edges.iter().any(|x| x.kind == "secret"), "den falske veggen står på kartet");
    assert!(map.floors.len() >= 2, "prøvenivået har to gulv: {:?}", map.floors);
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

/// Fase 4: kompilatorens regler for koblingene, takhøyden og antallet rom
/// står i redigeringens liste med et sted å gå til.
#[test]
fn nivaaredigeringen_bruker_kompilatorens_regler() {
    let mut e = blank();
    // En boss i et vanlig rom: taket er for lavt (bosshallen trengs).
    op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[2,0,0]}"#);
    let boss = op(&mut e, r#"{"op":"add_thing","kind":"boss","at":[8.0,0,0.0],"label":"navbar"}"#)[0].clone();
    let tall = e.issues().iter().find(|i| i.rule == "takhoyde" && i.msg.contains(&boss)).unwrap_or_else(|| panic!("ingen takhøyde for bossen: {:?}", errors(&e)));
    assert!(tall.at.is_some(), "takhøyden har et sted");
    op(&mut e, &format!(r#"{{"op":"delete","ids":["{boss}"]}}"#));
    // En spak som åpner noe som ikke finnes.
    let lever = op(&mut e, r#"{"op":"add_thing","kind":"lever","at":[1.0,0,1.0]}"#)[0].clone();
    op(&mut e, &format!(r#"{{"op":"link","from":"{lever}","to":"finnes_ikke","kind":"opens"}}"#));
    assert!(errors(&e).iter().any(|x| x.starts_with("kobling") && x.contains("finnes_ikke") && x.contains("finnes ikke")), "{:?}", errors(&e));
    // Det samme bruddet meldes én gang.
    let msgs: Vec<_> = e.issues().iter().map(|i| (i.rule, i.msg.clone())).collect();
    let mut uniq = msgs.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(msgs.len(), uniq.len(), "dobbelt i lista: {msgs:?}");
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
}

/// Paletten har modulene for alle temaene, gruppert som rom, ganger,
/// trapper, ferdigrom, mekanismer og bosshall, og hver modul har bildet sitt
/// (`module_shots.py`, `assets/levels/moduler/<tema>/<navn>.jpg`).
#[test]
fn paletten_har_alle_temaene_gruppert() {
    use crate::domain::edit::level::map;
    let k = kit();
    let p = map::palette(&k, vec![], vec![]);
    for theme in ["grav", "kloakk", "gruve", "borg", "kjerne"] {
        let mine: Vec<_> = p.modules.iter().filter(|m| m.theme == theme).collect();
        assert!(mine.len() >= 15, "{theme}: bare {} moduler", mine.len());
        for g in ["rom", "gang", "trapp", "mekanisme", "bosshall"] {
            assert!(mine.iter().any(|m| m.group == g), "{theme} har ingen i gruppen {g}");
        }
        for m in &mine {
            let name = m.id.split('/').nth(1).unwrap();
            let img = format!("{}/assets/levels/moduler/{theme}/{name}.jpg", env!("CARGO_MANIFEST_DIR"));
            assert!(std::path::Path::new(&img).exists(), "{} mangler bildet i paletten ({img})", m.id);
        }
    }
    let g = |id: &str| p.modules.iter().find(|m| m.id == id).map(|m| m.group);
    assert_eq!(g("grav/bosshall"), Some("bosshall"));
    assert_eq!(g("grav/porthall"), Some("mekanisme"));
    assert_eq!(g("grav/kapell"), Some("ferdig"));
    assert_eq!(g("grav/halvtrapp"), Some("trapp"));
    // Rekkefølgen følger gruppene.
    let order: Vec<usize> = p.modules.iter().map(|m| map::GROUPS.iter().position(|(x, _)| *x == m.group).unwrap()).collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]), "paletten står i gruppene sine");
}
