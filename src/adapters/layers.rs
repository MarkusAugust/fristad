//! Teksturlagene: bildene `tools/fetch-assets.ts` har samlet i
//! `assets/external/` med `manifest.json` (eierens egne bilder og
//! fotoskannede CC0-materialer fra Poly Haven). Rendereren legger dem over
//! de prosedyriske lagene fra `bake.wgsl`; mangler et lag, brukes det bakte.
//!
//! Native leser katalogen fra disk, nettleseren henter filene med `fetch`
//! og gir dem hit. Begge ender i `decode`.

use crate::domain::geometry::Layer;
use crate::domain::model::{Image, LayerImages};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default)]
struct Manifest {
    #[serde(default)]
    textures: HashMap<String, Entry>,
}

#[derive(Deserialize, Default, Clone)]
struct Entry {
    #[serde(default = "two")]
    size_m: f32,
    diffuse: Option<String>,
    normal: Option<String>,
    rough: Option<String>,
    height: Option<String>,
    ao: Option<String>,
}

fn two() -> f32 {
    2.0
}

impl Entry {
    fn files(&self) -> impl Iterator<Item = &String> {
        [&self.diffuse, &self.normal, &self.rough, &self.height, &self.ao].into_iter().flatten()
    }
}

/// Filstiene manifestet viser til, relative til katalogen manifestet ligger i.
pub fn paths_in(manifest_json: &str) -> Vec<String> {
    let m: Manifest = serde_json::from_str(manifest_json).unwrap_or_default();
    m.textures.values().flat_map(|e| e.files().cloned().collect::<Vec<_>>()).collect()
}

/// Lagene fra et manifest og filene det viser til.
pub fn from_files(manifest_json: &str, files: &HashMap<String, Vec<u8>>) -> Vec<(Layer, LayerImages)> {
    let m: Manifest = match serde_json::from_str(manifest_json) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("manifest.json er ugyldig: {e}");
            return vec![];
        }
    };
    let image = |p: &Option<String>| -> Option<Image> {
        let bytes = files.get(p.as_ref()?)?;
        let img = image::load_from_memory(bytes).ok()?.into_rgba8();
        Some(Image { width: img.width(), height: img.height(), rgba: img.into_raw() })
    };
    let mut out = vec![];
    for layer in Layer::ALL {
        let Some(e) = m.textures.get(layer.name()) else { continue };
        let Some(diffuse) = image(&e.diffuse) else { continue };
        out.push((layer, LayerImages { diffuse, normal: image(&e.normal), rough: image(&e.rough), height: image(&e.height), ao: image(&e.ao), size_m: e.size_m }));
    }
    log::info!("Teksturlag: {}", out.iter().map(|(l, _)| l.name()).collect::<Vec<_>>().join(", "));
    out
}

/// Lagene i en katalog (`assets/external`), om den finnes.
#[cfg(not(target_arch = "wasm32"))]
pub fn from_dir(dir: &std::path::Path) -> Vec<(Layer, LayerImages)> {
    let Ok(json) = std::fs::read_to_string(dir.join("manifest.json")) else {
        log::info!("Ingen teksturlag i {}; bruker de bakte", dir.display());
        return vec![];
    };
    let files: HashMap<String, Vec<u8>> = paths_in(&json).into_iter().filter_map(|p| std::fs::read(dir.join(&p)).ok().map(|b| (p, b))).collect();
    from_files(&json, &files)
}
