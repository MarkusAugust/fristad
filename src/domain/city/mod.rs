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
use crate::domain::model::Placement;
use crate::domain::scene::Light;
use crate::domain::text::Font;
use crate::domain::tokens::{Theme, Tokens};
use glam::{Affine3A, Quat, Vec3};

pub mod furniture;
pub mod house;
pub mod nature;

pub enum Shape {
    /// Boks med sentrum, hele mål og rotasjon om Y.
    Cuboid { center: Vec3, size: Vec3, yaw: f32 },
    /// Stående sylinder med sentrum, radius og halv høyde.
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
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
pub struct Builder<'a> {
    pub mesh: Mesh,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    pub props: Vec<Prop>,
    pub placements: Vec<Placement>,
    pub font: &'a Font,
    pub tokens: &'a Tokens,
    pub rng: Rng,
    /// Installasjonen som bygger nå, så propsene får riktig eier.
    pub owner: Option<usize>,
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

    pub fn place(&mut self, model: &'static str, t: Affine3A) {
        if let Some(p) = self.placements.iter_mut().find(|p| p.model == model) {
            p.transforms.push(t);
        } else {
            self.placements.push(Placement { model, transforms: vec![t] });
        }
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

pub fn build(tokens: &Tokens, font: &Font, installations: &mut [Box<dyn Installation>]) -> City {
    let mut b = Builder {
        mesh: Mesh::default(),
        colliders: vec![],
        lights: vec![],
        props: vec![],
        placements: vec![],
        font,
        tokens,
        rng: Rng(0x5eed_f215_7a1d),
        owner: None,
    };

    // Bakken: brostein på torget og i gatene, grus utenfor.
    b.mesh.quad(
        &Affine3A::IDENTITY,
        [Vec3::new(-200.0, 0.0, 200.0), Vec3::new(200.0, 0.0, 200.0), Vec3::new(200.0, 0.0, -200.0), Vec3::new(-200.0, 0.0, -200.0)],
        &Mat::tex(Layer::Gravel, 0xffffff),
        [1.0; 4],
    );
    for (c, s) in [
        (Vec3::new(0.0, 0.005, 0.0), Vec3::new(42.0, 0.01, 38.0)),
        (Vec3::new(0.0, 0.005, -40.0), Vec3::new(12.0, 0.01, 44.0)),
        (Vec3::new(0.0, 0.005, 38.0), Vec3::new(10.0, 0.01, 40.0)),
        (Vec3::new(-30.0, 0.005, 0.0), Vec3::new(24.0, 0.01, 9.0)),
        (Vec3::new(30.0, 0.005, 0.0), Vec3::new(24.0, 0.01, 9.0)),
    ] {
        let h = s * 0.5;
        b.mesh.quad(
            &Affine3A::from_translation(c),
            [Vec3::new(-h.x, 0.0, h.z), Vec3::new(h.x, 0.0, h.z), Vec3::new(h.x, 0.0, -h.z), Vec3::new(-h.x, 0.0, -h.z)],
            &COBBLE,
            [1.0; 4],
        );
    }
    b.solid_box(Vec3::new(0.0, -0.5, 0.0), Vec3::new(400.0, 1.0, 400.0), 0.0);

    house::district(&mut b);
    furniture::plaza(&mut b);
    furniture::streets(&mut b);
    nature::plaza_trees(&mut b);
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
        spawn: Vec3::new(7.0, 0.0, 9.0),
        spawn_yaw: 0.25,
    }
}

/// Transform med rotasjon om Y og posisjon.
pub fn at(p: Vec3, yaw: f32) -> Affine3A {
    Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p)
}
