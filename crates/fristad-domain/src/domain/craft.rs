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
    Mace,
}

/// Det et våpen kan som de andre ikke kan (vises som `Weapon.special`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trait {
    None,
    /// Dobbel skade i ryggen på fienden.
    Backstab,
    /// Hugget feier en bredere bue og tar flere på en gang.
    Cleave,
    /// Knuser panseret (modalens front tar full skade) og bryter balansen
    /// lettere.
    Crush,
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
    /// Ganger på skaden i hugget rett etter en parade.
    pub riposte: f32,
    /// Ganger på utholdenheten et hugg koster (tunge våpen koster mer).
    pub stamina: f32,
    pub trait_: Trait,
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
    /// Lyden når det tar imot et bitt (tre eller jern).
    pub sound: &'static str,
    /// Ganger på utholdenheten en blokk koster (vekten).
    pub stamina: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Armor {
    pub id: &'static str,
    pub name: &'static str,
    /// Ganger på skaden du tar.
    pub taken: f32,
    pub rarity: Rarity,
    /// Lyden som rasler i takt med skrittene.
    pub sound: &'static str,
    /// Armene i førsteperson med denne rustningen (`tools/models/arms.py`).
    pub arms: &'static str,
    /// Ganger på hvor fort utholdenheten kommer tilbake (tung rustning er treg).
    pub regen: f32,
    /// Ganger på hva en unnamanøver koster.
    pub dodge: f32,
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

impl Gear {
    /// Hvor fort utholdenheten kommer tilbake i rustningen (ganger).
    pub fn regen(&self) -> f32 {
        self.armor.map(|a| a.regen).unwrap_or(1.0)
    }

    /// Hva en unnamanøver koster i rustningen (ganger).
    pub fn dodge(&self) -> f32 {
        self.armor.map(|a| a.dodge).unwrap_or(1.0)
    }

    /// Buen hugget treffer i (cos av halve vinkelen): øksene feier bredere.
    pub fn arc(&self, base: f32) -> f32 {
        if self.weapon.trait_ == Trait::Cleave {
            base - 0.35
        } else {
            base
        }
    }
}

pub const RUSTY_SWORD: Weapon = Weapon { id: "rusty_sword", name: "Rustent sverd", model: "sword", kind: WeaponKind::Sword, damage: 0.8, speed: 1.0, reach: 2.2, rarity: Rarity::Common, special: "ingen", riposte: 2.0, stamina: 1.0, trait_: Trait::None };
pub const OLD_SHIELD: Shield = Shield { id: "old_shield", name: "Gammelt rundskjold", model: "shield_round", block: 0.25, parry_window: 0.22, rarity: Rarity::Common, sound: "block", stamina: 1.0 };

/// Våpnene, ett slag med egen modell hver (eieren 18. oktober: «alt skal se
/// forskjellig ut med forskjellige stats»), fra det du starter med til det
/// som smis av det Rammeverket legger igjen.
pub const WEAPONS: &[Weapon] = &[
    RUSTY_SWORD,
    Weapon { id: "component_sword", name: "Komponentsverd", model: "sword_rare", kind: WeaponKind::Sword, damage: 1.0, speed: 1.05, reach: 2.3, rarity: Rarity::Common, special: "parade gir dobbelt", riposte: 2.0, stamina: 1.0, trait_: Trait::None },
    Weapon { id: "tag_dagger", name: "Taggdolk", model: "dagger", kind: WeaponKind::Dagger, damage: 0.7, speed: 1.5, reach: 1.6, rarity: Rarity::Common, special: "dobbelt i ryggen", riposte: 2.0, stamina: 0.6, trait_: Trait::Backstab },
    Weapon { id: "card_axe", name: "Kortøks", model: "axe", kind: WeaponKind::Axe, damage: 1.45, speed: 0.8, reach: 2.0, rarity: Rarity::Rare, special: "feier bredt", riposte: 2.0, stamina: 1.3, trait_: Trait::Cleave },
    Weapon { id: "alert_mace", name: "Alertkølle", model: "mace", kind: WeaponKind::Mace, damage: 1.25, speed: 0.85, reach: 1.9, rarity: Rarity::Rare, special: "knuser panser og balanse", riposte: 2.0, stamina: 1.4, trait_: Trait::Crush },
    Weapon { id: "tooltip_stiletto", name: "Tooltipstikker", model: "dagger_rare", kind: WeaponKind::Dagger, damage: 0.95, speed: 1.6, reach: 1.7, rarity: Rarity::Rare, special: "dobbelt i ryggen", riposte: 2.5, stamina: 0.6, trait_: Trait::Backstab },
    Weapon { id: "divider_blade", name: "Dividerklinge", model: "sword_legend", kind: WeaponKind::Sword, damage: 1.35, speed: 1.1, reach: 2.5, rarity: Rarity::Epic, special: "parade gir trippel", riposte: 3.0, stamina: 1.0, trait_: Trait::None },
    Weapon { id: "tabs_axe", name: "Tabsøks", model: "axe_rare", kind: WeaponKind::Axe, damage: 1.8, speed: 0.75, reach: 2.1, rarity: Rarity::Epic, special: "feier bredt", riposte: 2.0, stamina: 1.5, trait_: Trait::Cleave },
];

pub const SHIELDS: &[Shield] = &[
    OLD_SHIELD,
    Shield { id: "card_shield", name: "Kortskjold", model: "shield_round_blue", block: 0.15, parry_window: 0.3, rarity: Rarity::Common, sound: "block", stamina: 0.9 },
    Shield { id: "badge_heater", name: "Badgeskjold", model: "shield_heater", block: 0.1, parry_window: 0.26, rarity: Rarity::Rare, sound: "block", stamina: 1.0 },
    Shield { id: "fieldset_tower", name: "Fieldset-tårnskjold", model: "shield_tower", block: 0.03, parry_window: 0.18, rarity: Rarity::Epic, sound: "block_metal", stamina: 1.5 },
    Shield { id: "dialog_heater", name: "Dialogskjold", model: "shield_heater_violet", block: 0.08, parry_window: 0.4, rarity: Rarity::Epic, sound: "block_metal", stamina: 0.8 },
];

pub const ARMORS: &[Armor] = &[
    Armor { id: "badge_jerkin", name: "Badgevams", taken: 0.85, rarity: Rarity::Common, arms: "arms_leather", regen: 1.0, dodge: 1.0, sound: "gear_leather" },
    Armor { id: "tag_cuirass", name: "Taggkyrass", taken: 0.78, rarity: Rarity::Rare, arms: "arms_leather_dark", regen: 1.05, dodge: 0.85, sound: "gear_leather" },
    Armor { id: "fieldset_mail", name: "Fieldset-brynje", taken: 0.65, rarity: Rarity::Rare, arms: "arms_mail", regen: 0.85, dodge: 1.2, sound: "gear_mail" },
    Armor { id: "search_plate", name: "Searchharnisk", taken: 0.5, rarity: Rarity::Epic, arms: "arms_plate", regen: 0.7, dodge: 1.4, sound: "gear_mail" },
    Armor { id: "dialog_plate", name: "Dialogharnisk", taken: 0.42, rarity: Rarity::Epic, arms: "arms_plate_dark", regen: 0.8, dodge: 1.25, sound: "gear_mail" },
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

/// Oppskriftene, fra temaet øverst i krypten til det nederste: hver bygd av
/// det kistene og eliten i sitt tema gir (`LOOT`, `ELITE_LOOT`).
pub const RECIPES: &[Recipe] = &[
    // Gravkjelleren
    Recipe { id: "component_sword", name: "Komponentsverd", needs: &[("button", 2), ("divider", 1)], requires: &[], result: Result::Weapon("component_sword"), rarity: Rarity::Common },
    Recipe { id: "tag_dagger", name: "Taggdolk", needs: &[("tag", 3), ("button", 1)], requires: &[], result: Result::Weapon("tag_dagger"), rarity: Rarity::Common },
    Recipe { id: "badge_jerkin", name: "Badgevams", needs: &[("badge", 3), ("tag", 2)], requires: &[], result: Result::Armor("badge_jerkin"), rarity: Rarity::Common },
    // Kloakken
    Recipe { id: "card_shield", name: "Kortskjold", needs: &[("card", 1), ("badge", 1)], requires: &[], result: Result::Shield("card_shield"), rarity: Rarity::Common },
    Recipe { id: "card_axe", name: "Kortøks", needs: &[("button", 1), ("card", 2), ("badge", 1)], requires: &["alert"], result: Result::Weapon("card_axe"), rarity: Rarity::Rare },
    Recipe { id: "tag_cuirass", name: "Taggkyrass", needs: &[("tag", 3), ("badge", 2), ("progress", 1)], requires: &[], result: Result::Armor("tag_cuirass"), rarity: Rarity::Rare },
    // Gruva
    Recipe { id: "alert_mace", name: "Alertkølle", needs: &[("alert", 2), ("progress", 1), ("card", 1)], requires: &["label"], result: Result::Weapon("alert_mace"), rarity: Rarity::Rare },
    Recipe { id: "tooltip_stiletto", name: "Tooltipstikker", needs: &[("tooltip", 2), ("help-text", 1), ("tag", 1)], requires: &["label"], result: Result::Weapon("tooltip_stiletto"), rarity: Rarity::Rare },
    Recipe { id: "badge_heater", name: "Badgeskjold", needs: &[("badge", 2), ("label", 1), ("card", 1)], requires: &["help-text"], result: Result::Shield("badge_heater"), rarity: Rarity::Rare },
    Recipe { id: "search_lantern", name: "Søkelykt", needs: &[("search", 1), ("progress", 1)], requires: &["label", "help-text"], result: Result::Lantern(2.2), rarity: Rarity::Rare },
    // Borgen under jorda
    Recipe { id: "fieldset_mail", name: "Fieldset-brynje", needs: &[("fieldset", 2), ("badge", 3), ("tag", 3)], requires: &["file-upload"], result: Result::Armor("fieldset_mail"), rarity: Rarity::Rare },
    Recipe { id: "divider_blade", name: "Dividerklinge", needs: &[("divider", 3), ("fieldset", 1), ("alert", 1)], requires: &["file-upload", "search"], result: Result::Weapon("divider_blade"), rarity: Rarity::Epic },
    Recipe { id: "fieldset_tower", name: "Fieldset-tårnskjold", needs: &[("fieldset", 1), ("card", 2), ("label", 2)], requires: &["file-upload"], result: Result::Shield("fieldset_tower"), rarity: Rarity::Epic },
    Recipe { id: "search_plate", name: "Searchharnisk", needs: &[("search", 2), ("fieldset", 2), ("alert", 1)], requires: &["file-upload", "label"], result: Result::Armor("search_plate"), rarity: Rarity::Epic },
    // Den korrupte kjernen
    Recipe { id: "tabs_axe", name: "Tabsøks", needs: &[("tabs", 1), ("dialog", 1), ("card", 2), ("fieldset", 1)], requires: &[], result: Result::Weapon("tabs_axe"), rarity: Rarity::Epic },
    Recipe { id: "dialog_heater", name: "Dialogskjold", needs: &[("dialog", 1), ("search", 1), ("badge", 2)], requires: &[], result: Result::Shield("dialog_heater"), rarity: Rarity::Epic },
    Recipe { id: "dialog_plate", name: "Dialogharnisk", needs: &[("dialog", 1), ("tabs", 1), ("fieldset", 2), ("search", 1)], requires: &[], result: Result::Armor("dialog_plate"), rarity: Rarity::Epic },
];

/// Hvor på kroppen en ting sitter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Weapon,
    Shield,
    Armor,
    Lantern,
}

impl Slot {
    pub fn key(self) -> &'static str {
        match self {
            Slot::Weapon => "weapon",
            Slot::Shield => "shield",
            Slot::Armor => "armor",
            Slot::Lantern => "lantern",
        }
    }
}

