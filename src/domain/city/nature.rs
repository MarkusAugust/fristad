//! Trær, busker og fjellene rundt byen.
//!
//! Trærne har stamme og greiner som dreide profiler, og krona er bladkort:
//! kryssede firkanter med en bakt bladtekstur som alfatestes, og som svaier
//! i vinden i vertex-shaderen.

use super::{Builder, Shape};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec2, Vec3};

const BARK: Mat = Mat::tex(Layer::Bark, 0xffffff);

pub fn plaza_trees(b: &mut Builder) {
    // Det store treet i bedet midt på torget.
    tree(b, Vec3::new(0.0, 0.55, 0.0), 1.7, 0);
    // Mindre trær i hjørnene.
    for (i, (x, z)) in [(-14.0, -12.0), (14.0, -12.0), (-14.0, 12.5), (14.0, 12.5)].into_iter().enumerate() {
        tree(b, Vec3::new(x, 0.3, z), 1.0, i as u32 + 1);
    }
    // Busker langs gata sørover og bak porten.
    for (x, z) in [(-2.6, 30.0), (2.6, 36.0), (-2.6, 42.0), (-3.6, -30.0), (3.6, -40.0)] {
        bush(b, Vec3::new(x, 0.0, z), 0.9);
    }
}

/// Et tre med stamme, greiner og bladkort. `seed` varierer formen.
pub fn tree(b: &mut Builder, base: Vec3, scale: f32, seed: u32) {
    let rng = &mut b.rng;
    for _ in 0..seed {
        rng.next();
    }
    let m = &mut b.mesh;
    let h = 3.4 * scale;
    // Stammen: bredere rotfot, og litt skjev.
    let lean = Quat::from_rotation_z(rng.range(-0.05, 0.05)) * Quat::from_rotation_x(rng.range(-0.05, 0.05));
    let t = Affine3A::from_rotation_translation(lean, base);
    let profile = [
        Vec2::new(0.36 * scale, 0.0),
        Vec2::new(0.26 * scale, 0.25 * scale),
        Vec2::new(0.2 * scale, h * 0.5),
        Vec2::new(0.15 * scale, h),
    ];
    m.lathe(&t, &profile, 12, &BARK, false, 0.7);
    b.colliders.push(Shape::Cylinder { center: base + Vec3::Y * h * 0.5, radius: 0.3 * scale, half_height: h * 0.5 });

    // Greiner ut fra toppen, og bladklaser rundt enden av hver.
    let leaf = Mat::special(kind::LEAF, 0xffffff, 0.7).flags(flag::WIND | flag::NO_WEATHER);
    let n_branches = 5 + (rng.next() * 3.0) as i32;
    let crown = base + Vec3::Y * h;
    let crown_r = 2.1 * scale;
    for k in 0..n_branches {
        let a = k as f32 / n_branches as f32 * std::f32::consts::TAU + rng.range(-0.3, 0.3);
        let pitch = rng.range(0.5, 1.1);
        let len = rng.range(1.4, 2.3) * scale;
        let dir = Vec3::new(a.cos() * pitch.cos(), pitch.sin(), a.sin() * pitch.cos()).normalize();
        let start = crown - Vec3::Y * rng.range(0.0, 0.8) * scale;
        let rot = Quat::from_rotation_arc(Vec3::Y, dir);
        let bt = Affine3A::from_rotation_translation(rot, start);
        m.lathe(&bt, &[Vec2::new(0.09 * scale, 0.0), Vec2::new(0.04 * scale, len)], 7, &BARK, false, 1.0);
        let end = start + dir * len;
        // Klaser langs ytre halvdel av greina.
        for j in 0..3 {
            let p = start + dir * len * (0.45 + j as f32 * 0.27) + Vec3::new(rng.range(-0.4, 0.4), rng.range(-0.3, 0.4), rng.range(-0.4, 0.4)) * scale;
            let size = rng.range(1.1, 1.6) * scale;
            leaf_cluster(m, p, size, (p.y - base.y) / (h + crown_r), rng.next() * 6.28, &leaf);
        }
        let _ = end;
    }
    // Fyll krona innenfra.
    for _ in 0..(6.0 * scale) as i32 {
        let p = crown + Vec3::new(rng.range(-1.0, 1.0), rng.range(-0.2, 1.3), rng.range(-1.0, 1.0)) * crown_r * 0.55;
        leaf_cluster(m, p, rng.range(1.2, 1.8) * scale, (p.y - base.y) / (h + crown_r), rng.next() * 6.28, &leaf);
    }
}

