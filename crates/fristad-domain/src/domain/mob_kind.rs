//! Slagene av fiender (`Kind`) og tallene bak dem (`Stats`): modellen,
//! navnet, liv, fart, skade og rekkevidde. Oppførselen står i `mobs.rs`.

use crate::domain::mobs::{FPS, MOB_HP};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Button,
    Tooltip,
    Input,
    /// Frosken i kloakken (Toast Frog): hopper og spytter.
    Toast,
    /// Eliten i kloakken (Select Broodmother).
    Select,
    Modal,
    /// Eliten i gravkjelleren (Datepicker Priest).
    Priest,
    Table,
    Framework,
    /// Gravkongens hall (nivå 2): ruller som et hjul og hopper opp og biter.
    Avatar,
    /// Gravkongens hall: en kjede av kuler som kryper etter deg.
    Breadcrumb,
    /// Gravkongens hall: ligger som en kiste og folder seg ut i bakhold.
    Accordion,
}

/// Tallene bak et slag.
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    /// Så nær må du være før den går til angrep, og så langt rekker slaget.
    pub range: f32,
    pub reach: f32,
    pub scale: f32,
    pub sight: f32,
    pub cooldown: f32,
    /// Sekundet i attack-klippet der slaget lander, og klippets lengde.
    pub attack_hit: f32,
    pub attack_len: f32,
    /// Skyter på avstand: (nærmeste, lengste avstand, skade).
    pub bolt: Option<(f32, f32, f32)>,
    /// Så mye av skaden slipper gjennom forfra når den ikke angriper.
    pub armor: f32,
    /// Slag under dette avbryter den ikke.
    pub poise: f32,
    pub tokens: u32,
}

impl Kind {
    pub const ALL: [Kind; 12] = [Kind::Button, Kind::Tooltip, Kind::Input, Kind::Toast, Kind::Select, Kind::Modal, Kind::Priest, Kind::Table, Kind::Framework, Kind::Avatar, Kind::Breadcrumb, Kind::Accordion];

