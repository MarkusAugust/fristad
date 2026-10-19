//! Kolliderne: bitenes egne (`solids` i manifestet), flyttet på plass og
//! flettet. Bokser som står rett etter dreiningen (alle dreininger er
//! kvarte omdreininger) slås sammen der de har samme høyde og bredde og
//! møtes: gulvene blir store rektangler, så kapselen ikke henger seg opp i
//! skjøtene, og murene ett løp.

use super::grid::{mm, mm3};
use crate::domain::world::crypt::level::{BoxDef, ColliderDef};
use glam::{Affine3A, Quat, Vec3};
use std::f32::consts::FRAC_PI_2;

/// En boks langs aksene, i mm (heltall, så sammenligningen er eksakt).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aabb {
    pub lo: [i32; 3],
    pub hi: [i32; 3],
}

impl Aabb {
    pub fn def(&self) -> ColliderDef {
        let c = |k: usize| mm((self.lo[k] + self.hi[k]) as f32 * 0.0005);
        let s = |k: usize| mm((self.hi[k] - self.lo[k]) as f32 * 0.001);
        ColliderDef { cuboid: Some(BoxDef { center: [c(0), c(1), c(2)], size: [s(0), s(1), s(2)], yaw: 0.0 }), hull: None }
    }
}

fn q(v: f32) -> i32 {
    (v * 1000.0).round() as i32
}

/// Legg til en kollider fra biten, med bitens form `t` (dreiningen `yaw`).
pub fn add(boxes: &mut Vec<Aabb>, hulls: &mut Vec<Vec<[f32; 3]>>, s: &ColliderDef, t: &Affine3A, yaw: f32) {
    if let Some(b) = &s.cuboid {
        let c = t.transform_point3(Vec3::from(b.center));
        let total = b.yaw + yaw;
        let turns = total / FRAC_PI_2;
        if (turns - turns.round()).abs() < 1e-3 {
            let odd = (turns.round() as i32).rem_euclid(2) == 1;
            let size = Vec3::from(b.size);
            let size = if odd { Vec3::new(size.z, size.y, size.x) } else { size };
            let lo = c - size * 0.5;
            let hi = c + size * 0.5;
            boxes.push(Aabb { lo: [q(lo.x), q(lo.y), q(lo.z)], hi: [q(hi.x), q(hi.y), q(hi.z)] });
        } else {
            // En skrå boks blir et skrog av hjørnene.
            let r = Quat::from_rotation_y(b.yaw);
            let h = Vec3::from(b.size) * 0.5;
            let mut pts = vec![];
            for k in 0..8 {
                let p = Vec3::new(if k & 1 == 0 { -h.x } else { h.x }, if k & 2 == 0 { -h.y } else { h.y }, if k & 4 == 0 { -h.z } else { h.z });
                pts.push(mm3(t.transform_point3(Vec3::from(b.center) + r * p)));
            }
            hulls.push(pts);
        }
    }
    if let Some(h) = &s.hull {
        hulls.push(h.iter().map(|p| mm3(t.transform_point3(Vec3::from(*p)))).collect());
    }
}

/// Slå sammen boksene som møtes langs x eller z med samme mål på tvers.
pub fn merge(mut boxes: Vec<Aabb>) -> Vec<Aabb> {
    boxes.sort();
    boxes.dedup();
    loop {
        let before = boxes.len();
        for axis in [0usize, 2] {
            boxes = sweep(boxes, axis);
        }
        // Bokser som ligger helt inni en annen.
        let mut keep = vec![true; boxes.len()];
        for i in 0..boxes.len() {
            for j in 0..boxes.len() {
                if i != j && keep[j] && inside(&boxes[i], &boxes[j]) && !(boxes[i] == boxes[j] && i < j) {
                    keep[i] = false;
                    break;
                }
            }
        }
        boxes = boxes.into_iter().zip(keep).filter(|(_, k)| *k).map(|(b, _)| b).collect();
        if boxes.len() == before {
            break;
        }
    }
    boxes.sort();
    boxes
}

fn inside(a: &Aabb, b: &Aabb) -> bool {
    (0..3).all(|k| a.lo[k] >= b.lo[k] && a.hi[k] <= b.hi[k])
}

/// Langs `axis`: boksene med de samme målene i de to andre aksene, som
/// møtes eller overlapper, blir én.
fn sweep(boxes: Vec<Aabb>, axis: usize) -> Vec<Aabb> {
    let others: Vec<usize> = (0..3).filter(|k| *k != axis).collect();
    let key = |b: &Aabb| (b.lo[others[0]], b.hi[others[0]], b.lo[others[1]], b.hi[others[1]], b.lo[axis]);
    let mut v = boxes;
    v.sort_by_key(key);
    let mut out: Vec<Aabb> = vec![];
    for b in v {
        if let Some(last) = out.last_mut() {
            let same = others.iter().all(|&k| last.lo[k] == b.lo[k] && last.hi[k] == b.hi[k]);
            if same && b.lo[axis] <= last.hi[axis] + 1 {
                last.hi[axis] = last.hi[axis].max(b.hi[axis]);
                continue;
            }
        }
        out.push(b);
    }
    out
}
