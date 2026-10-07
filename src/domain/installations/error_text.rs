//! `.fs-error-text`: feilmeldingen under fødselsnummerfeltet på
//! vedleggsarket. R bytter mellom feil og advarsel.

use super::form::{self, ERROR_TOP, LEFT};
use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["default", "warning"];
const TEXTS: [&str; 2] = ["Skriv fødselsnummeret med 11 siffer, uten mellomrom.", "Nummeret finnes ikke i folkeregisteret. Sjekk at det stemmer."];

#[derive(Default)]
pub struct ErrorText {
    variant: usize,
    props: [usize; 2],
}

impl Installation for ErrorText {
    fn component(&self) -> Component {
        Component { name: "error-text", class: ".fs-error-text" }
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
        let colors = [slot::DANGER_TEXT, slot::WARNING_TEXT];
        for k in 0..2 {
            let mut m = Mesh::default();
            let h = signs::paragraph(&mut m, b.font, &[TEXTS[k]], 0.0, 0.0, 12.0, 1.4, 0.0, colors[k]);
            let w = signs::width(b.font, TEXTS[k], 12.0) / PX;
            let rest = form::on_annex(LEFT, ERROR_TOP, 0.035);
            self.props[k] = b.prop("error_text", m, rest, Some((Vec3::new(w * 0.5 * PX, -h * 0.5 * PX, 0.0), Vec3::new(w * 0.5 * PX, h * 0.5 * PX, 0.03))));
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
