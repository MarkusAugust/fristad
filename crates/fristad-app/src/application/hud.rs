//! Det som vises over bildet i kampen, og det som sist ble sendt:
//! helsestolpene over fiendene, skadetallene, infopanelet om fienden du
//! sikter på og stolpene nederst. Får fiendene, helten, kampen og kameraet
//! som argument.

use super::camera::Camera;
use super::trade::inventory;
use crate::domain::combat::Combat;
use crate::domain::hero::Hero;
use crate::domain::mobs::Mobs;
use crate::ports::driving::{Event, FoeView, Mark};
use glam::Vec3;

#[derive(Default)]
pub(super) struct Hud {
    /// Skadetall som stiger: (sted, skade, alder, løpenummer).
    popups: Vec<(Vec3, f32, f32, u32)>,
    seq: u32,
    marks_shown: bool,
    last: Option<(u32, u32, u32, bool, u32, Option<(&'static str, u32)>, u32, u8)>,
    /// Fienden i infopanelet, sekundene siden du sist siktet på den, og det
    /// som sist ble sendt.
    foe: Option<usize>,
    foe_t: f32,
    foe_sent: Option<FoeView>,
}

/// Så lenge står infopanelet etter at siktet har gått av fienden (så det
/// ikke blinker når den hopper eller du snur deg i kampen).
const FOE_LINGER: f32 = 0.6;

impl Hud {
    pub fn clear(&mut self) {
        self.popups.clear();
    }

    /// Et skadetall som stiger fra `pos`.
    pub fn popup(&mut self, pos: Vec3, damage: f32) {
        self.seq = self.seq.wrapping_add(1);
        self.popups.push((pos, damage, 0.0, self.seq));
    }

    /// Infopanelet om fienden du sikter på (`aimed`): navnet, hva den er og
    /// livet, sendt når noe endres, og `None` litt etter at siktet er borte
    /// eller fienden er død.
    pub fn update_foe(&mut self, aimed: Option<usize>, dt: f32, mobs: &Mobs, events: &mut Vec<Event>) {
        if aimed.is_some() {
            self.foe = aimed;
            self.foe_t = 0.0;
        } else {
            self.foe_t += dt;
        }
        let view = self.foe.filter(|_| self.foe_t < FOE_LINGER).and_then(|i| mobs.mobs.get(i)).filter(|m| m.alive()).map(|m| {
            let rank = if m.boss || m.kind.boss() { "Boss" } else if m.elite { "Elite" } else { "" };
            let (hp, max_hp) = (m.hp.max(0.0).ceil() as u32, m.max_hp.ceil() as u32);
            FoeView { name: m.kind.label().into(), info: m.kind.info().into(), rank: rank.into(), hp, max_hp, ranged: m.kind.stats().bolt.is_some() }
        });
        if view.is_none() {
            self.foe = None;
        }
        if view != self.foe_sent {
            self.foe_sent = view.clone();
            events.push(Event::Foe(view));
        }
    }

    /// Merkene over fiendene, stolpene nederst og lageret, sendt når de
    /// har endret seg.
    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, dt: f32, in_crypt: bool, feet: Vec3, mobs: &Mobs, hero: &mut Hero, combat: &Combat, cam: &Camera, events: &mut Vec<Event>) {
        let marks = self.marks(dt, mobs, cam);
        if !marks.is_empty() || self.marks_shown {
            self.marks_shown = !marks.is_empty();
            events.push(Event::Marks(marks));
        }
        let boss = if in_crypt { mobs.boss().map(|(n, f)| (n, (f * 100.0).ceil() as u32)) } else { None };
        let fight = if in_crypt { mobs.fight(feet) } else { 0 };
        let h = &*hero;
        let hud = (h.hp.round().max(0.0) as u32, combat.stamina.round().max(0.0) as u32, h.coins, in_crypt, h.keys, boss, (h.lamp.fraction() * 100.0).ceil() as u32, fight);
        if self.last != Some(hud) {
            if self.last.as_ref().map(|h| h.2) != Some(hero.coins) {
                hero.dirty = true;
            }
            self.last = Some(hud);
            events.push(Event::Hud { hp: hud.0, stamina: hud.1, tokens: hud.2, cellar: hud.3, keys: hud.4, boss: hud.5.map(|(n, f)| (n.to_string(), f)), light: hud.6, fight: hud.7 });
        }
        if hero.dirty {
            hero.dirty = false;
            events.push(Event::Inventory(inventory(hero, combat.max_stamina)));
        }
    }

    /// Helsestolper over fiender som er truffet, og skadetall som stiger.
    fn marks(&mut self, dt: f32, mobs: &Mobs, cam: &Camera) -> Vec<Mark> {
        let mut out = vec![];
        for (i, m) in mobs.mobs.iter().enumerate() {
            if !m.alive() || m.hit > 3.0 || m.boss {
                continue;
            }
            if let Some((x, y)) = cam.in_view(m.pos + Vec3::Y * 1.65 * m.scale()) {
                out.push(Mark { id: i as u32, x, y, kind: "bar", value: (m.hp / m.max_hp).clamp(0.0, 1.0), text: String::new() });
            }
        }
        for p in &mut self.popups {
            p.2 += dt;
        }
        self.popups.retain(|p| p.2 < 0.9);
        for (pos, dmg, age, id) in &self.popups {
            if let Some((x, y)) = cam.in_view(*pos + Vec3::Y * (0.3 + age * 0.6)) {
                out.push(Mark { id: 100_000 + id, x, y, kind: "text", value: 1.0 - age / 0.9, text: format!("{}", dmg.round() as i32) });
            }
        }
        out
    }
}
