//! `.fs-breadcrumbs`: messingplatene i brosteinen ved inngangen fra sør,
//! som viser veien: Forsiden › Torget › Porten. F på en plate tar deg dit.

use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

/// Stien ligger på tvers av sørinngangen til torget.
const POS: Vec3 = Vec3::new(0.0, 0.0, 21.5);
/// Trinnene: tekst, om det er sida du er på, og dit du kommer (føtter, yaw).
const STEPS: [(&str, bool, Vec3, f32); 3] = [
    ("Forsiden", false, Vec3::new(1.5, 0.0, 31.0), 0.0),
    ("Torget", false, Vec3::new(0.0, 0.0, 8.5), 0.0),
    ("Porten", true, Vec3::new(0.0, 0.0, -13.0), 0.0),
];

#[derive(Default)]
pub struct Breadcrumbs {
    plates: [usize; 3],
}

impl Installation for Breadcrumbs {
    fn component(&self) -> Component {
        Component { name: "breadcrumbs", class: ".fs-breadcrumbs" }
    }

    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        let brass = Mat::plain(0xb8924a, 0.35).flags(flag::NO_DIRT);
        // 14px tekst på plater med 8px luft, lagt flatt i bakken, i en rad
        // som leses av den som kommer sørfra.
        let h = 14.0 * 1.5 + 12.0;
        let lay = Affine3A::from_quat(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
        let widths: Vec<f32> = STEPS.iter().map(|(label, ..)| signs::width(b.font, label, 14.0) / PX + 24.0).collect();
        let sep_w = 14.0;
        let gap = 8.0;
        let total: f32 = widths.iter().sum::<f32>() + 2.0 * (sep_w + 2.0 * gap);
        let base = b.on_ground(POS) + Vec3::Y * 0.012;
        let mut x = -total * 0.5;
        for (k, (label, current, _, _)) in STEPS.iter().enumerate() {
            let w = widths[k];
            let mut m = Mesh::default();
            if k > 0 {
                // Skilletegnet › mellom trinnene, i dempet farge.
                let mut sep = Mesh::default();
                sep.rounded_slab(&lay, Vec3::ZERO, sep_w * PX, h * PX, 0.02, 3.0 * PX, &brass);
                let mut t = Mesh::default();
                signs::text(&mut t, b.font, "›", Vec3::ZERO, 14.0, slot::NEUTRAL_TEXT_SUBTLE);
                t.transform(&(lay * Affine3A::from_translation(Vec3::Z * 0.022)));
                sep.append(&t);
                sep.transform(&Affine3A::from_translation(base + Vec3::X * ((x + sep_w * 0.5) * PX)));
                b.mesh.append(&sep);
                x += sep_w + gap;
            }
            m.rounded_slab(&lay, Vec3::ZERO, w * PX, h * PX, 0.02, 3.0 * PX, &brass);
            let mut text = Mesh::default();
            let color = if *current { slot::NEUTRAL_TEXT_STRONG } else { slot::ACCENT_TEXT };
            signs::text(&mut text, b.font, label, Vec3::ZERO, 14.0, color);
            text.transform(&(lay * Affine3A::from_translation(Vec3::Z * 0.022)));
            m.append(&text);
            let t = Affine3A::from_translation(base + Vec3::X * ((x + w * 0.5) * PX));
            self.plates[k] = b.prop("breadcrumbs", m, t, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, 0.4, h * PX * 0.5))));
            x += w + gap;
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        let Some(k) = self.plates.iter().position(|p| Some(*p) == env.aimed_prop) else { return };
        let (label, _, feet, yaw) = STEPS[k];
        env.teleport = Some((feet, yaw));
        env.toasts.push(format!("Gikk til «{label}»"));
    }

    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }
}
