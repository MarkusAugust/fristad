//! Fiendene i Legacy-kjelleren: korrupte komponenter som har mutert av
//! for mye JavaScript og forretningslogikk. Den første er den korrupte
//! knappen (`mutant_button`): den jager deg på rutenettet, varsler med
//! kroppen før den biter, rykker til når den treffes og segner om når den
//! dør, og faller fra hverandre i tokens. Figurene er skinnet
//! (`domain/anim.rs`): spillet driver en `Animator` per fiende og gir
//! rendereren leddmatrisene gjennom `Frame.skins`.

use crate::domain::anim::{Animator, Rig};
use crate::domain::city::dungeon::Dungeon;
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Mat4, Quat, Vec3};

pub const MOB_MODEL: &str = "mutant_button";
pub const TOKEN_MODEL: &str = "token";
pub const KEY_MODEL: &str = "key";
const MAX_MOBS: usize = 72;
const MAX_TOKENS: usize = 160;
const MAX_KEYS: usize = 8;
/// Elitefienden: større, seigere, og bærer nøkkelen til porten.
const ELITE_SCALE: f32 = 1.35;
const ELITE_HP: f32 = 260.0;
/// Fienden ser deg innen denne avstanden og jager.
const SIGHT: f32 = 26.0;
const SPEED: f32 = 2.1;
/// Så nær må du være før den går til angrep, og så langt rekker bittet
/// (den stuper et stykke fram i angrepet).
const ATTACK_RANGE: f32 = 1.7;
const BITE_REACH: f32 = 2.3;
const BITE_DAMAGE: f32 = 12.0;
const ATTACK_COOLDOWN: f32 = 0.9;
/// Sekundet i attack-klippet der bittet lander (bilde 15 av 30 i 24 fps,
/// `ATTACK_HIT` i tools/models/mobs.py).
const ATTACK_HIT: f32 = 15.0 / 24.0;
/// Uten rigg: angrepet varer så lenge, og bittet lander ved ATTACK_HIT.
const ATTACK_LEN: f32 = 1.25;
const HIT_LEN: f32 = 0.5;
const DIE_LEN: f32 = 1.5;
/// Liket ligger så lenge etter dødsklippet.
const CORPSE: f32 = 4.0;
pub const MOB_HP: f32 = 100.0;
/// Kroppen (til sikting): en stående kapsel fra beina til over hodet.
const BODY_TOP: f32 = 1.45;
const BODY_R: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Walk,
    /// Varsler og biter; `bitten` når bittet har landet (eller bommet).
    Attack { bitten: bool },
    Hit,
    Dying,
    Gone,
}

/// Klippene i riggen, slått opp én gang.
#[derive(Clone, Copy, Debug, Default)]
struct Clips {
    idle: usize,
    walk: usize,
    attack: usize,
    hit: usize,
    die: usize,
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub pos: Vec3,
    pub yaw: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub elite: bool,
    pub state: State,
    /// Sekunder siden den ble truffet (blink).
    pub hit: f32,
    /// Sekunder i tilstanden (til dem uten rigg, og til liket).
    t: f32,
    /// Står lamslått minst så lenge i Hit (paraden gir lang stun).
    stun: f32,
    cooldown: f32,
    phase: f32,
    anim: Option<Animator>,
    /// Neste cellemidte på veien mot spilleren.
    target: Option<Vec3>,
    repath: f32,
}

impl Mob {
    pub fn alive(&self) -> bool {
        !matches!(self.state, State::Dying | State::Gone)
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub pos: Vec3,
    pub age: f32,
    pub taken: bool,
}

/// Det du plukker opp: tokens, og nøkler til portene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pickup {
    Token,
    Key,
}

/// Det som skjedde dette bildet, til spillet.
#[derive(Default, Debug)]
pub struct Outcome {
    /// Bitt som landet: (fiende, skade, hvor det kom fra).
    pub bites: Vec<(usize, f32, Vec3)>,
    pub tokens: u32,
    pub keys: u32,
    /// Fiender som begynte et angrep (til lyd og varsel).
    pub windups: u32,
}

