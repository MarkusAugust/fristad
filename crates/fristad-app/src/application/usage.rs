//! F på det du sikter på, og det som flytter deg mellom byen og nivåene:
//! hjem gjennom virvelen, ned og opp trappene, død og oppvåkning.

use super::session::{sfx, Session};
use crate::domain::craft;
use crate::domain::mobs::Kind as MobKind;
use crate::domain::weather::Daylight;
use crate::domain::world::crypt;
use crate::domain::world::mechanisms::Mechanisms;
use crate::domain::world::Use;
use crate::ports::driving::Event;
use glam::Vec3;

impl Session {
    /// F på mål nummer `i`.
    pub(super) fn use_target(&mut self, i: usize, events: &mut Vec<Event>) {
        // Kister, porter, spaker og snarveier endrer merkene på kartet.
        self.map.dirty = true;
        match self.targets[i].target.what {
            Use::Sleep => {
                // Skjermen går i svart, og du våkner når døgnet har snudd.
                self.hero.restore(&mut self.combat);
                let d = if self.sky.daylight == Daylight::Day {
                    Daylight::Night
                } else {
                    Daylight::Day
                };
                self.sky.set_daylight(d, true);
                events.extend([Event::Fade, Event::Daylight(d), self.bell()]);
                events.push(Event::Toast(
                    if d == Daylight::Night {
                        "Du sov til kvelden."
                    } else {
                        "Du sov til morgenen."
                    }
                    .into(),
                ));
            }
            Use::Gallery(n) => {
                if let Some(c) = craft::GALLERY
                    .get(n)
                    .filter(|c| self.hero.found.contains(**c))
                {
                    events.push(Event::OpenDoc(c.to_string()));
                }
            }
            Use::Talk(n) => {
                if self.npcs.talk(n).is_some() {
                    let at = self
                        .npcs
                        .npcs
                        .get(n)
                        .map(|p| p.pos + Vec3::Y * 1.6)
                        .unwrap_or(self.eye());
                    events.push(self.sound_at("talk", at, 1.0));
                }
            }
            Use::Chest(n) => {
                if !self.mech.open_chest(n) {
                    return;
                }
                let chest = self.crypt.chests.get(n).cloned();
                let level = chest.as_ref().map(|c| c.level).unwrap_or(0);
                let list = craft::LOOT[craft::theme_of(level)];
                self.loot_seed ^= self.loot_seed << 13;
                self.loot_seed ^= self.loot_seed >> 7;
                self.loot_seed ^= self.loot_seed << 17;
                // Den sjeldne komponenten i kista, eller en fra temaets liste.
                let pick = chest.and_then(|c| c.loot).unwrap_or_else(|| {
                    list[(self.loot_seed >> 33) as usize % list.len()].to_string()
                });
                self.hero.carry(&pick, 1);
                events.push(self.sound_at("chest", self.target_pos(i), 1.0));
                events.extend([sfx("loot"), Event::Loot(Some(pick))]);
            }
            Use::Shortcut(n) => {
                if self.mech.shortcut_open(n) {
                    return;
                }
                let at = self.target_pos(i);
                if Mechanisms::shortcut_inside(&self.crypt, n, self.body.pos) {
                    self.mech.shortcuts[n] = 0.0;
                    if let Some(sw) = self.crypt.shortcuts.get(n).map(|s| s.switch) {
                        self.open_switch(sw);
                    }
                    events.push(self.sound_at("bolt", at, 1.0));
                    events.push(self.sound_at("door", at, 0.9));
                    events.push(Event::Toast(
                        "Du skyver slåen fra. Snarveien er åpen.".into(),
                    ));
                } else {
                    events.push(self.sound_at("locked", at, 0.8));
                    events.push(Event::Toast("Døra er boltet fra den andre siden.".into()));
                }
            }
            Use::Gate(n) => {
                if self.mech.gate_open(n) {
                    return;
                }
                let at = self.target_pos(i);
                let key = self.crypt.gates.get(n).is_some_and(|g| g.key);
                if !key {
                    events.push(self.sound_at("locked", at, 1.0));
                    events.push(Event::Toast(
                        "Gitteret sitter fast. En spak et sted heiser det.".into(),
                    ));
                } else if self.hero.keys > 0 {
                    self.hero.keys -= 1;
                    if let Some(sw) = self.crypt.gates.get(n).map(|g| g.switch) {
                        self.open_switch(sw);
                    }
                    events.push(sfx("key"));
                    events.push(self.sound_at("gate", at, 1.0));
                    events.push(Event::Toast("Nøkkelen passer. Porten sveives opp.".into()));
                } else {
                    events.push(self.sound_at("locked", at, 1.0));
                    events.push(Event::Toast(
                        "Porten er låst. Elitefienden i nivået bærer nøkkelen.".into(),
                    ));
                }
            }
            Use::Lever(n) => {
                let Some(opens) = self.mech.pull_lever(&self.crypt, n) else {
                    return;
                };
                events.push(self.sound_at("gate", self.target_pos(i), 1.0));
                for sw in opens {
                    self.open_switch(sw);
                    // Gitteret som går opp, høres der det står.
                    if let Some(g) = self.crypt.gates.iter().find(|g| g.switch == sw) {
                        events.push(self.sound_at("gate", g.pos + Vec3::Y * 1.5, 1.2));
                    }
                }
                events.push(Event::Toast(
                    "Spaken går ned med et tungt klonk. Et sted heises et gitter.".into(),
                ));
            }
            Use::Secret(n) => {
                if self.mech.secrets.get(n).is_none_or(|s| *s >= 0.0) {
                    return;
                }
                if let Some(sw) = self.crypt.secrets.get(n).map(|s| s.switch) {
                    self.open_switch(sw);
                }
                events.push(self.sound_at("crack", self.target_pos(i), 1.2));
                events.push(self.sound_at("door_close", self.target_pos(i), 0.8));
                events.push(Event::Toast(
                    "Steinene gir etter og synker i gulvet. Det er et rom bak muren!".into(),
                ));
            }
            Use::Ladder(n) => {
                let Some(l) = self.crypt.ladders.get(n).cloned() else {
                    return;
                };
                // Lemmen øverst går opp når du klatrer gjennom den.
                for t in 0..self.crypt.trapdoors.len() {
                    if self.crypt.trapdoors[t].pos.distance(l.top) < 2.5
                        && self.mech.trapdoors[t] < 0.0
                    {
                        let sw = self.crypt.trapdoors[t].switch;
                        self.open_switch(sw);
                    }
                }
                self.teleport(l.top, l.yaw + std::f32::consts::PI);
                events.extend([Event::Fade, sfx("rope")]);
            }
            Use::Rest(n) => {
                let first = self.run.rest(n);
                self.hero.restore(&mut self.combat);
                self.mobs.reset(&self.crypt);
                if self.run.slain.contains(&self.mobs.level) {
                    self.mobs.remove_bosses();
                }
                events.extend([Event::Fade, sfx("heal")]);
                events.push(Event::Toast(if first { "Du tenner lysene på alteret og hviler. Dør du, våkner du her. Fiendene i nivået er kommet tilbake." } else { "Du hviler ved alteret. Fiendene i nivået er kommet tilbake." }.into()));
            }
            Use::Lore(n) => {
                let Some(l) = self.crypt.lores.get(n).cloned() else {
                    return;
                };
                self.run.lore_read[n] = true;
                events.push(sfx("page"));
                events.push(Event::Toast(if l.text.is_empty() {
                    l.title
                } else {
                    format!("{}: «{}»", l.title, l.text)
                }));
            }
            Use::Bonus(n) => {
                let Some(b) = self.crypt.bonuses.get(n).cloned() else {
                    return;
                };
                if let Some(note) =
                    self.hero
                        .take_bonus(&b.key, b.kind == crypt::BonusKind::Life, &mut self.combat)
                {
                    self.say([note], events);
                    events.push(sfx("heal"));
                }
            }
            Use::Well(n) => {
                if self.mech.draw_well(n) {
                    events.push(self.sound_at("well", self.target_pos(i), 1.0));
                }
            }
            Use::Descend => {
                let Some((to, yaw)) = self.crypt.start() else {
                    events.push(Event::Toast(
                        "Døra er stengt. Krypten er ikke ferdig ennå.".into(),
                    ));
                    return;
                };
                self.portal.enter(0, to, yaw);
                self.teleport(to, yaw);
                self.enter_level(0);
                events.extend([Event::Fade, sfx("crypt_door")]);
                let name = self.crypt.levels[0].name.clone();
                events.push(Event::Toast(format!("Nivå 1: {name}. Venstre mus hugger (hold for tungt), høyre mus blokker, Shift og retning er unnamanøver.")));
            }
            Use::Portal => {
                let Some((to, yaw, level)) = self.portal.dest.take() else {
                    return;
                };
                self.warp(events);
                self.teleport(to, yaw);
                self.enter_level(level);
                events.push(Event::Toast(format!(
                    "Gjennom virvelen og ned i nivå {}. Portalen lukker seg bak deg.",
                    level + 1
                )));
            }
            Use::Ascend(k) => {
                events.extend([Event::Fade, sfx("door")]);
                if k == 0 {
                    let (to, yaw) = self.crypt_exit;
                    self.teleport(to, yaw);
                    self.bank(events);
                } else {
                    // Opp til trappa ned i nivået over, et par skritt fra den.
                    let up = self.crypt.levels[k - 1].clone();
                    let (p, yaw) = up.down.unwrap_or(up.start);
                    let front = Vec3::new(yaw.sin(), 0.0, yaw.cos());
                    let to = self.crypt.snap(p + front * 2.2, 3.0).unwrap_or(p);
                    self.teleport(to, yaw);
                    self.run.down_armed = false;
                    self.enter_level(k - 1);
                    events.push(Event::Toast(format!("Nivå {}: {}", up.n, up.name)));
                }
            }
        }
    }

