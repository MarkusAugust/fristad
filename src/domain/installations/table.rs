//! `.fs-table`: rutetabellen i busskuret ved Østre gate. R bytter variant
//! (vanlig, stripet, hoverable med raden under musa markert).

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "striped", "hoverable"];

/// Busskuret står i (46, -4.6) og vender mot nord; bakveggen er 1,6 m inn.
const SHELTER: Vec3 = Vec3::new(46.0, 0.0, -4.6);
const ROWS: [[&str; 3]; 4] = [["Rute", "Avgang", "Pris"], ["12 Nordgata", "08:15", "42 kr"], ["14 Sørgata", "08:40", "42 kr"], ["20 Tårnene", "09:05", "58 kr"]];
const COLS: [f32; 3] = [112.0, 72.0, 72.0];
const ROW_H: f32 = 24.0 + 16.0;

#[derive(Default)]
pub struct Table {
    variant: usize,
    stripes: [usize; 2],
    hover: usize,
}

impl Installation for Table {
    fn component(&self) -> Component {
        Component { name: "table", class: ".fs-table" }
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
        let p = b.on_ground(SHELTER);
        let t = at(p, 0.0);
        let w: f32 = COLS.iter().sum();
        let caption_h = 24.0 + 8.0;
        let h = caption_h + ROW_H * ROWS.len() as f32 + 2.0;
        // Plata på bakveggen, i canvas-fargen, som arket tabellen står på.
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, 1.55, -1.6 + 0.04));
        let mut m = Mesh::default();
        signs::boxed(&mut m, Vec3::ZERO, &Box2 { w: w + 24.0, h: h + 24.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
        let left = -w * 0.5;
        let top = h * 0.5;
        let z = 0.03;
        signs::paragraph(&mut m, b.font, &["Avganger fra Torget"], left, top, 16.0, 1.5, z, slot::NEUTRAL_TEXT_STRONG);
        let line = |m: &mut Mesh, y: f32, px: f32, s: u8| {
            m.cuboid(&Affine3A::from_translation(Vec3::new(0.0, y * PX, z - 0.004)), Vec3::new(w * PX, px * PX, 0.008), &Mat::plain(0xffffff, 0.5).slot(s).flags(flag::NO_DIRT | flag::NO_WEATHER));
        };
        for (r, row) in ROWS.iter().enumerate() {
            let row_top = top - caption_h - ROW_H * r as f32;
            let mut x = left;
            for (c, cell) in row.iter().enumerate() {
                let end = c == 2;
                let cx = if end { x + COLS[c] - 12.0 - signs::width(b.font, cell, 16.0) / PX } else { x + 12.0 };
                signs::paragraph(&mut m, b.font, &[cell], cx, row_top - 8.0, 16.0, 1.5, z, slot::NEUTRAL_TEXT_STRONG);
                x += COLS[c];
            }
            // Kantlinje under: 2 px sterk under overskriften, 1 px svak ellers,
            // ingen under siste rad.
            if r == 0 {
                line(&mut m, row_top - ROW_H + 1.0, 2.0, slot::NEUTRAL_BORDER);
            } else if r + 1 < ROWS.len() {
                line(&mut m, row_top - ROW_H + 0.5, 1.0, slot::NEUTRAL_BORDER_SUBTLE);
            }
        }
        let _ = b.prop("table", m, rest, Some((Vec3::ZERO, Vec3::new((w + 24.0) * PX * 0.5, (h + 24.0) * PX * 0.5, 0.05))));
        // Stripene (partallsrader) og hover-raden som egne props bak teksten.
        for (k, r) in [2usize].iter().enumerate() {
            let row_top = top - caption_h - ROW_H * *r as f32;
            let mut s = Mesh::default();
            s.cuboid(&Affine3A::from_translation(Vec3::new(0.0, (row_top - ROW_H * 0.5) * PX, 0.0)), Vec3::new(w * PX, ROW_H * PX, 0.006), &Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_SURFACE).flags(flag::NO_DIRT | flag::NO_WEATHER));
            self.stripes[k] = b.prop("table_stripe", s, rest * Affine3A::from_translation(Vec3::Z * 0.018), None);
        }
        self.stripes[1] = self.stripes[0];
        let row_top = top - caption_h - ROW_H;
        let mut hv = Mesh::default();
        hv.cuboid(&Affine3A::from_translation(Vec3::new(0.0, (row_top - ROW_H * 0.5) * PX, 0.0)), Vec3::new(w * PX, ROW_H * PX, 0.006), &Mat::plain(0xffffff, 0.5).slot(slot::ACCENT_SURFACE).flags(flag::NO_DIRT | flag::NO_WEATHER));
        self.hover = b.prop("table_hover", hv, rest * Affine3A::from_translation(Vec3::Z * 0.018), None);
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        if prop == self.stripes[0] {
            return (if self.variant == 1 { rest } else { rest * hidden }, [0.0; 4]);
        }
        if prop == self.hover {
            return (if self.variant == 2 { rest } else { rest * hidden }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, _tokens: &Tokens, _night: f32, _p: &mut [Vec3; PALETTE]) {}
}
