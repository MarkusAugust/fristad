//! `.fs-accordion`: oppslagsskapet ved Vestre gate, med to paneler som
//! kan foldes ut. F folder det øverste ut og inn, og det nederste flytter
//! seg etter, slik `<details>` gjør. R bytter variant (standard, plain).
//! Panelene er ark fra Fristils CSS: ett per variant og tilstand.

use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["default", "plain"];

#[derive(Default)]
pub struct Accordion {
    variant: usize,
    open: bool,
    /// Det første panelet: [variant][lukket, åpen].
    first: [[usize; 2]; 2],
    /// Det andre panelet per variant, og høydene som styrer hvor det står.
    second: [usize; 2],
    heights: [[f32; 2]; 2],
}

const POS: Vec3 = Vec3::new(-35.0, 0.0, -7.8);

impl Installation for Accordion {
    fn component(&self) -> Component {
        Component { name: "accordion", class: ".fs-accordion" }
    }
    fn variants(&self) -> &'static [&'static str] {
        &VARIANTS
    }
    fn variant(&self) -> usize {
        self.variant
    }
    fn set_variant(&mut self, i: usize) {
        self.variant = i % VARIANTS.len();
    }
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(POS);
        let t = at(p, 0.0);
        // Skapet: stolper, bakplate, og et lite tak i skifer.
        let m = &mut b.mesh;
        let (bw, bh) = (4.2, 2.8);
        for dx in [-1.9, 1.9] {
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, bh * 0.5 + 0.2, -0.06))), Vec3::new(0.12, bh + 0.8, 0.12), &OAK, 1.0);
        }
        let back = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.5, 0.45, 0.38));
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.7 + bh * 0.5, 0.0))), Vec3::new(bw, bh, 0.06), &back, 1.0);
        m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.7 + bh + 0.05, 0.0))), bw + 0.3, 0.6, 0.3, 0.1, &Mat::tex(Layer::Slate, 0xffffff), &OAK, &OAK);
        b.solid_box(p + Vec3::Y * (0.7 + bh * 0.5), Vec3::new(bw, bh, 0.2), 0.0);

        // Panelene henger fra toppen av skapet: origo i overkanten av hvert
        // ark, så det andre kan flyttes ned etter høyden på det første.
        let top_y = 0.7 + bh - 0.2;
        let top = t * Affine3A::from_translation(Vec3::new(0.0, top_y, 0.05));
        for (k, v) in VARIANTS.iter().enumerate() {
            for (s, state) in ["closed", "open"].iter().enumerate() {
                let mut m = Mesh::default();
                let s_id = format!("accordion:{v}:{state}");
                let h = b.sheets.sheet(&s_id).h;
                let (w, _) = signs::sheet(&mut m, b.sheets, &s_id, Vec3::new(0.0, -h * PX * 0.5, 0.0), 0.03, 4.0, slot::NEUTRAL_CANVAS);
                self.heights[k][s] = h;
                self.first[k][s] = b.prop("accordion", m, top, Some((Vec3::new(0.0, -h * PX * 0.5, 0.0), Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.05))));
            }
            let mut m = Mesh::default();
            let s_id = format!("accordion2:{v}:closed");
            let h = b.sheets.sheet(&s_id).h;
            signs::sheet(&mut m, b.sheets, &s_id, Vec3::new(0.0, -h * PX * 0.5, 0.0), 0.03, 4.0, slot::NEUTRAL_CANVAS);
            self.second[k] = b.prop("accordion_second", m, top, None);
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            self.open = !self.open;
            env.toasts.push(if self.open { "Panelet er foldet ut".into() } else { "Panelet er foldet sammen".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        let state = usize::from(self.open);
        for k in 0..2 {
            for s in 0..2 {
                if prop == self.first[k][s] {
                    return (if k == self.variant && s == state { rest } else { rest * hidden }, [0.0; 4]);
                }
            }
            if prop == self.second[k] {
                if k != self.variant {
                    return (rest * hidden, [0.0; 4]);
                }
                // Under det første panelet, med 8 px mellomrom.
                let dy = -(self.heights[k][state] + 8.0) * PX;
                return (rest * Affine3A::from_translation(Vec3::Y * dy), [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }
}
