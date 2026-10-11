//! En modell med egne teksturer, slik kjernen ser den. Hvor den kommer fra
//! (glTF) er adapterens sak.

use crate::domain::geometry::{Layer, Mesh};

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
    /// Cellestørrelsen for det grove detaljnivået (meter); 0 lar rendereren
    /// velge etter modellens størrelse.
    pub far_cell: f32,
    /// Et ferdig grovt detaljnivå (bladverk som tynnede kort); ellers
    /// lager rendereren det med `Mesh::clustered`.
    pub far_mesh: Option<Mesh>,
    /// Skjelettet og klippene, for modeller som er skinnet (figurene).
    pub rig: Option<crate::domain::anim::Rig>,
}

/// Hvor en modell står i byen, med alle instansene.
pub struct Placement {
    pub model: &'static str,
    pub transforms: Vec<glam::Affine3A>,
    /// Lakkfargen per instans (lineær RGB), for flater med flagget PAINT.
    pub tints: Vec<glam::Vec3>,
    /// Et annet utseende per instans (kjelleretasjene), eller ingen.
    pub looks: Vec<Option<Look>>,
    /// Plass til instanser spillet poserer per bilde (`Frame.instances`).
    pub reserve: usize,
}

/// Et annet utseende på samme modell: kjellerens modulsett bygges med fire
/// roller (`Look::ROLES`: mur, hvelv, kvaderstein og gulv), som hver etasje
/// gir sine egne teksturlag. Tonen (lineær) ganges på rollene, og flammene
/// (materialer som heter `fire…`) får fargen `flame`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub layers: [Layer; 4],
    pub tint: glam::Vec3,
    pub flame: glam::Vec3,
}

impl Look {
    /// Lagene modulsettet er bygd med, som rollene byttes ut fra.
    pub const ROLES: [Layer; 4] = [Layer::Stone, Layer::Ashlar, Layer::Flagstone, Layer::Tiles];
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
