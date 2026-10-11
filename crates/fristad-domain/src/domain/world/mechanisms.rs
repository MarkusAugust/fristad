//! Det som beveger seg når du bruker det: portgitrene i krypten, snarveidørene,
//! spakene, de falske veggene, fallemmene, kistelokkene og brønnene i byen.
//! Hver har en tilstand (lukket, i bevegelse, åpen) og en pose til bildet;
//! bryterne i krypten (`Crypt.open`) åpnes her, med alt de styrer.

use super::crypt::{self, Crypt};
use super::{at, town};
use glam::{Affine3A, Vec3};

/// Sekunder bøtta bruker før du drikker, og før brønnen er ledig igjen.
pub const WELL_UP: f32 = 3.8;
const WELL_DONE: f32 = 4.2;

/// Det som flyttet seg i et bilde, og som derfor må få kollideren sin flyttet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    Shortcut(usize),
    Secret(usize),
    Gate(usize),
    /// Lemmen har akkurat falt: det du sto på, er borte.
    Trapdoor(usize),
}

#[derive(Clone, Debug, Default)]
pub struct Mechanisms {
    /// Portene i krypten: 0 lukket, 1 oppe, og om de er låst opp.
    pub gates: Vec<(f32, bool)>,
    /// Snarveidørene: hvor langt de er åpnet (0..1), −1 når de er stengt.
    pub shortcuts: Vec<f32>,
    /// Spakene: hvor langt de er trukket (0..1), −1 før de er rørt.
    pub levers: Vec<f32>,
    /// De falske veggene: hvor langt de har sunket (0..1), −1 når de står.
    pub secrets: Vec<f32>,
    /// Fallemmene: hvor langt lemmen har falt (0..1), −1 når den er lukket.
    pub trapdoors: Vec<f32>,
    /// Kistene som er åpnet, og hvor langt lokket er kommet (0..1).
    pub chests: Vec<bool>,
    pub chest_open: Vec<f32>,
    /// Hvor langt hver brønn er kommet i å heise opp vann (sekunder), eller
    /// negativ når ingen bruker den.
    pub wells: Vec<f32>,
}

/// Fram mot 1 med farten `1 / secs`, bare når den er i gang (≥ 0).
fn run(v: &mut f32, dt: f32, secs: f32) -> bool {
    if *v >= 0.0 && *v < 1.0 {
        *v = (*v + dt / secs).min(1.0);
        return true;
    }
    false
}

impl Mechanisms {
    pub fn new(crypt: &Crypt, wells: usize) -> Self {
        Self {
            gates: vec![(0.0, false); crypt.gates.len()],
            shortcuts: vec![-1.0; crypt.shortcuts.len()],
            levers: vec![-1.0; crypt.levers.len()],
            secrets: vec![-1.0; crypt.secrets.len()],
            trapdoors: vec![-1.0; crypt.trapdoors.len()],
            chests: vec![false; crypt.chests.len()],
            chest_open: vec![0.0; crypt.chests.len()],
            wells: vec![-1.0; wells],
        }
    }

    pub fn shortcut_open(&self, n: usize) -> bool {
        self.shortcuts.get(n).is_some_and(|o| *o >= 0.0)
    }
    pub fn lever_pulled(&self, n: usize) -> bool {
        self.levers.get(n).is_some_and(|l| *l >= 0.0)
    }
    pub fn secret_open(&self, n: usize) -> bool {
        self.secrets.get(n).is_some_and(|s| *s >= 0.0)
    }
    pub fn gate_open(&self, n: usize) -> bool {
        self.gates.get(n).is_some_and(|g| g.1)
    }
    pub fn well_busy(&self, n: usize) -> bool {
        self.wells.get(n).is_some_and(|w| *w >= 0.0)
    }

