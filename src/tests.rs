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

/// Krypten tar deg ned i kjelleren og døra opp igjen; fiendene kommer,
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
    // Tilbake opp, og ned gjennom krypten.
    game.respawn();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let events = game.use_first(|u| *u == Use::Descend);
    assert!(events.iter().any(|e| matches!(e, Event::Fade)), "nedgangen skal tone til svart");
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y < DEPTH + 3.0, "døra i krypten skal ta deg ned: øyet er på y = {:.1}", game.eye().y);
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
    assert!(game.eye().y > 0.0, "døra opp skal ta deg til krypten: øyet er på y = {:.1}", game.eye().y);
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
    assert!(events.iter().any(|e| matches!(e, Event::Sound(s) if s.name == "well")), "ingen lyd fra brønnen");
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

/// Gjennomspillingen: en bot går ned gjennom krypten og gjennom alle
/// etasjene med ekte fysikk, slik en spiller må: åpner kistene, jakter
/// eliten til den har nøkkelen, låser opp portgitteret, går ned
/// vindeltrappa og slår Rammeverket nederst. Den slåss uten å blokke, så
/// den får fullt liv når livet er under 25 (og teller det som et dødsfall
/// en ekte spiller kunne fått). Står den fast, feiler testen med hvor.
/// Tar noen minutter: `cargo test --release gjennomspilling -- --ignored --nocapture`.
#[test]
#[ignore]
fn gjennomspilling() {
    use crate::domain::world::dungeon::{Dungeon, FLOOR_DROP};
    let p = world();
    let d: &Dungeon = &p.world.dungeon;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut bot = Bot { input: Input::default(), t: 0.0, near_deaths: 0, kills: 0, chests: 0, unopened: 0, log: vec![], swing: 0, last: Vec3::ZERO, stuck: 0.0, sidestep: 0.0, side: false, fighting: 0.0, ignore_until: 0.0, cell: None, blocked: 0.0 };
    bot.step(&mut game, &mut physics);
    let events = game.use_first(|u| *u == Use::Descend);
    assert!(events.iter().any(|e| matches!(e, Event::Fade)), "døra i krypten tok deg ikke ned");
    bot.step(&mut game, &mut physics);
    let spirals = p.world.placements.iter().find(|pl| pl.model == "dng_spiral").map(|pl| pl.transforms.clone()).unwrap_or_default();
    for (f, floor) in d.floors.iter().enumerate() {
        let t0 = bot.t;
        bot.note(format!("etasje {f} ({}): starter ved {:?}", crate::domain::world::dungeon::EPOCHS[f], floor.cell_of(game.eye())));
        // Kistene i etasjen.
        for (c, yaw, cf) in d.chests.iter() {
            if *cf != f {
                continue;
            }
            // Et sted ved kista det går an å stå, helst foran den.
            let _ = yaw;
            let feet = game.eye() - Vec3::Y * 1.6;
            let spots = [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z].map(|v| *c + v * 1.3);
            let at = spots.into_iter().filter(|q| d.walkable(*q, 0.3)).min_by(|a, b| a.distance(feet).total_cmp(&b.distance(feet))).unwrap_or(*c);
            // Veien kan gå bort fra kista først (ned trappa fra et galleri), så
            // boten får god tid før den gir opp.
            let reached = bot.walk_to_soft(&mut game, &mut physics, d, at, 0.5, 25.0);
            bot.aim(&mut game, *c + Vec3::Y * 0.4);
            bot.input.actions.push(Action::Use);
            let mut opened = false;
            for e in bot.step(&mut game, &mut physics) {
                if let Event::Loot(c) = e {
                    opened = true;
                    bot.chests += 1;
                    bot.note(format!("  kiste: {c:?}"));
                }
            }
            if !opened {
                bot.unopened += 1;
                bot.note(format!("  FANT IKKE FRAM til kista ved {:?} (kom {}fram)", floor.cell_of(*c), if reached { "" } else { "ikke " }));
            }
        }
        if f + 1 < d.floors.len() {
            // Eliten bærer nøkkelen.
            let keys = game.keys();
            let deadline = bot.t + 600.0;
            while game.keys() == keys {
                assert!(bot.t < deadline, "etasje {f}: fant ikke eliten med nøkkelen på ti minutter\n{}", bot.log.join("\n"));
                let feet = game.eye() - Vec3::Y * 1.6;
                let elite = game.living().into_iter().filter(|(m, _, e)| *e && (m.y - feet.y).abs() < 4.0).map(|(m, _, _)| m).min_by(|a, b| a.distance(feet).total_cmp(&b.distance(feet)));
                // Eliten slipper nøkkelen på gulvet: gå og hent den.
                let key = game.loose_keys().into_iter().filter(|k| (k.y - feet.y).abs() < 4.0).min_by(|a, b| a.distance(feet).total_cmp(&b.distance(feet)));
                let Some(e) = elite.or(key) else { panic!("etasje {f}: ingen elite og ingen nøkkel å finne\n{}", bot.log.join("\n")) };
                if !bot.fight(&mut game, &mut physics) {
                    bot.walk_toward(&mut game, &mut physics, d, e, &format!("eliten i etasje {f}"));
                }
            }
            bot.note(format!("  nøkkelen etter {:.0} s", bot.t - t0));
            // Portgitteret: still deg foran det på romsida og lås opp.
            let (g, gyaw) = d.gates[f];
            let exit = floor.exit.expect("etasjen har trapp");
            let fwd = Vec3::new(gyaw.sin(), 0.0, gyaw.cos());
            let stand = [g + fwd * 1.8, g - fwd * 1.8].into_iter().find(|q| {
                let c = floor.cell_of(*q);
                c != exit && floor.is_open(c.0, c.1)
            });
            let stand = stand.unwrap_or_else(|| panic!("etasje {f}: ingen plass foran portgitteret"));
            bot.walk_to(&mut game, &mut physics, d, stand, 0.6, &format!("portgitteret i etasje {f}"));
            bot.aim(&mut game, g + Vec3::Y * 1.3);
            bot.input.actions.push(Action::Use);
            bot.step(&mut game, &mut physics);
            assert_eq!(game.keys(), keys, "etasje {f}: portgitteret tok ikke nøkkelen (siktet du på det?)\n{}", bot.log.join("\n"));
            bot.idle(&mut game, &mut physics, 3.0);
            // Vindeltrappa: inn, rundt og ned, og ut i trappefoten under.
            let top = floor.center(exit.0, exit.1);
            let tf = *spirals.iter().min_by(|a, b| Vec3::from(a.translation).distance(top).total_cmp(&Vec3::from(b.translation).distance(top))).expect("vindeltrapp");
            let local = |x: f32, y: f32, z: f32| tf.transform_point3(Vec3::new(x, y, z));
            let mut path = vec![local(0.0, 0.0, -2.2), local(0.0, 0.0, -1.3)];
            let (a0, a1) = ((-50f32).to_radians(), (-130f32 + 1080.0).to_radians());
            let mut a = (-70f32).to_radians();
            while a < a1 {
                let y = -FLOOR_DROP * ((a - a0) / (a1 - a0)).clamp(0.0, 1.0);
                path.push(local(a.cos() * 1.2, y, a.sin() * 1.2));
                a += 12f32.to_radians();
            }
            // Helt ut av trappecella, ellers står boten på grensa til rommet.
            path.extend([local(0.0, -FLOOR_DROP, -1.3), local(0.0, -FLOOR_DROP, -3.0)]);
            for (k, w) in path.iter().enumerate() {
                bot.walk_straight(&mut game, &mut physics, *w, 0.45, &format!("vindeltrappa i etasje {f}, punkt {k} av {}", path.len()));
            }
            let below = &d.floors[f + 1];
            let now = game.eye() - Vec3::Y * 1.6;
            assert!((now.y - below.y).abs() < 2.5, "etasje {f}: kom ikke ned trappa (føttene på y = {:.1}, etasjen under på {:.1})\n{}", now.y, below.y, bot.log.join("\n"));
            bot.note(format!("  ned trappa etter {:.0} s", bot.t - t0));
        } else {
            // Nederst: Rammeverket.
            let boss = d.boss.expect("bossen");
            let deadline = bot.t + 900.0;
            loop {
                let alive = game.living().into_iter().any(|(_, k, _)| k.boss());
                if !alive {
                    break;
                }
                assert!(bot.t < deadline, "slo ikke Rammeverket på et kvarter\n{}", bot.log.join("\n"));
                if !bot.fight(&mut game, &mut physics) {
                    // Dit Rammeverket er nå, ikke der det sto.
                    let at = game.living().into_iter().find(|(_, k, _)| k.boss()).map(|(m, _, _)| m).unwrap_or(boss);
                    bot.walk_toward(&mut game, &mut physics, d, at, "Rammeverket");
                }
            }
            bot.note(format!("  Rammeverket falt etter {:.0} s", bot.t - t0));
        }
    }
    bot.note(format!("ferdig på {:.0} s spilltid: {} fiender drept, {} kister åpnet, {} ikke nådd, {} ganger nesten død", bot.t, bot.kills, bot.chests, bot.unopened, bot.near_deaths));
    eprintln!("{}", bot.log.join("\n"));
}

