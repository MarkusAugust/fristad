//! Turen ned i krypten: det som skjer av seg selv der nede (trappa ned,
//! fallemmene du tråkker på, trykkplatene og pilene, sekken du mistet,
//! lyden bak en falsk vegg, fall utfor kanten) og det som huskes (hvile-
//! stedene du har tent, lappene du har lest, bossene du har slått).

use super::{Crypt, HintKind, TrapKind};
use crate::domain::world::mechanisms::Mechanisms;
use glam::{Quat, Vec3};
use std::collections::{BTreeMap, BTreeSet};

/// En pil fra en felle: hvor den er, farten, alderen, og om den bare
/// skal skremme (første gang treffer den gulvet foran deg).
#[derive(Clone, Copy, Debug)]
pub struct Dart {
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub harmless: bool,
}

/// Det som skjedde nede i ett bilde.
#[derive(Clone, Debug, PartialEq)]
pub enum Happening {
    /// Trappa ned: videre til nivå `k`.
    Down(usize),
    /// Trappa ned finnes, men nivået under gjør ikke det.
    DownBlocked,
    /// Lemmen under deg faller (bryteren og hvor).
    Trapdoor { switch: usize, at: Vec3 },
    /// Trykkplata: pilene skyter ut av muren. Første gang som en advarsel.
    Arrows {
        plate: Vec3,
        from: Vec3,
        first: bool,
    },
    /// Pilene traff deg.
    Hurt(f32),
    /// Du fant igjen sekken du mistet.
    Bag(BTreeMap<String, u32>),
    /// Lyden av rotter bak en falsk vegg.
    Hint(Vec3),
}

#[derive(Clone, Debug)]
pub struct Run {
    /// Hvilestedene du har tent, og det siste (der du våkner når du dør).
    pub rests_lit: Vec<bool>,
    pub last_rest: Option<usize>,
    pub lore_read: Vec<bool>,
    /// Fellene: hvor mange ganger hver har gått av, og hvor lenge til den
    /// er klar igjen; pilene i lufta.
    traps: Vec<(u32, f32)>,
    pub darts: Vec<Dart>,
    /// Bossene du har slått, per nivå (de kommer ikke igjen).
    pub slain: BTreeSet<usize>,
    /// Det du bar da du døde: ligger der til du henter det.
    pub bags: Vec<(Vec3, BTreeMap<String, u32>)>,
    /// Trappa ned virker først når du har gått bort fra den (så du ikke
    /// går rett ned igjen når du kommer opp).
    pub down_armed: bool,
    /// Sekunder til neste lyd fra en hemmelighet.
    hint_t: f32,
    /// Der du sist sto trygt (til fall utfor kanten).
    safe: Vec3,
}

impl Run {
    pub fn new(crypt: &Crypt) -> Self {
        Self {
            rests_lit: vec![false; crypt.rests.len()],
            last_rest: None,
            lore_read: vec![false; crypt.lores.len()],
            traps: vec![(0, 0.0); crypt.traps.len()],
            darts: vec![],
            slain: BTreeSet::new(),
            bags: vec![],
            down_armed: true,
            hint_t: 5.0,
            safe: Vec3::ZERO,
        }
    }

    /// Tenn og hvil ved hvilested `n`. Gir om det var første gang.
    pub fn rest(&mut self, n: usize) -> bool {
        let first = !self.rests_lit[n];
        self.rests_lit[n] = true;
        self.last_rest = Some(n);
        first
    }

    pub fn rest_lit(&self, n: usize) -> bool {
        self.rests_lit.get(n).copied().unwrap_or(false)
    }

    /// Fall utfor kanten: står du trygt på gulvet, huskes stedet; har du
    /// falt mer enn tre meter under det laveste gulvet i nivået, gir den
    /// stedet du skal løftes opp til. (Fallemmer og sjakter ender på et
    /// gulv, så de er ikke fall.)
    pub fn rescue(&mut self, crypt: &Crypt, feet: Vec3, grounded: bool) -> Option<Vec3> {
        let k = crypt.level_at(feet + Vec3::Y * 0.5)?;
        let lowest = crypt.levels[k].min_y;
        if grounded
            && crypt
                .floor_under(feet + Vec3::Y * 0.3)
                .is_some_and(|f| (feet.y - f).abs() < 0.4)
        {
            self.safe = feet;
        } else if feet.y < lowest - 3.0 && self.safe != Vec3::ZERO {
            return Some(self.safe);
        }
        None
    }

