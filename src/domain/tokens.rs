//! Fristil-tokens som motoren kan bruke.
//!
//! `assets/tokens.json` lages av `tools/export-tokens.ts` fra
//! `@fristil/designsystem`, og bygges inn i binæren. Fargene oppgis i sRGB
//! som i CSS, og gjøres om til lineært lys her, fordi det er i lineært rom
//! lys kan legges sammen og ganges riktig.

use glam::Vec3;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Raw {
    fristil: String,
    light: HashMap<String, String>,
    dark: HashMap<String, String>,
    spacing: HashMap<String, f32>,
    #[serde(default)]
    font_size: HashMap<String, f32>,
    #[serde(default)]
    line_height: HashMap<String, f32>,
}

pub struct Tokens {
    pub version: String,
    light: HashMap<String, Vec3>,
    dark: HashMap<String, Vec3>,
    spacing: HashMap<String, f32>,
    font_size: HashMap<String, f32>,
    line_height: HashMap<String, f32>,
}

/// Lyst eller mørkt tema. I Fristad er lyst tema dag og mørkt tema natt.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Light,
    Dark,
}

pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn hex(s: &str) -> Vec3 {
    let s = s.trim_start_matches('#');
    let v = u32::from_str_radix(s, 16).unwrap_or(0xff00ff);
    let r = ((v >> 16) & 0xff) as f32 / 255.0;
    let g = ((v >> 8) & 0xff) as f32 / 255.0;
    let b = (v & 0xff) as f32 / 255.0;
    Vec3::new(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b))
}

impl Tokens {
    pub fn load() -> Self {
        let raw: Raw = serde_json::from_str(include_str!("../../assets/tokens.json"))
            .expect("assets/tokens.json er ugyldig. Kjør `bun run tokens`.");
        let conv = |m: HashMap<String, String>| {
            m.into_iter()
                .map(|(k, v)| (k.trim_start_matches("color-").to_string(), hex(&v)))
                .collect::<HashMap<_, _>>()
        };
        Self {
            version: raw.fristil,
            light: conv(raw.light),
            dark: conv(raw.dark),
            spacing: raw.spacing,
            font_size: raw.font_size,
            line_height: raw.line_height,
        }
    }

    /// Skriftstørrelsen i piksler, for eksempel `m` gir 16.
    pub fn font_size_px(&self, name: &str) -> f32 {
        self.font_size.get(name).copied().unwrap_or(16.0)
    }

    /// Linjehøyden som faktor, for eksempel `default` gir 1,5.
    pub fn line_height(&self, name: &str) -> f32 {
        self.line_height.get(name).copied().unwrap_or(1.5)
    }

    /// Fargen til et token, for eksempel `accent-fill`, i lineært lys.
    pub fn color(&self, theme: Theme, name: &str) -> Vec3 {
        let map = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark,
        };
        map.get(name).copied().unwrap_or_else(|| {
            log::warn!("Ukjent Fristil-token: {name}");
            Vec3::new(1.0, 0.0, 1.0)
        })
    }

    /// Fargen midt mellom temaene, der `t` er 0 for lyst og 1 for mørkt.
    pub fn blend(&self, name: &str, t: f32) -> Vec3 {
        self.color(Theme::Light, name)
            .lerp(self.color(Theme::Dark, name), t)
    }

    /// Spacing-trinnet i piksler, for eksempel `4` gir 16.
    pub fn spacing_px(&self, step: &str) -> f32 {
        self.spacing.get(step).copied().unwrap_or(16.0)
    }
}
