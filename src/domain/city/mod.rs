//! Fristad bygges her, prosedyrisk, fra Fristil-tokens.
//!
//! Målestokken er at 1 rem er en meter: `spacing-4` (16px) er 1 m,
//! `spacing-12` (48px) er 3 m, som er etasjehøyden, og `spacing-6` (24px) er
//! 1,5 m, som er vindushøyden. Komponentene i byen har egen målestokk:
//! én CSS-piksel er én centimeter.
//!
//! Koordinater: X øst, Y opp, Z sør. Torget ligger rundt origo, og porten
//! med den store knappen står i nord.

use crate::domain::geometry::{Layer, Mat, Mesh};
use crate::domain::installations::Installation;
use crate::domain::model::{ModelInfo, Placement};
use std::collections::HashMap;
use crate::domain::scene::Light;
use crate::domain::text::Font;
use crate::domain::tokens::{Theme, Tokens};
use glam::{Affine3A, Quat, Vec3};

pub mod furniture;
pub mod house;
pub mod nature;
pub mod street_life;
pub mod terrain;

pub enum Shape {
    /// Boks med sentrum, hele mål og rotasjon om Y.
    Cuboid { center: Vec3, size: Vec3, yaw: f32 },
    /// Stående sylinder med sentrum, radius og halv høyde.
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
    /// Terrenget: et rutenett av høyder, `cols` langs X og `rows` langs Z,
    /// sentrert i `center` og `size` meter stort.
    Heightfield { center: Vec3, size: Vec3, cols: usize, rows: usize, heights: Vec<f32> },
}

/// Noe i byen som flytter seg eller kan siktes på: knapper, dører, brytere.
pub struct Prop {
    pub name: &'static str,
    pub mesh: Mesh,
    /// Hvor den står i ro.
    pub rest: Affine3A,
    /// Kollisjon i propens eget koordinatsystem: sentrum og halve mål.
    pub collider: Option<(Vec3, Vec3)>,
    /// Installasjonen som eier den, og dermed hva E, R og F gjør.
    pub owner: Option<usize>,
}

pub struct City {
    pub mesh: Mesh,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    pub props: Vec<Prop>,
    pub placements: Vec<Placement>,
    pub spawn: Vec3,
    pub spawn_yaw: f32,
}

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }
    pub fn pick<T: Copy>(&mut self, v: &[T]) -> T {
        v[((self.next() * v.len() as f32) as usize).min(v.len() - 1)]
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.next() < p
    }
}

/// Det installasjonene og bydelene bygger med.
/// En husfasade, så gatelivet kan stille ting inntil ekte vegger.
#[derive(Clone, Copy, Debug)]
pub struct Facade {
    /// Midt på fasaden, i bakkehøyde.
    pub origin: Vec3,
    /// Langs fasaden (lokal +X for huset).
    pub along: Vec3,
    /// Ut fra fasaden, mot gata.
    pub facing: Vec3,
    pub width: f32,
    /// Dørposisjonen langs fasaden, fra midten.
    pub door: f32,
    pub shop: bool,
}

pub struct Builder<'a> {
    pub mesh: Mesh,
    pub facades: Vec<Facade>,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    pub props: Vec<Prop>,
    pub placements: Vec<Placement>,
    pub font: &'a Font,
    pub tokens: &'a Tokens,
    pub rng: Rng,
    /// Installasjonen som bygger nå, så propsene får riktig eier.
    pub owner: Option<usize>,
    /// Modellene som finnes, med mål. Byen bruker dem framfor prosedyrisk
    /// geometri når de er der.
    pub models: HashMap<String, ModelInfo>,
}