#[derive(Default)]
pub struct Mobs {
    pub mobs: Vec<Mob>,
    pub tokens: Vec<Token>,
    pub keys: Vec<Token>,
    seed: u64,
    rig: Option<Rig>,
    clips: Clips,
}

impl Mobs {
    pub fn new(dungeon: &Dungeon, rig: Option<Rig>) -> Self {
        let clips = rig
            .as_ref()
            .map(|r| {
                let find = |n: &str| r.clip(n).unwrap_or(0);
                Clips { idle: find("idle"), walk: find("walk"), attack: find("attack"), hit: find("hit"), die: find("die") }
            })
            .unwrap_or_default();
        if let Some(r) = &rig {
            for n in ["idle", "walk", "attack", "hit", "die"] {
                if r.clip(n).is_none() {
                    log::warn!("Riggen til {MOB_MODEL} mangler klippet {n}");
                }
            }
        }
        let mut m = Self { mobs: vec![], tokens: vec![], keys: vec![], seed: 0x9e37_79b9_7f4a_7c15, rig, clips };
        m.reset(dungeon);
        m
    }

    fn rnd(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Ny besetning: en fiende på hvert startsted og en elite i hvert
    /// utgangsrom, opptil MAX_MOBS, og skatten nederst.
    pub fn reset(&mut self, dungeon: &Dungeon) {
        self.tokens.clear();
        self.keys.clear();
        self.mobs.clear();
        let spawns: Vec<(Vec3, bool)> = dungeon.elites.iter().map(|p| (*p, true)).chain(dungeon.spawns.iter().map(|p| (*p, false))).take(MAX_MOBS).collect();
        for (p, elite) in spawns {
            let phase = self.rnd() * 6.0;
            let yaw = self.rnd() * std::f32::consts::TAU;
            let repath = self.rnd() * 0.5;
            let mut anim = self.rig.as_ref().map(|_| Animator::new(self.clips.idle, true));
            if let Some(a) = &mut anim {
                a.time = phase;
            }
            let hp = if elite { ELITE_HP } else { MOB_HP };
            self.mobs.push(Mob { pos: p, yaw, hp, max_hp: hp, elite, state: State::Idle, hit: 9.0, t: 0.0, stun: 0.0, cooldown: 0.5, phase, anim, target: None, repath });
        }
        for p in &dungeon.loot {
            self.tokens.push(Token { pos: *p, age: 9.0, taken: false });
        }
    }

    /// Hvor mange instanser hver modell trenger plass til.
    pub fn reserve() -> Vec<(&'static str, usize)> {
        vec![(MOB_MODEL, MAX_MOBS), (TOKEN_MODEL, MAX_TOKENS), (KEY_MODEL, MAX_KEYS)]
    }

    pub fn alive(&self) -> usize {
        self.mobs.iter().filter(|m| m.alive()).count()
    }

    /// Fienden strålen fra øyet treffer innen `max` meter, om noen.
    pub fn aimed(&self, eye: Vec3, dir: Vec3, max: f32) -> Option<usize> {
        // Fienden er en stående kapsel fra beina til over hodet, så du
        // treffer også når du sikter litt høyt eller lavt.
        let mut best: Option<(usize, f32)> = None;
        for (i, m) in self.mobs.iter().enumerate() {
            if !m.alive() {
                continue;
            }
            let s = if m.elite { ELITE_SCALE } else { 1.0 };
            let (a, b) = (m.pos + Vec3::Y * 0.2, m.pos + Vec3::Y * BODY_TOP * s);
            // Nærmeste avstand mellom strålen (eye + dir·t, t ≥ 0) og segmentet a–b.
            let (d1, d2, r) = (dir, b - a, eye - a);
            let (a11, a12, a22) = (d1.dot(d1), -d1.dot(d2), d2.dot(d2));
            let (b1, b2) = (-d1.dot(r), d2.dot(r));
            let det = a11 * a22 - a12 * a12;
            let u = if det.abs() > 1e-6 { (a11 * b2 - a12 * b1) / det } else { 0.0 };
            let u = u.clamp(0.0, 1.0);
            let t = (d1.dot(a + d2 * u - eye)).max(0.0);
            if t > max + BODY_R {
                continue;
            }
            let miss = (eye + d1 * t - (a + d2 * u)).length();
            if miss < BODY_R * s && best.map_or(true, |(_, bt)| t < bt) {
                best = Some((i, t));
            }
        }
        best.map(|(i, _)| i)
    }

    fn play(&mut self, i: usize, clip: usize, looping: bool, fade: f32) {
        if let (Some(rig), Some(a)) = (&self.rig, self.mobs[i].anim.as_mut()) {
            let _ = rig;
            a.play(clip, looping, fade);
        }
    }

    /// Et slag mot fiende `i` fra `from`: skade, rykk bakover, og tokens
    /// om den dør. Gir `true` om den døde nå.
    pub fn strike(&mut self, i: usize, damage: f32, from: Vec3, dungeon: &Dungeon) -> bool {
        let Some(m) = self.mobs.get_mut(i) else { return false };
        if !m.alive() {
            return false;
        }
        m.hp -= damage;
        m.hit = 0.0;
        let push = (m.pos - from).with_y(0.0).normalize_or_zero() * 0.5;
        if dungeon.walkable(m.pos + push, 0.5) {
            m.pos += push;
        }
        if m.hp <= 0.0 {
            m.state = State::Dying;
            m.t = 0.0;
            let base = m.pos;
            let elite = m.elite;
            let die = self.clips.die;
            self.play(i, die, false, 0.08);
            if elite && self.keys.len() < MAX_KEYS {
                self.keys.push(Token { pos: base + Vec3::new(0.0, 0.0, 0.3), age: 0.0, taken: false });
            }
            for k in 0..(if elite { 8 } else { 3 }) {
                if self.tokens.len() >= MAX_TOKENS {
                    break;
                }
                let a = k as f32 * 2.1 + self.rnd() * 0.8;
                let p = base + Vec3::new(a.cos() * 0.5, 0.0, a.sin() * 0.5);
                let p = if dungeon.walkable(p, 0.3) { p } else { base };
                self.tokens.push(Token { pos: p, age: 0.0, taken: false });
            }
            return true;
        }
        m.state = State::Hit;
        m.t = 0.0;
        m.stun = 0.0;
        let hit = self.clips.hit;
        self.play(i, hit, false, 0.05);
        false
    }

    /// Lamslått i `secs` sekunder (etter en parade): står og rister.
    pub fn stagger(&mut self, i: usize, secs: f32) {
        let Some(m) = self.mobs.get_mut(i) else { return };
        if !m.alive() {
            return;
        }
        m.state = State::Hit;
        m.t = 0.0;
        m.stun = secs;
        m.hit = 0.0;
        m.cooldown = secs + ATTACK_COOLDOWN;
        let hit = self.clips.hit;
        self.play(i, hit, false, 0.05);
    }

    /// Flytter fiendene, lar dem angripe, og plukker tokens. `player` er
    /// føttene.
    pub fn update(&mut self, dt: f32, player: Vec3, in_cellar: bool, dungeon: &Dungeon) -> Outcome {
        let mut out = Outcome::default();
        let n = self.mobs.len();
        let clips = self.clips;
        for i in 0..n {
            let rig = self.rig.as_ref();
            let (before, rest) = self.mobs.split_at_mut(i);
            let (m, after) = rest.split_first_mut().unwrap();
            m.hit += dt;
            m.cooldown -= dt;
            m.phase += dt;
            m.t += dt;
            if let (Some(r), Some(a)) = (rig, m.anim.as_mut()) {
                a.update(dt, r);
            }
            // Hvor langt et klipp uten løkke er kommet: fra riggen, eller
            // fra klokka om modellen mangler.
            let finished = |m: &Mob, len: f32| match (rig, &m.anim) {
                (Some(r), Some(a)) => {
                    let _ = r;
                    a.finished
                }
                _ => m.t >= len,
            };
            let clip_time = |m: &Mob| match &m.anim {
                Some(a) => a.time,
                None => m.t,
            };
            let to_player = (player - m.pos).with_y(0.0);
            let dist = to_player.length();
            // Spilleren i en annen etasje er utenfor syne.
            let same_floor = (player.y - m.pos.y).abs() < 5.0;
            match m.state {
                State::Gone => continue,
                State::Dying => {
                    if finished(m, DIE_LEN) && m.t > DIE_LEN + CORPSE {
                        m.state = State::Gone;
                    }
                    continue;
                }
                State::Hit => {
                    if finished(m, HIT_LEN) && m.t >= m.stun {
                        m.state = State::Idle;
                        if let Some(a) = m.anim.as_mut() {
                            a.play(clips.idle, true, 0.15);
                        }
                    }
                    continue;
                }
                State::Attack { bitten } => {
                    // Snur seg mot deg mens den varsler, og biter når
                    // kjeven smeller igjen, om du fortsatt er innen rekkevidde.
                    if !bitten {
                        if dist > 0.01 {
                            let want = to_player.x.atan2(to_player.z);
                            let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                            m.yaw += diff * (dt * 4.0).min(1.0);
                        }
                        if clip_time(m) >= ATTACK_HIT {
                            let facing = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
                            if dist <= BITE_REACH * (if m.elite { 1.3 } else { 1.0 }) && facing.dot(to_player.normalize_or_zero()) > 0.3 {
                                out.bites.push((i, if m.elite { BITE_DAMAGE * 1.8 } else { BITE_DAMAGE }, m.pos));
                            }
                            m.state = State::Attack { bitten: true };
                        }
                    }
                    if finished(m, ATTACK_LEN) {
                        m.state = State::Idle;
                        m.cooldown = ATTACK_COOLDOWN;
                        if let Some(a) = m.anim.as_mut() {
                            a.play(clips.idle, true, 0.15);
                        }
                    }
                    continue;
                }
                State::Idle | State::Walk => {}
            }
            if !in_cellar || dist > SIGHT || !same_floor {
                // Står og vagger.
                m.target = None;
                if m.state != State::Idle {
                    m.state = State::Idle;
                    if let Some(a) = m.anim.as_mut() {
                        a.play(clips.idle, true, 0.2);
                    }
                }
                continue;
            }
            if dist <= ATTACK_RANGE && m.cooldown <= 0.0 {
                m.state = State::Attack { bitten: false };
                m.t = 0.0;
                if let Some(a) = m.anim.as_mut() {
                    a.play(clips.attack, false, 0.08);
                }
                out.windups += 1;
                continue;
            }
            m.repath -= dt;
            if m.target.is_none() || m.repath <= 0.0 {
                m.target = dungeon.next_step(m.pos, player);
                m.repath = 0.5;
            }
            // Nær nok, men angrepet er ikke klart: står og vender seg mot deg.
            if dist <= ATTACK_RANGE {
                m.yaw = to_player.x.atan2(to_player.z);
                if m.state != State::Idle {
                    m.state = State::Idle;
                    if let Some(a) = m.anim.as_mut() {
                        a.play(clips.idle, true, 0.15);
                    }
                }
                continue;
            }
            let goal = m.target.unwrap_or(player);
            let dir = (goal - m.pos).with_y(0.0).normalize_or_zero();
            let mut step = dir * SPEED * dt;
            // Hold avstand til de andre.
            for o in before.iter().chain(after.iter()) {
                if !o.alive() {
                    continue;
                }
                let d = (m.pos - o.pos).with_y(0.0);
                let l = d.length();
                if l < 1.2 && l > 0.01 {
                    step += d / l * (1.2 - l) * dt * 2.0;
                }
            }
            let next = m.pos + step;
            let from = m.pos;
            if dungeon.walkable(next, 0.6) {
                m.pos = next;
            } else if dungeon.walkable(Vec3::new(next.x, m.pos.y, m.pos.z), 0.6) {
                m.pos.x = next.x;
            } else if dungeon.walkable(Vec3::new(m.pos.x, m.pos.y, next.z), 0.6) {
                m.pos.z = next.z;
            }
            m.pos.y = dungeon.height_at(m.pos);
            let moved = (m.pos - from).with_y(0.0).length() / dt.max(1e-4);
            let want_state = if moved > 0.3 { State::Walk } else { State::Idle };
            if m.state != want_state {
                m.state = want_state;
                if let Some(a) = m.anim.as_mut() {
                    a.play(if want_state == State::Walk { clips.walk } else { clips.idle }, true, 0.2);
                }
            }
            if let Some(a) = m.anim.as_mut() {
                if want_state == State::Walk {
                    a.speed = (moved / SPEED).clamp(0.5, 1.4);
                }
            }
            if dir.length_squared() > 0.0 {
                let want = dir.x.atan2(dir.z);
                let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                m.yaw += diff * (dt * 6.0).min(1.0);
            }
        }
        for t in &mut self.tokens {
            t.age += dt;
            if !t.taken && in_cellar && (t.pos - player).with_y(0.0).length() < 1.0 && (t.pos.y - player.y).abs() < 2.0 {
                t.taken = true;
                out.tokens += 1;
            }
        }
        for k in &mut self.keys {
            k.age += dt;
            if !k.taken && in_cellar && (k.pos - player).with_y(0.0).length() < 1.1 && (k.pos.y - player.y).abs() < 2.0 {
                k.taken = true;
                out.keys += 1;
            }
        }
        out
    }

    /// Instansene til rendereren, og leddmatrisene deres i `skins`.
    /// Uten rigg svever figuren og vagger som før.
    pub fn instances(&self, time: f32, skins: &mut Vec<Mat4>) -> Vec<ModelInstance> {
        let mut out = Vec::with_capacity(self.mobs.len() + self.tokens.len());
        for m in &self.mobs {
            if m.state == State::Gone {
                continue;
            }
            // Blinker hvitt rett etter et slag, og forstørres et øyeblikk.
            let flinch = (0.25 - m.hit).max(0.0) * 4.0;
            let flash = ((0.16 - m.hit) / 0.16).clamp(0.0, 1.0);
            match (&self.rig, &m.anim) {
                (Some(rig), Some(a)) => {
                    let pose = a.pose(rig);
                    let mats = rig.skin(&pose);
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    let s = if m.elite { ELITE_SCALE } else { 1.0 };
                    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(m.yaw), m.pos) * Affine3A::from_scale(Vec3::splat(s * (1.0 + flinch * 0.06)));
                    out.push(ModelInstance { model: MOB_MODEL, transform: t, skin: Some((base, mats.len() as u32)), flash });
                }
                _ => {
                    let bob = (m.phase * 2.3).sin() * 0.05;
                    let tilt = (m.phase * 1.7).cos() * 0.06;
                    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(m.yaw) * Quat::from_rotation_z(tilt - flinch * 0.3), m.pos + Vec3::Y * bob) * Affine3A::from_scale(Vec3::splat(1.0 + flinch * 0.15));
                    out.push(ModelInstance { model: MOB_MODEL, transform: t, skin: None, flash });
                }
            }
        }
        for t in &self.tokens {
            if t.taken {
                continue;
            }
            let spin = Quat::from_rotation_y(time * 2.0 + t.age);
            let pop = (t.age * 4.0).min(1.0);
            let hover = 0.1 + (time * 3.0 + t.age).sin() * 0.05;
            out.push(ModelInstance { model: TOKEN_MODEL, transform: Affine3A::from_rotation_translation(spin, t.pos + Vec3::Y * hover) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0 });
        }
        for k in &self.keys {
            if k.taken {
                continue;
            }
            let spin = Quat::from_rotation_y(time * 1.5 + k.age) * Quat::from_rotation_x(0.4);
            let pop = (k.age * 4.0).min(1.0);
            let hover = 0.25 + (time * 2.5 + k.age).sin() * 0.06;
            out.push(ModelInstance { model: KEY_MODEL, transform: Affine3A::from_rotation_translation(spin, k.pos + Vec3::Y * hover) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0 });
        }
        out
    }
}
