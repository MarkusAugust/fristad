//! `.fs-connection-status`: masta på taket ved torget som sier fra når
//! forbindelsen er borte. Er nettleseren offline, blir det uvær i byen.
//! V bytter vær manuelt, F her slår forbindelsen av og på for å prøve.

use super::{Action, Component, Env, Installation, WeatherKind};
use crate::domain::city::{Builder, IRON};
use crate::domain::geometry::{flag, kind, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

#[derive(Default)]
pub struct ConnectionStatus {
    online_lamp: usize,
    offline_lamp: usize,
    /// Været før forbindelsen gikk, så det kommer tilbake.
    before: Option<WeatherKind>,
    blink: f32,
}

const POS: Vec3 = Vec3::new(-12.0, 0.0, 14.6);

impl Installation for ConnectionStatus {
    fn component(&self) -> Component {
        Component { name: "connection-status", class: ".fs-connection-status" }
    }

    fn build(&mut self, b: &mut Builder) {
        // En mast med antenne og skap, og to lamper: grønn og rød.
        let m = &mut b.mesh;
        m.cuboid(&Affine3A::from_translation(POS + Vec3::Y * 0.7), Vec3::new(0.7, 1.4, 0.5), &IRON);
        m.cylinder(&Affine3A::from_translation(POS + Vec3::new(0.0, 1.4, 0.0)), 0.04, 0.03, 4.0, 8, &IRON, false);
        for k in 0..4 {
            let y = 3.6 + k as f32 * 0.4;
            m.cuboid(&Affine3A::from_translation(POS + Vec3::new(0.0, y, 0.0)), Vec3::new(1.2 - k as f32 * 0.2, 0.025, 0.025), &IRON);
        }
        b.solid_box(POS + Vec3::Y * 0.7, Vec3::new(0.7, 1.4, 0.5), 0.0);
        let lamp = |slot: u8| Mat::special(kind::LAMP, 0xffffff, 0.3).slot(slot).flags(flag::NO_WEATHER | flag::NO_DIRT);
        let mut on = Mesh::default();
        on.sphere(&Affine3A::IDENTITY, Vec3::splat(0.1), 8, 12, &lamp(slot::SUCCESS_FILL));
        let mut off = Mesh::default();
        off.sphere(&Affine3A::IDENTITY, Vec3::splat(0.1), 8, 12, &lamp(slot::DANGER_FILL));
        let at = Affine3A::from_translation(POS + Vec3::new(0.0, 1.55, 0.0));
        self.online_lamp = b.prop("connection", on, at, Some((Vec3::new(0.0, -0.8, 0.0), Vec3::new(0.4, 0.9, 0.3))));
        self.offline_lamp = b.prop("connection_off", off, at, None);
        b.light(POS + Vec3::new(0.0, 1.7, 0.3), Vec3::new(0.3, 1.0, 0.4), 0.4, 3.0);
    }

    fn tick(&mut self, env: &mut Env) {
        self.blink += env.dt;
        if !env.online {
            if self.before.is_none() {
                self.before = Some(env.weather);
                env.weather = WeatherKind::Storm;
                env.toasts.push("Forbindelsen til serveren er borte".into());
            }
        } else if let Some(w) = self.before.take() {
            env.weather = w;
            env.toasts.push("Forbindelsen er tilbake".into());
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            env.online = !env.online;
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        let hide = Affine3A::from_scale(Vec3::splat(0.0001));
        if prop == self.online_lamp {
            return (if env.online { rest } else { rest * hide }, [0.0; 4]);
        }
        if prop == self.offline_lamp {
            let blink = (self.blink * 4.0).sin() > 0.0;
            return (if !env.online && blink { rest } else { rest * hide }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
