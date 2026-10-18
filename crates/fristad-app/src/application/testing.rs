//! Det testene trenger av økta: bruke et mål som om du siktet på det, se
//! og sette liv, nøkler, porter og fiender, og dø med vilje. Det meste står
//! bak `testing`, så det ikke er en del av spillet.

use super::session::Session;
use crate::domain::mobs::Kind as MobKind;
use crate::domain::world::Use;
use crate::ports::driving::Event;
use glam::Vec3;

impl Session {
    /// Bruk det første målet av et slag, som om du siktet på det.
    pub fn use_first(&mut self, what: fn(&Use) -> bool) -> Vec<Event> {
        let mut events = vec![];
        if let Some(i) = self.targets.iter().position(|t| what(&t.target.what)) {
            self.use_target(i, &mut events);
        }
        events
    }

    /// Besetningen: slag og om den er elite.
    pub fn roster(&self) -> Vec<(MobKind, bool)> {
        self.mobs.mobs.iter().map(|m| (m.kind, m.elite)).collect()
    }

    /// Den nærmeste levende fienden (føttene).
    pub fn nearest_mob(&self, from: Vec3) -> Option<Vec3> {
        self.mobs.mobs.iter().filter(|m| m.alive()).map(|m| m.pos).min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }
}

#[cfg(any(test, feature = "testing"))]
impl Session {
    /// De levende fiendene (føttene, slaget, om den er elite), til
    /// gjennomspillingen.
    pub fn living(&self) -> Vec<(Vec3, MobKind, bool)> {
        self.mobs.mobs.iter().filter(|m| m.alive()).map(|m| (m.pos, m.kind, m.elite)).collect()
    }

    /// Fullt liv igjen, til gjennomspillingen (som teller hvor ofte boten
    /// ville ha dødd).
    pub fn heal(&mut self) {
        self.hero.hp = self.hero.max_hp;
    }

    pub fn hp_now(&self) -> f32 {
        self.hero.hp
    }

    pub fn stamina_now(&self) -> f32 {
        self.combat.stamina
    }

    pub fn set_vitals(&mut self, hp: f32, stamina: f32) {
        self.hero.hp = hp;
        self.combat.stamina = stamina;
    }

    pub fn keys(&self) -> u32 {
        self.hero.keys
    }

    /// Nøklene som ligger på gulvet etter en elite.
    pub fn loose_keys(&self) -> Vec<Vec3> {
        self.mobs.keys.iter().filter(|k| !k.taken).map(|k| k.pos).collect()
    }

    /// Felle eliten i nivået du er i med ett slag.
    pub fn fell_elite(&mut self, events: &mut Vec<Event>) -> bool {
        let Some(i) = self.mobs.mobs.iter().position(|m| m.alive() && m.elite) else { return false };
        let pos = self.mobs.mobs[i].pos;
        if self.mobs.strike(i, 1.0e6, pos + Vec3::X, &self.crypt).killed {
            self.killed(i, pos, events);
        }
        true
    }

    /// Bossen i nivået, om den lever (der den står).
    pub fn boss_pos(&self) -> Option<Vec3> {
        self.mobs.mobs.iter().find(|m| m.boss && m.alive()).map(|m| m.pos)
    }

    /// En nøkkel i lomma (boten som går uten fiender, der eliten er borte).
    pub fn give_key(&mut self) {
        self.hero.keys += 1;
    }

    /// Ingen fiender i nivået.
    pub fn clear_mobs(&mut self) {
        self.mobs.mobs.clear();
    }

    /// Nivået fiendene er satt ut i.
    pub fn mob_level(&self) -> usize {
        self.mobs.level
    }

    /// Krypten slik økta ser den (med bryterne som er åpnet).
    pub fn crypt_now(&self) -> &crate::domain::world::crypt::Crypt {
        &self.crypt
    }

    pub fn max_hp_now(&self) -> f32 {
        self.hero.max_hp
    }

    pub fn die_now(&mut self) -> Vec<Event> {
        let mut events = vec![];
        self.die(&mut events);
        events
    }

    /// Hvor langt portgitteret er heist (0–1) og om det er låst opp.
    pub fn gate(&self, n: usize) -> (f32, bool) {
        self.mech.gates[n]
    }

    /// Bruk akkurat dette målet, som om du siktet på det.
    pub fn use_exact(&mut self, what: Use) -> Vec<Event> {
        let mut events = vec![];
        if let Some(i) = self.targets.iter().position(|t| t.target.what == what) {
            self.use_target(i, &mut events);
        }
        events
    }

    pub fn feet(&self) -> Vec3 {
        self.body.feet()
    }

    pub fn lever_pulled(&self, n: usize) -> bool {
        self.mech.lever_pulled(n)
    }
    /// Heisen: om bremsen er løsnet, om den står oppe, og om den kjører.
    pub fn lift_state(&self, n: usize) -> (bool, bool, bool) {
        let free = self.crypt.lifts.get(n).is_some_and(|l| self.crypt.open.get(l.switch).copied().unwrap_or(false));
        (free, self.mech.lift_up(n), self.mech.lift_moving(n))
    }

    /// Åpne bryter `sw` (port, dør, falsk vegg, vann, bro) uten å gå dit.
    pub fn open_now(&mut self, sw: usize) {
        self.open_switch(sw);
    }
}
