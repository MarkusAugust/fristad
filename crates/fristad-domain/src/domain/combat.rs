//! Nærkampen i førsteperson: armene med sverd og skjold (modellen `arms`,
//! `tools/models/arms.py`), utholdenheten, komboen på tre hugg, det tunge
//! hugget som lades, blokk og parade med skjoldet, unnamanøveren med
//! usårbarhet, og det som gjør at treff synes: hitstop når fienden treffes,
//! kameraspark og rødt flash når vi treffes, rusk som spruter. Spillet
//! (`game.rs`) gir inndata og fiendene, og tar imot hendelsene.

use crate::domain::anim::{Animator, Rig};
use crate::domain::craft::{Gear, WeaponKind, OLD_SHIELD, RUSTY_SWORD};
use crate::domain::geometry::{kind, Mat, Mesh};
use crate::domain::mobs::Mobs;
use crate::domain::model::Model;
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};

pub const ARMS_MODEL: &str = "arms";
pub const DEBRIS_MODEL: &str = "debris";
const MAX_DEBRIS: usize = 96;

/// Sekundene i klippene der treffet lander (`SWING_HIT`, `HEAVY_HOLD`,
/// `HEAVY_HIT` i tools/models/arms.py).
const SWING_HIT: [f32; 3] = [5.0 / 24.0, 5.0 / 24.0, 7.0 / 24.0];
const SWING_LEN: [f32; 3] = [12.0 / 24.0, 12.0 / 24.0, 16.0 / 24.0];
const HEAVY_HOLD: f32 = 10.0 / 24.0;
const HEAVY_HIT: f32 = 13.0 / 24.0;
const HEAVY_LEN: f32 = 1.0;
const PARRY_LEN: f32 = 0.5;
const HURT_LEN: f32 = 10.0 / 24.0;
/// Holdes knappen så lenge, blir det lette hugget et tungt.
const HOLD_FOR_HEAVY: f32 = 0.16;
/// Rekkevidden og vinkelen (cos) på sverdet.
const REACH: f32 = 2.3;
const ARC: f32 = 0.55;
const LIGHT_DAMAGE: [f32; 3] = [22.0, 25.0, 34.0];
const HEAVY_DAMAGE: f32 = 58.0;
/// Utholdenhet: det hvert trekk koster, og hvor fort den kommer igjen.
pub const STAMINA_MAX: f32 = 100.0;
const COST_LIGHT: f32 = 12.0;
const COST_HEAVY: f32 = 28.0;
const COST_DODGE: f32 = 22.0;
const COST_BLOCK: f32 = 18.0;
const REGEN: f32 = 24.0;
const REGEN_DELAY: f32 = 0.8;
const DODGE_LEN: f32 = 0.42;
const DODGE_IFRAMES: f32 = 0.34;
const DODGE_SPEED: f32 = 7.5;
const HITSTOP: f32 = 0.08;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Idle,
    /// Lett hugg nummer `n` (0..3); `hit` når treffet er gjort.
    Swing {
        n: usize,
        hit: bool,
    },
    /// Tungt hugg: lades mens knappen holdes, hugger når den slippes.
    Heavy {
        charging: bool,
        hit: bool,
        charge: f32,
    },
    Block,
    Parry,
    Dodge,
    Hurt,
}

