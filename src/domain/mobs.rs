//! Fiendene i Legacy-kjelleren: korrupte komponenter som har mutert av
//! for mye JavaScript og forretningslogikk. De jager deg på rutenettet,
//! varsler med kroppen før de angriper, rykker til når de treffes og
//! segner om når de dør, og faller fra hverandre i tokens. Figurene er
//! skinnet (`domain/anim.rs`): spillet driver en `Animator` per fiende og
//! gir rendereren leddmatrisene gjennom `Frame.skins`.
//!
//! Slagene (`Kind`, modellene fra `tools/models/mobs.py`): den korrupte
//! knappen biter; tooltipen er liten, rask og går i flokk; inputen
//! spytter markører (`Bolt`) på avstand; modalen er en tank som slår med
//! begge nevene og tar lite skade forfra; tabellen er minibossen som
//! bærer nøkkelen på de dypeste etasjene; og Rammeverket er sluttbossen
//! nederst: sveiper, slår i bakken (alle innen `SLAM_RADIUS` tar skade)
//! og brøler fram tooltips når det mister liv.

use crate::domain::anim::{Animator, Rig};
use crate::domain::world::dungeon::Dungeon;
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Mat4, Quat, Vec3};

pub const TOKEN_MODEL: &str = "token";
pub const KEY_MODEL: &str = "key";
pub const BOLT_MODEL: &str = "bolt";
pub const MAX_MOBS: usize = 72;
const MAX_TOKENS: usize = 160;
const MAX_KEYS: usize = 8;
const MAX_BOLTS: usize = 24;
/// Elitefienden: større, seigere, og bærer nøkkelen til porten.
const ELITE_SCALE: f32 = 1.3;
const ELITE_HP: f32 = 2.6;
const HIT_LEN: f32 = 0.5;
const DIE_LEN: f32 = 1.5;
/// Liket ligger så lenge etter dødsklippet.
const CORPSE: f32 = 4.0;
pub const MOB_HP: f32 = 100.0;
/// Kroppen (til sikting) i modellens målestokk: en stående kapsel fra
/// beina til over hodet.
const BODY_TOP: f32 = 1.45;
const BODY_R: f32 = 0.5;
const BOLT_SPEED: f32 = 11.0;
const BOLT_LIFE: f32 = 2.2;
/// Bossens slag i bakken rammer alle innen denne avstanden.
const SLAM_RADIUS: f32 = 4.5;
const FPS: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Button,
    Tooltip,
    Input,
    Modal,
    Table,
    Framework,
}

/// Tallene bak et slag.
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    /// Så nær må du være før den går til angrep, og så langt rekker slaget.
    pub range: f32,
    pub reach: f32,
    pub scale: f32,
    pub sight: f32,
    pub cooldown: f32,
    /// Sekundet i attack-klippet der slaget lander, og klippets lengde.
    pub attack_hit: f32,
    pub attack_len: f32,
    /// Skyter på avstand: (nærmeste, lengste avstand, skade).
    pub bolt: Option<(f32, f32, f32)>,
    /// Så mye av skaden slipper gjennom forfra når den ikke angriper.
    pub armor: f32,
    /// Slag under dette avbryter den ikke.
    pub poise: f32,
    pub tokens: u32,
}

impl Kind {
    pub const ALL: [Kind; 6] = [Kind::Button, Kind::Tooltip, Kind::Input, Kind::Modal, Kind::Table, Kind::Framework];

