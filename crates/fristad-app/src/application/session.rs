//! Økta: spilleren i verden. Koordinerer reglene i kjernen (kroppen,
//! helten, mekanismene, portalen, kartet, turen i krypten, kampen,
//! fiendene, innbyggerne, dyrene og himmelen) med fysikken gjennom porten
//! `Physics`, og svarer med hendelser (`ports::driving::Event`).
//!
//! Økta er delt etter oppgave: `usage` (F på det du sikter på, død, nivåer),
//! `prompt` (siktet), `fight` (kampen og merkene over fiendene), `audio`
//! (lydene og lydbildet), `trade` (lageret, butikkene, smiinga og lagringen),
//! `view` (bildet) og `testing` (hjelperne testene bruker).

use crate::domain::body::{Body, Stance, EYE};
use crate::domain::combat::Combat;
use crate::domain::herd::Herd;
use crate::domain::hero::Hero;
use crate::domain::mobs::Mobs;
use crate::domain::note::Note;
use crate::domain::npcs::Npcs;
use crate::domain::scene::Light;
use crate::domain::weather::{Daylight, Sky, WeatherKind};
use crate::domain::world::crypt::map::Map;
use crate::domain::world::crypt::run::{Happening, Run};
use crate::domain::world::crypt::{self, Crypt};
use crate::domain::world::mechanisms::{Mechanisms, Moved};
use crate::domain::world::portal::Portal;
use crate::domain::world::{terrain, town};
use crate::domain::world::{Target, Use, World};
use crate::ports::driving::{Action, Bubble, Event, Input, MapMark, MapView, Prompt, Shot, Sound};
use crate::ports::{ColliderId, Physics};
use glam::{Affine3A, Vec3};
use std::collections::HashMap;

use super::audio::Soundscape;
use super::fight::Hud;

/// Så langt unna kan du sikte på en fiende og få infopanelet (meter).
const FOE_SIGHT: f32 = 30.0;

/// En lyd i hodet: dine egne skritt, hugg og stønn.
pub(super) fn sfx(name: &str) -> Event {
    Event::Sound(Sound::new(name))
}

/// Radianer per piksel musa flytter seg.
const SENSITIVITY: f32 = 0.0035;

/// Noe du kan bruke, med kollideren strålen treffer.
pub(super) struct TargetState {
    pub target: Target,
    pub collider: ColliderId,
}

/// Langt under verden: der kolliderne venter til de trengs.
fn away() -> Affine3A {
    Affine3A::from_translation(Vec3::new(0.0, -5000.0, 0.0))
}

pub struct Session {
    pub(super) lights: Vec<Light>,
    /// Pipene som ryker (toppen, tykkelsen).
    pub(super) smoke: Vec<(Vec3, f32)>,
    pub(super) targets: Vec<TargetState>,
    pub(super) by_collider: HashMap<ColliderId, usize>,
    pub(super) body: Body,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) time: f32,
    pub(super) sky: Sky,
    /// Det du sikter på (indeks i `targets`), fienden du sikter på, og
    /// siktet slik det ble sendt sist.
    pub(super) aim: Option<usize>,
    pub(super) aimed_mob: Option<usize>,
    pub(super) last_prompt: Option<Prompt>,
    pub paused: bool,
    /// Filmen styrer kameraet (`Command::Film`).
    pub film: Option<Shot>,
    pub(super) spawn: Vec3,
    pub(super) spawn_yaw: f32,
    pub(super) crypt_exit: (Vec3, f32),
    pub(super) portal: Portal,
    /// Kolliderne til portalringen (flyttes inn når den står der).
    pub(super) portal_solids: Vec<ColliderId>,
    pub(super) map: Map,
    pub(super) herd: Herd,
    pub(super) sound: Soundscape,
    pub(super) npcs: Npcs,
    pub(super) bubbles_shown: bool,
    pub(super) crypt: Crypt,
    pub(super) mobs: Mobs,
    pub(super) mech: Mechanisms,
    /// Kolliderne til fallemmene som holder deg til de faller.
    pub(super) trap_lids: Vec<Option<ColliderId>>,
    /// Dekkene på broene over avgrunnene (flyttes når broa senkes).
    pub(super) bridge_solids: Vec<ColliderId>,
    /// Dekkene på heisene (bevegelige kollidere).
    pub(super) lift_solids: Vec<ColliderId>,
    /// Hvor lenge siden du sist fikk høre at vannet er for dypt (`water.rs`).
    pub(super) wade_told: f32,
    pub(super) run: Run,
    pub(super) hero: Hero,
    pub(super) loot_seed: u64,
    /// Hendelser fra kommandoer (kjøp, smiing, lesing), til neste `update`.
    pub(super) pending: Vec<Event>,
    /// Målet og antallet leste lapper sist de ble sendt (`goals.rs`).
    pub(super) goal_sent: Option<(String, usize)>,
    pub(super) combat: Combat,
    pub(super) hud: Hud,
    /// Bildeforholdet fra sist `frame`, til å regne skjermposisjoner.
    pub(super) aspect: std::cell::Cell<f32>,
    /// Hvor mye du er i kjelleren (0 oppe, 1 nede), glattet, til lyset.
    pub(super) cellar_t: f32,
    /// Rammeverket er beseiret (denne turen).
    pub won: bool,
    pub debug: u32,
    /// Redigeringen (`editing.rs`), når du er i den, og dataene den
    /// begynner fra (bare med `prepare_for_edit`).
    pub(super) edit: Option<crate::domain::edit::Editor>,
    pub edit_data: Option<super::assembly::EditData>,
    pub(super) edit_ui: super::editing::EditUi,
}

