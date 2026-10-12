//! Portalen ved sørporten og virvelen: hvor portalen tar deg (der du sto
//! da du dro hjem med H, eller inngangen til det dypeste nivået når du
//! døde), klokkene som driver ringen, runene, virvlene og splintene, og
//! inngangene til nivåene du har nådd.

use super::town;
use crate::domain::scene::Light;
use glam::{Affine3A, Quat, Vec3};

/// Sekunder ringen bruker på å stige opp av bakken før virvelen åpner seg.
pub const PORTAL_RISE: f32 = 1.4;

#[derive(Clone, Debug)]
pub struct Portal {
    /// Dit den tar deg (føttene, retningen og nivået) når den er åpen.
    pub dest: Option<(Vec3, f32, usize)>,
    /// Sekunder siden den åpnet seg.
    pub t: f32,
    /// Sekunder siden du sist reiste gjennom virvelen (synsfeltet strekkes).
    pub warp_t: f32,
    /// Det første stedet du sto på gulvet i hvert nivå.
    pub entries: Vec<Option<(Vec3, f32)>>,
    /// Om siktemålet og kolliderne står ute nå.
    shown: bool,
}

impl Portal {
    pub fn new(levels: usize) -> Self {
        Self {
            dest: None,
            t: 0.0,
            warp_t: 10.0,
            entries: vec![None; levels],
            shown: true,
        }
    }

    pub fn open(&mut self, dest: (Vec3, f32, usize)) {
        self.dest = Some(dest);
        self.t = 0.0;
    }

    pub fn tick(&mut self, dt: f32) {
        self.warp_t += dt;
        if self.dest.is_some() {
            self.t += dt;
        }
    }

    /// Du står på gulvet i nivå `k`: første gang er det inngangen dit.
    pub fn enter(&mut self, k: usize, feet: Vec3, yaw: f32) {
        if let Some(e) = self.entries.get_mut(k) {
            e.get_or_insert((feet, yaw));
        }
    }

    /// Inngangen til det dypeste nivået du har nådd.
    pub fn deepest(&self) -> Option<(Vec3, f32, usize)> {
        self.entries
            .iter()
            .enumerate()
            .rev()
            .find_map(|(f, e)| e.map(|(p, y)| (p, y, f)))
    }

    /// Om portalen skal stå ute nå, når det har endret seg siden sist.
    pub fn shown_changed(&mut self) -> Option<bool> {
        let show = self.dest.is_some();
        (show != self.shown).then(|| {
            self.shown = show;
            show
        })
    }

    /// Synsfeltet: 72°, strukket ut og tilbake når du reiser gjennom
    /// virvelen.
    pub fn fov(&self) -> f32 {
        let t = self.warp_t / 1.3;
        if t < 1.0 {
            72.0 + 46.0 * (t * std::f32::consts::PI).sin().powi(2) * (1.0 - t * 0.3)
        } else {
            72.0
        }
    }

    /// Midten av virvelen, når ringen står i `pose`.
    pub fn center(pose: Affine3A) -> Vec3 {
        pose.transform_point3(Vec3::Y * town::PORTAL_CENTER)
    }

