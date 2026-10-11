//! Portene: grensesnittene mellom applikasjonen og omverdenen.
//!
//! Drevne porter (applikasjonen kaller ut, adapterne implementerer):
//!
//! - `Renderer` tegner et `Frame`. Adapter: wgpu.
//! - `Physics` holder kolliderne, flytter spilleren og skyter stråler.
//!   Adapter: Rapier.
//! - `ModelSource` gir modellene ved navn. Adapter: glTF-filene fra
//!   `tools/models/` som er bygd inn i binæren.
//!
//! Den drivende porten (`driving`) er det nettleseren, vinduet og
//! skjermbildeverktøyet sender inn (`Input`, `Command`) og får tilbake
//! (`Event`). Applikasjonen (`application::App`) implementerer den.

pub mod driving;

use crate::domain::geometry::Mesh;
use crate::domain::model::Model;
use crate::domain::scene::Frame;
use crate::domain::world::Shape;
use glam::{Affine3A, Vec3};

/// Det rendereren får én gang: den faste geometrien og modellene som står
/// i verden, med instansene.
pub struct StaticScene<'a> {
    pub mesh: &'a Mesh,
    pub chunks: &'a [crate::domain::geometry::Chunk],
    pub placements: &'a [crate::domain::model::Placement],
    pub models: &'a [Model],
}

/// Grafikknivået spilleren har valgt. Byttes aldri av seg selv.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Quality {
    /// For de svakeste maskinene (og det WebGL2 får): uten MSAA, uten
    /// solskygge og glød, ett lag på bakken, fire lys og halv teksturtetthet.
    Simple,
    /// For en vanlig maskin: MSAA 4×, kortere sikt, åtte lys.
    #[default]
    Standard,
    /// For en god maskin: lengre sikt, alle lys og MSAA 8× der kortet kan.
    Advanced,
}

impl Quality {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "enkel" => Some(Quality::Simple),
            "standard" => Some(Quality::Standard),
            "avansert" => Some(Quality::Advanced),
            _ => None,
        }
    }
}

/// Brytere til målingen i nettleseren (`?bench`): slå av deler av bildet
/// én om gangen, så tiden per del kan skilles fra hverandre.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bench {
    /// MSAA-prøver i stedet for nivåets (0: nivåets).
    pub msaa: u32,
    /// Alpha to coverage på alfakortene.
    pub a2c: bool,
    /// Solskyggen.
    pub shadow: bool,
    /// Flat farge i stedet for skyggeleggingen (måler rasterisering og
    /// overtegning alene).
    pub flat: bool,
    /// HDR-bildet i 4 byte per prøve (`Rg11b10Ufloat`) i stedet for 8, der
    /// kortet kan (måler om MSAA-målet sprenger flisminnet).
    pub small_hdr: bool,
    /// Tidsmålingen på grafikkortet (måler hva den selv koster).
    pub timers: bool,
}

impl Default for Bench {
    fn default() -> Self {
        Bench { msaa: 0, a2c: true, shadow: true, flat: false, small_hdr: false, timers: true }
    }
}

pub trait Renderer {
    fn render(&mut self, frame: &Frame);
    fn resize(&mut self, width: u32, height: u32);
    fn set_quality(&mut self, quality: Quality);
    /// Bryterne til målingen; `Bench::default()` er vanlig spill.
    fn set_bench(&mut self, _bench: Bench) {}
    /// Trekanter tegnet i hovedpasset sist bilde.
    fn triangles(&self) -> u32;
    /// Millisekunder på grafikkortet per del av bildet, om det kan måles.
    fn gpu_times(&self) -> Vec<(&'static str, f32)> {
        vec![]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ColliderId(pub u64);

pub struct Hit {
    pub collider: ColliderId,
    pub distance: f32,
}

pub trait Physics {
    /// Legg til en fast form i verden.
    fn add(&mut self, shape: &Shape) -> ColliderId;
    /// Legg til en boks som kan flyttes etterpå (porter, ting du sikter på).
    fn add_movable(&mut self, half: Vec3, pose: Affine3A) -> ColliderId;
    fn set_pose(&mut self, id: ColliderId, pose: Affine3A);
    /// Flytt en kapsel (halv høyde `half`, radius `radius`) med `delta`,
    /// med kollisjon og trappesteg. Gir faktisk flytting og om den står på
    /// bakken.
    fn move_capsule(&mut self, pos: Vec3, half: f32, radius: f32, delta: Vec3, dt: f32) -> (Vec3, bool);
    fn cast_ray(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<Hit>;
}

pub trait ModelSource {
    fn load(&self, name: &str) -> Option<Model>;
    /// Navnene på modellene som finnes.
    fn names(&self) -> Vec<String>;
    /// Det husene har med seg og kryptnivåene, ferdig lest.
    fn world_data(&self) -> crate::domain::world::Data;
}
