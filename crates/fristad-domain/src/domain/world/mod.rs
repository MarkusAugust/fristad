//! Verden: kjøpstaden Fristad rundt 1350, landskapet rundt og krypten
//! under. Alt bygges her fra modellene som finnes
//! (`ModelInfo`), uten å vite hvor de kommer fra.
//!
//! Koordinater: X øst, Y opp, Z sør. Torget ligger rundt origo, sørporten i
//! sør og borgen på haugen i nord.

use crate::domain::geometry::{Chunk, Mesh};
use crate::domain::model::{Look, ModelInfo, Placement};
use crate::domain::rng::Rng;
use crate::domain::scene::Light;
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::collections::HashMap;

pub mod crypt;
pub mod mechanisms;
pub mod nature;
pub mod portal;
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
    /// `holes` er rutene (rad langs Z, kolonne langs X) som er fjernet, så
    /// man kan gå ned i en kjeller under et hus.
    Heightfield { center: Vec3, size: Vec3, cols: usize, rows: usize, heights: Vec<f32>, holes: Vec<(usize, usize)> },
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
    /// Åpne kiste nummer n i krypten (`Crypt.chests`).
    Chest(usize),
    /// Port nummer n i krypten (`Crypt.gates`): spaken eller nøkkelen
    /// åpner den.
    Gate(usize),
    /// En snarveidør med slå på den ene sida (`Crypt.shortcuts`).
    Shortcut(usize),
    /// Spak nummer n: trekkes, og åpner portene sine.
    Lever(usize),
    /// Falsk vegg nummer n: F eller et slag åpner den.
    Secret(usize),
    /// Stige nummer n: opp til toppen.
    Ladder(usize),
    /// Hvilested nummer n: tenn lyset og hvil.
    Rest(usize),
    /// Lapp eller bok nummer n: les.
    Lore(usize),
    /// Varig styrke nummer n: ta den.
    Bonus(usize),
    /// Døra i krypten på kirkegården: ned til nivå 1.
    Descend,
    /// Døra opp i nivå n (fra 0): til nivået over, eller til krypten i byen.
    Ascend(usize),
    /// Brønn nummer n i byen: heis opp vann og drikk.
    Well(usize),
    /// Portalen ved sørporten: tilbake dit du var i krypten (åpen bare
    /// når du har gått hjem med H, eller har dødd nede).
    Portal,
    /// Senga i spillerens hus: sov, få tilbake liv og utholdenhet, og våkn
    /// når døgnet har snudd (dag til natt eller natt til dag).
    Sleep,
    /// Ramme nummer n på galleriveggen i spillerens hus
    /// (`craft::GALLERY[n]`): komponenten henger der når du har funnet den,
    /// og F åpner dokumentasjonen.
    Gallery(usize),
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
    /// Figuren (f.eks. `npc_smith`); usynlig om modellen ikke finnes.
    pub model: &'static str,
    pub pos: Vec3,
    pub yaw: f32,
    pub tint: Vec3,
    pub lines: &'static [&'static str],
    /// Punktene innbyggeren går mellom (føttene på bakken) og hvor lenge
    /// hen blir stående ved hvert (0: går rett forbi, langs gata).
    pub route: Vec<(Vec3, f32)>,
    /// Hvor høy figuren er (fra modellen; 1,8 m om den mangler), så
    /// snakkebobla og siktemålet står rett over hodet.
    pub height: f32,
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
    pub crypt: crypt::Crypt,
    pub npcs: Vec<NpcSpec>,
    /// Hvor du står når du kommer opp fra krypten (føttene, retningen).
    pub crypt_exit: (Vec3, f32),
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
    /// Kjellere under husene (`hole`): bakken tas bort der.
    pub holes: Vec<(Vec3, Vec2, f32)>,
    /// Det husene har med seg (kollidere, lys, kjeller …), per modell.
    pub insides: HashMap<String, town::Inside>,
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
            holes: vec![],
            insides: HashMap::new(),
        }
    }

    /// En kjeller under et hus: rutene i bakken som ligger helt innenfor
    /// rektangelet (midte, halve mål, yaw) fjernes, både i bildet og i
    /// kollideren, så man kan gå ned. Kjellerens murer må dekke kanten.
    pub fn hole(&mut self, center: Vec3, half: Vec2, yaw: f32) {
        self.holes.push((center, half, yaw));
    }

    pub fn light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: false, flicker: 0.0, room: None });
    }

    /// En pipe som ryker (toppen av pipa, tykkelse 0..1).
    pub fn smoke(&mut self, top: Vec3, strength: f32) {
        self.smoke.push((top, strength));
    }

    /// En lykt med et talglys: tennes i skumringen og blafrer svakt.
    pub fn lamp_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: false, flicker: 0.12, room: None });
    }

    /// Et lys som brenner hele døgnet (ild), med et lite flakk.
    pub fn fire_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: true, flicker: 0.18, room: None });
    }

    /// Et lys som brenner hele døgnet og flakker så mye du vil.
    pub fn flicker_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32, flicker: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: true, flicker, room: None });
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

    /// Sett ut en modell med et annet utseende (`Look`).
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
                // Murverket har fundament under bakken og kan gå ned i den
                // på oversida i skråninger; det
                // sjekkes bare for sveving, og bakken letes etter opp til en
                // meter over foten (ellers vinner et gulv lenger nede, som
                // nivåene i krypten).
                let rooted = town::FOUNDED.iter().any(|m| *m == pl.model);
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


