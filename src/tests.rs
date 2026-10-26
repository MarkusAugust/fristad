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

pub(crate) fn session(physics: &mut RapierPhysics) -> Session {
    session_in(world(), physics)
}

pub(crate) fn session_in(p: &Prepared, physics: &mut RapierPhysics) -> Session {
    Session::new(&p.world, physics, &rigs(&p.models))
}

/// Byen med prøvenivået (`assets/levels/nivaa_test.json`, kompilert av
/// `crypt::compose`) som eneste kryptnivå: de gamle testene over krypten
/// kjører på det også (`docs/nivaaredigering.md`, fase 1).
pub(crate) fn doc_world() -> &'static Prepared {
    static P: OnceLock<Prepared> = OnceLock::new();
    P.get_or_init(|| prepare(&DocSource("test"), 7))
}

/// De innebygde modellene, med et nivådokument som eneste kryptnivå.
pub(crate) struct DocSource(pub &'static str);

impl crate::ports::ModelSource for DocSource {
    fn load(&self, name: &str) -> Option<crate::domain::model::Model> {
        Embedded.load(name)
    }
    fn names(&self) -> Vec<String> {
        Embedded.names()
    }
    fn world_data(&self) -> crate::domain::world::Data {
        crate::adapters::data::world_data_with_doc(self.0)
    }
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
    let covers = |shape: &Shape, q: Vec3| shape.covers_shaped(q, 0.35);
    // Bitene av kryptnivåene er selve rommene, med murene som kollidere.
    let level_piece = |name: &str| name.strip_prefix("crypt_").is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()));
    let skip = |name: &str| level_piece(name) || name.ends_with("_interior") || name.ends_with("_leaves");
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
    // Midten av hver fast kollider (boks og sylinder), etter id.
    let mut middles = std::collections::HashMap::new();
    for c in &p.world.colliders {
        let id = physics.add(c);
        match c {
            Shape::Cuboid { center, .. } | Shape::Cylinder { center, .. } => {
                middles.insert(id, *center);
            }
            _ => {}
        }
    }
    let mut ids = std::collections::HashMap::new();
    for (i, t) in p.world.targets.iter().enumerate() {
        ids.insert(physics.add_movable(t.half, t.pose * Affine3A::from_translation(t.center)), i);
    }
    physics.move_capsule(Vec3::new(0.0, 50.0, 0.0), 0.5, 0.3, Vec3::ZERO, 0.016);
    let mut problems = vec![];
    for (i, t) in p.world.targets.iter().enumerate() {
        let frame = t.pose * Affine3A::from_translation(t.center);
        let center = frame.transform_point3(Vec3::ZERO);
        // Forfra (innbyggerne fra +x) må strålen treffe målet.
        let facing = match t.what {
            Use::Talk(_) => Vec3::X,
            _ => t.pose.transform_vector3(Vec3::Z).normalize_or_zero(),
        };
        let depth = if matches!(t.what, Use::Talk(_)) { t.half.x } else { t.half.z };
        let from = center + facing * (depth + 0.6);
        let hit = physics.cast_ray(from, -facing, 2.0);
        if hit.as_ref().and_then(|h| ids.get(&h.collider)) != Some(&i) {
            problems.push(format!("{:?} ved ({:.1}, {:.1}, {:.1}) kan ikke siktes på forfra", t.what, center.x, center.y, center.z));
            continue;
        }
        // Fra alle fire kanter, både vannrett og ovenfra på skrå (øyet 1,7 m
        // over foten av målet, 1,5 m unna, mot øvre del av målet): målet kan
        // stå inntil en mur, men tingens egen kollider skal aldri skygge for
        // siktet (relikvieskrinet, eieren 18. oktober). Tingens egen
        // kollider er den som har midten sin i målet.
        let to_local = frame.inverse();
        let inside = |q: Vec3, pad: f32| {
            let l = to_local.transform_point3(q);
            l.x.abs() <= t.half.x + pad && l.y.abs() <= t.half.y + pad && l.z.abs() <= t.half.z + pad
        };
        let foot = frame.transform_point3(-Vec3::Y * t.half.y);
        let upper = frame.transform_point3(Vec3::Y * t.half.y * 0.85);
        for (side_name, yaw) in [("forfra", 0.0f32), ("fra venstre", 90.0), ("bakfra", 180.0), ("fra høyre", 270.0)] {
            let side = glam::Quat::from_rotation_y(yaw.to_radians()) * facing;
            let reach = t.half.x.max(t.half.z) + 0.6;
            let eye = foot + Vec3::Y * 1.7 + side * (reach + 0.9);
            let rays = [("vannrett", center + side * reach, -side), ("ovenfra", eye, (upper - eye).normalize())];
            for (how, from, dir) in rays {
                if inside(from, 0.05) {
                    continue;
                }
                let Some(h) = physics.cast_ray(from, dir, 4.0) else { continue };
                if middles.get(&h.collider).is_some_and(|m| inside(*m, 0.0)) {
                    problems.push(format!("{:?} ved ({:.1}, {:.1}, {:.1}): en kollider skygger for siktet {side_name} {how}", t.what, center.x, center.y, center.z));
                }
            }
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

/// Målet på skjermen og journalen: i byen før du har vært nede sier målet
/// at krypten er ut nordporten; nede sier det hvilket nivå du står i og hva
/// som gjenstår; lappen du leser havner i journalen, og det dypeste nivået
/// og lappene huskes fra en lagring.
#[test]
fn malet_og_journalen_viser_veien_og_huskes() {
    fn goal(events: &[Event]) -> Option<(String, usize)> {
        events.iter().rev().find_map(|e| if let Event::Goal { text, journal } = e { Some((text.clone(), journal.len())) } else { None })
    }
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let (text, read) = goal(&game.update(&mut input, 1.0 / 30.0, &mut physics)).expect("målet sendes første bilde");
    assert!(text.contains("nordporten") && text.contains("krypten"), "før du har vært nede: {text}");
    assert_eq!(read, 0);
    assert!(goal(&game.update(&mut input, 1.0 / 30.0, &mut physics)).is_none(), "målet sendes bare når det endres");
    game.use_first(|u| *u == Use::Descend);
    let (text, _) = goal(&game.update(&mut input, 1.0 / 30.0, &mut physics)).expect("målet endres nede");
    assert!(text.starts_with("Nivå 1"), "nede: {text}");
    // En lapp havner i journalen.
    let lore = !game.use_first(|u| matches!(u, Use::Lore(_))).is_empty();
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    if lore {
        let (_, read) = goal(&events).expect("journalen sendes når du har lest");
        assert_eq!(read, 2, "Ansgars lapp fra første gang ned og lappen du leste");
    }
    let saved = game.save();
    assert_eq!(saved.deepest, Some(0));
    assert_eq!(saved.lore.len(), 1 + lore as usize);
    // Ei ny økt fra lagringen husker nivået og lappen.
    let mut physics2 = RapierPhysics::new();
    let mut again = session(&mut physics2);
    again.load(saved);
    assert_eq!(again.deepest(), Some(0));
    assert_eq!(again.journal().len(), 1 + lore as usize);
    assert!(again.goal().contains("Dypeste nivå"), "{}", again.goal());
}

/// Hvilestedet lagrer: når du hviler, sendes lageret, og nettlaget lagrer
/// spillet da (`case "inventory"` i `web/main.js`).
#[test]
fn hvilestedet_lagrer() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..3 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let quiet = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(!quiet.iter().any(|e| matches!(e, Event::Inventory(_))), "lageret sendes ikke uten grunn");
    game.use_first(|u| matches!(u, Use::Rest(_)));
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(events.iter().any(|e| matches!(e, Event::Inventory(_))), "hvilen skal sende lageret, så spillet lagres");
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
    // Kort på én linje til skjemaet, og hver ting har sin linje om seg selv.
    assert_eq!(button.brief().lines().count(), 1);
    assert!(button.brief().contains("Buttonsmeden"), "{}", button.brief());
    let ids = craft::WEAPONS.iter().map(|w| w.id).chain(craft::SHIELDS.iter().map(|s| s.id)).chain(craft::ARMORS.iter().map(|a| a.id)).chain(craft::RECIPES.iter().map(|r| r.id));
    let silent: Vec<&str> = ids.filter(|id| craft::about(id).is_empty()).collect();
    assert!(silent.is_empty(), "uten beskrivelse: {silent:?}");
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


/// Utstyret (eieren 18. oktober: «alt skal se forskjellig ut med forskjellige
/// stats»): hvert våpen, skjold og hver rustning har sin egen modell, som
/// finnes, og sine egne tall; alt utenom det du starter med kan smis, og
/// ikonet til kortet finnes. Det du har smidd, holder økta på og lagrer, og
/// en gammel lagring uten lista får det du har på.
#[test]
fn utstyret_ser_forskjellig_ut_og_kan_byttes() {
    use crate::domain::craft;
    let root = env!("CARGO_MANIFEST_DIR");
    let files: Vec<&str> = crate::adapters::gltf::model_files().map(|(n, _)| *n).collect();
    let mut models = vec![];
    let mut bad = vec![];
    let items: Vec<(&str, &str, Vec<f32>)> = craft::WEAPONS
        .iter()
        .map(|w| (w.id, w.model, vec![w.damage, w.speed, w.reach, w.riposte, w.stamina]))
        .chain(craft::SHIELDS.iter().map(|s| (s.id, s.model, vec![s.block, s.parry_window, s.stamina])))
        .chain(craft::ARMORS.iter().map(|a| (a.id, a.arms, vec![a.taken, a.regen, a.dodge])))
        .collect();
    for (id, model, stats) in &items {
        if !files.contains(model) {
            bad.push(format!("{id}: modellen {model} finnes ikke"));
        }
        if models.contains(model) {
            bad.push(format!("{id}: modellen {model} er brukt av noe annet"));
        }
        models.push(*model);
        if items.iter().any(|(o, _, s)| o != id && s == stats) {
            bad.push(format!("{id}: samme tall som noe annet"));
        }
        let starter = *id == craft::RUSTY_SWORD.id || *id == craft::OLD_SHIELD.id;
        let made = craft::RECIPES.iter().any(|r| matches!(r.result, craft::Result::Weapon(x) | craft::Result::Shield(x) | craft::Result::Armor(x) if x == *id));
        if !starter && !made {
            bad.push(format!("{id}: ingen oppskrift"));
        }
        if !std::path::Path::new(&format!("{root}/assets/textures/icons/{id}.png")).exists() {
            bad.push(format!("{id}: ikonet mangler"));
        }
    }
    assert!(bad.is_empty(), "utstyret:\n{}", bad.join("\n"));
    assert!(craft::WEAPONS.len() >= 5 && craft::SHIELDS.len() >= 5 && craft::ARMORS.len() >= 5);

    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    for (c, n) in [("button", 3), ("divider", 1), ("tag", 3)] {
        game.give(c, n);
    }
    assert!(game.craft("component_sword") && game.craft("tag_dagger"));
    assert_eq!(game.gear().weapon.id, "tag_dagger");
    assert!(!game.craft("tag_dagger"), "det du har, smis ikke på nytt");
    assert!(game.equip("component_sword"), "det du har smidd, kan du ta på igjen");
    assert!(!game.equip("tabs_axe"), "det du ikke har, kan du ikke ta på");
    let saved = game.save();
    assert!(saved.owned.iter().any(|o| o == "tag_dagger"));
    let mut physics2 = RapierPhysics::new();
    let mut fresh = session(&mut physics2);
    fresh.load(saved.clone());
    assert!(fresh.equip("tag_dagger"), "lagringen husker alt utstyret");
    let mut old = session(&mut physics2);
    old.load(SaveData { owned: vec![], ..saved });
    assert_eq!(old.gear().weapon.id, "component_sword");
    assert!(old.equip("rusty_sword") && !old.equip("tag_dagger"), "en gammel lagring har det du hadde på og det du startet med");
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

/// Trubaduren sitter på krakken ved ildstedet i Vertshuset (plassen er
/// data: `seated` i `assets/layout/town.json`, krakken i
/// `assets/layout/inside/tavern.json`), vendt ut i rommet, og figuren hans
/// (TRELLIS, `folk3d.py troubadour`) har beinet lutten henger i.
#[test]
fn trubaduren_sitter_ved_ildstedet() {
    let w = &world().world;
    let t = w.npcs.iter().find(|n| n.name == "Trubaduren").expect("trubaduren i town.json");
    assert!(t.seated && t.route.is_empty(), "trubaduren sitter og går ingen steder");
    let stools: Vec<Vec3> = w.placements.iter().filter(|p| p.model == "stool").flat_map(|p| p.transforms.iter().map(|a| Vec3::from(a.translation))).collect();
    let stool = stools.iter().min_by(|a, b| a.distance(t.pos).total_cmp(&b.distance(t.pos))).expect("krakker");
    assert!(stool.distance(t.pos) < 0.05, "trubaduren sitter ikke på en krakk: {:?} mot {stool:?}", t.pos);
    let hearth = w.lights.iter().filter(|l| l.always).map(|l| l.pos).min_by(|a, b| a.distance(t.pos).total_cmp(&b.distance(t.pos))).unwrap();
    assert!(hearth.distance(t.pos) < 2.5, "krakken står ikke ved ildstedet ({hearth:?})");
    assert_eq!(t.model, "npc_troubadour");
    let figure = world().models.iter().find(|m| m.name == t.model).expect("figuren til trubaduren finnes");
    assert!(figure.rig.as_ref().is_some_and(|r| r.joint("lute").is_some() && r.clip("play").is_some() && r.clip("sit").is_some() && r.joint("jaw").is_some()), "figuren har lutebeinet, kjeven og de sittende klippene");
    assert!(world().models.iter().any(|m| m.name == crate::domain::npcs::LUTE_MODEL), "lutten finnes");
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
/// Bitene til kryptnivåene hentes først når økta ber om nivået (plan del
/// 10a): de settes ut likevel, og `tools/pack-models.sh` legger dem i en
/// mappe per nivå (`delivery::dist_path`), med størrelsene per nivå i sizes.json.
#[test]
fn nettleseren_far_de_samme_modellene() {
    use crate::adapters::delivery::{dist_path, level_of};
    use crate::adapters::gltf::{model_files, Files};
    use crate::ports::ModelSource;
    let read = |p: &str| std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).expect(p);
    let all = Files(model_files().map(|(_, p)| (p.to_string(), read(p))).collect());
    let web = prepare(&all, 7);
    let native = world();
    let names = |p: &Prepared| p.models.iter().map(|m| (m.name.clone(), m.mesh.indices.len())).collect::<Vec<_>>();
    assert_eq!(names(&web), names(native));
    assert_eq!(web.world.placements.len(), native.world.placements.len());

    // Ved start: alt utenom bitene. De står ute som før, men lastes ikke.
    let start = Files(model_files().filter(|(_, p)| level_of(p).is_none()).map(|(_, p)| (p.to_string(), read(p))).collect());
    let pieces: Vec<String> = native.world.crypt.levels.iter().flat_map(|l| l.pieces.clone()).collect();
    assert!(!pieces.is_empty());
    let mut deferred: Vec<String> = start.deferred().into_iter().map(|d| d.info.name).collect();
    deferred.sort();
    let mut want = pieces.clone();
    want.sort();
    assert_eq!(deferred, want, "bitene som venter er bitene i nivåene");
    let lazy = prepare(&start, 7);
    assert_eq!(lazy.world.placements.len(), native.world.placements.len(), "bitene står ute");
    assert!(lazy.models.iter().all(|m| !pieces.contains(&m.name)));
    assert_eq!(lazy.models.len() + pieces.len(), native.models.len());

    // Pakkingen: en mappe per nivå, og størrelsene per nivå.
    let out = std::env::temp_dir().join(format!("fristad-pack-{}", std::process::id()));
    let status = std::process::Command::new("sh").arg("tools/pack-models.sh").arg(&out).current_dir(env!("CARGO_MANIFEST_DIR")).status().expect("sh");
    assert!(status.success());
    let sizes: serde_json::Value = serde_json::from_slice(&std::fs::read(out.join("models/sizes.json")).unwrap()).unwrap();
    for (_, p) in model_files() {
        let file = out.join(format!("models/{}.gz", dist_path(p)));
        assert!(file.exists(), "{} mangler", file.display());
        match level_of(p) {
            Some(n) => {
                assert!(dist_path(p).starts_with(&format!("crypt/{n:02}/")), "{p}");
                assert!(sizes["levels"][n.to_string()][p].is_u64(), "{p} mangler under nivå {n} i sizes.json");
                assert!(sizes[p].is_null(), "{p} skal bare stå under nivået");
            }
            None => assert!(sizes[p].is_u64(), "{p} mangler i sizes.json"),
        }
    }
    let _ = std::fs::remove_dir_all(&out);
}

/// En renderer som bare husker modellene den har.
#[derive(Default)]
pub(crate) struct Counting {
    pub(crate) models: Vec<String>,
}

impl crate::ports::Renderer for Counting {
    fn render(&mut self, _frame: &crate::domain::scene::Frame) {}
    fn resize(&mut self, _width: u32, _height: u32) {}
    fn set_quality(&mut self, _quality: crate::ports::Quality) {}
    fn triangles(&self) -> u32 {
        0
    }
    fn add_models(&mut self, models: &[crate::domain::model::Model]) {
        self.models.extend(models.iter().map(|m| m.name.clone()));
    }
    fn drop_models(&mut self, names: &[String]) {
        self.models.retain(|m| !names.contains(m));
    }
}

/// Økta ber om kryptnivåene før du går ned (`Event::FetchLevel`): det
/// første oppe i byen, og nivåene rundt deg nede. Rendereren får bitene
/// med `add_level` og slipper nivåer du har forlatt.
#[test]
fn kryptnivaaene_hentes_nivaa_for_nivaa() {
    use crate::adapters::delivery::{level_files, level_of};
    use crate::adapters::gltf::{model_files, Files};
    use crate::application::app::Pieces;
    use crate::application::App;
    use crate::ports::driving::Play;
    let read = |p: &str| std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).expect(p);
    let start = Files(model_files().filter(|(_, p)| level_of(p).is_none()).map(|(_, p)| (p.to_string(), read(p))).collect());
    let prepared = prepare(&start, 7);
    let levels: Vec<(Vec3, Vec<String>)> = prepared.world.crypt.levels.iter().map(|l| (l.start.0, l.pieces.clone())).collect();
    assert!(levels.len() >= 3);
    let mut app = App::new(prepared, Counting::default(), RapierPhysics::new());
    let asked = |app: &mut App<Counting, RapierPhysics>| -> Vec<u32> {
        let mut out = vec![];
        for _ in 0..3 {
            out.extend(app.tick(1.0 / 30.0).into_iter().filter_map(|e| if let Event::FetchLevel(n) = e { Some(n) } else { None }));
        }
        out
    };
    let load = |app: &mut App<Counting, RapierPhysics>, n: u32| {
        let models: Vec<_> = level_files(n).filter_map(|(name, p)| crate::adapters::gltf::load(name, &[("model.glb", &read(p))], -1.0, 0.0).ok()).collect();
        app.add_level(n, &models);
    };
    // Oppe i byen: bare det første nivået.
    assert_eq!(asked(&mut app), vec![1]);
    load(&mut app, 1);
    assert_eq!(app.level_pieces()[0], Pieces::Loaded);
    assert!(levels[0].1.iter().all(|p| app.renderer.models.contains(p)));
    // Nede i nivå 2: nivået over er der, nivå 2 og 3 hentes.
    app.session.place_feet(levels[1].0);
    assert_eq!(asked(&mut app), vec![2, 3]);
    load(&mut app, 2);
    load(&mut app, 3);
    // Opp i byen igjen: nivå 2 og 3 slippes, nivå 1 blir.
    app.session.place_feet(crate::domain::world::town::square_center());
    assert!(asked(&mut app).is_empty());
    assert_eq!(app.level_pieces()[1], Pieces::Missing);
    assert_eq!(app.level_pieces()[2], Pieces::Missing);
    assert!(levels[1].1.iter().all(|p| !app.renderer.models.contains(p)), "nivå 2 er sluppet");
    assert!(levels[0].1.iter().all(|p| app.renderer.models.contains(p)), "nivå 1 blir");

    // Utenom nettleseren er alt lastet, og ingenting bes om eller slippes.
    let mut native = App::new(prepare(&Embedded, 7), Counting::default(), RapierPhysics::new());
    assert!(native.level_pieces().iter().all(|p| *p == Pieces::Always));
    native.session.place_feet(levels[1].0);
    assert!(asked(&mut native).is_empty());
}

/// I nettleseren får teksturene et tak (armene, våpnene og troféene 2048, resten
/// 1024, normal- og metallkartene halvparten), og de pakkes ut først når
/// rendereren laster dem opp (`gltf::pixels`), så de aldri står utpakket
/// samtidig.
#[test]
fn teksturene_har_tak_i_nettleseren() {
    use crate::adapters::delivery::browser_caps;
    use crate::adapters::gltf::{load_capped, model_files, pixels};
    for name in ["arms", "npc_smith", "tavern", "gallery_card", "framework"] {
        let Some((_, p)) = model_files().find(|(n, _)| *n == name) else { continue };
        let bytes = std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let caps = browser_caps(name);
        let m = load_capped(name, &[("model.glb", &bytes)], -1.0, 0.0, caps).unwrap();
        let full = crate::adapters::gltf::load(name, &[("model.glb", &bytes)], -1.0, 0.0).unwrap();
        for (img, orig, cap) in [(&m.albedo, &full.albedo, caps.color), (&m.normal, &full.normal, caps.data), (&m.metal_rough, &full.metal_rough, caps.data)] {
            let (Some(img), Some(orig)) = (img, orig) else { continue };
            assert!(img.rgba.is_empty() && !img.encoded.is_empty(), "{name}: bildet pakkes ut senere");
            assert!(img.width.max(img.height) <= cap.max(orig.width.max(orig.height).min(cap)), "{name}: {}×{} over {cap}", img.width, img.height);
            assert_eq!(img.width.max(img.height), orig.width.max(orig.height).min(cap), "{name}");
            assert_eq!(pixels(img).len(), (img.width * img.height * 4) as usize, "{name}");
        }
    }
    assert_eq!(browser_caps("sword").color, 2048);
    assert_eq!(browser_caps("crypt_01_aa").color, 2048);
    // Troféene er skarpe i nettleseren også (plan 13d): platen er 2048.
    assert_eq!(browser_caps("gallery_card").color, 2048);
    let (_, p) = model_files().find(|(n, _)| *n == "gallery_card").expect("gallery_card");
    let bytes = std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let m = load_capped("gallery_card", &[("model.glb", &bytes)], -1.0, 0.0, browser_caps("gallery_card")).unwrap();
    assert_eq!(m.albedo.as_ref().map(|i| i.width), Some(2048), "troféet er 2048 i nettleseren");
    // De store fiendene er skarpe nært (Rammeverket er bakt i 4096), de
    // vanlige 2048 som resten.
    assert_eq!(browser_caps("framework").color, 4096);
    assert_eq!(browser_caps("mutant_button").color, 2048);
    let (_, p) = model_files().find(|(n, _)| *n == "framework").expect("framework");
    let bytes = std::fs::read(format!("{}/assets/models/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let m = load_capped("framework", &[("model.glb", &bytes)], -1.0, 0.0, browser_caps("framework")).unwrap();
    assert_eq!(m.albedo.as_ref().map(|i| i.width), Some(4096), "Rammeverket er 4096 i nettleseren");
    // Lat lasting: nettleseren (`Files`) laster først det lave taket og
    // husker taket over til rendereren oppgraderer (`Image.later`).
    use crate::ports::ModelSource;
    let files = crate::adapters::gltf::Files([(p.to_string(), bytes)].into_iter().collect());
    let staged = files.load("framework").expect("framework");
    let img = staged.albedo.as_ref().unwrap();
    assert_eq!((img.width.max(img.height), img.later.map(|(w, h)| w.max(h))), (crate::adapters::delivery::LOW.color, Some(4096)), "lavt først, 4096 senere");
    assert!(staged.normal.as_ref().is_none_or(|i| i.width.max(i.height) <= crate::adapters::delivery::LOW.data));
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
    assert!(a.tavern < 1.0, "ved smia er du ikke i Vertshuset: {a:?}");
    // Inne i Vertshuset (der trubaduren synger når han har fått mjød).
    let tavern = world().world.placements.iter().find(|p| p.model == "tavern").map(|p| Vec3::from(p.transforms[0].translation)).expect("Vertshuset");
    game.place_feet(tavern + Vec3::Y * 0.3);
    let a = last(&mut game, &mut physics);
    assert!(a.indoor && a.tavern >= 1.0, "inne i Vertshuset: {a:?}");
    game.use_first(|u| *u == Use::Descend);
    let a = last(&mut game, &mut physics);
    assert!(a.cellar, "i kjelleren: {a:?}");
}

/// Vertshuset høres i alle etasjene (eieren 18. oktober): skjenkestua er
/// 1, overetasjen og kjelleren litt svakere, men fortsatt inne i huset
/// (≥ 0,6, så byens spor tones ned), og utenfor døra under 0,5.
#[test]
fn vertshuset_hores_i_alle_etasjene() {
    use crate::ports::driving::Ambience;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let pl = world().world.placements.iter().find(|p| p.model == "tavern").map(|p| p.transforms[0]).expect("Vertshuset");
    let mut at = |game: &mut Session, physics: &mut RapierPhysics, local: Vec3| -> Ambience {
        game.place_feet(pl.transform_point3(local));
        let mut a = None;
        for _ in 0..10 {
            for e in game.update(&mut input, 1.0 / 30.0, physics) {
                if let Event::Ambience(x) = e {
                    a = Some(x);
                }
            }
        }
        a.unwrap_or(Ambience { tavern: -1.0, ..Default::default() })
    };
    let ground = at(&mut game, &mut physics, Vec3::new(2.0, 0.7, 1.0));
    assert!(ground.tavern >= 1.0, "skjenkestua: {ground:?}");
    let upstairs = at(&mut game, &mut physics, Vec3::new(-2.0, 4.0, 0.5));
    assert!((0.6..1.0).contains(&upstairs.tavern), "overetasjen: {upstairs:?}");
    let cellar = at(&mut game, &mut physics, Vec3::new(0.0, -2.1, 0.0));
    assert!((0.6..1.0).contains(&cellar.tavern), "kjelleren: {cellar:?}");
    let outside = at(&mut game, &mut physics, Vec3::new(0.0, 0.0, 9.0));
    assert!((0.0..0.5).contains(&outside.tavern), "utenfor døra: {outside:?}");
}

/// Trubaduren synger bare for et krus mjød (eieren 18. oktober): E gir en
/// egen dialog med prisen, uten token går det ikke, kruset koster 1 token,
/// og så synger han kvadet om Gorvek én gang (`Event::Song`) og spiller
/// (`play`) mens det går; mens han synger er knappen av, og ingen betaler to
/// ganger. Når kvadet er over, sitter han stille (`sit`).
#[test]
fn trubaduren_synger_for_et_krus_mjod() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let mut tick = |game: &mut Session, physics: &mut RapierPhysics| -> Vec<Event> { (0..10).flat_map(|_| game.update(&mut input, 1.0 / 30.0, physics)).collect() };
    let song = |events: &[Event]| events.iter().find_map(|e| if let Event::Song(on) = e { Some(*on) } else { None });
    tick(&mut game, &mut physics);
    assert!(!game.singing(), "uten mjød synger han ikke");
    assert_eq!(game.troubadour_clip(), Some("sit"), "uten mjød sitter han stille");
    match game.trade_troubadour() {
        Some(Event::Mead(m)) => assert!(m.price == 1 && !m.singing && m.text.contains("Gorvek"), "{m:?}"),
        e => panic!("E på trubaduren gir mjøddialogen: {e:?}"),
    }
    // Uten token.
    game.set_tokens(0);
    assert!(!game.buy_mead(), "mjød uten token");
    let ev = tick(&mut game, &mut physics);
    assert!(!game.singing() && song(&ev).is_none(), "sangen startet uten mjød");
    // Med én token: kruset koster 1, og han synger.
    game.set_tokens(1);
    assert!(game.buy_mead(), "mjød for 1 token");
    assert_eq!(game.tokens(), 0, "mjøden koster 1 token");
    let ev = tick(&mut game, &mut physics);
    assert_eq!(song(&ev), Some(true), "sangen begynner");
    assert!(ev.iter().any(|e| matches!(e, Event::Sound(s) if s.name == "coins")), "myntene klirrer");
    assert!(game.singing());
    // Takken står litt; så spiller han.
    for _ in 0..25 {
        tick(&mut game, &mut physics);
    }
    assert_eq!(game.troubadour_clip(), Some("play"), "han spiller mens sangen går");
    // Kjøper du igjen mens han synger, betaler du ikke.
    game.set_tokens(3);
    match game.trade_troubadour() {
        Some(Event::Mead(m)) => assert!(m.singing, "knappen er av mens han synger"),
        e => panic!("{e:?}"),
    }
    assert!(!game.buy_mead(), "to krus for samme sang");
    assert_eq!(game.tokens(), 3, "ingen betaler to ganger");
    assert!(game.singing(), "han synger videre");
    // Kvadet er over: han sitter stille og er tørr i strupen.
    game.end_song();
    let ev = tick(&mut game, &mut physics);
    assert_eq!(song(&ev), Some(false), "sangen er over");
    for _ in 0..25 {
        tick(&mut game, &mut physics);
    }
    assert!(!game.singing());
    assert_eq!(game.troubadour_clip(), Some("sit"), "etter sangen sitter han stille");
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

/// Kanon (`docs/kanon.md`): ingen moderne ord i det verden sier. Replikkene
/// (`world/cast.rs`), byen (`assets/layout/town.json`), lappene og navnene i
/// kryptnivåene og filmens tekster sjekkes. Bøkene viser LO-TEKs ekte
/// dokumentasjon og er unntatt.
#[test]
fn kanon_ingen_moderne_ord_i_verden() {
    const MODERN: &[&str] = &["react", "javascript", "js", "jsx", "typescript", "npm", "props", "webpack", "html", "css", "api", "frontend", "backend", "deploy"];
    let root = env!("CARGO_MANIFEST_DIR");
    let mut texts: Vec<(String, String)> = vec![];
    // Strengene i kildekoden (ikke kommentarene).
    let cast = std::fs::read_to_string(format!("{root}/crates/fristad-domain/src/domain/world/cast.rs")).expect("cast.rs");
    // Ansgars lapp første gang du går ned (`crypt/run.rs`).
    let run = std::fs::read_to_string(format!("{root}/crates/fristad-domain/src/domain/world/crypt/run.rs")).expect("run.rs");
    for (name, src) in [("cast.rs", &cast), ("run.rs", &run)] {
        for line in src.lines().filter(|l| !l.trim_start().starts_with("//")) {
            for (k, part) in line.split('"').enumerate() {
                if k % 2 == 1 {
                    texts.push((name.into(), part.to_string()));
                }
            }
        }
    }
    assert!(texts.iter().any(|(f, t)| f == "run.rs" && t.contains("Mønsterboka")), "Ansgars lapp sjekkes");
    fn strings(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::String(s) => out.push(s.clone()),
            serde_json::Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
            serde_json::Value::Object(o) => o.values().for_each(|x| strings(x, out)),
            _ => {}
        }
    }
    let mut files = vec![format!("{root}/assets/layout/town.json"), format!("{root}/assets/film/voice/lines.json")];
    for e in std::fs::read_dir(format!("{root}/assets/models/crypt")).expect("krypten").flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "json") {
            files.push(p.to_string_lossy().into_owned());
        }
    }
    for f in files {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&f).expect("fila")).expect("gyldig json");
        let mut out = vec![];
        strings(&v, &mut out);
        texts.extend(out.into_iter().map(|t| (f.clone(), t)));
    }
    let bad: Vec<String> = texts
        .iter()
        .filter(|(_, t)| t.contains(' '))
        .filter_map(|(f, t)| {
            let lower = t.to_lowercase();
            let hit = lower.split(|c: char| !c.is_alphanumeric()).find(|w| MODERN.contains(w))?;
            Some(format!("{f}: «{t}» ({hit})"))
        })
        .collect();
    assert!(bad.is_empty(), "moderne ord i verden (docs/kanon.md):\n{}", bad.join("\n"));
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
/// Bueskytteren holder buen i venstre hånd (eieren 20. oktober: «det ser
/// rart ut at den angriper med buen på ryggen og tomme hender»): grepet står
/// i hånda i hvile og i skuddet, og buen står loddrett når pila går
/// (bilde 15 av 30, `attack_shoot`).
#[test]
fn bueskytteren_holder_buen() {
    use glam::Affine3A;
    let p = world();
    assert!(crate::adapters::gltf::model_files().any(|(n, _)| *n == "bow"), "buen mangler");
    // Uten plass satt av tegner rendereren den aldri (eieren 20. oktober).
    assert!(crate::domain::mobs::Mobs::reserve().iter().any(|(m, n)| *m == "bow" && *n > 0), "buen har ingen plass i rendereren");
    let all = rigs(&p.models);
    let (_, rig) = all.iter().find(|(n, _)| n == "breadcrumb_archer").expect("bueskytteren");
    let (bow, hand) = (rig.joint("bow").expect("beinet «bow»"), rig.joint("hand.L").unwrap());
    let place = |pose: &crate::domain::anim::Pose| Affine3A::from_mat4(rig.joint_transform(pose, bow)) * Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let (attack, walk, idle) = (rig.clip("attack").unwrap(), rig.clip("walk").unwrap(), rig.clip("idle").unwrap());
    let chest = rig.joint("chest").expect("brystet");
    // Eieren 20. oktober: «Holder buen feil vei». Riggens front er +x
    // (`Mobs::joint_world` dreier +x mot yaw); buens rom fra bow.py: lemmene
    // langs y (z i Blender), buken mot +x, grepet 7,5 cm foran origo og
    // strengen 3,5 cm bak.
    let mut poses = vec![("skuddet", rig.sample(attack, 15.0 / 24.0, false))];
    for k in 0..4 {
        poses.push(("gangen", rig.sample(walk, k as f32 * 0.17, true)));
        poses.push(("ventingen", rig.sample(idle, k as f32 * 0.4, true)));
    }
    for (what, pose) in poses {
        let t = place(&pose);
        let grip = t.transform_point3(Vec3::X * 0.075);
        let wrist = rig.joint_transform(&pose, hand).w_axis.truncate();
        assert!(grip.distance(wrist) < 0.15, "{what}: grepet står {:.2} m fra håndleddet", grip.distance(wrist));
        let along = t.transform_vector3(Vec3::Y).normalize();
        let belly = t.transform_vector3(Vec3::X).normalize();
        assert!(along.y.abs() > if what == "skuddet" { 0.95 } else { 0.8 }, "{what}: buen står ikke loddrett: {along}");
        assert!(belly.x > 0.9, "{what}: buen buer ikke mot målet (buken {belly})");
        // Strengen er nærmere brystet enn staven.
        let c = rig.joint_transform(&pose, chest).w_axis.truncate();
        let string = t.transform_point3(Vec3::X * -0.035);
        let flat = |p: Vec3| Vec3::new(p.x - c.x, 0.0, p.z - c.z).length();
        assert!(flat(string) < flat(grip), "{what}: strengen står lenger fra brystet enn staven");
        // Den nederste tuppen står over gulvet.
        let low = t.transform_point3(Vec3::Y * 0.56).y.min(t.transform_point3(-Vec3::Y * 0.56).y);
        assert!(what == "skuddet" || low > -0.02, "{what}: buen går {:.2} m ned i gulvet", -low);
    }
}

