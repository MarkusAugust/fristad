//! `.fs-textarea`: begrunnelsesfeltet på søknadsskjemaet, tre rader høyt.
//! F skriver inn begrunnelsen ord for ord, R bytter tilstand.

use super::form::{self, COL_W, LEFT, TEXTAREA_H, TEXTAREA_TOP};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "success", "invalid", "disabled"];
const PLACEHOLDER: &str = "Skriv en kort begrunnelse";
/// Teksten kommer ord for ord, med linjeskift der feltet brekker.
const STEPS: [[&str; 2]; 6] = [
    ["Jeg flytter", ""],
    ["Jeg flytter til", ""],
    ["Jeg flytter til ny", ""],
    ["Jeg flytter til ny adresse", ""],
    ["Jeg flytter til ny adresse", "1. juni."],
    ["Jeg flytter til ny adresse", "1. juni. Leiekontrakten er"],
];

#[derive(Default)]
pub struct Textarea {
    variant: usize,
    filled: bool,
    typed: f32,
    field: usize,
    placeholder: usize,
    steps: [usize; 6],
}

impl Installation for Textarea {
    fn component(&self) -> Component {
        Component { name: "textarea", class: ".fs-textarea" }
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
        form::office(b);
        let cx = LEFT + COL_W * 0.5;
        let cy = TEXTAREA_TOP - TEXTAREA_H * 0.5;
        let rest = form::on_sheet(cx, cy, 0.035);
        let mut m = Mesh::default();
        form::field(&mut m, COL_W, TEXTAREA_H, TA_FILL, TA_BORDER);
        self.field = b.prop("textarea", m, rest, Some((Vec3::ZERO, Vec3::new(COL_W * PX * 0.5, TEXTAREA_H * PX * 0.5, 0.04))));
        // Padding 8/12, linjene øverst i feltet.
        let left = -COL_W * 0.5 + 12.0;
        let top = TEXTAREA_H * 0.5 - 8.0;
        let mut ph = Mesh::default();
        signs::paragraph(&mut ph, b.font, &[PLACEHOLDER], left, top, 16.0, 1.5, 0.025, slot::NEUTRAL_TEXT_SUBTLE);
        self.placeholder = b.prop("textarea_placeholder", ph, rest, None);
        for (k, lines) in STEPS.iter().enumerate() {
            let mut tm = Mesh::default();
            let shown: Vec<&str> = lines.iter().copied().filter(|l| !l.is_empty()).collect();
            signs::paragraph(&mut tm, b.font, &shown, left, top, 16.0, 1.5, 0.025, TA_TEXT);
            self.steps[k] = b.prop("textarea_value", tm, rest, None);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        let goal = if self.filled { STEPS.len() as f32 } else { 0.0 };
        let step = env.dt * 2.5;
        self.typed = if self.typed < goal { (self.typed + step).min(goal) } else { (self.typed - step * 3.0).max(goal) };
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use && self.variant != 3 {
            self.filled = !self.filled;
            env.toasts.push(if self.filled { "Skriver begrunnelsen".into() } else { "Feltet er tømt".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        let shown = self.typed.floor() as usize;
        if prop == self.placeholder {
            return (if shown == 0 { rest } else { rest * hidden }, [0.0; 4]);
        }
        for (k, p) in self.steps.iter().enumerate() {
            if prop == *p {
                return (if shown == k + 1 { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        let (fill, border, text) = match VARIANTS[self.variant] {
            "success" => (c("success-surface"), c("success-text"), c("neutral-text-strong")),
            "invalid" => (c("danger-surface"), c("danger-text"), c("neutral-text-strong")),
            "disabled" => (c("neutral-raised"), c("neutral-raised"), c("neutral-border")),
            _ => (c("neutral-canvas"), c("neutral-border"), c("neutral-text-strong")),
        };
        p[TA_FILL as usize] = fill;
        p[TA_BORDER as usize] = border;
        p[TA_TEXT as usize] = text;
    }
}

pub const TA_FILL: u8 = slot::TA_FILL;
pub const TA_BORDER: u8 = slot::TA_BORDER;
pub const TA_TEXT: u8 = slot::TA_TEXT;
