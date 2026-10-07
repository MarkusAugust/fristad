//! `.fs-input`: e-postfeltet på søknadsskjemaet. F skriver inn adressen
//! (og tømmer feltet igjen), R bytter tilstand (standard, suksess, ugyldig,
//! deaktivert).

use super::form::{self, COL_W, FIELD_H, INPUT_TOP, LEFT};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "success", "invalid", "disabled"];
const VALUE: &str = "ola@eksempel.no";
/// Skrivingen går ord for ord.
const STEPS: [&str; 3] = ["ola", "ola@eksempel", "ola@eksempel.no"];

#[derive(Default)]
pub struct Input {
    variant: usize,
    filled: bool,
    /// Hvor mye som er skrevet, 0..=STEPS.len().
    typed: f32,
    field: usize,
    placeholder: usize,
    steps: [usize; 3],
    caret: usize,
}

impl Installation for Input {
    fn component(&self) -> Component {
        Component { name: "input", class: ".fs-input" }
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
        let cy = INPUT_TOP - FIELD_H * 0.5;
        let rest = form::on_sheet(cx, cy, 0.035);
        let mut m = Mesh::default();
        form::field(&mut m, COL_W, FIELD_H, INPUT_FILL, INPUT_BORDER);
        self.field = b.prop("input", m, rest, Some((Vec3::ZERO, Vec3::new(COL_W * PX * 0.5, FIELD_H * PX * 0.5, 0.04))));
        // Padding 8/12: teksten starter 12 px inn, grunnlinja midt i feltet.
        let text_x = -COL_W * 0.5 + 12.0;
        let baseline = Vec3::new(text_x * PX, -0.36 * 16.0 * PX, 0.025);
        let mut ph = Mesh::default();
        signs::text_left(&mut ph, b.font, VALUE, baseline, 16.0, slot::NEUTRAL_TEXT_SUBTLE);
        self.placeholder = b.prop("input_placeholder", ph, rest, None);
        for (k, s) in STEPS.iter().enumerate() {
            let mut tm = Mesh::default();
            signs::text_left(&mut tm, b.font, s, baseline, 16.0, INPUT_TEXT);
            self.steps[k] = b.prop("input_value", tm, rest, None);
        }
        let mut caret = Mesh::default();
        caret.cuboid(&Affine3A::IDENTITY, Vec3::new(1.5 * PX, 20.0 * PX, 0.01), &Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_TEXT_STRONG).flags(flag::NO_DIRT | flag::NO_WEATHER));
        self.caret = b.prop("input_caret", caret, rest * Affine3A::from_translation(Vec3::new(text_x * PX, 0.0, 0.025)), None);
    }

    fn tick(&mut self, env: &mut Env) {
        let goal = if self.filled { STEPS.len() as f32 } else { 0.0 };
        let step = env.dt * 3.0;
        self.typed = if self.typed < goal { (self.typed + step).min(goal) } else { (self.typed - step * 3.0).max(goal) };
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use && self.variant != 3 {
            self.filled = !self.filled;
            env.toasts.push(if self.filled { "Skriver inn e-postadressen".into() } else { "Feltet er tømt".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
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
        if prop == self.caret {
            // Markøren står etter teksten og blinker mens feltet er «i fokus».
            let typing = (self.typed - (if self.filled { STEPS.len() as f32 } else { 0.0 })).abs() > 0.01;
            let blink = (env.time * 2.0).fract() < 0.5;
            if !(typing || (self.filled && blink)) {
                return (rest * hidden, [0.0; 4]);
            }
            let text = if shown == 0 { "" } else { STEPS[shown - 1] };
            let w = text.chars().count() as f32 * 8.6 + 1.0;
            return (rest * Affine3A::from_translation(Vec3::new(w * PX, 0.0, 0.0)), [0.0; 4]);
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
        p[INPUT_FILL as usize] = fill;
        p[INPUT_BORDER as usize] = border;
        p[INPUT_TEXT as usize] = text;
    }
}

pub const INPUT_FILL: u8 = slot::INPUT_FILL;
pub const INPUT_BORDER: u8 = slot::INPUT_BORDER;
pub const INPUT_TEXT: u8 = slot::INPUT_TEXT;
