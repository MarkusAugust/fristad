//! Rutene til innbyggerne i redigeringen (`NpcDef::route`): punktene på
//! bakken som innbyggeren går mellom, rundt og rundt, og hvor lenge hen står
//! ved hvert (0 er rett forbi). Punktene flyttes, legges til på streken
//! mellom to punkter og slettes; ventetiden står i egenskapspanelet
//! (`route.N.wait`). Alt er vanlige endringer av innbyggeren, så angre
//! gjelder dem også.

use super::editor::{Change, Editor, Thing};
use crate::domain::world::layout::{NpcDef, Ref};
use glam::Vec3;

/// Ventetiden et nytt punkt får (sekunder).
pub const NEW_WAIT: f32 = 2.0;

impl Editor {
    /// Ruta til innbyggeren `r` (tom for alt annet).
    pub fn route(&self, r: &Ref) -> Vec<(Vec3, f32)> {
        match self.get(r) {
            Some(Thing::Npc(n)) => n.route,
            _ => vec![],
        }
    }

    /// Endre ruta til innbyggeren `r` med `f`.
    fn edit_route(&mut self, r: &Ref, f: impl FnOnce(&mut Vec<(Vec3, f32)>) -> bool) -> bool {
        let Some(Thing::Npc(before)) = self.get(r) else { return false };
        let mut after: NpcDef = before.clone();
        if !f(&mut after.route) || after == before {
            return false;
        }
        self.commit(vec![Change::Set { r: r.clone(), before: Thing::Npc(before), after: Thing::Npc(after) }]);
        true
    }

    /// Flytt punkt `k` til `at`.
    pub fn route_move(&mut self, r: &Ref, k: usize, at: Vec3) -> bool {
        self.edit_route(r, |route| match route.get_mut(k) {
            Some(p) => {
                p.0 = at;
                true
            }
            None => false,
        })
    }

    /// Legg til et punkt `at` som nummer `k` (foran det som var `k`). Gir
    /// nummeret.
    pub fn route_insert(&mut self, r: &Ref, k: usize, at: Vec3) -> Option<usize> {
        let mut out = None;
        self.edit_route(r, |route| {
            let k = k.min(route.len());
            route.insert(k, (at, NEW_WAIT));
            out = Some(k);
            true
        });
        out
    }

    /// Slett punkt `k`.
    pub fn route_remove(&mut self, r: &Ref, k: usize) -> bool {
        self.edit_route(r, |route| {
            if k < route.len() {
                route.remove(k);
                true
            } else {
                false
            }
        })
    }

    /// Hvor lenge innbyggeren står ved punkt `k` (sekunder, 0 er rett forbi).
    pub fn route_wait(&mut self, r: &Ref, k: usize, wait: f32) -> bool {
        self.edit_route(r, |route| match route.get_mut(k) {
            Some(p) => {
                p.1 = wait.max(0.0);
                true
            }
            None => false,
        })
    }

    /// Stykket av ruta (fra punkt `k` til det neste, rundt) nærmest punktet
    /// `p` på bakken, og hvor nær (meter i x og z).
    pub fn route_segment(&self, r: &Ref, p: Vec3) -> Option<(usize, f32)> {
        let route = self.route(r);
        let n = route.len();
        if n < 2 {
            return None;
        }
        let flat = |v: Vec3| glam::Vec2::new(v.x, v.z);
        (0..n)
            .map(|k| {
                let (a, b) = (flat(route[k].0), flat(route[(k + 1) % n].0));
                let d = b - a;
                let t = ((flat(p) - a).dot(d) / d.length_squared().max(1e-6)).clamp(0.0, 1.0);
                (k, (a + d * t).distance(flat(p)))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}
