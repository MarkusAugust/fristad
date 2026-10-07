//! Felles byggeklosser for komponentene: plater med avrundede hjørner og
//! tekst i Inter, i målestokken 1 px = 1 cm.

use super::PX;
use crate::domain::geometry::{flag, kind, Mat, Mesh};
use crate::domain::text::Font;
use glam::Vec3;

/// En boks slik CSS ser den: bredde og høyde i piksler, hjørneradius,
/// kantlinje og fyll som palettplasser.
pub struct Box2 {
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub fill: u8,
    pub border: Option<u8>,
    pub depth: f32,
}

/// Bygger boksen med forsiden mot +Z og sentrum i `c`, i meter.
pub fn boxed(m: &mut Mesh, c: Vec3, b: &Box2) {
    let w = b.w * PX;
    let h = b.h * PX;
    let r = b.radius * PX;
    let fill = Mat::plain(0xffffff, 0.45).slot(b.fill).flags(flag::NO_DIRT | flag::NO_WEATHER);
    if let Some(border) = b.border {
        let bm = Mat::plain(0xffffff, 0.45).slot(border).flags(flag::NO_DIRT | flag::NO_WEATHER);
        m.rounded_slab(&glam::Affine3A::IDENTITY, c, w, h, b.depth, r, &bm);
        m.rounded_slab(&glam::Affine3A::IDENTITY, c + Vec3::Z * 0.004, w - 2.0 * PX, h - 2.0 * PX, b.depth, (r - PX).max(0.0), &fill);
    } else {
        m.rounded_slab(&glam::Affine3A::IDENTITY, c, w, h, b.depth, r, &fill);
    }
}

/// Tekst med skriftstørrelse i piksler, sentrert om `c` (midten av
/// x-høyden), i fargen fra palettplassen.
pub fn text(m: &mut Mesh, font: &Font, s: &str, c: Vec3, px: f32, slot: u8) {
    let mat = Mat::special(kind::TEXT, 0xffffff, 0.6).slot(slot).flags(flag::NO_DIRT | flag::NO_WEATHER);
    font.centered(m, s, c, px, PX, &mat);
}

/// Venstrejustert tekst med grunnlinja i `origin`.
pub fn text_left(m: &mut Mesh, font: &Font, s: &str, origin: Vec3, px: f32, slot: u8) {
    let mat = Mat::special(kind::TEXT, 0xffffff, 0.6).slot(slot).flags(flag::NO_DIRT | flag::NO_WEATHER);
    font.layout(m, s, origin, px, PX, &mat);
}

/// Bredden på en tekst i meter.
pub fn width(font: &Font, s: &str, px: f32) -> f32 {
    font.width(s, px) * PX
}

/// Avsnitt med flere linjer, i piksler relativt til platas sentrum (y opp):
/// `left` er venstrekanten, `top` overkanten av første linjeboks, `lh` er
/// linjehøyden som faktor. Returnerer høyden i piksler.
pub fn paragraph(m: &mut Mesh, font: &Font, lines: &[&str], left: f32, top: f32, px: f32, lh: f32, z: f32, slot: u8) -> f32 {
    for (k, line) in lines.iter().enumerate() {
        let baseline = top - k as f32 * lh * px - (lh * px * 0.5 + 0.36 * px);
        text_left(m, font, line, Vec3::new(left * PX, baseline * PX, z), px, slot);
    }
    lines.len() as f32 * lh * px
}

/// En `.fs-button` som plate: 16px tekst, padding 8/16, linjehøyde 1,5 og
/// 1px kant. Sentrert i `c`. Returnerer bredden i piksler.
pub fn button(m: &mut Mesh, font: &Font, label: &str, c: Vec3, fill: u8, border: u8, text: u8) -> f32 {
    let w = width(font, label, 16.0) / PX + 34.0;
    boxed(m, c, &Box2 { w, h: 42.0, radius: 4.0, fill, border: Some(border), depth: 0.02 });
    self::text(m, font, label, c + Vec3::Z * 0.024, 16.0, text);
    w
}
