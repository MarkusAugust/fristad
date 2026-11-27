//! Det som dras, vises der det er nå uten at byen bygges om for hvert steg
//! musa går: mens et drag varer, flyttes bare dataene, og `preview` sier
//! hvor instansene til det flyttede står. Byen bygges og reglene sjekkes
//! når draget slippes (`flush`). Å bygge alt for hvert steg tok flere
//! hundre millisekunder med krypten under byen.

use super::editor::{Change, Editor, Thing};
use crate::domain::world::layout::{Item, Ref};
use glam::Affine3A;

/// Endringen flytter bare en ting i byen eller et hus (alt annet er likt).
fn only_moves(c: &Change) -> bool {
    let Change::Set { r: Ref::Town(_) | Ref::Inside { .. }, before: Thing::Item(a), after: Thing::Item(b) } = c else {
        return false;
    };
    a.at.is_some() && b.at.is_some() && *b == Item { at: b.at, ..a.clone() }
}

impl Editor {
    /// Kan endringene vises uten å bygge byen: et drag som bare flytter ting.
    pub(super) fn previewable(&self, changes: &[Change]) -> bool {
        self.dragging() && changes.iter().all(only_moves)
    }

    /// Husk hvor tingene sto i byen slik den sist ble bygget, før
    /// endringene gjøres.
    pub(super) fn note_moved(&mut self, changes: &[Change]) {
        for c in changes {
            let Change::Set { r, .. } = c else { continue };
            if !self.moved.iter().any(|(m, _)| m == r) {
                if let Some(was) = self.pose_of(r) {
                    self.moved.push((r.clone(), was));
                }
            }
        }
    }

    /// Bygg byen og sjekk reglene om noe er dratt siden sist.
    pub fn flush(&mut self) {
        if !self.moved.is_empty() {
            self.refresh();
        }
    }

    /// Hvor huset med innholdet sto da byen sist ble bygget, om det er dratt.
    fn house_was(&self, house: &str) -> Option<Affine3A> {
        let i = self.layout.items.iter().position(|it| it.building && it.model.as_deref() == Some(house))?;
        self.moved.iter().find(|(r, _)| *r == Ref::Town(i)).map(|(_, was)| *was)
    }

    /// Instansene (modell, nummer) i `scene()` som er dratt siden byen sist
    /// ble bygget, med flyttingen som setter dem der de står nå. Det som
    /// står i et hus, følger huset.
    pub fn preview(&self) -> Vec<(&'static str, usize, Affine3A)> {
        if self.moved.is_empty() {
            return vec![];
        }
        let mut out = vec![];
        for (model, n, r) in &self.scene().origins {
            let was = match (self.moved.iter().find(|(m, _)| m == r), r) {
                (Some((_, was)), _) => Some(*was),
                // Huset er dratt: tingen sto der i husets rom.
                (None, Ref::Inside { house, .. }) => self.house_was(house).and_then(|h| Some(h * self.house_pose(house).inverse() * self.pose_of(r)?)),
                _ => None,
            };
            if let (Some(was), Some(now)) = (was, self.pose_of(r)) {
                out.push((*model, *n, now * was.inverse()));
            }
        }
        out
    }
}
