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
    assert!(e.issues().iter().any(|i| i.rule == "innhold" && i.msg == "Nivået mangler trappa ned"));
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

/// Toppen og bunnen (stigen, heisen, broa) og vannstanden følger innholdet
/// når det flyttes, dreies og limes inn.
#[test]
fn toppen_og_bunnen_folger_innholdet() {
    let mut e = blank();
    let r = op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[2,0,0]}"#)[0].clone();
    let l = op(&mut e, r#"{"op":"add_thing","kind":"ladder","at":[8.0,0,0.0]}"#)[0].clone();
    let mut t = e.doc().things.iter().find(|x| x.id == l).unwrap().clone();
    t.top = Some([8.0, 6.0, 0.5]);
    t.bottom = Some([8.0, -1.0, 0.0]);
    t.levels = vec![1.0, 0.2];
    e.apply(LevelOp::SetThing { thing: t }).unwrap();
    e.apply(LevelOp::MoveThing { id: l.clone(), to: [9.0, 1.0, 0.0] }).unwrap();
    let t = e.doc().things.iter().find(|x| x.id == l).unwrap().clone();
    assert_eq!((t.top, t.bottom), (Some([9.0, 7.0, 0.5]), Some([9.0, 0.0, 0.0])));
    assert_eq!(t.levels, vec![2.0, 1.2]);
    e.apply(LevelOp::Move { ids: vec![r.clone()], by: [0, 0, 1] }).unwrap();
    let t = e.doc().things.iter().find(|x| x.id == l).unwrap().clone();
    assert_eq!(t.top, Some([9.0, 7.0, 4.5]), "toppen følger modulen");
    e.apply(LevelOp::Turn { ids: vec![r.clone()], back: false }).unwrap();
    let t = e.doc().things.iter().find(|x| x.id == l).unwrap().clone();
    // Dreid om modulens midte (8, 4): (9, 4.5) → (7.5, 5).
    assert_eq!(t.top, Some([7.5, 7.0, 5.0]), "toppen dreies med");
    assert_eq!(t.pos, Some([8.0, 1.0, 5.0]));
}

