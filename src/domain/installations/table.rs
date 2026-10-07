//! `.fs-table`: rutetabellen på oppslagsplata ved busskuret ved Østre
//! gate. Tabellen er satt sammen av ark fra Fristils CSS: bildeteksten,
//! overskriftsraden (ett ark per sortering, med pila fra `aria-sort`) og
//! radene (ett ark per rad og bakgrunn). F på en kolonneoverskrift
//! sorterer stigende, F igjen synkende; radene flytter seg etter. R bytter
//! variant (vanlig, stripet, hoverable med raden du sikter på markert).

use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON, OAK};
use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "striped", "hoverable"];

/// Oppslagsplata står øst for busskuret (46, −4.6) og vender mot nord.
const BOARD: Vec3 = Vec3::new(51.6, 0.0, -4.6);
/// Radene: rutenummer, avgang i minutter og pris, til sorteringen.
const ROWS: [(u32, u32, u32); 3] = [(12, 8 * 60 + 15, 42), (14, 8 * 60 + 40, 42), (20, 9 * 60 + 5, 58)];
const COLS: [f32; 3] = [240.0, 120.0, 120.0];
const W: f32 = 480.0;
const BG: [&str; 3] = ["plain", "striped", "hover"];

#[derive(Default)]
pub struct Table {
    variant: usize,
    /// Sortert etter kolonne, stigende eller synkende.
    sort: Option<(usize, bool)>,
    /// Overskriftsraden per sortering: ingen, så (kolonne, stigende/synkende).
    heads: [usize; 7],
    /// Usynlige props du kan sikte på: én per kolonneoverskrift, én per rad.
    head_cols: [usize; 3],
    row_targets: [usize; 3],
    /// Radene: [rad][bakgrunn][uten/med kantlinje under].
    rows: [[[usize; 2]; 3]; 3],
    /// Overkanten av radene (px fra platas sentrum).
    rows_top: f32,
    row_h: f32,
    head_h: f32,
}

impl Table {
    /// Rekkefølgen radene vises i.
    fn order(&self) -> [usize; 3] {
        let mut idx = [0usize, 1, 2];
        if let Some((col, asc)) = self.sort {
            let key = |i: usize| match col {
                0 => ROWS[i].0,
                1 => ROWS[i].1,
                _ => ROWS[i].2,
            };
            idx.sort_by_key(|i| key(*i));
            if !asc {
                idx.reverse();
            }
        }
        idx
    }

    fn head_index(&self) -> usize {
        match self.sort {
            None => 0,
            Some((c, asc)) => 1 + c * 2 + usize::from(!asc),
        }
    }
}

