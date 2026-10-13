//! Helten: liv, tokens, nøkler, lageret i byen, det du bærer, komponentene
//! du har funnet, utstyret, bøkene du har lest og den varige styrken.
//! Reglene for handel, smiing, lesing, hvile og død. Utholdenheten bor i
//! kampen (`Combat`), så det som gir eller tar den, får kampen med.

use crate::domain::combat::Combat;
use crate::domain::craft::{self, Gear};
use crate::domain::note::Note;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Clone, Debug)]
pub struct Hero {
    pub hp: f32,
    pub max_hp: f32,
    pub coins: u32,
    pub keys: u32,
    /// Komponentene: trygt lagret i byen, og det du bærer på turen.
    pub inventory: BTreeMap<String, u32>,
    pub carried: BTreeMap<String, u32>,
    /// Hver komponent du noen gang har fått i byen (galleriveggen i
    /// huset ditt). Det du selger eller bruker opp, blir hengende.
    pub found: BTreeSet<String>,
    pub gear: Gear,
    /// Komponentene du har lest om i boka denne turen (liv tilbake én gang per).
    pub read_docs: HashSet<String>,
    /// Den varige styrken du har tatt (relikviene i krypten).
    pub bonus_taken: BTreeSet<String>,
    /// Lykta i beltet: oljen som brenner nede i krypten.
    pub lamp: crate::domain::lantern::Lantern,
    /// Lageret, utstyret eller tokenene har endret seg (sendes på nytt).
    pub dirty: bool,
}

impl Default for Hero {
    fn default() -> Self {
        Self {
            hp: 100.0,
            max_hp: 100.0,
            coins: 0,
            keys: 0,
            inventory: BTreeMap::new(),
            carried: BTreeMap::new(),
            found: BTreeSet::new(),
            gear: Gear { weapon: craft::RUSTY_SWORD, shield: craft::OLD_SHIELD, armor: None, lantern: 1.0 },
            read_docs: HashSet::new(),
            bonus_taken: BTreeSet::new(),
            lamp: Default::default(),
            dirty: true,
        }
    }
}

impl Hero {
    /// Noe du bærer med deg (fra en kiste, en fiende, sekken du mistet).
    pub fn carry(&mut self, component: &str, n: u32) {
        *self.carried.entry(component.to_string()).or_insert(0) += n;
        self.dirty = true;
    }

    /// Noe i lageret i byen.
    pub fn give(&mut self, component: &str, n: u32) {
        *self.inventory.entry(component.to_string()).or_insert(0) += n;
        self.discover();
        self.dirty = true;
    }

    /// Det du bar, legges i lageret. Gir hvor mange.
    pub fn bank(&mut self) -> Option<Note> {
        if self.carried.is_empty() {
            return None;
        }
        let n: u32 = self.carried.values().sum();
        for (k, v) in std::mem::take(&mut self.carried) {
            *self.inventory.entry(k).or_insert(0) += v;
        }
        self.discover();
        self.dirty = true;
        Some(Note::toast(format!("{n} komponenter er trygt i lageret.")))
    }

    /// Det som ligger i lageret er funnet, for alltid.
    pub fn discover(&mut self) {
        for (c, n) in &self.inventory {
            if *n > 0 && !self.found.contains(c) {
                self.found.insert(c.clone());
            }
        }
    }

    /// Fullt liv og full utholdenhet.
    pub fn restore(&mut self, combat: &mut Combat) {
        self.hp = self.max_hp;
        combat.stamina = combat.max_stamina;
    }

    /// Liv tilbake, aldri over det største.
    pub fn heal(&mut self, hp: f32) {
        self.hp = (self.hp + hp).min(self.max_hp);
    }

    /// Død: en firedel av tokenene og nøklene blir igjen, livet er fullt
    /// igjen, og bøkene kan leses på nytt. Gir tokenene som ble igjen og
    /// det du bar.
    pub fn die(&mut self) -> (u32, BTreeMap<String, u32>) {
        let lost = self.coins / 4;
        self.coins -= lost;
        self.keys = 0;
        self.hp = self.max_hp;
        self.read_docs.clear();
        let bag = std::mem::take(&mut self.carried);
        if !bag.is_empty() {
            self.dirty = true;
        }
        (lost, bag)
    }

    /// Varig styrke fra en relikvie (én gang per nøkkel).
    pub fn take_bonus(&mut self, key: &str, life: bool, combat: &mut Combat) -> Option<Note> {
        if !self.bonus_taken.insert(key.to_string()) {
            return None;
        }
        self.dirty = true;
        if life {
            self.max_hp += 10.0;
            self.heal(10.0);
            Some(Note::toast("Varig styrke: +10 liv."))
        } else {
            combat.max_stamina += 10.0;
            combat.stamina = combat.max_stamina;
            Some(Note::toast("Varig styrke: +10 utholdenhet."))
        }
    }