/// Boten i gjennomspillingen: styrer med tastene og musa som en spiller.
struct Bot {
    input: Input,
    t: f32,
    near_deaths: u32,
    kills: u32,
    chests: u32,
    unopened: u32,
    log: Vec<String>,
    swing: u32,
    last: Vec3,
    stuck: f32,
    sidestep: f32,
    side: bool,
    fighting: f32,
    ignore_until: f32,
    cell: Option<(i32, i32)>,
    blocked: f32,
}

impl Bot {
    const DT: f32 = 1.0 / 30.0;

    fn note(&mut self, s: String) {
        eprintln!("{s}");
        self.log.push(s);
    }

    fn step(&mut self, game: &mut Session, physics: &mut RapierPhysics) -> Vec<Event> {
        let events = game.update(&mut self.input, Self::DT, physics);
        self.t += Self::DT;
        if (self.t / 30.0).floor() != ((self.t - Self::DT) / 30.0).floor() {
            let feet = game.eye() - Vec3::Y * 1.6;
            eprintln!("  [{:.0} s] føttene ved ({:.1}, {:.1}, {:.1}), {} drept, {} kister, {} ganger nesten død", self.t, feet.x, feet.y, feet.z, self.kills, self.chests, self.near_deaths);
        }
        for e in &events {
            match e {
                Event::Hud { hp, cellar: true, .. } if *hp < 60 => {
                    self.near_deaths += 1;
                    game.heal();
                }
                Event::Toast(t) if t.contains("falt fra hverandre") => {
                    self.kills += 1;
                    self.fighting = 0.0;
                }
                Event::Toast(t) if t.contains("overkjørt") => panic!("boten døde likevel: {t}\n{}", self.log.join("\n")),
                _ => {}
            }
        }
        events
    }

