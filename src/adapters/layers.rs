//! Teksturlagene: de håndlagde, flisbare materialene fra
//! `tools/textures/layers.py` (bakt i Blender), bygd inn i binæren og
//! dekodet her. Rendereren legger dem over de prosedyriske lagene fra
//! `bake.wgsl`.

use crate::domain::geometry::Layer;
use crate::domain::model::{Image, LayerImages};

/// Lagene som er bygd inn: lag, meter per flis, og albedo og
/// normal/ruhet/høyde som PNG eller JPEG.
macro_rules! layers {
    ($($layer:ident: $size:literal => $name:literal),* $(,)?) => {
        const LAYERS: &[(Layer, f32, &[u8], &[u8], &[u8], &[u8])] = &[$(
            (Layer::$layer, $size,
             include_bytes!(concat!("../../assets/textures/", $name, "_albedo.jpg")),
             include_bytes!(concat!("../../assets/textures/", $name, "_normal.jpg")),
             include_bytes!(concat!("../../assets/textures/", $name, "_rough.jpg")),
             include_bytes!(concat!("../../assets/textures/", $name, "_height.jpg")))
        ),*];
    };
}

layers! {}

fn decode(bytes: &[u8]) -> Option<Image> {
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    Some(Image { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

/// Alle lagene som finnes.
pub fn all() -> Vec<(Layer, LayerImages)> {
    LAYERS
        .iter()
        .filter_map(|(layer, size, albedo, normal, rough, height)| {
            Some((*layer, LayerImages { diffuse: decode(albedo)?, normal: decode(normal), rough: decode(rough), height: decode(height), ao: None, size_m: *size }))
        })
        .collect()
}
