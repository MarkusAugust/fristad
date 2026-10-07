//! `.fs-tabs`: huset med tre dører ved Østre gate. Faneraden står over
//! dørene; den valgte fanen har døra si åpen, og panelet er rommet
//! innenfor. R velger neste fane, slik piltastene gjør.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, DARK_TRIM, OAK, STONE, WHITE_TRIM};
use crate::domain::geometry::{flag, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

pub const VARIANTS: [&str; 3] = ["Søknaden", "Vedlegg", "Vedtak"];
const PANELS: [&str; 3] = ["Søknaden", "Vedlegg", "Vedtak"];
const PANEL_TEXT: [[&str; 2]; 3] = [["Mottatt 4. mars.", "Saksnummer 2026-0481."], ["Fødselsattest mangler.", "Last opp før 1. april."], ["Innvilget fra 1. mai.", "Utbetaling den 15."]];

#[derive(Default)]
pub struct Tabs {
    variant: usize,
    doors: [usize; 3],
    /// Fanen i valgt og uvalgt stil, og streken under.
    labels: [[usize; 2]; 3],
    underline: [usize; 3],
    t: [f32; 3],
}

const POS: Vec3 = Vec3::new(36.0, 0.0, -10.0);
const W: f32 = 8.4;
const D: f32 = 5.0;
const WALL_H: f32 = 3.6;

impl Installation for Tabs {
    fn component(&self) -> Component {
        Component { name: "tabs", class: ".fs-tabs" }
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
        self.t = [1.0, 0.0, 0.0];
        let p = b.on_ground(POS);
        let t = at(p, 0.0);
        let wall = Mat::tex(Layer::Plaster, 0xffffff).tint(Vec3::new(0.93, 0.9, 0.84));
        let m = &mut b.mesh;
        // Kroppen: sokkel, vegger med tre døråpninger i fronten, flatt tak med gesims.
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, -0.75, -D * 0.5))), Vec3::new(W + 0.12, 2.5, D + 0.12), &STONE);
        // Huset er hult: bakvegg, sidevegger, tak og gulv, og skillevegger
        // mellom de tre rommene.
        let dark = Mat::plain(0x2b2a28, 0.9);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H * 0.5 + 0.5, -D + 0.1))), Vec3::new(W, WALL_H, 0.2), &wall, 1.0);
        for side in [-1.0f32, 1.0] {
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(side * (W * 0.5 - 0.1), WALL_H * 0.5 + 0.5, -D * 0.5))), Vec3::new(0.2, WALL_H, D), &wall, 1.0);
        }
        // Himlingen og gulvet stopper inne i veggene, så flatene deres ikke
        // ligger i samme plan som fasaden (det ga svarte striper som flimret).
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 0.3, -D * 0.5))), Vec3::new(W - 0.3, 0.2, D - 0.3), &dark);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.5 + 0.02, -D * 0.5))), Vec3::new(W - 0.3, 0.04, D - 0.3), &OAK, 1.0);
        for x in [-1.35f32, 1.35] {
            m.cuboid(&(t * Affine3A::from_translation(Vec3::new(x, WALL_H * 0.5 + 0.5, -D * 0.5))), Vec3::new(0.1, WALL_H, D - 0.2), &dark);
        }
        // Fronten mellom dørene, og over dem.
        let door_w = 1.0;
        let door_h = 2.1;
        let xs = [-2.7f32, 0.0, 2.7];
        let mut edges = vec![-W * 0.5];
        for x in xs {
            edges.push(x - door_w * 0.5);
            edges.push(x + door_w * 0.5);
        }
        edges.push(W * 0.5);
        for k in (0..edges.len()).step_by(2) {
            let (a, c) = (edges[k], edges[k + 1]);
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new((a + c) * 0.5, WALL_H * 0.5 + 0.5, -0.1))), Vec3::new(c - a, WALL_H, 0.2), &wall, 1.0);
        }
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.5 + door_h + (WALL_H - door_h) * 0.5, -0.1))), Vec3::new(W, WALL_H - door_h, 0.2), &wall, 1.0);
        // Gesims, tak og pipe.
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 0.56, -D * 0.5))), Vec3::new(W + 0.5, 0.14, D + 0.5), &WHITE_TRIM, 1.0);
        m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 0.62, -D * 0.5))), W, D, 1.4, 0.4, &Mat::tex(Layer::Slate, 0xffffff), &wall, &DARK_TRIM);
        // Dørkarmer og trinn.
        for x in xs {
            for sx in [-1.0f32, 1.0] {
                m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x + sx * (door_w * 0.5 + 0.05), 0.5 + door_h * 0.5, 0.0))), Vec3::new(0.1, door_h + 0.1, 0.22), &WHITE_TRIM, 1.0);
            }
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, 0.5 + door_h + 0.05, 0.0))), Vec3::new(door_w + 0.2, 0.1, 0.22), &WHITE_TRIM, 1.0);
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, 0.25, 0.35))), Vec3::new(door_w + 0.6, 0.5, 0.7), &STONE, 1.0);
        }
        // Panelene inne i rommene, som opplyste plater på bakveggen.
        for (k, x) in xs.iter().enumerate() {
            let mut panel = Mesh::default();
            let w = 200.0;
            let h = 16.0 + 20.0 * 1.25 + 8.0 + 2.0 * 16.0 * 1.5 + 16.0;
            signs::boxed(&mut panel, Vec3::ZERO, &Box2 { w, h, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
            let left = -w * 0.5 + 16.0;
            let top = h * 0.5 - 16.0;
            signs::paragraph(&mut panel, b.font, &[PANELS[k]], left, top, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
            signs::paragraph(&mut panel, b.font, &PANEL_TEXT[k], left, top - 20.0 * 1.25 - 8.0, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
            panel.transform(&(t * Affine3A::from_translation(Vec3::new(*x, 1.7, -D + 0.25))));
            m.append(&panel);
        }
        for x in xs {
            b.light(p + Vec3::new(x, WALL_H + 0.1, -D * 0.5), Vec3::new(1.0, 0.92, 0.8), 2.5, 6.0);
        }

        // Faneraden over dørene: knapper med padding 8/16, min-høyde 40,
        // mellomrom 4, og en strek under hele raden.
        let gap = 4.0;
        let widths: Vec<f32> = VARIANTS.iter().map(|l| signs::width(b.font, l, 16.0) / PX + 32.0).collect();
        let total: f32 = widths.iter().sum::<f32>() + gap * 2.0;
        let row_y = 0.5 + door_h + 0.55;
        let z = 0.03;
        let mut list = Mesh::default();
        list.cuboid(&Affine3A::from_translation(Vec3::new(0.0, -20.0 * PX, 0.0)), Vec3::new((total + 32.0) * PX, PX, 0.01), &Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_BORDER_SUBTLE).flags(flag::NO_DIRT | flag::NO_WEATHER));
        list.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, row_y, z))));
        b.mesh.append(&list);
        let mut x = -total * 0.5;
        for k in 0..3 {
            let cx = x + widths[k] * 0.5;
            let rest = t * Affine3A::from_translation(Vec3::new(cx * PX, row_y, z));
            for (s, (text_slot, label_name)) in [(slot::NEUTRAL_TEXT_STRONG, "tab_selected"), (slot::ACCENT_TEXT, "tab")].into_iter().enumerate() {
                let mut lm = Mesh::default();
                signs::text(&mut lm, b.font, VARIANTS[k], Vec3::new(0.0, 0.0, 0.01), 16.0, text_slot);
                let collider = if s == 0 { Some((Vec3::ZERO, Vec3::new(widths[k] * PX * 0.5, 20.0 * PX, 0.05))) } else { None };
                self.labels[k][s] = b.prop(label_name, lm, rest, collider);
            }
            let mut ul = Mesh::default();
            ul.cuboid(&Affine3A::from_translation(Vec3::new(0.0, -19.0 * PX, 0.0)), Vec3::new(widths[k] * PX, 2.0 * PX, 0.012), &Mat::plain(0xffffff, 0.5).slot(slot::ACCENT_BORDER).flags(flag::NO_DIRT | flag::NO_WEATHER));
            self.underline[k] = b.prop("tab_underline", ul, rest, None);
            x += widths[k] + gap;
        }

        // Dørene: fyllingsdører i malt tre, hengslet til venstre.
        for (k, x) in xs.iter().enumerate() {
            let mut door = Mesh::default();
            let paint = Mat::tex(Layer::Cladding, 0xffffff).tint([Vec3::new(0.2, 0.35, 0.55), Vec3::new(0.55, 0.2, 0.2), Vec3::new(0.25, 0.45, 0.3)][k]).flags(flag::ROT_UV | flag::NO_DIRT);
            door.cuboid_ao(&Affine3A::from_translation(Vec3::new(door_w * 0.5, door_h * 0.5, 0.0)), Vec3::new(door_w - 0.02, door_h - 0.02, 0.06), &paint, 1.0);
            for y in [0.55, 1.5] {
                door.cuboid_ao(&Affine3A::from_translation(Vec3::new(door_w * 0.5, y, 0.035)), Vec3::new(door_w - 0.3, 0.6, 0.02), &paint, 1.0);
            }
            door.sphere(&Affine3A::from_translation(Vec3::new(door_w - 0.15, 1.05, 0.06)), Vec3::splat(0.035), 5, 8, &Mat::tex(Layer::Metal, 0xffffff));
            self.doors[k] = b.prop("tab_door", door, t * Affine3A::from_translation(Vec3::new(x - door_w * 0.5, 0.5, 0.0)), Some((Vec3::new(door_w * 0.5, door_h * 0.5, 0.0), Vec3::new(door_w * 0.5, door_h * 0.5, 0.04))));
        }

        // Kollidere: veggene og trinnene.
        b.solid_box(p + Vec3::new(0.0, WALL_H * 0.5 + 0.5, -D + 0.1), Vec3::new(W, WALL_H, 0.2), 0.0);
        for side in [-1.0f32, 1.0] {
            b.solid_box(p + Vec3::new(side * (W * 0.5 - 0.1), WALL_H * 0.5 + 0.5, -D * 0.5), Vec3::new(0.2, WALL_H, D), 0.0);
        }
        for x in [-1.35f32, 1.35] {
            b.solid_box(p + Vec3::new(x, WALL_H * 0.5 + 0.5, -D * 0.5), Vec3::new(0.1, WALL_H, D - 0.2), 0.0);
        }
        for k in (0..edges.len()).step_by(2) {
            let (a, c) = (edges[k], edges[k + 1]);
            b.solid_box(p + Vec3::new((a + c) * 0.5, WALL_H * 0.5 + 0.5, -0.1), Vec3::new(c - a, WALL_H, 0.3), 0.0);
        }
        for x in xs {
            b.solid_box(p + Vec3::new(x, 0.25, 0.35), Vec3::new(door_w + 0.6, 0.5, 0.7), 0.0);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        for k in 0..3 {
            let goal = if k == self.variant { 1.0 } else { 0.0 };
            self.t[k] += (goal - self.t[k]) * (env.dt * 4.0).min(1.0);
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        for k in 0..3 {
            if prop == self.doors[k] {
                let e = self.t[k] * self.t[k] * (3.0 - 2.0 * self.t[k]);
                return (rest * Affine3A::from_rotation_y(-e * 1.75), [0.0; 4]);
            }
            let selected = k == self.variant;
            if prop == self.labels[k][0] {
                return (if selected { rest } else { rest * hidden }, [0.0; 4]);
            }
            if prop == self.labels[k][1] {
                return (if selected { rest * hidden } else { rest }, [0.0; 4]);
            }
            if prop == self.underline[k] {
                return (if selected { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }
}

#[allow(dead_code)]
fn _unused(_q: Quat) {}
