//! Rommene: omrisset av cellene i hvert rom (kantene som ikke deles med en
//! annen celle i rommet, lenket rundt), gulvet og høyden.

use super::grid::*;
use super::{Grid, Issue};
use crate::domain::world::crypt::doc::DocRoom;
use crate::domain::world::crypt::level::RoomDef;
use std::collections::{BTreeMap, BTreeSet};

pub fn rooms(g: &Grid, list: &[DocRoom]) -> (Vec<RoomDef>, Vec<Issue>) {
    let mut out = vec![];
    let mut issues = vec![];
    for r in list {
        let cells: Vec<usize> = (0..g.cells.len()).filter(|&k| g.cells[k].doc.room == r.id).collect();
        if cells.is_empty() {
            continue;
        }
        let squares: BTreeSet<(i32, i32)> = cells.iter().map(|&k| (g.cells[k].i, g.cells[k].j)).collect();
        let loops = outline(&squares);
        if loops.len() > 1 {
            issues.push(Issue::warn("rom", Some(g.cells[cells[0]].center()), format!("rommet «{}» har hull eller flere deler; bare det ytterste omrisset brukes", r.id)));
        }
        let Some(poly) = loops.into_iter().max_by(|a, b| area(a).abs().total_cmp(&area(b).abs())) else { continue };
        let y = cells
            .iter()
            .map(|&k| {
                let c = &g.cells[k];
                c.y + c.piece.floors.iter().copied().fold(f32::MAX, f32::min)
            })
            .fold(f32::MAX, f32::min);
        let top = cells
            .iter()
            .map(|&k| {
                let c = &g.cells[k];
                c.y + c.piece.floors.iter().copied().fold(f32::MIN, f32::max) + c.piece.clear
            })
            .fold(f32::MIN, f32::max);
        // Hvelvet: toppen av bitene (`top`), over den frie høyden.
        let vault = cells.iter().map(|&k| g.cells[k].y + g.cells[k].piece.top).fold(f32::MIN, f32::max);
        let rise = (vault - top).max(0.0);
        out.push(RoomDef { id: r.id.clone(), name: if r.name.is_empty() { r.id.clone() } else { r.name.clone() }, poly, y: mm(y), h: mm(top - y), rise: mm(rise), secret: r.secret, fog: mm(r.mood.fog.clamp(0.0, 1.0)), sound: r.mood.sound.clone() });
    }
    // Hvelvet når aldri opp i et rom over (to etasjer over hverandre).
    let boxes: Vec<([f32; 4], f32)> = out.iter().map(|r| (bounds(&r.poly), r.y)).collect();
    for r in out.iter_mut() {
        let b = bounds(&r.poly);
        let roof = r.y + r.h;
        for (o, oy) in &boxes {
            let over = b[0] < o[2] - 0.1 && o[0] < b[2] - 0.1 && b[1] < o[3] - 0.1 && o[1] < b[3] - 0.1;
            if over && *oy > roof - 0.05 {
                r.rise = mm((oy - roof - 0.35).clamp(0.0, r.rise));
            }
        }
    }
    (out, issues)
}

/// Rommets utstrekning: [min x, min z, maks x, maks z].
fn bounds(p: &[[f32; 2]]) -> [f32; 4] {
    p.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, q| [b[0].min(q[0]), b[1].min(q[1]), b[2].max(q[0]), b[3].max(q[1])])
}

fn area(p: &[[f32; 2]]) -> f32 {
    (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f32>()
        * 0.5
}

/// Omrisset av rutene (i, j) som lukkede løkker i nivåets rom.
fn outline(squares: &BTreeSet<(i32, i32)>) -> Vec<Vec<[f32; 2]>> {
    // Kantene i hjørnerutenettet, rettet rundt hver rute.
    let mut edges: BTreeMap<(i32, i32), Vec<(i32, i32)>> = BTreeMap::new();
    let mut count = 0;
    for &(i, j) in squares {
        let c = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)];
        for k in 0..4 {
            let (a, b) = (c[k], c[(k + 1) % 4]);
            // Kanten deles med naboruta: den går motsatt vei der.
            if let Some(v) = edges.get_mut(&b) {
                if let Some(p) = v.iter().position(|x| *x == a) {
                    v.remove(p);
                    count -= 1;
                    continue;
                }
            }
            edges.entry(a).or_default().push(b);
            count += 1;
        }
    }
    let mut loops = vec![];
    while count > 0 {
        let Some((&start, _)) = edges.iter().find(|(_, v)| !v.is_empty()) else { break };
        let mut pts = vec![start];
        let mut at = start;
        loop {
            let Some(next) = edges.get_mut(&at).and_then(|v| if v.is_empty() { None } else { Some(v.remove(0)) }) else { break };
            count -= 1;
            if next == start {
                break;
            }
            pts.push(next);
            at = next;
        }
        // Rette linjer: hjørnene midt på en rett kant tas bort.
        let n = pts.len();
        let keep: Vec<(i32, i32)> = (0..n)
            .filter(|&k| {
                let (a, b, c) = (pts[(k + n - 1) % n], pts[k], pts[(k + 1) % n]);
                (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0) != 0
            })
            .map(|k| pts[k])
            .collect();
        loops.push(
            keep.into_iter()
                .map(|v| {
                    let p = vertex_pos(v);
                    [mm(p.x), mm(p.y)]
                })
                .collect(),
        );
    }
    loops
}
