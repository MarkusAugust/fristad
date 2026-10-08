//! Økta: spilleren i verden. Setter sammen reglene fra kjernen (kamp,
//! fiender, smiing, innbyggere, dyr, vær) med fysikken gjennom porten
//! `Physics`, og svarer med hendelser (`ports::driving::Event`).

use crate::domain::combat::{Combat, CombatEvent};
use crate::domain::craft::{self, Gear};
use crate::domain::herd::Herd;
use crate::domain::mobs::{Kind as MobKind, Mobs};
use crate::domain::npcs::Npcs;
use crate::domain::scene::{Light, Weather};
use crate::domain::tokens::Tokens;
use crate::domain::weather::{Daylight, WeatherKind};
use crate::domain::world::dungeon::{self, Dungeon};
use crate::domain::world::town;
use crate::domain::world::{Target, Use, World};
use crate::ports::driving::{Action, Bubble, Event, Input, InventoryView, Mark, Prompt, RecipeView, SaveData, WareView};
use crate::ports::{ColliderId, Physics};
use glam::{Affine3A, Mat4, Vec3};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Radianer per piksel musa flytter seg.
const SENSITIVITY: f32 = 0.0035;
pub(super) const EYE: f32 = 0.8;
const RADIUS: f32 = 0.35;
const HALF: f32 = 0.55;

/// Stående, på huk eller krypende: kapselen (halv høyde, radius), øyet over
/// kapselens sentrum, og farten.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Stance {
    #[default]
    Stand,
    Crouch,
    Crawl,
}

impl Stance {
    fn capsule(self) -> (f32, f32) {
        match self {
            Stance::Stand => (HALF, RADIUS),
            Stance::Crouch => (0.15, RADIUS),
            Stance::Crawl => (0.0, 0.3),
        }
    }
    fn eye(self) -> f32 {
        match self {
            Stance::Stand => EYE,
            Stance::Crouch => 0.5,
            Stance::Crawl => 0.15,
        }
    }
    fn speed(self, sprint: bool) -> f32 {
        match self {
            Stance::Stand => {
                if sprint {
                    7.5
                } else {
                    4.2
                }
            }
            Stance::Crouch => 2.0,
            Stance::Crawl => 1.0,
        }
    }
}

/// Noe du kan bruke, med kollideren strålen treffer.
struct TargetState {
    target: Target,
    collider: ColliderId,
}

