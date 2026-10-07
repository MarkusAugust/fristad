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
    // Mindre trær i hjørnene: modellen om den finnes, ellers de prosedyriske.
    for (i, (x, z)) in [(-14.0, -12.0), (14.0, -12.0), (-14.0, 12.5), (14.0, 12.5)].into_iter().enumerate() {
        tree(b, Vec3::new(x, 0.3, z), 1.0, i as u32 + 1);
        let _ = i;
    }
    // Busker langs gata sørover og bak porten.
    for (k, (x, z)) in [(-2.6, 30.0), (2.6, 36.0), (-2.6, 42.0), (-3.6, -30.0), (3.6, -40.0)].into_iter().enumerate() {
        let p = b.on_ground(Vec3::new(x, 0.0, z));
        bush(b, p, 0.9 + (k as f32 * 0.37).sin().abs() * 0.3);
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

/// Gresstuster og ugress: små bladklaser langs kanter og i hjørner.
pub fn tufts(b: &mut Builder) {
    let grass = Mat::special(kind::LEAF, 0xc8d890, 0.8).flags(flag::WIND | flag::NO_WEATHER);
    // Langs fortauskantene rundt torget, og langs gatene.
    let mut spots: Vec<(f32, f32)> = vec![];
    for k in 0..70 {
        let t = k as f32 / 70.0;
        let a = t * std::f32::consts::TAU;
        // Rundt torget, litt innenfor husene.
        spots.push((a.cos() * 18.6 + (k as f32 * 2.3).sin() * 0.6, a.sin() * 16.4 + (k as f32 * 1.7).cos() * 0.6));
    }
    for k in 0..40 {
        let z = -26.0 - k as f32 * 0.65;
        spots.push((if k % 2 == 0 { -4.3 } else { 4.3 } + (k as f32 * 1.1).sin() * 0.3, z));
    }
    for k in 0..30 {
        let z = 27.0 + k as f32 * 0.7;
        spots.push((if k % 2 == 0 { -3.4 } else { 3.4 } + (k as f32 * 0.9).sin() * 0.3, z));
    }
    for (x, z) in spots {
        if !b.rng.chance(0.55) {
            continue;
        }
        let size = b.rng.range(0.3, 0.5);
        let p = b.on_ground(Vec3::new(x, 0.0, z)) + Vec3::Y * size * 0.38;
        let yaw = b.rng.next() * 6.28;
        leaf_cluster(&mut b.mesh, p, size, 0.05, yaw, &grass);
    }
}

/// Eføy på en vegg: bladkort tett inntil flaten, i en flekk som vokser opp
/// fra et hjørne. `t` er veggens ramme (forsiden mot +Z), `x` hvor på veggen.
pub fn ivy(b: &mut Builder, t: &Affine3A, x: f32, height: f32) {
    let leaf = Mat::special(kind::LEAF, 0xa8c890, 0.75).flags(flag::WIND | flag::NO_WEATHER);
    let n = (height * 6.0) as i32;
    for k in 0..n {
        let fy = k as f32 / n as f32;
        let spread = 0.9 * (1.0 - fy * 0.6);
        let dx = b.rng.range(-spread, spread);
        let p = Vec3::new(x + dx, 0.2 + fy * height, 0.08 + b.rng.range(0.0, 0.06));
        let size = b.rng.range(0.35, 0.55);
        let mut m = Mesh::default();
        let r = Quat::from_rotation_z(b.rng.range(-0.4, 0.4));
        let q = Affine3A::from_rotation_translation(r, p);
        let h = size * 0.5;
        m.quad_uv(&q, [Vec3::new(-h, -h, 0.0), Vec3::new(h, -h, 0.0), Vec3::new(h, h, 0.0), Vec3::new(-h, h, 0.0)], &leaf, [0.08; 4], [Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)]);
        m.transform(t);
        b.mesh.append(&m);
    }
}

/// Ei gran: stamme og tre kjegler oppå hverandre, litt skjev.
pub fn spruce(b: &mut Builder, base: Vec3, h: f32, seed: f32) {
    let m = &mut b.mesh;
    let g = 0.85 + (seed * 7.0).sin() * 0.15;
    let needles = Mat::plain(rgb(0.14 * g, 0.26 * g, 0.16 * g), 0.9);
    let lean = Quat::from_rotation_z((seed * 3.1).sin() * 0.04) * Quat::from_rotation_x((seed * 5.3).cos() * 0.04);
    let t = Affine3A::from_rotation_translation(lean, base);
    m.cylinder(&t, 0.14 * h / 8.0, 0.08 * h / 8.0, h * 0.35, 6, &BARK, false);
    for k in 0..3 {
        let y0 = h * (0.18 + 0.27 * k as f32);
        let r = h * (0.3 - 0.07 * k as f32);
        let hk = h * (0.5 - 0.09 * k as f32);
        let yaw = Quat::from_rotation_y(seed + k as f32 * 0.7);
        m.cylinder(&(t * Affine3A::from_rotation_translation(yaw, Vec3::Y * y0)), r, 0.02, hk, 7, &needles, false);
    }
}

