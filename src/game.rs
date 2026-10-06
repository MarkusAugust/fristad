//! Spillogikken: spilleren, fysikken, døgnet og knappen.

use crate::render::Frame;
use crate::tokens::{Theme, Tokens};
use crate::world::{self, Shape, World};
use glam::{Affine3A, Mat4, Vec3};
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;

/// Knappevariantene i Fristil, i den rekkefølgen R blar gjennom dem.
pub const BUTTON_VARIANTS: [&str; 4] = ["primary", "secondary", "ghost", "danger"];

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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    pub component: String,
    pub class: String,
    pub variant: String,
}

const EYE: f32 = 0.8;
const RADIUS: f32 = 0.35;
const HALF: f32 = 0.55;

struct Gate {
    open: bool,
    /// 0 lukket, 1 åpen.
    t: f32,
    doors: [(usize, ColliderHandle); 2],
}

pub struct Game {
    tokens: Tokens,
    world_lights: Vec<(Vec3, f32, Vec3, f32)>,
    prop_rest: Vec<Affine3A>,
    prop_pose: Vec<Affine3A>,
    prop_offsets: Vec<(Vec3, Vec3)>,
    colliders: ColliderSet,
    bodies: RigidBodySet,
    query: QueryPipeline,
    controller: KinematicCharacterController,
    pos: Vec3,
    vel_y: f32,
    grounded: bool,
    pub yaw: f32,
    pub pitch: f32,
    time: f32,
    theme: Theme,
    /// 0 er dag (lyst tema), 1 er natt (mørkt tema).
    night: f32,
    variant: usize,
    button_prop: usize,
    button_collider: ColliderHandle,
    press: f32,
    gate: Gate,
    aim: Option<Prompt>,
    highlight: f32,
    pub paused: bool,
    bob: f32,
}

fn iso(center: Vec3, yaw: f32) -> Isometry<Real> {
    Isometry::new(vector![center.x, center.y, center.z], vector![0.0, yaw, 0.0])
}

fn to_iso(t: &Affine3A) -> Isometry<Real> {
    let (_, rot, tr) = t.to_scale_rotation_translation();
    Isometry::from_parts(
        Translation::new(tr.x, tr.y, tr.z),
        rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(rot.w, rot.x, rot.y, rot.z)),
    )
}

