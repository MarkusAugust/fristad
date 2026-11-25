//! Kampen i økta: fiendene biter og du hugger (reglene står i
//! `domain::combat` og `domain::mobs`). Det som vises over bildet, står i
//! `hud.rs`.

use super::session::{sfx, Session};
use crate::domain::combat::CombatEvent;
use crate::domain::body::Stance;
use crate::domain::mobs::{Kind as MobKind, Sweep};
use crate::domain::world::Use;
use crate::ports::driving::Event;
use glam::Vec3;

/// Så nær den falske veggen må være for at et hugg slår den inn (målt fra øyet til midten av muren).
const WALL_REACH: f32 = 3.0;

impl Session {
    /// Fiendene: de biter, og du plukker tokens. Under hitstop står alt
    /// stille et øyeblikk.
    pub(super) fn fight(&mut self, dt: f32, feet: Vec3, in_crypt: bool, events: &mut Vec<Event>) {
        let sim_dt = if self.combat.hitstop > 0.0 { 0.0 } else { dt };
        let eye = self.eye();
        let fwd = self.forward();
        // Hvor heisene står (fiendene kjører heisen når den står der de er).
        self.crypt.lift_rest = (0..self.crypt.lifts.len()).map(|n| (!self.mech.lift_moving(n)).then(|| !self.mech.lift_up(n))).collect();
        // Et hopp eller en unnamanøver rister middene av ankelen.
        if (!self.body.grounded || self.combat.dodge_velocity().length_squared() > 1.0) && self.mobs.shake_off(feet) > 0 {
            events.push(sfx("dodge"));
        }
        let outcome = self.mobs.update(sim_dt, feet, in_crypt, &self.crypt);
        for &n in &outcome.lifts {
            if self.mech.ride_lift(&self.crypt, n).is_some() {
                let at = self.crypt.lifts[n].pos + Vec3::Y * 2.0;
                events.push(self.sound_at("gate", at, 1.2));
            }
        }
        self.hero.coins += outcome.tokens;
        if outcome.tokens > 0 {
            events.push(sfx("pickup"));
        }
        if outcome.keys > 0 {
            self.hero.keys += outcome.keys;
            events.extend([sfx("key"), Event::Toast("En rusten nøkkel. En låst port i nivået kan åpnes.".into())]);
        }
        for (name, at) in &outcome.sounds {
            // Tassingen høres bare når de er nær.
            if *name == "mob_step" && at.distance(feet) > 14.0 {
                continue;
            }
            events.push(self.sound_at(name, *at + Vec3::Y * if *name == "mob_step" { 0.0 } else { 1.0 }, 1.0));
        }
        for _ in 0..outcome.roars {
            events.push(Event::Toast("Rammeverket brøler, og tooltips kryper ut av veggene.".into()));
        }
        if outcome.quakes > 0 {
            self.combat.quake();
        }
        // Markørene spruter der de treffer, med et klask.
        for (at, dir) in &outcome.impacts {
            self.combat.splash(*at, *dir);
            events.push(self.sound_at("spit_hit", *at, 1.0));
        }
        // Bittene: blokk tar det meste, en parade alt (og lamslår), en
        // unnamanøver går klar. Comboboxens pisk går klar om du hopper over
        // den lave og dukker under den høye.
        let mut taken_total = 0.0;
        let (airborne, ducked) = (!self.body.grounded, self.body.stance == Stance::Crouch);
        let swept = outcome.sweeps.iter().filter(|(.., how)| !match how {
            Sweep::Low => airborne,
            Sweep::High => ducked,
        });
        for (mob, damage, from) in outcome.bites.iter().copied().chain(swept.map(|&(m, d, f, _)| (m, d, f))) {
            let (mob, damage, from) = (&mob, &damage, &from);
            let (taken, parried) = self.combat.incoming(*from, *damage, eye, fwd);
            if parried {
                self.mobs.stagger(*mob, 1.3);
                events.push(sfx("parry"));
            } else if taken < *damage * 0.99 && taken > 0.0 {
                events.push(sfx(self.hero.gear.shield.sound));
            } else if taken > 0.0 {
                events.push(sfx("hurt"));
            }
            if taken > 0.0 {
                events.push(Event::Hurt { angle: hurt_angle(*from - eye, fwd) });
                // Hammerslaget til Den fryktinngytende Button trykker deg ned.
                if outcome.presses.contains(mob) && !parried {
                    self.mobs.press();
                }
            }
            taken_total += taken;
        }
        if taken_total > 0.0 {
            self.hero.hp -= taken_total;
            if self.hero.hp <= 0.0 {
                self.die(events);
            }
        }
        // Våre hugg. Treffet høres der fienden står, med materialet den er
        // laget av (tabellen er tre, Rammeverket jern).
        let mut struck: Option<Vec3> = None;
        for e in self.combat.update(dt, sim_dt, eye, fwd, feet, &mut self.mobs, &self.crypt) {
            match e {
                CombatEvent::Hit { mob, damage, killed, pos } => {
                    self.hud.popup(pos, damage);
                    struck = Some(pos);
                    let material = match self.mobs.mobs.get(mob).map(|m| m.kind) {
                        Some(MobKind::Table) => Some("hit_wood"),
                        Some(MobKind::Framework | MobKind::Navbar) => Some("hit_metal"),
                        _ => None,
                    };
                    if let Some(m) = material {
                        events.push(self.sound_at(m, pos, 1.0));
                    }
                    if killed {
                        self.killed(mob, pos, events);
                    }
                }
                // Et hugg som ikke treffer noen fiende, men den falske
                // veggen du sikter på, slår den inn (som F).
                CombatEvent::Miss => {
                    if let Some(i) = self.aim.filter(|&i| matches!(self.targets[i].target.what, Use::Secret(_)) && self.target_pos(i).distance(eye) <= WALL_REACH) {
                        self.use_target(i, events);
                    }
                }
                CombatEvent::Sound(s) => match struck {
                    Some(at) if s != "swing" && s != "swing_heavy" => events.push(self.sound_at(s, at, 1.0)),
                    _ => events.push(sfx(s)),
                },
            }
        }
    }

