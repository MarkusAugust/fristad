//! Ressurs-adapteren: filene `tools/fetch-assets.ts` har hentet fra
//! Poly Haven (CC0) til `assets/external/`, lest til minne, bak portene
//! `ModelSource` og `TextureSource`. De innebygde modellene (lykta og reven)
//! er reserven når ingenting er hentet.
//!
//! Native leser katalogen fra disk, nettleseren henter filene med `fetch`
//! og gir dem hit. Begge ender i `MemoryAssets`.

use crate::adapters::gltf::{self, Embedded};
use crate::domain::model::{Image, LayerImages, Model};
use crate::domain::ports::{ModelSource, TextureSource};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default)]
pub struct Manifest {
    #[serde(default)]
    pub textures: HashMap<String, TextureEntry>,
    #[serde(default)]
    pub models: HashMap<String, ModelEntry>,
}

#[derive(Deserialize, Default, Clone)]
pub struct TextureEntry {
    pub id: Option<String>,
    #[serde(default = "two")]
    pub size_m: f32,
    pub diffuse: Option<String>,
    pub normal: Option<String>,
    pub rough: Option<String>,
    pub height: Option<String>,
    pub ao: Option<String>,
}

fn two() -> f32 {
    2.0
}

#[derive(Deserialize, Default, Clone)]
pub struct ModelEntry {
    pub id: Option<String>,
    #[serde(default)]
    pub height: f32,
    #[serde(default)]
    pub yaw: f32,
    pub gltf: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
}

/// Alle filene som trengs, lest til minne. Stiene er relative til
/// `assets/external/`, med skråstrek.
pub struct MemoryAssets {
    manifest: Manifest,
    files: HashMap<String, Vec<u8>>,
    embedded: Embedded,
}

impl MemoryAssets {
    pub fn empty() -> Self {
        Self { manifest: Manifest::default(), files: HashMap::new(), embedded: Embedded }
    }

    pub fn from_files(manifest_json: &str, files: Vec<(String, Vec<u8>)>) -> Self {
        let manifest: Manifest = serde_json::from_str(manifest_json).unwrap_or_else(|e| {
            log::warn!("manifest.json er ugyldig: {e}");
            Manifest::default()
        });
        Self { manifest, files: files.into_iter().collect(), embedded: Embedded }
    }

    /// Leser `assets/external/` fra disk, om den finnes.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_dir(dir: &std::path::Path) -> Self {
        let manifest_path = dir.join("manifest.json");
        let Ok(json) = std::fs::read_to_string(&manifest_path) else {
            return Self::empty();
        };
        let manifest: Manifest = match serde_json::from_str(&json) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("{}: {e}", manifest_path.display());
                return Self::empty();
            }
        };
        let mut files = HashMap::new();
        let mut read = |rel: &str| {
            if let Ok(bytes) = std::fs::read(dir.join(rel)) {
                files.insert(rel.to_string(), bytes);
            } else {
                log::warn!("Mangler {rel} i {}", dir.display());
            }
        };
        for t in manifest.textures.values() {
            for f in [&t.diffuse, &t.normal, &t.rough, &t.height, &t.ao].into_iter().flatten() {
                read(f);
            }
        }
        for m in manifest.models.values() {
            if let Some(g) = &m.gltf {
                read(g);
                let base = g.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
                for f in &m.files {
                    read(&format!("{base}/{f}"));
                }
            }
        }
        log::info!("Eksterne ressurser: {} teksturer, {} modeller, {} filer", manifest.textures.len(), manifest.models.len(), files.len());
        Self { manifest, files, embedded: Embedded }
    }

    /// Alle filstiene manifestet viser til, til nettleseren som må hente dem.
    pub fn paths_in(manifest_json: &str) -> Vec<String> {
        let Ok(m) = serde_json::from_str::<Manifest>(manifest_json) else { return vec![] };
        let mut out = vec![];
        for t in m.textures.values() {
            for f in [&t.diffuse, &t.normal, &t.rough, &t.height, &t.ao].into_iter().flatten() {
                out.push(f.clone());
            }
        }
        for md in m.models.values() {
            if let Some(g) = &md.gltf {
                out.push(g.clone());
                let base = g.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
                for f in &md.files {
                    out.push(format!("{base}/{f}"));
                }
            }
        }
        out
    }

    fn image(&self, path: &Option<String>) -> Option<Image> {
        let p = path.as_ref()?;
        let bytes = self.files.get(p)?;
        let img = image::load_from_memory(bytes).ok()?.into_rgba8();
        Some(Image { width: img.width(), height: img.height(), rgba: img.into_raw() })
    }
}

impl ModelSource for MemoryAssets {
    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.manifest.models.keys().filter(|n| self.manifest.models[*n].gltf.is_some()).cloned().collect();
        for n in self.embedded.names() {
            if !names.contains(&n) {
                names.push(n);
            }
        }
        names
    }

    fn load(&self, name: &str) -> Option<Model> {
        if let Some(entry) = self.manifest.models.get(name) {
            if let Some(g) = &entry.gltf {
                let base = g.rsplit_once('/').map(|(d, _)| d).unwrap_or("").to_string();
                let main = g.rsplit('/').next().unwrap_or(g).to_string();
                let mut files: Vec<(String, &[u8])> = vec![];
                if let Some(b) = self.files.get(g) {
                    files.push((main.clone(), b.as_slice()));
                }
                for f in &entry.files {
                    if let Some(b) = self.files.get(&format!("{base}/{f}")) {
                        files.push((f.clone(), b.as_slice()));
                    }
                }
                if !files.is_empty() {
                    let refs: Vec<(&str, &[u8])> = files.iter().map(|(n, b)| (n.as_str(), *b)).collect();
                    return gltf::load(name, &refs, entry.height, entry.yaw).map_err(|e| log::warn!("Modell {name}: {e}")).ok();
                }
            }
        }
        self.embedded.load(name)
    }
}

impl TextureSource for MemoryAssets {
    fn layer(&self, name: &str) -> Option<LayerImages> {
        let t = self.manifest.textures.get(name)?;
        let diffuse = self.image(&t.diffuse)?;
        Some(LayerImages {
            diffuse,
            normal: self.image(&t.normal),
            rough: self.image(&t.rough),
            height: self.image(&t.height),
            ao: self.image(&t.ao),
            size_m: t.size_m,
        })
    }
}