    /// Den åpne portalen: runene, virvelen (to lag som dreier hver sin vei
    /// og pulserer) og splintene som svever rundt ringen. Ringen stiger
    /// opp av bakken med en liten skjelving, så vokser virvelen fram med
    /// litt overskyting.
    pub fn poses(&self, pose: Affine3A, time: f32) -> Vec<(&'static str, Affine3A)> {
        if self.dest.is_none() {
            return vec![];
        }
        let [runes, swirl, shard] = town::PORTAL_PARTS;
        let r = (self.t / PORTAL_RISE).clamp(0.0, 1.0);
        let rise = 1.0 - (1.0 - r).powi(3);
        let shake = if r < 1.0 {
            (self.t * 47.0).sin() * 0.02 * (1.0 - r)
        } else {
            0.0
        };
        let ring = pose * Affine3A::from_translation(Vec3::new(shake, -3.4 * (1.0 - rise), 0.0));
        let mut out = vec![(town::PORTAL_RING, ring)];
        let x = self.opening();
        if x <= 0.0 {
            return out;
        }
        let grow = (1.0 + 2.2 * (x - 1.0).powi(3) + 1.2 * (x - 1.0).powi(2)).max(0.001);
        let t = time;
        let center = pose * Affine3A::from_translation(Vec3::Y * town::PORTAL_CENTER);
        out.push((runes, pose));
        let s1 = grow * (1.0 + 0.035 * (t * 3.1).sin());
        out.push((
            swirl,
            center * Affine3A::from_rotation_z(-t * 1.7) * Affine3A::from_scale(Vec3::splat(s1)),
        ));
        let s2 = grow * 0.8 * (1.0 + 0.05 * (t * 2.3 + 1.0).sin());
        out.push((
            swirl,
            center
                * Affine3A::from_translation(Vec3::Z * -0.06)
                * Affine3A::from_rotation_z(t * 1.1 + 1.0)
                * Affine3A::from_scale(Vec3::splat(s2)),
        ));
        for k in 0..town::PORTAL_SHARDS {
            let a = t * 0.55 + k as f32 * std::f32::consts::TAU / town::PORTAL_SHARDS as f32;
            let wobble = (t * 1.3 + k as f32 * 2.1).sin();
            let at = Vec3::new(
                a.cos() * (2.05 + 0.1 * wobble),
                1.95 + a.sin() * 1.6,
                0.25 + 0.2 * (t * 0.9 + k as f32).sin(),
            );
            let spin =
                Quat::from_rotation_y(t * 2.0 + k as f32) * Quat::from_rotation_x(0.5 * wobble);
            out.push((
                shard,
                pose * Affine3A::from_scale_rotation_translation(
                    Vec3::splat(grow * (0.8 + 0.25 * (k % 3) as f32)),
                    spin,
                    at,
                ),
            ));
        }
        out
    }

    /// Hvor langt virvelen har åpnet seg (0..1).
    fn opening(&self) -> f32 {
        ((self.t - PORTAL_RISE) / 1.6).clamp(0.0, 1.0)
    }

    /// Lyset fra den åpne portalen: fiolett, pulserende.
    pub fn light(&self, pose: Affine3A, time: f32) -> Option<Light> {
        self.dest?;
        let t = time;
        let front = pose.transform_vector3(Vec3::Z);
        Some(Light {
            pos: Self::center(pose) + front * 0.6,
            radius: 11.0,
            color: Vec3::new(0.55, 0.45, 1.0),
            intensity: 2.4 * self.opening() * (0.85 + 0.15 * (t * 4.7).sin() * (t * 1.9).cos()),
            always: true,
            flicker: 0.0,
            room: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inngangen_huskes_foerste_gang_og_den_dypeste_vinner() {
        let mut p = Portal::new(3);
        p.enter(0, Vec3::X, 0.0);
        p.enter(0, Vec3::Y, 1.0);
        p.enter(1, Vec3::Z, 2.0);
        assert_eq!(p.entries[0], Some((Vec3::X, 0.0)));
        assert_eq!(p.deepest(), Some((Vec3::Z, 2.0, 1)));
    }

    #[test]
    fn ringen_stiger_foer_virvelen_aapner_seg() {
        let mut p = Portal::new(1);
        assert_eq!(p.shown_changed(), Some(false));
        p.open((Vec3::ZERO, 0.0, 0));
        assert_eq!(p.shown_changed(), Some(true));
        assert_eq!(p.poses(Affine3A::IDENTITY, 0.0).len(), 1, "bare ringen");
        p.tick(PORTAL_RISE + 2.0);
        assert!(p.poses(Affine3A::IDENTITY, 0.0).len() > 3);
        assert!(p
            .light(Affine3A::IDENTITY, 0.0)
            .is_some_and(|l| l.intensity > 1.0));
    }
}
