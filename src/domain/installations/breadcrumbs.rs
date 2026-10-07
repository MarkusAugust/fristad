//! `.fs-breadcrumbs`: messingplater i brosteinen som viser veien fra gata i
//! sør til porten: Forsiden › Torget › Porten.

use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

#[derive(Default)]
pub struct Breadcrumbs {
    plates: Vec<usize>,
}

impl Installation for Breadcrumbs {
    fn component(&self) -> Component {
        Component { name: "breadcrumbs", class: ".fs-breadcrumbs" }
    }

    fn build(&mut self, b: &mut Builder) {
        let brass = Mat::plain(0xb8924a, 0.35).flags(flag::NO_DIRT);
        let steps = [("Forsiden", Vec3::new(1.5, 0.0, 26.0), false), ("Torget", Vec3::new(7.5, 0.0, 9.0), false), ("Porten", Vec3::new(6.0, 0.0, -11.0), true)];
        for (k, (label, pos, current)) in steps.iter().enumerate() {
            let mut m = Mesh::default();
            // 14px tekst på en plate med 8px luft, lagt flatt i bakken.
            let tw = signs::width(b.font, label, 14.0) / PX;
            let w = tw + 24.0;
            let h = 14.0 * 1.5 + 12.0;
            let lay = Affine3A::from_quat(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
            m.rounded_slab(&lay, Vec3::new(0.0, 0.0, 0.0), w * PX, h * PX, 0.02, 3.0 * PX, &brass);
            let mut text = Mesh::default();
            let color = if *current { slot::NEUTRAL_TEXT_STRONG } else { slot::ACCENT_TEXT };
            signs::text(&mut text, b.font, label, Vec3::new(0.0, 0.0, 0.0), 14.0, color);
            text.transform(&(lay * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.022))));
            m.append(&text);
            // Skilletegnet › foran alle unntatt den første.
            if k > 0 {
                let mut sep = Mesh::default();
                signs::text(&mut sep, b.font, "›", Vec3::ZERO, 14.0, slot::NEUTRAL_TEXT_SUBTLE);
                sep.transform(&(lay * Affine3A::from_translation(Vec3::new(-(w * 0.5 + 10.0) * PX, 0.0, 0.022))));
                let mut sep_plate = Mesh::default();
                sep_plate.rounded_slab(&lay, Vec3::new(-(w * 0.5 + 10.0) * PX, 0.0, 0.0), 14.0 * PX, h * PX, 0.02, 3.0 * PX, &brass);
                m.append(&sep_plate);
                m.append(&sep);
            }
            // Platene vender mot den som kommer sørfra.
            let yaw = (steps[(k + 1).min(2)].1 - steps[k.max(1) - 1].1).x.atan2((steps[(k + 1).min(2)].1 - steps[k.max(1) - 1].1).z) + std::f32::consts::PI;
            let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), b.on_ground(*pos) + Vec3::Y * 0.012);
            let i = b.prop("breadcrumbs", m, t, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5 + 0.2, 0.4, h * PX * 0.5))));
            self.plates.push(i);
        }
    }

    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }
}