    /// H (eller «Tilbake til sørporten»): fra kjelleren gjennom en virvel
    /// til sørporten, og portalen der våkner og husker hvor du var. Oppe i
    /// byen bare tilbake til porten.
    pub(super) fn go_home(&mut self, events: &mut Vec<Event>) {
        if self.in_crypt() && self.hero.hp > 0.0 {
            let feet = self.body.feet();
            let level = self.crypt.level_at(feet).unwrap_or(0);
            self.portal.open((feet, self.yaw, level));
            self.warp(events);
            self.respawn();
            // Snu deg mot portalen, så du ser den våkne.
            if let Some(ring) = self.portal_pose() {
                let d = Vec3::from(ring.translation) - self.body.pos;
                self.yaw = (-d.x).atan2(-d.z);
                self.pitch = 0.05;
            }
            let c = self.portal_center().unwrap_or(self.body.pos);
            events.push(self.sound_at("gate", c - Vec3::Y * 1.4, 1.0));
            events.push(self.sound_at("portal_open", c, 1.0));
            events.push(Event::Toast(
                "Virvelen tok deg hjem. Portalen ved sørporten tar deg tilbake dit du var.".into(),
            ));
        } else {
            self.respawn();
            events.extend([Event::Fade, Event::Toast("Tilbake ved sørporten".into())]);
        }
    }

