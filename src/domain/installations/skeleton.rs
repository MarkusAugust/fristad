//! `.fs-skeleton`: huset under bygging i enden av gata østover. Fasaden er
//! grå plassholdere som skimrer, med stillas rundt. R bytter variant
//! (tekst, sirkel, blokk) på den første plassholderen.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{Builder, IRON, OAK};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

pub const VARIANTS: [&str; 3] = ["text", "circle", "block"];

#[derive(Default)]
pub struct Skeleton {
    variant: usize,
    first: [usize; 3],
}

/// Tomta: nord for Østre gate bak busskuret, øst for tabs-huset, med fasaden mot gata.
const FRONT: Vec3 = Vec3::new(47.0, 0.0, -14.0);

impl Installation for Skeleton {
    fn component(&self) -> Component {
        Component { name: "skeleton", class: ".fs-skeleton" }
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
        // Huset: et grått volum i skeleton-materialet, med plassholdere
        // der vinduene og døra skulle vært.
        let rot = Quat::from_rotation_y(0.0);
        let front = b.on_ground(FRONT);
        let local = |p: Vec3| Affine3A::from_rotation_translation(rot, front + rot * p);
        let m = &mut b.mesh;
        let body = Mat::special(kind::SKELETON, 0xffffff, 0.8).slot(slot::NEUTRAL_SURFACE).flags(flag::NO_DIRT);
        let (w, d, h) = (7.2, 9.0, 9.4);
        // Kroppen går 1,5 m ned i bakken, så den står støtt i hellingen.
        m.cuboid(&local(Vec3::new(0.0, h * 0.5 - 0.75, -d * 0.5)), Vec3::new(w, h + 1.5, d), &body);
        m.gable_roof(&local(Vec3::new(0.0, h, -d * 0.5)), w, d, 3.0, 0.4, &body, &body, &body);
        b.solid_box(front + rot * Vec3::new(0.0, h * 0.5, -d * 0.5), Vec3::new(w + 0.2, h, d + 0.2), 0.0);

        // Plassholderne på fasaden, som i eksempelet: sirkel, to tekstlinjer, bilde.
        let canvas = slot::NEUTRAL_CANVAS;
        for f in 0..3 {
            let y = 1.2 + f as f32 * 3.0;
            for (k, x) in [-2.2f32, 0.0, 2.2].iter().enumerate() {
                let mut pm = Mesh::default();
                let kind_i = if f == 0 && k == 0 { self.variant } else { (f + k) % 3 };
                let (bw, bh, r) = match kind_i {
                    0 => (100.0, 16.0, 4.0),
                    1 => (48.0, 48.0, 24.0),
                    _ => (100.0, 150.0, 4.0),
                };
                signs::boxed(&mut pm, Vec3::ZERO, &Box2 { w: bw, h: bh, radius: r, fill: canvas, border: None, depth: 0.04 });
                let t = local(Vec3::new(*x, y + bh * PX * 0.5, 0.03));
                if f == 0 && k == 0 {
                    // Tre varianter av den første; bare én vises.
                    for v in 0..3 {
                        let (bw, bh, r) = match v {
                            0 => (100.0, 16.0, 4.0),
                            1 => (48.0, 48.0, 24.0),
                            _ => (100.0, 150.0, 4.0),
                        };
                        let mut vm = Mesh::default();
                        signs::boxed(&mut vm, Vec3::ZERO, &Box2 { w: bw, h: bh, radius: r, fill: canvas, border: None, depth: 0.04 });
                        let vt = local(Vec3::new(*x, y + bh * PX * 0.5, 0.03));
                        self.first[v] = b.prop("skeleton", vm, vt, if v == 0 { Some((Vec3::ZERO, Vec3::new(0.8, 0.8, 0.1))) } else { None });
                    }
                } else {
                    let mut static_m = pm;
                    static_m.transform(&t);
                    b.mesh.append(&static_m);
                }
            }
        }

        // Stillaset: stolper, tverrbjelker og plankedekker foran fasaden.
        let m = &mut b.mesh;
        for level in 0..3 {
            let y = 2.6 + level as f32 * 2.9;
            for x in [-3.0, -1.0, 1.0, 3.0] {
                m.cylinder(&local(Vec3::new(x, 0.0, 1.2)), 0.035, 0.035, 11.5, 8, &IRON, false);
                m.cylinder(&local(Vec3::new(x, 0.0, 0.3)), 0.035, 0.035, 11.5, 8, &IRON, false);
                m.cylinder(&(local(Vec3::new(x, y, 1.2)) * Affine3A::from_rotation_x(-std::f32::consts::FRAC_PI_2)), 0.03, 0.03, 0.9, 6, &IRON, false);
            }
            m.cylinder(&(local(Vec3::new(-3.2, y, 1.2)) * Affine3A::from_rotation_z(-std::f32::consts::FRAC_PI_2)), 0.03, 0.03, 6.4, 6, &IRON, false);
            m.cylinder(&(local(Vec3::new(-3.2, y + 1.0, 1.2)) * Affine3A::from_rotation_z(-std::f32::consts::FRAC_PI_2)), 0.03, 0.03, 6.4, 6, &IRON, false);
            for k in 0..3 {
                m.cuboid(&local(Vec3::new(0.0, y + 0.03, 0.45 + k as f32 * 0.3)), Vec3::new(6.4, 0.05, 0.28), &OAK);
            }
        }
        let _ = Layer::Oak;
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(v) = self.first.iter().position(|p| *p == prop) {
            if v == self.variant {
                return (rest, [0.0; 4]);
            }
            return (rest * Affine3A::from_scale(Vec3::splat(0.0001)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