    /// Åpne en bryter (port, dør, falsk vegg, lem) med det den styrer.
    pub fn open_switch(&mut self, crypt: &mut Crypt, sw: usize) {
        if let Some(o) = crypt.open.get_mut(sw) {
            *o = true;
        }
        for (g, gate) in crypt.gates.iter().enumerate() {
            if gate.switch == sw {
                self.gates[g].1 = true;
            }
        }
        let start = |v: &mut f32| {
            if *v < 0.0 {
                *v = 0.0;
            }
        };
        for (i, _) in crypt.shortcuts.iter().enumerate().filter(|(_, s)| s.switch == sw) {
            start(&mut self.shortcuts[i]);
        }
        for (i, _) in crypt.secrets.iter().enumerate().filter(|(_, s)| s.switch == sw) {
            start(&mut self.secrets[i]);
        }
        for (i, _) in crypt.trapdoors.iter().enumerate().filter(|(_, t)| t.switch == sw) {
            start(&mut self.trapdoors[i]);
        }
    }

    /// Kiste `n` åpnes. Gir om den var lukket.
    pub fn open_chest(&mut self, n: usize) -> bool {
        match self.chests.get_mut(n) {
            Some(c) if !*c => {
                *c = true;
                true
            }
            _ => false,
        }
    }

    /// Spak `n` trekkes. Gir bryterne den åpner, om den ikke var trukket.
    pub fn pull_lever(&mut self, crypt: &Crypt, n: usize) -> Option<Vec<usize>> {
        let l = self.levers.get_mut(n).filter(|l| **l < 0.0)?;
        *l = 0.0;
        Some(crypt.levers[n].opens.clone())
    }

    /// Brønn `n` tas i bruk. Gir om den var ledig.
    pub fn draw_well(&mut self, n: usize) -> bool {
        match self.wells.get_mut(n) {
            Some(w) if *w < 0.0 => {
                *w = 0.0;
                true
            }
            _ => false,
        }
    }

    /// Alt som er i gang, går videre. Gir det som flyttet seg, og hvor
    /// mange bøtter som kom opp (du drikker).
    pub fn tick(&mut self, dt: f32) -> (Vec<Moved>, u32) {
        let mut moved = vec![];
        for (open, opened) in self.chest_open.iter_mut().zip(&self.chests) {
            if *opened && *open < 1.0 {
                *open = (*open + dt / 0.9).min(1.0);
            }
        }
        for (n, s) in self.shortcuts.iter_mut().enumerate() {
            if run(s, dt, 1.2) {
                moved.push(Moved::Shortcut(n));
            }
        }
        for l in &mut self.levers {
            run(l, dt, 0.5);
        }
        for (n, s) in self.secrets.iter_mut().enumerate() {
            if run(s, dt, 2.0) {
                moved.push(Moved::Secret(n));
            }
        }
        for (n, t) in self.trapdoors.iter_mut().enumerate() {
            if *t == 0.0 {
                moved.push(Moved::Trapdoor(n));
            }
            run(t, dt, 0.35);
        }
        for (n, (open, unlocked)) in self.gates.iter_mut().enumerate() {
            if *unlocked && *open < 1.0 {
                *open = (*open + dt / 1.6).min(1.0);
                moved.push(Moved::Gate(n));
            }
        }
        let mut drank = 0;
        for w in &mut self.wells {
            if *w < 0.0 {
                continue;
            }
            let before = *w;
            *w += dt;
            if before < WELL_UP && *w >= WELL_UP {
                drank += 1;
            }
            if *w >= WELL_DONE {
                *w = -1.0;
            }
        }
        (moved, drank)
    }

    /// Om du (føttene eller kroppen i `pos`) står på sida av snarveidøra
    /// der slåen sitter.
    pub fn shortcut_inside(crypt: &Crypt, n: usize, pos: Vec3) -> bool {
        let Some(sc) = crypt.shortcuts.get(n) else { return false };
        (pos - sc.pos).with_y(0.0).dot(sc.latch) > 0.0
    }

