//! Rutinen mot tekst utenfor skilt: hver bokstav i byen (i bymeshen og i
//! propsene) må ha ei plate rett bak seg. For hvert hjørne av hver
//! teksttrekant skytes en stråle bakover langs normalen; treffer den ingen
//! annen flate innen `DEPTH`, står bokstaven utenfor plata si.

use crate::domain::geometry::{kind, Mesh};
use glam::{Affine3A, Vec3};
use std::collections::HashMap;

const DEPTH: f32 = 0.08;
const CELL: f32 = 0.5;

struct Grid {
    tris: Vec<[Vec3; 3]>,
    cells: HashMap<(i32, i32, i32), Vec<u32>>,
    /// Bare flater innenfor dette området (rundt all teksten) tas med;
    /// landskapet langt ute ville ellers fylt millioner av celler.
    lo: Vec3,
    hi: Vec3,
}

impl Grid {
    fn new(lo: Vec3, hi: Vec3) -> Self {
        Self { tris: vec![], cells: HashMap::new(), lo, hi }
    }

    fn add(&mut self, t: [Vec3; 3]) {
        let lo = t[0].min(t[1]).min(t[2]);
        let hi = t[0].max(t[1]).max(t[2]);
        // Store flater (bakken, åsene, fjellene) hopper vi over: de er
        // aldri plater, og de ville fylt tusenvis av celler. Det samme med
        // alt som ligger utenfor området teksten står i.
        if (hi - lo).max_element() > 24.0 || hi.cmplt(self.lo).any() || lo.cmpgt(self.hi).any() {
            return;
        }
        let i = self.tris.len() as u32;
        self.tris.push(t);
        let c0 = (lo / CELL).floor().as_ivec3();
        let c1 = (hi / CELL).floor().as_ivec3();
        for x in c0.x..=c1.x {
            for y in c0.y..=c1.y {
                for z in c0.z..=c1.z {
                    self.cells.entry((x, y, z)).or_default().push(i);
                }
            }
        }
    }