#[test]
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
    // En vanlig fiende, og Rammeverket i sitt nivå (TRELLIS-formen fra mob3d.py).
    let plain = mobs.mobs.iter().position(|m| !m.kind.boss()).expect("en fiende");
    let deep = p.world.crypt.spawns.iter().find(|s| Kind::from_label(&s.label) == Kind::Framework).expect("Rammeverket står i et nivå").level;
    let mut flip = 0;
    for (i, side) in [(Some(plain), 1.0f32), (Some(plain), -1.0), (None, 1.0), (None, -1.0)] {
        let i = i.unwrap_or_else(|| {
            if mobs.level != deep {
                mobs.enter(&p.world.crypt, deep);
            }
            mobs.mobs.iter().position(|m| m.kind == Kind::Framework).expect("Rammeverket")
        });
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
        assert!(ahead.dot(Vec3::Z) > 0.7, "uten dreining ser hodet til {} ikke fram ({ahead:?})", mobs.mobs[i].kind.model());
        assert!(ang(turned) < ang(ahead) - 0.3, "{} dreier ikke hodet mot spilleren på side {side}: {turned:?} mot {ahead:?}, ønsket {to:?}", mobs.mobs[i].kind.model());
        flip += 1;
    }
    assert_eq!(flip, 4);
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
/// svart, og døgnet snur (dag til natt og natt til dag). Hver natt koster
/// `SLEEP_PRICE` (10) tokens, og siktet sier det.
#[test]
fn senga_gir_hvile_og_snur_dognet() {
    use crate::domain::hero::SLEEP_PRICE;
    use crate::domain::weather::Daylight;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert_eq!(game.daylight(), Daylight::Day);
    assert_eq!(SLEEP_PRICE, 10);
    game.set_coins(25);
    let prompt = game.prompt_first(|u| *u == Use::Sleep).expect("senga");
    assert_eq!(prompt.verb, "Sov (10 tokens)");
    game.set_vitals(12.0, 3.0);
    let events = game.use_first(|u| *u == Use::Sleep);
    assert_eq!(game.coins_now(), 15, "søvnen koster 10 tokens");
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
    assert_eq!(game.coins_now(), 5);
}