impl Session {
    /// `rigs` er skjelettene til de skinnede modellene (navn, rigg).
    pub fn new(world: &World, physics: &mut dyn Physics, rigs: &[(String, crate::domain::anim::Rig)]) -> Self {
        for c in &world.colliders {
            physics.add(c);
        }
        let mut targets = vec![];
        let mut by_collider = HashMap::new();
        for (i, t) in world.targets.iter().enumerate() {
            // Arenaportene står oppe fra start.
            let lift = match t.what {
                Use::Gate(n) if world.crypt.gates.get(n).is_some_and(|g| g.arena.is_some()) => Vec3::Y * crate::domain::world::crypt::GATE_LIFT,
                _ => Vec3::ZERO,
            };
            let collider = physics.add_movable(t.half, t.pose * Affine3A::from_translation(t.center + lift));
            by_collider.insert(collider, i);
            targets.push(TargetState { target: *t, collider });
        }
        let rig_of = |n: &str| rigs.iter().find(|(m, _)| m == n).map(|(_, r)| r.clone());
        let wells = world.targets.iter().filter(|t| matches!(t.what, Use::Well(_))).count();
        Self {
            lights: world.lights.clone(),
            smoke: world.smoke.clone(),
            targets,
            by_collider,
            body: Body::new(terrain::at(world.spawn)),
            yaw: world.spawn_yaw,
            pitch: 0.05,
            time: 0.0,
            sky: Sky::default(),
            aim: None,
            aimed_mob: None,
            last_prompt: None,
            paused: false,
            film: None,
            spawn: world.spawn,
            spawn_yaw: world.spawn_yaw,
            crypt_exit: world.crypt_exit,
            portal: Portal::new(world.crypt.levels.len()),
            portal_solids: town::PORTAL_SOLIDS.iter().map(|(_, size)| physics.add_movable(*size * 0.5, away())).collect(),
            map: Map::new(&world.crypt),
            herd: Herd::new(),
            sound: Soundscape::new(world),
            npcs: Npcs::new(rig_of, world.npcs.iter().map(|n| (n.name, n.model, n.pos, n.yaw, n.tint, n.lines, n.route.clone(), n.height)).collect()),
            bubbles_shown: false,
            mobs: Mobs::new(&world.crypt, rigs),
            mech: Mechanisms::new(&world.crypt, wells),
            trap_lids: world
                .crypt
                .trapdoors
                .iter()
                .map(|t| t.lid.then(|| physics.add_movable(Vec3::new(t.size.x * 0.5, 0.05, t.size.y * 0.5), crate::domain::world::at(t.pos - Vec3::Y * 0.05, t.yaw))))
                .collect(),
            bridge_solids: {
                let m = Mechanisms::new(&world.crypt, 0);
                (0..world.crypt.bridges.len())
                    .map(|n| {
                        let b = &world.crypt.bridges[n];
                        physics.add_movable(Vec3::new(b.size.x * 0.5, 0.15, b.size.y * 0.5), m.bridge_collider(&world.crypt, n).unwrap_or_else(away))
                    })
                    .collect()
            },
            lift_solids: {
                let m = Mechanisms::new(&world.crypt, 0);
                (0..world.crypt.lifts.len())
                    .map(|n| {
                        let l = &world.crypt.lifts[n];
                        physics.add_movable(Vec3::new(l.size.x * 0.5, 0.15, l.size.y * 0.5), m.lift_collider(&world.crypt, n).unwrap_or_else(away))
                    })
                    .collect()
            },
            wade_told: f32::MAX,
            run: Run::new(&world.crypt),
            crypt: world.crypt.clone(),
            hero: Hero::default(),
            loot_seed: 0x2545_f491_4f6c_dd1d,
            pending: vec![],
            goal_sent: None,
            combat: Combat::new(rig_of(crate::domain::combat::ARMS_MODEL)),
            hud: Hud::default(),
            aspect: std::cell::Cell::new(16.0 / 9.0),
            cellar_t: 0.0,
            won: false,
            debug: 0,
            edit: None,
            edit_data: None,
            edit_ui: Default::default(),
        }
    }

