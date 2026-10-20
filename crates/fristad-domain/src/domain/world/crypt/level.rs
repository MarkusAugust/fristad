//! `crypt_NN.json` slik Blender skriver den (se `docs/krypten/format.md`).
//! Lesingen er raus: ukjente felt og merker hoppes over, og det som mangler
//! får en standardverdi, så et nivå under arbeid fortsatt kan lastes.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct LevelFile {
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub pieces: Vec<String>,
    #[serde(default)]
    pub rooms: Vec<RoomDef>,
    #[serde(default)]
    pub colliders: Vec<ColliderDef>,
    #[serde(default)]
    pub nav: NavDef,
    #[serde(default)]
    pub lights: Vec<LightDef>,
    #[serde(default)]
    pub marks: Vec<MarkDef>,
    /// Bitene fra byggesettet (nivåer kompilert fra et nivådokument,
    /// `compose`): modellen, stedet, dreiningen og speilingen. `pieces` står
    /// for de gamle nivåene.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<InstanceDef>,
}

/// En bit fra byggesettet satt ut i nivået.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct InstanceDef {
    pub model: String,
    pub pos: [f32; 3],
    /// Radianer om y (0: modellens +z mot +z).
    #[serde(default)]
    pub yaw: f32,
    /// Speilet langs modellens x før dreiningen.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mirror: bool,
    /// Steinbyttet (`stone`): lagene biten får i stedet for sine egne.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<super::stone::InstanceStone>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct RoomDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub poly: Vec<[f32; 2]>,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_h")]
    pub h: f32,
    /// Hvelvet over den frie høyden: så langt over `h` rommet når i lyset
    /// (hvelvene i salene og kuppelen i rotunden; ellers var de svarte).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rise: f32,
    #[serde(default)]
    pub secret: bool,
    /// Tåka i rommet, 0–1 (stemningen, `Mood.fog`).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fog: f32,
    /// Lyden i rommet (`doc::SOUNDS`; tomt: dryppet i kjelleren).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sound: String,
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

fn default_h() -> f32 {
    4.0
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BoxDef {
    pub center: [f32; 3],
    pub size: [f32; 3],
    #[serde(default)]
    pub yaw: f32,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct ColliderDef {
    #[serde(rename = "box")]
    pub cuboid: Option<BoxDef>,
    pub hull: Option<Vec<[f32; 3]>>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct NavDef {
    #[serde(default)]
    pub verts: Vec<[f32; 3]>,
    #[serde(default)]
    pub polys: Vec<Vec<u32>>,
    #[serde(default)]
    pub poly_room: Vec<Option<String>>,
    #[serde(default)]
    pub poly_gate: Vec<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LightDef {
    pub pos: [f32; 3],
    #[serde(default = "default_color")]
    pub color: [f32; 3],
    #[serde(default = "default_range")]
    pub range: f32,
    #[serde(default)]
    pub flicker: bool,
    /// Styrken (ikke i formatet ennå; 2 om den mangler).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
}

fn default_color() -> [f32; 3] {
    [1.0, 0.62, 0.3]
}

fn default_range() -> f32 {
    8.0
}

/// Et tall, eller to eller tre (mål).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(untagged)]
pub enum Size {
    One(f32),
    Two([f32; 2]),
    Three([f32; 3]),
}

impl Size {
    /// Bredde og dybde (x, z).
    pub fn wd(self) -> (f32, f32) {
        match self {
            Size::One(s) => (s, s),
            Size::Two([w, d]) => (w, d),
            Size::Three([w, _, d]) => (w, d),
        }
    }
}

/// Retningen på en felle: en vektor, eller en yaw i radianer.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(untagged)]
pub enum Dir {
    Yaw(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
}

/// Bakholdet: på (vekkes når du kommer innen 4 m), en radius, eller
/// rom-id-en der du må gå inn.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum AmbushDef {
    On(bool),
    Radius(f32),
    Room(String),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct MarkDef {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<[f32; 3]>,
    #[serde(default)]
    pub yaw: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patrol: Vec<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambush: Option<AmbushDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<Dir>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "for")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arena: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loot: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub levels: Vec<f32>,
    /// Steinen til den falske veggen (som muren den står i).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<super::stone::InstanceStone>,
}

/// Det som står i et nivå (`assets/layout/crypt_NN.json`): merkene og
/// lysene, som redigeringen endrer. Rommene, kolliderne og
/// navigasjonsnettet står i nivåfila fra `tools/crypt/build_level.py`, som
/// leser denne fila og bygger nettet rundt det som står her.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct CryptContents {
    #[serde(default)]
    pub marks: Vec<MarkDef>,
    #[serde(default)]
    pub lights: Vec<LightDef>,
}
