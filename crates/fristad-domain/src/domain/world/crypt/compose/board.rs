//! Prøvebrettet (`docs/nivaaredigering.md`, fase 1): hver cellebit mot hver
//! cellebit, med hver side av den ene mot hver side av den andre (begge
//! dreid på alle fire måter), par for par med fjell imellom, så hver
//! sammenføyning kan ses og testes for seg. Den andre biten står så
//! gulvene møtes ved kanten (trappene går opp eller ned til den).

use super::grid::*;
use crate::domain::world::crypt::doc::{DocCell, DocLight, DocRoom, LevelDoc, LevelId, Thing};
use crate::domain::world::crypt::kit::{KitDef, PieceKind};

/// Så mange par per rad.
const PER_ROW: usize = 30;

pub fn board(kit: &KitDef) -> LevelDoc {
    let cells: Vec<_> = kit.listed().filter(|p| p.kind == PieceKind::Cell && p.footprint.is_empty()).collect();
    let mut doc = LevelDoc { format: 1, level: LevelId::Name("brett".into()), name: "Prøvebrettet".into(), status: "utkast".into(), ..Default::default() };
    let mut n = 0usize;
    for a in &cells {
        for b in &cells {
            for combo in 0..16u8 {
                let (rot_a, rot) = (combo / 4, combo % 4);
                let (col, row) = (n % PER_ROW, n / PER_ROW);
                let (i, j) = (col as i32 * 3, row as i32 * 2);
                let room = format!("par{n}");
                // Den andre biten står så gulvet ved kanten er likt.
                let fa = a.floors[world_to_local(1, rot_a, false)];
                let fb = b.floors[world_to_local(3, rot, false)];
                let yb = (fa - fb).round() as i32;
                doc.rooms.push(DocRoom { id: room.clone(), name: format!("{} ({}°) mot {} ({}°)", a.id, rot_a as u32 * 90, b.id, rot as u32 * 90), theme: a.theme.clone(), ..Default::default() });
                doc.cells.push(DocCell { id: format!("a{n}"), bit: a.id.clone(), at: [i, 0, j], rot: rot_a, room: room.clone(), ..Default::default() });
                doc.cells.push(DocCell { id: format!("b{n}"), bit: b.id.clone(), at: [i + 1, yb, j], rot, room: room.clone(), ..Default::default() });
                // Trappa ned i nedgangen (festet i biten), så hullet har trinnene.
                for (cell, piece) in [(format!("a{n}"), a), (format!("b{n}"), b)] {
                    if piece.mounts.iter().any(|m| m.kind == "down") {
                        doc.things.push(Thing { id: format!("ned_{cell}"), kind: "down".into(), anchor: format!("celle:{cell}"), ..Default::default() });
                    }
                }
                doc.lights.push(DocLight {
                    id: format!("l{n}"),
                    anchor: "fri".into(),
                    pos: [i as f32 * CELL + HALF, 2.2, j as f32 * CELL],
                    color: [1.0, 0.75, 0.5],
                    range: 9.0,
                    flicker: false,
                    intensity: Some(2.5),
                });
                n += 1;
            }
        }
    }
    doc.things.push(Thing { id: "start".into(), kind: "start".into(), anchor: "celle:a0".into(), pos: Some([0.0, 0.0, 0.0]), ..Default::default() });
    doc
}
