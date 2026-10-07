//! `.fs-avatar`: saksbehandlerne på tavla på vestveggen av søknadskontoret.
//! Tre størrelser, en firkantet, og stabelen fra dokumentasjonen: tre små
//! som overlapper 8 px, med navnene i teksten under. R bytter form på den
//! fjerde (sirkel, firkant).

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
    signs::text(m, font, initials, c + Vec3::Z * 0.013, px, slot::ACCENT_TEXT);
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
        // Stabelen som i dokumentasjonen: `.fs-avatar-stack` med tre små
        // (24 px) som overlapper 8 px (spacing-2) og har 2 px canvas-ring.
        // Rekkefølgen snus i CSS: det første i markupen (ON) står lengst til
        // høyre, og de seinere tegnes oppå, så det venstre ligger øverst.
        let sy = -52.0;
        let left = -bw * 0.5 + 16.0;
        let stack = ["ON", "KB", "MH"];
        for (k, ini) in stack.iter().enumerate() {
            let x = left + 12.0 + (stack.len() - 1 - k) as f32 * 16.0;
            // Hver neste ligger 1,6 cm lenger fram, så ringen dekker
            // initialene på den under.
            let z = 0.03 + k as f32 * 0.016;
            signs::boxed(&mut board, Vec3::new(x * PX, sy * PX, z), &Box2 { w: 28.0, h: 28.0, radius: 14.0, fill: slot::NEUTRAL_CANVAS, border: None, depth: 0.02 });
            avatar(&mut board, b.font, Vec3::new(x * PX, sy * PX, z + 0.003), 24.0, 12.0, ini, false);
        }
        let names = signs::wrap(b.font, "Ola Nordmann, Kari Berg og Mats Hansen følger saken", 14.0, bw - 32.0);
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        signs::paragraph(&mut board, b.font, &names, left, sy - 12.0 - 8.0, 14.0, 1.5, 0.03, slot::NEUTRAL_TEXT);
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
        // Tavla er bw bred langs veggen og 10 cm tykk ut fra den (lokal z er forsiden).
        b.solid_box(p + Vec3::Y * 2.1, Vec3::new(bw * PX, bh * PX, 0.1), YAW);
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