/// Det kampen melder til spillet dette bildet.
#[derive(Clone, Debug, PartialEq)]
pub enum CombatEvent {
    /// Et hugg traff fiende `mob` med `damage`; `killed` om den døde.
    Hit {
        mob: usize,
        damage: f32,
        killed: bool,
        pos: Vec3,
    },
    /// Et hugg traff ingenting.
    Miss,
    Sound(&'static str),
}

#[derive(Clone, Copy, Debug)]
struct Debris {
    pos: Vec3,
    vel: Vec3,
    spin: Vec3,
    age: f32,
    size: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Clips {
    idle: usize,
    swing: [usize; 3],
    heavy: usize,
    block: usize,
    parry: usize,
    hurt: usize,
}

pub struct Combat {
    /// Lyder fra trekk som starter utenfor `update` (ladningen), til neste bilde.
    cues: Vec<&'static str>,
    pub gear: Gear,
    pub max_stamina: f32,
    pub stamina: f32,
    regen_delay: f32,
    pub phase: Phase,
    /// Sekunder i fasen.
    t: f32,
    /// Knappen holdes; hvor lenge.
    held: bool,
    hold_t: f32,
    /// Et nytt hugg er bestilt mens dette går.
    queued: bool,
    /// Tid siden siste hugg sluttet (komboen holdes 0,6 s).
    since_swing: f32,
    next_swing: usize,
    block_wanted: bool,
    /// Sekunder siden blokken startet, til paraden.
    block_t: f32,
    /// Neste hugg gjør dobbel skade (etter en parade).
    riposte: bool,
    dodge_dir: Vec3,
    pub hitstop: f32,
    /// Kameraspark (yaw, pitch) med fjæring, og rødt flash i kantene.
    kick: Vec2,
    kick_vel: Vec2,
    shake: f32,
    pub hurt: f32,
    rig: Option<Rig>,
    clips: Clips,
    anim: Option<Animator>,
    debris: Vec<Debris>,
    seed: u32,
}

impl Combat {
    pub fn new(rig: Option<Rig>) -> Self {
        let clips = rig
            .as_ref()
            .map(|r| {
                let find = |n: &str| r.clip(n).unwrap_or(0);
                Clips {
                    idle: find("idle"),
                    swing: [find("swing1"), find("swing2"), find("swing3")],
                    heavy: find("heavy"),
                    block: find("block"),
                    parry: find("parry"),
                    hurt: find("hurt"),
                }
            })
            .unwrap_or_default();
        let anim = rig.as_ref().map(|_| Animator::new(clips.idle, true));
        Self {
            gear: Gear {
                weapon: RUSTY_SWORD,
                shield: OLD_SHIELD,
                armor: None,
                lantern: 1.0,
            },
            max_stamina: STAMINA_MAX,
            stamina: STAMINA_MAX,
            regen_delay: 0.0,
            phase: Phase::Idle,
            t: 0.0,
            held: false,
            hold_t: 0.0,
            queued: false,
            since_swing: 9.0,
            next_swing: 0,
            block_wanted: false,
            block_t: 9.0,
            riposte: false,
            cues: vec![],
            dodge_dir: Vec3::ZERO,
            hitstop: 0.0,
            kick: Vec2::ZERO,
            kick_vel: Vec2::ZERO,
            shake: 0.0,
            hurt: 0.0,
            rig,
            clips,
            anim,
            debris: vec![],
            seed: 0x1234_5678,
        }
    }

    /// Hvor mange instanser hver modell trenger plass til: armene, rusket,
    /// og hvert våpen og skjold (ett av hvert kan være i hendene).
    pub fn reserve() -> Vec<(&'static str, usize)> {
        let mut v = vec![(ARMS_MODEL, 1), (DEBRIS_MODEL, MAX_DEBRIS)];
        for w in crate::domain::craft::WEAPONS {
            if !v.iter().any(|(m, _)| *m == w.model) {
                v.push((w.model, 1));
            }
        }
        for s in crate::domain::craft::SHIELDS {
            if !v.iter().any(|(m, _)| *m == s.model) {
                v.push((s.model, 1));
            }
        }
        v
    }

    pub fn set_gear(&mut self, gear: Gear) {
        self.gear = gear;
    }

    fn rnd(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed >> 8) as f32 / (1u32 << 24) as f32
    }

    fn play(&mut self, clip: usize, looping: bool, fade: f32) {
        if let Some(a) = &mut self.anim {
            a.play(clip, looping, fade);
            a.speed = 1.0;
        }
    }

    fn enter(&mut self, phase: Phase) {
        self.phase = phase;
        self.t = 0.0;
    }

    pub fn reset(&mut self) {
        self.stamina = self.max_stamina;
        self.enter(Phase::Idle);
        self.queued = false;
        self.held = false;
        self.riposte = false;
        self.hitstop = 0.0;
        self.hurt = 0.0;
        self.debris.clear();
        let idle = self.clips.idle;
        self.play(idle, true, 0.0);
    }

    pub fn blocking(&self) -> bool {
        matches!(self.phase, Phase::Block | Phase::Parry)
    }

    /// Usårbar: midt i unnamanøveren.
    pub fn invulnerable(&self) -> bool {
        self.phase == Phase::Dodge && self.t < DODGE_IFRAMES
    }

