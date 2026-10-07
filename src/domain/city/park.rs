//! Tokenparken: parken vest for byen der Fristils tokens står i jorda.
//! Fargebedene viser hver palett (accent, brand1 …) som en rad blomster i
//! trinnene surface → content, avstandsskalaen står som steinblokker der
//! høyden er trinnet, og en dam med benk ligger i sørvest. Parken henger på
//! enden av grusstien vestover.

use super::{at, Builder, Shape, IRON, OAK, STONE};
use crate::domain::geometry::{flag, kind, Mat, Mesh};
use crate::domain::tokens::Theme;
use glam::{Affine3A, Quat, Vec2, Vec3};

/// Parkens rektangel i verden.
pub const X0: f32 = -80.0;
pub const X1: f32 = -46.0;
pub const Z0: f32 = -17.0;
pub const Z1: f32 = 17.0;

const PALETTES: [&str; 9] = ["accent", "brand1", "brand2", "brand3", "neutral", "success", "warning", "danger", "visited"];
const STEPS: [&str; 9] = ["surface", "border-subtle", "border", "border-strong", "fill", "text", "text-strong", "text-subtle", "content"];
const SPACING: [&str; 13] = ["px", "0-5", "1", "2", "3", "4", "5", "6", "7", "8", "10", "12", "16"];

fn rgb(c: Vec3) -> u32 {
    let q = |v: f32| ((v.clamp(0.0, 1.0)) * 255.0) as u32;
    (q(c.x) << 16) | (q(c.y) << 8) | q(c.z)
}

/// Et skilt på to stolper: tittel i 26 px, undertekst i 14 px. Plata er så
/// brei som teksten trenger (`min_w` er det minste), og stolpene står bak
/// plata, aldri foran teksten.
fn sign(b: &mut Builder, p: Vec3, yaw: f32, title: &str, sub: &str, min_w: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    let w = (b.font.width(title, 26.0) * 0.012).max(b.font.width(sub, 14.0) * 0.012) + 0.5;
    let w = w.max(min_w);
    b.fits("parkskilt", title, 26.0, 0.012, w);
    b.fits("parkskilt", sub, 14.0, 0.012, w);
    let m = &mut b.mesh;
    for dx in [-w * 0.5 + 0.15, w * 0.5 - 0.15] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.75, -0.07))), Vec3::new(0.08, 1.9, 0.08), &OAK, 1.0);
    }
    let board = Mat::tex(crate::domain::geometry::Layer::Oak, 0xffffff).tint(Vec3::new(0.5, 0.45, 0.38));
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.45, 0.0))), Vec3::new(w, 0.76, 0.05), &board, 1.0);
    let ink = Mat::plain(0xf4efe4, 0.5).flags(flag::NO_DIRT);
    let mut text = Mesh::default();
    b.font.centered(&mut text, title, Vec3::new(0.0, 0.12, 0.0), 26.0, 0.012, &ink);
    b.font.centered(&mut text, sub, Vec3::new(0.0, -0.17, 0.0), 14.0, 0.012, &ink);
    text.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.45, 0.03))));
    m.append(&text);
    b.solid_box(p + Vec3::Y * 0.95, Vec3::new(w, 1.9, 0.15), yaw);
}

