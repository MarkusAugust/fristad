//! Landskapet rundt byen: åsene utenfor terrenget og kanten av verden.
//! Trærne, buskene, gresset og steinene er slettet til de nye modellene
//! kommer (`docs/ny-alt.md`).

use super::{terrain, Builder, Shape};
use crate::domain::geometry::{Layer, Mat};
use glam::Vec3;

/// Landskapet rundt terrenget: fra kanten av bakken (±200 m) stiger
/// rullende åser i ett sammenhengende polart rutenett. Lavt er det eng,
/// bratt og høyt er det berg. Ingen kollider: kanten av terrenget (`world_edge`) stopper deg
/// før du kommer dit.
pub fn landscape(b: &mut Builder) {
    let (r0, r1) = (196.0f32, 600.0f32);
    let (rings, spokes) = (64usize, 288usize);
    let grass = Mat::tex(Layer::Meadow, 0xffffff);
    // Granitten er lys i bildet; i bratte sider dempes den mot gråbrun.
    let rock = Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.5, 0.47, 0.44));
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
            // Bratte og høye sider er berg.
            let mat = if rise > 60.0 || up < 0.62 {
                &rock
            } else {
                &grass
            };
            b.mesh
                .quad_n([p[3], p[2], p[1], p[0]], [n[3], n[2], n[1], n[0]], mat, 1.0);
        }
    }
}

/// Høyden i landskapet utenfor terrenget: bakken glir over i rullende
/// åser fra 200 m, som blir høyere og mykere utover mot horisonten.
pub fn landscape_height(x: f32, z: f32) -> f32 {
    let r = (x * x + z * z).sqrt();
    let base = terrain::height(x, z);
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let hills = smooth((r - 200.0) / 70.0)
        * (terrain::noise(x * 0.012 + 20.0, z * 0.012 + 20.0) * 28.0
            + terrain::noise(x * 0.03 + 40.0, z * 0.03) * 8.0);
    // Lenger ut: bare høyere, myke åser, ingen fjell.
    let far =
        smooth((r - 260.0) / 200.0) * (terrain::noise(x * 0.005 + 70.0, z * 0.005 + 30.0) * 60.0);
    base + hills + far
}

/// Usynlige vegger rundt kanten av terrenget (±200 m), så ingen faller
/// ut av verden eller går inn i fjellene.
pub fn world_edge(b: &mut Builder) {
    let n = 36;
    let radius = 199.0;
    for i in 0..n {
        let a = (i as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
        // Fra 30 m under null (dalene mellom åsene) og opp. Kjelleren holder
        // seg innenfor ±185 m, så den går ikke inn i veggen.
        let c = Vec3::new(a.sin() * radius, 27.5, a.cos() * radius);
        b.solid(Shape::Cuboid {
            center: c,
            size: Vec3::new(
                2.0 * std::f32::consts::PI * radius / n as f32 + 2.0,
                115.0,
                2.0,
            ),
            yaw: a,
        });
    }
}
