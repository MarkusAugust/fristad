//! `.fs-help-text`: hjelpeteksten under fødselsnummerfeltet på
//! vedleggsarket. R bytter variant (dempet, sterk, suksess, advarsel).

use super::form::{self, HELP_TOP, LEFT};
use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "strong", "success", "warning"];
const TEXTS: [&str; 4] = ["Du finner nummeret på skattekortet.", "Nummeret må stemme med folkeregisteret.", "Nummeret er kontrollert.", "Nummeret ser ut som et D-nummer."];

#[derive(Default)]
pub struct HelpText {
    variant: usize,
    props: [usize; 4],
}

impl Installation for HelpText {
    fn component(&self) -> Component {
        Component { name: "help-text", class: ".fs-help-text" }
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
        let colors = [slot::NEUTRAL_TEXT_SUBTLE, slot::NEUTRAL_TEXT_STRONG, slot::SUCCESS_TEXT, slot::WARNING_TEXT];
        for k in 0..4 {
            let mut m = Mesh::default();
            // 12 px, linjehøyde 1,4 (compact).
            let h = signs::paragraph(&mut m, b.font, &[TEXTS[k]], 0.0, 0.0, 12.0, 1.4, 0.0, colors[k]);
            let w = signs::width(b.font, TEXTS[k], 12.0) / PX;
            let rest = form::on_annex(LEFT, HELP_TOP, 0.035);
            self.props[k] = b.prop("help_text", m, rest, Some((Vec3::new(w * 0.5 * PX, -h * 0.5 * PX, 0.0), Vec3::new(w * 0.5 * PX, h * 0.5 * PX, 0.03))));
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
