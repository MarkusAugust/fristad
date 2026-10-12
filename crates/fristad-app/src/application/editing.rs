//! Redigeringen i økta (`docs/redigering.md`): kommandoene fra nettlaget går
//! til `Editor` i kjernen, du flyr fritt og gjennom alt, og bildet viser det
//! som er flyttet på det nye stedet (`Frame.hidden` og `instances`) til
//! byen bygges på nytt når du trykker P.

use super::session::Session;
use crate::domain::edit::Editor;
use crate::domain::scene::ModelInstance;
use crate::domain::world::terrain;
use crate::ports::driving::{EditCommand, EditView, Event, Input};
use glam::{Affine3A, Quat, Vec3};
use std::collections::HashMap;

/// Fart i redigeringen (m/s), og med Shift.
const FLY: f32 = 6.0;
const FLY_FAST: f32 = 24.0;
/// Hvor langt siktekorset rekker når en ny ting settes ut.
const REACH: f32 = 80.0;

/// En pose som nøkkel (millimeter), til å se hva som står likt.
fn pose_key(model: &str, t: &Affine3A) -> (String, [i32; 12]) {
    let a = t.to_cols_array();
    (model.to_string(), a.map(|v| (v * 1000.0).round() as i32))
}

impl Session {
    pub fn editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Redigeringen, til testene og nettlaget.
    pub fn editor(&self) -> Option<&Editor> {
        self.edit.as_ref()
    }

    /// En kommando fra redigeringen. Gir om den gjorde noe.
    pub fn edit_command(&mut self, c: EditCommand) -> bool {
        if let EditCommand::Enter = c {
            let Some(d) = self.edit_data.clone() else { return false };
            self.edit = Some(Editor::new(d.sources, d.layout, d.contents, d.approved));
            self.push_edit_view();
            return true;
        }
        let eye = self.eye();
        let dir = self.forward();
        let yaw = self.yaw;
        let aim = self.aim_point();
        let Some(e) = self.edit.as_mut() else { return false };
        let done = match c {
            EditCommand::Enter => true,
            EditCommand::Leave => {
                self.edit = None;
                self.pending.push(Event::Edit(None));
                return true;
            }
            EditCommand::Pick { add } => e.pick(eye, dir, add).is_some(),
            EditCommand::Move(d) => {
                e.translate(d);
                true
            }
            EditCommand::Rotate(a) => {
                e.rotate(a);
                true
            }
            EditCommand::SetPose { r, pose } => {
                e.set_pose(&r, pose);
                true
            }
            EditCommand::Drop => {
                e.drop_to_surface();
                true
            }
            EditCommand::Copy => {
                e.duplicate();
                true
            }
            EditCommand::Delete => {
                e.delete();
                true
            }
            EditCommand::Place { model } => match aim {
                Some(p) => {
                    // Vendt mot deg (husmodellene har forsida mot lokal +Z).
                    e.place(&model, Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p));
                    true
                }
                None => false,
            },
            EditCommand::Line { points, kind } => !e.line(&points, &kind).is_empty(),
            EditCommand::WallHeight(h) => {
                e.set_wall_height(h);
                true
            }
            EditCommand::Undo => e.undo(),
            EditCommand::Redo => e.redo(),
        };
        self.push_edit_view();
        done
    }

    fn push_edit_view(&mut self) {
        let Some(e) = &self.edit else { return };
        let view = EditView {
            selected: e.selected(),
            breaches: e.breaches().iter().map(|b| (b.r.clone(), b.why.clone())).collect(),
            can_undo: e.can_undo(),
            can_redo: e.can_redo(),
            dirty: e.dirty(),
            clean: e.clean(),
        };
        self.pending.push(Event::Edit(Some(view)));
    }

    /// Der siktekorset treffer bakken eller noe å stå på (en ny ting settes
    /// ut der).
    pub fn aim_point(&self) -> Option<Vec3> {
        let e = self.edit.as_ref()?;
        let (eye, dir) = (self.eye(), self.forward());
        let mut last = eye;
        let mut s = 0.0;
        while s < REACH {
            s += 0.1;
            let p = eye + dir * s;
            let ground = e.surface_under(p.x, p.z, last.y.max(p.y) + 0.01, None);
            if p.y <= ground {
                return Some(Vec3::new(p.x, ground, p.z));
            }
            last = p;
        }
        None
    }

    /// Flyging i redigeringen: rett dit du ser, gjennom alt; mellomrom
    /// stiger, C synker, Shift går fort.
    pub(super) fn fly(&mut self, input: &Input, dt: f32) {
        let fwd = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(-fwd.z, 0.0, fwd.x);
        let mut wish = Vec3::ZERO;
        for (on, d) in [(input.forward, fwd), (input.back, -fwd), (input.right, right), (input.left, -right), (input.jump, Vec3::Y), (input.crouch, -Vec3::Y)] {
            if on && !self.paused {
                wish += d;
            }
        }
        let speed = if input.sprint { FLY_FAST } else { FLY };
        let mut pos = self.body.pos + wish.normalize_or_zero() * speed * dt;
        // Ikke under bakken (krypten nås med døra, ikke ved å fly ned).
        pos.y = pos.y.max(terrain::height(pos.x, pos.z) - 0.5);
        self.body.pos = pos;
        self.body.vel_y = 0.0;
    }

    /// Det redigeringen har endret, til bildet: de faste instansene som ikke
    /// lenger står slik (skjules), og instansene som står et nytt sted.
    pub(super) fn edit_instances(&self) -> (Vec<(&'static str, usize)>, Vec<ModelInstance>) {
        let (Some(e), Some(d)) = (&self.edit, &self.edit_data) else { return (vec![], vec![]) };
        let mut base: HashMap<(String, [i32; 12]), Vec<(&'static str, usize)>> = HashMap::new();
        for (m, n, t, _) in &d.base {
            base.entry(pose_key(m, t)).or_default().push((*m, *n));
        }
        let mut shown = vec![];
        let scene = e.scene();
        for pl in &scene.placements {
            for (t, tint) in pl.transforms.iter().zip(&pl.tints) {
                match base.get_mut(&pose_key(pl.model, t)).and_then(|v| v.pop()) {
                    Some(_) => {}
                    None => shown.push(ModelInstance { model: pl.model, transform: *t, skin: None, flash: 0.0, tint: (*tint != Vec3::ONE).then_some(*tint) }),
                }
            }
        }
        let hidden = base.into_values().flatten().collect();
        (hidden, shown)
    }
}