/// Et fargebed: steinkant, jord, og ni blomsterklaser i palettens trinn,
/// med trinnets navn på en liten plate foran hver.
fn bed(b: &mut Builder, from: Vec3, palette: &str) {
    let pitch = 0.72;
    let len = pitch * STEPS.len() as f32 + 0.4;
    let c = from + Vec3::X * (len * 0.5);
    let c = b.on_ground(c);
    let soil = Mat::plain(0x3a2e24, 0.95);
    let m = &mut b.mesh;
    // Kanten: fire lave steinbjelker, sunket litt i bakken.
    for (dz, w, d) in [(0.65, len, 0.14), (-0.65, len, 0.14)] {
        m.cuboid_ao(&Affine3A::from_translation(Vec3::new(c.x, c.y + 0.06, c.z + dz)), Vec3::new(w, 0.5, d), &STONE, 1.0);
    }
    for dx in [-len * 0.5, len * 0.5] {
        m.cuboid_ao(&Affine3A::from_translation(Vec3::new(c.x + dx, c.y + 0.06, c.z)), Vec3::new(0.14, 0.5, 1.44), &STONE, 1.0);
    }
    m.cuboid_ao(&Affine3A::from_translation(Vec3::new(c.x, c.y + 0.0, c.z)), Vec3::new(len - 0.1, 0.44, 1.2), &soil, 1.0);
    b.solid_box(Vec3::new(c.x, c.y + 0.15, c.z), Vec3::new(len + 0.1, 0.3, 1.5), 0.0);
    for (k, step) in STEPS.iter().enumerate() {
        let name = format!("{palette}-{step}");
        let col = b.tokens.color(Theme::Light, &name);
        let x = c.x - len * 0.5 + 0.2 + pitch * (k as f32 + 0.5);
        let base = Vec3::new(x, c.y + 0.22, c.z);
        let m = &mut b.mesh;
        // Bladverket: to grønne kuler, og sju blomster i tokenfargen oppå.
        let leaf = Mat::plain(0x3f6b2a, 0.85);
        m.sphere(&Affine3A::from_translation(base + Vec3::new(0.0, 0.12, 0.0)), Vec3::new(0.3, 0.2, 0.3), 5, 8, &leaf);
        m.sphere(&Affine3A::from_translation(base + Vec3::new(0.12, 0.08, -0.14)), Vec3::new(0.2, 0.16, 0.2), 4, 7, &leaf);
        let petal = Mat::plain(rgb(col), 0.55).flags(flag::NO_DIRT);
        for j in 0..7 {
            let a = j as f32 * 0.9 + k as f32;
            let r = if j == 0 { 0.0 } else { 0.17 };
            let fp = base + Vec3::new(a.cos() * r, 0.26 + (j as f32 * 1.7).sin() * 0.05, a.sin() * r);
            m.cylinder(&Affine3A::from_translation(fp - Vec3::Y * 0.12), 0.008, 0.008, 0.12, 5, &leaf, false);
            m.sphere(&Affine3A::from_translation(fp), Vec3::new(0.065, 0.045, 0.065), 4, 7, &petal);
        }
        // Platen foran: trinnets navn i 10 px på messing, lagt skrått.
        let brass = Mat::plain(0xb8924a, 0.35).flags(flag::NO_DIRT);
        let pt = Affine3A::from_rotation_translation(Quat::from_rotation_x(-0.9), Vec3::new(x, c.y + 0.3, c.z + 0.78));
        let pw = (b.font.width(step, 10.0) * 0.012 + 0.16).max(0.6);
        let m = &mut b.mesh;
        m.cuboid_ao(&pt, Vec3::new(pw, 0.2, 0.012), &brass, 1.0);
        let mut label = Mesh::default();
        b.font.centered(&mut label, step, Vec3::ZERO, 10.0, 0.012, &Mat::plain(0x2a2418, 0.6).flags(flag::NO_DIRT));
        label.transform(&(pt * Affine3A::from_translation(Vec3::Z * 0.008)));
        b.mesh.append(&label);
    }
    // Palettens navn på en plate ved enden; stolpen står bak plata, så den
    // aldri går gjennom teksten (teksten på baksiden sitter ved siden av).
    let post = Vec3::new(c.x - len * 0.5 - 0.35, c.y, c.z);
    let m = &mut b.mesh;
    m.cylinder(&Affine3A::from_translation(post - Vec3::Z * 0.05), 0.03, 0.03, 1.08, 8, &IRON, false);
    let brass = Mat::plain(0xb8924a, 0.35).flags(flag::NO_DIRT);
    let pw = b.font.width(palette, 16.0) * 0.012 + 0.16;
    m.cuboid_ao(&Affine3A::from_translation(post + Vec3::new(0.0, 1.0, 0.0)), Vec3::new(pw, 0.3, 0.03), &brass, 1.0);
    let mut label = Mesh::default();
    b.font.centered(&mut label, palette, Vec3::ZERO, 16.0, 0.012, &Mat::plain(0x2a2418, 0.6).flags(flag::NO_DIRT));
    label.transform(&Affine3A::from_translation(post + Vec3::new(0.0, 1.0, 0.018)));
    let mut back = Mesh::default();
    b.font.centered(&mut back, palette, Vec3::ZERO, 16.0, 0.012, &Mat::plain(0x2a2418, 0.6).flags(flag::NO_DIRT));
    back.transform(&(Affine3A::from_translation(post + Vec3::new(0.0, 1.0, -0.018)) * Affine3A::from_rotation_y(std::f32::consts::PI)));
    b.mesh.append(&label);
    b.mesh.append(&back);
}