    pub fn model(self) -> &'static str {
        match self {
            Kind::Button => "mutant_button",
            Kind::Tooltip => "mutant_tooltip",
            Kind::Input => "mutant_input",
            Kind::Modal => "mutant_modal",
            Kind::Table => "mutant_table",
            Kind::Framework => "framework",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Button => "Korrupt knapp",
            Kind::Tooltip => "Korrupt tooltip",
            Kind::Input => "Korrupt input",
            Kind::Modal => "Korrupt modal",
            Kind::Table => "Korrupt tabell",
            Kind::Framework => "Rammeverket",
        }
    }

    pub fn boss(self) -> bool {
        self == Kind::Framework
    }

    pub fn stats(self) -> Stats {
        let bite = (15.0 / FPS, 30.0 / FPS);
        let slam = (28.0 / FPS, 44.0 / FPS);
        match self {
            Kind::Button => Stats { hp: MOB_HP, speed: 2.1, damage: 12.0, range: 1.7, reach: 2.3, scale: 1.0, sight: 26.0, cooldown: 0.9, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Tooltip => Stats { hp: 38.0, speed: 3.6, damage: 6.0, range: 1.3, reach: 1.7, scale: 0.55, sight: 30.0, cooldown: 0.7, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 1 },
            Kind::Input => Stats { hp: 80.0, speed: 1.9, damage: 10.0, range: 1.8, reach: 2.2, scale: 1.15, sight: 34.0, cooldown: 1.6, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.5, 14.0, 14.0)), armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Modal => Stats { hp: 320.0, speed: 1.3, damage: 30.0, range: 2.2, reach: 3.0, scale: 1.5, sight: 24.0, cooldown: 1.8, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.3, poise: 40.0, tokens: 6 },
            Kind::Table => Stats { hp: 420.0, speed: 2.4, damage: 22.0, range: 2.2, reach: 3.0, scale: 1.4, sight: 30.0, cooldown: 1.1, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 30.0, tokens: 10 },
            Kind::Framework => Stats { hp: 1400.0, speed: 1.6, damage: 34.0, range: 3.0, reach: 4.0, scale: 2.3, sight: 40.0, cooldown: 1.2, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 55.0, tokens: 40 },
        }
    }

    /// Hva et vanlig startsted i etasje `floor` får (`r` i 0..1), og hvor
    /// mange: tooltipene kommer i flokk.
    pub fn spawn(floor: usize, r: f32) -> (Kind, usize) {
        let table: &[(Kind, f32)] = match floor {
            0 => &[(Kind::Button, 1.0)],
            1 => &[(Kind::Button, 0.65), (Kind::Tooltip, 0.35)],
            2 => &[(Kind::Button, 0.45), (Kind::Tooltip, 0.25), (Kind::Input, 0.3)],
            3 => &[(Kind::Button, 0.3), (Kind::Tooltip, 0.2), (Kind::Input, 0.3), (Kind::Modal, 0.2)],
            _ => &[(Kind::Button, 0.2), (Kind::Tooltip, 0.25), (Kind::Input, 0.25), (Kind::Modal, 0.3)],
        };
        let mut acc = 0.0;
        for (k, w) in table {
            acc += w;
            if r < acc {
                return (*k, if *k == Kind::Tooltip { 3 } else { 1 });
            }
        }
        (Kind::Button, 1)
    }

    /// Eliten som bærer nøkkelen: knappen øverst, tabellen nederst.
    pub fn elite(floor: usize) -> Kind {
        if floor >= 3 {
            Kind::Table
        } else {
            Kind::Button
        }
    }
}

/// Angrepet som er i gang.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blow {
    /// Bitt, sveip eller nevene: rekkevidde og kjegle foran.
    Melee,
    /// Markøren spyttes.
    Spit,
    /// Bossen slår i bakken: alle innen SLAM_RADIUS.
    Slam,
    /// Bossen brøler og kaller på tooltips.
    Roar,
}

