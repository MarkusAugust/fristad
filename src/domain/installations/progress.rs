//! `.fs-progress`: framdriftsstolpen over porten viser hvor åpen den er.
//! R bytter farge (data-color).

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["accent", "success"];

#[derive(Default)]
pub struct Progress {
    variant: usize,
    track: usize,
    fill: usize,
}

/// 320 px bred, 8 px høy, på overliggeren over porten.
const W: f32 = 320.0;
const H: f32 = 8.0;
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
        signs::boxed(&mut track, Vec3::ZERO, &Box2 { w: W, h: H, radius: 4.0, fill: slot::NEUTRAL_SURFACE, border: None, depth: 0.03 });
        // Etiketten under, som aria-label ville sagt det.
        signs::text(&mut track, b.font, "Porten åpnes", Vec3::new(0.0, -0.2, 0.025), 14.0, slot::NEUTRAL_TEXT_SUBTLE);
        self.track = b.prop("progress", track, Affine3A::from_translation(POS), Some((Vec3::ZERO, Vec3::new(W * PX * 0.5, 0.25, 0.05))));
        let mut fill = Mesh::default();
        // Fyllet bygges med venstre kant i origo, så det kan skaleres i x.
        let fm = Mat::plain(0xffffff, 0.45).slot(PROGRESS_FILL).flags(flag::NO_DIRT | flag::NO_WEATHER | flag::GLOW_ALWAYS);
        fill.rounded_slab(&Affine3A::IDENTITY, Vec3::new(W * PX * 0.5, 0.0, 0.0), W * PX, H * PX, 0.03, 4.0 * PX, &fm);
        self.fill = b.prop("progress_fill", fill, Affine3A::from_translation(POS + Vec3::new(-W * PX * 0.5, 0.0, 0.006)), None);
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.fill {
            let t = env.gate_t.max(0.02);
            return (rest * Affine3A::from_scale(Vec3::new(t, 1.0, 1.0)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let name = if VARIANTS[self.variant] == "success" { "success-fill" } else { "accent-fill" };
        p[PROGRESS_FILL as usize] = tokens.blend(name, night);
    }
}

pub const PROGRESS_FILL: u8 = slot::PROGRESS_FILL;