    /// Knappen for hugg trykket.
    pub fn press_attack(&mut self) {
        self.held = true;
        self.hold_t = 0.0;
        match self.phase {
            Phase::Idle | Phase::Block => {
                if self.stamina >= COST_LIGHT {
                    self.start_swing();
                }
            }
            Phase::Swing { .. } | Phase::Heavy { .. } | Phase::Parry => {
                // Bestilles til dette hugget er langt nok på vei.
                self.queued = true;
            }
            _ => {}
        }
    }

    pub fn release_attack(&mut self) {
        self.held = false;
    }

    pub fn set_block(&mut self, wanted: bool) {
        self.block_wanted = wanted;
    }

    /// Unnamanøver i retningen `dir` (vannrett), om det er utholdenhet til det.
    pub fn dodge(&mut self, dir: Vec3) -> bool {
        if !matches!(self.phase, Phase::Idle | Phase::Block | Phase::Swing { .. })
            || self.stamina < COST_DODGE
        {
            return false;
        }
        self.stamina -= COST_DODGE;
        self.regen_delay = REGEN_DELAY;
        self.dodge_dir = dir.with_y(0.0).normalize_or_zero();
        self.queued = false;
        self.enter(Phase::Dodge);
        let idle = self.clips.idle;
        self.play(idle, true, 0.15);
        true
    }

    /// Farten unnamanøveren gir føttene nå.
    pub fn dodge_velocity(&self) -> Vec3 {
        if self.phase != Phase::Dodge {
            return Vec3::ZERO;
        }
        let f = (1.0 - self.t / DODGE_LEN).clamp(0.0, 1.0);
        self.dodge_dir * DODGE_SPEED * f * f
    }

    /// Ganger på gangfarten: hugg og blokk bremser.
    pub fn move_factor(&self) -> f32 {
        match self.phase {
            Phase::Swing { .. } => 0.55,
            Phase::Heavy { .. } => 0.35,
            Phase::Block | Phase::Parry => 0.6,
            Phase::Hurt => 0.5,
            _ => 1.0,
        }
    }

    fn start_swing(&mut self) {
        // Komboen fortsetter om forrige hugg var nylig.
        if self.since_swing > 0.6 {
            self.next_swing = 0;
        }
        let n = self.next_swing % 3;
        self.next_swing = n + 1;
        self.stamina -= COST_LIGHT;
        self.regen_delay = REGEN_DELAY;
        self.queued = false;
        self.enter(Phase::Swing { n, hit: false });
        let clip = self.clips.swing[n];
        self.play(clip, false, 0.06);
        if let Some(a) = &mut self.anim {
            a.speed = self.gear.weapon.speed;
        }
    }

    fn start_heavy(&mut self) {
        self.stamina -= COST_HEAVY;
        self.regen_delay = REGEN_DELAY;
        self.queued = false;
        self.enter(Phase::Heavy {
            charging: true,
            hit: false,
            charge: 0.0,
        });
        self.cues.push("charge");
        let clip = self.clips.heavy;
        log::debug!("Tungt hugg lades (klipp {clip})");
        self.play(clip, false, 0.1);
    }

    /// Et bitt kommer fra `from` med `damage`. Gir skaden som slipper
    /// gjennom, og om det ble en parade.
    pub fn incoming(&mut self, from: Vec3, damage: f32, eye: Vec3, forward: Vec3) -> (f32, bool) {
        if self.invulnerable() {
            return (0.0, false);
        }
        let to = (from - eye).with_y(0.0).normalize_or_zero();
        let front = forward.with_y(0.0).normalize_or_zero().dot(to) > 0.2;
        let armor = self.gear.armor.map(|a| a.taken).unwrap_or(1.0);
        let damage = damage * armor;
        if self.blocking() && front {
            if self.block_t <= self.gear.shield.parry_window && self.phase == Phase::Block {
                self.enter(Phase::Parry);
                let parry = self.clips.parry;
                self.play(parry, false, 0.05);
                self.riposte = true;
                self.shake = (self.shake + 0.25).min(1.0);
                return (0.0, true);
            }
            self.stamina -= COST_BLOCK;
            self.regen_delay = REGEN_DELAY;
            self.shake = (self.shake + 0.3).min(1.0);
            if self.stamina <= 0.0 {
                // Garden brytes.
                self.stamina = 0.0;
                self.got_hit(to);
                return (damage * 0.6, false);
            }
            self.kick_vel += Vec2::new(0.0, 0.5);
            return (damage * self.gear.shield.block, false);
        }
        self.got_hit(to);
        (damage, false)
    }

