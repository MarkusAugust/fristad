//! Gatelivet: parkerte biler, sykler, kafébord, torgbod, lyslenker,
//! gateskilt, postkasse og busskur. Alt er prosedyrisk, bygd av de samme
//! primitivene som husene, og står på bakkehøyden.

use super::{at, Builder, Facade, Shape, IRON, OAK, STONE, WHITE_TRIM};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec2, Vec3};

const TIRE: Mat = Mat::plain(0x1c1c1c, 0.9);
const CHROME: Mat = Mat::tex(Layer::Metal, 0xffffff);
const DARK: Mat = Mat::plain(0x24262a, 0.55);
const CAR_GLASS: Mat = Mat::plain(0x1b232c, 0.04);

/// Lakkfargene: dempede, slik biler i en nordisk småby er.
const PAINTS: [u32; 8] = [0x2b3a55, 0x8c2f2f, 0xe6dfc8, 0x2f4a3a, 0xb9bcc0, 0xc9a13b, 0xeeeeea, 0x5b4a3c];

#[derive(Clone, Copy, PartialEq)]
pub enum CarStyle {
    /// Kantete stasjonsvogn, à la Volvo 240.
    Estate,
    /// Liten kombi.
    Hatch,
    /// Varebil med flat front.
    Van,
}

/// Et rør mellom to punkter.
fn tube(m: &mut Mesh, t: &Affine3A, a: Vec3, b: Vec3, r: f32, mat: &Mat) {
    let d = b - a;
    let len = d.length();
    if len < 1e-4 {
        return;
    }
    let rot = Quat::from_rotation_arc(Vec3::Y, d / len);
    m.cylinder(&(*t * Affine3A::from_rotation_translation(rot, a)), r, r, len, 8, mat, false);
}

/// En rute i sideprofilen: fra `p0` til `p1` (x langs bilen, y opp), tvers
/// over bilen, skjøvet litt utenfor karosseriet.
fn pane(m: &mut Mesh, t: &Affine3A, p0: Vec2, p1: Vec2, half_w: f32, mat: &Mat) {
    let d = (p1 - p0).normalize();
    let n = Vec3::new(d.y, -d.x, 0.0);
    // Normalen skal peke ut fra bilen: opp for taket, fram for frontruta.
    let n = if n.y < 0.0 { -n } else { n };
    let off = n * 0.012;
    let a = Vec3::new(p0.x, p0.y, -half_w) + off;
    let b = Vec3::new(p0.x, p0.y, half_w) + off;
    let c = Vec3::new(p1.x, p1.y, half_w) + off;
    let dd = Vec3::new(p1.x, p1.y, -half_w) + off;
    let cross = (b - a).cross(dd - a);
    if cross.dot(n) >= 0.0 {
        m.quad(t, [a, b, c, dd], mat, [1.0; 4]);
    } else {
        m.quad(t, [a, dd, c, b], mat, [1.0; 4]);
    }
}

/// Et hjul med dekk, felg og navkapsel, aksen langs Z.
fn wheel(m: &mut Mesh, t: &Affine3A, c: Vec3, r: f32, width: f32, side: f32) {
    let rot = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let wt = *t * Affine3A::from_rotation_translation(rot, c - Vec3::Z * width * 0.5);
    m.cylinder(&wt, r, r, width, 18, &TIRE, true);
    // Felgen ytterst, litt innenfor dekket.
    let hub = Mat::plain(0xa8aaae, 0.35);
    let face = c + Vec3::Z * (side * (width * 0.5 + 0.004));
    m.cylinder(&(*t * Affine3A::from_rotation_translation(rot, face - Vec3::Z * 0.01)), r * 0.62, r * 0.62, 0.02, 14, &hub, true);
    m.cylinder(&(*t * Affine3A::from_rotation_translation(rot, face + Vec3::Z * side * 0.012)), r * 0.2, r * 0.2, 0.03, 10, &DARK, true);
}

