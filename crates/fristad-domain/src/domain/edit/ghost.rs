//! Å sette ut raskt: fantomkopien som følger musa før slipp (og reglene på
//! den som farge), og festet til rutenettet og vinkelen.

use super::editor::Editor;
use super::rules;
use crate::domain::world::layout::SolidDef;
use glam::{Affine3A, Quat, Vec3};

/// Rutenettet og vinkelen det festes til (eieren: 0,5 m og 15°).
pub const GRID: f32 = 0.5;
pub const ANGLE: f32 = std::f32::consts::PI / 12.0;

/// Et tall festet til nærmeste `step`.
pub fn snap(v: f32, step: f32) -> f32 {
    (v / step).round() * step
}

/// En pose festet: x og z til rutenettet og dreiningen til vinkelen (høyden
/// står, den kommer fra det tingen står på).
pub fn snap_pose(pose: Affine3A) -> Affine3A {
    let (scale, rot, pos) = pose.to_scale_rotation_translation();
    let (yaw, _, _) = rot.to_euler(glam::EulerRot::YXZ);
    let rot = Quat::from_rotation_y(snap(yaw, ANGLE) - yaw) * rot;
    Affine3A::from_scale_rotation_translation(scale, rot, Vec3::new(snap(pos.x, GRID), pos.y, snap(pos.z, GRID)))
}

impl Editor {
    /// Kollideren en ny ting får: en boks rundt modellen når den er stor nok
    /// til at regelen krever en (over 60 cm høy og 30 cm bred), så det som
    /// settes ut fra katalogen er grønt med en gang.
    pub(super) fn default_solids(&self, model: &str) -> Vec<SolidDef> {
        match self.scene().bounds(model) {
            Some((lo, hi)) if hi.y - lo.y >= 0.6 && (hi.x - lo.x).max(hi.z - lo.z) >= 0.3 => vec![SolidDef::Cuboid { center: (lo + hi) * 0.5, size: hi - lo, yaw: 0.0 }],
            _ => vec![],
        }
    }

    /// Det som ville vært galt om `model` ble satt ut med `pose`, fort nok til
    /// hver musebevegelse (uten å bygge byen på nytt): står inni en kollider
    /// (en vegg, et hus, en annen ting), svever eller står nede, eller er
    /// ikke godkjent ennå. Tomt er grønt.
    pub fn ghost_breaches(&self, model: &str, pose: &Affine3A) -> Vec<String> {
        let mut out = vec![];
        let Some((lo, hi)) = self.scene().bounds(model) else {
            return vec![format!("{model} finnes ikke")];
        };
        if self.approved.as_ref().is_some_and(|ok| !ok.contains(model)) {
            out.push(format!("{model} er ikke godkjent ennå"));
        }
        let o = Vec3::from(pose.translation);
        let foot = rules::world_box(lo, hi, pose).0.y;
        let ground = self.surface_under(o.x, o.z, foot + 0.3, None);
        if foot - ground > 0.08 {
            out.push(format!("svever {:.0} cm", (foot - ground) * 100.0));
        } else if ground - foot > 0.3 {
            out.push(format!("står {:.0} cm nede", (ground - foot) * 100.0));
        }
        // Inni noe: punkter inne i tingen (samme rutenett som regelen).
        let inset = Vec3::new(0.04, 0.0, 0.04).min((hi - lo) * 0.25);
        let (a, z) = (lo + inset, hi - inset);
        let inside = (0..27).any(|k| {
            let f = |i: usize, lo: f32, hi: f32| lo + (hi - lo) * [0.0, 0.5, 1.0][i];
            let fy = [0.25, 0.5, 0.75][(k / 3) % 3];
            let p = pose.transform_point3(Vec3::new(f(k % 3, a.x, z.x), lo.y + (hi.y - lo.y) * fy, f(k / 9, a.z, z.z)));
            self.scene().colliders.iter().any(|c| c.covers(p, -0.02))
        });
        if inside && !rules::joins(model) {
            out.push("står inni noe".into());
        }
        out
    }
}
