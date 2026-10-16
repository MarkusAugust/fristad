//! Testene som setter sammen hele spillet med de ekte adapterne (de
//! innebygde glTF-modellene og Rapier), uten GPU. `cargo test` før push.

use crate::adapters::gltf::Embedded;
use crate::adapters::rapier::RapierPhysics;
use crate::application::{prepare, rigs, Prepared, Session};
use crate::domain::world::{Shape, Use};
use crate::ports::driving::{Action, Event, Input, SaveData};
use crate::ports::Physics;
use glam::{Affine3A, Vec2, Vec3};
use std::sync::OnceLock;

/// Verden bygges én gang og deles av testene (det tar noen sekunder).
pub(crate) fn world() -> &'static Prepared {
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
    let covers = |shape: &Shape, q: Vec3| shape.covers(q, 0.35);
    // Bitene av kryptnivåene er selve rommene, med murene som kollidere.
    let level_piece = |name: &str| name.strip_prefix("crypt_").is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()));
    let skip = |name: &str| level_piece(name) || name.ends_with("_interior");
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


/// Det du finner i kjelleren havner i lageret uansett hvordan du kommer
/// deg opp: også med H (tilbake til sørporten), ikke bare gjennom døra i
/// trappa. Skriptoriet ser det da.
#[test]
fn funnet_havner_i_lageret_ogsa_med_h() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let events = game.use_first(|u| matches!(u, Use::Chest(_)));
    let found = events.iter().find_map(|e| if let Event::Loot(Some(c)) = e { Some(c.clone()) } else { None }).expect("kista skal gi en komponent");
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.inventory_view().carried.iter().any(|(c, _)| *c == found), "{found} skal ligge i sekken nede");
    input.actions.push(Action::Home);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let v = game.inventory_view();
    assert!(v.carried.is_empty(), "sekken skal være tømt oppe: {:?}", v.carried);
    assert!(v.stash.iter().any(|(c, n)| *c == found && *n > 0), "{found} skal være i lageret: {:?}", v.stash);
}

/// H fra kjelleren tar deg gjennom en virvel til sørporten, og portalen der
/// våkner: F på den tar deg tilbake akkurat dit du var, og den lukker seg.
/// Dør du, tar den deg til inngangen til den dypeste etasjen du har nådd.
#[test]
fn portalen_tar_deg_tilbake() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    assert!(game.portal_destination().is_none(), "portalen sover før du har vært nede");
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    // Gå litt inn i startrommet, så stedet ikke er det samme som starten.
    input.forward = true;
    for _ in 0..20 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    input.forward = false;
    for _ in 0..10 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let was = game.eye();
    input.actions.push(Action::Home);
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(events.iter().any(|e| *e == Event::Warp), "H fra kjelleren skal gi virvelen");
    assert!(game.eye().y > 0.0, "H skal ta deg opp til byen");
    let (to, _, floor) = game.portal_destination().expect("portalen skal være åpen etter H");
    assert_eq!(floor, 0);
    // Portalen er åpen, kan siktes på forfra og tar deg tilbake.
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let events = game.use_first(|u| *u == Use::Portal);
    assert!(events.iter().any(|e| *e == Event::Warp), "portalen skal gi virvelen");
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let back = game.eye();
    assert!(Vec2::new(back.x - was.x, back.z - was.z).length() < 0.2 && (back.y - was.y).abs() < 0.3, "portalen skal ta deg tilbake dit du var: {was:?}, kom til {back:?} (føttene {to:?})");
    assert!(game.portal_destination().is_none(), "portalen lukker seg når du har gått gjennom");
    // Dør du nede, våkner den med inngangen til etasjen.
    game.die_now();
    let (_, _, floor) = game.portal_destination().expect("portalen skal våkne når du dør");
    assert_eq!(floor, 0, "bare første etasje er nådd");
}

/// Kartet: nede i krypten sendes kartet over nivået med rommene du har
/// sett (startrommet og døra opp), og hvor du står; mer blir sett når du
/// går; oppe forsvinner det.
#[test]
fn kartet_viser_det_du_har_sett() {
    use crate::ports::driving::MapView;
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let mut map: Option<MapView> = None;
    let mut wheres = 0;
    let take = |events: Vec<Event>, map: &mut Option<MapView>, wheres: &mut usize| {
        for e in events {
            match e {
                Event::Map(m) => *map = m,
                Event::Where { .. } => *wheres += 1,
                _ => {}
            }
        }
    };
    let e = game.use_first(|u| *u == Use::Descend);
    take(e, &mut map, &mut wheres);
    for _ in 0..10 {
        let e = game.update(&mut input, 1.0 / 30.0, &mut physics);
        take(e, &mut map, &mut wheres);
    }
    let m = map.clone().expect("kartet skal sendes nede i krypten");
    assert_eq!(m.level, c.levels[0].n);
    let seen = m.rooms.len();
    assert!(seen >= 1, "startrommet skal være sett");
    assert!(m.rooms.len() < c.levels[0].rooms.len(), "hele nivået skal ikke være sett med en gang");
    assert!(m.marks.iter().any(|k| k.kind == "ascend"), "døra opp skal stå på kartet: {:?}", m.marks);
    assert!(wheres > 0, "hvor du står skal sendes");
    // Står du i et annet rom, ser du det også.
    let far = c.spawns.iter().find(|s| s.level == 0 && c.room_at(s.pos + Vec3::Y * 0.2) != c.room_at(c.levels[0].start.0 + Vec3::Y * 0.2)).expect("et annet rom").pos;
    game.clear_mobs();
    game.place_feet(far);
    for _ in 0..10 {
        game.clear_mobs();
        let e = game.update(&mut input, 1.0 / 30.0, &mut physics);
        take(e, &mut map, &mut wheres);
    }
    let more = map.as_ref().map(|m| m.rooms.len()).unwrap_or(0);
    assert!(more > seen, "kartet skal vise det nye rommet: {seen} → {more}");
    input.actions.push(Action::Home);
    let e = game.update(&mut input, 1.0 / 30.0, &mut physics);
    take(e, &mut map, &mut wheres);
    let e = game.update(&mut input, 1.0 / 30.0, &mut physics);
    take(e, &mut map, &mut wheres);
    assert!(map.is_none(), "oppe i byen skal kartet være borte");
}