/// Én verdi på et kort: navnet, tallet (til sammenligningen), teksten som
/// vises, og om høyere er bedre.
#[derive(Clone, Debug, PartialEq)]
pub struct Stat {
    pub label: &'static str,
    pub value: f32,
    pub text: String,
    pub higher_better: bool,
}

fn stat(label: &'static str, value: f32, text: String, higher_better: bool) -> Stat {
    Stat { label, value, text, higher_better }
}

/// En ting du kan ha på deg, slik kortene viser den.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: &'static str,
    pub name: &'static str,
    pub slot: Slot,
    pub rarity: Rarity,
    /// Spesialtrekket i én linje (tom om ingen).
    pub special: String,
    pub stats: Vec<Stat>,
}

impl Weapon {
    pub fn stats(&self) -> Vec<Stat> {
        vec![
            stat("Skade", self.damage, format!("×{:.2}", self.damage), true),
            stat("Fart", self.speed, format!("×{:.2}", self.speed), true),
            stat("Rekkevidde", self.reach, format!("{:.1} m", self.reach), true),
            stat("Etter parade", self.riposte, format!("×{:.1}", self.riposte), true),
            stat("Utholdenhet per hugg", self.stamina, format!("×{:.2}", self.stamina), false),
        ]
    }
}

impl Shield {
    pub fn stats(&self) -> Vec<Stat> {
        vec![
            stat("Slipper gjennom", self.block, format!("{:.0} %", self.block * 100.0), false),
            stat("Paradevindu", self.parry_window, format!("{:.2} s", self.parry_window), true),
            stat("Utholdenhet per blokk", self.stamina, format!("×{:.2}", self.stamina), false),
        ]
    }
}

