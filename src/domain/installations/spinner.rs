//! `.fs-spinner`: ringen som snurrer på masta i det nordøstre hjørnet av
//! torget, 800 ms per runde som i CSS-en. R bytter størrelse.

use super::{Component, Env, Installation, PX};
use crate::domain::city::{Builder, IRON};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

pub const VARIANTS: [&str; 3] = ["small", "medium", "large"];

#[derive(Default)]
pub struct Spinner {
    variant: usize,
    ring: usize,
}

const POS: Vec3 = Vec3::new(12.5, 0.0, -14.0);

impl Installation for Spinner {
    fn component(&self) -> Component {
        Component { name: "spinner", class: ".fs-spinner" }
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
        self.variant = 1;
        let m = &mut b.mesh;
        m.lathe(&Affine3A::from_translation(POS), &[glam::Vec2::new(0.25, 0.0), glam::Vec2::new(0.25, 0.08), glam::Vec2::new(0.06, 0.12), glam::Vec2::new(0.06, 2.4), glam::Vec2::new(0.04, 2.6)], 12, &IRON, true, 0.7);
        b.solid_box(POS + Vec3::Y * 1.3, Vec3::new(0.12, 2.6, 0.12), 0.0);
        // Ringen: 2px tykk kant av 24px ring i CSS; her forstørret 4×, så
        // middels er 96 cm. Det øverste kvartsegmentet har aksentfargen.
        let mut ring = Mesh::default();
        let track = Mat::plain(0xffffff, 0.4).slot(slot::NEUTRAL_SURFACE).flags(flag::NO_WEATHER | flag::NO_DIRT);
        let accent = Mat::plain(0xffffff, 0.4).slot(slot::ACCENT_BORDER).flags(flag::NO_WEATHER | flag::NO_DIRT | flag::GLOW_ALWAYS);
        let r = 12.0 * PX * 4.0;
        let tube = 1.0 * PX * 4.0;
        // Ringen ligger i XZ i torus(); vi reiser den opp i XY.
        let stand = Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2);
        ring.torus(&stand, r, tube, 40, 10, &track);
        quarter(&mut ring, r, tube * 1.15, &accent);
        self.ring = b.prop("spinner", ring, Affine3A::from_translation(POS + Vec3::Y * 2.6), Some((Vec3::ZERO, Vec3::new(0.6, 0.6, 0.1))));
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.ring {
            let scale = [0.5, 1.0, 1.5][self.variant];
            let angle = -env.time / 0.8 * std::f32::consts::TAU;
            return (rest * Affine3A::from_scale(Vec3::splat(scale)) * Affine3A::from_quat(Quat::from_rotation_z(angle)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}

/// Den øverste fjerdedelen av ringen, som en tykkere torusbit i XY-planet.
fn quarter(m: &mut Mesh, radius: f32, tube: f32, mat: &Mat) {
    let seg = 12;
    let sides = 10;
    let base = m.vertices.len() as u32;
    for i in 0..=seg {
        let a = std::f32::consts::FRAC_PI_4 + i as f32 / seg as f32 * std::f32::consts::FRAC_PI_2;
        let c = Vec3::new(a.cos(), a.sin(), 0.0);
        for j in 0..=sides {
            let bb = j as f32 / sides as f32 * std::f32::consts::TAU;
            let n = c * bb.cos() + Vec3::Z * bb.sin();
            let p = c * radius + n * tube;
            m.vert(p, n, glam::Vec2::ZERO, mat, 1.0);
        }
    }
    let w = sides + 1;
    for i in 0..seg {
        for j in 0..sides {
            let a = base + i * w + j;
            let b2 = a + w;
            m.indices.extend_from_slice(&[a, a + 1, b2, a + 1, b2 + 1, b2]);
        }
    }
}