fn rgb(r: f32, g: f32, b: f32) -> u32 {
    let c = |v: f32| ((v.clamp(0.0, 1.0)) * 255.0) as u32;
    (c(r) << 16) | (c(g) << 8) | c(b)
}

/// Et steingjerde som følger terrenget, i stykker på to meter med små
/// forskyvninger, og med hull her og der.
pub fn stone_wall(b: &mut Builder, from: Vec3, to: Vec3, bend: f32) {
    let total = (to - from).length();
    let n = (total / 2.0).ceil() as i32;
    let dir = (to - from).normalize();
    let side = Vec3::new(-dir.z, 0.0, dir.x);
    for k in 0..n {
        if b.rng.chance(0.08) {
            continue;
        }
        let t = (k as f32 + 0.5) / n as f32;
        let curve = (t * std::f32::consts::PI).sin() * bend;
        let c = from + dir * (t * total) + side * (curve + b.rng.range(-0.12, 0.12));
        let gy = b.ground(c.x, c.z);
        let h = b.rng.range(0.8, 1.05);
        let yaw = dir.x.atan2(dir.z) + std::f32::consts::FRAC_PI_2 + b.rng.range(-0.05, 0.05);
        let tr = Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), Vec3::new(c.x, gy + h * 0.5 - 0.3, c.z));
        b.mesh.cuboid_ao(&tr, Vec3::new(2.15, h + 0.6, b.rng.range(0.5, 0.65)), &super::STONE, 1.0);
        b.colliders.push(Shape::Cuboid { center: Vec3::new(c.x, gy + h * 0.5, c.z), size: Vec3::new(2.15, h, 0.6), yaw });
    }
}

/// En stein: en flatklemt, halvt nedgravd kule i stein.
pub fn boulder(b: &mut Builder, p: Vec3, r: f32) {
    let p = b.on_ground(p);
    let s = Vec3::new(r * b.rng.range(0.8, 1.3), r * b.rng.range(0.5, 0.8), r * b.rng.range(0.8, 1.2));
    let rot = Quat::from_rotation_y(b.rng.next() * 6.28) * Quat::from_rotation_x(b.rng.range(-0.2, 0.2));
    let g = b.rng.range(0.8, 1.0);
    let rock = Mat::plain(rgb(0.42 * g, 0.41 * g, 0.39 * g), 0.95);
    b.mesh.sphere(&Affine3A::from_rotation_translation(rot, p + Vec3::Y * s.y * 0.35), s, 7, 10, &rock);
    b.colliders.push(Shape::Cylinder { center: p + Vec3::Y * s.y * 0.5, radius: s.x.min(s.z) * 0.8, half_height: s.y * 0.6 });
}