/// En parkert bil. `yaw` er retningen fronten peker. Modellene fra
/// `tools/models/cars.py` brukes når de finnes, ellers den ekstruderte.
pub fn car(b: &mut Builder, p: Vec3, yaw: f32, style: CarStyle, paint: u32) {
    let model = match style {
        CarStyle::Estate => "car_estate",
        CarStyle::Hatch => "car_hatch",
        CarStyle::Van => "car_van",
    };
    if b.has_model(model) {
        let p = b.on_ground(p);
        let lin = |v: u32| {
            let s = v as f32 / 255.0;
            if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        let tint = Vec3::new(lin(paint >> 16 & 255), lin(paint >> 8 & 255), lin(paint & 255));
        b.place_tinted(model, at(p, yaw), tint);
        let (len, w, h) = match style {
            CarStyle::Estate => (4.8, 1.75, 1.5),
            CarStyle::Hatch => (3.9, 1.7, 1.5),
            CarStyle::Van => (4.9, 1.95, 2.0),
        };
        b.solid_box(p + Vec3::Y * (h * 0.5), Vec3::new(len, h, w), yaw);
        return;
    }
    car_extruded(b, p, yaw, style, paint);
}

/// Den prosedyriske bilen: sideprofil ekstrudert til bredden.
fn car_extruded(b: &mut Builder, p: Vec3, yaw: f32, style: CarStyle, paint: u32) {
    let (profile, half_w, wheel_x, wheel_r): (Vec<Vec2>, f32, [f32; 2], f32) = match style {
        CarStyle::Estate => (
            vec![
                Vec2::new(-2.3, 0.3),
                Vec2::new(2.3, 0.3),
                Vec2::new(2.42, 0.42),
                Vec2::new(2.42, 0.64),
                Vec2::new(2.36, 0.82),
                Vec2::new(2.28, 0.94),
                Vec2::new(0.95, 1.02),
                Vec2::new(0.4, 1.46),
                Vec2::new(-1.45, 1.49),
                Vec2::new(-2.15, 1.32),
                Vec2::new(-2.36, 1.0),
                Vec2::new(-2.42, 0.64),
                Vec2::new(-2.42, 0.42),
                Vec2::new(-2.3, 0.3),
            ],
            0.86,
            [-1.45, 1.45],
            0.32,
        ),
        CarStyle::Hatch => (
            vec![
                Vec2::new(-1.85, 0.3),
                Vec2::new(1.85, 0.3),
                Vec2::new(1.98, 0.42),
                Vec2::new(1.98, 0.62),
                Vec2::new(1.9, 0.86),
                Vec2::new(0.75, 0.96),
                Vec2::new(0.25, 1.42),
                Vec2::new(-0.95, 1.45),
                Vec2::new(-1.65, 1.18),
                Vec2::new(-1.92, 0.8),
                Vec2::new(-1.98, 0.6),
                Vec2::new(-1.98, 0.42),
                Vec2::new(-1.85, 0.3),
            ],
            0.81,
            [-1.2, 1.2],
            0.3,
        ),
        CarStyle::Van => (
            vec![
                Vec2::new(-2.25, 0.32),
                Vec2::new(2.1, 0.32),
                Vec2::new(2.3, 0.5),
                Vec2::new(2.3, 0.92),
                Vec2::new(2.15, 1.12),
                Vec2::new(1.9, 1.98),
                Vec2::new(-2.2, 2.02),
                Vec2::new(-2.3, 0.5),
                Vec2::new(-2.25, 0.32),
            ],
            0.95,
            [-1.5, 1.45],
            0.34,
        ),
    };
    // Bilen følger hellingen mellom akslene.
    let dir = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let g = |x: f32| {
        let q = p + dir * x;
        b.ground(q.x, q.z)
    };
    let (hr, hf) = (g(wheel_x[0]), g(wheel_x[1]));
    let pitch = ((hf - hr) / (wheel_x[1] - wheel_x[0])).atan();
    let base = Vec3::new(p.x, (hr + hf) * 0.5 - 0.02, p.z);
    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw) * Quat::from_rotation_z(pitch), base);
    let m = &mut b.mesh;
    let body = Mat::plain(paint, 0.22).flags(flag::NO_DIRT);

    // Karosseriet: sideprofilen dratt tvers over. `extrude` jobber i
    // verdenskoordinater, så det bygges lokalt og flyttes etterpå.
    let mut shell = Mesh::default();
    shell.extrude(&profile, Vec3::new(0.0, 0.0, -half_w), Vec3::new(0.0, 0.0, half_w), Vec3::X, &body, true);
    shell.transform(&t);
    m.append(&shell);

    // Ruter.
    match style {
        CarStyle::Estate => {
            pane(m, &t, Vec2::new(0.95, 1.02), Vec2::new(0.4, 1.46), half_w * 0.9, &CAR_GLASS);
            pane(m, &t, Vec2::new(-1.45, 1.49), Vec2::new(-2.15, 1.32), half_w * 0.9, &CAR_GLASS);
            for side in [-1.0f32, 1.0] {
                for (x0, x1) in [(-2.05, -1.5), (-1.35, -0.35), (-0.2, 0.42)] {
                    let w = x1 - x0;
                    m.cuboid(&(t * Affine3A::from_translation(Vec3::new((x0 + x1) * 0.5, 1.22, side * (half_w + 0.004)))), Vec3::new(w, 0.36, 0.01), &CAR_GLASS);
                }
            }
            // Takgrind.
            for side in [-0.6f32, 0.6] {
                m.cuboid(&(t * Affine3A::from_translation(Vec3::new(-0.5, 1.55, side))), Vec3::new(1.7, 0.05, 0.04), &DARK);
            }
        }
        CarStyle::Hatch => {
            pane(m, &t, Vec2::new(0.75, 0.96), Vec2::new(0.25, 1.42), half_w * 0.9, &CAR_GLASS);
            pane(m, &t, Vec2::new(-0.95, 1.45), Vec2::new(-1.65, 1.18), half_w * 0.9, &CAR_GLASS);
            for side in [-1.0f32, 1.0] {
                for (x0, x1) in [(-1.3, -0.35), (-0.2, 0.3)] {
                    let w = x1 - x0;
                    m.cuboid(&(t * Affine3A::from_translation(Vec3::new((x0 + x1) * 0.5, 1.19, side * (half_w + 0.004)))), Vec3::new(w, 0.34, 0.01), &CAR_GLASS);
                }
            }
        }
        CarStyle::Van => {
            pane(m, &t, Vec2::new(2.15, 1.12), Vec2::new(1.9, 1.98), half_w * 0.9, &CAR_GLASS);
            for side in [-1.0f32, 1.0] {
                m.cuboid(&(t * Affine3A::from_translation(Vec3::new(1.35, 1.5, side * (half_w + 0.004)))), Vec3::new(0.9, 0.55, 0.01), &CAR_GLASS);
            }
            // Bakdørene markeres med en fuge.
            m.cuboid(&(t * Affine3A::from_translation(Vec3::new(-2.305, 1.15, 0.0))), Vec3::new(0.01, 1.4, 0.02), &DARK);
        }
    }

    // Hjul.
    for &x in &wheel_x {
        for side in [-1.0f32, 1.0] {
            wheel(m, &t, Vec3::new(x, wheel_r, side * (half_w - 0.12)), wheel_r, 0.2, side);
        }
    }

    // Støtfangere, grill, lykter, speil og skilt.
    let (front, back) = (profile[1].x + 0.06, profile[0].x - 0.06);
    let bumper_y = if style == CarStyle::Van { 0.45 } else { 0.42 };
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(front, bumper_y, 0.0))), Vec3::new(0.08, 0.12, half_w * 2.0 + 0.06), &CHROME);
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(back, bumper_y, 0.0))), Vec3::new(0.08, 0.12, half_w * 2.0 + 0.06), &CHROME);
    let lamp_y = if style == CarStyle::Van { 0.8 } else { 0.72 };
    let head = Mat::plain(0xe9ecd8, 0.1);
    let tail = Mat::plain(0x9a1a1a, 0.2);
    for side in [-1.0f32, 1.0] {
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(front - 0.03, lamp_y, side * (half_w - 0.22)))), Vec3::new(0.03, 0.14, 0.26), &head);
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(back + 0.03, lamp_y, side * (half_w - 0.2)))), Vec3::new(0.03, 0.12, 0.22), &tail);
        // Speil ved A-stolpen.
        let mx = match style {
            CarStyle::Estate => 0.7,
            CarStyle::Hatch => 0.55,
            CarStyle::Van => 1.75,
        };
        let my = if style == CarStyle::Van { 1.4 } else { 1.12 };
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(mx, my, side * (half_w + 0.1)))), Vec3::new(0.08, 0.1, 0.16), &body);
    }
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(front - 0.03, lamp_y, 0.0))), Vec3::new(0.03, 0.16, half_w * 0.9), &DARK);
    let plate = Mat::plain(0xf2f2ee, 0.4);
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(front + 0.02, 0.56, 0.0))), Vec3::new(0.01, 0.11, 0.5), &plate);
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(back - 0.02, 0.56, 0.0))), Vec3::new(0.01, 0.11, 0.5), &plate);
    // Dørhåndtak og fuger.
    for side in [-1.0f32, 1.0] {
        let xs: &[f32] = match style {
            CarStyle::Estate => &[-1.4, -0.3],
            CarStyle::Hatch => &[-0.3],
            CarStyle::Van => &[0.9],
        };
        for &x in xs {
            m.cuboid(&(t * Affine3A::from_translation(Vec3::new(x, 0.95, side * (half_w + 0.012)))), Vec3::new(0.14, 0.03, 0.02), &CHROME);
            m.cuboid(&(t * Affine3A::from_translation(Vec3::new(x + 0.5, 0.66, side * (half_w + 0.003)))), Vec3::new(0.01, 0.66, 0.005), &DARK);
        }
    }

    let len = profile[1].x - profile[0].x + 0.2;
    let h = profile.iter().map(|p| p.y).fold(0.0, f32::max);
    b.solid_box(base + Vec3::Y * h * 0.5, Vec3::new(len, h, half_w * 2.0 + 0.2), yaw);
}

