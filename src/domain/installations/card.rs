//! `.fs-card`: infotavla ved Vestre gate, med to kort slik dokumentasjonen
//! viser dem: ett med en badge, og ett fylt med en knapp. R bytter variant
//! på det øverste (standard, fylt, interaktivt); kortet er ark fra
//! Fristils CSS, ett per variant.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON, OAK};
use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "filled", "interactive"];

#[derive(Default)]
pub struct Card {
    variant: usize,
    cards: [usize; 3],
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
        let (bw, bh) = (4.2, 3.9);
        for dx in [-1.9, 1.9] {
            m.cylinder(&(t * Affine3A::from_translation(Vec3::new(dx, -0.3, -0.06))), 0.05, 0.05, bh + 0.5, 10, &IRON, false);
        }
        let board = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.45, 0.4, 0.33));
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.6 + bh * 0.5, 0.0))), Vec3::new(bw, bh, 0.06), &board, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.6 + bh + 0.08, 0.0))), Vec3::new(bw + 0.2, 0.1, 0.5), &OAK, 1.0);
        b.solid_box(p + Vec3::Y * (0.6 + bh * 0.5), Vec3::new(bw, bh, 0.2), YAW);

        // Kort 1 i alle tre variantene, oppå hverandre.
        let mut h1 = 0.0;
        for (k, v) in VARIANTS.iter().enumerate() {
            let mut c1 = Mesh::default();
            let (w, h) = signs::sheet(&mut c1, b.sheets, &format!("card:{v}"), Vec3::ZERO, 0.03, 8.0, if *v == "filled" { slot::NEUTRAL_SURFACE } else { slot::NEUTRAL_CANVAS });
            h1 = h;
            let y1 = 0.6 + bh - 0.25 - h * PX * 0.5;
            let rest = t * Affine3A::from_translation(Vec3::new(0.0, y1, 0.04));
            self.cards[k] = b.prop("card", c1, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.05))));
        }
        let y1 = 0.6 + bh - 0.25 - h1 * PX * 0.5;

        // Kort 2, fylt: tittel, tekst og knappen «Last opp vedlegg».
        let w = 360.0;
        let (pad, gap) = (16.0, 8.0);
        let title_h = 20.0 * 1.25;
        let p_h = 2.0 * 16.0 * 1.6;
        let left = -w * 0.5 + pad;
        let h2 = pad + title_h + gap + p_h + gap + 42.0 + pad;
        let mut c2 = Mesh::default();
        signs::boxed(&mut c2, Vec3::ZERO, &Box2 { w, h: h2, radius: 8.0, fill: slot::NEUTRAL_SURFACE, border: None, depth: 0.03 });
        let mut top = h2 * 0.5 - pad;
        signs::paragraph(&mut c2, b.font, &["Søknad om barnehageplass"], left, top, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= title_h + gap;
        signs::paragraph(&mut c2, b.font, &["Vi mangler fødselsattest før", "vi kan behandle søknaden."], left, top, 16.0, 1.6, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= p_h + gap;
        // Knappen: arket for secondary-varianten, ved venstrekanten i kortet.
        let btn_w = b.sheets.sheet("button:upload").w;
        signs::sheet(&mut c2, b.sheets, "button:upload", Vec3::new((left + btn_w * 0.5) * PX, (top - 21.0) * PX, 0.03), 0.02, 4.0, slot::NEUTRAL_CANVAS);
        let y2 = y1 - h1 * PX * 0.5 - 0.16 - h2 * PX * 0.5;
        c2.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, y2, 0.04))));
        b.mesh.append(&c2);
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(k) = self.cards.iter().position(|p| *p == prop) {
            return (if k == self.variant { rest } else { rest * Affine3A::from_scale(Vec3::splat(0.0001)) }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
