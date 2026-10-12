//! Montering: alt som må være klart før rendereren kan lages. Modellene
//! lastes gjennom `ModelSource`, verden bygges fra dem, og bare modellene
//! som faktisk står i verden blir med videre (i redigeringen alle, med plass
//! til å flytte dem).

use crate::domain::edit::Sources;
use crate::domain::model::{Model, ModelInfo, Placement};
use crate::domain::world::crypt::level::CryptContents;
use crate::domain::world::layout::{Item, Layout, Ref};
use crate::domain::world::{self, Data, World};
use crate::ports::ModelSource;
use glam::Affine3A;
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct Prepared {
    pub world: World,
    pub models: Vec<Model>,
    /// Målene til alle modellene som finnes, også de som ikke står ute (til
    /// redigeringen, som kan sette dem ut).
    pub infos: Vec<ModelInfo>,
    /// Det redigeringen trenger (bare med `prepare_for_edit`).
    pub edit: Option<EditData>,
}

/// Dataene byen ble bygget fra, og instansene den satte ut, til
/// redigeringen.
#[derive(Clone)]
pub struct EditData {
    pub sources: Sources,
    pub layout: Layout,
    pub contents: HashMap<String, Vec<Item>>,
    pub approved: Option<HashSet<String>>,
    /// De faste instansene fra byen (modell, nummer, pose), så det som er
    /// flyttet kan skjules der det sto.
    pub base: Vec<(&'static str, usize, Affine3A, Ref)>,
    /// Dataene i fila, når spillet er bygget på nytt fra redigeringen (P):
    /// det som er endret siden, må fortsatt være godkjent.
    pub baseline: Option<(Layout, HashMap<String, Vec<Item>>, BTreeMap<u32, CryptContents>)>,
}

/// Hvor mange instanser av hver modell redigeringen kan vise på nye steder.
pub const EDIT_RESERVE: usize = 64;

/// `seed` gir kjelleren: fast i tester og skjermbilder, fra klokka i nettleseren.
pub fn prepare(models: &dyn ModelSource, seed: u64) -> Prepared {
    prepare_with(models, seed, models.world_data(), false)
}

/// Som `prepare`, med alt redigeringen trenger: alle modellene lastes, og
/// hver får plass til å vises på nye steder.
pub fn prepare_for_edit(models: &dyn ModelSource, seed: u64) -> Prepared {
    prepare_with(models, seed, models.world_data(), true)
}

/// Bygg spillet fra `data` (dataene fra fila, eller fra redigeringen når du
/// trykker P for å spille).
pub fn prepare_with(models: &dyn ModelSource, seed: u64, data: Data, edit: bool) -> Prepared {
    let mut loaded: Vec<Model> = models.names().iter().filter_map(|n| models.load(n)).collect();
    // Gresstusten og ruskbitene lages i kjernen, ikke i Blender.
    loaded.push(crate::domain::combat::debris_model());
    let infos: Vec<ModelInfo> = loaded
        .iter()
        .map(|m| {
            let (min, max) = m.mesh.bounds();
            ModelInfo { name: m.name.clone(), min, max }
        })
        .collect();
    let kept = edit.then(|| {
        (
            Sources { models: infos.clone(), insides: data.insides.clone(), levels: data.levels.clone() },
            data.layout.clone().unwrap_or_default(),
            data.contents.iter().cloned().collect::<HashMap<_, _>>(),
            data.approved.clone(),
        )
    });
    let mut world = world::build(&infos, seed, data);
    log::info!("Verden: {} hjørner, {} modeller tilgjengelig", world.mesh.vertices.len(), infos.len());
    let edit = kept.map(|(sources, layout, contents, approved)| {
        let base = world.origins.iter().filter_map(|(m, n, r)| world.placements.iter().find(|p| p.model == *m).and_then(|p| p.transforms.get(*n)).map(|t| (*m, *n, *t, r.clone()))).collect();
        // Plass til å vise hver modell på nye steder.
        for info in &infos {
            let name = crate::domain::world::layout::intern(&info.name);
            match world.placements.iter_mut().find(|p| p.model == name) {
                Some(p) => p.reserve += EDIT_RESERVE,
                None => world.placements.push(Placement { model: name, transforms: vec![], tints: vec![], looks: vec![], reserve: EDIT_RESERVE, indoor: false }),
            }
        }
        EditData { sources, layout, contents, approved, base, baseline: None }
    });
    let models: Vec<Model> = loaded.into_iter().filter(|m| world.placements.iter().any(|p| p.model == m.name)).collect();
    Prepared { world, models, infos, edit }
}

/// Skjelettene til de skinnede modellene, til økta.
pub fn rigs(models: &[Model]) -> Vec<(String, crate::domain::anim::Rig)> {
    models.iter().filter_map(|m| m.rig.clone().map(|r| (m.name.clone(), r))).collect()
}