impl Game {
    pub fn new(tokens: Tokens, world: &World) -> Self {
        let mut colliders = ColliderSet::new();
        for c in &world.colliders {
            let col = match *c {
                Shape::Cuboid { center, half, yaw } => {
                    ColliderBuilder::cuboid(half.x, half.y, half.z).position(iso(center, yaw)).build()
                }
                Shape::Cylinder { center, radius, half_height } => {
                    ColliderBuilder::cylinder(half_height, radius).position(iso(center, 0.0)).build()
                }
            };
            colliders.insert(col);
        }

        let mut button_prop = 0;
        let mut button_collider = ColliderHandle::invalid();
        let mut doors = vec![];
        let mut prop_offsets = vec![];
        for (i, p) in world.props.iter().enumerate() {
            let (offset, half) = p.collider.unwrap_or((Vec3::ZERO, Vec3::splat(0.01)));
            prop_offsets.push((offset, half));
            let pose = p.rest * Affine3A::from_translation(offset);
            let h = colliders.insert(ColliderBuilder::cuboid(half.x, half.y, half.z).position(to_iso(&pose)).build());
            match p.name {
                "button" => {
                    button_prop = i;
                    button_collider = h;
                }
                "door_west" | "door_east" => doors.push((i, h)),
                _ => {}
            }
        }

        let mut query = QueryPipeline::new();
        query.update(&colliders);

        let controller = KinematicCharacterController {
            offset: CharacterLength::Absolute(0.02),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.5),
                min_width: CharacterLength::Absolute(0.15),
                include_dynamic_bodies: false,
            }),
            snap_to_ground: Some(CharacterLength::Absolute(0.3)),
            max_slope_climb_angle: 50f32.to_radians(),
            ..Default::default()
        };

        Self {
            tokens,
            world_lights: world.lights.iter().map(|l| (l.pos, l.radius, l.color, l.intensity)).collect(),
            prop_rest: world.props.iter().map(|p| p.rest).collect(),
            prop_pose: world.props.iter().map(|p| p.rest).collect(),
            prop_offsets,
            colliders,
            bodies: RigidBodySet::new(),
            query,
            controller,
            pos: world.spawn + Vec3::Y * (RADIUS + HALF + 0.05),
            vel_y: 0.0,
            grounded: false,
            yaw: world.spawn_yaw,
            pitch: 0.05,
            time: 0.0,
            theme: Theme::Light,
            night: 0.0,
            variant: 0,
            button_prop,
            button_collider,
            press: 0.0,
            gate: Gate { open: false, t: 0.0, doors: [doors[0], doors[1]] },
            aim: None,
            highlight: 0.0,
            paused: false,
            bob: 0.0,
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn variant(&self) -> &'static str {
        BUTTON_VARIANTS[self.variant]
    }

    pub fn fristil_version(&self) -> &str {
        &self.tokens.version
    }

    /// Sett tema, for eksempel fra panelet i nettleseren.
    pub fn set_theme(&mut self, theme: Theme, instant: bool) {
        self.theme = theme;
        if instant {
            self.night = if theme == Theme::Dark { 1.0 } else { 0.0 };
        }
    }

    pub fn set_variant(&mut self, name: &str) {
        if let Some(i) = BUTTON_VARIANTS.iter().position(|v| *v == name) {
            self.variant = i;
        }
    }

    /// Plasser kameraet, til skjermbilder og tester. `pos` er øyet.
    pub fn place(&mut self, eye: Vec3, yaw: f32, pitch: f32) {
        self.pos = eye - Vec3::Y * EYE;
        self.yaw = yaw;
        self.pitch = pitch;
    }

    pub fn set_gate(&mut self, open: bool, instant: bool) {
        self.gate.open = open;
        if instant {
            self.gate.t = if open { 1.0 } else { 0.0 };
        }
        self.update_props();
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::Y * (EYE + (self.bob * 9.0).sin() * 0.025)
    }

    pub fn update(&mut self, input: &mut Input, dt: f32) -> Vec<Event> {
        let dt = dt.clamp(0.0, 1.0 / 20.0);
        self.time += dt;
        let mut events = vec![];
        let actions = std::mem::take(&mut input.actions);

        if !self.paused {
            self.yaw -= input.look.0 * 0.0022;
            self.pitch = (self.pitch - input.look.1 * 0.0022).clamp(-1.5, 1.5);
        }
        input.look = (0.0, 0.0);

        // Gåing.
        let mut wish = Vec3::ZERO;
        if !self.paused {
            let fwd = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
            let right = Vec3::new(fwd.z * -1.0, 0.0, fwd.x);
            if input.forward { wish += fwd; }
            if input.back { wish -= fwd; }
            if input.right { wish += right; }
            if input.left { wish -= right; }
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
        let shape = Capsule::new_y(HALF, RADIUS);
        let pos = Isometry::translation(self.pos.x, self.pos.y, self.pos.z);
        let mv = self.controller.move_shape(
            dt,
            &self.bodies,
            &self.colliders,
            &self.query,
            &shape,
            &pos,
            vector![desired.x, desired.y, desired.z],
            QueryFilter::default(),
            |_| {},
        );
        self.pos += Vec3::new(mv.translation.x, mv.translation.y, mv.translation.z);
        self.grounded = mv.grounded;
        if self.pos.y < -20.0 {
            self.pos = Vec3::new(0.0, 2.0, 11.0);
        }
        if self.grounded && horizontal.length_squared() > 0.0 {
            self.bob += dt * speed / 4.2;
        }

        // Hva sikter vi på?
        let eye = self.eye();
        let dir = self.forward();
        let ray = Ray::new(point![eye.x, eye.y, eye.z], vector![dir.x, dir.y, dir.z]);
        let hit = self.query.cast_ray(&self.bodies, &self.colliders, &ray, 3.5, true, QueryFilter::default());
        let aim = match hit {
            Some((h, _)) if h == self.button_collider => Some(Prompt {
                component: "button".into(),
                class: ".fs-button".into(),
                variant: self.variant().into(),
            }),
            _ => None,
        };
        if aim != self.aim {
            self.aim = aim.clone();
            events.push(Event::Prompt(aim));
        }
        let target = if self.aim.is_some() { 1.0 } else { 0.0 };
        self.highlight += (target - self.highlight) * (dt * 10.0).min(1.0);

        for a in actions {
            match a {
                Action::Theme => {
                    self.theme = if self.theme == Theme::Light { Theme::Dark } else { Theme::Light };
                    events.push(Event::Theme(self.theme));
                }
                Action::Variant if self.aim.is_some() => {
                    self.variant = (self.variant + 1) % BUTTON_VARIANTS.len();
                    events.push(Event::Variant { component: "button".into(), variant: self.variant().into() });
                    let v = self.variant();
                    if let Some(p) = &mut self.aim {
                        p.variant = v.into();
                        events.push(Event::Prompt(Some(p.clone())));
                    }
                }
                Action::Use if self.aim.is_some() => {
                    self.press = 1.0;
                    self.gate.open = !self.gate.open;
                }
                Action::Inspect if self.aim.is_some() => {
                    events.push(Event::Inspect { component: "button".into(), variant: self.variant().into() });
                }
                _ => {}
            }
        }

        // Dag og natt glir over i hverandre på et par sekunder.
        let goal = if self.theme == Theme::Dark { 1.0 } else { 0.0 };
        let step = dt / 2.8;
        self.night = if self.night < goal { (self.night + step).min(goal) } else { (self.night - step).max(goal) };

        // Knappen og porten.
        self.press = (self.press - dt * 4.0).max(0.0);
        let gate_goal = if self.gate.open { 1.0 } else { 0.0 };
        let gstep = dt / 2.2;
        self.gate.t = if self.gate.t < gate_goal { (self.gate.t + gstep).min(gate_goal) } else { (self.gate.t - gstep).max(gate_goal) };
        self.update_props();

        events
    }

    fn update_props(&mut self) {
        let ease = |t: f32| t * t * (3.0 - 2.0 * t);
        let angle = ease(self.gate.t) * 105f32.to_radians();
        for (k, (i, h)) in self.gate.doors.iter().enumerate() {
            let sign = if k == 0 { 1.0 } else { -1.0 };
            let pose = self.prop_rest[*i] * Affine3A::from_rotation_y(sign * angle);
            self.prop_pose[*i] = pose;
            let (offset, _) = self.prop_offsets[*i];
            if let Some(c) = self.colliders.get_mut(*h) {
                c.set_position(to_iso(&(pose * Affine3A::from_translation(offset))));
            }
        }
        let push = (self.press * std::f32::consts::PI).sin().max(0.0) * 0.1;
        self.prop_pose[self.button_prop] = self.prop_rest[self.button_prop] * Affine3A::from_translation(Vec3::new(0.0, 0.0, -push));
        self.query.update(&self.colliders);
    }

    fn palette(&self) -> [Vec3; 16] {
        let t = self.night;
        let c = |name: &str| self.tokens.blend(name, t);
        let mut p = [Vec3::ONE; 16];
        let stone = crate::tokens::hex("#8f8c86");
        let (body, rim, label) = match self.variant() {
            "secondary" => (c("neutral-canvas"), c("accent-border"), c("accent-text")),
            "ghost" => (stone, stone, c("accent-text")),
            "danger" => (c("danger-surface"), c("danger-text"), c("danger-text")),
            _ => (c("accent-fill"), c("accent-fill"), c("accent-content")),
        };
        p[world::slot::BUTTON_BODY as usize] = body;
        p[world::slot::BUTTON_RIM as usize] = rim;
        p[world::slot::BUTTON_LABEL as usize] = label;
        p[world::slot::BRAND1 as usize] = c("brand1-fill");
        p[world::slot::BRAND2 as usize] = c("brand2-fill");
        p[world::slot::BRAND3 as usize] = c("brand3-fill");
        p[world::slot::ACCENT as usize] = c("accent-fill");
        p[world::slot::WARNING as usize] = c("warning-fill");
        p
    }

    pub fn frame(&self, aspect: f32) -> Frame {
        let t = self.night;
        let dusk = (t * std::f32::consts::PI).sin();
        let lerp3 = |a: Vec3, b: Vec3| a.lerp(b, t);

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
        let sun_intensity = 3.4 * (1.0 - t) + 0.38 * t;

        let sky_top = lerp3(Vec3::new(0.16, 0.34, 0.72), Vec3::new(0.008, 0.014, 0.04)).lerp(Vec3::new(0.18, 0.2, 0.42), dusk * 0.6);
        let sky_horizon = lerp3(Vec3::new(0.62, 0.74, 0.9), Vec3::new(0.016, 0.028, 0.06)).lerp(Vec3::new(1.0, 0.5, 0.28), dusk * 0.7);
        let ground = lerp3(Vec3::new(0.28, 0.26, 0.22), Vec3::new(0.012, 0.013, 0.018));

        let lamp = ((t - 0.35) / 0.5).clamp(0.0, 1.0);
        let lights = self
            .world_lights
            .iter()
            .map(|(p, r, c, i)| (*p, *r, *c, *i * lamp))
            .filter(|l| l.3 > 0.0)
            .collect();

        let props = self
            .prop_pose
            .iter()
            .enumerate()
            .map(|(i, p)| (*p, if i == self.button_prop { self.highlight } else { 0.0 }))
            .collect();

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
            ground_bounce: ground,
            fog_color: lerp3(Vec3::new(0.6, 0.7, 0.82), Vec3::new(0.012, 0.02, 0.04)),
            fog_density: 0.0035 + dusk * 0.002,
            fog_falloff: 0.025,
            exposure: 0.95 * (1.0 - t) + 2.6 * t,
            bloom: 0.06 + t * 0.06,
            windows_lit: 0.5,
            palette: self.palette(),
            lights,
            props,
        }
    }
}
