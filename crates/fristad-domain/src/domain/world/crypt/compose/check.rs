//! Reglene over resultatet (`docs/nivaaredigering.md`, del 10), regnet ut
//! for seg, uavhengig av valgene kompilatoren gjorde: de samme
//! funksjonene kjører i redigeringen og i `cargo test`.

use super::grid::*;
use super::{Issue, Severity};
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::kit::{KitDef, PieceKind};
use crate::domain::world::crypt::level::LevelFile;
use glam::Vec3;
use std::collections::HashMap;

/// En celle slik reglene ser den: ruta, gulvet, volumet og gulvene ved
/// sidene i nivåets rom.
struct C {
    id: String,
    i: i32,
    j: i32,
    y: f32,
    lo: f32,
    hi: f32,
    floors: [f32; 4],
}

fn cells(doc: &LevelDoc, kit: &KitDef) -> Vec<C> {
    super::expand(&super::modules::flatten(doc, kit).0, kit)
        .iter()
        .filter_map(|c| {
            let p = kit.piece(&c.bit).filter(|p| p.kind == PieceKind::Cell)?;
            let mirror = c.mirror && p.mirror;
            let y = c.at[1] as f32;
            let floors = [0, 1, 2, 3].map(|w| y + p.floors[world_to_local(w, c.rot % 4, mirror)]);
            Some(C { id: c.id.clone(), i: c.at[0], j: c.at[2], y, lo: y + p.bottom, hi: y + p.top.max(p.bottom + 0.5), floors })
        })
        .collect()
}

/// Bitene overlapper i volum: to celler i samme rute med volumer som går inn
/// i hverandre (gulvskive og hvelv er med). To moduler i samme celle er
/// den samme feilen, og meldes med modulene.
pub fn overlaps(doc: &LevelDoc, kit: &KitDef) -> Vec<Issue> {
    let cs = cells(doc, kit);
    let module = |id: &str| id.split_once('/').map(|(m, _)| m).filter(|m| doc.modules.iter().any(|x| x.id == *m)).map(str::to_string);
    let mut out = vec![];
    for a in 0..cs.len() {
        for b in a + 1..cs.len() {
            let (p, q) = (&cs[a], &cs[b]);
            if p.i == q.i && p.j == q.j && p.lo < q.hi - 0.01 && q.lo < p.hi - 0.01 {
                let msg = match (module(&p.id), module(&q.id)) {
                    (Some(x), Some(y)) if x != y => format!("modulene {x} og {y} overlapper i ({}, {}, {})", p.i, p.y, p.j),
                    _ => format!("{} og {} overlapper", p.id, q.id),
                };
                out.push(Issue { severity: Severity::Error, rule: "overlapp", at: Some(mm3(cell_center(p.i, p.y, p.j))), msg });
            }
        }
    }
    out
}

