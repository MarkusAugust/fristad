//! Torget og gatene: fortau, kantstein, bed, benker, pullerter, kummer,
//! plantekasser, sykkelstativ og lyktene (glTF-modellen Lantern).

use super::{at, Builder, Shape, CONCRETE, IRON, OAK, STONE};
use crate::domain::geometry::{kind, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec2, Vec3};

const GRASS: Mat = Mat::tex(Layer::Grass, 0xffffff);

/// Lykt: modellen Lantern (CC0, Khronos) med lyset i lyktehodet. Hodet
/// henger mot −X i modellen, så `yaw` vender det ut mot gata.
pub fn lantern(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    if b.has_model("lamp") {
        // Ekstern lykt: lyset settes nær toppen av modellen.
        let h = b.model_height("lamp");
        b.place("lamp", t);
        b.light(p + Vec3::Y * (h - 0.4), Vec3::new(1.0, 0.74, 0.46), 9.0, 15.0);
        b.solid(Shape::Cylinder { center: p + Vec3::Y * (h * 0.5), radius: 0.25, half_height: h * 0.5 });
        return;
    }
    b.place("lantern", t);
    let head = t.transform_point3(Vec3::new(-0.94, 2.85, 0.0));
    b.light(head, Vec3::new(1.0, 0.74, 0.46), 9.0, 15.0);
    b.solid(Shape::Cylinder { center: p + Vec3::Y * 2.0, radius: 0.3, half_height: 2.0 });
}

pub fn bench(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    if b.has_model("bench") {
        b.place("bench", at(p, yaw));
        b.solid_box(p + Vec3::Y * 0.4, Vec3::new(2.0, 0.8, 0.8), yaw);
        return;
    }
    let rot = Quat::from_rotation_y(yaw);
    let t = |q: Vec3| Affine3A::from_rotation_translation(rot, p + rot * q);
    let m = &mut b.mesh;
    // Spiler i eik, sete og rygg, på støpte ender i jern.
    for k in 0..4 {
        m.cuboid(&t(Vec3::new(0.0, 0.46, -0.2 + k as f32 * 0.13)), Vec3::new(1.9, 0.04, 0.11), &OAK);
    }
    for k in 0..3 {
        m.cuboid(&(t(Vec3::new(0.0, 0.68 + k as f32 * 0.13, -0.3)) * Affine3A::from_rotation_x(-0.2)), Vec3::new(1.9, 0.11, 0.035), &OAK);
    }
    for sx in [-0.85, 0.85] {
        let profile = [Vec2::new(0.0, 0.0), Vec2::new(0.06, 0.0), Vec2::new(0.06, 0.03), Vec2::new(0.0, 0.03)];
        let _ = profile;
        m.cuboid(&t(Vec3::new(sx, 0.22, 0.0)), Vec3::new(0.06, 0.44, 0.5), &IRON);
        m.cuboid(&(t(Vec3::new(sx, 0.7, -0.32)) * Affine3A::from_rotation_x(-0.2)), Vec3::new(0.06, 0.5, 0.06), &IRON);
        m.cuboid(&t(Vec3::new(sx, 0.02, 0.0)), Vec3::new(0.08, 0.04, 0.62), &IRON);
    }
    b.solid_box(p + Vec3::Y * 0.3, Vec3::new(2.0, 0.6, 0.7), yaw);
}

pub fn bollard(b: &mut Builder, p: Vec3) {
    let p = b.on_ground(p);
    let profile = [Vec2::new(0.11, 0.0), Vec2::new(0.11, 0.08), Vec2::new(0.07, 0.1), Vec2::new(0.07, 0.8), Vec2::new(0.09, 0.84), Vec2::new(0.09, 0.9), Vec2::new(0.0, 0.98)];
    b.mesh.lathe(&Affine3A::from_translation(p), &profile, 12, &IRON, true, 0.7);
    b.solid(Shape::Cylinder { center: p + Vec3::Y * 0.5, radius: 0.12, half_height: 0.5 });
}

