//! `.fs-badge`: statusmerkene på oppslagstavla. R bytter farge på det
//! første (standard, suksess, advarsel, fare, nøytral).

use super::signs::{self, Box2};
use super::tag::{board, board_t};
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 5] = ["accent", "success", "warning", "danger", "neutral"];

#[derive(Default)]
pub struct Badge {
    variant: usize,
    badges: [usize; 4],
}

impl Installation for Badge {
    fn component(&self) -> Component {
        Component { name: "badge", class: ".fs-badge" }
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
        board(b);
        // 12px halvfet tekst, padding 2/8, linjehøyde 1,5, radius 16 (pille).
        let labels = [("Under behandling", 0), ("Innvilget", 1), ("Mangler vedlegg", 2), ("Avslått", 3)];
        let mut x = -1.0;
        for (k, (label, style_i)) in labels.iter().enumerate() {
            let tw = signs::width(b.font, label, 12.0) / PX;
            let w = tw + 16.0;
            let h = 18.0 + 4.0;
            let mut m = Mesh::default();
            let (fill, text) = if k == 0 { (BADGE_FILL, BADGE_TEXT) } else { style(*style_i) };
            signs::boxed(&mut m, Vec3::ZERO, &Box2 { w, h, radius: 16.0, fill, border: None, depth: 0.02 });
            signs::text(&mut m, b.font, label, Vec3::new(0.0, 0.0, 0.024), 12.0, text);
            let pos = board_t() * Affine3A::from_translation(Vec3::new(x + w * PX * 0.5, 1.45, 0.04));
            self.badges[k] = b.prop("badge", m, pos, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.03))));
            x += w * PX + 0.08;
        }
        // Litt tekst på tavla, som en sak.
        let mut text = Mesh::default();
        signs::text_left(&mut text, b.font, "Søknad 2026-0481", Vec3::new(-1.0, 1.02, 0.035), 14.0, slot::NEUTRAL_TEXT_STRONG);
        text.transform(&board_t());
        b.mesh.append(&text);
    }

    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        let (fill, text) = match VARIANTS[self.variant] {
            "success" => (c("success-surface"), c("success-text")),
            "warning" => (c("warning-surface"), c("warning-text")),
            "danger" => (c("danger-surface"), c("danger-text")),
            "neutral" => (c("neutral-surface"), c("neutral-text")),
            _ => (c("accent-surface"), c("accent-text")),
        };
        p[BADGE_FILL as usize] = fill;
        p[BADGE_TEXT as usize] = text;
    }
}

pub const BADGE_FILL: u8 = slot::BADGE_FILL;
pub const BADGE_TEXT: u8 = slot::BADGE_TEXT;

fn style(i: usize) -> (u8, u8) {
    match i {
        1 => (slot::SUCCESS_SURFACE, slot::SUCCESS_TEXT),
        2 => (slot::WARNING_SURFACE, slot::WARNING_TEXT),
        3 => (slot::DANGER_SURFACE, slot::DANGER_TEXT),
        _ => (slot::ACCENT_SURFACE, slot::ACCENT_TEXT),
    }
}