    /// Tokens som spretter ut `ahead` meter foran deg (til skjermbildene).
    pub fn drop_tokens(&mut self, ahead: f32, n: u32) {
        let f = self.forward().with_y(0.0).normalize_or_zero();
        let at = self.body.sole() + f * ahead;
        self.mobs.drop_tokens(at, n);
    }

    /// En nøkkel som eliten mister, `ahead` meter foran deg og `side` til
    /// høyre (`--key`).
    pub fn drop_key(&mut self, ahead: f32, side: f32) {
        let f = self.forward().with_y(0.0).normalize_or_zero();
        let at = self.body.sole() + f * ahead + f.cross(Vec3::Y) * side;
        self.mobs.drop_key(at);
    }

    /// Bare én korrupt input, `ahead` meter foran deg og `side` til høyre
    /// (der gulvet er fritt), som har sett deg og spytter straks (til
    /// skjermbildene, `--bolt`).
    pub fn input_ahead(&mut self, ahead: f32, side: f32) -> bool {
        self.foe_ahead(MobKind::Input, ahead, side)
    }

    /// Bare én fiende av slaget `kind` foran deg, som har sett deg (til
    /// skjermbildene, `snapshot --bolt --bolt-kind`).
    pub fn foe_ahead(&mut self, kind: MobKind, ahead: f32, side: f32) -> bool {
        let feet = self.body.sole();
        let f = self.forward().with_y(0.0).normalize_or_zero();
        let right = f.cross(Vec3::Y);
        let c = &self.crypt;
        let spot = (0..=12).map(|k| ahead - k as f32 * 0.4).map(|d| feet + f * d + right * side).map(|q| Vec3::new(q.x, c.height_at(q + Vec3::Y * 0.5), q.z)).find(|q| c.walkable(*q));
        let Some(at) = spot else { return false };
        self.mobs.mobs.clear();
        self.mobs.bolts.clear();
        let i = self.mobs.spawn_aware(kind, at);
        self.mobs.mobs[i].yaw = (feet - at).x.atan2((feet - at).z);
        true
    }

    /// Midt på kroppen til den første fienden, også når den flyr (til
    /// skjermbildene, `snapshot --bolt-view`).
    pub fn foe_at(&self) -> Option<Vec3> {
        self.mobs.mobs.first().map(|m| m.body_mid())
    }

    /// Hvor mange markører som er i lufta.
    pub fn bolts_in_air(&self) -> usize {
        self.mobs.bolts.len()
    }

    /// Oljen i lykta som brøk (0 er sluknet); til skjermbildene.
    pub fn set_oil(&mut self, fraction: f32) {
        self.hero.lamp.oil = fraction.clamp(0.0, 1.0) * crate::domain::lantern::FULL;
    }

    /// Lykta brenner nede i krypten, og sier fra når oljen er lav og når
    /// den slukner.
    pub(super) fn burn_lamp(&mut self, dt: f32, in_crypt: bool, events: &mut Vec<Event>) {
        if !in_crypt {
            return;
        }
        let before = self.hero.lamp.fraction();
        self.hero.lamp.burn(dt);
        let after = self.hero.lamp.fraction();
        if before >= crate::domain::lantern::LOW && after < crate::domain::lantern::LOW {
            events.push(Event::Toast("Lykta begynner å gå tom for olje. Hvil ved et alter, eller kjøp olje hos Apotekeren.".into()));
        }
        if before > 0.0 && after <= 0.0 {
            events.push(Event::Toast("Lykta har sluknet. Mørket lukker seg rundt deg.".into()));
        }
    }
}

/// Hvor noe kom fra (`to`, fra øyet), som vinkel fra blikket `fwd` i
/// radianer: 0 rett foran, positiv til høyre, ±π bak (`Event::Hurt`).
pub(super) fn hurt_angle(to: Vec3, fwd: Vec3) -> f32 {
    let f = fwd.with_y(0.0).normalize_or(Vec3::NEG_Z);
    let right = f.cross(Vec3::Y);
    let d = to.with_y(0.0);
    d.dot(right).atan2(d.dot(f))
}
