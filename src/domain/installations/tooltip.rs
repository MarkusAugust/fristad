//! `.fs-tooltip`: arkivknappen ved søknadskontoret. Boblen viser seg når
//! du sikter på knappen, slik den gjør ved hover og fokus.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

const POS: Vec3 = Vec3::new(42.2, 0.0, 9.0);
const YAW: f32 = std::f32::consts::PI;

#[derive(Default)]
pub struct Tooltip {
    button: usize,
    bubble: usize,
    t: f32,
}

impl Installation for Tooltip {
    fn component(&self) -> Component {
        Component { name: "tooltip", class: ".fs-tooltip" }
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(POS);
        let t = at(p, YAW);
        // En stolpe i jern med en liten plate, som knappen sitter på.
        b.mesh.cylinder(&t, 0.035, 0.035, 1.3, 8, &IRON, false);
        // Plata i canvas-fargen: ghost-knappen har ingen egen bakgrunn.
        let mut plate = Mesh::default();
        signs::boxed(&mut plate, Vec3::ZERO, &Box2 { w: 120.0, h: 56.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.05 });
        plate.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.3, 0.0))));
        b.mesh.append(&plate);
        b.solid_box(p + Vec3::Y * 0.7, Vec3::new(0.3, 1.6, 0.3), YAW);
        // Ghost-knappen «Arkiver»: ingen kant, ingen bakgrunn, aksenttekst.
        let mut m = Mesh::default();
        let w = signs::width(b.font, "Arkiver", 16.0) / PX + 34.0;
        signs::boxed(&mut m, Vec3::ZERO, &Box2 { w, h: 42.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: None, depth: 0.02 });
        signs::text(&mut m, b.font, "Arkiver", Vec3::new(0.0, 0.0, 0.024), 16.0, slot::ACCENT_TEXT);
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, 1.3, 0.04));
        self.button = b.prop("tooltip", m, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, 21.0 * PX, 0.04))));
        // Boblen: 14 px tekst med linjehøyde 1,25, padding 4/8, radius 4,
        // mørk flate med lys tekst, 8 px over knappen.
        let lines = ["Saken flyttes til arkivet og kan", "hentes fram igjen"];
        let bw = lines.iter().map(|l| signs::width(b.font, l, 14.0) / PX).fold(0.0, f32::max) + 16.0;
        let bh = 8.0 + 14.0 * 1.25 * 2.0;
        let mut bubble = Mesh::default();
        signs::boxed(&mut bubble, Vec3::ZERO, &Box2 { w: bw, h: bh, radius: 4.0, fill: slot::NEUTRAL_TEXT_STRONG, border: None, depth: 0.02 });
        signs::paragraph(&mut bubble, b.font, &lines, -bw * 0.5 + 8.0, bh * 0.5 - 4.0, 14.0, 1.25, 0.024, slot::NEUTRAL_CANVAS);
        // Liten pil ned mot knappen.
        let dark = Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_TEXT_STRONG).flags(flag::NO_DIRT | flag::NO_WEATHER);
        bubble.cuboid(&(Affine3A::from_translation(Vec3::new(0.0, (-bh * 0.5) * PX, 0.0)) * Affine3A::from_rotation_z(0.785)), Vec3::new(8.0 * PX, 8.0 * PX, 0.02), &dark);
        self.bubble = b.prop("tooltip_bubble", bubble, rest * Affine3A::from_translation(Vec3::new(0.0, (21.0 + 8.0 + bh * 0.5) * PX, 0.0)), None);
    }

    fn tick(&mut self, env: &mut Env) {
        let goal = if env.aimed_prop == Some(self.button) { 1.0 } else { 0.0 };
        // 120 ms toning, som i CSS-en.
        self.t += (goal - self.t) * (env.dt / 0.12).min(1.0);
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.bubble {
            let s = if self.t > 0.05 { 1.0 } else { 0.0001 };
            return (rest * Affine3A::from_scale(Vec3::splat(s)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
