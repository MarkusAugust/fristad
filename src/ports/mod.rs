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

pub trait Renderer {
    fn render(&mut self, frame: &Frame);
    fn resize(&mut self, width: u32, height: u32);
    /// Intern oppløsning som andel av vinduet (0.4–1.0), for svake GPU-er.
    fn set_scale(&mut self, scale: f32);
    /// Kvalitet 0–1: færre prøver i skygger, SSAO og tåke på svake GPU-er.
    fn set_quality(&mut self, quality: f32);
    /// Redusert grafikk for svake maskiner: færre skygger, ingen SSAO eller
    /// glød, færre lys og grovere detalj.
    fn set_reduced(&mut self, reduced: bool);
    /// Trekanter tegnet i hovedpasset sist bilde.
    fn triangles(&self) -> u32;
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
}
