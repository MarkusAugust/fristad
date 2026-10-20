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
        let tall = check::headroom(&f);
        assert!(tall.is_empty(), "{}: {:?}", m.id, tall.iter().map(|i| &i.msg).collect::<Vec<_>>());
        let mech = check::mechanisms(&f);
        assert!(mech.is_empty(), "{}: {:?}", m.id, mech.iter().map(|i| &i.msg).collect::<Vec<_>>());
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

/// Hver side i galleriet (`moduler:<n>`, bildene til eieren) har plass til
/// lyset i alle rommene, alle modulene står på en side, og et nivå med
/// flere enn 64 rom er en feil.
#[test]
fn galleriene_har_lys_i_alle_rommene() {
    let k = kit();
    let pages = modules::gallery_pages(&k);
    assert!(!pages.is_empty());
    for (n, doc) in pages.iter().enumerate() {
        let (f, _) = compose::compile(doc, &k);
        assert!(check::rooms(&f).is_empty(), "side {}: {} rom", n + 1, f.rooms.len());
        assert!(f.marks.iter().any(|m| m.kind == "start"), "side {}: ingen start", n + 1);
    }
    let on_pages: usize = pages.iter().map(|d| d.modules.iter().filter(|m| !m.id.contains("_nabo")).count()).sum();
    assert_eq!(on_pages, k.modules.len(), "hver modul står på én side");
    let (all, _) = compose::compile(&modules::gallery(&k), &k);
    assert!(!check::rooms(&all).is_empty(), "alle modulene i ett nivå skal være for mange rom ({})", all.rooms.len());
}

/// Bossene er 4–5 m høye: bosshallen har plass til dem i hvert tema, med
/// arenaporten bak deg, og en boss i et vanlig rom er en feil.
#[test]
fn bossene_far_plass_i_bosshallen() {
    let k = kit();
    assert!(check::mob_height("navbar") >= 5.0 && check::mob_height("rammeverket") >= 4.5, "{} {}", check::mob_height("navbar"), check::mob_height("rammeverket"));
    for t in ["grav", "kloakk", "gruve", "borg", "kjerne"] {
        let (f, _) = showcase(&format!("{t}/bosshall"));
        let boss = marks(&f, "boss");
        assert_eq!(boss.len(), 1, "{t}");
        let gate = marks(&f, "gate");
        assert!(gate.len() == 1 && gate[0].arena.is_some(), "{t}: arenaporten");
        let hall = f.rooms.iter().map(|r| r.h).fold(0.0f32, f32::max);
        assert!(hall >= 7.0, "{t}: hallen er {hall} m");
    }
    let mut doc = modules::showcase(&k, "grav/rom_1x1").unwrap();
    doc.things.push(crate::domain::world::crypt::doc::Thing { id: "boss".into(), kind: "boss".into(), anchor: "celle:m/c00".into(), pos: Some([0.0; 3]), label: Some("navbar".into()), ..Default::default() });
    let (f, _) = compose::compile(&doc, &k);
    assert!(check::headroom(&f).iter().any(|i| i.msg.contains("navbar")), "{:?}", check::headroom(&f));
}

