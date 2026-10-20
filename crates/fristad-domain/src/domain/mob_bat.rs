//! Alert-flaggermusa (del av `mobs.rs`): den eneste som flyr. Den henger
//! `HOVER` over gulvet, for høyt for sverdet (`SWING_TOP`), og har to ting
//! ingen annen fiende har (eieren 20. oktober: «alle monster angriper
//! ulikt»):
//!
//! - **Varselskriket**: når den får øye på deg, henger den stille og skriker
//!   (klippet `alert`, øyet blinker ravgult, `Mob::shrieking`), og alle
//!   fiender innen `SHRIEK_RADIUS` i samme nivå våkner, også de i bakhold,
//!   snur seg mot deg og kommer (`Mobs::wake`).
//! - **Stupet**: den løfter seg, legger vingene bakover og stuper i en bue
//!   ned til brystet ditt, biter (bilde 18 av 36) og slår seg opp igjen
//!   bakover til flyhøyden (`fly`). Bare nede kan du hugge den; en parade
//!   slår den i gulvet, der den spreller til den kommer seg opp igjen
//!   (`GROUNDED`). Flokken stuper etter tur (`waits`), og mellom stupene
//!   kretser den rundt deg.

use super::*;

/// Flyhøyden over gulvet (føttene i modellen).
pub const HOVER: f32 = 2.3;
/// Over denne høyden over føttene dine når ikke sverdet den.
pub const SWING_TOP: f32 = 1.85;
/// Skriket vekker alt innen så mange meter i samme nivå.
pub const SHRIEK_RADIUS: f32 = 14.0;
/// Bittet treffer innen så langt (vannrett) når stupet er nede.
pub(super) const BITE: f32 = 1.3;
/// Høyden over gulvet ditt den biter i (føttene dens; kroppen er 0,25 høy).
const BITE_AT: f32 = 0.95;
/// På gulvet etter en parade.
const GROUNDED: f32 = 0.02;
/// Opp og ned ellers, og farten i stupet.
const CLIMB: f32 = 2.4;
const DIVE_SPEED: f32 = 7.5;
/// Så nær stuper den (vannrett, fra deg).
const DIVE_END: f32 = 0.55;
/// Farten den kretser rundt deg med mens den venter på neste stup.
const CIRCLE: f32 = 1.3;

impl Mob {
    /// Høyden over gulvet den flyr i (0 for alle andre).
    pub fn air(&self) -> f32 {
        self.air
    }

    /// For høyt oppe for sverdet ditt (`feet` er føttene dine).
    pub fn out_of_reach(&self, feet: Vec3) -> bool {
        self.kind == Kind::AlertBat && self.pos.y + self.air - feet.y > SWING_TOP
    }

    /// Midt på kroppen (skadetallene og rusket).
    pub fn body_mid(&self) -> Vec3 {
        if self.kind == Kind::AlertBat {
            self.pos + Vec3::Y * (self.air + 0.12 * self.scale())
        } else {
            self.pos + Vec3::Y * self.scale()
        }
    }

    /// Skriker den nå: hvor sterkt (0..1) og øyet (det ravgule blinket).
    pub fn shrieking(&self) -> Option<(f32, Vec3)> {
        if self.kind != Kind::AlertBat || self.state != State::Alert {
            return None;
        }
        let k = (std::f32::consts::PI * (self.t / ALERT_LEN).clamp(0.0, 1.0)).sin();
        let fwd = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        Some((k * (0.75 + 0.25 * (self.t * 40.0).sin()), self.pos + Vec3::Y * (self.air + 0.16) + fwd * 0.13))
    }
}

/// Når den får øye på deg: skriket (lyden, og stedet til `Mobs::wake`).
pub(super) fn notice(m: &Mob, shrieks: &mut Vec<(Vec3, usize)>) {
    if m.kind == Kind::AlertBat {
        shrieks.push((m.pos, m.level));
    }
}

/// Flokken stuper etter tur: venter når en annen flaggermus nær den er i et
/// stup som ikke har bitt ennå.
pub(super) fn waits(m: &Mob, before: &[Mob], after: &[Mob]) -> bool {
    m.kind == Kind::AlertBat
        && before.iter().chain(after).any(|o| o.kind == Kind::AlertBat && o.alive() && o.level == m.level && o.pos.distance(m.pos) < 5.0 && matches!(o.state, State::Attack { landed: false, .. }))
}

