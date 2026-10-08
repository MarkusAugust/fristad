//! Verden: kjøpstaden Fristad rundt 1350, borgen på haugen, landskapet
//! rundt og kjelleren under. Alt bygges her fra modellene som finnes
//! (`ModelInfo`), uten å vite hvor de kommer fra.
//!
//! Koordinater: X øst, Y opp, Z sør. Torget ligger rundt origo, sørporten i
//! sør og borgen på haugen i nord.

use crate::domain::geometry::{Chunk, Layer, Mat, Mesh};
use crate::domain::model::{Look, ModelInfo, Placement};
use crate::domain::rng::Rng;
use crate::domain::scene::Light;
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::collections::HashMap;

pub mod dungeon;
pub mod nature;
pub mod terrain;
pub mod town;

#[derive(Clone, Debug)]
pub enum Shape {
    /// Boks med sentrum, hele mål og rotasjon om Y.
    Cuboid { center: Vec3, size: Vec3, yaw: f32 },
    /// Stående sylinder med sentrum, radius og halv høyde.
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
    /// Terrenget: et rutenett av høyder, `cols` langs X og `rows` langs Z,
    /// sentrert i `center` og `size` meter stort.
    Heightfield { center: Vec3, size: Vec3, cols: usize, rows: usize, heights: Vec<f32> },
    /// Kapsel mellom to punkter (trestammer og greiner).
    Capsule { a: Vec3, b: Vec3, radius: f32 },
    /// Konveks omhylling av punktene (trappetrinn, fjell).
    Hull { points: Vec<Vec3> },
}

/// Hva F gjør med noe du sikter på.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Snakk med innbygger nummer n (E åpner butikken eller verkstedet).
    Talk(usize),
    /// Åpne kiste nummer n i kjelleren (`Dungeon.chests`).
    Chest(usize),
    /// Lås opp port nummer n i kjelleren (`Dungeon.gates`).
    Gate(usize),
    /// Døra i krypten under borgen: ned i kjelleren.
    Descend,
    /// Døra øverst i trappa i startrommet: opp til krypten.
    Ascend,
    /// Brønn nummer n i byen: heis opp vann og drikk.
    Well(usize),
}

/// Noe du kan sikte på og bruke: en boks (sentrum og halve mål) med pose.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub what: Use,
    pub pose: Affine3A,
    pub center: Vec3,
    pub half: Vec3,
}

/// En innbygger slik byen beskriver den for spillet.
#[derive(Clone, Debug)]
pub struct NpcSpec {
    pub name: &'static str,
    /// Figuren: en egen modell (f.eks. `npc_smith`) eller `npcs::NPC_MODEL`.
    pub model: &'static str,
    pub pos: Vec3,
    pub yaw: f32,
    pub tint: Vec3,
    pub lines: &'static [&'static str],
    /// Punktene innbyggeren går mellom (føttene på bakken) og hvor lenge
    /// hen blir stående ved hvert (0: går rett forbi, langs gata).
    pub route: Vec<(Vec3, f32)>,
}

pub struct World {
    /// Den faste geometrien, stokket i biter (`chunks`).
    pub mesh: Mesh,
    pub chunks: Vec<Chunk>,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    /// Pipene som ryker: toppen av pipa og hvor tykk røyken er (0..1).
    pub smoke: Vec<(Vec3, f32)>,
    pub placements: Vec<Placement>,
    pub targets: Vec<Target>,
    pub spawn: Vec3,
    pub spawn_yaw: f32,
    /// Det `Builder::audit` fant: modeller som svever eller synker.
    pub audit: Vec<String>,
    pub dungeon: dungeon::Dungeon,
    pub npcs: Vec<NpcSpec>,
    /// Hvor du står når du kommer opp fra kjelleren (føttene, retningen).
    pub crypt: (Vec3, f32),
}

