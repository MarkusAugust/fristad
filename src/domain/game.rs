//! Spillet: spilleren, døgnet, været og samspillet mellom installasjonene.
//! Fysikken går gjennom `ports::Physics`, så kjernen vet ikke hvilken
//! motor som brukes.

use crate::domain::city::City;
use crate::domain::installations::{self, Action as InstallAction, Env, Installation, WeatherKind};
use crate::domain::palette;
use crate::domain::ports::{ColliderId, Physics};
use crate::domain::scene::{Frame, Light, Weather};
use crate::domain::tokens::{Theme, Tokens};
use crate::domain::craft::{self, Gear};
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Default)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    /// C holdes inne: huk.
    pub crouch: bool,
    /// Høyre musetast holdes inne: blokk med skjoldet.
    pub block: bool,
    pub look: (f32, f32),
    pub actions: Vec<Action>,
}

impl Input {
    /// Tast trykket eller sluppet. `code` er `KeyboardEvent.code` i
    /// nettleseren, og samme navn fra winit native: "KeyW", "Space" osv.
    pub fn key(&mut self, code: &str, down: bool) {
        match code {
            "KeyW" | "ArrowUp" => self.forward = down,
            "KeyS" | "ArrowDown" => self.back = down,
            "KeyA" | "ArrowLeft" => self.left = down,
            "KeyD" | "ArrowRight" => self.right = down,
            "Space" => self.jump = down,
            "ShiftLeft" | "ShiftRight" => {
                self.sprint = down;
                // I kjelleren er Shift med en retning en unnamanøver.
                if down {
                    self.actions.push(Action::Dodge);
                }
            }
            "KeyC" => self.crouch = down,
            "KeyX" if down => self.actions.push(Action::Crawl),
            "KeyE" if down => self.actions.push(Action::Inspect),
            "KeyR" if down => self.actions.push(Action::Variant),
            "KeyF" if down => self.actions.push(Action::Use),
            "KeyT" if down => self.actions.push(Action::Theme),
            "KeyV" if down => self.actions.push(Action::Weather),
            "KeyH" if down => self.actions.push(Action::Home),
            _ => {}
        }
    }

    /// Musetast trykket eller sluppet: 0 er venstre (hugg, hold for tungt),
    /// 2 er høyre (blokk mens den holdes).
    pub fn mouse(&mut self, button: u32, down: bool) {
        match button {
            0 => self.actions.push(if down { Action::Attack } else { Action::AttackRelease }),
            2 => self.block = down,
            _ => {}
        }
    }

    /// Et tegn skrevet mens et felt har fokus. `key` er `KeyboardEvent.key`:
    /// ett tegn, eller "Backspace", "Enter", "Escape".
    pub fn text(&mut self, key: &str) {
        let c = match key {
            "Backspace" => '\u{8}',
            "Enter" => '\n',
            "Escape" => '\u{1b}',
            _ => {
                let mut it = key.chars();
                match (it.next(), it.next()) {
                    (Some(c), None) => c,
                    _ => return,
                }
            }
        };
        self.actions.push(Action::Text(c));
    }

    /// Slipp alle taster, for eksempel når vinduet mister fokus.
    pub fn release_all(&mut self) {
        let actions = std::mem::take(&mut self.actions);
        *self = Input { actions, ..Default::default() };
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// E: åpne komponenten i et panel med dokumentasjon.
    Inspect,
    /// R: neste variant.
    Variant,
    /// F: bruk.
    Use,
    /// X: kryp av og på.
    Crawl,
    /// T: bytt tema, og dermed dag og natt.
    Theme,
    /// V: neste vær.
    Weather,
    /// H: tilbake til torget.
    Home,
    /// Et tegn til feltet som har fokus.
    Text(char),
    /// Venstre musetast ned: hugg (i kjelleren), ellers som F.
    Attack,
    AttackRelease,
    /// Shift trykket: unnamanøver i kjelleren om du går en vei.
    Dodge,
}

/// Et merke på skjermen (0..1 fra øverst til venstre): helsestolpe over en
/// fiende som er truffet, eller et skadetall som stiger.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    /// "bar" eller "text".
    pub kind: &'static str,
    /// Stolpens fyll, eller tekstens styrke, 0..1.
    pub value: f32,
    pub text: String,
}