    fn aim(&mut self, game: &mut Session, at: Vec3) {
        let d = at - game.eye();
        game.set_look(-d.x.atan2(-d.z), d.y.atan2(d.with_y(0.0).length()));
    }

    fn idle(&mut self, game: &mut Session, physics: &mut RapierPhysics, secs: f32) {
        self.input.key("KeyW", false);
        let end = self.t + secs;
        while self.t < end {
            self.step(game, physics);
        }
    }

    /// Slåss med den nærmeste fienden innen tre meter på samme etasje.
    /// Gir om det var noen å slåss med.
    fn fight(&mut self, game: &mut Session, physics: &mut RapierPhysics) -> bool {
        let feet = game.eye() - Vec3::Y * 1.6;
        // Bare fiender boten faktisk når: i samme celle eller en nabocelle
        // med fri vei (ikke bak en vegg).
        let d = &world().world.dungeon;
        let reach = |m: &Vec3| -> bool {
            let Some(f) = d.floor_of(feet) else { return false };
            let fl = &d.floors[f];
            let (a, b) = (fl.cell_of(feet), fl.cell_of(*m));
            a == b || ((a.0 - b.0).abs() + (a.1 - b.1).abs() == 1 && fl.passable(a, b))
        };
        let near = game.living().into_iter().filter(|(m, _, _)| (m.y - feet.y).abs() < 2.5 && m.with_y(0.0).distance(feet.with_y(0.0)) < 3.0 && reach(m)).min_by(|a, b| a.0.distance(feet).total_cmp(&b.0.distance(feet)));
        if self.t < self.ignore_until {
            return false;
        }
        let Some((m, kind, _)) = near else {
            self.fighting = 0.0;
            return false;
        };
        // Står fienden utenfor rekkevidden (den kommer seg ikke nærmere forbi
        // en sarkofag eller søyle), går boten inn til den. Blir det stående
        // slik, går den videre en stund, som en spiller ville gjort.
        self.fighting += Self::DT;
        if self.fighting > 20.0 {
            self.note(format!("    {:.0} s: ga opp {kind:?} ved ({:.1}, {:.1}, {:.1}) etter 20 s uten å felle den", self.t, m.x, m.y, m.z));
            self.fighting = 0.0;
            self.ignore_until = self.t + 6.0;
            return false;
        }
        self.input.key("KeyW", m.with_y(0.0).distance(feet.with_y(0.0)) > 1.6 * if kind.boss() { 1.5 } else { 1.0 });
        self.aim(game, m + Vec3::Y * 0.8 * if kind.boss() { 2.0 } else { 1.0 });
        self.swing += 1;
        self.input.actions.push(if self.swing % 2 == 0 { Action::Attack } else { Action::AttackRelease });
        self.step(game, physics);
        true
    }