    /// «Tilbake til sørporten» fra menyen i nettleseren.
    pub fn go_home_now(&mut self) {
        let mut events = vec![];
        self.go_home(&mut events);
        self.pending.extend(events);
    }

    pub(super) fn warp(&mut self, events: &mut Vec<Event>) {
        self.portal.warp_t = 0.0;
        self.body.jumped = true;
        events.extend([Event::Warp, sfx("warp")]);
    }

    /// Åpne en bryter (port, dør, falsk vegg, lem) med det den styrer.
    pub(super) fn open_switch(&mut self, sw: usize) {
        self.mech.open_switch(&mut self.crypt, sw);
        self.map.dirty = true;
    }

    /// Fiendene i nivået du kommer til (bossene du har slått, er borte).
    pub(super) fn enter_level(&mut self, k: usize) {
        if self.mobs.level != k || self.mobs.mobs.is_empty() {
            self.mobs.enter(&self.crypt, k);
        }
        if self.run.slain.contains(&k) {
            self.mobs.remove_bosses();
        }
    }

    /// Til et annet nivå (trappa ned): startstedet der.
    pub fn go_to_level(&mut self, k: usize) -> bool {
        let Some(l) = self.crypt.levels.get(k) else {
            return false;
        };
        let (to, yaw) = l.start;
        self.portal.enter(k, to, yaw);
        self.teleport(to, yaw);
        self.enter_level(k);
        true
    }

