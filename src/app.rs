//! Applikasjonslaget: setter sammen kjernen med portene, og er det de
//! drivende adapterne (nettleser, eget vindu, skjermbilde) snakker med.

use crate::domain::city::{self, City};
use crate::domain::game::{Event, Game, Input};
use crate::domain::installations;
use crate::domain::model::{LayerImages, Model, ModelInfo};
use crate::domain::ports::{ModelSource, Physics, Renderer, TextureSource};
use crate::domain::geometry::Layer;
use crate::domain::text::Font;
use crate::domain::tokens::Tokens;

/// Alt som må være klart før rendereren kan lages: byen og modellene.
pub struct Prepared {
    pub tokens: Tokens,
    pub city: City,
    pub models: Vec<Model>,
    /// Fotoskannede materialer per teksturlag, der de finnes.
    pub layers: Vec<(Layer, LayerImages)>,
    pub installations: Vec<Box<dyn installations::Installation>>,
    pub font: Font,
}

pub fn prepare(models: &dyn ModelSource, textures: &dyn TextureSource) -> Prepared {
    let tokens = Tokens::load();
    let font = Font::load();
    let sheets = crate::domain::sheets::Sheets::load();
    let mut installations = installations::all();
    // Modellene lastes først, så byen vet hvilke som finnes og hvor store de er.
    let mut loaded: Vec<Model> = models.names().iter().filter_map(|n| models.load(n)).collect();
    // Gresstusten er en prosedyrisk modell, instansiert over enga.
    loaded.push(city::nature::grass_model());
    loaded.push(crate::domain::combat::debris_model());
    let infos: Vec<ModelInfo> = loaded
        .iter()
        .map(|m| {
            let (min, max) = m.mesh.bounds();
            ModelInfo { name: m.name.clone(), min, max }
        })
        .collect();
    let city = city::build(&tokens, &font, &sheets, &infos, &mut installations);
    log::info!("Byen: {} hjørner, {} props, {} modeller tilgjengelig", city.mesh.vertices.len(), city.props.len(), infos.len());
    let models: Vec<Model> = loaded.into_iter().filter(|m| city.placements.iter().any(|p| p.model == m.name)).collect();
    // Panelet holdes bakt: malte bord skal være rene, og de skannede plankene
    // er for værbitte til å tones med malingsfargen.
    let layers: Vec<(Layer, LayerImages)> = Layer::ALL.iter().filter(|l| **l != Layer::Cladding).filter_map(|l| textures.layer(l.name()).map(|i| (*l, i))).collect();
    if !layers.is_empty() {
        log::info!("Fotoskannede materialer: {}", layers.iter().map(|(l, _)| l.name()).collect::<Vec<_>>().join(", "));
    }
    Prepared { tokens, city, models, layers, installations, font }
}

#[cfg(test)]
mod tests {
    /// Rutinen mot svevende modeller og tekst utenfor skilt: byen bygges
    /// med de innebygde modellene (og de eksterne om de er hentet); ingen
    /// instans får sveve over eller stå nede i terrenget, og ingen tekst
    /// meldt med `Builder::fits` får være bredere enn plata. `cargo test` før push.
    #[test]
    fn modellene_star_pa_bakken() {
        std::env::set_var("TEXT_AUDIT", "1");
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        assert!(prepared.city.audit.is_empty(), "\n{}", prepared.city.audit.join("\n"));
    }

