//! Fiendene som er en kjede av ledd (del av `mobs.rs`; riggene og klippene i
//! `tools/models/mob_chain.py`). Eieren 20. oktober: «alle monster angriper
//! ulikt».
//!
//! - **Den grusomme Comboboxen** slår halen rundt seg en hel omdreining:
//!   lavt langs gulvet (`attack`, hopp over den) eller høyt i brysthøyde
//!   (`lash_high`, dukk under). Pisken når rundt hele kroppen, så den
//!   treffer også bak den. Ellers velger den deg (`choose`: platene foran
//!   skrangler) og hamrer plata ned foran seg (`strike`) med dobbel skade.
//! - **Divider-ormen** biter når den er nær. Lenger unna graver den seg ned
//!   (`burrow`), går under gulvet der ingen ser eller treffer den, og dukker
//!   opp der du står (`emerge`) og biter: flytt deg når gulvet sprekker.

use super::*;

/// Ormen graver seg ned når du er så langt unna (vannrett).
const BURROW_FROM: f32 = 3.0;
const BURROW_TO: f32 = 14.0;
/// Under gulvet går den så fort, og dukker opp så nær deg, eller etter så
/// mange sekunder om den ikke kommer fram.
const UNDER_SPEED: f32 = 3.2;
const EMERGE_NEAR: f32 = 0.9;
const UNDER_MAX: f32 = 6.0;
/// Bittet når den dukker opp, treffer innen så langt.
const EMERGE_REACH: f32 = 1.4;
/// Slaget med plata gjør så mye mer enn den vanlige skaden.
const STRIKE: f32 = 2.0;

/// Hvordan et slag kan unngås (økta ser hva du gjør når det lander).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sweep {
    /// Langs gulvet: hopp over.
    Low,
    /// I brysthøyde: dukk under.
    High,
}

impl Blow {
    /// (sekundet slaget lander, klippets lengde) for kjedenes egne slag.
    pub(super) fn chain_timing(self) -> Option<(f32, f32)> {
        Some(match self {
            Blow::Lash { .. } => (20.0 / FPS, 36.0 / FPS),
            Blow::Choose => (6.0 / FPS, 24.0 / FPS),
            Blow::Strike => (15.0 / FPS, 30.0 / FPS),
            Blow::Burrow => (12.0 / FPS, 24.0 / FPS),
            Blow::Emerge => (12.0 / FPS, 30.0 / FPS),
            // Billens stangeløp styres av `mob_badge.rs`; lengden er bare et tak.
            Blow::Charge => (14.0 / FPS, 4.0),
            _ => return None,
        })
    }
}

impl Mob {
    /// Under gulvet (ormen): ingen ser, sikter på eller treffer den.
    pub fn under(&self) -> bool {
        self.under.is_some()
    }
}

/// Angrepet når du er innen rekkevidde: Comboboxen velger mellom pisken
/// lavt, pisken høyt og valget (`r` er terningen), de andre biter.
pub(super) fn close_blow(kind: Kind, r: f32, clips: &Clips) -> (Blow, usize) {
    match kind {
        Kind::Combobox if r < 0.35 => (Blow::Lash { high: false }, clips.attack),
        Kind::Combobox if r < 0.7 => (Blow::Lash { high: true }, clips.lash_high.unwrap_or(clips.attack)),
        Kind::Combobox => (Blow::Choose, clips.choose.unwrap_or(clips.idle)),
        _ => (Blow::Melee, clips.attack),
    }
}

/// Ormen som ser deg et stykke unna, graver seg ned (`Some` med klippet).
pub(super) fn far_blow(m: &Mob, dist: f32, clips: &Clips) -> Option<(Blow, usize)> {
    (m.kind == Kind::DividerWorm && m.cooldown <= 0.0 && (BURROW_FROM..BURROW_TO).contains(&dist)).then(|| (Blow::Burrow, clips.burrow.unwrap_or(clips.idle)))
}

/// Det som skjer når et av kjedenes slag lander. `dist` og `facing` som for bittet.
pub(super) fn land(i: usize, m: &Mob, blow: Blow, dist: f32, ahead: bool, stats: &Stats, out: &mut Outcome) {
    let dmg = stats.damage * if m.elite { 1.8 } else { 1.0 };
    let reach = stats.reach * if m.elite { 1.3 } else { 1.0 };
    match blow {
        // Halen feier rundt hele kroppen.
        Blow::Lash { high } => {
            out.sounds.push(("swing_heavy", m.pos));
            if dist <= reach {
                out.sweeps.push((i, dmg, m.pos, if high { Sweep::High } else { Sweep::Low }));
            }
        }
        Blow::Choose => out.sounds.push(("charge", m.pos)),
        Blow::Strike => {
            out.sounds.push(("slam", m.pos));
            if dist <= stats.range * 1.1 && ahead {
                out.bites.push((i, dmg * STRIKE, m.pos));
            }
        }
        Blow::Burrow => out.sounds.push(("crack", m.pos)),
        Blow::Emerge => {
            out.sounds.push(("crack", m.pos));
            if dist <= EMERGE_REACH {
                out.bites.push((i, dmg, m.pos));
            }
        }
        _ => {}
    }
}

/// Når klippet er ferdig: valget går over i slaget med plata, og ormen som
/// har gravd seg ned, er under gulvet. `true` når den har tatt over.
pub(super) fn finish(m: &mut Mob, blow: Blow, clips: &Clips, out: &mut Outcome) -> bool {
    match blow {
        Blow::Choose => {
            m.state = State::Attack { blow: Blow::Strike, landed: false };
            m.t = 0.0;
            if let Some(a) = m.anim.as_mut() {
                a.play(clips.strike.unwrap_or(clips.attack), false, 0.06);
            }
            out.sounds.push(("windup_big", m.pos));
            true
        }
        Blow::Burrow => {
            m.under = Some(0.0);
            m.state = State::Walk;
            m.path.clear();
            true
        }
        _ => false,
    }
}

/// Under gulvet: går mot deg og dukker opp ved deg (eller etter `UNDER_MAX`).
/// `true` når den fortsatt er under (resten av tilstanden står over).
pub(super) fn tunnel(m: &mut Mob, dt: f32, player: Vec3, dist: f32, crypt: &Crypt, stats: &Stats, clips: &Clips, out: &mut Outcome) -> bool {
    let Some(t) = m.under else { return false };
    let t = t + dt;
    if dist > EMERGE_NEAR && t < UNDER_MAX {
        m.under = Some(t);
        walk(m, player, UNDER_SPEED, dt, crypt, Vec3::ZERO, stats, clips, out);
        return true;
    }
    m.under = None;
    let to = (player - m.pos).with_y(0.0);
    if to.length() > 0.01 {
        m.yaw = to.x.atan2(to.z);
    }
    m.state = State::Attack { blow: Blow::Emerge, landed: false };
    m.t = 0.0;
    if let Some(a) = m.anim.as_mut() {
        a.play(clips.emerge.unwrap_or(clips.attack), false, 0.0);
    }
    // Gulvet sprekker før den bryter opp.
    out.sounds.push(("windup", m.pos));
    true
}