impl Armor {
    pub fn stats(&self) -> Vec<Stat> {
        vec![
            stat("Skade du tar", self.taken, format!("{:.0} %", self.taken * 100.0), false),
            stat("Utholdenhet tilbake", self.regen, format!("×{:.2}", self.regen), true),
            stat("Unnamanøver koster", self.dodge, format!("×{:.2}", self.dodge), false),
        ]
    }
}

/// Uten rustning: verdiene å sammenligne med.
pub fn no_armor() -> Item {
    Item {
        id: "",
        name: "Ingen rustning",
        slot: Slot::Armor,
        rarity: Rarity::Common,
        special: String::new(),
        stats: vec![stat("Skade du tar", 1.0, "100 %".into(), false), stat("Utholdenhet tilbake", 1.0, "×1.00".into(), true), stat("Unnamanøver koster", 1.0, "×1.00".into(), false)],
    }
}

/// Kortet til en ting (våpen, skjold eller rustning) etter id.
pub fn item(id: &str) -> Option<Item> {
    if let Some(w) = weapon(id) {
        return Some(Item { id: w.id, name: w.name, slot: Slot::Weapon, rarity: w.rarity, special: if w.trait_ == Trait::None && w.riposte <= 2.0 { String::new() } else { w.special.into() }, stats: w.stats() });
    }
    if let Some(s) = shield(id) {
        return Some(Item { id: s.id, name: s.name, slot: Slot::Shield, rarity: s.rarity, special: String::new(), stats: s.stats() });
    }
    armor(id).map(|a| Item { id: a.id, name: a.name, slot: Slot::Armor, rarity: a.rarity, special: String::new(), stats: a.stats() })
}