    /// Vi ble truffet: kameraet sparkes bort fra angriperen og tilbake,
    /// skjelver, kantene blir røde, og armene slenges.
    fn got_hit(&mut self, to: Vec3) {
        // Sparket i yaw bort fra retningen angrepet kom fra: positivt
        // = mot venstre i vår yaw-konvensjon, så vi snur bort fra `to`.
        let side = if to.x * 1.0 > 0.0 { 1.0 } else { -1.0 };
        self.kick_vel += Vec2::new(side * 2.2, 1.6);
        self.shake = 1.0;
        self.hurt = 1.0;
        if !matches!(self.phase, Phase::Dodge) {
            self.enter(Phase::Hurt);
            self.queued = false;
            let hurt = self.clips.hurt;
            self.play(hurt, false, 0.04);
        }
    }

    /// Bakken rister (bossens slag): kameraet skjelver.
    pub fn quake(&mut self) {
        self.shake = 1.0;
        self.kick_vel += Vec2::new(0.0, 1.2);
    }

    /// Kameradreiningen fra spark og skjelv nå: (yaw, pitch) i radianer.
    pub fn view_kick(&self, time: f32) -> (f32, f32) {
        let s = self.shake * self.shake;
        let yaw =
            self.kick.x * 0.06 + ((time * 47.0).sin() + (time * 31.0).cos() * 0.5) * 0.012 * s;
        let pitch =
            self.kick.y * 0.05 + ((time * 53.0).cos() + (time * 29.0).sin() * 0.5) * 0.01 * s;
        (yaw, pitch)
    }

