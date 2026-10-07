//! `.fs-heading`: overskriften på vedleggsarket bak søknadskontoret. R blar
//! gjennom størrelsene (standard, mega, xl, m, s, xs i versaler).

use super::form::{self, ANNEX_TITLE_TOP, LEFT};
use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 6] = ["default", "mega", "xl", "m", "s", "xs"];
/// Tokenet hver størrelse bruker.
const SIZES: [&str; 6] = ["xl", "mega", "xxl", "l", "m", "s"];
const TITLE: &str = "Vedlegg til søknaden";

#[derive(Default)]
pub struct Heading {
    variant: usize,
    props: [usize; 6],
}

impl Installation for Heading {
    fn component(&self) -> Component {
        Component { name: "heading", class: ".fs-heading" }
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
        let upper = TITLE.to_uppercase();
        for (k, size) in SIZES.iter().enumerate() {
            let px = b.tokens.font_size_px(size);
            let text = if VARIANTS[k] == "xs" { upper.as_str() } else { TITLE };
            let mut m = Mesh::default();
            let h = signs::paragraph(&mut m, b.font, &[text], 0.0, 0.0, px, 1.2, 0.0, slot::NEUTRAL_TEXT_STRONG);
            let w = signs::width(b.font, text, px) / PX;
            let rest = form::on_annex(LEFT, ANNEX_TITLE_TOP, 0.035);
            self.props[k] = b.prop("heading", m, rest, Some((Vec3::new(w * 0.5 * PX, -h * 0.5 * PX, 0.0), Vec3::new(w * 0.5 * PX, h * 0.5 * PX, 0.03))));
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
