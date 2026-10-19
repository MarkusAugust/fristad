//! Markørene inputen og Toast spytter (`Bolt`, planen del 13e): ladningen
//! i munnen før skuddet, flukten, treffet og det som tegnes (markøren med en
//! hale av mindre kopier og gløden i munnen) og lyser.

use super::{Blow, Kind, Mob, Mobs, Outcome, State, BOLT_MODEL};
use crate::domain::scene::ModelInstance;
use crate::domain::world::crypt::Crypt;
use glam::{Affine3A, Quat, Vec3};

const MAX_BOLTS: usize = 24;
/// Halen av gnister bak hver markør: så mange små kopier av modellen.
const BOLT_TRAIL: usize = 4;
/// Markørene i lufta, halene og gløden i munnen før skuddet (`reserve`).
pub(super) const BOLT_INSTANCES: usize = MAX_BOLTS * (1 + BOLT_TRAIL) + 16;
pub(super) const BOLT_SPEED: f32 = 11.0;
const BOLT_LIFE: f32 = 2.2;

/// En markør som flyr: hvem som spyttet den, hvor den er og hvor den skal.
#[derive(Clone, Debug)]
pub struct Bolt {
    pub owner: usize,
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub damage: f32,
}

impl Mob {
    /// Munnen, der markøren spyttes fra: inputen og Toast er lave (rundt
    /// en halv meter), de andre står oppreist.
    pub fn mouth(&self) -> Vec3 {
        let s = self.scale();
        let (up, ahead) = match self.kind {
            Kind::Input | Kind::Toast => (0.32, 0.5),
            _ => (1.3, 0.6),
        };
        self.pos + Vec3::Y * up * s + Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos()) * ahead * s
    }

    /// Lader den et skudd: hvor langt ladningen er kommet (0..1) og munnen.
    pub fn charging(&self) -> Option<(f32, Vec3)> {
        match self.state {
            State::Attack { blow: Blow::Spit, landed: false } => {
                let (hit_at, _) = Blow::Spit.timing(&self.kind.stats());
                Some(((self.t / hit_at).clamp(0.0, 1.0), self.mouth()))
            }
            _ => None,
        }
    }
}

impl Mobs {
    /// Markørene i lufta med halen av gnister bak, og gløden som vokser i
    /// munnen på dem som lader et skudd (varselet før spyttet).
    pub(super) fn bolt_instances(&self, time: f32) -> Vec<ModelInstance> {
        let mut out = vec![];
        let aligned = |dir: Vec3, spin: f32| {
            let yaw = dir.x.atan2(dir.z);
            let pitch = dir.y.clamp(-1.0, 1.0).asin();
            Quat::from_rotation_y(yaw - std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(pitch) * Quat::from_rotation_x(spin)
        };
        for (k, b) in self.bolts.iter().enumerate() {
            let dir = b.vel.normalize_or(Vec3::X);
            // Vokser ut av munnen de første hundredelene.
            let pop = (0.4 + b.age * 6.0).min(1.0);
            out.push(ModelInstance { model: BOLT_MODEL, transform: Affine3A::from_rotation_translation(aligned(dir, time * 12.0 + k as f32), b.pos) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0, tint: None });
            // Gnistene bak: mindre og mindre, og bare så langt den har flydd.
            let flown = b.age * b.vel.length();
            for j in 1..=BOLT_TRAIL {
                let back = 0.38 * j as f32;
                if back > flown {
                    break;
                }
                let wob = Vec3::new((time * 17.0 + j as f32 * 2.1).sin(), (time * 13.0 + j as f32 * 1.3).cos(), 0.0) * 0.04 * j as f32;
                let size = 0.55 * (1.0 - j as f32 / (BOLT_TRAIL + 1) as f32);
                let at = b.pos - dir * back + wob;
                out.push(ModelInstance { model: BOLT_MODEL, transform: Affine3A::from_rotation_translation(aligned(dir, time * 9.0 + j as f32), at) * Affine3A::from_scale(Vec3::splat(size)), skin: None, flash: 0.0, tint: None });
            }
        }
        for m in &self.mobs {
            if let Some((charge, at)) = m.charging() {
                // Gløden i munnen vokser og dirrer til skuddet, med halen
                // bakover, så den ses som en klump forfra.
                let throb = 1.0 + 0.18 * (time * 38.0).sin();
                let rot = aligned(Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos()), time * 7.0);
                out.push(ModelInstance { model: BOLT_MODEL, transform: Affine3A::from_rotation_translation(rot, at) * Affine3A::from_scale(Vec3::splat((0.25 + 0.75 * charge) * 0.7 * throb)), skin: None, flash: 0.0, tint: None });
            }
        }
        out.truncate(BOLT_INSTANCES);
        out
    }

    /// Lyset fra markørene i lufta og fra munnene som lader (sted, styrke
    /// 0..1), til lysene i bildet.
    pub fn glows(&self) -> Vec<(Vec3, f32)> {
        let mut out: Vec<(Vec3, f32)> = self.bolts.iter().map(|b| (b.pos, 1.0)).collect();
        out.extend(self.mobs.iter().filter_map(|m| m.charging().map(|(c, at)| (at, 0.3 + 0.7 * c))));
        out
    }

    /// De nye markørene tas med, og alle flyr rett fram til de treffer deg
    /// (brystet), en vegg (utenfor gulvet) eller gulvet: der spruter de.
    pub(super) fn fly_bolts(&mut self, new: Vec<Bolt>, dt: f32, player: Vec3, crypt: &Crypt, out: &mut Outcome) {
        for b in new {
            if self.bolts.len() < MAX_BOLTS {
                self.bolts.push(b);
            }
        }
        let chest = player + Vec3::Y * 0.9;
        for b in &mut self.bolts {
            b.pos += b.vel * dt;
            b.age += dt;
            if (b.pos - chest).length() < 0.6 {
                out.bites.push((b.owner, b.damage, b.pos - b.vel));
                out.impacts.push((b.pos, b.vel.normalize_or_zero()));
                b.age = BOLT_LIFE + 1.0;
            } else if crypt.floor_under(b.pos).is_none_or(|f| b.pos.y - f < 0.05) {
                out.impacts.push((b.pos - b.vel * dt, b.vel.normalize_or_zero()));
                b.age = BOLT_LIFE + 1.0;
            }
        }
        self.bolts.retain(|b| b.age < BOLT_LIFE);
    }
}
