//! `.fs-accordion`: oppslagsskapet ved Vestre gate, med to paneler som
//! kan foldes ut. F folder det øverste ut og inn, og det nederste flytter
//! seg etter, slik `<details>` gjør. R bytter variant (standard, plain).

use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{flag, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["default", "plain"];

const ITEMS: [(&str, [&str; 4]); 2] = [
    ("Hvem kan søke om bostøtte?", ["Alle som bor og er folkeregistrert i", "boligen kan søke. Du må være over", "18 år, eller ha egne barn boende", "hos deg."]),
    ("Hvor lang er behandlingstiden?", ["Vi behandler søknaden innen tre", "uker. Mangler vi vedlegg, får du", "beskjed i digital postkasse.", ""]),
];

#[derive(Default)]
pub struct Accordion {
    variant: usize,
    open: [bool; 2],
    t: [f32; 2],
    summary: [usize; 2],
    content: [usize; 2],
    chevron: [[usize; 2]; 2],
}

const POS: Vec3 = Vec3::new(-35.0, 0.0, -7.8);
const W: f32 = 340.0;
const SUMMARY_H: f32 = 12.0 + 24.0 + 12.0;
const LINE_H: f32 = 24.0;

fn content_h(k: usize) -> f32 {
    let lines = ITEMS[k].1.iter().filter(|l| !l.is_empty()).count() as f32;
    lines * LINE_H + 16.0
}

impl Installation for Accordion {
    fn component(&self) -> Component {
        Component { name: "accordion", class: ".fs-accordion" }
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
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        self.open = [false, true];
        self.t = [0.0, 1.0];
        let p = b.on_ground(POS);
        let t = at(p, 0.0);
        // Skapet: stolper, bakplate, og et lite tak i skifer.
        let m = &mut b.mesh;
        let (bw, bh) = (3.9, 2.6);
        for dx in [-1.75, 1.75] {
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, bh * 0.5 + 0.2, 0.0))), Vec3::new(0.12, bh + 0.8, 0.12), &OAK, 1.0);
        }
        let back = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.5, 0.45, 0.38));
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.7 + bh * 0.5, 0.0))), Vec3::new(bw, bh, 0.06), &back, 1.0);
        m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.7 + bh + 0.05, 0.0))), bw + 0.3, 0.6, 0.3, 0.1, &Mat::tex(Layer::Slate, 0xffffff), &OAK, &OAK);
        b.solid_box(p + Vec3::Y * (0.7 + bh * 0.5), Vec3::new(bw, bh, 0.2), 0.0);

        // Panelene: summary-plate med tittel og vinkel, og innholdet under
        // som en egen plate som foldes ut (skaleres i høyden fra toppen).
        let top_y = 0.7 + bh - 0.2;
        for k in 0..2 {
            let mut s = Mesh::default();
            signs::boxed(&mut s, Vec3::ZERO, &Box2 { w: W, h: SUMMARY_H, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(ACC_BORDER), depth: 0.03 });
            signs::paragraph(&mut s, b.font, &[ITEMS[k].0], -W * 0.5 + 16.0, SUMMARY_H * 0.5 - 12.0, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
            let rest = t * Affine3A::from_translation(Vec3::new(0.0, top_y - SUMMARY_H * PX * 0.5, 0.05));
            self.summary[k] = b.prop("accordion", s, rest, Some((Vec3::ZERO, Vec3::new(W * PX * 0.5, SUMMARY_H * PX * 0.5, 0.05))));
            // Vinkelen: to streker på 2 px som peker ned (lukket) eller opp (åpen).
            for (o, down) in [false, true].into_iter().enumerate() {
                let mut c = Mesh::default();
                let am = Mat::plain(0xffffff, 0.5).slot(slot::ACCENT_TEXT).flags(flag::NO_DIRT | flag::NO_WEATHER);
                let a = if down { 0.785 } else { -0.785 };
                for sign in [-1.0f32, 1.0] {
                    let r = Affine3A::from_rotation_z(sign * a);
                    c.cuboid(&(Affine3A::from_translation(Vec3::new(sign * 3.0 * PX, if down { 1.5 } else { -1.5 } * PX, 0.0)) * r), Vec3::new(9.0 * PX, 2.0 * PX, 0.01), &am);
                }
                self.chevron[k][o] = b.prop("accordion_chevron", c, rest * Affine3A::from_translation(Vec3::new((W * 0.5 - 16.0 - 5.0) * PX, 0.0, 0.035)), None);
            }
            let ch = content_h(k);
            let mut content = Mesh::default();
            // Innholdet henger under summary-plata: origo i overkanten.
            signs::boxed(&mut content, Vec3::new(0.0, -ch * PX * 0.5, 0.0), &Box2 { w: W, h: ch, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(ACC_BORDER), depth: 0.03 });
            let lines: Vec<&str> = ITEMS[k].1.iter().copied().filter(|l| !l.is_empty()).collect();
            signs::paragraph(&mut content, b.font, &lines, -W * 0.5 + 16.0, 0.0, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
            self.content[k] = b.prop("accordion_content", content, rest * Affine3A::from_translation(Vec3::new(0.0, -SUMMARY_H * PX * 0.5 + 1.0 * PX, -0.004)), None);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        for k in 0..2 {
            let goal = if self.open[k] { 1.0 } else { 0.0 };
            self.t[k] += (goal - self.t[k]) * (env.dt * 8.0).min(1.0);
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            self.open[0] = !self.open[0];
            env.toasts.push(if self.open[0] { "Panelet er foldet ut".into() } else { "Panelet er foldet sammen".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        // Alt under det første panelet skyves ned etter hvor åpent det er.
        let shift = |k: usize| -> f32 {
            let mut y = 0.0;
            for j in 0..k {
                y -= (SUMMARY_H + content_h(j) * self.t[j] + 8.0) * PX;
            }
            y
        };
        for k in 0..2 {
            let dy = shift(k);
            if prop == self.summary[k] {
                return (Affine3A::from_translation(Vec3::Y * dy) * rest, [0.0; 4]);
            }
            if prop == self.content[k] {
                let s = self.t[k].max(0.0001);
                return (Affine3A::from_translation(Vec3::Y * dy) * rest * Affine3A::from_scale(Vec3::new(1.0, s, 1.0)), [0.0; 4]);
            }
            let open = self.t[k] > 0.5;
            if prop == self.chevron[k][0] {
                let s = if open { 1.0 } else { 0.0001 };
                return (Affine3A::from_translation(Vec3::Y * dy) * rest * Affine3A::from_scale(Vec3::splat(s)), [0.0; 4]);
            }
            if prop == self.chevron[k][1] {
                let s = if open { 0.0001 } else { 1.0 };
                return (Affine3A::from_translation(Vec3::Y * dy) * rest * Affine3A::from_scale(Vec3::splat(s)), [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        // Plain: ingen kant på sidene og toppen. Her blir hele kanten lik fyllet.
        let c = |n: &str| tokens.blend(n, night);
        p[ACC_BORDER as usize] = if VARIANTS[self.variant] == "plain" { c("neutral-canvas") } else { c("neutral-border-subtle") };
    }
}

pub const ACC_BORDER: u8 = slot::ACC_BORDER;
