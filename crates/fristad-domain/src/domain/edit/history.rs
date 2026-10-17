//! Angre-historikken som liste: hvert steg med et navn («Flyttet bench»,
//! «Endret lys»), og å gå rett til et steg (angre eller gjøre
//! om så mange som trengs). Alt eieren gjør er endringer, også egenskapene,
//! så alt står her.

use super::editor::{Change, Editor, Thing};
use crate::domain::world::layout::Ref;

impl Change {
    fn r(&self) -> &Ref {
        match self {
            Change::Set { r, .. } | Change::Insert { r, .. } | Change::Remove { r, .. } => r,
        }
    }
}

/// Det en ting heter i historikken.
fn name(t: &Thing) -> String {
    match t {
        Thing::Item(it) => it.model.clone().unwrap_or_else(|| "usynlig vegg".into()),
        Thing::Npc(n) => n.name.clone(),
        Thing::Mark(m) => match &m.label {
            Some(l) => format!("{} ({l})", m.kind),
            None => m.kind.clone(),
        },
        Thing::Light(_) | Thing::Lamp(_) => "lys".into(),
    }
}

/// Bare stedet eller dreiningen er endret (ikke noe annet).
fn only_moved(before: &Thing, after: &Thing) -> bool {
    match (before.clone(), after.clone()) {
        (Thing::Item(mut a), Thing::Item(b)) => {
            a.at = b.at;
            a == b
        }
        (Thing::Npc(mut a), Thing::Npc(b)) => {
            (a.pos, a.yaw, a.route) = (b.pos, b.yaw, b.route.clone());
            a == b
        }
        (Thing::Mark(mut a), Thing::Mark(b)) => {
            (a.pos, a.yaw, a.patrol) = (b.pos, b.yaw, b.patrol.clone());
            a == b
        }
        (Thing::Light(mut a), Thing::Light(b)) => {
            a.pos = b.pos;
            a == b
        }
        (Thing::Lamp(mut a), Thing::Lamp(b)) => {
            a.pos = b.pos;
            a == b
        }
        _ => false,
    }
}

/// Et navn på et steg: hva som ble gjort, og med hva (eller hvor mange).
pub(super) fn describe(changes: &[Change]) -> String {
    let (mut verb, mut names) = ("Endret", vec![]);
    for c in changes {
        let (v, n) = match c {
            Change::Insert { thing, .. } => ("Satte ut", name(thing)),
            Change::Remove { thing, .. } => ("Slettet", name(thing)),
            Change::Set { before, after, .. } => (if only_moved(before, after) { "Flyttet" } else { "Endret" }, name(after)),
        };
        if names.is_empty() {
            verb = v;
        } else if verb != v {
            verb = "Endret";
        }
        if !names.contains(&n) {
            names.push(n);
        }
    }
    // Samme ting flere ganger i ett drag er én ting.
    let things = changes.iter().map(|c| format!("{:?}", c.r())).collect::<std::collections::HashSet<_>>().len();
    match (names.as_slice(), things) {
        ([one], 1) => format!("{verb} {one}"),
        ([one], n) => format!("{verb} {n} × {one}"),
        (_, n) => format!("{verb} {n} ting"),
    }
}

impl Editor {
    /// Historikken: stegene som kan angres (eldst først), så de som kan
    /// gjøres om, og hvor mange som er gjort (stedet i lista du står).
    pub fn history(&self) -> (Vec<String>, usize) {
        let mut out: Vec<String> = self.undo.iter().map(|c| describe(c)).collect();
        out.extend(self.redo.iter().rev().map(|c| describe(c)));
        (out, self.undo.len())
    }

    /// Gå til steget `n` i historikken (0 er slik det var da redigeringen
    /// begynte): angre eller gjør om til så mange er gjort.
    pub fn undo_to(&mut self, n: usize) -> bool {
        let mut moved = false;
        while self.undo.len() > n && self.undo() {
            moved = true;
        }
        while self.undo.len() < n && self.redo() {
            moved = true;
        }
        moved
    }
}