/// Enga rundt byen: løvtrær, busker, steiner, steingjerder, tuster, og
/// granskog som tetner mot åsene.
pub fn meadow(b: &mut Builder) {
    let free = |b: &Builder, x: f32, z: f32, margin: f32| super::town_dist(x, z) > margin && super::path_dist(x, z) > 3.0 && b.ground(x, z) > -100.0;
    // Løvtrær i klynger og enkeltvis.
    let mut k = 0u32;
    for i in 0..70 {
        let a = i as f32 * 2.399;
        let dist = 40.0 + (i as f32 * 1.37).sin().abs() * 70.0;
        let x = a.sin() * dist * 1.15 + b.rng.range(-6.0, 6.0);
        let z = a.cos() * dist + b.rng.range(-6.0, 6.0);
        if !free(b, x, z, 6.0) {
            continue;
        }
        let scale = b.rng.range(0.7, 1.5);
        let p = b.on_ground(Vec3::new(x, -0.1, z));
        tree(b, p, scale, k % 7);
        k += 1;
    }
    // Busker og kratt.
    for i in 0..90 {
        let a = i as f32 * 2.399 + 0.5;
        let dist = 36.0 + (i as f32 * 0.83).cos().abs() * 80.0;
        let x = a.sin() * dist * 1.1 + b.rng.range(-5.0, 5.0);
        let z = a.cos() * dist + b.rng.range(-5.0, 5.0);
        if !free(b, x, z, 4.0) {
            continue;
        }
        let p = b.on_ground(Vec3::new(x, 0.0, z));
        let scale = b.rng.range(0.7, 1.3);
        bush(b, p - Vec3::Y * 0.1, scale);
    }
    // Steiner.
    for i in 0..40 {
        let a = i as f32 * 2.399 + 1.3;
        let dist = 38.0 + (i as f32 * 1.91).sin().abs() * 90.0;
        let (x, z) = (a.sin() * dist * 1.1, a.cos() * dist);
        if !free(b, x, z, 3.0) {
            continue;
        }
        let r = b.rng.range(0.4, 1.4);
        boulder(b, Vec3::new(x, 0.0, z), r);
    }
    // Steingjerder langs åkrene og enga.
    for (from, to, bend) in [
        (Vec3::new(50.0, 0.0, -14.0), Vec3::new(140.0, 0.0, -22.0), 6.0),
        (Vec3::new(52.0, 0.0, -66.0), Vec3::new(130.0, 0.0, -70.0), -4.0),
        (Vec3::new(-48.0, 0.0, 30.0), Vec3::new(-115.0, 0.0, 85.0), 8.0),
        (Vec3::new(-40.0, 0.0, -40.0), Vec3::new(-90.0, 0.0, -75.0), -5.0),
        (Vec3::new(20.0, 0.0, 70.0), Vec3::new(70.0, 0.0, 120.0), 7.0),
    ] {
        stone_wall(b, from, to, bend);
    }
    // Tuster i enga.
    let grass = Mat::special(kind::LEAF, 0xd0dc90, 0.8).flags(flag::WIND | flag::NO_WEATHER);
    for i in 0..260 {
        let a = i as f32 * 2.399 + 0.9;
        let dist = 36.0 + (i as f32 * 0.57).sin().abs() * 60.0;
        let x = a.sin() * dist * 1.1 + b.rng.range(-4.0, 4.0);
        let z = a.cos() * dist + b.rng.range(-4.0, 4.0);
        if super::town_dist(x, z) < 1.0 {
            continue;
        }
        let size = b.rng.range(0.4, 0.7);
        let p = b.on_ground(Vec3::new(x, 0.0, z)) + Vec3::Y * size * 0.38;
        let yaw = b.rng.next() * 6.28;
        leaf_cluster(&mut b.mesh, p, size, 0.05, yaw, &grass);
    }
    // Granskog: spredt fra 70 meter, tett fra 110.
    for i in 0..520 {
        let a = i as f32 * 0.618 * std::f32::consts::TAU;
        let dist = 70.0 + b.rng.range(0.0, 190.0);
        let x = a.sin() * dist + b.rng.range(-5.0, 5.0);
        let z = a.cos() * dist + b.rng.range(-5.0, 5.0);
        if !free(b, x, z, 10.0) {
            continue;
        }
        let density = ((dist - 70.0) / 50.0).clamp(0.15, 1.0);
        if !b.rng.chance(density) {
            continue;
        }
        let h = b.rng.range(7.0, 15.0);
        let p = super::terrain::at(Vec3::new(x, 0.0, z)) - Vec3::Y * 0.3;
        spruce(b, p, h, i as f32);
    }
}

/// Åser mellom skogen og fjellene: myke, gresskledde rygger.
pub fn hills(b: &mut Builder) {
    let grass = Mat::tex(Layer::Grass, 0xc8f098);
    for i in 0..14 {
        let a = i as f32 / 14.0 * std::f32::consts::TAU + b.rng.range(-0.2, 0.2);
        let dist = b.rng.range(190.0, 300.0);
        let p = Vec3::new(a.sin() * dist, -6.0, a.cos() * dist);
        let r = b.rng.range(70.0, 130.0);
        let h = b.rng.range(14.0, 30.0);
        b.mesh.sphere(&Affine3A::from_translation(p), Vec3::new(r, h, r * b.rng.range(0.7, 1.0)), 8, 20, &grass);
    }
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
        // Et fjell av flere skjeve kjegler, så silhuetten blir ujevn.
        for j in 0..3 {
            let off = Vec3::new(b.rng.range(-0.4, 0.4) * r, 0.0, b.rng.range(-0.4, 0.4) * r);
            let hj = h * b.rng.range(0.6, 1.0);
            let rj = r * b.rng.range(0.5, 0.9);
            let lean = Quat::from_rotation_z(b.rng.range(-0.08, 0.08)) * Quat::from_rotation_x(b.rng.range(-0.08, 0.08));
            let t = Affine3A::from_rotation_translation(lean, p + off);
            b.mesh.cylinder(&t, rj, 0.0, hj, 7 + j, &rock, false);
            if hj > 110.0 {
                let k = 0.24;
                b.mesh.cylinder(&(t * Affine3A::from_translation(Vec3::Y * (hj * (1.0 - k) + 0.3))), rj * k * 1.03, 0.0, hj * k, 7 + j, &snow, false);
            }
        }
    }
}
