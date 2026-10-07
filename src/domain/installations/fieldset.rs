//! `.fs-fieldset` og `.fs-legend`: gruppa med vedlegg på vedleggsarket. R
//! bytter tilstand på legenden (vanlig, påkrevd, ugyldig, suksess,
//! deaktivert).

use super::form::{self, ANNEX_TOP, RIGHT};
use super::label::label;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 5] = ["default", "required", "invalid", "success", "disabled"];
const LEGEND: &str = "Vedlegg";

#[derive(Default)]
pub struct Fieldset {
    variant: usize,
    props: [usize; 5],
}

impl Installation for Fieldset {
    fn component(&self) -> Component {
        Component { name: "fieldset", class: ".fs-fieldset" }
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
        form::annex(b);
        for (k, v) in VARIANTS.iter().enumerate() {
            let (color, mark) = match *v {
                "required" => (slot::NEUTRAL_TEXT_STRONG, Some((" *", 16.0, slot::DANGER_TEXT))),
                "invalid" => (slot::DANGER_TEXT, None),
                "success" => (slot::SUCCESS_TEXT, None),
                "disabled" => (slot::NEUTRAL_TEXT_SUBTLE, None),
                _ => (slot::NEUTRAL_TEXT_STRONG, None),
            };
            let mut m = Mesh::default();
            // Legenden: 16 px semibold, linjehøyde 1,5.
            let w = label(&mut m, b.font, LEGEND, 16.0, color, mark);
            let rest = form::on_annex(RIGHT, ANNEX_TOP, 0.035);
            self.props[k] = b.prop("legend", m, rest, Some((Vec3::new(w * 0.5 * PX, -12.0 * PX, 0.0), Vec3::new(w * 0.5 * PX, 12.0 * PX, 0.03))));
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        for (k, p) in self.props.iter().enumerate() {
            if prop == *p {
                return (if k == self.variant { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }
}
