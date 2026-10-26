//! Målet og journalen: én linje øverst til venstre som sier hva du kan gjøre
//! nå (gå til krypten, finne trappa ned, slå bossen, bære funnet hjem, smi),
//! og journalen (J) med lappene og bøkene du har lest i krypten. Det dypeste
//! nivået du har nådd og lappene du har lest, lagres (`SaveData.deepest`,
//! `SaveData.lore`), så målet husker deg neste gang.

use super::session::Session;
use crate::domain::lantern;
use crate::domain::world::crypt::run::{ANSGAR_TEXT, ANSGAR_TITLE};
use crate::ports::driving::Event;

impl Session {
    /// Det dypeste nivået du har stått i (0 er nivå 1).
    pub fn deepest(&self) -> Option<usize> {
        self.portal.entries.iter().rposition(|e| e.is_some())
    }

    /// Målet nå, som én setning.
    pub fn goal(&self) -> String {
        if self.in_crypt() {
            let k = self.mobs.level;
            let Some(level) = self.crypt.levels.get(k) else { return String::new() };
            let boss = self.crypt.spawns.iter().any(|s| s.level == k && s.boss) && !self.run.slain.contains(&k);
            let mut text = if boss {
                format!("Nivå {}: {}. Noe stort vokter bunnen av nivået; slå det, så åpner veien ned seg.", level.n, level.name)
            } else if level.down.is_some() {
                format!("Nivå {}: {}. Finn trappa ned, der det blå lyset står.", level.n, level.name)
            } else {
                format!("Nivå {}: {}. Dette er bunnen av krypten.", level.n, level.name)
            };
            if self.hero.lamp.fraction() < lantern::LOW {
                text.push_str(" Lykta brenner ned: finn et hvilested, eller gå opp.");
            } else if !self.hero.carried.is_empty() {
                text.push_str(" Det du bærer, er trygt først i byen (H).");
            }
            return text;
        }
        match self.deepest() {
            None => "Gå ut nordporten til kirkegården. Døra i krypten der fører ned.".into(),
            Some(d) => {
                let n: u32 = self.hero.inventory.values().sum();
                let deep = self.crypt.levels.get(d).map(|l| l.n).unwrap_or(d as u32 + 1);
                if n > 0 {
                    format!("Du har {n} komponenter i lageret: smi utstyr hos Smeden, eller gå ned igjen. Dypeste nivå: {deep}.")
                } else {
                    format!("Gå ned i krypten igjen. Dypeste nivå du har nådd: {deep}.")
                }
            }
        }
    }

    /// Lappene og bøkene du har lest, tittel og tekst: Ansgars lapp fra
    /// første gang du gikk ned, så de i krypten i den rekkefølgen de står.
    pub fn journal(&self) -> Vec<(String, String)> {
        let ansgar = self.run.welcomed.then(|| (ANSGAR_TITLE.to_string(), ANSGAR_TEXT.to_string()));
        ansgar.into_iter().chain(self.crypt.lores.iter().zip(&self.run.lore_read).filter(|(_, r)| **r).map(|(l, _)| (l.title.clone(), l.text.clone()))).collect()
    }

    /// Titlene i journalen, til lagringen (`SaveData.lore`).
    pub(super) fn lore_titles(&self) -> Vec<String> {
        self.journal().into_iter().map(|(t, _)| t).collect()
    }

    /// Sender målet og journalen når noe er endret.
    pub(super) fn update_goal(&mut self, events: &mut Vec<Event>) {
        let text = self.goal();
        let read = self.run.lore_read.iter().filter(|r| **r).count() + self.run.welcomed as usize;
        if self.goal_sent.as_ref().is_some_and(|(t, n)| *t == text && *n == read) {
            return;
        }
        self.goal_sent = Some((text.clone(), read));
        events.push(Event::Goal { text, journal: self.journal() });
    }

    /// Fra en lagring: nivåene du har nådd (portalen tar deg til
    /// inngangen) og lappene du har lest. Ansgars lapp er lest om tittelen
    /// står der; gamle lagringer uten den får den bare om du ikke har
    /// kommet lenger enn nivå 1.
    pub(super) fn restore_progress(&mut self, deepest: Option<usize>, lore: &[String]) {
        self.run.welcomed = lore.iter().any(|t| t == ANSGAR_TITLE) || deepest.is_some_and(|d| d > 0);
        if let Some(d) = deepest {
            for k in 0..=d.min(self.portal.entries.len().saturating_sub(1)) {
                if self.portal.entries[k].is_none() {
                    self.portal.entries[k] = self.crypt.levels.get(k).map(|l| l.start);
                }
            }
        }
        for (l, read) in self.crypt.lores.iter().zip(self.run.lore_read.iter_mut()) {
            if lore.contains(&l.title) {
                *read = true;
            }
        }
        self.goal_sent = None;
    }
}