/// Det spillet forteller verten, som i nettleseren er JavaScript-laget.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Hva du sikter på, eller `None`.
    Prompt(Option<Prompt>),
    /// Åpne panelet for en komponent.
    Inspect { component: String, variant: String },
    Theme(Theme),
    Variant { component: String, variant: String },
    /// En kort melding, til `.fs-toast`.
    Toast(String),
    Weather(String),
    /// Liv, utholdenhet, tokens og om du er nede i kjelleren, når noe av
    /// det endrer seg.
    Hud { hp: u32, stamina: u32, tokens: u32, cellar: bool, keys: u32, boss: Option<(String, u32)> },
    /// En lyd å spille: swing, swing_heavy, hit, hit_heavy, kill, block,
    /// parry, hurt, dodge, growl, pickup, death.
    Sound(String),
    /// Merkene på skjermen dette bildet (tom liste når de er borte).
    Marks(Vec<Mark>),
    /// Snakkeboblene over innbyggerne dette bildet (tom liste når de er borte).
    Speech(Vec<Bubble>),
    /// Lageret og utstyret, når noe endrer seg (og en gang ved start).
    Inventory(InventoryView),
    /// E på en innbygger med butikk: varene.
    Shop { npc: String, wares: Vec<WareView> },
    /// E på Skjemakontoret: oppskriftene som skjema.
    Workshop { recipes: Vec<RecipeView> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct InventoryView {
    /// Trygt i byen, og det du bærer på turen (mistes om du dør).
    pub stash: Vec<(String, u32)>,
    pub carried: Vec<(String, u32)>,
    pub weapon: String,
    pub weapon_special: String,
    pub shield: String,
    pub armor: String,
    pub lantern: bool,
    pub max_hp: u32,
    pub max_stamina: u32,
    pub tokens: u32,
    /// Komponentene du har lest om i boka denne turen.
    pub read: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WareView {
    pub id: String,
    pub name: String,
    pub price: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecipeView {
    pub id: String,
    pub name: String,
    pub rarity: String,
    pub result: String,
    /// (komponent, trengs, har).
    pub needs: Vec<(String, u32, u32)>,
    /// (komponent, har).
    pub requires: Vec<(String, bool)>,
    pub ok: bool,
}

/// Det som lagres i nettleseren mellom øktene.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SaveData {
    pub stash: Vec<(String, u32)>,
    pub weapon: String,
    pub shield: String,
    pub armor: Option<String>,
    pub lantern: f32,
    pub max_hp: f32,
    pub max_stamina: f32,
    pub coins: u32,
}

/// En snakkeboble: over hvem (id), hvor på skjermen (0..1), navn og tekst.
#[derive(Clone, Debug, PartialEq)]
pub struct Bubble {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    pub component: String,
    pub class: String,
    pub variant: String,
    /// Om R gjør noe.
    pub has_variants: bool,
    /// Om F gjør noe.
    pub usable: bool,
    /// Navnet i siktet når det ikke er en komponent (tom ellers).
    pub label: String,
    /// Om E åpner panelet.
    pub inspectable: bool,
    /// Hva F gjør.
    pub verb: String,
}

/// Radianer per piksel musa flytter seg.
const SENSITIVITY: f32 = 0.0035;
const EYE: f32 = 0.8;
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

/// Reisen når du følger en lenke: løftes rett opp over hustakene, flys
/// bort, og dales ned i fallskjerm dit du skulle. Posisjonene er føttene.
#[derive(Clone, Copy, Debug)]
struct Flight {
    from: Vec3,
    to: Vec3,
    yaw_to: f32,
    /// Hvor høyt turen går.
    altitude: f32,
    /// Sekunder brukt, og hvor lenge hver fase varer.
    t: f32,
    lift: f32,
    cruise: f32,
    drop: f32,
}

impl Flight {
    const SPEED: f32 = 30.0;
    const SINK: f32 = 7.5;
    const ABOVE: f32 = 26.0;

    fn new(from: Vec3, to: Vec3, yaw_to: f32) -> Self {
        let altitude = from.y.max(to.y) + Self::ABOVE;
        let dist = (Vec2::new(to.x, to.z) - Vec2::new(from.x, from.z)).length();
        Self { from, to, yaw_to, altitude, t: 0.0, lift: 1.6, cruise: (dist / Self::SPEED).max(1.2), drop: (altitude - to.y) / Self::SINK }
    }

    fn total(&self) -> f32 {
        self.lift + self.cruise + self.drop
    }

    /// Retningen turen går, som yaw.
    fn heading(&self) -> f32 {
        let d = self.to - self.from;
        (-d.x).atan2(-d.z)
    }

    /// Føttene nå, og hvor langt ut i dalingen vi er (0 før fallskjermen).
    fn feet(&self) -> (Vec3, f32) {
        let smooth = |u: f32| {
            let u = u.clamp(0.0, 1.0);
            u * u * (3.0 - 2.0 * u)
        };
        if self.t < self.lift {
            let u = smooth(self.t / self.lift);
            return (Vec3::new(self.from.x, self.from.y + (self.altitude - self.from.y) * u, self.from.z), 0.0);
        }
        if self.t < self.lift + self.cruise {
            let u = smooth((self.t - self.lift) / self.cruise);
            let p = self.from.lerp(self.to, u);
            return (Vec3::new(p.x, self.altitude, p.z), 0.0);
        }
        let v = ((self.t - self.lift - self.cruise) / self.drop).clamp(0.0, 1.0);
        // Dalingen: jevn fart, med litt sving i vinden.
        let sway = (1.0 - v) * 0.6;
        let p = Vec3::new(self.to.x + (self.t * 1.1).sin() * sway, self.to.y + (self.altitude - self.to.y) * (1.0 - v), self.to.z + (self.t * 0.8).cos() * sway);
        (p, v)
    }
}

struct PropState {
    /// Trykket inn (1 rett etter F), glir tilbake på et kvart sekund.
    press: f32,
    /// En bevegelig del, ikke komponenten: kan ikke siktes på.
    part: bool,
    rest: Affine3A,
    offset: Vec3,
    owner: Option<usize>,
    collider: Option<ColliderId>,
    pose: Affine3A,
    extra: [f32; 4],
}

pub struct Game {
    tokens: Tokens,
    lights: Vec<Light>,
    /// Lys som hører til lyktene (styres av bryteren), mot dørlamper osv.
    props: Vec<PropState>,
    by_collider: HashMap<ColliderId, usize>,
    installations: Vec<Box<dyn Installation>>,
    env: Env,
    pos: Vec3,
    vel_y: f32,
    grounded: bool,
    pub yaw: f32,
    pub pitch: f32,
    time: f32,
    theme: Theme,
    /// 0 er dag (lyst tema), 1 er natt (mørkt tema).
    night: f32,
    weather: Weather,
    fog_mul: f32,
    lightning_timer: f32,
    aim: Option<(usize, usize)>,
    highlight: f32,
    pub paused: bool,
    bob: f32,
    jumped: bool,
    stance: Stance,
    crawl: bool,
    /// Øyehøyden over kapselens sentrum, glattet mellom stillingene.
    eye_h: f32,
    /// Der du starter, og dit H tar deg: føttene og retningen.
    spawn: Vec3,
    spawn_yaw: f32,
    flight: Option<Flight>,
    /// Fallskjermen, en prop uten eier som spillet selv poserer.
    parachute: Option<usize>,
    /// Installasjonen som har tastaturfokus (et skjemafelt du skriver i).
    editing: Option<usize>,
    /// Dyrene på engene.
    herd: crate::domain::herd::Herd,
    /// Innbyggerne rundt torget.
    npcs: crate::domain::npcs::Npcs,
    bubbles_shown: bool,
    /// Kjelleren og fiendene i den.
    dungeon: crate::domain::city::dungeon::Dungeon,
    mobs: crate::domain::mobs::Mobs,
    /// Liv og tokens (mynter) du bærer.
    hp: f32,
    max_hp: f32,
    coins: u32,
    /// Komponentene: trygt lagret i byen, og det du bærer på turen.
    inventory: BTreeMap<String, u32>,
    carried: BTreeMap<String, u32>,
    gear: Gear,
    /// Komponentene du har lest om i boka denne turen (liv tilbake én gang per).
    read_docs: HashSet<String>,
    inv_dirty: bool,
    loot_seed: u64,
    /// Hendelser fra kall utenfra (kjøp, smiing, lesing), til neste `update`.
    pending: Vec<Event>,
    /// Nærkampen: armene, utholdenheten, huggene og treffeffektene.
    combat: crate::domain::combat::Combat,
    /// Skadetall som stiger: (sted, skade, alder), med løpenummer.
    popups: Vec<(Vec3, f32, f32, u32)>,
    popup_seq: u32,
    marks_shown: bool,
    /// Bildeforholdet fra sist `frame`, til å regne skjermposisjoner.
    aspect: std::cell::Cell<f32>,
    /// Fienden du sikter på nå.
    aimed_mob: Option<usize>,
    /// Hvor mye du er i kjelleren (0 oppe, 1 nede), glattet, til lyset.
    cellar_t: f32,
    last_hud: Option<(u32, u32, u32, bool, u32, Option<(&'static str, u32)>)>,
    /// Rammeverket er beseiret (denne turen).
    pub won: bool,
    pub debug: u32,
}

impl Game {
    /// `rigs` er skjelettene til de skinnede modellene (navn, rigg), fra
    /// modellene appen lastet.
    pub fn new(tokens: Tokens, font: crate::domain::text::Font, city: &City, mut installations: Vec<Box<dyn Installation>>, physics: &mut dyn Physics, rigs: Vec<(String, crate::domain::anim::Rig)>) -> Self {
        for c in &city.colliders {
            physics.add(c);
        }
        let mut props = vec![];
        let mut by_collider = HashMap::new();
        for (i, p) in city.props.iter().enumerate() {
            let collider = p.collider.map(|(offset, half)| {
                let id = physics.add_movable(half, p.rest * Affine3A::from_translation(offset));
                by_collider.insert(id, i);
                id
            });
            props.push(PropState { press: 0.0, part: p.part, rest: p.rest, offset: p.collider.map(|c| c.0).unwrap_or(Vec3::ZERO), owner: p.owner, collider, pose: p.rest, extra: [0.0; 4] });
        }
        for inst in &mut installations {
            inst.set_variant(inst.variant());
        }
        let env = Env { font: Some(std::sync::Arc::new(font)), ..Env::default() };
        let rig_of = |n: &str| rigs.iter().find(|(m, _)| m == n).map(|(_, r)| r.clone());
        let arms_rig = rig_of(crate::domain::combat::ARMS_MODEL);
        Self {
            tokens,
            lights: city.lights.clone(),
            props,
            by_collider,
            installations,
            env,
            pos: crate::domain::city::terrain::at(city.spawn) + Vec3::Y * (RADIUS + HALF + 0.05),
            vel_y: 0.0,
            grounded: false,
            yaw: city.spawn_yaw,
            pitch: 0.05,
            time: 0.0,
            theme: Theme::Light,
            night: 0.0,
            weather: Weather { wind: 0.2, ..Default::default() },
            fog_mul: 1.0,
            lightning_timer: 0.0,
            aim: None,
            highlight: 0.0,
            paused: false,
            bob: 0.0,
            jumped: false,
            stance: Stance::Stand,
            crawl: false,
            eye_h: EYE,
            spawn: city.spawn,
            spawn_yaw: city.spawn_yaw,
            flight: None,
            parachute: city.props.iter().position(|p| p.name == "parachute"),
            editing: None,
            herd: crate::domain::herd::Herd::new(),
            npcs: crate::domain::npcs::Npcs::new(rig_of(crate::domain::npcs::NPC_MODEL), city.npcs.iter().map(|n| (n.name, n.pos, n.yaw, n.tint, n.lines)).collect()),
            bubbles_shown: false,
            mobs: crate::domain::mobs::Mobs::new(&city.dungeon, &rigs),
            dungeon: city.dungeon.clone(),
            hp: 100.0,
            max_hp: 100.0,
            coins: 0,
            inventory: BTreeMap::new(),
            carried: BTreeMap::new(),
            gear: Gear { weapon: craft::RUSTY_SWORD, shield: craft::OLD_SHIELD, armor: None, lantern: 1.0 },
            read_docs: HashSet::new(),
            inv_dirty: true,
            loot_seed: 0x2545_f491_4f6c_dd1d,
            pending: vec![],
            combat: crate::domain::combat::Combat::new(arms_rig),
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

    /// Om et felt har tastaturfokus, så tastene skal skrives, ikke styre.
    pub fn editing(&self) -> bool {
        self.editing.is_some()
    }

    /// Props med ny geometri siden sist, til rendereren.
    pub fn take_dirty(&mut self) -> Vec<(usize, crate::domain::geometry::Mesh)> {
        std::mem::take(&mut self.env.dirty)
    }

    /// Etter en handling: tok eller slapp installasjonen fokuset?
    fn sync_edit(&mut self, owner: usize) {
        if let Some(e) = self.env.edit.take() {
            self.editing = if e { Some(owner) } else { None };
        }
    }

    /// Om du er på reise (løftet, i lufta eller i fallskjermen).
    pub fn flying(&self) -> bool {
        self.flight.is_some()
    }

    /// Start en flytur dit (føttene, y over terrenget). Til testene.
    pub fn fly_to(&mut self, feet: Vec3, yaw: f32) {
        self.env.teleport = Some((feet, yaw));
    }

    /// Snu kameraet uten å flytte det.
    pub fn set_look(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
    }

    /// Tilbake til torget, som H og knappen i pausemenyen.
    pub fn respawn(&mut self) {
        self.env.teleport = Some((self.spawn, self.spawn_yaw));
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn fristil_version(&self) -> &str {
        &self.tokens.version
    }

    pub fn weather(&self) -> WeatherKind {
        self.env.weather
    }

    pub fn set_theme(&mut self, theme: Theme, instant: bool) {
        self.theme = theme;
        if instant {
            self.night = if theme == Theme::Dark { 1.0 } else { 0.0 };
        }
    }

    pub fn set_weather(&mut self, kind: WeatherKind, instant: bool) {
        self.env.weather = kind;
        if instant {
            let (target, fog) = Self::weather_target(kind);
            self.weather = target;
            self.fog_mul = fog;
        }
    }

    pub fn set_online(&mut self, online: bool) {
        self.env.online = online;
    }

    /// Sett variant på en komponent, for eksempel fra panelet i nettleseren.
    /// Bruk en komponent (som F), uten å sikte på den. Til testene.
    /// `navn` eller `navn#n` for den n-te installasjonen med det navnet
    /// (sendeknappen er `button#1`).
    pub fn use_component(&mut self, component: &str) {
        let (name, nth) = match component.split_once('#') {
            Some((n, k)) => (n, k.parse().unwrap_or(0)),
            None => (component, 0),
        };
        let mut seen = 0;
        for (i, inst) in self.installations.iter_mut().enumerate() {
            if inst.component().name == name {
                if seen == nth {
                    if let Some(e) = self.editing.filter(|e| *e != i) {
                        self.installations[e].act(InstallAction::Blur, &mut self.env);
                        self.editing = None;
                        self.env.edit = None;
                        return self.use_component(component);
                    }
                    inst.act(InstallAction::Use, &mut self.env);
                    self.sync_edit(i);
                    break;
                }
                seen += 1;
            }
        }
    }

    /// Skriv i feltet som har fokus. Til testene.
    pub fn type_text(&mut self, text: &str) {
        if let Some(o) = self.editing {
            for c in text.chars() {
                self.installations[o].act(InstallAction::Type(c), &mut self.env);
            }
            self.sync_edit(o);
        }
    }

    pub fn set_variant(&mut self, component: &str, name: &str) {
        for inst in &mut self.installations {
            if inst.component().name == component {
                if let Some(i) = inst.variants().iter().position(|v| *v == name) {
                    inst.set_variant(i);
                }
            }
        }
    }

    pub fn variant_of(&self, component: &str) -> String {
        self.installations
            .iter()
            .find(|i| i.component().name == component)
            .map(|i| i.variants().get(i.variant()).copied().unwrap_or("").to_string())
            .unwrap_or_default()
    }

    /// Plasser kameraet, til skjermbilder og tester. `eye` er øyet.
    pub fn place(&mut self, eye: Vec3, yaw: f32, pitch: f32) {
        self.pos = eye - Vec3::Y * EYE;
        self.yaw = yaw;
        self.pitch = pitch;
        self.jumped = true;
    }

    pub fn set_gate(&mut self, open: bool, instant: bool) {
        self.env.gate_open = open;
        if instant {
            self.env.gate_t = if open { 1.0 } else { 0.0 };
        }
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), -self.yaw.cos() * self.pitch.cos())
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::Y * (self.eye_h + (self.bob * 9.0).sin() * 0.025)
    }

    fn prompt_for(&self, inst: usize) -> Prompt {
        let i = &self.installations[inst];
        let c = i.component();
        Prompt {
            component: c.name.into(),
            class: c.class.into(),
            variant: i.variants().get(i.variant()).copied().unwrap_or("").into(),
            has_variants: !i.variants().is_empty(),
            usable: i.usable(),
            label: i.label().into(),
            inspectable: i.inspectable(),
            verb: i.verb().into(),
        }
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

        // Gåing. Skriver du i et felt, går tastene dit i stedet.
        let mut wish = Vec3::ZERO;
        if !self.paused && self.editing.is_none() {
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
        let horizontal = if self.flight.is_some() { Vec3::ZERO } else { wish.normalize_or_zero() * speed * self.combat.move_factor() + self.combat.dodge_velocity() };
        if self.grounded {
            self.vel_y = -1.0;
            if input.jump && !self.paused && self.editing.is_none() && self.stance == Stance::Stand {
                self.vel_y = 5.4;
            }
        } else {
            self.vel_y -= 18.0 * dt;
        }
        if let Some(mut f) = self.flight {
            // På reise: ingen fysikk, bare turen. Kameraet snus mot reisemålet
            // under løftet og mot landingsretningen i starten av dalingen.
            // Mellomrom hopper over resten av turen.
            f.t += dt;
            if input.jump && !self.paused {
                f.t = f.total();
                input.jump = false;
            }
            let (feet, v) = f.feet();
            self.pos = feet + Vec3::Y * (half + radius);
            self.vel_y = 0.0;
            self.grounded = false;
            let target = if f.t < f.lift { Some(f.heading()) } else if v > 0.0 && v < 0.35 { Some(f.yaw_to) } else { None };
            if let Some(goal) = target {
                let diff = (goal - self.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                self.yaw += diff * (dt * 2.5).min(1.0);
            }
            if f.t >= f.total() {
                self.pos = f.to + Vec3::Y * (half + radius + 0.1);
                self.yaw = f.yaw_to;
                self.flight = None;
            } else {
                self.flight = Some(f);
            }
        } else {
            let desired = (horizontal + Vec3::Y * self.vel_y) * dt;
            let (moved, grounded) = physics.move_capsule(self.pos, half, radius, desired, dt);
            self.pos += moved;
            self.grounded = grounded;
        }
        // Under nederste etasje i kjelleren er det ingenting.
        let abyss = self.dungeon.floors.last().map(|f| f.y).unwrap_or(crate::domain::city::dungeon::DEPTH) - 12.0;
        if self.pos.y < abyss {
            // Falt gjennom verden: rett tilbake til torget.
            let (half, radius) = self.stance.capsule();
            self.pos = crate::domain::city::terrain::at(self.spawn) + Vec3::Y * (half + radius + 0.2);
            self.yaw = self.spawn_yaw;
            self.flight = None;
        }
        if self.grounded && horizontal.length_squared() > 0.0 {
            self.bob += dt * speed / 4.2;
        }

        // Hva sikter vi på?
        let eye = self.eye();
        let dir = self.forward();
        let hit = physics.cast_ray(eye, dir, 3.5);
        let in_cellar = self.pos.y < crate::domain::city::dungeon::DEPTH + 6.0;
        // Fiendene i kjelleren går foran propsene i siktet.
        let aimed_mob = if in_cellar { self.mobs.aimed(eye, dir, 2.6) } else { None };
        let aim = if aimed_mob.is_some() { None } else { hit.and_then(|h| self.by_collider.get(&h.collider).copied()).filter(|p| !self.props[*p].part).and_then(|p| self.props[p].owner.map(|o| (p, o))) };
        // Installasjonene får vite hva som siktes på før de får handlingene.
        self.env.aimed_prop = aim.map(|(p, _)| p);
        let mob_changed = aimed_mob.is_some() != self.aimed_mob.is_some();
        self.aimed_mob = aimed_mob;
        if aim.map(|a| a.1) != self.aim.map(|a| a.1) || mob_changed {
            self.aim = aim;
            events.push(Event::Prompt(if let Some(i) = aimed_mob { Some(Self::mob_prompt(self.mobs.mobs[i].kind.label())) } else { aim.map(|(_, o)| self.prompt_for(o)) }));
        } else {
            self.aim = aim;
        }
        let target = if self.aim.is_some() { 1.0 } else { 0.0 };
        self.highlight += (target - self.highlight) * (dt * 10.0).min(1.0);
        self.combat.set_block(input.block && in_cellar && !self.paused);

        for a in actions {
            match a {
                Action::Attack if in_cellar => self.combat.press_attack(),
                Action::AttackRelease => self.combat.release_attack(),
                Action::Dodge => {
                    if in_cellar && self.grounded && wish.length_squared() > 0.0 && !self.paused && self.combat.dodge(wish) {
                        events.push(Event::Sound("dodge".into()));
                    }
                }
                Action::Theme => {
                    self.theme = if self.theme == Theme::Light { Theme::Dark } else { Theme::Light };
                    events.push(Event::Theme(self.theme));
                }
                Action::Weather => {
                    let all = WeatherKind::ALL;
                    let i = all.iter().position(|w| *w == self.env.weather).unwrap_or(0);
                    self.env.weather = all[(i + 1) % all.len()];
                    events.push(Event::Weather(self.env.weather.name().into()));
                    events.push(Event::Toast(format!("Været: {}", self.env.weather.name())));
                }
                Action::Variant => {
                    if let Some((_, o)) = self.aim {
                        let inst = &mut self.installations[o];
                        if !inst.variants().is_empty() {
                            let n = inst.variants().len();
                            inst.set_variant((inst.variant() + 1) % n);
                            let p = self.prompt_for(o);
                            events.push(Event::Variant { component: p.component.clone(), variant: p.variant.clone() });
                            events.push(Event::Prompt(Some(p)));
                        }
                    }
                }
                Action::Use if in_cellar && self.aim.is_none() => {
                    // F uten noe å bruke: et hugg.
                    self.combat.press_attack();
                    self.combat.release_attack();
                }
                Action::Use | Action::Attack => {
                    // F et annet sted tar fokuset fra feltet du skrev i.
                    if let Some(e) = self.editing {
                        if self.aim.map(|a| a.1) != Some(e) {
                            self.installations[e].act(InstallAction::Blur, &mut self.env);
                            self.editing = None;
                            self.env.edit = None;
                        }
                    }
                    if let Some((p, o)) = self.aim {
                        if self.installations[o].usable() {
                            self.props[p].press = 1.0;
                        }
                        self.installations[o].act(InstallAction::Use, &mut self.env);
                        self.sync_edit(o);
                    }
                }
                Action::Text(c) => {
                    if let Some(o) = self.editing {
                        // F mens du sikter på en annen komponent: slipp feltet og
                        // bruk den du sikter på, i stedet for å skrive «f».
                        if matches!(c, 'f' | 'F') && self.aim.map(|a| a.1).is_some_and(|a| a != o) {
                            self.installations[o].act(InstallAction::Blur, &mut self.env);
                            self.editing = None;
                            self.env.edit = None;
                            if let Some((p, o2)) = self.aim {
                                if self.installations[o2].usable() {
                                    self.props[p].press = 1.0;
                                }
                                self.installations[o2].act(InstallAction::Use, &mut self.env);
                                self.sync_edit(o2);
                            }
                            continue;
                        }
                        self.installations[o].act(InstallAction::Type(c), &mut self.env);
                        self.sync_edit(o);
                    }
                }
                Action::Crawl => {
                    self.crawl = !self.crawl;
                }
                Action::Home => {
                    self.env.teleport = Some((self.spawn, self.spawn_yaw));
                    events.push(Event::Toast("Tilbake til torget".into()));
                }
                Action::Inspect => {
                    if let Some((_, o)) = self.aim {
                        let p = self.prompt_for(o);
                        if p.component == "npc" {
                            // Butikk eller verksted, etter hvem du sikter på.
                            if let Some(i) = self.env.aimed_npc {
                                events.push(self.npc_panel(i));
                            }
                        } else {
                            events.push(Event::Inspect { component: p.component, variant: p.variant });
                        }
                    }
                }
            }
        }

        // Dag og natt glir over i hverandre på et par sekunder.
        let goal = if self.theme == Theme::Dark { 1.0 } else { 0.0 };
        let step = dt / 2.8;
        self.night = if self.night < goal { (self.night + step).min(goal) } else { (self.night - step).max(goal) };

        // Installasjonene.
        self.env.time = self.time;
        self.env.dt = dt;
        self.env.night = self.night;
        for inst in &mut self.installations {
            inst.tick(&mut self.env);
        }
        // Lenker, brødsmuler og H: en flytur dit (målets y er høyden over
        // terrenget). Er du alt på reise, byttes målet.
        if let Some((feet, yaw)) = self.env.teleport.take() {
            let (half, radius) = self.stance.capsule();
            let ground = crate::domain::city::terrain::height(feet.x, feet.z);
            let to = Vec3::new(feet.x, ground + feet.y, feet.z);
            let from = match self.flight {
                Some(f) => f.feet().0,
                None => self.pos - Vec3::Y * (half + radius),
            };
            self.flight = Some(Flight::new(from, to, yaw));
            self.vel_y = 0.0;
            self.env.toasts.push("Mellomrom hopper over flyturen".into());
        }
        // Kister: en komponent fra etasjens liste.
        if let Some(c) = self.env.open_chest.take() {
            let floor = self.dungeon.chests.get(c).map(|c| c.2).unwrap_or(0);
            let list = craft::LOOT[floor.min(craft::LOOT.len() - 1)];
            self.loot_seed ^= self.loot_seed << 13;
            self.loot_seed ^= self.loot_seed >> 7;
            self.loot_seed ^= self.loot_seed << 17;
            let pick = list[(self.loot_seed >> 33) as usize % list.len()];
            *self.carried.entry(pick.to_string()).or_insert(0) += 1;
            self.inv_dirty = true;
            events.push(Event::Sound("pickup".into()));
            events.push(Event::Toast(format!("I kista: {pick}. Bær den opp, så er den din.")));
        }
        for t in self.env.toasts.drain(..) {
            events.push(Event::Toast(t));
        }
        if let Some(prev) = events.iter().rev().find_map(|e| if let Event::Weather(w) = e { Some(w.clone()) } else { None }) {
            let _ = prev;
        }

        // Ned i kjelleren og opp igjen: rett dit, uten flytur. Opp i dagen
        // er det du bærer trygt.
        if let Some((feet, yaw)) = self.env.drop.take() {
            let (half, radius) = self.stance.capsule();
            self.pos = feet + Vec3::Y * (half + radius + 0.05);
            self.yaw = yaw;
            self.flight = None;
            self.vel_y = 0.0;
            self.jumped = true;
            if feet.y > crate::domain::city::dungeon::DEPTH + 6.0 {
                self.bank(&mut events);
            }
        }

        self.update_weather(dt);
        self.update_props(physics, dt);
        let (half, radius) = self.stance.capsule();
        let feet = self.pos - Vec3::Y * (half + radius);
        self.herd.update(dt, feet);
        // Innbyggerne: F på en av dem gir neste replikk; boblene som står
        // sendes hvert bilde (og én tom når de er borte).
        if let Some(i) = self.env.talk_to.take() {
            if self.npcs.talk(i).is_some() {
                events.push(Event::Sound("talk".into()));
            }
        }
        let speeches = self.npcs.update(dt, feet);
        if !speeches.is_empty() || self.bubbles_shown {
            self.bubbles_shown = !speeches.is_empty();
            let bubbles = speeches
                .iter()
                .filter_map(|s| self.project(s.pos).map(|(x, y)| Bubble { id: s.npc as u32, x, y, name: s.name.into(), text: s.text.clone() }))
                .collect();
            events.push(Event::Speech(bubbles));
        }
        // Fiendene: de biter, og du plukker tokens. Under hitstop står
        // alt stille et øyeblikk.
        let in_cellar = self.pos.y < crate::domain::city::dungeon::DEPTH + 6.0;
        self.cellar_t += ((if in_cellar { 1.0 } else { 0.0 }) - self.cellar_t) * (dt * 4.0).min(1.0);
        let sim_dt = if self.combat.hitstop > 0.0 { 0.0 } else { dt };
        let eye = self.eye();
        let fwd = self.forward();
        let outcome = self.mobs.update(sim_dt, feet, in_cellar, &self.dungeon);
        self.coins += outcome.tokens;
        if outcome.tokens > 0 {
            events.push(Event::Sound("pickup".into()));
        }
        if outcome.keys > 0 {
            self.env.keys += outcome.keys;
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
                // Død: opp til brønnen, en firedel av tokenene blir igjen nede.
                let lost = self.coins / 4;
                self.coins -= lost;
                self.mobs.reset(&self.dungeon);
                self.combat.reset();
                self.popups.clear();
                self.env.keys = 0;
                self.hp = self.max_hp;
                if !self.carried.is_empty() {
                    let n: u32 = self.carried.values().sum();
                    self.carried.clear();
                    self.inv_dirty = true;
                    events.push(Event::Toast(format!("{n} komponenter ble igjen i kjelleren.")));
                }
                self.read_docs.clear();
                let up = crate::domain::city::terrain::at(crate::domain::city::dungeon::WELL + Vec3::new(1.9, 0.0, 0.0));
                self.pos = up + Vec3::Y * (half + radius + 0.05);
                self.yaw = std::f32::consts::FRAC_PI_2;
                self.flight = None;
                self.jumped = true;
                events.push(Event::Sound("death".into()));
                events.push(Event::Toast(format!("Du ble overkjørt av forretningslogikk. {lost} tokens ble igjen nede.")));
            }
        }
        // Våre hugg.
        for e in self.combat.update(dt, sim_dt, eye, fwd, feet, &mut self.mobs, &self.dungeon) {
            use crate::domain::combat::CombatEvent;
            match e {
                CombatEvent::Hit { mob, damage, killed, pos } => {
                    self.popup_seq = self.popup_seq.wrapping_add(1);
                    self.popups.push((pos, damage, 0.0, self.popup_seq));
                    if killed {
                        let (kind, elite) = self.mobs.mobs.get(mob).map(|m| (m.kind, m.elite)).unwrap_or((crate::domain::mobs::Kind::Button, false));
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
                        } else if kind != crate::domain::mobs::Kind::Tooltip {
                            events.push(Event::Toast(format!("{} falt fra hverandre. Plukk opp tokenene.", kind.label())));
                        }
                        if self.aimed_mob.is_some() {
                            self.aimed_mob = None;
                            events.push(Event::Prompt(None));
                        }
                    }
                }
                CombatEvent::Miss => {}
                CombatEvent::Sound(s) => events.push(Event::Sound(s.into())),
            }
        }
        // Merkene: helsestolper over fiender som er truffet, og skadetall.
        let marks = self.marks(dt);
        if !marks.is_empty() || self.marks_shown {
            self.marks_shown = !marks.is_empty();
            events.push(Event::Marks(marks));
        }
        let boss = if in_cellar { self.mobs.boss().map(|(n, f)| (n, (f * 100.0).ceil() as u32)) } else { None };
        let hud = (self.hp.round().max(0.0) as u32, self.combat.stamina.round().max(0.0) as u32, self.coins, in_cellar, self.env.keys, boss);
        if self.last_hud != Some(hud) {
            if self.last_hud.as_ref().map(|h| h.2) != Some(self.coins) {
                self.inv_dirty = true;
            }
            self.last_hud = Some(hud.clone());
            events.push(Event::Hud { hp: hud.0, stamina: hud.1, tokens: hud.2, cellar: hud.3, keys: hud.4, boss: hud.5.map(|(n, f)| (n.to_string(), f)) });
        }
        if self.inv_dirty {
            self.inv_dirty = false;
            events.push(Event::Inventory(self.inventory_view()));
        }
        events
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
    fn npc_panel(&self, i: usize) -> Event {
        let name = self.npcs.npcs.get(i).map(|n| n.name).unwrap_or("");
        let wares = |w: &[craft::Ware]| w.iter().map(|w| WareView { id: w.id.into(), name: w.name.into(), price: w.price }).collect();
        match name {
            "Knappesmeden" => Event::Shop { npc: name.into(), wares: wares(craft::SMITH_WARES) },
            "Apotekeren" => Event::Shop { npc: name.into(), wares: wares(craft::APOTHECARY_WARES) },
            "Gorvek" => Event::Shop { npc: name.into(), wares: wares(craft::GORVEK_WARES) },
            _ => Event::Workshop { recipes: self.recipe_views() },
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

    /// Smi etter en oppskrift (fra skjemaet i verkstedet). Gir om det gikk.
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

    /// Et punkt i verden til skjermen (0..1), om det er foran kameraet.
    fn project(&self, p: Vec3) -> Option<(f32, f32)> {
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
    pub fn roster(&self) -> Vec<(crate::domain::mobs::Kind, bool)> {
        self.mobs.mobs.iter().map(|m| (m.kind, m.elite)).collect()
    }

    /// Den nærmeste levende fienden (føttene), til testene.
    pub fn nearest_mob(&self, from: Vec3) -> Option<Vec3> {
        self.mobs.mobs.iter().filter(|m| m.alive()).map(|m| m.pos).min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }

    /// Siktet på en fiende.
    fn mob_prompt(label: &str) -> Prompt {
        Prompt { component: "mutant".into(), class: String::new(), variant: String::new(), has_variants: false, usable: true, label: label.into(), inspectable: false, verb: "Hugg".into() }
    }

    fn weather_target(kind: WeatherKind) -> (Weather, f32) {
        match kind {
            WeatherKind::Clear => (Weather { cloud: 0.12, wind: 0.2, ..Default::default() }, 1.0),
            WeatherKind::Overcast => (Weather { cloud: 0.85, wind: 0.45, ..Default::default() }, 1.4),
            WeatherKind::Rain => (Weather { cloud: 0.92, rain: 0.75, wet: 1.0, wind: 0.6, ..Default::default() }, 1.8),
            WeatherKind::Storm => (Weather { cloud: 1.0, rain: 1.0, wet: 1.0, wind: 1.3, ..Default::default() }, 2.2),
            WeatherKind::Snow => (Weather { cloud: 0.75, snow: 0.8, snow_cover: 1.0, wind: 0.3, ..Default::default() }, 1.6),
            WeatherKind::Fog => (Weather { cloud: 0.6, wet: 0.35, wind: 0.1, ..Default::default() }, 6.0),
        }
    }

    fn update_weather(&mut self, dt: f32) {
        let (t, fog) = Self::weather_target(self.env.weather);
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
        if self.env.weather == WeatherKind::Storm {
            self.lightning_timer -= dt;
            if self.lightning_timer <= 0.0 {
                w.lightning = 1.0;
                self.lightning_timer = 3.0 + ((self.time * 7.3).sin() * 0.5 + 0.5) * 9.0;
            }
        }
        w.lightning *= (1.0 - dt * 9.0).max(0.0);
    }

    fn update_props(&mut self, physics: &mut dyn Physics, dt: f32) {
        for (i, p) in self.props.iter_mut().enumerate() {
            let (mut pose, extra) = match p.owner {
                Some(o) => self.installations[o].pose(i, p.rest, &self.env),
                None => (p.rest, [0.0; 4]),
            };
            // Trykket: propen går 2 cm inn langs sin egen −Z (inn i plata) og
            // tilbake, som en knapp som trykkes.
            if p.press > 0.0 {
                p.press = (p.press - dt * 4.0).max(0.0);
                let push = (p.press * std::f32::consts::PI).sin().max(0.0) * 0.02;
                pose = pose * Affine3A::from_translation(Vec3::new(0.0, 0.0, -push));
            }
            p.pose = pose;
            p.extra = extra;
            if let Some(id) = p.collider {
                physics.set_pose(id, pose * Affine3A::from_translation(p.offset));
            }
        }
        if let Some((prop, _)) = self.aim {
            self.props[prop].extra[0] = self.highlight;
        }
        // Fallskjermen henger over hodet mens du daler, og folder seg ut de
        // første tidelene. Ellers er den gjemt.
        if let Some(i) = self.parachute {
            let pose = match self.flight.map(|f| (f, f.feet().1)) {
                Some((f, v)) if v > 0.0 => {
                    let open = ((f.t - f.lift - f.cruise) / 0.5).clamp(0.05, 1.0);
                    let open = open * open * (3.0 - 2.0 * open);
                    let sway = Quat::from_rotation_x((f.t * 0.8).sin() * 0.08) * Quat::from_rotation_z((f.t * 1.1).cos() * 0.08);
                    Affine3A::from_rotation_translation(sway, self.eye() + Vec3::Y * 0.9) * Affine3A::from_scale(Vec3::new(open, open.max(0.6), open))
                }
                _ => Affine3A::from_scale(Vec3::splat(0.0001)),
            };
            self.props[i].pose = pose;
        }
    }

    pub fn frame(&self, aspect: f32) -> Frame {
        self.aspect.set(aspect);
        let mut skins = vec![];
        // Kameraet sparkes og skjelver når vi treffes (bare det du ser,
        // ikke retningen du går).
        let (ky, kp) = self.combat.view_kick(self.time);
        let (vyaw, vpitch) = (self.yaw + ky, (self.pitch + kp).clamp(-1.5, 1.5));
        let view_dir = Vec3::new(-vyaw.sin() * vpitch.cos(), vpitch.sin(), -vyaw.cos() * vpitch.cos());
        let in_cellar = self.pos.y < crate::domain::city::dungeon::DEPTH + 6.0;
        let t = self.night;
        let dusk = (t * std::f32::consts::PI).sin();
        let lerp3 = |a: Vec3, b: Vec3| a.lerp(b, t);
        let w = self.weather;
        let overcast = w.cloud.clamp(0.0, 1.0);

        // Sola står lavt i sørvest om dagen. Mens temaet bytter går den ned,
        // og månen kommer opp i sørøst.
        let dir = |az: f32, el: f32| {
            let (az, el) = (az.to_radians(), el.to_radians());
            Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos())
        };
        let el = 27.0 * (1.0 - t) + 38.0 * t - dusk * 24.0;
        let az = -68.0 * (1.0 - t) + 42.0 * t;
        let sun_dir = dir(az, el.max(3.0));

        let day_sun = Vec3::new(1.0, 0.92, 0.80);
        let dusk_sun = Vec3::new(1.0, 0.48, 0.22);
        let moon = Vec3::new(0.55, 0.66, 1.0);
        let sun_color = lerp3(day_sun, moon).lerp(dusk_sun, dusk * 0.85);
        let sun_intensity = (3.0 * (1.0 - t) + 0.38 * t) * (1.0 - overcast * 0.8);

        let grey = |c: Vec3| c.lerp(Vec3::splat(c.dot(Vec3::new(0.3, 0.59, 0.11))), overcast * 0.6) * (1.0 - overcast * 0.25);
        let sky_top = grey(lerp3(Vec3::new(0.16, 0.34, 0.72), Vec3::new(0.008, 0.014, 0.04)).lerp(Vec3::new(0.18, 0.2, 0.42), dusk * 0.6));
        let sky_horizon = grey(lerp3(Vec3::new(0.62, 0.74, 0.9), Vec3::new(0.016, 0.028, 0.06)).lerp(Vec3::new(1.0, 0.5, 0.28), dusk * 0.7));
        let ground = lerp3(Vec3::new(0.28, 0.26, 0.22), Vec3::new(0.012, 0.013, 0.018)) * (1.0 - overcast * 0.3);
        let snow_bounce = Vec3::new(0.5, 0.52, 0.58) * w.snow_cover * (1.0 - t * 0.95);

        let lamp = ((t - 0.35) / 0.5).clamp(0.0, 1.0).max(overcast * 0.5 * (1.0 - t)) * 0.6 + ((t - 0.35) / 0.5).clamp(0.0, 1.0) * 0.4;
        let street = self.installations.iter().map(|i| i.lights_on(&self.env)).fold(1.0, f32::min);
        let eye_now = self.eye();
        let mut lights: Vec<Light> = self
            .lights
            .iter()
            .enumerate()
            .map(|(i, l)| {
                // Flakket: ild blafrer mykt, lysrør slukker i rykk.
                let mut k = 1.0;
                if l.flicker > 0.0 {
                    let t = self.time * 9.0 + i as f32 * 1.7;
                    let soft = 1.0 - l.flicker * 0.5 * (0.5 + 0.5 * (t.sin() * (t * 0.37).cos() + (t * 2.3).sin() * 0.5));
                    let blackout = if l.flicker > 0.4 && ((t * 0.31).sin() * (t * 0.173 + i as f32).cos()) > 0.72 { 0.1 } else { 1.0 };
                    k = soft * blackout;
                }
                Light { intensity: l.intensity * k * if l.always { 1.0 } else { lamp * street }, ..*l }
            })
            .filter(|l| l.intensity > 0.01)
            .collect();
        // Nede i kjelleren bærer du et lys, så du ser våpenet og det som
        // kommer mot deg.
        if in_cellar {
            lights.push(Light { pos: eye_now + view_dir * 0.4 + Vec3::Y * 0.2, radius: 6.0 * self.gear.lantern.sqrt(), color: Vec3::new(1.0, 0.8, 0.6), intensity: 0.8 * self.gear.lantern, always: true, flicker: 0.0 });
        }
        // Rendereren tar de første 48: de nærmeste først.
        lights.sort_by(|a, b| a.pos.distance_squared(eye_now).total_cmp(&b.pos.distance_squared(eye_now)));
        // Nede i kjelleren: lite himmellys, mer tåke, øynene vant til mørket.
        let c = self.cellar_t;

        let mut pal = palette::build(&self.tokens, t);
        for inst in &self.installations {
            inst.palette(&self.tokens, t, &mut pal);
        }
        let props = self.props.iter().map(|p| (p.pose, p.extra)).collect();

        let eye = self.eye();
        Frame {
            view: Mat4::look_to_rh(eye, view_dir, Vec3::Y),
            proj: Mat4::perspective_rh(72f32.to_radians(), aspect, 0.1, 2500.0),
            camera: eye,
            time: self.time,
            sun_dir,
            sun_color,
            sun_intensity: sun_intensity * (1.0 - c * 0.85),
            night: t,
            sky_top: sky_top * (1.0 - c * 0.9),
            sky_horizon: sky_horizon * (1.0 - c * 0.9),
            ground_bounce: (ground + snow_bounce) * (1.0 - c * 0.8),
            fog_color: grey(lerp3(Vec3::new(0.6, 0.7, 0.82), Vec3::new(0.012, 0.02, 0.04))) * (1.0 - c * 0.95),
            fog_density: (0.0013 + dusk * 0.0012) * self.fog_mul * (1.0 + c * 6.0),
            fog_falloff: 0.03,
            exposure: (0.5 * (1.0 - t) + 2.0 * t) * (1.0 + overcast * 0.35) * (1.0 + c * 1.5),
            bloom: 0.06 + t * 0.06,
            windows_lit: 0.55,
            street_lights: street,
            aurora: 1.0 - overcast,
            weather: w,
            palette: pal,
            lights,
            props,
            instances: {
                let mut v = self.herd.instances(self.time);
                v.extend(self.npcs.instances(&mut skins));
                v.extend(self.mobs.instances(self.time, &mut skins));
                v.extend(self.combat.instances(eye, vyaw, vpitch, in_cellar, &mut skins));
                v
            },
            skins,
            hurt: self.combat.hurt,
            taa: !self.jumped,
            debug: self.debug,
        }
    }

    /// Etter at `frame` er hentet: neste bilde kan blandes igjen.
    pub fn settle(&mut self) {
        self.jumped = false;
    }
}

pub use installations::WeatherKind as Kind;