pub fn manhole(b: &mut Builder, p: Vec3) {
    let p = b.on_ground(p);
    b.mesh.cylinder(&Affine3A::from_translation(p), 0.4, 0.4, 0.025, 20, &IRON, true);
}

pub fn planter(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    if b.has_model("planter") {
        b.place("planter", t);
        b.solid_box(p + Vec3::Y * 0.4, Vec3::new(1.0, 0.8, 1.0), yaw);
        return;
    }
    let m = &mut b.mesh;
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.3, 0.0))), Vec3::new(1.4, 0.6, 0.6), &CONCRETE);
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.59, 0.0))), Vec3::new(1.3, 0.04, 0.5), &GRASS);
    // Blomster: små fargede kuler.
    let colors = [0xd84a5a, 0xf2c14e, 0xe9e2d0, 0x9b59b6];
    for k in 0..9 {
        let x = -0.5 + k as f32 * 0.125;
        let c = colors[k % 4];
        m.sphere(&(t * Affine3A::from_translation(Vec3::new(x, 0.72 + (k % 3) as f32 * 0.04, -0.12 + (k % 2) as f32 * 0.24))), Vec3::splat(0.07), 5, 7, &Mat::plain(c, 0.7));
        m.cylinder(&(t * Affine3A::from_translation(Vec3::new(x, 0.6, -0.12 + (k % 2) as f32 * 0.24))), 0.008, 0.006, 0.12, 5, &Mat::plain(0x3f6b2a, 0.8), false);
    }
    b.solid_box(p + Vec3::Y * 0.3, Vec3::new(1.4, 0.6, 0.6), yaw);
}

pub fn bike_rack(b: &mut Builder, p: Vec3, yaw: f32) {
    let p = b.on_ground(p);
    let t = at(p, yaw);
    for k in 0..3 {
        let x = -0.6 + k as f32 * 0.6;
        b.mesh.torus(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), Vec3::new(x, 0.0, 0.0))), 0.4, 0.025, 20, 8, &IRON);
    }
    b.solid_box(p + Vec3::Y * 0.3, Vec3::new(1.8, 0.6, 0.2), yaw);
}

/// Fortau langs en fasaderekke: kantstein og en plate i betong, i stykker på
/// tre meter som hvert ligger på bakkehøyden, så de følger terrenget med
/// små trinn.
pub fn sidewalk(b: &mut Builder, from: Vec3, to: Vec3, outward: Vec3, width: f32) {
    let along = (to - from).normalize();
    let total = (to - from).length();
    let rot = Quat::from_rotation_arc(Vec3::X, along);
    let n = (total / 3.0).ceil().max(1.0) as i32;
    let len = total / n as f32;
    for k in 0..n {
        let c = from + along * (len * (k as f32 + 0.5)) + outward * (width * 0.5);
        let gy = b.ground(c.x, c.z);
        let m = &mut b.mesh;
        // Kassene strekker seg 60 cm ned i bakken, så de ikke henger i lufta i hellinger.
        let t = Affine3A::from_rotation_translation(rot, Vec3::new(c.x, gy + 0.12 - 0.36, c.z));
        m.cuboid_ao(&t, Vec3::new(len + 0.02, 0.12 + 0.6, width), &CONCRETE, 1.0);
        let cc = c + outward * (width * 0.5 - 0.15);
        let curb = Affine3A::from_rotation_translation(rot, Vec3::new(cc.x, gy + 0.14 - 0.37, cc.z));
        m.cuboid_ao(&curb, Vec3::new(len + 0.02, 0.14 + 0.6, 0.3), &STONE, 1.0);
    }
}

