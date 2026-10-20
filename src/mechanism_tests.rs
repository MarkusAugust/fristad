//! Mekanismene som moduler (`docs/nivaaredigering.md` del 24): hver modul
//! med en mekanisme virker alene (koblingene inni modulen), og reglene over
//! koblingene (`compose::check::links`) fanger det som ikke henger sammen.

use crate::adapters::data::kit;
use crate::domain::world::crypt::compose::{self, check, modules};
use crate::domain::world::crypt::level::{LevelFile, MarkDef};

fn showcase(id: &str) -> (LevelFile, compose::Report) {
    let k = kit();
    let doc = modules::showcase(&k, id).unwrap_or_else(|| panic!("modulen {id} finnes ikke"));
    compose::compile(&doc, &k)
}

fn marks<'a>(f: &'a LevelFile, kind: &str) -> Vec<&'a MarkDef> {
    f.marks.iter().filter(|m| m.kind == kind).collect()
}

/// Hver modul i alle temaene kompileres alene uten brudd i koblingene, og
/// uten andre feil enn åpningene som ender i vegg.
#[test]
fn modulene_har_hele_koblinger() {
    let k = kit();
    assert!(!k.modules.is_empty());
    for m in &k.modules {
        let (f, r) = showcase(&m.id);
        let bad: Vec<String> = r.errors().filter(|i| !i.msg.contains("ender i vegg")).map(|i| format!("{}: {}", i.rule, i.msg)).collect();
        assert!(bad.is_empty(), "{}: {bad:?}", m.id);
        let links = check::links(&f);
        assert!(links.is_empty(), "{}: {:?}", m.id, links.iter().map(|i| &i.msg).collect::<Vec<_>>());
    }
}

/// Spaken åpner porten, eliten bærer nøkkelen, snarveien har slåsiden,
/// trykkplata har pilfella og hvilestedet har lappen.
#[test]
fn mekanismene_virker_i_modulene() {
    let (f, _) = showcase("grav/porthall");
    let gate = marks(&f, "gate");
    assert_eq!(gate.len(), 1, "porthallen har én port");
    let id = gate[0].id.clone().unwrap();
    assert!(marks(&f, "lever").iter().any(|l| l.opens.contains(&id)), "spaken åpner porten");
    assert_eq!(marks(&f, "chest").len(), 1);

    let (f, _) = showcase("grav/nokkelport");
    let id = marks(&f, "gate")[0].id.clone().unwrap();
    let elite = marks(&f, "elite");
    assert!(elite.len() == 1 && elite[0].opens.contains(&id), "eliten bærer nøkkelen til porten");
    assert!(elite[0].patrol.len() >= 2, "eliten går vakt");

    let (f, _) = showcase("grav/snarvei");
    let s = marks(&f, "shortcut");
    assert!(s.len() == 1 && s[0].from.is_some(), "snarveien har slåen på én side");

    let (f, _) = showcase("grav/fellegang");
    assert_eq!(marks(&f, "trap").first().and_then(|t| t.label.as_deref()), Some("arrows"));

    let (f, _) = showcase("grav/hvilested");
    assert_eq!(marks(&f, "rest").len(), 1);
    assert!(marks(&f, "lore").iter().all(|l| l.text.as_deref().is_some_and(|t| !t.is_empty())));
}

/// Reglene fanger koblinger som ikke henger sammen.
#[test]
fn koblingene_som_ikke_henger_sammen_er_feil() {
    let mark = |kind: &str, id: &str, opens: &[&str]| MarkDef { kind: kind.into(), id: Some(id.into()), pos: Some([0.0; 3]), opens: opens.iter().map(|s| s.to_string()).collect(), ..Default::default() };
    let f = LevelFile {
        marks: vec![
            mark("lever", "spak", &["borte"]),
            mark("gate", "port", &[]),
            mark("bridge", "bro", &[]),
            mark("water", "sluse", &[]),
            mark("lift", "heis", &[]),
            mark("mob", "knapp", &["port"]),
            mark("elite", "elite", &["bro"]),
            MarkDef { kind: "hint".into(), id: Some("sprekk".into()), for_id: Some("port".into()), ..Default::default() },
        ],
        ..Default::default()
    };
    let msgs: Vec<String> = check::links(&f).into_iter().map(|i| i.msg).collect();
    for want in ["«borte», som ikke finnes", "porten port kan ikke åpnes", "vindebroa bro har ingen spak", "slusa sluse har ingen spak", "heisen heis har ingen spak", "bare eliten", "passer bare i en port", "ikke er en falsk vegg"] {
        assert!(msgs.iter().any(|m| m.contains(want)), "mangler «{want}»: {msgs:#?}");
    }
    let ok = LevelFile { marks: vec![mark("lever", "spak", &["port", "bro"]), mark("gate", "port", &[]), mark("bridge", "bro", &[])], ..Default::default() };
    assert!(check::links(&ok).is_empty(), "{:?}", check::links(&ok));
}
