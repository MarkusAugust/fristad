//! `.fs-select`: fylkesvelgeren på søknadsskjemaet. F åpner lista, og
//! neste F velger neste fylke og lukker den. R bytter tilstand fram til
//! «Send søknad» er trykket; da er den ugyldig til et fylke er valgt.

use super::form::{self, COL_W, ERROR_TOPS, FIELD_H, LEFT, SELECT_TOP};
use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "invalid", "disabled"];
const OPTIONS: [&str; 4] = ["Velg fylke", "Innlandet", "Vestland", "Troms"];
const ROW_H: f32 = 36.0;

#[derive(Default)]
pub struct Select {
    variant: usize,
    selected: usize,
    open: bool,
    t: f32,
    field: usize,
    values: [usize; 4],
    list: usize,
    highlight: [usize; 4],
    error: usize,
}

impl Installation for Select {
    fn component(&self) -> Component {
        Component { name: "select", class: ".fs-select" }
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
        self.selected = 0;
        self.error = form::error_text(b, "Velg fylket du bor i.", LEFT, ERROR_TOPS[1]);
        let cx = LEFT + COL_W * 0.5;
        let cy = SELECT_TOP - FIELD_H * 0.5;
        let rest = form::on_sheet(cx, cy, 0.035);
        let mut m = Mesh::default();
        form::field(&mut m, COL_W, FIELD_H, SELECT_FILL, SELECT_BORDER);
        // Pila: to streker som danner en vinkel ned, 1 rem fra høyre kant.
        let arrow = Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_TEXT_SUBTLE).flags(flag::NO_DIRT | flag::NO_WEATHER);
        let ax = COL_W * 0.5 - 16.0 - 5.0;
        for sign in [-1.0f32, 1.0] {
            let r = Affine3A::from_rotation_z(sign * 0.785);
            m.cuboid(&(Affine3A::from_translation(Vec3::new((ax + sign * 2.8) * PX, 1.0 * PX, 0.025)) * r), Vec3::new(8.0 * PX, 2.0 * PX, 0.01), &arrow);
        }
        self.field = b.prop("select", m, rest, Some((Vec3::ZERO, Vec3::new(COL_W * PX * 0.5, FIELD_H * PX * 0.5, 0.04))));
        let text_x = -COL_W * 0.5 + 12.0;
        let baseline = Vec3::new(text_x * PX, -0.36 * 16.0 * PX, 0.025);
        for (k, o) in OPTIONS.iter().enumerate() {
            let mut tm = Mesh::default();
            signs::text_left(&mut tm, b.font, o, baseline, 16.0, SELECT_TEXT);
            self.values[k] = b.prop("select_value", tm, rest, None);
        }
        // Lista som åpner seg under feltet: origo i overkanten, så den kan
        // foldes ut i høyden.
        let lh = OPTIONS.len() as f32 * ROW_H + 8.0;
        let mut list = Mesh::default();
        signs::boxed(&mut list, Vec3::new(0.0, -lh * PX * 0.5, 0.0), &Box2 { w: COL_W, h: lh, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER), depth: 0.03 });
        for (k, o) in OPTIONS.iter().enumerate() {
            let y = -4.0 - ROW_H * (k as f32 + 0.5);
            signs::text_left(&mut list, b.font, o, Vec3::new(text_x * PX, (y - 0.36 * 16.0) * PX, 0.04), 16.0, slot::NEUTRAL_TEXT_STRONG);
        }
        let list_rest = rest * Affine3A::from_translation(Vec3::new(0.0, (-FIELD_H * 0.5 - 4.0) * PX, 0.03));
        self.list = b.prop("select_list", list, list_rest, None);
        for k in 0..OPTIONS.len() {
            let mut hl = Mesh::default();
            let y = -4.0 - ROW_H * (k as f32 + 0.5);
            signs::boxed(&mut hl, Vec3::new(0.0, y * PX, 0.0), &Box2 { w: COL_W - 8.0, h: ROW_H - 4.0, radius: 4.0, fill: slot::ACCENT_SURFACE, border: None, depth: 0.02 });
            self.highlight[k] = b.prop("select_highlight", hl, list_rest * Affine3A::from_translation(Vec3::Z * 0.005), None);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        let goal = if self.open { 1.0 } else { 0.0 };
        self.t += (goal - self.t) * (env.dt * 14.0).min(1.0);
        env.form.county = self.selected;
        if env.form.submitted && self.variant != 2 {
            self.variant = if env.form.errors[1] { 1 } else { 0 };
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use || self.variant == 2 {
            return;
        }
        if self.open {
            self.selected = (self.selected + 1) % OPTIONS.len();
            self.open = false;
            env.toasts.push(format!("Valgte «{}»", OPTIONS[self.selected]));
        } else {
            self.open = true;
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        if prop == self.error {
            return (if env.form.submitted && env.form.errors[1] { rest } else { rest * hidden }, [0.0; 4]);
        }
        for (k, p) in self.values.iter().enumerate() {
            if prop == *p {
                return (if k == self.selected { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        let s = self.t.max(0.0001);
        if prop == self.list {
            return (rest * Affine3A::from_scale(Vec3::new(1.0, s, 1.0)), [0.0; 4]);
        }
        for (k, p) in self.highlight.iter().enumerate() {
            if prop == *p {
                let on = k == self.selected && self.t > 0.5;
                return (if on { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        let (fill, border, text) = match VARIANTS[self.variant] {
            "invalid" => (c("danger-surface"), c("danger-text"), c("neutral-text-strong")),
            "disabled" => (c("neutral-raised"), c("neutral-raised"), c("neutral-border")),
            _ => (c("neutral-canvas"), c("neutral-border"), c("neutral-text-strong")),
        };
        p[SELECT_FILL as usize] = fill;
        p[SELECT_BORDER as usize] = border;
        p[SELECT_TEXT as usize] = text;
    }
}

pub const SELECT_FILL: u8 = slot::SELECT_FILL;
pub const SELECT_BORDER: u8 = slot::SELECT_BORDER;
pub const SELECT_TEXT: u8 = slot::SELECT_TEXT;