    /// Det reglene har å si, som hendelser.
    pub(super) fn say(&self, notes: impl IntoIterator<Item = Note>, events: &mut Vec<Event>) {
        for n in notes {
            events.push(match n {
                Note::Sfx(name) => sfx(name),
                Note::At(name, at, gain) => self.sound_at(name, at, gain),
                Note::Toast(text) => Event::Toast(text),
                Note::Fade => Event::Fade,
            });
        }
    }

    /// Midten av et mål i verden.
    pub(super) fn target_pos(&self, i: usize) -> Vec3 {
        let t = &self.targets[i].target;
        t.pose.transform_point3(t.center)
    }

    pub fn weather_kind(&self) -> WeatherKind {
        self.sky.kind
    }

    pub fn daylight(&self) -> Daylight {
        self.sky.daylight
    }

    pub fn set_daylight(&mut self, d: Daylight, instant: bool) {
        self.sky.set_daylight(d, instant);
    }

    pub fn set_weather(&mut self, kind: WeatherKind, instant: bool) {
        self.sky.set_weather(kind, instant);
    }

    /// Snu kameraet uten å flytte det.
    pub fn set_look(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
    }

    /// Plasser kameraet, til skjermbilder og tester. `eye` er øyet.
    pub fn place(&mut self, eye: Vec3, yaw: f32, pitch: f32) {
        self.body.pos = eye - Vec3::Y * EYE;
        self.body.jumped = true;
        self.yaw = yaw;
        self.pitch = pitch;
    }

    /// Tilbake til sørporten.
    pub fn respawn(&mut self) {
        self.body.teleport(terrain::at(self.spawn) + Vec3::Y * 0.05);
        self.yaw = self.spawn_yaw;
    }

    /// Rett dit, uten fysikk på veien: føttene og retningen.
    pub(super) fn teleport(&mut self, feet: Vec3, yaw: f32) {
        self.body.teleport(feet);
        self.yaw = yaw;
        self.pitch = 0.0;
    }