    /// Det som skjer av seg selv i nivå `k` når føttene står i `feet`.
    pub fn tick(
        &mut self,
        dt: f32,
        crypt: &Crypt,
        mech: &Mechanisms,
        k: usize,
        feet: Vec3,
    ) -> Vec<Happening> {
        let mut out = vec![];
        let flat = |a: Vec3, b: Vec3| a.with_y(0.0).distance(b.with_y(0.0));
        // Trappa ned: toner over til neste nivå.
        if let Some((p, _)) = crypt.levels[k].down {
            let near = flat(feet, p) < 1.2 && (feet.y - p.y).abs() < 2.5;
            if near && self.down_armed {
                self.down_armed = false;
                if k + 1 < crypt.levels.len() {
                    return vec![Happening::Down(k + 1)];
                }
                out.push(Happening::DownBlocked);
            } else if !near && flat(feet, p) > 2.0 {
                self.down_armed = true;
            }
        }
        // Fallemmene: står du på lemmen, faller den opp under deg.
        for (i, t) in crypt.trapdoors.iter().enumerate() {
            if t.level != k || !t.lid || t.bolted || mech.trapdoors.get(i).is_none_or(|x| *x >= 0.0)
            {
                continue;
            }
            let local = Quat::from_rotation_y(-t.yaw) * (feet - t.pos);
            if local.x.abs() < t.size.x * 0.5 - 0.1
                && local.z.abs() < t.size.y * 0.5 - 0.1
                && local.y.abs() < 0.5
            {
                out.push(Happening::Trapdoor {
                    switch: t.switch,
                    at: t.pos,
                });
            }
        }
        // Trykkplatene: pilene skyter ut av muren. Første gang treffer de
        // gulvet foran deg.
        for (i, t) in crypt.traps.iter().enumerate() {
            self.traps[i].1 -= dt;
            if t.level != k || t.kind != TrapKind::Arrows || self.traps[i].1 > 0.0 {
                continue;
            }
            let d = feet - t.pos;
            if d.x.abs() < t.size.x * 0.5 && d.z.abs() < t.size.y * 0.5 && d.y.abs() < 0.6 {
                let first = self.traps[i].0 == 0;
                self.traps[i] = (self.traps[i].0 + 1, 3.0);
                let side = Vec3::new(-t.dir.z, 0.0, t.dir.x);
                for (j, off) in [-0.45f32, 0.0, 0.45].iter().enumerate() {
                    let from = feet - t.dir * 6.0 + side * *off + Vec3::Y * (1.0 + 0.15 * j as f32);
                    let aim = if first {
                        feet - t.dir * 1.6 + side * *off
                    } else {
                        feet + side * *off * 0.3 + Vec3::Y * 1.0
                    };
                    let vel = (aim - from).normalize_or(t.dir) * 16.0;
                    self.darts.push(Dart {
                        pos: from,
                        vel,
                        age: 0.0,
                        harmless: first,
                    });
                }
                out.push(Happening::Arrows {
                    plate: t.pos,
                    from: feet - t.dir * 5.0 + Vec3::Y,
                    first,
                });
            }
        }
        // Pilene flyr til de treffer deg, gulvet eller en mur.
        let chest = feet + Vec3::Y * 1.0;
        let mut hurt = 0.0;
        for d in &mut self.darts {
            d.pos += d.vel * dt;
            d.age += dt;
            if !d.harmless && d.pos.distance(chest) < 0.6 {
                hurt += 12.0;
                d.age = 9.0;
            } else if crypt.floor_under(d.pos).is_some_and(|f| d.pos.y < f + 0.05) {
                d.age = 9.0;
            }
        }
        self.darts.retain(|d| d.age < 1.2);
        if hurt > 0.0 {
            out.push(Happening::Hurt(hurt));
        }
        // Sekken du mistet: gå bort til den.
        if let Some(i) = self
            .bags
            .iter()
            .position(|(p, _)| flat(*p, feet) < 1.2 && (p.y - feet.y).abs() < 2.0)
        {
            out.push(Happening::Bag(self.bags.remove(i).1));
        }
        // Lyden bak muren: en hemmelighet i nærheten.
        self.hint_t -= dt;
        if self.hint_t <= 0.0 {
            self.hint_t = 9.0;
            if let Some(h) = crypt
                .hints
                .iter()
                .find(|h| h.level == k && h.kind == HintKind::Sound && h.pos.distance(feet) < 12.0)
            {
                out.push(Happening::Hint(h.pos + Vec3::Y * 1.2));
            }
        }
        out
    }
}
