//! Helten: liv, tokens, nøkler, lageret i byen, det du bærer, komponentene
//! du har funnet, utstyret, bøkene du har lest og den varige styrken.
//! Reglene for handel, smiing, lesing, hvile og død. Utholdenheten bor i
//! kampen (`Combat`), så det som gir eller tar den, får kampen med.

use crate::domain::combat::Combat;
use crate::domain::craft::{self, Gear};
use crate::domain::note::Note;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Oljen boka fyller på i lykta: en firedel av en full lykt (to minutter).
pub const BOOK_OIL: f32 = crate::domain::lantern::FULL / 4.0;
/// Hva en natt i senga di koster (eieren 18. oktober), i tokens.
pub const SLEEP_PRICE: u32 = 10;

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
    /// Alt utstyret du har (våpen, skjold og rustninger etter id), også det
    /// du ikke har på deg; byttes i inventaret (`equip`).
    pub owned: BTreeSet<String>,
    /// Komponentene du har lest om i boka denne turen (liv tilbake én gang per).
    pub read_docs: HashSet<String>,
    /// Den varige styrken du har tatt (relikviene i krypten).
    pub bonus_taken: BTreeSet<String>,
    /// Lykta i beltet: oljen som brenner nede i krypten.
    pub lamp: crate::domain::lantern::Lantern,
    /// Lageret, utstyret eller tokenene har endret seg (sendes på nytt).
    pub dirty: bool,
}

/// Tokenene en ny spiller har (eieren 19. oktober: ett, så det første
/// kruset mjød til trubaduren er innen rekkevidde).
pub const START_COINS: u32 = 1;

