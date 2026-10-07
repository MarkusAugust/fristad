//! `.fs-checkbox`: avkryssingsboksene på søknadsskjemaet. F krysser av
//! den første, R bytter tilstand (standard, ugyldig, deaktivert, ubestemt).
//! Boksene er ark fra Fristils CSS, ett per tilstand og av/på.

use super::form::{self, CHECK_ROWS, ERROR_TOPS, RIGHT};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "invalid", "disabled", "indeterminate"];
const SIZE: f32 = 20.0;

#[derive(Default)]
pub struct Checkbox {
    variant: usize,
    checked: bool,
    /// [tilstand default/invalid/disabled][av, på], og den ubestemte.
    boxes: [[usize; 2]; 3],
    dash: usize,
    error: usize,
}

impl Installation for Checkbox {
    fn component(&self) -> Component {
        Component { name: "checkbox", class: ".fs-checkbox" }
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
        let cx = RIGHT + SIZE * 0.5;
        let rest = form::on_sheet(cx, CHECK_ROWS[0] - 12.0, 0.035);
        let collider = Some((Vec3::ZERO, Vec3::new(SIZE * PX * 0.5 + 0.02, SIZE * PX * 0.5 + 0.02, 0.04)));
        for (v, name) in ["default", "invalid", "disabled"].iter().enumerate() {
            for (s, state) in ["off", "on"].iter().enumerate() {
                let mut m = Mesh::default();
                signs::sheet(&mut m, b.sheets, &format!("checkbox:{name}:{state}"), Vec3::ZERO, 0.02, 4.0, if *name == "disabled" { slot::NEUTRAL_RAISED } else { slot::NEUTRAL_CANVAS });
                self.boxes[v][s] = b.prop("checkbox", m, rest, collider);
            }
        }
        let mut ind = Mesh::default();
        signs::sheet(&mut ind, b.sheets, "checkbox:indeterminate", Vec3::ZERO, 0.02, 4.0, slot::NEUTRAL_CANVAS);
        self.dash = b.prop("checkbox", ind, rest, collider);
        self.error = form::error_text(b, "Du må godta vilkårene for å sende søknaden.", RIGHT, ERROR_TOPS[3]);
        // Den andre raden står avkrysset, som en kulisse.
        let mut row2 = Mesh::default();
        signs::sheet(&mut row2, b.sheets, "checkbox:default:on", Vec3::ZERO, 0.02, 4.0, slot::NEUTRAL_CANVAS);
        row2.transform(&form::on_sheet(cx, CHECK_ROWS[1] - 12.0, 0.035));
        b.mesh.append(&row2);
    }

    fn tick(&mut self, env: &mut Env) {
        env.form.terms = self.checked;
        if env.form.submitted && self.variant != 2 {
            self.variant = if env.form.errors[3] { 1 } else { 0 };
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use && self.variant != 2 {
            self.checked = !self.checked;
            env.toasts.push(if self.checked { "Vilkårene er godtatt".into() } else { "Vilkårene er ikke godtatt".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        let show = |on: bool| if on { rest } else { rest * hidden };
        if prop == self.error {
            return (show(env.form.submitted && env.form.errors[3]), [0.0; 4]);
        }
        // Ubestemt: egen boks til den krysses av; ellers tilstandens av/på.
        let indeterminate = self.variant == 3 && !self.checked;
        if prop == self.dash {
            return (show(indeterminate), [0.0; 4]);
        }
        let state = if self.variant == 3 { 0 } else { self.variant };
        for v in 0..3 {
            for s in 0..2 {
                if prop == self.boxes[v][s] {
                    return (show(!indeterminate && v == state && (s == 1) == self.checked), [0.0; 4]);
                }
            }
        }
        (rest, [0.0; 4])
    }
}
