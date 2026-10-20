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

use super::stone::StoneRef;
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
    /// Nivåets tema (`grav`, `kloakk` …) for rommene som ikke har sitt
    /// eget: byggesettet og standardsteinen (`steiner.json`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub theme: String,
    /// Steinen i hele nivået (over temaets standardstein).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<StoneRef>,
    /// Modulene eieren har satt ut (del 21): utvides til celler, rom,
    /// innhold og kantvalg før kompileringen (`compose::modules::flatten`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<DocModule>,
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
    /// Stemningen (fase 2: bare faklene; tåke, lyd og forfall kommer i fase 5).
    #[serde(default, skip_serializing_if = "Mood::is_default")]
    pub mood: Mood,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<StoneRef>,
}

/// Stemningen i et rom (`docs/nivaaredigering.md` del 8.4): faklene,
/// lysstyrken og fargen på lysene, tåka og lyden.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Mood {
    /// Andelen av fakkelfestene i rommets murer som får en fakkel (0–1),
    /// valgt med nivåets frø og plassen.
    #[serde(default)]
    pub torches: f32,
    /// Lysstyrken: faklene og lysene i rommet ganges med den (1 er vanlig,
    /// 0,3 nesten mørkt, 2 dobbelt så sterkt).
    #[serde(default = "unit", skip_serializing_if = "is_unit")]
    pub light: f32,
    /// Fargen på faklene og modulenes lys i rommet (utelatt: ilden; lysene
    /// eieren setter ut har sin egen).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[f32; 3]>,
    /// Tåka i rommet, 0–1 (`Room.fog`; økta gjør den til tåke rundt deg).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fog: f32,
    /// Lyden i rommet (`SOUNDS`; tomt: dryppet i kjelleren).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sound: String,
}

impl Default for Mood {
    fn default() -> Self {
        Mood { torches: 0.0, light: 1.0, color: None, fog: 0.0, sound: String::new() }
    }
}

/// Lydene et rom kan ha, med navnet eieren ser: løkkene i `Ambience` nede
/// i krypten (over dryppet, som alltid er der).
pub const SOUNDS: [(&str, &str); 6] = [("", "Drypp (kjelleren)"), ("vann", "Rennende vann"), ("vind", "Grottevind"), ("kloft", "Vinden fra kløfta"), ("lava", "Lava"), ("ild", "Ild som knitrer")];

fn unit() -> f32 {
    1.0
}

fn is_unit(v: &f32) -> bool {
    *v == 1.0
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

impl Mood {
    fn is_default(&self) -> bool {
        *self == Mood::default()
    }

    /// Bare faklene (resten som vanlig).
    pub fn torches(torches: f32) -> Self {
        Mood { torches, ..Mood::default() }
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<StoneRef>,
}

/// Eierens valg for en kant: `vegg`, `dor`, `port`, `bue`, `aapen`,
/// `falsk`, `snarvei` (vindu, gitter og rekkverk kommer i fase 6).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocEdge {
    pub cell: String,
    /// `N`, `E`, `S` eller `W` i cellens rom.
    pub side: String,
    /// Tomt: kanten låser bare utseendet eller steinen.
    #[serde(default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Steinen i muren på denne siden (halvkanten til cella).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<StoneRef>,
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
    /// Toppen (stigen, broas feste i hvelvet) og bunnen (heisen), i
    /// ankerets rom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<[f32; 3]>,
    /// Vannstanden høy og lav (slusa), over ankerets gulv.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub levels: Vec<f32>,
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

/// En modul satt ut i nivået (`docs/nivaaredigering.md`, del 21): malen
/// `module` (`grav/kapell`) med midtcella (modulens (0, 0)) i `at`, dreid
/// `rot` kvart omdreininger med klokka og speilet langs x før den dreies,
/// som cellene. Cellene får id-en `<id>/<celle>`, og rommet heter `id`
/// (eller `room`, om modulen skal høre til et rom en annen modul har).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocModule {
    pub id: String,
    pub module: String,
    /// (i, y, j).
    pub at: [i32; 3],
    #[serde(default)]
    pub rot: u8,
    #[serde(default)]
    pub mirror: bool,
    /// Rommet modulen hører til (utelatt: sitt eget, som heter `id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
    /// Navnet på rommet (utelatt: malens).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mood: Option<Mood>,
    /// Steinen i modulen (over rommets og nivåets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<StoneRef>,
    /// Eierens valg for åpningene: dør, port, falsk vegg, gang, bue,
    /// snarvei eller vegg.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<DocOverride>,
    /// Innholdet fra malen eieren har tatt bort i dette nivået (malens id-er,
    /// `spak`). Står det et innhold i dokumentet med id-en `<modul>/<id>`,
    /// er det malens innhold flyttet eller endret (koblingene i malen peker
    /// dit); ellers er det fjernet.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<String>,
}

/// Eierens valg for én åpning i en modul. `kind`: `dor`, `gang`, `bue`,
/// `port`, `falsk`, `snarvei` eller `vegg`. Porten, snarveien og den falske
/// veggen blir innhold med id-en `id` (utelatt: `<modul>/<åpning>`), som
/// koblingene kan peke på.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct DocOverride {
    pub opening: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// En modul (`assets/levels/moduler/<tema>/<navn>.json`): cellene, rommet,
/// standardinnholdet, lysene og åpningene, i modulens eget rom (midtcella
/// i (0, 0, 0), `rot` 0). Hver ytre side av en celle som ikke er lukket, er
/// en åpning (`auto`), utenom `walls`; `openings` gir en åpning et annet
/// slag eller navn. Id-en til en åpning er `i,j,S` (cella og siden i
/// modulens rom) om den ikke har et navn.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct ModuleDef {
    /// `grav/kapell`.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub theme: String,
    /// `rom`, `gang`, `trapp` eller `ferdig` (paletten grupperer etter det).
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub about: String,
    /// Rommene i modulen; det første er hovedrommet (cellene uten `room`).
    #[serde(default)]
    pub rooms: Vec<DocRoom>,
    #[serde(default)]
    pub cells: Vec<DocCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub openings: Vec<ModuleOpening>,
    /// Ytre sider som alltid er mur.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub walls: Vec<ModuleOpening>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<DocEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub things: Vec<Thing>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DocLink>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lights: Vec<DocLight>,
}

/// En åpning (eller en mur) på en ytre side av en modul: cella (i, j) og
/// siden i modulens rom, og slaget (`auto`: dør mot et rom, gangåpning mot
/// en gang; `gang`, `bue`, `dor`, `port`, `falsk`, `snarvei`).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct ModuleOpening {
    #[serde(default)]
    pub id: String,
    pub at: [i32; 2],
    pub side: String,
    #[serde(default)]
    pub kind: String,
}
