//! Utstyret og oppskriftene: komponentene du vinner i kjelleren (kister og
//! elitefiender) smis til våpen, skjold og rustning hos Skjemakontoret, der
//! oppskriften er et Fristil-skjema. Utstyret ser ut som ekte våpen
//! (modellene fra `tools/models/arms.py`), men er bygd av komponenter.
//! Rene data; `game.rs` eier lageret og `combat.rs` bruker tallene.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponKind {
    Sword,
    Axe,
    Dagger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "vanlig",
            Rarity::Rare => "sjelden",
            Rarity::Epic => "episk",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weapon {
    pub id: &'static str,
    pub name: &'static str,
    pub model: &'static str,
    pub kind: WeaponKind,
    /// Ganger på grunnskaden.
    pub damage: f32,
    /// Ganger på hastigheten på klippene.
    pub speed: f32,
    pub reach: f32,
    pub rarity: Rarity,
    /// Spesialtrekket, slik det vises.
    pub special: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shield {
    pub id: &'static str,
    pub name: &'static str,
    pub model: &'static str,
    /// Så mye av skaden slipper gjennom en blokk.
    pub block: f32,
    /// Sekunder etter at blokken startet der et bitt blir parade.
    pub parry_window: f32,
    pub rarity: Rarity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Armor {
    pub id: &'static str,
    pub name: &'static str,
    /// Ganger på skaden du tar.
    pub taken: f32,
    pub rarity: Rarity,
}

/// Det du har på deg.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gear {
    pub weapon: Weapon,
    pub shield: Shield,
    pub armor: Option<Armor>,
    /// Lykta: hvor sterkt lyset du bærer nede er.
    pub lantern: f32,
}

pub const RUSTY_SWORD: Weapon = Weapon { id: "rusty_sword", name: "Rustent sverd", model: "sword", kind: WeaponKind::Sword, damage: 0.8, speed: 1.0, reach: 2.2, rarity: Rarity::Common, special: "ingen" };
pub const OLD_SHIELD: Shield = Shield { id: "old_shield", name: "Gammelt rundskjold", model: "shield_round", block: 0.25, parry_window: 0.22, rarity: Rarity::Common };

pub const WEAPONS: &[Weapon] = &[
    RUSTY_SWORD,
    Weapon { id: "component_sword", name: "Komponentsverd", model: "sword", kind: WeaponKind::Sword, damage: 1.0, speed: 1.0, reach: 2.3, rarity: Rarity::Common, special: "kombo på tre" },
    Weapon { id: "card_axe", name: "Kortøks", model: "axe", kind: WeaponKind::Axe, damage: 1.45, speed: 0.8, reach: 2.0, rarity: Rarity::Rare, special: "går gjennom blokk" },
    Weapon { id: "tag_dagger", name: "Taggdolk", model: "dagger", kind: WeaponKind::Dagger, damage: 0.7, speed: 1.5, reach: 1.6, rarity: Rarity::Rare, special: "dobbelt bakfra" },
    Weapon { id: "divider_blade", name: "Dividerklinge", model: "sword", kind: WeaponKind::Sword, damage: 1.35, speed: 1.1, reach: 2.5, rarity: Rarity::Epic, special: "parade gir trippel" },
];

pub const SHIELDS: &[Shield] = &[
    OLD_SHIELD,
    Shield { id: "card_shield", name: "Kortskjold", model: "shield_round", block: 0.12, parry_window: 0.3, rarity: Rarity::Common },
    Shield { id: "fieldset_tower", name: "Fieldset-tårnskjold", model: "shield_tower", block: 0.05, parry_window: 0.36, rarity: Rarity::Epic },
];

pub const ARMORS: &[Armor] = &[
    Armor { id: "badge_jerkin", name: "Badgevams", taken: 0.85, rarity: Rarity::Common },
    Armor { id: "fieldset_mail", name: "Fieldset-brynje", taken: 0.65, rarity: Rarity::Epic },
];

/// Hva en oppskrift gir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Result {
    Weapon(&'static str),
    Shield(&'static str),
    Armor(&'static str),
    /// Lykta: sterkere bærelys.
    Lantern(f32),
}

/// En oppskrift: et skjema med felt (komponent, antall). Avanserte
/// oppskrifter krever at du har de avanserte skjemakomponentene i tillegg
/// (`requires`), som Skjemakontoret trenger for å kunne ta imot skjemaet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Recipe {
    pub id: &'static str,
    pub name: &'static str,
    pub needs: &'static [(&'static str, u32)],
    pub requires: &'static [&'static str],
    pub result: Result,
    pub rarity: Rarity,
}

