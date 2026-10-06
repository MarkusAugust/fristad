//! Portene: det kjernen trenger fra omverdenen, som traits. Adapterne i
//! `adapters/` implementerer dem.
//!
//! - `Renderer` tegner en `Frame`. Adapter: wgpu.
//! - `Physics` holder kolliderne, flytter spilleren og skyter stråler.
//!   Adapter: Rapier.
//! - `ModelSource` gir modellene byen ber om ved navn. Adapter: glTF.
//!
//! Den drivende siden (nettleser, eget vindu, skjermbilde) kaller
//! `app::App`, som eier kjernen og portene.

use crate::domain::city::Shape;
use crate::domain::model::Model;
use crate::domain::scene::Frame;
use glam::{Affine3A, Vec3};

pub trait Renderer {
    fn render(&mut self, frame: &Frame);
    fn resize(&mut self, width: u32, height: u32);
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
    /// Legg til en boks som kan flyttes etterpå, til props.
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
}
