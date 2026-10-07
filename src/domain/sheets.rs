//! Komponentene rendret fra Fristils egen HTML og CSS (tools/render-sheets.mjs)
//! til ett teksturatlas per tema. Indeksen sier hvor hvert ark ligger, i
//! atlaspiksler; målene i CSS-piksler er atlaspiksler delt på skalaen, og
//! i byen er 1 CSS-piksel = 1 cm.

use glam::Vec2;
use serde::Deserialize;
use std::collections::HashMap;

pub const LIGHT_PNG: &[u8] = include_bytes!("../../assets/sheets/light.png");
pub const DARK_PNG: &[u8] = include_bytes!("../../assets/sheets/dark.png");

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Deserialize)]
struct Raw {
    scale: f32,
    size: f32,
    sheets: HashMap<String, Rect>,
}

/// Et ark: uv-hjørnene i atlaset og målene i CSS-piksler.
#[derive(Clone, Copy, Debug)]
pub struct Sheet {
    pub uv0: Vec2,
    pub uv1: Vec2,
    pub w: f32,
    pub h: f32,
}

pub struct Sheets {
    raw: Raw,
}

impl Sheets {
    pub fn load() -> Self {
        let raw: Raw = serde_json::from_str(include_str!("../../assets/sheets/index.json")).expect("assets/sheets/index.json er ugyldig. Kjør `node tools/render-sheets.mjs`.");
        Self { raw }
    }

    pub fn get(&self, id: &str) -> Option<Sheet> {
        let r = self.raw.sheets.get(id)?;
        let s = self.raw.size;
        Some(Sheet { uv0: Vec2::new(r.x / s, r.y / s), uv1: Vec2::new((r.x + r.w) / s, (r.y + r.h) / s), w: r.w / self.raw.scale, h: r.h / self.raw.scale })
    }

    /// Som `get`, men sier fra i loggen og gir et tomt ark om id-en mangler.
    pub fn sheet(&self, id: &str) -> Sheet {
        self.get(id).unwrap_or_else(|| {
            log::warn!("Arket {id} finnes ikke i atlaset");
            Sheet { uv0: Vec2::ZERO, uv1: Vec2::ZERO, w: 10.0, h: 10.0 }
        })
    }
}