pub const RECIPES: &[Recipe] = &[
    Recipe { id: "component_sword", name: "Komponentsverd", needs: &[("button", 2), ("divider", 1)], requires: &[], result: Result::Weapon("component_sword"), rarity: Rarity::Common },
    Recipe { id: "card_shield", name: "Kortskjold", needs: &[("card", 1), ("badge", 1)], requires: &[], result: Result::Shield("card_shield"), rarity: Rarity::Common },
    Recipe { id: "badge_jerkin", name: "Badgevams", needs: &[("badge", 3), ("tag", 2)], requires: &[], result: Result::Armor("badge_jerkin"), rarity: Rarity::Common },
    Recipe { id: "card_axe", name: "Kortøks", needs: &[("button", 1), ("card", 2), ("badge", 1)], requires: &["label"], result: Result::Weapon("card_axe"), rarity: Rarity::Rare },
    Recipe { id: "tag_dagger", name: "Taggdolk", needs: &[("tooltip", 1), ("tag", 2)], requires: &["label"], result: Result::Weapon("tag_dagger"), rarity: Rarity::Rare },
    Recipe { id: "search_lantern", name: "Søkelykt", needs: &[("search", 1), ("progress", 1)], requires: &["label", "help-text"], result: Result::Lantern(2.2), rarity: Rarity::Rare },
    Recipe { id: "divider_blade", name: "Dividerklinge", needs: &[("divider", 3), ("button", 2), ("alert", 1)], requires: &["fieldset", "file-upload"], result: Result::Weapon("divider_blade"), rarity: Rarity::Epic },
    Recipe { id: "fieldset_tower", name: "Fieldset-tårnskjold", needs: &[("fieldset", 1), ("card", 2), ("badge", 2)], requires: &["fieldset", "file-upload"], result: Result::Shield("fieldset_tower"), rarity: Rarity::Epic },
    Recipe { id: "fieldset_mail", name: "Fieldset-brynje", needs: &[("fieldset", 2), ("badge", 3), ("tag", 3)], requires: &["fieldset", "file-upload", "search"], result: Result::Armor("fieldset_mail"), rarity: Rarity::Epic },
];

/// Komponentene kistene i hver etasje kan gi, fra lette til tunge.
pub const LOOT: [&[&str]; 5] = [
    &["button", "divider", "badge", "tag", "tag"],
    &["card", "alert", "progress", "badge", "button"],
    &["label", "help-text", "tooltip", "card", "divider"],
    &["fieldset", "file-upload", "search", "label", "alert"],
    &["fieldset", "file-upload", "search", "dialog", "tabs"],
];

/// Det elitefienden alltid bærer, per etasje.
pub const ELITE_LOOT: [&str; 5] = ["divider", "card", "label", "fieldset", "file-upload"];

pub fn weapon(id: &str) -> Option<Weapon> {
    WEAPONS.iter().copied().find(|w| w.id == id)
}
pub fn shield(id: &str) -> Option<Shield> {
    SHIELDS.iter().copied().find(|s| s.id == id)
}
pub fn armor(id: &str) -> Option<Armor> {
    ARMORS.iter().copied().find(|a| a.id == id)
}
pub fn recipe(id: &str) -> Option<Recipe> {
    RECIPES.iter().copied().find(|r| r.id == id)
}

/// Om lageret dekker oppskriften: hva som mangler (tom liste er ja).
pub fn missing(r: &Recipe, inventory: &BTreeMap<String, u32>) -> Vec<(String, u32)> {
    let mut out = vec![];
    for (c, n) in r.needs {
        let have = inventory.get(*c).copied().unwrap_or(0);
        if have < *n {
            out.push((c.to_string(), n - have));
        }
    }
    for c in r.requires {
        if inventory.get(*c).copied().unwrap_or(0) == 0 && !out.iter().any(|(m, _)| m == c) {
            out.push((c.to_string(), 1));
        }
    }
    out
}

/// Butikkene: hva som selges hvor, for tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ware {
    pub id: &'static str,
    pub name: &'static str,
    pub price: u32,
    pub what: WareKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WareKind {
    /// En komponent til lageret.
    Component(&'static str),
    /// Liv tilbake.
    Heal(f32),
    /// Varig: mer liv, mer utholdenhet.
    MaxHp(f32),
    MaxStamina(f32),
}

pub const SMITH_WARES: &[Ware] = &[
    Ware { id: "buy_button", name: "Knapp", price: 12, what: WareKind::Component("button") },
    Ware { id: "buy_divider", name: "Divider", price: 14, what: WareKind::Component("divider") },
    Ware { id: "buy_badge", name: "Badge", price: 10, what: WareKind::Component("badge") },
    Ware { id: "buy_tag", name: "Tag", price: 8, what: WareKind::Component("tag") },
];
pub const APOTHECARY_WARES: &[Ware] = &[
    Ware { id: "potion", name: "Styrkedrikk (+40 liv)", price: 10, what: WareKind::Heal(40.0) },
    Ware { id: "big_potion", name: "Fullt liv", price: 22, what: WareKind::Heal(100.0) },
];
pub const GORVEK_WARES: &[Ware] = &[
    Ware { id: "train_hp", name: "Herding (+20 maks liv)", price: 30, what: WareKind::MaxHp(20.0) },
    Ware { id: "train_stamina", name: "Pust (+20 maks utholdenhet)", price: 30, what: WareKind::MaxStamina(20.0) },
];

pub fn ware(id: &str) -> Option<Ware> {
    SMITH_WARES.iter().chain(APOTHECARY_WARES).chain(GORVEK_WARES).copied().find(|w| w.id == id)
}
