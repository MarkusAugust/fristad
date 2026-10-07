//! `.fs-input`: e-postfeltet på søknadsskjemaet. F gir feltet tastaturfokus,
//! så du skriver adressen selv; Enter eller F et annet sted avslutter. R
//! bytter tilstand (standard, suksess, ugyldig, deaktivert) fram til
//! «Send søknad» er trykket; da følger tilstanden valideringen.

use super::form::{self, COL_W, ERROR_TOPS, FIELD_H, INPUT_TOP, LEFT};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "success", "invalid", "disabled"];
const PLACEHOLDER: &str = "navn@eksempel.no";
const MAX: usize = 40;

#[derive(Default)]
pub struct Input {
    variant: usize,
    value: String,
    editing: bool,
    /// Bredden på det som er skrevet, i px, til markøren.
    width: f32,
    field: usize,
    placeholder: usize,
    text: usize,
    caret: usize,
    error: usize,
}

impl Input {
    /// Bygg teksten på nytt etter en endring.
    fn rebuild(&mut self, env: &mut Env) {
        let Some(font) = env.font.clone() else { return };
        let mut m = Mesh::default();
        signs::text_left(&mut m, &font, &self.value, baseline(), 16.0, INPUT_TEXT);
        self.width = font.width(&self.value, 16.0);
        env.dirty.push((self.text, m));
    }

    fn stop(&mut self, env: &mut Env) {
        if self.editing {
            self.editing = false;
            env.edit = Some(false);
        }
    }
}

/// Padding 8/12: teksten starter 12 px inn, grunnlinja midt i feltet.
fn text_x() -> f32 {
    -COL_W * 0.5 + 12.0
}
fn baseline() -> Vec3 {
    Vec3::new(text_x() * PX, -0.36 * 16.0 * PX, 0.025)
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
        let mut ph = Mesh::default();
        signs::text_left(&mut ph, b.font, PLACEHOLDER, baseline(), 16.0, slot::NEUTRAL_TEXT_SUBTLE);
        self.placeholder = b.prop("input_placeholder", ph, rest, None);
        // Det du skriver: tom til å begynne med, bygges om underveis.
        self.text = b.prop("input_value", Mesh::default(), rest, None);
        let mut caret = Mesh::default();
        caret.cuboid(&Affine3A::IDENTITY, Vec3::new(1.5 * PX, 20.0 * PX, 0.01), &Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_TEXT_STRONG).flags(flag::NO_DIRT | flag::NO_WEATHER));
        self.caret = b.prop("input_caret", caret, rest * Affine3A::from_translation(Vec3::new(text_x() * PX, 0.0, 0.025)), None);
        self.error = form::error_text(b, "Skriv en e-postadresse med krøllalfa og punktum.", LEFT, ERROR_TOPS[0]);
    }

    fn tick(&mut self, env: &mut Env) {
        if env.form.email != self.value {
            env.form.email = self.value.clone();
        }
        if env.form.submitted {
            self.variant = if env.form.errors[0] { 2 } else { 1 };
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        match action {
            Action::Use => {
                if self.variant == 3 {
                    env.toasts.push("Feltet er deaktivert".into());
                } else if self.editing {
                    self.stop(env);
                } else {
                    self.editing = true;
                    env.edit = Some(true);
                    env.toasts.push("Skriv e-postadressen. Enter avslutter.".into());
                }
            }
            Action::Type('\n') | Action::Type('\u{1b}') => self.stop(env),
            Action::Type('\u{8}') => {
                if self.value.pop().is_some() {
                    self.rebuild(env);
                }
            }
            Action::Type(c) => {
                if !c.is_control() && self.value.chars().count() < MAX {
                    self.value.push(c);
                    self.rebuild(env);
                }
            }
            Action::Blur => self.stop(env),
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        let show = |on: bool| if on { rest } else { rest * hidden };
        if prop == self.placeholder {
            return (show(self.value.is_empty() && !self.editing), [0.0; 4]);
        }
        if prop == self.caret {
            // Markøren står etter teksten og blinker mens feltet har fokus.
            let blink = (env.time * 2.0).fract() < 0.5;
            if !(self.editing && blink) {
                return (rest * hidden, [0.0; 4]);
            }
            return (rest * Affine3A::from_translation(Vec3::new((self.width + 1.0) * PX, 0.0, 0.0)), [0.0; 4]);
        }
        if prop == self.error {
            return (show(env.form.submitted && env.form.errors[0]), [0.0; 4]);
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
