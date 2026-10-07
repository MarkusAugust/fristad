//! Fiendene i Legacy-kjelleren: korrupte komponenter som har mutert av
//! for mye JavaScript og forretningslogikk. Den første er den korrupte
//! knappen (`mutant_button`): den jager deg på rutenettet, biter når den
//! er nær, og faller fra hverandre i tokens når den dør. Spillet
//! (`game.rs`) sier hvor spilleren er, hva som ble truffet, og tegner
//! instansene gjennom `Frame.instances`.

use crate::domain::city::dungeon::Dungeon;
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Quat, Vec3};

pub const MOB_MODEL: &str = "mutant_button";
pub const TOKEN_MODEL: &str = "token";
const MAX_MOBS: usize = 24;
const MAX_TOKENS: usize = 72;
/// Fienden ser deg innen denne avstanden og jager.
const SIGHT: f32 = 16.0;
const SPEED: f32 = 2.1;
const BITE_RANGE: f32 = 1.5;
const BITE_DAMAGE: f32 = 12.0;
const BITE_COOLDOWN: f32 = 1.3;
pub const MOB_HP: f32 = 100.0;
/// Hvor høyt kroppen svever over gulvet (midten av plata).
const BODY_H: f32 = 0.78;
const BODY_R: f32 = 0.55;

#[derive(Clone, Debug)]
pub struct Mob {
    pub pos: Vec3,
    pub yaw: f32,
    pub hp: f32,
    pub alive: bool,
    /// Sekunder siden den ble truffet (blink og rykk).
    hit: f32,
    bite_cd: f32,
    phase: f32,
    /// Neste cellemidte på veien mot spilleren.
    target: Option<Vec3>,
    repath: f32,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub pos: Vec3,
    pub age: f32,
    pub taken: bool,
}

/// Det som skjedde dette bildet, til spillet.
#[derive(Default, Debug)]
pub struct Outcome {
    pub damage: f32,
    pub tokens: u32,
}

#[derive(Default)]
pub struct Mobs {
    pub mobs: Vec<Mob>,
    pub tokens: Vec<Token>,
    seed: u64,
}

impl Mobs {
    pub fn new(dungeon: &Dungeon) -> Self {
        let mut m = Self { mobs: vec![], tokens: vec![], seed: 0x9e37_79b9_7f4a_7c15 };
        m.reset(dungeon);
        m
    }

