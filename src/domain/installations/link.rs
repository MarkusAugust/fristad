//! `.fs-link`: veiviseren på torget, med armer som peker dit lenkene går.
//! F på en arm følger lenka: du flys dit. R og panelet bytter tilstand på
//! alle lenkene i byen (vanlig, besøkt, deaktivert); teksten på armene er
//! ark rendret fra Fristils CSS, ett per tilstand og side.

use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "visited", "disabled"];
const POS: Vec3 = Vec3::new(-7.4, 0.0, 12.4);
/// Armene: tekst, retning (yaw, dit armen peker) og høyde.
const ARMS: [(&str, f32, f32); 3] = [("Mine søknader", 0.05, 2.45), ("Torget", 0.53, 2.1), ("Dokumentasjonen", -1.32, 1.75)];
/// Dit lenkene går: føttene, og retningen du ser i når du kommer fram.
const TARGETS: [(Vec3, f32); 3] = [
    (Vec3::new(36.0, 0.0, 5.0), std::f32::consts::PI),
    (Vec3::new(0.0, 0.0, 8.5), 0.0),
    (Vec3::new(9.0, 0.0, 76.0), -std::f32::consts::FRAC_PI_2),
];

#[derive(Default)]
pub struct Link {
    variant: usize,
    /// Planken per arm (den du sikter på).
    arms: [usize; 3],
    /// Teksten per arm, side og variant.
    labels: [[[usize; 3]; 2]; 3],
}

impl Installation for Link {
    fn component(&self) -> Component {
        Component { name: "link", class: ".fs-link" }
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
        let p = b.on_ground(POS);
        let m = &mut b.mesh;
        m.cylinder(&Affine3A::from_translation(p), 0.07, 0.06, 2.7, 10, &OAK, true);
        m.sphere(&Affine3A::from_translation(p + Vec3::Y * 2.75), Vec3::splat(0.09), 5, 8, &OAK);
        b.solid_box(p + Vec3::Y * 1.3, Vec3::new(0.3, 2.6, 0.3), 0.0);
        for (k, (label, yaw, y)) in ARMS.iter().enumerate() {
            let t = at(p + Vec3::Y * *y, *yaw);
            let sw = b.sheets.sheet(&format!("link:default:{label}")).w;
            let w = sw + 40.0;
            // Armen: en planke med spiss ende.
            let mut arm = Mesh::default();
            let plank = Mat::plain(0xf2efe8, 0.6).flags(flag::NO_DIRT);
            arm.cuboid(&Affine3A::from_translation(Vec3::new(w * PX * 0.5, 0.0, 0.0)), Vec3::new(w * PX, 0.26, 0.03), &plank);
            arm.cuboid(&(Affine3A::from_translation(Vec3::new(w * PX + 0.092, 0.0, 0.0)) * Affine3A::from_rotation_z(0.785)), Vec3::new(0.184, 0.184, 0.03), &plank);
            arm.transform(&(t * Affine3A::from_translation(Vec3::new(0.06, 0.0, 0.0))));
            let collider = Some((t.transform_point3(Vec3::new(0.06 + w * PX * 0.5, 0.0, 0.0)), Vec3::new(w * PX * 0.5, 0.13, 0.05)));
            self.arms[k] = b.prop("link", arm, Affine3A::IDENTITY, collider);
            // Teksten på hver side, i alle tre tilstandene.
            for (s, side) in [1.0f32, -1.0].into_iter().enumerate() {
                for (v, name) in VARIANTS.iter().enumerate() {
                    let mut txt = Mesh::default();
                    signs::sheet(&mut txt, b.sheets, &format!("link:{name}:{label}"), Vec3::ZERO, 0.01, 0.0, slot::NEUTRAL_CANVAS);
                    let rest = t * Affine3A::from_translation(Vec3::new(0.06 + w * PX * 0.5, 0.0, side * 0.02)) * Affine3A::from_rotation_y(if side < 0.0 { std::f32::consts::PI } else { 0.0 });
                    self.labels[k][s][v] = b.prop("link_label", txt, rest, None);
                }
            }
        }
    }

    fn usable(&self) -> bool {
        true
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        let Some(k) = self.arms.iter().position(|a| Some(*a) == env.aimed_prop) else { return };
        if VARIANTS[self.variant] == "disabled" {
            env.toasts.push("Lenka er deaktivert".into());
            return;
        }
        let (feet, yaw) = TARGETS[k];
        env.teleport = Some((feet, yaw));
        env.toasts.push(format!("Fulgte lenka «{}»", ARMS[k].0));
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        for arm in &self.labels {
            for side in arm {
                if let Some(v) = side.iter().position(|p| *p == prop) {
                    return (if v == self.variant { rest } else { rest * Affine3A::from_scale(Vec3::splat(0.0001)) }, [0.0; 4]);
                }
            }
        }
        (rest, [0.0; 4])
    }
}