/// Uten 10 tokens blir det ingen søvn: et avslag som i butikkene («Har ikke
/// råd», lyden `deny`), og livet, døgnet og tokenene er som før.
#[test]
fn senga_koster_og_avslar_uten_tokens() {
    use crate::domain::weather::Daylight;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.set_coins(9);
    game.set_vitals(12.0, 3.0);
    let prompt = game.prompt_first(|u| *u == Use::Sleep).expect("senga");
    assert_eq!(prompt.verb, "Sov (10 tokens)");
    assert!(prompt.info.contains("Har ikke råd"), "siktet sier at du ikke har råd: {}", prompt.info);
    let events = game.use_first(|u| *u == Use::Sleep);
    assert!(events.iter().any(|e| matches!(e, Event::Sound(s) if s.name == "deny")), "avslaget: {events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("Har ikke råd"))), "{events:?}");
    assert!(!events.contains(&Event::Fade) && !events.iter().any(|e| matches!(e, Event::Daylight(_))), "ingen søvn: {events:?}");
    assert_eq!((game.coins_now(), game.daylight(), game.hp_now()), (9, Daylight::Day, 12.0));
    game.set_coins(10);
    let events = game.use_first(|u| *u == Use::Sleep);
    assert!(events.contains(&Event::Fade) && game.coins_now() == 0, "med akkurat nok sover du: {events:?}");
}

/// Eieren 18. oktober: «det skal alltid lønne seg veldig å lese boka». Å
/// lese om en komponent gir fullt liv, full utholdenhet og olje i lykta,
/// sier det i toasten, og bare én gang per komponent per tur.
#[test]
fn boka_gir_fullt_liv_utholdenhet_og_olje() {
    use crate::domain::hero::BOOK_OIL;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    game.set_vitals(10.0, 2.0);
    game.set_oil(0.1);
    let oil = game.oil_now();
    assert!(game.read_doc("button"));
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert_eq!(game.hp_now(), game.max_hp_now(), "fullt liv");
    assert!(game.stamina_now() >= 99.0, "full utholdenhet: {}", game.stamina_now());
    assert!((game.oil_now() - (oil + BOOK_OIL)).abs() < 1.0, "olje i lykta: {} mot {}", game.oil_now(), oil);
    assert!(events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("fullt liv") && t.contains("full utholdenhet") && t.contains("olje"))), "toasten sier hva boka ga: {events:?}");
    game.set_vitals(10.0, 2.0);
    assert!(!game.read_doc("button"), "én gang per komponent per tur");
    assert_eq!(game.hp_now(), 10.0);
}

