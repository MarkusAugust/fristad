//! Mekanismene som moduler (`docs/nivaaredigering.md` del 24): hver modul
//! med en mekanisme virker alene (koblingene inni modulen), og reglene over
//! koblingene (`compose::check::links`) fanger det som ikke henger sammen.
//!
//! Her står det som trenger skjelettene og økta. Reglene testes i kjernen
//! (`crates/fristad-domain/tests/regler/mechanism.rs`).

use crate::adapters::data::kit;
use crate::domain::world::crypt::compose::{self, modules};
use crate::domain::world::crypt::level::LevelFile;

fn showcase(id: &str) -> (LevelFile, compose::Report) {
    let k = kit();
    let doc = modules::showcase(&k, id).unwrap_or_else(|| panic!("modulen {id} finnes ikke"));
    compose::compile(&doc, &k)
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
        if id != "grav/stige" {
            continue;
        }

        // I stigen spiller fienden klatreklippet (`mob_motion.climb`) med
        // ansiktet mot stigen, opp og ned; uten klippet glir den i gangen.
        let all = crate::application::rigs(&crate::tests::world().models);
        let climbers: Vec<Kind> = Kind::ALL.into_iter().filter(|k| all.iter().any(|(n, r)| n == k.model() && r.clip("climb").is_some())).collect();
        assert!(climbers.contains(&Kind::SkeletonLoader), "Skeleton Loader har ikke klatreklippet: {climbers:?}");
        let face = Vec3::new(0.0, 0.0, -1.0); // stigen står mot nord i modulen (yaw 0, foten bak)
        for (k, want) in [(Kind::SkeletonLoader, "climb"), (Kind::Button, "walk")] {
            assert_eq!(climbers.contains(&k), want == "climb", "{k:?}");
            for (start, goal) in [(from, to), (to, from)] {
                let mut mobs = Mobs::new(&c, &all);
                let i = mobs.spawn_aware(k, start);
                let mut seen = std::collections::BTreeSet::new();
                let mut faced = true;
                for _ in 0..30 * 40 {
                    mobs.update(1.0 / 30.0, goal, true, &c);
                    let m = &mobs.mobs[i];
                    if m.pos.y > o.y + 1.5 && m.pos.y < o.y + 4.5 {
                        seen.insert(mobs.clip_name(i).unwrap_or("?").to_string());
                        let ahead = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
                        faced &= ahead.dot(face) > 0.9;
                    }
                    if mobs.mobs[i].pos.distance(goal) < 2.5 {
                        break;
                    }
                }
                let dir = if start == from { "opp" } else { "ned" };
                assert!(mobs.mobs[i].pos.distance(goal) < 2.5, "{k:?} kom ikke {dir} stigen");
                assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![want.to_string()], "{k:?} spiller feil klipp {dir} stigen");
                assert!(faced || want == "walk", "{k:?} vender ikke ansiktet mot stigen {dir}");
            }
        }
    }
}

/// Klatreklippet står på trinnene (`hops::climb_phase`): fasen regnes fra
/// høyden over stigens fot delt på trinnavstanden, ikke fra klokka, så
/// foten som holder står oppå et trinn opp og ned stigen, og fasen er den
/// samme hvor fienden enn begynner (også i en stige som er strukket litt).
#[test]
fn klatreklippet_star_pa_trinnene() {
    use crate::domain::mobs::{Kind, Mobs};
    use crate::domain::world::crypt::hops::{climb_phase, ladder_rung};
    use crate::domain::world::{crypt, Builder};
    use glam::{Vec2, Vec3};
    // Fasen: samme to trinn høyere, og venstre fot nederst på trinnet sitt
    // (fasen 0,06 i `mob_motion.climb`) når foten står oppå et trinn.
    for h in [3.0, 5.6, 6.0, 7.4] {
        let r = ladder_rung(h);
        assert!((0.3..0.37).contains(&r), "trinnavstanden {r} i en stige på {h} m");
        for k in 0..6 {
            let on = r * (k as f32 + 0.5) + 0.021;
            let want = if k % 2 == 0 { 0.06 } else { 0.56 };
            let ph = climb_phase(on, r);
            assert!((ph - want).abs() < 1e-4, "{h} m, trinn {k}: fasen {ph}, ikke {want}");
            assert!((climb_phase(on + 0.13 + 2.0 * r, r) - climb_phase(on + 0.13, r)).abs() < 1e-4);
        }
    }
    // I stigen i modulen: foten som holder, står på et trinn opp og ned.
    let (f, _) = showcase("grav/stige");
    let mut c = crypt::build(&mut Builder::new(&[]), Vec2::ZERO, vec![(1, f)]);
    c.open = vec![true; c.switches.len()];
    let o = c.levels[0].origin;
    let (below, above) = (Vec3::new(0.0, 0.0, 4.0) + o, Vec3::new(0.0, 6.0, -4.0) + o);
    let l = c.ladders[0].clone();
    let r = ladder_rung(l.top.y - l.pos.y);
    let all = crate::application::rigs(&crate::tests::world().models);
    for (start, goal) in [(below, above), (above, below)] {
        let mut mobs = Mobs::new(&c, &all);
        let i = mobs.spawn_aware(Kind::SkeletonLoader, start);
        // Ankelen over sålen når den står (hvileposen).
        let rest = ["foot.L", "foot.R"].map(|b| mobs.joint_world(i, b).unwrap().y - start.y).iter().copied().fold(f32::MAX, f32::min);
        let mut last: Option<[f32; 2]> = None;
        let (mut checked, mut worst, mut on) = (0, 0.0f32, 0.0f32);
        for _ in 0..30 * 40 {
            mobs.update(1.0 / 30.0, goal, true, &c);
            let m = &mobs.mobs[i];
            let climbing = mobs.clip_name(i) == Some("climb");
            on = if climbing { on + 1.0 / 30.0 } else { 0.0 };
            let feet = ["foot.L", "foot.R"].map(|b| mobs.joint_world(i, b).unwrap().y);
            if let Some(prev) = last.filter(|_| on > 0.3 && m.pos.y > l.pos.y + 0.6 && m.pos.y < l.top.y - 0.6) {
                for (y, p) in feet.iter().zip(prev) {
                    // Foten står stille mens fienden stiger: den holder.
                    if (y - p).abs() < 0.006 {
                        let sole = y - rest - l.pos.y - 0.021;
                        let off = (sole / r - 0.5 - (sole / r - 0.5).round()).abs() * r;
                        worst = worst.max(off);
                        checked += 1;
                    }
                }
            }
            last = Some(feet);
            if m.pos.distance(goal) < 2.5 {
                break;
            }
        }
        let dir = if start == below { "opp" } else { "ned" };
        assert!(checked > 20, "{dir}: føttene holdt aldri i stigen ({checked})");
        assert!(worst < 0.05, "{dir}: foten står {worst:.3} m fra trinnet");
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
