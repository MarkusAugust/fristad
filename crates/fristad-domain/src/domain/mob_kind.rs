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
    Modal,
    /// Eliten i gravkjelleren (Datepicker Priest).
    Priest,
    Table,
    Framework,
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
    pub const ALL: [Kind; 8] = [Kind::Button, Kind::Tooltip, Kind::Input, Kind::Toast, Kind::Modal, Kind::Priest, Kind::Table, Kind::Framework];

    pub fn model(self) -> &'static str {
        match self {
            Kind::Button => "mutant_button",
            Kind::Tooltip => "mutant_tooltip",
            Kind::Input => "mutant_input",
            Kind::Toast => "mutant_toast",
            Kind::Modal => "mutant_modal",
            Kind::Priest => "datepicker_priest",
            Kind::Table => "mutant_table",
            Kind::Framework => "framework",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Button => "Korrupt button",
            Kind::Tooltip => "Korrupt tooltip",
            Kind::Input => "Korrupt input",
            Kind::Toast => "Korrupt toast",
            Kind::Modal => "Korrupt modal",
            Kind::Priest => "Datepicker-presten",
            Kind::Table => "Korrupt tabell",
            Kind::Framework => "Rammeverket",
        }
    }

    pub fn boss(self) -> bool {
        self == Kind::Framework
    }

    pub fn stats(self) -> Stats {
        let bite = (15.0 / FPS, 30.0 / FPS);
        let slam = (28.0 / FPS, 44.0 / FPS);
        match self {
            Kind::Button => Stats { hp: MOB_HP, speed: 1.4, damage: 12.0, range: 1.7, reach: 2.3, scale: 1.0, sight: 26.0, cooldown: 0.9, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Tooltip => Stats { hp: 38.0, speed: 3.6, damage: 6.0, range: 1.3, reach: 1.7, scale: 0.55, sight: 30.0, cooldown: 0.7, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 1 },
            Kind::Input => Stats { hp: 80.0, speed: 1.9, damage: 10.0, range: 1.8, reach: 2.2, scale: 1.15, sight: 34.0, cooldown: 1.6, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.5, 14.0, 14.0)), armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Toast => Stats { hp: 45.0, speed: 2.4, damage: 8.0, range: 1.4, reach: 1.9, scale: 1.0, sight: 28.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.0, 12.0, 10.0)), armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Modal => Stats { hp: 320.0, speed: 1.3, damage: 30.0, range: 2.2, reach: 3.0, scale: 1.5, sight: 24.0, cooldown: 1.8, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.3, poise: 40.0, tokens: 6 },
            Kind::Priest => Stats { hp: 160.0, speed: 1.2, damage: 22.0, range: 2.0, reach: 2.7, scale: 0.8, sight: 28.0, cooldown: 1.5, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.6, poise: 25.0, tokens: 6 },
            Kind::Table => Stats { hp: 420.0, speed: 2.4, damage: 22.0, range: 2.2, reach: 3.0, scale: 1.4, sight: 30.0, cooldown: 1.1, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 30.0, tokens: 10 },
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
