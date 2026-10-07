//! `.fs-file-upload`: opplastingsfeltet i vedleggsgruppa på vedleggsarket,
//! med stiplet kant og nettleserens egen knapp. F velger en fil (og fjerner
//! den igjen), så lista `.fs-file-upload-list` under får en rad. R bytter
//! tilstand (standard, ugyldig, suksess, deaktivert).

use super::form::{self, COL_W, FILE_ROW_H, FILE_ROW_TOP, RIGHT, UPLOAD_H, UPLOAD_LABEL_TOP, UPLOAD_TOP};
use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "invalid", "success", "disabled"];
const FILE: &str = "fodselsattest.pdf";

#[derive(Default)]
pub struct FileUpload {
    variant: usize,
    chosen: bool,
    fields: [usize; 4],
    /// «Ingen fil valgt» i vanlig og deaktivert farge, og filnavnet.
    none: [usize; 2],
    name: usize,
    row: usize,
}

impl Installation for FileUpload {
    fn component(&self) -> Component {
        Component { name: "file-upload", class: ".fs-file-upload" }
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
        form::annex(b);
        // Ledeteksten over feltet står fast.
        let mut fixed = Mesh::default();
        signs::paragraph(&mut fixed, b.font, &["Last opp vedlegg"], 0.0, 0.0, 14.0, 1.5, 0.0, slot::NEUTRAL_TEXT_STRONG);
        fixed.transform(&form::on_annex(RIGHT, UPLOAD_LABEL_TOP, 0.03));
        b.mesh.append(&fixed);

        let cx = RIGHT + COL_W * 0.5;
        let cy = UPLOAD_TOP - UPLOAD_H * 0.5;
        let rest = form::on_annex(cx, cy, 0.035);
        let btn = "Velg fil";
        let bw = signs::width(b.font, btn, 16.0) / PX + 32.0;
        let bx = -COL_W * 0.5 + 8.0 + bw * 0.5;
        for (k, v) in VARIANTS.iter().enumerate() {
            let (fill, border, btn_text, btn_border) = match *v {
                "invalid" => (slot::DANGER_SURFACE, slot::DANGER_TEXT, slot::ACCENT_TEXT, slot::ACCENT_BORDER),
                "success" => (slot::NEUTRAL_CANVAS, slot::SUCCESS_TEXT, slot::ACCENT_TEXT, slot::ACCENT_BORDER),
                "disabled" => (slot::NEUTRAL_RAISED, slot::NEUTRAL_RAISED, slot::NEUTRAL_BORDER, slot::NEUTRAL_BORDER),
                _ => (slot::NEUTRAL_CANVAS, slot::NEUTRAL_BORDER, slot::ACCENT_TEXT, slot::ACCENT_BORDER),
            };
            let mut m = Mesh::default();
            signs::boxed(&mut m, Vec3::ZERO, &Box2 { w: COL_W, h: UPLOAD_H, radius: 4.0, fill, border: None, depth: 0.02 });
            form::dashed(&mut m, COL_W, UPLOAD_H, border, 0.012);
            // Knappen: padding 8/16, 1 px kant i aksentfargen, gjennomsiktig.
            signs::boxed(&mut m, Vec3::new(bx * PX, 0.0, 0.012), &Box2 { w: bw, h: 40.0, radius: 4.0, fill, border: Some(btn_border), depth: 0.01 });
            signs::text(&mut m, b.font, btn, Vec3::new(bx * PX, 0.0, 0.024), 16.0, btn_text);
            self.fields[k] = b.prop("file_upload", m, rest, Some((Vec3::ZERO, Vec3::new(COL_W * PX * 0.5, UPLOAD_H * PX * 0.5, 0.04))));
        }
        // Teksten etter knappen, 12 px margin.
        let tx = bx + bw * 0.5 + 12.0;
        let baseline = Vec3::new(tx * PX, -0.36 * 16.0 * PX, 0.024);
        for (k, color) in [slot::NEUTRAL_TEXT_STRONG, slot::NEUTRAL_BORDER].into_iter().enumerate() {
            let mut m = Mesh::default();
            signs::text_left(&mut m, b.font, "Ingen fil valgt", baseline, 16.0, color);
            self.none[k] = b.prop("file_upload_text", m, rest, None);
        }
        let mut m = Mesh::default();
        signs::text_left(&mut m, b.font, FILE, baseline, 16.0, slot::NEUTRAL_TEXT_STRONG);
        self.name = b.prop("file_upload_text", m, rest, None);

        // Raden i lista: flate med padding 8/12, filnavnet og en ghost-knapp.
        let mut row = Mesh::default();
        signs::boxed(&mut row, Vec3::ZERO, &Box2 { w: COL_W, h: FILE_ROW_H, radius: 4.0, fill: slot::NEUTRAL_SURFACE, border: None, depth: 0.02 });
        signs::text_left(&mut row, b.font, FILE, Vec3::new((-COL_W * 0.5 + 12.0) * PX, -0.36 * 14.0 * PX, 0.014), 14.0, slot::NEUTRAL_TEXT_STRONG);
        let remove = "Fjern fodselsattest.pdf";
        let rw = signs::width(b.font, remove, 14.0) / PX;
        signs::text_left(&mut row, b.font, remove, Vec3::new((COL_W * 0.5 - 12.0 - rw) * PX, -0.36 * 14.0 * PX, 0.014), 14.0, slot::ACCENT_TEXT);
        self.row = b.prop("file_upload_row", row, form::on_annex(cx, FILE_ROW_TOP - FILE_ROW_H * 0.5, 0.035), None);
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        if VARIANTS[self.variant] == "disabled" {
            env.toasts.push("Feltet er deaktivert".into());
            return;
        }
        self.chosen = !self.chosen;
        env.toasts.push(if self.chosen { format!("Valgte {FILE}") } else { format!("Fjernet {FILE}") });
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        let show = |on: bool| if on { rest } else { rest * hidden };
        let disabled = VARIANTS[self.variant] == "disabled";
        for (k, p) in self.fields.iter().enumerate() {
            if prop == *p {
                return (show(k == self.variant), [0.0; 4]);
            }
        }
        if prop == self.none[0] {
            return (show(!self.chosen && !disabled), [0.0; 4]);
        }
        if prop == self.none[1] {
            return (show(!self.chosen && disabled), [0.0; 4]);
        }
        if prop == self.name {
            return (show(self.chosen), [0.0; 4]);
        }
        if prop == self.row {
            return (show(self.chosen), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
