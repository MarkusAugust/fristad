//! Fiendene i krypten: korrupte komponenter som har mutert av for mye
//! JavaScript og forretningslogikk. De står fast i hvert nivå (merkene i
//! `crypt_NN.json`), patruljerer, sover i bakhold eller vokter, og jager deg
//! på navigasjonsnettet (A* og trakt, `world/crypt/nav.rs`),
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
use crate::domain::world::crypt::{Ambush, Crypt};
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
/// Klippet `alert` (22 bilder).
const ALERT_LEN: f32 = 22.0 / 24.0;
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
            Kind::Button => "Korrupt button",
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
            Kind::Button => Stats { hp: MOB_HP, speed: 1.4, damage: 12.0, range: 1.7, reach: 2.3, scale: 1.0, sight: 26.0, cooldown: 0.9, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Tooltip => Stats { hp: 38.0, speed: 3.6, damage: 6.0, range: 1.3, reach: 1.7, scale: 0.55, sight: 30.0, cooldown: 0.7, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 1 },
            Kind::Input => Stats { hp: 80.0, speed: 1.9, damage: 10.0, range: 1.8, reach: 2.2, scale: 1.15, sight: 34.0, cooldown: 1.6, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.5, 14.0, 14.0)), armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Modal => Stats { hp: 320.0, speed: 1.3, damage: 30.0, range: 2.2, reach: 3.0, scale: 1.5, sight: 24.0, cooldown: 1.8, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.3, poise: 40.0, tokens: 6 },
            Kind::Table => Stats { hp: 420.0, speed: 2.4, damage: 22.0, range: 2.2, reach: 3.0, scale: 1.4, sight: 30.0, cooldown: 1.1, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 30.0, tokens: 10 },
            Kind::Framework => Stats { hp: 1400.0, speed: 1.6, damage: 34.0, range: 3.0, reach: 4.0, scale: 2.3, sight: 40.0, cooldown: 1.2, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 55.0, tokens: 40 },
        }
    }

    /// Slaget etter navnet i nivåfila (`label` på merket): norsk eller
    /// engelsk, og «knapp i bakhold» er en knapp. Ukjente blir knapper.
    pub fn from_label(label: &str) -> Kind {
        let l = label.to_lowercase();
        let has = |w: &[&str]| w.iter().any(|x| l.contains(x));
        if has(&["rammeverk", "framework"]) {
            Kind::Framework
        } else if has(&["tabell", "table"]) {
            Kind::Table
        } else if has(&["modal"]) {
            Kind::Modal
        } else if has(&["input"]) {
            Kind::Input
        } else if has(&["tooltip"]) {
            Kind::Tooltip
        } else {
            if !has(&["knapp", "button"]) {
                log::warn!("Ukjent fiende «{label}»: blir en knapp");
            }
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
    /// Har nettopp fått øye på deg (klippet `alert`), og snur seg mot deg.
    Alert,
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
    /// Klippene fra mob_motion.py (eldre modeller har dem ikke).
    alert: Option<usize>,
    hit2: Option<usize>,
    stagger: Option<usize>,
    die2: Option<usize>,
    /// Leddene hodet ser med (nakke, hode), og hvilken vei som er opp i
    /// riggens rom.
    look: Option<(usize, usize)>,
    up: Vec3,
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
    /// Hjørnene på veien dit den skal (navigasjonsnettet), og målet veien
    /// ble regnet mot.
    path: Vec<Vec3>,
    goal: Vec3,
    repath: f32,
    /// Der den står vakt, ruta den går (punktene, og hvor den er), og om
    /// den ligger i bakhold til du kommer.
    home: Vec3,
    patrol: Vec<Vec3>,
    patrol_i: usize,
    ambush: Option<Ambush>,
    dormant: bool,
    /// Nivået den hører til.
    pub level: usize,
    /// Hvor mange ganger bossen har brølt (ved 2/3 og 1/3 liv).
    roars: u32,
    /// Meter gått siden forrige skritt (bossens tunge skritt).
    stride: f32,
    /// Farten et slag gir (glir bakover og bremses).
    knock: Vec3,
    /// Hvor mye hodet er dreid mot deg (radianer), og hvor fort kroppen
    /// snur seg (til å lene seg inn i svingen).
    pub look: f32,
    turn: f32,
    last_yaw: f32,
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
    /// Slaget brøt gjennom balansen til en av de tunge (`poise`).
    pub staggered: bool,
}

/// Det som skjedde dette bildet, til spillet.
#[derive(Default, Debug)]
pub struct Outcome {
    /// Slag som landet: (fiende, skade, hvor det kom fra).
    pub bites: Vec<(usize, f32, Vec3)>,
    pub tokens: u32,
    pub keys: u32,
    /// Lydene fiendene lager, og hvor: varsel før slaget (`windup`,
    /// `windup_big`), at de får øye på deg (`notice`), spytt, slag i
    /// bakken, brøl og bossens tunge skritt (`boss_step`).
    pub sounds: Vec<(&'static str, Vec3)>,
    /// Slag i bakken (kameraet rister).
    pub quakes: u32,
    /// Brøl.
    pub roars: u32,
}

#[derive(Default)]
pub struct Mobs {
    pub mobs: Vec<Mob>,
    /// Nivået som er befolket.
    pub level: usize,
    pub tokens: Vec<Token>,
    pub keys: Vec<Token>,
    pub bolts: Vec<Bolt>,
    seed: u64,
    rigs: Vec<Option<Rig>>,
    clips: Vec<Clips>,
}

impl Mobs {
    pub fn new(crypt: &Crypt, rigs: &[(String, Rig)]) -> Self {
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
                    let joint = |n: &str| r.joint(n);
                    Clips {
                        idle: find("idle"),
                        walk: find("walk"),
                        attack: find("attack"),
                        hit: find("hit"),
                        die: find("die"),
                        slam: r.clip("slam"),
                        roar: r.clip("roar"),
                        alert: r.clip("alert"),
                        hit2: r.clip("hit2"),
                        stagger: r.clip("stagger"),
                        die2: r.clip("die2"),
                        look: joint("neck").zip(joint("head")),
                        up: r.root.inverse().transform_vector3(Vec3::Y).normalize_or_zero(),
                    }
                })
                .unwrap_or_default();
            if rig.is_none() {
                log::warn!("Modellen {} har ingen rigg; fienden svever", k.model());
            }
            all_rigs.push(rig);
            clips.push(c);
        }
        let mut m = Self { mobs: vec![], level: 0, tokens: vec![], keys: vec![], bolts: vec![], seed: 0x9e37_79b9_7f4a_7c15, rigs: all_rigs, clips };
        m.enter(crypt, 0);
        m
    }

    fn rnd(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }

    fn spawn(&mut self, kind: Kind, p: Vec3, elite: bool) {
        self.spawn_at(kind, p, None, elite, vec![], None, self.level);
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_at(&mut self, kind: Kind, p: Vec3, yaw: Option<f32>, elite: bool, patrol: Vec<Vec3>, ambush: Option<Ambush>, level: usize) {
        if self.mobs.len() >= MAX_MOBS {
            return;
        }
        let phase = self.rnd() * 6.0;
        let yaw = yaw.unwrap_or(self.rnd() * std::f32::consts::TAU);
        let repath = self.rnd() * 0.5;
        let clips = self.clips[kind as usize];
        let mut anim = self.rigs[kind as usize].as_ref().map(|_| Animator::new(clips.idle, true));
        if let Some(a) = &mut anim {
            a.time = phase;
        }
        let hp = kind.stats().hp * if elite { ELITE_HP } else { 1.0 };
        let dormant = ambush.is_some();
        self.mobs.push(Mob { kind, pos: p, yaw, hp, max_hp: hp, elite, state: State::Idle, hit: 9.0, aware: false, t: 0.0, stun: 0.0, cooldown: 0.5, phase, anim, path: vec![], goal: p, repath, home: p, patrol, patrol_i: 0, ambush, dormant, level, roars: 0, stride: 0.0, knock: Vec3::ZERO, look: 0.0, turn: 0.0, last_yaw: yaw });
    }

    /// Befolk et nivå: fiendene står der merkene sier, med rutene og
    /// bakholdene sine. Det som var i et annet nivå, forsvinner.
    pub fn enter(&mut self, crypt: &Crypt, level: usize) {
        self.level = level;
        self.reset(crypt);
    }

    /// Ny besetning i nivået (etter hvile eller død).
    pub fn reset(&mut self, crypt: &Crypt) {
        self.tokens.clear();
        self.keys.clear();
        self.bolts.clear();
        self.mobs.clear();
        let level = self.level;
        for s in crypt.spawns.iter().filter(|s| s.level == level) {
            let kind = Kind::from_label(&s.label);
            let elite = s.elite && !kind.boss();
            for k in 0..s.n {
                // En flokk står i en ring rundt merket.
                let a = k as f32 * std::f32::consts::TAU / s.n as f32;
                let q = s.pos + Vec3::new(a.cos(), 0.0, a.sin()) * if s.n > 1 { 0.6 + 0.15 * s.n as f32 } else { 0.0 };
                let q = if crypt.walkable(q) { Vec3::new(q.x, crypt.height_at(q), q.z) } else { s.pos };
                self.spawn_at(kind, q, Some(s.yaw), elite, s.patrol.clone(), s.ambush.clone(), level);
            }
        }
    }

    /// Bossene i nivået er slått for godt (de kommer ikke igjen).
    pub fn remove_bosses(&mut self) {
        self.mobs.retain(|m| !m.kind.boss());
    }

    /// Alle har sett deg og ingen sover (til testene som ser om de finner
    /// fram).
    #[cfg(any(test, feature = "testing"))]
    pub fn wake_all(&mut self) {
        for m in &mut self.mobs {
            m.aware = true;
            m.dormant = false;
        }
    }

    /// Om fienden ligger i bakhold og sover ennå (til testene).
    #[cfg(any(test, feature = "testing"))]
    pub fn dormant(&self, i: usize) -> bool {
        self.mobs[i].dormant
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
    pub fn strike(&mut self, i: usize, damage: f32, from: Vec3, crypt: &Crypt) -> Strike {
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
            if elite && self.keys.len() < MAX_KEYS {
                // Der det går an å komme til: på nettet, ikke inni en mur.
                let p = base + Vec3::new(0.0, 0.0, 0.3);
                let p = if crypt.walkable(p) { p } else { base };
                self.keys.push(Token { pos: p, age: 0.0, taken: false });
            }
            for k in 0..n {
                if self.tokens.len() >= MAX_TOKENS {
                    break;
                }
                let a = k as f32 * 2.1 + self.rnd() * 0.8;
                let r = 0.5 + (k / 6) as f32 * 0.4;
                let p = base + Vec3::new(a.cos() * r, 0.0, a.sin() * r);
                let p = if crypt.walkable(p) { p } else { base };
                self.tokens.push(Token { pos: p, age: 0.0, taken: false });
            }
            return Strike { killed: true, armored, staggered: false };
        }
        // De tunge lar seg ikke avbryte av lette slag.
        if damage < stats.poise {
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

    /// Fri sikt fra fienden til spilleren, grovt: punkter langs linja
    /// står på det åpne nettet.
    fn line_of_sight(crypt: &Crypt, from: Vec3, to: Vec3) -> bool {
        let n = ((to - from).length() / 0.75).ceil().max(1.0) as usize;
        (1..n).all(|k| {
            let q = from.lerp(to, k as f32 / n as f32);
            crypt.walkable(Vec3::new(q.x, crypt.height_at(q + Vec3::Y * 0.5), q.z))
        })
    }

    /// Flytter fiendene, lar dem angripe, og plukker tokens. `player` er
    /// føttene, `in_crypt` om du er nede.
    pub fn update(&mut self, dt: f32, player: Vec3, in_crypt: bool, crypt: &Crypt) -> Outcome {
        let mut out = Outcome::default();
        let n = self.mobs.len();
        let mut summons: Vec<Vec3> = vec![];
        let mut bolts: Vec<Bolt> = vec![];
        let player_level = if in_crypt { crypt.level_at(player) } else { None };
        let player_room = if in_crypt { crypt.room_at(player) } else { None };
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
            // Spilleren i et annet nivå (eller langt over eller under) er
            // utenfor syne.
            let same_floor = player_level == Some(m.level) && (player.y - m.pos.y).abs() < 6.0;
            if m.alive() {
                // Dyttet fra et slag glir ut.
                if m.knock.length_squared() > 1e-4 {
                    let step = m.knock * dt;
                    let q = m.pos + step;
                    if crypt.walkable(q) {
                        m.pos = Vec3::new(q.x, crypt.height_at(q), q.z);
                    }
                    m.knock *= (1.0 - dt * 7.0).max(0.0);
                }
                // Hodet dreies mot deg når den har sett deg (opptil ~55°).
                // Oppe i byen (i testene) står den stille og ser likevel etter deg.
                let want = if m.aware && (same_floor || !in_crypt) && dist > 0.3 {
                    let rel = (to_player.x.atan2(to_player.z) - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                    rel.clamp(-0.95, 0.95)
                } else {
                    0.0
                };
                m.look += (want - m.look) * (dt * 6.0).min(1.0);
                // Hvor fort kroppen snur seg, glattet.
                if dt > 0.0 {
                    let dy = (m.yaw - m.last_yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                    m.turn += (dy / dt - m.turn) * (dt * 5.0).min(1.0);
                }
                m.last_yaw = m.yaw;
            } else {
                m.look *= (1.0 - dt * 4.0).max(0.0);
                m.turn = 0.0;
            }
            match m.state {
                State::Gone => continue,
                State::Dying => {
                    if finished(m, DIE_LEN) && m.t > DIE_LEN + CORPSE {
                        m.state = State::Gone;
                    }
                    continue;
                }
                State::Alert => {
                    // Snur seg mot deg mens den varsler.
                    if dist > 0.01 {
                        let want = to_player.x.atan2(to_player.z);
                        let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                        m.yaw += diff * (dt * 5.0).min(1.0);
                    }
                    if finished(m, ALERT_LEN) {
                        m.state = State::Idle;
                        if let Some(a) = m.anim.as_mut() {
                            a.play(clips.idle, true, 0.15);
                        }
                    }
                    continue;
                }
                State::Hit => {
                    // Svimmel (løkke) til lamslåingen er over; ellers klippet ut.
                    let over = if m.stun > 0.0 { m.t >= m.stun } else { finished(m, HIT_LEN) };
                    if over {
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
                                        out.sounds.push(("spit", m.pos));
                                    }
                                }
                                Blow::Slam => {
                                    out.quakes += 1;
                                    out.sounds.push(("slam", m.pos));
                                    if dist <= SLAM_RADIUS {
                                        out.bites.push((i, stats.damage * 1.3, m.pos));
                                    }
                                }
                                Blow::Roar => {
                                    out.roars += 1;
                                    out.sounds.push(("roar", m.pos));
                                    out.sounds.push(("crack", m.pos));
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
            // Hold avstand til de andre (en enkel frastøtning).
            let mut push = Vec3::ZERO;
            for o in before.iter().chain(after.iter()) {
                if !o.alive() || o.level != m.level {
                    continue;
                }
                let d = (m.pos - o.pos).with_y(0.0);
                let l = d.length();
                let keep = 0.6 * (s + o.scale());
                if l < keep && l > 0.01 {
                    push += d / l * (keep - l) * 2.0;
                }
            }
            // Bakhold: sover til du kommer nær nok eller går inn i rommet.
            if m.dormant {
                let wake = same_floor
                    && match &m.ambush {
                        Some(Ambush::Near(radius)) => dist < *radius,
                        Some(Ambush::Room(room)) => player_room == Some((m.level, *room)),
                        None => true,
                    };
                if !wake {
                    continue;
                }
                m.dormant = false;
            }
            // Ser den deg? Når den har sett deg, jager den deg til du er langt
            // unna; ellers må sikten være fri (eller du står helt inntil).
            let sees = same_floor && if m.aware { dist <= stats.sight * 1.6 } else { dist <= stats.sight && (dist < 3.0 || Self::line_of_sight(crypt, m.pos, player)) };
            if !sees {
                if !in_crypt {
                    // Oppe i byen står de stille (og ser fortsatt etter deg).
                    set_state(m, State::Idle, &clips);
                    continue;
                }
                m.aware = false;
                // Går ruta si, eller tilbake dit den sto vakt.
                let goal = if m.patrol.is_empty() { m.home } else { m.patrol[m.patrol_i % m.patrol.len()] };
                if goal.with_y(0.0).distance(m.pos.with_y(0.0)) < 0.6 {
                    if !m.patrol.is_empty() {
                        m.patrol_i = (m.patrol_i + 1) % m.patrol.len();
                    }
                    set_state(m, State::Idle, &clips);
                    continue;
                }
                if !in_crypt {
                    set_state(m, State::Idle, &clips);
                    continue;
                }
                walk(m, goal, stats.speed * 0.55, dt, crypt, push, &stats, &clips, &mut out);
                continue;
            }
            if !m.aware {
                out.sounds.push(("notice", m.pos));
                m.aware = true;
                m.path.clear();
                // Får øye på deg: rykker opp og snur seg mot deg først.
                if let Some(al) = clips.alert {
                    m.state = State::Alert;
                    m.t = 0.0;
                    if let Some(a) = m.anim.as_mut() {
                        a.play(al, false, 0.1);
                    }
                    continue;
                }
            }
            m.aware = true;
            let windup = if kind.boss() || m.elite { "windup_big" } else { "windup" };
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
                    out.sounds.push((windup, m.pos));
                    continue;
                }
            }
            // Rekkevidden måles til kroppen, og bare på samme gulv.
            let level_ok = (player.y - m.pos.y).abs() < 1.6 * s.max(1.0);
            if dist <= stats.range && level_ok && m.cooldown <= 0.0 {
                let blow = if kind.boss() && clips.slam.is_some() && r < 0.35 { Blow::Slam } else { Blow::Melee };
                m.state = State::Attack { blow, landed: false };
                m.t = 0.0;
                if let Some(a) = m.anim.as_mut() {
                    a.play(if blow == Blow::Slam { clips.slam.unwrap_or(clips.attack) } else { clips.attack }, false, 0.08);
                }
                out.sounds.push((windup, m.pos));
                continue;
            }
            // Skyter på avstand når sikten er fri.
            if let Some((near, far, _)) = stats.bolt {
                if dist >= near && dist <= far && m.cooldown <= 0.0 && Self::line_of_sight(crypt, m.pos, player) {
                    m.yaw = to_player.x.atan2(to_player.z);
                    m.state = State::Attack { blow: Blow::Spit, landed: false };
                    m.t = 0.0;
                    if let Some(a) = m.anim.as_mut() {
                        a.play(clips.attack, false, 0.08);
                    }
                    out.sounds.push((windup, m.pos));
                    continue;
                }
            }
            // Nær nok, men angrepet er ikke klart: står og vender seg mot deg.
            if dist <= stats.range && level_ok {
                m.yaw = to_player.x.atan2(to_player.z);
                set_state(m, State::Idle, &clips);
                continue;
            }
            walk(m, player, stats.speed, dt, crypt, push, &stats, &clips, &mut out);
        }
        for p in summons {
            let p = if crypt.walkable(p) { p } else { continue };
            let p = Vec3::new(p.x, crypt.height_at(p), p.z);
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
            } else if crypt.floor_under(b.pos).is_none_or(|f| b.pos.y - f < 0.05) {
                b.age = BOLT_LIFE + 1.0;
            }
        }
        self.bolts.retain(|b| b.age < BOLT_LIFE);
        for t in &mut self.tokens {
            t.age += dt;
            if !t.taken && in_crypt && (t.pos - player).with_y(0.0).length() < 1.0 && (t.pos.y - player.y).abs() < 2.0 {
                t.taken = true;
                out.tokens += 1;
            }
        }
        for k in &mut self.keys {
            k.age += dt;
            if !k.taken && in_crypt && (k.pos - player).with_y(0.0).length() < 1.5 && (k.pos.y - player.y).abs() < 2.0 {
                k.taken = true;
                out.keys += 1;
            }
        }
        out
    }

    /// Instansene til rendereren, og leddmatrisene deres i `skins`.
    /// Uten rigg svever figuren og vagger som før.
    /// Hvilken vei hodet til fiende `i` peker (vannrett, i verden), med
    /// dreiningen mot spilleren. Til testene.
    pub fn head_dir(&self, i: usize) -> Option<Vec3> {
        let m = self.mobs.get(i)?;
        let rig = self.rigs[m.kind as usize].as_ref()?;
        let a = m.anim.as_ref()?;
        let c = &self.clips[m.kind as usize];
        let (neck, head) = c.look?;
        let mut pose = a.pose(rig);
        turn_joints(rig, &mut pose, &[(neck, 0.35 * m.look), (head, 0.65 * m.look)], c.up);
        // Beinet peker langs sin lokale y (fra Blender).
        let d = rig.joint_transform(&pose, head).transform_vector3(Vec3::Y);
        let rot = Quat::from_rotation_y(m.yaw - std::f32::consts::FRAC_PI_2);
        Some((rot * d).with_y(0.0).normalize_or_zero())
    }

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
                    let mut pose = a.pose(rig);
                    let clips = &self.clips[m.kind as usize];
                    // Hodet ser mot deg: nakken og hodet dreies om opp-aksen.
                    if let Some((neck, head)) = clips.look {
                        if m.look.abs() > 1e-3 {
                            turn_joints(rig, &mut pose, &[(neck, 0.35 * m.look), (head, 0.65 * m.look)], clips.up);
                        }
                    }
                    let mats = rig.skin(&pose);
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    // Lener seg inn i svingen (toppen mot sida den svinger mot).
                    let lean = (-m.turn * 0.06).clamp(-0.18, 0.18);
                    // Modellens front er +X; yaw regner front som (sin, 0, cos).
                    let rot = Quat::from_rotation_y(m.yaw - std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(lean);
                    let t = Affine3A::from_rotation_translation(rot, m.pos) * Affine3A::from_scale(Vec3::splat(s * (1.0 + flinch * 0.06)));
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

/// Bytt til en tilstand som står eller går, med klippet sitt.
fn set_state(m: &mut Mob, want: State, clips: &Clips) {
    if m.state != want {
        m.state = want;
        if let Some(a) = m.anim.as_mut() {
            a.play(if want == State::Walk { clips.walk } else { clips.idle }, true, 0.2);
        }
    }
}

/// Ett steg mot `goal` langs veien på navigasjonsnettet: hjørne for hjørne
/// (veien regnes på nytt to ganger i sekundet, eller når målet flytter seg),
/// med frastøtningen fra de andre (`push`). Steget prøves rett fram, langs
/// aksene og skrått til sidene; det som går av nettet, tas ikke. Står den
/// utenfor nettet (dyttet ut), går den inn igjen.
#[allow(clippy::too_many_arguments)]
fn walk(m: &mut Mob, goal: Vec3, speed: f32, dt: f32, crypt: &Crypt, push: Vec3, stats: &Stats, clips: &Clips, out: &mut Outcome) {
    m.repath -= dt;
    if m.repath <= 0.0 || m.path.is_empty() || m.goal.distance(goal) > 1.0 {
        m.path = crypt.path(m.pos, goal).unwrap_or_default();
        m.goal = goal;
        m.repath = 0.5;
    }
    while m.path.len() > 1 && m.path[0].with_y(0.0).distance(m.pos.with_y(0.0)) < 0.3 {
        m.path.remove(0);
    }
    let from = m.pos;
    let on = crypt.walkable(from);
    let next = if on { m.path.first().copied().unwrap_or(goal) } else { crypt.snap(from, 3.0).unwrap_or(goal) };
    let dir = (next - from).with_y(0.0).normalize_or_zero();
    // Står en annen rett i veien (de går mot hverandre), går den til siden
    // i stedet for å bli stående og dytte.
    let mut push = push;
    if push.length_squared() > 1e-6 && push.dot(dir) < 0.0 {
        let side = Vec3::new(-dir.z, 0.0, dir.x);
        let away = if push.dot(side) >= 0.0 { side } else { -side };
        push += away * push.length() * 1.5;
    }
    let step = dir * speed * dt + push * dt * if on { 1.0 } else { 0.0 };
    let turn = |a: f32| Quat::from_rotation_y(a) * step;
    let tries = [step, Vec3::new(step.x, 0.0, 0.0), Vec3::new(0.0, 0.0, step.z), turn(0.9), turn(-0.9), turn(1.6), turn(-1.6)];
    if !on {
        m.pos = from + step;
    } else if let Some(t) = tries.into_iter().find(|t| t.length_squared() > 1e-10 && crypt.walkable(from + *t)) {
        m.pos = from + t;
    }
    m.pos.y = crypt.height_at(m.pos + Vec3::Y * 0.3);
    let s = m.scale();
    // Skrittene: bossens tunge, de andres tassing (ikke tooltipene, som svever).
    if m.kind != Kind::Tooltip {
        m.stride += (m.pos - from).with_y(0.0).length();
        if m.stride > 1.4 * s {
            m.stride = 0.0;
            out.sounds.push((if m.kind.boss() { "boss_step" } else { "mob_step" }, m.pos));
        }
    }
    let moved = (m.pos - from).with_y(0.0).length() / dt.max(1e-4);
    let want_state = if moved > 0.3 { State::Walk } else { State::Idle };
    set_state(m, want_state, clips);
    if let Some(a) = m.anim.as_mut() {
        if want_state == State::Walk {
            // Gangen er laget for slagets fart (mob_motion.py), så føttene
            // står stille på gulvet når farten stemmer.
            a.speed = (moved / stats.speed.max(0.1)).clamp(0.4, 1.6);
        }
    }
    if dir.length_squared() > 0.0 {
        let want = dir.x.atan2(dir.z);
        let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        m.yaw += diff * (dt * 6.0).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slagene_etter_navnet() {
        assert_eq!(Kind::from_label("knapp i bakhold"), Kind::Button);
        assert_eq!(Kind::from_label("tooltip"), Kind::Tooltip);
        assert_eq!(Kind::from_label("Input"), Kind::Input);
        assert_eq!(Kind::from_label("modalen"), Kind::Modal);
        assert_eq!(Kind::from_label("tabell"), Kind::Table);
        assert_eq!(Kind::from_label("Rammeverket"), Kind::Framework);
        for k in Kind::ALL {
            let s = k.stats();
            assert!(s.attack_hit < s.attack_len, "{k:?}");
            assert!(s.reach >= s.range, "{k:?}");
        }
    }
}

/// Dreier ledd om opp-aksen i riggens rom (`up`), hvert med sin vinkel, uten
/// å flytte dem: lokalt blir dreiningen forelderens globale dreining
/// konjugert med opp-dreiningen. Brukes til at hodet ser mot spilleren.
fn turn_joints(rig: &Rig, pose: &mut crate::domain::anim::Pose, joints: &[(usize, f32)], up: Vec3) {
    let globals = rig.globals(pose);
    for &(j, angle) in joints {
        let Some(joint) = rig.joints.get(j) else { continue };
        let parent = joint.parent.map(|p| globals[p].to_scale_rotation_translation().1).unwrap_or(Quat::IDENTITY);
        let q = Quat::from_axis_angle(up, angle);
        if let Some(slot) = pose.0.get_mut(j) {
            slot.1 = (parent.inverse() * q * parent * slot.1).normalize();
        }
    }
}