/// Etasjene (fase 4): stigen og heisen kobler etasjene i nåbarheten, en
/// modul ingen kommer til, meldes, og en trapp som ikke går noen vei i den
/// ene enden, meldes.
#[test]
fn etasjene_henger_sammen() {
    let mut e = blank();
    // Stigen øst for kapellet: forrommet nede mot kapellet, rommet oppe 6 m over.
    let s = op(&mut e, r#"{"op":"add_module","module":"grav/stige","at":[2,0,-1]}"#)[0].clone();
    // (En kiste midt i rommet oppe stenger den smale gangen i nettet; en fiende gjør ikke det.)
    let chest = op(&mut e, r#"{"op":"add_thing","kind":"mob","at":[8.0,6.0,-8.0],"label":"button"}"#)[0].clone();
    assert!(!errors(&e).iter().any(|x| x.contains(&chest)), "fienden oppe nås over stigen: {:?}", errors(&e));
    assert!(!e.issues().iter().any(|i| i.rule == "naa" && i.msg.contains("kan ikke nås fra start")), "{:?}", e.issues());
    assert!(e.map().things.iter().any(|t| t.kind == "ladder" && t.other_end.is_some_and(|p| p[1] > 5.0)), "kartet kjenner toppen av stigen");
    // Uten stigen kommer ingen opp.
    let ladder = e.map().things.into_iter().find(|t| t.kind == "ladder").unwrap().id;
    e.apply(LevelOp::Delete { ids: vec![ladder] }).unwrap();
    assert!(errors(&e).iter().any(|x| x.contains(&chest) && x.contains("kan ikke nås")), "{:?}", errors(&e));
    assert!(e.undo());
    let _ = s;
    // En halv etasje inntil kapellet: den ene enden møter kapellet, den andre ingenting.
    op(&mut e, r#"{"op":"add_module","module":"grav/halv_etasje","at":[-2,0,0]}"#);
    let stair = e.issues().iter().filter(|i| i.rule == "trapp").map(|i| i.msg.clone()).collect::<Vec<_>>();
    assert!(!stair.is_empty() && stair.iter().all(|m| m.contains("går ikke noen vei")), "{stair:?}");
    // Et rom ingen kommer til.
    op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[10,0,10]}"#);
    assert!(e.issues().iter().any(|i| i.rule == "naa" && i.msg.contains("kan ikke nås fra start")), "{:?}", e.issues());
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

/// «Rett» (del 27): de vanlige bruddene har en retting når den er entydig,
/// og den som endrer dokumentet, er en vanlig handling med angre.
#[test]
fn rett_retter_de_vanlige_bruddene() {
    use crate::domain::edit::level::fixes::FixKind;
    let fix = |e: &LevelEditor, rule: &str, has: &str| {
        let k = e.issues().iter().position(|i| i.rule == rule && i.msg.contains(has)).unwrap_or_else(|| panic!("ingen {rule} med «{has}»: {:?}", e.issues()));
        e.fix(k).cloned().unwrap_or_else(|| panic!("ingen retting for {:?}", e.issues()[k]))
    };
    let mut e = blank();
    // En spak uten mål kobles til porten.
    let r = op(&mut e, r#"{"op":"add_module","module":"grav/rom_3x2","at":[3,0,0]}"#)[0].clone();
    let opening = e.map().modules.iter().find(|m| m.id == r).unwrap().openings.iter().find(|o| o.side == 3 && o.at == [2, 0]).unwrap().id.clone();
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{opening}","kind":"port"}}"#));
    let gate = e.doc().modules.iter().find(|m| m.id == r).unwrap().overrides[0].id.clone().unwrap();
    let lever = op(&mut e, r#"{"op":"add_thing","kind":"lever","at":[1.0,0,1.0]}"#)[0].clone();
    let f = fix(&e, "kobling", &format!("spaken {lever} åpner ingenting"));
    let FixKind::Op { op: o } = f.what else { panic!("{f:?}") };
    assert_eq!(o, LevelOp::Link { from: lever.clone(), to: gate.clone(), kind: "opens".into() });
    e.apply(o).unwrap();
    assert!(!errors(&e).iter().any(|x| x.contains("åpner ingenting") || x.contains("kan ikke åpnes")), "{:?}", errors(&e));
    assert!(e.undo() && errors(&e).iter().any(|x| x.contains("åpner ingenting")), "angre gir bruddet tilbake");
    assert!(e.redo());
    // Den tomme lappen: tekstfeltet.
    let lore = op(&mut e, r#"{"op":"add_thing","kind":"lore","at":[2.0,0,2.0]}"#)[0].clone();
    let f = fix(&e, "innhold", &format!("lappen {lore} er tom"));
    assert_eq!(f.what, FixKind::Focus { thing: lore, field: "text" });
    // Mangler trappa ned: «Sett ut» tar den fra paletten.
    assert_eq!(fix(&e, "innhold", "Nivået mangler trappa ned").what, FixKind::Arm { kind: "down" });
    // En boss i et vanlig rom: bosshallen.
    let boss = op(&mut e, r#"{"op":"add_thing","kind":"boss","at":[16.0,0,0.0],"label":"navbar"}"#)[0].clone();
    assert_eq!(fix(&e, "takhoyde", &boss).what, FixKind::Palette { module: "grav/bosshall".into() });
    assert!(e.kit().module("grav/bosshall").is_some());
    // Et rom ingen kommer til: den nærmeste åpningen.
    let far = op(&mut e, r#"{"op":"add_module","module":"grav/rom_1x1","at":[10,0,0]}"#)[0].clone();
    let FixKind::Show { at, module } = fix(&e, "naa", "kan ikke nås fra start (ingen åpning").what else { panic!() };
    assert_eq!(module, far);
    assert!((at[0] - 38.0).abs() < 0.01 && at[2].abs() < 0.01, "åpningen mot vest, nærmest rommet: {at:?}");
    // En dør mot ingenting: mur.
    let west = e.map().modules.iter().find(|m| m.id == far).unwrap().openings.iter().find(|o| o.side == 3).unwrap().id.clone();
    op(&mut e, &format!(r#"{{"op":"opening","module":"{far}","opening":"{west}","kind":"dor"}}"#));
    let f = fix(&e, "aapning", "møter ingenting");
    assert_eq!(f.label, "Gjør til mur");
    let FixKind::Op { op: o } = f.what else { panic!() };
    e.apply(o).unwrap();
    assert!(!e.issues().iter().any(|i| i.rule == "aapning"), "{:?}", e.issues());
    assert_eq!(e.doc().modules.iter().find(|m| m.id == far).unwrap().overrides[0].kind, "vegg");
    assert!(e.undo() && e.issues().iter().any(|i| i.rule == "aapning"));
    // Kartet har rettingen med til siden.
    assert!(e.map().issues.iter().any(|i| i.fix.as_ref().is_some_and(|f| f.label == "Gjør til mur")));
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

/// Åpningene på kanten av utvalget tar med eierens slag inn i den nye
/// modulen (del 27): porten mot et rom utenfor utvalget står der igjen med
/// samme navn og spaken koblet, og muren eieren satte i en åpning mot
/// ingenting er mur i modulen.
#[test]
fn utvalget_tar_med_slaget_paa_kanten() {
    let mut e = blank();
    let r = op(&mut e, r#"{"op":"add_module","module":"grav/rom_3x2","at":[10,0,0]}"#)[0].clone();
    let map = e.map();
    let m = map.modules.iter().find(|m| m.id == r).unwrap();
    let west = m.openings.iter().filter(|o| o.side == 3).min_by_key(|o| o.at[1].abs()).unwrap().clone();
    let east = m.openings.iter().find(|o| o.side == 1).expect("en åpning mot øst").clone();
    let g = op(&mut e, &format!(r#"{{"op":"add_module","module":"grav/rom_1x1","at":[{},0,{}]}}"#, west.at[0] - 1, west.at[1]))[0].clone();
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{}","kind":"port"}}"#, west.id));
    op(&mut e, &format!(r#"{{"op":"opening","module":"{r}","opening":"{}","kind":"vegg"}}"#, east.id));
    let gate = e.doc().modules.iter().find(|m| m.id == r).unwrap().overrides[0].id.clone().unwrap();
    let lever = op(&mut e, r#"{"op":"add_thing","kind":"lever","at":[41.0,0,1.0]}"#)[0].clone();
    op(&mut e, &format!(r#"{{"op":"link","from":"{lever}","to":"{gate}","kind":"opens"}}"#));
    let gates = |e: &LevelEditor| e.file().marks.iter().filter(|m| m.kind == "gate").count();
    assert_eq!(gates(&e), 1, "porten mot rommet utenfor");
    let errs = errors(&e);
    // Bare rommet med porten og spaken; rommet i vest er utenfor utvalget.
    let def = e.as_module(&[r.clone(), lever.clone()], "Rom med port", "rom_port").expect("modulen");
    let o = def.openings.iter().find(|o| o.kind == "port").unwrap_or_else(|| panic!("porten følger med: {:?}", def.openings));
    assert_eq!((o.at, o.side.as_str(), o.id.as_str()), ([west.at[0] - 10, west.at[1]], "W", gate.as_str()), "på samme kant, med samme navn");
    assert!(def.walls.iter().any(|w| w.at == [east.at[0] - 10, east.at[1]] && w.side == "E"), "muren eieren satte: {:?}", def.walls);
    assert!(def.links.iter().any(|l| l.to == gate && l.kind == "opens"), "spaken peker på porten: {:?}", def.links);
    // Satt ut igjen der den sto, ved siden av det samme rommet.
    e.add_module(def);
    op(&mut e, &format!(r#"{{"op":"delete","ids":["{r}","{lever}"]}}"#));
    assert_eq!(gates(&e), 0, "porten gikk med rommet");
    let n = op(&mut e, r#"{"op":"add_module","module":"grav/rom_port","at":[10,0,0]}"#)[0].clone();
    assert!(e.doc().modules.iter().any(|m| m.id == g));
    let id = format!("{n}/{gate}");
    assert!(e.file().marks.iter().any(|m| m.kind == "gate" && m.id.as_deref() == Some(id.as_str())), "porten står der igjen som {id}");
    assert_eq!(gates(&e), 1);
    assert!(e.file().marks.iter().any(|m| m.kind == "lever" && m.opens.contains(&id)), "spaken åpner porten");
    assert_eq!(errors(&e).len(), errs.len(), "de samme bruddene som før: {:?}", errors(&e));
}
