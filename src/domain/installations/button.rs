//! `.fs-button`: den store knappen ved porten i nord. F trykker på den og
//! porten går opp, R bytter variant. Porten og dørene hører til her.

use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation};
use crate::domain::city::{at, Builder, IRON, OAK, STONE};
use crate::domain::geometry::{flag, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["primary", "secondary", "ghost", "danger"];
const GATE_Z: f32 = -18.0;

#[derive(Default)]
pub struct Button {
    variant: usize,
    button: usize,
    doors: [usize; 2],
    press: f32,
}

impl Installation for Button {
    fn component(&self) -> Component {
        Component { name: "button", class: ".fs-button" }
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
        let z = GATE_Z;
        // Pilarene i stein med gesims og sokkel.
        let pillar = Vec3::new(2.0, 6.2, 2.0);
        for x in [-4.5f32, 4.5] {
            let m = &mut b.mesh;
            m.cuboid(&Affine3A::from_translation(Vec3::new(x, pillar.y * 0.5, z - 1.0)), pillar, &STONE);
            m.cuboid(&Affine3A::from_translation(Vec3::new(x, pillar.y + 0.15, z - 1.0)), Vec3::new(2.4, 0.3, 2.4), &STONE);
            m.cuboid(&Affine3A::from_translation(Vec3::new(x, pillar.y + 0.4, z - 1.0)), Vec3::new(2.1, 0.2, 2.1), &STONE);
            m.sphere(&Affine3A::from_translation(Vec3::new(x, pillar.y + 0.8, z - 1.0)), Vec3::splat(0.4), 8, 12, &STONE);
            m.cuboid(&Affine3A::from_translation(Vec3::new(x, 0.25, z - 1.0)), Vec3::new(2.4, 0.5, 2.4), &STONE);
            b.solid_box(Vec3::new(x, pillar.y * 0.5, z - 1.0), pillar + Vec3::splat(0.2), 0.0);
        }
        // Overliggeren med en stein-bue over.
        b.mesh.cuboid(&Affine3A::from_translation(Vec3::new(0.0, 5.6, z - 1.0)), Vec3::new(7.2, 1.0, 1.8), &STONE);
        b.mesh.cuboid(&Affine3A::from_translation(Vec3::new(0.0, 6.25, z - 1.0)), Vec3::new(7.6, 0.3, 2.1), &STONE);
        b.solid_box(Vec3::new(0.0, 5.9, z - 1.0), Vec3::new(7.6, 1.6, 2.1), 0.0);

        // Portdørene i eik med jernbeslag, som dreier om hengslene.
        let door_w = 3.5;
        let door_h = 5.1;
        for (k, (name, hinge_x, dir)) in [("door_west", -3.5f32, 1.0f32), ("door_east", 3.5, -1.0)].into_iter().enumerate() {
            let mut m = Mesh::default();
            let cx = dir * door_w * 0.5;
            m.cuboid(&Affine3A::from_translation(Vec3::new(cx, door_h * 0.5, 0.0)), Vec3::new(door_w, door_h, 0.16), &OAK.flags(flag::ROT_UV));
            for i in 0..5 {
                let px = dir * (0.35 + i as f32 * 0.7);
                m.cuboid(&Affine3A::from_translation(Vec3::new(px, door_h * 0.5, 0.1)), Vec3::new(0.62, door_h - 0.2, 0.04), &OAK.flags(flag::ROT_UV));
            }
            for y in [0.9, door_h * 0.5, door_h - 0.9] {
                m.cuboid(&Affine3A::from_translation(Vec3::new(cx, y, 0.13)), Vec3::new(door_w - 0.2, 0.14, 0.04), &IRON);
                for i in 0..5 {
                    let px = dir * (0.35 + i as f32 * 0.7);
                    m.sphere(&Affine3A::from_translation(Vec3::new(px, y, 0.16)), Vec3::splat(0.03), 4, 6, &IRON);
                }
            }
            // Ring å dra i.
            m.torus(&(Affine3A::from_translation(Vec3::new(dir * (door_w - 0.4), 1.6, 0.2)) * Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2)), 0.12, 0.02, 14, 6, &IRON);
            self.doors[k] = b.prop(name, m, Affine3A::from_translation(Vec3::new(hinge_x, 0.0, z - 1.0)), Some((Vec3::new(cx, door_h * 0.5, 0.0), Vec3::new(door_w * 0.5, door_h * 0.5, 0.12))));
        }

        // Knappen: `.fs-button` med 16px tekst, padding 8/16, linjehøyde
        // 1,5 og 1px kant, i målestokken 1 px = 1 cm.
        let label = "Åpne porten";
        let text_w = signs::width(b.font, label, 16.0) / super::PX;
        let w = text_w + 32.0 + 2.0;
        let h = 24.0 + 16.0 + 2.0;
        let mut m = Mesh::default();
        // Radius er spacing-1 (4 px), i piksler som resten av boksen.
        signs::boxed(&mut m, Vec3::ZERO, &Box2 { w, h, radius: b.tokens.spacing_px("1"), fill: slot::BUTTON_BODY, border: Some(slot::BUTTON_RIM), depth: 0.05 });
        let label_mat = slot::BUTTON_LABEL;
        signs::text(&mut m, b.font, label, Vec3::new(0.0, 0.0, 0.045), 16.0, label_mat);
        // Plata knappen sitter på.
        let plate = Mat::tex(Layer::Metal, 0xffffff);
        let mount = Vec3::new(4.5, 1.35, z + 0.02);
        b.mesh.cuboid(&Affine3A::from_translation(mount), Vec3::new(w * super::PX + 0.16, h * super::PX + 0.16, 0.04), &plate);
        self.button = b.prop("button", m, Affine3A::from_translation(mount + Vec3::Z * 0.045), Some((Vec3::new(0.0, 0.0, 0.0), Vec3::new(w * super::PX * 0.5, h * super::PX * 0.5, 0.08))));
        let _ = at;
    }

    fn tick(&mut self, env: &mut Env) {
        self.press = (self.press - env.dt * 4.0).max(0.0);
        let goal = if env.gate_open { 1.0 } else { 0.0 };
        let step = env.dt / 2.2;
        env.gate_t = if env.gate_t < goal { (env.gate_t + step).min(goal) } else { (env.gate_t - step).max(goal) };
    }

    fn usable(&self) -> bool {
        true
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            self.press = 1.0;
            env.gate_open = !env.gate_open;
            env.toasts.push(if env.gate_open { "Porten åpnes".into() } else { "Porten lukkes".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.button {
            let push = (self.press * std::f32::consts::PI).sin().max(0.0) * 0.03;
            return (rest * Affine3A::from_translation(Vec3::new(0.0, 0.0, -push)), [0.0; 4]);
        }
        let ease = |t: f32| t * t * (3.0 - 2.0 * t);
        let angle = ease(env.gate_t) * 105f32.to_radians();
        if prop == self.doors[0] {
            return (rest * Affine3A::from_rotation_y(angle), [0.0; 4]);
        }
        if prop == self.doors[1] {
            return (rest * Affine3A::from_rotation_y(-angle), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |name: &str| tokens.blend(name, night);
        let (body, rim, label) = match VARIANTS[self.variant] {
            "secondary" => (c("neutral-canvas"), c("accent-border"), c("accent-text")),
            // Ghost: ingen bakgrunn og ingen kant, så plata viser canvas-fargen.
            "ghost" => (c("neutral-canvas"), c("neutral-canvas"), c("accent-text")),
            "danger" => (c("danger-surface"), c("danger-text"), c("danger-text")),
            _ => (c("accent-fill"), c("accent-fill"), c("accent-content")),
        };
        p[slot::BUTTON_BODY as usize] = body;
        p[slot::BUTTON_RIM as usize] = rim;
        p[slot::BUTTON_LABEL as usize] = label;
    }
}