    /// Det du bar opp fra kjelleren legges i lageret.
    pub(super) fn bank(&mut self, events: &mut Vec<Event>) {
        if let Some(note) = self.hero.bank() {
            self.say([note], events);
        }
    }

    /// Død: du våkner ved det siste hvilestedet du tente, og det du bar
    /// blir liggende der du døde. Har du ikke tent noe, våkner du ved
    /// krypten i byen, og portalen tar deg til inngangen til det dypeste
    /// nivået du har nådd. En firedel av tokenene blir igjen nede. Fiendene
    /// kommer igjen, men portene du har åpnet, står fortsatt oppe.
    pub(super) fn die(&mut self, events: &mut Vec<Event>) {
        let (lost, bag) = self.hero.die();
        self.combat.reset();
        self.hud.clear();
        let feet = self.body.feet();
        if !bag.is_empty() {
            let n: u32 = bag.values().sum();
            if self.in_crypt() {
                let at = self.crypt.snap(feet, 3.0).unwrap_or(feet);
                self.run.bags.push((at, bag));
                events.push(Event::Toast(format!(
                    "{n} komponenter ligger igjen der du døde."
                )));
            } else {
                events.push(Event::Toast(format!("{n} komponenter ble borte.")));
            }
        }
        events.extend([Event::Fade, sfx("death")]);
        if let Some(r) = self
            .run
            .last_rest
            .and_then(|r| self.crypt.rests.get(r).cloned())
        {
            let front = Vec3::new(r.yaw.sin(), 0.0, r.yaw.cos());
            let to = self
                .crypt
                .snap(r.pos + front * 1.3, 3.0)
                .unwrap_or(r.pos + front * 1.3);
            self.teleport(to, r.yaw + std::f32::consts::PI);
            self.enter_level(r.level);
            events.push(Event::Toast(format!("Du ble overkjørt av forretningslogikk, og våkner ved hvilestedet. {lost} tokens ble igjen.")));
            return;
        }
        self.mobs.reset(&self.crypt);
        // Portalen ved sørporten tar deg til inngangen til det dypeste
        // nivået du har nådd.
        if let Some(d) = self.portal.deepest() {
            self.portal.open(d);
            events.push(Event::Toast(format!(
                "Portalen ved sørporten har våknet: den tar deg til inngangen til nivå {}.",
                d.2 + 1
            )));
        }
        let (to, yaw) = self.crypt_exit;
        self.teleport(to, yaw);
        events.push(Event::Toast(format!(
            "Du ble overkjørt av forretningslogikk. {lost} tokens ble igjen nede."
        )));
    }

    /// En fiende falt: bossen gir skatten og seieren, eliten nøkkelen og en
    /// komponent.
    pub(super) fn killed(&mut self, mob: usize, pos: Vec3, events: &mut Vec<Event>) {
        let (kind, elite) = self
            .mobs
            .mobs
            .get(mob)
            .map(|m| (m.kind, m.elite))
            .unwrap_or((MobKind::Button, false));
        if kind.boss() {
            self.run.slain.insert(self.mobs.level);
            for c in craft::BOSS_LOOT {
                self.hero.carry(c, 1);
            }
            self.won = true;
            self.map.dirty = true;
            events.push(Event::Toast("Rammeverket er beseiret! Skatten i alkoven er din, og en dialog og en tabs ligger igjen i buret.".into()));
        } else if elite {
            let c = craft::ELITE_LOOT[craft::theme_of(self.crypt.level_at(pos).unwrap_or(0))];
            self.hero.carry(c, 1);
            events.push(self.sound_at("key", pos - Vec3::Y * 0.8, 1.0));
            events.push(Event::Toast(format!(
                "Elitefienden falt fra hverandre: nøkkelen og en {c} ligger igjen."
            )));
        } else if kind != MobKind::Tooltip {
            events.push(Event::Toast(format!(
                "{} falt fra hverandre. Plukk opp tokenene.",
                kind.label()
            )));
        }
        if self.aimed_mob == Some(mob) {
            self.aimed_mob = None;
        }
    }
}
