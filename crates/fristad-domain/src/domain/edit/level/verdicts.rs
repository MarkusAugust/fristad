//! Eierens dom i nivåredigeringen (eieren 26. november): paletten merker
//! modulene og tingene som ikke er godkjent, og et nivå kan ikke merkes
//! «klar» med noe som er underkjent. Usikkert, og det som venter på dom
//! (nytt, eller endret etter dommen), gir en advarsel. Utkast kan bruke alt.
//!
//! Dommene står i `assets/catalog/approvals.json` og leses av adapteren
//! (`data::level_verdicts`); kjernen får dem ferdig lest.

use crate::domain::world::crypt::compose::{Issue, Severity};
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::kit::KitDef;
use serde::Serialize;
use std::collections::BTreeMap;

/// Kortet på godkjenningssiden (`mekanismer` i `assets/catalog/modules.json`)
/// som dømmer en ting fra paletten.
pub const THING_CARDS: [(&str, &str); 8] = [
    ("up", "mek_dor_opp"),
    ("down", "mek_trapp_ned"),
    ("rest", "mek_hvilested"),
    ("lore", "mek_lappepult"),
    ("bonus", "mek_styrke"),
    ("lever", "mek_portgitter"),
    ("trap", "mek_trykkplate"),
    ("hint", "mek_sprekk"),
];

/// Modellen bildet av en ting i paletten viser (`assets/catalog/thumbs/`).
pub const THING_PICTURES: [(&str, &str); 10] = [
    ("up", "kit_up"),
    ("down", "kit_down"),
    ("rest", "kit_rest"),
    ("chest", "chest"),
    ("rare", "chest"),
    ("lore", "kit_lore"),
    ("bonus", "kit_bonus"),
    ("lever", "kit_lever"),
    ("trap", "kit_plate"),
    ("hint", "kit_crack"),
];

/// Et avvik: `underkjent`, `usikker` eller `venter`, med eierens grunn.
#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Verdict {
    pub state: String,
    pub why: String,
}

/// Avvikene, for modulene (`<tema>/<navn>`) og for tingene og fiendene
/// (nøkkelen i paletten: slaget, eller `mob:<slag>`). Det som er godkjent,
/// står ikke her.
#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Verdicts {
    pub modules: BTreeMap<String, Verdict>,
    pub things: BTreeMap<String, Verdict>,
}

fn issue(what: String, v: &Verdict) -> Issue {
    let why = if v.why.is_empty() { String::new() } else { format!(": {}", v.why) };
    let (severity, msg) = match v.state.as_str() {
        "underkjent" => (Severity::Error, format!("{what} er underkjent{why}")),
        "usikker" => (Severity::Warning, format!("{what} er usikker{why}")),
        _ => (Severity::Warning, format!("{what} venter på dom")),
    };
    Issue { severity, rule: "dom", at: None, msg }
}

/// Bruddene for det nivået bruker: én melding per modul og per slag ting,
/// også tingene som følger med modulene.
pub fn check(doc: &LevelDoc, kit: &KitDef, verdicts: &Verdicts) -> Vec<Issue> {
    let mut out = vec![];
    for m in &doc.modules {
        if let Some(v) = verdicts.modules.get(&m.module) {
            out.push(issue(format!("Modulen {}", m.module), v));
        }
    }
    let inside = doc.modules.iter().filter_map(|m| kit.module(&m.module)).flat_map(|m| m.things.iter());
    for t in doc.things.iter().chain(inside) {
        let key = match (t.kind.as_str(), &t.label) {
            ("mob" | "elite" | "boss", Some(l)) => format!("mob:{l}"),
            (k, _) => k.to_string(),
        };
        if let Some(v) = verdicts.things.get(&key) {
            let name = super::map::THING_NAMES.iter().find(|(k, _)| *k == t.kind).map_or(t.kind.as_str(), |(_, n)| *n);
            out.push(issue(if key.starts_with("mob:") { format!("{name} ({})", &key[4..]) } else { name.to_string() }, v));
        }
    }
    out
}