    /// Snarveidøra dreid om hengslet, bort fra slåen, etter hvor langt den
    /// er åpnet.
    pub fn shortcut_pose(&self, crypt: &Crypt, n: usize) -> Option<Affine3A> {
        let sc = crypt.shortcuts.get(n)?;
        let open = self.shortcuts.get(n).copied().unwrap_or(-1.0).max(0.0);
        let ease = 1.0 - (1.0 - open).powi(3);
        let front = Vec3::new(sc.yaw.sin(), 0.0, sc.yaw.cos());
        let away = if sc.latch.dot(front) > 0.0 { 1.0 } else { -1.0 };
        Some(at(sc.pos, sc.yaw) * Affine3A::from_translation(Vec3::new(-sc.width * 0.5, 0.0, 0.0)) * Affine3A::from_rotation_y(away * ease * 1.75))
    }

    /// Kollideren til snarveidøra (midt i bladet).
    pub fn shortcut_collider(&self, crypt: &Crypt, n: usize) -> Option<Affine3A> {
        let w = crypt.shortcuts.get(n)?.width;
        Some(self.shortcut_pose(crypt, n)? * Affine3A::from_translation(Vec3::new(w * 0.5, crypt::SHORTCUT_H * 0.5, 0.0)))
    }

    pub fn shortcut_poses(&self, crypt: &Crypt) -> Vec<Affine3A> {
        (0..self.shortcuts.len()).filter_map(|n| self.shortcut_pose(crypt, n)).collect()
    }

    /// Portgitteret i målets pose, heist så langt det er kommet.
    pub fn gate_lifted(&self, n: usize, pose: Affine3A) -> Affine3A {
        pose * Affine3A::from_translation(Vec3::Y * gate_lift(self.gates.get(n).map(|g| g.0).unwrap_or(0.0)))
    }

    /// Portgitteret der det står nå (til bildet), skalert til bredden.
    pub fn gate_pose(&self, crypt: &Crypt, n: usize, pose: Affine3A) -> Affine3A {
        let w = crypt.gates.get(n).map(|g| g.width).unwrap_or(crypt::GATE_W);
        self.gate_lifted(n, pose) * Affine3A::from_scale(Vec3::new(w / crypt::GATE_W, 1.0, 1.0))
    }

    /// Hvor langt den falske veggen `n` har sunket (m).
    pub fn secret_sink(&self, crypt: &Crypt, n: usize) -> f32 {
        let h = crypt.secrets.get(n).map(|s| s.height).unwrap_or(0.0);
        (h + 0.1) * self.secrets.get(n).copied().unwrap_or(-1.0).max(0.0)
    }

