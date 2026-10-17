//! Testene for redigeringsmodusen (`docs/redigering.md`, steg 3): reglene
//! over dagens by, og redigeringen i kjernen og økta.

use crate::adapters::data::world_data;
use crate::domain::edit::{rules, town_scene, Sources};
use std::collections::{HashMap, HashSet};

fn sources() -> (Sources, crate::domain::world::layout::Layout, HashMap<String, Vec<crate::domain::world::layout::Item>>) {
    let p = crate::tests::world();
    let d = world_data();
    (Sources { models: p.infos.clone(), insides: d.insides, levels: d.levels }, d.layout.expect("byen"), d.contents.into_iter().collect())
}

/// Dagens by bryter ingen regler.
#[test]
fn dagens_by_folger_reglene() {
    let (src, layout, contents) = sources();
    let b = town_scene(&src, &layout, &contents);
    let breaches = rules::check(&b, &layout, None, &HashSet::new());
    let lines: Vec<String> = breaches.iter().map(|b| format!("{:?}: {}", b.r, b.why)).collect();
    assert!(breaches.is_empty(), "{} brudd:\n{}", lines.len(), lines.join("\n"));
}

/// Reglene slår ut: en tønne løftet 50 cm svever, en tønne midt i en
/// markedsbod står inni den, et hus flyttet inntil et annet står for tett,
/// et hus midt på torget står ute på torget, og en endret ting med en modell
/// som ikke er godkjent, er ikke godkjent.
#[test]
fn reglene_finner_feil() {
    use crate::domain::world::layout::Ref;
    let (src, layout, contents) = sources();
    let find = |l: &crate::domain::world::layout::Layout, model: &str| l.items.iter().position(|it| it.model.as_deref() == Some(model)).unwrap();
    let breaches = |l: &crate::domain::world::layout::Layout, approved: Option<&HashSet<String>>, touched: &HashSet<Ref>| {
        let b = town_scene(&src, l, &contents);
        rules::check(&b, l, approved, touched)
    };
    let has = |bs: &[rules::Breach], r: &Ref, word: &str| bs.iter().any(|b| &b.r == r && b.why.contains(word));

    let barrel = find(&layout, "barrel");
    let mut l = layout.clone();
    l.items[barrel].at.as_mut().unwrap().pos.y += 0.5;
    let bs = breaches(&l, None, &HashSet::new());
    assert!(has(&bs, &Ref::Town(barrel), "svever"), "{bs:?}");

    // Et våpenstativ i gjerdet rundt Torgils' plass (runde 7).
    let rack = find(&layout, "weapon_rack");
    let fence = layout.items.iter().position(|it| it.model.as_deref() == Some("rail_fence")).unwrap();
    let mut l = layout.clone();
    l.items[rack].at.as_mut().unwrap().pos = l.items[fence].at.unwrap().pos;
    let bs = breaches(&l, None, &HashSet::new());
    assert!(has(&bs, &Ref::Town(rack), "inni rail_fence"), "{bs:?}");

    let stall = find(&layout, "market_stall");
    let mut l = layout.clone();
    let p = l.items[stall].at.unwrap().pos;
    l.items[barrel].at.as_mut().unwrap().pos = p;
    let bs = breaches(&l, None, &HashSet::new());
    assert!(has(&bs, &Ref::Town(barrel), "står inni market_stall"), "{bs:?}");

    let (home, house) = (find(&layout, "home"), find(&layout, "house_b"));
    let mut l = layout.clone();
    let p = l.items[home].at.unwrap().pos;
    l.items[house].at.as_mut().unwrap().pos = p + glam::Vec3::new(3.0, 0.0, 0.0);
    let bs = breaches(&l, None, &HashSet::new());
    assert!(has(&bs, &Ref::Town(house), "for tett på home"), "{bs:?}");

    let mut l = layout.clone();
    l.items[house].at.as_mut().unwrap().pos = glam::Vec3::ZERO;
    let bs = breaches(&l, None, &HashSet::new());
    assert!(has(&bs, &Ref::Town(house), "ute på torget"), "{bs:?}");

    let touched: HashSet<Ref> = [Ref::Town(barrel)].into();
    let bs = breaches(&layout, Some(&HashSet::new()), &touched);
    assert!(has(&bs, &Ref::Town(barrel), "ikke godkjent"), "{bs:?}");
    assert!(!bs.iter().any(|b| b.r != Ref::Town(barrel)), "bare det endrede sjekkes for godkjenning: {bs:?}");
}

fn editor() -> crate::domain::edit::Editor {
    let (src, layout, contents) = sources();
    crate::domain::edit::Editor::new(src, layout, contents, None)
}