    /// Smi etter en oppskrift. Gir om det gikk, og det som skal sies.
    pub fn craft(&mut self, id: &str, combat: &mut Combat) -> (bool, Vec<Note>) {
        let Some(r) = craft::recipe(id) else { return (false, vec![]) };
        let missing = craft::missing(&r, &self.inventory);
        if !missing.is_empty() {
            let list: Vec<String> = missing.iter().map(|(c, n)| format!("{c} ×{n}")).collect();
            return (false, vec![Note::Sfx("deny"), Note::toast(format!("Skjemaet avvist: mangler {}.", list.join(", ")))]);
        }
        for (c, n) in r.needs {
            if let Some(v) = self.inventory.get_mut(*c) {
                *v = v.saturating_sub(*n);
            }
        }
        let (what, sound) = match r.result {
            craft::Result::Weapon(id) => {
                if let Some(w) = craft::weapon(id) {
                    self.gear.weapon = w;
                }
                ("våpenet", "equip_metal")
            }
            craft::Result::Shield(id) => {
                if let Some(s) = craft::shield(id) {
                    self.gear.shield = s;
                }
                ("skjoldet", "equip_wood")
            }
            craft::Result::Armor(id) => {
                self.gear.armor = craft::armor(id);
                ("rustningen", "equip_leather")
            }
            craft::Result::Lantern(k) => {
                self.gear.lantern = k;
                ("lykta", "equip_metal")
            }
        };
        combat.set_gear(self.gear);
        self.dirty = true;
        (true, vec![Note::Sfx("craft"), Note::Sfx(sound), Note::toast(format!("Skjemaet er godkjent. {} er smidd: {what} er byttet.", r.name))])
    }

    /// Kjøp i en butikk, for tokens. Gir om det gikk, og det som skal sies.
    pub fn buy(&mut self, id: &str, combat: &mut Combat) -> (bool, Vec<Note>) {
        let Some(w) = craft::ware(id) else { return (false, vec![]) };
        if self.coins < w.price {
            return (false, vec![Note::Sfx("deny"), Note::toast(format!("Du mangler {} tokens.", w.price - self.coins))]);
        }
        self.coins -= w.price;
        match w.what {
            craft::WareKind::Component(c) => {
                *self.inventory.entry(c.to_string()).or_insert(0) += 1;
                self.discover();
            }
            craft::WareKind::Heal(h) => self.heal(h),
            craft::WareKind::MaxHp(h) => {
                self.max_hp += h;
                self.heal(h);
            }
            craft::WareKind::MaxStamina(st) => {
                combat.max_stamina += st;
                combat.stamina = combat.max_stamina;
            }
            craft::WareKind::Oil => self.lamp.fill(),
        }
        self.dirty = true;
        (true, vec![Note::Sfx("coins"), Note::toast(format!("Kjøpt: {}.", w.name))])
    }

    /// Boka: å lese dokumentasjonen for en komponent gir liv tilbake, én
    /// gang per komponent per tur.
    pub fn read_doc(&mut self, component: &str) -> (bool, Vec<Note>) {
        if !self.read_docs.insert(component.to_string()) {
            return (false, vec![]);
        }
        let before = self.hp;
        self.heal(20.0);
        self.dirty = true;
        let mut notes = vec![Note::Sfx("book")];
        if self.hp > before {
            notes.push(Note::Sfx("heal"));
            notes.push(Note::toast(format!("Du leste om {component}: +{:.0} liv.", self.hp - before)));
        }
        (true, notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn det_du_bar_havner_i_lageret_og_blir_funnet() {
        let mut h = Hero::default();
        h.carry("button", 2);
        assert!(h.bank().is_some());
        assert_eq!(h.inventory.get("button"), Some(&2));
        assert!(h.found.contains("button") && h.carried.is_empty());
        assert!(h.bank().is_none(), "ingenting å legge i lageret");
    }

    #[test]
    fn doeden_tar_en_firedel_og_det_du_bar() {
        let mut h = Hero { coins: 40, keys: 2, hp: 3.0, ..Default::default() };
        h.carry("tabs", 1);
        let (lost, bag) = h.die();
        assert_eq!((lost, h.coins, h.keys, h.hp), (10, 30, 0, 100.0));
        assert_eq!(bag.get("tabs"), Some(&1));
    }

    #[test]
    fn boka_gir_liv_en_gang() {
        let mut h = Hero { hp: 50.0, ..Default::default() };
        assert!(h.read_doc("button").0);
        assert_eq!(h.hp, 70.0);
        assert!(!h.read_doc("button").0);
        assert_eq!(h.hp, 70.0);
    }
}
