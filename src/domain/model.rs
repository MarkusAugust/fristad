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

/// Et fotoskannet materiale som erstatter et bakt teksturlag.
pub struct LayerImages {
    pub diffuse: Image,
    pub normal: Option<Image>,
    pub rough: Option<Image>,
    pub height: Option<Image>,
    pub ao: Option<Image>,
    /// Hvor mange meter én flis dekker.
    pub size_m: f32,
}

/// Det byen trenger å vite om en modell før den bygges: at den finnes, og
/// hvor stor den er (føttene i y = 0, sentrert i x og z).
#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub name: String,
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}
