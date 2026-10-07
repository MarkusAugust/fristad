//! `.fs-avatar`: saksbehandlerne på tavla på vestveggen av søknadskontoret.
//! Tre størrelser, en firkantet, og en stabel. R bytter variant på den
//! første (sirkel, firkant).

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["circle", "square"];
/// Vestveggen av kontoret (36, 10.5) vender mot -X.
const POS: Vec3 = Vec3::new(31.15, 0.0, 13.0);
const YAW: f32 = -std::f32::consts::FRAC_PI_2;

#[derive(Default)]
pub struct Avatar {
    variant: usize,
    circle: usize,
    square: usize,
}

/// En avatar: sirkel eller avrundet firkant med initialer, 1 px kant innenfor.
fn avatar(m: &mut Mesh, font: &crate::domain::text::Font, c: Vec3, size: f32, px: f32, initials: &str, square: bool) {
    let r = if square { 4.0 } else { size * 0.5 };
    signs::boxed(m, c, &Box2 { w: size, h: size, radius: r, fill: slot::ACCENT_SURFACE, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.02 });
    signs::text(m, font, initials, c + Vec3::Z * 0.024, px, slot::ACCENT_TEXT);
}

impl Installation for Avatar {
    fn component(&self) -> Component {
        Component { name: "avatar", class: ".fs-avatar" }
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
        let (bw, bh) = (480.0, 200.0);
        let m = &mut b.mesh;
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 2.1, 0.0))), Vec3::new(bw * PX + 0.2, bh * PX + 0.2, 0.05), &OAK, 1.0);
        let mut board = Mesh::default();
        signs::boxed(&mut board, Vec3::ZERO, &Box2 { w: bw, h: bh, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: None, depth: 0.03 });
        signs::paragraph(&mut board, b.font, &["Saksbehandlerne"], -bw * 0.5 + 16.0, bh * 0.5 - 12.0, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
        // Rad med liten (24), standard (40), stor (64) og firkantet (40).
        let y = 10.0;
        avatar(&mut board, b.font, Vec3::new((-bw * 0.5 + 16.0 + 12.0) * PX, y * PX, 0.03), 24.0, 12.0, "ON", false);
        avatar(&mut board, b.font, Vec3::new((-bw * 0.5 + 16.0 + 24.0 + 16.0 + 20.0) * PX, y * PX, 0.03), 40.0, 14.0, "ON", false);
        avatar(&mut board, b.font, Vec3::new((-bw * 0.5 + 16.0 + 24.0 + 16.0 + 40.0 + 16.0 + 32.0) * PX, y * PX, 0.03), 64.0, 20.0, "ON", false);
        // Stabelen: tre overlappende med 2 px canvas-ring, nederst.
        let sy = -62.0;
        for (k, ini) in ["KB", "ON", "AL"].iter().enumerate().rev() {
            let x = -bw * 0.5 + 16.0 + 20.0 + k as f32 * 32.0;
            signs::boxed(&mut board, Vec3::new(x * PX, sy * PX, 0.03 + k as f32 * 0.003), &Box2 { w: 44.0, h: 44.0, radius: 22.0, fill: slot::NEUTRAL_CANVAS, border: None, depth: 0.02 });
            avatar(&mut board, b.font, Vec3::new(x * PX, sy * PX, 0.033 + k as f32 * 0.003), 40.0, 14.0, ini, false);
        }
        signs::paragraph(&mut board, b.font, &["Tre på saken"], -bw * 0.5 + 16.0 + 120.0, sy + 10.0, 14.0, 1.5, 0.03, slot::NEUTRAL_TEXT_SUBTLE);
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, 2.1, 0.03));
        let _ = b.prop("avatar_board", board, rest, Some((Vec3::ZERO, Vec3::new(bw * PX * 0.5, bh * PX * 0.5, 0.05))));
        // Den fjerde (firkantet i dokumentasjonen) bytter form med R.
        let cx = (-bw * 0.5 + 16.0 + 24.0 + 16.0 + 40.0 + 16.0 + 64.0 + 24.0 + 20.0) * PX;
        let mut circ = Mesh::default();
        avatar(&mut circ, b.font, Vec3::ZERO, 40.0, 14.0, "KB", false);
        // Litt foran tavla, så boksen ikke forsvinner inn i den.
        self.circle = b.prop("avatar", circ, rest * Affine3A::from_translation(Vec3::new(cx, y * PX, 0.02)), Some((Vec3::ZERO, Vec3::new(0.2, 0.2, 0.04))));
        let mut sq = Mesh::default();
        avatar(&mut sq, b.font, Vec3::ZERO, 40.0, 14.0, "KB", true);
        self.square = b.prop("avatar_square", sq, rest * Affine3A::from_translation(Vec3::new(cx, y * PX, 0.02)), None);
        b.solid_box(p + Vec3::Y * 2.1, Vec3::new(0.1, bh * PX, bw * PX), YAW);
        let _ = flag::NO_DIRT;
        let _ = Mat::plain(0, 0.0);
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        if prop == self.circle {
            return (if self.variant == 0 { rest } else { rest * hidden }, [0.0; 4]);
        }
        if prop == self.square {
            return (if self.variant == 1 { rest } else { rest * hidden }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, _tokens: &Tokens, _night: f32, _p: &mut [Vec3; PALETTE]) {}
}
