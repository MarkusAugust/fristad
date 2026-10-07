//! `.fs-alert`: oppslaget på veggen ved porten. R bytter farge
//! (data-color: nøytral, info, suksess, advarsel, fare), og teksten følger.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{Builder, IRON};
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 5] = ["neutral", "info", "success", "warning", "danger"];
const TEXTS: [(&str, &str); 5] = [
    ("Porten er bemannet", "Vakta svarer innen tre minutter."),
    ("Torget er oppdatert", "Du må gå inn på nytt for å se endringene."),
    ("Søknaden er sendt", "Kvittering er på vei til postkassa di."),
    ("Glatt brostein", "Det har regnet. Gå forsiktig over torget."),
    ("Porten stenger ved midnatt", "Etter det må du bruke gata i sør."),
];

#[derive(Default)]
pub struct Alert {
    variant: usize,
    boards: [usize; 5],
}

const POS: Vec3 = Vec3::new(-12.5, 1.75, -17.9);

impl Installation for Alert {
    fn component(&self) -> Component {
        Component { name: "alert", class: ".fs-alert" }
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
        // Oppslaget henger på den nærmeste fasaden: husrekkene bølger, så
        // veggen står ikke nøyaktig der POS sier. Det legges ved siden av døra.
        let f = b.facades.iter().filter(|f| f.width > 5.0).min_by(|a, c| a.origin.distance(POS).total_cmp(&c.origin.distance(POS))).copied();
        let wall = match f {
            Some(f) => {
                let half = 1.7;
                let u = if f.door >= 0.0 { f.door - half - 0.7 } else { f.door + half + 0.7 };
                let u = u.clamp(-f.width * 0.5 + half + 0.3, f.width * 0.5 - half - 0.3);
                let yaw = f.facing.x.atan2(f.facing.z);
                // 22 cm ut fra veggen: skilt, vinduskarmer og sokkel stikker ut 10–15 cm.
                Affine3A::from_rotation_translation(glam::Quat::from_rotation_y(yaw), f.origin + f.along * u + f.facing * 0.22 + Vec3::Y * POS.y)
            }
            None => Affine3A::from_translation(POS),
        };
        // Festene på veggen.
        for dx in [-1.5, 1.5] {
            b.mesh.cuboid(&(wall * Affine3A::from_translation(Vec3::new(dx, 0.0, -0.12))), Vec3::new(0.06, 0.9, 0.24), &IRON);
        }
        // Ett oppslag per variant; bare det valgte vises.
        for (k, v) in VARIANTS.iter().enumerate() {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("alert:{v}"), Vec3::ZERO, 0.04, 4.0, slot::NEUTRAL_SURFACE);
            self.boards[k] = b.prop("alert", m, wall, if k == 0 { Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.06))) } else { None });
        }
        let _ = (TEXTS, colors, Box2 { w: 0.0, h: 0.0, radius: 0.0, fill: 0, border: None, depth: 0.0 });
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let k = self.boards.iter().position(|p| *p == prop).unwrap_or(0);
        if k == self.variant {
            (rest, [0.0; 4])
        } else {
            (rest * Affine3A::from_scale(Vec3::splat(0.0001)), [0.0; 4])
        }
    }

    fn palette(&self, _tokens: &Tokens, _night: f32, _p: &mut [Vec3; PALETTE]) {}
}

/// Flate, kantlinje og tekst per variant, som palettplasser.
fn colors(k: usize) -> (u8, u8, u8) {
    match VARIANTS[k] {
        // Teksten er neutral-text-strong i alle fargene; streken er
        // fargens text (accent-border for info).
        "info" => (slot::ACCENT_SURFACE, slot::ACCENT_BORDER, slot::NEUTRAL_TEXT_STRONG),
        "success" => (slot::SUCCESS_SURFACE, slot::SUCCESS_TEXT, slot::NEUTRAL_TEXT_STRONG),
        "warning" => (slot::WARNING_SURFACE, slot::WARNING_TEXT, slot::NEUTRAL_TEXT_STRONG),
        "danger" => (slot::DANGER_SURFACE, slot::DANGER_TEXT, slot::NEUTRAL_TEXT_STRONG),
        _ => (slot::NEUTRAL_SURFACE, slot::NEUTRAL_TEXT, slot::NEUTRAL_TEXT_STRONG),
    }
}
