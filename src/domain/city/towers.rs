//! Tårnene: typografiskalaen som åtte steintårn i en halvsirkel nord for
//! byen, der stien fra Nordgata ender. Høyden på hvert tårn er
//! skriftstørrelsen (20 cm per px), og på hvert tårn henger en plakett med
//! ordet «Søknad» i den størrelsen, i målestokken 1 px = 1 cm.

use super::{at, Builder, Shape, IRON, OAK, STONE};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec2, Vec3};

pub const CENTER: Vec3 = Vec3::new(0.0, 0.0, -88.0);
pub const RADIUS: f32 = 16.0;
const SIZES: [&str; 8] = ["xxs", "xs", "s", "m", "l", "xl", "xxl", "mega"];

fn tower(b: &mut Builder, p: Vec3, px: f32, name: &str, face: f32) {
    let p = b.on_ground(p);
    let h = px * 0.2 + 1.2;
    let r = 1.0 + px * 0.02;
    let m = &mut b.mesh;
    // Stammen: litt smalere oppover, sunket i bakken, med en krans øverst.
    m.lathe(&Affine3A::from_translation(p), &[Vec2::new(r + 0.2, -1.5), Vec2::new(r + 0.2, 0.4), Vec2::new(r, 0.5), Vec2::new(r * 0.88, h), Vec2::new(r * 0.98, h), Vec2::new(r * 0.98, h + 0.35)], 16, &STONE, false, 0.7);
    // Taket: en kjegle i skifer, med en spiss i jern.
    let slate = Mat::tex(Layer::Slate, 0xffffff);
    m.cylinder(&Affine3A::from_translation(p + Vec3::Y * (h + 0.35)), r * 1.1, 0.05, r * 1.6, 16, &slate, false);
    m.cylinder(&Affine3A::from_translation(p + Vec3::Y * (h + 0.35 + r * 1.6)), 0.03, 0.03, 0.6, 6, &IRON, true);
    // Vinduer rundt, ett per etasje, og ei dør mot plassen.
    let dark = Mat::plain(0x14161a, 0.6);
    let floors = ((h - 1.0) / 2.6).floor().max(1.0) as i32;
    for f in 0..floors {
        let a = face + 0.6 + f as f32 * 1.3;
        let y = 2.0 + f as f32 * 2.6;
        let rr = r * (1.0 - 0.12 * (y / h));
        let q = p + Vec3::new(a.sin() * rr, y, a.cos() * rr);
        let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(a), q);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0))), Vec3::new(0.5, 0.8, 0.12), &dark, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, -0.45, 0.04))), Vec3::new(0.7, 0.08, 0.2), &STONE, 1.0);
    }
    let door_q = p + Vec3::new(face.sin() * (r + 0.08), 0.5, face.cos() * (r + 0.08));
    let dt = Affine3A::from_rotation_translation(Quat::from_rotation_y(face), door_q);
    m.cuboid_ao(&(dt * Affine3A::from_translation(Vec3::new(0.0, 1.0, 0.0))), Vec3::new(0.9, 2.0, 0.12), &OAK, 1.0);
    m.cuboid_ao(&(dt * Affine3A::from_translation(Vec3::new(0.0, 2.05, 0.0))), Vec3::new(1.1, 0.12, 0.2), &STONE, 1.0);
    // Plaketten ved døra: ordet i tokenets størrelse, og navnet under.
    let word = "Søknad";
    let sub = format!("font-size-{name} · {px:.0} px");
    // Plata er så brei som den bredeste linja trenger.
    let ww = b.font.width(word, px).max(b.font.width(&sub, 10.0)) * 0.01;
    let pw = (ww + 0.24).max(0.9);
    b.fits("tårnplakett", word, px, 0.01, pw);
    b.fits("tårnplakett", &sub, 10.0, 0.01, pw);
    let ph = px * 0.01 * 1.2 + 0.3;
    let pt = Affine3A::from_rotation_translation(Quat::from_rotation_y(face), p + Vec3::new(face.sin() * (r + 0.1), 0.5 + 1.0, face.cos() * (r + 0.1)));
    let pt = pt * Affine3A::from_translation(Vec3::new(0.45 + pw * 0.5 + 0.1, if px > 20.0 { 0.9 } else { 0.4 }, 0.0));
    let m = &mut b.mesh;
    let brass = Mat::plain(0xb8924a, 0.35).flags(flag::NO_DIRT);
    m.cuboid_ao(&(pt * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0))), Vec3::new(pw, ph, 0.03), &brass, 1.0);
    let ink = Mat::special(kind::TEXT, 0x2a2418, 0.6).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let mut text = Mesh::default();
    b.font.centered(&mut text, word, Vec3::new(0.0, 0.08, 0.0), px, 0.01, &ink);
    b.font.centered(&mut text, &sub, Vec3::new(0.0, -ph * 0.5 + 0.09, 0.0), 10.0, 0.01, &ink);
    text.transform(&(pt * Affine3A::from_translation(Vec3::Z * 0.018)));
    b.mesh.append(&text);
    b.solid(Shape::Cylinder { center: p + Vec3::Y * (h * 0.5), radius: r + 0.1, half_height: h * 0.5 });
}

