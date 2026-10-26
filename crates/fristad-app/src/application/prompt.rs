//! Siktet: navnet, verbet for F, handelen for E og en kort forklaring for
//! det du sikter på, slik det er nå (LO-TEKs tooltip rundt siktekorset).

use super::session::Session;
use crate::domain::craft;
use crate::domain::world::crypt;
use crate::domain::world::mechanisms::Mechanisms;
use crate::domain::world::Use;
use crate::ports::driving::Prompt;

fn p(label: &str, verb: &str, trade: &str, info: &str) -> Prompt {
    Prompt { label: label.into(), verb: verb.into(), trade: trade.into(), info: info.into() }
}

impl Session {
    /// Siktet for et mål, slik det er nå.
    pub(super) fn prompt_for(&self, i: usize) -> Prompt {
        match self.targets[i].target.what {
            Use::Sleep => {
                let price = crate::domain::hero::SLEEP_PRICE;
                let verb = format!("Sov ({price} tokens)");
                if self.hero.coins < price {
                    p("Senga di", &verb, "", &format!("Har ikke råd: du har {} tokens.", self.hero.coins))
                } else {
                    p("Senga di", &verb, "", "Sov til døgnet snur, og våkn med fullt liv og full utholdenhet.")
                }
            }
            Use::Gallery(n) => match craft::GALLERY.get(n) {
                Some(c) if self.hero.found.contains(*c) => p(&craft::title(c), "Les", "", "Du har funnet den. Les om den i boka."),
                Some(c) => p("Tom ramme", "", "", &format!("Her henger {} når du har funnet den. {}", craft::title(c), craft::source(c).describe())),
                None => p("Tom ramme", "", "", ""),
            },
            Use::Talk(n) => {
                let name = self.npcs.npcs.get(n).map(|c| c.name).unwrap_or("Innbygger");
                let (trade, info) = match name {
                    "Smeden" => ("Smi", "Smeden smir utstyr av komponentene fra krypten."),
                    "Apotekeren" => ("Handle", "Drikker som leger, og tokens for det du har funnet."),
                    "Torgils" => ("Handle", "Lærer deg trekk på øvingsplassen."),
                    "Skriptoriet" => ("Handle", "Kjøp og bytt komponenter, og les om dem i bøkene."),
                    "Trubaduren" => ("Kjøp mjød", "Et krus mjød (1 token), så synger han kvadet om Gorvek."),
                    "Vertinnen" => ("", "Vertshuset. Her får du høre hva som går rykter om krypten."),
                    _ => ("", ""),
                };
                p(name, "Snakk", trade, info)
            }
            Use::Shortcut(n) => {
                if self.mech.shortcut_open(n) {
                    p("Snarveien", "", "", "Døra står åpen: en kortere vei gjennom nivået.")
                } else if Mechanisms::shortcut_inside(&self.crypt, n, self.body.pos) {
                    p("Eikedør med slå", "Skyv slåen", "", "En snarvei tilbake mot hvilestedet.")
                } else {
                    p("Låst dør", "", "", "Boltet fra den andre siden. Kanskje du finner veien rundt.")
                }
            }
            Use::Lever(n) => {
                if self.mech.lever_pulled(n) {
                    p("Spaken", "", "", "Trukket. Noe et sted har åpnet seg.")
                } else {
                    p("Spak", "Trekk", "", "En rusten spak i muren. Den henger sammen med noe et sted.")
                }
            }
            Use::Secret(n) => {
                if self.mech.secret_open(n) {
                    p("Åpning i muren", "", "", "Muren ga etter.")
                } else {
                    p("Sprukken mur", "Skyv", "", "Steinene sitter løst her, og det trekker kaldt gjennom fugene. Slå den inn med våpenet, eller skyv (F).")
                }
            }
            Use::Ladder(n) => {
                // Gjennom en lem bare der stigen ender ved en.
                let hatch = self.crypt.ladders.get(n).is_some_and(|l| self.crypt.trapdoors.iter().any(|t| t.pos.distance(l.top) < 2.5));
                p("Stige", "Klatre opp", "", if hatch { "Opp gjennom lemmen i taket." } else { "Opp til kanten over deg." })
            }
            Use::Lift(n) => {
                let braked = self.crypt.lifts.get(n).is_some_and(|l| !self.crypt.open.get(l.switch).copied().unwrap_or(false));
                if braked {
                    p("Heisen", "Dra", "", "Plattformen henger i kjettinger ned i sjakta. Bremsen på vinsjen holder den.")
                } else if self.mech.lift_moving(n) {
                    p("Heisen", "", "", "Den kjører.")
                } else if self.mech.lift_up(n) {
                    p("Heisen", "Kjør ned", "", "Dra i styrestangen, så går heisen ned sjakta.")
                } else {
                    p("Heisen", "Kjør opp", "", "Dra i styrestangen, så heises du opp.")
                }
            }
            Use::Winch(n) => {
                if self.mech.lift_up(n) {
                    p("Vinsjen", "", "", "Trommelen med kjettingen som holder heisen. Heisen står oppe.")
                } else {
                    p("Vinsjen", "Sveiv", "", "Sveiv heisen opp fra bunnen av sjakta.")
                }
            }
            Use::Rest(n) => {
                if self.run.rest_lit(n) {
                    p("Hvilestedet", "Hvil", "", "Fullt liv og full utholdenhet. Fiendene kommer tilbake. Dør du, våkner du her.")
                } else {
                    p("Alteret med lysestaken", "Tenn og hvil", "", "Tenn lysene og hvil: fullt liv og full utholdenhet, og her våkner du om du dør.")
                }
            }
            Use::Lore(n) => {
                let title = self.crypt.lores.get(n).map(|l| l.title.as_str()).unwrap_or("En lapp");
                p(title, "Les", "", if self.run.lore_read.get(n).copied().unwrap_or(false) { "Du har lest den." } else { "Noe er skrevet her." })
            }
            Use::Bonus(n) => {
                let taken = self.crypt.bonuses.get(n).is_some_and(|b| self.hero.bonus_taken.contains(&b.key));
                let life = self.crypt.bonuses.get(n).is_some_and(|b| b.kind == crypt::BonusKind::Life);
                if taken {
                    p("Tomt relikvieskrin", "", "", "Du har tatt det som var her.")
                } else {
                    p("Relikvie", "Ta", "", if life { "Varig styrke: +10 liv." } else { "Varig styrke: +10 utholdenhet." })
                }
            }
            Use::Chest(n) => {
                if self.mech.chests[n] {
                    p("Tom kiste", "", "", "Du har allerede tatt det som lå her.")
                } else {
                    p("Kiste", "Åpne", "", "Komponenter og tokens. Det du bærer mistes om du dør.")
                }
            }
            Use::Gate(n) => {
                let key = self.crypt.gates.get(n).is_some_and(|g| g.key);
                if self.mech.gate_open(n) {
                    p("Gitteret er oppe", "", "", "Veien videre er åpen.")
                } else if !key {
                    p("Jerngitter", "", "", "Gitteret sitter fast. En spak et sted heiser det.")
                } else if self.hero.keys > 0 {
                    p("Rusten port", "Lås opp", "", "Du har en nøkkel.")
                } else {
                    p("Rusten port", "Lås opp", "", "Låst. En sterkere fiende i nivået bærer nøkkelen.")
                }
            }
            Use::Descend => p("Krypten under kirka", "Gå ned", "", "Ned i krypten, der de korrupte komponentene holder til."),
            Use::Ascend(0) => p("Trappa opp", "Gå opp", "", "Opp til krypten og byen. Det du bærer legges i lageret."),
            Use::Ascend(k) => p("Trappa opp", "Gå opp", "", &format!("Opp til nivå {}.", self.crypt.levels.get(k - 1).map(|l| l.n).unwrap_or(k as u32))),
            Use::Portal => match self.portal.dest {
                Some((_, _, f)) => p("Portalen", "Gå gjennom", "", &format!("Virvelen tar deg tilbake til krypten, nivå {}.", f + 1)),
                None => p("Portalen", "", "", "Lukket."),
            },
            Use::Well(n) => {
                if self.mech.well_busy(n) {
                    p("Brønn", "", "", "Sveiven går. Bøtta er på vei opp.")
                } else {
                    p("Brønn", "Heis opp vann", "", "Kaldt vann fra dypet: litt liv tilbake og full utholdenhet.")
                }
            }
        }
    }
}
