//! Utseendet: tallene fra frøet og plassen som velger variantene, faklene i
//! festene, og de sammensatte bitene delt i delcellene sine
//! (`docs/nivaaredigering.md`, del 4.2 og 9).

use super::grid::*;
use super::{CellOut, Grid};
use crate::domain::world::crypt::doc::{DocCell, LevelDoc};
use crate::domain::world::crypt::kit::{KitDef, KitPiece, PieceKind};
use crate::domain::world::crypt::level::{InstanceDef, LightDef};
use glam::Vec3;

/// Et tall i [0, 1) fra frøet og plassen (`splitmix64`): det samme i wasm og
/// maskinkode, og uavhengig av rekkefølgen cellene står i.
pub fn roll(seed: u64, parts: [i64; 4]) -> f32 {
    let mix = |mut z: u64| {
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let mut h = mix(seed);
    for p in parts {
        h = mix(h ^ p as u64);
    }
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// Faklene i festene på en halvkant: rommets fakkeltetthet avgjør hvor mange
/// av festene som får en fakkel (med frøet), og hver fakkel gir et lys ved
/// flammen.
pub(super) fn torches(g: &Grid, out: &mut CellOut, piece: &KitPiece, pos: Vec3, yaw: f32, room: &str, key: [i64; 4]) {
    let density = g.torches.get(room).copied().unwrap_or(0.0);
    if density <= 0.0 {
        return;
    }
    let Some(torch) = g.kit.of_kind(&piece.theme, PieceKind::Prop) else { return };
    let t = transform(pos, yaw, false);
    for (n, m) in piece.mounts.iter().filter(|m| m.kind == "fakkel").enumerate() {
        if roll(g.seed, [key[0], key[1], key[2], key[3] * 16 + n as i64]) >= density {
            continue;
        }
        let at = t.transform_point3(Vec3::from(m.pos));
        let turn = yaw + m.yaw;
        out.instances.push(InstanceDef { model: torch.model.clone(), pos: mm3(at), yaw: turn, mirror: false });
        let flame = torch.mounts.iter().find(|m| m.kind == "lys").map(|m| Vec3::from(m.pos)).unwrap_or(Vec3::new(0.0, 0.5, 0.28));
        let light = transform(at, turn, false).transform_point3(flame);
        out.lights.push(LightDef { pos: mm3(light), color: [1.0, 0.6, 0.28], range: 7.5, flicker: true, intensity: None, room: Some(room.to_string()) });
    }
}

/// Cellene i dokumentet med de sammensatte bitene delt i delbitene sine
/// (`KitDef::add`): midtcella beholder id-en (innhold festet til den følger
/// biten), de andre får `<id>#<k>`.
pub fn expand(doc: &LevelDoc, kit: &KitDef) -> Vec<DocCell> {
    let mut out = vec![];
    for c in &doc.cells {
        let Some(piece) = kit.piece(&c.bit).filter(|p| !p.footprint.is_empty()) else {
            out.push(c.clone());
            continue;
        };
        let t = transform(Vec3::ZERO, cell_yaw(c.rot % 4), c.mirror && piece.mirror);
        for (k, fc) in piece.footprint.iter().enumerate() {
            let d = t.transform_vector3(Vec3::new(fc.at[0] as f32, 0.0, fc.at[1] as f32));
            let center = fc.at == [0, 0];
            out.push(DocCell {
                id: if center { c.id.clone() } else { format!("{}#{k}", c.id) },
                bit: format!("{}#{k}", piece.id),
                at: [c.at[0] + d.x.round() as i32, c.at[1], c.at[2] + d.z.round() as i32],
                ..c.clone()
            });
        }
    }
    out
}