/// Tre kryssede firkanter med bladteksturen, litt tilfeldig vridd.
/// Vindvekten (hvor høyt i krona) sendes i okklusjonsbyten.
fn leaf_cluster(m: &mut Mesh, c: Vec3, size: f32, height_frac: f32, yaw: f32, leaf: &Mat) {
    let h = size * 0.5;
    let wind = height_frac.clamp(0.1, 1.0);
    for k in 0..3 {
        let rot = Quat::from_rotation_y(yaw + k as f32 * 1.05) * Quat::from_rotation_x(if k == 2 { 1.3 } else { 0.2 * k as f32 - 0.1 });
        let t = Affine3A::from_rotation_translation(rot, c);
        let p = [Vec3::new(-h, -h, 0.0), Vec3::new(h, -h, 0.0), Vec3::new(h, h, 0.0), Vec3::new(-h, h, 0.0)];
        let uv = [Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)];
        m.quad_uv(&t, p, leaf, [wind; 4], uv);
    }
}

pub fn bush(b: &mut Builder, base: Vec3, scale: f32) {
    let leaf = Mat::special(kind::LEAF, 0xd9f0c0, 0.75).flags(flag::WIND | flag::NO_WEATHER);
    let rng = &mut b.rng;
    for _ in 0..5 {
        let p = base + Vec3::new(rng.range(-0.5, 0.5), rng.range(0.3, 0.8), rng.range(-0.5, 0.5)) * scale;
        let yaw = rng.next() * 6.28;
        leaf_cluster(&mut b.mesh, p, rng.range(0.8, 1.1) * scale, 0.15, yaw, &leaf);
    }
    b.colliders.push(Shape::Cylinder { center: base + Vec3::Y * 0.5 * scale, radius: 0.5 * scale, half_height: 0.5 * scale });
}

/// Fjell i det fjerne, så byen ligger i et landskap og ikke i et tomrom.
pub fn mountains(b: &mut Builder) {
    let rock = Mat::plain(0x5d6670, 0.95);
    let snow = Mat::plain(0xf4f6f8, 0.6);
    let n = 34;
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU + b.rng.range(-0.06, 0.06);
        let dist = b.rng.range(330.0, 480.0);
        let h = b.rng.range(70.0, 190.0);
        let r = b.rng.range(110.0, 190.0);
        let p = Vec3::new(a.sin() * dist, -4.0, a.cos() * dist);
        b.mesh.cylinder(&Affine3A::from_translation(p), r, 0.0, h, 7, &rock, false);
        if h > 120.0 {
            let k = 0.22;
            b.mesh.cylinder(&Affine3A::from_translation(p + Vec3::Y * (h * (1.0 - k) + 0.3)), r * k * 1.03, 0.0, h * k, 7, &snow, false);
        }
    }
    // Skog i mellomgrunnen: mørke kjegler mellom byen og fjellene.
    let pine = Mat::plain(0x1f3321, 0.9);
    for i in 0..260 {
        let a = i as f32 * 0.618 * std::f32::consts::TAU;
        let dist = 95.0 + b.rng.range(0.0, 160.0);
        let p = Vec3::new(a.sin() * dist, 0.0, a.cos() * dist);
        if p.x.abs() < 40.0 && p.z.abs() < 70.0 {
            continue;
        }
        let h = b.rng.range(7.0, 14.0);
        b.mesh.cylinder(&Affine3A::from_translation(p), h * 0.3, 0.0, h, 6, &pine, false);
    }
}