/// Avstandsskalaen: tretten steinblokker på en sokkel, der høyden er
/// trinnet i spacing-skalaen (2 cm per px), med navnet foran.
fn spacing_scale(b: &mut Builder, from: Vec3) {
    let pitch = 0.6;
    let len = pitch * SPACING.len() as f32 + 0.4;
    let c = b.on_ground(from + Vec3::Z * (len * 0.5));
    let m = &mut b.mesh;
    m.cuboid_ao(&Affine3A::from_translation(Vec3::new(c.x, c.y + 0.05, c.z)), Vec3::new(1.0, 0.5, len), &STONE, 1.0);
    b.solid_box(Vec3::new(c.x, c.y + 0.15, c.z), Vec3::new(1.0, 0.3, len), 0.0);
    let block = Mat::tex(crate::domain::geometry::Layer::Concrete, 0xffffff).tint(Vec3::new(0.85, 0.83, 0.8));
    for (k, step) in SPACING.iter().enumerate() {
        let px = b.tokens.spacing_px(step);
        let h = (px * 0.02).max(0.012);
        let z = c.z - len * 0.5 + 0.2 + pitch * (k as f32 + 0.5);
        let m = &mut b.mesh;
        m.cuboid_ao(&Affine3A::from_translation(Vec3::new(c.x, c.y + 0.3 + h * 0.5, z)), Vec3::new(0.5, h, 0.4), &block, 1.0);
        let brass = Mat::plain(0xb8924a, 0.35).flags(flag::NO_DIRT);
        let pt = Affine3A::from_rotation_translation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(-0.9), Vec3::new(c.x - 0.42, c.y + 0.32, z));
        let text = format!("{step}  {px:.0} px");
        let pw = (b.font.width(&text, 9.0) * 0.012 + 0.14).max(0.5);
        let m = &mut b.mesh;
        m.cuboid_ao(&pt, Vec3::new(pw, 0.18, 0.012), &brass, 1.0);
        let mut label = Mesh::default();
        b.font.centered(&mut label, &text, Vec3::ZERO, 9.0, 0.012, &Mat::plain(0x2a2418, 0.6).flags(flag::NO_DIRT));
        label.transform(&(pt * Affine3A::from_translation(Vec3::Z * 0.008)));
        b.mesh.append(&label);
    }
}

/// Dammen: steinkant, vann, siv og en benk.
fn pond(b: &mut Builder, p: Vec3) {
    let p = b.on_ground(p);
    let m = &mut b.mesh;
    let profile = [Vec2::new(3.4, -0.6), Vec2::new(3.4, 0.35), Vec2::new(3.1, 0.4), Vec2::new(2.8, 0.1), Vec2::new(0.0, 0.1)];
    m.lathe(&Affine3A::from_translation(p), &profile, 28, &STONE, false, 0.7);
    let water = Mat::special(kind::WATER, 0x203040, 0.02);
    m.cylinder(&Affine3A::from_translation(p + Vec3::Y * 0.22), 3.0, 3.0, 0.01, 28, &water, true);
    b.solid(Shape::Cylinder { center: p + Vec3::Y * 0.2, radius: 3.5, half_height: 0.2 });
    // Siv: tynne strå i klynger langs kanten.
    let reed = Mat::plain(0x6b7a3a, 0.8).flags(flag::WIND);
    for k in 0..14 {
        let a = k as f32 * 0.9;
        let base = p + Vec3::new(a.cos() * 2.6, 0.2, a.sin() * 2.6);
        for j in 0..5 {
            let o = Vec3::new((j as f32 * 1.3).sin() * 0.15, 0.0, (j as f32 * 2.1).cos() * 0.15);
            let h = 0.7 + (j as f32 * 0.7 + k as f32).sin().abs() * 0.5;
            b.mesh.cylinder(&Affine3A::from_translation(base + o), 0.012, 0.004, h, 4, &reed, false);
        }
    }
    // `p` ligger alt på bakken; benken får flate koordinater og finner sin egen høyde.
    super::furniture::bench(b, Vec3::new(p.x, 0.0, p.z + 4.6), std::f32::consts::PI);
}

