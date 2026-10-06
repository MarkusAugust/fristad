//! En modell med egne teksturer, slik kjernen ser den. Hvor den kommer fra
//! (glTF) er adapterens sak.

use crate::domain::geometry::Mesh;

pub struct Image {
    pub width: u32,
    pub height: u32,
    /// RGBA8.
    pub rgba: Vec<u8>,
}

pub struct Model {
    pub name: String,
    pub mesh: Mesh,
    pub albedo: Option<Image>,
    pub normal: Option<Image>,
    /// Ruhet i G, metall i B, som i glTF.
    pub metal_rough: Option<Image>,
    pub emissive: Option<Image>,
    pub roughness: f32,
    pub metallic: f32,
}

/// Hvor en modell står i byen, med alle instansene.
pub struct Placement {
    pub model: &'static str,
    pub transforms: Vec<glam::Affine3A>,
}
