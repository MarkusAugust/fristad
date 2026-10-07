//! Terrenget: byen ligger i en svak helling, med torget planert. Gata bak
//! porten stiger mot nord, gata i sør faller litt, og utenfor husene bølger
//! bakken svakt. Alt som bygges spør `height` om hvor bakken er.

use glam::Vec3;

/// Hvor langt torget er planert, og hvor bredt overgangen er.
const FLAT: f32 = 21.0;
const BLEND: f32 = 12.0;

fn hash(x: f32, z: f32) -> f32 {
    let v = (x * 127.1 + z * 311.7).sin() * 43758.547;
    v - v.floor()
}

pub fn noise(x: f32, z: f32) -> f32 {
    let (ix, iz) = (x.floor(), z.floor());
    let (fx, fz) = (x - ix, z - iz);
    let (ux, uz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let a = hash(ix, iz);
    let b = hash(ix + 1.0, iz);
    let c = hash(ix, iz + 1.0);
    let d = hash(ix + 1.0, iz + 1.0);
    let top = a + (b - a) * ux;
    let bottom = c + (d - c) * ux;
    top + (bottom - top) * uz
}

fn d_from_center(x: f32, z: f32) -> f32 {
    (x * x + z * z).sqrt()
}

/// Bakkehøyden i meter.
pub fn height(x: f32, z: f32) -> f32 {
    // Hellingen: opp mot nord, med en svak skråning mot vest.
    let slope = -z * 0.022 + x * 0.006;
    // Bølger på 40–90 meter.
    let waves = (noise(x * 0.025 + 3.0, z * 0.025) - 0.5) * 2.2 + (noise(x * 0.06, z * 0.06 + 7.0) - 0.5) * 0.7;
    // Lenger ute ruller landskapet mer.
    let far = ((d_from_center(x, z) - 50.0) / 90.0).clamp(0.0, 1.0);
    let rolling = (noise(x * 0.011 + 11.0, z * 0.011 + 5.0) - 0.5) * 9.0 * far;
    let raw = slope + waves + rolling;
    // Torget er planert: helt flatt innenfor FLAT, og glir over til terrenget.
    let d = x.abs().max(z.abs());
    let t = ((d - FLAT) / BLEND).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    raw * t
}

pub fn at(p: Vec3) -> Vec3 {
    Vec3::new(p.x, height(p.x, p.z), p.z)
}

/// Normalen til bakken, til å legge ting flatt på den.
pub fn normal(x: f32, z: f32) -> Vec3 {
    let e = 0.5;
    let dx = height(x + e, z) - height(x - e, z);
    let dz = height(x, z + e) - height(x, z - e);
    Vec3::new(-dx, 2.0 * e, -dz).normalize()
}