/// En sykkel. `yaw` er framover, `lean` heller den sidelengs (radianer).
pub fn bicycle(b: &mut Builder, p: Vec3, yaw: f32, lean: f32, paint: u32) {
    let p = b.on_ground(p);
    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw) * Quat::from_rotation_x(lean), p);
    let m = &mut b.mesh;
    let frame = Mat::plain(paint, 0.3).flags(flag::NO_DIRT);
    let r = 0.34;
    let rear = Vec3::new(-0.52, r, 0.0);
    let front = Vec3::new(0.52, r, 0.0);
    let vertical = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    for c in [rear, front] {
        m.torus(&(t * Affine3A::from_rotation_translation(vertical, c)), r, 0.022, 24, 6, &TIRE);
        m.torus(&(t * Affine3A::from_rotation_translation(vertical, c)), r - 0.03, 0.008, 24, 5, &CHROME);
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let rim = c + Vec3::new(a.cos(), a.sin(), 0.0) * (r - 0.03);
            tube(m, &t, c, rim, 0.003, &CHROME);
        }
        m.cylinder(&(t * Affine3A::from_rotation_translation(vertical, c - Vec3::Z * 0.04)), 0.03, 0.03, 0.08, 8, &DARK, true);
    }
    // Ramma.
    let bb = Vec3::new(-0.05, 0.3, 0.0);
    let seat = Vec3::new(-0.2, 0.92, 0.0);
    let head_top = Vec3::new(0.36, 0.9, 0.0);
    let head_bot = Vec3::new(0.44, 0.58, 0.0);
    tube(m, &t, bb, seat, 0.016, &frame);
    tube(m, &t, seat, head_top, 0.016, &frame);
    tube(m, &t, bb, head_bot, 0.018, &frame);
    tube(m, &t, head_top, head_bot, 0.018, &frame);
    for side in [-0.045f32, 0.045] {
        tube(m, &t, rear + Vec3::Z * side, bb + Vec3::Z * side * 0.6, 0.011, &frame);
        tube(m, &t, rear + Vec3::Z * side, seat + Vec3::Z * side * 0.4, 0.011, &frame);
        tube(m, &t, head_bot + Vec3::Z * side * 0.6, front + Vec3::Z * side, 0.011, &frame);
    }
    // Styre, sete, pedaler og kjede.
    tube(m, &t, head_top, head_top + Vec3::new(-0.02, 0.1, 0.0), 0.014, &CHROME);
    let bar = head_top + Vec3::new(-0.02, 0.1, 0.0);
    tube(m, &t, bar - Vec3::Z * 0.27, bar + Vec3::Z * 0.27, 0.012, &CHROME);
    for side in [-1.0f32, 1.0] {
        tube(m, &t, bar + Vec3::Z * side * 0.27, bar + Vec3::new(-0.1, 0.0, side * 0.27), 0.016, &DARK);
    }
    tube(m, &t, seat, seat + Vec3::new(-0.02, 0.1, 0.0), 0.012, &CHROME);
    m.rounded_slab(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), seat + Vec3::new(-0.05, 1.03 - seat.y, 0.0))), Vec3::ZERO, 0.26, 0.14, 0.05, 0.05, &DARK);
    m.cylinder(&(t * Affine3A::from_rotation_translation(vertical, bb - Vec3::Z * 0.09)), 0.012, 0.012, 0.18, 6, &CHROME, true);
    m.torus(&(t * Affine3A::from_rotation_translation(vertical, bb + Vec3::Z * 0.055)), 0.085, 0.006, 20, 4, &CHROME);
    for (side, a) in [(-1.0f32, 0.9f32), (1.0, 0.9 + std::f32::consts::PI)] {
        let crank = bb + Vec3::new(a.cos(), a.sin(), 0.0) * 0.16;
        tube(m, &t, bb + Vec3::Z * side * 0.09, crank + Vec3::Z * side * 0.09, 0.008, &CHROME);
        m.cuboid(&(t * Affine3A::from_translation(crank + Vec3::Z * side * 0.13)), Vec3::new(0.09, 0.02, 0.07), &DARK);
    }
    tube(m, &t, bb + Vec3::Z * 0.055 + Vec3::Y * 0.085, rear + Vec3::Z * 0.055 + Vec3::Y * 0.04, 0.004, &DARK);
    tube(m, &t, bb + Vec3::Z * 0.055 - Vec3::Y * 0.085, rear + Vec3::Z * 0.055 - Vec3::Y * 0.04, 0.004, &DARK);
    // Skjermer.
    for c in [rear, front] {
        m.torus(&(t * Affine3A::from_rotation_translation(vertical, c + Vec3::Y * 0.0)), r + 0.03, 0.004, 24, 4, &frame);
    }
    b.solid_box(p + Vec3::Y * 0.5, Vec3::new(1.7, 1.0, 0.3), yaw);
}

