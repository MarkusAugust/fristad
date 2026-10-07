//! `.fs-tag`: merkelappene på oppslagstavla ved gata i sør. R bytter
//! variant på den første (vanlig, fylt, deaktivert).

use super::signs;
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "filled", "disabled"];

#[derive(Default)]
pub struct Tag {
    variant: usize,
    tags: [usize; 3],
}

/// Oppslagstavla står ved innkjøringen fra sør.
pub const BOARD: Vec3 = Vec3::new(13.5, 0.0, 16.0);

/// Tavla vender mot torget (nordover): lokale posisjoner går gjennom denne.
pub fn board_t() -> Affine3A {
    at(BOARD, std::f32::consts::PI)
}

/// Tavla deles med badge; den første som bygger lager den.
pub fn board(b: &mut Builder) {
    if b.props.iter().any(|p| p.name == "board") {
        return;
    }
    let t = board_t();
    let m = &mut b.mesh;
    for dx in [-1.1, 1.1] {
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(dx, 1.1, 0.0))), Vec3::new(0.12, 2.2, 0.12), &OAK);
    }
    let cork = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.7, 0.6, 0.45));
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.45, 0.0))), Vec3::new(2.4, 1.3, 0.06), &cork);
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, 2.15, 0.0))), Vec3::new(2.5, 0.1, 0.12), &OAK);
    m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, 2.2, 0.0))), 2.6, 0.5, 0.3, 0.1, &Mat::tex(Layer::Slate, 0xffffff), &OAK, &OAK);
    b.solid_box(BOARD + Vec3::Y * 1.1, Vec3::new(2.5, 2.2, 0.3), 0.0);
    // En tom prop som markerer at tavla finnes.
    b.prop("board", Mesh::default(), Affine3A::IDENTITY, None);
}

impl Installation for Tag {
    fn component(&self) -> Component {
        Component { name: "tag", class: ".fs-tag" }
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
        // Tre tagger på en rad, som ark fra CSS-en. Den første finnes i
        // alle tre variantene oppå hverandre; bare den valgte vises.
        let back = |v: &str| match v {
            "filled" => slot::NEUTRAL_SURFACE,
            "disabled" => slot::NEUTRAL_RAISED,
            _ => slot::NEUTRAL_CANVAS,
        };
        let mut x = -1.0;
        let mut w0 = 0.0;
        for (k, v) in VARIANTS.iter().enumerate() {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("tag:{v}:Bostøtte"), Vec3::ZERO, 0.02, 4.0, back(v));
            w0 = w;
            let pos = board_t() * Affine3A::from_translation(Vec3::new(x + w * PX * 0.5, 1.85, 0.04));
            self.tags[k] = b.prop("tag", m, pos, if k == 0 { Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.03))) } else { None });
        }
        x += w0 * PX + 0.08;
        for (v, label) in [("filled", "Barnehage"), ("disabled", "Arkivert")] {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("tag:{v}:{label}"), Vec3::ZERO, 0.02, 4.0, back(v));
            let pos = board_t() * Affine3A::from_translation(Vec3::new(x + w * PX * 0.5, 1.85, 0.04));
            b.prop("tag", m, pos, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.03))));
            x += w * PX + 0.08;
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(k) = self.tags.iter().position(|p| *p == prop) {
            return (if k == self.variant { rest } else { rest * Affine3A::from_scale(Vec3::splat(0.0001)) }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        // Den første taggen bytter stil med varianten via egne plasser.
        let c = |n: &str| tokens.blend(n, night);
        let (fill, border, text) = match VARIANTS[self.variant] {
            "filled" => (c("neutral-surface"), c("neutral-surface"), c("neutral-text-strong")),
            // disabled-surface er neutral-raised, disabled-text er neutral-border.
            "disabled" => (c("neutral-raised"), c("neutral-raised"), c("neutral-border")),
            _ => (c("neutral-canvas"), c("neutral-border-subtle"), c("neutral-text-strong")),
        };
        p[TAG_FILL as usize] = fill;
        p[TAG_BORDER as usize] = border;
        p[TAG_TEXT as usize] = text;
    }
}

pub const TAG_FILL: u8 = slot::TAG_FILL;
pub const TAG_BORDER: u8 = slot::TAG_BORDER;
pub const TAG_TEXT: u8 = slot::TAG_TEXT;

