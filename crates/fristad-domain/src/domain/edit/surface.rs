//! Det noe kan stå på i byen og krypten: bakken, gulvene og toppen av
//! kolliderne, funnet i rutenettene over hver (`world::Grid`).

use super::editor::Editor;
use super::rules;
use crate::domain::world::layout::Ref;
use crate::domain::world::{Builder, Grid, Shape};
use glam::{Quat, Vec3};

impl Editor {
    /// Byen og krypten med rutenettene over gulvene og kolliderne i hver.
    pub(super) fn grids(&self) -> [(&Builder, &(Grid, Grid)); 2] {
        let town = self.grids.get_or_init(|| (self.scene().floor_grid(), rules::collider_grid(&self.scene().colliders)));
        [(self.scene(), town), (&self.below.scene, self.below.grids())]
    }

    /// Høyden på det noe kan stå på i (x, z) under `below`: bakken, et gulv,
    /// eller toppen av en kollider (et bord, en kasse), men ikke tingens egne.
    pub fn surface_under(&self, x: f32, z: f32, below: f32, own: Option<&Ref>) -> f32 {
        // Det nærmeste gulvet under, som `Builder::stand_height`.
        let score = |top: f32| if top <= below + 0.3 { below - top } else { 1e6 + top - below };
        let grids = self.grids();
        let mut best = grids.iter().map(|(b, g)| b.stand_height_in(&g.0, x, z, below)).min_by(|a, b| score(*a).total_cmp(&score(*b))).unwrap_or(0.0);
        for (b, (_, solids)) in grids {
            for k in solids.at(x, z) {
                if own.is_some() && b.solid_origins[k].as_ref() == own {
                    continue;
                }
                if let Shape::Cuboid { center, size, yaw } = &b.colliders[k] {
                    let d = Quat::from_rotation_y(-*yaw) * (Vec3::new(x, center.y, z) - *center);
                    let top = center.y + size.y * 0.5;
                    if d.x.abs() <= size.x * 0.5 && d.z.abs() <= size.z * 0.5 && top <= below && top > best {
                        best = top;
                    }
                }
            }
        }
        best
    }
}
