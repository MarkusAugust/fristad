//! `.fs-progress`: framdriftsstolpen over porten viser hvor åpen den er.
//! R bytter farge (data-color). Sporet og fyllet er ark fra Fristils CSS;
//! fyllet skaleres i bredden etter verdien.

use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["accent", "success", "warning", "danger"];

#[derive(Default)]
pub struct Progress {
    variant: usize,
    track: usize,
    fills: [usize; 4],
}

const POS: Vec3 = Vec3::new(0.0, 5.0, -18.08);

impl Installation for Progress {
    fn component(&self) -> Component {
        Component { name: "progress", class: ".fs-progress" }
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

    fn build(&mut self, b: &mut Builder) {
        let mut track = Mesh::default();
        let (w, h) = signs::sheet(&mut track, b.sheets, "progress:track", Vec3::ZERO, 0.03, 4.0, slot::NEUTRAL_SURFACE);
        // Etiketten under, som aria-label ville sagt det.
        signs::text(&mut track, b.font, "Porten åpnes", Vec3::new(0.0, -0.2, 0.025), 14.0, slot::NEUTRAL_TEXT_SUBTLE);
        self.track = b.prop("progress", track, Affine3A::from_translation(POS), Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, 0.25, 0.05))));
        for (k, c) in VARIANTS.iter().enumerate() {
            let mut fill = Mesh::default();
            // Fyllet bygges med venstre kant i origo, så det kan skaleres i x.
            signs::sheet(&mut fill, b.sheets, &format!("progress:{c}"), Vec3::new(w * PX * 0.5, 0.0, 0.0), 0.03, 4.0, slot::NEUTRAL_SURFACE);
            let _ = h;
            self.fills[k] = b.prop("progress_fill", fill, Affine3A::from_translation(POS + Vec3::new(-w * PX * 0.5, 0.0, 0.006)), None);
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(k) = self.fills.iter().position(|p| *p == prop) {
            if k != self.variant {
                return (rest * Affine3A::from_scale(Vec3::splat(0.0001)), [0.0; 4]);
            }
            let t = env.gate_t.max(0.02);
            return (rest * Affine3A::from_scale(Vec3::new(t, 1.0, 1.0)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
