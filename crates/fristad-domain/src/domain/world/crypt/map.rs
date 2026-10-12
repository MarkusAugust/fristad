//! Kartet over krypten: et rom avdekkes når du går inn i det (fog of war),
//! og merkene i rommene du har sett (trappene, portene, kistene, spakene,
//! snarveiene, hvilestedene og bossen) følger med.

use super::run::Run;
use super::Crypt;
use crate::domain::world::mechanisms::Mechanisms;
use glam::Vec3;

/// Kartet over ett nivå slik du kjenner det.
#[derive(Clone, Debug, PartialEq)]
pub struct Known {
    pub level: u32,
    pub name: String,
    /// Rommene du har sett, som polygoner (x, z).
    pub rooms: Vec<Vec<[f32; 2]>>,
    /// Merkene: slaget og stedet (x, z).
    pub marks: Vec<(&'static str, f32, f32)>,
}

/// Det kartet har å si dette bildet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapUpdate {
    /// Nytt kart (`Some(None)` når du er oppe igjen).
    pub map: Option<Option<Known>>,
    /// Hvor du står (x, z, retning), når det har endret seg.
    pub at: Option<(f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct Map {
    /// Rommene du har sett i hvert nivå.
    seen: Vec<Vec<bool>>,
    /// Nivået kartet sist ble sendt for (`None` oppe).
    level: Option<usize>,
    /// Noe på kartet har endret seg (en port, en kiste, en spak).
    pub dirty: bool,
    last_at: (f32, f32, f32),
}

impl Map {
    pub fn new(crypt: &Crypt) -> Self {
        Self {
            seen: crypt
                .levels
                .iter()
                .map(|l| vec![false; l.rooms.len()])
                .collect(),
            level: None,
            dirty: false,
            last_at: (0.0, 0.0, 0.0),
        }
    }

    /// Føttene står i `feet` (i nivå `level`, eller oppe).
    pub fn update(
        &mut self,
        crypt: &Crypt,
        mech: &Mechanisms,
        run: &Run,
        level: Option<usize>,
        feet: Vec3,
        yaw: f32,
    ) -> MapUpdate {
        let mut out = MapUpdate::default();
        let Some(k) = level else {
            if self.level.take().is_some() {
                out.map = Some(None);
            }
            return out;
        };
        let mut fresh = false;
        if let Some((l, r)) = crypt.room_at(feet + Vec3::Y * 0.3) {
            if let Some(seen) = self.seen.get_mut(l).and_then(|s| s.get_mut(r)) {
                fresh = !*seen;
                *seen = true;
            }
        }
        if fresh || self.level != Some(k) || self.dirty {
            self.level = Some(k);
            self.dirty = false;
            out.map = Some(Some(self.known(crypt, mech, run, k)));
        }
        let w = (feet.x, feet.z, yaw);
        let last = self.last_at;
        if (w.0 - last.0).abs() + (w.1 - last.1).abs() > 0.02 || (w.2 - last.2).abs() > 0.005 {
            self.last_at = w;
            out.at = Some(w);
        }
        out
    }

    /// Kartet over nivå `k` slik du kjenner det.
    pub fn known(&self, crypt: &Crypt, mech: &Mechanisms, run: &Run, k: usize) -> Known {
        let level = &crypt.levels[k];
        let seen = &self.seen[k];
        let rooms = level
            .rooms
            .iter()
            .zip(seen)
            .filter(|(_, s)| **s)
            .map(|(r, _)| r.poly.iter().map(|p| [p.x, p.y]).collect())
            .collect();
        let known = |p: Vec3| {
            crypt
                .room_at(p + Vec3::Y * 0.3)
                .is_some_and(|(l, r)| l == k && seen.get(r).copied().unwrap_or(false))
        };
        let mut marks = vec![];
        let mut mark = |kind: &'static str, p: Vec3| {
            if known(p) {
                marks.push((kind, p.x, p.z));
            }
        };
        if let Some((p, _)) = level.down {
            mark("exit", p);
        }
        if let Some((p, _)) = level.up {
            mark("ascend", p);
        }
        for (g, gate) in crypt.gates.iter().enumerate().filter(|(_, g)| g.level == k) {
            mark(
                if mech.gate_open(g) {
                    "gate_open"
                } else {
                    "gate"
                },
                gate.pos,
            );
        }
        for (c, chest) in crypt
            .chests
            .iter()
            .enumerate()
            .filter(|(_, c)| c.level == k)
        {
            if !mech.chests.get(c).copied().unwrap_or(true) {
                mark("chest", chest.pos);
            }
        }
        for s in crypt.shortcuts.iter().filter(|s| s.level == k) {
            mark("shortcut", s.pos);
        }
        for (i, l) in crypt
            .levers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.level == k)
        {
            mark(
                if mech.lever_pulled(i) {
                    "lever_pulled"
                } else {
                    "lever"
                },
                l.pos,
            );
        }
        for (i, r) in crypt.rests.iter().enumerate().filter(|(_, r)| r.level == k) {
            mark(if run.rest_lit(i) { "rest_lit" } else { "rest" }, r.pos);
        }
        if !run.slain.contains(&k) {
            for b in crypt.spawns.iter().filter(|s| s.level == k && s.boss) {
                mark("boss", b.pos);
            }
        }
        Known {
            level: level.n,
            name: level.name.clone(),
            rooms,
            marks,
        }
    }
}
