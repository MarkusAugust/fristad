//! Linjene i redigeringen (L): en rad av en modell langs punktene på
//! bakken (gjerdebitene etter hverandre), eller en usynlig vegg.

use super::editor::{Change, Editor, Thing};
use crate::domain::world::layout::{Item, Pose, Ref, SolidDef};
use crate::domain::world::terrain;
use glam::{Quat, Vec3};

/// Hva en linje blir: en rad av en modell (gjerdebitene etter hverandre,
/// `spacing` meter mellom midtene, eller modellens lengde om 0), eller en
/// usynlig vegg så høy som `height`.
#[derive(Clone, Debug, PartialEq)]
pub enum LineKind {
    Row { model: String, spacing: f32 },
    Wall { height: f32 },
}

/// Tykkelsen på en usynlig vegg.
const WALL_THICK: f32 = 0.3;

impl Editor {
    /// En linje gjennom punktene på bakken: en rad av en modell langs hvert
    /// stykke, eller en usynlig vegg. Følger terrenget; blir valget, og angres
    /// som én handling.
    pub fn line(&mut self, points: &[Vec3], kind: &LineKind) -> Vec<Ref> {
        let mut items = vec![];
        for w in points.windows(2) {
            let (a, b) = (w[0], w[1]);
            let d = glam::Vec2::new(b.x - a.x, b.z - a.z);
            let len = d.length();
            if len < 0.05 {
                continue;
            }
            // Lokal x langs stykket.
            let yaw = (-d.y).atan2(d.x);
            match kind {
                LineKind::Row { model, spacing } => {
                    let step = if *spacing > 0.0 { *spacing } else { self.scene().bounds(model).map(|(lo, hi)| hi.x - lo.x).filter(|s| *s > 0.05).unwrap_or(1.0) };
                    let n = (len / step).round().max(1.0) as usize;
                    for k in 0..n {
                        let f = (k as f32 + 0.5) / n as f32;
                        let (x, z) = (a.x + d.x * f, a.z + d.y * f);
                        let p = Vec3::new(x, terrain::height(x, z), z);
                        items.push(Item { model: Some(model.clone()), at: Some(Pose { pos: p, rot: Quat::from_rotation_y(yaw), scale: Vec3::ONE }), ..Default::default() });
                    }
                }
                LineKind::Wall { height } => {
                    let (x, z) = (a.x + d.x * 0.5, a.z + d.y * 0.5);
                    // Foten i det laveste punktet langs veggen, så den når bakken.
                    let low = (0..=8).map(|k| terrain::height(a.x + d.x * k as f32 / 8.0, a.z + d.y * k as f32 / 8.0)).fold(f32::MAX, f32::min);
                    let p = Vec3::new(x, low, z);
                    let solid = SolidDef::Cuboid { center: Vec3::new(0.0, height * 0.5, 0.0), size: Vec3::new(len, *height, WALL_THICK), yaw: 0.0 };
                    items.push(Item { model: None, at: Some(Pose { pos: p, rot: Quat::from_rotation_y(yaw), scale: Vec3::ONE }), solids: vec![solid], ..Default::default() });
                }
            }
        }
        let start = self.layout.items.len();
        let refs: Vec<Ref> = (start..start + items.len()).map(Ref::Town).collect();
        let changes = items.into_iter().enumerate().map(|(k, it)| Change::Insert { r: Ref::Town(start + k), thing: Thing::Item(it) }).collect();
        self.commit(changes);
        self.selection = refs.clone();
        refs
    }

    /// Endre høyden på det valgte (usynlige vegger).
    pub fn set_wall_height(&mut self, height: f32) {
        let mut changes = vec![];
        for r in self.selection.clone() {
            let Some(Thing::Item(before)) = self.get(&r) else {
                continue;
            };
            let mut it = before.clone();
            for s in &mut it.solids {
                if let SolidDef::Cuboid { center, size, .. } = s {
                    size.y = height;
                    center.y = height * 0.5;
                }
            }
            if it != before {
                changes.push(Change::Set { r, before: Thing::Item(before), after: Thing::Item(it) });
            }
        }
        self.commit(changes);
    }
}
