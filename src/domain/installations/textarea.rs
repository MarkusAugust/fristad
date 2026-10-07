//! `.fs-textarea`: begrunnelsesfeltet på søknadsskjemaet, tre rader høyt.
//! F gir feltet tastaturfokus, så du skriver begrunnelsen selv (teksten
//! brytes til feltets bredde); Enter avslutter. R bytter tilstand fram til
//! «Send søknad» er trykket; da følger tilstanden valideringen. Feltet er
//! ark fra Fristils CSS, ett per tilstand; det du skriver legges oppå.

use super::form::{self, COL_W, ERROR_TOPS, LEFT, TEXTAREA_H, TEXTAREA_TOP};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "success", "invalid", "disabled"];
const PLACEHOLDER: &str = "Skriv en kort begrunnelse";
const MAX: usize = 160;
const LINES: usize = 3;
const LH: f32 = 24.0;

#[derive(Default)]
pub struct Textarea {
    variant: usize,
    value: String,
    editing: bool,
    /// Hvor markøren står: linje og bredde i px.
    caret_at: (usize, f32),
    fields: [usize; 4],
    placeholder: usize,
    text: usize,
    caret: usize,
    error: usize,
}

/// Padding 8/12, linjene øverst i feltet.
fn left() -> f32 {
    -COL_W * 0.5 + 12.0
}
fn top() -> f32 {
    TEXTAREA_H * 0.5 - 8.0
}

fn surface(v: &str) -> u8 {
    match v {
        "success" => slot::SUCCESS_SURFACE,
        "invalid" => slot::DANGER_SURFACE,
        "disabled" => slot::NEUTRAL_RAISED,
        _ => slot::NEUTRAL_CANVAS,
    }
}

impl Textarea {
    fn rebuild(&mut self, env: &mut Env) {
        let Some(font) = env.font.clone() else { return };
        let mut lines = signs::wrap(&font, &self.value, 16.0, COL_W - 24.0);
        if self.value.ends_with(' ') {
            // Mellomrommet på slutten hører til linja, så markøren flytter seg.
            if let Some(last) = lines.last_mut() {
                last.push(' ');
            }
        }
        // Viser de siste linjene om teksten er lengre enn feltet.
        let skip = lines.len().saturating_sub(LINES);
        let shown: Vec<&str> = lines.iter().skip(skip).map(String::as_str).collect();
        let mut m = Mesh::default();
        signs::paragraph(&mut m, &font, &shown, left(), top(), 16.0, 1.5, 0.03, TA_TEXT);
        self.caret_at = (shown.len().saturating_sub(1), shown.last().map(|l| font.width(l, 16.0)).unwrap_or(0.0));
        env.dirty.push((self.text, m));
    }

    fn stop(&mut self, env: &mut Env) {
        if self.editing {
            self.editing = false;
            env.edit = Some(false);
        }
    }
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
        for (k, v) in VARIANTS.iter().enumerate() {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("textarea:{v}"), Vec3::ZERO, 0.02, 4.0, surface(v));
            self.fields[k] = b.prop("textarea", m, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.04))));
        }
        let mut ph = Mesh::default();
        signs::paragraph(&mut ph, b.font, &[PLACEHOLDER], left(), top(), 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_SUBTLE);
        self.placeholder = b.prop("textarea_placeholder", ph, rest, None);
        self.text = b.prop("textarea_value", Mesh::default(), rest, None);
        let mut caret = Mesh::default();
        caret.cuboid(&Affine3A::IDENTITY, Vec3::new(1.5 * PX, 20.0 * PX, 0.01), &Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_TEXT_STRONG).flags(flag::NO_DIRT | flag::NO_WEATHER));
        self.caret = b.prop("textarea_caret", caret, rest, None);
        self.error = form::error_text(b, "Skriv en kort begrunnelse, minst ti tegn.", LEFT, ERROR_TOPS[2]);
    }

    fn tick(&mut self, env: &mut Env) {
        if env.form.reason != self.value {
            env.form.reason = self.value.clone();
        }
        if env.form.submitted && self.variant != 3 {
            self.variant = if env.form.errors[2] { 2 } else { 1 };
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
                    env.toasts.push("Skriv begrunnelsen. Enter avslutter.".into());
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
        if let Some(k) = self.fields.iter().position(|p| *p == prop) {
            return (show(k == self.variant), [0.0; 4]);
        }
        if prop == self.placeholder {
            return (show(self.value.is_empty() && !self.editing), [0.0; 4]);
        }
        if prop == self.caret {
            let blink = (env.time * 2.0).fract() < 0.5;
            if !(self.editing && blink) {
                return (rest * hidden, [0.0; 4]);
            }
            let (line, w) = self.caret_at;
            let x = left() + w + 1.0;
            let y = top() - line as f32 * LH - LH * 0.5;
            return (rest * Affine3A::from_translation(Vec3::new(x * PX, y * PX, 0.03)), [0.0; 4]);
        }
        if prop == self.error {
            return (show(env.form.submitted && env.form.errors[2]), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        p[TA_TEXT as usize] = if VARIANTS[self.variant] == "disabled" { c("neutral-border") } else { c("neutral-text-strong") };
    }
}

pub const TA_TEXT: u8 = slot::TA_TEXT;