/// Én linje om tingen etter kanon (plan 13a), som fiendene har sin: hva den
/// er og hva den er god til. Tom for en id som ikke finnes.
pub fn about(id: &str) -> &'static str {
    match id {
        "" => "Bare skjorta og kappa du kom til porten i.",
        "rusty_sword" => "Sverdet du kom til porten med. Rusten i eggen, men det biter ennå.",
        "component_sword" => "Smidd etter mønsterboka av Button og Divider: rett, enkelt og til å stole på.",
        "tag_dagger" => "Kort og lett. Den som ser en annen vei, får stikket dobbelt i ryggen.",
        "card_axe" => "Bladet er slått av Card. Feier bredt og tar flere i samme hugg.",
        "alert_mace" => "Køllehodet ringer som stormklokka. Slår gjennom panser og velter balansen.",
        "tooltip_stiletto" => "Tynn som en nål og like rask. Gir mest etter en parade og i ryggen.",
        "divider_blade" => "Skiller rent der leddene sitter, slik Gorvek gjorde. En parade gir trippelt.",
        "tabs_axe" => "Den tyngste øksa i Fristad, smidd av delene fra den korrupte kjernen.",
        "old_shield" => "Et slitt rundskjold av bord og jernbeslag. Bedre enn ingenting.",
        "card_shield" => "Lett, blåmalt rundskjold. Paraden kommer lettere, og blokken sliter mindre.",
        "badge_heater" => "Trekantskjold med merket fra mønsterboka. Holder godt og glipper sjelden.",
        "fieldset_tower" => "Et tårnskjold som et helt skjema: nesten ingenting slipper gjennom, men det er tungt å løfte.",
        "dialog_heater" => "Fiolett trekantskjold med det lengste paradevinduet i byen.",
        "badge_jerkin" => "Lærvams med Badge sydd på brystet. Tar av for det verste uten å hindre deg.",
        "tag_cuirass" => "Mørkt herdet lær. Lett å bevege seg i, og pusten kommer fortere tilbake.",
        "fieldset_mail" => "Ringbrynje knyttet ring for ring. Holder mye, men du blir tyngre på foten.",
        "search_plate" => "Harnisk av stål. Nesten ingenting biter, men hvert unnaskritt koster.",
        "dialog_plate" => "Mørkt harnisk fra den korrupte kjernen, det beste vernet i riket.",
        "search_lantern" => "Lykt med slipt glass som kaster lyset dobbelt så langt inn i mørket.",
        _ => "",
    }
}