impl Installation for Table {
    fn component(&self) -> Component {
        Component { name: "table", class: ".fs-table" }
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
        let p = b.on_ground(BOARD);
        let t = at(p, 0.0);
        // Målene fra arkene.
        let cap = b.sheets.sheet("table-caption");
        let head = b.sheets.sheet("table-head:none");
        let row = b.sheets.sheet("table-row:plain:0:mid");
        self.head_h = head.h;
        self.row_h = row.h;
        let h = cap.h + head.h + row.h * 3.0;
        let top = h * 0.5;
        self.rows_top = top - cap.h - head.h;
        // Plata: to stolper i jern, ei plate i eik med kant, og arket i canvas.
        let (bw, bh) = ((W + 48.0) * PX, (h + 48.0) * PX);
        let m = &mut b.mesh;
        for dx in [-bw * 0.5 + 0.2, bw * 0.5 - 0.2] {
            m.cylinder(&(t * Affine3A::from_translation(Vec3::new(dx, -0.3, -0.06))), 0.05, 0.05, 1.0 + bh + 0.3, 10, &IRON, false);
        }
        let board = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.45, 0.4, 0.33));
        let center_y = 1.0 + bh * 0.5;
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, center_y, 0.0))), Vec3::new(bw + 0.1, bh + 0.1, 0.06), &board, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.0 + bh + 0.08, 0.0))), Vec3::new(bw + 0.3, 0.1, 0.2), &OAK, 1.0);
        b.solid_box(p + Vec3::new(0.0, center_y, 0.0), Vec3::new(bw + 0.1, bh + 0.1, 0.06), 0.0);
        let rest = t * Affine3A::from_translation(Vec3::new(0.0, center_y, 0.04));
        let mut sheet = Mesh::default();
        signs::boxed(&mut sheet, Vec3::ZERO, &Box2 { w: W + 48.0, h: h + 48.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
        // Bildeteksten står fast.
        signs::sheet(&mut sheet, b.sheets, "table-caption", Vec3::new(0.0, (top - cap.h * 0.5) * PX, 0.02), 0.01, 0.0, slot::NEUTRAL_CANVAS);
        sheet.transform(&rest);
        b.mesh.append(&sheet);
        // Overskriftsraden, ett ark per sortering.
        let head_y = top - cap.h - head.h * 0.5;
        for (k, id) in ["none", "0a", "0d", "1a", "1d", "2a", "2d"].iter().enumerate() {
            let mut m = Mesh::default();
            signs::sheet(&mut m, b.sheets, &format!("table-head:{id}"), Vec3::ZERO, 0.01, 0.0, slot::NEUTRAL_CANVAS);
            self.heads[k] = b.prop("table_head", m, rest * Affine3A::from_translation(Vec3::new(0.0, head_y * PX, 0.02)), None);
        }
        // Kolonneoverskriftene du kan sikte på.
        let mut x = -W * 0.5;
        for c in 0..3 {
            let cx = x + COLS[c] * 0.5;
            self.head_cols[c] = b.prop("table", Mesh::default(), rest * Affine3A::from_translation(Vec3::new(cx * PX, head_y * PX, 0.03)), Some((Vec3::ZERO, Vec3::new(COLS[c] * PX * 0.5, head.h * PX * 0.5, 0.04))));
            x += COLS[c];
        }
        // Radene: alle utgaver oppå hverandre, posert etter sorteringen.
        for i in 0..3 {
            for (g, bg) in BG.iter().enumerate() {
                for (l, last) in ["mid", "last"].iter().enumerate() {
                    let mut m = Mesh::default();
                    signs::sheet(&mut m, b.sheets, &format!("table-row:{bg}:{i}:{last}"), Vec3::ZERO, 0.01, 0.0, if *bg == "striped" { slot::NEUTRAL_SURFACE } else { slot::NEUTRAL_CANVAS });
                    self.rows[i][g][l] = b.prop("table_row", m, rest * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.02)), None);
                }
            }
            self.row_targets[i] = b.prop("table", Mesh::default(), rest * Affine3A::from_translation(Vec3::Z * 0.03), Some((Vec3::ZERO, Vec3::new(W * PX * 0.5, row.h * PX * 0.5, 0.04))));
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        let Some(c) = self.head_cols.iter().position(|h| Some(*h) == env.aimed_prop) else { return };
        let asc = !matches!(self.sort, Some((sc, true)) if sc == c);
        self.sort = Some((c, asc));
        let name = ["rute", "avgang", "pris"][c];
        env.toasts.push(format!("Sortert etter {name}, {}", if asc { "stigende" } else { "synkende" }));
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        if let Some(k) = self.heads.iter().position(|h| *h == prop) {
            return (if k == self.head_index() { rest } else { rest * hidden }, [0.0; 4]);
        }
        let order = self.order();
        let slot_y = |slot: usize| (self.rows_top - self.row_h * (slot as f32 + 0.5)) * PX;
        // Raden du sikter på, i hoverable-varianten; ellers den første.
        let aimed_slot = self.row_targets.iter().position(|r| Some(*r) == env.aimed_prop);
        for (slot, &i) in order.iter().enumerate() {
            if prop == self.row_targets[slot] {
                return (rest * Affine3A::from_translation(Vec3::Y * slot_y(slot)), [0.0; 4]);
            }
            for g in 0..3 {
                for l in 0..2 {
                    if prop == self.rows[i][g][l] {
                        let want_g = match self.variant {
                            1 if slot % 2 == 1 => 1,
                            2 if aimed_slot.unwrap_or(0) == slot => 2,
                            _ => 0,
                        };
                        let want_l = usize::from(slot == 2);
                        let shown = g == want_g && l == want_l;
                        return (if shown { rest * Affine3A::from_translation(Vec3::Y * slot_y(slot)) } else { rest * hidden }, [0.0; 4]);
                    }
                }
            }
        }
        (rest, [0.0; 4])
    }
}