    /// Armene på spakene: oppe til de trekkes, så ned.
    pub fn lever_poses(&self, crypt: &Crypt) -> Vec<Affine3A> {
        crypt
            .levers
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let t = self.levers.get(i).copied().unwrap_or(-1.0).max(0.0);
                let ease = t * t * (3.0 - 2.0 * t);
                at(l.pos, l.yaw) * Affine3A::from_translation(crypt::LEVER_PIVOT) * Affine3A::from_rotation_x(-0.6 + 1.2 * ease)
            })
            .collect()
    }

    /// De falske veggene, sunket så langt de er kommet (og skjelver mens
    /// de synker).
    pub fn secret_poses(&self, crypt: &Crypt, time: f32) -> Vec<Affine3A> {
        crypt
            .secrets
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let t = self.secrets.get(i).copied().unwrap_or(-1.0).max(0.0);
                let shake = if t > 0.0 && t < 1.0 { (time * 40.0).sin() * 0.01 } else { 0.0 };
                at(s.pos, s.yaw) * Affine3A::from_translation(Vec3::new(shake, -(s.height + 0.1) * t * t, 0.0))
            })
            .collect()
    }

    /// Lemmene: lukket, eller falt ned om hengslet.
    pub fn trapdoor_poses(&self, crypt: &Crypt) -> Vec<Affine3A> {
        crypt
            .trapdoors
            .iter()
            .enumerate()
            .filter(|(_, t)| t.lid)
            .map(|(i, t)| {
                let x = self.trapdoors.get(i).copied().unwrap_or(-1.0).max(0.0);
                at(t.pos, t.yaw) * Affine3A::from_translation(Vec3::new(0.0, 0.0, -t.size.y * 0.5)) * Affine3A::from_rotation_x(x * 1.5)
            })
            .collect()
    }

    /// Lokket på kiste `n` i kistas pose: dreid opp om hengslet, med en
    /// liten sprett når det slår mot stoppen.
    pub fn chest_lid(&self, n: usize, pose: Affine3A) -> Affine3A {
        let x = self.chest_open.get(n).copied().unwrap_or(0.0);
        let k = 1.0 + 2.2 * (x - 1.0).powi(3) + 1.2 * (x - 1.0).powi(2);
        let angle = 1.85 * k.clamp(0.0, 1.08);
        pose * Affine3A::from_translation(crypt::CHEST_HINGE) * Affine3A::from_rotation_x(-angle)
    }

    /// Vinda, tauet og bøtta til brønn `n` i brønnens pose.
    pub fn well_parts(&self, n: usize, pose: Affine3A) -> [(&'static str, Affine3A); 3] {
        let h = well_height(self.wells.get(n).copied().unwrap_or(-1.0));
        let [winch, rope, bucket] = town::WELL_PARTS;
        let turn = (town::WELL_REST - h) / town::WELL_DRUM;
        let top = Vec3::new(0.0, town::WELL_AXLE, town::WELL_ROPE_Z);
        [
            (winch, pose * Affine3A::from_translation(Vec3::Y * town::WELL_AXLE) * Affine3A::from_rotation_x(turn)),
            (rope, pose * Affine3A::from_translation(top) * Affine3A::from_scale(Vec3::new(1.0, town::WELL_AXLE - h, 1.0))),
            (bucket, pose * Affine3A::from_translation(Vec3::new(0.0, h, town::WELL_ROPE_Z))),
        ]
    }
}

/// Høyden på bøyla over brønnkanten etter `t` sekunder: slippes ned i
/// vannet, ligger litt, og sveives opp i rykk.
fn well_height(t: f32) -> f32 {
    use town::{WELL_DEPTH as DOWN, WELL_REST as UP};
    if t < 0.0 || t >= WELL_UP {
        UP
    } else if t < 1.0 {
        UP + (DOWN - UP) * t * t
    } else if t < 1.4 {
        DOWN + ((t - 1.0) * 14.0).sin() * 0.02
    } else {
        let s = (t - 1.4) / (WELL_UP - 1.4);
        DOWN + (UP - DOWN) * s + (s * 40.0).sin() * 0.01 * (1.0 - s)
    }
}

/// Rykkete heving, som en sveiv.
fn gate_lift(t: f32) -> f32 {
    t * crypt::GATE_LIFT + (t * 24.0).sin() * 0.02 * (1.0 - t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broennen_gir_vann_en_gang_og_blir_ledig() {
        let mut m = Mechanisms { wells: vec![-1.0], ..Default::default() };
        assert!(m.draw_well(0));
        assert!(!m.draw_well(0), "opptatt");
        let drank: u32 = (0..500).map(|_| m.tick(0.01).1).sum();
        assert_eq!(drank, 1);
        assert!(!m.well_busy(0));
    }

    #[test]
    fn boetta_slippes_ned_og_heises_opp() {
        assert_eq!(well_height(-1.0), town::WELL_REST);
        assert!((well_height(1.2) - town::WELL_DEPTH).abs() < 0.05);
        assert_eq!(well_height(WELL_UP), town::WELL_REST);
    }
}