/// Flytt, drei, kopier, slett, linje og ny ting, og så angre alt: dataene
/// er nøyaktig som før. Gjør om alt og angre igjen: det samme.
#[test]
fn redigeringen_flytter_og_angrer() {
    use crate::domain::edit::LineKind;
    use crate::domain::world::layout::Ref;
    use glam::{Affine3A, Vec3};
    let mut e = editor();
    let (l0, c0) = (e.layout.clone(), e.contents.clone());
    assert!(!e.dirty() && e.clean(), "{:?}", e.breaches());
    let barrel = l0.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let chair = Ref::Inside { house: "tavern".into(), i: 0 };

    e.selection = vec![Ref::Town(barrel), chair.clone()];
    e.translate(Vec3::new(0.5, 0.0, -0.25));
    let moved = e.pose_of(&Ref::Town(barrel)).unwrap().translation;
    assert!((Vec3::from(moved) - l0.items[barrel].at.unwrap().pos - Vec3::new(0.5, 0.0, -0.25)).length() < 1e-4);
    e.rotate(0.3);
    e.duplicate();
    assert_eq!(e.selection.len(), 2);
    assert_eq!(e.layout.items.len(), l0.items.len() + 1);
    assert_eq!(e.contents["tavern"].len(), c0["tavern"].len() + 1);
    e.delete();
    e.selection = vec![Ref::Town(3)];
    e.delete();
    let wall = e.line(&[Vec3::new(30.0, 0.0, 30.0), Vec3::new(34.0, 0.0, 30.0), Vec3::new(34.0, 0.0, 34.0)], &LineKind::Wall { height: 3.0 });
    assert_eq!(wall.len(), 2);
    e.line(&[Vec3::new(40.0, 0.0, 30.0), Vec3::new(46.0, 0.0, 30.0)], &LineKind::Row { model: "wattle_fence".into(), spacing: 0.0 });
    e.place("barrel", Affine3A::from_translation(Vec3::new(5.0, 0.0, 6.0)));
    assert!(e.dirty());
    let (l1, c1) = (e.layout.clone(), e.contents.clone());
    while e.undo() {}
    assert!(e.layout == l0 && e.contents == c0, "angre ga ikke tilbake det samme");
    assert!(!e.dirty());
    while e.redo() {}
    assert!(e.layout == l1 && e.contents == c1, "gjøre om ga ikke det samme");
    while e.undo() {}
    assert!(e.layout == l0 && e.contents == c0);
}

/// En ny ting inne i et hus lagres i husets rom og følger huset når det
/// flyttes; den usynlige veggen får kollider og kan få ny høyde.
#[test]
fn ny_ting_i_huset_folger_huset() {
    use crate::domain::edit::LineKind;
    use crate::domain::world::layout::Ref;
    use glam::{Affine3A, Vec3};
    let mut e = editor();
    let tavern = e.layout.items.iter().position(|it| it.model.as_deref() == Some("tavern")).unwrap();
    let t = e.pose_of(&Ref::Town(tavern)).unwrap();
    let inside = t.transform_point3(Vec3::new(0.0, 0.1, 0.0));
    let r = e.place("stool", Affine3A::from_translation(inside)).unwrap();
    assert!(matches!(&r, Ref::Inside { house, .. } if house == "tavern"), "{r:?}");
    let before = Vec3::from(e.pose_of(&r).unwrap().translation);
    e.selection = vec![Ref::Town(tavern)];
    e.translate(Vec3::new(0.0, 0.0, 1.0));
    let after = Vec3::from(e.pose_of(&r).unwrap().translation);
    assert!((after - before - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-4, "{before} -> {after}");

    let wall = e.line(&[Vec3::new(30.0, 0.0, 30.0), Vec3::new(34.0, 0.0, 30.0)], &LineKind::Wall { height: 3.0 });
    let n = e.scene().solid_origins.iter().filter(|o| o.as_ref() == Some(&wall[0])).count();
    assert_eq!(n, 1, "veggen har én kollider");
    e.set_wall_height(4.5);
    let s = e.selected();
    assert!((s[0].half.y - 2.25).abs() < 1e-4, "{:?}", s[0]);
}

/// Klikk utenfra på Vertshuset velger huset; klikk inne i Vertshuset mot et
/// møbel velger møbelet, ikke huset rundt deg.
#[test]
fn klikk_velger_riktig_ting() {
    use crate::domain::world::layout::Ref;
    use glam::Vec3;
    let mut e = editor();
    let tavern = e.layout.items.iter().position(|it| it.model.as_deref() == Some("tavern")).unwrap();
    let t = e.pose_of(&Ref::Town(tavern)).unwrap();
    let c = Vec3::from(t.translation) + Vec3::Y * 2.0;
    let eye = c + Vec3::new(30.0, 2.0, 0.0);
    assert_eq!(e.pick(eye, (c - eye).normalize(), false), Some(Ref::Town(tavern)));
    assert_eq!(e.selection, vec![Ref::Town(tavern)]);

    let table = (0..e.contents["tavern"].len()).map(|i| Ref::Inside { house: "tavern".into(), i }).find(|r| e.label(r).starts_with("table")).unwrap();
    let target = Vec3::from(e.pose_of(&table).unwrap().translation) + Vec3::Y * 0.5;
    let eye = target + Vec3::new(0.0, 1.2, 0.0);
    let hit = e.pick(eye, (target - eye).normalize(), false);
    assert_eq!(hit, Some(table.clone()));
    // Shift-klikk legger til og tar bort.
    e.pick(eye, (target - eye).normalize(), true);
    assert!(e.selection.is_empty());
}

/// Hvor lang tid én endring tar (byen bygges på nytt og reglene sjekkes).
#[test]
#[ignore]
fn tid_per_endring() {
    use crate::domain::world::layout::Ref;
    let mut e = editor();
    e.selection = vec![Ref::Town(100)];
    let t = std::time::Instant::now();
    for _ in 0..5 {
        e.translate(glam::Vec3::X * 0.1);
    }
    eprintln!("én endring: {:.0} ms", t.elapsed().as_secs_f64() * 1000.0 / 5.0);
}

/// Spillet satt sammen for redigering (alle modellene, med plass til å flytte).
fn edit_world() -> &'static crate::application::Prepared {
    static P: std::sync::OnceLock<crate::application::Prepared> = std::sync::OnceLock::new();
    P.get_or_init(|| crate::application::prepare_for_edit(&crate::adapters::gltf::Embedded, 7))
}

fn edit_session(physics: &mut crate::adapters::rapier::RapierPhysics) -> crate::application::Session {
    let p = edit_world();
    let mut s = crate::application::Session::new(&p.world, physics, &crate::application::rigs(&p.models));
    s.edit_data = p.edit.clone();
    s
}

