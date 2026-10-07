//! Nedgangen til Legacy-kjelleren: brønnen sørvest på torget. F på
//! brønnen klatrer deg ned i startrommet under den; stigen der nede
//! tar deg opp igjen. Begge er «drop»-teleporter (ingen flytur). Dette
//! er ingen komponent, så E gjør ingenting.

use super::{Action, Component, Env, Installation};
use crate::domain::city::dungeon::{DEPTH, WELL};
use crate::domain::city::{Builder, Shape, IRON, OAK, STONE};
use crate::domain::geometry::{kind, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec2, Vec3};

#[derive(Default)]
pub struct Cellar {
    well: usize,
    ladder: usize,
    /// Føttene nede i startrommet, og ved brønnkanten oppe.
    down: Vec3,
    up: Vec3,
    aimed_ladder: bool,
}

impl Installation for Cellar {
    fn component(&self) -> Component {
        Component { name: "cellar", class: "" }
    }
    fn usable(&self) -> bool {
        true
    }
    fn inspectable(&self) -> bool {
        false
    }
    fn label(&self) -> &'static str {
        if self.aimed_ladder { "Stigen opp" } else { "Brønnen" }
    }
    fn verb(&self) -> &'static str {
        if self.aimed_ladder { "Klatre opp" } else { "Klatre ned" }
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(WELL);
        self.up = p + Vec3::new(1.9, 0.0, 0.0);
        self.down = b.dungeon.as_ref().map(|d| d.start).unwrap_or(Vec3::new(WELL.x, DEPTH, WELL.z));
        // Brønnen: steinring med vann, to stolper og et lite tak.
        let m = &mut b.mesh;
        m.lathe(&Affine3A::from_translation(p), &[Vec2::new(1.1, 0.0), Vec2::new(1.1, 0.9), Vec2::new(0.85, 0.95), Vec2::new(0.85, 0.3)], 20, &STONE, false, 0.7);
        let water = Mat::special(kind::WATER, 0x203040, 0.02);
        m.cylinder(&Affine3A::from_translation(p + Vec3::Y * 0.6), 0.86, 0.86, 0.01, 20, &water, true);
        for sx in [-0.9, 0.9] {
            m.cuboid(&Affine3A::from_translation(p + Vec3::new(sx, 1.4, 0.0)), Vec3::new(0.12, 1.6, 0.12), &OAK);
        }
        m.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), p + Vec3::new(1.0, 1.75, 0.0)), 0.06, 0.06, 2.0, 8, &OAK, true);
        m.gable_roof(&Affine3A::from_translation(p + Vec3::Y * 2.1), 2.4, 1.6, 0.6, 0.2, &Mat::tex(Layer::Tiles, 0xffffff), &OAK, &OAK);
        b.solid(Shape::Cylinder { center: p + Vec3::Y * 0.5, radius: 1.15, half_height: 0.5 });
        // Vinsjen med tauet ned: det du sikter på.
        let mut winch = Mesh::default();
        winch.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(0.0, 1.75, 0.0)), 0.14, 0.14, 0.5, 10, &OAK, true);
        winch.cylinder(&Affine3A::from_translation(Vec3::new(0.0, 1.2, 0.0)), 0.012, 0.012, 1.1, 6, &Mat::plain(0x8a7a5a, 0.9), false);
        winch.cylinder(&Affine3A::from_translation(Vec3::new(0.0, 0.7, 0.0)), 0.16, 0.14, 0.22, 10, &IRON, true);
        self.well = b.prop("well", winch, Affine3A::from_translation(p), Some((Vec3::new(0.0, 1.2, 0.0), Vec3::new(0.5, 0.7, 0.5))));
        // Stigen i startrommet nede: to vanger og trinn, inntil veggen i nord.
        let foot = self.down + Vec3::new(0.0, 0.0, -(1.5 * crate::domain::city::dungeon::CELL - 0.3));
        let mut ladder = Mesh::default();
        for sx in [-0.25, 0.25] {
            ladder.cylinder(&Affine3A::from_translation(Vec3::new(sx, 0.0, 0.0)), 0.025, 0.025, 3.1, 8, &IRON, true);
        }
        for k in 0..10 {
            ladder.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(0.0, 0.25 + k as f32 * 0.3, 0.0)), 0.014, 0.014, 0.5, 6, &IRON, true);
        }
        self.ladder = b.prop("ladder", ladder, Affine3A::from_translation(foot), Some((Vec3::new(0.0, 1.2, 0.0), Vec3::new(0.3, 1.2, 0.15))));
        b.solid_box(foot + Vec3::Y * 1.5, Vec3::new(0.6, 3.0, 0.2), 0.0);
    }

    fn tick(&mut self, env: &mut Env) {
        self.aimed_ladder = env.aimed_prop == Some(self.ladder);
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        if env.aimed_prop == Some(self.ladder) {
            env.drop = Some((self.up, std::f32::consts::FRAC_PI_2));
            env.toasts.push("Opp i dagslyset igjen".into());
        } else {
            env.drop = Some((self.down, 0.0));
            env.toasts.push("Ned i Legacy-kjelleren. Venstre mus hugger (hold for tungt), høyre mus blokker, Shift og retning er unnamanøver.".into());
        }
    }
}
