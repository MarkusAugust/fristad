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
    /// Skjelettene til de skinnede modellene, også dem som hentes senere
    /// (fra manifestet), til økta.
    pub rigs: Vec<(String, crate::domain::anim::Rig)>,
    /// Modellene som står i verden, men hentes senere (nettleseren): bitene
    /// til kryptnivåene og de andre (`App` ber om dem).
    pub later: Vec<String>,
    /// Det redigeringen trenger (bare med `prepare_for_edit`).
    pub edit: Option<EditData>,
    /// Nivåredigeringen (bare med `prepare_level_editor`).
    pub level: Option<super::level_editing::LevelSetup>,
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
    assemble(models, seed, data, edit, None)
}

/// Nivåredigeringen (`?nivaa`): byen med dokumentet i `editor` som eneste
/// kryptnivå (nivå 1, så prøveturen kan gå rett ned), og plass til å vise
/// nivået slik det endres (`level_editing::level_reserves`). Alle modellene
/// lastes.
pub fn prepare_level_editor(models: &dyn ModelSource, seed: u64, editor: crate::domain::edit::level::LevelEditor) -> Prepared {
    let mut data = models.world_data();
    data.levels.clear();
    data.docs = vec![(1, editor.doc().clone())];
    data.kit = editor.kit().clone();
    assemble(models, seed, data, false, Some(editor))
}

fn assemble(models: &dyn ModelSource, seed: u64, mut data: Data, edit: bool, level: Option<crate::domain::edit::level::LevelEditor>) -> Prepared {
    // Nivådokumentene kompileres før noe annet, så redigeringen ser de samme
    // nivåene som spillet.
    data.compose_levels();
    let mut loaded: Vec<Model> = models.names().iter().filter_map(|n| models.load(n)).collect();
    // Gresstusten og ruskbitene lages i kjernen, ikke i Blender.
    loaded.push(crate::domain::combat::debris_model());
    let mut infos: Vec<ModelInfo> = loaded
        .iter()
        .map(|m| {
            let (min, max) = m.mesh.bounds();
            ModelInfo { name: m.name.clone(), min, max, solids: m.solids.clone() }
        })
        .collect();
    let mut rigs = rigs(&loaded);
    // Modellene som kommer senere (nettleseren) settes ut som de andre,
    // etter målene, kolliderne og skjelettet fra manifestet (bitene til
    // kryptnivåene har dem i nivåfila).
    let mut later = vec![];
    for d in models.deferred() {
        if infos.iter().any(|i| i.name == d.info.name) {
            continue;
        }
        later.push(d.info.name.clone());
        if let Some(r) = d.rig {
            rigs.push((d.info.name.clone(), r));
        }
        infos.push(d.info);
    }
    let kept = edit.then(|| {
        (
            Sources { models: infos.clone(), insides: data.insides.clone(), levels: data.levels.clone() },
            data.layout.clone().unwrap_or_default(),
            data.contents.iter().cloned().collect::<HashMap<_, _>>(),
            data.approved.clone(),
        )
    });
    let anchor = data.layout.as_ref().map(|l| l.crypt_anchor).unwrap_or_default();
    let mut world = world::build(&infos, seed, data);
    // Nivåredigeringen viser nivået som bevegelige instanser: plass til dem.
    let level = level.map(|editor| {
        for (name, n) in super::level_editing::level_reserves(editor.kit(), &infos) {
            let name = crate::domain::world::layout::intern(&name);
            match world.placements.iter_mut().find(|p| p.model == name) {
                Some(p) => p.reserve = p.reserve.max(n),
                None => world.placements.push(Placement { model: name, transforms: vec![], tints: vec![], looks: vec![], reserve: n, indoor: false }),
            }
        }
        super::level_editing::LevelSetup { editor, anchor }
    });
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
    later.retain(|n| world.placements.iter().any(|p| p.model == n));
    Prepared { world, models, infos, rigs, later, edit, level }
}

/// Skjelettene til de skinnede modellene, til økta.
pub fn rigs(models: &[Model]) -> Vec<(String, crate::domain::anim::Rig)> {
    models.iter().filter_map(|m| m.rig.clone().map(|r| (m.name.clone(), r))).collect()
}

/// Slår sammen instansene av modellene `names` (bitene fra byggesettet) til
/// én modell per rute på `cell` meter (og 20 m i høyden), som rendereren
/// tegner med ett kall per rute i stedet for ett per modell (målingen i
/// fase 0, `docs/nivaaredigering.md`; i spillet gjøres det ved lasting i
/// fase 7). Bare ekte modeller settes sammen: hjørnene flyttes, ingenting
/// lages. Gir antallet ruter.
pub fn merge_instances(p: &mut Prepared, names: &[&str], cell: f32) -> usize {
    use crate::domain::geometry::Mesh;
    let mut chunks: BTreeMap<(i32, i32, i32), Mesh> = BTreeMap::new();
    for pl in p.world.placements.iter_mut().filter(|pl| names.contains(&pl.model)) {
        let Some(model) = p.models.iter().find(|m| m.name == pl.model) else { continue };
        if model.albedo.is_some() || model.rig.is_some() {
            continue; // egne teksturer eller skjelett: kan ikke deles
        }
        let looks: Vec<_> = pl.looks.iter().copied().chain(std::iter::repeat(None)).take(pl.transforms.len()).collect();
        for (t, look) in pl.transforms.drain(..).zip(looks) {
            let o = glam::Vec3::from(t.translation);
            let key = ((o.x / cell).floor() as i32, (o.y / 20.0).floor() as i32, (o.z / cell).floor() as i32);
            let mesh = chunks.entry(key).or_insert_with(|| Mesh { vertices: vec![], indices: vec![] });
            let base = mesh.vertices.len() as u32;
            let flip = t.matrix3.determinant() < 0.0;
            for v in &model.mesh.vertices {
                let mut w = *v;
                // Steinbyttet bakes inn i hjørnene (som `look` i world.wgsl).
                if let Some(l) = &look {
                    w.mat[0] = l.layer(v.mat[0], v.normal[1]);
                }
                w.pos = t.transform_point3(glam::Vec3::from(v.pos)).to_array();
                w.normal = (t.matrix3 * glam::Vec3A::from(glam::Vec3::from(v.normal))).normalize_or_zero().to_array();
                mesh.vertices.push(w);
            }
            for tri in model.mesh.indices.chunks_exact(3) {
                if flip {
                    mesh.indices.extend([base + tri[0], base + tri[2], base + tri[1]]);
                } else {
                    mesh.indices.extend(tri.iter().map(|i| base + i));
                }
            }
        }
        pl.tints.clear();
        pl.looks.clear();
    }
    let n = chunks.len();
    for ((x, y, z), mesh) in chunks {
        let name = crate::domain::world::layout::intern(&format!("merged_{x}_{y}_{z}"));
        let model = Model {
            name: name.to_string(),
            mesh,
            albedo: None,
            normal: None,
            metal_rough: None,
            emissive: None,
            roughness: 0.5,
            metallic: 0.0,
            far_cell: 0.0,
            far_mesh: None,
            rig: None,
            solids: vec![],
        };
        p.models.push(model);
        p.world.placements.push(Placement { model: name, transforms: vec![Affine3A::IDENTITY], tints: vec![], looks: vec![], reserve: 0, indoor: false });
    }
    n
}
