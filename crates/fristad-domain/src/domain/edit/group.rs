//! Gruppene i redigeringen: flere ting i byen eller i samme hus samlet
//! under et navn (`Item::group`). Velger du én, velges hele gruppen, så den
//! flyttes, dreies (om midten av gruppen), kopieres (til en ny gruppe) og
//! slettes samlet. «Lag gruppe» og «Løs opp» er vanlige endringer av
//! dataene, så angre gjelder dem også.

use super::editor::{Change, Editor, Thing};
use crate::domain::world::layout::{Item, Ref};

impl Editor {
    /// Gruppen en ting hører til.
    pub fn group_of(&self, r: &Ref) -> Option<String> {
        match self.get(r)? {
            Thing::Item(Item { group, .. }) => group,
            _ => None,
        }
    }

    /// Alle i samme gruppe som `r` (i samme liste: byen eller samme hus),
    /// med `r` først; uten gruppe bare `r`.
    pub fn members(&self, r: &Ref) -> Vec<Ref> {
        let Some(g) = self.group_of(r) else { return vec![r.clone()] };
        let mut out = vec![r.clone()];
        let items: Vec<Ref> = match r {
            Ref::Town(_) => (0..self.layout.items.len()).map(Ref::Town).collect(),
            Ref::Inside { house, .. } => (0..self.contents.get(house).map(|v| v.len()).unwrap_or(0)).map(|i| Ref::Inside { house: house.clone(), i }).collect(),
            _ => vec![],
        };
        out.extend(items.into_iter().filter(|o| o != r && self.group_of(o).as_deref() == Some(g.as_str())));
        out
    }

    /// Ta med resten av gruppene til det som er valgt.
    pub(super) fn expand_groups(&mut self) {
        let mut out: Vec<Ref> = vec![];
        for r in &self.selection {
            for m in self.members(r) {
                if !out.contains(&m) {
                    out.push(m);
                }
            }
        }
        self.selection = out;
    }

    /// Legg til eller ta bort `r` med gruppen sin (Shift+klikk).
    pub(super) fn toggle(&mut self, r: &Ref) {
        let all = self.members(r);
        if self.selection.contains(r) {
            self.selection.retain(|s| !all.contains(s));
        } else {
            for m in all {
                if !self.selection.contains(&m) {
                    self.selection.push(m);
                }
            }
        }
    }

    /// Det ene som er valgt, om valget er én ting eller én gruppe.
    pub(super) fn single(&self) -> Option<&Ref> {
        let first = self.selection.first()?;
        (self.selection.len() == 1 || self.members(first).len() == self.selection.len()).then_some(first)
    }

    /// Gruppen til det valgte, om det er akkurat én gruppe.
    pub fn single_group(&self) -> Option<String> {
        self.single().and_then(|r| self.group_of(r)).filter(|_| self.selection.len() > 1)
    }

    /// Det valgte som skal flyttes: ikke lysene på en ting som selv er valgt
    /// (de følger tingen).
    pub(super) fn moving(&self) -> Vec<Ref> {
        self.selection.iter().filter(|r| !matches!(r, Ref::ItemLight { item, .. } if self.selection.contains(item))).cloned().collect()
    }

    /// Et gruppenavn som ikke er brukt, fra `name` («gjerde», «gjerde 2» …).
    pub(super) fn fresh_group(&self, name: &str, taken: &[String]) -> String {
        let base = name.trim_end_matches(|c: char| c.is_ascii_digit()).trim_end().to_string();
        let base = if base.is_empty() { name.to_string() } else { base };
        (2..).map(|n| format!("{base} {n}")).find(|g| !taken.contains(g) && !self.group_used(g)).expect("et ledig navn")
    }

    /// Navnet er brukt av en gruppe i byen eller i et hus.
    fn group_used(&self, g: &str) -> bool {
        self.layout.items.iter().chain(self.contents.values().flatten()).any(|it| it.group.as_deref() == Some(g))
    }

    /// Lag en gruppe av det valgte (tingene i byen og i husene) med navnet
    /// `name` (tomt: «gruppe N»). Gir navnet, eller feilen.
    pub fn make_group(&mut self, name: &str) -> Result<String, String> {
        let refs: Vec<Ref> = self.selection.iter().filter(|r| matches!(r, Ref::Town(_) | Ref::Inside { .. })).cloned().collect();
        if refs.len() < 2 {
            return Err("Velg minst to ting i byen eller i samme hus for å lage en gruppe.".into());
        }
        let lists: std::collections::HashSet<String> = refs.iter().map(Self::list_of).collect();
        if lists.len() > 1 {
            return Err("En gruppe står i byen eller i ett hus, ikke begge.".into());
        }
        let name = match name.trim() {
            "" => (1..).map(|n| format!("gruppe {n}")).find(|g| !self.group_used(g)).expect("et ledig navn"),
            n => n.to_string(),
        };
        let changes = refs.iter().filter_map(|r| self.set_group(r, Some(name.clone()))).collect();
        self.commit(changes);
        self.selection = refs;
        Ok(name)
    }

    /// Løs opp gruppene i det valgte (tingene blir stående og valgt).
    pub fn ungroup(&mut self) -> usize {
        let changes: Vec<Change> = self.selection.clone().iter().filter_map(|r| self.set_group(r, None)).collect();
        let n = changes.len();
        self.commit(changes);
        n
    }

    fn set_group(&self, r: &Ref, group: Option<String>) -> Option<Change> {
        let Some(Thing::Item(before)) = self.get(r) else { return None };
        let after = Item { group, ..before.clone() };
        (after != before).then(|| Change::Set { r: r.clone(), before: Thing::Item(before), after: Thing::Item(after) })
    }
}
