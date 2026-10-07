//! Gorveks hus: et tømmerhus med torvtak ved veien nordover, rett utenfor
//! byen, med åpen dør. Innenfor bor en barbarkriger fra en film fra 80- og
//! 90-tallet: peis med brennende ild, bjørnefell på gulvet, høvdingstol
//! med gevir, langbord med benker, brisk med skinn, kiste, øltønner, gryte
//! over ilden, skjold, øks og sverd på veggene, elggevir over peisen,
//! fakler, kranium på hylla og malerier av bedriftene hans. Innboet er
//! modeller fra `tools/models/gorvek.py`; flammene er en prop som
//! spillet animerer.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, DARK_TRIM, OAK, STONE};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};

/// Midten av gulvet; fronten med døra vender mot vest, mot veien.
pub const POS: Vec3 = Vec3::new(17.5, 0.0, -71.5);
pub const YAW: f32 = -FRAC_PI_2;
/// Innvendige mål: lengde (lokal x), dybde (lokal z) og vegghøyde.
const W: f32 = 9.0;
const D: f32 = 6.5;
const LOG_R: f32 = 0.15;
const LOGS: usize = 9;
/// Gulvets overkant over tomta.
const FLOOR: f32 = 0.45;
/// Døra: halv bredde og høyde.
const DOOR_HW: f32 = 0.7;
const DOOR_H: f32 = 2.1;

#[derive(Default)]
pub struct Gorvek {
    flames: [usize; 3],
}

/// Barket tømmer.
fn timber() -> Mat {
    Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.68, 0.5, 0.3))
}

/// En liggende stokk langs lokal x fra `x0` til `x1` i høyden `y`, `z` inn.
fn log_x(m: &mut Mesh, local: &dyn Fn(Vec3) -> Affine3A, x0: f32, x1: f32, y: f32, z: f32) {
    let t = local(Vec3::new(x0, y, z)) * Affine3A::from_rotation_z(-FRAC_PI_2);
    m.cylinder(&t, LOG_R, LOG_R, x1 - x0, 10, &timber(), true);
}

/// En liggende stokk langs lokal z.
fn log_z(m: &mut Mesh, local: &dyn Fn(Vec3) -> Affine3A, z0: f32, z1: f32, y: f32, x: f32) {
    let t = local(Vec3::new(x, y, z0)) * Affine3A::from_rotation_x(FRAC_PI_2);
    m.cylinder(&t, LOG_R, LOG_R, z1 - z0, 10, &timber(), true);
}