    /// Ingen skal kunne gå gjennom noe: hver modellinstans som er høy nok
    /// til å gå inn i (over 60 cm) og bred nok til å merke (over 30 cm) må
    /// ha en kollider rundt foten: et punkt 50 cm over bunnen ved origo
    /// skal ligge i (eller innen 35 cm fra) en boks, sylinder
    /// eller kapsel. Bladverk, gress og det som henger på vegger er unntatt.
    #[test]
    fn alt_stort_har_kollider() {
        use crate::domain::city::Shape;
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        let tol = 0.35;
        let covers = |shape: &Shape, p: glam::Vec3| -> bool {
            match shape {
                Shape::Cuboid { center, size, yaw } => {
                    let d = glam::Quat::from_rotation_y(-*yaw) * (p - *center);
                    d.x.abs() <= size.x * 0.5 + tol && d.y.abs() <= size.y * 0.5 + tol && d.z.abs() <= size.z * 0.5 + tol
                }
                Shape::Cylinder { center, radius, half_height } => {
                    let d = p - *center;
                    (d.x * d.x + d.z * d.z).sqrt() <= radius + tol && d.y.abs() <= half_height + tol
                }
                Shape::Capsule { a, b, radius } => {
                    let ab = *b - *a;
                    let t = ((p - *a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
                    (p - (*a + ab * t)).length() <= radius + tol
                }
                Shape::Heightfield { .. } | Shape::Hull { .. } => false,
            }
        };
        let skip = |name: &str| name.ends_with("_leaves") || name.ends_with("_needles") || name == "grass" || name.starts_with("window_") || name == "door_lamp" || name == "flower_box";
        let mut problems = vec![];
        for pl in &prepared.city.placements {
            if skip(pl.model) {
                continue;
            }
            let Some(model) = prepared.models.iter().find(|m| m.name == pl.model) else { continue };
            let (lo, hi) = model.mesh.bounds();
            for t in &pl.transforms {
                // Boksen rundt instansen i verden.
                let mut wlo = glam::Vec3::splat(f32::MAX);
                let mut whi = glam::Vec3::splat(f32::MIN);
                for k in 0..8 {
                    let c = glam::Vec3::new(if k & 1 == 0 { lo.x } else { hi.x }, if k & 2 == 0 { lo.y } else { hi.y }, if k & 4 == 0 { lo.z } else { hi.z });
                    let w = t.transform_point3(c);
                    wlo = wlo.min(w);
                    whi = whi.max(w);
                }
                let size = whi - wlo;
                if size.y < 0.6 || size.x.max(size.z) < 0.3 {
                    continue;
                }
                // Origo står ved foten (trær, kraner har tyngdepunktet langt unna).
                let o = glam::Vec3::from(t.translation);
                let p = glam::Vec3::new(o.x, wlo.y + 0.5, o.z);
                if !prepared.city.colliders.iter().any(|c| covers(c, p)) {
                    problems.push(format!("{} ved ({:.1}, {:.1}, {:.1}) har ingen kollider", pl.model, p.x, p.y, p.z));
                }
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    /// Og fysikken må faktisk stoppe deg: kapselen går mot en trestamme og
    /// mot kanten av verden, og skal bli stående innenfor begge.
    #[test]
    fn kapselen_stopper_mot_trær_og_kanten() {
        use crate::domain::ports::Physics;
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        let mut physics = crate::adapters::rapier::RapierPhysics::new();
        for c in &prepared.city.colliders {
            physics.add(c);
        }
        let (half, radius) = (0.5f32, 0.3f32);
        // Gå rett mot et mål fra `from`, 0,1 m om gangen; gir sluttpunktet og
        // det nærmeste vi kom `mark` (vannrett).
        let mut walk = |from: glam::Vec3, to: glam::Vec3, mark: glam::Vec3| -> (glam::Vec3, f32) {
            let mut pos = from;
            let dir = (to - from).normalize();
            let mut nearest = f32::MAX;
            for _ in 0..((to - from).length() / 0.1) as usize {
                let (moved, _) = physics.move_capsule(pos, half, radius, dir * 0.1 - glam::Vec3::Y * 0.05, 0.016);
                pos += moved;
                nearest = nearest.min((pos - mark).with_y(0.0).length());
            }
            (pos, nearest)
        };
        // Eiker: stammen står i instansens origo. Kapselen (radius 0,3) skal
        // aldri komme nærmere enn stammens radius pluss sin egen.
        let oak = prepared.city.placements.iter().find(|p| p.model == "oak1_trunk" && p.transforms.len() > 3).expect("ingen eik");
        for t in oak.transforms.iter().take(4) {
            let base = glam::Vec3::from(t.translation);
            let y = half + radius + 0.1;
            let (_, nearest) = walk(base + glam::Vec3::new(2.0, y, 0.0), base + glam::Vec3::new(-2.0, y, 0.0), base);
            assert!(nearest > 0.45, "gikk gjennom eika ved ({:.1}, {:.1}): kom {:.2} m fra stammen", base.x, base.z, nearest);
        }
        // Kanten av verden: fra 185 m ute, rett utover; veggen ved 199 m skal stoppe oss.
        let start = glam::Vec3::new(185.0, crate::domain::city::terrain::height(185.0, 0.0) + half + radius + 0.1, 0.0);
        let (end, _) = walk(start, glam::Vec3::new(215.0, start.y, 0.0), glam::Vec3::ZERO);
        assert!(end.x < 199.0, "gikk ut av verden: endte ved x = {:.1}", end.x);
    }

    /// Alt du kan sikte på må kunne treffes: for hver prop med kollider
    /// (som ikke er en bevegelig del) skytes en stråle fra 60 cm rett foran
    /// den, og det første den treffer må være en prop fra samme
    /// installasjon. Ellers ligger en tavle eller vegg i veien, og
    /// komponenten er «død» i spillet.
    #[test]
    fn alt_du_kan_sikte_pa_kan_treffes() {
        use crate::domain::ports::Physics;
        use std::collections::HashMap;
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        let mut physics = crate::adapters::rapier::RapierPhysics::new();
        for c in &prepared.city.colliders {
            physics.add(c);
        }
        // Propsene slik installasjonene poserer dem i ro (gjemte varianter,
        // lukkede lister), som i spillet: kolliderne følger posen.
        let env = crate::domain::installations::Env::default();
        let posed: Vec<glam::Affine3A> = prepared.city.props.iter().enumerate().map(|(i, p)| match p.owner { Some(o) => prepared.installations[o].pose(i, p.rest, &env).0, None => p.rest }).collect();
        let mut by_collider = HashMap::new();
        for (i, p) in prepared.city.props.iter().enumerate() {
            if let Some((offset, half)) = p.collider {
                let id = physics.add_movable(half, posed[i] * glam::Affine3A::from_translation(offset));
                by_collider.insert(id, i);
            }
        }
        // Spørrestrukturen bygges først når noe beveger seg (som i spillet).
        physics.move_capsule(glam::Vec3::new(0.0, 50.0, 0.0), 0.5, 0.3, glam::Vec3::ZERO, 0.016);
        let mut problems = vec![];
        for (i, p) in prepared.city.props.iter().enumerate() {
            let (Some((offset, _)), Some(owner), false) = (p.collider, p.owner, p.part) else { continue };
            // Gjemte props (skala ~0) kan ikke siktes på, og skal ikke.
            if posed[i].matrix3.x_axis.length_squared() < 1e-6 {
                continue;
            }
            let center = posed[i].transform_point3(offset);
            let facing = posed[i].transform_vector3(glam::Vec3::Z).normalize_or_zero();
            if facing.length_squared() < 0.5 {
                continue;
            }
            // Fra 60 cm foran, så smug og vegger på andre siden ikke forstyrrer.
            let from = center + facing * 0.6;
            let hit = physics.cast_ray(from, -facing, 0.9);
            let ok = hit.as_ref().and_then(|h| by_collider.get(&h.collider)).is_some_and(|j| prepared.city.props[*j].owner == Some(owner));
            if !ok {
                let name = prepared.installations[owner].component().name;
                problems.push(format!("{} ({}) ved ({:.1}, {:.1}, {:.1}) kan ikke siktes på: {}", name, p.name, center.x, center.y, center.z, match hit { Some(h) => format!("strålen traff noe annet {:.2} m unna: {}", h.distance, by_collider.get(&h.collider).map(|j| format!("prop {} ({})", prepared.city.props[*j].name, prepared.city.props[*j].owner.map(|o| prepared.installations[o].component().name).unwrap_or("-"))).unwrap_or_else(|| prepared.city.colliders.get((h.collider.0 - 1) as usize).map(|c| format!("{c:?}").chars().take(160).collect()).unwrap_or_default())), None => "strålen traff ingenting".into() }));
            }
            let _ = i;
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    /// Brønnen tar deg ned i kjelleren, og stigen opp igjen; fiendene
    /// kommer, biter, og kan slås i hjel med F.
    #[test]
    fn kjelleren_kan_nas_og_fiendene_biter() {
        use crate::domain::city::dungeon::DEPTH;
        use crate::domain::game::{Action, Event};
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        let mut physics = crate::adapters::rapier::RapierPhysics::new();
        let mut game = crate::domain::game::Game::new(prepared.tokens, prepared.font, &prepared.city, prepared.installations, &mut physics, super::rigs(&prepared.models));
        let mut input = crate::domain::game::Input::default();
        game.update(&mut input, 1.0 / 30.0, &mut physics);
        assert!(game.eye().y > 0.0, "starter oppe");
        // Alle slagene er med: knapper øverst, tooltips, inputer og
        // modaler nedover, tabellen som elite nederst, og én boss.
        let roster = game.roster();
        for k in crate::domain::mobs::Kind::ALL {
            assert!(roster.iter().any(|(r, _)| *r == k), "ingen {k:?} i kjelleren");
        }
        assert_eq!(roster.iter().filter(|(k, _)| k.boss()).count(), 1, "akkurat én boss");
        assert!(roster.iter().any(|(k, e)| *k == crate::domain::mobs::Kind::Table && *e), "tabellen er elite nederst");
        // De dype etasjene er under −20 m: du skal kunne stå der uten å
        // bli sendt tilbake til torget som «falt gjennom verden».
        let deep = &prepared.city.dungeon.floors[prepared.city.dungeon.floors.len() - 1];
        let there = deep.center(deep.start.0, deep.start.1);
        game.place(there + glam::Vec3::Y * 1.7, 0.0, 0.0);
        for _ in 0..30 {
            game.update(&mut input, 1.0 / 30.0, &mut physics);
        }
        assert!((game.eye().y - there.y - 1.7).abs() < 1.0, "sto ikke igjen nederst: øyet er på y = {:.1}, gulvet på {:.1}", game.eye().y, there.y);
        game.place(crate::domain::city::terrain::at(prepared.city.spawn) + glam::Vec3::Y * 1.7, prepared.city.spawn_yaw, 0.0);
        game.update(&mut input, 1.0 / 30.0, &mut physics);
        game.use_component("cellar");
        let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
        assert!(game.eye().y < DEPTH + 3.0, "F på brønnen skal ta deg ned: øyet er på y = {:.1}", game.eye().y);
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
            let feet = game.eye() - glam::Vec3::Y * 0.8;
            if let Some(m) = game.nearest_mob(feet) {
                let d = m + glam::Vec3::Y * 0.8 - game.eye();
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
        // Og opp igjen med stigen.
        game.use_component("cellar#0");
        let _ = game.update(&mut input, 1.0 / 30.0, &mut physics);
    }

    /// Smiinga: med komponentene i lageret godtas skjemaet for komponentsverdet
    /// og våpenet byttes; uten dem avvises det. Kjøp koster tokens, og
    /// lagringen tar med utstyret.
    #[test]
    fn skjemaet_smir_utstyr() {
        use crate::domain::game::{Event, SaveData};
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        let mut physics = crate::adapters::rapier::RapierPhysics::new();
        let mut game = crate::domain::game::Game::new(prepared.tokens, prepared.font, &prepared.city, prepared.installations, &mut physics, super::rigs(&prepared.models));
        let mut input = crate::domain::game::Input::default();
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
        let mut fresh = crate::domain::game::Game::new(crate::domain::tokens::Tokens::load(), crate::domain::text::Font::load(), &prepared.city, crate::domain::installations::all(), &mut physics, vec![]);
        fresh.load(SaveData { coins: 50, ..saved });
        assert_eq!(fresh.gear().weapon.id, "component_sword");
        assert!(fresh.buy("potion"), "med tokens går kjøpet");
    }

    /// Variantene skal virke 1:1 med designsystemet: alle installasjonene
    /// som er samme komponent må ha de samme variantene (ellers bytter R og
    /// panelet bare noen av dem), og det du sikter på må endre seg for hver
    /// variant: for hver prop som kan siktes på, må de synlige propsene
    /// innen 2,5 m (hvilke, og hvor de står) være ulike for alle variantene.
    /// Palettfarger teller ikke: variantene skal være egne ark fra CSS-en.
    #[test]
    fn variantene_endrer_det_du_sikter_pa() {
        use crate::domain::installations::Env;
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let mut prepared = super::prepare(&assets, &assets);
        let mut env = Env::default();
        env.dt = 0.5;
        let mut problems = vec![];
        let names: Vec<String> = prepared.installations.iter().map(|i| i.component().name.to_string()).collect();
        for (i, inst) in prepared.installations.iter_mut().enumerate() {
            let name = &names[i];
            let variants = inst.variants().to_vec();
            if variants.is_empty() {
                continue;
            }
            let mine: Vec<usize> = (0..prepared.city.props.len()).filter(|p| prepared.city.props[*p].owner == Some(i)).collect();
            // Posene per variant, etter at animasjonene har fått roe seg.
            let mut poses: Vec<Vec<(usize, glam::Affine3A)>> = vec![];
            for k in 0..variants.len() {
                inst.set_variant(k);
                for _ in 0..20 {
                    inst.tick(&mut env);
                }
                poses.push(mine.iter().map(|p| (*p, inst.pose(*p, prepared.city.props[*p].rest, &env).0)).filter(|(_, pose)| pose.matrix3.x_axis.length_squared() > 1e-6).collect());
            }
            inst.set_variant(0);
            // Hver prop som kan siktes på: det synlige rundt den må endre seg.
            for p in &mine {
                if prepared.city.props[*p].part {
                    continue;
                }
                let Some((offset, _)) = prepared.city.props[*p].collider else { continue };
                let at = prepared.city.props[*p].rest.transform_point3(offset);
                let mut sigs: Vec<(String, String)> = vec![];
                for (k, v) in variants.iter().enumerate() {
                    let mut sig = String::new();
                    for (q, pose) in &poses[k] {
                        if pose.translation.distance(at.into()) > 2.5 {
                            continue;
                        }
                        let m = pose.to_cols_array();
                        sig += &format!("{q}:{:?};", m.iter().map(|x| (x * 1000.0).round() as i64).collect::<Vec<_>>());
                    }
                    if let Some((other, _)) = sigs.iter().find(|(_, s)| *s == sig) {
                        problems.push(format!("{name} (installasjon {i}, propen «{}» ved ({:.1}, {:.1}, {:.1})): varianten «{v}» ser ut som «{other}»", prepared.city.props[*p].name, at.x, at.y, at.z));
                    }
                    sigs.push((v.to_string(), sig));
                }
            }
        }
        // Samme komponent, samme varianter.
        let lists: Vec<Vec<&str>> = prepared.installations.iter().map(|i| i.variants().to_vec()).collect();
        for i in 0..names.len() {
            for j in 0..i {
                if names[i] == names[j] && lists[i] != lists[j] {
                    problems.push(format!("{}: installasjon {j} har variantene {:?}, installasjon {i} har {:?}", names[i], lists[j], lists[i]));
                }
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }
}

/// Skjelettene til de skinnede modellene, til spillet.
pub fn rigs(models: &[Model]) -> Vec<(String, crate::domain::anim::Rig)> {
    models.iter().filter_map(|m| m.rig.clone().map(|r| (m.name.clone(), r))).collect()
}

pub struct App<R: Renderer, P: Physics> {
    pub game: Game,
    pub input: Input,
    pub renderer: R,
    pub physics: P,
}

impl<R: Renderer, P: Physics> App<R, P> {
    pub fn new(prepared: Prepared, renderer: R, mut physics: P) -> Self {
        let game = Game::new(prepared.tokens, prepared.font, &prepared.city, prepared.installations, &mut physics, rigs(&prepared.models));
        Self { game, input: Input::default(), renderer, physics }
    }

    /// Ett steg av spillet. Gir hendelsene til verten.
    pub fn tick(&mut self, dt: f32) -> Vec<Event> {
        let events = self.game.update(&mut self.input, dt, &mut self.physics);
        // Tekst som ble skrevet: propsene får ny geometri.
        for (i, mesh) in self.game.take_dirty() {
            self.renderer.update_prop(i, &mesh);
        }
        events
    }

    /// Tegn bildet.
    pub fn draw(&mut self, aspect: f32) {
        let frame = self.game.frame(aspect);
        self.renderer.render(&frame);
        self.game.settle();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }
}