/// Mekanismene med egne biter (`kit_mech.py`, docs/nivaaredigering.md del
/// 25): vindebroa over kløfta (dekket er gangbart bak broas bryter), heisen
/// i sjakta mellom to etasjer, stigen opp til fallemmen over luka, galleriet
/// med trappa og gangen ut fra galleriet, og kanalen med vann, i alle temaene
/// (kanalen i kloakken).
#[test]
fn mekanismene_har_bitene_sine() {
    let model = |f: &LevelFile, name: &str| f.instances.iter().filter(|i| i.model == name).count();
    for t in ["grav", "kloakk", "gruve", "borg", "kjerne"] {
        let (f, _) = showcase(&format!("{t}/vindebro"));
        assert_eq!(model(&f, check::CHASM), 1, "{t}: kløfta");
        let bro = marks(&f, "bridge");
        assert!(bro.len() == 1 && bro[0].top.is_some(), "{t}: broa med feste i hvelvet");
        let id = bro[0].id.clone().unwrap();
        assert!(f.nav.poly_gate.iter().any(|g| g.as_deref() == Some(id.as_str())), "{t}: dekket er gangbart bak broas bryter");

        let (f, _) = showcase(&format!("{t}/heis"));
        let heis = marks(&f, "lift");
        assert!(heis.len() == 1 && heis[0].bottom.is_some_and(|b| heis[0].pos.unwrap()[1] - b[1] > 5.9), "{t}: heisen går 6 m");
        assert!(f.rooms.len() >= 2 && f.rooms.iter().any(|r| r.y > 5.9), "{t}: rommet oppe");

        let (f, _) = showcase(&format!("{t}/stige"));
        let stige = marks(&f, "ladder");
        assert!(stige.len() == 1 && stige[0].top.is_some_and(|p| p[1] > 5.9), "{t}: stigen når opp");
        assert!(marks(&f, "trapdoor").first().is_some_and(|l| l.to.is_some()), "{t}: fallemmen vet rommet under");

        let (f, _) = showcase(&format!("{t}/galleri"));
        let ys: Vec<f32> = f.nav.verts.iter().map(|v| v[1]).collect();
        assert!(ys.iter().any(|y| (y - 5.0).abs() < 0.01) && ys.iter().any(|y| y.abs() < 0.01), "{t}: galleriet har gulv nede og oppe");
        assert!(f.instances.iter().any(|i| i.model.starts_with("grav_vegg") && (i.pos[1] - 5.0).abs() < 0.01), "{t}: kanten ut fra galleriet står 5 m oppe");
    }
    // Galleriet føyes sammen med en gang 5 m oppe i nord: gangåpning, ingen mur.
    let k = kit();
    let mut doc = modules::showcase(&k, "grav/galleri").unwrap();
    doc.modules.push(crate::domain::world::crypt::doc::DocModule { id: "g".into(), module: "grav/gang_1".into(), at: [0, 5, -2], ..Default::default() });
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", r.errors().map(|i| &i.msg).collect::<Vec<_>>());
    assert!(!f.instances.iter().any(|i| i.model == "grav_vegg" && (i.pos[1] - 5.0).abs() < 0.01 && (i.pos[2] + 6.0).abs() < 0.01), "muren står i gangen ut fra galleriet");
    let (f, _) = showcase("kloakk/kanal");
    assert_eq!(f.instances.iter().filter(|i| check::CANALS.contains(&i.model.as_str())).count(), 3);
    assert!(marks(&f, "water").first().is_some_and(|w| w.levels.len() == 2 && w.levels[0] > w.levels[1]), "kanalen har vann som kan tappes");
}

/// Plastsømmene i kjernen: muren med sømmer (`grav/vegg` varianten `som`)
/// står bare der steinen vil ha den (kjernestein og plast), aldri ellers.
#[test]
fn kjernen_har_plastsommer_i_muren() {
    let count = |id: &str| showcase(id).0.instances.iter().filter(|i| i.model == "grav_vegg_som").count();
    assert!(count("kjerne/rom_3x3") > 0, "kjernen har sømmer i muren");
    for id in ["grav/rom_3x3", "kloakk/rom_3x3", "borg/rom_3x3"] {
        assert_eq!(count(id), 0, "{id}: sømmene hører kjernen til");
    }
}

