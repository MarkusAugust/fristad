//! Byen som data (`assets/layout/town.json`, `docs/redigering.md`): hver
//! ting som står i byen er et `Item` med modell, sted, dreining og skala,
//! og med det den har med seg i sitt eget rom (kollidere, ting du kan
//! bruke, lys, røyk, gulv, kjellere, plass til bevegelige deler). Det et hus
//! har med seg fra Blender (`<navn>_inside.json`), følger modellen og står
//! ikke her (`building`). Innbyggerne er sted, rute og en nøkkel til
//! replikkene (`cast`).
//!
//! `apply` bygger byen fra dataene. Redigeringsmodusen endrer dataene, ikke
//! koden.

use super::{cast, town, Builder, NpcSpec, Shape, Use};
use crate::domain::model::Look;
use crate::domain::scene::Light;
use glam::{Affine3A, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

/// Sted, dreining og skala.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pose {
    pub pos: Vec3,
    pub rot: Quat,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub scale: Vec3,
}

fn one() -> Vec3 {
    Vec3::ONE
}

fn is_one(v: &Vec3) -> bool {
    *v == Vec3::ONE
}

impl Pose {
    pub fn of(t: Affine3A) -> Self {
        let (scale, rot, pos) = t.to_scale_rotation_translation();
        Self { pos, rot, scale }
    }

    pub fn affine(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rot, self.pos)
    }

    /// Dreiningen om Y.
    pub fn yaw(&self) -> f32 {
        let (y, _, _) = self.rot.to_euler(glam::EulerRot::YXZ);
        y
    }
}

/// Et lys i tingens rom.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LightDef {
    pub pos: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub intensity: f32,
    pub always: bool,
    pub flicker: f32,
}

/// Noe du kan bruke med F, i tingens rom.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TargetDef {
    pub what: UseDef,
    pub pose: Pose,
    pub center: Vec3,
    pub half: Vec3,
}

/// `Use` som data (innbyggerne lages fra `npcs`, ikke her).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UseDef {
    Well(usize),
    Descend,
    Portal,
    Sleep,
    Gallery(usize),
}

impl UseDef {
    fn to_use(self) -> Use {
        match self {
            Self::Well(n) => Use::Well(n),
            Self::Descend => Use::Descend,
            Self::Portal => Use::Portal,
            Self::Sleep => Use::Sleep,
            Self::Gallery(n) => Use::Gallery(n),
        }
    }
}

/// En kollider i tingens rom.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SolidDef {
    Cuboid { center: Vec3, size: Vec3, yaw: f32 },
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
    Capsule { a: Vec3, b: Vec3, radius: f32 },
    Hull { points: Vec<Vec3> },
}

/// En ting i byen.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Item {
    /// Modellen; ingen for det som bare er kollidere eller ting du kan bruke
    /// (portalens siktemål).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub at: Option<Pose>,
    /// Et hus: det det har med seg fra Blender (innsidemodellen, kolliderne,
    /// lysene, røyken, kjelleren, småtingene) følger modellen.
    #[serde(default, skip_serializing_if = "is_false")]
    pub building: bool,
    /// Henger på en vegg eller står oppå noe (ikke målt mot bakken).
    #[serde(default, skip_serializing_if = "is_false")]
    pub mounted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solids: Vec<SolidDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<TargetDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lights: Vec<LightDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub smoke: Vec<(Vec3, f32)>,
    /// Gulv (midte, halve mål, yaw, overkant), i tingens rom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub floors: Vec<(Vec3, Vec2, f32, f32)>,
    /// Kjellere (midte, halve mål, yaw), i tingens rom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holes: Vec<(Vec3, Vec2, f32)>,
    /// Plass til deler spillet beveger (brønnens vinde, portalen).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserve: Vec<(String, usize)>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// En innbygger: navnet, figuren, fargen, replikkene (nøkkel i `cast`) og
/// ruta (punkt og hvor lenge hen står der).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NpcDef {
    pub name: String,
    pub model: String,
    pub pos: Vec3,
    pub yaw: f32,
    pub tint: Vec3,
    pub lines: String,
    pub route: Vec<(Vec3, f32)>,
}

/// Byen som data.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Layout {
    /// Der du starter (føttene) og retningen.
    pub spawn: (Vec3, f32),
    /// Der du kommer opp fra krypten.
    pub crypt_exit: (Vec3, f32),
    /// Krypten på kirkegården (x, z), som nivåene henger under.
    pub crypt_anchor: Vec2,
    pub items: Vec<Item>,
    pub npcs: Vec<NpcDef>,
}