/// Kafébord med stoler og parasoll.
pub fn cafe_set(b: &mut Builder, p: Vec3, yaw: f32, parasol: bool) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    let green = Mat::plain(0x2c4a3e, 0.45);
    let m = &mut b.mesh;
    m.lathe(&t, &[Vec2::new(0.22, 0.0), Vec2::new(0.22, 0.02), Vec2::new(0.03, 0.04), Vec2::new(0.03, 0.72)], 12, &green, true, 0.8);
    m.cylinder(&(t * Affine3A::from_translation(Vec3::Y * 0.72)), 0.34, 0.34, 0.03, 20, &OAK, true);
    for k in 0..3 {
        let a = k as f32 * 2.1 + 0.4;
        let c = Vec3::new(a.cos() * 0.62, 0.0, a.sin() * 0.62);
        let ct = t * Affine3A::from_rotation_translation(Quat::from_rotation_y(-a + std::f32::consts::FRAC_PI_2), c);
        m.cuboid(&(ct * Affine3A::from_translation(Vec3::new(0.0, 0.45, 0.0))), Vec3::new(0.38, 0.025, 0.38), &OAK);
        m.cuboid(&(ct * Affine3A::from_rotation_translation(Quat::from_rotation_x(0.12), Vec3::new(0.0, 0.68, 0.18))), Vec3::new(0.36, 0.42, 0.02), &OAK);
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            m.cuboid(&(ct * Affine3A::from_translation(Vec3::new(sx * 0.17, 0.22, sz * 0.17))), Vec3::new(0.02, 0.44, 0.02), &green);
        }
    }
    if parasol {
        let canvas = Mat::plain(0xe9e0c8, 0.7).flags(flag::NO_DIRT);
        m.cylinder(&(t * Affine3A::from_translation(Vec3::Y * 0.74)), 0.02, 0.02, 1.6, 8, &green, false);
        m.cylinder(&(t * Affine3A::from_translation(Vec3::Y * 2.0)), 1.15, 0.0, 0.4, 8, &canvas, false);
        m.cylinder(&(t * Affine3A::from_translation(Vec3::Y * 1.99)), 1.15, 0.0, 0.4, 8, &Mat::plain(0xd8cfb8, 0.8), false);
        m.sphere(&(t * Affine3A::from_translation(Vec3::Y * 2.42)), Vec3::splat(0.03), 4, 6, &green);
    }
    b.solid(Shape::Cylinder { center: p + Vec3::Y * 0.4, radius: 0.9, half_height: 0.4 });
}