/// Et steg vannrett langs gulvet, om det er gulv der.
fn glide(m: &mut Mob, step: Vec3, crypt: &Crypt) {
    let q = m.pos + step.with_y(0.0);
    if crypt.walkable(q) {
        m.pos = Vec3::new(q.x, crypt.height_at(q + Vec3::Y * 0.3), q.z);
    }
}

/// Flyhøyden og stupet, hvert bilde før tilstanden. `player` er føttene dine.
pub(super) fn fly(m: &mut Mob, dt: f32, player: Vec3, crypt: &Crypt, stats: &Stats) {
    if m.kind != Kind::AlertBat {
        return;
    }
    let toward = |m: &mut Mob, want: f32, rate: f32| m.air += (want - m.air).clamp(-rate * dt, rate * dt);
    let to = (player - m.pos).with_y(0.0);
    let dist = to.length();
    let dir = to.normalize_or_zero();
    match m.state {
        State::Dying | State::Gone => toward(m, 0.0, 5.0),
        // Slått i gulvet av en parade: spreller der til lamslåingen er over.
        State::Hit if m.stun > 0.5 => toward(m, GROUNDED, 7.0),
        State::Attack { blow: Blow::Melee, landed } => {
            let (hit_at, _) = Blow::Melee.timing(stats);
            let bite = (player.y + BITE_AT - m.pos.y).clamp(0.2, HOVER);
            if m.t < 0.3 * hit_at {
                // Løfter seg og trekker seg litt bakover før stupet.
                toward(m, HOVER + 0.3, CLIMB);
                glide(m, -dir * 1.0 * dt, crypt);
            } else if !landed {
                // Stupet: en bue ned mot brystet ditt, raskere jo nærmere bittet.
                let left = (hit_at - m.t).max(dt);
                let k = (dt / left).min(1.0);
                m.air += (bite - m.air) * k;
                let step = ((dist - DIVE_END) * k).clamp(-DIVE_SPEED * dt, DIVE_SPEED * dt);
                glide(m, dir * step, crypt);
            } else {
                // Slår seg opp igjen og bakover, bort fra deg.
                toward(m, HOVER, CLIMB * 1.2);
                if m.t < hit_at + 0.45 {
                    glide(m, -dir * 2.6 * dt, crypt);
                }
            }
        }
        _ => {
            toward(m, HOVER, CLIMB);
            // Kretser rundt deg mens den venter på neste stup.
            if m.aware && m.state == State::Idle && dist < stats.range + 0.6 && dist > 0.5 {
                let side = if (m.phase * 0.35).sin() >= 0.0 { 1.0 } else { -1.0 };
                glide(m, Vec3::new(-dir.z, 0.0, dir.x) * side * CIRCLE * dt, crypt);
            }
        }
    }
}

impl Mobs {
    /// Skrikene vekker alle i samme nivå innen `SHRIEK_RADIUS`, også de i
    /// bakhold: de snur seg mot deg (klippet `alert`) og kommer. De som
    /// vekkes, skriker ikke selv.
    pub(super) fn wake(&mut self, shrieks: &[(Vec3, usize)]) {
        for &(at, level) in shrieks {
            for m in self.mobs.iter_mut().filter(|m| m.alive() && m.level == level && !m.aware) {
                if m.pos.with_y(0.0).distance(at.with_y(0.0)) > SHRIEK_RADIUS || (m.pos.y - at.y).abs() > 6.0 {
                    continue;
                }
                m.dormant = false;
                m.aware = true;
                m.path.clear();
                if let (Some(al), Some(a)) = (self.clips[m.kind as usize].alert, m.anim.as_mut()) {
                    m.state = State::Alert;
                    m.t = 0.0;
                    a.play(al, false, 0.1);
                }
            }
        }
    }

    /// Øynene som blinker ravgult i skriket (sted, styrke 0..1), til lysene.
    pub fn shrieks(&self) -> Vec<(Vec3, f32)> {
        self.mobs.iter().filter_map(|m| m.shrieking().map(|(k, at)| (at, k))).collect()
    }

    /// Sett ut en fiende som ikke har sett deg, sovende i bakhold (bare
    /// nærmere enn `near` vekker den) eller våken (til testene).
    #[cfg(any(test, feature = "testing"))]
    pub fn spawn_unaware(&mut self, kind: Kind, p: Vec3, near: Option<f32>) -> usize {
        self.spawn_at(kind, p, Some(0.0), false, vec![], near.map(Ambush::Near), self.level);
        self.mobs.len() - 1
    }
}
