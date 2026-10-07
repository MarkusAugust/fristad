//! `.fs-tooltip`: arkivknappen ved søknadskontoret. Boblen viser seg når
//! du sikter på knappen, slik den gjør ved hover og fokus, og blir stående
//! mens du ser på boblen. Knappen gjør ikke noe annet. Knappen og boblen
//! er ark rendret fra Fristils CSS.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON};
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

const POS: Vec3 = Vec3::new(42.2, 0.0, 9.0);
const YAW: f32 = std::f32::consts::PI;

#[derive(Default)]
pub struct Tooltip {
    button: usize,
    bubble: usize,
    /// Sekunder siden siktet sist var på knappen eller boblen.
    since: f32,
    shown: bool,
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
        let bw_btn = b.sheets.sheet("tooltip:button").w;
        let mut plate = Mesh::default();
        signs::boxed(&mut plate, Vec3::ZERO, &Box2 { w: bw_btn + 32.0, h: 58.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.05 });
        plate.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.3, 0.0))));
        b.mesh.append(&plate);
        b.solid_box(p + Vec3::Y * 0.7, Vec3::new(0.3, 1.6, 0.3), YAW);
        // Ghost-knappen «Arkiver» fra CSS-en.
        let mut m = Mesh::default();
        let (w, h) = signs::sheet(&mut m, b.sheets, "tooltip:button", Vec3::ZERO, 0.02, 4.0, slot::NEUTRAL_CANVAS);
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, 1.3, 0.04));
        self.button = b.prop("tooltip", m, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.04))));
        // Boblen fra CSS-en, 8 px over knappen, med pil ned.
        let mut bubble = Mesh::default();
        let (bw, bh) = signs::sheet(&mut bubble, b.sheets, "tooltip:bubble", Vec3::ZERO, 0.02, 4.0, slot::NEUTRAL_TEXT_STRONG);
        signs::boxed(&mut bubble, Vec3::new(0.0, -bh * 0.5 * PX, -0.004), &Box2 { w: 8.0, h: 8.0, radius: 0.0, fill: slot::NEUTRAL_TEXT_STRONG, border: None, depth: 0.012 });
        let brest = rest * Affine3A::from_translation(Vec3::new(0.0, (h * 0.5 + 8.0 + bh * 0.5) * PX, 0.0));
        self.bubble = b.prop("tooltip_bubble", bubble, brest, Some((Vec3::ZERO, Vec3::new(bw * PX * 0.5, bh * PX * 0.5, 0.03))));
    }

    fn tick(&mut self, env: &mut Env) {
        if env.aimed_prop == Some(self.button) || env.aimed_prop == Some(self.bubble) {
            self.since = 0.0;
        } else {
            self.since += env.dt;
        }
        // Som hover: vises mens siktet er på, og forsvinner 0,2 s etter.
        self.shown = self.since < 0.2;
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.bubble {
            let s = if self.shown { 1.0 } else { 0.0001 };
            return (rest * Affine3A::from_scale(Vec3::splat(s)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