/// Torgbod med stripet seilduk og frukt i kasser.
pub fn market_stall(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    let m = &mut b.mesh;
    let wood = Mat::tex(Layer::Oak, 0xffffff).flags(flag::ROT_UV);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::Y * 0.9)), Vec3::new(2.6, 0.06, 1.2), &wood, 1.0);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.5, 0.55))), Vec3::new(2.6, 0.8, 0.04), &wood, 1.0);
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let h = if sz < 0.0 { 2.5 } else { 2.2 };
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(sx * 1.22, h * 0.5, sz * 0.55))), Vec3::new(0.08, h, 0.08), &wood, 1.0);
    }
    // Seilduken i striper, skrå framover.
    let tilt = Affine3A::from_rotation_x(0.2);
    for k in 0..8 {
        let c = if k % 2 == 0 { 0xc23b3b } else { 0xf2ede2 };
        let stripe = Mat::plain(c, 0.75).flags(flag::NO_DIRT);
        let x = -1.4 + 0.4 * k as f32;
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(x, 2.4, 0.1)) * tilt), Vec3::new(0.4, 0.02, 1.8), &stripe);
    }
    // Kasser med frukt.
    let fruits = [0xc8322b, 0x8fbf4a, 0xe8912b, 0xd9b53a];
    for (k, x) in [-0.85f32, -0.1, 0.65].into_iter().enumerate() {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, 1.0, 0.0))), Vec3::new(0.6, 0.18, 0.4), &OAK, 1.0);
        for i in 0..12 {
            let fx = x - 0.22 + (i % 4) as f32 * 0.14;
            let fz = -0.12 + (i / 4) as f32 * 0.12;
            let fy = 1.1 + if i % 3 == 0 { 0.03 } else { 0.0 };
            m.sphere(&(t * Affine3A::from_translation(Vec3::new(fx, fy, fz))), Vec3::splat(0.045), 5, 7, &Mat::plain(fruits[(k + i / 5) % 4], 0.5));
        }
    }
    b.solid_box(p + Vec3::Y * 0.5, Vec3::new(2.7, 1.0, 1.3), yaw);
}

