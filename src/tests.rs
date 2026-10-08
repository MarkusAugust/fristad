//! Testene som setter sammen hele spillet med de ekte adapterne (de
//! innebygde glTF-modellene og Rapier), uten GPU. `cargo test` før push.

use crate::adapters::gltf::Embedded;
use crate::adapters::rapier::RapierPhysics;
use crate::application::{prepare, rigs, Prepared, Session};
use crate::domain::world::{Shape, Use};
use crate::ports::driving::{Action, Event, Input, SaveData};
use crate::ports::Physics;
use glam::{Affine3A, Quat, Vec3};
use std::sync::OnceLock;

/// Verden bygges én gang og deles av testene (det tar noen sekunder).
fn world() -> &'static Prepared {
    static P: OnceLock<Prepared> = OnceLock::new();
    P.get_or_init(|| prepare(&Embedded, 7))
}

fn session(physics: &mut RapierPhysics) -> Session {
    let p = world();
    Session::new(&p.world, physics, &rigs(&p.models))
}

/// Rutinen mot svevende modeller: ingen instans får sveve over eller stå
/// nede i bakken (eller gulvet den står på).
#[test]
fn modellene_star_pa_bakken() {
    let p = world();
    assert!(p.world.audit.is_empty(), "\n{}", p.world.audit.join("\n"));
}