pub struct Session {
    pub(super) tokens: Tokens,
    pub(super) lights: Vec<Light>,
    /// Pipene som ryker (toppen, tykkelsen).
    pub(super) smoke: Vec<(Vec3, f32)>,
    targets: Vec<TargetState>,
    by_collider: HashMap<ColliderId, usize>,
    pub(super) pos: Vec3,
    vel_y: f32,
    grounded: bool,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) time: f32,
    pub(super) daylight: Daylight,
    /// 0 er dag, 1 er natt, glir mellom.
    pub(super) night: f32,
    pub(super) weather_kind: WeatherKind,
    pub(super) weather: Weather,
    pub(super) fog_mul: f32,
    lightning_timer: f32,
    /// Det du sikter på (indeks i `targets`), og siktet slik det ble sendt sist.
    aim: Option<usize>,
    last_prompt: Option<Prompt>,
    pub paused: bool,
    bob: f32,
    pub(super) jumped: bool,
    stance: Stance,
    crawl: bool,
    eye_h: f32,
    spawn: Vec3,
    spawn_yaw: f32,
    crypt_exit: (Vec3, f32),
    pub(super) herd: Herd,
    pub(super) npcs: Npcs,
    bubbles_shown: bool,
    pub(super) dungeon: Dungeon,
    pub(super) mobs: Mobs,
    hp: f32,
    max_hp: f32,
    coins: u32,
    keys: u32,
    /// Portene i kjelleren: 0 lukket, 1 oppe, og om de er låst opp.
    pub(super) gates: Vec<(f32, bool)>,
    /// Hvor langt hver brønn er kommet i å heise opp vann (sekunder), eller
    /// negativ når ingen bruker den.
    pub(super) wells: Vec<f32>,
    chests: Vec<bool>,
    /// Hvor langt lokket på hver kiste er åpnet (0–1).
    chest_open: Vec<f32>,
    /// Komponentene: trygt lagret i byen, og det du bærer på turen.
    inventory: BTreeMap<String, u32>,
    carried: BTreeMap<String, u32>,
    gear: Gear,
    /// Komponentene du har lest om i boka denne turen (liv tilbake én gang per).
    read_docs: HashSet<String>,
    inv_dirty: bool,
    loot_seed: u64,
    /// Hendelser fra kommandoer (kjøp, smiing, lesing), til neste `update`.
    pending: Vec<Event>,
    pub(super) combat: Combat,
    /// Skadetall som stiger: (sted, skade, alder, løpenummer).
    popups: Vec<(Vec3, f32, f32, u32)>,
    popup_seq: u32,
    marks_shown: bool,
    /// Bildeforholdet fra sist `frame`, til å regne skjermposisjoner.
    pub(super) aspect: std::cell::Cell<f32>,
    aimed_mob: Option<usize>,
    /// Hvor mye du er i kjelleren (0 oppe, 1 nede), glattet, til lyset.
    pub(super) cellar_t: f32,
    last_hud: Option<(u32, u32, u32, bool, u32, Option<(&'static str, u32)>)>,
    /// Rammeverket er beseiret (denne turen).
    pub won: bool,
    pub debug: u32,
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
            let collider = physics.add_movable(t.half, t.pose * Affine3A::from_translation(t.center));
            by_collider.insert(collider, i);
            targets.push(TargetState { target: *t, collider });
        }
        let rig_of = |n: &str| rigs.iter().find(|(m, _)| m == n).map(|(_, r)| r.clone());
        let feet = crate::domain::world::terrain::at(world.spawn);
        let (half, radius) = Stance::Stand.capsule();
        Self {
            tokens: Tokens::load(),
            lights: world.lights.clone(),
            smoke: world.smoke.clone(),
            targets,
            by_collider,
            pos: feet + Vec3::Y * (half + radius + 0.05),
            vel_y: 0.0,
            grounded: false,
            yaw: world.spawn_yaw,
            pitch: 0.05,
            time: 0.0,
            daylight: Daylight::Day,
            night: 0.0,
            weather_kind: WeatherKind::Clear,
            weather: Weather { wind: 0.2, ..Default::default() },
            fog_mul: 1.0,
            lightning_timer: 0.0,
            aim: None,
            last_prompt: None,
            paused: false,
            bob: 0.0,
            jumped: false,
            stance: Stance::Stand,
            crawl: false,
            eye_h: EYE,
            spawn: world.spawn,
            spawn_yaw: world.spawn_yaw,
            crypt_exit: world.crypt,
            herd: Herd::new(),
            npcs: Npcs::new(rig_of, world.npcs.iter().map(|n| (n.name, n.model, n.pos, n.yaw, n.tint, n.lines, n.route.clone())).collect()),
            bubbles_shown: false,
            mobs: Mobs::new(&world.dungeon, rigs),
            dungeon: world.dungeon.clone(),
            hp: 100.0,
            max_hp: 100.0,
            coins: 0,
            keys: 0,
            gates: vec![(0.0, false); world.dungeon.gates.len()],
            wells: vec![-1.0; world.targets.iter().filter(|t| matches!(t.what, Use::Well(_))).count()],
            chests: vec![false; world.dungeon.chests.len()],
            chest_open: vec![0.0; world.dungeon.chests.len()],
            inventory: BTreeMap::new(),
            carried: BTreeMap::new(),
            gear: Gear { weapon: craft::RUSTY_SWORD, shield: craft::OLD_SHIELD, armor: None, lantern: 1.0 },
            read_docs: HashSet::new(),
            inv_dirty: true,
            loot_seed: 0x2545_f491_4f6c_dd1d,
            pending: vec![],
            combat: Combat::new(rig_of(crate::domain::combat::ARMS_MODEL)),
            popups: vec![],
            popup_seq: 0,
            marks_shown: false,
            aspect: std::cell::Cell::new(16.0 / 9.0),
            aimed_mob: None,
            cellar_t: 0.0,
            last_hud: None,
            won: false,
            debug: 0,
        }
    }

    pub fn fristil_version(&self) -> &str {
        &self.tokens.version
    }

    pub fn weather_kind(&self) -> WeatherKind {
        self.weather_kind
    }

    pub fn daylight(&self) -> Daylight {
        self.daylight
    }

    pub fn set_daylight(&mut self, d: Daylight, instant: bool) {
        self.daylight = d;
        if instant {
            self.night = if d == Daylight::Night { 1.0 } else { 0.0 };
        }
    }

    pub fn set_weather(&mut self, kind: WeatherKind, instant: bool) {
        self.weather_kind = kind;
        if instant {
            let (target, fog) = kind.target();
            self.weather = target;
            self.fog_mul = fog;
        }
    }

    /// Snu kameraet uten å flytte det.
    pub fn set_look(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
    }

    /// Plasser kameraet, til skjermbilder og tester. `eye` er øyet.
    pub fn place(&mut self, eye: Vec3, yaw: f32, pitch: f32) {
        self.pos = eye - Vec3::Y * EYE;
        self.yaw = yaw;
        self.pitch = pitch;
        self.jumped = true;
    }

    /// Tilbake til sørporten.
    pub fn respawn(&mut self) {
        let (half, radius) = self.stance.capsule();
        self.pos = crate::domain::world::terrain::at(self.spawn) + Vec3::Y * (half + radius + 0.1);
        self.yaw = self.spawn_yaw;
        self.vel_y = 0.0;
        self.jumped = true;
    }

    pub(super) fn forward(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), -self.yaw.cos() * self.pitch.cos())
    }

    /// Hvor innbyggerne står nå (til testene).
    pub fn npc_positions(&self) -> Vec<Vec3> {
        self.npcs.npcs.iter().map(|n| n.pos).collect()
    }

    /// Flytt spilleren (til testene).
    pub fn place_feet(&mut self, feet: Vec3) {
        self.teleport(feet, 0.0);
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::Y * (self.eye_h + (self.bob * 9.0).sin() * 0.025)
    }

    pub(super) fn in_cellar(&self) -> bool {
        self.pos.y < dungeon::DEPTH + 6.0
    }

    /// Siktet for et mål, slik det er nå.
    fn prompt_for(&self, i: usize) -> Prompt {
        let p = |label: &str, verb: &str, trade: &str, info: &str| Prompt { label: label.into(), verb: verb.into(), trade: trade.into(), info: info.into() };
        match self.targets[i].target.what {
            Use::Talk(n) => {
                let name = self.npcs.npcs.get(n).map(|c| c.name).unwrap_or("Innbygger");
                let (trade, info) = match name {
                    "Knappesmeden" => ("Handle", "Smeden selger våpen og skjold for tokens."),
                    "Apotekeren" => ("Handle", "Drikker som leger, og tokens for det du har funnet."),
                    "Gorvek" => ("Handle", "Lærer deg trekk på øvingsplassen."),
                    "Skriptoriet" => ("Smi og les", "Smi utstyr av komponentene fra krypten, og les om dem i bøkene."),
                    "Vertinnen" => ("", "Vertshuset. Her får du høre hva som går rykter om krypten."),
                    _ => ("", ""),
                };
                p(name, "Snakk", trade, info)
            }
            Use::Chest(n) => {
                if self.chests[n] {
                    p("Tom kiste", "", "", "Du har allerede tatt det som lå her.")
                } else {
                    p("Kiste", "Åpne", "", "Komponenter og tokens. Det du bærer mistes om du dør.")
                }
            }
            Use::Gate(n) => {
                if self.gates[n].1 {
                    p("Porten er oppe", "", "", "Trappa ned til neste etasje er bak porten.")
                } else if self.keys > 0 {
                    p("Rusten port", "Lås opp", "", "Du har en nøkkel. Trappa ned til neste etasje er bak porten.")
                } else {
                    p("Rusten port", "Lås opp", "", "Låst. En sterkere fiende i denne etasjen bærer nøkkelen.")
                }
            }
            Use::Descend => p("Krypten under borgen", "Gå ned", "", "Ned i kjelleren, der de korrupte komponentene holder til."),
            Use::Ascend => p("Døra opp til krypten", "Gå opp", "", "Opp til byen. Det du bærer legges i lageret."),
            Use::Well(n) => {
                if self.wells.get(n).is_some_and(|w| *w >= 0.0) {
                    p("Brønn", "", "", "Sveiven går. Bøtta er på vei opp.")
                } else {
                    p("Brønn", "Heis opp vann", "", "Kaldt vann fra dypet: litt liv tilbake og full utholdenhet.")
                }
            }
        }
    }

    fn mob_prompt(label: &str) -> Prompt {
        Prompt { label: label.into(), verb: "Hugg".into(), trade: String::new(), info: "Klikk for å hugge, høyreklikk for å blokke.".into() }
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

        // Gåing.
        let mut wish = Vec3::ZERO;
        if !self.paused {
            let fwd = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
            let right = Vec3::new(-fwd.z, 0.0, fwd.x);
            if input.forward {
                wish += fwd;
            }
            if input.back {
                wish -= fwd;
            }
            if input.right {
                wish += right;
            }
            if input.left {
                wish -= right;
            }
        }
        // Stillingen: X slår kryping av og på, C holdes for huk.
        let stance = if self.crawl {
            Stance::Crawl
        } else if input.crouch && !self.paused {
            Stance::Crouch
        } else {
            Stance::Stand
        };
        if stance != self.stance {
            // Føttene blir der de er når kapselen bytter størrelse.
            let (h0, r0) = self.stance.capsule();
            let (h1, r1) = stance.capsule();
            self.pos.y += (h1 + r1) - (h0 + r0);
            self.stance = stance;
        }
        let (half, radius) = self.stance.capsule();
        self.eye_h += (self.stance.eye() - self.eye_h) * (dt * 10.0).min(1.0);
        let speed = self.stance.speed(input.sprint);
        // Hugg og blokk bremser, unnamanøveren kaster deg til sida.
        let horizontal = wish.normalize_or_zero() * speed * self.combat.move_factor() + self.combat.dodge_velocity();
        if self.grounded {
            self.vel_y = -1.0;
            if input.jump && !self.paused && self.stance == Stance::Stand {
                self.vel_y = 5.4;
            }
        } else {
            self.vel_y -= 18.0 * dt;
        }
        let desired = (horizontal + Vec3::Y * self.vel_y) * dt;
        let (moved, grounded) = physics.move_capsule(self.pos, half, radius, desired, dt);
        self.pos += moved;
        self.grounded = grounded;
        // Under nederste etasje i kjelleren er det ingenting.
        let abyss = self.dungeon.floors.last().map(|f| f.y).unwrap_or(dungeon::DEPTH) - 12.0;
        if self.pos.y < abyss {
            self.respawn();
        }
        if self.grounded && horizontal.length_squared() > 0.0 {
            self.bob += dt * speed / 4.2;
        }

        // Hva sikter vi på? Fiendene i kjelleren går foran.
        let eye = self.eye();
        let dir = self.forward();
        let in_cellar = self.in_cellar();
        let hit = physics.cast_ray(eye, dir, 3.5);
        let aimed_mob = if in_cellar { self.mobs.aimed(eye, dir, 2.6) } else { None };
        self.aimed_mob = aimed_mob;
        self.aim = if aimed_mob.is_some() { None } else { hit.and_then(|h| self.by_collider.get(&h.collider).copied()) };
        self.combat.set_block(input.block && in_cellar && !self.paused);

        for a in actions {
            match a {
                Action::Attack if in_cellar && self.aim.is_none() => self.combat.press_attack(),
                Action::AttackRelease => self.combat.release_attack(),
                Action::Dodge => {
                    if in_cellar && self.grounded && wish.length_squared() > 0.0 && !self.paused && self.combat.dodge(wish) {
                        events.push(Event::Sound("dodge".into()));
                    }
                }
                Action::Daylight => {
                    self.daylight = if self.daylight == Daylight::Day { Daylight::Night } else { Daylight::Day };
                    events.push(Event::Daylight(self.daylight));
                }
                Action::Weather => {
                    let all = WeatherKind::ALL;
                    let i = all.iter().position(|w| *w == self.weather_kind).unwrap_or(0);
                    self.weather_kind = all[(i + 1) % all.len()];
                    events.push(Event::Weather(self.weather_kind.name().into()));
                    events.push(Event::Toast(format!("Været: {}", self.weather_kind.name())));
                }
                Action::Use | Action::Attack => match self.aim {
                    Some(i) => self.use_target(i, &mut events),
                    None if in_cellar => {
                        // F uten noe å bruke: et hugg.
                        self.combat.press_attack();
                        self.combat.release_attack();
                    }
                    None => {}
                },
                Action::Trade => {
                    if let Some(i) = self.aim {
                        if let Use::Talk(n) = self.targets[i].target.what {
                            if let Some(e) = self.npc_panel(n) {
                                events.push(e);
                            }
                        }
                    }
                }
                Action::Crawl => self.crawl = !self.crawl,
                Action::Home => {
                    self.respawn();
                    events.push(Event::Fade);
                    events.push(Event::Toast("Tilbake ved sørporten".into()));
                }
            }
        }

        // Siktet: sendes når det endrer seg.
        let prompt = match (self.aimed_mob, self.aim) {
            (Some(m), _) => Some(Self::mob_prompt(self.mobs.mobs[m].kind.label())),
            (None, Some(i)) => Some(self.prompt_for(i)),
            _ => None,
        };
        if prompt != self.last_prompt {
            self.last_prompt = prompt.clone();
            events.push(Event::Prompt(prompt));
        }

        // Dag og natt glir over i hverandre på et par sekunder.
        let goal = if self.daylight == Daylight::Night { 1.0 } else { 0.0 };
        let step = dt / 2.8;
        self.night = if self.night < goal { (self.night + step).min(goal) } else { (self.night - step).max(goal) };

        self.update_weather(dt);
        self.update_gates(physics, dt);
        self.update_wells(dt, &mut events);
        let (half, radius) = self.stance.capsule();
        let feet = self.pos - Vec3::Y * (half + radius);
        self.herd.update(dt, feet);
        // Innbyggerne: boblene som står sendes hvert bilde (og én tom når de er borte).
        let speeches = self.npcs.update(dt, feet);
        // Siktemålet følger innbyggeren.
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
        self.cellar_t += ((if in_cellar { 1.0 } else { 0.0 }) - self.cellar_t) * (dt * 4.0).min(1.0);
        self.fight(dt, feet, in_cellar, &mut events);
        // Merkene: helsestolper over fiender som er truffet, og skadetall.
        let marks = self.marks(dt);
        if !marks.is_empty() || self.marks_shown {
            self.marks_shown = !marks.is_empty();
            events.push(Event::Marks(marks));
        }
        let boss = if in_cellar { self.mobs.boss().map(|(n, f)| (n, (f * 100.0).ceil() as u32)) } else { None };
        let hud = (self.hp.round().max(0.0) as u32, self.combat.stamina.round().max(0.0) as u32, self.coins, in_cellar, self.keys, boss);
        if self.last_hud != Some(hud) {
            if self.last_hud.as_ref().map(|h| h.2) != Some(self.coins) {
                self.inv_dirty = true;
            }
            self.last_hud = Some(hud);
            events.push(Event::Hud { hp: hud.0, stamina: hud.1, tokens: hud.2, cellar: hud.3, keys: hud.4, boss: hud.5.map(|(n, f)| (n.to_string(), f)) });
        }
        if self.inv_dirty {
            self.inv_dirty = false;
            events.push(Event::Inventory(self.inventory_view()));
        }
        events
    }

    /// F på mål nummer `i`.
    fn use_target(&mut self, i: usize, events: &mut Vec<Event>) {
        match self.targets[i].target.what {
            Use::Talk(n) => {
                if self.npcs.talk(n).is_some() {
                    events.push(Event::Sound("talk".into()));
                }
            }
            Use::Chest(n) => {
                if self.chests[n] {
                    return;
                }
                self.chests[n] = true;
                let floor = self.dungeon.chests.get(n).map(|c| c.2).unwrap_or(0);
                let list = craft::LOOT[floor.min(craft::LOOT.len() - 1)];
                self.loot_seed ^= self.loot_seed << 13;
                self.loot_seed ^= self.loot_seed >> 7;
                self.loot_seed ^= self.loot_seed << 17;
                let pick = list[(self.loot_seed >> 33) as usize % list.len()];
                *self.carried.entry(pick.to_string()).or_insert(0) += 1;
                self.inv_dirty = true;
                events.push(Event::Sound("chest".into()));
                events.push(Event::Loot(Some(pick.to_string())));
            }
            Use::Gate(n) => {
                if self.gates[n].1 {
                    return;
                }
                if self.keys > 0 {
                    self.keys -= 1;
                    self.gates[n].1 = true;
                    events.push(Event::Sound("door".into()));
                    events.push(Event::Toast("Nøkkelen passer. Porten sveives opp; trappa ned ligger bak.".into()));
                } else {
                    events.push(Event::Toast("Porten er låst. Elitefienden i rommet bærer nøkkelen.".into()));
                }
            }
            Use::Well(n) => {
                if let Some(w) = self.wells.get_mut(n) {
                    if *w < 0.0 {
                        *w = 0.0;
                        events.push(Event::Sound("well".into()));
                    }
                }
            }
            Use::Descend => {
                let to = self.dungeon.start;
                self.teleport(to, 0.0);
                events.push(Event::Fade);
                events.push(Event::Sound("door".into()));
                events.push(Event::Toast("Ned i kjelleren under borgen. Venstre mus hugger (hold for tungt), høyre mus blokker, Shift og retning er unnamanøver.".into()));
            }
            Use::Ascend => {
                let (to, yaw) = self.crypt_exit;
                self.teleport(to, yaw);
                events.push(Event::Fade);
                events.push(Event::Sound("door".into()));
                self.bank(events);
            }
        }
    }

    /// Rett dit, uten fysikk på veien: føttene og retningen.
    fn teleport(&mut self, feet: Vec3, yaw: f32) {
        let (half, radius) = self.stance.capsule();
        self.pos = feet + Vec3::Y * (half + radius + 0.05);
        self.yaw = yaw;
        self.pitch = 0.0;
        self.vel_y = 0.0;
        self.jumped = true;
    }

    /// Portene sveives opp, og kolliderne følger. Lokkene på kistene som er
    /// åpnet, går opp.
    fn update_gates(&mut self, physics: &mut dyn Physics, dt: f32) {
        for (open, opened) in self.chest_open.iter_mut().zip(&self.chests) {
            if *opened && *open < 1.0 {
                *open = (*open + dt / 0.9).min(1.0);
            }
        }
        for t in &self.targets {
            if let Use::Gate(n) = t.target.what {
                let (open, unlocked) = &mut self.gates[n];
                if *unlocked && *open < 1.0 {
                    *open = (*open + dt / 1.6).min(1.0);
                    let lift = Affine3A::from_translation(Vec3::Y * gate_lift(*open));
                    physics.set_pose(t.collider, t.target.pose * lift * Affine3A::from_translation(t.target.center));
                }
            }
        }
    }

    /// Brønnene: bøtta slippes ned, sveives opp, og du drikker.
    fn update_wells(&mut self, dt: f32, events: &mut Vec<Event>) {
        for w in &mut self.wells {
            if *w < 0.0 {
                continue;
            }
            let before = *w;
            *w += dt;
            if before < WELL_UP && *w >= WELL_UP {
                self.hp = (self.hp + 25.0).min(self.max_hp);
                self.combat.stamina = self.combat.max_stamina;
                events.push(Event::Toast("Du heiser opp kaldt vann og drikker. Litt liv tilbake, og pusten er full.".into()));
            }
            if *w >= WELL_DONE {
                *w = -1.0;
            }
        }
    }

    /// Fiendene: de biter, og du plukker tokens. Under hitstop står alt
    /// stille et øyeblikk.
    fn fight(&mut self, dt: f32, feet: Vec3, in_cellar: bool, events: &mut Vec<Event>) {
        let sim_dt = if self.combat.hitstop > 0.0 { 0.0 } else { dt };
        let eye = self.eye();
        let fwd = self.forward();
        let outcome = self.mobs.update(sim_dt, feet, in_cellar, &self.dungeon);
        self.coins += outcome.tokens;
        if outcome.tokens > 0 {
            events.push(Event::Sound("pickup".into()));
        }
        if outcome.keys > 0 {
            self.keys += outcome.keys;
            events.push(Event::Sound("pickup".into()));
            events.push(Event::Toast("En rusten nøkkel. Porten i utgangsrommet kan låses opp.".into()));
        }
        for _ in 0..outcome.windups {
            events.push(Event::Sound("growl".into()));
        }
        for _ in 0..outcome.spits {
            events.push(Event::Sound("spit".into()));
        }
        for _ in 0..outcome.roars {
            events.push(Event::Sound("roar".into()));
            events.push(Event::Toast("Rammeverket brøler, og tooltips kryper ut av veggene.".into()));
        }
        if outcome.quakes > 0 {
            events.push(Event::Sound("slam".into()));
            self.combat.quake();
        }
        // Bittene: blokk tar det meste, en parade alt (og lamslår), en
        // unnamanøver går klar.
        let mut taken_total = 0.0;
        for (mob, damage, from) in &outcome.bites {
            let (taken, parried) = self.combat.incoming(*from, *damage, eye, fwd);
            if parried {
                self.mobs.stagger(*mob, 1.3);
                events.push(Event::Sound("parry".into()));
            } else if taken < *damage * 0.99 && taken > 0.0 {
                events.push(Event::Sound("block".into()));
            } else if taken > 0.0 {
                events.push(Event::Sound("hurt".into()));
            }
            taken_total += taken;
        }
        if taken_total > 0.0 {
            self.hp -= taken_total;
            if self.hp <= 0.0 {
                self.die(events);
            }
        }
        // Våre hugg.
        for e in self.combat.update(dt, sim_dt, eye, fwd, feet, &mut self.mobs, &self.dungeon) {
            match e {
                CombatEvent::Hit { mob, damage, killed, pos } => {
                    self.popup_seq = self.popup_seq.wrapping_add(1);
                    self.popups.push((pos, damage, 0.0, self.popup_seq));
                    if killed {
                        self.killed(mob, pos, events);
                    }
                }
                CombatEvent::Miss => {}
                CombatEvent::Sound(s) => events.push(Event::Sound(s.into())),
            }
        }
    }

    /// Død: opp til krypten, det du bar er borte, en firedel av tokenene
    /// blir igjen nede.
    fn die(&mut self, events: &mut Vec<Event>) {
        let lost = self.coins / 4;
        self.coins -= lost;
        self.mobs.reset(&self.dungeon);
        self.combat.reset();
        self.popups.clear();
        self.keys = 0;
        self.hp = self.max_hp;
        self.gates.iter_mut().for_each(|g| *g = (0.0, false));
        if !self.carried.is_empty() {
            let n: u32 = self.carried.values().sum();
            self.carried.clear();
            self.inv_dirty = true;
            events.push(Event::Toast(format!("{n} komponenter ble igjen i kjelleren.")));
        }
        self.read_docs.clear();
        let (to, yaw) = self.crypt_exit;
        self.teleport(to, yaw);
        events.push(Event::Fade);
        events.push(Event::Sound("death".into()));
        events.push(Event::Toast(format!("Du ble overkjørt av forretningslogikk. {lost} tokens ble igjen nede.")));
    }

    fn killed(&mut self, mob: usize, pos: Vec3, events: &mut Vec<Event>) {
        let (kind, elite) = self.mobs.mobs.get(mob).map(|m| (m.kind, m.elite)).unwrap_or((MobKind::Button, false));
        if kind.boss() {
            for c in ["dialog", "tabs"] {
                *self.carried.entry(c.to_string()).or_insert(0) += 1;
            }
            self.inv_dirty = true;
            self.won = true;
            events.push(Event::Toast("Rammeverket er beseiret! Skatten i alkoven er din, og en dialog og en tabs ligger igjen i buret.".into()));
        } else if elite {
            let floor = self.dungeon.floor_of(pos).unwrap_or(0).min(craft::ELITE_LOOT.len() - 1);
            let c = craft::ELITE_LOOT[floor];
            *self.carried.entry(c.to_string()).or_insert(0) += 1;
            self.inv_dirty = true;
            events.push(Event::Toast(format!("Elitefienden falt fra hverandre: nøkkelen og en {c} ligger igjen.")));
        } else if kind != MobKind::Tooltip {
            events.push(Event::Toast(format!("{} falt fra hverandre. Plukk opp tokenene.", kind.label())));
        }
        if self.aimed_mob == Some(mob) {
            self.aimed_mob = None;
        }
    }

    /// Det du bar opp fra kjelleren legges i lageret.
    fn bank(&mut self, events: &mut Vec<Event>) {
        if self.carried.is_empty() {
            return;
        }
        let n: u32 = self.carried.values().sum();
        for (k, v) in std::mem::take(&mut self.carried) {
            *self.inventory.entry(k).or_insert(0) += v;
        }
        self.inv_dirty = true;
        events.push(Event::Toast(format!("{n} komponenter er trygt i lageret.")));
    }

    pub fn inventory_view(&self) -> InventoryView {
        InventoryView {
            stash: self.inventory.iter().filter(|(_, n)| **n > 0).map(|(k, v)| (k.clone(), *v)).collect(),
            carried: self.carried.iter().filter(|(_, n)| **n > 0).map(|(k, v)| (k.clone(), *v)).collect(),
            weapon: format!("{} ({})", self.gear.weapon.name, self.gear.weapon.rarity.name()),
            weapon_special: self.gear.weapon.special.into(),
            shield: format!("{} ({})", self.gear.shield.name, self.gear.shield.rarity.name()),
            armor: self.gear.armor.map(|a| format!("{} ({})", a.name, a.rarity.name())).unwrap_or_else(|| "ingen".into()),
            lantern: self.gear.lantern > 1.0,
            max_hp: self.max_hp.round() as u32,
            max_stamina: self.combat.max_stamina.round() as u32,
            tokens: self.coins,
            read: self.read_docs.iter().cloned().collect(),
        }
    }

    /// Butikken eller verkstedet til innbygger `i`.
    fn npc_panel(&self, i: usize) -> Option<Event> {
        let name = self.npcs.npcs.get(i).map(|n| n.name).unwrap_or("");
        let wares = |w: &[craft::Ware]| w.iter().map(|w| WareView { id: w.id.into(), name: w.name.into(), price: w.price }).collect();
        match name {
            "Knappesmeden" => Some(Event::Shop { npc: name.into(), wares: wares(craft::SMITH_WARES) }),
            "Apotekeren" => Some(Event::Shop { npc: name.into(), wares: wares(craft::APOTHECARY_WARES) }),
            "Gorvek" => Some(Event::Shop { npc: name.into(), wares: wares(craft::GORVEK_WARES) }),
            "Skriptoriet" => Some(Event::Workshop { recipes: self.recipe_views() }),
            _ => None,
        }
    }

    fn recipe_views(&self) -> Vec<RecipeView> {
        craft::RECIPES
            .iter()
            .map(|r| {
                let have = |c: &str| self.inventory.get(c).copied().unwrap_or(0);
                let result = match r.result {
                    craft::Result::Weapon(id) => craft::weapon(id).map(|w| format!("{} (skade ×{:.2}, fart ×{:.2}, rekkevidde {:.1} m, {})", w.name, w.damage, w.speed, w.reach, w.special)).unwrap_or_default(),
                    craft::Result::Shield(id) => craft::shield(id).map(|s| format!("{} (slipper {:.0} % gjennom, paradevindu {:.2} s)", s.name, s.block * 100.0, s.parry_window)).unwrap_or_default(),
                    craft::Result::Armor(id) => craft::armor(id).map(|a| format!("{} (tar {:.0} % skade)", a.name, a.taken * 100.0)).unwrap_or_default(),
                    craft::Result::Lantern(k) => format!("Søkelykt (bærelyset ×{k:.1})"),
                };
                RecipeView {
                    id: r.id.into(),
                    name: r.name.into(),
                    rarity: r.rarity.name().into(),
                    result,
                    needs: r.needs.iter().map(|(c, n)| (c.to_string(), *n, have(c))).collect(),
                    requires: r.requires.iter().map(|c| (c.to_string(), have(c) > 0)).collect(),
                    ok: craft::missing(r, &self.inventory).is_empty(),
                }
            })
            .collect()
    }

    /// Smi etter en oppskrift (fra skjemaet i skriptoriet). Gir om det gikk.
    pub fn craft(&mut self, id: &str) -> bool {
        let Some(r) = craft::recipe(id) else { return false };
        let missing = craft::missing(&r, &self.inventory);
        if !missing.is_empty() {
            let list: Vec<String> = missing.iter().map(|(c, n)| format!("{c} ×{n}")).collect();
            self.pending.push(Event::Toast(format!("Skjemaet avvist: mangler {}.", list.join(", "))));
            return false;
        }
        for (c, n) in r.needs {
            if let Some(v) = self.inventory.get_mut(*c) {
                *v = v.saturating_sub(*n);
            }
        }
        let what = match r.result {
            craft::Result::Weapon(id) => {
                if let Some(w) = craft::weapon(id) {
                    self.gear.weapon = w;
                }
                "våpenet"
            }
            craft::Result::Shield(id) => {
                if let Some(s) = craft::shield(id) {
                    self.gear.shield = s;
                }
                "skjoldet"
            }
            craft::Result::Armor(id) => {
                self.gear.armor = craft::armor(id);
                "rustningen"
            }
            craft::Result::Lantern(k) => {
                self.gear.lantern = k;
                "lykta"
            }
        };
        self.combat.set_gear(self.gear);
        self.inv_dirty = true;
        self.pending.push(Event::Sound("parry".into()));
        self.pending.push(Event::Toast(format!("Skjemaet er godkjent. {} er smidd: {what} er byttet.", r.name)));
        true
    }

    /// Kjøp i en butikk, for tokens. Gir om det gikk.
    pub fn buy(&mut self, id: &str) -> bool {
        let Some(w) = craft::ware(id) else { return false };
        if self.coins < w.price {
            self.pending.push(Event::Toast(format!("Du mangler {} tokens.", w.price - self.coins)));
            return false;
        }
        self.coins -= w.price;
        match w.what {
            craft::WareKind::Component(c) => {
                *self.inventory.entry(c.to_string()).or_insert(0) += 1;
            }
            craft::WareKind::Heal(h) => self.hp = (self.hp + h).min(self.max_hp),
            craft::WareKind::MaxHp(h) => {
                self.max_hp += h;
                self.hp = (self.hp + h).min(self.max_hp);
            }
            craft::WareKind::MaxStamina(st) => {
                self.combat.max_stamina += st;
                self.combat.stamina = self.combat.max_stamina;
            }
        }
        self.inv_dirty = true;
        self.pending.push(Event::Sound("pickup".into()));
        self.pending.push(Event::Toast(format!("Kjøpt: {}.", w.name)));
        true
    }

    /// Boka: å lese dokumentasjonen for en komponent gir liv tilbake, én
    /// gang per komponent per tur.
    pub fn read_doc(&mut self, component: &str) -> bool {
        if self.read_docs.contains(component) {
            return false;
        }
        self.read_docs.insert(component.to_string());
        let before = self.hp;
        self.hp = (self.hp + 20.0).min(self.max_hp);
        self.inv_dirty = true;
        if self.hp > before {
            self.pending.push(Event::Toast(format!("Du leste om {component}: +{:.0} liv.", self.hp - before)));
        }
        true
    }

    pub fn save(&self) -> SaveData {
        SaveData {
            stash: self.inventory.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            weapon: self.gear.weapon.id.into(),
            shield: self.gear.shield.id.into(),
            armor: self.gear.armor.map(|a| a.id.to_string()),
            lantern: self.gear.lantern,
            max_hp: self.max_hp,
            max_stamina: self.combat.max_stamina,
            coins: self.coins,
        }
    }

    pub fn load(&mut self, d: SaveData) {
        self.inventory = d.stash.into_iter().collect();
        if let Some(w) = craft::weapon(&d.weapon) {
            self.gear.weapon = w;
        }
        if let Some(s) = craft::shield(&d.shield) {
            self.gear.shield = s;
        }
        self.gear.armor = d.armor.as_deref().and_then(craft::armor);
        self.gear.lantern = if d.lantern > 0.0 { d.lantern } else { 1.0 };
        if d.max_hp >= 100.0 {
            self.max_hp = d.max_hp;
            self.hp = self.max_hp;
        }
        if d.max_stamina >= 100.0 {
            self.combat.max_stamina = d.max_stamina;
            self.combat.stamina = d.max_stamina;
        }
        self.coins = d.coins;
        self.combat.set_gear(self.gear);
        self.inv_dirty = true;
    }

    /// Til testene: legg komponenter i lageret.
    pub fn give(&mut self, component: &str, n: u32) {
        *self.inventory.entry(component.to_string()).or_insert(0) += n;
        self.inv_dirty = true;
    }

    pub fn gear(&self) -> Gear {
        self.gear
    }

    /// Til testene: bruk det første målet av et slag, som om du siktet på det.
    pub fn use_first(&mut self, what: fn(&Use) -> bool) -> Vec<Event> {
        let mut events = vec![];
        if let Some(i) = self.targets.iter().position(|t| what(&t.target.what)) {
            self.use_target(i, &mut events);
        }
        events
    }

    /// Et punkt i verden til skjermen (0..1), om det er foran kameraet.
    pub(super) fn project(&self, p: Vec3) -> Option<(f32, f32)> {
        let view = Mat4::look_to_rh(self.eye(), self.forward(), Vec3::Y);
        let proj = Mat4::perspective_rh(72f32.to_radians(), self.aspect.get(), 0.1, 2500.0);
        let c = proj * view * p.extend(1.0);
        if c.w < 0.1 {
            return None;
        }
        let x = (c.x / c.w + 1.0) * 0.5;
        let y = (1.0 - c.y / c.w) * 0.5;
        ((-0.1..=1.1).contains(&x) && (-0.1..=1.1).contains(&y)).then_some((x, y))
    }

    fn marks(&mut self, dt: f32) -> Vec<Mark> {
        let mut out = vec![];
        for (i, m) in self.mobs.mobs.iter().enumerate() {
            if !m.alive() || m.hit > 3.0 || m.kind.boss() {
                continue;
            }
            if let Some((x, y)) = self.project(m.pos + Vec3::Y * 1.65 * m.scale()) {
                out.push(Mark { id: i as u32, x, y, kind: "bar", value: (m.hp / m.max_hp).clamp(0.0, 1.0), text: String::new() });
            }
        }
        for p in &mut self.popups {
            p.2 += dt;
        }
        self.popups.retain(|p| p.2 < 0.9);
        for (pos, dmg, age, id) in &self.popups {
            if let Some((x, y)) = self.project(*pos + Vec3::Y * (0.3 + age * 0.6)) {
                out.push(Mark { id: 100_000 + id, x, y, kind: "text", value: 1.0 - age / 0.9, text: format!("{}", dmg.round() as i32) });
            }
        }
        out
    }

    /// Besetningen: slag og om den er elite, til testene.
    pub fn roster(&self) -> Vec<(MobKind, bool)> {
        self.mobs.mobs.iter().map(|m| (m.kind, m.elite)).collect()
    }

    /// De levende fiendene (føttene, slaget, om den er elite), til
    /// gjennomspillingen i testene.
    #[cfg(test)]
    pub fn living(&self) -> Vec<(Vec3, MobKind, bool)> {
        self.mobs.mobs.iter().filter(|m| m.alive()).map(|m| (m.pos, m.kind, m.elite)).collect()
    }

    /// Fullt liv igjen, til gjennomspillingen i testene (som teller hvor
    /// ofte boten ville ha dødd).
    #[cfg(test)]
    pub fn heal(&mut self) {
        self.hp = self.max_hp;
    }

    #[cfg(test)]
    pub fn keys(&self) -> u32 {
        self.keys
    }

    /// Nøklene som ligger på gulvet etter en elite, til gjennomspillingen.
    #[cfg(test)]
    pub fn loose_keys(&self) -> Vec<Vec3> {
        self.mobs.keys.iter().filter(|k| !k.taken).map(|k| k.pos).collect()
    }

    /// Den nærmeste levende fienden (føttene), til testene.
    pub fn nearest_mob(&self, from: Vec3) -> Option<Vec3> {
        self.mobs.mobs.iter().filter(|m| m.alive()).map(|m| m.pos).min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }

    fn update_weather(&mut self, dt: f32) {
        let (t, fog) = self.weather_kind.target();
        let w = &mut self.weather;
        let ease = |v: &mut f32, goal: f32, rate: f32| *v += (goal - *v) * (dt * rate).min(1.0);
        ease(&mut w.cloud, t.cloud, 0.4);
        ease(&mut w.rain, t.rain, 0.6);
        ease(&mut w.snow, t.snow, 0.6);
        ease(&mut w.wind, t.wind, 0.3);
        let wet_rate = if t.wet > w.wet { 0.25 } else { 0.05 };
        ease(&mut w.wet, t.wet, wet_rate);
        let snow_rate = if t.snow_cover > w.snow_cover { 0.12 } else { 0.06 };
        ease(&mut w.snow_cover, t.snow_cover, snow_rate);
        ease(&mut self.fog_mul, fog, 0.3);
        // Lyn i uvær: korte blink med tilfeldige mellomrom.
        if self.weather_kind == WeatherKind::Storm {
            self.lightning_timer -= dt;
            if self.lightning_timer <= 0.0 {
                w.lightning = 1.0;
                self.lightning_timer = 3.0 + ((self.time * 7.3).sin() * 0.5 + 0.5) * 9.0;
            }
        }
        w.lightning *= (1.0 - dt * 9.0).max(0.0);
    }

    /// Portgitrene der de står nå (til bildet).
    pub(super) fn gate_poses(&self) -> Vec<Affine3A> {
        self.targets
            .iter()
            .filter_map(|t| match t.target.what {
                Use::Gate(n) => Some(t.target.pose * Affine3A::from_translation(Vec3::Y * gate_lift(self.gates[n].0))),
                _ => None,
            })
            .collect()
    }

    /// Lokkene på kistene: dreid opp om hengslet etter hvor langt de er
    /// åpnet, med en liten sprett når de slår mot stoppen.
    pub(super) fn chest_poses(&self) -> Vec<Affine3A> {
        self.targets
            .iter()
            .filter_map(|t| match t.target.what {
                Use::Chest(n) => {
                    let x = self.chest_open.get(n).copied().unwrap_or(0.0);
                    // Ease-out med litt overskyting (lokket slår mot stoppen).
                    let k = 1.0 + 2.2 * (x - 1.0).powi(3) + 1.2 * (x - 1.0).powi(2);
                    let angle = 1.85 * k.clamp(0.0, 1.08);
                    Some(t.target.pose * Affine3A::from_translation(dungeon::CHEST_HINGE) * Affine3A::from_rotation_x(-angle))
                }
                _ => None,
            })
            .collect()
    }

    /// Brønnvindene, tauene og bøttene der de er nå (til bildet).
    pub(super) fn well_poses(&self) -> Vec<(&'static str, Affine3A)> {
        let mut out = vec![];
        for t in &self.targets {
            let Use::Well(n) = t.target.what else { continue };
            let h = well_height(self.wells.get(n).copied().unwrap_or(-1.0));
            let pose = t.target.pose;
            let [winch, rope, bucket] = town::WELL_PARTS;
            let turn = (town::WELL_REST - h) / town::WELL_DRUM;
            out.push((winch, pose * Affine3A::from_translation(Vec3::Y * town::WELL_AXLE) * Affine3A::from_rotation_x(turn)));
            let top = Vec3::new(0.0, town::WELL_AXLE, town::WELL_ROPE_Z);
            out.push((rope, pose * Affine3A::from_translation(top) * Affine3A::from_scale(Vec3::new(1.0, town::WELL_AXLE - h, 1.0))));
            out.push((bucket, pose * Affine3A::from_translation(Vec3::new(0.0, h, town::WELL_ROPE_Z))));
        }
        out
    }
}

/// Når bøtta er oppe og du drikker, og når brønnen er ledig igjen (s).
const WELL_UP: f32 = 3.8;
const WELL_DONE: f32 = 4.2;

/// Høyden på bøyla over brønnkanten etter `t` sekunder: slippes ned i
/// vannet, ligger litt, og sveives opp i rykk.
fn well_height(t: f32) -> f32 {
    use town::{WELL_DEPTH as DOWN, WELL_REST as UP};
    if t < 0.0 || t >= WELL_UP {
        UP
    } else if t < 1.0 {
        UP + (DOWN - UP) * t * t
    } else if t < 1.4 {
        DOWN + ((t - 1.0) * 14.0).sin() * 0.02
    } else {
        let s = (t - 1.4) / (WELL_UP - 1.4);
        DOWN + (UP - DOWN) * s + (s * 40.0).sin() * 0.01 * (1.0 - s)
    }
}

/// Rykkete heving, som en sveiv.
fn gate_lift(t: f32) -> f32 {
    t * dungeon::GATE_LIFT + (t * 24.0).sin() * 0.02 * (1.0 - t)
}