pub fn plaza(b: &mut Builder) {
    // Bedet i midten: steinkant, gress og en lav hekk av kuler.
    b.mesh.lathe(
        &Affine3A::IDENTITY,
        &[Vec2::new(4.7, 0.0), Vec2::new(4.7, 0.45), Vec2::new(4.6, 0.55), Vec2::new(4.3, 0.55), Vec2::new(4.3, 0.0)],
        48,
        &STONE,
        true,
        0.7,
    );
    b.mesh.cylinder(&Affine3A::from_translation(Vec3::Y * 0.5), 4.3, 4.3, 0.06, 48, &GRASS, true);
    b.solid(Shape::Cylinder { center: Vec3::Y * 0.275, radius: 4.7, half_height: 0.275 });

    // Mindre bed i hjørnene.
    for (x, z) in [(-14.0, -12.0), (14.0, -12.0), (-14.0, 12.5), (14.0, 12.5)] {
        b.mesh.lathe(&Affine3A::from_translation(Vec3::new(x, 0.0, z)), &[Vec2::new(1.4, 0.0), Vec2::new(1.4, 0.25), Vec2::new(1.3, 0.3), Vec2::new(1.1, 0.3), Vec2::new(1.1, 0.0)], 24, &STONE, true, 0.7);
        b.mesh.cylinder(&Affine3A::from_translation(Vec3::new(x, 0.26, z)), 1.1, 1.1, 0.04, 24, &GRASS, true);
    }

    // Benker rundt bedet.
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
        let p = Vec3::new(a.sin() * 6.6, 0.0, a.cos() * 6.6);
        bench(b, p, a);
    }

    // Lykter rundt torget og i gatene.
    for (x, z, yaw) in [
        (-9.0, -15.5, 0.0),
        (9.0, -15.5, std::f32::consts::PI),
        (-17.5, -6.0, -1.57),
        (17.5, -6.0, 1.57),
        (-17.5, 6.0, -1.57),
        (17.5, 6.0, 1.57),
        (-9.0, 15.5, 0.0),
        (9.0, 15.5, std::f32::consts::PI),
    ] {
        lantern(b, Vec3::new(x, 0.0, z), yaw);
    }
    for z in [-34.0, -46.0] {
        lantern(b, Vec3::new(-3.6, 0.0, z), -1.57);
        lantern(b, Vec3::new(3.6, 0.0, z - 6.0), 1.57);
    }
    for z in [32.0, 42.0] {
        lantern(b, Vec3::new(3.0, 0.0, z), 1.57);
    }

    // Fortau langs fasadene rundt torget.
    sidewalk(b, Vec3::new(-19.6, 0.0, -17.6), Vec3::new(19.6, 0.0, -17.6), Vec3::Z, 1.8);
    sidewalk(b, Vec3::new(-19.6, 0.0, 17.6), Vec3::new(19.6, 0.0, 17.6), -Vec3::Z, 1.8);
    sidewalk(b, Vec3::new(-19.6, 0.0, -17.6), Vec3::new(-19.6, 0.0, 17.6), Vec3::X, 1.8);
    sidewalk(b, Vec3::new(19.6, 0.0, -17.6), Vec3::new(19.6, 0.0, 17.6), -Vec3::X, 1.8);

    // Pullerter mot gatene, kummer, plantekasser og sykkelstativ.
    for x in [-7.5, -6.0, 6.0, 7.5] {
        bollard(b, Vec3::new(x, 0.0, 15.0));
    }
    for (x, z) in [(-11.0, -4.0), (12.0, 8.0), (0.0, 24.0), (-1.5, -26.0)] {
        manhole(b, Vec3::new(x, 0.0, z));
    }
    planter(b, Vec3::new(-15.5, 0.0, -2.0), 1.57);
    planter(b, Vec3::new(-15.5, 0.0, 2.0), 1.57);
    planter(b, Vec3::new(15.5, 0.0, -2.0), 1.57);
    bike_rack(b, Vec3::new(11.0, 0.0, -15.0), 0.0);
    bike_rack(b, Vec3::new(-11.0, 0.0, 15.0), 0.0);

    // En gammel brønn med vann i, sørvest på torget.
    well(b, Vec3::new(-11.0, 0.0, 9.0));

    // Reven som står og ser på deg ved bedet.
    let fox = b.on_ground(Vec3::new(6.3, 0.0, 3.2));
    b.place("fox", at(fox, 2.3));
    b.solid_box(fox + Vec3::Y * 0.4, Vec3::new(1.2, 0.8, 0.5), 2.3);

    // Eksterne modeller der de finnes: tønner og kasser ved butikkene,
    // søppelbøtte og sykkel ved sykkelstativet, en kjerre ved brønnen.
    // Poly Haven-modellene er grupper (flere tønner, flere bøtter), så én av hver.
    let extras: [(&'static str, Vec3, f32); 5] = [
        ("barrel", Vec3::new(-16.6, 0.0, -9.5), 0.3),
        ("crate", Vec3::new(17.4, 0.0, 9.5), 0.4),
        ("bin", Vec3::new(12.0, 0.0, -15.6), 0.0),
        ("bicycle", Vec3::new(10.4, 0.0, -14.7), 1.2),
        ("cart", Vec3::new(-13.4, 0.0, 7.2), 0.6),
    ];
    for (name, p, yaw) in extras {
        if b.has_model(name) {
            let p = b.on_ground(p);
            // Lakken på egne modeller: mørk grønn bøtte, rød sykkel.
            let tint = match name {
                "bin" => super::street_life::srgb_to_linear(0x2f4a3a),
                "bicycle" => super::street_life::srgb_to_linear(0x8c2f2f),
                _ => Vec3::ONE,
            };
            b.place_tinted(name, at(p, yaw), tint);
            let h = b.model_height(name).max(0.5);
            b.solid_box(p + Vec3::Y * (h * 0.5), Vec3::new(0.8, h, 0.8), yaw);
        }
    }
}