    pub(super) fn forward(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), -self.yaw.cos() * self.pitch.cos())
    }

    /// Øyet og retningen der du begynner (ved sørporten), til filmen.
    pub fn spawn_eye(&self) -> (Vec3, f32) {
        (Body::new(terrain::at(self.spawn)).eye(), self.spawn_yaw)
    }

    /// Startstedet (føttene) og retningen i kryptnivå `k` (fra 0), til filmen.
    pub fn level_start(&self, k: usize) -> Option<(Vec3, f32)> {
        self.crypt.levels.get(k).map(|l| l.start)
    }

    /// Origo i kryptnivå `k` (fra 0), til filmen.
    pub fn level_origin(&self, k: usize) -> Option<Vec3> {
        self.crypt.levels.get(k).map(|l| l.origin)
    }

    /// Hvor innbyggerne står nå (til testene).
    pub fn npc_positions(&self) -> Vec<Vec3> {
        self.npcs.npcs.iter().map(|n| n.pos).collect()
    }

    /// Snakk med innbygger `i` som om du har nådd `depth` nivåer (til testene).
    pub fn talk_to(&mut self, i: usize, depth: u8) -> Option<String> {
        self.npcs.talk(i, depth).map(|s| s.text)
    }

    /// Flytt spilleren (til testene).
    pub fn place_feet(&mut self, feet: Vec3) {
        self.teleport(feet, 0.0);
    }

    pub fn eye(&self) -> Vec3 {
        self.body.eye()
    }

    pub(super) fn in_crypt(&self) -> bool {
        self.body.pos.y < crypt::TOP
    }

    /// Hendelser som skal ut neste bilde.
    pub fn queue(&mut self, events: Vec<Event>) {
        self.pending.extend(events);
    }

    /// Hvor portalen tar deg nå (føttene, retningen og nivået).
    pub fn portal_destination(&self) -> Option<(Vec3, f32, usize)> {
        self.portal.dest
    }

    /// Kryptnivåene (indekser) som må ha bitene sine nå: der du står og
    /// nivåene over og under, og oppe i byen det første og det portalen tar
    /// deg til. Nivåene hentes før du går ned (plan del 10a).
    pub fn wanted_levels(&self) -> Vec<usize> {
        let n = self.crypt.levels.len();
        let mut out = vec![];
        match self.crypt.level_at(self.body.pos).filter(|_| self.in_crypt()) {
            Some(k) => out.extend(k.saturating_sub(1)..=(k + 1).min(n.saturating_sub(1))),
            None => {
                if n > 0 {
                    out.push(0);
                }
                if let Some((_, _, k)) = self.portal.dest {
                    out.push(k);
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn update(&mut self, input: &mut Input, dt: f32, physics: &mut dyn Physics) -> Vec<Event> {
        let dt = dt.clamp(0.0, 1.0 / 20.0);
        self.time += dt;
        let mut events = std::mem::take(&mut self.pending);
        let actions = std::mem::take(&mut input.actions);

        if !self.paused {
            self.yaw -= input.look.0 * SENSITIVITY;
            self.pitch = (self.pitch - input.look.1 * SENSITIVITY).clamp(-1.5, 1.5);
        }
        input.look = (0.0, 0.0);
        // I redigeringen flyr du, og ingenting annet skjer, men T og V
        // bytter dag og natt og været, så lysene kan ses om natta.
        if self.edit.is_some() {
            self.fly(input, dt);
            self.sky.tick(dt, self.time);
            let in_crypt = self.in_crypt();
            for a in actions.into_iter().filter(|a| matches!(a, Action::Daylight | Action::Weather)) {
                self.act(a, Vec3::ZERO, in_crypt, &mut events);
            }
            return events;
        }
        if let Some(shot) = self.film {
            self.film_tick(shot, dt);
            return events;
        }
        let wish = self.walk(input, dt, physics, &mut events);
        self.update_crypt(dt, &mut events);

        // Hva sikter vi på? Fiendene i krypten går foran.
        let eye = self.eye();
        let dir = self.forward();
        let in_crypt = self.in_crypt();
        self.update_portal(dt, in_crypt, physics);
        self.update_map(in_crypt, &mut events);
        self.update_goal(&mut events);
        let hit = physics.cast_ray(eye, dir, 3.5);
        self.aimed_mob = if in_crypt { self.mobs.aimed(eye, dir, 2.6) } else { None };
        self.aim = if self.aimed_mob.is_some() { None } else { hit.as_ref().and_then(|h| self.by_collider.get(&h.collider).copied()) };
        // Infopanelet viser fienden du sikter på, også på avstand (til
        // første vegg).
        let wall = physics.cast_ray(eye, dir, FOE_SIGHT).map_or(FOE_SIGHT, |h| h.distance);
        let foe = if in_crypt { self.aimed_mob.or_else(|| self.mobs.aimed(eye, dir, wall)) } else { None };
        self.update_foe(foe, dt, &mut events);
        self.combat.set_block(input.block && in_crypt && !self.paused);

        for a in actions {
            self.act(a, wish, in_crypt, &mut events);
        }

        // Siktet: sendes når det endrer seg.
        // En fiende får ikke sikteboblen: infopanelet til høyre viser den
        // (`Event::Foe`, eieren 18. oktober).
        let prompt = match (self.aimed_mob, self.aim) {
            (None, Some(i)) => Some(self.prompt_for(i)),
            (None, None) if in_crypt => self.water_prompt(eye, dir, hit.map_or(4.0, |h| h.distance)),
            _ => None,
        };
        if prompt != self.last_prompt {
            self.last_prompt = prompt.clone();
            events.push(Event::Prompt(prompt));
        }

        // Dag og natt glir over i hverandre, og været mot målet.
        if self.sky.tick(dt, self.time) && !in_crypt {
            events.push(Event::Sound(Sound { name: "thunder".into(), at: None, gain: if self.sound.indoor { 0.6 } else { 1.0 } }));
        }
        // Oppe i dagslyset er det du bar trygt i lageret, uansett hvordan du
        // kom deg opp (døra i trappa, H eller «Tilbake til sørporten»).
        if !self.in_crypt() {
            self.bank(&mut events);
        }
        self.update_mechanisms(physics, dt, &mut events);
        let feet = self.body.sole();
        self.herd.update(dt, feet);
        self.update_people(dt, feet, in_crypt, physics, &mut events);
        self.cellar_t += ((if in_crypt { 1.0 } else { 0.0 }) - self.cellar_t) * (dt * 4.0).min(1.0);
        self.fight(dt, feet, in_crypt, &mut events);
        self.burn_lamp(dt, in_crypt, &mut events);
        self.update_hud(dt, in_crypt, &mut events);
        self.ambience(dt, feet, in_crypt, physics, &mut events);
        events
    }

    /// Et bilde i filmen: kameraet der filmen sier, byfolket og fiendene
    /// lever (fiendene i nivået kameraet er i, og de ser ingen), ingen kamp,
    /// fysikk eller lyder fra spilleren.
    fn film_tick(&mut self, shot: Shot, dt: f32) {
        // Føttene så langt under øyet som når du står (filmen slutter der du står).
        self.body.teleport(shot.eye - Vec3::Y * Body::new(Vec3::ZERO).eye().y);
        self.yaw = shot.yaw;
        self.pitch = shot.pitch;
        self.sky.tick(dt, self.time);
        let in_crypt = self.in_crypt();
        if in_crypt {
            if let Some(k) = self.crypt.level_at(self.body.feet()) {
                if self.mobs.level != k || self.mobs.mobs.is_empty() {
                    self.enter_level(k);
                }
            }
        }
        // Langt over der kameraet står: ingen ser det, og ingen kommer bort.
        let unseen = self.body.feet() + Vec3::Y * 500.0;
        let _ = self.mobs.update(dt, unseen, in_crypt, &self.crypt);
        let _ = self.npcs.update(dt, unseen);
        self.herd.update(dt, unseen);
        self.cellar_t += ((if in_crypt { 1.0 } else { 0.0 }) - self.cellar_t) * (dt * 4.0).min(1.0);
    }

    /// Gåing, stilling, hopp og skritt, og fall utfor kanten i krypten. Gir
    /// retningen du vil gå (til unnamanøveren).
    fn walk(&mut self, input: &Input, dt: f32, physics: &mut dyn Physics, events: &mut Vec<Event>) -> Vec3 {
        let mut wish = Vec3::ZERO;
        if !self.paused {
            let fwd = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
            let right = Vec3::new(-fwd.z, 0.0, fwd.x);
            for (on, d) in [(input.forward, fwd), (input.back, -fwd), (input.right, right), (input.left, -right)] {
                if on {
                    wish += d;
                }
            }
        }
        // C holdes for huk.
        self.body.set_stance(if input.crouch && !self.paused { Stance::Crouch } else { Stance::Stand }, dt);
        let speed = self.body.stance.speed(input.sprint);
        // Hugg og blokk bremser, unnamanøveren kaster deg til sida.
        let horizontal = wish.normalize_or_zero() * speed * self.combat.move_factor() + self.combat.dodge_velocity();
        let jumped = self.body.vertical(dt, input.jump && !self.paused);
        let desired = (horizontal + Vec3::Y * self.body.vel_y) * dt;
        let (half, radius) = self.body.capsule();
        let (moved, grounded) = physics.move_capsule(self.body.pos, half, radius, desired, dt);
        let moved = self.wade(moved, dt, events);
        let steps = self.body.moved(moved, grounded, jumped, !self.paused, input.sprint);
        // Skrittene etter underlaget, med rustningen som rasler i takt; hopp
        // og landing (hardere jo lenger fallet).
        let feet = self.body.feet();
        if steps.jumped {
            events.push(sfx("jump"));
        }
        if let Some(hard) = steps.landed {
            events.push(Event::Sound(Sound { name: "land".into(), at: None, gain: 0.5 + hard * 0.7 }));
            events.push(Event::Sound(Sound { name: self.surface(feet).into(), at: None, gain: 0.9 }));
        }
        if let Some(gain) = steps.step {
            let surface = self.surface(feet);
            events.push(Event::Sound(Sound { name: surface.into(), at: None, gain }));
            // Søle i regnvær.
            let wet = self.sky.weather.wet;
            if wet > 0.6 && matches!(surface, "step_gravel" | "step_dirt") {
                events.push(Event::Sound(Sound { name: "step_water".into(), at: None, gain: gain * 0.5 * wet }));
            }
            if let Some(a) = self.hero.gear.armor {
                events.push(Event::Sound(Sound { name: a.sound.into(), at: None, gain }));
            }
        }
        // Under det nederste nivået i krypten er det ingenting.
        let abyss = self.crypt.levels.last().map(|l| l.min_y).unwrap_or(crypt::TOP) - 30.0;
        if self.body.pos.y < abyss {
            self.respawn();
        }
        if self.body.grounded && horizontal.length_squared() > 0.0 {
            self.body.bob += dt * speed / 4.2;
        }
        // Falt utfor kanten i krypten: opp igjen der du sist sto trygt, med vondt.
        if self.in_crypt() {
            let feet = self.body.feet();
            let depth = self.flood_depth(feet);
            if let Some(back) = self.run.rescue(&self.crypt, feet, self.body.grounded, depth) {
                self.teleport(back, self.yaw);
                if depth > crypt::WADE {
                    events.extend([Event::Fade, sfx("step_water"), Event::Toast("Du falt i det dype vannet, og dro deg opp igjen der det var tørt, våt og kald.".into())]);
                } else {
                    self.hero.hp -= 30.0;
                    events.extend([Event::Fade, sfx("land"), sfx("hurt"), Event::Toast("Du falt utfor kanten, og kom deg opp igjen med skrubbsår og en vond rygg.".into())]);
                }
            }
        }
        wish
    }

    /// En handling fra spilleren.
    fn act(&mut self, a: Action, wish: Vec3, in_crypt: bool, events: &mut Vec<Event>) {
        match a {
            Action::Attack if in_crypt && self.aim.is_none() => self.combat.press_attack(),
            Action::AttackRelease => self.combat.release_attack(),
            Action::Dodge => {
                if in_crypt && self.body.grounded && wish.length_squared() > 0.0 && !self.paused && self.combat.dodge(wish) {
                    events.push(sfx("dodge"));
                }
            }
            Action::Daylight => {
                let d = self.sky.turn();
                // Kirkeklokka ringer inn skumringen og morgenen.
                if !in_crypt {
                    events.push(self.bell());
                }
                events.push(Event::Daylight(d));
            }
            Action::Weather => {
                let w = self.sky.next_weather();
                events.push(Event::Weather(w.name().into()));
                events.push(Event::Toast(format!("Været: {}", w.name())));
            }
            Action::Use | Action::Attack => match self.aim {
                Some(i) => self.use_target(i, events),
                None if in_crypt => {
                    // F uten noe å bruke: et hugg.
                    self.combat.press_attack();
                    self.combat.release_attack();
                }
                None => {}
            },
            Action::Trade => {
                if let Some(Use::Talk(n)) = self.aim.map(|i| self.targets[i].target.what) {
                    events.extend(self.npc_panel(n));
                }
            }
            Action::Home => self.go_home(events),
        }
    }

    /// Innbyggerne: de går, snakker (boblene som står sendes hvert bilde,
    /// og én tom når de er borte), og siktemålet følger dem.
    fn update_people(&mut self, dt: f32, feet: Vec3, in_crypt: bool, physics: &mut dyn Physics, events: &mut Vec<Event>) {
        let speeches = self.npcs.update(dt, feet);
        self.npc_steps(feet, in_crypt, events);
        for t in &self.targets {
            if let Use::Talk(n) = t.target.what {
                if let Some(npc) = self.npcs.npcs.get(n) {
                    physics.set_pose(t.collider, Affine3A::from_translation(npc.pos + t.target.center));
                }
            }
        }
        if !speeches.is_empty() || self.bubbles_shown {
            self.bubbles_shown = !speeches.is_empty();
            let bubbles = speeches
                .iter()
                .filter_map(|s| self.project(s.pos).map(|(x, y)| Bubble { id: s.npc as u32, x, y, name: s.name.into(), text: s.text.clone() }))
                .collect();
            events.push(Event::Speech(bubbles));
        }
    }

    /// Klokkene til portalen og virvelen, inngangene til nivåene, og
    /// siktemålet og kolliderne i ringen (bare ute når portalen er åpen).
    fn update_portal(&mut self, dt: f32, in_crypt: bool, physics: &mut dyn Physics) {
        self.portal.tick(dt);
        if in_crypt && self.body.grounded && self.hero.hp > 0.0 {
            let feet = self.body.feet();
            if let Some(k) = self.crypt.level_at(feet) {
                if self.portal.entries.get(k).is_some_and(|e| e.is_none()) && self.crypt.walkable(feet) {
                    self.portal.enter(k, feet, self.yaw);
                }
            }
        }
        if let Some(show) = self.portal.shown_changed() {
            if let Some(t) = self.targets.iter().find(|t| t.target.what == Use::Portal) {
                physics.set_pose(t.collider, if show { t.target.pose * Affine3A::from_translation(t.target.center) } else { away() });
                for (id, (c, _)) in self.portal_solids.iter().zip(town::PORTAL_SOLIDS) {
                    physics.set_pose(*id, if show { t.target.pose * Affine3A::from_translation(c) } else { away() });
                }
            }
        }
    }

    /// Ringen (plattformens midte og retning), om byen har den.
    pub(super) fn portal_pose(&self) -> Option<Affine3A> {
        self.targets.iter().find(|t| t.target.what == Use::Portal).map(|t| t.target.pose)
    }

    pub(super) fn portal_center(&self) -> Option<Vec3> {
        self.portal_pose().map(Portal::center)
    }

    /// Kartet: sendt når det endrer seg, og hvor du står hvert bilde.
    fn update_map(&mut self, in_crypt: bool, events: &mut Vec<Event>) {
        let feet = self.body.feet();
        let level = if in_crypt { self.crypt.level_at(feet) } else { None };
        let u = self.map.update(&self.crypt, &self.mech, &self.run, level, feet, self.yaw);
        if let Some(m) = u.map {
            events.push(Event::Map(m.map(|k| MapView { level: k.level, name: k.name, rooms: k.rooms, marks: k.marks.into_iter().map(|(kind, x, z)| MapMark { kind: kind.into(), x, z }).collect() })));
        }
        if let Some((x, z, yaw)) = u.at {
            events.push(Event::Where { x, z, yaw });
        }
    }

    /// Det som skjer av seg selv nede (`Run::tick`).
    fn update_crypt(&mut self, dt: f32, events: &mut Vec<Event>) {
        if !self.in_crypt() {
            return;
        }
        let feet = self.body.feet();
        let Some(k) = self.crypt.level_at(feet) else { return };
        if k != self.mobs.level {
            self.enter_level(k);
        }
        // Bossarenaen: porten slår igjen bak deg mens bossen lever.
        let inside = self.crypt.room_at(feet).filter(|(l, _)| *l == k).map(|(_, r)| r);
        let boss_alive = self.mobs.mobs.iter().any(|m| m.boss && m.alive() && m.level == k);
        for n in self.mech.arena_gates(&mut self.crypt, k, inside, boss_alive) {
            let at = self.crypt.gates[n].pos + Vec3::Y * 1.5;
            events.push(self.sound_at("gate", at, 1.4));
            events.push(Event::Toast("Steinporten slår igjen bak deg!".into()));
            self.map.dirty = true;
        }
        for h in self.run.tick(dt, &self.crypt, &self.mech, k, feet) {
            match h {
                Happening::Down(n) => {
                    self.go_to_level(n);
                    let l = &self.crypt.levels[n];
                    events.extend([Event::Fade, sfx("crypt_door"), Event::Toast(format!("Nivå {}: {}", l.n, l.name))]);
                }
                Happening::DownBlocked => {
                    events.extend([sfx("locked"), Event::Toast("Trappa ned er rast sammen. Dypere har ingen kommet ennå.".into())]);
                }
                Happening::Trapdoor { switch, at } => {
                    self.open_switch(switch);
                    events.push(self.sound_at("crack", at, 1.2));
                    events.push(self.sound_at("door_close", at, 1.0));
                }
                Happening::Arrows { plate, from, first } => {
                    events.push(self.sound_at("click", plate, 1.0));
                    events.push(self.sound_at("swing", from, 1.2));
                    if first {
                        events.push(Event::Toast("Klikk! Pilene smeller i gulvet foran deg. Se opp for trykkplatene.".into()));
                    }
                }
                Happening::Hurt(hurt) => {
                    let (taken, _) = self.combat.incoming(feet - Vec3::Y, hurt, self.eye(), self.forward());
                    self.hero.hp -= taken;
                    events.push(sfx("hurt"));
                    if self.hero.hp <= 0.0 {
                        self.die(events);
                        return;
                    }
                }
                Happening::Bag(bag) => {
                    let n: u32 = bag.values().sum();
                    for (c, v) in bag {
                        self.hero.carry(&c, v);
                    }
                    events.extend([sfx("bag"), Event::Toast(format!("Du fant igjen det du mistet: {n} komponenter."))]);
                }
                Happening::Hint(at) => events.push(self.sound_at("rats", at, 0.8)),
            }
        }
    }

    /// Mekanismene går videre, og kolliderne følger: snarveidørene dreies,
    /// de falske veggene synker, lemmene faller, gitrene heises. Når
    /// bøtta er oppe, drikker du.
    fn update_mechanisms(&mut self, physics: &mut dyn Physics, dt: f32, events: &mut Vec<Event>) {
        let (mut moved, drank) = self.mech.tick(dt);
        moved.extend(self.mech.tick_lifts(&self.crypt, dt));
        let target = |what: Use| self.targets.iter().find(|t| t.target.what == what);
        for m in moved {
            match m {
                Moved::Lift(n) => {
                    let (Some(pose), Some(id)) = (self.mech.lift_collider(&self.crypt, n), self.lift_solids.get(n)) else { continue };
                    physics.set_pose(*id, pose);
                    let deck = self.mech.lift_pose(&self.crypt, n).unwrap_or(pose);
                    if let Some(t) = target(Use::Lift(n)) {
                        physics.set_pose(t.collider, deck * Affine3A::from_translation(t.target.center));
                    }
                    // Står du på dekket, blir du med det (kapselen følger ikke en kollider som flyttes).
                    let l = &self.crypt.lifts[n];
                    let feet = self.body.feet();
                    let local = deck.inverse().transform_point3(feet);
                    let dy = self.mech.lift_dy.get(n).copied().unwrap_or(0.0);
                    if local.x.abs() < l.size.x * 0.5 + 0.3 && local.z.abs() < l.size.y * 0.5 + 0.3 && local.y > -0.6 + dy.min(0.0) && local.y < 0.8 {
                        let deck_y = deck.translation.y;
                        self.body.pos.y += deck_y + 0.02 - feet.y;
                        self.body.vel_y = 0.0;
                        self.body.grounded = true;
                    }
                }
                Moved::Shortcut(n) => {
                    if let (Some(pose), Some(t)) = (self.mech.shortcut_collider(&self.crypt, n), target(Use::Shortcut(n))) {
                        physics.set_pose(t.collider, pose);
                    }
                }
                Moved::Secret(n) => {
                    if let Some(t) = target(Use::Secret(n)) {
                        let sink = self.mech.secret_sink(&self.crypt, n);
                        physics.set_pose(t.collider, t.target.pose * Affine3A::from_translation(t.target.center - Vec3::Y * sink));
                    }
                }
                Moved::Trapdoor(n) => {
                    if let Some(Some(id)) = self.trap_lids.get(n) {
                        physics.set_pose(*id, away());
                    }
                }
                Moved::Bridge(n) => {
                    if let (Some(pose), Some(id)) = (self.mech.bridge_collider(&self.crypt, n), self.bridge_solids.get(n)) {
                        physics.set_pose(*id, pose);
                    }
                }
                Moved::Gate(n) => {
                    if let Some(t) = target(Use::Gate(n)) {
                        physics.set_pose(t.collider, self.mech.gate_lifted(n, t.target.pose) * Affine3A::from_translation(t.target.center));
                    }
                }
            }
        }
        for _ in 0..drank {
            self.hero.heal(25.0);
            self.combat.stamina = self.combat.max_stamina;
            events.push(Event::Toast("Du heiser opp kaldt vann og drikker. Litt liv tilbake, og pusten er full.".into()));
        }
    }
}
