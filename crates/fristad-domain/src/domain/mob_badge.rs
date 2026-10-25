//! Badge-billen (del av `mobs.rs`; riggen og klippene i
//! `tools/models/mob_rigid.py`, `beetle`). Kuppelskallet tar det meste
//! forfra (`Stats.armor`), og et stykke unna stormer den:
//!
//! - **Stangeløpet**: den skraper med forbeina og senker hornet
//!   (`charge_wind`, `WIND`), og løper så rett fram i den retningen den
//!   sto (`charge`, `RUSH`) til den treffer deg (`RAM` ganger skaden) eller
//!   har løpt fra seg (`RUSH_MAX`). Den svinger ikke underveis: gå til
//!   siden.
//! - **Velt**: løper den i en mur, velter den over på ryggen og spreller
//!   (`flipped`) i `FLIP_SECS`. Da er buken åpen, og den tar `BELLY` ganger
//!   så mye.

use super::*;

/// Den stormer når du er så langt unna (vannrett) og sikten er fri.
const CHARGE_FROM: f32 = 3.0;
const CHARGE_TO: f32 = 9.0;
/// Klippet `charge_wind` (14 bilder): skraper og senker hornet.
const WIND: f32 = 14.0 / FPS;
/// Farten i løpet, og hvor lenge den løper.
const RUSH: f32 = 7.0;
const RUSH_MAX: f32 = 1.6;
/// Treffer deg innen så langt foran seg, med så mange ganger skaden.
const RAM_REACH: f32 = 0.9;
const RAM: f32 = 1.8;
/// Ligger velt så lenge, og tar så mange ganger skaden da.
pub const FLIP_SECS: f32 = 2.5;
pub const BELLY: f32 = 2.0;

impl Mob {
    /// Velt på ryggen (Badge-billen etter å ha løpt i muren).
    pub fn flipped(&self) -> bool {
        self.flipped
    }
}

/// Billen som ser deg et stykke unna med fri sikt, begynner stangeløpet
/// (`Some` med klippet).
pub(super) fn far_blow(m: &Mob, dist: f32, clear: bool, clips: &Clips) -> Option<(Blow, usize)> {
    (m.kind == Kind::BadgeBeetle && m.cooldown <= 0.0 && clear && (CHARGE_FROM..CHARGE_TO).contains(&dist)).then(|| (Blow::Charge, clips.charge_wind.unwrap_or(clips.idle)))
}

/// Stangeløpet, hvert bilde (`true` når billen er i det). `to_player` er
/// vannrett mot føttene dine.
pub(super) fn rush(i: usize, m: &mut Mob, dt: f32, to_player: Vec3, crypt: &Crypt, stats: &Stats, clips: &Clips, out: &mut Outcome) -> bool {
    let State::Attack { blow: Blow::Charge, landed } = m.state else { return false };
    if !landed {
        // Skraper og sikter: snur seg mot deg til løpet begynner.
        if to_player.length() > 0.01 {
            let want = to_player.x.atan2(to_player.z);
            let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            m.yaw += diff * (dt * 6.0).min(1.0);
        }
        if m.t >= WIND {
            m.state = State::Attack { blow: Blow::Charge, landed: true };
            m.t = 0.0;
            if let Some(a) = m.anim.as_mut() {
                a.play(clips.charge.unwrap_or(clips.walk), true, 0.06);
            }
            out.sounds.push(("windup_big", m.pos));
        }
        return true;
    }
    let dir = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
    let ahead = to_player.dot(dir);
    if ahead > 0.0 && ahead <= RAM_REACH && (to_player - dir * ahead).length() < 0.6 {
        out.bites.push((i, stats.damage * RAM * if m.elite { 1.8 } else { 1.0 }, m.pos));
        out.sounds.push(("hit_heavy", m.pos));
        stop(m, stats, clips);
        return true;
    }
    let q = m.pos + dir * RUSH * dt;
    if !crypt.walkable(q) {
        // I muren: velter over på ryggen og spreller.
        m.state = State::Hit;
        m.t = 0.0;
        m.stun = FLIP_SECS;
        m.flipped = true;
        m.cooldown = FLIP_SECS + stats.cooldown;
        if let Some(a) = m.anim.as_mut() {
            a.play(clips.flipped.unwrap_or(clips.stagger.unwrap_or(clips.hit)), true, 0.1);
        }
        out.sounds.push(("clang", m.pos));
        out.sounds.push(("crack", m.pos));
        return true;
    }
    m.pos = Vec3::new(q.x, crypt.height_at(q + Vec3::Y * 0.3), q.z);
    m.stride += RUSH * dt;
    if m.stride > 0.8 {
        m.stride = 0.0;
        out.sounds.push(("mob_step", m.pos));
    }
    if m.t >= RUSH_MAX {
        stop(m, stats, clips);
    }
    true
}

fn stop(m: &mut Mob, stats: &Stats, clips: &Clips) {
    m.state = State::Idle;
    m.cooldown = stats.cooldown;
    if let Some(a) = m.anim.as_mut() {
        a.play(clips.idle, true, 0.15);
    }
}