/// I redigeringen flyr du gjennom veggene: rett gjennom Vertshuset, der
/// du ellers blir stående ved veggen.
#[test]
fn flyging_gaar_gjennom_vegger() {
    use crate::ports::driving::{EditCommand, Input};
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let tavern = edit_world().edit.as_ref().unwrap().layout.items.iter().find(|it| it.model.as_deref() == Some("tavern")).unwrap().at.unwrap().pos;
    // Øst for Vertshuset, mot vest (yaw 90).
    let start = glam::Vec3::new(tavern.x + 14.0, crate::domain::world::terrain::height(tavern.x + 14.0, tavern.z) + 1.7, tavern.z);
    let run = |s: &mut crate::application::Session, physics: &mut crate::adapters::rapier::RapierPhysics| {
        s.place(start, std::f32::consts::FRAC_PI_2, 0.0);
        let mut input = Input { forward: true, ..Default::default() };
        for _ in 0..240 {
            s.update(&mut input, 1.0 / 60.0, physics);
        }
        s.eye().x
    };
    let walked = run(&mut s, &mut physics);
    assert!(walked > tavern.x, "uten redigering stopper veggen deg ({walked} > {})", tavern.x);
    assert!(s.edit_command(EditCommand::Enter));
    let flown = run(&mut s, &mut physics);
    assert!(flown < tavern.x - 5.0, "i redigeringen flyr du gjennom ({flown} < {})", tavern.x - 5.0);
}

/// Det som er flyttet, skjules der det sto og tegnes der det står nå.
#[test]
fn bildet_viser_det_flyttede() {
    use crate::domain::world::layout::Ref;
    use crate::ports::driving::{EditCommand, Event};
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    assert!(s.edit_command(EditCommand::Enter));
    let frame = s.frame(1.6);
    assert!(frame.hidden.is_empty(), "ingenting er endret ennå: {:?}", frame.hidden);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let old = d.layout.items[barrel].at.unwrap().affine();
    let new = glam::Affine3A::from_translation(glam::Vec3::new(1.0, 0.0, 0.5)) * old;
    assert!(s.edit_command(EditCommand::SetPose { r: Ref::Town(barrel), pose: new }));
    let frame = s.frame(1.6);
    assert_eq!(frame.hidden.iter().filter(|(m, _)| *m == "barrel").count(), 1, "{:?}", frame.hidden);
    assert!(frame.instances.iter().any(|i| i.model == "barrel" && (glam::Vec3::from(i.transform.translation) - glam::Vec3::from(new.translation)).length() < 1e-3));
    // Nettlaget får det valgte og reglene.
    let mut input = crate::ports::driving::Input::default();
    let events = s.update(&mut input, 0.016, &mut physics);
    assert!(events.iter().any(|e| matches!(e, Event::Edit(Some(v)) if v.dirty && v.can_undo)), "{events:?}");
    assert!(s.edit_command(EditCommand::Undo));
    assert!(s.frame(1.6).hidden.is_empty());
}

/// P: spillet bygges på nytt fra dataene i redigeringen, og tønna står der
/// den ble flyttet, med kollider.
#[test]
fn ombygging_etter_redigering() {
    use crate::domain::edit::rules;
    use crate::domain::world::layout::Ref;
    let mut e = editor();
    let barrel = e.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    e.selection = vec![Ref::Town(barrel)];
    e.translate(glam::Vec3::new(0.0, 0.0, 1.5));
    assert!(e.clean(), "{:?}", e.breaches());
    let new = e.pose_of(&Ref::Town(barrel)).unwrap();
    let (layout, contents) = e.data();
    let mut data = world_data();
    data.layout = Some(layout.clone());
    data.contents = contents.clone().into_iter().collect();
    let p = crate::application::prepare_with(&crate::adapters::gltf::Embedded, 7, data, false);
    let pl = p.world.placements.iter().find(|pl| pl.model == "barrel").unwrap();
    let t = pl.transforms.iter().find(|t| (glam::Vec3::from(t.translation) - glam::Vec3::from(new.translation)).length() < 1e-3).expect("tønna står der den ble flyttet");
    let info = p.infos.iter().find(|i| i.name == "barrel").unwrap();
    assert!(rules::lacks_collider(&p.world.colliders, info.min, info.max, t).is_none(), "tønna har kollider der den står nå");
}

/// Dra i x-pila: det valgte flyttes bare langs x, og hele draget angres som
/// én handling. Dra i ringen: det dreies.
#[test]
fn handtakene_drar() {
    use crate::domain::world::layout::Ref;
    use crate::ports::driving::EditCommand;
    use glam::Vec3;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let at = d.layout.items[barrel].at.unwrap().pos;
    s.place(at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, -0.3);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let centre = s.to_screen(at + Vec3::Y * 0.4).unwrap();
    assert!(s.edit_command(EditCommand::Pick { add: false, at: Some(centre) }));
    assert_eq!(s.editor().unwrap().selection, vec![Ref::Town(barrel)]);
    let (p, len) = s.handles().unwrap();
    // Ta tak midt på z-pila (den går på tvers av bildet her) og dra den.
    let grab = s.to_screen(p + Vec3::Z * len * 0.6).unwrap();
    let tip = s.to_screen(p + Vec3::Z * len).unwrap();
    assert!(s.edit_command(EditCommand::Press { at: grab, add: false, grab: Default::default() }));
    let step = (tip.0 - grab.0, tip.1 - grab.1);
    for k in 1..=4 {
        assert!(s.edit_command(EditCommand::Drag { to: (grab.0 + step.0 * k as f32, grab.1 + step.1 * k as f32) }));
    }
    assert!(s.edit_command(EditCommand::Release));
    let now = s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap().translation;
    let moved = Vec3::from(now) - at;
    assert!(moved.z > 0.5 && moved.x.abs() < 1e-3 && moved.y.abs() < 1e-3, "flyttet {moved}");
    assert!(s.edit_command(EditCommand::Undo));
    assert!(!s.editor().unwrap().can_undo(), "draget var én handling");
    assert!((Vec3::from(s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap().translation) - at).length() < 1e-4);
    // Ringen: ta tak på ringen og dra et stykke rundt.
    s.edit_command(EditCommand::Pick { add: false, at: Some(centre) });
    let (p, len) = s.handles().unwrap();
    let on_ring = |s: &crate::application::Session, a: f32| s.to_screen(p + glam::Quat::from_rotation_y(a) * Vec3::X * len * 0.8).unwrap();
    let (a, b) = (on_ring(&s, 1.2), on_ring(&s, 1.6));
    assert!(s.edit_command(EditCommand::Press { at: a, add: false, grab: Default::default() }));
    s.edit_command(EditCommand::Drag { to: b });
    s.edit_command(EditCommand::Release);
    let yaw = crate::domain::world::layout::Pose::of(s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap()).yaw() - d.layout.items[barrel].at.unwrap().yaw();
    assert!((yaw - 0.4).abs() < 0.05, "dreid {yaw}");
}