/// Tingen en oppskrift gir, som kort (lykta har sitt eget).
pub fn recipe_item(r: &Recipe) -> Item {
    match r.result {
        Result::Weapon(id) | Result::Shield(id) | Result::Armor(id) => item(id).unwrap_or_else(no_armor),
        Result::Lantern(k) => Item { id: r.id, name: r.name, slot: Slot::Lantern, rarity: r.rarity, special: "lyser lenger inn i mørket".into(), stats: vec![stat("Bærelyset", k, format!("×{k:.1}"), true)] },
    }
}

/// Temaene i krypten, to nivåer hvert (nivå 1–2, 3–4 …).
pub const THEMES: [&str; 5] = ["Gravkjelleren", "Kloakken", "Gruva", "Borgen under jorda", "Den korrupte kjernen"];

/// Temaet nivå `level` (fra 0) hører til: indeksen i `LOOT` og `THEMES`.
pub fn theme_of(level: usize) -> usize {
    (level / 2).min(THEMES.len() - 1)
}

/// Komponentene kistene i hvert tema kan gi, fra lette til tunge.
pub const LOOT: [&[&str]; 5] = [
    &["button", "divider", "badge", "tag", "tag"],
    &["card", "alert", "progress", "badge", "button"],
    &["label", "help-text", "tooltip", "card", "divider"],
    &["fieldset", "file-upload", "search", "label", "alert"],
    &["fieldset", "file-upload", "search", "dialog", "tabs"],
];

/// Galleriveggen i spillerens hus: hver unike komponent som finnes i
/// spillet har sin ramme, i denne rekkefølgen (fra de letteste oppe til
/// venstre til Rammeverkets nederst til høyre).
pub const GALLERY: [&str; 15] = ["button", "tag", "badge", "divider", "card", "alert", "progress", "label", "help-text", "tooltip", "fieldset", "file-upload", "search", "dialog", "tabs"];

/// Modellen som henger i hver ramme, i samme rekkefølge som `GALLERY`: en
/// plate med bildet av selve Fristil-komponenten i standardvarianten
/// (`assets/textures/components/<komponent>.jpg`, rendret av
/// `tools/gallery/render.mjs`, bakt inn av `tools/models/home.py`).
pub const GALLERY_MODELS: [&str; 15] = [
    "gallery_button",
    "gallery_tag",
    "gallery_badge",
    "gallery_divider",
    "gallery_card",
    "gallery_alert",
    "gallery_progress",
    "gallery_label",
    "gallery_help_text",
    "gallery_tooltip",
    "gallery_fieldset",
    "gallery_file_upload",
    "gallery_search",
    "gallery_dialog",
    "gallery_tabs",
];

/// Navnet på en komponent slik Fristil skriver tittelen («help-text» blir
/// «Help Text»), som nettlaget viser den fra `web/docs.json`.
pub fn title(component: &str) -> String {
    component
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Det Rammeverket legger igjen når det er beseiret.
pub const BOSS_LOOT: [&str; 2] = ["dialog", "tabs"];

/// Det elitefienden alltid bærer, per tema.
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
    /// Olje til lykta: fyller den.
    Oil,
}