    /// Ett steg mot et mål som kan flytte seg, langs veien på rutenettet.
    fn walk_toward(&mut self, game: &mut Session, physics: &mut RapierPhysics, d: &crate::domain::world::dungeon::Dungeon, to: Vec3, what: &str) {
        let feet = game.eye() - Vec3::Y * 1.6;
        let mut next = d.next_step(feet, to).unwrap_or_else(|| panic!("ingen vei til {what} fra {feet:?}\n{}", self.log.join("\n")));
        // Følg linja mellom cellemidtene (gangene er smale): er boten langt
        // ut til siden, går den først til midten av cella den står i.
        if let Some(f) = d.floor_of(feet) {
            let fl = &d.floors[f];
            let c = fl.cell_of(feet);
            let cur = fl.center(c.0, c.1);
            let seg = (next - cur).with_y(0.0);
            if seg.length() > 0.1 && fl.cell_of(next) != c {
                let off = feet.with_y(0.0) - cur.with_y(0.0);
                let side = (off - seg.normalize() * off.dot(seg.normalize())).length();
                // Står det en søyle midt i cella, går den rett på i stedet.
                if side > 0.5 && self.stuck < 1.5 && self.blocked < 2.0 {
                    next = cur;
                }
            }
        }
        if (self.t / 10.0).floor() != ((self.t - Self::DT) / 10.0).floor() {
            let f = d.floor_of(feet).unwrap_or(0);
            self.note(format!("    {:.0} s: står i {:?} ({:.1}, {:.1}, {:.1}), på vei til {what} i {:?}, neste {:?}", self.t, d.floors[f].cell_of(feet), feet.x, feet.y, feet.z, d.floors[f].cell_of(to), d.floors[f].cell_of(next)));
        }
        self.aim(game, next.with_y(game.eye().y));
        self.input.key("KeyW", true);
        // Står den stille (en sarkofag, en søyle), går den sidelengs en stund,
        // annenhver gang til venstre og høyre, som en spiller ville gjort.
        let moved = feet.distance(self.last);
        self.last = feet;
        self.stuck = if moved < 0.01 { self.stuck + Self::DT } else { 0.0 };
        // Hvor lenge boten har stått i samme celle uten å komme videre.
        let cell = d.floor_of(feet).map(|f| d.floors[f].cell_of(feet));
        if cell == self.cell {
            self.blocked += Self::DT;
        } else {
            self.cell = cell;
            self.blocked = 0.0;
        }
        if self.blocked > 8.0 {
            self.blocked = 0.0;
        }
        if self.stuck > 1.0 && self.sidestep <= 0.0 {
            self.sidestep = 0.8;
            self.side = !self.side;
        }
        let side = if self.side { "KeyA" } else { "KeyD" };
        self.input.key(side, self.sidestep > 0.0);
        self.sidestep -= Self::DT;
        self.step(game, physics);
        if self.sidestep <= 0.0 {
            self.input.key("KeyA", false);
            self.input.key("KeyD", false);
        }
    }

