//! Rutenettet og standardmålene (`docs/nivaaredigering.md`, del 3). De
//! samme tallene står i `tools/crypt/kit_common.py`.

use glam::{Affine3A, Quat, Vec2, Vec3};
use std::f32::consts::FRAC_PI_2;

/// Cella: 4 × 4 m, midten i (4i, y, 4j).
pub const CELL: f32 = 4.0;
pub const HALF: f32 = CELL / 2.0;
/// Halvkanten: halve muren på 0,8 m.
pub const WALL_T: f32 = 0.4;
/// Murflaten, fra cellemidten.
pub const FACE: f32 = HALF - WALL_T;
/// Navigasjonsnettet holder seg så langt fra murene og det som står på gulvet.
pub const NAV_INSET: f32 = 0.4;
/// Rutene nettet rastres i (går opp i murflaten, åpningene og søylene).
pub const SUB: f32 = 0.2;
/// Åpningene (felles for alle temaer).
pub const GANG_W: f32 = 2.0 * FACE;
pub const DOOR_W: f32 = 1.6;
pub const GATE_W: f32 = 2.8;
pub const SECRET_W: f32 = 2.0;
pub const SECRET_H: f32 = 2.6;

/// Tønnehvelvet i gangene og trappene (`kit_common.py`): vederlaget
/// `GANG_S` og pilhøyden `GANG_R` over halve cella.
pub const GANG_S: f32 = 1.1;
pub const GANG_R: f32 = 2.5;

/// Høyden på gangprofilen `u` meter fra gangens midtlinje (`kit_common.gang_h`).
pub fn gang_h(u: f32) -> f32 {
    GANG_S + GANG_R * (1.0 - (u / HALF).powi(2)).max(0.0).sqrt()
}

/// Fakkelen: flammen står så høyt over festet og så langt ut fra muren
/// (`kit_cells.FLAME`), toppen av flammen så høyt over midten, og den
/// holder så stor avstand til hvelvet.
pub const TORCH_FLAME_Y: f32 = 0.5;
pub const TORCH_TIP: f32 = 0.3;
pub const TORCH_CLEAR: f32 = 0.15;

/// Sidene: N (−z), Ø (+x), S (+z), V (−x).
pub const SIDES: [&str; 4] = ["N", "E", "S", "W"];

pub fn side_index(s: &str) -> Option<usize> {
    match s {
        "N" | "n" => Some(0),
        "E" | "e" | "Ø" | "ø" | "O" => Some(1),
        "S" | "s" => Some(2),
        "W" | "w" | "V" | "v" => Some(3),
        _ => None,
    }
}

pub fn opposite(w: usize) -> usize {
    (w + 2) % 4
}

fn mirror_side(s: usize) -> usize {
    match s {
        1 => 3,
        3 => 1,
        x => x,
    }
}

/// Siden i nivået for en side i bitens rom.
pub fn local_to_world(l: usize, rot: u8, mirror: bool) -> usize {
    let l = if mirror { mirror_side(l) } else { l };
    (l + rot as usize) % 4
}

/// Siden i bitens rom for en side i nivået.
pub fn world_to_local(w: usize, rot: u8, mirror: bool) -> usize {
    let l = (w + 4 - (rot as usize % 4)) % 4;
    if mirror {
        mirror_side(l)
    } else {
        l
    }
}

/// Retningen ut gjennom siden, i rutene (i, j).
pub fn outward(w: usize) -> (i32, i32) {
    [(0, -1), (1, 0), (0, 1), (-1, 0)][w % 4]
}

pub fn outward_v(w: usize) -> Vec2 {
    let (x, z) = outward(w);
    Vec2::new(x as f32, z as f32)
}

/// Dreiningen til en bit med `rot` kvart omdreininger med klokka.
pub fn cell_yaw(rot: u8) -> f32 {
    -(rot as f32 % 4.0) * FRAC_PI_2
}

/// Dreiningen til halvkanten på siden `w` (modellens +z inn i cella).
pub fn side_yaw(w: usize) -> f32 {
    -(w as f32) * FRAC_PI_2
}

/// Hjørnene: NV (0), NØ (1), SØ (2), SV (3). Hjørnet ligger mellom siden
/// `c` og siden `(c + 3) % 4`.
pub fn corner_yaw(c: usize) -> f32 {
    -(c as f32) * FRAC_PI_2
}

pub fn corner_sides(c: usize) -> [usize; 2] {
    [c, (c + 3) % 4]
}

/// Hjørnet (vi, vj) i rutenettet av hjørner, for cella (i, j): NV er (i, j).
pub fn corner_vertex(i: i32, j: i32, c: usize) -> (i32, i32) {
    match c % 4 {
        0 => (i, j),
        1 => (i + 1, j),
        2 => (i + 1, j + 1),
        _ => (i, j + 1),
    }
}

/// Hjørnet i nivåets rom.
pub fn vertex_pos(v: (i32, i32)) -> Vec2 {
    Vec2::new(v.0 as f32 * CELL - HALF, v.1 as f32 * CELL - HALF)
}

/// Cellene rundt hjørnet (vi, vj) og hvilket hjørne det er for hver:
/// NV-cella (hjørnet er dens SØ) osv.
pub fn around_vertex(v: (i32, i32)) -> [((i32, i32), usize); 4] {
    let (vi, vj) = v;
    [((vi - 1, vj - 1), 2), ((vi, vj - 1), 3), ((vi, vj), 0), ((vi - 1, vj), 1)]
}

/// Midten av cella i nivåets rom (gulvet i y).
pub fn cell_center(i: i32, y: f32, j: i32) -> Vec3 {
    Vec3::new(i as f32 * CELL, y, j as f32 * CELL)
}

/// Formen til en bit i nivået: flyttet, dreid og (først) speilet langs x.
/// Kvarte omdreininger blir eksakte (cos og sin 0 eller ±1), så kantene to
/// biter deler, får nøyaktig de samme hjørnene og ingen sprekk på en piksel.
pub fn transform(pos: Vec3, yaw: f32, mirror: bool) -> Affine3A {
    let turns = yaw / FRAC_PI_2;
    let m = if (turns - turns.round()).abs() < 1e-4 {
        let (s, c) = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][(turns.round() as i32).rem_euclid(4) as usize];
        Affine3A::from_mat3_translation(glam::Mat3::from_cols(Vec3::new(c, 0.0, -s), Vec3::Y, Vec3::new(s, 0.0, c)), pos)
    } else {
        Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), pos)
    };
    if mirror {
        m * Affine3A::from_scale(Vec3::new(-1.0, 1.0, 1.0))
    } else {
        m
    }
}

/// Avrundet til mm, så resultatet er likt i wasm og maskinkode.
pub fn mm(v: f32) -> f32 {
    let r = (v * 1000.0).round() / 1000.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

pub fn mm3(v: Vec3) -> [f32; 3] {
    [mm(v.x), mm(v.y), mm(v.z)]
}