/// Lyslenke mellom to punkter: en hengende ledning med pærer som lyser om natta.
pub fn string_lights(b: &mut Builder, a: Vec3, c: Vec3, sag: f32) {
    let m = &mut b.mesh;
    let wire = Mat::plain(0x1a1a1a, 0.8);
    let bulb = Mat::special(kind::LAMP, 0xffd79a, 0.3);
    let n = ((c - a).length() / 0.6).ceil().max(2.0) as i32;
    let point = |k: i32| {
        let u = k as f32 / n as f32;
        a.lerp(c, u) - Vec3::Y * (sag * 4.0 * u * (1.0 - u))
    };
    for k in 0..n {
        let (p0, p1) = (point(k), point(k + 1));
        tube(m, &Affine3A::IDENTITY, p0, p1, 0.006, &wire);
        if k % 2 == 1 {
            let q = p0 - Vec3::Y * 0.06;
            tube(m, &Affine3A::IDENTITY, p0, q, 0.006, &wire);
            m.sphere(&Affine3A::from_translation(q - Vec3::Y * 0.04), Vec3::splat(0.045), 5, 8, &bulb);
        }
    }
}

/// Gateskilt: stolpe med blått skilt og hvit tekst, teksten vendt mot `yaw`.
pub fn street_sign(b: &mut Builder, p: Vec3, yaw: f32, name: &str) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    let blue = Mat::plain(0x1f4e9c, 0.4).flags(flag::NO_DIRT);
    let white = Mat::plain(0xf6f6f2, 0.5).flags(flag::NO_DIRT);
    let w = b.font.width(name, 16.0) * 0.012 + 0.2;
    let m = &mut b.mesh;
    m.cylinder(&t, 0.03, 0.03, 2.5, 8, &IRON, false);
    m.rounded_slab(&(t * Affine3A::from_translation(Vec3::new(0.0, 2.35, 0.03))), Vec3::ZERO, w, 0.24, 0.03, 0.03, &blue);
    let mut text = Mesh::default();
    b.font.centered(&mut text, name, Vec3::ZERO, 16.0, 0.012, &white);
    text.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, 2.35, 0.05))));
    m.append(&text);
    let mut back = Mesh::default();
    b.font.centered(&mut back, name, Vec3::ZERO, 16.0, 0.012, &white);
    back.transform(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_y(std::f32::consts::PI), Vec3::new(0.0, 2.35, 0.01))));
    m.append(&back);
}

/// Postkasse: rød, på stolpe.
pub fn mailbox(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    let red = Mat::plain(0xc8102e, 0.35).flags(flag::NO_DIRT);
    let m = &mut b.mesh;
    m.cylinder(&t, 0.04, 0.04, 0.95, 8, &IRON, false);
    m.rounded_slab(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.2, 0.0))), Vec3::ZERO, 0.46, 0.52, 0.34, 0.06, &red);
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.36, 0.175))), Vec3::new(0.3, 0.025, 0.01), &DARK);
    let mut text = Mesh::default();
    b.font.centered(&mut text, "POST", Vec3::ZERO, 10.0, 0.012, &Mat::plain(0xf6f0e6, 0.5));
    text.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.14, 0.172))));
    m.append(&text);
    b.solid_box(p + Vec3::Y * 0.75, Vec3::new(0.5, 1.5, 0.4), yaw);
}

/// Busskur i tre med benk og skilt.
pub fn bus_shelter(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    let wood = Mat::tex(Layer::Cladding, 0xffffff).tint(Vec3::new(0.45, 0.28, 0.2)).flags(flag::ROT_UV);
    let m = &mut b.mesh;
    // Bakveggen står i z = -d, åpningen vender mot +Z.
    let (w, d, h) = (3.0, 1.6, 2.4);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, h * 0.5, -d))), Vec3::new(w, h, 0.06), &wood, 1.0);
    for side in [-1.0f32, 1.0] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(side * (w * 0.5 - 0.03), h * 0.5, -d * 0.5))), Vec3::new(0.06, h, d), &wood, 1.0);
    }
    m.cuboid_ao(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_x(0.08), Vec3::new(0.0, h + 0.08, -d * 0.5 + 0.1))), Vec3::new(w + 0.3, 0.08, d + 0.5), &Mat::tex(Layer::Slate, 0xffffff), 1.0);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.46, -d + 0.3))), Vec3::new(w - 0.3, 0.05, 0.4), &OAK, 1.0);
    for side in [-1.0f32, 1.0] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(side * (w * 0.5 - 0.3), 0.22, -d + 0.3))), Vec3::new(0.06, 0.44, 0.36), &IRON, 1.0);
    }
    // Stoppskiltet.
    let sp = Vec3::new(w * 0.5 + 0.6, 0.0, 0.3);
    m.cylinder(&(t * Affine3A::from_translation(sp)), 0.03, 0.03, 2.6, 8, &IRON, false);
    let yellow = Mat::plain(0xf0c020, 0.4).flags(flag::NO_DIRT);
    m.cylinder(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), sp + Vec3::new(0.0, 2.4, 0.02))), 0.3, 0.3, 0.03, 20, &yellow, true);
    let mut text = Mesh::default();
    b.font.centered(&mut text, "BUSS", Vec3::ZERO, 14.0, 0.012, &DARK);
    text.transform(&(t * Affine3A::from_translation(sp + Vec3::new(0.0, 2.4, 0.045))));
    m.append(&text);
    let q = Quat::from_rotation_y(yaw);
    b.solid_box(p + q * Vec3::new(0.0, h * 0.5, -d), Vec3::new(w, h, 0.1), yaw);
    for side in [-1.0f32, 1.0] {
        b.solid_box(p + q * Vec3::new(side * (w * 0.5 - 0.03), h * 0.5, -d * 0.5), Vec3::new(0.1, h, d), yaw);
    }
    b.solid_box(p + q * Vec3::new(0.0, 0.3, -d + 0.3), Vec3::new(w, 0.6, 0.4), yaw);
}

