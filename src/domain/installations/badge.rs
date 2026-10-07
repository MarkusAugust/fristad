//! `.fs-badge`: statusmerkene på oppslagstavla, og gateskiltene i
//! gatemunningene (samme komponent, forstørret tre ganger). R bytter farge
//! på det første merket (standard, suksess, advarsel, fare, nøytral).
//! Alle er ark rendret fra Fristils CSS (`signs::sheet`).

use super::signs;
use super::tag::{board, board_t};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON};
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 5] = ["accent", "success", "warning", "danger", "neutral"];
/// Gateskiltene: sted, og retningen skiltet vender.
const STREETS: [(&str, Vec3, f32); 4] = [
    ("Nordgata", Vec3::new(-6.6, 0.0, -18.2), 0.0),
    ("Sørgata", Vec3::new(5.6, 0.0, 18.2), std::f32::consts::PI),
    ("Østre gate", Vec3::new(18.6, 0.0, -5.4), std::f32::consts::FRAC_PI_2),
    ("Vestre gate", Vec3::new(-18.6, 0.0, 5.4), -std::f32::consts::FRAC_PI_2),
];

#[derive(Default)]
pub struct Badge {
    variant: usize,
    /// Det første merket i alle fem fargene; bare den valgte vises.
    first: [usize; 5],
}

fn surface(color: &str) -> u8 {
    match color {
        "success" => slot::SUCCESS_SURFACE,
        "warning" => slot::WARNING_SURFACE,
        "danger" => slot::DANGER_SURFACE,
        "neutral" => slot::NEUTRAL_SURFACE,
        _ => slot::ACCENT_SURFACE,
    }
}

impl Installation for Badge {
    fn component(&self) -> Component {
        Component { name: "badge", class: ".fs-badge" }
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
        board(b);
        let mut x = -1.0;
        // Det første merket, i fem utgaver oppå hverandre.
        let mut w0 = 0.0;
        for (k, c) in VARIANTS.iter().enumerate() {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("badge:{c}:behandling"), Vec3::ZERO, 0.02, 16.0, surface(c));
            w0 = w;
            let pos = board_t() * Affine3A::from_translation(Vec3::new(x + w * PX * 0.5, 1.45, 0.04));
            self.first[k] = b.prop("badge", m, pos, if k == 0 { Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.03))) } else { None });
        }
        x += w0 * PX + 0.08;
        for (c, key) in [("success", "innvilget"), ("warning", "mangler"), ("danger", "avslatt")] {
            let mut m = Mesh::default();
            let (w, h) = signs::sheet(&mut m, b.sheets, &format!("badge:{c}:{key}"), Vec3::ZERO, 0.02, 16.0, surface(c));
            let pos = board_t() * Affine3A::from_translation(Vec3::new(x + w * PX * 0.5, 1.45, 0.04));
            b.prop("badge", m, pos, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.03))));
            x += w * PX + 0.08;
        }
        // Litt tekst på tavla, som en sak.
        let mut text = Mesh::default();
        signs::text_left(&mut text, b.font, "Søknad 2026-0481", Vec3::new(-1.0, 1.02, 0.035), 14.0, slot::NEUTRAL_TEXT_STRONG);
        text.transform(&board_t());
        b.mesh.append(&text);

        // Gateskiltene: en stolpe med merket i tre gangers størrelse, lesbart
        // fra begge sider.
        for (name, pos, yaw) in STREETS {
            let p = b.on_ground(pos);
            let t = at(p, yaw);
            b.mesh.cylinder(&t, 0.03, 0.03, 2.5, 8, &IRON, false);
            for (side, rot) in [(1.0f32, 0.0f32), (-1.0, std::f32::consts::PI)] {
                let mut m = Mesh::default();
                let (w, h) = signs::sheet(&mut m, b.sheets, &format!("sign:{name}"), Vec3::ZERO, 0.03, 48.0, slot::ACCENT_SURFACE);
                let rest = t * Affine3A::from_rotation_translation(glam::Quat::from_rotation_y(rot), Vec3::new(0.0, 2.35, side * 0.02));
                b.prop("badge", m, rest, if side > 0.0 { Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.04))) } else { None });
            }
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(k) = self.first.iter().position(|p| *p == prop) {
            return (if k == self.variant { rest } else { rest * Affine3A::from_scale(Vec3::splat(0.0001)) }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, _tokens: &Tokens, _night: f32, _p: &mut [Vec3; PALETTE]) {}
}
