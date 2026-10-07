//! `.fs-select`: fylkesvelgeren på søknadsskjemaet. F åpner lista; sikt på
//! et fylke i lista og trykk F for å velge det (raden du sikter på
//! markeres, som hover), F på feltet igjen lukker uten å velge. R bytter tilstand fram til
//! «Send søknad» er trykket; da er den ugyldig til et fylke er valgt.
//! Feltet og lista er ark fra Fristils CSS, ett felt per tilstand.

use super::form::{self, COL_W, ERROR_TOPS, FIELD_H, LEFT, SELECT_TOP};
use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "invalid", "disabled"];
const OPTIONS: [&str; 4] = ["Velg fylke", "Innlandet", "Vestland", "Troms"];
const ROW_H: f32 = 40.0;

#[derive(Default)]
pub struct Select {
    variant: usize,
    selected: usize,
    open: bool,
    t: f32,
    fields: [usize; 3],
    values: [usize; 4],
    list: usize,
    highlight: [usize; 4],
    /// Usynlige rader du kan sikte på mens lista er åpen.
    rows: [usize; 4],
    error: usize,
}

fn surface(v: &str) -> u8 {
    match v {
        "invalid" => slot::DANGER_SURFACE,
        "disabled" => slot::NEUTRAL_RAISED,
        _ => slot::NEUTRAL_CANVAS,
    }
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
        for (k, v) in VARIANTS.iter().enumerate() {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("select:{v}"), Vec3::ZERO, 0.02, 4.0, surface(v));
            self.fields[k] = b.prop("select", m, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.04))));
        }
        // Arket er et tomt felt; verdien tegnes som tekst oppå.
        let text_x = -COL_W * 0.5 + 12.0;
        let baseline = Vec3::new(text_x * PX, -0.36 * 16.0 * PX, 0.03);
        for (k, o) in OPTIONS.iter().enumerate() {
            let mut tm = Mesh::default();
            signs::text_left(&mut tm, b.font, o, baseline, 16.0, SELECT_TEXT);
            self.values[k] = b.prop("select_value", tm, rest, None);
        }
        // Lista som åpner seg under feltet: origo i overkanten, så den kan
        // foldes ut i høyden.
        let mut list = Mesh::default();
        let lh = b.sheets.sheet("select:list").h;
        signs::sheet(&mut list, b.sheets, "select:list", Vec3::new(0.0, -lh * PX * 0.5, 0.0), 0.03, 4.0, slot::NEUTRAL_CANVAS);
        let list_rest = rest * Affine3A::from_translation(Vec3::new(0.0, (-FIELD_H * 0.5 - 4.0) * PX, 0.03));
        self.list = b.prop("select_list", list, list_rest, None);
        for k in 0..OPTIONS.len() {
            let y = -4.0 - ROW_H * (k as f32 + 0.5);
            // Markeringen ligger bak teksten: et tynt lag mellom arket og teksten
            // går ikke, så den tegnes gjennomsiktig ved å legge raden foran med
            // teksten på nytt.
            let mut hl = Mesh::default();
            signs::boxed(&mut hl, Vec3::new(0.0, y * PX, 0.0), &Box2 { w: COL_W - 8.0, h: ROW_H - 4.0, radius: 4.0, fill: slot::ACCENT_SURFACE, border: None, depth: 0.02 });
            signs::text_left(&mut hl, b.font, OPTIONS[k], Vec3::new(text_x * PX, (y - 0.36 * 16.0) * PX, 0.012), 16.0, slot::NEUTRAL_TEXT_STRONG);
            self.highlight[k] = b.prop("select_highlight", hl, list_rest * Affine3A::from_translation(Vec3::Z * 0.005), None);
            self.rows[k] = b.prop("select_row", Mesh::default(), list_rest, Some((Vec3::new(0.0, y * PX, 0.0), Vec3::new(COL_W * PX * 0.5, ROW_H * PX * 0.5, 0.03))));
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
            if let Some(k) = self.rows.iter().position(|r| Some(*r) == env.aimed_prop) {
                self.selected = k;
                env.toasts.push(format!("Valgte «{}»", OPTIONS[k]));
            }
            self.open = false;
        } else {
            self.open = true;
            env.toasts.push("Sikt på et fylke og trykk F".into());
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        if let Some(k) = self.fields.iter().position(|p| *p == prop) {
            return (if k == self.variant { rest } else { rest * hidden }, [0.0; 4]);
        }
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
                // Raden du sikter på, ellers den valgte, mens lista er åpen.
                let aimed = self.rows.iter().position(|r| Some(*r) == env.aimed_prop);
                let on = self.t > 0.5 && aimed.unwrap_or(self.selected) == k;
                return (if on { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        if let Some(_) = self.rows.iter().position(|r| *r == prop) {
            // Radene kan bare siktes på mens lista er åpen; ellers flyttes de bort.
            return (if self.open { rest } else { Affine3A::from_translation(Vec3::Y * -100.0) * rest * hidden }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        p[SELECT_TEXT as usize] = if VARIANTS[self.variant] == "disabled" { c("neutral-border") } else { c("neutral-text-strong") };
    }
}

pub const SELECT_TEXT: u8 = slot::SELECT_TEXT;