/// Modellregelen: alt som står i verden er en ekte modell fra
/// `tools/models/` med teksturer: bakte (albedo) eller de håndlagde
/// teksturlagene (`layer_`-materialer). Glass og lys er egne slag. Bare
/// ruskbitene, som lages i kjernen, er unntatt.
#[test]
fn alt_er_ekte_modeller_med_teksturer() {
    use crate::domain::geometry::kind;
    let p = world();
    let procedural = ["debris"];
    let mut problems = vec![];
    for m in &p.models {
        if procedural.contains(&m.name.as_str()) {
            continue;
        }
        let baked = m.mesh.vertices.iter().filter(|v| v.mat[0] == kind::MODEL).count();
        if baked > 0 && m.albedo.is_none() {
            problems.push(format!("{} har {baked} hjørner uten bakt albedo-tekstur", m.name));
        }
        let plain = m.mesh.vertices.iter().filter(|v| v.mat[0] == kind::PLAIN).count();
        if plain > 0 {
            problems.push(format!("{} har {plain} hjørner med flat farge", m.name));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Ingen skal kunne gå gjennom noe: hver modellinstans som er høy nok til å
/// gå inn i (over 60 cm) og bred nok til å merke (over 30 cm) må ha en
/// kollider rundt foten. Bladverk og gress er unntatt.
#[test]
fn alt_stort_har_kollider() {
    let p = world();
    let tol = 0.35;
    let covers = |shape: &Shape, q: Vec3| -> bool {
        match shape {
            Shape::Cuboid { center, size, yaw } => {
                let d = Quat::from_rotation_y(-*yaw) * (q - *center);
                d.x.abs() <= size.x * 0.5 + tol && d.y.abs() <= size.y * 0.5 + tol && d.z.abs() <= size.z * 0.5 + tol
            }
            Shape::Cylinder { center, radius, half_height } => {
                let d = q - *center;
                (d.x * d.x + d.z * d.z).sqrt() <= radius + tol && d.y.abs() <= half_height + tol
            }
            Shape::Capsule { a, b, radius } => {
                let ab = *b - *a;
                let t = ((q - *a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
                (q - (*a + ab * t)).length() <= radius + tol
            }
            Shape::Heightfield { .. } | Shape::Hull { .. } => false,
        }
    };
    let skip = |name: &str| name.ends_with("_leaves") || name.ends_with("_needles") || name.starts_with("grass_") || name.starts_with("flowers_") || name == "door_lamp" || name == "gorvek_torch" || name == "crypt_stair";
    let mut problems = vec![];
    for pl in &p.world.placements {
        if skip(pl.model) {
            continue;
        }
        let Some(model) = p.models.iter().find(|m| m.name == pl.model) else { continue };
        let (lo, hi) = model.mesh.bounds();
        for t in &pl.transforms {
            let mut wlo = Vec3::splat(f32::MAX);
            let mut whi = Vec3::splat(f32::MIN);
            for k in 0..8 {
                let c = Vec3::new(if k & 1 == 0 { lo.x } else { hi.x }, if k & 2 == 0 { lo.y } else { hi.y }, if k & 4 == 0 { lo.z } else { hi.z });
                let w = t.transform_point3(c);
                wlo = wlo.min(w);
                whi = whi.max(w);
            }
            let size = whi - wlo;
            if size.y < 0.6 || size.x.max(size.z) < 0.3 {
                continue;
            }
            // Ved foten, eller høyere opp på midtlinja for det som er åpent
            // nede (portrommet i porthuset).
            let o = Vec3::from(t.translation);
            let q = Vec3::new(o.x, wlo.y + 0.5, o.z);
            let steps = ((whi.y - wlo.y - 1.0) / 0.5).max(0.0) as usize;
            if !(0..=steps).any(|k| p.world.colliders.iter().any(|c| covers(c, q + Vec3::Y * (k as f32 * 0.5)))) {
                problems.push(format!("{} ved ({:.1}, {:.1}, {:.1}) har ingen kollider", pl.model, q.x, q.y, q.z));
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Og fysikken må faktisk stoppe deg: kapselen går mot en trestamme og mot
/// kanten av verden, og skal bli stående innenfor begge.
#[test]
fn kapselen_stopper_mot_trær_og_kanten() {
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.5f32, 0.3f32);
    let mut walk = |from: Vec3, to: Vec3, mark: Vec3| -> (Vec3, f32) {
        let mut pos = from;
        let dir = (to - from).normalize();
        let mut nearest = f32::MAX;
        for _ in 0..((to - from).length() / 0.1) as usize {
            let (moved, _) = physics.move_capsule(pos, half, radius, dir * 0.1 - Vec3::Y * 0.05, 0.016);
            pos += moved;
            nearest = nearest.min((pos - mark).with_y(0.0).length());
        }
        (pos, nearest)
    };
    let oak = p.world.placements.iter().find(|pl| pl.model == "oak1_trunk" && pl.transforms.len() > 3).expect("ingen eik");
    for t in oak.transforms.iter().take(4) {
        let base = Vec3::from(t.translation);
        let y = half + radius + 0.1;
        let (_, nearest) = walk(base + Vec3::new(2.0, y, 0.0), base + Vec3::new(-2.0, y, 0.0), base);
        assert!(nearest > 0.4, "gikk gjennom eika ved ({:.1}, {:.1}): kom {:.2} m fra stammen", base.x, base.z, nearest);
    }
    let start = Vec3::new(185.0, crate::domain::world::terrain::height(185.0, 0.0) + half + radius + 0.1, 0.0);
    let (end, _) = walk(start, Vec3::new(215.0, start.y, 0.0), Vec3::ZERO);
    assert!(end.x < 199.0, "gikk ut av verden: endte ved x = {:.1}", end.x);
}

/// Alt du kan bruke må kunne siktes på: en stråle fra 60 cm foran hvert mål
/// (i målets +Z) mot midten skal treffe målet først, ikke en vegg.
#[test]
fn alt_du_kan_bruke_kan_siktes_pa() {
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let mut ids = std::collections::HashMap::new();
    for (i, t) in p.world.targets.iter().enumerate() {
        ids.insert(physics.add_movable(t.half, t.pose * Affine3A::from_translation(t.center)), i);
    }
    physics.move_capsule(Vec3::new(0.0, 50.0, 0.0), 0.5, 0.3, Vec3::ZERO, 0.016);
    let mut problems = vec![];
    for (i, t) in p.world.targets.iter().enumerate() {
        let center = t.pose.transform_point3(t.center);
        // Innbyggerne siktes på fra alle kanter; resten forfra.
        let facing = match t.what {
            Use::Talk(_) => Vec3::X,
            _ => t.pose.transform_vector3(Vec3::Z).normalize_or_zero(),
        };
        let depth = if matches!(t.what, Use::Talk(_)) { t.half.x } else { t.half.z };
        let from = center + facing * (depth + 0.6);
        let hit = physics.cast_ray(from, -facing, 2.0);
        if hit.as_ref().and_then(|h| ids.get(&h.collider)) != Some(&i) {
            problems.push(format!("{:?} ved ({:.1}, {:.1}, {:.1}) kan ikke siktes på", t.what, center.x, center.y, center.z));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Krypta tar deg ned i kjelleren og døra opp igjen; fiendene kommer,
/// biter, og kan slås i hjel.
#[test]
fn kjelleren_kan_nas_og_fiendene_biter() {
    use crate::domain::mobs::Kind;
    use crate::domain::world::dungeon::DEPTH;
    let p = world();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y > 0.0, "starter oppe");
    // Alle slagene er med, og akkurat én boss; tabellen er elite nederst.
    let roster = game.roster();
    for k in Kind::ALL {
        assert!(roster.iter().any(|(r, _)| *r == k), "ingen {k:?} i kjelleren");
    }
    assert_eq!(roster.iter().filter(|(k, _)| k.boss()).count(), 1, "akkurat én boss");
    assert!(roster.iter().any(|(k, e)| *k == Kind::Table && *e), "tabellen er elite nederst");
    // De dype etasjene: du skal kunne stå der uten å havne oppe igjen.
    let deep = &p.world.dungeon.floors[p.world.dungeon.floors.len() - 1];
    let there = deep.center(deep.start.0, deep.start.1);
    game.place(there + Vec3::Y * 1.7, 0.0, 0.0);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!((game.eye().y - there.y - 1.7).abs() < 1.0, "sto ikke igjen nederst: øyet er på y = {:.1}, gulvet på {:.1}", game.eye().y, there.y);
    // Tilbake opp, og ned gjennom krypta.
    game.respawn();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let events = game.use_first(|u| *u == Use::Descend);
    assert!(events.iter().any(|e| matches!(e, Event::Fade)), "nedgangen skal tone til svart");
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y < DEPTH + 3.0, "døra i krypta skal ta deg ned: øyet er på y = {:.1}", game.eye().y);
    assert!(events.iter().any(|e| matches!(e, Event::Hud { cellar: true, .. })), "HUD-en skal melde at du er nede");
    // Fiendene jager: etter noen sekunder biter en, og livet synker.
    let mut bitten = false;
    for _ in 0..(30 * 40) {
        for e in game.update(&mut input, 1.0 / 30.0, &mut physics) {
            if let Event::Hud { hp, .. } = e {
                if hp < 100 {
                    bitten = true;
                }
            }
        }
        if bitten {
            break;
        }
    }
    assert!(bitten, "ingen fiende rakk fram og bet på 40 sekunder");
    // Snu deg mot den nærmeste og slå til den dør.
    let mut killed = false;
    'outer: for _ in 0..(30 * 20) {
        let feet = game.eye() - Vec3::Y * 0.8;
        if let Some(m) = game.nearest_mob(feet) {
            let d = m + Vec3::Y * 0.8 - game.eye();
            let flat = d.with_y(0.0).length();
            game.set_look(-d.x.atan2(-d.z), d.y.atan2(flat));
        }
        input.actions.push(Action::Use);
        for e in game.update(&mut input, 1.0 / 30.0, &mut physics) {
            if let Event::Toast(t) = e {
                if t.contains("falt fra hverandre") {
                    killed = true;
                    break 'outer;
                }
            }
        }
    }
    assert!(killed, "fikk ikke slått i hjel en fiende på 20 sekunder");
    // Og opp igjen gjennom døra i startrommet.
    game.use_first(|u| *u == Use::Ascend);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y > 0.0, "døra opp skal ta deg til krypta: øyet er på y = {:.1}", game.eye().y);
}

/// Smiinga: med komponentene i lageret godtas skjemaet for komponentsverdet
/// og våpenet byttes; uten dem avvises det. Kjøp koster tokens, og
/// lagringen tar med utstyret.
#[test]
fn skjemaet_smir_utstyr() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    assert!(!game.craft("component_sword"), "uten komponenter skal skjemaet avvises");
    game.give("button", 2);
    game.give("divider", 1);
    assert!(game.craft("component_sword"), "med komponentene skal sverdet smis");
    assert_eq!(game.gear().weapon.id, "component_sword");
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(events.iter().any(|e| matches!(e, Event::Inventory(v) if v.stash.iter().all(|(c, n)| !(c == "button" && *n > 0)))), "knappene skal være brukt opp");
    assert!(!game.buy("potion"), "uten tokens kan du ikke kjøpe");
    let saved = game.save();
    assert_eq!(saved.weapon, "component_sword");
    let mut physics2 = RapierPhysics::new();
    let mut fresh = session(&mut physics2);
    fresh.load(SaveData { coins: 50, ..saved });
    assert_eq!(fresh.gear().weapon.id, "component_sword");
    assert!(fresh.buy("potion"), "med tokens går kjøpet");
}

/// Hodet skal aldri gå gjennom taket i kjelleren: over hver celle, også
/// toppen av trappene og de hevede rommene, er det minst plass til en
/// stående spiller (1,8 m) som hopper.
#[test]
fn hodet_gar_ikke_gjennom_taket() {
    use crate::domain::world::dungeon::{Cell, HEADROOM, N};
    let p = world();
    let mut problems = vec![];
    for (k, f) in p.world.dungeon.floors.iter().enumerate() {
        for iz in 0..N as i32 {
            for ix in 0..N as i32 {
                if f.cell(ix, iz) == Cell::Rock {
                    continue;
                }
                let room = f.ceil_at(ix, iz) - f.top_at(ix, iz);
                if room < HEADROOM - 0.01 {
                    problems.push(format!("etasje {k}, celle ({ix}, {iz}): {room:.2} m under taket"));
                }
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Husene man kan gå inn i: gå med Rapier fra gata, gjennom døråpningen og
/// inn i rommet (opp trinnene til gulvet), uten å bli stoppet av veggene.
#[test]
fn man_kan_ga_inn_i_husene() {
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.55f32, 0.35f32);
    // Modell, start ute og mål inne, i Blenders akser (x, y): døra mot −y.
    let houses = [("tavern", (-1.2, -7.0), (-1.2, -2.5)), ("forge", (1.6, -1.6), (1.6, 1.8)), ("scriptorium", (0.0, -6.0), (0.0, -1.5)), ("apothecary", (1.9, -7.0), (1.9, -1.5)), ("stave_church", (0.0, -7.6), (0.0, 0.0))];
    let mut problems = vec![];
    for (model, from, to) in houses {
        let Some(pl) = p.world.placements.iter().find(|pl| pl.model == model) else {
            problems.push(format!("{model} mangler"));
            continue;
        };
        let t = pl.transforms[0];
        let world = |(x, y): (f32, f32)| t.transform_point3(Vec3::new(x, 0.0, -y));
        let (a, b) = (world(from), world(to));
        let mut pos = a + Vec3::Y * (half + radius + 0.1);
        let dir = (b - a).with_y(0.0).normalize();
        for _ in 0..((b - a).length() / 0.05) as usize * 2 {
            if (b - pos).with_y(0.0).dot(dir) < 0.05 {
                break;
            }
            let (moved, _) = physics.move_capsule(pos, half, radius, dir * 0.05 - Vec3::Y * 0.03, 0.016);
            pos += moved;
        }
        let left = (pos - b).with_y(0.0).length();
        if left > 0.6 {
            problems.push(format!("{model}: stoppet {left:.1} m fra målet inne, ved ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn man_kan_ga_opp_trappene() {
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.55f32, 0.35f32);
    // Modell, foten av trappa og målet oppe (Blenders akser), gulvet nede
    // og høyden oppe over bakken.
    let stairs = [("tavern", (5.39, -3.6), (5.39, 1.6), 0.6, 3.0), ("apothecary", (3.1, -2.7), (3.1, 1.8), 0.7, 3.1), ("scriptorium", (-2.43, -2.75), (-2.43, 2.0), 0.36, 2.9)];
    let mut problems = vec![];
    for (model, from, to, floor, up) in stairs {
        let Some(pl) = p.world.placements.iter().find(|pl| pl.model == model) else {
            problems.push(format!("{model} mangler"));
            continue;
        };
        let t = pl.transforms[0];
        let world = |(x, y): (f32, f32)| t.transform_point3(Vec3::new(x, 0.0, -y));
        let (a, b) = (world(from), world(to));
        let base = t.transform_point3(Vec3::ZERO).y;
        let mut pos = a + Vec3::Y * (floor + half + radius + 0.05);
        let dir = (b - a).with_y(0.0).normalize();
        for _ in 0..500 {
            if (b - pos).with_y(0.0).dot(dir) < 0.05 {
                break;
            }
            let (moved, _) = physics.move_capsule(pos, half, radius, dir * 0.05 - Vec3::Y * 0.03, 0.016);
            pos += moved;
        }
        let left = (pos - b).with_y(0.0).length();
        let feet = pos.y - half - radius - base;
        if left > 0.6 || feet < up {
            problems.push(format!("{model}: stoppet {left:.1} m fra målet oppe, med føttene {feet:.2} m over bakken"));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Brønnene: bøtta slippes ned og sveives opp, og du drikker når den er oppe.
#[test]
fn bronnen_gir_vann() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    assert!(world().world.targets.iter().filter(|t| matches!(t.what, Use::Well(_))).count() >= 2, "for få brønner");
    let mut events = game.use_first(|u| matches!(u, Use::Well(_)));
    for _ in 0..(5.0 * 30.0) as usize {
        events.extend(game.update(&mut input, 1.0 / 30.0, &mut physics));
    }
    assert!(events.iter().any(|e| matches!(e, Event::Sound(s) if s == "well")), "ingen lyd fra brønnen");
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("drikker"))), "fikk ikke drikke");
}

/// Pipene ryker: husene, Smia og Vertshuset har røyk, og den står over
/// bakken der pipene er.
#[test]
fn pipene_ryker() {
    let smoke = &world().world.smoke;
    assert!(smoke.len() >= 20, "bare {} piper ryker", smoke.len());
    for (p, s) in smoke {
        let ground = crate::domain::world::terrain::height(p.x, p.z);
        assert!(p.y > ground + 5.0 && *s > 0.0, "røyken ved ({:.1}, {:.1}, {:.1}) står for lavt", p.x, p.y, p.z);
    }
}

/// Innbyggerne går rundt på gjøremålene sine, og stopper når du kommer nær.
#[test]
fn innbyggerne_gar_og_stopper() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    // Langt unna alle: utenfor sørporten.
    game.place_feet(Vec3::new(0.0, 0.5, 90.0));
    let start = game.npc_positions();
    for _ in 0..(20.0 * 30.0) as usize {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let moved = game.npc_positions().iter().zip(&start).filter(|(a, b)| a.distance(**b) > 0.5).count();
    assert!(moved >= 3, "bare {moved} innbyggere har gått noe sted på 20 s");
    // Stå ved den første: hen blir stående.
    let near = game.npc_positions()[0];
    game.place_feet(near + Vec3::new(2.0, 0.0, 0.0));
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let here = game.npc_positions()[0];
    for _ in 0..(5.0 * 30.0) as usize {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!(game.npc_positions()[0].distance(here) < 0.05, "innbyggeren gikk fra deg mens du sto ved siden av");
}

/// Nettleseren henter modellfilene selv (`Files`, stiene i `MODEL_FILES`):
/// byen skal bli den samme som med modellene bygd inn.
#[test]
fn nettleseren_far_de_samme_modellene() {
    use crate::adapters::gltf::{Files, MODEL_FILES};
    let files = MODEL_FILES.iter().map(|(_, p)| (p.to_string(), std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).expect(p))).collect();
    let web = prepare(&Files(files), 7);
    let native = world();
    let names = |p: &Prepared| p.models.iter().map(|m| (m.name.clone(), m.mesh.indices.len())).collect::<Vec<_>>();
    assert_eq!(names(&web), names(native));
    assert_eq!(web.world.placements.len(), native.world.placements.len());
}