/// Første gang du går ned i krypten, åpnes lappen fra mester Ansgar midt
/// på skjermen (`Event::Lore`) og føres i journalen; neste gang ikke, heller
/// ikke etter lagring og innlasting. Gamle lagringer uten lappen får den
/// bare om du ikke har kommet lenger enn nivå 1.
#[test]
fn ansgars_lapp_kommer_forste_gang_du_gar_ned() {
    use crate::domain::world::crypt::run::{ANSGAR_TEXT, ANSGAR_TITLE};
    let letter = |events: &[Event]| events.iter().any(|e| matches!(e, Event::Lore { title, text } if title == ANSGAR_TITLE && text == ANSGAR_TEXT));
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    assert!(game.journal().is_empty());
    let events = game.use_first(|u| *u == Use::Descend);
    assert!(letter(&events), "lappen første gang ned: {events:?}");
    assert_eq!(game.journal().first().map(|(t, _)| t.as_str()), Some(ANSGAR_TITLE), "lappen står i journalen");
    for w in ["Mønsterboka", "Rammeverket", "Trykk H", "Apotekeren", "Skriptoriet", "hvilestedene", "Dør du"] {
        assert!(ANSGAR_TEXT.contains(w), "lappen sier {w}");
    }
    let words = ANSGAR_TEXT.split_whitespace().count();
    assert!((120..=200).contains(&words), "lappen leses på under ett minutt: {words} ord");
    let events = game.use_first(|u| *u == Use::Descend);
    assert!(!letter(&events), "bare første gang: {events:?}");
    let saved = game.save();
    assert!(saved.lore.iter().any(|t| t == ANSGAR_TITLE), "lappen huskes: {:?}", saved.lore);
    let mut physics2 = RapierPhysics::new();
    let mut fresh = session(&mut physics2);
    fresh.load(saved);
    assert!(!letter(&fresh.use_first(|u| *u == Use::Descend)), "ikke igjen etter ny økt");
    assert!(fresh.journal().iter().any(|(t, _)| t == ANSGAR_TITLE));
    // Gamle lagringer uten lappen: nivå 1 eller ingenting får den, dypere ikke.
    for (deepest, shown) in [(None, true), (Some(0), true), (Some(2), false)] {
        let mut physics3 = RapierPhysics::new();
        let mut old = session(&mut physics3);
        old.load(SaveData { deepest, ..Default::default() });
        assert_eq!(letter(&old.use_first(|u| *u == Use::Descend)), shown, "gammel lagring med dypeste {deepest:?}");
    }
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
    crypt_open_in(world())
}

fn crypt_open_in(p: &Prepared) -> Crypt {
    let mut c = p.world.crypt.clone();
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
    levels_have_what_they_need(world());
}

#[test]
fn provenivaaet_har_det_det_skal_og_taket_er_hoyt_nok() {
    levels_have_what_they_need(doc_world());
}

fn levels_have_what_they_need(p: &Prepared) {
    let c = &p.world.crypt;
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
    enemies_find_their_way(world());
}

/// Det samme i prøvenivået fra nivådokumentet.
#[test]
fn fiendene_finner_fram_i_provenivaaet() {
    enemies_find_their_way(doc_world());
}

