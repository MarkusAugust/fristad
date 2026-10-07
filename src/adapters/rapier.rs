//! Fysikk-adapteren: Rapier bak `ports::Physics`.

use crate::domain::world::Shape;
use crate::ports::{ColliderId, Hit, Physics};
use glam::{Affine3A, Vec3};
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;
use std::collections::HashMap;

pub struct RapierPhysics {
    colliders: ColliderSet,
    bodies: RigidBodySet,
    query: QueryPipeline,
    controller: KinematicCharacterController,
    handles: HashMap<u64, ColliderHandle>,
    next: u64,
    dirty: bool,
}

fn iso(center: Vec3, yaw: f32) -> Isometry<Real> {
    Isometry::new(vector![center.x, center.y, center.z], vector![0.0, yaw, 0.0])
}

fn to_iso(t: &Affine3A) -> Isometry<Real> {
    let (_, rot, tr) = t.to_scale_rotation_translation();
    Isometry::from_parts(
        Translation::new(tr.x, tr.y, tr.z),
        rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(rot.w, rot.x, rot.y, rot.z)),
    )
}

impl Default for RapierPhysics {
    fn default() -> Self {
        Self::new()
    }
}

impl RapierPhysics {
    pub fn new() -> Self {
        let controller = KinematicCharacterController {
            offset: CharacterLength::Absolute(0.02),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.5),
                min_width: CharacterLength::Absolute(0.15),
                include_dynamic_bodies: false,
            }),
            snap_to_ground: Some(CharacterLength::Absolute(0.3)),
            max_slope_climb_angle: 50f32.to_radians(),
            ..Default::default()
        };
        Self { colliders: ColliderSet::new(), bodies: RigidBodySet::new(), query: QueryPipeline::new(), controller, handles: HashMap::new(), next: 1, dirty: true }
    }

    fn insert(&mut self, c: Collider) -> ColliderId {
        let h = self.colliders.insert(c);
        let id = ColliderId(self.next);
        self.next += 1;
        self.handles.insert(id.0, h);
        self.dirty = true;
        id
    }

    fn refresh(&mut self) {
        if self.dirty {
            self.query.update(&self.colliders);
            self.dirty = false;
        }
    }
}

impl Physics for RapierPhysics {
    fn add(&mut self, shape: &Shape) -> ColliderId {
        let c = match shape {
            Shape::Cuboid { center, size, yaw } => ColliderBuilder::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5).position(iso(*center, *yaw)).build(),
            Shape::Cylinder { center, radius, half_height } => ColliderBuilder::cylinder(*half_height, *radius).position(iso(*center, 0.0)).build(),
            Shape::Heightfield { center, size, cols, rows, ref heights } => {
                // Rapier: rader langs X? Nei: matrisen er (nrows × ncols) der
                // rad i er langs Z og kolonne j langs X, skalert til `size`.
                let m = rapier3d::na::DMatrix::from_fn(*rows, *cols, |i, j| heights[i * cols + j]);
                ColliderBuilder::heightfield(m, vector![size.x, size.y, size.z]).position(iso(*center, 0.0)).build()
            }
            Shape::Capsule { a, b, radius } => ColliderBuilder::new(SharedShape::capsule(point![a.x, a.y, a.z], point![b.x, b.y, b.z], *radius)).build(),
            Shape::Hull { points } => {
                let pts: Vec<_> = points.iter().map(|p| point![p.x, p.y, p.z]).collect();
                match SharedShape::convex_hull(&pts) {
                    Some(hull) => ColliderBuilder::new(hull).build(),
                    None => return ColliderId(u64::MAX),
                }
            }
        };
        self.insert(c)
    }

    fn add_movable(&mut self, half: Vec3, pose: Affine3A) -> ColliderId {
        self.insert(ColliderBuilder::cuboid(half.x, half.y, half.z).position(to_iso(&pose)).build())
    }

    fn set_pose(&mut self, id: ColliderId, pose: Affine3A) {
        if let Some(h) = self.handles.get(&id.0) {
            if let Some(c) = self.colliders.get_mut(*h) {
                c.set_position(to_iso(&pose));
                self.dirty = true;
            }
        }
    }

    fn move_capsule(&mut self, pos: Vec3, half: f32, radius: f32, delta: Vec3, dt: f32) -> (Vec3, bool) {
        self.refresh();
        let shape = Capsule::new_y(half, radius);
        let p = Isometry::translation(pos.x, pos.y, pos.z);
        let mv = self.controller.move_shape(dt, &self.bodies, &self.colliders, &self.query, &shape, &p, vector![delta.x, delta.y, delta.z], QueryFilter::default(), |_| {});
        (Vec3::new(mv.translation.x, mv.translation.y, mv.translation.z), mv.grounded)
    }

    fn cast_ray(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<Hit> {
        let ray = Ray::new(point![origin.x, origin.y, origin.z], vector![dir.x, dir.y, dir.z]);
        let (h, t) = self.query.cast_ray(&self.bodies, &self.colliders, &ray, max, true, QueryFilter::default())?;
        let id = self.handles.iter().find(|(_, v)| **v == h).map(|(k, _)| ColliderId(*k))?;
        Some(Hit { collider: id, distance: t })
    }
}