    /// Ett steg. `dt` er ekte tid; `sim_dt` er null under hitstop. Huggene
    /// treffer fiendene i `mobs` innen rekkevidde i kjeglen foran.
    pub fn update(
        &mut self,
        dt: f32,
        sim_dt: f32,
        eye: Vec3,
        forward: Vec3,
        feet: Vec3,
        mobs: &mut Mobs,
        crypt: &crate::domain::world::crypt::Crypt,
    ) -> Vec<CombatEvent> {
        let mut out: Vec<CombatEvent> = self.cues.drain(..).map(CombatEvent::Sound).collect();
        self.hitstop = (self.hitstop - dt).max(0.0);
        // Fjæra på kameraet.
        let k = 60.0;
        let d = 10.0;
        self.kick_vel += (-self.kick * k - self.kick_vel * d) * dt;
        self.kick += self.kick_vel * dt;
        self.shake = (self.shake - dt * 4.0).max(0.0);
        self.hurt = (self.hurt - dt * 1.8).max(0.0);
        self.regen_delay = (self.regen_delay - dt).max(0.0);
        if self.regen_delay <= 0.0
            && !matches!(
                self.phase,
                Phase::Block | Phase::Heavy { charging: true, .. }
            )
        {
            self.stamina = (self.stamina + REGEN * dt).min(self.max_stamina);
        }
        self.since_swing += sim_dt;
        self.block_t += dt;
        if self.held {
            self.hold_t += dt;
        }
        // Fasetida går i våpenets tempo under hugg.
        let tempo = if matches!(self.phase, Phase::Swing { .. } | Phase::Heavy { .. }) {
            self.gear.weapon.speed
        } else {
            1.0
        };
        self.t += sim_dt * tempo;
        if let (Some(r), Some(a)) = (&self.rig, &mut self.anim) {
            a.update(sim_dt, r);
        }
        let clip_time = |s: &Self| s.anim.as_ref().map(|a| a.time).unwrap_or(s.t);

        match self.phase {
            Phase::Idle => {
                if self.block_wanted && self.stamina > 0.0 {
                    self.enter(Phase::Block);
                    self.block_t = 0.0;
                    let b = self.clips.block;
                    self.play(b, false, 0.08);
                } else if self.queued && self.stamina >= COST_LIGHT {
                    self.start_swing();
                }
            }
            Phase::Block => {
                if !self.block_wanted {
                    self.enter(Phase::Idle);
                    let idle = self.clips.idle;
                    self.play(idle, true, 0.12);
                }
            }
            Phase::Parry => {
                if self.t >= PARRY_LEN {
                    if self.block_wanted {
                        self.enter(Phase::Block);
                        self.block_t = 9.0;
                        let b = self.clips.block;
                        self.play(b, false, 0.1);
                    } else {
                        self.enter(Phase::Idle);
                        let idle = self.clips.idle;
                        self.play(idle, true, 0.12);
                    }
                }
            }
            Phase::Swing { n, hit } => {
                let ct = clip_time(self);
                // Holdes knappen fra starten av, lades et tungt hugg i stedet.
                if !hit
                    && self.held
                    && self.hold_t >= HOLD_FOR_HEAVY
                    && ct < SWING_HIT[n]
                    && self.stamina >= COST_HEAVY
                {
                    self.stamina += COST_LIGHT;
                    self.start_heavy();
                } else if !hit && ct >= SWING_HIT[n] {
                    let mut dmg = LIGHT_DAMAGE[n] * self.gear.weapon.damage;
                    if self.riposte {
                        dmg *= self.gear.weapon.riposte;
                        self.riposte = false;
                    }
                    self.phase = Phase::Swing { n, hit: true };
                    self.strike(dmg, false, eye, forward, feet, mobs, crypt, &mut out);
                } else if self.t >= SWING_LEN[n] {
                    self.since_swing = 0.0;
                    self.enter(Phase::Idle);
                    if self.queued && self.stamina >= COST_LIGHT {
                        self.start_swing();
                    } else {
                        let idle = self.clips.idle;
                        self.play(idle, true, 0.15);
                    }
                } else if hit
                    && self.queued
                    && self.t >= SWING_LEN[n] * 0.7
                    && self.stamina >= COST_LIGHT
                {
                    // Neste hugg i komboen får begynne litt før dette er ferdig.
                    self.since_swing = 0.0;
                    self.start_swing();
                }
            }
            Phase::Heavy {
                charging,
                hit,
                charge,
            } => {
                let ct = clip_time(self);
                if charging {
                    // Holdes ved toppen av ladningen til knappen slippes
                    // (eller ladningen er full).
                    // Full ladning: et lite klang i stålet.
                    if charge < 1.0 && charge + dt >= 1.0 {
                        out.push(CombatEvent::Sound("charged"));
                    }
                    let charge = charge + dt;
                    self.phase = Phase::Heavy {
                        charging: true,
                        hit: false,
                        charge,
                    };
                    if ct >= HEAVY_HOLD {
                        log::debug!("Tungt hugg holdes ved {ct:.2} s, ladning {charge:.2}");
                        if let Some(a) = &mut self.anim {
                            a.speed = 0.0;
                            a.time = HEAVY_HOLD;
                        }
                        self.t = HEAVY_HOLD;
                        if !self.held || charge >= 1.2 {
                            self.phase = Phase::Heavy {
                                charging: false,
                                hit: false,
                                charge,
                            };
                            if let Some(a) = &mut self.anim {
                                a.speed = 1.0;
                            }
                        }
                    }
                } else if !hit && ct >= HEAVY_HIT {
                    let mut dmg = HEAVY_DAMAGE
                        * self.gear.weapon.damage
                        * (1.0 + (charge / 1.2).clamp(0.0, 1.0) * 0.4);
                    if self.riposte {
                        dmg *= self.gear.weapon.riposte;
                        self.riposte = false;
                    }
                    self.phase = Phase::Heavy {
                        charging: false,
                        hit: true,
                        charge,
                    };
                    self.strike(dmg, true, eye, forward, feet, mobs, crypt, &mut out);
                } else if self.t >= HEAVY_LEN {
                    self.since_swing = 0.0;
                    self.next_swing = 0;
                    self.enter(Phase::Idle);
                    let idle = self.clips.idle;
                    self.play(idle, true, 0.15);
                }
            }
            Phase::Dodge => {
                if self.t >= DODGE_LEN {
                    self.enter(Phase::Idle);
                }
            }
            Phase::Hurt => {
                if self.t >= HURT_LEN {
                    self.enter(Phase::Idle);
                    let idle = self.clips.idle;
                    self.play(idle, true, 0.12);
                }
            }
        }

        // Rusket faller, spretter på gulvet og blir borte.
        for p in &mut self.debris {
            p.age += sim_dt;
            p.vel.y -= 14.0 * sim_dt;
            p.pos += p.vel * sim_dt;
            let floor = crypt.height_at(p.pos);
            if p.pos.y < floor + 0.02 {
                p.pos.y = floor + 0.02;
                p.vel.y = -p.vel.y * 0.35;
                p.vel.x *= 0.6;
                p.vel.z *= 0.6;
            }
        }
        self.debris.retain(|p| p.age < 1.6);
        out
    }