    /// Gå langs veien på rutenettet til du er innen `near` meter fra målet,
    /// og slåss med det som kommer underveis.
    fn walk_to(&mut self, game: &mut Session, physics: &mut RapierPhysics, d: &crate::domain::world::dungeon::Dungeon, to: Vec3, near: f32, what: &str) {
        let mut best = f32::MAX;
        let mut since = self.t;
        loop {
            let feet = game.eye() - Vec3::Y * 1.6;
            let dist = feet.with_y(0.0).distance(to.with_y(0.0));
            if dist < near {
                break;
            }
            if self.fight(game, physics) {
                since = self.t;
                continue;
            }
            if dist < best - 0.2 {
                best = dist;
                since = self.t;
            }
            if self.t - since > 8.0 {
                panic!("sto fast på vei til {what}: føttene ved {feet:?} (celle {:?}), {dist:.1} m igjen\n{}", d.floor_of(feet).map(|f| d.floors[f].cell_of(feet)), self.log.join("\n"));
            }
            self.walk_toward(game, physics, d, to, what);
        }
        self.input.key("KeyW", false);
    }

    /// Som `walk_to`, men gir `false` i stedet for å feile om boten står
    /// fast i `patience` sekunder.
    fn walk_to_soft(&mut self, game: &mut Session, physics: &mut RapierPhysics, d: &crate::domain::world::dungeon::Dungeon, to: Vec3, near: f32, patience: f32) -> bool {
        let mut best = f32::MAX;
        let mut since = self.t;
        loop {
            let feet = game.eye() - Vec3::Y * 1.6;
            let dist = feet.with_y(0.0).distance(to.with_y(0.0));
            if dist < near {
                self.input.key("KeyW", false);
                return true;
            }
            if self.fight(game, physics) {
                since = self.t;
                continue;
            }
            if dist < best - 0.2 {
                best = dist;
                since = self.t;
            }
            if self.t - since > patience || d.next_step(feet, to).is_none() {
                self.input.key("KeyW", false);
                return false;
            }
            self.walk_toward(game, physics, d, to, "kista");
        }
    }

    /// Gå rett mot et punkt (i trappa, der rutenettet ikke hjelper).
    fn walk_straight(&mut self, game: &mut Session, physics: &mut RapierPhysics, to: Vec3, near: f32, what: &str) {
        let start = self.t;
        loop {
            let feet = game.eye() - Vec3::Y * 1.6;
            if feet.with_y(0.0).distance(to.with_y(0.0)) < near {
                break;
            }
            assert!(self.t - start < 6.0, "sto fast i {what}: føttene ved {feet:?}, målet {to:?}\n{}", self.log.join("\n"));
            self.aim(game, to.with_y(game.eye().y));
            self.input.key("KeyW", true);
            self.step(game, physics);
        }
    }
}




/// Fiendene går rundt det som står i rommene, men ingen dør eller gang er
/// sperret av hindringene: langs hver dør og hvert gangforløp mellom to
/// celler er det plass til en kropp. (Inne i rommene kan noe stenge, som
/// apsisen rundt alteret i kapellet.)
#[test]
fn fiendene_gar_rundt_det_som_star() {
    let d = &world().world.dungeon;
    let (mut open, mut shut, mut centers, mut blocked) = (0, vec![], 0, 0);
    for (f, fl) in d.floors.iter().enumerate() {
        for iz in 0..40 {
            for ix in 0..40 {
                if !fl.is_open(ix, iz) {
                    continue;
                }
                let c = fl.center(ix, iz);
                centers += 1;
                if !d.clear(c, 0.3) {
                    blocked += 1;
                }
                for n in [(ix + 1, iz), (ix, iz + 1)] {
                    if fl.is_open(n.0, n.1) && fl.passable((ix, iz), n) && !fl.same_room((ix, iz), n) {
                        open += 1;
                        // Et sted langs kanten der en kropp kommer forbi.
                        let e = (c + fl.center(n.0, n.1)) * 0.5;
                        let across = if n.0 != ix { Vec3::Z } else { Vec3::X };
                        if !(-8..=8).any(|k| {
                            let q = e + across * (k as f32 * 0.2);
                            d.walkable(q, 0.3) && d.clear(q, 0.3)
                        }) {
                            shut.push(format!("etasje {f}: {:?}–{:?}", (ix, iz), n));
                        }
                    }
                }
            }
        }
    }
    eprintln!("{open} åpne kanter, {} sperret; {blocked} av {centers} cellemidter står i noe", shut.len());
    assert!(shut.is_empty(), "hindringene sperrer veien mellom cellene:\n{}", shut.join("\n"));
}