/// Tett: hver side av hver celle er enten lukket av en mur (en halvkant som
/// ikke er åpen), eller åpen mot en celle med gulvet i samme høyde; og hvert
/// hjørne der en mur ender eller fjellet står på skrå, har muren i hjørnet.
/// Leser instansene i nivåfila, ikke valgene kompilatoren gjorde.
pub fn tight(doc: &LevelDoc, kit: &KitDef, file: &LevelFile) -> Vec<Issue> {
    let cs = cells(doc, kit);
    let opening: HashMap<&str, &str> = kit
        .pieces
        .iter()
        .filter(|p| p.kind == PieceKind::Edge)
        .flat_map(|p| std::iter::once(p.model.as_str()).chain(p.variants.iter().map(|v| v.model.as_str())).map(move |m| (m, p.opening.as_deref().unwrap_or(""))))
        .collect();
    let corners: Vec<&str> = kit.pieces.iter().filter(|p| p.kind == PieceKind::Corner).map(|p| p.model.as_str()).collect();
    let key = |p: [f32; 3], yaw: f32| {
        ((p[0] * 100.0).round() as i64, (p[1] * 100.0).round() as i64, (p[2] * 100.0).round() as i64, ((yaw.rem_euclid(std::f32::consts::TAU) / std::f32::consts::FRAC_PI_2).round() as i64) % 4)
    };
    let mut halves: HashMap<(i64, i64, i64, i64), &str> = HashMap::new();
    let mut fills: HashMap<(i64, i64, i64, i64), bool> = HashMap::new();
    for inst in &file.instances {
        if let Some(o) = opening.get(inst.model.as_str()) {
            halves.insert(key(inst.pos, inst.yaw), o);
        } else if corners.contains(&inst.model.as_str()) {
            fills.insert(key(inst.pos, inst.yaw), true);
        }
    }
    let mut cols: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (k, c) in cs.iter().enumerate() {
        cols.entry((c.i, c.j)).or_default().push(k);
    }
    let neighbor = |a: usize, w: usize| -> bool {
        let c = &cs[a];
        let (di, dj) = outward(w);
        cols.get(&(c.i + di, c.j + dj)).is_some_and(|v| v.iter().any(|&b| (cs[b].floors[opposite(w)] - c.floors[w]).abs() < 0.01))
    };
    let mut out = vec![];
    let solid = |o: Option<&&str>| o.is_some_and(|o| *o != "arch");
    for (a, c) in cs.iter().enumerate() {
        let center = cell_center(c.i, c.y, c.j);
        for w in 0..4 {
            let o = outward_v(w) * HALF;
            let at = [center.x + o.x, c.floors[w], center.z + o.y];
            let half = halves.get(&key(at, side_yaw(w)));
            let closed = half.is_some_and(|h| *h == "wall");
            if !closed && !neighbor(a, w) {
                let what = half.map(|h| format!("halvkanten «{h}»")).unwrap_or_else(|| "ingen mur".into());
                out.push(Issue { severity: Severity::Error, rule: "tett", at: Some(at), msg: format!("{} er åpen mot {} ({what}) uten noe på andre siden", c.id, SIDES[w]) });
            }
        }
        for k in 0..4 {
            let v = corner_vertex(c.i, c.j, k);
            let p = vertex_pos(v);
            // En mur som ender i hjørnet (i cella selv eller en nabo), eller
            // fjell på skrå.
            let mut need = false;
            for ((i, j), corner) in around_vertex(v) {
                let list: Vec<usize> = cols.get(&(i, j)).map(|l| l.iter().copied().filter(|&b| cs[b].lo < c.hi - 0.01 && c.lo < cs[b].hi - 0.01).collect()).unwrap_or_default();
                if list.is_empty() {
                    need = true;
                }
                for b in list {
                    let cb = &cs[b];
                    let cc = cell_center(cb.i, cb.y, cb.j);
                    for w in corner_sides(corner) {
                        let o = outward_v(w) * HALF;
                        let at = [cc.x + o.x, cb.floors[w], cc.z + o.y];
                        if solid(halves.get(&key(at, side_yaw(w)))) {
                            need = true;
                        }
                    }
                }
            }
            if need && !fills.contains_key(&key([p.x, c.y, p.y], corner_yaw(k))) {
                out.push(Issue { severity: Severity::Error, rule: "tett", at: Some([p.x, c.y, p.y]), msg: format!("{}: muren ender i lufta i hjørnet {k}", c.id) });
            }
        }
    }
    out
}

/// Brudd fra kompileringen som gjelder kantene (to sider som ikke passer,
/// åpning mot fjell, trapp som ender i vegg).
pub fn edges(issues: &[Issue]) -> Vec<&Issue> {
    issues.iter().filter(|i| i.rule == "kant").collect()
}

/// Cella ved et punkt i nivåets rom (til «Gå til»), også i modulene.
pub fn cell_at(doc: &LevelDoc, kit: &KitDef, p: Vec3) -> Option<String> {
    let (i, j) = ((p.x / CELL).round() as i32, (p.z / CELL).round() as i32);
    super::modules::flatten(doc, kit).0.cells.into_iter().find(|c| c.at[0] == i && c.at[2] == j && (p.y - c.at[1] as f32).abs() < 3.0).map(|c| c.id)
}