pub struct Builder {
    pub mesh: Mesh,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    pub smoke: Vec<(Vec3, f32)>,
    pub placements: Vec<Placement>,
    pub targets: Vec<Target>,
    pub rng: Rng,
    /// Modellene som finnes, med mål.
    pub models: HashMap<String, ModelInfo>,
    /// Gulv over terrenget, til `audit` (`floor`).
    pub floors: Vec<(Vec3, Vec2, f32, f32)>,
    /// Instanser som henger på vegger eller står oppå noe (`place_mounted`).
    pub mounted: Vec<(&'static str, usize)>,
    pub npcs: Vec<NpcSpec>,
    /// Modeller byen ville bruke, men som mangler (til loggen og testene).
    pub missing: Vec<&'static str>,
}

impl Builder {
    pub fn new(models: &[ModelInfo]) -> Self {
        Self {
            mesh: Mesh::default(),
            colliders: vec![],
            lights: vec![],
            smoke: vec![],
            placements: vec![],
            targets: vec![],
            rng: Rng(0x5eed_f215_7a1d),
            models: models.iter().map(|m| (m.name.clone(), m.clone())).collect(),
            floors: vec![],
            mounted: vec![],
            npcs: vec![],
            missing: vec![],
        }
    }

    pub fn light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: false, flicker: 0.0 });
    }

    /// En pipe som ryker (toppen av pipa, tykkelse 0..1).
    pub fn smoke(&mut self, top: Vec3, strength: f32) {
        self.smoke.push((top, strength));
    }

    /// En lykt med et talglys: tennes i skumringen og blafrer svakt.
    pub fn lamp_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: false, flicker: 0.12 });
    }

    /// Et lys som brenner hele døgnet (ild), med et lite flakk.
    pub fn fire_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: true, flicker: 0.18 });
    }

    /// Et lys som brenner hele døgnet og flakker så mye du vil.
    pub fn flicker_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32, flicker: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: true, flicker });
    }

    /// Noe du kan bruke med F.
    pub fn target(&mut self, what: Use, pose: Affine3A, center: Vec3, half: Vec3) {
        self.targets.push(Target { what, pose, center, half });
    }

    /// Et gulv over terrenget: `audit` måler mot det i stedet for bakken
    /// innenfor rektangelet. `center` er midten, `half` halve målene i x og
    /// z, `top` gulvets overkant (y i verden).
    pub fn floor(&mut self, center: Vec3, half: Vec2, yaw: f32, top: f32) {
        self.floors.push((center, half, yaw, top));
    }

    /// Høyden på det noe kan stå på i et punkt: det registrerte gulvet eller
    /// terrenget som ligger nærmest under `y`.
    pub fn stand_height(&self, x: f32, z: f32, y: f32) -> f32 {
        let mut best = terrain::height(x, z);
        let score = |top: f32| if top <= y + 0.3 { y - top } else { 1e6 + top - y };
        for (c, half, yaw, top) in &self.floors {
            let (s, co) = yaw.sin_cos();
            let (dx, dz) = (x - c.x, z - c.z);
            let lx = dx * co - dz * s;
            let lz = dx * s + dz * co;
            if lx.abs() <= half.x && lz.abs() <= half.y && score(*top) < score(best) {
                best = *top;
            }
        }
        best
    }

    pub fn solid(&mut self, shape: Shape) {
        self.colliders.push(shape);
    }

    pub fn solid_box(&mut self, center: Vec3, size: Vec3, yaw: f32) {
        self.colliders.push(Shape::Cuboid { center, size, yaw });
    }

    pub fn has_model(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }

    /// Målene til en modell (min, maks) i modellens rom, om den finnes.
    pub fn bounds(&self, name: &str) -> Option<(Vec3, Vec3)> {
        self.models.get(name).map(|m| (m.min, m.max))
    }

    pub fn model_height(&self, name: &str) -> f32 {
        self.models.get(name).map(|m| m.max.y - m.min.y).unwrap_or(0.0)
    }

    /// Sett ut en modell. Mangler den, meldes det (`missing`), og ingenting
    /// settes ut: alt som synes skal være en ekte modell.
    pub fn place(&mut self, model: &'static str, t: Affine3A) -> bool {
        self.place_tinted(model, t, Vec3::ONE)
    }

    /// Sett ut en modell med egen farge (lineær RGB) på flatene som er
    /// merket «paint» i modellen.
    pub fn place_tinted(&mut self, model: &'static str, t: Affine3A, tint: Vec3) -> bool {
        self.place_look(model, t, tint, None)
    }

    /// Sett ut en modell med et annet utseende (`Look`: kjelleretasjene).
    pub fn place_look(&mut self, model: &'static str, t: Affine3A, tint: Vec3, look: Option<Look>) -> bool {
        if !self.has_model(model) {
            if !self.missing.contains(&model) {
                self.missing.push(model);
            }
            return false;
        }
        if let Some(p) = self.placements.iter_mut().find(|p| p.model == model) {
            p.transforms.push(t);
            p.tints.push(tint);
            p.looks.push(look);
        } else {
            self.placements.push(Placement { model, transforms: vec![t], tints: vec![tint], looks: vec![look], reserve: 0 });
        }
        true
    }

    /// Sett ut en modell som henger på en vegg eller står oppå noe annet,
    /// så `audit` ikke måler den mot bakken.
    pub fn place_mounted(&mut self, model: &'static str, t: Affine3A) -> bool {
        self.place_mounted_look(model, t, None)
    }

    /// Som `place_mounted`, med et annet utseende.
    pub fn place_mounted_look(&mut self, model: &'static str, t: Affine3A, look: Option<Look>) -> bool {
        if !self.place_look(model, t, Vec3::ONE, look) {
            return false;
        }
        if let Some(p) = self.placements.iter().find(|p| p.model == model) {
            self.mounted.push((model, p.transforms.len() - 1));
        }
        true
    }

    /// Sett av plass til `n` instanser av en modell som spillet flytter per
    /// bilde (dyr, figurer, porter).
    pub fn reserve(&mut self, model: &'static str, n: usize) {
        if !self.has_model(model) {
            log::warn!("Modellen {model} mangler, ingen plass satt av");
            return;
        }
        if let Some(p) = self.placements.iter_mut().find(|p| p.model == model) {
            p.reserve += n;
        } else {
            self.placements.push(Placement { model, transforms: vec![], tints: vec![], looks: vec![], reserve: n });
        }
    }

    /// Bakkehøyden der noe skal stå.
    pub fn ground(&self, x: f32, z: f32) -> f32 {
        terrain::height(x, z)
    }

    /// Punktet løftet opp på bakken.
    pub fn on_ground(&self, p: Vec3) -> Vec3 {
        Vec3::new(p.x, p.y + terrain::height(p.x, p.z), p.z)
    }

    /// Noe som står på bakken og vippes etter hellingen (dyr, kjerrer).
    pub fn on_slope(&self, p: Vec3, yaw: f32) -> Affine3A {
        let n = terrain::normal(p.x, p.z);
        Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, n) * Quat::from_rotation_y(yaw), self.on_ground(p))
    }

    /// Rutinen mot svevende modeller: hver instans sjekkes mot bakken (eller
    /// det registrerte gulvet). Svever det laveste hjørnet mer enn 8 cm over,
    /// eller står et hjørne mer enn 30 cm under, meldes det.
    pub fn audit(&self) -> Vec<String> {
        let mut out = vec![];
        for pl in &self.placements {
            let Some(info) = self.models.get(pl.model) else { continue };
            for (n, t) in pl.transforms.iter().enumerate() {
                if self.mounted.contains(&(pl.model, n)) {
                    continue;
                }
                // Måles innen 60 cm fra origo: trær har krona langt utenfor stammen.
                let (hx, hz) = ((info.max.x - info.min.x).min(1.2) * 0.5, (info.max.z - info.min.z).min(1.2) * 0.5);
                let corners = [(-hx, -hz), (hx, -hz), (-hx, hz), (hx, hz)];
                // Trestammer står i skråninger og går ned i bakken på oversida,
                // som ekte trær, og murverket har fundament under bakken; de
                // sjekkes bare for sveving, og bakken letes etter opp til en
                // meter over foten (ellers vinner et gulv lenger nede, som
                // kjelleren under krypten).
                let rooted = pl.model.ends_with("_trunk") || town::FOUNDED.iter().any(|m| *m == pl.model);
                let reach = if rooted { 1.0 } else { 0.0 };
                let gaps = corners.map(|(x, z)| {
                    let w = t.transform_point3(Vec3::new(x, info.min.y, z));
                    w.y - self.stand_height(w.x, w.z, w.y + reach)
                });
                let lowest = gaps.iter().cloned().fold(f32::MAX, f32::min);
                let at = t.translation;
                if lowest > 0.08 {
                    out.push(format!("{} ved ({:.1}, {:.1}) svever {:.0} cm over bakken", pl.model, at.x, at.z, lowest * 100.0));
                } else if lowest < -0.3 && !rooted {
                    out.push(format!("{} ved ({:.1}, {:.1}) står {:.0} cm nede i bakken", pl.model, at.x, at.z, -lowest * 100.0));
                }
            }
        }
        out
    }
}

