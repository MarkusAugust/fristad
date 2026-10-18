//! Lysene i byen i redigeringen: lyktestolpenes egne lys (`Item::lights`)
//! og lysene modellene har med seg (vegglykta ved døra, ildstedene og
//! lysestakene i `<navn>_inside.json`). De velges, flyttes, kopieres,
//! slettes og får farge, rekkevidde, styrke og blafring som lysene i krypten.
//! Endres et av modellens lys, får tingen sin egen liste (`Item::lamps`);
//! blir den lik modellens igjen (angre), forsvinner den, så dataene er som før.

use super::editor::{Editor, Thing};
use crate::domain::world::layout::{Item, LightDef, Ref};
use crate::domain::world::town;
use glam::Vec3;

impl Editor {
    /// Tingen et lys hører til.
    pub(super) fn item_of(&self, r: &Ref) -> Option<Item> {
        match self.get(r)? {
            Thing::Item(it) => Some(it),
            _ => None,
        }
    }

    /// Lysene modellen til tingen har med seg (uten tingens endringer).
    pub(super) fn model_lamps(&self, it: &Item) -> Vec<LightDef> {
        it.model.as_deref().and_then(|m| self.src.insides.iter().find(|(n, _)| n == m)).map(|(_, inside)| town::model_lamps(inside)).unwrap_or_default()
    }

    /// Lysene tingen har med seg slik de står nå.
    fn lamps_of(&self, it: &Item) -> Vec<LightDef> {
        it.lamps.clone().unwrap_or_else(|| self.model_lamps(it))
    }

    /// Lyset `r` (et `Ref::ItemLight`).
    pub(super) fn get_lamp(&self, item: &Ref, model: bool, i: usize) -> Option<Thing> {
        let it = self.item_of(item)?;
        let l = if model { self.lamps_of(&it).get(i).cloned() } else { it.lights.get(i).cloned() };
        l.map(Thing::Lamp)
    }

    /// Sett, legg inn eller ta ut lyset `i` på tingen `item` (`None` tar ut,
    /// `insert` legger inn foran), og skriv tingen tilbake.
    pub(super) fn apply_lamp(&mut self, item: &Ref, model: bool, i: usize, l: Option<&LightDef>, insert: bool) {
        let Some(mut it) = self.item_of(item) else { return };
        let mut list = if model { self.lamps_of(&it) } else { it.lights.clone() };
        match (l, insert) {
            (Some(l), true) => list.insert(i.min(list.len()), l.clone()),
            (Some(l), false) => {
                if let Some(x) = list.get_mut(i) {
                    *x = l.clone();
                }
            }
            (None, _) => {
                if i < list.len() {
                    list.remove(i);
                }
            }
        }
        if model {
            it.lamps = (list != self.model_lamps(&it)).then_some(list);
        } else {
            it.lights = list;
        }
        match item {
            Ref::Town(k) => self.layout.items[*k] = it,
            Ref::Inside { house, i } => self.contents.get_mut(house).expect("huset")[*i] = it,
            _ => {}
        }
    }

    /// Alle lysene på tingene i byen og husene.
    pub(super) fn lamp_refs(&self) -> Vec<Ref> {
        let mut items: Vec<(Ref, &Item)> = self.layout.items.iter().enumerate().map(|(k, it)| (Ref::Town(k), it)).collect();
        let mut houses: Vec<&String> = self.contents.keys().collect();
        houses.sort();
        for h in houses {
            items.extend(self.contents[h].iter().enumerate().map(|(i, it)| (Ref::Inside { house: h.clone(), i }, it)));
        }
        let mut out = vec![];
        for (r, it) in items {
            let item = Box::new(r);
            out.extend((0..it.lights.len()).map(|i| Ref::ItemLight { item: item.clone(), model: false, i }));
            let n = it.lamps.as_ref().map(|v| v.len()).unwrap_or_else(|| self.model_lamps(it).len());
            out.extend((0..n).map(|i| Ref::ItemLight { item: item.clone(), model: true, i }));
        }
        out
    }

    /// Hvor et lys står i verden.
    pub(super) fn lamp_pos(&self, item: &Ref, l: &LightDef) -> Option<Vec3> {
        Some(self.pose_of(item)?.transform_point3(l.pos))
    }

    /// Lyset flyttet til `world` (i verden), i tingens rom.
    pub(super) fn lamp_moved(&self, item: &Ref, mut l: LightDef, world: Vec3) -> Option<LightDef> {
        l.pos = self.pose_of(item)?.inverse().transform_point3(world);
        Some(l)
    }

    /// Alle lysene (i byen, i husene og i krypten): hvor de står i verden og
    /// fargen, til markeringene i redigeringen.
    pub fn light_marks(&self) -> Vec<(Ref, Vec3, Vec3)> {
        let mut out = vec![];
        for r in self.lamp_refs() {
            if let (Some(Thing::Lamp(l)), Some(t)) = (self.get(&r), self.pose_of(&r)) {
                out.push((r, Vec3::from(t.translation), l.color));
            }
        }
        for (n, c) in &self.crypt {
            for (i, l) in c.lights.iter().enumerate() {
                let r = Ref::CryptLight { level: *n, i };
                if let Some(t) = self.pose_of(&r) {
                    out.push((r, Vec3::from(t.translation), Vec3::from(l.color)));
                }
            }
        }
        out
    }
}
