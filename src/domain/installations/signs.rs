//! Felles byggeklosser for komponentene: plater med avrundede hjørner og
//! tekst i Inter, i målestokken 1 px = 1 cm.

use super::PX;
use crate::domain::geometry::{flag, kind, Mat, Mesh};
use crate::domain::text::Font;
use glam::{Vec2, Vec3};

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

/// Et komponentark fra atlaset som en plate med dybde: forsiden er bildet
/// av komponenten slik CSS-en rendrer den, baksiden og kantene en flate i
/// `back`-plassen (komponentens bakgrunn). `c` er platas sentrum, forsiden
/// vender mot +Z. Returnerer målene i px.
pub fn sheet(m: &mut Mesh, sheets: &crate::domain::sheets::Sheets, id: &str, c: Vec3, depth: f32, radius_px: f32, back: u8) -> (f32, f32) {
    let s = sheets.sheet(id);
    let (w, h) = (s.w * PX, s.h * PX);
    // Kroppen stopper 2 mm bak forsiden, så bildet ikke flimrer mot den.
    let body = Mat::plain(0xffffff, 0.5).slot(back).flags(flag::NO_DIRT | flag::NO_WEATHER);
    m.rounded_slab(&glam::Affine3A::IDENTITY, c - Vec3::Z * 0.001, w - PX, h - PX, depth - 0.002, (radius_px * PX).max(0.0), &body);
    let front = Mat::special(kind::SHEET, 0xffffff, 0.45).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let z = c.z + depth * 0.5;
    let (x0, x1, y0, y1) = (c.x - w * 0.5, c.x + w * 0.5, c.y - h * 0.5, c.y + h * 0.5);
    m.quad_uv(
        &glam::Affine3A::IDENTITY,
        [Vec3::new(x0, y0, z), Vec3::new(x1, y0, z), Vec3::new(x1, y1, z), Vec3::new(x0, y1, z)],
        &front,
        [1.0; 4],
        [Vec2::new(s.uv0.x, s.uv1.y), Vec2::new(s.uv1.x, s.uv1.y), Vec2::new(s.uv1.x, s.uv0.y), Vec2::new(s.uv0.x, s.uv0.y)],
    );
    (s.w, s.h)
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

/// Bryter en tekst i linjer som ikke er bredere enn `max_px` piksler.
pub fn wrap(font: &Font, text: &str, px: f32, max_px: f32) -> Vec<String> {
    let mut lines = vec![];
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if !line.is_empty() && font.width(&candidate, px) > max_px {
            lines.push(std::mem::take(&mut line));
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
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