pub const IRON: Mat = Mat::tex(Layer::Metal, 0xffffff);
pub const STONE: Mat = Mat::tex(Layer::Stone, 0xffffff);
pub const OAK: Mat = Mat::tex(Layer::Oak, 0xffffff);

/// Bygg verden. `seed` gir kjelleren (samme frø, samme kjeller).
pub fn build(models: &[ModelInfo], seed: u64) -> World {
    let mut b = Builder::new(models);

    // Bakken: et rutenett som følger terrenget, med brostein på torget,
    // jordveier og gress.
    ground(&mut b);
    let town = town::build(&mut b);
    nature::meadow(&mut b);
    nature::grass(&mut b);
    nature::landscape(&mut b);
    nature::world_edge(&mut b);
    // Kjelleren under borgen. Et nytt frø gir nye etasjer.
    let dungeon = dungeon::build(&mut b, seed, town.crypt_anchor);
    for (model, n) in crate::domain::mobs::Mobs::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in crate::domain::combat::Combat::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in crate::domain::npcs::Npcs::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in crate::domain::herd::Herd::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in dungeon::reserve(&dungeon) {
        b.reserve(model, n);
    }

    let audit = b.audit();
    for line in &audit {
        log::warn!("{line}");
    }
    if !b.missing.is_empty() {
        log::warn!("Modeller byen ville bruke, men som mangler: {}", b.missing.join(", "));
    }

    // Biter på 24 m, med småting (under 18 cm) sist i hver bit.
    let (mesh, chunks) = b.mesh.chunked(24.0, 0.18);
    log::info!("Verden: {} biter, {} modeller i bruk", chunks.len(), b.placements.len());
    World {
        mesh,
        chunks,
        colliders: b.colliders,
        lights: b.lights,
        smoke: b.smoke,
        placements: b.placements,
        targets: b.targets,
        dungeon,
        npcs: b.npcs,
        spawn: town.spawn,
        spawn_yaw: town.spawn_yaw,
        crypt: town.crypt_exit,
        audit,
    }
}