/// Blomsterkrukke i terrakotta med grønt og blomster.
pub fn flower_pot(m: &mut Mesh, t: &Affine3A, size: f32, flower: u32) {
    let clay = Mat::plain(0xb0623f, 0.8);
    m.lathe(t, &[Vec2::new(size * 0.6, 0.0), Vec2::new(size * 0.8, size * 0.9), Vec2::new(size * 0.85, size * 0.9), Vec2::new(size * 0.85, size * 1.0), Vec2::new(size * 0.5, size * 1.0)], 12, &clay, true, 0.7);
    let leaf = Mat::plain(0x3f6b2a, 0.8);
    m.sphere(&(*t * Affine3A::from_translation(Vec3::Y * size * 1.1)), Vec3::new(size * 0.8, size * 0.5, size * 0.8), 5, 8, &leaf);
    for k in 0..5 {
        let a = k as f32 * 1.3;
        m.sphere(&(*t * Affine3A::from_translation(Vec3::new(a.cos() * size * 0.45, size * 1.45, a.sin() * size * 0.45))), Vec3::splat(size * 0.14), 4, 6, &Mat::plain(flower, 0.6));
    }
}

/// Tøy på snor mellom to punkter.
pub fn laundry(m: &mut Mesh, a: Vec3, c: Vec3, rng: &mut super::Rng) {
    let wire = Mat::plain(0x3a3a3a, 0.8);
    tube(m, &Affine3A::IDENTITY, a, c, 0.005, &wire);
    let along = (c - a).normalize();
    let len = (c - a).length();
    let colors = [0xf2efe8, 0x8fb4d9, 0xf0c9a0, 0xd9a0b0, 0xa8c8a0, 0xf6f1d4];
    let mut s = 0.3;
    while s < len - 0.6 {
        let w = rng.range(0.35, 0.6);
        let h = rng.range(0.5, 0.9);
        let p = a + along * (s + w * 0.5) - Vec3::Y * h * 0.5;
        let yaw = along.x.atan2(along.z) + std::f32::consts::FRAC_PI_2;
        let cloth = Mat::plain(rng.pick(&colors), 0.85).flags(flag::NO_DIRT | flag::WIND);
        m.cuboid(&Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p), Vec3::new(w, h, 0.012), &cloth);
        s += w + rng.range(0.1, 0.35);
    }
}