/// Reglene for mekanismebitene: kløft uten bro, bro uten kløft, heis uten
/// sjakt, fallem uten luke og kanal uten vann.
#[test]
fn mekanismene_uten_bitene_sine_er_feil() {
    let k = kit();
    let msgs = |doc: &crate::domain::world::crypt::doc::LevelDoc| -> Vec<String> { check::mechanisms(&compose::compile(doc, &k).0).into_iter().map(|i| i.msg).collect() };
    let mut doc = modules::showcase(&k, "grav/vindebro").unwrap();
    let full = modules::flatten(&doc, &k).0;
    doc = crate::domain::world::crypt::doc::LevelDoc { things: full.things.iter().filter(|t| t.kind != "bridge").cloned().collect(), links: vec![], ..full.clone() };
    assert!(msgs(&doc).iter().any(|m| m.contains("kløfta har ingen vindebro")), "{:?}", msgs(&doc));
    let mut doc = modules::showcase(&k, "grav/rom_1x1").unwrap();
    for (kind, id) in [("bridge", "bro"), ("lift", "heis"), ("trapdoor", "lem")] {
        doc.things.push(crate::domain::world::crypt::doc::Thing { id: id.into(), kind: kind.into(), anchor: "celle:m/c00".into(), pos: Some([0.0; 3]), bottom: Some([0.0, -6.0, 0.0]), ..Default::default() });
    }
    let m = msgs(&doc);
    for want in ["vindebroa bro står ikke over en kløft", "heisen heis står ikke i en sjakt", "fallemmen lem ligger ikke over en luke"] {
        assert!(m.iter().any(|x| x.contains(want)), "mangler «{want}»: {m:?}");
    }
    let mut doc = modules::showcase(&k, "kloakk/kanal").unwrap();
    let full = modules::flatten(&doc, &k).0;
    doc = crate::domain::world::crypt::doc::LevelDoc { things: full.things.iter().filter(|t| t.kind != "water").cloned().collect(), links: vec![], ..full.clone() };
    assert!(msgs(&doc).iter().any(|m| m.contains("kanalen har ikke vann")), "{:?}", msgs(&doc));
}


/// Stigen og heisen er lenker i navigasjonsnettet (`crypt::hops`): en
/// fiende nede finner fram til deg oppe over stigen når fallemmen er åpen,
/// og med heisen når bremsen er løsnet og heisen står nede der den er.
/// Nåbarheten i den kompilerte fila (`compose::reach`, redigeringens regel)
/// går over begge.
#[test]
fn fiendene_folger_deg_opp_stigen_og_med_heisen() {
    use crate::domain::mobs::{Kind, Mobs};
    use crate::domain::world::{crypt, Builder};
    use glam::{Vec2, Vec3};
    let (below, above) = (Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 6.0, -4.0));
    for id in ["grav/stige", "grav/heis"] {
        let (f, _) = showcase(id);
        // Fila: oppe kan nås fra nede bare over lenken.
        let (nav, seen) = compose::reach(&f, below).unwrap_or_else(|| panic!("{id}: nede står ikke på nettet"));
        let all = |_: &crypt::nav::Poly| true;
        let (top, _) = nav.nearest(above, 1.5, &all).unwrap_or_else(|| panic!("{id}: oppe står ikke på nettet"));
        assert!(seen[top as usize], "{id}: oppe kan ikke nås fra nede");
        assert_eq!(nav.hops.len(), 1, "{id}: én lenke");
        let mut bare = nav.clone();
        bare.hops.clear();
        let (a, _) = bare.nearest(below, 1.5, &all).unwrap();
        assert!(!bare.reachable(a, &all)[top as usize], "{id}: oppe kan nås uten lenken");

        // Krypten: veien er stengt til fallemmen eller bremsen er åpnet.
        let mut c = crypt::build(&mut Builder::new(&[]), Vec2::ZERO, vec![(1, f.clone())]);
        let o = c.levels[0].origin;
        let (from, to) = (below + o, above + o);
        assert!(c.route(from, to).is_none(), "{id}: veien går over lenken med fallemmen eller bremsen stengt");
        c.open = vec![true; c.switches.len()];
        if id == "grav/heis" {
            // Heisen står oppe: fienden nede kommer ikke med den, og heller
            // ikke mens den kjører.
            for rest in [Some(false), None] {
                c.lift_rest = vec![rest];
                assert!(c.route(from, to).is_none(), "{id}: veien går med heisen {rest:?}");
            }
            c.lift_rest = vec![Some(true)];
        }
        assert!(c.path(from, to).is_none(), "{id}: veien til fots går opp");
        let path = c.route(from, to).unwrap_or_else(|| panic!("{id}: ingen vei opp"));
        assert!(path.iter().any(|p| p.y > o.y + 1.0 && p.y < o.y + 5.0) || path.windows(2).any(|w| (w[1].y - w[0].y).abs() > 3.0), "{id}: veien går ikke opp lenken: {path:?}");

        // Fienden går dit, klatrer eller kjører, og tar deg igjen oppe.
        let mut mobs = Mobs::new(&c, &[]);
        let i = mobs.spawn_aware(Kind::Button, from);
        let (mut rode, mut best) = (false, f32::MAX);
        for _ in 0..30 * 40 {
            let out = mobs.update(1.0 / 30.0, to, true, &c);
            rode |= out.lifts.contains(&0);
            let p = mobs.mobs[i].pos;
            best = best.min(p.distance(to));
            if best < 2.5 {
                break;
            }
        }
        assert!(best < 2.5, "{id}: fienden kom ikke opp til deg (nærmest {best:.1} m, står ved {:?})", mobs.mobs[i].pos - o);
        assert_eq!(rode, id == "grav/heis", "{id}: fienden kjørte heisen");
    }
}

