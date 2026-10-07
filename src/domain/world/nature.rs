//! Trær, busker og fjellene rundt byen.
//!
//! Trærne har stamme og greiner som dreide profiler, og krona er bladkort:
//! kryssede firkanter med en bakt bladtekstur som alfatestes, og som svaier
//! i vinden i vertex-shaderen.

use super::{at, terrain, Builder, Shape};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::collections::HashMap;
use std::sync::OnceLock;

const BARK: Mat = Mat::tex(Layer::Bark, 0xffffff);

/// Løvtrærne fra tools/models/trees.py: stamme og bladverk, i par.
const BROADLEAF: [(&str, &str); 3] = [("oak1_trunk", "oak1_leaves"), ("oak2_trunk", "oak2_leaves"), ("birch1_trunk", "birch1_leaves")];
const SPRUCES: [(&str, &str, f32); 2] = [("spruce1_trunk", "spruce1_needles", 12.0), ("spruce2_trunk", "spruce2_needles", 9.0)];
const BUSHES: [(&str, &str); 2] = [("bush1_trunk", "bush1_leaves"), ("bush2_trunk", "bush2_leaves")];

/// Et tre med stamme, greiner og bladkort. `seed` varierer formen. Finnes
/// modellene fra Blender, brukes de (eik eller bjørk etter seed); ellers
/// det prosedyriske treet.
/// Greinskjelettene fra `tools/models/trees.py` (`<navn>_branches.json`):
/// segmenter [x0, y0, z0, x1, y1, z1, r] grovere enn 5 cm, som blir
/// kapselkollidere så ingen går gjennom stammer og lave greiner.
const BRANCHES: [(&str, &str); 7] = [
    ("oak1_trunk", include_str!("../../../assets/models/trees/oak1_branches.json")),
    ("oak2_trunk", include_str!("../../../assets/models/trees/oak2_branches.json")),
    ("birch1_trunk", include_str!("../../../assets/models/trees/birch1_branches.json")),
    ("spruce1_trunk", include_str!("../../../assets/models/trees/spruce1_branches.json")),
    ("spruce2_trunk", include_str!("../../../assets/models/trees/spruce2_branches.json")),
    ("bush1_trunk", include_str!("../../../assets/models/trees/bush1_branches.json")),
    ("bush2_trunk", include_str!("../../../assets/models/trees/bush2_branches.json")),
];

fn branches(model: &str) -> &'static [[f32; 7]] {
    static PARSED: OnceLock<HashMap<&'static str, Vec<[f32; 7]>>> = OnceLock::new();
    let all = PARSED.get_or_init(|| BRANCHES.iter().map(|(n, json)| (*n, serde_json::from_str(json).unwrap_or_default())).collect());
    all.get(model).map(|v| v.as_slice()).unwrap_or(&[])
}

/// Kollidere for stammen og greinene du kan nå (under 3,6 m), som kapsler
/// langs skjelettet; `true` om modellen har et skjelett.
fn branch_colliders(b: &mut Builder, model: &str, t: &Affine3A, scale: f32) -> bool {
    let segs = branches(model);
    if segs.is_empty() {
        return false;
    }
    for s in segs {
        if s[1].min(s[4]) * scale > 3.6 {
            continue;
        }
        let a = t.transform_point3(Vec3::new(s[0], s[1], s[2]));
        let c = t.transform_point3(Vec3::new(s[3], s[4], s[5]));
        b.colliders.push(Shape::Capsule { a, b: c, radius: s[6] * scale });
    }
    true
}

