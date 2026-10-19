//! Byggesettet for nivåredigeringen (`docs/nivaaredigering.md`, del 4):
//! bitene fra Blender (`tools/crypt/kit_cells.py`), beskrevet i manifestet
//! `assets/models/crypt/kit/<tema>.json`, som adapterne leser og gir
//! kjernen. Kjernen ser bare dataene: hvilke flater en bit vil ha, gulvene,
//! kolliderne og det gangbare. Geometrien er modellen (`model`).
//!
//! Bitenes rom (akser som i spillet, x mot øst, y opp, z mot sør):
//! - **celle**: midten av cella på gulvet; sidene i rekkefølgen N (−z),
//!   Ø (+x), S (+z), V (−x).
//! - **halvkant**: midt på kantlinja; muren går 0,4 m inn i cella mot +z.
//! - **kvarthjørne**: i hjørnet; fyller ruta 0,4 × 0,4 m mot +x og +z.
//! - **søyle**: i hjørnet der fire celler møtes.

use super::level::ColliderDef;
use serde::{Deserialize, Serialize};

/// Alle bitene nivåene kan bygges av (alle temaene slått sammen).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct KitDef {
    #[serde(default)]
    pub pieces: Vec<KitPiece>,
}

/// Ett manifest (`<tema>.json`).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct KitFile {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub pieces: Vec<KitPiece>,
}

impl KitDef {
    pub fn add(&mut self, f: KitFile) {
        self.pieces.extend(f.pieces);
    }

    pub fn piece(&self, id: &str) -> Option<&KitPiece> {
        self.pieces.iter().find(|p| p.id == id)
    }

    /// Halvkanten for en åpning i et tema (eller i et hvilket som helst tema).
    pub fn edge(&self, theme: &str, opening: &str) -> Option<&KitPiece> {
        let of = |p: &&KitPiece| p.kind == PieceKind::Edge && p.opening.as_deref() == Some(opening);
        self.pieces.iter().filter(of).find(|p| p.theme == theme).or_else(|| self.pieces.iter().find(of))
    }

    /// Kvarthjørnet eller søyla i et tema.
    pub fn of_kind(&self, theme: &str, kind: PieceKind) -> Option<&KitPiece> {
        self.pieces.iter().filter(|p| p.kind == kind).find(|p| p.theme == theme).or_else(|| self.pieces.iter().find(|p| p.kind == kind))
    }

    /// Modellene bitene bruker.
    pub fn models(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.pieces.iter().map(|p| p.model.as_str()).collect();
        out.sort();
        out.dedup();
        out
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PieceKind {
    #[default]
    Cell,
    Edge,
    Corner,
    Pillar,
}

/// Hva en side av en celle vil ha.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Face {
    /// `open` (rom), `gang` eller `closed`.
    pub kind: String,
    /// Hvelvet ved kanten: `room` eller `gang` (en gangåpning trenger ingen
    /// overmur på en side som har gangprofilen).
    #[serde(default)]
    pub profile: String,
}

/// En gangbar flate i bitens rom: rektangelet (x0, z0, x1, z1) med høyden
/// `y[0]` ved z0 og `y[1]` ved z1 (skrå for trappene).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Walk {
    pub rect: [f32; 4],
    #[serde(default)]
    pub y: [f32; 2],
}

/// Et feste i biten for innhold som hører til den (trappa ned i nedgangen).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Mount {
    pub kind: String,
    pub pos: [f32; 3],
    /// Radianer.
    #[serde(default)]
    pub yaw: f32,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct KitPiece {
    /// `grav/romfelt`.
    pub id: String,
    pub model: String,
    #[serde(default)]
    pub kind: PieceKind,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub v: u32,
    // Cellene.
    #[serde(default)]
    pub faces: Vec<Face>,
    #[serde(default)]
    pub floors: [f32; 4],
    /// Fri høyde over det gangbare.
    #[serde(default)]
    pub clear: f32,
    /// Volumet biten bruker i høyden (over gulvet), mot overlapp.
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub walk: Vec<Walk>,
    #[serde(default)]
    pub mounts: Vec<Mount>,
    // Halvkantene.
    #[serde(default)]
    pub opening: Option<String>,
    #[serde(default)]
    pub width: f32,
    #[serde(default)]
    pub height: f32,
    // Søyla.
    #[serde(default)]
    pub radius: f32,
    #[serde(default)]
    pub solids: Vec<ColliderDef>,
    #[serde(default)]
    pub mirror: bool,
}
