//! Draget i redigeringen av byen: byen bygges om for hvert steg musa går,
//! og krypten under bare når den er endret (`edit::below`). Eieren 27.
//! november: å flytte noe i byen var ubrukelig tregt, for hvert steg bygde
//! byen og hele krypten og sjekket alle reglene.

use crate::application::Session;
use crate::edit_tests::{edit_session, edit_world};
use crate::ports::driving::EditCommand;
use glam::Vec3;

/// Trykk på tingen i `at` sett fra `eye`; gir hvor på skjermen.
fn press(s: &mut Session, eye: Vec3, yaw: f32, at: Vec3) -> (f32, f32) {
    s.place(eye, yaw, -0.3);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let p = s.to_screen(at).unwrap();
    s.edit_command(EditCommand::Press { at: p, add: false, grab: Default::default() });
    p
}

/// Dra det som er trykket på i `p`, `steps` steg bortover.
fn drag(s: &mut Session, p: (f32, f32), steps: usize) {
    for k in 1..=steps {
        assert!(s.edit_command(EditCommand::Drag { to: (p.0 + 0.02 * k as f32, p.1) }), "steg {k} flyttet ikke");
    }
}

/// Krypten bygges ikke om når noe dras i byen: den står urørt (de samme
/// kolliderne på samme sted i minnet) gjennom draget og slippet, mens tønna
/// står der den er dratt for hvert steg. Et merke som flyttes i krypten,
/// bygger den om.
#[test]
fn krypten_bygges_ikke_om_naar_noe_dras_i_byen() {
    use crate::domain::world::layout::Ref;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let old = d.layout.items[barrel].at.unwrap().affine();
    let at = Vec3::from(old.translation);
    let p = press(&mut s, at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, at + Vec3::Y * 0.4);
    let below = |s: &Session| s.editor().unwrap().scenes()[1].colliders.as_ptr();
    let crypt = below(&s);
    let stands = |s: &Session, t: glam::Affine3A| s.editor().unwrap().scene().placements.iter().filter(|p| p.model == "barrel").flat_map(|p| &p.transforms).any(|x| x.abs_diff_eq(t, 1e-4));
    drag(&mut s, p, 4);
    let now = s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap();
    assert!((Vec3::from(now.translation) - at).length() > 0.3, "tønna er dratt: {now:?}");
    assert!(stands(&s, now) && !stands(&s, old), "byen er bygget med tønna der den er dratt");
    assert!(s.editor().unwrap().dirty());
    s.edit_command(EditCommand::Release);
    assert_eq!(below(&s), crypt, "krypten er ikke bygget om");
    assert!(s.edit_command(EditCommand::Undo));
    assert!(stands(&s, old) && !s.editor().unwrap().dirty());
    assert_eq!(below(&s), crypt, "krypten er ikke bygget om");
    // Et merke i krypten flyttes: da bygges den, med merket der det står.
    let mut e = crate::domain::edit::Editor::new(d.sources.clone(), d.layout.clone(), d.contents.clone(), d.approved.clone());
    let poses = |e: &crate::domain::edit::Editor| e.scenes()[1].placements.iter().flat_map(|p| p.transforms.clone()).map(|t| t.translation).collect::<Vec<_>>();
    let before = poses(&e);
    let chest = d.sources.levels[0].1.marks.iter().position(|m| m.kind == "chest").expect("en kiste på første nivå");
    e.selection = vec![Ref::Crypt { level: d.sources.levels[0].0, i: chest }];
    e.translate(Vec3::X * 0.1);
    assert_ne!(poses(&e), before, "krypten er bygget om");
}

/// Vinda, tauet og bøtta følger brønnen for hvert steg den dras med musa,
/// ikke først når den slippes (eieren 12. oktober).
#[test]
fn de_bevegelige_delene_folger_mens_bronnen_dras() {
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let well = d.layout.items.iter().position(|it| it.model.as_deref() == Some("well")).unwrap();
    let at = d.layout.items[well].at.unwrap().pos;
    let parts = |s: &Session| -> Vec<Vec3> {
        let f = s.frame(1.6);
        f.instances.iter().filter(|i| crate::domain::world::town::WELL_PARTS.contains(&i.model)).map(|i| Vec3::from(i.transform.translation)).filter(|p| p.distance(at) < 8.0).collect()
    };
    let p = press(&mut s, at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, at + Vec3::Y * 0.5);
    let before = parts(&s);
    drag(&mut s, p, 4);
    let by = s.editor().unwrap().pose_of(&crate::domain::world::layout::Ref::Town(well)).unwrap().translation - glam::Vec3A::from(at);
    assert!(by.length() > 0.3, "brønnen er dratt: {by:?}");
    let during = parts(&s);
    for p in &before {
        assert!(during.iter().any(|q| (*q - (*p + Vec3::from(by))).length() < 1e-3), "{p} fulgte ikke med: {during:?}");
    }
}

/// Redigeringen bygger byen og krypten hver for seg (krypten bare når den
/// er endret). Det gir det samme som å bygge dem i ett, slik spillet gjør:
/// de samme instansene, kolliderne, lysene og målene, og reglene finner
/// det samme, også når noe svever i byen og i krypten.
#[test]
fn by_og_krypt_hver_for_seg_er_som_i_ett() {
    use crate::domain::edit::{rules, scene, Editor};
    use crate::domain::world::layout::Ref;
    use crate::domain::world::Builder;
    let d = edit_world().edit.as_ref().unwrap();
    let (mut src, mut layout) = (d.sources.clone(), d.layout.clone());
    let barrel = layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    layout.items[barrel].at.as_mut().unwrap().pos.y += 2.0;
    let mark = src.levels[0].1.marks.iter_mut().find(|m| m.kind == "chest").expect("en kiste på første nivå");
    mark.pos.as_mut().unwrap()[1] += 2.0;
    let e = Editor::new(src.clone(), layout.clone(), d.contents.clone(), d.approved.clone());
    let (whole, _) = scene(&src, &layout, &d.contents, e.crypt_data());
    let poses = |bs: &[&Builder]| {
        let mut v: Vec<_> = bs.iter().flat_map(|b| &b.placements).flat_map(|p| p.transforms.iter().map(|t| (p.model, t.to_cols_array().map(|v| (v * 1000.0).round() as i32)))).collect();
        v.sort();
        v
    };
    assert_eq!(poses(&e.scenes()), poses(&[&whole]));
    let count = |f: fn(&Builder) -> usize| e.scenes().iter().map(|b| f(b)).sum::<usize>();
    assert_eq!((count(|b| b.colliders.len()), count(|b| b.lights.len()), count(|b| b.targets.len())), (whole.colliders.len(), whole.lights.len(), whole.targets.len()));
    let want = rules::check(&whole, &layout, d.approved.as_ref(), &e.touched());
    assert!(want.iter().any(|b| b.r == Ref::Town(barrel)) && want.iter().any(|b| matches!(b.r, Ref::Crypt { .. })), "prøven har brudd i byen og i krypten: {want:?}");
    for b in &want {
        assert!(e.breaches().contains(b), "mangler: {b:?}");
    }
    // Utover dette finner redigeringen bare det som gjelder merkene i krypten for seg.
    for b in e.breaches() {
        assert!(want.contains(b) || matches!(b.r, Ref::Crypt { .. } | Ref::CryptLight { .. }), "for mye: {b:?}");
    }
}
