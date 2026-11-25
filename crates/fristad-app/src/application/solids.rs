//! Kolliderne som følger mekanismene: lemmene over fallemmene, dekkene på
//! broene og heisene, og steinringen til portalen. Reglene står i kjernen
//! (`domain::world::mechanisms`); her flyttes det fysikken ser, så det du
//! står på og det som stenger veien er der det synes. Delen eier
//! kolliderne og får fysikken, krypten og målene som argument fra økta.

use super::session::{away, TargetState};
use crate::domain::body::Body;
use crate::domain::world::crypt::Crypt;
use crate::domain::world::mechanisms::{Mechanisms, Moved};
use crate::domain::world::{self, town, Use, World};
use crate::ports::{ColliderId, Physics};
use glam::{Affine3A, Vec2, Vec3};

pub(super) struct Solids {
    /// Lemmene som holder deg til de faller.
    trap_lids: Vec<Option<ColliderId>>,
    /// Dekkene på broene over avgrunnene (flyttes når broa senkes).
    bridges: Vec<ColliderId>,
    /// Dekkene på heisene.
    lifts: Vec<ColliderId>,
    /// Portalringen (flyttes inn når den står der).
    portal: Vec<ColliderId>,
}

impl Solids {
    pub fn new(world: &World, physics: &mut dyn Physics) -> Self {
        let c = &world.crypt;
        // Der de står før noe har rørt seg.
        let rest = Mechanisms::new(c, 0);
        let deck = |size: Vec2| Vec3::new(size.x * 0.5, 0.15, size.y * 0.5);
        Self {
            trap_lids: c.trapdoors.iter().map(|t| t.lid.then(|| physics.add_movable(Vec3::new(t.size.x * 0.5, 0.05, t.size.y * 0.5), world::at(t.pos - Vec3::Y * 0.05, t.yaw)))).collect(),
            bridges: (0..c.bridges.len()).map(|n| physics.add_movable(deck(c.bridges[n].size), rest.bridge_collider(c, n).unwrap_or_else(away))).collect(),
            lifts: (0..c.lifts.len()).map(|n| physics.add_movable(deck(c.lifts[n].size), rest.lift_collider(c, n).unwrap_or_else(away))).collect(),
            portal: town::PORTAL_SOLIDS.iter().map(|(_, size)| physics.add_movable(*size * 0.5, away())).collect(),
        }
    }

    /// Kolliderne og siktemålene følger det som har rørt seg: snarveidørene
    /// dreies, de falske veggene synker, lemmene faller, gitrene heises.
    /// Gir dekkene på heisene som kjørte (nummeret og posen), så økta kan ta
    /// med den som står på dem (`carry`).
    pub fn follow(&self, moved: &[Moved], mech: &Mechanisms, crypt: &Crypt, targets: &[TargetState], physics: &mut dyn Physics) -> Vec<(usize, Affine3A)> {
        let target = |what: Use| targets.iter().find(|t| t.target.what == what);
        let mut decks = vec![];
        for m in moved {
            match *m {
                Moved::Lift(n) => {
                    let (Some(pose), Some(id)) = (mech.lift_collider(crypt, n), self.lifts.get(n)) else { continue };
                    physics.set_pose(*id, pose);
                    let deck = mech.lift_pose(crypt, n).unwrap_or(pose);
                    if let Some(t) = target(Use::Lift(n)) {
                        t.set_pose(physics, deck * Affine3A::from_translation(t.target.center));
                    }
                    decks.push((n, deck));
                }
                Moved::Shortcut(n) => {
                    if let (Some(pose), Some(t)) = (mech.shortcut_collider(crypt, n), target(Use::Shortcut(n))) {
                        t.set_pose(physics, pose);
                    }
                }
                Moved::Secret(n) => {
                    if let Some(t) = target(Use::Secret(n)) {
                        let sink = mech.secret_sink(crypt, n);
                        t.set_pose(physics, t.target.pose * Affine3A::from_translation(t.target.center - Vec3::Y * sink));
                    }
                }
                Moved::Trapdoor(n) => {
                    if let Some(Some(id)) = self.trap_lids.get(n) {
                        physics.set_pose(*id, away());
                    }
                }
                Moved::Bridge(n) => {
                    if let (Some(pose), Some(id)) = (mech.bridge_collider(crypt, n), self.bridges.get(n)) {
                        physics.set_pose(*id, pose);
                    }
                }
                Moved::Gate(n) => {
                    if let Some(t) = target(Use::Gate(n)) {
                        t.set_pose(physics, mech.gate_lifted(n, t.target.pose) * Affine3A::from_translation(t.target.center));
                    }
                }
            }
        }
        decks
    }

    /// Siktemålet og kolliderne i portalringen: ute når portalen står der,
    /// ellers langt under verden.
    pub fn show_portal(&self, show: bool, targets: &[TargetState], physics: &mut dyn Physics) {
        let Some(t) = targets.iter().find(|t| t.target.what == Use::Portal) else { return };
        t.set_pose(physics, if show { t.target.pose * Affine3A::from_translation(t.target.center) } else { away() });
        for (id, (c, _)) in self.portal.iter().zip(town::PORTAL_SOLIDS) {
            physics.set_pose(*id, if show { t.target.pose * Affine3A::from_translation(c) } else { away() });
        }
    }
}

/// Står du på dekket (`size`, posen `deck`, som gikk `dy` dette steget), blir
/// du med det: kapselen følger ikke en kollider som flyttes.
pub(super) fn carry(body: &mut Body, size: Vec2, deck: Affine3A, dy: f32) {
    let feet = body.feet();
    let local = deck.inverse().transform_point3(feet);
    if local.x.abs() < size.x * 0.5 + 0.3 && local.z.abs() < size.y * 0.5 + 0.3 && local.y > -0.6 + dy.min(0.0) && local.y < 0.8 {
        body.pos.y += deck.translation.y + 0.02 - feet.y;
        body.vel_y = 0.0;
        body.grounded = true;
    }
}
