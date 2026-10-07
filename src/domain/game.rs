//! Spillet: spilleren, døgnet, været og samspillet mellom installasjonene.
//! Fysikken går gjennom `ports::Physics`, så kjernen vet ikke hvilken
//! motor som brukes.

use crate::domain::city::City;
use crate::domain::installations::{self, Action as InstallAction, Env, Installation, WeatherKind};
use crate::domain::palette;
use crate::domain::ports::{ColliderId, Physics};
use crate::domain::scene::{Frame, Light, Weather};
use crate::domain::tokens::{Theme, Tokens};
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};
use std::collections::HashMap;

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
            "ShiftLeft" | "ShiftRight" => self.sprint = down,
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
    pub debug: u32,
}

impl Game {
    pub fn new(tokens: Tokens, font: crate::domain::text::Font, city: &City, mut installations: Vec<Box<dyn Installation>>, physics: &mut dyn Physics) -> Self {
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
            props.push(PropState { rest: p.rest, offset: p.collider.map(|c| c.0).unwrap_or(Vec3::ZERO), owner: p.owner, collider, pose: p.rest, extra: [0.0; 4] });
        }
        for inst in &mut installations {
            inst.set_variant(inst.variant());
        }
        let env = Env { font: Some(std::sync::Arc::new(font)), ..Env::default() };
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
        }
    }

    pub fn update(&mut self, input: &mut Input, dt: f32, physics: &mut dyn Physics) -> Vec<Event> {
        let dt = dt.clamp(0.0, 1.0 / 20.0);
        self.time += dt;
        let mut events = vec![];
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
        let horizontal = if self.flight.is_some() { Vec3::ZERO } else { wish.normalize_or_zero() * speed };
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
        if self.pos.y < -20.0 {
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
        let aim = hit.and_then(|h| self.by_collider.get(&h.collider).copied()).and_then(|p| self.props[p].owner.map(|o| (p, o)));
        // Installasjonene får vite hva som siktes på før de får handlingene.
        self.env.aimed_prop = aim.map(|(p, _)| p);
        if aim.map(|a| a.1) != self.aim.map(|a| a.1) {
            self.aim = aim;
            events.push(Event::Prompt(aim.map(|(_, o)| self.prompt_for(o))));
        } else {
            self.aim = aim;
        }
        let target = if self.aim.is_some() { 1.0 } else { 0.0 };
        self.highlight += (target - self.highlight) * (dt * 10.0).min(1.0);

        for a in actions {
            match a {
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
                Action::Use => {
                    // F et annet sted tar fokuset fra feltet du skrev i.
                    if let Some(e) = self.editing {
                        if self.aim.map(|a| a.1) != Some(e) {
                            self.installations[e].act(InstallAction::Blur, &mut self.env);
                            self.editing = None;
                            self.env.edit = None;
                        }
                    }
                    if let Some((_, o)) = self.aim {
                        self.installations[o].act(InstallAction::Use, &mut self.env);
                        self.sync_edit(o);
                    }
                }
                Action::Text(c) => {
                    if let Some(o) = self.editing {
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
                        events.push(Event::Inspect { component: p.component, variant: p.variant });
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
        for t in self.env.toasts.drain(..) {
            events.push(Event::Toast(t));
        }
        if let Some(prev) = events.iter().rev().find_map(|e| if let Event::Weather(w) = e { Some(w.clone()) } else { None }) {
            let _ = prev;
        }

        self.update_weather(dt);
        self.update_props(physics);
        events
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

    fn update_props(&mut self, physics: &mut dyn Physics) {
        for (i, p) in self.props.iter_mut().enumerate() {
            let (pose, extra) = match p.owner {
                Some(o) => self.installations[o].pose(i, p.rest, &self.env),
                None => (p.rest, [0.0; 4]),
            };
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
        let lights: Vec<Light> = self
            .lights
            .iter()
            .map(|l| Light { intensity: l.intensity * lamp * street, ..*l })
            .filter(|l| l.intensity > 0.01)
            .collect();

        let mut pal = palette::build(&self.tokens, t);
        for inst in &self.installations {
            inst.palette(&self.tokens, t, &mut pal);
        }
        let props = self.props.iter().map(|p| (p.pose, p.extra)).collect();

        let eye = self.eye();
        Frame {
            view: Mat4::look_to_rh(eye, self.forward(), Vec3::Y),
            proj: Mat4::perspective_rh(72f32.to_radians(), aspect, 0.1, 2500.0),
            camera: eye,
            time: self.time,
            sun_dir,
            sun_color,
            sun_intensity,
            night: t,
            sky_top,
            sky_horizon,
            ground_bounce: ground + snow_bounce,
            fog_color: grey(lerp3(Vec3::new(0.6, 0.7, 0.82), Vec3::new(0.012, 0.02, 0.04))),
            fog_density: (0.0013 + dusk * 0.0012) * self.fog_mul,
            fog_falloff: 0.03,
            exposure: (0.5 * (1.0 - t) + 2.0 * t) * (1.0 + overcast * 0.35),
            bloom: 0.06 + t * 0.06,
            windows_lit: 0.55,
            street_lights: street,
            aurora: 1.0 - overcast,
            weather: w,
            palette: pal,
            lights,
            props,
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
