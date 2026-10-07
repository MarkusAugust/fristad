//! Terrenget: kjøpstaden ligger planert innenfor bymuren, borgen står på en
//! haug i nord, og utenfor bølger enga mot åsene. Alt som bygges spør
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

/// Borghaugen: midten, radius og høyde.
pub const HILL: (f32, f32, f32, f32) = (0.0, -122.0, 46.0, 15.0);
/// Borggården på toppen er planert innenfor denne radien.
pub const KEEP_FLAT: f32 = 24.0;

/// Hvor langt ut fra bymuren et punkt er, i meter (negativt innenfor).
/// Muren følger en avrundet firkant rundt torget.
pub fn town_edge(x: f32, z: f32) -> f32 {
    let (hx, hz, r) = (44.0, 40.0, 12.0);
    let qx = (x.abs() - hx).max(0.0);
    let qz = (z.abs() - hz).max(0.0);
    let outside = (qx * qx + qz * qz).sqrt();
    let inside = (x.abs() - hx).max(z.abs() - hz).min(0.0);
    outside + inside - r
}

/// Terrenget uten planeringene.
fn natural(x: f32, z: f32) -> f32 {
    // En svak helling opp mot nord, og bølger på 40–90 meter.
    let slope = -z * 0.018 + x * 0.004;
    let waves = (noise(x * 0.025 + 3.0, z * 0.025) - 0.5) * 2.4 + (noise(x * 0.06, z * 0.06 + 7.0) - 0.5) * 0.8;
    // Lenger ute ruller landskapet mer.
    let far = ((x * x + z * z).sqrt() - 70.0).max(0.0) / 90.0;
    let rolling = (noise(x * 0.011 + 11.0, z * 0.011 + 5.0) - 0.5) * 10.0 * far.min(1.0);
    slope + waves + rolling
}

/// Bakkehøyden i meter.
pub fn height(x: f32, z: f32) -> f32 {
    let raw = natural(x, z);
    // Byen er planert innenfor muren, med en jevn overgang utenfor.
    let t = smooth((town_edge(x, z) + 2.0) / 16.0);
    let mut h = raw * t;
    // Borghaugen, med flat topp der borgen står.
    let (hx, hz, hr, hh) = HILL;
    let d = ((x - hx).powi(2) + (z - hz).powi(2)).sqrt();
    let top = hh + natural(hx, hz);
    if d < hr {
        let k = smooth(1.0 - (d - KEEP_FLAT).max(0.0) / (hr - KEEP_FLAT));
        // Skråningen er litt ujevn, som en naturlig haug.
        let lumps = (noise(x * 0.08, z * 0.08 + 2.0) - 0.5) * 1.6 * (1.0 - smooth(1.0 - (d - KEEP_FLAT).max(0.0) / 8.0));
        h = h * (1.0 - k) + top * k + lumps * k;
    }
    // Veien opp til borgen er skåret inn i haugen.
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