/// Et maleri i ramme på en vegg: `t` er midten av lerretet med forsiden
/// mot +Z. Motivene er enkle flater: fjellet med sola, havet med skipet, og
/// dragen.
fn painting(m: &mut Mesh, t: &Affine3A, w: f32, h: f32, motif: usize) {
    let frame = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.3, 0.2, 0.1));
    let fw = 0.06;
    for (c, s) in [
        (Vec3::new(0.0, h * 0.5 + fw * 0.5, 0.0), Vec3::new(w + 2.0 * fw, fw, 0.05)),
        (Vec3::new(0.0, -h * 0.5 - fw * 0.5, 0.0), Vec3::new(w + 2.0 * fw, fw, 0.05)),
        (Vec3::new(-w * 0.5 - fw * 0.5, 0.0, 0.0), Vec3::new(fw, h, 0.05)),
        (Vec3::new(w * 0.5 + fw * 0.5, 0.0, 0.0), Vec3::new(fw, h, 0.05)),
    ] {
        m.cuboid_ao(&(*t * Affine3A::from_translation(c)), s, &frame, 1.0);
    }
    let paint = |c: u32| Mat::plain(c, 0.85).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let quad = |m: &mut Mesh, x0: f32, y0: f32, x1: f32, y1: f32, z: f32, c: u32| {
        m.quad(t, [Vec3::new(x0, y0, z), Vec3::new(x1, y0, z), Vec3::new(x1, y1, z), Vec3::new(x0, y1, z)], &paint(c), [1.0; 4]);
    };
    let tri = |m: &mut Mesh, a: Vec2, b: Vec2, c: Vec2, z: f32, col: u32| {
        m.tri(t, [Vec3::new(a.x, a.y, z), Vec3::new(b.x, b.y, z), Vec3::new(c.x, c.y, z)], &paint(col));
    };
    let (hw, hh) = (w * 0.5, h * 0.5);
    match motif {
        0 => {
            // Fjellet: himmel, sol, tre tinder, og Gorvek på toppen.
            quad(m, -hw, -hh, hw, hh, 0.0, 0xc9a66b);
            quad(m, -hw, 0.0, hw, hh, 0.001, 0x7fa6c9);
            m.cylinder(&(*t * Affine3A::from_translation(Vec3::new(hw * 0.55, hh * 0.55, 0.002)) * Affine3A::from_rotation_x(FRAC_PI_2)), 0.08, 0.08, 0.002, 14, &paint(0xf4c542), true);
            tri(m, Vec2::new(-hw, -hh * 0.2), Vec2::new(-hw * 0.3, hh * 0.7), Vec2::new(hw * 0.3, -hh * 0.2), 0.003, 0x5a6470);
            tri(m, Vec2::new(-hw * 0.2, -hh * 0.2), Vec2::new(hw * 0.45, hh * 0.35), Vec2::new(hw, -hh * 0.2), 0.004, 0x6f7a86);
            tri(m, Vec2::new(-hw * 0.47, hh * 0.35), Vec2::new(-hw * 0.3, hh * 0.7), Vec2::new(-hw * 0.13, hh * 0.35), 0.005, 0xf0f0f0);
            quad(m, -hw, -hh, hw, -hh * 0.2, 0.006, 0x4f7a3a);
            // Krigeren: kropp, hode og sverd løftet mot himmelen.
            quad(m, -hw * 0.34, hh * 0.68, -hw * 0.26, hh * 0.92, 0.007, 0x2a1d14);
            m.cylinder(&(*t * Affine3A::from_translation(Vec3::new(-hw * 0.3, hh * 0.96, 0.008)) * Affine3A::from_rotation_x(FRAC_PI_2)), 0.03, 0.03, 0.002, 8, &paint(0x2a1d14), true);
            quad(m, -hw * 0.27, hh * 0.85, -hw * 0.25, hh * 1.0, 0.008, 0xd9d9d9);
        }
        1 => {
            // Havet: himmel i storm, bølger og langskipet med seil.
            quad(m, -hw, -hh, hw, hh, 0.0, 0x8a98a6);
            quad(m, -hw, -hh, hw, -hh * 0.1, 0.001, 0x2f4f6e);
            for k in 0..4 {
                let x = -hw + (k as f32 + 0.5) * w / 4.0;
                tri(m, Vec2::new(x - 0.12, -hh * 0.3), Vec2::new(x, -hh * 0.15), Vec2::new(x + 0.12, -hh * 0.3), 0.002, 0x5d86a8);
            }
            tri(m, Vec2::new(-hw * 0.6, -hh * 0.2), Vec2::new(-hw * 0.75, hh * 0.1), Vec2::new(hw * 0.6, -hh * 0.2), 0.003, 0x4a2f1c);
            quad(m, -hw * 0.6, -hh * 0.2, hw * 0.55, 0.0, 0.003, 0x4a2f1c);
            tri(m, Vec2::new(hw * 0.55, -hh * 0.2), Vec2::new(hw * 0.75, hh * 0.15), Vec2::new(hw * 0.55, 0.0), 0.003, 0x4a2f1c);
            quad(m, -hw * 0.02, 0.0, hw * 0.02, hh * 0.8, 0.004, 0x2a1d14);
            tri(m, Vec2::new(0.0, hh * 0.75), Vec2::new(-hw * 0.5, hh * 0.1), Vec2::new(hw * 0.5, hh * 0.1), 0.005, 0xe8e2d0);
            quad(m, -hw * 0.5, hh * 0.1, hw * 0.5, hh * 0.2, 0.006, 0xb0302a);
        }
        _ => {
            // Dragen: mørk himmel, dragen i grønt med ild, og krigeren med skjold.
            quad(m, -hw, -hh, hw, hh, 0.0, 0x3b1f2b);
            quad(m, -hw, -hh, hw, -hh * 0.5, 0.001, 0x2a2420);
            tri(m, Vec2::new(-hw * 0.2, hh * 0.1), Vec2::new(-hw * 0.9, hh * 0.9), Vec2::new(hw * 0.1, hh * 0.3), 0.002, 0x3f7a3a);
            tri(m, Vec2::new(hw * 0.0, hh * 0.1), Vec2::new(hw * 0.5, hh * 0.9), Vec2::new(hw * 0.3, -hh * 0.1), 0.002, 0x3f7a3a);
            quad(m, -hw * 0.3, -hh * 0.15, hw * 0.4, hh * 0.2, 0.003, 0x4f9a46);
            tri(m, Vec2::new(hw * 0.4, hh * 0.2), Vec2::new(hw * 0.8, hh * 0.35), Vec2::new(hw * 0.4, -hh * 0.1), 0.004, 0x4f9a46);
            tri(m, Vec2::new(hw * 0.8, hh * 0.25), Vec2::new(hw * 1.0, -hh * 0.3), Vec2::new(hw * 0.7, hh * 0.1), 0.005, 0xf08a2a);
            tri(m, Vec2::new(hw * 0.85, hh * 0.2), Vec2::new(hw * 0.95, -hh * 0.15), Vec2::new(hw * 0.75, hh * 0.1), 0.006, 0xf6c744);
            quad(m, -hw * 0.75, -hh * 0.9, -hw * 0.6, -hh * 0.45, 0.004, 0x2a1d14);
            m.cylinder(&(*t * Affine3A::from_translation(Vec3::new(-hw * 0.56, -hh * 0.62, 0.005)) * Affine3A::from_rotation_x(FRAC_PI_2)), 0.09, 0.09, 0.002, 12, &paint(0xb0302a), true);
        }
    }
}