impl Default for Hero {
    fn default() -> Self {
        Self {
            hp: 100.0,
            max_hp: 100.0,
            coins: START_COINS,
            keys: 0,
            inventory: BTreeMap::new(),
            carried: BTreeMap::new(),
            found: BTreeSet::new(),
            gear: Gear { weapon: craft::RUSTY_SWORD, shield: craft::OLD_SHIELD, armor: None, lantern: 1.0 },
            owned: [craft::RUSTY_SWORD.id, craft::OLD_SHIELD.id].into_iter().map(String::from).collect(),
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
        let made = match r.result {
            craft::Result::Weapon(id) | craft::Result::Shield(id) | craft::Result::Armor(id) => self.owned.contains(id),
            craft::Result::Lantern(k) => self.gear.lantern >= k,
        };
        if made {
            return (false, vec![Note::Sfx("deny"), Note::toast(format!("Du har allerede {}.", r.name))]);
        }
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
        if let craft::Result::Weapon(id) | craft::Result::Shield(id) | craft::Result::Armor(id) = r.result {
            self.owned.insert(id.to_string());
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

    /// Ta på noe du har (fra inventaret). Gir om det gikk, og det som skal
    /// sies.
    pub fn equip(&mut self, id: &str, combat: &mut Combat) -> (bool, Vec<Note>) {
        if !self.owned.contains(id) {
            return (false, vec![Note::Sfx("deny")]);
        }
        let (name, sound) = if let Some(w) = craft::weapon(id) {
            self.gear.weapon = w;
            (w.name, "equip_metal")
        } else if let Some(s) = craft::shield(id) {
            self.gear.shield = s;
            (s.name, if s.sound == "block_metal" { "equip_metal" } else { "equip_wood" })
        } else if let Some(a) = craft::armor(id) {
            self.gear.armor = Some(a);
            (a.name, "equip_leather")
        } else {
            return (false, vec![]);
        };
        combat.set_gear(self.gear);
        self.dirty = true;
        (true, vec![Note::Sfx(sound), Note::toast(format!("Du har på deg {name}."))])
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

    /// Bytte i Skriptoriet (`loot::SWAPS`, plan 14): `n` av en komponent
    /// fra lageret og avgiften i tokens mot én av en dyrere. Avslås uten nok
    /// av begge, og ingenting endres da.
    pub fn swap(&mut self, id: &str) -> (bool, Vec<Note>) {
        let Some(s) = crate::domain::loot::swap(id) else { return (false, vec![]) };
        let have = self.inventory.get(s.give).copied().unwrap_or(0);
        if have < s.n {
            return (false, vec![Note::Sfx("deny"), Note::toast(format!("Du har {have} {} i lageret, og byttet krever {}.", craft::title(s.give), s.n))]);
        }
        if self.coins < s.fee {
            return (false, vec![Note::Sfx("deny"), Note::toast(format!("Du mangler {} tokens til avgiften.", s.fee - self.coins))]);
        }
        self.coins -= s.fee;
        if let Some(v) = self.inventory.get_mut(s.give) {
            *v -= s.n;
        }
        *self.inventory.entry(s.get.to_string()).or_insert(0) += 1;
        self.discover();
        self.dirty = true;
        (true, vec![Note::Sfx("coins"), Note::toast(format!("Byttet: {} × {} mot én {}.", s.n, craft::title(s.give), craft::title(s.get)))])
    }

    /// Boka (eieren 18. oktober: «det skal alltid lønne seg veldig å lese
    /// boka»): å lese dokumentasjonen for en komponent gir fullt liv, full
    /// utholdenhet og `BOOK_OIL` olje i lykta, én gang per komponent per tur
    /// (til du dør). Toasten sier hva den ga.
    pub fn read_doc(&mut self, component: &str, combat: &mut Combat) -> (bool, Vec<Note>) {
        if !self.read_docs.insert(component.to_string()) {
            return (false, vec![]);
        }
        let (hp, stamina, oil) = (self.hp, combat.stamina, self.lamp.oil);
        self.restore(combat);
        self.lamp.top_up(BOOK_OIL);
        self.dirty = true;
        let mut gave = vec![];
        if self.hp > hp {
            gave.push(format!("fullt liv (+{:.0})", self.hp - hp));
        }
        if combat.stamina > stamina {
            gave.push("full utholdenhet".to_string());
        }
        if self.lamp.oil > oil {
            gave.push(format!("{:.0} minutter olje i lykta", ((self.lamp.oil - oil) / 60.0).max(1.0)));
        }
        let mut notes = vec![Note::Sfx("book")];
        let title = craft::title(component);
        if gave.is_empty() {
            notes.push(Note::toast(format!("Du leste om {title}. Du var alt uthvilt, med full lykt.")));
        } else {
            notes.push(Note::Sfx("heal"));
            let last = gave.pop().unwrap_or_default();
            let list = if gave.is_empty() { last } else { format!("{} og {last}", gave.join(", ")) };
            notes.push(Note::toast(format!("Du leste om {title}: {list}.")));
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
    fn utstyret_du_har_kan_byttes() {
        let mut h = Hero::default();
        let mut c = Combat::new(None);
        assert!(!h.equip("card_axe", &mut c).0, "øksa har du ikke");
        h.owned.insert("card_axe".into());
        assert!(h.equip("card_axe", &mut c).0);
        assert_eq!((h.gear.weapon.id, c.gear.weapon.id), ("card_axe", "card_axe"));
        assert!(h.equip("rusty_sword", &mut c).0, "det gamle sverdet er fortsatt ditt");
        assert_eq!(h.gear.weapon.id, "rusty_sword");
    }

    #[test]
    fn boka_gir_fullt_liv_utholdenhet_og_olje_en_gang() {
        let mut h = Hero { hp: 20.0, ..Default::default() };
        h.lamp.oil = 30.0;
        let mut c = Combat::new(None);
        c.stamina = 5.0;
        let (ok, notes) = h.read_doc("button", &mut c);
        assert!(ok);
        assert_eq!((h.hp, c.stamina), (h.max_hp, c.max_stamina), "fullt liv og full utholdenhet");
        assert_eq!(h.lamp.oil, 30.0 + BOOK_OIL, "olje i lykta");
        let toast = notes.iter().find_map(|n| if let Note::Toast(t) = n { Some(t.clone()) } else { None }).expect("toasten");
        assert!(toast.contains("fullt liv (+80)") && toast.contains("full utholdenhet") && toast.contains("2 minutter olje"), "{toast}");
        h.hp = 20.0;
        c.stamina = 5.0;
        assert!(!h.read_doc("button", &mut c).0, "én gang per komponent per tur");
        assert_eq!((h.hp, c.stamina), (20.0, 5.0));
        assert!(h.read_doc("card", &mut c).0, "en annen komponent gir igjen");
        h.die();
        h.hp = 20.0;
        assert!(h.read_doc("button", &mut c).0, "etter døden kan den leses på nytt");
    }
}