    /// Sverdet treffer alle fiender innen rekkevidde i kjeglen foran.
    #[allow(clippy::too_many_arguments)]
    fn strike(
        &mut self,
        damage: f32,
        heavy: bool,
        eye: Vec3,
        forward: Vec3,
        feet: Vec3,
        mobs: &mut Mobs,
        crypt: &crate::domain::world::crypt::Crypt,
        out: &mut Vec<CombatEvent>,
    ) {
        let fwd = forward.with_y(0.0).normalize_or_zero();
        let mut any = false;
        let n = mobs.mobs.len();
        let reach = self.gear.weapon.reach.max(REACH * 0.6);
        for i in 0..n {
            let m = &mobs.mobs[i];
            if !m.alive() {
                continue;
            }
            let to = m.pos - feet;
            let flat = to.with_y(0.0);
            let dist = flat.length();
            // Rekkevidden måles til kroppen, ikke midten: store fiender
            // (tabellene, Rammeverket) står lenger unna når de slår.
            if dist > reach + 0.4 * m.scale() || flat.normalize_or_zero().dot(fwd) < ARC {
                continue;
            }
            any = true;
            // Dolken gjør dobbelt bakfra: fienden vender bort fra deg.
            let mut damage = damage;
            if self.gear.weapon.kind == WeaponKind::Dagger {
                let facing = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
                if facing.dot(flat.normalize_or_zero()) > 0.3 {
                    damage *= 2.0;
                }
            }
            let strike = mobs.strike(i, damage, feet, crypt);
            let killed = strike.killed;
            let body = mobs.mobs[i].pos + Vec3::Y * 1.0 * mobs.mobs[i].scale();
            self.spawn_debris(
                body,
                fwd,
                if strike.armored {
                    3
                } else if heavy {
                    14
                } else {
                    8
                },
            );
            out.push(CombatEvent::Hit {
                mob: i,
                damage,
                killed,
                pos: body,
            });
            out.push(CombatEvent::Sound(if killed {
                "kill"
            } else if strike.armored {
                "clang"
            } else if heavy {
                "hit_heavy"
            } else {
                "hit"
            }));
            if strike.staggered {
                out.push(CombatEvent::Sound("stagger"));
            }
        }
        if any {
            self.hitstop = HITSTOP;
            self.shake = (self.shake + 0.35).min(1.0);
        } else {
            out.push(CombatEvent::Miss);
            out.push(CombatEvent::Sound(if heavy {
                "swing_heavy"
            } else {
                "swing"
            }));
        }
        let _ = eye;
    }

    fn spawn_debris(&mut self, at: Vec3, fwd: Vec3, n: usize) {
        for _ in 0..n {
            if self.debris.len() >= MAX_DEBRIS {
                self.debris.remove(0);
            }
            let a = self.rnd() * std::f32::consts::TAU;
            let up = 2.0 + self.rnd() * 3.5;
            let spread = 1.0 + self.rnd() * 2.5;
            let vel =
                Vec3::new(a.cos() * spread, up, a.sin() * spread) + fwd * (1.5 + self.rnd() * 2.0);
            let spin = Vec3::new(self.rnd() - 0.5, self.rnd() - 0.5, self.rnd() - 0.5) * 20.0;
            let size = 0.6 + self.rnd() * 0.8;
            let jitter = Vec3::new(self.rnd() - 0.5, self.rnd() - 0.5, self.rnd() - 0.5) * 0.3;
            self.debris.push(Debris {
                pos: at + jitter,
                vel,
                spin,
                age: 0.0,
                size,
            });
        }
    }