/// Katalogen i redigeringen: husene for seg, gjenstandene for seg, og
/// delene spillet beveger og innsidene av husene er ikke med.
#[test]
fn katalogen_har_kategorier() {
    use crate::adapters::data::{buildings, category};
    let b = buildings();
    let cat = |name: &str| {
        let path = crate::adapters::gltf::model_files().find(|(n, _)| *n == name).map(|(_, p)| p).unwrap_or_else(|| panic!("{name} finnes ikke"));
        category(name, path, &b)
    };
    for h in ["stave_church", "tavern", "house_a", "crypt", "home"] {
        assert_eq!(cat(h), Some("Hus"), "{h}");
    }
    for g in ["market_cross", "well", "barrel", "cart", "signpost"] {
        assert_eq!(cat(g), Some("Gjenstander"), "{g}");
    }
    assert_eq!(cat("stone_fence"), Some("Gjerder"));
    assert_eq!(cat("town_wall"), Some("Mur og porter"));
    assert_eq!(cat("barrel_rack"), Some("Møbler"));
    for none in ["tavern_interior", "chest_lid", "well_rope", "gallery_button", "portal_ring"] {
        assert_eq!(cat(none), None, "{none}");
    }
}

/// Kirkas boks dekker steingjerdet ved siden av: første klikk gir kirka,
/// klikk på samme sted igjen (eller Tab) gir gjerdet, og så kirka igjen.
#[test]
fn klikk_igjen_gir_det_neste_under_pekeren() {
    use crate::domain::world::layout::Ref;
    use glam::Vec3;
    let mut e = editor();
    let church = e.layout.items.iter().position(|it| it.model.as_deref() == Some("stave_church")).unwrap();
    let (t, c, h) = e.box_of(&Ref::Town(church)).unwrap();
    let inv = t.inverse();
    // Et steingjerde med foten inni kirkas boks.
    let fence = (0..e.layout.items.len())
        .filter(|i| e.layout.items[*i].model.as_deref() == Some("stone_fence"))
        .find(|i| {
            let p = inv.transform_point3(Vec3::from(e.pose_of(&Ref::Town(*i)).unwrap().translation) + Vec3::Y * 0.3) - c;
            p.abs().cmple(h).all()
        })
        .expect("et steingjerde inni kirkas boks");
    let top = Vec3::from(e.pose_of(&Ref::Town(fence)).unwrap().translation) + Vec3::Y * 40.0;
    let down = -Vec3::Y;
    assert_eq!(e.pick(top, down, false), Some(Ref::Town(church)));
    assert_eq!(e.pick(top, down, false), Some(Ref::Town(fence)), "andre klikk: gjerdet under");
    assert!(e.under().len() >= 2);
    let third = e.cycle();
    assert!(third.is_some() && third != Some(Ref::Town(fence)), "Tab går videre");
}

/// Lagringen: uten endringer er filene redigeringen lagrer nøyaktig de i
/// repoet; etter en flytting leses de tilbake til det som ble redigert.
#[test]
fn lagringen_gir_filene() {
    use crate::adapters::data::{content_files, crypt_content_files, layout_file, layout_files};
    use crate::domain::world::layout::{Item, Layout, Ref};
    let mut e = editor();
    let (layout, contents) = e.data();
    for (path, text) in layout_files(layout, contents, e.crypt_data()) {
        let repo = if path == "assets/layout/town.json" {
            layout_file().unwrap()
        } else if let Some(n) = path.strip_prefix("assets/layout/crypt_") {
            let n: u32 = n.trim_end_matches(".json").parse().unwrap();
            crypt_content_files().iter().find(|(k, _)| *k == n).map(|(_, t)| *t).unwrap()
        } else {
            let house = path.trim_start_matches("assets/layout/inside/").trim_end_matches(".json");
            content_files().iter().find(|(h, _)| *h == house).map(|(_, t)| *t).unwrap()
        };
        assert_eq!(text, repo, "{path} er ikke lik fila i repoet");
    }
    e.selection = vec![Ref::Town(10), Ref::Inside { house: "tavern".into(), i: 2 }];
    e.translate(glam::Vec3::new(0.25, 0.0, -0.5));
    let (layout, contents) = e.data();
    for (path, text) in layout_files(layout, contents, e.crypt_data()) {
        if path == "assets/layout/town.json" {
            let back: Layout = serde_json::from_str(&text).unwrap();
            let (a, b) = (back.items[10].at.unwrap().pos, layout.items[10].at.unwrap().pos);
            assert!((a - b).length() < 1e-4, "{a} {b}");
        } else if path.ends_with("/tavern.json") {
            let back: Vec<Item> = serde_json::from_str(&text).unwrap();
            assert!((back[2].at.unwrap().pos - contents["tavern"][2].at.unwrap().pos).length() < 1e-4);
        }
    }
}