pub fn build(b: &mut Builder) {
    // Porten: to steinpilarer med skilt, der stien kommer inn fra øst.
    for dz in [-2.2f32, 2.2] {
        let q = b.on_ground(Vec3::new(X1, 0.0, dz));
        b.mesh.cuboid_ao(&Affine3A::from_translation(q + Vec3::Y * 0.9), Vec3::new(0.6, 2.4, 0.6), &STONE, 1.0);
        b.mesh.sphere(&Affine3A::from_translation(q + Vec3::Y * 2.3), Vec3::splat(0.3), 6, 10, &STONE);
        b.solid_box(q + Vec3::Y * 1.0, Vec3::new(0.6, 2.4, 0.6), 0.0);
    }
    sign(b, Vec3::new(X1 - 1.2, 0.0, 4.2), std::f32::consts::FRAC_PI_2, "TOKENPARKEN", "Fargene, avstandene og dammen", 2.6);

    // Hekken rundt parken: busker tett i tett, med åpning ved porten.
    let mut k = 0;
    let mut along = |b: &mut Builder, from: Vec3, to: Vec3| {
        let n = ((to - from).length() / 1.1).ceil() as i32;
        for i in 0..=n {
            let p = from.lerp(to, i as f32 / n as f32);
            if (p.x - X1).abs() < 0.5 && p.z.abs() < 3.2 {
                continue;
            }
            let wob = Vec3::new(((k as f32) * 1.7).sin() * 0.25, 0.0, ((k as f32) * 2.3).cos() * 0.25);
            let q = b.on_ground(p + wob);
            super::nature::bush(b, q - Vec3::Y * 0.1, 0.8 + ((k as f32) * 0.9).sin().abs() * 0.35);
            k += 1;
        }
    };
    along(b, Vec3::new(X0, 0.0, Z0), Vec3::new(X1, 0.0, Z0));
    along(b, Vec3::new(X0, 0.0, Z1), Vec3::new(X1, 0.0, Z1));
    along(b, Vec3::new(X0, 0.0, Z0), Vec3::new(X0, 0.0, Z1));
    along(b, Vec3::new(X1, 0.0, Z0), Vec3::new(X1, 0.0, Z1));

    // Fargebedene: fire nord for stien, fem sør for den.
    let bed_len = 0.72 * 9.0 + 0.4;
    for (i, name) in PALETTES.iter().enumerate() {
        let (row, col) = if i < 4 { (0, i) } else { (1, i - 4) };
        let z = if row == 0 { 4.5 } else { -4.5 };
        let z2 = if row == 0 { 9.5 } else { -9.5 };
        // Annethvert bed lenger fra stien, så de ikke står på rekke.
        let zz = if col % 2 == 0 { z } else { z2 };
        let x = X1 - 4.0 - (bed_len + 1.4) * (col as f32) - bed_len;
        bed(b, Vec3::new(x, 0.0, zz), name);
    }
    sign(b, Vec3::new(X1 - 5.0, 0.0, 2.3), 0.0, "Fargebedene", "Hver palett fra surface til content", 2.4);

    // Avstandsskalaen langs vestkanten, og dammen i sørvest.
    spacing_scale(b, Vec3::new(X0 + 4.0, 0.0, -4.0));
    sign(b, Vec3::new(X0 + 4.0, 0.0, 6.0), std::f32::consts::FRAC_PI_2, "Avstandsskalaen", "Høyden er trinnet, 2 cm per px", 2.6);
    pond(b, Vec3::new(X0 + 11.0, 0.0, -11.0));

    // Et par trær og lykter inne i parken.
    for (x, z, s) in [(-56.0, 13.0, 1.3), (-70.0, 13.5, 1.1), (-52.0, -13.5, 1.0)] {
        let p = b.on_ground(Vec3::new(x, -0.1, z));
        super::nature::tree(b, p, s, 5);
    }
    for (x, z) in [(X1 - 3.0, 2.0), (X0 + 8.0, 2.0), (-62.0, -2.0)] {
        super::furniture::lantern(b, Vec3::new(x, 0.0, z), 0.0);
    }
}
