//! De rustne portene i Legacy-kjelleren: et gitter i alkovens munning på
//! hver etasje, foran trappa ned. F med en nøkkel (`Env.keys`, fra
//! elitefienden i rommet) låser opp, og gitteret sveives opp i taket.
//! Ingen komponent, så E gjør ingenting.

use super::{Action, Component, Env, Installation};
use crate::domain::city::{Builder, IRON};
use crate::domain::geometry::{flag, Mat, Mesh};
use glam::{Affine3A, Quat, Vec3};

const W: f32 = 3.9;
const H: f32 = 2.9;
/// Så langt opp går gitteret når det er åpent.
const LIFT: f32 = 2.7;

#[derive(Default)]
pub struct Gate {
    props: Vec<usize>,
    /// 0 lukket, 1 oppe, per port.
    open: Vec<f32>,
    opening: Vec<bool>,
    aimed: Option<usize>,
}

impl Installation for Gate {
    fn component(&self) -> Component {
        Component { name: "gate", class: "" }
    }
    fn usable(&self) -> bool {
        true
    }
    fn inspectable(&self) -> bool {
        false
    }
    fn label(&self) -> &'static str {
        "Rusten port"
    }
    fn verb(&self) -> &'static str {
        "Lås opp"
    }

    fn build(&mut self, b: &mut Builder) {
        let gates = b.dungeon.as_ref().map(|d| d.gates.clone()).unwrap_or_default();
        let rust = IRON.tint(Vec3::new(0.32, 0.2, 0.13)).flags(flag::NO_WEATHER);
        let dark = Mat::plain(0x2a1d14, 0.8).flags(flag::NO_WEATHER);
        for (p, yaw) in gates {
            // Gitteret: loddrette stenger med spisser, to tverrstenger, i en
            // ramme som står igjen når gitteret heves.
            let mut m = Mesh::default();
            let bars = 9;
            for i in 0..bars {
                let x = -W * 0.5 + 0.25 + (W - 0.5) * i as f32 / (bars - 1) as f32;
                // Sylinderne har foten i origo.
                m.cylinder(&Affine3A::from_translation(Vec3::new(x, 0.0, 0.0)), 0.035, 0.035, H, 6, &rust, true);
                m.cylinder(&Affine3A::from_translation(Vec3::new(x, H, 0.0)), 0.04, 0.0, 0.25, 6, &rust, true);
            }
            for y in [0.5, H - 0.5] {
                m.cuboid(&Affine3A::from_translation(Vec3::new(0.0, y, 0.0)), Vec3::new(W - 0.3, 0.09, 0.06), &rust);
            }
            // Bunnstanga med rust og skitt.
            m.cuboid(&Affine3A::from_translation(Vec3::new(0.0, 0.06, 0.0)), Vec3::new(W - 0.3, 0.12, 0.09), &dark);
            let rest = Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p);
            let i = b.prop("gate", m, rest, Some((Vec3::new(0.0, H * 0.5, 0.0), Vec3::new(W * 0.5, H * 0.5, 0.12))));
            self.props.push(i);
            self.open.push(0.0);
            self.opening.push(false);
            // Rammen: to stolper og en overligger, fast i bymeshen.
            let frame = &mut b.mesh;
            for sx in [-1.0, 1.0] {
                frame.cuboid(&(rest * Affine3A::from_translation(Vec3::new(sx * W * 0.5, H * 0.5 + 0.2, 0.0))), Vec3::new(0.2, H + 0.4, 0.3), &rust);
            }
            frame.cuboid(&(rest * Affine3A::from_translation(Vec3::new(0.0, H + 0.35, 0.0))), Vec3::new(W + 0.2, 0.3, 0.3), &rust);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        self.aimed = env.aimed_prop.and_then(|p| self.props.iter().position(|q| *q == p));
        for i in 0..self.open.len() {
            if self.opening[i] {
                self.open[i] = (self.open[i] + env.dt / 1.6).min(1.0);
            }
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        let Some(i) = self.aimed else { return };
        if self.opening[i] {
            return;
        }
        if env.keys > 0 {
            env.keys -= 1;
            self.opening[i] = true;
            env.toasts.push("Nøkkelen passer. Porten sveives opp; trappa ned ligger bak.".into());
        } else {
            env.toasts.push("Porten er låst. Elitefienden i rommet bærer nøkkelen.".into());
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let Some(i) = self.props.iter().position(|p| *p == prop) else { return (rest, [0.0; 4]) };
        let t = self.open[i];
        // Rykkete heving, som en sveiv.
        let lift = t * LIFT + (t * 24.0).sin() * 0.02 * (1.0 - t);
        (rest * Affine3A::from_translation(Vec3::new(0.0, lift, 0.0)), [0.0; 4])
    }
}
