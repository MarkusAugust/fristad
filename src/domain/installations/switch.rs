//! `.fs-switch`: bryteren for gatelysene, på en stolpe ved torget.
//! F slår den av og på, og lyktene følger.

use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{Builder, IRON};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

#[derive(Default)]
pub struct Switch {
    body: usize,
    knob: usize,
    on: bool,
    t: f32,
}

const POS: Vec3 = Vec3::new(-6.4, 0.0, -15.2);

impl Installation for Switch {
    fn component(&self) -> Component {
        Component { name: "switch", class: ".fs-switch" }
    }

    fn build(&mut self, b: &mut Builder) {
        self.on = true;
        self.t = 1.0;
        // Stolpe i jern med en plate, og etiketten «Gatelys» ved siden av.
        let m = &mut b.mesh;
        m.cylinder(&Affine3A::from_translation(POS), 0.05, 0.05, 1.45, 10, &IRON, true);
        m.cylinder(&Affine3A::from_translation(POS), 0.16, 0.1, 0.06, 12, &IRON, true);
        let plate = Vec3::new(POS.x, 1.3, POS.z + 0.05);
        m.cuboid(&Affine3A::from_translation(plate), Vec3::new(1.5, 0.5, 0.05), &Mat::plain(0xf4f1ea, 0.5).slot(slot::NEUTRAL_CANVAS).flags(flag::NO_DIRT));
        signs::text_left(m, b.font, "Gatelys", Vec3::new(plate.x - 0.2, plate.y - 0.06, plate.z + 0.03), 16.0, slot::NEUTRAL_TEXT_STRONG);
        b.solid_box(POS + Vec3::Y * 0.75, Vec3::new(0.3, 1.5, 0.3), 0.0);

        // Selve bryteren: 40 × 24 px med 1px kant, knotten 24 px i diameter.
        let mut body = Mesh::default();
        signs::boxed(&mut body, Vec3::ZERO, &Box2 { w: 40.0, h: 24.0, radius: 12.0, fill: SWITCH_BODY, border: Some(slot::SWITCH_BORDER), depth: 0.04 });
        let rest = Affine3A::from_translation(Vec3::new(plate.x - 0.48, plate.y, plate.z + 0.045));
        self.body = b.prop("switch", body, rest, Some((Vec3::ZERO, Vec3::new(0.22, 0.14, 0.06))));
        let mut knob = Mesh::default();
        let km = Mat::plain(0xffffff, 0.4).slot(slot::SWITCH_KNOB).flags(flag::NO_DIRT | flag::NO_WEATHER);
        knob.cylinder(&Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2), 8.2 * PX, 8.2 * PX, 0.02, 20, &km, true);
        self.knob = b.prop("switch_knob", knob, rest * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.045)), None);
    }

    fn tick(&mut self, env: &mut Env) {
        let goal = if self.on { 1.0 } else { 0.0 };
        self.t += (goal - self.t) * (env.dt * 12.0).min(1.0);
        env.street_lights = self.on;
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            self.on = !self.on;
            env.toasts.push(if self.on { "Gatelysene er tent".into() } else { "Gatelysene er slukket".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.knob {
            // Knotten glir fra venstre til høyre: 16 px.
            let x = (-8.0 + 16.0 * self.t) * PX;
            return (rest * Affine3A::from_translation(Vec3::new(x, 0.0, 0.0)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        // Påslått bruker aksentfyll, avslått nøytral flate. Deler plass med
        // knappen? Nei: knappen setter sin egen etterpå, så vi bruker
        // bakgrunnsfargen fra en fri plass. (Plass 1 settes av knappen;
        // bryteren får kroppen sin via NEUTRAL_SURFACE/ACCENT_FILL under.)
        let on = tokens.blend("accent-fill", night);
        let off = tokens.blend("neutral-surface", night);
        p[SWITCH_BODY as usize] = off.lerp(on, self.t);
        p[slot::SWITCH_BORDER as usize] = tokens.blend("neutral-border", night).lerp(on, self.t);
        p[slot::SWITCH_KNOB as usize] = tokens.blend("neutral-text-strong", night).lerp(tokens.blend("accent-content", night), self.t);
    }

    fn lights_on(&self, _env: &Env) -> f32 {
        self.t
    }
}

/// Bryteren har egen palettplass så den ikke kolliderer med knappen.
pub const SWITCH_BODY: u8 = slot::SWITCH_BODY;
