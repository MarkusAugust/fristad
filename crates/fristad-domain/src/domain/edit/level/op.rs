//! Handlingene i nivåredigeringen (`LevelOp`), som json fra kartet.

use super::Head;
use crate::domain::world::crypt::doc::{DocLight, DocModule, LevelDoc, Thing};
use serde::{Deserialize, Serialize};

/// Det eieren gjør, som json fra nettlaget (`{"op": "add_module", …}`).
/// Rutene er (i, y, j) (cellene på 4 × 4 m med gulvet i y meter), stedene
/// for innhold er i nivåets rom (meter).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum LevelOp {
    /// Sett ut en modul (`grav/kapell`) med midtcella i `at`.
    AddModule {
        module: String,
        at: [i32; 3],
        #[serde(default)]
        rot: u8,
        #[serde(default)]
        mirror: bool,
    },
    /// Flytt moduler og innhold (rutene; innholdet 4 m per rute og 1 m per trinn).
    Move {
        ids: Vec<String>,
        by: [i32; 3],
    },
    /// Drei det valgte 90° med klokka (`back`: mot) om den første modulen.
    Turn {
        ids: Vec<String>,
        #[serde(default)]
        back: bool,
    },
    /// Speil det valgte langs x om den første modulen.
    Mirror {
        ids: Vec<String>,
    },
    Delete {
        ids: Vec<String>,
    },
    /// Lim inn kopier (nye id-er), flyttet `by`.
    Paste {
        #[serde(default)]
        modules: Vec<DocModule>,
        #[serde(default)]
        things: Vec<Thing>,
        by: [i32; 3],
    },
    /// Bytt ut en modul (samme id) med det egenskapspanelet har endret.
    SetModule {
        module: DocModule,
    },
    /// Slaget for en åpning i en modul (`auto` eller tomt fjerner valget).
    Opening {
        module: String,
        opening: String,
        kind: String,
        #[serde(default)]
        id: Option<String>,
    },
    /// Sett ut innhold (`mob`, `chest`, `lever` …) i nivåets rom.
    AddThing {
        kind: String,
        at: [f32; 3],
        #[serde(default)]
        yaw: f32,
        #[serde(default)]
        label: Option<String>,
    },
    MoveThing {
        id: String,
        to: [f32; 3],
    },
    SetThing {
        thing: Thing,
    },
    Link {
        from: String,
        to: String,
        kind: String,
    },
    Unlink {
        from: String,
        to: String,
    },
    AddLight {
        at: [f32; 3],
        #[serde(default)]
        color: Option<[f32; 3]>,
        #[serde(default)]
        range: Option<f32>,
    },
    SetLight {
        light: DocLight,
    },
    SetHead {
        head: Head,
    },
    /// Løsne innholdet `<modul>/<id>` fra modulmalen: det blir eierens i
    /// dokumentet med samme id og sted (malen tar det bort i dette nivået,
    /// `DocModule.hidden`), så det kan flyttes og endres. `move_thing`,
    /// `set_thing` og `delete` på malens innhold gjør dette selv.
    Detach {
        id: String,
    },
    /// Malens innhold tilbake slik malen har det (det endrede eller fjernede
    /// glemmes).
    Reattach {
        module: String,
        thing: String,
    },
    /// Sett ut en mal eieren har lagret (`LevelEditor::template`,
    /// `assets/levels/maler/<navn>.json`) med den første modulen i `at`,
    /// speilet langs x og så dreid `rot` kvart omdreininger med klokka. Alt
    /// får nye id-er, og koblingene i malen følger med.
    Stamp {
        template: Box<LevelDoc>,
        at: [i32; 3],
        #[serde(default)]
        rot: u8,
        #[serde(default)]
        mirror: bool,
    },
}

