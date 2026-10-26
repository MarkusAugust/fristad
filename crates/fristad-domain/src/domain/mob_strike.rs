//! Slagene mot fiendene (del av `mobs.rs`): skaden, panseret forfra,
//! buken på billen som ligger velt, rykket bakover, balansen som brister,
//! dødsfallet med tokens og nøkkelen, og lamslåingen etter en parade.

use super::*;

/// Det du plukker opp: tokens, og nøkler til portene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pickup {
    Token,
    Key,
}

/// Hva et slag mot en fiende førte til.
#[derive(Clone, Copy, Debug, Default)]
pub struct Strike {
    pub killed: bool,
    /// Panseret tok det meste (modalen forfra).
    pub armored: bool,
    /// Slaget brøt gjennom balansen til en av de tunge (`poise`).
    pub staggered: bool,
}

impl Mobs {
    /// Et slag mot fiende `i` fra `from`: skade, rykk bakover, og tokens
    /// om den dør.
    pub fn strike(&mut self, i: usize, damage: f32, from: Vec3, crypt: &Crypt) -> Strike {
        self.strike_with(i, damage, from, crypt, false)
    }

    /// Som `strike`, og med `crush` (kølla) går slaget gjennom panseret og
    /// teller dobbelt mot balansen.
    pub fn strike_with(&mut self, i: usize, damage: f32, from: Vec3, crypt: &Crypt, crush: bool) -> Strike {
        let Some(m) = self.mobs.get_mut(i) else { return Strike::default() };
        if !m.alive() {
            return Strike::default();
        }
        let stats = m.kind.stats();
        let mut damage = damage;
        let mut armored = false;
        // Panseret: modalen tar lite forfra når den ikke er midt i et slag.
        if !crush && stats.armor < 1.0 && matches!(m.state, State::Idle | State::Walk) {
            let facing = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
            let to = (from - m.pos).with_y(0.0).normalize_or_zero();
            if facing.dot(to) > 0.2 {
                damage *= stats.armor;
                armored = true;
            }
        }
        // Billen som ligger velt, tar mer i buken.
        if m.flipped {
            damage *= badge::BELLY;
        }
        m.hp -= damage;
        m.hit = 0.0;
        m.aware = true;
        m.dormant = false;
        // Slaget dytter den bakover: en fart som bremses (`knock`), mer jo
        // hardere slaget og jo lettere fienden.
        let away = (m.pos - from).with_y(0.0).normalize_or_zero();
        m.knock += away * (1.6 + (damage / 50.0).min(2.0)) / m.scale();
        let clips = self.clips[m.kind as usize];
        if m.hp <= 0.0 {
            m.state = State::Dying;
            m.t = 0.0;
            let base = m.pos;
            let elite = m.elite;
            let n = if elite { 8 } else { stats.tokens };
            let die = if self.rnd() < 0.5 { clips.die2.unwrap_or(clips.die) } else { clips.die };
            self.play(i, die, false, 0.08);
            if elite {
                // Der det går an å komme til: på nettet, ikke inni en mur.
                let p = base + Vec3::new(0.0, 0.0, 0.3);
                self.drop_key(if crypt.walkable(p) { p } else { base });
            }
            self.drop_tokens(base, n);
            return Strike { killed: true, armored, staggered: false };
        }
        // De tunge lar seg ikke avbryte av lette slag, og billen ligger der
        // den ligger.
        if m.flipped {
            return Strike { killed: false, armored, staggered: false };
        }
        if damage * if crush { 2.0 } else { 1.0 } < stats.poise {
            return Strike { killed: false, armored, staggered: false };
        }
        m.state = State::Hit;
        m.t = 0.0;
        m.stun = 0.0;
        // De tunge vakler når balansen brister; de andre rykker til en side.
        let clip = if stats.poise > 0.0 {
            m.stun = 1.1;
            clips.stagger.unwrap_or(clips.hit)
        } else if self.rnd() < 0.5 {
            clips.hit2.unwrap_or(clips.hit)
        } else {
            clips.hit
        };
        let looping = stats.poise > 0.0 && clips.stagger.is_some();
        self.play(i, clip, looping, 0.05);
        Strike { killed: false, armored, staggered: stats.poise > 0.0 }
    }

    /// Et ekstra dytt bort fra `from` (det tunge hugget), mindre for de store.
    pub fn shove(&mut self, i: usize, from: Vec3, push: f32) {
        if let Some(m) = self.mobs.get_mut(i).filter(|m| m.alive()) {
            m.knock += (m.pos - from).with_y(0.0).normalize_or_zero() * push / m.scale();
        }
    }

    /// Lamslått i `secs` sekunder (etter en parade): står og rister.
    pub fn stagger(&mut self, i: usize, secs: f32) {
        let Some(m) = self.mobs.get_mut(i) else { return };
        if !m.alive() {
            return;
        }
        let stats = m.kind.stats();
        m.state = State::Hit;
        m.t = 0.0;
        m.stun = secs;
        m.hit = 0.0;
        m.cooldown = secs + stats.cooldown;
        let clips = self.clips[m.kind as usize];
        match clips.stagger {
            Some(c) => self.play(i, c, true, 0.08),
            None => self.play(i, clips.hit, false, 0.05),
        }
    }
}