    /// Instansene: armene i øyet med kameraets dreining (bare nede i
    /// kjelleren), og rusket. Leddmatrisene legges i `skins`.
    pub fn instances(
        &self,
        eye: Vec3,
        yaw: f32,
        pitch: f32,
        show_arms: bool,
        skins: &mut Vec<Mat4>,
    ) -> Vec<ModelInstance> {
        let mut out = vec![];
        if show_arms {
            let look = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
            let arms_t = Affine3A::from_rotation_translation(look, eye);
            match (&self.rig, &self.anim) {
                (Some(rig), Some(a)) => {
                    let pose = a.pose(rig);
                    let mats = rig.skin(&pose);
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    out.push(ModelInstance {
                        model: ARMS_MODEL,
                        transform: arms_t,
                        skin: Some((base, mats.len() as u32)),
                        flash: 0.0,
                        tint: None,
                    });
                    // Våpenet og skjoldet er modellert i armenes rom og følger
                    // hendenes skinnmatriser.
                    for (joint, model) in [
                        ("hand.R", self.gear.weapon.model),
                        ("hand.L", self.gear.shield.model),
                    ] {
                        if let Some(j) = rig.joint(joint) {
                            let t = arms_t * Affine3A::from_mat4(mats[j]);
                            out.push(ModelInstance {
                                model,
                                transform: t,
                                skin: None,
                                flash: 0.0,
                                tint: None,
                            });
                        }
                    }
                }
                _ => {
                    out.push(ModelInstance {
                        model: ARMS_MODEL,
                        transform: arms_t,
                        skin: None,
                        flash: 0.0,
                        tint: None,
                    });
                    out.push(ModelInstance {
                        model: self.gear.weapon.model,
                        transform: arms_t,
                        skin: None,
                        flash: 0.0,
                        tint: None,
                    });
                    out.push(ModelInstance {
                        model: self.gear.shield.model,
                        transform: arms_t,
                        skin: None,
                        flash: 0.0,
                        tint: None,
                    });
                }
            }
        }
        for p in &self.debris {
            let fade = (1.6 - p.age).clamp(0.0, 1.0).min(1.0);
            let rot = Quat::from_euler(
                glam::EulerRot::XYZ,
                p.spin.x * p.age,
                p.spin.y * p.age,
                p.spin.z * p.age,
            );
            out.push(ModelInstance {
                model: DEBRIS_MODEL,
                transform: Affine3A::from_rotation_translation(rot, p.pos)
                    * Affine3A::from_scale(Vec3::splat(p.size * fade)),
                skin: None,
                flash: 0.0,
                tint: None,
            });
        }
        out
    }
}

/// Rusket som spruter av fiendene: en liten krøllparentes av flate
/// biter, i knappeblått med lys kant. Lagt til modellene i `app::prepare`.
pub fn debris_model() -> Model {
    let mut m = Mesh::default();
    let blue = Mat::plain(0x2f6fd8, 0.5);
    let pale = Mat::plain(0xdfe8ff, 0.5);
    // En } av tre biter: strek opp, knekk, strek ned (5 × 9 cm).
    let w = 0.012;
    let seg = |m: &mut Mesh, a: Vec3, b: Vec3, mat: &Mat| {
        let d = (b - a).normalize_or_zero();
        let n = Vec3::new(-d.y, d.x, 0.0) * w;
        let c = [a - n, a + n, b + n, b - n];
        m.quad(&Affine3A::IDENTITY, c, mat, [1.0; 4]);
        let c2 = [a + n, a - n, b - n, b + n];
        m.quad(&Affine3A::IDENTITY, c2, mat, [1.0; 4]);
    };
    seg(
        &mut m,
        Vec3::new(-0.02, 0.045, 0.0),
        Vec3::new(0.0, 0.03, 0.0),
        &blue,
    );
    seg(
        &mut m,
        Vec3::new(0.0, 0.03, 0.0),
        Vec3::new(0.0, 0.01, 0.0),
        &blue,
    );
    seg(
        &mut m,
        Vec3::new(0.0, 0.01, 0.0),
        Vec3::new(0.02, 0.0, 0.0),
        &pale,
    );
    seg(
        &mut m,
        Vec3::new(0.02, 0.0, 0.0),
        Vec3::new(0.0, -0.01, 0.0),
        &pale,
    );
    seg(
        &mut m,
        Vec3::new(0.0, -0.01, 0.0),
        Vec3::new(0.0, -0.03, 0.0),
        &blue,
    );
    seg(
        &mut m,
        Vec3::new(0.0, -0.03, 0.0),
        Vec3::new(-0.02, -0.045, 0.0),
        &blue,
    );
    for v in &mut m.vertices {
        v.mat[0] = kind::MODEL;
    }
    Model {
        name: DEBRIS_MODEL.into(),
        mesh: m,
        albedo: None,
        normal: None,
        metal_rough: None,
        emissive: None,
        roughness: 0.5,
        metallic: 0.0,
        far_cell: 0.2,
        far_mesh: None,
        rig: None,
    }
}
