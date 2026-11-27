//! Nivåredigeringen, fase 3 (`docs/nivaaredigering.md` del 12 og 23):
//! `LevelEditor` i kjernen over nivådokumentet: modulene settes ut,
//! flyttes, dreies, speiles og slettes, åpningene får slag, innholdet
//! kobles, alt kan angres, og reglene (kompilatorens og nåbarheten,
//! koblingene og takhøyden) sier fra med sted. Økta med nivået i 3D og
//! lagringen i fast form testes i hovedpakken (`src/level_editor_tests.rs`).

use crate::data::{kit, level_doc};
use crate::domain::edit::level::{LevelEditor, LevelOp};
use crate::domain::world::crypt::compose::Severity;

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

/// Eierens dom ved «klar» (eieren 26. november): en underkjent modul eller
/// ting stopper, usikkert og det som venter på dom advarer, og et utkast
/// kan bruke alt.
#[test]
fn underkjent_stopper_klar_og_usikkert_advarer() {
    use crate::domain::edit::level::verdicts::{Verdict, Verdicts};
    let odd = |state: &str, why: &str| Verdict { state: state.into(), why: why.into() };
    let module = blank().doc().modules[0].module.clone();
    let klar = |e: &mut LevelEditor| {
        let mut head = e.head();
        head.status = "klar".into();
        e.apply(LevelOp::SetHead { head })
    };
    let dom = |e: &LevelEditor| e.issues().iter().filter(|i| i.rule == "dom").map(|i| (i.severity == Severity::Error, i.msg.clone())).collect::<Vec<_>>();

    // Uten avvik: ingen melding om dommen.
    assert!(dom(&blank()).is_empty());

    // Usikker modul og en ting som venter: advarsler, ikke feil.
    let mut v = Verdicts::default();
    v.modules.insert(module.clone(), odd("usikker", "for mørk"));
    v.things.insert("start".into(), odd("venter", ""));
    let e = blank().with_verdicts(v);
    let found = dom(&e);
    assert!(found.iter().all(|(error, _)| !error), "{found:?}");
    assert!(found.iter().any(|(_, m)| m.contains("er usikker: for mørk")), "{found:?}");
    assert!(found.iter().any(|(_, m)| m.contains("venter på dom")), "{found:?}");

    // Underkjent modul: en feil med eierens grunn, og nivået kan ikke merkes klart.
    let mut v = Verdicts::default();
    v.modules.insert(module, odd("underkjent", "feil stein"));
    let mut e = blank().with_verdicts(v);
    assert!(dom(&e).iter().any(|(error, m)| *error && m.contains("er underkjent: feil stein")), "{:?}", dom(&e));
    assert!(!e.clean());
    assert!(klar(&mut e).is_err(), "et nivå med en underkjent modul ble merket klart");
    assert_eq!(e.doc().status, "utkast");
}
