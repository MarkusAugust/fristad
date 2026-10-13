//! Lageret, butikkene, verkstedet, boka og lagringen: kommandoene fra
//! nettlaget og det de viser. Reglene står i `domain::hero` og
//! `domain::craft`.

use super::session::Session;
use crate::domain::craft::{self, Gear};
use crate::domain::note::Note;
use crate::ports::driving::{Event, InventoryView, RecipeView, SaveData, WareView};

impl Session {
    /// Det reglene sa, til neste bilde.
    fn later(&mut self, notes: Vec<Note>) {
        let mut events = vec![];
        self.say(notes, &mut events);
        self.pending.extend(events);
    }

    pub fn inventory_view(&self) -> InventoryView {
        let h = &self.hero;
        let list = |m: &std::collections::BTreeMap<String, u32>| m.iter().filter(|(_, n)| **n > 0).map(|(k, v)| (k.clone(), *v)).collect();
        InventoryView {
            stash: list(&h.inventory),
            carried: list(&h.carried),
            weapon: format!("{} ({})", h.gear.weapon.name, h.gear.weapon.rarity.name()),
            weapon_special: h.gear.weapon.special.into(),
            shield: format!("{} ({})", h.gear.shield.name, h.gear.shield.rarity.name()),
            armor: h.gear.armor.map(|a| format!("{} ({})", a.name, a.rarity.name())).unwrap_or_else(|| "ingen".into()),
            lantern: h.gear.lantern > 1.0,
            max_hp: h.max_hp.round() as u32,
            max_stamina: self.combat.max_stamina.round() as u32,
            tokens: h.coins,
            read: h.read_docs.iter().cloned().collect(),
        }
    }

    /// Butikken eller verkstedet til innbygger `i`.
    pub(super) fn npc_panel(&self, i: usize) -> Option<Event> {
        let name = self.npcs.npcs.get(i).map(|n| n.name).unwrap_or("");
        let wares = |w: &[craft::Ware]| w.iter().map(|w| WareView { id: w.id.into(), name: w.name.into(), price: w.price }).collect();
        match name {
            "Buttonsmeden" => Some(Event::Shop { npc: name.into(), wares: wares(craft::SMITH_WARES) }),
            "Apotekeren" => Some(Event::Shop { npc: name.into(), wares: wares(craft::APOTHECARY_WARES) }),
            "Gorvek" => Some(Event::Shop { npc: name.into(), wares: wares(craft::GORVEK_WARES) }),
            "Skriptoriet" => Some(Event::Workshop { recipes: self.recipe_views() }),
            _ => None,
        }
    }

    fn recipe_views(&self) -> Vec<RecipeView> {
        let have = |c: &str| self.hero.inventory.get(c).copied().unwrap_or(0);
        craft::RECIPES
            .iter()
            .map(|r| {
                let result = match r.result {
                    craft::Result::Weapon(id) => craft::weapon(id).map(|w| format!("{} (skade ×{:.2}, fart ×{:.2}, rekkevidde {:.1} m, {})", w.name, w.damage, w.speed, w.reach, w.special)).unwrap_or_default(),
                    craft::Result::Shield(id) => craft::shield(id).map(|s| format!("{} (slipper {:.0} % gjennom, paradevindu {:.2} s)", s.name, s.block * 100.0, s.parry_window)).unwrap_or_default(),
                    craft::Result::Armor(id) => craft::armor(id).map(|a| format!("{} (tar {:.0} % skade)", a.name, a.taken * 100.0)).unwrap_or_default(),
                    craft::Result::Lantern(k) => format!("Søkelykt (bærelyset ×{k:.1})"),
                };
                RecipeView {
                    id: r.id.into(),
                    name: r.name.into(),
                    rarity: r.rarity.name().into(),
                    result,
                    needs: r.needs.iter().map(|(c, n)| (c.to_string(), *n, have(c))).collect(),
                    requires: r.requires.iter().map(|c| (c.to_string(), have(c) > 0)).collect(),
                    sources: r
                        .needs
                        .iter()
                        .map(|(c, _)| *c)
                        .chain(r.requires.iter().copied())
                        .map(|c| {
                            let s = craft::source(c);
                            (c.to_string(), s.describe(), s.crypt_only())
                        })
                        .collect(),
                    ok: craft::missing(r, &self.hero.inventory).is_empty(),
                }
            })
            .collect()
    }

    /// Smi etter en oppskrift (fra skjemaet i skriptoriet). Gir om det gikk.
    pub fn craft(&mut self, id: &str) -> bool {
        let (ok, notes) = self.hero.craft(id, &mut self.combat);
        self.later(notes);
        ok
    }

    /// Kjøp i en butikk, for tokens. Gir om det gikk.
    pub fn buy(&mut self, id: &str) -> bool {
        let (ok, notes) = self.hero.buy(id, &mut self.combat);
        self.later(notes);
        ok
    }

    /// Boka: å lese dokumentasjonen for en komponent gir liv tilbake, én
    /// gang per komponent per tur.
    pub fn read_doc(&mut self, component: &str) -> bool {
        let (ok, notes) = self.hero.read_doc(component);
        self.later(notes);
        ok
    }

    pub fn save(&self) -> SaveData {
        let h = &self.hero;
        SaveData {
            stash: h.inventory.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            weapon: h.gear.weapon.id.into(),
            shield: h.gear.shield.id.into(),
            armor: h.gear.armor.map(|a| a.id.to_string()),
            lantern: h.gear.lantern,
            max_hp: h.max_hp,
            max_stamina: self.combat.max_stamina,
            coins: h.coins,
            found: h.found.iter().cloned().collect(),
            bonuses: h.bonus_taken.iter().cloned().collect(),
            oil: Some(h.lamp.oil),
        }
    }

    pub fn load(&mut self, d: SaveData) {
        let h = &mut self.hero;
        h.inventory = d.stash.into_iter().collect();
        h.found = d.found.into_iter().collect();
        h.bonus_taken = d.bonuses.into_iter().collect();
        h.discover();
        if let Some(w) = craft::weapon(&d.weapon) {
            h.gear.weapon = w;
        }
        if let Some(s) = craft::shield(&d.shield) {
            h.gear.shield = s;
        }
        h.gear.armor = d.armor.as_deref().and_then(craft::armor);
        h.gear.lantern = if d.lantern > 0.0 { d.lantern } else { 1.0 };
        h.lamp.oil = d.oil.unwrap_or(crate::domain::lantern::FULL).clamp(0.0, crate::domain::lantern::FULL);
        if d.max_hp >= 100.0 {
            h.max_hp = d.max_hp;
            h.hp = h.max_hp;
        }
        if d.max_stamina >= 100.0 {
            self.combat.max_stamina = d.max_stamina;
            self.combat.stamina = d.max_stamina;
        }
        h.coins = d.coins;
        h.dirty = true;
        self.combat.set_gear(h.gear);
    }

    /// Legg komponenter i lageret (testene og kommandoer).
    pub fn give(&mut self, component: &str, n: u32) {
        self.hero.give(component, n);
    }

    /// Komponentene du har funnet (galleriveggen).
    pub fn found(&self) -> Vec<String> {
        self.hero.found.iter().cloned().collect()
    }

    pub fn gear(&self) -> Gear {
        self.hero.gear
    }
}
