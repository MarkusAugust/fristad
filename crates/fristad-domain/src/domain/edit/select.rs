//! Å velge i dybden og i flokk: det som lå under pekeren ved siste klikk
//! (lista i panelet, Tab og Shift+Tab, det bakerste med Alt+klikk), alt
//! innenfor en ramme på skjermen, og å sette ned igjen det som sto på noe
//! når et drag slippes.

use super::editor::Editor;
use crate::domain::world::layout::Ref;
use glam::{Affine3A, Vec3};

/// Så langt over det den står på en ting kan være og likevel regnes som
/// stående (samme grense som at noe svever, 8 cm).
const STANDING: f32 = 0.08;

impl Editor {
    /// Velg nummer `k` i lista over det som lå under pekeren (nærmest først).
    /// Med `add` legges den til i valget eller tas bort.
    pub fn select_under(&mut self, k: usize, add: bool) -> Option<Ref> {
        let r = self.under.get(k)?.clone();
        if add {
            match self.selection.iter().position(|s| *s == r) {
                Some(i) => {
                    self.selection.remove(i);
                }
                None => self.selection.push(r.clone()),
            }
        } else {
            self.selection = vec![r.clone()];
        }
        Some(r)
    }

    /// Shift+Tab: det forrige under pekeren (det nærmere), rundt og rundt.
    pub fn cycle_back(&mut self) -> Option<Ref> {
        let n = self.under.len();
        if n == 0 {
            return None;
        }
        let k = match self.selection.as_slice() {
            [cur] => self.under.iter().position(|r| r == cur).map(|k| (k + n - 1) % n).unwrap_or(n - 1),
            _ => n - 1,
        };
        self.select_under(k, false)
    }

    /// Alt+klikk: det bakerste strålen treffer (det som står inni eller bak
    /// alt det andre), og lista blir det som lå under pekeren.
    pub fn pick_deepest(&mut self, origin: Vec3, dir: Vec3) -> Option<Ref> {
        let hits = self.hits(origin, dir);
        self.last_ray = Some((origin, dir));
        self.under = hits;
        let k = self.under.len().checked_sub(1)?;
        self.select_under(k, false)
    }

    /// Velg alt med midten innenfor rammen på skjermen. `screen` gir hvor et
    /// punkt i verden står på skjermen (0..1), eller `None` bak kameraet;
    /// `near` er hvor langt unna noe kan være (meter fra øyet). Med `add`
    /// legges det til valget.
    pub fn select_box(&mut self, eye: Vec3, near: f32, a: (f32, f32), b: (f32, f32), add: bool, screen: impl Fn(Vec3) -> Option<(f32, f32)>) -> usize {
        let (x0, x1) = (a.0.min(b.0), a.0.max(b.0));
        let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
        let mut found = vec![];
        for r in self.refs() {
            let Some((t, c, _)) = self.box_of(&r) else { continue };
            let p = t.transform_point3(c);
            if p.distance(eye) > near {
                continue;
            }
            if let Some((x, y)) = screen(p) {
                if (x0..=x1).contains(&x) && (y0..=y1).contains(&y) {
                    found.push(r);
                }
            }
        }
        if !add {
            self.selection.clear();
        }
        for r in &found {
            if !self.selection.contains(r) {
                self.selection.push(r.clone());
            }
        }
        found.len()
    }

    /// Det valgte som står på noe (bakken, et gulv, et bord) nå: det settes
    /// ned igjen når et drag langs bakken slippes (`settle`), så det følger
    /// skråningen. Det som henger eller svever med vilje, blir der det er.
    pub fn standing(&self) -> Vec<Ref> {
        self.selection
            .iter()
            .filter(|r| {
                let (Some(foot), Some(t)) = (self.foot(r), self.pose_of(r)) else {
                    return false;
                };
                let o = Vec3::from(t.translation);
                (foot - self.surface_under(o.x, o.z, foot + 0.3, Some(r))).abs() <= STANDING
            })
            .cloned()
            .collect()
    }

    /// Sett tingene ned på det som er under dem (i samme angregruppe som
    /// draget, om det er en).
    pub fn settle(&mut self, refs: &[Ref]) {
        let mut changes = vec![];
        for r in refs {
            let (Some(foot), Some(t)) = (self.foot(r), self.pose_of(r)) else {
                continue;
            };
            let o = Vec3::from(t.translation);
            let delta = Vec3::Y * (self.surface_under(o.x, o.z, foot + 0.3, Some(r)) - foot);
            if delta.y.abs() > 1e-4 {
                changes.extend(self.pose_change(r, |t| Affine3A::from_translation(delta) * t));
            }
        }
        self.commit(changes);
    }
}