/// Krypten i redigeringen: et merke i nivå 1 kan velges, flyttes langs
/// gulvet og lagres i `crypt_01.json`; løftet det fra gulvet, brytes regelen,
/// og en ny kiste satt ut nede blir et merke i nivået.
#[test]
fn krypten_redigeres_og_lagres() {
    use crate::adapters::data::layout_files;
    use crate::domain::world::crypt::level::CryptContents;
    use crate::domain::world::layout::Ref;
    use glam::{Affine3A, Vec3};
    let mut e = editor();
    assert!(e.clean(), "krypten slik den står bryter reglene: {:?}", e.breaches());
    let chest = e.crypt[&1].marks.iter().position(|m| m.kind == "chest").expect("en kiste i nivå 1");
    let r = Ref::Crypt { level: 1, i: chest };
    let before = e.pose_of(&r).unwrap();
    assert!(before.translation.y < -10.0, "kista står nede i krypten");
    // Et klikk ovenfra treffer kista.
    let top = Vec3::from(before.translation) + Vec3::Y * 1.5;
    assert_eq!(e.pick(top, -Vec3::Y, false), Some(r.clone()));
    // Løftet en meter: brudd. Angret: grønt.
    e.translate(Vec3::Y);
    assert!(e.breaches().iter().any(|b| b.r == r), "kista svever uten brudd");
    e.undo();
    assert!(e.clean());
    // Lagret: crypt_01.json har kista der den står.
    let (layout, contents) = e.data();
    let files = layout_files(layout, contents, e.crypt_data());
    let (_, text) = files.iter().find(|(p, _)| p == "assets/layout/crypt_01.json").unwrap();
    let back: CryptContents = serde_json::from_str(text).unwrap();
    assert_eq!(back.marks[chest].kind, "chest");
    // En ny kiste ved siden av blir et merke i nivå 1.
    let at = Affine3A::from_translation(Vec3::from(before.translation) + Vec3::new(0.0, 0.0, 0.0));
    let n = e.crypt[&1].marks.len();
    assert_eq!(e.place(crate::domain::world::crypt::CHEST, at), Some(Ref::Crypt { level: 1, i: n }));
    assert_eq!(e.crypt[&1].marks.len(), n + 1);
    // Det byggesettet ikke har merker for, kan ikke settes ut nede.
    assert_eq!(e.place("stave_church", at), None);
}

/// Nede i krypten: P redigerer nivået du står i. Et klikk på kista velger
/// den, piltasten flytter den, bildet viser den der den står nå, og P
/// (spillet bygget på nytt fra redigeringen) setter den ut der, med kista
/// du kan åpne.
#[test]
fn krypten_redigeres_der_du_star() {
    use crate::domain::world::layout::Ref;
    use crate::ports::driving::EditCommand;
    use glam::Vec3;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let e = crate::domain::edit::Editor::new(d.sources.clone(), d.layout.clone(), d.contents.clone(), None);
    let chest = e.crypt[&1].marks.iter().position(|m| m.kind == "chest").unwrap();
    let r = Ref::Crypt { level: 1, i: chest };
    let at = Vec3::from(e.pose_of(&r).unwrap().translation);
    // Stå nede i krypten foran kista og se ned på den.
    s.place(at + Vec3::new(0.0, 1.7, 2.0), 0.0, -0.6);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let centre = s.to_screen(at + Vec3::Y * 0.3).unwrap();
    assert!(s.edit_command(EditCommand::Pick { add: false, at: Some(centre) }));
    assert_eq!(s.editor().unwrap().selection, vec![r.clone()]);
    assert!(s.edit_command(EditCommand::Move(Vec3::new(0.5, 0.0, 0.0))));
    let moved = Vec3::from(s.editor().unwrap().pose_of(&r).unwrap().translation);
    assert!((moved - at - Vec3::new(0.5, 0.0, 0.0)).length() < 1e-3);
    // I redigeringen flyr du fritt i nivået, også under bakken i byen.
    let (eye, _, _) = s.view_pose();
    assert!(eye.y < crate::domain::world::crypt::TOP);
    let frame = s.frame(1.6);
    assert!(frame.hidden.iter().any(|(m, _)| *m == "chest"), "{:?}", frame.hidden);
    assert!(frame.instances.iter().any(|i| i.model == "chest" && (Vec3::from(i.transform.translation) - moved).length() < 1e-3));
    // P: spillet bygges fra redigeringen, og kista står og åpnes der.
    let editor = s.editor().unwrap();
    let mut data = world_data();
    for (n, f) in data.levels.iter_mut() {
        f.marks = editor.crypt_data()[n].marks.clone();
    }
    let p = crate::application::prepare_with(&crate::adapters::gltf::Embedded, 7, data, false);
    let pl = p.world.placements.iter().find(|pl| pl.model == "chest").unwrap();
    assert!(pl.transforms.iter().any(|t| (Vec3::from(t.translation) - moved).length() < 1e-3), "kista står der den ble flyttet");
    assert!(
        p.world.targets.iter().any(|t| matches!(t.what, crate::domain::world::Use::Chest(_)) && (Vec3::from(t.pose.translation).with_y(0.0) - moved.with_y(0.0)).length() < 1.0),
        "kista kan åpnes der"
    );
}

