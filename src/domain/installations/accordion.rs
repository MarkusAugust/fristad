//! `.fs-accordion`: oppslagsskapet ved Vestre gate, med to paneler som
//! kan foldes ut hver for seg, slik to `<details>` gjør. F på et panel
//! folder det ut og inn, og det nederste flytter seg etter det øverste.
//! R bytter variant (standard, plain). Panelene er ark fra Fristils CSS:
//! ett per panel, variant og tilstand, og de henger i øyehøyde.

use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["default", "plain"];

#[derive(Default)]
pub struct Accordion {
    variant: usize,
    /// Om hvert av de to panelene er foldet ut.
    open: [bool; 2],
    /// Panelene: [panel][variant][lukket, åpen].
    panels: [[[usize; 2]; 2]; 2],
    /// Høydene (px) som styrer hvor det andre panelet står.
    heights: [[[f32; 2]; 2]; 2],
}

const POS: Vec3 = Vec3::new(-35.0, 0.0, -7.8);
/// Overkanten av det første panelet: lukket står begge i øyehøyde, og
/// foldet ut rekker det nederste ned til knehøyde.
const TOP: f32 = 2.25;

impl Installation for Accordion {
    fn component(&self) -> Component {
        Component { name: "accordion", class: ".fs-accordion" }
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
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(POS);
        let t = at(p, 0.0);
        // Skapet: stolper, bakplate, og et lite tak i skifer.
        let m = &mut b.mesh;
        let (bw, bh, base) = (4.2, 2.3, 0.2);
        for dx in [-1.9, 1.9] {
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, (base + bh + 0.3) * 0.5, -0.06))), Vec3::new(0.12, base + bh + 0.3, 0.12), &OAK, 1.0);
        }
        let back = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.5, 0.45, 0.38));
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, base + bh * 0.5, 0.0))), Vec3::new(bw, bh, 0.06), &back, 1.0);
        m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, base + bh + 0.05, 0.0))), bw + 0.3, 0.6, 0.3, 0.1, &Mat::tex(Layer::Slate, 0xffffff), &OAK, &OAK);
        // Tynn kollider: panelene foran plata må treffes før skapet.
        b.solid_box(p + Vec3::Y * (base + bh * 0.5), Vec3::new(bw, bh, 0.06), 0.0);

        // Panelene henger fra TOP: origo i overkanten av hvert ark, så det
        // andre kan flyttes ned etter høyden på det første.
        let top = t * Affine3A::from_translation(Vec3::new(0.0, TOP, 0.05));
        for (panel, prefix) in ["accordion", "accordion2"].iter().enumerate() {
            for (k, v) in VARIANTS.iter().enumerate() {
                for (s, state) in ["closed", "open"].iter().enumerate() {
                    let mut m = Mesh::default();
                    let s_id = format!("{prefix}:{v}:{state}");
                    let h = b.sheets.sheet(&s_id).h;
                    let (w, _) = signs::sheet(&mut m, b.sheets, &s_id, Vec3::new(0.0, -h * PX * 0.5, 0.0), 0.03, 4.0, slot::NEUTRAL_CANVAS);
                    self.heights[panel][k][s] = h;
                    let name = if panel == 0 { "accordion" } else { "accordion_second" };
                    self.panels[panel][k][s] = b.prop(name, m, top, Some((Vec3::new(0.0, -h * PX * 0.5, 0.0), Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.05))));
                }
            }
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        // Panelet du sikter på; uten sikte (snapshot --use) det øverste.
        let panel = env.aimed_prop.and_then(|aimed| (0..2).find(|p| self.panels[*p].iter().flatten().any(|id| *id == aimed))).unwrap_or(0);
        self.open[panel] = !self.open[panel];
        env.toasts.push(if self.open[panel] { "Panelet er foldet ut".into() } else { "Panelet er foldet sammen".into() });
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        for panel in 0..2 {
            for k in 0..2 {
                for s in 0..2 {
                    if prop != self.panels[panel][k][s] {
                        continue;
                    }
                    if k != self.variant || s != usize::from(self.open[panel]) {
                        return (rest * hidden, [0.0; 4]);
                    }
                    // Det andre panelet står under det første, med 8 px mellomrom.
                    let dy = if panel == 1 { -(self.heights[0][k][usize::from(self.open[0])] + 8.0) * PX } else { 0.0 };
                    return (rest * Affine3A::from_translation(Vec3::Y * dy), [0.0; 4]);
                }
            }
        }
        (rest, [0.0; 4])
    }
}