    fn rnd(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Ny besetning: en fiende på hvert startsted, opptil MAX_MOBS.
    pub fn reset(&mut self, dungeon: &Dungeon) {
        self.tokens.clear();
        self.mobs.clear();
        let spawns: Vec<Vec3> = dungeon.spawns.iter().copied().take(MAX_MOBS).collect();
        for p in &spawns {
            let phase = self.rnd() * 6.0;
            let yaw = self.rnd() * std::f32::consts::TAU;
            let repath = self.rnd() * 0.5;
            self.mobs.push(Mob { pos: *p, yaw, hp: MOB_HP, alive: true, hit: 9.0, bite_cd: 0.5, phase, target: None, repath });
        }
    }

    /// Hvor mange instanser hver modell trenger plass til.
    pub fn reserve() -> Vec<(&'static str, usize)> {
        vec![(MOB_MODEL, MAX_MOBS), (TOKEN_MODEL, MAX_TOKENS)]
    }

    pub fn alive(&self) -> usize {
        self.mobs.iter().filter(|m| m.alive).count()
    }

    /// Fienden strålen fra øyet treffer innen `max` meter, om noen.
    pub fn aimed(&self, eye: Vec3, dir: Vec3, max: f32) -> Option<usize> {
        // Fienden er en stående kapsel fra beina til over kroppen, så du
        // treffer også når du sikter litt over eller under plata.
        let mut best: Option<(usize, f32)> = None;
        for (i, m) in self.mobs.iter().enumerate() {
            if !m.alive {
                continue;
            }
            let (a, b) = (m.pos + Vec3::Y * 0.2, m.pos + Vec3::Y * (BODY_H + 0.5));
            // Nærmeste avstand mellom strålen (eye + dir·t, t ≥ 0) og segmentet a–b.
            let (d1, d2, r) = (dir, b - a, eye - a);
            let (a11, a12, a22) = (d1.dot(d1), -d1.dot(d2), d2.dot(d2));
            let (b1, b2) = (-d1.dot(r), d2.dot(r));
            let det = a11 * a22 - a12 * a12;
            let (mut t, mut u) = if det.abs() > 1e-6 { ((b1 * a22 - b2 * a12) / det, (a11 * b2 - a12 * b1) / det) } else { (0.0, 0.0) };
            u = u.clamp(0.0, 1.0);
            t = (d1.dot(a + d2 * u - eye)).max(0.0);
            if t > max + BODY_R {
                continue;
            }
            let miss = (eye + d1 * t - (a + d2 * u)).length();
            if miss < BODY_R && best.map_or(true, |(_, bt)| t < bt) {
                best = Some((i, t));
            }
        }
        best.map(|(i, _)| i)
    }

    /// Et slag mot fiende `i` fra `from`: skade, rykk bakover, og tokens
    /// om den dør. Gir `true` om den døde nå.
    pub fn strike(&mut self, i: usize, damage: f32, from: Vec3, dungeon: &Dungeon) -> bool {
        let Some(m) = self.mobs.get_mut(i) else { return false };
        if !m.alive {
            return false;
        }
        m.hp -= damage;
        m.hit = 0.0;
        let push = (m.pos - from).with_y(0.0).normalize_or_zero() * 0.6;
        if dungeon.walkable(m.pos + push, 0.5) {
            m.pos += push;
        }
        if m.hp <= 0.0 {
            m.alive = false;
            let base = m.pos;
            for k in 0..3 {
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
        false
    }

    /// Flytter fiendene og plukker tokens. `player` er føttene.
    pub fn update(&mut self, dt: f32, player: Vec3, in_cellar: bool, dungeon: &Dungeon) -> Outcome {
        let mut out = Outcome::default();
        let n = self.mobs.len();
        for i in 0..n {
            let (before, rest) = self.mobs.split_at_mut(i);
            let (m, after) = rest.split_first_mut().unwrap();
            m.hit += dt;
            m.bite_cd -= dt;
            m.phase += dt;
            if !m.alive {
                continue;
            }
            let to_player = (player - m.pos).with_y(0.0);
            let dist = to_player.length();
            if !in_cellar || dist > SIGHT {
                // Står og vagger.
                m.target = None;
                continue;
            }
            m.repath -= dt;
            if m.target.is_none() || m.repath <= 0.0 {
                m.target = dungeon.next_step(m.pos, player);
                m.repath = 0.5;
            }
            if dist <= BITE_RANGE {
                m.yaw = to_player.x.atan2(to_player.z);
                if m.bite_cd <= 0.0 {
                    m.bite_cd = BITE_COOLDOWN;
                    out.damage += BITE_DAMAGE;
                }
                continue;
            }
            let goal = m.target.unwrap_or(player);
            let dir = (goal - m.pos).with_y(0.0).normalize_or_zero();
            let mut step = dir * SPEED * dt;
            // Hold avstand til de andre.
            for o in before.iter().chain(after.iter()) {
                if !o.alive {
                    continue;
                }
                let d = (m.pos - o.pos).with_y(0.0);
                let l = d.length();
                if l < 1.2 && l > 0.01 {
                    step += d / l * (1.2 - l) * dt * 2.0;
                }
            }
            let next = m.pos + step;
            if dungeon.walkable(next, 0.6) {
                m.pos = next;
            } else if dungeon.walkable(Vec3::new(next.x, m.pos.y, m.pos.z), 0.6) {
                m.pos.x = next.x;
            } else if dungeon.walkable(Vec3::new(m.pos.x, m.pos.y, next.z), 0.6) {
                m.pos.z = next.z;
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
        out
    }

    /// Instansene til rendereren: fiendene som svever, vagger og blinker
    /// når de er truffet, og tokens som roterer.
    pub fn instances(&self, time: f32) -> Vec<ModelInstance> {
        let mut out = Vec::with_capacity(self.mobs.len() + self.tokens.len());
        for m in &self.mobs {
            if !m.alive {
                continue;
            }
            let bob = (m.phase * 2.3).sin() * 0.05;
            let tilt = (m.phase * 1.7).cos() * 0.06;
            // Et rykk og en forstørrelse rett etter et slag.
            let flinch = (0.25 - m.hit).max(0.0) * 4.0;
            let scale = 1.0 + flinch * 0.15;
            let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(m.yaw) * Quat::from_rotation_z(tilt - flinch * 0.3), m.pos + Vec3::Y * bob) * Affine3A::from_scale(Vec3::splat(scale));
            out.push(ModelInstance { model: MOB_MODEL, transform: t });
        }
        for t in &self.tokens {
            if t.taken {
                continue;
            }
            let spin = Quat::from_rotation_y(time * 2.0 + t.age);
            let pop = (t.age * 4.0).min(1.0);
            let hover = 0.1 + (time * 3.0 + t.age).sin() * 0.05;
            out.push(ModelInstance { model: TOKEN_MODEL, transform: Affine3A::from_rotation_translation(spin, t.pos + Vec3::Y * hover) * Affine3A::from_scale(Vec3::splat(pop)) });
        }
        out
    }
}
