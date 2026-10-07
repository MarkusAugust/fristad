//! `.fs-divider`: oppslaget på østveggen av søknadskontoret, der fire
//! avsnitt skilles med linjer. R bytter variant på den første (vanlig,
//! sterk, svak); linjene er ark fra Fristils CSS.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "strong", "subtle"];
/// Østveggen av kontoret (36, 10.5) vender mot +X.
const POS: Vec3 = Vec3::new(40.85, 0.0, 13.0);
const YAW: f32 = std::f32::consts::FRAC_PI_2;
const LINES: [&str; 4] = ["Saken er registrert.", "Du får svar innen tre uker.", "Kontakt oss hvis noe er uklart.", "Saksnummer 2026-0481."];

#[derive(Default)]
pub struct Divider {
    variant: usize,
    first: [usize; 3],
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
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, 2.1, 0.03));
        let mut line_ys = vec![];
        for (k, line) in LINES.iter().enumerate() {
            signs::paragraph(&mut board, b.font, &[line], left, top, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
            top -= 24.0;
            if k < 3 {
                line_ys.push(top - 16.0);
                top -= 33.0;
            }
        }
        // Den første linja i alle tre variantene (bare den valgte vises), de
        // to andre faste: sterk og svak, som i dokumentasjonen.
        for (k, v) in VARIANTS.iter().enumerate() {
            let mut m = Mesh::default();
            signs::sheet(&mut m, b.sheets, &format!("divider:{v}"), Vec3::ZERO, 0.01, 0.0, slot::NEUTRAL_CANVAS);
            self.first[k] = b.prop("divider_line", m, rest * Affine3A::from_translation(Vec3::new(0.0, line_ys[0] * PX, 0.03)), None);
        }
        for (k, v) in ["strong", "subtle"].iter().enumerate() {
            signs::sheet(&mut board, b.sheets, &format!("divider:{v}"), Vec3::new(0.0, line_ys[k + 1] * PX, 0.03), 0.01, 0.0, slot::NEUTRAL_CANVAS);
        }
        let _ = b.prop("divider", board, rest, Some((Vec3::ZERO, Vec3::new(bw * PX * 0.5, bh * PX * 0.5, 0.05))));
        // Tavla er bw bred langs veggen og 10 cm tykk ut fra den (lokal z er forsiden).
        b.solid_box(p + Vec3::Y * 2.1, Vec3::new(bw * PX, bh * PX, 0.1), YAW);
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(k) = self.first.iter().position(|p| *p == prop) {
            return (if k == self.variant { rest } else { rest * Affine3A::from_scale(Vec3::splat(0.0001)) }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