pub fn srgb_to_linear(c: u32) -> Vec3 {
    let f = |v: u32| {
        let c = v as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    Vec3::new(f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255))
}

/// Bakken: et rutenett på 2 m som deler hjørnene, så vektene mellom
/// lagene glir jevnt over (`town::ground_weights`), og heightfield-kollider.
fn ground(b: &mut Builder) {
    use crate::domain::geometry::{kind, Vertex};
    let half = 200.0f32;
    let cell = 2.0f32;
    let n = (2.0 * half / cell) as usize;
    let mut heights = Vec::with_capacity((n + 1) * (n + 1));
    let base = b.mesh.vertices.len() as u32;
    for iz in 0..=n {
        for ix in 0..=n {
            let x = -half + ix as f32 * cell;
            let z = -half + iz as f32 * cell;
            let y = terrain::height(x, z);
            heights.push(y);
            let w = town::ground_weights(x, z);
            let to8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            b.mesh.vertices.push(Vertex {
                pos: [x, y, z],
                normal: terrain::normal(x, z).to_array(),
                // Uv-en er veikoordinatene (søla legges etter veien), og
                // vektene bærer tørt gress og søle.
                uv: w.road.to_array(),
                color: [to8(w.cobble), to8(w.dirt), to8(w.rock), to8(w.forest)],
                mat: [kind::TERRAIN, 0, 0, to8(w.earth)],
                joints: [0; 4],
                weights: [to8(w.dry), to8(w.wet), 0, 0],
            });
        }
    }
    let at = |ix: usize, iz: usize| base + (iz * (n + 1) + ix) as u32;
    for iz in 0..n {
        for ix in 0..n {
            let (a, c, d, e) = (at(ix, iz), at(ix + 1, iz), at(ix + 1, iz + 1), at(ix, iz + 1));
            // Diagonalen følger bakken: den korteste av de to.
            if (heights[iz * (n + 1) + ix] - heights[(iz + 1) * (n + 1) + ix + 1]).abs() < (heights[iz * (n + 1) + ix + 1] - heights[(iz + 1) * (n + 1) + ix]).abs() {
                b.mesh.indices.extend_from_slice(&[a, e, d, a, d, c]);
            } else {
                b.mesh.indices.extend_from_slice(&[a, e, c, c, e, d]);
            }
        }
    }
    b.solid(Shape::Heightfield { center: Vec3::ZERO, size: Vec3::new(2.0 * half, 1.0, 2.0 * half), cols: n + 1, rows: n + 1, heights });
}

pub fn at(p: Vec3, yaw: f32) -> Affine3A {
    Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p)
}
