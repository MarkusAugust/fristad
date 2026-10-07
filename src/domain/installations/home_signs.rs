//! «Torget»-skiltene: lenker (`.fs-link`) på stolper utenfor torget, ved
//! Tårnene, Tokenparken, Verkstedet, søknadskontoret og infotavlene i vest.
//! F på et skilt tar deg med flytur tilbake til torget.

use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK, SPAWN, SPAWN_YAW};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

/// Skiltene: hvor de står, og hvilken vei de vender (yaw).
const SIGNS: [(Vec3, f32); 5] = [
    (Vec3::new(3.2, 0.0, -79.0), 0.0),
    (Vec3::new(-45.0, 0.0, 2.6), -std::f32::consts::FRAC_PI_2),
    (Vec3::new(10.6, 0.0, 71.0), 0.0),
    (Vec3::new(30.4, 0.0, 4.6), std::f32::consts::FRAC_PI_2),
    (Vec3::new(-30.0, 0.0, 0.0), -std::f32::consts::FRAC_PI_2),
];

#[derive(Default)]
pub struct HomeSigns {
    arms: Vec<usize>,
}

impl Installation for HomeSigns {
    fn component(&self) -> Component {
        Component { name: "link", class: ".fs-link" }
    }
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        for (pos, yaw) in SIGNS {
            let p = b.on_ground(pos);
            let t = at(p, yaw);
            let m = &mut b.mesh;
            m.cylinder(&Affine3A::from_translation(p - Vec3::Y * 0.4), 0.06, 0.05, 2.7, 8, &OAK, true);
            b.solid_box(p + Vec3::Y * 1.2, Vec3::new(0.2, 2.4, 0.2), 0.0);
            // Plakaten: canvas med kant, lenka «Torget» med understrek, og
            // avstanden under i dempet tekst.
            let dist = ((SPAWN.x - pos.x).powi(2) + (SPAWN.z - pos.z).powi(2)).sqrt();
            let label = "Torget";
            let sub = format!("{} m, F for å fly dit", (dist / 10.0).round() as i32 * 10);
            let w = (signs::width(b.font, &sub, 12.0) / PX).max(signs::width(b.font, label, 20.0) / PX) + 32.0;
            let h = 78.0;
            let mut arm = Mesh::default();
            signs::boxed(&mut arm, Vec3::ZERO, &signs::Box2 { w, h, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
            let tw = signs::width(b.font, label, 20.0) / PX;
            let lx = -w * 0.5 + 16.0;
            signs::text_left(&mut arm, b.font, label, Vec3::new(lx * PX, 10.0 * PX, 0.02), 20.0, slot::ACCENT_TEXT);
            arm.cuboid(&Affine3A::from_translation(Vec3::new((lx + tw * 0.5) * PX, (10.0 - 0.14 * 20.0 - 1.0) * PX, 0.02)), Vec3::new(tw * PX, 1.6 * PX, 0.004), &Mat::plain(0xffffff, 0.5).slot(slot::ACCENT_TEXT).flags(flag::NO_DIRT | flag::NO_WEATHER));
            signs::text_left(&mut arm, b.font, &sub, Vec3::new(lx * PX, -18.0 * PX, 0.02), 12.0, slot::NEUTRAL_TEXT_SUBTLE);
            let rest = t * Affine3A::from_translation(Vec3::new(0.0, 1.9, 0.07));
            let id = b.prop("link", arm, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.05))));
            self.arms.push(id);
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use || !self.arms.iter().any(|a| Some(*a) == env.aimed_prop) {
            return;
        }
        env.teleport = Some((SPAWN, SPAWN_YAW));
        env.toasts.push("Flyr til Torget".into());
    }
}