/// Alt gatelivet på plass: biler i gatene og langs torget, sykler ved
/// stativene og inntil veggene, kafé, bod, lyslenker, skilt og postkasse.
pub fn all(b: &mut Builder) {
    let pi = std::f32::consts::PI;
    // Biler.
    let cars: [(f32, f32, f32, CarStyle); 10] = [
        (-3.3, -31.0, 0.0, CarStyle::Estate),
        (-3.3, -37.5, 0.0, CarStyle::Hatch),
        (-3.3, -45.0, pi, CarStyle::Estate),
        (2.7, 31.5, pi, CarStyle::Hatch),
        (2.7, 38.5, 0.0, CarStyle::Estate),
        (16.3, -8.0, 0.0, CarStyle::Hatch),
        (16.3, 6.0, pi, CarStyle::Estate),
        (-16.3, -10.0, pi, CarStyle::Hatch),
        (36.0, -2.6, -pi * 0.5, CarStyle::Van),
        (25.5, 2.6, pi * 0.5, CarStyle::Estate),
    ];
    for (i, (x, z, yaw, style)) in cars.into_iter().enumerate() {
        let paint = PAINTS[(i * 3 + 1) % PAINTS.len()];
        car(b, Vec3::new(x, 0.0, z), yaw, style, paint);
    }

    // Sykler i stativene.
    let bike_paints = [0x2f4a7a, 0x7a2f2f, 0x2a2a2a, 0x5a7a3a, 0xd9c9a0, 0x8a4a2a];
    for (k, (x, z)) in [(10.4, -15.0), (11.6, -15.0), (-11.6, 15.0), (-10.4, 15.0)].into_iter().enumerate() {
        bicycle(b, Vec3::new(x, 0.0, z), pi * 0.5, 0.0, bike_paints[k % bike_paints.len()]);
    }
    // Sykler lent mot veggene, og krukker ved dørene.
    let facades: Vec<Facade> = b.facades.clone();
    let mut n_bikes = 0;
    for f in facades.iter() {
        if f.width < 6.0 {
            continue;
        }
        let wants_bike = b.rng.chance(0.28) && n_bikes < 9;
        if wants_bike {
            // Et stykke langs fasaden som ikke er døra.
            let mut u = b.rng.range(-f.width * 0.5 + 1.2, f.width * 0.5 - 1.2);
            if (u - f.door).abs() < 1.3 {
                u = if f.door > 0.0 { f.door - 1.6 } else { f.door + 1.6 };
            }
            let p = f.origin + f.along * u + f.facing * 0.38;
            let yaw = f.along.x.atan2(f.along.z);
            // Len toppen inn mot veggen.
            let wall_local = Quat::from_rotation_y(-yaw) * (-f.facing);
            let lean = 0.2 * wall_local.z.signum();
            let paint = b.rng.pick(&bike_paints);
            bicycle(b, p, yaw, lean, paint);
            n_bikes += 1;
        }
        if !f.shop && b.rng.chance(0.45) {
            let side = if b.rng.chance(0.5) { 1.0 } else { -1.0 };
            let p = f.origin + f.along * (f.door + side * 0.85) + f.facing * 0.5;
            let p = b.on_ground(p);
            let size = b.rng.range(0.22, 0.32);
            let flower = b.rng.pick(&[0xd84a5a, 0xf2c14e, 0xe9e2d0, 0xc05aa0, 0xf07f3c]);
            flower_pot(&mut b.mesh, &Affine3A::from_translation(p), size, flower);
        }
    }

    // Kafé på torget, utenfor husene nordvest og sørøst.
    cafe_set(b, Vec3::new(-6.5, 0.0, -14.4), 0.3, true);
    cafe_set(b, Vec3::new(-4.3, 0.0, -14.2), 1.4, false);
    cafe_set(b, Vec3::new(4.5, 0.0, 14.2), 0.9, true);
    cafe_set(b, Vec3::new(2.4, 0.0, 14.4), 2.2, false);

    // Torgboder.
    market_stall(b, Vec3::new(10.5, 0.0, -5.5), 0.6);
    market_stall(b, Vec3::new(-9.0, 0.0, -5.0), 2.1);

    // Lyslenker mellom lyktene rundt torget, i en åttekant.
    let top = if b.has_model("lamp") { b.model_height("lamp") - 0.55 } else { 3.0 };
    let posts = [(-9.0, -15.5), (9.0, -15.5), (17.5, -6.0), (17.5, 6.0), (9.0, 15.5), (-9.0, 15.5), (-17.5, 6.0), (-17.5, -6.0)];
    for k in 0..posts.len() {
        let (x0, z0) = posts[k];
        let (x1, z1) = posts[(k + 1) % posts.len()];
        let a = b.on_ground(Vec3::new(x0, top, z0));
        let c = b.on_ground(Vec3::new(x1, top, z1));
        string_lights(b, a, c, 0.6);
    }

    // Gateskilt ved gatemunningene, postkasse og busskur.
    street_sign(b, Vec3::new(-6.6, 0.0, -18.2), 0.0, "Nordgata");
    street_sign(b, Vec3::new(5.6, 0.0, 18.2), pi, "Sørgata");
    street_sign(b, Vec3::new(18.6, 0.0, -5.4), pi * 0.5, "Østre gate");
    street_sign(b, Vec3::new(-18.6, 0.0, 5.4), -pi * 0.5, "Vestre gate");
    mailbox(b, Vec3::new(13.6, 0.0, -16.6), 0.0);
    bus_shelter(b, Vec3::new(46.0, 0.0, -4.6), 0.0);

    // Søppelbøtter ved benkene om modellen finnes.
    if b.has_model("bin") {
        for (x, z, yaw) in [(-8.0, 5.2, 0.7), (8.4, -5.0, -0.5)] {
            let p = b.on_ground(Vec3::new(x, 0.0, z));
            b.place("bin", at(p, yaw));
            b.colliders.push(Shape::Cylinder { center: p + Vec3::Y * 0.5, radius: 0.3, half_height: 0.5 });
        }
    }
    let _ = (STONE, WHITE_TRIM);
}