/// Skriptoriet sier hvor hver komponent i oppskriftene finnes, og de som
/// ikke kan kjøpes hos Buttonsmeden merkes «bare i krypten». Alle
/// komponentene i oppskriftene må finnes et sted.
#[test]
fn skjemaet_sier_hvor_komponentene_finnes() {
    use crate::domain::craft;
    let button = craft::source("button");
    assert!(!button.crypt_only() && button.describe().contains("Buttonsmeden"), "{}", button.describe());
    let card = craft::source("card");
    assert!(card.crypt_only() && card.describe().starts_with("Finnes bare i krypten"), "{}", card.describe());
    let mut nowhere = vec![];
    for r in craft::RECIPES {
        for c in r.needs.iter().map(|(c, _)| *c).chain(r.requires.iter().copied()) {
            let s = craft::source(c);
            if s.chests.is_empty() && s.elite.is_empty() && !s.boss && s.smith.is_none() {
                nowhere.push(format!("{c} ({})", r.name));
            }
        }
    }
    assert!(nowhere.is_empty(), "finnes ingen steder: {}", nowhere.join(", "));
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
    let houses = [("tavern", (-1.6, -7.0), (-1.6, -2.0)), ("forge", (0.4, -6.0), (0.4, 0.8)), ("scriptorium", (0.0, -6.0), (0.0, -1.5)), ("apothecary", (1.9, -7.0), (1.9, -1.5)), ("stave_church", (0.0, -7.6), (0.0, 0.0))];
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
    let stairs = [("tavern", (5.3, -3.55), (0.2, -3.55), 0.6, 3.6), ("apothecary", (3.1, -2.7), (3.1, 1.8), 0.7, 3.1), ("scriptorium", (-3.875, -2.75), (-3.875, 2.2), 0.36, 2.8)];
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

/// Ølkjelleren under Vertshuset: ned trappa fra kjøkkenet, gjennom hullet
/// i bakken (`Builder::hole`), til kjellergulvet.
#[test]
fn man_kan_ga_ned_i_kjelleren() {
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.55f32, 0.35f32);
    let pl = p.world.placements.iter().find(|pl| pl.model == "tavern").expect("tavern");
    let t = pl.transforms[0];
    let world = |(x, y): (f32, f32)| t.transform_point3(Vec3::new(x, 0.0, -y));
    let (a, b) = (world((1.5, 2.4)), world((1.5, -2.6)));
    let base = t.transform_point3(Vec3::ZERO).y;
    let mut pos = a + Vec3::Y * (0.6 + half + radius + 0.05);
    let dir = (b - a).with_y(0.0).normalize();
    for _ in 0..600 {
        if (b - pos).with_y(0.0).dot(dir) < 0.05 {
            break;
        }
        let (moved, _) = physics.move_capsule(pos, half, radius, dir * 0.05 - Vec3::Y * 0.06, 0.016);
        pos += moved;
    }
    let feet = pos.y - half - radius - base;
    assert!(feet < -1.8 && (pos - b).with_y(0.0).length() < 0.6, "stoppet med føttene {feet:.2} m over bakken, {:.1} m fra målet", (pos - b).with_y(0.0).length());
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
    assert!(smoke.len() >= 12, "bare {} piper ryker", smoke.len());
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

/// Nettleseren henter modellfilene selv (`Files`, stiene i `model_files`,
/// også kryptmodellene): byen skal bli den samme som med modellene bygd inn.
#[test]
fn nettleseren_far_de_samme_modellene() {
    use crate::adapters::gltf::{model_files, Files};
    let files = model_files().map(|(_, p)| (p.to_string(), std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).expect(p))).collect();
    let web = prepare(&Files(files), 7);
    let native = world();
    let names = |p: &Prepared| p.models.iter().map(|m| (m.name.clone(), m.mesh.indices.len())).collect::<Vec<_>>();
    assert_eq!(names(&web), names(native));
    assert_eq!(web.world.placements.len(), native.world.placements.len());
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
    let start = world().world.crypt.start().expect("krypten har et nivå").0;
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
    // Lydnavnene er strenger med små bokstaver og understrek i kall til
    // sfx, sound_at, Sound { name: … }, fiendelydene og kamplydene (i hele
    // applikasjonen, fiendene og kampen), lydene reglene sier fra om
    // (`Note`), og lydene utstyret har (`sound: "…"`).
    const CALLS: &[&str] = &["sfx(\"", "sound_at(\"", "name: \"", "sounds.push((\"", "CombatEvent::Sound(", "\"step_", "\"gear_", "\"hit_", "\"equip_", "\"windup", "\"block_", "cues.push(\"", "sounds.push((if", "let name = [", "Note::Sfx(\"", "Note::At(\""];
    let mut sources: Vec<(String, &[&str])> = vec![];
    for e in std::fs::read_dir(format!("{root}/crates/fristad-app/src/application")).expect("applikasjonen").flatten() {
        sources.push((e.path().to_string_lossy().into_owned(), CALLS));
    }
    for f in ["mobs.rs", "combat.rs", "hero.rs", "world/crypt/run.rs", "world/mechanisms.rs"] {
        sources.push((format!("{root}/crates/fristad-domain/src/domain/{f}"), CALLS));
    }
    sources.push((format!("{root}/crates/fristad-domain/src/domain/craft.rs"), &["sound: \""]));
    for (path, markers) in &sources {
        let src = std::fs::read_to_string(path).expect("kilden");
        for marker in markers.iter().copied() {
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

/// Fiendene har alle klippene fra mob_motion.py, og hodet dreier seg mot
/// spilleren når de har sett deg.
#[test]
#[ignore = "venter på de nye fiendene: de gamle modellene med klippene er slettet (rydd/gamle), så ingen slag har rigg"]
fn fiendene_ser_mot_deg() {
    use crate::domain::mobs::{Kind, Mobs, State};
    let p = world();
    let all = rigs(&p.models);
    for k in Kind::ALL {
        let (_, rig) = all.iter().find(|(n, _)| n == k.model()).unwrap_or_else(|| panic!("{} har ingen rigg", k.model()));
        for c in ["idle", "walk", "attack", "alert", "hit", "hit2", "stagger", "die", "die2"] {
            assert!(rig.clip(c).is_some(), "{} mangler klippet {c}", k.model());
        }
    }
    let mut mobs = Mobs::new(&p.world.crypt, &all);
    let i = mobs.mobs.iter().position(|m| !m.kind.boss()).expect("en fiende");
    let mut flip = 0;
    for side in [1.0f32, -1.0] {
        let m = &mut mobs.mobs[i];
        m.yaw = 0.0;
        m.aware = true;
        m.state = State::Idle;
        // Spilleren står skrått til siden for fienden (front er +z ved yaw 0).
        let to = Vec3::new(side * 0.8, 0.0, 0.6).normalize();
        let player = m.pos + to * 6.0;
        for _ in 0..60 {
            // Utenfor kjelleren står den stille, men ser fortsatt mot deg.
            mobs.update(1.0 / 30.0, player, false, &p.world.crypt);
        }
        let turned = mobs.head_dir(i).expect("hodet");
        let look = mobs.mobs[i].look;
        mobs.mobs[i].look = 0.0;
        let ahead = mobs.head_dir(i).expect("hodet");
        mobs.mobs[i].look = look;
        let ang = |d: Vec3| d.dot(to).clamp(-1.0, 1.0).acos();
        assert!(ahead.dot(Vec3::Z) > 0.7, "uten dreining ser hodet ikke fram ({ahead:?})");
        assert!(ang(turned) < ang(ahead) - 0.3, "hodet dreier seg ikke mot spilleren på side {side}: {turned:?} mot {ahead:?}, ønsket {to:?}");
        flip += 1;
    }
    assert_eq!(flip, 2);
}

/// Alle de tjue husmodellene står et sted: de femten byhusene i byen, og
/// de laftede og bordkledde som våningshus på bondegårdene utenfor muren.
#[test]
fn alle_husene_er_i_bruk() {
    let p = world();
    let missing: Vec<String> = (b'a'..=b't').map(|c| format!("house_{}", c as char)).filter(|m| !p.world.placements.iter().any(|pl| pl.model == m.as_str())).collect();
    assert!(missing.is_empty(), "står ingen steder: {}", missing.join(", "));
}

/// Senga i huset ditt: F gir fullt liv og full utholdenhet, skjermen går i
/// svart, og døgnet snur (dag til natt og natt til dag).
#[test]
fn senga_gir_hvile_og_snur_dognet() {
    use crate::domain::weather::Daylight;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert_eq!(game.daylight(), Daylight::Day);
    game.set_vitals(12.0, 3.0);
    let events = game.use_first(|u| *u == Use::Sleep);
    assert!(events.contains(&Event::Fade), "skjermen skal gå i svart: {events:?}");
    assert!(events.contains(&Event::Daylight(Daylight::Night)), "døgnet skal snu: {events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("kvelden"))), "{events:?}");
    assert_eq!(game.daylight(), Daylight::Night);
    assert!(game.hp_now() >= 100.0, "fullt liv etter søvnen: {}", game.hp_now());
    assert!(game.stamina_now() >= 100.0, "full utholdenhet etter søvnen: {}", game.stamina_now());
    let events = game.use_first(|u| *u == Use::Sleep);
    assert!(events.contains(&Event::Daylight(Daylight::Day)), "og tilbake til morgenen: {events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("morgenen"))), "{events:?}");
    assert_eq!(game.daylight(), Daylight::Day);
}

/// Galleriveggen: en komponent du bærer opp fra kjelleren, henger i sin
/// ramme for alltid, også etter lagring og innlasting, og selv om lageret
/// er tomt (solgt eller brukt opp).
#[test]
fn funnet_henger_i_galleriet() {
    use crate::domain::craft::{GALLERY, GALLERY_MODELS};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    assert!(game.found().is_empty(), "ingenting funnet før du har vært nede");
    game.use_first(|u| *u == Use::Descend);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let events = game.use_first(|u| matches!(u, Use::Chest(_)));
    let found = events.iter().find_map(|e| if let Event::Loot(Some(c)) = e { Some(c.clone()) } else { None }).expect("kista skal gi en komponent");
    assert!(GALLERY.contains(&found.as_str()), "{found} skal ha en ramme");
    assert!(!game.found().contains(&found), "det du bærer nede er ikke ditt ennå");
    input.actions.push(Action::Home);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.found().contains(&found), "{found} skal henge i galleriet: {:?}", game.found());
    let frame = game.frame(16.0 / 9.0);
    let pieces: Vec<&str> = frame.instances.iter().map(|i| i.model).filter(|m| GALLERY_MODELS.contains(m)).collect();
    assert_eq!(pieces.len(), game.found().iter().filter(|c| GALLERY.contains(&c.as_str())).count(), "én komponent per fylt ramme");
    // Rammen viser bildet av akkurat den komponenten, ikke en annen.
    let n = GALLERY.iter().position(|c| *c == found).unwrap();
    assert!(pieces.contains(&GALLERY_MODELS[n]), "{found} skal henge som {}: {pieces:?}", GALLERY_MODELS[n]);

    let saved = game.save();
    assert!(saved.found.contains(&found));
    let mut physics2 = RapierPhysics::new();
    let mut fresh = session(&mut physics2);
    fresh.load(SaveData { stash: vec![], ..saved });
    assert!(fresh.found().contains(&found), "galleriet skal huskes etter innlasting, også med tomt lager");
    // Gamle lagringer uten galleriet: lageret teller.
    let mut physics3 = RapierPhysics::new();
    let mut old = session(&mut physics3);
    old.load(SaveData { stash: vec![("badge".into(), 1)], found: vec![], ..Default::default() });
    assert_eq!(old.found(), vec!["badge".to_string()]);
}

/// F på en fylt ramme åpner boka for komponenten i rammen; en tom ramme
/// gjør ingenting.
#[test]
fn rammen_apner_boka() {
    use crate::domain::craft::GALLERY;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    assert_eq!(GALLERY[4], "card");
    let events = game.use_first(|u| *u == Use::Gallery(4));
    assert!(!events.iter().any(|e| matches!(e, Event::OpenDoc(_))), "tom ramme: {events:?}");
    game.give("card", 1);
    let events = game.use_first(|u| *u == Use::Gallery(4));
    assert!(events.contains(&Event::OpenDoc("card".into())), "{events:?}");
    let events = game.use_first(|u| *u == Use::Gallery(0));
    assert!(!events.iter().any(|e| matches!(e, Event::OpenDoc(_))), "button er ikke funnet: {events:?}");
}

// --- Krypten ------------------------------------------------------------------
//
// Testene går gjennom nivåene som finnes (`crypt_NN.json`: det ekte nivået
// fra Blender, ellers testnivået), styrt av merkene, så de virker for alle
// nivåer uten å vite hvordan de ser ut.

use crate::domain::world::crypt::{self, Crypt};

/// Krypten med alle bryterne åpne (porter, dører, falske vegger, lemmer).
fn crypt_open() -> Crypt {
    let mut c = world().world.crypt.clone();
    c.open = vec![true; c.switches.len()];
    c
}

/// Gulvet under (x, z) nærmest høyden i punktet: hjørnene på veien kan stå
/// langt fra hverandre (en lang gang med trapp), så høyden på en rett linje
/// imellom er ikke gulvet.
fn floor_near(c: &Crypt, p: Vec3) -> Vec3 {
    let mut best: Option<f32> = None;
    for off in [0.0f32, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0, 4.0, -4.0] {
        if let Some(h) = c.floor_under(p + Vec3::Y * off) {
            if best.is_none_or(|b| (h - p.y).abs() < (b - p.y).abs()) {
                best = Some(h);
            }
        }
    }
    best.map_or(p, |h| p.with_y(h))
}

/// Punktet `d` meter langs en vei av hjørner.
fn along(path: &[Vec3], mut d: f32) -> Vec3 {
    for w in path.windows(2) {
        let l = w[0].distance(w[1]);
        if d <= l {
            return w[0].lerp(w[1], d / l.max(1e-4));
        }
        d -= l;
    }
    *path.last().unwrap()
}

/// Om det er åpen vei helt fram fra `a` til `b` (`Crypt::path` går til
/// nærmeste åpne sted ved målet).
fn connected(c: &Crypt, a: Vec3, b: Vec3) -> bool {
    c.path(a, b).is_some_and(|p| p.last().is_some_and(|e| e.with_y(0.0).distance(b.with_y(0.0)) < 0.3))
}

fn path_len(path: &[Vec3]) -> f32 {
    path.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// Et sted på gulvet foran et punkt (`yaw` sier hvor fronten er; negativ
/// `d` er bak), også bak en stengt port.
fn in_front(c: &Crypt, pos: Vec3, yaw: f32, d: f32) -> Vec3 {
    let f = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let q = pos + f * d;
    match c.floor_under(q + Vec3::Y * 0.3) {
        Some(y) if (y - pos.y).abs() < 1.0 => q.with_y(y),
        _ => c.snap(q + Vec3::Y * 0.3, 2.0).unwrap_or(q),
    }
}

/// Ned i krypten fra kirkegården: døra toner til svart og setter deg i
/// nivå 1, fiendene jager og biter, og de kan slås i hjel. Døra opp tar deg
/// til krypten i byen igjen.
#[test]
fn krypten_kan_nas_og_fiendene_biter() {
    let p = world();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y > 0.0, "starter oppe");
    let events = game.use_first(|u| *u == Use::Descend);
    assert!(events.iter().any(|e| matches!(e, Event::Fade)), "nedgangen skal tone til svart");
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y < crypt::TOP, "døra i krypten skal ta deg ned: øyet er på y = {:.1}", game.eye().y);
    assert!(events.iter().any(|e| matches!(e, Event::Hud { cellar: true, .. })), "HUD-en skal melde at du er nede");
    // Du står igjen på gulvet.
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let (start, _) = p.world.crypt.start().unwrap();
    assert!((game.feet().y - start.y).abs() < 0.5, "falt gjennom gulvet: føttene på {:.2}, startstedet på {:.2}", game.feet().y, start.y);
    // Still deg nær en fiende som er våken, og vent til den biter.
    let c = &p.world.crypt;
    let s = c.spawns.iter().find(|s| s.level == 0 && s.ambush.is_none() && !s.boss).expect("en fiende i nivå 1");
    let near = c.path(s.pos, start).map(|mut path| {
        path.insert(0, s.pos);
        along(&path, 3.0)
    }).unwrap_or(s.pos);
    game.place_feet(near);
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
    let mut killed = false;
    'outer: for _ in 0..(30 * 20) {
        if let Some(m) = game.nearest_mob(game.feet()) {
            let d = m + Vec3::Y * 0.8 - game.eye();
            game.set_look(-d.x.atan2(-d.z), d.y.atan2(d.with_y(0.0).length()));
        }
        game.heal();
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
    game.use_first(|u| *u == Use::Ascend(0));
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y > 0.0, "døra opp skal ta deg til krypten: øyet er på y = {:.1}", game.eye().y);
}

/// Hvert nivå har start, hvilested og trappa ned (unntatt det nederste), og
/// taket står minst `HEADROOM` over hvert hjørne i navigasjonsnettet.
#[test]
fn kryptnivaaene_har_det_de_skal_og_taket_er_hoyt_nok() {
    let c = &world().world.crypt;
    assert!(!c.levels.is_empty(), "ingen kryptnivåer");
    let mut problems = vec![];
    for (k, l) in c.levels.iter().enumerate() {
        if c.rests.iter().filter(|r| r.level == k).count() != 1 {
            problems.push(format!("nivå {}: ikke nøyaktig ett hvilested", l.n));
        }
        if l.down.is_none() && l.n != 10 {
            problems.push(format!("nivå {}: ingen trapp ned", l.n));
        }
        if l.nav.polys.is_empty() {
            problems.push(format!("nivå {}: tomt navigasjonsnett", l.n));
        }
        for (i, poly) in l.nav.polys.iter().enumerate() {
            for v in &poly.pts {
                let q = Vec2::new(v.x, v.z);
                let roof = l.rooms.iter().filter(|r| v.y >= r.y - 1.2 && v.y <= r.y + r.h && r.contains(q)).map(|r| r.y + r.h).fold(f32::MIN, f32::max);
                if roof == f32::MIN {
                    problems.push(format!("nivå {}: hjørnet {v:?} i polygon {i} står ikke i noe rom", l.n));
                } else if roof - v.y < crypt::HEADROOM - 0.01 {
                    problems.push(format!("nivå {}: bare {:.2} m til taket over {v:?}", l.n, roof - v.y));
                }
            }
        }
    }
    problems.dedup();
    assert!(problems.is_empty(), "\n{}", problems.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
}

/// Navigasjonsnettet: hver fiende står på nettet og har vei til startstedet
/// når alle porter er åpne, ingen polygon er avskåret (unntatt i hemmelige
/// rom, som har en annen vei ut), og fiendene finner fram: en fiende som har
/// sett deg, følger deg hele veien fra der den står til startstedet uten å
/// sette seg fast.
#[test]
fn fiendene_finner_fram_i_krypten() {
    use crate::domain::mobs::Mobs;
    let p = world();
    let c = crypt_open();
    let all = rigs(&p.models);
    let mut problems = vec![];
    for (k, l) in c.levels.iter().enumerate() {
        let (start, _) = l.start;
        let Some(root) = l.nav.locate(start + Vec3::Y * 0.3, 0.6) else {
            problems.push(format!("nivå {}: startstedet står ikke på nettet", l.n));
            continue;
        };
        let reach = l.nav.reachable(root, &|_| true);
        for (i, poly) in l.nav.polys.iter().enumerate() {
            let secret = poly.room.is_some_and(|r| l.rooms[r].secret);
            if !reach[i] && !secret {
                problems.push(format!("nivå {}: polygon {i} ved {:?} kan ikke nås fra start", l.n, poly.center));
            }
        }
        for s in c.spawns.iter().filter(|s| s.level == k) {
            if !c.walkable(s.pos + Vec3::Y * 0.1) {
                problems.push(format!("nivå {}: {} ved {:?} står ikke på nettet", l.n, s.label, s.pos));
            } else if c.path(s.pos, start).is_none() {
                problems.push(format!("nivå {}: {} ved {:?} har ingen vei til start", l.n, s.label, s.pos));
            }
        }
        // Hver fiende følger deg til start. Du går foran, og venter til den
        // har tatt deg igjen.
        let n = {
            let mut m = Mobs::new(&c, &all);
            m.enter(&c, k);
            m.mobs.len()
        };
        for i in 0..n {
            let mut mobs = Mobs::new(&c, &all);
            mobs.enter(&c, k);
            let one = mobs.mobs.remove(i);
            mobs.mobs = vec![one];
            mobs.wake_all();
            let from = mobs.mobs[0].pos;
            let label = format!("{:?}", mobs.mobs[0].kind);
            let Some(mut path) = c.path(from, start) else { continue };
            path.insert(0, from);
            let len = path_len(&path);
            let mut ahead = 4.0f32.min(len);
            let speed = mobs.mobs[0].kind.stats().speed;
            let limit = len / speed.max(0.5) * 3.0 + 20.0;
            let (mut t, mut best, mut since) = (0.0f32, f32::MAX, 0.0f32);
            loop {
                // Hjørnene på veien er langt fra hverandre i en lang gang med
                // trapp: høyden imellom er gulvet, ikke en rett linje.
                let p0 = along(&path, ahead);
                let player = floor_near(&c, p0);
                mobs.update(1.0 / 30.0, player, true, &c);
                t += 1.0 / 30.0;
                let m = &mobs.mobs[0];
                let gap = m.pos.with_y(0.0).distance(player.with_y(0.0));
                if gap < 2.6 && (m.pos.y - player.y).abs() < 1.6 {
                    if ahead >= len {
                        break;
                    }
                    ahead = (ahead + 4.0).min(len);
                    best = f32::MAX;
                }
                if gap < best - 0.1 {
                    best = gap;
                    since = t;
                }
                if t - since > 8.0 || t > limit {
                    problems.push(format!("nivå {}: {label} fra {from:?} satt fast ved {:?} ({gap:.1} m bak deg ved {player:?}) etter {t:.0} s", l.n, m.pos));
                    break;
                }
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Fiendene med en rute går den når de ikke ser deg, og de i bakhold sover
/// til du kommer nær eller går inn i rommet deres.
#[test]
fn fiendene_patruljerer_og_ligger_i_bakhold() {
    use crate::domain::mobs::Mobs;
    use crate::domain::world::crypt::Ambush;
    let p = world();
    let c = crypt_open();
    let all = rigs(&p.models);
    let far = Vec3::new(0.0, 500.0, 0.0);
    let (mut patrols, mut ambushes) = (0, 0);
    for (k, _) in c.levels.iter().enumerate() {
        let mut mobs = Mobs::new(&c, &all);
        mobs.enter(&c, k);
        let spawns: Vec<_> = c.spawns.iter().filter(|s| s.level == k).collect();
        // Rutene: hvert punkt nås.
        let mut visited: Vec<Vec<f32>> = mobs.mobs.iter().map(|_| vec![f32::MAX; 8]).collect();
        let route = |m: &crate::domain::mobs::Mob| spawns.iter().find(|s| !s.patrol.is_empty() && s.pos.distance(m.pos) < 3.0).map(|s| s.patrol.clone());
        let routes: Vec<Option<Vec<Vec3>>> = mobs.mobs.iter().map(route).collect();
        // To minutter: Button Wight tasser på korte bein (1,4 m/s).
        for _ in 0..(30 * 120) {
            mobs.update(1.0 / 30.0, far, true, &c);
            for (i, m) in mobs.mobs.iter().enumerate() {
                if let Some(r) = &routes[i] {
                    for (j, q) in r.iter().enumerate().take(8) {
                        visited[i][j] = visited[i][j].min(m.pos.with_y(0.0).distance(q.with_y(0.0)));
                    }
                }
            }
        }
        for (i, r) in routes.iter().enumerate() {
            let Some(r) = r else { continue };
            patrols += 1;
            for (j, q) in r.iter().enumerate().take(8) {
                assert!(visited[i][j] < 1.5, "fienden på ruta {r:?} kom aldri til punkt {j} ({q:?}); nærmest {:.1} m", visited[i][j]);
            }
        }
        // Bakholdet.
        for s in spawns.iter().filter(|s| s.ambush.is_some()) {
            ambushes += 1;
            let mut mobs = Mobs::new(&c, &all);
            mobs.enter(&c, k);
            let i = mobs.mobs.iter().position(|m| m.pos.distance(s.pos) < 3.0).expect("fienden i bakhold");
            let home = mobs.mobs[i].pos;
            for _ in 0..90 {
                mobs.update(1.0 / 30.0, far, true, &c);
            }
            assert!(mobs.dormant(i) && mobs.mobs[i].pos.distance(home) < 0.05, "fienden i bakhold ved {home:?} sov ikke");
            // Du kommer inn i rommet, et par meter fra den.
            let near = match s.ambush {
                Some(Ambush::Room(r)) => {
                    let room = &c.levels[k].rooms[r];
                    c.snap(Vec3::new((room.min.x + room.max.x) * 0.5, room.y + 0.3, (room.min.y + room.max.y) * 0.5), 3.0).unwrap()
                }
                _ => c.path(home, c.levels[k].start.0).map(|mut path| {
                    path.insert(0, home);
                    along(&path, 2.0)
                }).unwrap(),
            };
            for _ in 0..60 {
                mobs.update(1.0 / 30.0, near, true, &c);
            }
            assert!(!mobs.dormant(i), "fienden i bakhold ved {home:?} våknet ikke da du sto ved {near:?}");
        }
    }
    assert!(patrols > 0, "ingen fiender patruljerer");
    assert!(ambushes > 0, "ingen fiender ligger i bakhold");
}

/// Boten som går gjennom krypten med ekte fysikk: styrer med tastene og
/// musa, følger veien på navigasjonsnettet, trekker spakene den trenger, og
/// slåss (i gjennomspillingen) med det som kommer.
struct Bot {
    input: Input,
    t: f32,
    fight: bool,
    near_deaths: u32,
    kills: u32,
    chests: u32,
    log: Vec<String>,
    swing: u32,
    last: Vec3,
    stuck: f32,
    sidestep: f32,
    side: bool,
}

impl Bot {
    const DT: f32 = 1.0 / 30.0;

    fn new(fight: bool) -> Self {
        Bot { input: Input::default(), t: 0.0, fight, near_deaths: 0, kills: 0, chests: 0, log: vec![], swing: 0, last: Vec3::ZERO, stuck: 0.0, sidestep: 0.0, side: false }
    }

    fn note(&mut self, s: String) {
        eprintln!("{s}");
        self.log.push(s);
    }

    fn fail(&self, msg: String) -> ! {
        panic!("{msg}\n{}", self.log.join("\n"))
    }

    fn step(&mut self, game: &mut Session, physics: &mut RapierPhysics) -> Vec<Event> {
        if !self.fight {
            game.clear_mobs();
        }
        let events = game.update(&mut self.input, Self::DT, physics);
        self.t += Self::DT;
        for e in &events {
            match e {
                Event::Hud { hp, cellar: true, .. } if *hp < 60 => {
                    self.near_deaths += 1;
                    game.heal();
                }
                Event::Toast(t) if t.contains("falt fra hverandre") => self.kills += 1,
                Event::Toast(t) if t.contains("overkjørt") => self.fail(format!("boten døde: {t}")),
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

    /// Slåss med den nærmeste fienden innen tre meter som er på samme gulv
    /// og ikke bak en mur. Gir om det var noen.
    fn brawl(&mut self, game: &mut Session, physics: &mut RapierPhysics) -> bool {
        if !self.fight {
            return false;
        }
        let feet = game.feet();
        let c = game.crypt_now();
        let near = game
            .living()
            .into_iter()
            .filter(|(m, _, _)| (m.y - feet.y).abs() < 2.0 && m.with_y(0.0).distance(feet.with_y(0.0)) < 3.0)
            .filter(|(m, _, _)| c.path(feet, *m).is_some_and(|p| path_len(&p) < 4.0))
            .min_by(|a, b| a.0.distance(feet).total_cmp(&b.0.distance(feet)));
        let Some((m, kind, _)) = near else { return false };
        self.input.key("KeyW", m.with_y(0.0).distance(feet.with_y(0.0)) > 1.5 * if kind.boss() { 1.5 } else { 1.0 });
        self.aim(game, m + Vec3::Y * 0.8);
        self.swing += 1;
        self.input.actions.push(if self.swing % 2 == 0 { Action::Attack } else { Action::AttackRelease });
        self.step(game, physics);
        true
    }

    /// Gå langs navigasjonsnettet til du er innen `near` meter fra målet.
    /// Gir `false` om det ikke er noen vei (en port er stengt), og feiler om
    /// boten står fast.
    fn walk_to(&mut self, game: &mut Session, physics: &mut RapierPhysics, to: Vec3, near: f32, what: &str) -> bool {
        let (mut best, mut since) = (f32::MAX, self.t);
        loop {
            let feet = game.feet();
            let dist = feet.with_y(0.0).distance(to.with_y(0.0));
            if dist < near && (feet.y - to.y).abs() < 1.5 {
                self.input.key("KeyW", false);
                return true;
            }
            if self.brawl(game, physics) {
                since = self.t;
                continue;
            }
            // Slipp et tungt hugg som lades, ellers går boten ikke videre.
            if self.swing % 2 == 0 && self.swing > 0 {
                self.swing += 1;
                self.input.actions.push(Action::AttackRelease);
            }
            let Some(path) = game.crypt_now().path(feet + Vec3::Y * 0.2, to) else {
                self.input.key("KeyW", false);
                return false;
            };
            let left = path_len(&path) + path.first().map_or(0.0, |c| c.distance(feet));
            if left < best - 0.2 {
                best = left;
                since = self.t;
            }
            if self.t - since > 10.0 {
                let mobs: Vec<_> = game.living().into_iter().filter(|(m, _, _)| m.distance(feet) < 6.0).map(|(m, k, _)| format!("{k:?} ved {m:?}")).collect();
                self.fail(format!("sto fast på vei til {what}: føttene ved {feet:?}, {left:.1} m igjen, neste hjørne {:?}, fiender nær: {mobs:?}, tastene {:?}, liv {} utholdenhet {}, øyet {:.2} over føttene", path.first(), (self.input.forward, self.input.left, self.input.right, self.input.crouch, self.input.block), game.hp_now(), game.stamina_now(), game.eye().y - feet.y));
            }
            let next = path.iter().copied().find(|c| c.with_y(0.0).distance(feet.with_y(0.0)) > 0.25).unwrap_or(to);
            if std::env::var("FRISTAD_BOT_TRACE").is_ok() && (self.t * 30.0) as u32 % 15 == 0 {
                eprintln!("  [{:.1}] føttene {feet:?} neste {next:?} igjen {left:.1}", self.t);
            }
            self.aim(game, next.with_y(game.eye().y));
            self.input.key("KeyW", true);
            let moved = feet.distance(self.last);
            self.last = feet;
            self.stuck = if moved < 0.01 { self.stuck + Self::DT } else { 0.0 };
            if self.stuck > 1.0 && self.sidestep <= 0.0 {
                self.sidestep = 0.6;
                self.side = !self.side;
            }
            self.input.key(if self.side { "KeyA" } else { "KeyD" }, self.sidestep > 0.0);
            self.sidestep -= Self::DT;
            self.step(game, physics);
            if self.sidestep <= 0.0 {
                self.input.key("KeyA", false);
                self.input.key("KeyD", false);
            }
        }
    }

    /// Gjennom nivå `k`: fra der boten står til trappa ned, og trekker
    /// spakene (og henter nøkkelen) når veien er stengt.
    fn through_level(&mut self, game: &mut Session, physics: &mut RapierPhysics, k: usize) {
        let c = &world().world.crypt;
        let l = &c.levels[k];
        let Some((down, _)) = l.down else { return };
        let t0 = self.t;
        self.note(format!("nivå {} ({}): starter ved {:?}", l.n, l.name, game.feet()));
        if self.fight {
            // Kistene på veien (de som kan nås nå).
            for (i, ch) in c.chests.iter().enumerate().filter(|(_, ch)| ch.level == k) {
                let at = in_front(c, ch.pos, ch.yaw, 0.9);
                if self.walk_to(game, physics, at, 0.6, "kista") {
                    for e in game.use_exact(Use::Chest(i)) {
                        if matches!(e, Event::Loot(Some(_))) {
                            self.chests += 1;
                        }
                    }
                }
            }
        }
        for _ in 0..12 {
            // Trappa ned tar deg videre alt 1,2 m fra merket: er boten i
            // nivået under, er den kommet fram.
            if self.walk_to(game, physics, down, 1.0, "trappa ned") || c.level_at(game.feet()) == Some(k + 1) {
                self.note(format!("  trappa ned etter {:.0} s", self.t - t0));
                return;
            }
            // Veien er stengt: en spak som kan nås, eller nøkkelen.
            let feet = game.feet();
            let lever = c.levers.iter().enumerate().find(|(i, lv)| lv.level == k && !game.lever_pulled(*i) && game.crypt_now().path(feet + Vec3::Y * 0.2, in_front(c, lv.pos, lv.yaw, 0.9)).is_some());
            if let Some((i, lv)) = lever {
                let at = in_front(c, lv.pos, lv.yaw, 0.9);
                self.walk_to(game, physics, at, 0.5, "spaken");
                self.aim(game, lv.pos + Vec3::Y * 0.9);
                game.use_exact(Use::Lever(i));
                self.note(format!("  trakk spaken ved {:?} etter {:.0} s", lv.pos, self.t - t0));
                self.idle(game, physics, 2.5);
                continue;
            }
            let gate = c.gates.iter().enumerate().find(|(i, g)| g.level == k && g.key && !game.gate(*i).1);
            if let Some((i, g)) = gate {
                let mut events = vec![];
                if !game.fell_elite(&mut events) {
                    // Uten fiender (den korte gjennomgangen) er eliten borte: nøkkelen i lomma.
                    game.give_key();
                }
                if let Some(key) = game.loose_keys().first().copied() {
                    self.walk_to(game, physics, key, 0.4, "nøkkelen");
                }
                let side = [in_front(c, g.pos, g.yaw, 1.5), in_front(c, g.pos, g.yaw, -1.5)].into_iter().find(|q| game.crypt_now().path(game.feet() + Vec3::Y * 0.2, *q).is_some());
                if let Some(q) = side {
                    self.walk_to(game, physics, q, 0.5, "porten");
                }
                game.use_exact(Use::Gate(i));
                self.idle(game, physics, 3.0);
                continue;
            }
            // Porten ut av arenaen er stengt så lenge bossen lever: slå den.
            if let Some(boss) = game.boss_pos() {
                self.note(format!("  går mot bossen ved {boss:?} etter {:.0} s", self.t - t0));
                let t1 = self.t;
                while let Some(at) = game.boss_pos() {
                    if self.t - t1 > 180.0 {
                        self.fail(format!("nivå {}: fikk ikke slått bossen på tre minutter", l.n));
                    }
                    if !self.brawl(game, physics) {
                        self.walk_to(game, physics, at, 1.8, "bossen");
                    }
                }
                self.note(format!("  bossen falt etter {:.0} s", self.t - t0));
                self.idle(game, physics, 2.0);
                continue;
            }
            self.fail(format!("nivå {}: ingen vei til trappa ned fra {feet:?}, og ingen spak eller port å åpne", l.n));
        }
        self.fail(format!("nivå {}: kom ikke fram til trappa ned", l.n));
    }
}

/// Hele krypten kan gås med ekte fysikk (Rapier og økta): fra startstedet
/// i nivå 1, med spakene trukket etter tur, ned trappa i hvert nivå til det
/// nederste. Fiendene er borte, så det er veien som testes.
#[test]
fn kryptnivaaene_kan_gas_gjennom() {
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut bot = Bot::new(false);
    bot.step(&mut game, &mut physics);
    game.use_first(|u| *u == Use::Descend);
    bot.idle(&mut game, &mut physics, 0.5);
    for k in 0..c.levels.len() {
        bot.through_level(&mut game, &mut physics, k);
        bot.idle(&mut game, &mut physics, 0.5);
        if k + 1 < c.levels.len() {
            assert_eq!(c.level_at(game.feet()), Some(k + 1), "trappa ned i nivå {} tok deg ikke til neste\n{}", k + 1, bot.log.join("\n"));
        }
    }
}

/// Gjennomspillingen: boten går gjennom alle nivåene med fiendene, åpner
/// kistene den når, trekker spakene og slåss. Den slåss uten å blokke, så
/// den får fullt liv når livet er under 60 (og teller det).
/// `cargo test --release gjennomspilling -- --ignored --nocapture`.
#[test]
#[ignore]
fn gjennomspilling() {
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut bot = Bot::new(true);
    bot.step(&mut game, &mut physics);
    game.use_first(|u| *u == Use::Descend);
    bot.idle(&mut game, &mut physics, 0.5);
    for k in 0..c.levels.len() {
        bot.through_level(&mut game, &mut physics, k);
        bot.idle(&mut game, &mut physics, 0.5);
    }
    bot.note(format!("ferdig på {:.0} s spilltid: {} fiender drept, {} kister åpnet, {} ganger nesten død", bot.t, bot.kills, bot.chests, bot.near_deaths));
}

/// Hvilestedet: før du har tent det, våkner du i byen når du dør. Har du
/// tent lysene, våkner du ved alteret, med fullt liv.
#[test]
fn hvilestedet_vekker_deg_der_du_tente() {
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.die_now();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.eye().y > 0.0, "uten tent hvilested skal du våkne i byen");
    game.use_first(|u| *u == Use::Descend);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(!game.frame(1.6).instances.iter().any(|i| i.model == crate::domain::world::crypt::KIT_REST_FLAME), "flammen brenner før hvilestedet er tent");
    let events = game.use_exact(Use::Rest(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("tenner"))), "lysene ble ikke tent: {events:?}");
    // Tent: flammen synes i skåla (eierens dom i runde 7: «ikke noe brennende lys?»).
    let rest = c.rests[0].pos;
    assert!(game.frame(1.6).instances.iter().any(|i| i.model == crate::domain::world::crypt::KIT_REST_FLAME && (glam::Vec3::from(i.transform.translation) - rest).length() < 1e-3), "ingen flamme på det tente hvilestedet");
    game.place_feet(c.levels[0].down.map(|d| d.0).unwrap_or(rest + Vec3::X * 5.0));
    game.set_vitals(10.0, 0.0);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.die_now();
    for _ in 0..10 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.feet();
    assert!(feet.with_y(0.0).distance(rest.with_y(0.0)) < 3.0 && (feet.y - rest.y).abs() < 1.0, "våknet ikke ved hvilestedet {rest:?}, men ved {feet:?}");
    assert_eq!(game.hp_now(), game.max_hp_now());
}

/// Spaken heiser gitteret den hører til: før er det ingen vei gjennom, etter
/// er gitteret oppe og veien åpen.
#[test]
fn spaken_heiser_gitteret() {
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    let (n, lever) = c.levers.iter().enumerate().find(|(_, l)| !l.opens.is_empty()).expect("ingen spak i krypten");
    let g = c.gates.iter().position(|g| lever.opens.contains(&g.switch)).expect("spaken åpner ingen port");
    let gate = &c.gates[g];
    let (a, b) = (in_front(c, gate.pos, gate.yaw, 1.6), in_front(c, gate.pos, gate.yaw, -1.6));
    assert!(!connected(game.crypt_now(), a, b), "det er vei forbi det stengte gitteret");
    game.place_feet(in_front(c, lever.pos, lever.yaw, 0.9));
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.use_exact(Use::Lever(n));
    for _ in 0..120 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!(game.lever_pulled(n));
    assert_eq!(game.gate(g), (1.0, true), "gitteret er ikke oppe");
    assert!(connected(game.crypt_now(), a, b), "ingen vei gjennom det åpne gitteret");
    // Og du kommer gjennom med fysikken.
    game.place_feet(a);
    let d = b - a;
    game.set_look((-d.x).atan2(-d.z), 0.0);
    input.key("KeyW", true);
    for _ in 0..75 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.feet();
    assert!((feet - gate.pos).with_y(0.0).dot(d.with_y(0.0).normalize()) > 0.8, "kom ikke gjennom gitteret: føttene ved {feet:?}");
}

/// Snarveidøra er boltet fra den ene sida, kan åpnes fra sida med slåen, og
/// står åpen så du kan gå gjennom.
#[test]
fn snarveien_apnes_fra_en_side() {
    let c = &world().world.crypt;
    let sc = c.shortcuts.first().expect("ingen snarvei i krypten").clone();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    let snap = |q: Vec3| c.snap(q + Vec3::Y * 0.3, 1.5).unwrap_or(q);
    let latch_side = snap(sc.pos + sc.latch * 1.6);
    let other_side = snap(sc.pos - sc.latch * 1.6);
    game.place_feet(other_side);
    for _ in 0..5 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let events = game.use_exact(Use::Shortcut(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("boltet"))), "døra gikk opp fra feil side: {events:?}");
    assert!(!connected(game.crypt_now(), latch_side, other_side), "det er vei gjennom den stengte døra");
    game.place_feet(latch_side);
    for _ in 0..5 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let events = game.use_exact(Use::Shortcut(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("åpen"))), "døra gikk ikke opp fra slåsida: {events:?}");
    for _ in 0..60 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!(connected(game.crypt_now(), latch_side, other_side), "ingen vei gjennom den åpne døra");
    let d = -sc.latch;
    game.set_look((-d.x).atan2(-d.z), 0.0);
    input.key("KeyW", true);
    for _ in 0..75 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.feet();
    assert!((feet - sc.pos).with_y(0.0).dot(d) > 1.0, "kom ikke gjennom snarveien: føttene ved {feet:?}");
}

/// Den falske veggen: ingen vei forbi før du bruker den, så synker den og du
/// kan gå inn i rommet bak.
#[test]
fn den_falske_veggen_apnes() {
    let c = &world().world.crypt;
    let s = c.secrets.first().expect("ingen falsk vegg i krypten").clone();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    let (front, back) = (in_front(c, s.pos, s.yaw, 1.4), in_front(c, s.pos, s.yaw, -1.4));
    assert!(!connected(game.crypt_now(), front, back), "det er vei gjennom den falske veggen før den er åpnet");
    game.place_feet(front);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let events = game.use_exact(Use::Secret(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("rom bak"))), "veggen ga ikke etter: {events:?}");
    for _ in 0..90 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!(connected(game.crypt_now(), front, back), "ingen vei forbi den åpnede veggen");
    let d = (back - front).with_y(0.0).normalize();
    game.set_look((-d.x).atan2(-d.z), 0.0);
    input.key("KeyW", true);
    for _ in 0..75 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.feet();
    assert!((feet - s.pos).with_y(0.0).dot(d) > 0.8, "kom ikke inn bak veggen: føttene ved {feet:?}");
}

/// Fallemmen eller sjakta: du faller ned i rommet under, og stigen der tar
/// deg opp igjen.
#[test]
fn fallemmen_og_stigen() {
    let c = &world().world.crypt;
    let t = c.trapdoors.iter().find(|t| !t.bolted).expect("ingen fallem eller sjakt i krypten").clone();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    game.place_feet(t.pos + Vec3::Y * 0.3);
    for _ in 0..120 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.feet();
    assert!(feet.y < t.pos.y - 2.0, "falt ikke ned: føttene ved {feet:?}, lemmen ved {:?}", t.pos);
    if let Some(to) = t.to {
        assert_eq!(c.room_at(feet + Vec3::Y * 0.2).map(|r| r.1), Some(to), "havnet ikke i rommet under ({feet:?})");
    }
    let (n, ladder) = c.ladders.iter().enumerate().min_by(|a, b| a.1.pos.distance(feet).total_cmp(&b.1.pos.distance(feet))).expect("ingen stige");
    assert!(c.path(feet + Vec3::Y * 0.2, ladder.pos).is_some(), "ingen vei fra der du landet til stigen");
    game.place_feet(in_front(c, ladder.pos, ladder.yaw, 0.6));
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.use_exact(Use::Ladder(n));
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = game.feet();
    assert!((feet.y - ladder.top.y).abs() < 0.6 && feet.with_y(0.0).distance(ladder.top.with_y(0.0)) < 1.5, "stigen tok deg ikke opp til {:?}: føttene ved {feet:?}", ladder.top);
}

/// Kistene: den sjeldne komponenten ligger i sin kiste, lappene kan leses,
/// og den varige styrken gir mer liv bare én gang.
#[test]
fn kistene_lappene_og_den_varige_styrken() {
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let (i, rare) = c.chests.iter().enumerate().find(|(_, ch)| ch.loot.is_some()).expect("ingen sjelden komponent i krypten");
    let events = game.use_exact(Use::Chest(i));
    assert!(events.iter().any(|e| *e == Event::Loot(rare.loot.clone())), "kista ga ikke {:?}: {events:?}", rare.loot);
    assert!(crate::domain::craft::title(rare.loot.as_deref().unwrap()) != "", "den sjeldne komponenten finnes ikke i spillet");
    let events = game.use_exact(Use::Chest(i));
    assert!(!events.iter().any(|e| matches!(e, Event::Loot(_))), "kista ga to ganger");
    let lore = c.lores.first().expect("ingen lapp i krypten");
    let events = game.use_exact(Use::Lore(0));
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains(&lore.title))), "lappen ble ikke lest: {events:?}");
    let before = game.max_hp_now();
    game.use_exact(Use::Bonus(0));
    let after = game.max_hp_now();
    game.use_exact(Use::Bonus(0));
    assert!(after > before || game.stamina_now() > 100.0, "den varige styrken ga ingenting");
    assert_eq!(game.max_hp_now(), after, "den varige styrken ga to ganger");
    assert!(game.save().bonuses.len() == 1, "styrken skal lagres");
}

/// Pilfella: første gang treffer pilene gulvet foran deg, siden gjør de
/// vondt.
#[test]
fn pilfella_laerer_deg_forst() {
    use crate::domain::world::crypt::TrapKind;
    let c = &world().world.crypt;
    let (_, trap) = c.traps.iter().enumerate().find(|(_, t)| t.kind == TrapKind::Arrows).expect("ingen pilfelle i krypten");
    let trap = trap.clone();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    let off = c.path(trap.pos, c.levels[trap.level].start.0).map(|mut p| {
        p.insert(0, trap.pos);
        along(&p, 3.0)
    }).unwrap();
    game.place_feet(off);
    for _ in 0..10 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let hp = game.hp_now();
    game.place_feet(trap.pos + Vec3::Y * 0.05);
    let mut warned = false;
    for _ in 0..40 {
        for e in game.update(&mut input, 1.0 / 30.0, &mut physics) {
            warned |= matches!(e, Event::Toast(t) if t.contains("Klikk"));
        }
    }
    assert!(warned, "trykkplata ved {:?} klikket ikke", trap.pos);
    assert_eq!(game.hp_now(), hp, "pilene skal treffe gulvet første gang");
    game.place_feet(off);
    for _ in 0..120 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    game.place_feet(trap.pos + Vec3::Y * 0.05);
    for _ in 0..40 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    assert!(game.hp_now() < hp, "pilene gjorde ikke vondt andre gang");
}

/// Lyset følger rommene: et lys lyser i sitt rom og i rom med åpen
/// forbindelse, og et stengt gitter stopper det til det åpnes.
#[test]
fn lyset_folger_rommene() {
    let w = &world().world;
    let c = &w.crypt;
    let lit: Vec<_> = w.lights.iter().filter(|l| l.pos.y < crypt::TOP).collect();
    assert!(!lit.is_empty(), "ingen lys i krypten");
    for l in &lit {
        assert!(l.room.is_some(), "lyset ved {:?} står ikke i noe rom", l.pos);
    }
    let gate = &c.gates[0];
    // Et sted på hver side, utenfor rommet gitteret står i.
    let gate_room = c.room_at(gate.pos + Vec3::Y * 0.2).map(|r| r.1);
    let side = |sgn: f32| {
        (1..12)
            .map(|k| in_front(c, gate.pos, gate.yaw, sgn * 0.6 * k as f32))
            .find_map(|q| c.room_at(q + Vec3::Y * 0.2).map(|r| r.1).filter(|r| Some(*r) != gate_room))
            .expect("ingen rom ved gitteret")
    };
    // Rommet gitteret står i (gangen), eller rommet på andre siden.
    let ra = side(1.0);
    let rb = gate_room.filter(|g| *g != ra).unwrap_or_else(|| side(-1.0));
    assert_ne!(ra, rb, "gitteret står ikke mellom to rom");
    let mut c = c.clone();
    assert_eq!(c.light_mask(gate.level, ra) & (1 << rb), 0, "lyset går gjennom det stengte gitteret");
    c.open[gate.switch] = true;
    assert_ne!(c.light_mask(gate.level, ra) & (1 << rb), 0, "lyset går ikke gjennom det åpne gitteret");
}



/// Måler tiden per `update` og `frame` i byen og i krypten (feilsøking av
/// bildefrekvensen). `cargo test --release maal_tiden -- --ignored --nocapture`.
#[test]
#[ignore]
fn maal_tiden_per_bilde() {
    let mut physics = RapierPhysics::new();
    let mut s = session(&mut physics);
    let mut input = Input::default();
    for (name, descend) in [("byen", false), ("krypten", true)] {
        if descend {
            s.use_first(|u| *u == Use::Descend);
            for _ in 0..60 {
                s.update(&mut input, 1.0 / 60.0, &mut physics);
            }
        }
        let mut upd = 0.0f64;
        let mut frm = 0.0f64;
        let n = 120;
        for _ in 0..n {
            let t = std::time::Instant::now();
            s.update(&mut input, 1.0 / 60.0, &mut physics);
            upd += t.elapsed().as_secs_f64();
            let t = std::time::Instant::now();
            let f = s.frame(16.0 / 9.0);
            frm += t.elapsed().as_secs_f64();
            s.settle();
            std::hint::black_box(f.instances.len());
        }
        println!("{name}: update {:.2} ms, frame {:.2} ms per bilde", upd / n as f64 * 1e3, frm / n as f64 * 1e3);
    }
}

/// Skriver hvor rammene på galleriveggen står (til `snapshot --gallery`):
/// `cargo test --release galleriveggen_star -- --ignored --nocapture`.
#[test]
#[ignore]
fn galleriveggen_star() {
    let p = world();
    for t in p.world.targets.iter().filter(|t| matches!(t.what, Use::Gallery(_))) {
        let (_, rot, pos) = t.pose.to_scale_rotation_translation();
        let front = rot * Vec3::Z;
        println!("{:?}: {:.2} {:.2} {:.2}  front {:.2} {:.2}", t.what, pos.x, pos.y, pos.z, front.x, front.z);
    }
}

/// Trappereglene i byggesettet (`kit.stairs`): du går stående (ikke huk)
/// opp og ned alle tre trappene uten å bli stoppet eller presset ned, og
/// det er minst en meter fritt gulv foran nederste trinn.
#[test]
fn trappene_har_hoderom() {
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.55f32, 0.35f32);
    // Modell, foten (0,5 m foran nederste trinn) og toppen (Blenders
    // akser), gulvet nede og høyden oppe, og retningen bort fra trappa
    // ved foten.
    let stairs = [("tavern", (5.15, -3.55), (-0.3, -3.55), 0.6, 3.7, (1.0, 0.0)), ("apothecary", (3.1, -2.65), (3.1, 1.9), 0.7, 3.3, (0.0, -1.0)), ("scriptorium", (-3.875, -2.85), (-3.875, 2.2), 0.36, 2.8, (0.0, -1.0))];
    let mut problems = vec![];
    for (model, foot, top, floor, up, away) in stairs {
        let Some(pl) = p.world.placements.iter().find(|pl| pl.model == model) else {
            problems.push(format!("{model} mangler"));
            continue;
        };
        let t = pl.transforms[0];
        let world = |(x, y): (f32, f32)| t.transform_point3(Vec3::new(x, 0.0, -y));
        let base = t.transform_point3(Vec3::ZERO).y;
        let (a, b) = (world(foot), world(top));
        // Opp og ned, stående.
        for (from, to, z_from, z_to, way) in [(a, b, floor, up, "opp"), (b, a, up, floor, "ned")] {
            let mut pos = from + Vec3::Y * (z_from + half + radius + 0.05);
            let dir = (to - from).with_y(0.0).normalize();
            let (mut stuck, mut top_feet) = (0, f32::MIN);
            for _ in 0..600 {
                if (to - pos).with_y(0.0).dot(dir) < 0.05 {
                    break;
                }
                let (moved, _) = physics.move_capsule(pos, half, radius, dir * 0.05 - Vec3::Y * 0.03, 0.016);
                pos += moved;
                top_feet = top_feet.max(pos.y - half - radius - base);
                stuck = if moved.with_y(0.0).length() < 0.01 { stuck + 1 } else { 0 };
                if stuck > 20 {
                    break;
                }
            }
            let left = (pos - to).with_y(0.0).length();
            let feet = pos.y - half - radius - base;
            if stuck > 20 && left > 0.6 {
                problems.push(format!("{model} {way}: stoppet med føttene {feet:.2} m over bakken, {left:.1} m fra målet (for lavt under taket?)"));
            } else if left > 0.6 || (way == "opp" && feet < up) || (way == "ned" && feet > z_to + 0.3) {
                problems.push(format!("{model} {way}: endte {left:.1} m fra målet med føttene {feet:.2} m over bakken"));
            }
            if way == "ned" && top_feet > up + 0.5 {
                problems.push(format!("{model} {way}: ble løftet til {top_feet:.2} m"));
            }
        }
        // Fritt gulv foran nederste trinn: stråler bort fra trappa i kne-,
        // hofte- og hodehøyde treffer ingenting innen en meter.
        let dir = t.transform_vector3(Vec3::new(away.0, 0.0, -away.1)).normalize();
        for h in [0.4, 1.0, 1.6] {
            let origin = world(foot) - dir * 0.5 + Vec3::Y * (floor + h);
            // Veggkolliderne er 5 cm tykkere enn pussen (`kit.WALL_PAD`).
            if let Some(hit) = physics.cast_ray(origin, dir, 0.98) {
                problems.push(format!("{model}: noe står {:.2} m foran nederste trinn i {h} m høyde", hit.distance));
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Gorveks øvingsplass har en åpning mot gata: man går med Rapier fra
/// Storgata inn til midten av plassen uten å stoppe mot noe.
#[test]
fn man_kan_ga_til_gorvek() {
    use crate::domain::world::town::{main_x, YARD};
    let p = world();
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.55f32, 0.35f32);
    let (yc, _) = YARD;
    let from = Vec3::new(main_x(yc.y), 0.0, yc.y);
    let to = Vec3::new(yc.x, 0.0, yc.y);
    let ground = |q: Vec3| Vec3::new(q.x, crate::domain::world::terrain::height(q.x, q.z), q.z);
    let (a, b) = (ground(from), ground(to));
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
    assert!(left <= 0.6, "stoppet {left:.1} m fra midten av Gorveks plass, ved ({:.1}, {:.1})", pos.x, pos.z);
    assert!(p.world.targets.iter().any(|t| matches!(t.what, Use::Talk(_)) && t.pose.translation.distance(b.into()) < 6.0), "ingen å snakke med på plassen");
}

/// Sperringene i byen er synlige: hver gjerdekollider (`FENCE_SOLID_H` høy,
/// satt av `town::fence`) er dekket av en modellinstans (gjerde eller
/// steinmur) langs hele lengden, prøvd hver 0,5 m med 0,3 m slingring.
#[test]
fn sperringene_er_synlige() {
    use crate::domain::world::town::FENCE_SOLID_H;
    let p = world();
    let bounds: std::collections::HashMap<&str, (Vec3, Vec3)> = p
        .models
        .iter()
        .map(|m| {
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            for v in &m.mesh.vertices {
                lo = lo.min(Vec3::from(v.pos));
                hi = hi.max(Vec3::from(v.pos));
            }
            (m.name.as_str(), (lo, hi))
        })
        .collect();
    let covered = |q: Vec3| {
        p.world.placements.iter().any(|pl| {
            let Some((lo, hi)) = bounds.get(pl.model) else { return false };
            pl.transforms.iter().any(|t| {
                if t.translation.distance(q.into()) > 16.0 {
                    return false;
                }
                let l = t.inverse().transform_point3(q);
                (l.x > lo.x - 0.3 && l.x < hi.x + 0.3) && (l.z > lo.z - 0.3 && l.z < hi.z + 0.3) && (l.y > lo.y - 0.5 && l.y < hi.y + 0.5)
            })
        })
    };
    let (mut fences, mut problems) = (0, vec![]);
    for c in &p.world.colliders {
        let Shape::Cuboid { center, size, yaw } = c else { continue };
        if size.y != FENCE_SOLID_H || center.y < -5.0 {
            continue;
        }
        fences += 1;
        let along = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let n = (size.x / 0.5).ceil().max(1.0) as i32;
        for k in 0..=n {
            let q = *center + along * (size.x * (k as f32 / n as f32 - 0.5));
            if !covered(q) {
                problems.push(format!("usynlig sperring ved ({:.1}, {:.1})", q.x, q.z));
                break;
            }
        }
    }
    assert!(fences > 20, "fant bare {fences} gjerdekollidere");
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Hver modell har rollene sine som data (`extras.fristad`, skrevet av
/// `tools/models/slim.py`), og lastes uten feil: et materiale uten roller,
/// med et ukjent slag eller et teksturlag som ikke finnes, stopper lastingen.
#[test]
fn modellene_har_roller() {
    use crate::ports::ModelSource;
    let bad: Vec<String> = Embedded.names().into_iter().filter(|n| Embedded.load(n).is_none()).collect();
    assert!(bad.is_empty(), "modeller som ikke kan lastes (se loggen): {bad:?}");
}

/// Murverket i krypten tegnes helt også langt unna: det grove detaljnivået
/// slår sammen hjørner og åpnet sprekker med himmel i skjøtene (eieren 13.
/// oktober). Kryptbitene er lette nok til at de aldri får et (`lod::far_detail`,
/// `LOD_MIN_TRIS`); `snapshot --leak-scan` finner det som synes gjennom.
#[test]
fn kryptmurene_forenkles_ikke_pa_avstand() {
    let p = world();
    let heavy: Vec<String> = p
        .models
        .iter()
        .filter(|m| m.name.starts_with("crypt_") && m.far_mesh.is_none() && crate::adapters::wgpu::lod::far_detail(&m.mesh, 0.2).is_some())
        .map(|m| format!("{} ({} trekanter)", m.name, m.mesh.indices.len() / 3))
        .collect();
    assert!(heavy.is_empty(), "kryptbiter som ville fått sprekker på avstand: {heavy:?}");
}

/// Hvert teksturlag en modell bruker, har et bilde i `assets/external`
/// (`bun tools/fetch-assets.ts`). Et lag uten bilde faller ellers stille
/// tilbake til det prosedyriske laget (hvite dører, 11. okt.).
#[test]
fn alle_lagene_har_bilde() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/external");
    let have = crate::adapters::layers::from_dir(&dir);
    let missing = crate::adapters::layers::missing(&have, &crate::adapters::gltf::used_layers());
    assert!(missing.is_empty(), "teksturlag uten bilde: {missing:?} (kjør `bun tools/fetch-assets.ts`)");
}

/// En modell byen eller systemene bruker, finnes, eller den står i
/// `model::PLANNED` (fiendene, figurene, våpnene …). Og en modell som står
/// der, men nå finnes, skal strykes fra lista.
#[test]
fn alle_modellnavn_finnes_eller_ventes() {
    use crate::domain::model::PLANNED;
    let missing = &world().world.missing;
    let unknown: Vec<_> = missing.iter().filter(|m| !PLANNED.iter().any(|(p, _)| p == *m)).collect();
    assert!(unknown.is_empty(), "modeller som brukes, men verken finnes eller ventes: {unknown:?}");
    let done: Vec<_> = PLANNED.iter().map(|(p, _)| *p).filter(|p| !missing.contains(p)).collect();
    assert!(done.is_empty(), "modeller i PLANNED som finnes nå (eller ikke brukes lenger), stryk dem: {done:?}");
}

/// Blender-skriptene henger sammen (`tools/models/check_scripts.py`): hver
/// fil kompilerer, og hvert `town.X`, `kit.X` … finnes i modulen. Ellers
/// viser feilen seg først ved neste eksport.
#[test]
fn blenderskriptene_henger_sammen() {
    let root = env!("CARGO_MANIFEST_DIR");
    let out = std::process::Command::new("python3").arg(format!("{root}/tools/models/check_scripts.py")).output().expect("python3");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
}

/// Dommen mister ingenting (eieren 13. okt.): hver dom gjelder versjonen
/// eieren så, hver ting i katalogen har en fil, og alt som venter på eieren
/// får plass på siden (`tools/qa/review.py`, `audit`).
#[test]
fn dommen_mister_ingenting() {
    let root = env!("CARGO_MANIFEST_DIR");
    let out = std::process::Command::new("python3").args(["-m", "tools.qa", "approvals", "audit"]).current_dir(root).output().expect("python3");
    assert!(out.status.success(), "{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
}

/// Katalogen (`docs/katalog.md`, `python3 -m tools.qa catalog`) dekker alt:
/// hver modell spillet har, står i `assets/catalog/models.json` med hva den
/// er, hvert teksturlag og hvert malt bilde i `textures.json`, og eierens dom
/// i `approvals.json` gjelder noe som finnes.
#[test]
fn katalogen_dekker_alt() {
    let root = env!("CARGO_MANIFEST_DIR");
    let read = |f: &str| -> serde_json::Value { serde_json::from_str(&std::fs::read_to_string(format!("{root}/assets/catalog/{f}")).expect(f)).expect(f) };
    let models = read("models.json");
    let textures = read("textures.json");
    let approvals = read("approvals.json");
    let m = models["models"].as_object().expect("models");
    let t = textures["textures"].as_object().expect("textures");
    let mut bad = vec![];
    for (name, _) in crate::adapters::gltf::model_files() {
        if !m.contains_key(*name) {
            bad.push(format!("modellen {name} mangler i models.json"));
        }
    }
    let dirs = |d: &str, file: &str| -> Vec<String> {
        std::fs::read_dir(format!("{root}/{d}")).expect(d).flatten().filter(|e| e.path().join(file).exists()).map(|e| e.file_name().to_string_lossy().into_owned()).collect()
    };
    for name in dirs("assets/textures/painted", "albedo.png").into_iter().chain(dirs("assets/textures", "albedo.jpg")) {
        if !t.contains_key(&name) {
            bad.push(format!("teksturen {name} mangler i textures.json"));
        }
    }
    for (kind, list) in [("models", m), ("textures", t)] {
        for name in approvals[kind].as_object().expect("approvals").keys() {
            if !list.contains_key(name) {
                bad.push(format!("approvals.json dømmer {kind}/{name}, som ikke finnes"));
            }
        }
    }
    assert!(bad.is_empty(), "katalogen er ikke komplett:\n{}", bad.join("\n"));
}

/// Byen er data (`assets/layout/town.json`, `docs/redigering.md`): spillet
/// bygger den fra fila, med husene, gjerdene, muren og innbyggerne, og fila
/// står i fast form (`layout_json`), så det redigeringsmodusen lagrer, gir
/// små endringer i git.
#[test]
fn byen_er_data() {
    use crate::adapters::data::{layout_file, layout_json};
    use crate::domain::world::layout::Layout;
    let text = layout_file().expect("assets/layout/town.json mangler");
    let layout: Layout = serde_json::from_str(text).unwrap();
    assert_eq!(layout_json(&layout), text, "fila står ikke i fast form: cargo run --bin layout");
    // Husenes innhold i samme faste form (redigeringen lagrer slik).
    for (house, text) in crate::adapters::data::content_files() {
        let items: Vec<crate::domain::world::layout::Item> = serde_json::from_str(text).unwrap();
        assert_eq!(crate::adapters::data::contents_json(&items), *text, "assets/layout/inside/{house}.json står ikke i fast form: cargo run --bin layout");
    }
    let w = &world().world;
    let count = |m: &str| w.placements.iter().find(|p| p.model == m).map_or(0, |p| p.transforms.len());
    assert_eq!(layout.items.iter().filter(|i| i.building).count(), 71, "husene og uthusene i fila");
    assert_eq!(count("tavern"), 1);
    assert!(count("town_wall") > 40 && count("wattle_fence") > 100, "muren og gjerdene står");
    assert_eq!(w.npcs.len(), layout.npcs.len());
    assert!(w.targets.iter().any(|t| t.what == Use::Portal) && w.targets.iter().filter(|t| matches!(t.what, Use::Well(_))).count() == 2);
}

/// Møblene er egne modeller (`docs/redigering.md`, steg 2): det som står
/// inne i husene (`assets/layout/inside/<hus>.json`) er modeller som finnes,
/// i `assets/models/furniture/`, og ingen har samme navn som en annen modell
/// (da ville den ene skjule den andre). Senga i spillerens hus har med seg
/// det du sikter på for å sove.
#[test]
fn moblene_er_egne_modeller() {
    use crate::adapters::gltf::model_files;
    let data = crate::adapters::data::world_data();
    let mut bad = vec![];
    let mut seen = std::collections::HashMap::new();
    for (name, path) in model_files() {
        if let Some(other) = seen.insert(*name, *path) {
            bad.push(format!("{name} er både {other} og {path}"));
        }
    }
    for (house, items) in &data.contents {
        for m in items.iter().filter_map(|i| i.model.as_deref()) {
            match model_files().find(|(n, _)| *n == m) {
                Some((_, path)) if path.starts_with("furniture/") => {}
                Some((_, path)) => bad.push(format!("{m} i {house} er {path}, ikke et møbel")),
                None => bad.push(format!("{m} i {house} finnes ikke")),
            }
        }
    }
    assert!(data.contents.len() >= 6, "husene med innside mangler møbler");
    assert!(bad.is_empty(), "\n{}", bad.join("\n"));
    let w = &world().world;
    assert_eq!(w.targets.iter().filter(|t| t.what == Use::Sleep).count(), 1, "senga har med seg siktemålet");
}

/// Hvert møbel står i sitt eget rom med foten i origo: ingenting under
/// gulvet, og kolliderne ligger der møbelet er. Fanger feil som tønnene som
/// havnet 3 m over kollideren sin (`kit.barrel`, runde 5).
#[test]
fn moblene_star_der_kolliderne_er() {
    use crate::adapters::gltf::model_files;
    use crate::ports::ModelSource;
    let data = crate::adapters::data::world_data();
    let mut bad = vec![];
    for (name, path) in model_files().filter(|(_, p)| p.starts_with("furniture/")) {
        let Some(m) = Embedded.load(name) else { continue };
        let (lo, hi) = m.mesh.bounds();
        if lo.y < -0.05 {
            bad.push(format!("{name} ({path}) går {:.0} cm under gulvet", -lo.y * 100.0));
        }
        let Some((_, inside)) = data.insides.iter().find(|(n, _)| n == name) else { continue };
        for (c, size, _) in &inside.boxes {
            let (c, h) = (Vec3::from(*c), Vec3::from(*size) * 0.5);
            let far = (0..3).any(|k| c[k] + h[k] < lo[k] - 0.05 || c[k] - h[k] > hi[k] + 0.05);
            if far {
                bad.push(format!("{name}: kollideren ved {c:.2} ligger utenfor møbelet ({lo:.2} .. {hi:.2})"));
            }
        }
    }
    assert!(bad.is_empty(), "\n{}", bad.join("\n"));
}
