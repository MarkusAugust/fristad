//! Det som står i kryptnivåene i redigeringen: merkene (kister, spaker,
//! porter, hvilesteder, fiender …) og lysene i `assets/layout/crypt_NN.json`.
//! Hvilken modell hvert merke har, boksen å klikke på, og regelen om at et
//! merke står på gulvet i et rom.

use super::editor::{Change, Editor, Thing};
use super::rules::Breach;
use crate::domain::world::crypt::level::{CryptContents, MarkDef};
use crate::domain::world::crypt::{self as cw, Crypt};
use crate::domain::world::layout::Ref;
use crate::domain::world::layout::Pose;
use glam::{Affine3A, Vec3};
use std::collections::BTreeMap;

/// Modellen et merke settes ut med i spillet (som i `crypt::build_level`).
pub fn model_of(m: &MarkDef) -> Option<&str> {
    Some(match m.kind.as_str() {
        "rest" => cw::KIT_REST,
        "lever" => cw::KIT_LEVER,
        "lore" => cw::KIT_LORE,
        "bonus" => cw::KIT_BONUS,
        "chest" | "rare" => cw::CHEST,
        "up" => cw::KIT_UP,
        "down" => cw::KIT_DOWN,
        "shortcut" => cw::KIT_SHORTCUT,
        "ladder" => cw::KIT_LADDER,
        "trap" => cw::KIT_PLATE,
        "gate" => cw::PORTCULLIS,
        "secret_wall" => cw::KIT_SECRET,
        "trapdoor" => cw::KIT_TRAPDOOR,
        "bridge" => cw::KIT_BRIDGE,
        "hint" => cw::KIT_CRACK,
        "prop" => return m.label.as_deref(),
        _ => return None,
    })
}

/// Merket en modell fra katalogen blir når den settes ut i krypten.
pub fn mark_for(model: &str) -> Option<MarkDef> {
    let kind = match model {
        cw::KIT_REST => "rest",
        cw::KIT_LEVER => "lever",
        cw::KIT_LORE => "lore",
        cw::KIT_BONUS => "bonus",
        cw::CHEST => "chest",
        cw::KIT_PLATE => "trap",
        cw::KIT_CRACK => "hint",
        m if cw::PROPS.contains(&m) => "prop",
        _ => return None,
    };
    Some(MarkDef { kind: kind.into(), label: (kind == "prop" || kind == "hint").then(|| if kind == "hint" { "crack".into() } else { model.to_string() }), ..Default::default() })
}

/// Boksen til et merke uten modell (midte og halve mål i merkets rom):
/// fiendene som en figur, de usynlige delene som en plate.
pub fn default_box(m: &MarkDef) -> (Vec3, Vec3) {
    match m.kind.as_str() {
        "mob" | "elite" | "boss" => (Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.35, 0.9, 0.35)),
        "gate" => (Vec3::new(0.0, cw::GATE_H * 0.5, 0.0), Vec3::new(m.width.unwrap_or(cw::GATE_W) * 0.5, cw::GATE_H * 0.5, 0.1)),
        "secret_wall" => (Vec3::new(0.0, 1.3, 0.0), Vec3::new(m.width.unwrap_or(2.0) * 0.5, 1.3, 0.15)),
        "trapdoor" => {
            let (w, d) = m.size.map(|s| s.wd()).unwrap_or((1.6, 1.6));
            (Vec3::new(0.0, 0.05, 0.0), Vec3::new(w * 0.5, 0.05, d * 0.5))
        }
        _ => (Vec3::new(0.0, 0.3, 0.0), Vec3::splat(0.3)),
    }
}

/// Merkene som står på gulvet (ikke stigens topp, lysene eller det som
/// henger på veggen).
fn on_floor(kind: &str) -> bool {
    matches!(kind, "rest" | "lever" | "lore" | "bonus" | "chest" | "rare" | "mob" | "elite" | "boss" | "trap" | "prop" | "gate")
}

/// Regelen for krypten: et merke på gulvet står inne i et rom i sitt nivå,
/// og med foten på gulvet der (innen 8 cm over og 30 cm under, som ellers).
pub fn breaches(c: &Crypt, crypt: &BTreeMap<u32, CryptContents>) -> Vec<Breach> {
    let mut out = vec![];
    for (n, contents) in crypt {
        let Some((k, level)) = c.levels.iter().enumerate().find(|(_, l)| l.n == *n) else {
            continue;
        };
        for (i, m) in contents.marks.iter().enumerate() {
            let Some(p) = m.pos else { continue };
            if !on_floor(&m.kind) {
                continue;
            }
            let w = Vec3::from(p) + level.origin;
            let r = Ref::Crypt { level: *n, i };
            match c.room_at(w).filter(|(lk, _)| *lk == k) {
                None => out.push(Breach { r, why: format!("{} står utenfor rommene i nivå {n}", m.kind) }),
                Some((_, ri)) => {
                    // Gulvet i rommet, eller nettet under (trapper og ramper),
                    // det som er nærmest: galleriet over salen har sitt eget
                    // gulv selv om nettet under finner salens.
                    let room = level.rooms[ri].y;
                    let floor = c.floor_under(w).filter(|f| (w.y - f).abs() < (w.y - room).abs()).unwrap_or(room);
                    let gap = w.y - floor;
                    if gap > 0.08 {
                        out.push(Breach { r, why: format!("{} svever {:.0} cm over gulvet", m.kind, gap * 100.0) });
                    } else if gap < -0.3 {
                        out.push(Breach { r, why: format!("{} står {:.0} cm nede i gulvet", m.kind, -gap * 100.0) });
                    }
                }
            }
        }
    }
    out
}

impl Editor {
    /// Origo i kryptnivået tingen står i.
    pub(super) fn crypt_origin(&self, r: &Ref) -> Option<Vec3> {
        let n = match r {
            Ref::Crypt { level, .. } | Ref::CryptLight { level, .. } => *level,
            _ => return None,
        };
        self.built.levels.iter().find(|l| l.n == n).map(|l| l.origin)
    }

    /// Krypten slik den står nå (nivåene, rommene og origo).
    pub fn built_crypt(&self) -> &Crypt {
        &self.built
    }

    /// Det som står i kryptnivåene, til lagring og til å bygge spillet på nytt.
    pub fn crypt_data(&self) -> &BTreeMap<u32, CryptContents> {
        &self.crypt
    }

    /// Et nytt merke i nivået under `pose` (bare det byggesettet har merker
    /// for); blir valget.
    pub(super) fn place_in_crypt(&mut self, model: &str, pose: Affine3A) -> Option<Ref> {
        let at = Vec3::from(pose.translation);
        let level = &self.built.levels[self.built.level_at(at)?];
        let mut m = mark_for(model)?;
        m.pos = Some((at - level.origin).to_array());
        m.yaw = Pose::of(pose).yaw();
        let r = Ref::Crypt { level: level.n, i: self.crypt.get(&level.n).map(|c| c.marks.len()).unwrap_or(0) };
        self.commit(vec![Change::Insert { r: r.clone(), thing: Thing::Mark(m) }]);
        self.selection = vec![r.clone()];
        Some(r)
    }
}