    /// Om strålen fra `o` langs `d` (enhet) treffer en trekant innen `max`.
    fn hits(&self, o: Vec3, d: Vec3, max: f32) -> bool {
        let p1 = o + d * max;
        let lo = o.min(p1);
        let hi = o.max(p1);
        let c0 = (lo / CELL).floor().as_ivec3();
        let c1 = (hi / CELL).floor().as_ivec3();
        for x in c0.x..=c1.x {
            for y in c0.y..=c1.y {
                for z in c0.z..=c1.z {
                    let Some(list) = self.cells.get(&(x, y, z)) else { continue };
                    for &i in list {
                        if let Some(t) = ray_tri(o, d, &self.tris[i as usize]) {
                            if t >= -0.002 && t <= max {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

fn ray_tri(o: Vec3, d: Vec3, t: &[Vec3; 3]) -> Option<f32> {
    let e1 = t[1] - t[0];
    let e2 = t[2] - t[0];
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-7 {
        return None;
    }
    let inv = 1.0 / det;
    let s = o - t[0];
    let u = s.dot(p) * inv;
    if !(-0.001..=1.001).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < -0.001 || u + v > 1.001 {
        return None;
    }
    Some(e2.dot(q) * inv)
}

fn tris(mesh: &Mesh, t: &Affine3A, text: bool) -> Vec<([Vec3; 3], Vec3)> {
    let mut out = vec![];
    for idx in mesh.indices.chunks(3) {
        if idx.len() < 3 {
            break;
        }
        let v = [&mesh.vertices[idx[0] as usize], &mesh.vertices[idx[1] as usize], &mesh.vertices[idx[2] as usize]];
        let is_text = v[0].mat[0] == kind::TEXT;
        if is_text != text {
            continue;
        }
        let p = [t.transform_point3(Vec3::from(v[0].pos)), t.transform_point3(Vec3::from(v[1].pos)), t.transform_point3(Vec3::from(v[2].pos))];
        let n = t.transform_vector3(Vec3::from(v[0].normal)).normalize_or_zero();
        out.push((p, n));
    }
    out
}

/// Sjekker bymeshen og propsene (i hvilestillingen). Gir én melding per
/// halvmeter der bokstaver står utenfor plata.
pub fn audit(mesh: &Mesh, props: &[(&Mesh, Affine3A)]) -> Vec<String> {
    // Området teksten står i, med en margin.
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    let mut extend = |p: [Vec3; 3]| {
        for c in p {
            lo = lo.min(c);
            hi = hi.max(c);
        }
    };
    for (p, _) in tris(mesh, &Affine3A::IDENTITY, true) {
        extend(p);
    }
    for (m, rest) in props {
        if rest.matrix3.x_axis.length_squared() < 1e-6 {
            continue;
        }
        for (p, _) in tris(m, rest, true) {
            extend(p);
        }
    }
    let mut grid = Grid::new(lo - Vec3::splat(2.0), hi + Vec3::splat(2.0));
    for (t, _) in tris(mesh, &Affine3A::IDENTITY, false) {
        grid.add(t);
    }
    for (m, rest) in props {
        // Gjemte props (skala ~0) teller ikke.
        if rest.matrix3.x_axis.length_squared() < 1e-6 {
            continue;
        }
        for (t, _) in tris(m, rest, false) {
            grid.add(t);
        }
    }
    if std::env::var("TEXT_AUDIT_STATS").is_ok() {
        let n_text = tris(mesh, &Affine3A::IDENTITY, true).len();
        let n_all = mesh.indices.len() / 3;
        let near: Vec<&(i32, i32, i32)> = grid.cells.keys().filter(|(x, y, z)| (*x - 40).abs() <= 1 && (*y - 6).abs() <= 1 && (*z - 26).abs() <= 1).collect();
        log::warn!("tekstdebug: {} trekanter i gitteret av {n_all} i bymeshen, {n_text} teksttrekanter, {} celler, celler rundt (40,6,26): {near:?}", grid.tris.len(), grid.cells.len());
    }
    let mut bad: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut shown = 0;
    let mut check = |p: [Vec3; 3], n: Vec3| {
        for c in p {
            if !grid.hits(c, -n, DEPTH) {
                let at = std::env::var("TEXT_AUDIT_AT").ok().and_then(|v| {
                    let p: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                    (p.len() == 3).then(|| Vec3::new(p[0], p[1], p[2]))
                });
                if shown < 2 && at.is_some_and(|a| a.distance(c) < 1.0) {
                    shown += 1;
                    let cell = ((c.x / CELL).floor() as i32, (c.y / CELL).floor() as i32, (c.z / CELL).floor() as i32);
                    let list = grid.cells.get(&cell).map(|l| l.len()).unwrap_or(0);
                    let mut nearest: Vec<(f32, [Vec3; 3])> = grid.cells.get(&cell).map(|l| l.iter().filter_map(|&i| ray_tri(c, -n, &grid.tris[i as usize]).map(|t| (t, grid.tris[i as usize]))).collect()).unwrap_or_default();
                    nearest.sort_by(|a, b| a.0.abs().partial_cmp(&b.0.abs()).unwrap());
                    log::warn!("tekstdebug: hjørne {c:?} normal {n:?}, {list} trekanter i cella, nærmeste langs strålen: {:?}", nearest.iter().take(3).collect::<Vec<_>>());
                    let near: Vec<String> = grid.tris.iter().filter(|t| t.iter().any(|v| v.distance(c) < 0.3)).take(6).map(|t| format!("{:?}", t)).collect();
                    log::warn!("tekstdebug: trekanter i gitteret innen 30 cm: {}", near.join(" | "));
                    let cells: Vec<_> = grid.cells.keys().filter(|(x, y, z)| (*x - cell.0).abs() <= 1 && (*y - cell.1).abs() <= 1 && (*z - cell.2).abs() <= 1).collect();
                    log::warn!("tekstdebug: celle {cell:?}, naboceller {cells:?}");
                }
                *bad.entry(((c.x * 2.0).round() as i32, (c.y * 2.0).round() as i32, (c.z * 2.0).round() as i32)).or_default() += 1;
            }
        }
    };
    for (p, n) in tris(mesh, &Affine3A::IDENTITY, true) {
        check(p, n);
    }
    for (m, rest) in props {
        if rest.matrix3.x_axis.length_squared() < 1e-6 {
            continue;
        }
        for (p, n) in tris(m, rest, true) {
            check(p, n);
        }
    }
    let mut out: Vec<String> = bad.iter().filter(|(_, n)| **n >= 3).map(|((x, y, z), n)| format!("tekst uten plate bak ved ({:.1}, {:.1}, {:.1}): {n} hjørner", *x as f32 * 0.5, *y as f32 * 0.5, *z as f32 * 0.5)).collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Mat;
    use glam::Vec2;

    fn glyph(m: &mut Mesh, c: Vec3) {
        let mat = Mat::special(kind::TEXT, 0xffffff, 0.5);
        let h = 0.1;
        m.quad_uv(&Affine3A::IDENTITY, [c + Vec3::new(-h, -h, 0.0), c + Vec3::new(h, -h, 0.0), c + Vec3::new(h, h, 0.0), c + Vec3::new(-h, h, 0.0)], &mat, [1.0; 4], [Vec2::ZERO; 4]);
    }

    #[test]
    fn plate_bak_teksten_godtas() {
        let mut m = Mesh::default();
        m.cuboid_ao(&Affine3A::from_translation(Vec3::new(3.0, 1.5, -2.0)), Vec3::new(1.0, 0.5, 0.04), &Mat::plain(0x808080, 0.5), 1.0);
        glyph(&mut m, Vec3::new(3.0, 1.5, -2.0 + 0.02 + 0.015));
        assert!(audit(&m, &[]).is_empty(), "{:?}", audit(&m, &[]));
    }

    #[test]
    fn tekst_uten_plate_meldes() {
        let mut m = Mesh::default();
        glyph(&mut m, Vec3::new(3.0, 1.5, -2.0));
        assert!(!audit(&m, &[]).is_empty());
    }
}
