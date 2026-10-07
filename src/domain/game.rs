//! Spillet: spilleren, døgnet, været og samspillet mellom installasjonene.
//! Fysikken går gjennom `ports::Physics`, så kjernen vet ikke hvilken
//! motor som brukes.

use crate::domain::city::City;
use crate::domain::installations::{self, Action as InstallAction, Env, Installation, WeatherKind};
use crate::domain::palette;
use crate::domain::ports::{ColliderId, Physics};
use crate::domain::scene::{Frame, Light, Weather};
use crate::domain::tokens::{Theme, Tokens};
use glam::{Affine3A, Mat4, Vec3};
use std::collections::HashMap;

#[derive(Default)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
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
            "KeyE" if down => self.actions.push(Action::Inspect),
            "KeyR" if down => self.actions.push(Action::Variant),
            "KeyF" if down => self.actions.push(Action::Use),
            "KeyT" if down => self.actions.push(Action::Theme),
            "KeyV" if down => self.actions.push(Action::Weather),
            _ => {}
        }
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
    /// T: bytt tema, og dermed dag og natt.
    Theme,
    /// V: neste vær.
    Weather,
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
    pub debug: u32,
}

impl Game {
    pub fn new(tokens: Tokens, city: &City, mut installations: Vec<Box<dyn Installation>>, physics: &mut dyn Physics) -> Self {
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
        Self {
            tokens,
            lights: city.lights.clone(),
            props,
            by_collider,
            installations,
            env: Env::default(),
            pos: city.spawn + Vec3::Y * (RADIUS + HALF + 0.05),
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
            debug: 0,
        }
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
    pub fn use_component(&mut self, component: &str) {
        for inst in &mut self.installations {
            if inst.component().name == component {
                inst.act(InstallAction::Use, &mut self.env);
            }
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
        self.pos + Vec3::Y * (EYE + (self.bob * 9.0).sin() * 0.025)
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
        let speed = if input.sprint { 7.5 } else { 4.2 };
        let horizontal = wish.normalize_or_zero() * speed;
        if self.grounded {
            self.vel_y = -1.0;
            if input.jump && !self.paused {
                self.vel_y = 5.4;
            }
        } else {
            self.vel_y -= 18.0 * dt;
        }
        let desired = (horizontal + Vec3::Y * self.vel_y) * dt;
        let (moved, grounded) = physics.move_capsule(self.pos, HALF, RADIUS, desired, dt);
        self.pos += moved;
        self.grounded = grounded;
        if self.pos.y < -20.0 {
            self.pos = Vec3::new(0.0, 2.0, 11.0);
        }
        if self.grounded && horizontal.length_squared() > 0.0 {
            self.bob += dt * speed / 4.2;
        }

        // Hva sikter vi på?
        let eye = self.eye();
        let dir = self.forward();
        let hit = physics.cast_ray(eye, dir, 3.5);
        let aim = hit.and_then(|h| self.by_collider.get(&h.collider).copied()).and_then(|p| self.props[p].owner.map(|o| (p, o)));
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
                    if let Some((_, o)) = self.aim {
                        self.installations[o].act(InstallAction::Use, &mut self.env);
                    }
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
