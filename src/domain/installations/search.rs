//! `.fs-input.fs-search`: søkefeltet med forstørrelsesglass nederst på
//! vedleggsarket, med Søk-knappen ved siden av. R bytter tilstand
//! (standard, ugyldig, deaktivert), som på input.

use super::form::{self, FIELD_H, LEFT, SEARCH_LABEL_TOP, SEARCH_TOP, SEARCH_W};
use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "invalid", "disabled"];

#[derive(Default)]
pub struct Search {
    variant: usize,
    fields: [usize; 3],
}

/// Forstørrelsesglasset fra `--fs-icon-search`: 20 px, sirkel med radius 7
/// og et håndtak, strek 2, i en dempet grå (snur med temaet).
fn icon(m: &mut Mesh, c: Vec3, color: u8) {
    let s = 20.0 / 24.0 * PX;
    let mat = Mat::plain(0xffffff, 0.6).slot(color).flags(flag::NO_DIRT | flag::NO_WEATHER);
    // Sirkelen med sentrum (11, 11) i et 24-rutenett (y opp).
    let cc = c + Vec3::new((11.0 - 12.0) * s, (12.0 - 11.0) * s, 0.0);
    m.torus(&Affine3A::from_translation(cc), 7.0 * s, 1.0 * s, 24, 6, &mat);
    // Håndtaket fra (16.5, 16.5) til (21, 21).
    let hc = c + Vec3::new((18.75 - 12.0) * s, (12.0 - 18.75) * s, 0.0);
    let len = (4.5f32 * 4.5 * 2.0).sqrt() * s + 2.0 * s;
    m.cuboid(&Affine3A::from_rotation_translation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4), hc), Vec3::new(len, 2.0 * s, 2.0 * s), &mat);
}

impl Installation for Search {
    fn component(&self) -> Component {
        Component { name: "search", class: ".fs-search" }
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
        form::annex(b);
        // Ledeteksten og Søk-knappen står fast.
        let mut fixed = Mesh::default();
        signs::paragraph(&mut fixed, b.font, &["Søk i vedleggene"], LEFT, SEARCH_LABEL_TOP, 14.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
        let bw = signs::width(b.font, "Søk", 16.0) / PX + 34.0;
        let cy = SEARCH_TOP - FIELD_H * 0.5;
        signs::button(&mut fixed, b.font, "Søk", Vec3::new((LEFT + SEARCH_W + 8.0 + bw * 0.5) * PX, cy * PX, 0.03), slot::ACCENT_FILL, slot::ACCENT_FILL, slot::ACCENT_CONTENT);
        fixed.transform(&form::annex_t());
        b.mesh.append(&fixed);

        let rest = form::on_annex(LEFT + SEARCH_W * 0.5, cy, 0.035);
        for (k, v) in VARIANTS.iter().enumerate() {
            let (fill, border, text, ic) = match *v {
                "invalid" => (slot::DANGER_SURFACE, slot::DANGER_TEXT, slot::NEUTRAL_TEXT_SUBTLE, slot::NEUTRAL_TEXT_SUBTLE),
                "disabled" => (slot::NEUTRAL_RAISED, slot::NEUTRAL_RAISED, slot::NEUTRAL_BORDER, slot::NEUTRAL_BORDER),
                _ => (slot::NEUTRAL_CANVAS, slot::NEUTRAL_BORDER, slot::NEUTRAL_TEXT_SUBTLE, slot::NEUTRAL_TEXT_SUBTLE),
            };
            let mut m = Mesh::default();
            signs::boxed(&mut m, Vec3::ZERO, &Box2 { w: SEARCH_W, h: FIELD_H, radius: 4.0, fill, border: Some(border), depth: 0.02 });
            // Ikonet 12 px inn, teksten 40 px inn (spacing-10).
            icon(&mut m, Vec3::new((-SEARCH_W * 0.5 + 12.0 + 10.0) * PX, 0.0, 0.016), ic);
            signs::text_left(&mut m, b.font, "Søk i vedleggene", Vec3::new((-SEARCH_W * 0.5 + 40.0) * PX, -0.36 * 16.0 * PX, 0.024), 16.0, text);
            self.fields[k] = b.prop("search", m, rest, Some((Vec3::ZERO, Vec3::new(SEARCH_W * PX * 0.5, FIELD_H * PX * 0.5, 0.04))));
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        for (k, p) in self.fields.iter().enumerate() {
            if prop == *p {
                return (if k == self.variant { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }
}