/// Musa uten håndtakene (eieren 12. oktober): trykk på tønna og dra, så
/// følger den pekeren langs bakken; med Alt heises den, med Ctrl dreies den.
/// Et klikk uten å dra flytter ingenting, og hvert drag angres for seg.
#[test]
fn musa_drar_heiser_og_dreier() {
    use crate::domain::world::layout::{Pose, Ref};
    use crate::ports::driving::{EditCommand, Grab};
    use glam::Vec3;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let at = d.layout.items[barrel].at.unwrap().pos;
    s.place(at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, -0.3);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let pose = |s: &crate::application::Session| s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap();
    let centre = s.to_screen(at + Vec3::Y * 0.4).unwrap();
    let drag = |s: &mut crate::application::Session, grab: Grab, by: (f32, f32)| {
        assert!(s.edit_command(EditCommand::Press { at: centre, add: false, grab }));
        for k in 1..=5 {
            s.edit_command(EditCommand::Drag { to: (centre.0 + by.0 * k as f32 / 5.0, centre.1 + by.1 * k as f32 / 5.0) });
        }
        s.edit_command(EditCommand::Release);
    };
    // Et klikk velger og flytter ingenting.
    drag(&mut s, Grab::Move, (0.0, 0.0));
    assert_eq!(s.editor().unwrap().selection, vec![Ref::Town(barrel)]);
    assert!(!s.editor().unwrap().can_undo(), "klikket flyttet noe");
    // Dra mot høyre i bildet (du ser mot øst): tønna går sørover langs bakken.
    drag(&mut s, Grab::Move, (0.1, 0.0));
    let moved = Vec3::from(pose(&s).translation) - at;
    // Satt ned igjen på bakken der den står nå (millimeter i den planerte byen).
    assert!(moved.z > 0.5 && moved.y.abs() < 0.02, "flyttet {moved}");
    let under = s.to_screen(Vec3::from(pose(&s).translation) + Vec3::Y * 0.4).unwrap();
    assert!((under.0 - centre.0 - 0.1).abs() < 0.02, "tønna fulgte pekeren: {under:?}");
    assert!(s.edit_command(EditCommand::Undo));
    // Alt: opp når musa går opp.
    drag(&mut s, Grab::Lift, (0.0, -0.05));
    let lifted = Vec3::from(pose(&s).translation) - at;
    assert!(lifted.y > 0.2 && lifted.x.abs() < 1e-3 && lifted.z.abs() < 1e-3, "heist {lifted}");
    assert!(s.edit_command(EditCommand::Undo));
    // Ctrl: en kvart bildebredde mot venstre er en kvart runde.
    drag(&mut s, Grab::Turn, (-0.25, 0.0));
    let yaw = Pose::of(pose(&s)).yaw() - d.layout.items[barrel].at.unwrap().yaw();
    assert!((yaw - std::f32::consts::FRAC_PI_2).abs() < 0.05, "dreid {yaw}");
    assert!((Vec3::from(pose(&s).translation) - at).length() < 1e-3, "dreid om seg selv");
    assert!(s.edit_command(EditCommand::Undo));
    assert!(!s.editor().unwrap().can_undo());
}

/// Flere valgt (et gjerde i biter) dreies samlet om midten, som én ting:
/// avstandene mellom bitene står, og midten står der den sto.
#[test]
fn flere_valgt_dreies_samlet() {
    use crate::domain::world::layout::Ref;
    use glam::Vec3;
    let mut e = editor();
    let fences: Vec<usize> = (0..e.layout.items.len()).filter(|i| e.layout.items[*i].model.as_deref() == Some("rail_fence")).take(3).collect();
    let at = |e: &crate::domain::edit::Editor, i: usize| Vec3::from(e.pose_of(&Ref::Town(i)).unwrap().translation);
    let before: Vec<Vec3> = fences.iter().map(|i| at(&e, *i)).collect();
    let mid = before.iter().sum::<Vec3>() / before.len() as f32;
    e.selection = fences.iter().map(|i| Ref::Town(*i)).collect();
    e.rotate(std::f32::consts::FRAC_PI_2);
    for (k, i) in fences.iter().enumerate() {
        let want = mid + glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2) * (before[k] - mid);
        assert!((at(&e, *i) - want).length() < 1e-3, "bit {k}: {} skulle stå {want}", at(&e, *i));
    }
}

/// Vinda, tauet og bøtta følger brønnen du flytter i redigeringen (eieren i
/// runde 8: «stangen og bøtta fulgte ikke med»).
#[test]
fn bevegelige_deler_folger_med_i_redigeringen() {
    use crate::domain::world::layout::Ref;
    use crate::ports::driving::EditCommand;
    use glam::Vec3;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let well = d.layout.items.iter().position(|it| it.model.as_deref() == Some("well")).unwrap();
    let at = d.layout.items[well].at.unwrap().pos;
    s.place(at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, -0.3);
    assert!(s.edit_command(EditCommand::Enter));
    let parts = |s: &crate::application::Session| -> Vec<Vec3> {
        s.frame(1.6).instances.iter().filter(|i| crate::domain::world::town::WELL_PARTS.contains(&i.model)).map(|i| Vec3::from(i.transform.translation)).filter(|p| p.distance(at) < 6.0).collect()
    };
    let before = parts(&s);
    assert_eq!(before.len(), 3, "vinda, tauet og bøtta ved brønnen");
    let c = s.to_screen(at + Vec3::Y * 0.5).unwrap();
    assert!(s.edit_command(EditCommand::Pick { add: false, at: Some(c) }));
    assert_eq!(s.editor().unwrap().selection, vec![Ref::Town(well)]);
    assert!(s.edit_command(EditCommand::Move(Vec3::new(2.0, 0.0, 0.0))));
    let after: Vec<Vec3> =
        s.frame(1.6).instances.iter().filter(|i| crate::domain::world::town::WELL_PARTS.contains(&i.model)).map(|i| Vec3::from(i.transform.translation)).filter(|p| p.distance(at) < 8.0).collect();
    for p in &before {
        assert!(after.iter().any(|q| (*q - (*p + Vec3::X * 2.0)).length() < 1e-3), "{p} fulgte ikke med: {after:?}");
    }
}

