//! Formene kolliderne har (`Shape`), med hvor de dekker og hvordan de
//! settes ut med en pose.

use glam::{Affine3A, Quat, Vec2, Vec3};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    /// Boks med sentrum, hele mål og rotasjon om Y.
    Cuboid { center: Vec3, size: Vec3, yaw: f32 },
    /// Stående sylinder med sentrum, radius og halv høyde.
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
    /// Terrenget: et rutenett av høyder, `cols` langs X og `rows` langs Z,
    /// sentrert i `center` og `size` meter stort.
    /// `holes` er rutene (rad langs Z, kolonne langs X) som er fjernet, så
    /// man kan gå ned i en kjeller under et hus.
    Heightfield { center: Vec3, size: Vec3, cols: usize, rows: usize, heights: Vec<f32>, holes: Vec<(usize, usize)> },
    /// Kapsel mellom to punkter (trestammer og greiner).
    Capsule { a: Vec3, b: Vec3, radius: f32 },
    /// Konveks omhylling av punktene (trappetrinn, fjell).
    Hull { points: Vec<Vec3> },
}

impl Shape {
    /// Ligger punktet i kollideren, med `tol` meter slingring? Terrenget og
    /// skallene teller ikke (de er bakke og trapper, ikke ting).
    pub fn covers(&self, q: Vec3, tol: f32) -> bool {
        match self {
            Shape::Cuboid { center, size, yaw } => {
                let d = Quat::from_rotation_y(-*yaw) * (q - *center);
                d.x.abs() <= size.x * 0.5 + tol && d.y.abs() <= size.y * 0.5 + tol && d.z.abs() <= size.z * 0.5 + tol
            }
            Shape::Cylinder { center, radius, half_height } => {
                let d = q - *center;
                (d.x * d.x + d.z * d.z).sqrt() <= radius + tol && d.y.abs() <= half_height + tol
            }
            Shape::Capsule { a, b, radius } => {
                let ab = *b - *a;
                let t = ((q - *a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
                (q - (*a + ab * t)).length() <= radius + tol
            }
            Shape::Heightfield { .. } | Shape::Hull { .. } => false,
        }
    }

    /// Som `covers`, og et skrog som er et prisme (kolliderne som følger
    /// formen, `solids.py`: samme omriss i bunnen og toppen) teller også.
    pub fn covers_shaped(&self, q: Vec3, tol: f32) -> bool {
        match self {
            Shape::Hull { points } => prism(points).is_some_and(|(ring, y0, y1)| q.y >= y0 - tol && q.y <= y1 + tol && in_ring(&ring, Vec2::new(q.x, q.z), tol)),
            other => other.covers(q, tol),
        }
    }

    /// Formen satt ut med posen `t` (en modells kollidere i verden). En boks
    /// skaleres langs sine egne akser; terrenget flyttes ikke.
    pub fn posed(&self, t: &Affine3A) -> Shape {
        let (scale, rot, _) = t.to_scale_rotation_translation();
        let (axis, angle) = rot.to_axis_angle();
        let yaw_of = if axis.y < 0.0 { -angle } else { angle };
        match self {
            Shape::Cuboid { center, size, yaw } => Shape::Cuboid { center: t.transform_point3(*center), size: *size * scale, yaw: *yaw + yaw_of },
            Shape::Cylinder { center, radius, half_height } => Shape::Cylinder { center: t.transform_point3(*center), radius: radius * scale.x.max(scale.z), half_height: half_height * scale.y },
            Shape::Capsule { a, b, radius } => Shape::Capsule { a: t.transform_point3(*a), b: t.transform_point3(*b), radius: radius * scale.x.max(scale.z) },
            Shape::Hull { points } => Shape::Hull { points: points.iter().map(|p| t.transform_point3(*p)).collect() },
            other => other.clone(),
        }
    }

    /// Hjørnene (eller punktene) formen spenner over, til mål og siktemål.
    pub fn corners(&self) -> Vec<Vec3> {
        match self {
            Shape::Cuboid { center, size, yaw } => {
                let r = Quat::from_rotation_y(*yaw);
                (0..8).map(|k| *center + r * (Vec3::new(if k & 1 == 0 { -0.5 } else { 0.5 }, if k & 2 == 0 { -0.5 } else { 0.5 }, if k & 4 == 0 { -0.5 } else { 0.5 }) * *size)).collect()
            }
            Shape::Cylinder { center, radius, half_height } => (0..8)
                .map(|k| *center + Vec3::new(if k & 1 == 0 { -radius } else { *radius }, if k & 2 == 0 { -half_height } else { *half_height }, if k & 4 == 0 { -radius } else { *radius }))
                .collect(),
            Shape::Capsule { a, b, radius } => vec![a.min(*b) - Vec3::splat(*radius), a.max(*b) + Vec3::splat(*radius)],
            Shape::Hull { points } => points.clone(),
            Shape::Heightfield { .. } => vec![],
        }
    }
}

/// Omrisset (i rekkefølge), bunnen og toppen når skroget er et prisme: halve
/// punktene i bunnen og halve i toppen, rett over hverandre.
fn prism(points: &[Vec3]) -> Option<(Vec<Vec2>, f32, f32)> {
    let n = points.len() / 2;
    if n < 3 || points.len() != 2 * n {
        return None;
    }
    let (lo, hi) = points.split_at(n);
    let (y0, y1) = (lo[0].y, hi[0].y);
    let same = |a: &[Vec3], y: f32| a.iter().all(|p| (p.y - y).abs() < 1e-3);
    if !same(lo, y0) || !same(hi, y1) || lo.iter().zip(hi).any(|(a, b)| (a.x - b.x).abs() > 1e-3 || (a.z - b.z).abs() > 1e-3) {
        return None;
    }
    Some((lo.iter().map(|p| Vec2::new(p.x, p.z)).collect(), y0.min(y1), y0.max(y1)))
}

/// Om punktet ligger i den konvekse ringen, eller innen `tol` fra kanten.
fn in_ring(ring: &[Vec2], q: Vec2, tol: f32) -> bool {
    let n = ring.len();
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let sides: Vec<f32> = (0..n).map(|i| cross(ring[(i + 1) % n] - ring[i], q - ring[i])).collect();
    if sides.iter().all(|s| *s >= 0.0) || sides.iter().all(|s| *s <= 0.0) {
        return true;
    }
    (0..n).any(|i| {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        let t = ((q - a).dot(b - a) / (b - a).length_squared().max(1e-9)).clamp(0.0, 1.0);
        (a + (b - a) * t).distance(q) <= tol
    })
}
