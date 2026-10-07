//! `.fs-label`: ledeteksten over fødselsnummerfeltet på vedleggsarket. R
//! bytter markering (vanlig, påkrevd med stjerne, påkrevd med tekst, valgfri,
//! deaktivert).

use super::form::{self, ANNEX_TOP, LEFT};
use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 5] = ["default", "required", "required-text", "optional", "disabled"];
const TEXT: &str = "Fødselsnummer";

#[derive(Default)]
pub struct Label {
    variant: usize,
    props: [usize; 5],
}

/// Ledetekst med markering bak: (tekst, px, farge). 14 px, linjehøyde 1,5.
pub fn label(m: &mut Mesh, font: &crate::domain::text::Font, text: &str, px: f32, color: u8, mark: Option<(&str, f32, u8)>) -> f32 {
    signs::paragraph(m, font, &[text], 0.0, 0.0, px, 1.5, 0.0, color);
    let mut w = signs::width(font, text, px) / PX;
    if let Some((mark, mpx, mc)) = mark {
        // Markeringen står på samme grunnlinje som teksten.
        let baseline = -(1.5 * px * 0.5 + 0.36 * px);
        signs::text_left(m, font, mark, Vec3::new(w * PX, baseline * PX, 0.0), mpx, mc);
        w += signs::width(font, mark, mpx) / PX;
    }
    w
}

impl Installation for Label {
    fn component(&self) -> Component {
        Component { name: "label", class: ".fs-label" }
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
                "required" => (slot::NEUTRAL_TEXT_STRONG, Some((" *", 14.0, slot::DANGER_TEXT))),
                "required-text" => (slot::NEUTRAL_TEXT_STRONG, Some((" (påkrevd)", 12.0, slot::DANGER_TEXT))),
                "optional" => (slot::NEUTRAL_TEXT_STRONG, Some((" (valgfri)", 12.0, slot::NEUTRAL_TEXT_SUBTLE))),
                "disabled" => (slot::NEUTRAL_TEXT_SUBTLE, None),
                _ => (slot::NEUTRAL_TEXT_STRONG, None),
            };
            let mut m = Mesh::default();
            let w = label(&mut m, b.font, TEXT, 14.0, color, mark);
            let rest = form::on_annex(LEFT, ANNEX_TOP, 0.035);
            self.props[k] = b.prop("label", m, rest, Some((Vec3::new(w * 0.5 * PX, -10.5 * PX, 0.0), Vec3::new(w * 0.5 * PX, 10.5 * PX, 0.03))));
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