/// Velge i dybden: lista over det som lå under pekeren kan klikkes i,
/// Shift+Tab går den andre veien, og Alt+klikk gir det bakerste.
#[test]
fn dybden_velges_fra_lista_og_bakfra() {
    use crate::domain::world::layout::Ref;
    use glam::Vec3;
    let mut e = editor();
    let church = e.layout.items.iter().position(|it| it.model.as_deref() == Some("stave_church")).unwrap();
    let (t, c, h) = e.box_of(&Ref::Town(church)).unwrap();
    let inv = t.inverse();
    let fence = (0..e.layout.items.len())
        .filter(|i| e.layout.items[*i].model.as_deref() == Some("stone_fence"))
        .find(|i| {
            let p = inv.transform_point3(Vec3::from(e.pose_of(&Ref::Town(*i)).unwrap().translation) + Vec3::Y * 0.3) - c;
            p.abs().cmple(h).all()
        })
        .expect("et steingjerde inni kirkas boks");
    let top = Vec3::from(e.pose_of(&Ref::Town(fence)).unwrap().translation) + Vec3::Y * 40.0;
    assert_eq!(e.pick(top, -Vec3::Y, false), Some(Ref::Town(church)));
    let under = e.under().to_vec();
    let k = under.iter().position(|r| *r == Ref::Town(fence)).expect("gjerdet står i lista");
    assert_eq!(e.select_under(k, false), Some(Ref::Town(fence)));
    assert_eq!(e.selection, vec![Ref::Town(fence)]);
    // Shift i lista legger til.
    e.select_under(0, true);
    assert_eq!(e.selection.len(), 2);
    // Shift+Tab fra det første går rundt til det bakerste.
    e.select_under(0, false);
    assert_eq!(e.cycle_back(), under.last().cloned());
    // Alt+klikk: det bakerste med en gang.
    assert_eq!(e.pick_deepest(top, -Vec3::Y), under.last().cloned());
}

/// Egenskapene: x og dreiningen skrives inn, fargen settes, merkene i
/// krypten får `opens` og lysene rekkevidden, gale verdier gir en feil, og
/// alt står i historikken og kan angres ved å klikke på et steg.
#[test]
fn egenskapene_settes_og_angres_fra_historikken() {
    use crate::domain::world::layout::{Pose, Ref};
    let mut e = editor();
    let barrel = e.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let start = e.layout.items[barrel].clone();
    e.selection = vec![Ref::Town(barrel)];
    let props = e.props();
    let x: f32 = props.iter().find(|p| p.key == "x").unwrap().value.parse().unwrap();
    assert!(props.iter().any(|p| p.key == "tint"), "{props:?}");
    e.set_prop("x", &format!("{}", x + 2.0)).unwrap();
    let moved = e.pose_of(&Ref::Town(barrel)).unwrap().translation.x;
    assert!((moved - (x + 2.0)).abs() < 0.01, "x {moved}");
    e.set_prop("yaw", "90").unwrap();
    let yaw = Pose::of(e.pose_of(&Ref::Town(barrel)).unwrap()).yaw().to_degrees();
    assert!((yaw - 90.0).abs() < 0.5, "dreining {yaw}");
    e.set_prop("tint", "#ff8000").unwrap();
    assert_eq!(e.layout.items[barrel].tint.map(crate::domain::edit::props::hex).as_deref(), Some("#ff8000"));
    assert!(e.set_prop("x", "mye").is_err());
    assert!(e.set_prop("tint", "rød").is_err());
    let (steps, at) = e.history();
    assert_eq!(at, 3);
    assert_eq!(steps, vec!["Flyttet barrel", "Flyttet barrel", "Endret barrel"]);
    // Klikk på det første steget: bare flyttingen står.
    assert!(e.undo_to(1));
    assert_eq!(e.layout.items[barrel].tint, start.tint);
    assert_eq!(e.history().0.len(), 3, "stegene etter står til å gjøres om");
    assert!(e.undo_to(0));
    assert_eq!(e.layout.items[barrel], start);
    assert!(e.undo_to(3));
    assert!(e.layout.items[barrel].tint.is_some());

    // Spaken i nivå 1 åpner to porter; lyset når lenger.
    let lever = e.crypt[&1].marks.iter().position(|m| m.kind == "lever").unwrap();
    e.selection = vec![Ref::Crypt { level: 1, i: lever }];
    assert!(e.props().iter().any(|p| p.key == "opens"));
    e.set_prop("opens", "gitter_1, gitter_2").unwrap();
    assert_eq!(e.crypt[&1].marks[lever].opens, vec!["gitter_1".to_string(), "gitter_2".to_string()]);
    e.selection = vec![Ref::CryptLight { level: 1, i: 0 }];
    e.set_prop("range", "14").unwrap();
    e.set_prop("color", "#ffffff").unwrap();
    e.set_prop("flicker", "nei").unwrap();
    let l = &e.crypt[&1].lights[0];
    assert_eq!((l.range, l.color, l.flicker), (14.0, [1.0, 1.0, 1.0], false));
    let mob = e.crypt[&1].marks.iter().position(|m| m.kind == "mob").unwrap();
    e.selection = vec![Ref::Crypt { level: 1, i: mob }];
    e.set_prop("patrol", "0,0,11; 4,0,33").unwrap();
    assert_eq!(e.crypt[&1].marks[mob].patrol, vec![[0.0, 0.0, 11.0], [4.0, 0.0, 33.0]]);
    assert!(e.props().iter().any(|p| p.key == "label" && p.label == "slag"));
}

