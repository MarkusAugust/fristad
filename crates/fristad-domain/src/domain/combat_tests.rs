//! Testene for nærkampen (`combat.rs`): det tunge hugget kjennes i kameraet,
//! synes i bildet, treffer hardere og dytter mer.

use super::*;
use crate::domain::world::crypt::Crypt;

/// Eieren 13. oktober: «ingen merkbar forskjell på vanlig slag og hardt
/// slag». Ladningen løfter kameraet, og når hugget slippes, kaster
/// kameraet seg ned forbi der det sto.
#[test]
fn det_tunge_hugget_kjennes_i_kameraet() {
    let crypt = Crypt::default();
    let mut mobs = Mobs::new(&crypt, &[]);
    let mut c = Combat::new(None);
    let step = |c: &mut Combat, mobs: &mut Mobs, secs: f32| {
        for _ in 0..(secs * 60.0) as usize {
            c.update(1.0 / 60.0, 1.0 / 60.0, Vec3::Y, -Vec3::Z, Vec3::ZERO, mobs, &crypt);
        }
    };
    c.press_attack();
    step(&mut c, &mut mobs, 0.1);
    let light = c.view_kick(0.0).1;
    step(&mut c, &mut mobs, 0.9);
    assert!(matches!(c.phase, Phase::Heavy { charging: true, .. }), "holdt knapp lader et tungt hugg");
    let charged = c.view_kick(0.0).1;
    assert!(charged > 0.03 && charged > light + 0.03, "ladningen løfter blikket: {charged:.3} mot {light:.3}");
    c.release_attack();
    let mut lowest = f32::MAX;
    for _ in 0..30 {
        step(&mut c, &mut mobs, 1.0 / 60.0);
        lowest = lowest.min(c.view_kick(0.0).1);
    }
    assert!(lowest < -0.025, "hugget kaster blikket ned: {lowest:.3}");
}

/// Eieren 18. oktober: «Ser ingen forskjell på tungt og raskt slag med
/// våpenet. Virker det egentlig?» Holdt knapp gir et tungt hugg som
/// treffer hardere og dytter fienden lenger enn et klikk, og ladningen
/// løfter sverdarmen inn i bildet.
#[test]
fn det_tunge_hugget_treffer_hardere_og_dytter_mer() {
    let crypt = Crypt::default();
    let hit = |hold: bool| {
        let mut mobs = Mobs::new(&crypt, &[]);
        mobs.spawn(crate::domain::mob_kind::Kind::Table, Vec3::new(0.0, 0.0, -1.6), false);
        let mut c = Combat::new(None);
        c.press_attack();
        if !hold {
            c.release_attack();
        }
        let (mut damage, mut push, mut lifted) = (0.0, 0.0f32, 0.0f32);
        for n in 0..120 {
            if hold && n == 50 {
                c.release_attack();
            }
            for e in c.update(1.0 / 60.0, 1.0 / 60.0, Vec3::Y * 1.6, -Vec3::Z, Vec3::ZERO, &mut mobs, &crypt) {
                if let CombatEvent::Hit { damage: d, .. } = e {
                    damage += d;
                    push = push.max(mobs.mobs[0].knock.length());
                }
            }
            lifted = lifted.max(c.lift);
        }
        (damage, push, lifted)
    };
    let (light, light_push, light_lift) = hit(false);
    let (heavy, heavy_push, heavy_lift) = hit(true);
    assert!(light > 0.0 && heavy >= light * 2.0, "det tunge hugget gjør for lite skade: {heavy:.0} mot {light:.0}");
    assert!(heavy_push >= light_push * 2.0, "det tunge hugget dytter for lite: {heavy_push:.2} mot {light_push:.2}");
    assert!(light_lift == 0.0 && heavy_lift > 0.6, "ladningen løfter ikke armen: {heavy_lift:.2}");
}