pub fn tree(b: &mut Builder, base: Vec3, scale: f32, seed: u32) {
    let (trunk, leaves) = BROADLEAF[seed as usize % BROADLEAF.len()];
    if b.has_model(trunk) && b.has_model(leaves) {
        // Modellen er ca. 8 m høy ved skala 1; det prosedyriske treet var 5,5.
        let s = scale * 0.72;
        let yaw = b.rng.next() * std::f32::consts::TAU;
        // Stammen settes i den laveste bakken rundt foten, så ingen side
        // svever i en skråning.
        let low = [(-0.5f32, 0.0f32), (0.5, 0.0), (0.0, -0.5), (0.0, 0.5)].iter().map(|(dx, dz)| b.ground(base.x + dx, base.z + dz)).fold(f32::MAX, f32::min);
        let base = Vec3::new(base.x, base.y.min(low - 0.05), base.z);
        let t = at(base, yaw) * Affine3A::from_scale(Vec3::splat(s));
        b.place(trunk, t);
        b.place_mounted(leaves, t);
        if !branch_colliders(b, trunk, &t, s) {
            b.colliders.push(Shape::Cylinder { center: base + Vec3::Y * 2.5 * s, radius: 0.32 * s, half_height: 2.5 * s });
        }
        return;
    }
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
    let (stems, leaves) = BUSHES[(b.rng.next() * 2.0) as usize % 2];
    if b.has_model(stems) && b.has_model(leaves) {
        let yaw = b.rng.next() * std::f32::consts::TAU;
        let t = at(base, yaw) * Affine3A::from_scale(Vec3::splat(scale));
        b.place(stems, t);
        b.place_mounted(leaves, t);
        b.colliders.push(Shape::Cylinder { center: base + Vec3::Y * 0.5 * scale, radius: 0.5 * scale, half_height: 0.5 * scale });
        return;
    }
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
    let (trunk, needles, model_h) = SPRUCES[(seed as usize) % SPRUCES.len()];
    if b.has_model(trunk) && b.has_model(needles) {
        let s = h / model_h;
        let lean = Quat::from_rotation_z((seed * 3.1).sin() * 0.03) * Quat::from_rotation_x((seed * 5.3).cos() * 0.03);
        let t = Affine3A::from_rotation_translation(lean * Quat::from_rotation_y(seed * 2.4), base) * Affine3A::from_scale(Vec3::splat(s));
        b.place(trunk, t);
        b.place_mounted(needles, t);
        if !branch_colliders(b, trunk, &t, s) {
            b.colliders.push(Shape::Cylinder { center: base + Vec3::Y * h * 0.4, radius: 0.25 * s, half_height: h * 0.4 });
        }
        return;
    }
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
    // Ikke i byen, ikke på veiene, og ikke i bratte skråninger (borghaugen).
    let free = |b: &Builder, x: f32, z: f32, margin: f32| super::town::town_dist(x, z) > margin && super::town::path_dist(x, z) > 3.0 && b.ground(x, z) > -100.0 && terrain::normal(x, z).y > 0.97;
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
        (Vec3::new(70.0, 0.0, -18.0), Vec3::new(140.0, 0.0, -24.0), 6.0),
        (Vec3::new(62.0, 0.0, -72.0), Vec3::new(130.0, 0.0, -76.0), -4.0),
        (Vec3::new(-64.0, 0.0, 30.0), Vec3::new(-120.0, 0.0, 85.0), 8.0),
        (Vec3::new(-64.0, 0.0, -40.0), Vec3::new(-110.0, 0.0, -80.0), -5.0),
        (Vec3::new(25.0, 0.0, 72.0), Vec3::new(70.0, 0.0, 120.0), 7.0),
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
        if super::town::town_dist(x, z) < 1.0 {
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
        let p = super::terrain::at(Vec3::new(x, 0.0, z)) - Vec3::Y * 0.05;
        spruce(b, p, h, i as f32);
    }
}

/// Landskapet rundt terrenget: fra kanten av bakken (±200 m) stiger
/// åser og så fjell i ett sammenhengende polart rutenett, formet av
/// ridged noise. Lavt er det eng, bratt og høyt er det berg, og toppene
/// har snø. Ingen kollider: kanten av terrenget (`world_edge`) stopper deg
/// før du kommer dit.
pub fn landscape(b: &mut Builder) {
    let (r0, r1) = (196.0f32, 600.0f32);
    let (rings, spokes) = (64usize, 288usize);
    let grass = Mat::tex(Layer::Grass, 0xc8f098);
    let rock = Mat::tex(Layer::Rock, 0xffffff);
    let snow = Mat::tex(Layer::Snow, 0xffffff);
    // Radiene tettere innerst, der detaljene er nær.
    let radius = |j: usize| {
        let t = j as f32 / rings as f32;
        r0 + (r1 - r0) * t * t.sqrt()
    };
    let pos = |j: usize, i: usize| {
        let a = i as f32 / spokes as f32 * std::f32::consts::TAU;
        let r = radius(j);
        let (x, z) = (a.sin() * r, a.cos() * r);
        Vec3::new(x, landscape_height(x, z), z)
    };
    let normal = |p: Vec3| {
        let d = 3.0;
        let hx = landscape_height(p.x + d, p.z) - landscape_height(p.x - d, p.z);
        let hz = landscape_height(p.x, p.z + d) - landscape_height(p.x, p.z - d);
        Vec3::new(-hx, 2.0 * d, -hz).normalize()
    };
    for j in 0..rings {
        for i in 0..spokes {
            let i2 = (i + 1) % spokes;
            let p = [pos(j, i), pos(j, i2), pos(j + 1, i2), pos(j + 1, i)];
            let n = [normal(p[0]), normal(p[1]), normal(p[2]), normal(p[3])];
            let c = (p[0] + p[1] + p[2] + p[3]) * 0.25;
            let up = (n[0] + n[1] + n[2] + n[3]).normalize().y;
            let ground = terrain::height(c.x, c.z);
            let rise = c.y - ground;
            // Snøgrensa varierer rundt horisonten; bratte sider er berg.
            let snowline = 200.0 + (terrain::noise(c.x * 0.004, c.z * 0.004) - 0.5) * 80.0;
            let mat = if c.y > snowline && up > 0.6 {
                &snow
            } else if rise > 60.0 || up < 0.62 {
                &rock
            } else {
                &grass
            };
            b.mesh.quad_n([p[3], p[2], p[1], p[0]], [n[3], n[2], n[1], n[0]], mat, 1.0);
        }
    }
}

