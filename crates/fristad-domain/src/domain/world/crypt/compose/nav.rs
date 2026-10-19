//! Navigasjonsnettet: det gangbare i bitene rastres i ruter på `SUB` (0,2 m)
//! per plan, holdt `NAV_INSET` fra murene, hjørnene, søylene og det som
//! står på gulvet, og slås grådig sammen til rektangler med samme rom og
//! bryter. Trappene er skrå firkanter. `Nav::new` syr kantene sammen.

use super::grid::*;
use super::{Cell, CellOut, Open};
use crate::domain::world::crypt::level::{MarkDef, NavDef};
use glam::{Vec2, Vec3};
use std::collections::{BTreeMap, BTreeSet};

/// Rommet og bryteren (porten, snarveien, den falske veggen) til en rute.
pub type Label = (String, Option<String>);

const EPS: f32 = 1e-3;

fn gi(v: f32) -> i32 {
    (v / SUB).round() as i32
}

/// Avstanden mellom to rektangler (lo, hi).
fn rect_dist(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> f32 {
    let dx = (b.0.x - a.1.x).max(a.0.x - b.1.x).max(0.0);
    let dz = (b.0.y - a.1.y).max(a.0.y - b.1.y).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// Det gangbare i cella `c`.
#[allow(clippy::too_many_arguments)]
pub fn cell(out: &mut CellOut, c: &Cell, opens: &[Open; 4], switches: &[Option<String>; 4], fills: &[bool; 4], pillars: &[Vec2], pillar_r: f32, room: &str) {
    let center = Vec2::new(c.center().x, c.center().z);
    // Hver side med mur: hvor langt ut nettet går, og åpningen gjennom den.
    let walls: Vec<Option<f32>> = (0..4)
        .map(|w| {
            let profile_gang = c.face(w).1 == "gang";
            match opens[w] {
                Open::Arch => None,
                Open::Gang if profile_gang => None,
                o => Some(o.width()),
            }
        })
        .collect();
    let fill_rects: Vec<(Vec2, Vec2)> = (0..4)
        .filter(|&k| fills[k])
        .map(|k| {
            let v = vertex_pos(corner_vertex(c.i, c.j, k));
            let inward = Vec2::new(if v.x < center.x { 1.0 } else { -1.0 }, if v.y < center.y { 1.0 } else { -1.0 });
            let o = v + inward * WALL_T;
            (v.min(o), v.max(o))
        })
        .collect();
    for walk in &c.piece.walk {
        let [x0, z0, x1, z1] = walk.rect;
        let flat = (walk.y[0] - walk.y[1]).abs() < 1e-4;
        if flat {
            let a = c.point(Vec3::new(x0, 0.0, z0));
            let b = c.point(Vec3::new(x1, 0.0, z1));
            let (lo, hi) = (Vec2::new(a.x.min(b.x), a.z.min(b.z)), Vec2::new(a.x.max(b.x), a.z.max(b.z)));
            let y = c.y + walk.y[0];
            for gz in gi(lo.y)..gi(hi.y) {
                for gx in gi(lo.x)..gi(hi.x) {
                    let s = (Vec2::new(gx as f32 * SUB, gz as f32 * SUB), Vec2::new((gx + 1) as f32 * SUB, (gz + 1) as f32 * SUB));
                    if let Some(gate) = walkable(s, center, &walls, switches, &fill_rects, pillars, pillar_r) {
                        out.raster.push(((q(y), gz, gx), (room.to_string(), gate)));
                    }
                }
            }
        } else {
            // En trapp: firkanten i bitens rom, holdt fra murene på sidene.
            let mut r = [x0, z0, x1, z1];
            for l in 0..4 {
                let w = local_to_world(l, c.rot(), c.mirror());
                if walls[w].is_some() {
                    let lim = FACE - NAV_INSET;
                    match l {
                        0 => r[1] = r[1].max(-lim),
                        1 => r[2] = r[2].min(lim),
                        2 => r[3] = r[3].min(lim),
                        _ => r[0] = r[0].max(-lim),
                    }
                }
            }
            let y_at = |z: f32| walk.y[0] + (z - z0) / (z1 - z0) * (walk.y[1] - walk.y[0]);
            let pts = [(r[0], r[1]), (r[2], r[1]), (r[2], r[3]), (r[0], r[3])].map(|(x, z)| mm3(c.point(Vec3::new(x, y_at(z), z))));
            out.slopes.push((pts.to_vec(), (room.to_string(), None)));
        }
    }
}

fn q(y: f32) -> i32 {
    (y * 1000.0).round() as i32
}

/// Om ruta `s` er gangbar i cella med midten `center`, og i så fall bryteren
/// den står bak (i åpningen gjennom en port).
fn walkable(s: (Vec2, Vec2), center: Vec2, walls: &[Option<f32>], switches: &[Option<String>; 4], fills: &[(Vec2, Vec2)], pillars: &[Vec2], pillar_r: f32) -> Option<Option<String>> {
    let lim = FACE - NAV_INSET;
    let mut gate = None;
    for w in 0..4 {
        let Some(width) = walls[w] else { continue };
        let o = outward_v(w);
        let along = Vec2::new(-o.y, o.x);
        // Hvor langt ut mot siden ruta når, og hvor den står langs siden.
        let corners = [s.0, Vec2::new(s.1.x, s.0.y), s.1, Vec2::new(s.0.x, s.1.y)].map(|p| p - center);
        let out_max = corners.iter().map(|p| p.dot(o)).fold(f32::MIN, f32::max);
        if out_max <= lim + EPS {
            continue;
        }
        let a_min = corners.iter().map(|p| p.dot(along)).fold(f32::MAX, f32::min);
        let a_max = corners.iter().map(|p| p.dot(along)).fold(f32::MIN, f32::max);
        let half = width * 0.5 - NAV_INSET;
        if half > 0.0 && a_min >= -half - EPS && a_max <= half + EPS {
            if switches[w].is_some() {
                gate = switches[w].clone();
            }
            continue;
        }
        return None;
    }
    for f in fills {
        if rect_dist(s, *f) < NAV_INSET - EPS {
            return None;
        }
    }
    for p in pillars {
        if rect_dist(s, (*p, *p)) < pillar_r + NAV_INSET - EPS {
            return None;
        }
    }
    Some(gate)
}

/// Fotavtrykket til det som står på gulvet (i merkets rom: x0, z0, x1, z1).
fn footprint(kind: &str) -> Option<[f32; 4]> {
    Some(match kind {
        "rest" => [-0.8, -0.5, 0.8, 0.5],
        "chest" | "rare" => [-0.45, -0.3, 0.45, 0.3],
        "lever" => [-0.25, -0.25, 0.25, 0.15],
        "lore" | "bonus" => [-0.3, -0.3, 0.3, 0.3],
        "up" => [-1.6, -1.2, 1.6, 0.1],
        "down" => [-1.5, -4.0, 1.5, 0.15],
        _ => return None,
    })
}

/// Hull i nettet rundt det som står på gulvet (fotavtrykket og `NAV_INSET`).
pub fn cut_marks(raster: &mut BTreeMap<(i32, i32, i32), Label>, marks: &[MarkDef]) {
    for m in marks {
        let (Some(f), Some(p)) = (footprint(&m.kind), m.pos) else { continue };
        let p = Vec3::from(p);
        let rot = glam::Quat::from_rotation_y(m.yaw);
        let pts = [(f[0], f[1]), (f[2], f[1]), (f[2], f[3]), (f[0], f[3])].map(|(x, z)| p + rot * Vec3::new(x, 0.0, z));
        let lo = pts.iter().fold(Vec2::splat(f32::MAX), |a, v| a.min(Vec2::new(v.x, v.z))) - Vec2::splat(NAV_INSET);
        let hi = pts.iter().fold(Vec2::splat(f32::MIN), |a, v| a.max(Vec2::new(v.x, v.z))) + Vec2::splat(NAV_INSET);
        raster.retain(|&(y, gz, gx), _| {
            let yy = y as f32 * 0.001;
            if (yy - p.y).abs() > 0.6 {
                return true;
            }
            let (x0, z0) = (gx as f32 * SUB, gz as f32 * SUB);
            !(x0 + SUB > lo.x + EPS && x0 < hi.x - EPS && z0 + SUB > lo.y + EPS && z0 < hi.y - EPS)
        });
    }
}

/// Nettet: rutene slått sammen til rektangler per plan, og trappene.
pub fn build(raster: &BTreeMap<(i32, i32, i32), Label>, slopes: &[(Vec<[f32; 3]>, Label)]) -> NavDef {
    let mut nav = NavDef::default();
    let mut index: BTreeMap<[i32; 3], u32> = BTreeMap::new();
    let mut vert = |nav: &mut NavDef, p: [f32; 3]| -> u32 {
        let k = [(p[0] * 1000.0).round() as i32, (p[1] * 1000.0).round() as i32, (p[2] * 1000.0).round() as i32];
        *index.entry(k).or_insert_with(|| {
            nav.verts.push([mm(p[0]), mm(p[1]), mm(p[2])]);
            (nav.verts.len() - 1) as u32
        })
    };
    let mut seen: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    for (&(y, gz, gx), label) in raster {
        if seen.contains(&(y, gz, gx)) {
            continue;
        }
        let same = |k: (i32, i32, i32), seen: &BTreeSet<(i32, i32, i32)>| raster.get(&k) == Some(label) && !seen.contains(&k);
        // Så langt mot +x som mulig, og så rad for rad mot +z.
        let mut x1 = gx + 1;
        while same((y, gz, x1), &seen) {
            x1 += 1;
        }
        let mut z1 = gz + 1;
        while (gx..x1).all(|x| same((y, z1, x), &seen)) {
            z1 += 1;
        }
        for z in gz..z1 {
            for x in gx..x1 {
                seen.insert((y, z, x));
            }
        }
        let fy = y as f32 * 0.001;
        let (ax, az, bx, bz) = (gx as f32 * SUB, gz as f32 * SUB, x1 as f32 * SUB, z1 as f32 * SUB);
        let poly = [[ax, fy, az], [bx, fy, az], [bx, fy, bz], [ax, fy, bz]].map(|p| vert(&mut nav, p));
        nav.polys.push(poly.to_vec());
        nav.poly_room.push(Some(label.0.clone()));
        nav.poly_gate.push(label.1.clone());
    }
    for (pts, label) in slopes {
        let poly: Vec<u32> = pts.iter().map(|p| vert(&mut nav, *p)).collect();
        nav.polys.push(poly);
        nav.poly_room.push(Some(label.0.clone()));
        nav.poly_gate.push(label.1.clone());
    }
    nav
}
