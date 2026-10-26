//! Tag-midden (del av `mobs.rs`; riggen og klippene i
//! `tools/models/mob_rigid.py`, `mite`). Svak og rask, og i flokk:
//!
//! - **Ringen**: flokken løper ikke rett på deg, men hver tar sin plass i
//!   en ring rundt deg (`RING`, plassen etter nummeret i flokken) og piler
//!   inn derfra (`DART`).
//! - **Fast**: biter den, henger den seg i ankelen din (`latch`) og gnager
//!   (`GNAW`), og hver midd som henger i deg, gjør deg tregere
//!   (`Mobs::drag`).
//! - **Rist dem av**: et hopp eller en unnamanøver kaster dem av
//!   (`Mobs::shake_off`), og de ligger svimle et øyeblikk (`SHAKEN`).

use super::*;

/// Ringen rundt deg flokken tar plass i, og fra hvor langt unna.
const RING: f32 = 1.7;
const RING_FROM: f32 = 6.0;
/// Så nær plassen sin må den stå før den piler inn.
const SLOT_NEAR: f32 = 0.6;
/// Farten i pilen inn, og så nær den kommer.
const DART: f32 = 8.0;
const DART_FROM: f32 = 10.0 / FPS;
const CLING: f32 = 0.35;
/// Gnager så ofte, med så stor del av skaden.
const GNAW: f32 = 0.9;
const GNAW_PART: f32 = 0.5;
/// Ristet av: kastet så langt og svimmel så lenge.
const THROW: f32 = 4.0;
pub const SHAKEN: f32 = 1.0;
/// Hver midd i ankelen tar så mye av farten din, ned til `DRAG_MIN`.
const DRAG_EACH: f32 = 0.18;
const DRAG_MIN: f32 = 0.4;

impl Mob {
    /// Fast i ankelen din (Tag-midden).
    pub fn latched(&self) -> bool {
        self.latch.is_some()
    }
}

/// Der midden går mot: plassen i ringen rundt deg når den er nær nok (den
/// første ledige med gulv, fra plassen etter nummeret og rundt), ellers rett
/// mot deg.
pub(super) fn goal(i: usize, m: &Mob, player: Vec3, dist: f32, crypt: &Crypt) -> Vec3 {
    if m.kind != Kind::TagMite || dist > RING_FROM {
        return player;
    }
    let a0 = i as f32 * 2.399_963;
    (0..12)
        .map(|k| {
            let a = a0 + k as f32 * std::f32::consts::TAU / 12.0;
            player + Vec3::new(a.cos(), 0.0, a.sin()) * RING
        })
        .find(|q| crypt.walkable(*q))
        .unwrap_or(player)
}

/// Står midden på plassen sin i ringen (de andre slagene alltid): først da
/// piler den inn.
pub(super) fn in_place(i: usize, m: &Mob, player: Vec3, dist: f32, crypt: &Crypt) -> bool {
    m.kind != Kind::TagMite || goal(i, m, player, dist, crypt).with_y(0.0).distance(m.pos.with_y(0.0)) < SLOT_NEAR
}

/// Pilen inn: fra bilde 10 i bittet farer midden mot ankelen din.
pub(super) fn dart(m: &mut Mob, dt: f32, player: Vec3, crypt: &Crypt) {
    if m.kind != Kind::TagMite || !matches!(m.state, State::Attack { blow: Blow::Melee, landed: false }) || m.t < DART_FROM {
        return;
    }
    let to = (player - m.pos).with_y(0.0);
    let left = to.length() - CLING;
    if left <= 0.0 {
        return;
    }
    let q = m.pos + to.normalize_or_zero() * (DART * dt).min(left);
    if crypt.walkable(q) {
        m.pos = Vec3::new(q.x, crypt.height_at(q + Vec3::Y * 0.3), q.z);
    }
}

/// Bittet landet: midden henger seg i ankelen din.
pub(super) fn bite(m: &mut Mob, player: Vec3, clips: &Clips) {
    if m.kind != Kind::TagMite {
        return;
    }
    let off = (m.pos - player).with_y(0.0).normalize_or_zero();
    m.latch = Some((off * CLING, 0.0));
    if let Some(a) = m.anim.as_mut() {
        a.play(clips.latch.unwrap_or(clips.idle), true, 0.08);
    }
}

/// Fast i ankelen: følger føttene dine og gnager. `true` når den henger fast.
pub(super) fn cling(i: usize, m: &mut Mob, dt: f32, player: Vec3, stats: &Stats, out: &mut Outcome) -> bool {
    let Some((off, t)) = m.latch else { return false };
    let t = t + dt;
    m.pos = player + off;
    m.yaw = (-off.x).atan2(-off.z);
    m.state = State::Idle;
    if t >= GNAW {
        out.bites.push((i, stats.damage * GNAW_PART * if m.elite { 1.8 } else { 1.0 }, m.pos));
        out.sounds.push(("spit_hit", m.pos));
        m.latch = Some((off, t - GNAW));
    } else {
        m.latch = Some((off, t));
    }
    true
}

impl Mobs {
    /// Hvor mye av farten din som er igjen med middene i ankelen (ingen
    /// når hammerslaget har trykket deg ned, `mob_press.rs`).
    pub fn drag(&self) -> f32 {
        if self.pressed() {
            return 0.0;
        }
        (1.0 - DRAG_EACH * self.mobs.iter().filter(|m| m.latched()).count() as f32).max(DRAG_MIN)
    }

    /// Et hopp eller en unnamanøver kaster middene av deg: de flyr ut og
    /// ligger svimle et øyeblikk. Gir hvor mange.
    pub fn shake_off(&mut self, player: Vec3) -> usize {
        let mut n = 0;
        for i in 0..self.mobs.len() {
            let Some((off, _)) = self.mobs[i].latch else { continue };
            n += 1;
            let m = &mut self.mobs[i];
            m.latch = None;
            m.pos = player + off;
            m.knock = off.normalize_or_zero() * THROW;
            self.stagger(i, SHAKEN);
        }
        n
    }
}