/// Eliten slipper nøkkelen der den kan plukkes opp, også etter at du har
/// dødd og fiendene er kommet igjen, og portgitteret du har låst opp, står
/// oppe etter døden.
#[test]
fn nokkelen_og_porten_etter_doden() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    let pick = |game: &mut Session, physics: &mut RapierPhysics, input: &mut Input| {
        let mut events = vec![];
        game.fell_elite(0, &mut events);
        let key = *game.loose_keys().first().expect("eliten slapp ingen nøkkel");
        let before = game.keys();
        game.place_feet(key + Vec3::new(0.3, 0.0, 0.0));
        for _ in 0..20 {
            game.update(input, 1.0 / 30.0, physics);
        }
        assert_eq!(game.keys(), before + 1, "nøkkelen ved {key:?} ble ikke plukket opp");
    };
    pick(&mut game, &mut physics, &mut input);
    game.use_first(|u| matches!(u, Use::Gate(0)));
    for _ in 0..90 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert_eq!(game.gate(0), (1.0, true), "portgitteret gikk ikke opp");
    game.use_first(|u| *u == Use::Descend);
    game.die_now();
    for _ in 0..10 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert_eq!(game.gate(0), (1.0, true), "portgitteret falt ned igjen da du døde");
    // Fiendene er kommet igjen, og eliten slipper en ny nøkkel.
    game.use_first(|u| *u == Use::Descend);
    pick(&mut game, &mut physics, &mut input);
}

/// Lydbildet følger deg: folk på torget, hamring ved smia, stille i
/// kjelleren.
#[test]
fn lydene_i_byen_folger_deg() {
    use crate::ports::driving::Ambience;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let mut last = |game: &mut Session, physics: &mut RapierPhysics| -> Ambience {
        let mut a = None;
        for _ in 0..20 {
            for e in game.update(&mut input, 1.0 / 30.0, physics) {
                if let Event::Ambience(x) = e {
                    a = Some(x);
                }
            }
        }
        a.unwrap_or_default()
    };
    let sq = crate::domain::world::town::square_center();
    game.place_feet(sq);
    let a = last(&mut game, &mut physics);
    assert!(a.square > 0.8 && !a.cellar, "på torget: {a:?}");
    let forge = world().world.placements.iter().find(|p| p.model == "forge").map(|p| Vec3::from(p.transforms[0].translation)).expect("smia");
    game.place_feet(forge + Vec3::new(0.0, 0.0, 6.0));
    let a = last(&mut game, &mut physics);
    assert!(a.forge > 0.3, "ved smia: {a:?}");
    game.use_first(|u| *u == Use::Descend);
    let a = last(&mut game, &mut physics);
    assert!(a.cellar, "i kjelleren: {a:?}");
}