/// Høyden i landskapet utenfor terrenget: bakken glir over i åser fra
/// 200 m og fjell fra 280 m, med rygger og tinder fra ridged noise som
/// stiger utover, så horisonten er en sammenhengende fjellrekke.
pub fn landscape_height(x: f32, z: f32) -> f32 {
    let r = (x * x + z * z).sqrt();
    let base = terrain::height(x, z);
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let hills = smooth((r - 200.0) / 70.0) * (terrain::noise(x * 0.012 + 20.0, z * 0.012 + 20.0) * 28.0 + terrain::noise(x * 0.03 + 40.0, z * 0.03) * 8.0);
    // Ridged noise: toppene blir skarpe rygger.
    let ridge = |fx: f32, fz: f32, s: f32| 1.0 - (terrain::noise(fx * s, fz * s) * 2.0 - 1.0).abs();
    let ridges = ridge(x + 300.0, z, 0.0055) * 0.6 + ridge(x, z + 700.0, 0.012) * 0.28 + ridge(x + 900.0, z + 100.0, 0.03) * 0.12;
    // Store massiv per retning, så noen tinder rager over de andre.
    let massif = 0.55 + terrain::noise((x + 1000.0) * 0.0025, (z + 1000.0) * 0.0025) * 0.9;
    let rise = smooth((r - 280.0) / 150.0) * (120.0 + (r - 280.0).max(0.0) * 0.35);
    let mountains = rise * ridges.powf(1.4) * massif;
    base + hills + mountains
}

/// Usynlige vegger rundt kanten av terrenget (±200 m), så ingen faller
/// ut av verden eller går inn i fjellene.
pub fn world_edge(b: &mut Builder) {
    let n = 36;
    let radius = 199.0;
    for i in 0..n {
        let a = (i as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
        let c = Vec3::new(a.sin() * radius, 20.0, a.cos() * radius);
        b.colliders.push(Shape::Cuboid { center: c, size: Vec3::new(2.0 * std::f32::consts::PI * radius / n as f32 + 2.0, 80.0, 2.0), yaw: a });
    }
}

/// Gresset på enga: tuster i et rutenett med tilfeldig forskyvning, tettest
/// nær byen og tynnere utover, der bakken er eng. Tustene er modellene fra
/// tools/models/grass.py; tørre tuster der enga er tørr.
pub fn grass(b: &mut Builder) {
    // Tustene fra tools/models/grass.py: grønne, tørre, spredte strå og
    // høye strå med aks.
    let tufts: Vec<&str> = ["grass_a", "grass_b", "grass_c", "grass_d"].into_iter().filter(|m| b.has_model(m)).collect();
    if tufts.is_empty() {
        return;
    }
    let step = 1.25;
    let mut n = 0;
    let mut z = -150.0;
    while z < 150.0 {
        let mut x = -150.0;
        while x < 150.0 {
            let (px, pz) = (x + b.rng.range(-0.6, 0.6), z + b.rng.range(-0.6, 0.6));
            x += step;
            let d = (px * px + pz * pz).sqrt();
            let keep = if d < 90.0 { 1.0 } else { (1.0 - (d - 90.0) / 70.0).max(0.12) };
            if !b.rng.chance(keep) {
                continue;
            }
            // Gresset gror der bakken er gress (ikke på stein, jord eller berg).
            let gw = super::town::ground_weights(px, pz);
            if gw.cobble + gw.dirt + gw.rock + gw.forest > 0.35 {
                continue;
            }
            let yaw = b.rng.range(0.0, std::f32::consts::TAU);
            let s = b.rng.range(1.0, 1.6);
            let sy = s * b.rng.range(0.85, 1.2);
            // Tørt der enga er tørr, ellers mest grønne tuster og strå.
            let dry = gw.dry;
            let r = b.rng.range(0.0, 1.0);
            let pick = if r < dry * 0.6 { 1 } else if r < 0.55 { 0 } else if r < 0.85 { 2 } else { 3 };
            let model = tufts[pick.min(tufts.len() - 1)];
            // Tusten vippes etter bakken, så den står i skråningene også.
            b.place(model, b.on_slope(Vec3::new(px, -0.02, pz), yaw) * Affine3A::from_scale(Vec3::new(s, sy, s)));
            n += 1;
        }
        z += step;
    }
    log::info!("Gress: {n} tuster");
}
