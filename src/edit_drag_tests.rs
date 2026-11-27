//! Draget i redigeringen av byen: det som dras, vises der det er uten at
//! byen bygges om for hvert steg musa går (`edit::preview`). Eieren 27.
//! november: å flytte noe i byen var ubrukelig tregt, for hvert steg bygde
//! byen og hele krypten og sjekket alle reglene.

use crate::application::Session;
use crate::edit_tests::{edit_session, edit_world};
use crate::ports::driving::EditCommand;
use glam::Vec3;

/// Det redigeringen viser på nytt sted og skjuler der det sto, i fast
/// rekkefølge (millimeter).
fn shown(s: &Session) -> (Vec<(&'static str, [i32; 12])>, Vec<(&'static str, usize)>) {
    let f = s.frame(1.6);
    let mut moved: Vec<_> = f.instances.iter().map(|i| (i.model, i.transform.to_cols_array().map(|v| (v * 1000.0).round() as i32))).collect();
    let mut hidden = f.hidden.clone();
    moved.sort();
    hidden.sort();
    (moved, hidden)
}

/// Trykk på tingen i `at` sett fra `eye` og dra den `steps` steg bortover.
fn drag(s: &mut Session, eye: Vec3, yaw: f32, at: Vec3, steps: usize) {
    s.place(eye, yaw, -0.3);
    s.frame(1.6);
    assert!(s.edit_command(EditCommand::Enter));
    let p = s.to_screen(at).unwrap();
    s.edit_command(EditCommand::Press { at: p, add: false, grab: Default::default() });
    for k in 1..=steps {
        assert!(s.edit_command(EditCommand::Drag { to: (p.0 + 0.02 * k as f32, p.1) }), "steg {k} flyttet ikke");
    }
}

/// Mens tønna dras, står byen som den ble bygget (ingen ombygging per
/// steg), og bildet viser tønna der dataene sier den er. Når den slippes,
/// bygges byen med tønna der den ble sluppet.
#[test]
fn draget_vises_uten_at_byen_bygges_om() {
    use crate::domain::world::layout::Ref;
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let barrel = d.layout.items.iter().position(|it| it.model.as_deref() == Some("barrel")).unwrap();
    let old = d.layout.items[barrel].at.unwrap().affine();
    let at = Vec3::from(old.translation);
    drag(&mut s, at + Vec3::new(-6.0, 2.5, 0.0), -std::f32::consts::FRAC_PI_2, at + Vec3::Y * 0.4, 4);
    let stands = |s: &Session, t: glam::Affine3A| s.editor().unwrap().scene().placements.iter().filter(|p| p.model == "barrel").flat_map(|p| &p.transforms).any(|x| x.abs_diff_eq(t, 1e-4));
    let now = s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap();
    assert!((Vec3::from(now.translation) - at).length() > 0.3, "tønna er dratt: {now:?}");
    assert!(stands(&s, old) && !stands(&s, now), "byen er ikke bygget om mens draget varer");
    let during = shown(&s);
    assert_eq!(during.1.iter().filter(|(m, _)| *m == "barrel").count(), 1, "tønna er skjult der den sto: {:?}", during.1);
    let near = |t: &[i32; 12]| (t[9] as f32 / 1000.0 - now.translation.x).abs() < 2e-3 && (t[11] as f32 / 1000.0 - now.translation.z).abs() < 2e-3;
    assert!(during.0.iter().any(|(m, t)| *m == "barrel" && near(t)), "tønna vises der den er: {:?}", during.0);
    assert!(s.edit_command(EditCommand::Release));
    // Tønna settes ned på bakken der den slippes, så høyden kan være en annen.
    let set = s.editor().unwrap().pose_of(&Ref::Town(barrel)).unwrap();
    assert!((set.translation.x - now.translation.x).abs() < 1e-4 && (set.translation.z - now.translation.z).abs() < 1e-4);
    assert!(stands(&s, set) && !stands(&s, old), "byen er bygget når draget slippes");
    assert_eq!(shown(&s).1, during.1, "det samme er skjult etter at byen ble bygget");
    assert!(s.edit_command(EditCommand::Undo));
    assert!(stands(&s, old) && shown(&s).1.is_empty(), "angre setter tønna tilbake");
}

/// Et hus tar med seg alt det har (innsiden, møblene, det som står inni)
/// mens det dras: bildet under draget er det samme som etter at byen er
/// bygget på nytt.
#[test]
fn huset_tar_med_seg_innholdet_mens_det_dras() {
    let mut physics = crate::adapters::rapier::RapierPhysics::new();
    let mut s = edit_session(&mut physics);
    let d = edit_world().edit.as_ref().unwrap();
    let tavern = d.layout.items.iter().find(|it| it.model.as_deref() == Some("tavern")).unwrap().at.unwrap().pos;
    drag(&mut s, tavern + Vec3::new(30.0, 8.0, 0.0), std::f32::consts::FRAC_PI_2, tavern + Vec3::Y * 2.0, 3);
    assert_eq!(s.editor().unwrap().selected().first().map(|x| x.label.as_str()), Some("tavern"));
    let during = shown(&s);
    assert!(during.0.len() > 5 && during.0.iter().any(|(m, _)| *m == "tavern"), "huset og innholdet vises på nytt sted: {} instanser", during.0.len());
    assert!(s.edit_command(EditCommand::Release));
    let after = shown(&s);
    assert_eq!(during.1, after.1, "det samme er skjult");
    assert_eq!(during.0, after.0, "huset og innholdet står likt under draget og etter ombyggingen");
}
