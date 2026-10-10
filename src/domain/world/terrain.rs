//! Terrenget: kjøpstaden ligger planert innenfor bymuren, kirkegården
//! utenfor nordporten er planert, og utenfor ruller åsene. Alt som bygges spør
//! `height` om hvor bakken er.

use glam::Vec3;

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

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Kirkegården utenfor nordporten er planert: midten (x, z) og halve mål.
pub const YARD_FLAT: (f32, f32, f32, f32) = (-17.0, -50.0, 12.0, 10.0);

/// Hvor langt ut fra bymuren et punkt er, i meter (negativt innenfor).
/// Muren følger et ujevnt, organisk omriss rundt torget.
pub fn town_edge(x: f32, z: f32) -> f32 {
    (x * x + z * z).sqrt() - wall_radius(z.atan2(x))
}

/// Murens avstand fra torget i retningen `a` (radianer fra +x mot +z).
pub fn wall_radius(a: f32) -> f32 {
    // Ujevnt og eggeformet, slik muren ble lagt rundt det som var bygd:
    // lengst ut mot sørøst, inntrykt i nordvest, med knekker.
    38.0 + 3.6 * (a + 0.4).sin() + 2.4 * (2.0 * a + 0.7).sin() + 1.5 * (3.0 * a + 1.9).sin() + 0.9 * (5.0 * a + 0.3).sin()
}

/// Terrenget uten planeringene: rullende åser som vokser utover.
fn natural(x: f32, z: f32) -> f32 {
    let r = (x * x + z * z).sqrt();
    let near = (noise(x * 0.025 + 3.0, z * 0.025) - 0.5) * 2.0;
    // Åsene: store, myke bølger fra muren og ut.
    let far = smooth((r - 45.0) / 80.0);
    let hills = ((noise(x * 0.012 + 11.0, z * 0.012 + 5.0) - 0.5) * 22.0 + (noise(x * 0.03 + 4.0, z * 0.03 + 9.0) - 0.5) * 5.0) * far;
    near + hills
}

/// Bakkehøyden i meter.
pub fn height(x: f32, z: f32) -> f32 {
    let raw = natural(x, z);
    // Byen er planert innenfor muren, med en jevn overgang utenfor.
    let t = smooth((town_edge(x, z) + 2.0) / 22.0);
    let mut h = raw * t;
    // Kirkegården: planert til høyden ved nordporten.
    let (cx, cz, hx, hz) = YARD_FLAT;
    let dx = ((x - cx).abs() - hx).max(0.0);
    let dz = ((z - cz).abs() - hz).max(0.0);
    let k = smooth(1.0 - (dx * dx + dz * dz).sqrt() / 12.0);
    h = h * (1.0 - k) + 0.0 * k;
    h
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