/// Å sette ut raskt: fantomet følger pekeren og sier fra om reglene, R
/// dreier det, festet setter det på rutenettet og vinkelen, og slippet
/// setter det ned på bakken. Et drag med festet på havner også på
/// rutenettet, og et drag langs bakken setter tingen ned igjen.
#[test]
fn fantomet_festet_og_slippet() {
    use crate::domain::edit::ghost::{snap, ANGLE, GRID};
    use crate::domain::world::layout::{Pose, Ref};
    use crate::ports::driving::{EditCommand, Event, Grab};
    use glam::Vec3;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let at = d.layout.items[barrel].at.unwrap().pos;
    s.place(at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, -0.3);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let view = |s: &mut crate::application::Session, physics: &mut crate::adapters::rapier::RapierPhysics| {
        let mut input = crate::ports::driving::Input::default();
        s.update(&mut input, 0.016, physics).into_iter().rev().find_map(|e| match e {
            Event::Edit(Some(v)) => Some(v),
            _ => None,
        })
    };
    // Festet på og dreid, så et sted der fantomet er grønt.
    s.edit_command(EditCommand::GhostTurn(std::f32::consts::FRAC_PI_2));
    s.edit_command(EditCommand::Snap(true));
    let mut spot = None;
    let mut red = 0;
    for k in 0..40 {
        let p = at + Vec3::new((k % 5) as f32 * 1.5 - 3.0, 0.0, (k / 5) as f32 * 1.5 - 5.0);
        let Some(sp) = s.to_screen(p) else { continue };
        assert!(s.edit_command(EditCommand::Ghost(Some(("barrel".into(), sp)))));
        let v = view(&mut s, &mut physics).expect("utsikten");
        let g = v.ghost.expect("fantomet står");
        if g.is_empty() {
            spot = Some(sp);
            break;
        }
        red += 1;
    }
    let spot = spot.expect("et grønt sted nær tønna");
    // Der den gamle tønna står, er fantomet rødt (pekeren på tønna setter
    // det oppå den, som er lov).
    let old = d.layout.items[barrel].at.unwrap().affine();
    let bad = s.editor().unwrap().ghost_breaches("barrel", &old);
    assert!(bad.iter().any(|b| b.contains("inni")), "{bad:?} (prøvde {red} røde steder først)");
    s.edit_command(EditCommand::Ghost(Some(("barrel".into(), spot))));
    let frame = s.frame(1.6);
    assert!(frame.instances.iter().any(|i| i.model == "barrel" && (Vec3::from(i.transform.translation) - at).length() > 0.5), "fantomet tegnes");
    assert!(s.edit_command(EditCommand::Place { model: "barrel".into(), at: Some(spot) }));
    let e = s.editor().unwrap();
    // Den nye tønna er valget (i byen eller i huset den står i).
    let new: Ref = e.selection[0].clone();
    assert_eq!(e.label(&new), "barrel");
    let p = Pose::of(e.pose_of(&new).unwrap());
    assert!((p.pos.x - snap(p.pos.x, GRID)).abs() < 1e-3 && (p.pos.z - snap(p.pos.z, GRID)).abs() < 1e-3, "på rutenettet: {}", p.pos);
    assert!((p.yaw() - snap(p.yaw(), ANGLE)).abs() < 1e-3, "på vinkelen: {}", p.yaw().to_degrees());
    assert!(e.clean(), "satt ned uten brudd: {:?}", e.breaches());
    assert!(s.frame(1.6).instances.iter().filter(|i| i.model == "barrel").count() >= 1);

    // Dra den nye tønna med festet på: den havner på rutenettet.
    let centre = s.to_screen(p.pos + Vec3::Y * 0.4).unwrap();
    assert!(s.edit_command(EditCommand::Press { at: centre, add: false, grab: Grab::Move }));
    for k in 1..=6 {
        s.edit_command(EditCommand::Drag { to: (centre.0 + 0.013 * k as f32, centre.1) });
    }
    s.edit_command(EditCommand::Release);
    let q = s.editor().unwrap().pose_of(&new).unwrap();
    let q = Pose::of(q);
    assert!((q.pos.z - snap(q.pos.z, GRID)).abs() < 1e-3 && (q.pos - p.pos).length() > 0.4, "draget festet: {} fra {}", q.pos, p.pos);
    assert!(s.editor().unwrap().clean(), "satt ned etter draget: {:?}", s.editor().unwrap().breaches());
    s.edit_command(EditCommand::Ghost(None));
    assert!(view(&mut s, &mut physics).unwrap().ghost.is_none());
}

/// Boksvalg: en ramme rundt tønna på skjermen velger den.
#[test]
fn rammen_velger_det_innenfor() {
    use crate::domain::world::layout::Ref;
    use crate::ports::driving::EditCommand;
    use glam::Vec3;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let at = d.layout.items[barrel].at.unwrap().pos;
    s.place(at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, -0.3);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let c = s.to_screen(at + Vec3::Y * 0.4).unwrap();
    assert!(s.edit_command(EditCommand::SelectBox { a: (c.0 - 0.03, c.1 - 0.05), b: (c.0 + 0.03, c.1 + 0.05), add: false }));
    assert!(s.editor().unwrap().selection.contains(&Ref::Town(barrel)), "{:?}", s.editor().unwrap().selection);
}
