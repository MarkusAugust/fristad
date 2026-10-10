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
use crate::ports::driving::{Action, Ambience, Bubble, Event, Input, InventoryView, MapMark, MapView, Mark, Prompt, RecipeView, SaveData, Sound, WareView};
use crate::ports::{ColliderId, Physics};
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// En lyd i hodet: dine egne skritt, hugg og stønn.
fn sfx(name: &str) -> Event {
    Event::Sound(Sound::new(name))
}

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
    /// Portalen ved sørporten: dit den tar deg (føttene, retningen og
    /// etasjen) når den er åpen, hvor lenge den har vært åpen (s), om
    /// siktemålet står ute nå, og inngangen til hver etasje du har nådd.
    portal: Option<(Vec3, f32, usize)>,
    pub(super) portal_t: f32,
    portal_shown: bool,
    /// Kolliderne til ringen (flyttes inn når den står der).
    portal_solids: Vec<ColliderId>,
    entries: Vec<Option<(Vec3, f32)>>,
    /// Kartet: cellene du har sett i hver etasje, etasjen kartet sist ble
    /// sendt for (`None` oppe), om det må sendes på nytt, og hvor du sto da
    /// posisjonen sist ble sendt.
    seen: Vec<Vec<bool>>,
    map_floor: Option<usize>,
    map_dirty: bool,
    last_where: (f32, f32, f32),
    /// Sekunder siden du sist reiste gjennom virvelen (synsfeltet strekkes).
    pub(super) warp_t: f32,
    pub(super) herd: Herd,
    /// Der lydene i byen kommer fra: torget, smia og kirka (flatt), og
    /// lydbildet som sist ble sendt.
    sound_spots: [Vec2; 3],
    last_ambience: Ambience,
    /// Der du sist sto trygt i kjelleren (til fall i kløftene).
    safe: Vec3,
    /// Meter gått siden forrige skritt, og den raskeste fallfarten siden
    /// du sist sto (til landingen).
    stride: f32,
    fall: f32,
    /// Det som lager lyd i løkke der det står: ild (ildsteder og fakler),
    /// rennende vann og lava.
    fires: Vec<Vec3>,
    waters: Vec<Vec3>,
    lavas: Vec<Vec3>,
    /// Kløftene (vind fra dypet).
    chasms: Vec<Vec3>,
    /// Sekunder til neste fjerne lyd i kjelleren (rotter, kjetting, dører,
    /// noe som knurrer i et rom ved siden av), og terningen til dem.
    distant_t: f32,
    sound_seed: u32,
    /// Innendørs (tak over deg og vegger rundt), målt med stråler.
    indoor: bool,
    /// Innbyggerne: hvor de sto sist bilde og hvor langt de har gått siden
    /// forrige skritt (skrittene deres høres når du er nær).
    npc_steps: Vec<(Vec3, f32)>,
    /// Snarveidørene: hvor langt de er åpnet (0..1), −1 når de er stengt.
    pub(super) shortcuts: Vec<f32>,
    ambience_t: f32,
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
    /// Hver komponent du noen gang har fått i byen (galleriveggen i
    /// huset ditt). Det du selger eller bruker opp, blir hengende.
    found: BTreeSet<String>,
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
            portal: None,
            portal_t: 0.0,
            portal_shown: true,
            portal_solids: town::PORTAL_SOLIDS.iter().map(|(_, size)| physics.add_movable(*size * 0.5, Affine3A::from_translation(Vec3::new(0.0, -5000.0, 0.0)))).collect(),
            entries: vec![None; world.dungeon.floors.len()],
            seen: vec![vec![false; dungeon::N * dungeon::N]; world.dungeon.floors.len()],
            map_floor: None,
            map_dirty: false,
            last_where: (0.0, 0.0, 0.0),
            warp_t: 10.0,
            herd: Herd::new(),
            sound_spots: {
                let at = |name: &str| world.placements.iter().find(|p| p.model == name).and_then(|p| p.transforms.first()).map(|t| Vec2::new(t.translation.x, t.translation.z)).unwrap_or(Vec2::splat(1.0e4));
                let sq = crate::domain::world::town::square_center();
                [Vec2::new(sq.x, sq.z), at("forge"), at("stave_church")]
            },
            last_ambience: Ambience::default(),
            safe: Vec3::ZERO,
            stride: 0.0,
            fall: 0.0,
            fires: world.lights.iter().filter(|l| l.always).map(|l| l.pos).collect(),
            waters: Self::places(world, &["dng_hall_sewer", "dng_cistern"]),
            lavas: Self::places(world, &["dng_hall_lava", "dng_chasm"]),
            chasms: Self::places(world, &["dng_chasm"]),
            distant_t: 6.0,
            sound_seed: 0x9e37_79b9,
            indoor: false,
            npc_steps: vec![],
            shortcuts: vec![-1.0; world.dungeon.shortcuts.len()],
            ambience_t: 0.0,
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
            found: BTreeSet::new(),
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

    /// Hvor modellene med disse navnene står.
    fn places(world: &World, names: &[&str]) -> Vec<Vec3> {
        world.placements.iter().filter(|p| names.contains(&p.model)).flat_map(|p| p.transforms.iter().map(|t| Vec3::from(t.translation))).collect()
    }

    /// En lyd fra et sted i verden, sett fra øyet: x til høyre, y opp, −z
    /// rett fram (slik nettleserens lytter står).
    pub(super) fn sound_at(&self, name: &str, at: Vec3, gain: f32) -> Event {
        let d = at - self.eye();
        let (s, c) = self.yaw.sin_cos();
        Event::Sound(Sound { name: name.into(), at: Some([d.x * c - d.z * s, d.y, d.x * s + d.z * c]), gain })
    }

    /// Midten av et mål i verden.
    fn target_pos(&self, i: usize) -> Vec3 {
        let t = &self.targets[i].target;
        t.pose.transform_point3(t.center)
    }

    /// Hva du går på: navnet på skrittlyden.
    fn surface(&self, feet: Vec3) -> &'static str {
        if self.in_cellar() {
            // Plankebrua over kløfta, vann i kloakkgangene, ellers stein.
            if let Some(f) = self.dungeon.floor_of(feet + Vec3::Y * 0.5) {
                if self.waters.iter().any(|w| (w.y - self.dungeon.floors[f].y).abs() < 3.0 && w.with_y(0.0).distance(feet.with_y(0.0)) < 2.2) && self.dungeon.floors[f].theme == 2 {
                    return "step_water";
                }
            }
            if self.lavas.iter().any(|c| c.distance(feet) < 3.0) && self.dungeon.walkable(feet, 0.0) && !self.dungeon.walkable(feet + Vec3::X * 1.2, 0.0) {
                return "step_wood";
            }
            return "step_rock";
        }
        let ground = crate::domain::world::terrain::height(feet.x, feet.z);
        if feet.y > ground + 0.3 {
            // Gulv, trapper og galleri i husene (og murkronene).
            return if self.indoor { "step_wood" } else { "step_rock" };
        }
        if self.weather.snow_cover > 0.35 {
            return "step_snow";
        }
        let g = town::ground_weights(feet.x, feet.z);
        let grass = (1.0 - g.cobble - g.dirt - g.rock - g.forest).max(0.0);
        let (mut best, mut name) = (grass, "step_grass");
        for (w, n) in [(g.cobble, "step_rock"), (g.rock, "step_rock"), (g.dirt * (1.0 - g.earth), "step_gravel"), (g.dirt * g.earth, "step_dirt"), (g.forest, "step_dirt")] {
            if w > best {
                best = w;
                name = n;
            }
        }
        name
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
            Use::Sleep => p("Senga di", "Sov", "", "Sov til døgnet snur, og våkn med fullt liv og full utholdenhet."),
            Use::Gallery(n) => match craft::GALLERY.get(n) {
                Some(c) if self.found.contains(*c) => p(&craft::title(c), "Les", "", "Du har funnet den. Les om den i boka."),
                Some(c) => p("Tom ramme", "", "", &format!("Her henger {} når du har funnet den. {}", craft::title(c), craft::source(c).describe())),
                None => p("Tom ramme", "", "", ""),
            },
            Use::Talk(n) => {
                let name = self.npcs.npcs.get(n).map(|c| c.name).unwrap_or("Innbygger");
                let (trade, info) = match name {
                    "Buttonsmeden" => ("Handle", "Smeden selger våpen og skjold for tokens."),
                    "Apotekeren" => ("Handle", "Drikker som leger, og tokens for det du har funnet."),
                    "Gorvek" => ("Handle", "Lærer deg trekk på øvingsplassen."),
                    "Skriptoriet" => ("Smi og les", "Smi utstyr av komponentene fra krypten, og les om dem i bøkene."),
                    "Vertinnen" => ("", "Vertshuset. Her får du høre hva som går rykter om krypten."),
                    _ => ("", ""),
                };
                p(name, "Snakk", trade, info)
            }
            Use::Shortcut(n) => {
                if self.shortcuts.get(n).is_some_and(|o| *o >= 0.0) {
                    p("Snarveien", "", "", "Døra står åpen: veien tilbake til startrommet.")
                } else if self.shortcut_inside(n) {
                    p("Døra med slå", "Skyv slåen", "", "En snarvei tilbake til startrommet i etasjen.")
                } else {
                    p("Låst dør", "", "", "Boltet fra den andre siden. Kanskje du finner veien rundt.")
                }
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
            Use::Ascend => p("Døra opp til overflaten", "Gå opp", "", "Opp til krypten og byen. Det du bærer legges i lageret."),
            Use::Portal => match self.portal {
                Some((_, _, f)) => p("Portalen", "Gå gjennom", "", &format!("Virvelen tar deg tilbake til kjelleren, etasje {}.", f + 1)),
                None => p("Portalen", "", "", "Lukket."),
            },
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
        let mut jumped_now = false;
        if self.grounded {
            self.vel_y = -1.0;
            if input.jump && !self.paused && self.stance == Stance::Stand {
                self.vel_y = 5.4;
                jumped_now = true;
            }
        } else {
            self.vel_y -= 18.0 * dt;
        }
        let desired = (horizontal + Vec3::Y * self.vel_y) * dt;
        let (moved, grounded) = physics.move_capsule(self.pos, half, radius, desired, dt);
        self.pos += moved;
        let was_grounded = self.grounded;
        self.grounded = grounded;
        // Skrittene: ett per skrittlengde, etter underlaget, med rustningen
        // som rasler i takt; hopp og landing (hardere jo lenger fallet).
        {
            let feet = self.pos - Vec3::Y * (half + radius + 0.05);
            if jumped_now {
                events.push(sfx("jump"));
                self.fall = 0.0;
            }
            if !self.grounded {
                self.fall = self.fall.min(self.vel_y);
            } else if !was_grounded && !self.jumped {
                if self.fall < -4.0 {
                    let hard = ((-self.fall - 4.0) / 8.0).clamp(0.0, 1.0);
                    events.push(Event::Sound(Sound { name: "land".into(), at: None, gain: 0.5 + hard * 0.7 }));
                    events.push(Event::Sound(Sound { name: self.surface(feet).into(), at: None, gain: 0.9 }));
                    self.stride = 0.0;
                }
                self.fall = 0.0;
            }
            if self.grounded && !self.paused {
                self.stride += moved.with_y(0.0).length();
                let (length, gain) = match self.stance {
                    Stance::Stand if input.sprint => (1.25, 1.0),
                    Stance::Stand => (0.85, 0.75),
                    Stance::Crouch => (0.6, 0.35),
                    Stance::Crawl => (0.5, 0.25),
                };
                if self.stride >= length {
                    self.stride = 0.0;
                    let surface = self.surface(feet);
                    events.push(Event::Sound(Sound { name: surface.into(), at: None, gain }));
                    // Søle i regnvær.
                    if self.weather.wet > 0.6 && matches!(surface, "step_gravel" | "step_dirt") {
                        events.push(Event::Sound(Sound { name: "step_water".into(), at: None, gain: gain * 0.5 * self.weather.wet }));
                    }
                    // Tauet og plankene knirker på brua over kløfta.
                    if self.in_cellar() && surface == "step_wood" {
                        events.push(Event::Sound(Sound { name: "rope".into(), at: None, gain: gain * 0.8 }));
                    }
                    if let Some(a) = self.gear.armor {
                        let layer = if a.id.contains("mail") { "gear_mail" } else { "gear_leather" };
                        events.push(Event::Sound(Sound { name: layer.into(), at: None, gain }));
                    }
                }
            }
        }
        // Under nederste etasje i kjelleren er det ingenting.
        let abyss = self.dungeon.floors.last().map(|f| f.y).unwrap_or(dungeon::DEPTH) - 12.0;
        if self.pos.y < abyss {
            self.respawn();
        }
        if self.grounded && horizontal.length_squared() > 0.0 {
            self.bob += dt * speed / 4.2;
        }

        // Kløftene i kjelleren: står du trygt på gulvet, huskes stedet; har
        // du falt mer enn tre meter under gulvet, løftes du opp dit igjen,
        // med vondt.
        {
            let (half, radius) = self.stance.capsule();
            let feet = self.pos - Vec3::Y * (half + radius + 0.05);
            if self.in_cellar() {
                // (Vindeltrappa ned i tårncella er ikke et fall.)
                let stair = |f: usize| self.dungeon.floors[f].exit == Some(self.dungeon.floors[f].cell_of(feet));
                if let Some(f) = self.dungeon.floor_of(feet + Vec3::Y * 3.5).filter(|f| !stair(*f)) {
                    let floor_y = self.dungeon.floors[f].y;
                    if self.grounded && (feet.y - self.dungeon.height_at(feet)).abs() < 0.4 && self.dungeon.walkable(feet, 0.0) {
                        self.safe = feet;
                    } else if feet.y < floor_y - 3.0 && self.safe != Vec3::ZERO {
                        self.hp -= 30.0;
                        let back = self.safe;
                        self.teleport(back, self.yaw);
                        events.push(Event::Fade);
                        events.push(sfx("land"));
                        events.push(sfx("hurt"));
                        events.push(Event::Toast("Du falt i kløfta, og kom deg opp igjen med skrubbsår og en vond rygg.".into()));
                    }
                }
            }
        }
        // Hva sikter vi på? Fiendene i kjelleren går foran.
        let eye = self.eye();
        let dir = self.forward();
        let in_cellar = self.in_cellar();
        self.update_portal(dt, in_cellar, physics);
        self.update_map(in_cellar, &mut events);
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
                        events.push(sfx("dodge"));
                    }
                }
                Action::Daylight => {
                    self.daylight = if self.daylight == Daylight::Day { Daylight::Night } else { Daylight::Day };
                    // Kirkeklokka ringer inn skumringen og morgenen.
                    if !in_cellar {
                        let church = self.sound_spots[2];
                        events.push(self.sound_at("bell", Vec3::new(church.x, self.eye().y + 12.0, church.y), 2.5));
                    }
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
                Action::Home => self.go_home(&mut events),
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

        if self.update_weather(dt) && !in_cellar {
            events.push(Event::Sound(Sound { name: "thunder".into(), at: None, gain: if self.indoor { 0.6 } else { 1.0 } }));
        }
        // Oppe i dagslyset er det du bar trygt i lageret, uansett hvordan du
        // kom deg opp (døra i trappa, H eller «Tilbake til sørporten»).
        if !self.carried.is_empty() && !self.in_cellar() {
            self.bank(&mut events);
        }
        self.update_gates(physics, dt);
        self.update_wells(dt, &mut events);
        let (half, radius) = self.stance.capsule();
        let feet = self.pos - Vec3::Y * (half + radius);
        self.herd.update(dt, feet);
        // Innbyggerne: boblene som står sendes hvert bilde (og én tom når de er borte).
        let speeches = self.npcs.update(dt, feet);
        // Skrittene til innbyggerne som går nær deg.
        self.npc_steps.resize(self.npcs.npcs.len(), (Vec3::ZERO, 0.0));
        for i in 0..self.npcs.npcs.len() {
            let p = self.npcs.npcs[i].pos;
            let (last, stride) = self.npc_steps[i];
            let d = p.with_y(0.0).distance(last.with_y(0.0));
            let mut stride = if d < 1.0 { stride + d } else { 0.0 };
            if stride > 0.8 {
                stride = 0.0;
                if p.distance(feet) < 16.0 && !in_cellar {
                    let surface = self.surface(p + Vec3::Y * 0.05);
                    events.push(self.sound_at(surface, p, 0.45));
                }
            }
            self.npc_steps[i] = (p, stride);
        }
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
        self.ambience(dt, feet, in_cellar, physics, &mut events);
        events
    }

    /// F på mål nummer `i`.
    fn use_target(&mut self, i: usize, events: &mut Vec<Event>) {
        // Kister, porter og snarveier endrer merkene på kartet.
        self.map_dirty = true;
        match self.targets[i].target.what {
            Use::Sleep => {
                // Skjermen går i svart, og du våkner når døgnet har snudd.
                self.hp = self.max_hp;
                self.combat.stamina = self.combat.max_stamina;
                let d = if self.daylight == Daylight::Day { Daylight::Night } else { Daylight::Day };
                self.set_daylight(d, true);
                events.push(Event::Fade);
                events.push(Event::Daylight(d));
                let church = self.sound_spots[2];
                events.push(self.sound_at("bell", Vec3::new(church.x, self.eye().y + 12.0, church.y), 2.5));
                events.push(Event::Toast(if d == Daylight::Night { "Du sov til kvelden." } else { "Du sov til morgenen." }.into()));
            }
            Use::Gallery(n) => {
                if let Some(c) = craft::GALLERY.get(n).filter(|c| self.found.contains(**c)) {
                    events.push(Event::OpenDoc(c.to_string()));
                }
            }
            Use::Talk(n) => {
                if self.npcs.talk(n).is_some() {
                    let at = self.npcs.npcs.get(n).map(|p| p.pos + Vec3::Y * 1.6).unwrap_or(self.eye());
                    events.push(self.sound_at("talk", at, 1.0));
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
                events.push(self.sound_at("chest", self.target_pos(i), 1.0));
                events.push(sfx("loot"));
                events.push(Event::Loot(Some(pick.to_string())));
            }
            Use::Shortcut(n) => {
                if self.shortcuts.get(n).is_some_and(|o| *o >= 0.0) {
                    return;
                }
                if self.shortcut_inside(n) {
                    self.shortcuts[n] = 0.0;
                    if let Some(sc) = self.dungeon.shortcuts.get(n).copied() {
                        self.dungeon.floors[sc.floor].open(sc.edge);
                    }
                    events.push(self.sound_at("bolt", self.target_pos(i), 1.0));
                    events.push(self.sound_at("door", self.target_pos(i), 0.9));
                    events.push(Event::Toast("Du skyver slåen fra. Snarveien tilbake til startrommet er åpen.".into()));
                } else {
                    events.push(self.sound_at("locked", self.target_pos(i), 0.8));
                    events.push(Event::Toast("Døra er boltet fra den andre siden.".into()));
                }
            }
            Use::Gate(n) => {
                if self.gates[n].1 {
                    return;
                }
                if self.keys > 0 {
                    self.keys -= 1;
                    self.gates[n].1 = true;
                    events.push(sfx("key"));
                    events.push(self.sound_at("gate", self.target_pos(i), 1.0));
                    events.push(Event::Toast("Nøkkelen passer. Porten sveives opp; trappa ned ligger bak.".into()));
                } else {
                    events.push(self.sound_at("locked", self.target_pos(i), 1.0));
                    events.push(Event::Toast("Porten er låst. Elitefienden i rommet bærer nøkkelen.".into()));
                }
            }
            Use::Well(n) => {
                if let Some(w) = self.wells.get_mut(n) {
                    if *w < 0.0 {
                        *w = 0.0;
                        let at = self.target_pos(i);
                        events.push(self.sound_at("well", at, 1.0));
                    }
                }
            }
            Use::Descend => {
                let to = self.dungeon.start;
                if let Some(e) = self.entries.first_mut() {
                    e.get_or_insert((to, 0.0));
                }
                self.teleport(to, 0.0);
                events.push(Event::Fade);
                events.push(sfx("crypt_door"));
                events.push(Event::Toast("Ned i kjelleren under borgen. Venstre mus hugger (hold for tungt), høyre mus blokker, Shift og retning er unnamanøver.".into()));
            }
            Use::Portal => {
                let Some((to, yaw, floor)) = self.portal.take() else { return };
                self.warp(events);
                self.teleport(to, yaw);
                events.push(Event::Toast(format!("Gjennom virvelen og ned i etasje {}. Portalen lukker seg bak deg.", floor + 1)));
            }
            Use::Ascend => {
                let (to, yaw) = self.crypt_exit;
                self.teleport(to, yaw);
                events.push(Event::Fade);
                events.push(sfx("door"));
                self.bank(events);
            }
        }
    }

    /// H (eller «Tilbake til sørporten»): fra kjelleren gjennom en virvel
    /// til sørporten, og portalen der våkner og husker hvor du var. Oppe i
    /// byen bare tilbake til porten.
    pub(super) fn go_home(&mut self, events: &mut Vec<Event>) {
        if self.in_cellar() && self.hp > 0.0 {
            let (half, radius) = self.stance.capsule();
            let feet = self.pos - Vec3::Y * (half + radius + 0.05);
            let floor = self.dungeon.floor_of(feet).unwrap_or(0);
            self.portal = Some((feet, self.yaw, floor));
            self.portal_t = 0.0;
            self.warp(events);
            self.respawn();
            // Snu deg mot portalen, så du ser den våkne.
            if let Some(ring) = self.portal_pose() {
                let d = Vec3::from(ring.translation) - self.pos;
                self.yaw = (-d.x).atan2(-d.z);
                self.pitch = 0.05;
            }
            let c = self.portal_center().unwrap_or(self.pos);
            events.push(self.sound_at("gate", c - Vec3::Y * 1.4, 1.0));
            events.push(self.sound_at("portal_open", c, 1.0));
            events.push(Event::Toast("Virvelen tok deg hjem. Portalen ved sørporten tar deg tilbake dit du var.".into()));
        } else {
            self.respawn();
            events.push(Event::Fade);
            events.push(Event::Toast("Tilbake ved sørporten".into()));
        }
    }

    /// Hendelser som skal ut neste bilde.
    pub fn queue(&mut self, events: Vec<Event>) {
        self.pending.extend(events);
    }

    /// «Tilbake til sørporten» fra menyen i nettleseren.
    pub fn go_home_now(&mut self) {
        let mut events = vec![];
        self.go_home(&mut events);
        self.pending.extend(events);
    }

    fn warp(&mut self, events: &mut Vec<Event>) {
        self.warp_t = 0.0;
        self.jumped = true;
        events.push(Event::Warp);
        events.push(sfx("warp"));
    }

    /// Ringen (plattformens midte og retning), om byen har den.
    pub(super) fn portal_pose(&self) -> Option<Affine3A> {
        self.targets.iter().find(|t| t.target.what == Use::Portal).map(|t| t.target.pose)
    }

    fn portal_center(&self) -> Option<Vec3> {
        self.portal_pose().map(|p| p.transform_point3(Vec3::Y * town::PORTAL_CENTER))
    }

    /// Hvor portalen tar deg nå (føttene, retningen og etasjen).
    pub fn portal_destination(&self) -> Option<(Vec3, f32, usize)> {
        self.portal
    }

    /// Klokkene til portalen og virvelen, inngangene til etasjene, og
    /// siktemålet i ringen (bare ute når portalen er åpen).
    fn update_portal(&mut self, dt: f32, in_cellar: bool, physics: &mut dyn Physics) {
        self.warp_t += dt;
        if self.portal.is_some() {
            self.portal_t += dt;
        }
        if in_cellar && self.grounded && self.hp > 0.0 {
            let (half, radius) = self.stance.capsule();
            let feet = self.pos - Vec3::Y * (half + radius + 0.05);
            if let Some(f) = self.dungeon.floor_of(feet) {
                if self.entries.get(f).is_some_and(|e| e.is_none()) && self.dungeon.walkable(feet, 0.3) {
                    self.entries[f] = Some((feet, self.yaw));
                }
            }
        }
        let show = self.portal.is_some();
        if show != self.portal_shown {
            self.portal_shown = show;
            let away = Affine3A::from_translation(Vec3::new(0.0, -5000.0, 0.0));
            if let Some(t) = self.targets.iter().find(|t| t.target.what == Use::Portal) {
                physics.set_pose(t.collider, if show { t.target.pose * Affine3A::from_translation(t.target.center) } else { away });
                for (id, (c, _)) in self.portal_solids.iter().zip(town::PORTAL_SOLIDS) {
                    physics.set_pose(*id, if show { t.target.pose * Affine3A::from_translation(c) } else { away });
                }
            }
        }
    }

    /// Kartet: avdekk det du ser (hele rommet du står i, og gangene et par
    /// celler rundt deg), send kartet når det endrer seg, og hvor du står
    /// hvert bilde.
    fn update_map(&mut self, in_cellar: bool, events: &mut Vec<Event>) {
        let (half, radius) = self.stance.capsule();
        let feet = self.pos - Vec3::Y * (half + radius + 0.05);
        let floor = if in_cellar { self.dungeon.floor_of(feet) } else { None };
        let Some(k) = floor else {
            if self.map_floor.is_some() {
                self.map_floor = None;
                events.push(Event::Map(None));
            }
            return;
        };
        let n = dungeon::N as i32;
        let f = &self.dungeon.floors[k];
        let me = f.cell_of(feet);
        let unit = f.unit_at(me);
        let seen = &mut self.seen[k];
        let mut fresh = false;
        for iz in (me.1 - 3).max(0)..(me.1 + 4).min(n) {
            for ix in (me.0 - 3).max(0)..(me.0 + 4).min(n) {
                let i = (iz * n + ix) as usize;
                if seen[i] || f.cell(ix, iz) == dungeon::Cell::Rock {
                    continue;
                }
                let near = ((ix - me.0) as f32).hypot((iz - me.1) as f32) <= 2.3;
                if near || (unit >= 0 && f.unit_at((ix, iz)) == unit) {
                    seen[i] = true;
                    fresh = true;
                }
            }
        }
        // Store rom kan gå utenfor ruta rundt deg: hele rommet avdekkes.
        if unit >= 0 && fresh {
            for iz in 0..n {
                for ix in 0..n {
                    if f.unit_at((ix, iz)) == unit {
                        seen[(iz * n + ix) as usize] = true;
                    }
                }
            }
        }
        if fresh || self.map_floor != Some(k) || self.map_dirty {
            self.map_floor = Some(k);
            self.map_dirty = false;
            events.push(Event::Map(Some(self.map_view(k))));
        }
        let w = (feet.x, feet.z, self.yaw);
        if (w.0 - self.last_where.0).abs() + (w.1 - self.last_where.1).abs() > 0.02 || (w.2 - self.last_where.2).abs() > 0.005 {
            self.last_where = w;
            events.push(Event::Where { x: w.0, z: w.1, yaw: w.2 });
        }
    }

    /// Kartet over etasje `k` slik du kjenner det.
    fn map_view(&self, k: usize) -> MapView {
        let n = dungeon::N as i32;
        let f = &self.dungeon.floors[k];
        let seen = &self.seen[k];
        let mut cells = vec![0u8; (n * n) as usize];
        for iz in 0..n {
            for ix in 0..n {
                if f.cell(ix, iz) == dungeon::Cell::Rock {
                    continue;
                }
                let mut c = 1;
                if f.passable((ix, iz), (ix + 1, iz)) {
                    c |= 2;
                }
                if f.passable((ix, iz), (ix, iz + 1)) {
                    c |= 4;
                }
                cells[(iz * n + ix) as usize] = c;
            }
        }
        let known = |p: Vec3| {
            let c = f.cell_of(p);
            (0..n).contains(&c.0) && (0..n).contains(&c.1) && seen[(c.1 * n + c.0) as usize]
        };
        let mut marks = vec![];
        // Trappa ned står i en celle uten gulv: den synes når en nabo er sett.
        if let Some(e) = f.exit {
            let c = f.center(e.0, e.1);
            if [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| (0..n).contains(&(e.0 + dx)) && (0..n).contains(&(e.1 + dz)) && seen[((e.1 + dz) * n + e.0 + dx) as usize]) {
                marks.push(MapMark { kind: "exit".into(), x: c.x, z: c.z });
            }
        }
        let mut mark = |kind: &str, p: Vec3| {
            if known(p) {
                marks.push(MapMark { kind: kind.into(), x: p.x, z: p.z });
            }
        };
        for (g, (p, _)) in self.dungeon.gates.iter().enumerate() {
            if self.dungeon.floor_of(*p) == Some(k) {
                mark(if self.gates.get(g).is_some_and(|s| s.1) { "gate_open" } else { "gate" }, *p);
            }
        }
        for (c, (p, _, fl)) in self.dungeon.chests.iter().enumerate() {
            if *fl == k && !self.chests.get(c).copied().unwrap_or(true) {
                mark("chest", *p);
            }
        }
        for s in self.dungeon.shortcuts.iter().filter(|s| s.floor == k) {
            mark("shortcut", s.pos);
        }
        if k == 0 {
            if let Some(t) = self.targets.iter().find(|t| t.target.what == Use::Ascend) {
                mark("ascend", t.target.pose.transform_point3(Vec3::ZERO));
            }
        }
        if let (Some(b), false) = (self.dungeon.boss, self.won) {
            if self.dungeon.floor_of(b) == Some(k) {
                mark("boss", b);
            }
        }
        MapView { floor: k, name: dungeon::EPOCHS.get(k).copied().unwrap_or("").into(), origin: [f.origin.0, f.origin.1], cell: dungeon::CELL, n: dungeon::N, cells, seen: seen.clone(), marks }
    }

    /// Den åpne portalen: runene, virvelen (to lag som dreier hver sin vei
    /// og pulserer) og splintene som svever rundt ringen. Den vokser fram de
    /// første sekundene med litt overskyting.
    pub(super) fn portal_poses(&self) -> Vec<(&'static str, Affine3A)> {
        let (Some(pose), true) = (self.portal_pose(), self.portal.is_some()) else { return vec![] };
        let [runes, swirl, shard] = town::PORTAL_PARTS;
        // Ringen stiger opp av bakken (med en liten skjelving), så åpner
        // virvelen seg.
        let r = (self.portal_t / PORTAL_RISE).clamp(0.0, 1.0);
        let rise = 1.0 - (1.0 - r).powi(3);
        let shake = if r < 1.0 { (self.portal_t * 47.0).sin() * 0.02 * (1.0 - r) } else { 0.0 };
        let ring = pose * Affine3A::from_translation(Vec3::new(shake, -3.4 * (1.0 - rise), 0.0));
        let mut out = vec![(town::PORTAL_RING, ring)];
        let x = ((self.portal_t - PORTAL_RISE) / 1.6).clamp(0.0, 1.0);
        if x <= 0.0 {
            return out;
        }
        let grow = (1.0 + 2.2 * (x - 1.0).powi(3) + 1.2 * (x - 1.0).powi(2)).max(0.001);
        let t = self.time;
        let center = pose * Affine3A::from_translation(Vec3::Y * town::PORTAL_CENTER);
        out.push((runes, pose));
        let s1 = grow * (1.0 + 0.035 * (t * 3.1).sin());
        out.push((swirl, center * Affine3A::from_rotation_z(-t * 1.7) * Affine3A::from_scale(Vec3::splat(s1))));
        let s2 = grow * 0.8 * (1.0 + 0.05 * (t * 2.3 + 1.0).sin());
        out.push((swirl, center * Affine3A::from_translation(Vec3::Z * -0.06) * Affine3A::from_rotation_z(t * 1.1 + 1.0) * Affine3A::from_scale(Vec3::splat(s2))));
        for k in 0..town::PORTAL_SHARDS {
            let a = t * 0.55 + k as f32 * std::f32::consts::TAU / town::PORTAL_SHARDS as f32;
            let wobble = (t * 1.3 + k as f32 * 2.1).sin();
            let at = Vec3::new(a.cos() * (2.05 + 0.1 * wobble), 1.95 + a.sin() * 1.6, 0.25 + 0.2 * (t * 0.9 + k as f32).sin());
            let spin = Quat::from_rotation_y(t * 2.0 + k as f32) * Quat::from_rotation_x(0.5 * wobble);
            out.push((shard, pose * Affine3A::from_scale_rotation_translation(Vec3::splat(grow * (0.8 + 0.25 * (k % 3) as f32)), spin, at)));
        }
        out
    }

    /// Lyset fra den åpne portalen: fiolett, pulserende.
    pub(super) fn portal_light(&self) -> Option<Light> {
        let c = self.portal_center()?;
        if self.portal.is_none() {
            return None;
        }
        let x = ((self.portal_t - PORTAL_RISE) / 1.6).clamp(0.0, 1.0);
        let t = self.time;
        let front = self.portal_pose()?.transform_vector3(Vec3::Z);
        Some(Light { pos: c + front * 0.6, radius: 11.0, color: Vec3::new(0.55, 0.45, 1.0), intensity: 2.4 * x * (0.85 + 0.15 * (t * 4.7).sin() * (t * 1.9).cos()), always: true, flicker: 0.0 })
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
        for n in 0..self.shortcuts.len() {
            if self.shortcuts[n] >= 0.0 && self.shortcuts[n] < 1.0 {
                self.shortcuts[n] = (self.shortcuts[n] + dt / 1.2).min(1.0);
                if let (Some(pose), Some(t)) = (self.shortcut_pose(n), self.targets.iter().find(|t| t.target.what == Use::Shortcut(n))) {
                    use crate::domain::world::dungeon::{SHORTCUT_H, SHORTCUT_W};
                    physics.set_pose(t.collider, pose * Affine3A::from_translation(Vec3::new(SHORTCUT_W * 0.5, SHORTCUT_H * 0.5, 0.0)));
                }
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
            events.push(sfx("pickup"));
        }
        if outcome.keys > 0 {
            self.keys += outcome.keys;
            events.push(sfx("key"));
            events.push(Event::Toast("En rusten nøkkel. Porten i utgangsrommet kan låses opp.".into()));
        }
        for (name, at) in &outcome.sounds {
            // Tassingen høres bare når de er nær.
            if *name == "mob_step" && at.distance(feet) > 14.0 {
                continue;
            }
            events.push(self.sound_at(name, *at + Vec3::Y * if *name == "mob_step" { 0.0 } else { 1.0 }, 1.0));
        }
        for _ in 0..outcome.roars {
            events.push(Event::Toast("Rammeverket brøler, og tooltips kryper ut av veggene.".into()));
        }
        if outcome.quakes > 0 {
            self.combat.quake();
        }
        // Bittene: blokk tar det meste, en parade alt (og lamslår), en
        // unnamanøver går klar.
        let mut taken_total = 0.0;
        for (mob, damage, from) in &outcome.bites {
            let (taken, parried) = self.combat.incoming(*from, *damage, eye, fwd);
            if parried {
                self.mobs.stagger(*mob, 1.3);
                events.push(sfx("parry"));
            } else if taken < *damage * 0.99 && taken > 0.0 {
                events.push(sfx(if self.gear.shield.model == "shield_tower" { "block_metal" } else { "block" }));
            } else if taken > 0.0 {
                events.push(sfx("hurt"));
            }
            taken_total += taken;
        }
        if taken_total > 0.0 {
            self.hp -= taken_total;
            if self.hp <= 0.0 {
                self.die(events);
            }
        }
        // Våre hugg. Treffet høres der fienden står, med materialet den er
        // laget av (tabellen er tre, Rammeverket jern).
        let mut struck: Option<Vec3> = None;
        for e in self.combat.update(dt, sim_dt, eye, fwd, feet, &mut self.mobs, &self.dungeon) {
            match e {
                CombatEvent::Hit { mob, damage, killed, pos } => {
                    self.popup_seq = self.popup_seq.wrapping_add(1);
                    self.popups.push((pos, damage, 0.0, self.popup_seq));
                    struck = Some(pos);
                    let material = match self.mobs.mobs.get(mob).map(|m| m.kind) {
                        Some(MobKind::Table) => Some("hit_wood"),
                        Some(MobKind::Framework) => Some("hit_metal"),
                        _ => None,
                    };
                    if let Some(m) = material {
                        events.push(self.sound_at(m, pos, 1.0));
                    }
                    if killed {
                        self.killed(mob, pos, events);
                    }
                }
                CombatEvent::Miss => {}
                CombatEvent::Sound(s) => match struck {
                    Some(at) if s != "swing" && s != "swing_heavy" => events.push(self.sound_at(s, at, 1.0)),
                    _ => events.push(sfx(s)),
                },
            }
        }
    }

    /// Lydbildet: hvor nær du er torget, smia, kirka og dyrene, sendt fire
    /// ganger i sekundet når det har endret seg.
    fn ambience(&mut self, dt: f32, feet: Vec3, in_cellar: bool, physics: &mut dyn Physics, events: &mut Vec<Event>) {
        self.ambience_t -= dt;
        if self.ambience_t > 0.0 {
            return;
        }
        self.ambience_t = 0.25;
        // Fjerne lyder i kjelleren: noe beveger seg i rommene ved siden av.
        if in_cellar {
            self.distant_t -= 0.25;
            if self.distant_t <= 0.0 {
                let mut roll = || {
                    self.sound_seed ^= self.sound_seed << 13;
                    self.sound_seed ^= self.sound_seed >> 17;
                    self.sound_seed ^= self.sound_seed << 5;
                    (self.sound_seed % 10_000) as f32 / 10_000.0
                };
                let (wait, pick, angle, dist) = (roll(), roll(), roll(), roll());
                self.distant_t = 8.0 + wait * 14.0;
                let name = ["rats", "rats", "door_close", "locked", "notice", "windup", "crack"][(pick * 7.0) as usize % 7];
                let a = angle * std::f32::consts::TAU;
                let at = self.eye() + Vec3::new(a.cos(), 0.0, a.sin()) * (12.0 + dist * 14.0);
                events.push(self.sound_at(name, at, if name == "rats" { 0.9 } else { 0.35 }));
            }
        }
        // Innendørs: tak over deg og vegger på minst tre kanter.
        let eye = self.eye();
        let roof = physics.cast_ray(eye, Vec3::Y, 12.0).is_some();
        let walls = [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z].iter().filter(|d| physics.cast_ray(eye, **d, 9.0).is_some()).count();
        self.indoor = !in_cellar && roof && walls >= 3;
        let close = |list: &[Vec3], r: f32| list.iter().map(|q| (1.0 - q.distance(eye) / r).clamp(0.0, 1.0)).fold(0.0, f32::max).powi(2);
        let p = Vec2::new(feet.x, feet.z);
        let near = |at: Vec2, r: f32| (1.0 - p.distance(at) / r).clamp(0.0, 1.0).powi(2);
        let [square, forge, church] = self.sound_spots;
        let meadow = self.herd.positions().map(|a| near(a, 45.0)).fold(0.0, f32::max);
        let a = Ambience {
            square: near(square, 55.0),
            forge: near(forge, 32.0),
            church: near(church, 140.0),
            meadow,
            fire: close(&self.fires, 7.0),
            water: if in_cellar { close(&self.waters, 9.0) } else { 0.0 },
            lava: if in_cellar { close(&self.lavas, 10.0) } else { 0.0 },
            chasm: if in_cellar { close(&self.chasms, 14.0) } else { 0.0 },
            portal: match (in_cellar, self.portal.is_some(), self.portal_center()) {
                (false, true, Some(c)) => close(&[c], 16.0),
                _ => 0.0,
            },
            rain: if in_cellar { 0.0 } else { self.weather.rain },
            wind: if in_cellar { 0.0 } else { self.weather.wind },
            night: self.daylight == Daylight::Night,
            cellar: in_cellar,
            indoor: self.indoor,
        };
        let last = self.last_ambience;
        let moved = [a.square - last.square, a.forge - last.forge, a.church - last.church, a.meadow - last.meadow, a.fire - last.fire, a.water - last.water, a.lava - last.lava, a.chasm - last.chasm, a.portal - last.portal, a.rain - last.rain, a.wind - last.wind]
            .iter()
            .fold(0.0f32, |m, d| m.max(d.abs()));
        if moved > 0.02 || a.night != last.night || a.cellar != last.cellar || a.indoor != last.indoor {
            self.last_ambience = a;
            events.push(Event::Ambience(a));
        }
    }

    /// Død: opp til krypten, det du bar er borte, en firedel av tokenene
    /// blir igjen nede. Fiendene kommer igjen, men portene du har låst opp,
    /// står fortsatt oppe.
    fn die(&mut self, events: &mut Vec<Event>) {
        let lost = self.coins / 4;
        self.coins -= lost;
        self.mobs.reset(&self.dungeon);
        self.combat.reset();
        self.popups.clear();
        self.keys = 0;
        self.hp = self.max_hp;
        // Portgitrene du har låst opp, står oppe: veien ned er din.
        if !self.carried.is_empty() {
            let n: u32 = self.carried.values().sum();
            self.carried.clear();
            self.inv_dirty = true;
            events.push(Event::Toast(format!("{n} komponenter ble igjen i kjelleren.")));
        }
        self.read_docs.clear();
        // Portalen ved sørporten tar deg til inngangen til den dypeste
        // etasjen du har nådd.
        let deepest = self.entries.iter().enumerate().rev().find_map(|(f, e)| e.map(|(p, y)| (p, y, f)));
        if let Some((p, y, f)) = deepest {
            self.portal = Some((p, y, f));
            self.portal_t = 0.0;
            events.push(Event::Toast(format!("Portalen ved sørporten har våknet: den tar deg til inngangen til etasje {}.", f + 1)));
        }
        let (to, yaw) = self.crypt_exit;
        self.teleport(to, yaw);
        events.push(Event::Fade);
        events.push(sfx("death"));
        events.push(Event::Toast(format!("Du ble overkjørt av forretningslogikk. {lost} tokens ble igjen nede.")));
    }

    fn killed(&mut self, mob: usize, pos: Vec3, events: &mut Vec<Event>) {
        let (kind, elite) = self.mobs.mobs.get(mob).map(|m| (m.kind, m.elite)).unwrap_or((MobKind::Button, false));
        if kind.boss() {
            for c in craft::BOSS_LOOT {
                *self.carried.entry(c.to_string()).or_insert(0) += 1;
            }
            self.inv_dirty = true;
            self.won = true;
            self.map_dirty = true;
            events.push(Event::Toast("Rammeverket er beseiret! Skatten i alkoven er din, og en dialog og en tabs ligger igjen i buret.".into()));
        } else if elite {
            let floor = self.dungeon.floor_of(pos).unwrap_or(0).min(craft::ELITE_LOOT.len() - 1);
            let c = craft::ELITE_LOOT[floor];
            *self.carried.entry(c.to_string()).or_insert(0) += 1;
            self.inv_dirty = true;
            events.push(self.sound_at("key", pos - Vec3::Y * 0.8, 1.0));
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
        self.discover();
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
            "Buttonsmeden" => Some(Event::Shop { npc: name.into(), wares: wares(craft::SMITH_WARES) }),
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
                    sources: r.needs.iter().map(|(c, _)| *c).chain(r.requires.iter().copied()).map(|c| {
                        let s = craft::source(c);
                        (c.to_string(), s.describe(), s.crypt_only())
                    }).collect(),
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
            self.pending.push(sfx("deny"));
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
        self.pending.push(sfx("craft"));
        self.pending.push(sfx(match r.result {
            craft::Result::Shield(_) => "equip_wood",
            craft::Result::Armor(_) => "equip_leather",
            _ => "equip_metal",
        }));
        self.pending.push(Event::Toast(format!("Skjemaet er godkjent. {} er smidd: {what} er byttet.", r.name)));
        true
    }

    /// Kjøp i en butikk, for tokens. Gir om det gikk.
    pub fn buy(&mut self, id: &str) -> bool {
        let Some(w) = craft::ware(id) else { return false };
        if self.coins < w.price {
            self.pending.push(sfx("deny"));
            self.pending.push(Event::Toast(format!("Du mangler {} tokens.", w.price - self.coins)));
            return false;
        }
        self.coins -= w.price;
        match w.what {
            craft::WareKind::Component(c) => {
                *self.inventory.entry(c.to_string()).or_insert(0) += 1;
                self.discover();
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
        self.pending.push(sfx("coins"));
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
        self.pending.push(sfx("book"));
        if self.hp > before {
            self.pending.push(sfx("heal"));
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
            found: self.found.iter().cloned().collect(),
        }
    }

    pub fn load(&mut self, d: SaveData) {
        self.inventory = d.stash.into_iter().collect();
        self.found = d.found.into_iter().collect();
        self.discover();
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
        self.discover();
        self.inv_dirty = true;
    }

    /// Det som ligger i lageret er funnet, for alltid.
    fn discover(&mut self) {
        for (c, n) in &self.inventory {
            if *n > 0 && !self.found.contains(c) {
                self.found.insert(c.clone());
            }
        }
    }

    /// Komponentene du har funnet (galleriveggen).
    pub fn found(&self) -> Vec<String> {
        self.found.iter().cloned().collect()
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
    pub fn hp_now(&self) -> f32 {
        self.hp
    }

    #[cfg(test)]
    pub fn stamina_now(&self) -> f32 {
        self.combat.stamina
    }

    /// Til testene: sett liv og utholdenhet.
    #[cfg(test)]
    pub fn set_vitals(&mut self, hp: f32, stamina: f32) {
        self.hp = hp;
        self.combat.stamina = stamina;
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

    /// Felle eliten i en etasje med ett slag, til testene.
    #[cfg(test)]
    pub fn fell_elite(&mut self, floor: usize, events: &mut Vec<Event>) {
        let i = self.mobs.mobs.iter().position(|m| m.alive() && m.elite && self.dungeon.floor_of(m.pos) == Some(floor)).expect("eliten");
        let from = self.mobs.mobs[i].pos + Vec3::X;
        let pos = self.mobs.mobs[i].pos;
        if self.mobs.strike(i, 1.0e6, from, &self.dungeon).killed {
            self.killed(i, pos, events);
        }
    }

    /// Dø, til testene.
    #[cfg(test)]
    pub fn die_now(&mut self) -> Vec<Event> {
        let mut events = vec![];
        self.die(&mut events);
        events
    }

    /// Hvor langt portgitteret er heist (0–1) og om det er låst opp.
    #[cfg(test)]
    pub fn gate(&self, n: usize) -> (f32, bool) {
        self.gates[n]
    }

    /// Den nærmeste levende fienden (føttene), til testene.
    pub fn nearest_mob(&self, from: Vec3) -> Option<Vec3> {
        self.mobs.mobs.iter().filter(|m| m.alive()).map(|m| m.pos).min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }

    /// Været glir mot målet. Gir om det lynte (tordenen kommer etter).
    fn update_weather(&mut self, dt: f32) -> bool {
        let mut flash = false;
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
                flash = true;
                self.lightning_timer = 3.0 + ((self.time * 7.3).sin() * 0.5 + 0.5) * 9.0;
            }
        }
        w.lightning *= (1.0 - dt * 9.0).max(0.0);
        flash
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

    /// Om du står på gangsida av snarveidøra (der slåen sitter).
    fn shortcut_inside(&self, n: usize) -> bool {
        let Some(sc) = self.dungeon.shortcuts.get(n) else { return false };
        let into_room = Vec3::new(sc.yaw.sin(), 0.0, sc.yaw.cos());
        (self.pos - sc.pos).with_y(0.0).dot(into_room) < 0.0
    }

    /// Snarveidøra dreid om hengslet, ut mot gangen, etter hvor langt den
    /// er åpnet.
    fn shortcut_pose(&self, n: usize) -> Option<Affine3A> {
        use crate::domain::world::dungeon::SHORTCUT_HINGE;
        let sc = self.dungeon.shortcuts.get(n)?;
        let open = self.shortcuts.get(n).copied().unwrap_or(-1.0).max(0.0);
        let ease = 1.0 - (1.0 - open).powi(3);
        let frame = Affine3A::from_rotation_translation(Quat::from_rotation_y(sc.yaw), sc.pos);
        Some(frame * Affine3A::from_translation(SHORTCUT_HINGE) * Affine3A::from_rotation_y(ease * 1.75))
    }

    pub(super) fn shortcut_poses(&self) -> Vec<Affine3A> {
        (0..self.shortcuts.len()).filter_map(|n| self.shortcut_pose(n)).collect()
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

    /// Komponentene på galleriveggen: én i hver ramme du har fylt, i
    /// rammens pose og med sin egen farge.
    pub(super) fn gallery_poses(&self) -> Vec<(Affine3A, Vec3)> {
        self.targets
            .iter()
            .filter_map(|t| match t.target.what {
                Use::Gallery(n) => {
                    let c = craft::GALLERY.get(n)?;
                    self.found.contains(*c).then(|| (t.target.pose, Vec3::from(craft::GALLERY_TINTS[n])))
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

/// Sekunder ringen bruker på å stige opp av bakken før virvelen åpner seg.
const PORTAL_RISE: f32 = 1.4;

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