/// Den boltede fallemmen åpnes bare nedenfra: står du på den, faller du
/// ikke, og fiendene kan ikke gå over stigen under den; klatrer du opp
/// stigen, går den opp, og da kan fiendene også bruke stigen.
#[test]
fn den_boltede_fallemmen_apnes_bare_nedenfra() {
    use crate::adapters::rapier::RapierPhysics;
    use crate::domain::world::Use;
    use crate::ports::driving::Input;
    use glam::Vec3;
    let c = &crate::tests::world().world.crypt;
    let t = c.trapdoors.iter().find(|t| t.bolted).expect("ingen boltet fallem i krypten").clone();
    let (n, ladder) = c.ladders.iter().enumerate().filter(|(_, l)| l.level == t.level).min_by(|a, b| a.1.top.distance(t.pos).total_cmp(&b.1.top.distance(t.pos))).expect("ingen stige under lemmen");
    assert!(ladder.top.distance(t.pos) < 2.5, "stigen når ikke lemmen");
    let mut physics = RapierPhysics::new();
    let mut game = crate::tests::session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    game.place_feet(t.pos + Vec3::Y * 0.3);
    for _ in 0..90 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!(game.feet().y > t.pos.y - 0.5, "du falt gjennom den boltede lemmen ({:?})", game.feet());
    assert!(!game.crypt_now().open[t.switch], "den boltede lemmen gikk opp ovenfra");
    let below = ladder.pos + Vec3::new(ladder.yaw.sin(), 0.0, ladder.yaw.cos()) * 0.6;
    let hop = c.levels[t.level].nav.hops.iter().find(|h| h.pts.iter().any(|p| p.distance(ladder.top) < 0.1)).expect("stigen er ikke en lenke i nettet").clone();
    assert_eq!(hop.gate, Some(t.switch), "lemmen stenger ikke stigen");
    let usable = |game: &crate::application::Session| [true, false].map(|f| game.crypt_now().hop_open(&hop, f));
    assert_eq!(usable(&game), [false, false], "fiendene går over stigen gjennom den stengte lemmen");
    game.place_feet(below);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.use_exact(Use::Ladder(n));
    assert!(game.crypt_now().open[t.switch], "lemmen gikk ikke opp nedenfra");
    assert_eq!(usable(&game), [true, true], "fiendene kan ikke bruke stigen når lemmen er åpen");
}