/// Kløfta: faller du utfor broen, havner du der du sist sto trygt, med
/// vondt.
#[test]
fn kloften_tar_deg_opp_igjen() {
    let w = &world().world;
    let Some(pl) = w.placements.iter().find(|p| p.model == "dng_chasm") else {
        panic!("ingen kløft i kjelleren med dette frøet");
    };
    let t = pl.transforms[0];
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    // Stå trygt på vestsida av kløfta, og hopp så ned i den.
    let west = t.transform_point3(Vec3::new(2.0, 0.0, 6.0));
    let pit = t.transform_point3(Vec3::new(6.0, 0.0, 2.5));
    game.place_feet(west);
    for _ in 0..20 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let hp = game.hp_now();
    game.place_feet(pit + Vec3::Y * 0.5);
    let mut fell = false;
    for _ in 0..120 {
        for e in game.update(&mut input, 1.0 / 30.0, &mut physics) {
            if matches!(&e, Event::Toast(s) if s.contains("kløfta")) {
                fell = true;
            }
        }
    }
    let feet = game.eye() - Vec3::Y * 1.6;
    assert!(fell, "falt ikke i kløfta (føttene ved {feet:?})");
    assert!((feet.y - west.y).abs() < 1.0 && feet.with_y(0.0).distance(west.with_y(0.0)) < 2.0, "kom ikke opp igjen der du sto: {feet:?}, trygt var {west:?}");
    assert!(game.hp_now() < hp, "fallet gjorde ikke vondt");
}


/// Snarveidøra er boltet fra startrommet, kan åpnes fra gangsida, og
/// står åpen så du kan gå gjennom.
#[test]
fn snarveien_apnes_fra_innsida() {
    let d = &world().world.dungeon;
    let sc = *d.shortcuts.first().expect("ingen snarvei i kjelleren");
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let into_room = Vec3::new(sc.yaw.sin(), 0.0, sc.yaw.cos());
    let room_side = sc.pos + into_room * 1.6;
    let hall_side = sc.pos - into_room * 1.6;
    game.place_feet(room_side);
    for _ in 0..5 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let events = game.use_first(|u| *u == Use::Shortcut(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("boltet"))), "døra gikk opp fra startrommet: {events:?}");
    game.place_feet(hall_side);
    for _ in 0..5 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let events = game.use_first(|u| *u == Use::Shortcut(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("åpen"))), "døra gikk ikke opp fra gangen: {events:?}");
    for _ in 0..60 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    // Gå gjennom døråpningen og inn i startrommet.
    let yaw = (-(into_room.x)).atan2(-(into_room.z));
    game.set_look(yaw, 0.0);
    input.key("KeyW", true);
    for _ in 0..90 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.eye() - Vec3::Y * 1.6;
    assert!((feet - sc.pos).with_y(0.0).dot(into_room) > 1.0, "kom ikke gjennom snarveien: føttene ved {feet:?}");
}




/// Skrittene følger underlaget: brostein på torget, gress på enga og stein
/// i kjelleren.
#[test]
fn skrittene_folger_underlaget() {
    use crate::domain::world::{terrain, town};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let steps = |game: &mut Session, physics: &mut RapierPhysics, feet: Vec3| -> Vec<String> {
        let mut heard = vec![];
        // Prøv fire retninger, i tilfelle noe står i veien.
        for k in 0..4 {
            game.place_feet(feet);
            game.set_look(k as f32 * std::f32::consts::FRAC_PI_2, 0.0);
            let mut input = Input { forward: true, ..Default::default() };
            for _ in 0..60 {
                for e in game.update(&mut input, 1.0 / 30.0, physics) {
                    if let Event::Sound(s) = e {
                        if s.name.starts_with("step_") {
                            heard.push(s.name);
                        }
                    }
                }
            }
        }
        heard
    };
    let sq = town::square_center();
    let heard = steps(&mut game, &mut physics, terrain::at(sq + Vec3::new(4.0, 0.0, 4.0)));
    assert!(heard.iter().any(|s| s == "step_rock"), "ingen skritt på brosteinen: {heard:?}");
    // Et sted på enga der det bare er gress.
    let meadow = (0..400)
        .map(|i| {
            let a = i as f32 * 0.37;
            let r = 70.0 + (i % 40) as f32 * 2.0;
            terrain::at(Vec3::new(a.cos() * r, 0.0, a.sin() * r))
        })
        .find(|p| {
            let g = town::ground_weights(p.x, p.z);
            g.cobble + g.dirt + g.rock + g.forest < 0.05 && terrain::normal(p.x, p.z).y > 0.97
        })
        .expect("fant ingen eng");
    let heard = steps(&mut game, &mut physics, meadow);
    assert!(heard.iter().any(|s| s == "step_grass"), "ingen skritt i gresset ved {meadow:?}: {heard:?}");
    let start = world().world.dungeon.start;
    let heard = steps(&mut game, &mut physics, start);
    assert!(heard.iter().all(|s| s != "step_grass") && heard.iter().any(|s| s == "step_rock"), "skrittene i kjelleren: {heard:?}");
}

