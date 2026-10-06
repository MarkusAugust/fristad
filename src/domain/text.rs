//! Tekst i byen: Inter som SDF-atlas, lagt ut som firkanter med uv.
//!
//! Atlaset lages av `bake_font` og bygges inn her. Målestokken for
//! komponentene i byen er at én CSS-piksel er én centimeter, så en knapp
//! med 16px tekst og 8/16px padding blir 42 × 80 cm, lesbar fra ti meter.

use crate::domain::geometry::{kind, Mat, Mesh};
use glam::{Affine3A, Vec2, Vec3};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Copy)]
pub struct Glyph {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub xoff: f32,
    pub yoff: f32,
    pub adv: f32,
}

#[derive(Deserialize)]
pub struct Font {
    pub em: f32,
    pub spread: f32,
    pub atlas: [u32; 2],
    pub ascent: f32,
    pub descent: f32,
    pub glyphs: HashMap<String, Glyph>,
    pub kern: HashMap<String, f32>,
}

pub const ATLAS_PNG: &[u8] = include_bytes!("../../assets/font/inter.png");

impl Font {
    pub fn load() -> Self {
        serde_json::from_str(include_str!("../../assets/font/inter.json")).expect("assets/font/inter.json er ugyldig. Kjør bake_font.")
    }

    fn glyph(&self, ch: char) -> Option<&Glyph> {
        let mut buf = [0u8; 4];
        self.glyphs.get(ch.encode_utf8(&mut buf) as &str)
    }

    /// Bredden på en tekst i piksler ved skriftstørrelsen `px`.
    pub fn width(&self, text: &str, px: f32) -> f32 {
        let s = px / self.em;
        let mut w = 0.0;
        let mut prev: Option<char> = None;
        for ch in text.chars() {
            if let Some(p) = prev {
                w += self.kern.get(&format!("{p}{ch}")).copied().unwrap_or(0.0) * s;
            }
            w += self.glyph(ch).map(|g| g.adv).unwrap_or(0.0) * s;
            prev = Some(ch);
        }
        w
    }

    /// Legger ut `text` i `m` med forsiden mot +Z. `origin` er venstre ende
    /// av grunnlinja i lokale koordinater, `px` er skriftstørrelsen i
    /// piksler og `scale` er meter per piksel (0,01 for komponentene).
    /// Bokstavene ligger litt foran z = 0 så de ikke flimrer mot flaten bak.
    pub fn layout(&self, m: &mut Mesh, text: &str, origin: Vec3, px: f32, scale: f32, mat: &Mat) {
        let mat = Mat { kind: kind::TEXT, ..*mat };
        let s = px / self.em * scale;
        let (aw, ah) = (self.atlas[0] as f32, self.atlas[1] as f32);
        let mut pen = origin.x;
        let mut prev: Option<char> = None;
        for ch in text.chars() {
            if let Some(p) = prev {
                pen += self.kern.get(&format!("{p}{ch}")).copied().unwrap_or(0.0) * s;
            }
            prev = Some(ch);
            let Some(g) = self.glyph(ch) else { continue };
            if g.w > 0 {
                let x0 = pen + g.xoff * s;
                let y0 = origin.y + g.yoff * s;
                let x1 = x0 + g.w as f32 * s;
                let y1 = y0 + g.h as f32 * s;
                let u0 = g.x as f32 / aw;
                let v0 = g.y as f32 / ah;
                let u1 = (g.x + g.w) as f32 / aw;
                let v1 = (g.y + g.h) as f32 / ah;
                let z = origin.z;
                m.quad_uv(
                    &Affine3A::IDENTITY,
                    [Vec3::new(x0, y0, z), Vec3::new(x1, y0, z), Vec3::new(x1, y1, z), Vec3::new(x0, y1, z)],
                    &mat,
                    [1.0; 4],
                    [Vec2::new(u0, v1), Vec2::new(u1, v1), Vec2::new(u1, v0), Vec2::new(u0, v0)],
                );
            }
            pen += g.adv * s;
        }
    }

    /// Som `layout`, men sentrert om `center` (midt på x-høyden).
    pub fn centered(&self, m: &mut Mesh, text: &str, center: Vec3, px: f32, scale: f32, mat: &Mat) {
        let w = self.width(text, px) * scale;
        // Versalhøyden i Inter er omtrent 0,73 em; midten av den er 0,36 em over grunnlinja.
        let y = center.y - 0.36 * px * scale;
        self.layout(m, text, Vec3::new(center.x - w * 0.5, y, center.z), px, scale, mat);
    }
}