impl Installation for Gorvek {
    fn component(&self) -> Component {
        Component { name: "gorvek", class: "" }
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(POS);
        let t = at(p, YAW);
        let rot = Quat::from_rotation_y(YAW);
        let local = move |v: Vec3| t * Affine3A::from_translation(v);
        let world = |v: Vec3| t.transform_point3(v);
        let (hw, hd) = (W * 0.5, D * 0.5);

        // Tomta: steinmur ned i bakken og plankegulv oppå.
        {
            let m = &mut b.mesh;
            // Sokkelen stopper 6 cm under plankene, så gulvet ikke z-kjemper med steinen.
            m.cuboid_ao(&local(Vec3::new(0.0, (FLOOR - 0.06 - 1.4) * 0.5, 0.0)), Vec3::new(W + 0.5, FLOOR - 0.06 + 1.4, D + 0.5), &STONE, 1.0);
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR - 0.04, 0.0)), Vec3::new(W, 0.08, D), &OAK.flags(flag::NO_WEATHER), 1.0);
            // To trappesteiner foran døra.
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR * 0.5 - 0.5, hd + 0.5)), Vec3::new(2.0, FLOOR + 1.0, 1.0), &STONE, 1.0);
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR * 0.25 - 0.5, hd + 1.2)), Vec3::new(2.0, FLOOR * 0.5 + 1.0, 0.5), &STONE, 1.0);
        }
        b.solid_box(world(Vec3::new(0.0, FLOOR * 0.5 - 0.7, 0.0)), Vec3::new(W + 0.5, FLOOR + 1.44, D + 0.5), YAW);
        b.solid_box(world(Vec3::new(0.0, FLOOR * 0.5 - 0.5, hd + 0.5)), Vec3::new(2.0, FLOOR + 1.0, 1.0), YAW);
        b.solid_box(world(Vec3::new(0.0, FLOOR * 0.25 - 0.5, hd + 1.2)), Vec3::new(2.0, FLOOR * 0.5 + 1.0, 0.5), YAW);
        b.floor(world(Vec3::ZERO), Vec2::new(hw + 0.25, hd + 0.25), YAW, p.y + FLOOR);
        b.floor(world(Vec3::new(0.0, 0.0, hd + 0.5)), Vec2::new(1.0, 0.5), YAW, p.y + FLOOR);

        // Tømmerveggene: ni stokker høye, med døråpning og to små vinduer i
        // fronten. Stokkene krysser hverandre i hjørnene.
        let win_y = (1.3, 2.0);
        let wins = [-2.8f32, 2.8];
        for i in 0..LOGS {
            let y = FLOOR + LOG_R + i as f32 * 2.0 * LOG_R;
            let m = &mut b.mesh;
            // Bakveggen og sideveggene.
            log_x(m, &local, -hw - LOG_R, hw + LOG_R, y, -hd);
            log_z(m, &local, -hd - LOG_R, hd + LOG_R, y, -hw);
            log_z(m, &local, -hd - LOG_R, hd + LOG_R, y, hw);
            // Fronten: hel over døra, ellers delt rundt døra og vinduene.
            let in_door = y - LOG_R < DOOR_H + FLOOR;
            let in_win = y + LOG_R > win_y.0 + FLOOR && y - LOG_R < win_y.1 + FLOOR;
            let mut cuts: Vec<(f32, f32)> = vec![];
            if in_door {
                cuts.push((-DOOR_HW - 0.08, DOOR_HW + 0.08));
            }
            if in_win {
                for wx in wins {
                    cuts.push((wx - 0.4, wx + 0.4));
                }
            }
            cuts.sort_by(|a, c| a.0.partial_cmp(&c.0).unwrap());
            let mut x = -hw - LOG_R;
            for (c0, c1) in cuts {
                if c0 > x {
                    log_x(m, &local, x, c0, y, hd);
                }
                x = c1;
            }
            log_x(m, &local, x, hw + LOG_R, y, hd);
        }
        let wall_top = FLOOR + LOGS as f32 * 2.0 * LOG_R;
        // Dørkarm, vinduskarmer med glass, og døra på gløtt innover.
        {
            let m = &mut b.mesh;
            let frame = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.35, 0.24, 0.12));
            for sx in [-1.0f32, 1.0] {
                m.cuboid_ao(&local(Vec3::new(sx * (DOOR_HW + 0.04), FLOOR + DOOR_H * 0.5, hd)), Vec3::new(0.1, DOOR_H, 0.36), &frame, 1.0);
            }
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR + DOOR_H + 0.05, hd)), Vec3::new(2.0 * DOOR_HW + 0.18, 0.12, 0.36), &frame, 1.0);
            let glass = Mat::special(kind::GLASS, 0x1d2630, 0.05);
            for wx in wins {
                let (y0, y1) = (FLOOR + win_y.0, FLOOR + win_y.1);
                let cy = (y0 + y1) * 0.5;
                for sx in [-1.0f32, 1.0] {
                    m.cuboid_ao(&local(Vec3::new(wx + sx * 0.37, cy, hd)), Vec3::new(0.06, y1 - y0 + 0.12, 0.34), &frame, 1.0);
                }
                for sy in [-1.0f32, 1.0] {
                    m.cuboid_ao(&local(Vec3::new(wx, cy + sy * (y1 - y0) * 0.5, hd)), Vec3::new(0.8, 0.06, 0.34), &frame, 1.0);
                }
                m.cuboid(&local(Vec3::new(wx, cy, hd)), Vec3::new(0.7, y1 - y0, 0.02), &glass);
                // Sprosse.
                m.cuboid_ao(&local(Vec3::new(wx, cy, hd)), Vec3::new(0.03, y1 - y0, 0.05), &frame, 1.0);
            }
            // Døra: tykke planker med jernbeslag, hengslet til venstre, åpen
            // 110 grader innover.
            let door_t = local(Vec3::new(-DOOR_HW, FLOOR, hd - 0.1)) * Affine3A::from_rotation_y(-1.9);
            let plank = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.4, 0.28, 0.15)).flags(flag::ROT_UV);
            m.cuboid_ao(&(door_t * Affine3A::from_translation(Vec3::new(DOOR_HW - 0.02, DOOR_H * 0.5, 0.0))), Vec3::new(2.0 * DOOR_HW - 0.06, DOOR_H - 0.04, 0.08), &plank, 1.0);
            let iron = Mat::tex(Layer::Metal, 0x202020);
            for y in [0.5, 1.6] {
                m.cuboid_ao(&(door_t * Affine3A::from_translation(Vec3::new(DOOR_HW - 0.02, y, 0.05))), Vec3::new(2.0 * DOOR_HW - 0.2, 0.1, 0.02), &iron, 1.0);
            }
            m.torus(&(door_t * Affine3A::from_translation(Vec3::new(2.0 * DOOR_HW - 0.25, 1.05, 0.07)) * Affine3A::from_rotation_x(FRAC_PI_2)), 0.07, 0.012, 12, 6, &iron);
        }
        // Veggkollidere: baksida, sidene og fronten på hver side av døra, og
        // over den.
        let wall_h = wall_top - FLOOR;
        b.solid_box(world(Vec3::new(0.0, FLOOR + wall_h * 0.5, -hd)), Vec3::new(W + 0.4, wall_h, 2.0 * LOG_R), YAW);
        for sx in [-1.0f32, 1.0] {
            b.solid_box(world(Vec3::new(sx * hw, FLOOR + wall_h * 0.5, 0.0)), Vec3::new(2.0 * LOG_R, wall_h, D + 0.4), YAW);
            let seg_w = hw - DOOR_HW - 0.08 + LOG_R;
            b.solid_box(world(Vec3::new(sx * (DOOR_HW + 0.08 + seg_w * 0.5), FLOOR + wall_h * 0.5, hd)), Vec3::new(seg_w, wall_h, 2.0 * LOG_R), YAW);
        }
        b.solid_box(world(Vec3::new(0.0, FLOOR + DOOR_H + (wall_h - DOOR_H) * 0.5 + 0.05, hd)), Vec3::new(2.0 * DOOR_HW + 0.2, wall_h - DOOR_H, 2.0 * LOG_R), YAW);

        // Taket: torvtak på takstoler, med mønet på langs og stort utstikk.
        {
            let m = &mut b.mesh;
            let turf = Mat::tex(Layer::Grass, 0xffffff);
            let gable = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.5, 0.36, 0.2)).flags(flag::ROT_UV);
            m.gable_roof(&local(Vec3::new(0.0, wall_top, 0.0)), W + 2.0 * LOG_R, D + 2.0 * LOG_R, 2.6, 0.7, &turf, &gable, &DARK_TRIM);
            // Takbjelkene under mønet, synlige innenfra.
            for k in 0..5 {
                let x = -hw + 0.6 + k as f32 * (W - 1.2) / 4.0;
                m.cuboid_ao(&local(Vec3::new(x, wall_top + 0.05, 0.0)), Vec3::new(0.18, 0.2, D + 0.2), &OAK, 1.0);
            }
            m.cuboid_ao(&local(Vec3::new(0.0, wall_top + 2.45, 0.0)), Vec3::new(W + 1.6, 0.2, 0.2), &OAK, 1.0);
            // Pipa i stein over peisen.
            m.cuboid_ao(&local(Vec3::new(0.0, wall_top + 1.3, -hd + 0.5)), Vec3::new(0.9, 2.6, 0.9), &STONE, 1.0);
            m.cuboid_ao(&local(Vec3::new(0.0, wall_top + 2.65, -hd + 0.5)), Vec3::new(1.1, 0.12, 1.1), &STONE, 1.0);
        }

        // Peisen ved bakveggen: ildsted, bakvegg, sidevanger, hylle og hette.
        let hearth_z = -hd + 0.55;
        {
            let m = &mut b.mesh;
            // Steinplata foran peisen starter 2 cm over plankene, så undersiden
            // ikke ligger i gulvplanet (det ga flimring).
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR + 0.08, hearth_z + 0.1)), Vec3::new(2.4, 0.12, 1.3), &STONE, 1.0);
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR + 1.1, -hd + 0.2)), Vec3::new(2.4, 2.2, 0.4), &STONE, 1.0);
            for sx in [-1.0f32, 1.0] {
                m.cuboid_ao(&local(Vec3::new(sx * 1.05, FLOOR + 0.75, hearth_z)), Vec3::new(0.3, 1.3, 0.9), &STONE, 1.0);
            }
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR + 1.46, hearth_z + 0.05)), Vec3::new(2.6, 0.12, 0.6), &OAK, 1.0);
            m.cuboid_ao(&local(Vec3::new(0.0, FLOOR + 2.1, hearth_z - 0.1)), Vec3::new(2.2, 1.2, 0.7), &STONE, 1.0);
            // Vedkubbene som brenner, og glørne under.
            let charred = Mat::plain(0x2b1d12, 0.9);
            for (k, (dx, yaw)) in [(-0.18f32, 0.4f32), (0.15, -0.5), (0.0, 1.3)].into_iter().enumerate() {
                let lt = local(Vec3::new(dx, FLOOR + 0.2 + k as f32 * 0.07, hearth_z)) * Affine3A::from_rotation_y(yaw) * Affine3A::from_rotation_z(-FRAC_PI_2) * Affine3A::from_translation(Vec3::new(0.0, -0.3, 0.0));
                m.cylinder(&lt, 0.07, 0.07, 0.6, 8, &charred, true);
            }
            m.cuboid(&local(Vec3::new(0.0, FLOOR + 0.16, hearth_z)), Vec3::new(0.9, 0.06, 0.6), &Mat::plain(0xff5a1a, 0.9).flags(flag::GLOW_ALWAYS | flag::NO_DIRT | flag::NO_WEATHER));
        }
        b.solid_box(world(Vec3::new(0.0, FLOOR + 1.1, -hd + 0.5)), Vec3::new(2.5, 2.2, 1.1), YAW);
        b.fire_light(world(Vec3::new(0.0, FLOOR + 0.9, hearth_z + 0.4)), Vec3::new(1.0, 0.55, 0.22), 6.0, 11.0);
        // Flammene: tre tunger som spillet lar blafre.
        for (k, (dx, dz, s)) in [(0.0f32, 0.0f32, 1.0f32), (-0.2, 0.1, 0.7), (0.18, -0.08, 0.8)].into_iter().enumerate() {
            let mut fm = Mesh::default();
            let col = if k == 0 { 0xffb340 } else { 0xff7a1e };
            let flame = Mat::plain(col, 0.95).flags(flag::GLOW_ALWAYS | flag::NO_DIRT | flag::NO_WEATHER);
            let profile = [Vec2::new(0.0, 0.0), Vec2::new(0.16, 0.04), Vec2::new(0.2, 0.18), Vec2::new(0.13, 0.42), Vec2::new(0.05, 0.62), Vec2::new(0.0, 0.75)];
            fm.lathe(&Affine3A::from_scale(Vec3::splat(s)), &profile, 10, &flame, false, 1.0);
            self.flames[k] = b.prop("gorvek_fire", fm, local(Vec3::new(dx, FLOOR + 0.22, hearth_z + dz)), None);
        }

        // Innboet.
        let place = |b: &mut Builder, name: &'static str, v: Vec3, yaw: f32| {
            b.place(name, local(v + Vec3::Y * FLOOR) * Affine3A::from_rotation_y(yaw));
        };
        // Bjørnefellen foran peisen, med hodet mot døra.
        place(b, "gorvek_bearskin", Vec3::new(0.0, 0.0, hearth_z + 2.0), -FRAC_PI_2);
        // Høvdingstolen vendt mot ilden.
        place(b, "gorvek_throne", Vec3::new(0.0, 0.0, 1.7), FRAC_PI_2);
        b.solid_box(world(Vec3::new(0.0, FLOOR + 0.5, 1.7)), Vec3::new(0.8, 1.0, 0.8), YAW);
        // Langbordet med benker til venstre, og krakker.
        place(b, "gorvek_table", Vec3::new(-2.7, 0.0, 0.6), 0.0);
        place(b, "gorvek_bench", Vec3::new(-2.7, 0.0, 1.35), 0.0);
        place(b, "gorvek_bench", Vec3::new(-2.7, 0.0, -0.15), 0.0);
        b.solid_box(world(Vec3::new(-2.7, FLOOR + 0.4, 0.6)), Vec3::new(2.5, 0.8, 1.9), YAW);
        place(b, "gorvek_stool", Vec3::new(-4.2, 0.0, 0.6), 0.3);
        place(b, "gorvek_stool", Vec3::new(1.4, 0.0, 2.4), 1.1);
        // Brisken langs høyre bakhjørne, med kista ved fotenden.
        place(b, "gorvek_bed", Vec3::new(3.2, 0.0, -2.4), 0.0);
        b.solid_box(world(Vec3::new(3.2, FLOOR + 0.3, -2.4)), Vec3::new(2.2, 0.6, 1.2), YAW);
        place(b, "gorvek_chest", Vec3::new(4.0, 0.0, -0.9), FRAC_PI_2);
        b.solid_box(world(Vec3::new(4.0, FLOOR + 0.35, -0.9)), Vec3::new(0.6, 0.7, 1.0), YAW);
        // Øltønnene i venstre bakhjørne.
        place(b, "gorvek_keg", Vec3::new(-4.0, 0.0, -2.6), 0.2);
        place(b, "gorvek_keg", Vec3::new(-3.3, 0.0, -2.75), 1.4);
        b.place_mounted("gorvek_keg", local(Vec3::new(-3.65, FLOOR + 0.8, -2.65)) * Affine3A::from_rotation_y(0.9));
        b.solid_box(world(Vec3::new(-3.65, FLOOR + 0.8, -2.65)), Vec3::new(1.4, 1.6, 0.8), YAW);
        // Gryta over ilden.
        b.place_mounted("gorvek_cauldron", local(Vec3::new(0.0, FLOOR + 0.12, hearth_z + 0.05)));
        // Kraniet på peishylla.
        b.place_mounted("gorvek_skull", local(Vec3::new(0.8, FLOOR + 1.52, hearth_z + 0.1)) * Affine3A::from_rotation_y(-0.6));
        // På veggene: elggeviret over peisen, skjold og våpen, fakler og
        // malerier. Veggtingene har origo i festet med forsiden mot +Z.
        let on_back = |v: Vec3| local(Vec3::new(v.x, v.y, -hd + LOG_R + 0.02));
        let on_left = |v: Vec3| local(Vec3::new(-hw + LOG_R + 0.02, v.y, v.z)) * Affine3A::from_rotation_y(FRAC_PI_2);
        let on_right = |v: Vec3| local(Vec3::new(hw - LOG_R - 0.02, v.y, v.z)) * Affine3A::from_rotation_y(-FRAC_PI_2);
        let on_front = |v: Vec3| local(Vec3::new(v.x, v.y, hd - LOG_R - 0.02)) * Affine3A::from_rotation_y(PI);
        b.place_mounted("gorvek_antlers", on_back(Vec3::new(0.0, FLOOR + 2.35, 0.0)));
        b.place_mounted("gorvek_shield", on_back(Vec3::new(-2.6, FLOOR + 1.9, 0.0)));
        b.place_mounted("gorvek_axe", on_back(Vec3::new(-2.6, FLOOR + 1.9, 0.0)) * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.12)) * Affine3A::from_rotation_z(0.7) * Affine3A::from_rotation_x(FRAC_PI_2));
        b.place_mounted("gorvek_sword", on_back(Vec3::new(-2.6, FLOOR + 1.9, 0.0)) * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.14)) * Affine3A::from_rotation_z(-0.7) * Affine3A::from_rotation_x(FRAC_PI_2));
        b.place_mounted("gorvek_shield", on_right(Vec3::new(0.0, FLOOR + 1.9, 1.6)));
        for (tf, pos) in [(on_left(Vec3::new(0.0, FLOOR + 1.75, 1.6)), Vec3::new(-hw + 0.45, FLOOR + 2.2, 1.6)), (on_left(Vec3::new(0.0, FLOOR + 1.75, -1.6)), Vec3::new(-hw + 0.45, FLOOR + 2.2, -1.6)), (on_right(Vec3::new(0.0, FLOOR + 1.75, -1.0)), Vec3::new(hw - 0.45, FLOOR + 2.2, -1.0))] {
            b.place_mounted("gorvek_torch", tf);
            b.fire_light(world(pos), Vec3::new(1.0, 0.62, 0.25), 2.2, 6.0);
        }
        {
            let m = &mut b.mesh;
            painting(m, &on_left(Vec3::new(0.0, FLOOR + 1.75, 0.0)), 1.3, 0.95, 0);
            painting(m, &on_right(Vec3::new(0.0, FLOOR + 1.8, 0.3)), 1.1, 0.8, 1);
            painting(m, &on_front(Vec3::new(-2.8, FLOOR + 0.95, 0.0)), 0.9, 0.5, 2);
            let _ = on_front;
        }

        // Utenfor: vedstabelen ved veggen og skiltet ved døra.
        {
            let m = &mut b.mesh;
            let wood = Mat::tex(Layer::Bark, 0xffffff);
            for row in 0..3 {
                let n = 6 - row;
                for k in 0..n {
                    let x = 2.3 + k as f32 * 0.3 + row as f32 * 0.15;
                    let y = 0.14 + row as f32 * 0.26;
                    let lt = local(Vec3::new(x, y, hd + 0.75)) * Affine3A::from_rotation_x(FRAC_PI_2) * Affine3A::from_translation(Vec3::new(0.0, -0.4, 0.0));
                    m.cylinder(&lt, 0.13, 0.13, 0.8, 8, &wood, true);
                }
            }
        }
        b.solid_box(world(Vec3::new(3.2, 0.4, hd + 0.75)), Vec3::new(2.0, 0.8, 0.9), YAW);
        {
            let sp = local(Vec3::new(1.5, 0.0, hd + 1.0));
            let m = &mut b.mesh;
            m.cylinder(&(sp * Affine3A::from_translation(Vec3::new(0.0, -0.4, -0.05))), 0.06, 0.05, 2.2, 8, &OAK, true);
            let mut board = Mesh::default();
            let w = signs::width(b.font, "Gorveks hus", 22.0).max(signs::width(b.font, "Gå inn", 12.0)) / PX + 40.0;
            signs::boxed(&mut board, Vec3::ZERO, &Box2 { w, h: 60.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
            signs::text(&mut board, b.font, "Gorveks hus", Vec3::new(0.0, 6.0 * PX, 0.02), 22.0, slot::NEUTRAL_TEXT_STRONG);
            signs::text(&mut board, b.font, "Gå inn", Vec3::new(0.0, -16.0 * PX, 0.02), 12.0, slot::NEUTRAL_TEXT_SUBTLE);
            board.transform(&(sp * Affine3A::from_translation(Vec3::new(0.0, 1.6, 0.0))));
            m.append(&board);
        }
        b.solid_box(world(Vec3::new(1.5, 0.9, hd + 1.0)), Vec3::new(0.2, 1.8, 0.2), YAW);
        let _ = rot;
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(k) = self.flames.iter().position(|p| *p == prop) {
            // Flammene blafrer: strekkes og krummes ujevnt.
            let t = env.time * 7.0 + k as f32 * 2.1;
            let sy = 1.0 + 0.22 * (t).sin() + 0.1 * (t * 1.7 + 0.5).sin();
            let sx = 1.0 + 0.12 * (t * 1.3 + 1.0).cos();
            let lean = 0.12 * (t * 0.9).sin();
            return (rest * Affine3A::from_rotation_z(lean) * Affine3A::from_scale(Vec3::new(sx, sy.max(0.3), sx)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