impl Blow {
    /// (sekundet slaget lander, klippets lengde).
    fn timing(self, s: &Stats) -> (f32, f32) {
        match self {
            Blow::Melee => (s.attack_hit, s.attack_len),
            Blow::Spit => (15.0 / FPS, 30.0 / FPS),
            Blow::Slam => (28.0 / FPS, 44.0 / FPS),
            Blow::Roar => (12.0 / FPS, 36.0 / FPS),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Walk,
    /// Varsler og slår; `landed` når slaget har landet (eller bommet).
    Attack { blow: Blow, landed: bool },
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
    slam: Option<usize>,
    roar: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub kind: Kind,
    pub pos: Vec3,
    pub yaw: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub elite: bool,
    pub state: State,
    /// Sekunder siden den ble truffet (blink).
    pub hit: f32,
    /// Om den har sett deg (bossens stolpe vises da).
    pub aware: bool,
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
    /// Hvor mange ganger bossen har brølt (ved 2/3 og 1/3 liv).
    roars: u32,
}

impl Mob {
    pub fn alive(&self) -> bool {
        !matches!(self.state, State::Dying | State::Gone)
    }

    pub fn scale(&self) -> f32 {
        self.kind.stats().scale * if self.elite { ELITE_SCALE } else { 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub pos: Vec3,
    pub age: f32,
    pub taken: bool,
}

/// En markør som flyr: hvem som spyttet den, hvor den er og hvor den skal.
#[derive(Clone, Debug)]
pub struct Bolt {
    pub owner: usize,
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub damage: f32,
}

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
}

/// Det som skjedde dette bildet, til spillet.
#[derive(Default, Debug)]
pub struct Outcome {
    /// Slag som landet: (fiende, skade, hvor det kom fra).
    pub bites: Vec<(usize, f32, Vec3)>,
    pub tokens: u32,
    pub keys: u32,
    /// Fiender som begynte et angrep (til lyd og varsel).
    pub windups: u32,
    /// Markører som ble spyttet.
    pub spits: u32,
    /// Slag i bakken (kameraet rister).
    pub quakes: u32,
    /// Brøl.
    pub roars: u32,
}

#[derive(Default)]
pub struct Mobs {
    pub mobs: Vec<Mob>,
    pub tokens: Vec<Token>,
    pub keys: Vec<Token>,
    pub bolts: Vec<Bolt>,
    seed: u64,
    rigs: Vec<Option<Rig>>,
    clips: Vec<Clips>,
}

impl Mobs {
    pub fn new(dungeon: &Dungeon, rigs: &[(String, Rig)]) -> Self {
        let mut all_rigs = vec![];
        let mut clips = vec![];
        for k in Kind::ALL {
            let rig = rigs.iter().find(|(n, _)| n == k.model()).map(|(_, r)| r.clone());
            let c = rig
                .as_ref()
                .map(|r| {
                    let find = |n: &str| r.clip(n).unwrap_or(0);
                    for n in ["idle", "walk", "attack", "hit", "die"] {
                        if r.clip(n).is_none() {
                            log::warn!("Riggen til {} mangler klippet {n}", k.model());
                        }
                    }
                    Clips { idle: find("idle"), walk: find("walk"), attack: find("attack"), hit: find("hit"), die: find("die"), slam: r.clip("slam"), roar: r.clip("roar") }
                })
                .unwrap_or_default();
            if rig.is_none() {
                log::warn!("Modellen {} har ingen rigg; fienden svever", k.model());
            }
            all_rigs.push(rig);
            clips.push(c);
        }
        let mut m = Self { mobs: vec![], tokens: vec![], keys: vec![], bolts: vec![], seed: 0x9e37_79b9_7f4a_7c15, rigs: all_rigs, clips };
        m.reset(dungeon);
        m
    }

    fn rnd(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }

    fn spawn(&mut self, kind: Kind, p: Vec3, elite: bool) {
        if self.mobs.len() >= MAX_MOBS {
            return;
        }
        let phase = self.rnd() * 6.0;
        let yaw = self.rnd() * std::f32::consts::TAU;
        let repath = self.rnd() * 0.5;
        let clips = self.clips[kind as usize];
        let mut anim = self.rigs[kind as usize].as_ref().map(|_| Animator::new(clips.idle, true));
        if let Some(a) = &mut anim {
            a.time = phase;
        }
        let hp = kind.stats().hp * if elite { ELITE_HP } else { 1.0 };
        self.mobs.push(Mob { kind, pos: p, yaw, hp, max_hp: hp, elite, state: State::Idle, hit: 9.0, aware: false, t: 0.0, stun: 0.0, cooldown: 0.5, phase, anim, target: None, repath, roars: 0 });
    }

    /// Ny besetning: fiender etter etasjens tabell på hvert startsted, en
    /// elite i hvert utgangsrom, bossen nederst, og skatten.
    pub fn reset(&mut self, dungeon: &Dungeon) {
        self.tokens.clear();
        self.keys.clear();
        self.bolts.clear();
        self.mobs.clear();
        if let Some(p) = dungeon.boss {
            self.spawn(Kind::Framework, p, false);
        }
        for p in dungeon.elites.clone() {
            let floor = dungeon.floor_of(p).unwrap_or(0);
            self.spawn(Kind::elite(floor), p, true);
        }
        for p in dungeon.spawns.clone() {
            let floor = dungeon.floor_of(p).unwrap_or(0);
            let r = self.rnd();
            let (kind, n) = Kind::spawn(floor, r);
            for k in 0..n {
                let a = k as f32 * 2.1;
                let q = p + Vec3::new(a.cos(), 0.0, a.sin()) * if n > 1 { 0.9 } else { 0.0 };
                let q = if dungeon.walkable(q, 0.4) { q } else { p };
                self.spawn(kind, q, false);
            }
        }
        for p in &dungeon.loot {
            self.tokens.push(Token { pos: *p, age: 9.0, taken: false });
        }
    }

    /// Hvor mange instanser hver modell trenger plass til.
    pub fn reserve() -> Vec<(&'static str, usize)> {
        let mut v: Vec<(&'static str, usize)> = Kind::ALL.iter().map(|k| (k.model(), MAX_MOBS)).collect();
        v.push((TOKEN_MODEL, MAX_TOKENS));
        v.push((KEY_MODEL, MAX_KEYS));
        v.push((BOLT_MODEL, MAX_BOLTS));
        v
    }

    pub fn alive(&self) -> usize {
        self.mobs.iter().filter(|m| m.alive()).count()
    }

    /// Bossen, om den lever og har sett deg: (navn, liv 0..1).
    pub fn boss(&self) -> Option<(&'static str, f32)> {
        self.mobs.iter().find(|m| m.kind.boss() && m.alive() && m.aware).map(|m| (m.kind.label(), (m.hp / m.max_hp).clamp(0.0, 1.0)))
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
            let s = m.scale();
            let (a, b) = (m.pos + Vec3::Y * 0.2 * s, m.pos + Vec3::Y * BODY_TOP * s);
            // Nærmeste avstand mellom strålen (eye + dir·t, t ≥ 0) og segmentet a–b.
            let (d1, d2, r) = (dir, b - a, eye - a);
            let (a11, a12, a22) = (d1.dot(d1), -d1.dot(d2), d2.dot(d2));
            let (b1, b2) = (-d1.dot(r), d2.dot(r));
            let det = a11 * a22 - a12 * a12;
            let u = if det.abs() > 1e-6 { (a11 * b2 - a12 * b1) / det } else { 0.0 };
            let u = u.clamp(0.0, 1.0);
            let t = (d1.dot(a + d2 * u - eye)).max(0.0);
            if t > max + BODY_R * s {
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
        if let Some(a) = self.mobs[i].anim.as_mut() {
            a.play(clip, looping, fade);
        }
    }

    /// Et slag mot fiende `i` fra `from`: skade, rykk bakover, og tokens
    /// om den dør.
    pub fn strike(&mut self, i: usize, damage: f32, from: Vec3, dungeon: &Dungeon) -> Strike {
        let Some(m) = self.mobs.get_mut(i) else { return Strike::default() };
        if !m.alive() {
            return Strike::default();
        }
        let stats = m.kind.stats();
        let mut damage = damage;
        let mut armored = false;
        // Panseret: modalen tar lite forfra når den ikke er midt i et slag.
        if stats.armor < 1.0 && matches!(m.state, State::Idle | State::Walk) {
            let facing = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
            let to = (from - m.pos).with_y(0.0).normalize_or_zero();
            if facing.dot(to) > 0.2 {
                damage *= stats.armor;
                armored = true;
            }
        }
        m.hp -= damage;
        m.hit = 0.0;
        m.aware = true;
        let push = (m.pos - from).with_y(0.0).normalize_or_zero() * 0.5 / m.scale();
        if dungeon.walkable(m.pos + push, 0.5) && dungeon.clear(m.pos + push, 0.3 * m.scale()) {
            m.pos += push;
        }
        let clips = self.clips[m.kind as usize];
        if m.hp <= 0.0 {
            m.state = State::Dying;
            m.t = 0.0;
            let base = m.pos;
            let elite = m.elite;
            let n = if elite { 8 } else { stats.tokens };
            self.play(i, clips.die, false, 0.08);
            if elite && self.keys.len() < MAX_KEYS {
                self.keys.push(Token { pos: base + Vec3::new(0.0, 0.0, 0.3), age: 0.0, taken: false });
            }
            for k in 0..n {
                if self.tokens.len() >= MAX_TOKENS {
                    break;
                }
                let a = k as f32 * 2.1 + self.rnd() * 0.8;
                let r = 0.5 + (k / 6) as f32 * 0.4;
                let p = base + Vec3::new(a.cos() * r, 0.0, a.sin() * r);
                let p = if dungeon.walkable(p, 0.3) { p } else { base };
                self.tokens.push(Token { pos: p, age: 0.0, taken: false });
            }
            return Strike { killed: true, armored };
        }
        // De tunge lar seg ikke avbryte av lette slag.
        if damage < stats.poise {
            return Strike { killed: false, armored };
        }
        m.state = State::Hit;
        m.t = 0.0;
        m.stun = 0.0;
        self.play(i, clips.hit, false, 0.05);
        Strike { killed: false, armored }
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
        self.play(i, clips.hit, false, 0.05);
    }

    /// Fri sikt fra fienden til spilleren, grovt: punkter langs linja
    /// ligger i åpne celler.
    fn line_of_sight(dungeon: &Dungeon, from: Vec3, to: Vec3) -> bool {
        let n = ((to - from).length() / 1.0).ceil().max(1.0) as usize;
        (1..n).all(|k| dungeon.walkable(from.lerp(to, k as f32 / n as f32), 0.0))
    }

    /// Flytter fiendene, lar dem angripe, og plukker tokens. `player` er
    /// føttene.
    pub fn update(&mut self, dt: f32, player: Vec3, in_cellar: bool, dungeon: &Dungeon) -> Outcome {
        let mut out = Outcome::default();
        let n = self.mobs.len();
        let mut summons: Vec<Vec3> = vec![];
        let mut bolts: Vec<Bolt> = vec![];
        for i in 0..n {
            let kind = self.mobs[i].kind;
            let stats = kind.stats();
            let clips = self.clips[kind as usize];
            let r = self.rnd();
            let rig = self.rigs[kind as usize].as_ref();
            let (before, rest) = self.mobs.split_at_mut(i);
            let (m, after) = rest.split_first_mut().unwrap();
            m.hit += dt;
            m.cooldown -= dt;
            m.phase += dt;
            m.t += dt;
            if let (Some(r), Some(a)) = (rig, m.anim.as_mut()) {
                a.update(dt, r);
            }
            let s = m.scale();
            // Hvor langt et klipp uten løkke er kommet: fra riggen, eller
            // fra klokka om modellen mangler.
            let finished = |m: &Mob, len: f32| match &m.anim {
                Some(a) if rig.is_some() => a.finished,
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
                State::Attack { blow, landed } => {
                    let (hit_at, len) = blow.timing(&stats);
                    // Snur seg mot deg mens den varsler, og slår når
                    // klippet når treffpunktet, om du fortsatt er innen rekkevidde.
                    if !landed {
                        if dist > 0.01 && blow != Blow::Roar {
                            let want = to_player.x.atan2(to_player.z);
                            let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                            m.yaw += diff * (dt * 4.0).min(1.0);
                        }
                        if clip_time(m) >= hit_at {
                            let facing = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
                            match blow {
                                Blow::Melee => {
                                    let reach = stats.reach * if m.elite { 1.3 } else { 1.0 };
                                    if dist <= reach && facing.dot(to_player.normalize_or_zero()) > 0.3 {
                                        out.bites.push((i, stats.damage * if m.elite { 1.8 } else { 1.0 }, m.pos));
                                    }
                                }
                                Blow::Spit => {
                                    if let Some((_, _, dmg)) = stats.bolt {
                                        let from = m.pos + Vec3::Y * 1.3 * s + facing * 0.6 * s;
                                        let aim = (player + Vec3::Y * 0.9 - from).normalize_or_zero();
                                        bolts.push(Bolt { owner: i, pos: from, vel: aim * BOLT_SPEED, age: 0.0, damage: dmg });
                                        out.spits += 1;
                                    }
                                }
                                Blow::Slam => {
                                    out.quakes += 1;
                                    if dist <= SLAM_RADIUS {
                                        out.bites.push((i, stats.damage * 1.3, m.pos));
                                    }
                                }
                                Blow::Roar => {
                                    out.roars += 1;
                                    for k in 0..3 {
                                        let a = k as f32 * 2.1 + r * 6.0;
                                        summons.push(m.pos + Vec3::new(a.cos(), 0.0, a.sin()) * 2.2);
                                    }
                                }
                            }
                            m.state = State::Attack { blow, landed: true };
                        }
                    }
                    if finished(m, len) {
                        m.state = State::Idle;
                        m.cooldown = stats.cooldown;
                        if let Some(a) = m.anim.as_mut() {
                            a.play(clips.idle, true, 0.15);
                        }
                    }
                    continue;
                }
                State::Idle | State::Walk => {}
            }
            if !in_cellar || dist > stats.sight || !same_floor {
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
            m.aware = true;
            // Bossen brøler ved 2/3 og 1/3 liv og kaller på tooltips.
            if kind.boss() {
                let frac = m.hp / m.max_hp;
                let due = (m.roars == 0 && frac < 0.67) || (m.roars == 1 && frac < 0.34);
                if due {
                    m.roars += 1;
                    m.state = State::Attack { blow: Blow::Roar, landed: false };
                    m.t = 0.0;
                    if let Some(a) = m.anim.as_mut() {
                        a.play(clips.roar.unwrap_or(clips.attack), false, 0.1);
                    }
                    out.windups += 1;
                    continue;
                }
            }
            if dist <= stats.range && m.cooldown <= 0.0 {
                let blow = if kind.boss() && clips.slam.is_some() && r < 0.35 { Blow::Slam } else { Blow::Melee };
                m.state = State::Attack { blow, landed: false };
                m.t = 0.0;
                if let Some(a) = m.anim.as_mut() {
                    a.play(if blow == Blow::Slam { clips.slam.unwrap_or(clips.attack) } else { clips.attack }, false, 0.08);
                }
                out.windups += 1;
                continue;
            }
            // Skyter på avstand når sikten er fri.
            if let Some((near, far, _)) = stats.bolt {
                if dist >= near && dist <= far && m.cooldown <= 0.0 && Self::line_of_sight(dungeon, m.pos, player) {
                    m.yaw = to_player.x.atan2(to_player.z);
                    m.state = State::Attack { blow: Blow::Spit, landed: false };
                    m.t = 0.0;
                    if let Some(a) = m.anim.as_mut() {
                        a.play(clips.attack, false, 0.08);
                    }
                    out.windups += 1;
                    continue;
                }
            }
            m.repath -= dt;
            if m.target.is_none() || m.repath <= 0.0 {
                m.target = dungeon.next_step(m.pos, player);
                m.repath = 0.5;
            }
            // Nær nok, men angrepet er ikke klart: står og vender seg mot deg.
            if dist <= stats.range {
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
            let mut step = dir * stats.speed * dt;
            // Hold avstand til de andre.
            for o in before.iter().chain(after.iter()) {
                if !o.alive() {
                    continue;
                }
                let d = (m.pos - o.pos).with_y(0.0);
                let l = d.length();
                let keep = 0.6 * (s + o.scale());
                if l < keep && l > 0.01 {
                    step += d / l * (keep - l) * dt * 2.0;
                }
            }
            let from = m.pos;
            let margin = (0.6 * s).min(1.5);
            // Går rundt det som står i rommet (sarkofager, søyler, kister):
            // rett fram, langs aksene, eller skrått til sidene. Står den
            // allerede i noe (født der), får den gå ut av det.
            let body = 0.3 * s;
            let free = dungeon.clear(from, body);
            let ok = |q: Vec3| dungeon.walkable(q, margin) && (!free || dungeon.clear(q, body));
            let turn = |a: f32| Quat::from_rotation_y(a) * step;
            let tries = [step, Vec3::new(step.x, 0.0, 0.0), Vec3::new(0.0, 0.0, step.z), turn(0.9), turn(-0.9), turn(1.6), turn(-1.6)];
            if let Some(t) = tries.into_iter().find(|t| t.length_squared() > 1e-10 && ok(from + *t)) {
                m.pos = from + t;
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
                    // Skrittlengden følger størrelsen: små tasser fort.
                    a.speed = (moved / (2.1 * s)).clamp(0.5, 1.8);
                }
            }
            if dir.length_squared() > 0.0 {
                let want = dir.x.atan2(dir.z);
                let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                m.yaw += diff * (dt * 6.0).min(1.0);
            }
        }
        for p in summons {
            let p = if dungeon.walkable(p, 0.4) { p } else { continue };
            let p = Vec3::new(p.x, dungeon.height_at(p), p.z);
            self.spawn(Kind::Tooltip, p, false);
            if let Some(m) = self.mobs.last_mut() {
                m.cooldown = 1.0;
                m.aware = true;
            }
        }
        for b in bolts {
            if self.bolts.len() < MAX_BOLTS {
                self.bolts.push(b);
            }
        }
        // Markørene flyr rett fram til de treffer deg eller en vegg.
        let chest = player + Vec3::Y * 0.9;
        for b in &mut self.bolts {
            b.pos += b.vel * dt;
            b.age += dt;
            if (b.pos - chest).length() < 0.6 {
                out.bites.push((b.owner, b.damage, b.pos - b.vel));
                b.age = BOLT_LIFE + 1.0;
            } else if !dungeon.walkable(b.pos, 0.05) || (b.pos.y - dungeon.height_at(b.pos)) < 0.05 {
                b.age = BOLT_LIFE + 1.0;
            }
        }
        self.bolts.retain(|b| b.age < BOLT_LIFE);
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
            let s = m.scale();
            let model = m.kind.model();
            match (&self.rigs[m.kind as usize], &m.anim) {
                (Some(rig), Some(a)) => {
                    let pose = a.pose(rig);
                    let mats = rig.skin(&pose);
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    // Modellens front er +X; yaw regner front som (sin, 0, cos).
                    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(m.yaw - std::f32::consts::FRAC_PI_2), m.pos) * Affine3A::from_scale(Vec3::splat(s * (1.0 + flinch * 0.06)));
                    out.push(ModelInstance { model, transform: t, skin: Some((base, mats.len() as u32)), flash, tint: None });
                }
                _ => {
                    let bob = (m.phase * 2.3).sin() * 0.05;
                    let tilt = (m.phase * 1.7).cos() * 0.06;
                    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(m.yaw - std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(tilt - flinch * 0.3), m.pos + Vec3::Y * bob) * Affine3A::from_scale(Vec3::splat(s * (1.0 + flinch * 0.15)));
                    out.push(ModelInstance { model, transform: t, skin: None, flash, tint: None });
                }
            }
        }
        for b in &self.bolts {
            let yaw = b.vel.x.atan2(b.vel.z);
            let pitch = (b.vel.y / b.vel.length().max(1e-4)).asin();
            let rot = Quat::from_rotation_y(yaw - std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(pitch) * Quat::from_rotation_x(time * 12.0);
            out.push(ModelInstance { model: BOLT_MODEL, transform: Affine3A::from_rotation_translation(rot, b.pos), skin: None, flash: 0.0, tint: None });
        }
        for t in &self.tokens {
            if t.taken {
                continue;
            }
            let spin = Quat::from_rotation_y(time * 2.0 + t.age);
            let pop = (t.age * 4.0).min(1.0);
            let hover = 0.1 + (time * 3.0 + t.age).sin() * 0.05;
            out.push(ModelInstance { model: TOKEN_MODEL, transform: Affine3A::from_rotation_translation(spin, t.pos + Vec3::Y * hover) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0, tint: None });
        }
        for k in &self.keys {
            if k.taken {
                continue;
            }
            let spin = Quat::from_rotation_y(time * 1.5 + k.age) * Quat::from_rotation_x(0.4);
            let pop = (k.age * 4.0).min(1.0);
            let hover = 0.25 + (time * 2.5 + k.age).sin() * 0.06;
            out.push(ModelInstance { model: KEY_MODEL, transform: Affine3A::from_rotation_translation(spin, k.pos + Vec3::Y * hover) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0, tint: None });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etasjene_har_sine_slag() {
        assert_eq!(Kind::spawn(0, 0.99).0, Kind::Button);
        assert_eq!(Kind::spawn(1, 0.9), (Kind::Tooltip, 3));
        assert_eq!(Kind::spawn(2, 0.9).0, Kind::Input);
        assert_eq!(Kind::spawn(4, 0.9).0, Kind::Modal);
        assert_eq!(Kind::elite(0), Kind::Button);
        assert_eq!(Kind::elite(4), Kind::Table);
        for k in Kind::ALL {
            let s = k.stats();
            assert!(s.attack_hit < s.attack_len, "{k:?}");
            assert!(s.reach >= s.range, "{k:?}");
        }
    }
}