/// Alle lydene spillet ber om finnes i lydbiblioteket (`assets/sounds`), med
/// filer og kreditering, og de syntetiske lydene manifestet viser til finnes
/// i `web/sounds.js`.
#[test]
fn lydene_finnes() {
    let root = env!("CARGO_MANIFEST_DIR");
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{root}/assets/sounds/sounds.json")).expect("sounds.json")).expect("gyldig json");
    let manifest = manifest.as_object().expect("et objekt");
    let credits = std::fs::read_to_string(format!("{root}/assets/sounds/CREDITS.md")).expect("CREDITS.md");
    let js = std::fs::read_to_string(format!("{root}/web/sounds.js")).expect("sounds.js");
    let mut used = std::collections::BTreeSet::new();
    for path in ["src/application/session.rs", "src/domain/mobs.rs", "src/domain/combat.rs"] {
        let src = std::fs::read_to_string(format!("{root}/{path}")).expect("kilden");
        // Lydnavnene er strenger med små bokstaver og understrek i kall
        // til sfx, sound_at, Sound { name: … }, fiendelydene og kamplydene.
        for marker in ["sfx(\"", "sound_at(\"", "name: \"", "sounds.push((\"", "CombatEvent::Sound(", "\"step_", "\"gear_", "\"hit_", "\"equip_", "\"windup", "\"block_"] {
            let mut rest = src.as_str();
            while let Some(i) = rest.find(marker) {
                rest = &rest[i + marker.len()..];
                // Hent alle strengene på resten av linja.
                let line = &rest[..rest.find('\n').unwrap_or(rest.len())];
                let line = if marker.starts_with('"') {
                    format!("\"{}{line}", &marker[1..])
                } else if marker.ends_with('"') {
                    format!("\"{line}")
                } else {
                    line.to_string()
                };
                for (k, part) in line.split('"').enumerate() {
                    if k % 2 == 1 && !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                        used.insert(part.to_string());
                    }
                }
            }
        }
    }
    // Lydene nettlaget selv ber om.
    for n in ["page", "bag", "click", "heartbeat", "sheep", "cow"] {
        used.insert(n.to_string());
    }
    // Ord som ikke er lyder.
    for n in ["mail", "shield_tower", "outdoor"] {
        used.remove(n);
    }
    let missing: Vec<_> = used.iter().filter(|n| !manifest.contains_key(n.as_str())).collect();
    assert!(missing.is_empty(), "lyder spillet ber om som ikke er i sounds.json: {missing:?}");
    let synth_has = |n: &str| js.contains(&format!("    {n}: (c)"));
    let mut files = std::collections::BTreeSet::new();
    for (name, e) in manifest {
        let list = e["files"].as_array().expect("files");
        assert!(!list.is_empty() || e.get("synth").is_some(), "{name} har verken filer eller syntetisk lyd");
        if let Some(s) = e.get("synth").and_then(|s| s.as_str()) {
            assert!(synth_has(s), "{name}: den syntetiske lyden {s} finnes ikke i web/sounds.js");
        }
        for f in list {
            let f = f.as_str().expect("filnavn");
            assert!(std::path::Path::new(&format!("{root}/assets/sounds/{f}")).exists(), "{name}: {f} mangler");
            assert!(credits.contains(&format!("`{f}`")), "{f} er ikke kreditert i CREDITS.md");
            files.insert(f.to_string());
        }
    }
    for f in std::fs::read_dir(format!("{root}/assets/sounds")).expect("mappa").flatten() {
        let n = f.file_name().to_string_lossy().to_string();
        if n.ends_with(".mp3") {
            assert!(files.contains(&n), "{n} er ikke i sounds.json");
        }
    }
}