    pub fn model(self) -> &'static str {
        match self {
            Kind::Button => "mutant_button",
            Kind::Tooltip => "mutant_tooltip",
            Kind::Input => "mutant_input",
            Kind::Toast => "mutant_toast",
            Kind::Select => "mutant_select",
            Kind::Modal => "mutant_modal",
            Kind::Priest => "datepicker_priest",
            Kind::Table => "mutant_table",
            Kind::Framework => "framework",
            Kind::Avatar => "avatar_skull",
            Kind::Breadcrumb => "breadcrumb_crawler",
            Kind::Accordion => "accordion_coffin",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Button => "Korrupt button",
            Kind::Tooltip => "Korrupt tooltip",
            Kind::Input => "Korrupt input",
            Kind::Toast => "Korrupt toast",
            Kind::Select => "Select-moren",
            Kind::Modal => "Korrupt modal",
            Kind::Priest => "Datepicker-presten",
            Kind::Table => "Korrupt tabell",
            Kind::Framework => "Rammeverket",
            Kind::Avatar => "Korrupt avatar",
            Kind::Breadcrumb => "Korrupt breadcrumb",
            Kind::Accordion => "Korrupt accordion",
        }
    }

    /// Kroppen å sikte på, i modellens målestokk: (toppen, radius, halv
    /// lengde). Uten lengde er det en stående kapsel fra beina til toppen;
    /// med lengde ligger den langs ryggen i halve høyden (de lange).
    pub fn body(self) -> (f32, f32, f32) {
        match self {
            Kind::Input => (0.5, 0.32, 0.3),
            Kind::Table => (0.5, 0.36, 1.3),
            Kind::Toast => (0.55, 0.36, 0.0),
            Kind::Priest => (2.05, 0.45, 0.0),
            Kind::Select => (1.55, 0.6, 0.0),
            Kind::Avatar => (0.72, 0.34, 0.0),
            Kind::Breadcrumb => (0.55, 0.3, 0.55),
            Kind::Accordion => (1.2, 0.36, 0.0),
            _ => (1.45, 0.5, 0.0),
        }
    }

    pub fn boss(self) -> bool {
        self == Kind::Framework
    }

    /// Slaget som står i stedet når slagets egen modell mangler: Rammeverket
    /// har ingen modell ennå (`model::PLANNED`), så modalen står der, større
    /// (`Stats.scale`).
    pub fn stand_in(self) -> Option<Kind> {
        match self {
            Kind::Framework => Some(Kind::Modal),
            _ => None,
        }
    }

    pub fn stats(self) -> Stats {
        let bite = (15.0 / FPS, 30.0 / FPS);
        let slam = (28.0 / FPS, 44.0 / FPS);
        match self {
            Kind::Button => Stats { hp: MOB_HP, speed: 1.4, damage: 12.0, range: 1.7, reach: 2.3, scale: 1.0, sight: 26.0, cooldown: 0.9, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Tooltip => Stats { hp: 38.0, speed: 3.6, damage: 6.0, range: 1.3, reach: 1.7, scale: 0.55, sight: 30.0, cooldown: 0.7, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 1 },
            Kind::Input => Stats { hp: 80.0, speed: 1.9, damage: 10.0, range: 1.8, reach: 2.2, scale: 1.15, sight: 34.0, cooldown: 1.6, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.5, 14.0, 14.0)), armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Toast => Stats { hp: 45.0, speed: 2.4, damage: 8.0, range: 1.4, reach: 1.9, scale: 1.0, sight: 28.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.0, 12.0, 10.0)), armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Select => Stats { hp: 150.0, speed: 1.6, damage: 18.0, range: 2.0, reach: 2.6, scale: 1.0, sight: 26.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 0.7, poise: 20.0, tokens: 6 },
            Kind::Modal => Stats { hp: 320.0, speed: 1.3, damage: 30.0, range: 2.2, reach: 3.0, scale: 1.5, sight: 24.0, cooldown: 1.8, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.3, poise: 40.0, tokens: 6 },
            Kind::Priest => Stats { hp: 160.0, speed: 1.2, damage: 22.0, range: 2.0, reach: 2.7, scale: 0.8, sight: 28.0, cooldown: 1.5, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.6, poise: 25.0, tokens: 6 },
            Kind::Table => Stats { hp: 420.0, speed: 2.4, damage: 22.0, range: 2.2, reach: 3.0, scale: 1.4, sight: 30.0, cooldown: 1.1, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 30.0, tokens: 10 },
            // Rullingen er 2πR (1,9 m) på 18 bilder: 2,6 m/s.
            Kind::Avatar => Stats { hp: 40.0, speed: 2.6, damage: 9.0, range: 1.3, reach: 1.8, scale: 1.0, sight: 28.0, cooldown: 1.0, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Breadcrumb => Stats { hp: 55.0, speed: 1.9, damage: 8.0, range: 1.5, reach: 2.0, scale: 1.0, sight: 30.0, cooldown: 1.0, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Accordion => Stats { hp: 90.0, speed: 1.3, damage: 14.0, range: 1.6, reach: 2.2, scale: 1.0, sight: 18.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 0.8, poise: 10.0, tokens: 3 },
            Kind::Framework => Stats { hp: 1400.0, speed: 1.6, damage: 34.0, range: 3.0, reach: 4.0, scale: 2.3, sight: 40.0, cooldown: 1.2, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 55.0, tokens: 40 },
        }
    }

    /// Slaget etter navnet i nivåfila (`label` på merket): norsk eller
    /// engelsk, og «knapp i bakhold» er en knapp. Ukjente blir knapper.
    pub fn from_label(label: &str) -> Kind {
        let l = label.to_lowercase();
        let has = |w: &[&str]| w.iter().any(|x| l.contains(x));
        if has(&["rammeverk", "framework"]) {
            Kind::Framework
        } else if has(&["tabell", "table"]) {
            Kind::Table
        } else if has(&["datepicker", "prest", "priest"]) {
            Kind::Priest
        } else if has(&["modal"]) {
            Kind::Modal
        } else if has(&["select", "brood"]) {
            Kind::Select
        } else if has(&["avatar"]) {
            Kind::Avatar
        } else if has(&["breadcrumb", "brødsmule", "smule"]) {
            Kind::Breadcrumb
        } else if has(&["accordion", "trekkspill", "coffin"]) {
            Kind::Accordion
        } else if has(&["toast", "frosk"]) {
            Kind::Toast
        } else if has(&["input"]) {
            Kind::Input
        } else if has(&["tooltip"]) {
            Kind::Tooltip
        } else {
            if !has(&["knapp", "button"]) {
                log::warn!("Ukjent fiende «{label}»: blir en knapp");
            }
            Kind::Button
        }
    }
}