/// Data fra Blender som ikke er modeller: det husene har med seg
/// (`<navn>_inside.json`) og kryptnivåene (`crypt_NN.json`). Adapterne leser
/// filene; kjernen får dem ferdig tolket.
#[derive(Default)]
pub struct Data {
    pub insides: Vec<(String, town::Inside)>,
    pub levels: Vec<(u32, crypt::level::LevelFile)>,
}

/// Bygg verden. Krypten er fast; `seed` brukes ikke til noe der lenger.
pub fn build(models: &[ModelInfo], _seed: u64, data: Data) -> World {
    let mut b = Builder::new(models);
    b.insides = data.insides.into_iter().collect();

    // Bakken: et rutenett som følger terrenget, med brostein på torget,
    // jordveier og gress.
    let ground_at = ground(&mut b);
    let town = town::build(&mut b);
    cut_ground(&mut b, ground_at);
    nature::landscape(&mut b);
    nature::world_edge(&mut b);
    // Krypten: de faste nivåene under kirkegården.
    let crypt = crypt::build(&mut b, town.crypt_anchor, data.levels);
    for (model, n) in crate::domain::mobs::Mobs::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in crate::domain::combat::Combat::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in crate::domain::herd::Herd::reserve() {
        b.reserve(model, n);
    }
    for (model, n) in crypt::reserve(&crypt) {
        b.reserve(model, n);
    }
    // Galleriveggen i spillerens hus: én modell per ramme (bildet av
    // komponenten), satt ut av økta når komponenten er funnet.
    if b.targets.iter().any(|t| matches!(t.what, Use::Gallery(_))) {
        for model in crate::domain::craft::GALLERY_MODELS {
            b.reserve(model, 1);
        }
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
        crypt,
        npcs: b.npcs,
        spawn: town.spawn,
        spawn_yaw: town.spawn_yaw,
        crypt_exit: town.crypt_exit,
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
fn ground(b: &mut Builder) -> usize {
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
    let first = b.mesh.indices.len();
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
    b.solid(Shape::Heightfield { center: Vec3::ZERO, size: Vec3::new(2.0 * half, 1.0, 2.0 * half), cols: n + 1, rows: n + 1, heights, holes: vec![] });
    first
}

/// Tar bort rutene i bakken over kjellerne (`Builder::hole`): trekantene
/// blir tomme, og kollideren får hull. `first` er der bakkens indekser
/// begynner (to trekanter per rute, rad for rad).
fn cut_ground(b: &mut Builder, first: usize) {
    let (half, cell) = (200.0f32, 2.0f32);
    let n = (2.0 * half / cell) as usize;
    let inside = |p: Vec2| {
        b.holes.iter().any(|(c, h, yaw)| {
            let d = p - Vec2::new(c.x, c.z);
            let (s, co) = yaw.sin_cos();
            // Lokale akser som i `at`: x dreid med yaw om Y.
            let lx = d.x * co - d.y * s;
            let lz = d.x * s + d.y * co;
            lx.abs() <= h.x && lz.abs() <= h.y
        })
    };
    let mut removed = vec![];
    for (c, h, _) in b.holes.clone() {
        let r = h.length();
        let lo = |v: f32| (((v - r + half) / cell).floor().max(0.0) as usize).min(n - 1);
        let hi = |v: f32| (((v + r + half) / cell).ceil().max(0.0) as usize).min(n - 1);
        for iz in lo(c.z)..=hi(c.z) {
            for ix in lo(c.x)..=hi(c.x) {
                let x0 = -half + ix as f32 * cell;
                let z0 = -half + iz as f32 * cell;
                let corners = [Vec2::new(x0, z0), Vec2::new(x0 + cell, z0), Vec2::new(x0, z0 + cell), Vec2::new(x0 + cell, z0 + cell)];
                if corners.iter().all(|p| inside(*p)) && !removed.contains(&(iz, ix)) {
                    removed.push((iz, ix));
                }
            }
        }
    }
    for &(iz, ix) in &removed {
        let k = first + (iz * n + ix) * 6;
        let v = b.mesh.indices[k];
        b.mesh.indices[k..k + 6].fill(v);
    }
    for shape in &mut b.colliders {
        if let Shape::Heightfield { holes, .. } = shape {
            holes.extend(removed.iter().copied());
        }
    }
}

pub fn at(p: Vec3, yaw: f32) -> Affine3A {
    Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p)
}
