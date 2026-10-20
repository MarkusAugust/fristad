//! Dyrene på engene: sauer og kuer som beiter, rusler rundt og løper noen
//! skritt unna når spilleren kommer for nær. Hvert dyr er fem
//! modellinstanser (kroppen og fire bein fra `tools/models/animals.py`)
//! som spillet poserer per bilde, så beina svinger når de går. Kroppen
//! står på bakken og vippes etter hellingen.

use crate::domain::world::terrain;
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Quat, Vec2, Vec3};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Sheep,
    Cow,
}

impl Kind {
    fn body(self) -> &'static str {
        match self {
            Kind::Sheep => "sheep_body",
            Kind::Cow => "cow_body",
        }
    }
    /// Beinmodellen for bein k (0–1 fram, 2–3 bak): kua har egne fram- og
    /// bakbein (`tools/models/cow.py`), sauen ett bein for alle fire.
    fn leg(self, k: usize) -> &'static str {
        match (self, k < 2) {
            (Kind::Sheep, _) => "sheep_leg",
            (Kind::Cow, true) => "cow_leg_front",
            (Kind::Cow, false) => "cow_leg_hind",
        }
    }
    /// Beinfestene i modellens koordinater (x fram, y opp, z høyre), fra
    /// LEGS og SHEEP_LEGS i animals.py (Blender: x fram, y venstre, z opp).
    fn legs(self) -> [Vec3; 4] {
        match self {
            Kind::Sheep => [Vec3::new(0.3, 0.46, -0.13), Vec3::new(0.3, 0.46, 0.13), Vec3::new(-0.32, 0.46, -0.13), Vec3::new(-0.32, 0.46, 0.13)],
            Kind::Cow => [Vec3::new(0.6, 1.0, -0.24), Vec3::new(0.6, 1.0, 0.24), Vec3::new(-0.62, 1.02, -0.26), Vec3::new(-0.62, 1.02, 0.26)],
        }
    }
    fn walk_speed(self) -> f32 {
        match self {
            Kind::Sheep => 0.7,
            Kind::Cow => 0.6,
        }
    }
    fn run_speed(self) -> f32 {
        match self {
            Kind::Sheep => 4.2,
            Kind::Cow => 4.8,
        }
    }
    /// Hvor nær spilleren må komme før dyret skvetter.
    fn fear_dist(self) -> f32 {
        match self {
            Kind::Sheep => 6.0,
            Kind::Cow => 5.0,
        }
    }
    /// Skritt per meter, så beina svinger i takt med farten.
    fn stride(self) -> f32 {
        match self {
            Kind::Sheep => 3.2,
            Kind::Cow => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    /// Står og beiter så lenge.
    Grazing(f32),
    /// Rusler mot et punkt.
    Walking(Vec2),
    /// Løper fra spilleren så lenge.
    Fleeing(f32),
}

#[derive(Clone, Debug)]
pub struct Animal {
    pub kind: Kind,
    /// Flate koordinater; høyden hentes fra terrenget.
    pos: Vec2,
    yaw: f32,
    speed: f32,
    state: State,
    /// Hvor langt dyret har gått, til beinsvingen.
    phase: f32,
    /// Hvor det hører hjemme, og hvor langt fra det går.
    home: Vec2,
    range: f32,
    /// Litt ulik størrelse.
    scale: f32,
}

/// Flokkene: slag, midtpunkt, radius og antall. Engene er valgt utenfor
/// husene, stiene og granskogen (se `nature::meadow`); kuene har jordet
/// mellom steingjerdene nordøst, sauene enga sørvest for Tokenparken.
pub const HERDS: [(Kind, Vec2, f32, usize); 3] = [
    (Kind::Cow, Vec2::new(78.0, -42.0), 14.0, 5),
    (Kind::Sheep, Vec2::new(-66.0, 54.0), 13.0, 8),
    (Kind::Sheep, Vec2::new(40.0, 62.0), 10.0, 5),
];

pub struct Herd {
    pub animals: Vec<Animal>,
    seed: u32,
}

impl Herd {
    /// Hvor dyrene står (flatt), til lydbildet.
    pub fn positions(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.animals.iter().map(|a| a.pos)
    }

    pub fn new() -> Self {
        let mut seed = 0x9e37_79b9u32;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed & 0xffff) as f32 / 65536.0
        };
        let mut animals = vec![];
        for (kind, home, range, count) in HERDS {
            for _ in 0..count {
                let a = rnd() * std::f32::consts::TAU;
                let r = rnd().sqrt() * range * 0.7;
                let pos = home + Vec2::new(a.cos(), a.sin()) * r;
                animals.push(Animal { kind, pos, yaw: rnd() * std::f32::consts::TAU, speed: 0.0, state: State::Grazing(rnd() * 6.0), phase: rnd() * 10.0, home, range, scale: 0.9 + rnd() * 0.2 });
            }
        }
        Self { animals, seed }
    }

    /// Hvor mange instanser hver modell trenger plass til.
    pub fn reserve() -> Vec<(&'static str, usize)> {
        let mut out: Vec<(&'static str, usize)> = vec![];
        for (kind, _, _, count) in HERDS {
            for (name, n) in [(kind.body(), count), (kind.leg(0), count * 2), (kind.leg(2), count * 2)] {
                if let Some(e) = out.iter_mut().find(|e| e.0 == name) {
                    e.1 += n;
                } else {
                    out.push((name, n));
                }
            }
        }
        out
    }

    fn rnd(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed & 0xffff) as f32 / 65536.0
    }

    /// Ett steg: `player` er spillerens føtter.
    pub fn update(&mut self, dt: f32, player: Vec3) {
        let p2 = Vec2::new(player.x, player.z);
        let n = self.animals.len();
        for i in 0..n {
            let mut a = self.animals[i].clone();
            let away = a.pos - p2;
            let dist = away.length();
            // Skvetter: løper bort fra spilleren, og tar flokken med seg litt.
            if dist < a.kind.fear_dist() && !matches!(a.state, State::Fleeing(_)) {
                a.state = State::Fleeing(1.6 + self.rnd() * 1.2);
                a.yaw = (-away.x).atan2(-away.y);
            }
            let mut target_speed = 0.0;
            match a.state {
                State::Grazing(t) => {
                    let t = t - dt;
                    a.state = if t > 0.0 {
                        State::Grazing(t)
                    } else {
                        // Et nytt mål i nærheten, innenfor hjemmeområdet.
                        let ang = self.rnd() * std::f32::consts::TAU;
                        let r = 2.0 + self.rnd() * 6.0;
                        let mut goal = a.pos + Vec2::new(ang.cos(), ang.sin()) * r;
                        if goal.distance(a.home) > a.range {
                            goal = a.home + (goal - a.home).normalize() * a.range * 0.6;
                        }
                        State::Walking(goal)
                    };
                }
                State::Walking(goal) => {
                    let d = goal - a.pos;
                    if d.length() < 0.4 {
                        a.state = State::Grazing(3.0 + self.rnd() * 9.0);
                    } else {
                        target_speed = a.kind.walk_speed();
                        let want = (-d.x).atan2(-d.y);
                        a.yaw = turn(a.yaw, want, dt * 2.0);
                    }
                }
                State::Fleeing(t) => {
                    let t = t - dt;
                    target_speed = a.kind.run_speed();
                    // Vekk fra spilleren, og hjemover om det bærer for langt ut.
                    let mut dir = if dist > 0.01 { away / dist } else { Vec2::X };
                    if a.pos.distance(a.home) > a.range {
                        dir = (dir + (a.home - a.pos).normalize() * 1.5).normalize();
                    }
                    let want = (-dir.x).atan2(-dir.y);
                    a.yaw = turn(a.yaw, want, dt * 4.0);
                    a.state = if t > 0.0 { State::Fleeing(t) } else { State::Grazing(2.0 + self.rnd() * 4.0) };
                }
            }
            a.speed += (target_speed - a.speed) * (dt * 5.0).min(1.0);
            // Fram er lokal +X: med yaw om Y blir det (cos, −sin) i xz.
            let fwd = Vec2::new(a.yaw.cos(), -a.yaw.sin());
            a.pos += fwd * a.speed * dt;
            a.phase += a.speed * dt * a.kind.stride();
            // Dyrene holder litt avstand til hverandre.
            for j in 0..n {
                if j == i {
                    continue;
                }
                let d = a.pos - self.animals[j].pos;
                let min = if a.kind == Kind::Cow { 2.4 } else { 1.4 };
                let l = d.length();
                if l < min && l > 0.01 {
                    a.pos += d / l * (min - l) * 0.5;
                }
            }
            self.animals[i] = a;
        }
    }

    /// Instansene til rendereren: kroppen på bakken, vippet etter
    /// hellingen, og beina svingende i takt med farten.
    pub fn instances(&self, time: f32) -> Vec<ModelInstance> {
        let mut out = Vec::with_capacity(self.animals.len() * 5);
        for a in &self.animals {
            let g = terrain::height(a.pos.x, a.pos.y);
            let n = terrain::normal(a.pos.x, a.pos.y);
            // Kroppen gynger litt når dyret går.
            let bob = (a.phase * 2.0).sin().abs() * 0.025 * (a.speed / a.kind.walk_speed()).min(2.0);
            let body = Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, n) * Quat::from_rotation_y(a.yaw), Vec3::new(a.pos.x, g + bob, a.pos.y)) * Affine3A::from_scale(Vec3::splat(a.scale));
            out.push(ModelInstance { model: a.kind.body(), transform: body, skin: None, flash: 0.0, tint: None, look: None });
            // Beinsvingen: diagonale par i motfase, større utslag når det løper.
            let amp = if a.speed > a.kind.walk_speed() * 1.5 { 0.55 } else { 0.35 } * (a.speed / a.kind.walk_speed()).min(1.0);
            let idle = (time * 0.7 + a.phase).sin() * 0.02;
            for (k, hip) in a.kind.legs().iter().enumerate() {
                let sign = if k == 0 || k == 3 { 1.0 } else { -1.0 };
                let swing = a.phase.sin() * amp * sign + idle;
                let leg = body * Affine3A::from_translation(*hip) * Affine3A::from_rotation_z(swing);
                out.push(ModelInstance { model: a.kind.leg(k), transform: leg, skin: None, flash: 0.0, tint: None, look: None });
            }
        }
        out
    }
}

impl Default for Herd {
    fn default() -> Self {
        Self::new()
    }
}

/// Snu `yaw` mot `want` med høyst `step` radianer.
fn turn(yaw: f32, want: f32, step: f32) -> f32 {
    let diff = (want - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    yaw + diff.clamp(-step, step)
}