fn well(b: &mut Builder, p: Vec3) {
    let p = b.on_ground(p);
    let m = &mut b.mesh;
    m.lathe(&Affine3A::from_translation(p), &[Vec2::new(1.1, 0.0), Vec2::new(1.1, 0.9), Vec2::new(0.85, 0.95), Vec2::new(0.85, 0.3)], 20, &STONE, false, 0.7);
    let water = Mat::special(kind::WATER, 0x203040, 0.02);
    m.cylinder(&Affine3A::from_translation(p + Vec3::Y * 0.6), 0.86, 0.86, 0.01, 20, &water, true);
    // Stolper og lite tak.
    for sx in [-0.9, 0.9] {
        m.cuboid(&Affine3A::from_translation(p + Vec3::new(sx, 1.4, 0.0)), Vec3::new(0.12, 1.6, 0.12), &OAK);
    }
    m.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), p + Vec3::new(1.0, 1.75, 0.0)), 0.06, 0.06, 2.0, 8, &OAK, true);
    m.gable_roof(&Affine3A::from_translation(p + Vec3::Y * 2.1), 2.4, 1.6, 0.6, 0.2, &Mat::tex(Layer::Tiles, 0xffffff), &OAK, &OAK);
    b.solid(Shape::Cylinder { center: p + Vec3::Y * 0.5, radius: 1.15, half_height: 0.5 });
}

pub fn streets(b: &mut Builder) {
    // Fortau i gatene.
    sidewalk(b, Vec3::new(-4.6, 0.0, -52.0), Vec3::new(-4.6, 0.0, -27.5), Vec3::X, 1.2);
    sidewalk(b, Vec3::new(4.6, 0.0, -52.0), Vec3::new(4.6, 0.0, -27.5), -Vec3::X, 1.2);
    sidewalk(b, Vec3::new(-3.6, 0.0, 27.5), Vec3::new(-3.6, 0.0, 48.0), Vec3::X, 1.0);
    sidewalk(b, Vec3::new(3.6, 0.0, 27.5), Vec3::new(3.6, 0.0, 48.0), -Vec3::X, 1.0);
    let _ = Mesh::default();
}
