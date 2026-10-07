//! Montering: alt som må være klart før rendereren kan lages. Modellene
//! lastes gjennom `ModelSource`, verden bygges fra dem, og bare modellene
//! som faktisk står i verden blir med videre.

use crate::domain::model::{Model, ModelInfo};
use crate::domain::world::{self, World};
use crate::ports::ModelSource;

pub struct Prepared {
    pub world: World,
    pub models: Vec<Model>,
}

/// `seed` gir kjelleren: fast i tester og skjermbilder, fra klokka i nettleseren.
pub fn prepare(models: &dyn ModelSource, seed: u64) -> Prepared {
    let mut loaded: Vec<Model> = models.names().iter().filter_map(|n| models.load(n)).collect();
    // Gresstusten og ruskbitene lages i kjernen, ikke i Blender.
    loaded.push(world::nature::grass_model());
    loaded.push(crate::domain::combat::debris_model());
    let infos: Vec<ModelInfo> = loaded
        .iter()
        .map(|m| {
            let (min, max) = m.mesh.bounds();
            ModelInfo { name: m.name.clone(), min, max }
        })
        .collect();
    let world = world::build(&infos, seed);
    log::info!("Verden: {} hjørner, {} modeller tilgjengelig", world.mesh.vertices.len(), infos.len());
    let models: Vec<Model> = loaded.into_iter().filter(|m| world.placements.iter().any(|p| p.model == m.name)).collect();
    Prepared { world, models }
}

/// Skjelettene til de skinnede modellene, til økta.
pub fn rigs(models: &[Model]) -> Vec<(String, crate::domain::anim::Rig)> {
    models.iter().filter_map(|m| m.rig.clone().map(|r| (m.name.clone(), r))).collect()
}