impl Builder<'_> {
    pub fn prop(&mut self, name: &'static str, mesh: Mesh, rest: Affine3A, collider: Option<(Vec3, Vec3)>) -> usize {
        self.props.push(Prop { name, mesh, rest, collider, owner: self.owner });
        self.props.len() - 1
    }

    pub fn light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity });
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

    /// Høyden på en modell, eller 0 om den ikke finnes.
    pub fn model_height(&self, name: &str) -> f32 {
        self.models.get(name).map(|m| m.max.y - m.min.y).unwrap_or(0.0)
    }

    pub fn place(&mut self, model: &'static str, t: Affine3A) {
        if let Some(p) = self.placements.iter_mut().find(|p| p.model == model) {
            p.transforms.push(t);
        } else {
            self.placements.push(Placement { model, transforms: vec![t] });
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

    /// Meter per spacing-trinn.
    pub fn unit(&self, step: &str) -> f32 {
        self.tokens.spacing_px(step) / 16.0
    }

    /// Fasadefargene hentes fra Fristil sine lyse tokens og blandes mot
    /// kalkhvitt, slik malte nordiske trehus og pussede murhus ser ut.
    pub fn paints(&self) -> Vec<Vec3> {
        let lime = Vec3::new(0.93, 0.9, 0.84);
        [
            ("neutral-surface", 0.2),
            ("brand3-fill", 0.82),
            ("warning-fill", 0.45),
            ("accent-border-subtle", 0.3),
            ("brand1-border-subtle", 0.25),
            ("neutral-raised", 0.1),
            ("brand3-fill", 0.62),
            ("warning-border-subtle", 0.55),
            ("success-border-subtle", 0.4),
            ("brand2-border-subtle", 0.35),
        ]
        .iter()
        .map(|(name, amount)| lime.lerp(self.tokens.color(Theme::Light, name), *amount))
        .collect()
    }
}

pub const WHITE_TRIM: Mat = Mat::plain(0xf2efe8, 0.55);
pub const DARK_TRIM: Mat = Mat::plain(0x2b2d30, 0.6);
pub const IRON: Mat = Mat::tex(Layer::Metal, 0xffffff);
pub const STONE: Mat = Mat::tex(Layer::Stone, 0xffffff);
pub const CONCRETE: Mat = Mat::tex(Layer::Concrete, 0xffffff);
pub const COBBLE: Mat = Mat::tex(Layer::Cobble, 0xffffff);
pub const OAK: Mat = Mat::tex(Layer::Oak, 0xffffff);

pub fn build(tokens: &Tokens, font: &Font, models: &[ModelInfo], installations: &mut [Box<dyn Installation>]) -> City {
    let mut b = Builder {
        mesh: Mesh::default(),
        facades: vec![],
        colliders: vec![],
        lights: vec![],
        props: vec![],
        placements: vec![],
        font,
        tokens,
        rng: Rng(0x5eed_f215_7a1d),
        owner: None,
        models: models.iter().map(|m| (m.name.clone(), m.clone())).collect(),
    };

    // Bakken: et rutenett som følger terrenget, med brostein på torget og i
    // gatene, grus rundt husene og gress utenfor.
    ground(&mut b);

    house::district(&mut b);
    furniture::plaza(&mut b);
    furniture::streets(&mut b);
    street_life::all(&mut b);
    nature::plaza_trees(&mut b);
    nature::tufts(&mut b);
    nature::meadow(&mut b);
    nature::hills(&mut b);
    nature::mountains(&mut b);

    for (i, inst) in installations.iter_mut().enumerate() {
        b.owner = Some(i);
        inst.build(&mut b);
    }
    b.owner = None;

    City {
        mesh: b.mesh,
        colliders: b.colliders,
        lights: b.lights,
        props: b.props,
        placements: b.placements,
        spawn: terrain::at(Vec3::new(7.0, 0.0, 9.0)),
        spawn_yaw: 0.25,
    }
}

/// Avstand fra et punkt til et rektangel (0 innenfor).
fn rect_dist(x: f32, z: f32, hx: f32, z0: f32, z1: f32) -> f32 {
    let dx = (x.abs() - hx).max(0.0);
    let dz = (z0 - z).max(z - z1).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// Avstand fra et punkt til bebyggelsen: torget med husrekkene og gatene.
pub fn town_dist(x: f32, z: f32) -> f32 {
    rect_dist(x, z, 34.0, -32.0, 32.0)
        .min(rect_dist(x, z, 13.0, -66.0, -19.0))
        .min(rect_dist(x, z, 12.0, 19.0, 62.0))
        .min(rect_dist(z, x, 11.0, -46.0, -19.0))
        .min(rect_dist(z, x, 11.0, 19.0, 46.0))
}

/// Stiene ut av byen: grusveier som slynger seg videre fra gatene.
/// Gir avstanden til nærmeste sti.
pub fn path_dist(x: f32, z: f32) -> f32 {
    let mut d = f32::MAX;
    // Nordover fra gata, østover og vestover fra torget, sørover fra sørgata.
    let n = terrain::noise(z * 0.03, 1.0);
    if z < -60.0 {
        d = d.min((x - (n - 0.5) * 24.0).abs());
    }
    if z > 56.0 {
        let n = terrain::noise(z * 0.03, 9.0);
        d = d.min((x - (n - 0.5) * 30.0 - (z - 56.0) * 0.25).abs());
    }
    if x > 44.0 {
        let n = terrain::noise(x * 0.03, 4.0);
        d = d.min((z - (n - 0.5) * 26.0).abs());
    }
    if x < -44.0 {
        let n = terrain::noise(x * 0.03, 6.0);
        d = d.min((z - (n - 0.5) * 20.0 + (x + 44.0) * 0.2).abs());
    }
    d
}

/// Fargen på enga i et punkt: grønnere i søkk, gulere på ryggene, med
/// flekker, og et par åkrer i gult.
fn meadow_tint(x: f32, z: f32) -> u32 {
    let n1 = terrain::noise(x * 0.05 + 2.0, z * 0.05);
    let n2 = terrain::noise(x * 0.17, z * 0.17 + 3.0);
    let green = Vec3::new(0.72, 0.95, 0.55);
    let straw = Vec3::new(1.0, 0.93, 0.62);
    let dark = Vec3::new(0.6, 0.82, 0.48);
    let mut c = green.lerp(straw, (n1 * 1.3 - 0.2).clamp(0.0, 1.0));
    c = c.lerp(dark, (n2 - 0.55).max(0.0) * 1.5);
    // Åkrer: to ovaler.
    for (cx, cz, rx, rz) in [(95.0, -40.0, 40.0, 22.0), (-80.0, 70.0, 30.0, 26.0)] {
        let e = ((x - cx) / rx).powi(2) + ((z - cz) / rz).powi(2);
        if e < 1.0 {
            c = Vec3::new(1.05, 0.95, 0.6);
        }
    }
    let to8 = |v: f32| ((v.clamp(0.0, 1.0)) * 255.0) as u32;
    (to8(c.x) << 16) | (to8(c.y) << 8) | to8(c.z)
}

/// Hva bakken er dekket av i et punkt.
fn ground_mat(x: f32, z: f32) -> Mat {
    let in_square = x.abs() < 21.0 && z.abs() < 19.0;
    let in_north = x.abs() < 6.0 && (-62.0..-19.0).contains(&z);
    let in_south = x.abs() < 5.0 && (19.0..58.0).contains(&z);
    let in_west = z.abs() < 4.5 && (-42.0..-19.0).contains(&x);
    let in_east = z.abs() < 4.5 && (19.0..42.0).contains(&x);
    if in_square || in_north || in_south || in_west || in_east {
        return COBBLE;
    }
    // Et smalt belte med grus rundt husene, med ujevn kant, og grusstiene.
    let edge = 2.0 + terrain::noise(x * 0.09, z * 0.09) * 4.0;
    if town_dist(x, z) < edge || path_dist(x, z) < 1.6 + terrain::noise(x * 0.2, z * 0.2) * 1.2 {
        return Mat::tex(Layer::Gravel, 0xf0e6d6);
    }
    Mat::tex(Layer::Grass, meadow_tint(x, z))
}

fn ground(b: &mut Builder) {
    let half = 200.0f32;
    let cell = 2.0f32;
    let n = (2.0 * half / cell) as usize;
    let mut heights = Vec::with_capacity((n + 1) * (n + 1));
    for iz in 0..=n {
        for ix in 0..=n {
            let x = -half + ix as f32 * cell;
            let z = -half + iz as f32 * cell;
            heights.push(terrain::height(x, z));
        }
    }
    for iz in 0..n {
        for ix in 0..n {
            let x0 = -half + ix as f32 * cell;
            let z0 = -half + iz as f32 * cell;
            let h = |i: usize, j: usize| heights[j * (n + 1) + i];
            let p = [
                Vec3::new(x0, h(ix, iz + 1), z0 + cell),
                Vec3::new(x0 + cell, h(ix + 1, iz + 1), z0 + cell),
                Vec3::new(x0 + cell, h(ix + 1, iz), z0),
                Vec3::new(x0, h(ix, iz), z0),
            ];
            let nrm = [
                terrain::normal(p[0].x, p[0].z),
                terrain::normal(p[1].x, p[1].z),
                terrain::normal(p[2].x, p[2].z),
                terrain::normal(p[3].x, p[3].z),
            ];
            let mat = ground_mat(x0 + cell * 0.5, z0 + cell * 0.5);
            b.mesh.quad_n(p, nrm, &mat, 1.0);
        }
    }
    b.solid(Shape::Heightfield { center: Vec3::ZERO, size: Vec3::new(2.0 * half, 1.0, 2.0 * half), cols: n + 1, rows: n + 1, heights });
}

/// Transform med rotasjon om Y og posisjon.
pub fn at(p: Vec3, yaw: f32) -> Affine3A {
    Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p)
}