/// Hvilken ting i dataene: en ting i byen (nummer i `Layout.items`), et
/// møbel i et hus (nummer i husets innhold, `assets/layout/inside/<hus>.json`),
/// en innbygger, eller et merke eller lys i et kryptnivå
/// (`assets/layout/crypt_NN.json`). Redigeringen bruker den til å finne tingen igjen fra
/// det som er satt ut (`Builder.origins`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ref {
    Town(usize),
    Inside { house: String, i: usize },
    Npc(usize),
    Crypt { level: u32, i: usize },
    CryptLight { level: u32, i: usize },
}

/// Navnene i dataene som `&'static str`, som resten av verden bruker: hvert
/// navn lekker én gang (det er bare så mange modeller og innbyggere).
pub fn intern(s: &str) -> &'static str {
    static NAMES: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
    let mut g = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    let set = g.get_or_insert_with(HashSet::new);
    if let Some(n) = set.get(s) {
        return n;
    }
    let n: &'static str = Box::leak(s.to_string().into_boxed_str());
    set.insert(n);
    n
}

/// Bygg byen fra dataene. Gir det byen forteller resten av verden.
pub fn apply(b: &mut Builder, layout: &Layout) -> town::Town {
    for (i, item) in layout.items.iter().enumerate() {
        b.origin = Some(Ref::Town(i));
        apply_item(b, item, Affine3A::IDENTITY);
    }
    for (i, n) in layout.npcs.iter().enumerate() {
        b.origin = Some(Ref::Npc(i));
        town::settle(b, NpcSpec { name: intern(&n.name), model: intern(&n.model), pos: n.pos, yaw: n.yaw, tint: n.tint, lines: cast::lines(&n.lines), route: n.route.clone(), height: 0.0 });
    }
    b.origin = None;
    town::Town { spawn: layout.spawn.0, spawn_yaw: layout.spawn.1, crypt_anchor: layout.crypt_anchor, crypt_exit: layout.crypt_exit }
}

/// Sett ut én ting, i rommet til `parent` (verden, eller huset den står i).
/// En modell med `<navn>_inside.json` tar med seg kolliderne og lysene sine
/// (`town::carry`); et hus også innsida og det som står inne.
pub fn apply_item(b: &mut Builder, item: &Item, parent: Affine3A) {
    let t = parent * item.at.map(|p| p.affine()).unwrap_or(Affine3A::IDENTITY);
    let y0 = Pose::of(t).yaw();
    if let Some(m) = &item.model {
        let m = intern(m);
        if item.building {
            town::building_at(b, m, t);
        } else {
            let placed = if item.mounted { b.place_mounted_look(m, t, None::<Look>) } else { b.place_tinted(m, t, item.tint.unwrap_or(Vec3::ONE)) };
            if placed {
                town::carry(b, m, t);
            }
        }
    }
    let p = |v: &Vec3| t.transform_point3(*v);
    for s in &item.solids {
        b.solid(match s {
            SolidDef::Cuboid { center, size, yaw } => Shape::Cuboid { center: p(center), size: *size, yaw: yaw + y0 },
            SolidDef::Cylinder { center, radius, half_height } => Shape::Cylinder { center: p(center), radius: *radius, half_height: *half_height },
            SolidDef::Capsule { a, b: e, radius } => Shape::Capsule { a: p(a), b: p(e), radius: *radius },
            SolidDef::Hull { points } => Shape::Hull { points: points.iter().map(p).collect() },
        });
    }
    for d in &item.targets {
        b.target(d.what.to_use(), t * d.pose.affine(), d.center, d.half);
    }
    for l in &item.lights {
        b.push_light(Light { pos: p(&l.pos), radius: l.radius, color: l.color, intensity: l.intensity, always: l.always, flicker: l.flicker, room: None });
    }
    for (top, s) in &item.smoke {
        b.smoke(p(top), *s);
    }
    for (c, half, yaw, top) in &item.floors {
        b.floor(p(c), *half, yaw + y0, top + t.translation.y);
    }
    for (c, half, yaw) in &item.holes {
        b.hole(p(c), *half, yaw + y0);
    }
    for (m, n) in &item.reserve {
        b.reserve(intern(m), *n);
    }
}
