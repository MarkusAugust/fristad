//! Nivådokumentet (`assets/levels/nivaa_NN.json`, `docs/nivaaredigering.md`
//! del 6): kilden nivåredigeringen endrer, og som kjernen kompilerer til
//! `LevelFile` (`compose`). Lesingen er raus som for `LevelFile`: ukjente
//! felt hoppes over, og det som mangler får en standardverdi.
//!
//! - Cellene står på rutenettet på 4 × 4 m: `at` er (i, y, j) med midten i
//!   (4i, y, 4j) i nivåets rom og gulvet i y (heltall meter).
//! - `rot` dreier biten 90° med klokka sett ovenfra (N → Ø), og `mirror`
//!   speiler den langs x før den dreies.
//! - Sidene (`N`, `E`, `S`, `W`) i `edges` og i ankrene er i cellens eget rom,
//!   så de følger biten når den dreies eller speiles.
//! - `yaw` i dokumentet er grader (0 ser mot +z i ankerets rom).

use serde::{Deserialize, Serialize};

/// Nummeret (1–10) eller navnet (`test`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum LevelId {
    N(u32),
    Name(String),
}

impl Default for LevelId {
    fn default() -> Self {
        LevelId::Name(String::new())
    }
}

impl LevelId {
    pub fn number(&self) -> Option<u32> {
        match self {
            LevelId::N(n) => Some(*n),
            LevelId::Name(s) => s.parse().ok(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct LevelDoc {
    #[serde(default = "one")]
    pub format: u32,
    #[serde(default)]
    pub level: LevelId,
    #[serde(default)]
    pub name: String,
    /// `utkast` eller `klar`.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub seed: u64,
    /// Settene og versjonene nivået ble laget med (`grav@1`).
    #[serde(default)]
    pub kit: Vec<String>,
    #[serde(default)]
    pub bands: Vec<Band>,
    #[serde(default)]
    pub rooms: Vec<DocRoom>,
    #[serde(default)]
    pub cells: Vec<DocCell>,
    #[serde(default)]
    pub edges: Vec<DocEdge>,
    #[serde(default)]
    pub things: Vec<Thing>,
    #[serde(default)]
    pub links: Vec<DocLink>,
    #[serde(default)]
    pub lights: Vec<DocLight>,
}

fn one() -> u32 {
    1
}

impl LevelDoc {
    pub fn ready(&self) -> bool {
        self.status == "klar"
    }
}

/// En etasje i kartet: bare en visning.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Band {
    pub name: String,
    pub from: f32,
    pub to: f32,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocRoom {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocCell {
    pub id: String,
    /// Biten (`grav/romfelt`).
    pub bit: String,
    /// (i, y, j).
    pub at: [i32; 3],
    #[serde(default)]
    pub rot: u8,
    #[serde(default)]
    pub mirror: bool,
    #[serde(default)]
    pub room: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Eierens valg for en kant: `vegg`, `dor`, `port`, `bue`, `aapen`,
/// `falsk`, `snarvei` (vindu, gitter og rekkverk kommer i fase 6).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocEdge {
    pub cell: String,
    /// `N`, `E`, `S` eller `W` i cellens rom.
    pub side: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Innholdet: det som blir merkene i nivåfila.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Thing {
    pub id: String,
    /// Som merkene: `start`, `up`, `down`, `rest`, `mob`, `elite`, `boss`,
    /// `chest`, `rare`, `lore`, `bonus`, `lever`, `gate`, `shortcut`,
    /// `secret_wall`, `hint`, `trap`, `prop` …
    pub kind: String,
    /// `fri`, `celle:<id>` eller `kant:<celle>/<side>`.
    #[serde(default)]
    pub anchor: String,
    /// I ankerets rom (utelatt: festet i biten, eller midt på kanten).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pos: Option<[f32; 3]>,
    /// Grader.
    #[serde(default)]
    pub yaw: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Patruljen, i ankerets rom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patrol: Vec<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambush: Option<super::level::AmbushDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<super::level::Size>,
}

/// En kobling: `opens` (spak eller elite til port), `key` (eliten bærer
/// nøkkelen), `arena` (port til rom), `exit` (porten ut av arenaen; `to`
/// er porten), `for` (hint til hemmelighet), `to` (fallem til rom), `from`
/// (snarveiens slåside: rommet).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocLink {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocLight {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub anchor: String,
    pub pos: [f32; 3],
    #[serde(default = "warm")]
    pub color: [f32; 3],
    #[serde(default = "eight")]
    pub range: f32,
    #[serde(default)]
    pub flicker: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intensity: Option<f32>,
}

fn warm() -> [f32; 3] {
    [1.0, 0.62, 0.3]
}

fn eight() -> f32 {
    8.0
}