fn enemies_find_their_way(p: &Prepared) {
    use crate::domain::mobs::Mobs;
    let c = crypt_open_in(p);
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
            // Tatt igjen: innen 2,6 m, eller innen rekkevidden for de store (Rammeverket slår fra 3 m).
            let caught = 2.6f32.max(mobs.mobs[0].kind.stats().range + 0.3);
            let rise = 1.6 * mobs.mobs[0].kind.stats().scale.max(1.0);
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
                if gap < caught && (m.pos.y - player.y).abs() < rise {
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
    /// Verden boten går i (spillet, eller prøvenivået).
    world: &'static Prepared,
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
        Bot { world: world(), input: Input::default(), t: 0.0, fight, near_deaths: 0, kills: 0, chests: 0, log: vec![], swing: 0, last: Vec3::ZERO, stuck: 0.0, sidestep: 0.0, side: false }
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
            // Står den og vender (trakten gir et hjørne like ved, så et annet
            // når den har tatt et skritt), sikter den lenger fram.
            let reach = if self.t - since > 3.0 { 0.6 } else { 0.25 };
            let next = path.iter().copied().find(|c| c.with_y(0.0).distance(feet.with_y(0.0)) > reach).unwrap_or(to);
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

    /// Gå rett mot `to` uten navigasjonsnettet i `secs` sekunder (om bord
    /// på heisen, som ikke er med i nettet).
    fn stride(&mut self, game: &mut Session, physics: &mut RapierPhysics, to: Vec3, secs: f32) {
        let end = self.t + secs;
        while self.t < end {
            self.aim(game, to.with_y(game.eye().y));
            self.input.key("KeyW", true);
            self.step(game, physics);
        }
        self.input.key("KeyW", false);
    }

    /// Heisen i nivå `k` når bremsen er løsnet: hent den opp om den står
    /// nede, gå om bord fra den åpne sida, dra i styrestangen og vent til
    /// den har kjørt. Gir `false` om det ikke er noen heis å ta herfra.
    fn ride_lift(&mut self, game: &mut Session, physics: &mut RapierPhysics, c: &Crypt, k: usize) -> bool {
        let feet = game.feet();
        let Some((i, l)) = c.lifts.iter().enumerate().find(|(i, l)| l.level == k && game.lift_state(*i).0 && (l.pos.y - feet.y).abs() < 2.0) else { return false };
        let pose = crate::domain::world::at(l.pos, l.yaw);
        let board = pose.transform_point3(Vec3::new(-(l.size.x * 0.5 + 0.7), 0.0, 0.0));
        if !self.walk_to(game, physics, board, 0.6, "heisen") {
            return false;
        }
        if !game.lift_state(i).1 {
            self.aim(game, pose.transform_point3(Vec3::new(0.0, 0.7, crate::domain::world::crypt::LIFT_WINCH_Z)));
            game.use_exact(Use::Winch(i));
            self.note(format!("  sveiver heisen opp etter {:.0} s", self.t));
            let t1 = self.t;
            while game.lift_state(i).2 && self.t - t1 < 20.0 {
                self.step(game, physics);
            }
            self.walk_to(game, physics, board, 0.6, "heisen");
        }
        self.stride(game, physics, l.pos, 0.9);
        self.aim(game, pose.transform_point3(Vec3::new(l.size.x * 0.5 - 0.3, 0.75, l.size.y * 0.5 - 0.3)));
        game.use_exact(Use::Lift(i));
        self.note(format!("  tar heisen ned fra {:?} etter {:.0} s", game.feet(), self.t));
        let t1 = self.t;
        self.idle(game, physics, 0.5);
        while game.lift_state(i).2 && self.t - t1 < 20.0 {
            self.step(game, physics);
        }
        self.idle(game, physics, 0.5);
        let feet = game.feet();
        if (feet.y - (l.pos.y - l.drop)).abs() > 1.5 {
            self.fail(format!("heisen tok ikke boten med ned: føttene ved {feet:?}, bunnen på {:.1}", l.pos.y - l.drop));
        }
        self.note(format!("  nede ved {feet:?}"));
        true
    }

    /// Det nederste nivået har ingen trapp ned: boten går ned til bossen i
    /// arenaen (porten står oppe til bossen stenger den bak deg), og slår den
    /// når den slåss.
    fn final_level(&mut self, game: &mut Session, physics: &mut RapierPhysics, k: usize) {
        let w = self.world;
        let c = &w.world.crypt;
        let l = &c.levels[k];
        let Some(boss) = c.spawns.iter().find(|s| s.level == k && s.boss) else { return };
        let t0 = self.t;
        self.note(format!("nivå {} ({}): starter ved {:?}, bossen venter ved {:?}", l.n, l.name, game.feet(), boss.pos));
        if !self.walk_to(game, physics, boss.pos, 2.5, "bossen") && game.boss_pos().is_none() && game.feet().distance(boss.pos) > 4.0 {
            self.fail(format!("nivå {}: kom ikke fram til bossen ved {:?}", l.n, boss.pos));
        }
        let arena = c.room_at(game.feet()).map(|(_, r)| l.rooms[r].id.clone());
        self.note(format!("  ved bossen i {arena:?} etter {:.0} s", self.t - t0));
        let t1 = self.t;
        while let Some(at) = game.boss_pos() {
            if self.t - t1 > 240.0 {
                self.fail(format!("nivå {}: fikk ikke slått bossen på fire minutter", l.n));
            }
            if !self.brawl(game, physics) {
                self.walk_to(game, physics, at, 1.8, "bossen");
            }
        }
        if self.fight {
            self.note(format!("  bossen falt etter {:.0} s", self.t - t0));
        }
        self.idle(game, physics, 2.0);
    }

    /// Gjennom nivå `k`: fra der boten står til trappa ned, og trekker
    /// spakene (og henter nøkkelen) når veien er stengt.
    fn through_level(&mut self, game: &mut Session, physics: &mut RapierPhysics, k: usize) {
        let w = self.world;
        let c = &w.world.crypt;
        let l = &c.levels[k];
        let Some((down, _)) = l.down else {
            self.final_level(game, physics, k);
            return;
        };
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
            // Heisen (bremsen løsnet av spaken): ned sjakta med den.
            if self.ride_lift(game, physics, c, k) {
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
    walk_through(world());
}

/// Det samme i prøvenivået fra nivådokumentet: fra startstedet, gjennom
/// porten med spaken, ned trappa i nedgangen.
#[test]
fn provenivaaet_kan_gas_gjennom() {
    let p = doc_world();
    walk_through(p);
    // Nivået har bare ett nivå: trappa ned sier at det ikke går dypere.
}

fn walk_through(p: &'static Prepared) {
    let c = &p.world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session_in(p, &mut physics);
    let mut bot = Bot::new(false);
    bot.world = p;
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
    // Komponentene ved bunnen og hva de holder til (plan 14), med en firedel
    // av tokenene hos Buttonsmeden.
    let v = game.inventory_view();
    let mut got = std::collections::BTreeMap::new();
    for (c, n) in v.stash.iter().chain(&v.carried) {
        *got.entry(c.clone()).or_insert(0) += n;
    }
    let made = crate::domain::loot::forge(&got, game.coins_now() / 4, true);
    bot.note(format!("komponentene ved bunnen ({} i alt, {} tokens): {got:?}", got.values().sum::<u32>(), game.coins_now()));
    bot.note(format!("holder til {} oppskrifter: {}", made.len(), made.iter().map(|r| format!("{} ({})", r.name, r.rarity.name())).collect::<Vec<_>>().join(", ")));
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

/// Sprekken som hinter om en falsk vegg står på selve veggen (som har sin
/// egen sprekk), aldri på muren ved siden av, der et slag ikke gjør noe.
#[test]
fn sprekken_star_pa_den_falske_veggen() {
    let p = world();
    let c = &p.world.crypt;
    let cracks: Vec<Vec3> = p.world.placements.iter().filter(|pl| pl.model == crypt::KIT_CRACK).flat_map(|pl| pl.transforms.iter().map(|t| t.transform_point3(Vec3::ZERO))).collect();
    let mut bad = vec![];
    for s in &c.secrets {
        for q in &cracks {
            if q.distance(s.pos) < s.width * 0.5 + 2.0 {
                bad.push(format!("en sprekk ved ({:.1}, {:.1}, {:.1}) står {:.1} m fra den falske veggen ved ({:.1}, {:.1}, {:.1}), ikke på den", q.x, q.y, q.z, q.distance(s.pos), s.pos.x, s.pos.y, s.pos.z));
            }
        }
    }
    assert!(bad.is_empty(), "\n{}", bad.join("\n"));
}

/// Den falske veggen ligger i flukt med muren rundt (eieren 18. oktober: den
/// stakk ut og røpet seg): hver har nivåets egen bit med murens tekstur
/// (`crypt_NN_secret_<id>`), forsida ligger i murens plan, og målet går
/// ikke ut til sidene.
#[test]
fn den_falske_veggen_ligger_i_flukt() {
    let p = world();
    let c = &p.world.crypt;
    assert!(!c.secrets.is_empty(), "ingen falske vegger i krypten");
    let mut bad = vec![];
    for (i, s) in c.secrets.iter().enumerate() {
        if s.model == crypt::KIT_SECRET {
            bad.push(format!("den falske veggen {i} i nivå {} har ikke sin egen bit (build_level.py N --secrets)", c.levels[s.level].n));
            continue;
        }
        let Some(m) = p.models.iter().find(|m| m.name == s.model) else {
            bad.push(format!("{} finnes ikke", s.model));
            continue;
        };
        // Midt på flaten (over sokkelen) ligger forsida i murens plan; i et
        // rundt rom bøyer den seg med muren ut mot kantene.
        let (lo, hi) = m.mesh.bounds();
        let front = m.mesh.vertices.iter().filter(|v| v.pos[0].abs() < 0.3 && (0.5..2.0).contains(&v.pos[1])).map(|v| v.pos[2]).fold(f32::MIN, f32::max);
        if front > 0.02 || (hi.x - s.width * 0.5).abs() > 0.1 || (lo.x + s.width * 0.5).abs() > 0.1 {
            bad.push(format!("{} går fra {lo:?} til {hi:?} (forsida midt på {front:.2} m ut): står ut fra muren eller er ikke {} m bred", s.model, s.width));
        }
        let t = p.world.targets.iter().find(|t| t.what == Use::Secret(i)).expect("den falske veggen har intet mål");
        if t.half.x > s.width * 0.5 + 0.01 || t.center.z + t.half.z > 0.1 {
            bad.push(format!("målet til {} går utenfor den falske delen", s.model));
        }
    }
    assert!(bad.is_empty(), "\n{}", bad.join("\n"));
}

/// Den falske veggen slås inn med et hugg, ikke bare med F (eieren 18.
/// oktober: «hvordan åpner man den?»; siktet sier «Slå, eller skyv»).
#[test]
fn den_falske_veggen_slas_inn() {
    let c = &world().world.crypt;
    let s = c.secrets.first().expect("ingen falsk vegg i krypten").clone();
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    let front = in_front(c, s.pos, s.yaw, 1.4);
    game.place_feet(front);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let eye = game.eye();
    let d = (s.pos + Vec3::Y * 1.2 - eye).normalize();
    game.set_look((-d.x).atan2(-d.z), d.y.asin());
    let mut opened = false;
    for k in 0..60 {
        if k == 1 {
            input.actions.push(Action::Attack);
        }
        if k == 3 {
            input.actions.push(Action::AttackRelease);
        }
        let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
        opened |= events.iter().any(|e| matches!(e, Event::Toast(t) if t.contains("rom bak")));
    }
    assert!(opened, "hugget slo ikke inn den falske veggen");
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
    // Lappen vises som et kort midt på skjermen (`Event::Lore`), ikke som et varsel.
    assert!(events.iter().any(|e| matches!(e, Event::Lore { title, text } if *title == lore.title && *text == lore.text)), "lappen ble ikke lest: {events:?}");
    assert!(!events.iter().any(|e| matches!(e, Event::Toast(_))), "lappen kom som varsel i hjørnet: {events:?}");
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
    light_follows_rooms(world());
}

#[test]
fn lyset_folger_rommene_i_provenivaaet() {
    light_follows_rooms(doc_world());
}

fn light_follows_rooms(p: &Prepared) {
    let w = &p.world;
    let c = &w.crypt;
    let lit: Vec<_> = w.lights.iter().filter(|l| l.pos.y < crypt::TOP).collect();
    assert!(!lit.is_empty(), "ingen lys i krypten");
    for l in &lit {
        assert!(l.room.is_some(), "lyset ved {:?} står ikke i noe rom", l.pos);
    }
    let gate = &c.gates[0];
    // Et sted på hver side, utenfor rommet gitteret står i.
    let gate_room = c.room_at(gate.pos + Vec3::Y * 0.2).map(|r| r.1);
    let side = |sgn: f32| (1..12).map(|k| in_front(c, gate.pos, gate.yaw, sgn * 0.6 * k as f32)).find_map(|q| c.room_at(q + Vec3::Y * 0.2).map(|r| r.1).filter(|r| Some(*r) != gate_room));
    // Rommet gitteret står i (gangen), eller rommet på andre siden. Står
    // gitteret på grensen mellom to rom (en kant i nivådokumentet), kan
    // rommet foran være det gitteret regnes til: da er det andre bak.
    let ra = side(1.0).or(gate_room).expect("ingen rom ved gitteret");
    let rb = gate_room.filter(|g| *g != ra).or_else(|| side(-1.0)).expect("ingen rom bak gitteret");
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

/// Torgils' øvingsplass har en åpning mot gata: man går med Rapier fra
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
    assert!(left <= 0.6, "stoppet {left:.1} m fra midten av Torgils' plass, ved ({:.1}, {:.1})", pos.x, pos.z);
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

/// Hvert teksturlag spillet bruker, er eierens malte bilde (plan del 12):
/// `assets/textures/<lag>/meta.json` peker på kilden i
/// `assets/textures/painted/<lag>/`, og ingen lag i manifestet kommer fra
/// Poly Haven (fotoskannet). Lag som venter på eierens bilde, står i
/// `WAITING` med hvorfor; stryk dem når bildet er levert.
#[test]
fn alle_lagene_er_malte() {
    const WAITING: &[&str] = &[];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut bad = vec![];
    for name in crate::adapters::gltf::used_layers() {
        let name = name.as_str();
        if WAITING.contains(&name) {
            continue;
        }
        let dir = root.join("assets/textures").join(name);
        let Ok(meta) = std::fs::read_to_string(dir.join("meta.json")) else {
            // Lag uten bilde bakes prosedyrisk (bake.wgsl) og er ikke fotoskannet.
            continue;
        };
        let meta: serde_json::Value = serde_json::from_str(&meta).expect("meta.json");
        let src = meta["source"].as_str().unwrap_or("");
        if src.is_empty() || !root.join("assets/textures/painted").join(name).join(src).exists() {
            bad.push(format!("{name}: kilden «{src}» finnes ikke i assets/textures/painted/{name}/"));
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("assets/external/manifest.json")) {
        let manifest: serde_json::Value = serde_json::from_str(&text).expect("manifest.json");
        for (name, entry) in manifest["textures"].as_object().into_iter().flatten() {
            let id = entry["id"].as_str().unwrap_or("");
            if !id.starts_with("egen:") && !WAITING.contains(&name.as_str()) {
                bad.push(format!("{name}: fotoskannet ({id}) i manifestet"));
            }
        }
    }
    assert!(bad.is_empty(), "lag som ikke er eierens malte bilde: {bad:#?}");
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

/// Hvert eksempel i boka har en flate å stå på (eieren 18. oktober: Divider
/// sto i fire kolonner fordi panelet ikke kjente layouten `flow` fra
/// LO-TEK-docs): `web/docs-flate.css`, som `tools/export-docs.ts` skriver
/// rett fra Preview.astro, har en regel for hver layout i `web/docs.json`.
#[test]
fn eksemplene_i_boka_har_sin_flate() {
    let root = env!("CARGO_MANIFEST_DIR");
    let docs: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{root}/web/docs.json")).expect("docs.json")).expect("docs.json");
    let flate = std::fs::read_to_string(format!("{root}/web/docs-flate.css")).expect("web/docs-flate.css (bun tools/export-docs.ts)");
    let main = std::fs::read_to_string(format!("{root}/web/main.js")).expect("main.js");
    let mut bad = vec![];
    for (name, d) in docs.as_object().expect("docs") {
        let Some(layout) = d["demo"]["layout"].as_str() else { continue };
        let rule = format!("data-layout=\"{layout}\"");
        if !flate.contains(&rule) || !main.contains(&rule) {
            bad.push(format!("{name}: layouten «{layout}» mangler i docs-flate.css eller reserven i main.js"));
        }
    }
    assert!(bad.is_empty(), "\n{}", bad.join("\n"));
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

/// Innbyggerne havner aldri oppå hverandre (eieren 18. oktober): hver har
/// sin egen rute og sine egne ventepunkter, og når to møtes går de rundt
/// hverandre (`npcs::steer`, `keep_apart`). Over halvannet minutt står
/// ingen to nærmere enn 0,8 m, og de går faktisk noe sted.
#[test]
fn innbyggerne_gar_ikke_oppa_hverandre() {
    let layout: crate::domain::world::layout::Layout = serde_json::from_str(crate::adapters::data::layout_file().unwrap()).unwrap();
    let npcs = &layout.npcs;
    // Egne replikker og egne ruter med egne ventepunkter.
    for (i, a) in npcs.iter().enumerate() {
        for b in &npcs[i + 1..] {
            assert_ne!(a.lines, b.lines, "{} og {} har de samme replikkene", a.name, b.name);
            for (p, w) in a.route.iter().filter(|(_, w)| *w > 0.0) {
                for (q, v) in b.route.iter().filter(|(_, v)| *v > 0.0) {
                    assert!(p.distance(*q) > 1.4, "{} ({w} s) og {} ({v} s) venter på samme sted: {p:?} {q:?}", a.name, b.name);
                }
            }
        }
    }
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    // Langt unna alle: utenfor sørporten.
    game.place_feet(Vec3::new(0.0, 0.5, 90.0));
    let start = game.npc_positions();
    let mut closest = (f32::MAX, 0, 0, 0.0);
    let dt = 1.0 / 30.0;
    for step in 0..(90.0 / dt) as usize {
        game.update(&mut input, dt, &mut physics);
        let at = game.npc_positions();
        for i in 0..at.len() {
            for j in i + 1..at.len() {
                let gap = Vec2::new(at[i].x - at[j].x, at[i].z - at[j].z).length();
                if gap < closest.0 {
                    closest = (gap, i, j, step as f32 * dt);
                }
            }
        }
    }
    let names: Vec<&str> = npcs.iter().map(|n| n.name.as_str()).collect();
    assert!(closest.0 >= 0.8, "{} og {} sto {:.2} m fra hverandre etter {:.1} s", names[closest.1], names[closest.2], closest.0, closest.3);
    let moved = game.npc_positions().iter().zip(&start).filter(|(a, b)| a.distance(**b) > 0.5).count();
    assert!(moved >= 6, "bare {moved} innbyggere har gått noe sted på 90 s");
}

/// Hver innbygger har sine egne replikker etter rollen (eieren 18.
/// oktober), minst fem, aldri den samme to ganger på rad, og noen som
/// kommer først når du har vært dypere nede.
#[test]
fn innbyggerne_sier_sitt_eget() {
    use crate::domain::world::cast;
    let layout: crate::domain::world::layout::Layout = serde_json::from_str(crate::adapters::data::layout_file().unwrap()).unwrap();
    let mut progress = 0;
    for n in &layout.npcs {
        let lines = cast::lines(&n.lines);
        assert!(lines.len() >= 5, "{} har bare {} replikker", n.name, lines.len());
        progress += lines.iter().filter(|l| l.from > 0).count();
    }
    assert!(progress >= 20, "bare {progress} replikker følger framgangen");
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let k = game.npc_positions().len();
    let mut first = std::collections::HashSet::new();
    for i in 0..k {
        let mut last = String::new();
        let mut said = std::collections::HashSet::new();
        for _ in 0..12 {
            let s = game.talk_to(i, 0).expect("innbyggeren svarer");
            assert_ne!(s, last, "innbygger {i} sa det samme to ganger på rad");
            said.insert(s.clone());
            last = s;
        }
        assert!(said.len() >= 4, "innbygger {i} sa bare {} ulike ting", said.len());
        first.insert(game.talk_to(i, 0).unwrap());
        // Dypere nede: det nye kommer først.
        let lines = cast::lines(&layout.npcs[i].lines);
        let deep = game.talk_to(i, 10).unwrap();
        if lines.iter().any(|l| l.from > 0) {
            assert!(lines.iter().any(|l| l.text == deep && l.from > 0), "{}: «{deep}» er ikke noe nytt dypt nede", layout.npcs[i].name);
        }
    }
    assert!(first.len() == k, "to innbyggere sier det samme");
}

/// Lykta synes i byen (stolpen «Lys» sendes også der, så du ser hva du må
/// fylle hos Apotekeren), men oljen brenner bare nede i krypten (eieren 18.
/// oktober).
#[test]
fn lykta_brenner_bare_i_krypten() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.set_oil(0.5);
    let mut light = None;
    for _ in 0..(10.0 * 30.0) as usize {
        for e in game.update(&mut input, 1.0 / 30.0, &mut physics) {
            if let Event::Hud { light: l, cellar, .. } = e {
                assert!(!cellar, "du står i byen");
                light = Some(l);
            }
        }
    }
    assert_eq!(light, Some(50), "stolpen «Lys» viser oljen i byen, og den har ikke brent");
    game.use_first(|u| *u == Use::Descend);
    let mut down = None;
    for _ in 0..(20.0 * 30.0) as usize {
        for e in game.update(&mut input, 1.0 / 30.0, &mut physics) {
            if let Event::Hud { light: l, cellar: true, .. } = e {
                down = Some(l);
            }
        }
    }
    assert!(down.is_some_and(|l| l < 50), "nede brenner oljen: {down:?}");
}

/// Eieren 18. oktober: siktet på en fiende gir infopanelet til høyre
/// (`Event::Foe`), ikke sikteboblen midt på skjermen (`Event::Prompt`).
#[test]
fn fienden_vises_i_infopanelet_og_ikke_i_siktet() {
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    game.update(&mut input, 1.0 / 60.0, &mut physics);
    let mob = game.nearest_mob(game.feet()).expect("ingen fiende i nivå 1");
    let target = mob + Vec3::Y * 0.6;
    let mut seen = false;
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        let eye = target + Vec3::new(a.cos(), 0.0, a.sin()) * 2.0 + Vec3::Y * 0.5;
        let d = (target - eye).normalize();
        game.place(eye, (-d.x).atan2(-d.z), d.y.asin());
        let events = game.update(&mut input, 1.0 / 60.0, &mut physics);
        if events.iter().any(|e| matches!(e, Event::Foe(Some(f)) if !f.name.is_empty() && !f.info.is_empty() && f.max_hp > 0)) {
            assert!(!events.iter().any(|e| matches!(e, Event::Prompt(Some(_)))), "fienden fikk sikteboblen: {events:?}");
            seen = true;
            break;
        }
    }
    assert!(seen, "infopanelet kom ikke når du siktet på fienden ved {mob:?}");
}

/// Angrep på avstand synes (planen del 13e): inputen lader (lyd og glød i
/// munnen) et halvt sekund før skuddet, markøren tegnes hvert bilde fra den
/// spyttes til den treffer, og treffer den deg, sier `Event::Hurt` hvor den
/// kom fra (her fra høyre). Infopanelet sier at den skyter på avstand.
/// Et sted `dists` meter unna føttene dine i nivå 1, på samme gulv og med
/// fritt gulv hele veien (til fiendene som settes ut i testene).
fn open_spot(c: &crate::domain::world::crypt::Crypt, feet: Vec3, dists: &[f32]) -> Vec3 {
    let ground = |q: Vec3| Vec3::new(q.x, c.height_at(q + Vec3::Y * 0.5), q.z);
    let clear = |a: Vec3, b: Vec3| (1..20).all(|k| c.walkable(ground(a.lerp(b, k as f32 / 20.0))));
    (0..32)
        .flat_map(|k| dists.iter().map(move |&d| (k, d)))
        .map(|(k, d)| {
            let a = k as f32 * std::f32::consts::TAU / 32.0;
            ground(feet + Vec3::new(a.cos(), 0.0, a.sin()) * d)
        })
        .find(|q| c.walkable(*q) && (q.y - feet.y).abs() < 0.3 && clear(feet, *q))
        .unwrap_or_else(|| panic!("ingen plass {dists:?} m unna startstedet i nivå 1"))
}

/// Alert-flaggermusa (eieren 20. oktober: «alle monster angriper ulikt»):
/// når den ser deg, skriker den, og en knapp som sover i bakhold like ved
/// våkner; mellom stupene henger den for høyt for sverdet, og stupet går
/// ned til deg og biter.
#[test]
fn flaggermusa_skriker_vekker_de_andre_og_stuper() {
    use crate::domain::mobs::{bat, Kind};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let feet = game.feet();
    let c = game.crypt_now().clone();
    let ground = |q: Vec3| Vec3::new(q.x, c.height_at(q + Vec3::Y * 0.5), q.z);
    let at = open_spot(&c, feet, &[6.0, 7.5, 9.0]);
    // Knappen sover i bakhold et stykke bak flaggermusa, utenfor rekkevidden
    // som ellers vekker den.
    let away = (at - feet).with_y(0.0).normalize();
    let sleeper = [3.0f32, 2.0, 4.0].into_iter().map(|d| ground(at + away * d)).find(|q| c.walkable(*q)).expect("ingen plass til knappen");
    game.clear_mobs();
    let b = game.add_unaware(Kind::AlertBat, at, None);
    let k = game.add_unaware(Kind::Button, sleeper, Some(1.0));
    let to = (at - feet).with_y(0.0).normalize();
    game.set_look((-to.x).atan2(-to.z), 0.0);
    game.heal();
    let hp0 = game.hp_now();
    let (mut shriek, mut woke, mut high, mut low, mut bitten) = (false, false, 0, f32::MAX, false);
    for _ in 0..(10 * 60) {
        let events = game.update(&mut input, 1.0 / 60.0, &mut physics);
        shriek |= events.iter().any(|e| matches!(e, Event::Sound(s) if s.name == "shriek"));
        let (pos, air, aware, _) = game.mob_now(b);
        if aware && !woke {
            assert!(shriek, "flaggermusa så deg uten å skrike");
        }
        woke |= game.mob_now(k).2;
        let above = pos.y + air - game.feet().y;
        if !bitten && above > bat::SWING_TOP {
            high += 1;
        }
        low = low.min(above);
        bitten |= game.hp_now() < hp0;
        if bitten && woke {
            break;
        }
    }
    assert!(shriek, "flaggermusa skrek ikke");
    assert!(woke, "knappen i bakhold våknet ikke av skriket");
    assert!(high >= 30, "flaggermusa hang over sverdet i bare {high} bilder");
    assert!(bitten, "stupet bet ikke (lavest {low:.2} m over føttene dine)");
    assert!(low < bat::SWING_TOP, "stupet kom aldri ned dit sverdet når ({low:.2} m)");
}

/// Den grusomme Comboboxen (eieren 25. oktober: «jobb videre med combobox
/// og ormen»): den pisker lavt og høyt og velger deg før plata hamrer ned
/// med dobbel skade. Den høye pisken går klar når du dukker, den lave når du
/// hopper over den.
#[test]
fn comboboxen_pisker_lavt_og_hoyt_og_hamrer_platene_ned() {
    use crate::domain::mobs::{Blow, Kind};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let feet = game.feet();
    let c = game.crypt_now().clone();
    let at = open_spot(&c, feet, &[2.2, 2.5, 2.0]);
    game.only_mob(Kind::Combobox, at);
    let to = (at - feet).with_y(0.0).normalize();
    game.set_look((-to.x).atan2(-to.z), 0.0);
    let dmg = Kind::Combobox.stats().damage;
    // Hvor mye livet falt da hvert slag landet: (lav pisk, høy pisk, plata),
    // stående, dukket eller i hoppet.
    let (mut low_stand, mut low_jump, mut high_stand, mut high_duck, mut strike) = (vec![], vec![], vec![], vec![], vec![]);
    let mut chose = false;
    let mut last: Option<(Blow, bool)> = None;
    let mut mode = 0u32; // hvilket slag i rekken: annenhver gang står du, annenhver gang unngår du
    let mut jump_at: Option<f32> = None;
    let mut t = 0.0f32;
    for _ in 0..(90 * 60) {
        let now = game.mob_attack(0);
        // Et nytt slag: bestem om du skal unngå det.
        if let Some((blow, false)) = now {
            if last.map(|l| l.0) != Some(blow) || last.is_some_and(|l| l.1) {
                mode += 1;
                if let Blow::Lash { high: false } = blow {
                    // Hoppet skal være i lufta når halen feier forbi (bilde 20).
                    jump_at = (mode % 2 == 0).then_some(t + 20.0 / 24.0 - 0.3);
                }
            }
        }
        let dodge = mode % 2 == 0;
        input.crouch = dodge && matches!(now, Some((Blow::Lash { high: true }, _)));
        input.jump = jump_at.is_some_and(|j| t >= j && t < j + 0.05);
        game.heal();
        let hp = game.hp_now();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        t += 1.0 / 60.0;
        let lost = hp - game.hp_now();
        if let (Some((blow, false)), Some((b2, true))) = (now, game.mob_attack(0)) {
            assert_eq!(blow, b2);
            match blow {
                Blow::Lash { high: false } if dodge => low_jump.push(lost),
                Blow::Lash { high: false } => low_stand.push(lost),
                Blow::Lash { high: true } if dodge => high_duck.push(lost),
                Blow::Lash { high: true } => high_stand.push(lost),
                Blow::Strike => strike.push(lost),
                _ => {}
            }
        }
        chose |= matches!(now, Some((Blow::Choose, _)));
        last = now;
        if [&low_stand, &low_jump, &high_stand, &high_duck, &strike].iter().all(|v| !v.is_empty()) {
            break;
        }
    }
    assert!(chose, "Comboboxen valgte deg aldri før plata");
    for (name, v) in [("den lave pisken stående", &low_stand), ("den lave pisken i hoppet", &low_jump), ("den høye pisken stående", &high_stand), ("den høye pisken dukket", &high_duck), ("plata", &strike)] {
        assert!(!v.is_empty(), "{name}: kom aldri på 90 s");
    }
    assert!(low_stand.iter().all(|&l| l > 0.5 * dmg), "den lave pisken traff ikke deg som sto: {low_stand:?}");
    assert!(high_stand.iter().all(|&l| l > 0.5 * dmg), "den høye pisken traff ikke deg som sto: {high_stand:?}");
    assert!(low_jump.iter().all(|&l| l == 0.0), "hoppet gikk ikke klar av den lave pisken: {low_jump:?}");
    assert!(high_duck.iter().all(|&l| l == 0.0), "å dukke gikk ikke klar av den høye pisken: {high_duck:?}");
    let lash = low_stand.iter().chain(&high_stand).fold(0.0f32, |a, &b| a.max(b));
    assert!(strike.iter().any(|&l| l > 1.5 * lash), "plata gjorde ikke dobbelt så mye som pisken ({strike:?} mot {lash})");
}

/// Divider-ormen graver seg ned et stykke unna, går under gulvet der du
/// verken ser eller treffer den, og dukker opp ved deg og biter.
#[test]
fn ormen_graver_seg_ned_og_dukker_opp_ved_deg() {
    use crate::domain::mobs::{Blow, Kind};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let feet = game.feet();
    let c = game.crypt_now().clone();
    let at = open_spot(&c, feet, &[6.0, 7.0, 5.0]);
    game.only_mob(Kind::DividerWorm, at);
    let to = (at - feet).with_y(0.0).normalize();
    game.set_look((-to.x).atan2(-to.z), 0.0);
    game.heal();
    let hp0 = game.hp_now();
    let (mut burrowed, mut hidden, mut emerged_near, mut bitten) = (false, 0, false, false);
    for _ in 0..(12 * 60) {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        burrowed |= matches!(game.mob_attack(0), Some((Blow::Burrow, _)));
        if game.mob_under(0) {
            hidden += 1;
            let frame = game.frame(16.0 / 9.0);
            assert!(!frame.instances.iter().any(|i| i.model == "divider_worm"), "ormen under gulvet ble tegnet");
        }
        if matches!(game.mob_attack(0), Some((Blow::Emerge, _))) {
            emerged_near |= game.mob_now(0).0.with_y(0.0).distance(game.feet().with_y(0.0)) < 1.5;
        }
        bitten |= emerged_near && game.hp_now() < hp0;
        if bitten {
            break;
        }
    }
    assert!(burrowed, "ormen gravde seg aldri ned");
    assert!(hidden >= 30, "ormen var under gulvet i bare {hidden} bilder");
    assert!(emerged_near, "ormen dukket ikke opp ved deg");
    assert!(bitten, "ormen bet ikke da den dukket opp");
}

/// Badge-billen: skallet tar det meste forfra; et stykke unna skraper den
/// og stormer rett fram, og stangeløpet treffer hardere enn bittet. Går du
/// til siden, løper den i muren, velter og tar dobbelt så mye i buken.
#[test]
fn billen_stormer_og_velter_i_muren() {
    use crate::domain::mobs::{badge, Blow, Kind};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let feet = game.feet();
    let c = game.crypt_now().clone();
    let ground = |q: Vec3| Vec3::new(q.x, c.height_at(q + Vec3::Y * 0.5), q.z);
    let clear = |a: Vec3, b: Vec3| (1..24).all(|k| c.walkable(ground(a.lerp(b, k as f32 / 24.0))));
    // En retning med 6 m fritt gulv foran deg, en mur 1–4 m bak deg og
    // plass til å gå 1,8 m til siden.
    let (at, aside) = (0..64)
        .find_map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 64.0;
            let d = Vec3::new(a.cos(), 0.0, a.sin());
            let at = ground(feet + d * 6.0);
            let wall = (1..30).map(|s| s as f32 * 0.15).find(|&s| !c.walkable(ground(feet - d * s)))?;
            let side = [1.0f32, -1.0].into_iter().map(|sg| ground(feet + Vec3::new(-d.z, 0.0, d.x) * 1.8 * sg)).find(|q| c.walkable(*q) && clear(feet, *q))?;
            (c.walkable(at) && (at.y - feet.y).abs() < 0.3 && clear(feet, at) && (1.0..4.0).contains(&wall)).then_some((at, side))
        })
        .expect("ingen gang med en mur bak deg i nivå 1");
    let face = |game: &mut Session| {
        let to = (at - game.feet()).with_y(0.0).normalize();
        game.set_look((-to.x).atan2(-to.z), 0.0);
    };
    // Skallet forfra mot sida (den står stille og ser på deg).
    game.only_mob(Kind::BadgeBeetle, at);
    game.turn_mob(0, feet);
    let front = game.strike_mob(0, 10.0, at + (feet - at).normalize() * 1.0);
    game.only_mob(Kind::BadgeBeetle, at);
    game.turn_mob(0, feet);
    let side = (at - feet).normalize();
    let flank = game.strike_mob(0, 10.0, at + Vec3::new(-side.z, 0.0, side.x));
    assert!(front < 0.5 * flank, "skallet tok ikke slaget forfra ({front} mot {flank} fra sida)");
    // Stangeløpet rett på deg.
    game.only_mob(Kind::BadgeBeetle, at);
    face(&mut game);
    let (mut ran, mut rammed) = (false, 0.0f32);
    for _ in 0..(6 * 60) {
        game.heal();
        let hp = game.hp_now();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        ran |= matches!(game.mob_attack(0), Some((Blow::Charge, true)));
        if ran {
            rammed = rammed.max(hp - game.hp_now());
        }
        if rammed > 0.0 {
            break;
        }
    }
    assert!(ran, "billen stormet aldri");
    assert!(rammed > 1.5 * Kind::BadgeBeetle.stats().damage, "stangeløpet traff ikke hardere enn bittet ({rammed})");
    // Til siden når løpet begynner: den løper i muren og velter.
    game.step_to(feet);
    face(&mut game);
    game.only_mob(Kind::BadgeBeetle, at);
    let (mut stepped, mut flipped) = (false, false);
    for _ in 0..(8 * 60) {
        game.heal();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        if !stepped && matches!(game.mob_attack(0), Some((Blow::Charge, true))) {
            game.step_to(aside);
            stepped = true;
        }
        if game.mob_flipped(0) {
            flipped = true;
            break;
        }
    }
    assert!(stepped, "billen stormet ikke den andre gangen");
    assert!(flipped, "billen veltet ikke i muren");
    let pos = game.mob_now(0).0;
    let belly = game.strike_mob(0, 10.0, pos + Vec3::new(-side.z, 0.0, side.x));
    assert!((belly - 10.0 * badge::BELLY).abs() < 0.01, "buken tok ikke dobbelt ({belly})");
    for _ in 0..((badge::FLIP_SECS + 0.5) * 60.0) as usize {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    assert!(!game.mob_flipped(0), "billen ble liggende på ryggen");
}

/// Tag-midden: flokken sprer seg rundt deg før den piler inn, den som
/// biter, henger seg i ankelen, gnager og gjør deg treg, og et hopp rister
/// den av.
#[test]
fn middene_omringer_deg_og_henger_seg_fast() {
    use crate::domain::mobs::Kind;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let c = game.crypt_now().clone();
    // Et åpent rom i nivå 1 (gulv rundt deg i ringen), og fire midder
    // samlet et stykke unna: de skal komme fra hver sin kant.
    let start = game.feet();
    let ground = |q: Vec3| Vec3::new(q.x, c.height_at(q + Vec3::Y * 0.5), q.z);
    let open = |p: Vec3| (0..16).all(|k| [1.2f32, 1.7, 2.4].iter().all(|&r| {
        let a = k as f32 * std::f32::consts::TAU / 16.0;
        let q = ground(p + Vec3::new(a.cos(), 0.0, a.sin()) * r);
        c.walkable(q) && (q.y - p.y).abs() < 0.3
    }));
    let feet = (0..41 * 41)
        .map(|k| ground(start + Vec3::new((k % 41) as f32 - 20.0, 0.0, (k / 41) as f32 - 20.0) * 0.5))
        .filter(|p| c.walkable(*p) && open(*p))
        .min_by(|a, b| a.distance(start).total_cmp(&b.distance(start)))
        .expect("ingen åpen plass i nivå 1");
    game.step_to(feet);
    let at = open_spot(&c, feet, &[7.0, 8.0, 6.0, 5.0]);
    game.only_mob(Kind::TagMite, at);
    for k in 1..4 {
        let q = at + Vec3::new(0.3 * k as f32, 0.0, 0.0);
        game.add_mob(Kind::TagMite, if c.walkable(q) { q } else { at });
    }
    let mut sides = std::collections::BTreeSet::new();
    for _ in 0..(6 * 60) {
        game.heal();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        for i in 0..4 {
            let d = (game.mob_now(i).0 - game.feet()).with_y(0.0);
            if (1.2..2.4).contains(&d.length()) {
                sides.insert(((d.z.atan2(d.x) + std::f32::consts::PI) / std::f32::consts::FRAC_PI_2) as i32 % 4);
            }
        }
    }
    assert!(sides.len() >= 3, "middene kom bare fra {} kanter rundt deg", sides.len());
    // Én midd: den biter, henger fast og gnager, og du blir treg.
    game.only_mob(Kind::TagMite, open_spot(&c, game.feet(), &[3.0, 4.0, 2.5]));
    let (mut latched, mut drag, mut gnawed) = (false, 1.0f32, 0.0f32);
    for _ in 0..(8 * 60) {
        let hp = game.hp_now();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        let (l, d) = game.mob_latched(0);
        if latched && l {
            gnawed += hp - game.hp_now();
        }
        if l {
            drag = drag.min(d);
        }
        latched |= l;
        if latched && gnawed > 0.0 {
            break;
        }
    }
    assert!(latched, "midden hang seg aldri fast");
    assert!(drag < 0.95, "midden i ankelen gjorde deg ikke treg ({drag})");
    assert!(gnawed > 0.0, "midden i ankelen gnagde ikke");
    // Et hopp rister den av.
    input.jump = true;
    for _ in 0..3 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        input.jump = false;
    }
    assert!(!game.mob_latched(0).0, "hoppet ristet ikke midden av");
    assert!((game.mob_latched(0).1 - 1.0).abs() < 1e-3, "farten kom ikke tilbake");
}

/// Den fryktinngytende Button: hammerslaget med begge nevene treffer og
/// trykker deg ned, så du står fast og ikke kommer noen vei, og slipper deg
/// etter `PRESS_SECS`.
#[test]
fn den_fryktinngytende_button_trykker_deg_ned() {
    use crate::domain::mobs::{press, Kind};
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let feet = game.feet();
    let c = game.crypt_now().clone();
    let at = open_spot(&c, feet, &[2.2, 2.0, 2.5]);
    game.only_mob(Kind::DreadButton, at);
    game.turn_mob(0, feet);
    let to = (at - feet).with_y(0.0).normalize();
    game.set_look((-to.x).atan2(-to.z), 0.0);
    let (mut hit, mut pressed) = (0.0f32, false);
    for _ in 0..(8 * 60) {
        game.heal();
        let hp = game.hp_now();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
        hit = hit.max(hp - game.hp_now());
        if game.pressed_now() {
            pressed = true;
            break;
        }
    }
    assert!(hit > 0.0, "hammerslaget traff ikke");
    assert!(pressed, "hammerslaget trykket deg ikke ned");
    // Trykket ned: du prøver å gå bort, men står fast.
    let from = game.feet();
    input.back = true;
    for _ in 0..30 {
        game.heal();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    assert!(game.feet().with_y(0.0).distance(from.with_y(0.0)) < 0.05, "du kom deg unna mens du var trykket ned");
    for _ in 0..((press::PRESS_SECS * 60.0) as usize) {
        game.heal();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let stuck = game.feet();
    for _ in 0..30 {
        game.heal();
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    assert!(game.feet().with_y(0.0).distance(stuck.with_y(0.0)) > 0.3, "du kom deg ikke løs etter trykket");
}

#[test]
fn markoren_synes_og_treffet_sier_hvor_den_kom_fra() {
    use crate::domain::mobs::Kind;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.use_first(|u| *u == Use::Descend);
    for _ in 0..30 {
        game.update(&mut input, 1.0 / 60.0, &mut physics);
    }
    let feet = game.feet();
    let c = game.crypt_now().clone();
    // Et sted 6–9 m unna med fritt gulv hele veien.
    let clear = |a: Vec3, b: Vec3| (1..20).all(|k| c.walkable(Vec3::new(a.lerp(b, k as f32 / 20.0).x, c.height_at(a.lerp(b, k as f32 / 20.0) + Vec3::Y * 0.5), a.lerp(b, k as f32 / 20.0).z)));
    let at = (0..32)
        .flat_map(|k| [6.0f32, 7.5, 9.0].map(move |d| (k, d)))
        .map(|(k, d)| {
            let a = k as f32 * std::f32::consts::TAU / 32.0;
            let q = feet + Vec3::new(a.cos(), 0.0, a.sin()) * d;
            Vec3::new(q.x, c.height_at(q + Vec3::Y * 0.5), q.z)
        })
        .find(|q| c.walkable(*q) && (q.y - feet.y).abs() < 0.3 && clear(feet, *q))
        .expect("ingen plass til inputen 6–9 m unna startstedet i nivå 1");
    game.only_mob(Kind::Input, at);
    // Snu deg så inputen står til høyre for deg.
    let to = (at - feet).with_y(0.0).normalize();
    let fwd = Vec3::new(to.z, 0.0, -to.x);
    game.set_look((-fwd.x).atan2(-fwd.z), 0.0);
    let (mut t, mut charge_at, mut spit_at, mut flying, mut hurt) = (0.0f32, None, None, 0, None);
    let mut glow_before = false;
    let mut foe_ranged = false;
    let mut trail = false;
    for _ in 0..(8 * 60) {
        let events = game.update(&mut input, 1.0 / 60.0, &mut physics);
        t += 1.0 / 60.0;
        for e in &events {
            match e {
                Event::Sound(s) if s.name == "spit_charge" && charge_at.is_none() => charge_at = Some(t),
                Event::Sound(s) if s.name == "spit" && spit_at.is_none() => spit_at = Some(t),
                Event::Hurt { angle } => hurt = Some(*angle),
                _ => {}
            }
        }
        let frame = game.frame(16.0 / 9.0);
        let drawn = frame.instances.iter().filter(|i| i.model == "bolt").count();
        let bolts = game.bolts_flying();
        if charge_at.is_some() && spit_at.is_none() && drawn > 0 {
            glow_before = true;
        }
        if !bolts.is_empty() {
            flying += 1;
            assert!(drawn >= bolts.len(), "markøren i lufta ved {:?} ble ikke tegnet", bolts[0]);
            trail |= drawn > bolts.len();
            assert!(frame.lights.iter().any(|l| bolts.iter().any(|b| l.pos.distance(*b) < 0.01)), "markøren lyser ikke");
        }
        if hurt.is_some() {
            break;
        }
    }
    let (charge_at, spit_at) = (charge_at.expect("ingen ladelyd før skuddet"), spit_at.expect("inputen skjøt ikke"));
    assert!(spit_at - charge_at >= 0.4, "varselet kom bare {:.2} s før skuddet", spit_at - charge_at);
    assert!(glow_before, "ingen glød i munnen mens den ladet");
    assert!(flying >= 10, "markøren var bare i lufta i {flying} bilder");
    assert!(trail, "markøren hadde ingen hale av gnister");
    let angle = hurt.expect("markøren traff deg ikke");
    assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 0.35, "skuddet kom fra høyre, men vinkelen var {angle:.2}");
    // Infopanelet: sikt på den.
    for _ in 0..30 {
        let mob = game.nearest_mob(game.feet()).expect("inputen");
        let d = (mob + Vec3::Y * 0.7 - game.eye()).normalize();
        game.set_look((-d.x).atan2(-d.z), d.y.asin());
        let events = game.update(&mut input, 1.0 / 60.0, &mut physics);
        foe_ranged |= events.iter().any(|e| matches!(e, Event::Foe(Some(f)) if f.ranged));
    }
    assert!(foe_ranged, "infopanelet sa ikke at inputen skyter på avstand");
}