pub fn build(b: &mut Builder) {
    // Tårnene i en halvsirkel på nordsiden av plassen, fra minst til størst.
    for (k, name) in SIZES.iter().enumerate() {
        let px = b.tokens.font_size_px(name);
        let a = -1.35 + 2.7 * (k as f32 / (SIZES.len() - 1) as f32);
        let p = CENTER + Vec3::new(a.sin() * RADIUS, 0.0, -a.cos() * RADIUS);
        // Døra og plaketten vender mot plassens midte.
        let face = (CENTER.x - p.x).atan2(CENTER.z - p.z);
        tower(b, p, px, name, face);
    }
    // Skiltet ved inngangen til plassen, og lykter.
    let sp = b.on_ground(CENTER + Vec3::new(-3.0, 0.0, 9.0));
    let t = at(sp, 0.0);
    for dx in [-1.3f32, 1.3] {
        b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.75, 0.0))), Vec3::new(0.08, 1.9, 0.08), &OAK, 1.0);
    }
    let board = Mat::tex(Layer::Oak, 0xffffff).tint(Vec3::new(0.5, 0.45, 0.38));
    let (title, sub) = ("TÅRNENE", "Skriftstørrelsene, 20 cm per px");
    let bw = (b.font.width(title, 26.0).max(b.font.width(sub, 14.0)) * 0.012 + 0.5).max(2.8);
    b.fits("tårnskiltet", title, 26.0, 0.012, bw);
    b.fits("tårnskiltet", sub, 14.0, 0.012, bw);
    b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.45, 0.0))), Vec3::new(bw, 0.76, 0.05), &board, 1.0);
    let ink = Mat::plain(0xf4efe4, 0.5).flags(flag::NO_DIRT);
    let mut text = Mesh::default();
    b.font.centered(&mut text, title, Vec3::new(0.0, 0.12, 0.0), 26.0, 0.012, &ink);
    b.font.centered(&mut text, sub, Vec3::new(0.0, -0.17, 0.0), 14.0, 0.012, &ink);
    text.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, 1.45, 0.03))));
    b.mesh.append(&text);
    b.solid_box(sp + Vec3::Y * 0.95, Vec3::new(2.8, 1.9, 0.15), 0.0);
    for (x, z) in [(-7.0, 2.0), (7.0, 2.0), (0.0, -6.0)] {
        super::furniture::lantern(b, CENTER + Vec3::new(x, 0.0, z), 0.0);
    }
    super::furniture::bench(b, CENTER + Vec3::new(0.0, 0.0, 1.0), std::f32::consts::PI);
}