pub const SMITH_WARES: &[Ware] = &[
    Ware { id: "buy_button", name: "Button", price: 12, what: WareKind::Component("button") },
    Ware { id: "buy_divider", name: "Divider", price: 14, what: WareKind::Component("divider") },
    Ware { id: "buy_badge", name: "Badge", price: 10, what: WareKind::Component("badge") },
    Ware { id: "buy_tag", name: "Tag", price: 8, what: WareKind::Component("tag") },
];
pub const APOTHECARY_WARES: &[Ware] = &[
    Ware { id: "potion", name: "Styrkedrikk (+40 liv)", price: 10, what: WareKind::Heal(40.0) },
    Ware { id: "big_potion", name: "Fullt liv", price: 22, what: WareKind::Heal(100.0) },
    Ware { id: "lamp_oil", name: "Lampeolje (fyller lykta)", price: 6, what: WareKind::Oil },
];
pub const GORVEK_WARES: &[Ware] = &[
    Ware { id: "train_hp", name: "Herding (+20 maks liv)", price: 30, what: WareKind::MaxHp(20.0) },
    Ware { id: "train_stamina", name: "Pust (+20 maks utholdenhet)", price: 30, what: WareKind::MaxStamina(20.0) },
];

/// Hvor en komponent finnes: i kistene (temaene, fra 1), hos eliten
/// (temaene), hos Rammeverket, og prisen hos Buttonsmeden om han selger
/// den.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub chests: Vec<usize>,
    pub elite: Vec<usize>,
    pub boss: bool,
    pub smith: Option<u32>,
}

pub fn source(component: &str) -> Source {
    Source {
        chests: LOOT.iter().enumerate().filter(|(_, l)| l.contains(&component)).map(|(f, _)| f + 1).collect(),
        elite: ELITE_LOOT.iter().enumerate().filter(|(_, c)| **c == component).map(|(f, _)| f + 1).collect(),
        boss: BOSS_LOOT.contains(&component),
        smith: SMITH_WARES.iter().find(|w| matches!(w.what, WareKind::Component(c) if c == component)).map(|w| w.price),
    }
}

impl Source {
    /// Bare i krypten (ikke til salgs i byen).
    pub fn crypt_only(&self) -> bool {
        self.smith.is_none()
    }

    /// Hvor den finnes, kort på én linje til skjemaet i Skriptoriet (plan
    /// 13c): «Kister i Gruva · elite i Kloakken · Buttonsmeden 12 tokens».
    pub fn brief(&self) -> String {
        let name = |f: &usize| THEMES.get(f.wrapping_sub(1)).copied().unwrap_or("krypten");
        let floors = |v: &[usize]| v.iter().map(name).collect::<Vec<_>>().join(" og ");
        let mut parts = vec![];
        if !self.chests.is_empty() {
            parts.push(format!("kister i {}", floors(&self.chests)));
        }
        if !self.elite.is_empty() {
            parts.push(format!("elite i {}", floors(&self.elite)));
        }
        if self.boss {
            parts.push("Rammeverket".to_string());
        }
        if let Some(price) = self.smith {
            parts.push(format!("Buttonsmeden {price} tokens"));
        }
        let mut line = parts.join(" · ");
        if let Some(first) = line.get(..1) {
            line = first.to_uppercase() + &line[1..];
        }
        if line.is_empty() {
            "Krypten".into()
        } else {
            line
        }
    }

    /// Hvor den finnes, som en setning.
    pub fn describe(&self) -> String {
        let name = |f: &usize| THEMES.get(f.wrapping_sub(1)).copied().unwrap_or("krypten").to_string();
        let floors = |v: &[usize]| match v {
            [a] => name(a),
            [rest @ .., last] => format!("{} og {}", rest.iter().map(name).collect::<Vec<_>>().join(", "), name(last)),
            [] => String::new(),
        };
        let mut parts = vec![];
        if !self.chests.is_empty() {
            parts.push(format!("kistene i {}", floors(&self.chests)));
        }
        if !self.elite.is_empty() {
            parts.push(format!("eliten i {}", floors(&self.elite)));
        }
        if self.boss {
            parts.push("Rammeverket".to_string());
        }
        let crypt = match parts.as_slice() {
            [] => "krypten".to_string(),
            [a] => a.clone(),
            [rest @ .., last] => format!("{} og {last}", rest.join(", ")),
        };
        match self.smith {
            Some(price) => format!("Finnes i krypten ({crypt}) og hos Buttonsmeden for {price} tokens."),
            None => format!("Finnes bare i krypten: {crypt}. Kan ikke kjøpes."),
        }
    }
}

pub fn ware(id: &str) -> Option<Ware> {
    SMITH_WARES.iter().chain(APOTHECARY_WARES).chain(GORVEK_WARES).copied().find(|w| w.id == id)
}
