//! `.fs-card`: infotavla ved Vestre gate, med to kort slik dokumentasjonen
//! viser dem: ett med en badge, og ett fylt med en knapp. R bytter variant
//! på det øverste (standard, fylt, interaktivt).

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON, OAK};
use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "filled", "interactive"];

#[derive(Default)]
pub struct Card {
    variant: usize,
}

const POS: Vec3 = Vec3::new(-35.0, 0.0, 7.8);
const YAW: f32 = std::f32::consts::PI;

impl Installation for Card {
    fn component(&self) -> Component {
        Component { name: "card", class: ".fs-card" }
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
        // Tavla: to stolper i jern, en plate i mørkt tre, og et lite tak.
        let m = &mut b.mesh;
        let (bw, bh) = (3.6, 3.7);
        for dx in [-1.6, 1.6] {
            m.cylinder(&(t * Affine3A::from_translation(Vec3::new(dx, -0.3, 0.0))), 0.05, 0.05, bh + 0.5, 10, &IRON, false);
        }
        let board = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.45, 0.4, 0.33));
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.6 + bh * 0.5, 0.0))), Vec3::new(bw, bh, 0.06), &board, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.6 + bh + 0.08, 0.0))), Vec3::new(bw + 0.2, 0.1, 0.5), &OAK, 1.0);
        b.solid_box(p + Vec3::Y * (0.6 + bh * 0.5), Vec3::new(bw, bh, 0.2), YAW);

        // Kort 1: tittel, tekst og badge «Innvilget».
        let w = 300.0;
        let (pad, gap) = (16.0, 8.0);
        let title_h = 20.0 * 1.25;
        let p_h = 2.0 * 16.0 * 1.6;
        let badge_h = 14.0 * 1.4 + 4.0;
        let h1 = pad + title_h + gap + p_h + gap + badge_h + pad;
        let mut c1 = Mesh::default();
        signs::boxed(&mut c1, Vec3::ZERO, &Box2 { w, h: h1, radius: 8.0, fill: CARD_FILL, border: Some(CARD_BORDER), depth: 0.03 });
        let left = -w * 0.5 + pad;
        let mut top = h1 * 0.5 - pad;
        signs::paragraph(&mut c1, b.font, &["Søknad om bostøtte"], left, top, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= title_h + gap;
        signs::paragraph(&mut c1, b.font, &["Sendt 4. mars.", "Saksnummer 2026-0481."], left, top, 16.0, 1.6, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= p_h + gap;
        let bw_px = signs::width(b.font, "Innvilget", 14.0) / PX + 16.0;
        signs::boxed(&mut c1, Vec3::new((left + bw_px * 0.5) * PX, (top - badge_h * 0.5) * PX, 0.03), &Box2 { w: bw_px, h: badge_h, radius: 999.0, fill: slot::SUCCESS_FILL, border: None, depth: 0.02 });
        signs::text(&mut c1, b.font, "Innvilget", Vec3::new((left + bw_px * 0.5) * PX, (top - badge_h * 0.5) * PX, 0.054), 14.0, slot::NEUTRAL_CANVAS);
        let y1 = 0.6 + bh - 0.25 - h1 * PX * 0.5;
        c1.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, y1, 0.04))));
        let c1_prop = b.prop("card", c1, Affine3A::IDENTITY, Some((t.transform_point3(Vec3::new(0.0, y1, 0.04)), Vec3::new(w * PX * 0.5, h1 * PX * 0.5, 0.05))));
        let _ = c1_prop;

        // Kort 2, fylt: tittel, tekst og knappen «Last opp vedlegg».
        let h2 = pad + title_h + gap + p_h + gap + 42.0 + pad;
        let mut c2 = Mesh::default();
        signs::boxed(&mut c2, Vec3::ZERO, &Box2 { w, h: h2, radius: 8.0, fill: slot::NEUTRAL_SURFACE, border: None, depth: 0.03 });
        let mut top = h2 * 0.5 - pad;
        signs::paragraph(&mut c2, b.font, &["Søknad om barnehageplass"], left, top, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= title_h + gap;
        signs::paragraph(&mut c2, b.font, &["Vi mangler fødselsattest før", "vi kan behandle søknaden."], left, top, 16.0, 1.6, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= p_h + gap;
        let btn_w = signs::width(b.font, "Last opp vedlegg", 16.0) / PX + 34.0;
        signs::button(&mut c2, b.font, "Last opp vedlegg", Vec3::new((left + btn_w * 0.5) * PX, (top - 21.0) * PX, 0.03), slot::NEUTRAL_CANVAS, slot::ACCENT_BORDER, slot::ACCENT_TEXT);
        let y2 = y1 - h1 * PX * 0.5 - 0.16 - h2 * PX * 0.5;
        c2.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, y2, 0.04))));
        b.mesh.append(&c2);
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        let (fill, border) = match VARIANTS[self.variant] {
            "filled" => (c("neutral-surface"), c("neutral-surface")),
            // Interaktivt kort vist i hover: aksentkant.
            "interactive" => (c("neutral-canvas"), c("accent-border")),
            _ => (c("neutral-canvas"), c("neutral-border-subtle")),
        };
        p[CARD_FILL as usize] = fill;
        p[CARD_BORDER as usize] = border;
    }

    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }
}

pub const CARD_FILL: u8 = slot::CARD_FILL;
pub const CARD_BORDER: u8 = slot::CARD_BORDER;
