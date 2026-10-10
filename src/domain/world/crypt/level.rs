//! `crypt_NN.json` slik Blender skriver den (se `docs/krypten/format.md`).
//! Lesingen er raus: ukjente felt og merker hoppes over, og det som mangler
//! får en standardverdi, så et nivå under arbeid fortsatt kan lastes.

use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
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
}

#[derive(Deserialize, Default, Debug)]
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
    #[serde(default)]
    pub secret: bool,
}

fn default_h() -> f32 {
    4.0
}

#[derive(Deserialize, Debug)]
pub struct BoxDef {
    pub center: [f32; 3],
    pub size: [f32; 3],
    #[serde(default)]
    pub yaw: f32,
}

#[derive(Deserialize, Default, Debug)]
pub struct ColliderDef {
    #[serde(rename = "box")]
    pub cuboid: Option<BoxDef>,
    pub hull: Option<Vec<[f32; 3]>>,
}

#[derive(Deserialize, Default, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct LightDef {
    pub pos: [f32; 3],
    #[serde(default = "default_color")]
    pub color: [f32; 3],
    #[serde(default = "default_range")]
    pub range: f32,
    #[serde(default)]
    pub flicker: bool,
    /// Styrken (ikke i formatet ennå; 2 om den mangler).
    pub intensity: Option<f32>,
    pub room: Option<String>,
}

fn default_color() -> [f32; 3] {
    [1.0, 0.62, 0.3]
}

fn default_range() -> f32 {
    8.0
}

/// Et tall, eller to eller tre (mål).
#[derive(Deserialize, Debug, Clone, Copy)]
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
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum Dir {
    Yaw(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
}

/// Bakholdet: på (vekkes når du kommer innen 4 m), en radius, eller
/// rom-id-en der du må gå inn.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AmbushDef {
    On(bool),
    Radius(f32),
    Room(String),
}

#[derive(Deserialize, Default, Debug)]
pub struct MarkDef {
    pub kind: String,
    pub pos: Option<[f32; 3]>,
    #[serde(default)]
    pub yaw: f32,
    pub id: Option<String>,
    pub label: Option<String>,
    pub n: Option<u32>,
    #[serde(default)]
    pub patrol: Vec<[f32; 3]>,
    pub ambush: Option<AmbushDef>,
    #[serde(default)]
    pub opens: Vec<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub from: Option<String>,
    pub size: Option<Size>,
    pub to: Option<String>,
    pub top: Option<[f32; 3]>,
    pub bottom: Option<[f32; 3]>,
    pub dir: Option<Dir>,
    pub text: Option<String>,
    #[serde(rename = "for")]
    pub for_id: Option<String>,
    pub arena: Option<String>,
    pub loot: Option<String>,
    #[serde(default)]
    pub levels: Vec<f32>,
}
