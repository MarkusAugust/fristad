//! `.fs-divider`: oppslaget på østveggen av søknadskontoret, der fire
//! avsnitt skilles med linjer: vanlig, sterk og svak. R bytter variant på
//! den første.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "strong", "subtle"];
/// Østveggen av kontoret (36, 10.5) vender mot +X.
const POS: Vec3 = Vec3::new(40.85, 0.0, 13.0);
const YAW: f32 = std::f32::consts::FRAC_PI_2;
const LINES: [&str; 4] = ["Saken er registrert.", "Du får svar innen tre uker.", "Kontakt oss hvis noe er uklart.", "Saksnummer 2026-0481."];

#[derive(Default)]
pub struct Divider {
    variant: usize,
}

impl Installation for Divider {
    fn component(&self) -> Component {
        Component { name: "divider", class: ".fs-divider" }
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
        let p = b.on_ground(POS);
        let t = at(p, YAW);
        let (bw, bh) = (360.0, 4.0 * 24.0 + 3.0 * 33.0 + 32.0);
        b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 2.1, 0.0))), Vec3::new(bw * PX + 0.2, bh * PX + 0.2, 0.05), &OAK, 1.0);
        let mut board = Mesh::default();
        signs::boxed(&mut board, Vec3::ZERO, &Box2 { w: bw, h: bh, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: None, depth: 0.03 });
        // Avsnitt på 24 px, med 16 px marg over og under hver linje (33 px i alt).
        let left = -bw * 0.5 + 16.0;
        let mut top = bh * 0.5 - 16.0;
        let slots = [DIVIDER_FIRST, slot::NEUTRAL_BORDER, slot::NEUTRAL_SURFACE];
        for (k, line) in LINES.iter().enumerate() {
            signs::paragraph(&mut board, b.font, &[line], left, top, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
            top -= 24.0;
            if k < 3 {
                let y = top - 16.0;
                board.cuboid(&Affine3A::from_translation(Vec3::new(0.0, y * PX, 0.03)), Vec3::new((bw - 32.0) * PX, 1.0 * PX, 0.006), &Mat::plain(0xffffff, 0.5).slot(slots[k]).flags(flag::NO_DIRT | flag::NO_WEATHER));
                top -= 33.0;
            }
        }
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, 2.1, 0.03));
        let _ = b.prop("divider", board, rest, Some((Vec3::ZERO, Vec3::new(bw * PX * 0.5, bh * PX * 0.5, 0.05))));
        b.solid_box(p + Vec3::Y * 2.1, Vec3::new(0.1, bh * PX, bw * PX), YAW);
    }

    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        p[DIVIDER_FIRST as usize] = match VARIANTS[self.variant] {
            "strong" => c("neutral-border-strong"),
            "subtle" => c("neutral-surface"),
            _ => c("neutral-border-subtle"),
        };
    }
}

pub const DIVIDER_FIRST: u8 = slot::DIVIDER_FIRST;
