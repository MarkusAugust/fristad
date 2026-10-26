//! Hva krypten gir av komponenter, og byttet i Skriptoriet (plan 14;
//! eieren 18. oktober: «det er for vanskelig å samle komponentene som trengs
//! for å bygge utstyr»). Kistene, eliten, bossene og de vanlige fiendene gir
//! etter `RULES`, fra temaets liste (`craft::LOOT`), og `haul` regner ut hva
//! en gjennomspilling gir, så `forge` kan si hvilke oppskrifter det holder
//! til. Rene data og regler; økta ruller terningen.

use crate::domain::craft::{self, Rarity, Recipe};
use crate::domain::rng::Rng;
use std::collections::BTreeMap;

/// Hvor mye hver kilde gir. `chest`: komponenter fra temaets liste per
/// vanlig kiste; `rare`: i tillegg til den sjeldne komponenten i en sjelden
/// kiste; `elite` og `boss`: i tillegg til det eliten i temaet bærer;
/// `final_boss`: hvor mange av hver Rammeverket legger igjen; `mob`: sjansen
/// for at en vanlig fiende mister én fra temaets liste.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub chest: u32,
    pub rare: u32,
    pub elite: u32,
    pub boss: u32,
    pub final_boss: u32,
    pub mob: f32,
}

/// Slik det var før plan 14: én komponent per kiste, eliten og bossene det
/// eliten bærer, og ingenting fra de vanlige fiendene.
pub const BEFORE: Rules = Rules { chest: 1, rare: 0, elite: 0, boss: 0, final_boss: 1, mob: 0.0 };

/// Slik det er nå: de sjeldne kistene, eliten og bossene gir mer fra
/// temaets liste, og en av tolv vanlige fiender mister en komponent
/// (tallene i `komponentene_holder_til_utstyret`).
pub const RULES: Rules = Rules { chest: 1, rare: 2, elite: 2, boss: 3, final_boss: 1, mob: 0.08 };

/// Hvor sjelden en komponent er: det byttet regner etter, og det kortene
/// viser. Rammeverkets er episke, krypten dypere ned sjeldne.
pub fn rarity(component: &str) -> Rarity {
    match component {
        "dialog" | "tabs" => Rarity::Epic,
        "alert" | "tooltip" | "fieldset" | "file-upload" | "search" => Rarity::Rare,
        _ => Rarity::Common,
    }
}

/// `n` komponenter fra temaets liste i nivå `level` (fra 0).
fn picks(level: usize, n: u32, rng: &mut Rng) -> Vec<String> {
    let list = craft::LOOT[craft::theme_of(level)];
    (0..n).map(|_| rng.pick(list).to_string()).collect()
}

/// Det en kiste i nivå `level` gir. En sjelden kiste (`rare`) har sin egen
/// komponent; er den ikke en av spillets (et navn fra nivåfila som ingen
/// oppskrift bruker), gir den en til fra listen.
pub fn chest(level: usize, rare: Option<&str>, rules: &Rules, rng: &mut Rng) -> Vec<String> {
    match rare {
        Some(c) if craft::GALLERY.contains(&c) => std::iter::once(c.to_string()).chain(picks(level, rules.rare, rng)).collect(),
        Some(_) => picks(level, rules.rare + 1, rng),
        None => picks(level, rules.chest, rng),
    }
}

/// Det eliten i nivå `level` legger igjen (nøkkelen kommer i tillegg).
pub fn elite(level: usize, rules: &Rules, rng: &mut Rng) -> Vec<String> {
    let c = craft::ELITE_LOOT[craft::theme_of(level)].to_string();
    std::iter::once(c).chain(picks(level, rules.elite, rng)).collect()
}

/// Det en boss legger igjen: bossen i et tema det eliten bærer og mer fra
/// listen, Rammeverket (`last`) det som bare finnes hos det.
pub fn boss(level: usize, last: bool, rules: &Rules, rng: &mut Rng) -> Vec<String> {
    if last {
        return craft::BOSS_LOOT.iter().flat_map(|c| std::iter::repeat_n(c.to_string(), rules.final_boss as usize)).collect();
    }
    let c = craft::ELITE_LOOT[craft::theme_of(level)].to_string();
    std::iter::once(c).chain(picks(level, rules.boss, rng)).collect()
}

/// Om en vanlig fiende i nivå `level` mister en komponent.
pub fn mob(level: usize, rules: &Rules, rng: &mut Rng) -> Option<String> {
    rng.chance(rules.mob).then(|| picks(level, 1, rng).remove(0))
}

/// Det som finnes i ett nivå, til tellingen: kistene (med den sjeldne
/// komponenten), eliten, bossene og de vanlige fiendene.
#[derive(Clone, Debug, Default)]
pub struct Level {
    pub chests: Vec<Option<String>>,
    pub elites: u32,
    pub bosses: u32,
    pub last_boss: bool,
    pub mobs: u32,
    /// Tokenene fiendene i nivået mister.
    pub tokens: u32,
}

/// Hva en gjennomspilling av nivåene gir med reglene og frøet.
pub fn haul(levels: &[Level], rules: &Rules, seed: u64) -> BTreeMap<String, u32> {
    let mut rng = Rng(seed | 1);
    let mut got: BTreeMap<String, u32> = BTreeMap::new();
    let mut add = |v: Vec<String>| {
        for c in v {
            *got.entry(c).or_insert(0) += 1;
        }
    };
    for (k, l) in levels.iter().enumerate() {
        for c in &l.chests {
            add(chest(k, c.as_deref(), rules, &mut rng));
        }
        for _ in 0..l.elites {
            add(elite(k, rules, &mut rng));
        }
        for _ in 0..l.bosses {
            add(boss(k, false, rules, &mut rng));
        }
        if l.last_boss {
            add(boss(k, true, rules, &mut rng));
        }
        for _ in 0..l.mobs {
            add(mob(k, rules, &mut rng).into_iter().collect());
        }
    }
    got
}

/// Hvilke oppskrifter lageret holder til når du smir det du kan, de
/// enkleste først (trinn for trinn, i rekkefølgen i `RECIPES`). Det som
/// mangler, kjøpes i Skriptoriet for `tokens` om det selges der, og
/// ellers byttes det til (`trade`) når et bytte kan gi det.
pub fn forge(stash: &BTreeMap<String, u32>, tokens: u32, trade: bool) -> Vec<&'static Recipe> {
    let mut have = stash.clone();
    let mut purse = tokens;
    let mut made = vec![];
    for tier in [Rarity::Common, Rarity::Rare, Rarity::Epic] {
        for r in craft::RECIPES.iter().filter(|r| r.rarity == tier) {
            for _ in 0..12 {
                let Some((c, _)) = craft::missing(r, &have).into_iter().next() else { break };
                let price = craft::SCRIPTORIUM_WARES.iter().find(|w| matches!(w.what, craft::WareKind::Component(x) if x == c)).map(|w| w.price);
                if let Some(p) = price.filter(|p| *p <= purse) {
                    purse -= p;
                    *have.entry(c).or_insert(0) += 1;
                    continue;
                }
                let swap = SWAPS
                    .iter()
                    .filter(|s| trade && s.get == c && s.fee <= purse && have.get(s.give).copied().unwrap_or(0) >= s.n + spare(r, s.give))
                    .max_by_key(|s| have.get(s.give).copied().unwrap_or(0));
                let Some(s) = swap else { break };
                purse -= s.fee;
                *have.entry(s.give.into()).or_insert(0) -= s.n;
                *have.entry(c).or_insert(0) += 1;
            }
            if craft::missing(r, &have).is_empty() {
                for (c, n) in r.needs {
                    *have.entry(c.to_string()).or_insert(0) -= n;
                }
                made.push(r);
            }
        }
    }
    made
}

/// Hvor mange av `c` oppskriften selv trenger (de byttes ikke bort).
fn spare(r: &Recipe, c: &str) -> u32 {
    r.needs.iter().find(|(x, _)| *x == c).map(|(_, n)| *n).unwrap_or(0) + u32::from(r.requires.contains(&c))
}

/// Et bytte i Skriptoriet: `n` av `give` mot én `get`, og `fee` tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swap {
    pub id: &'static str,
    pub give: &'static str,
    pub n: u32,
    pub get: &'static str,
    pub fee: u32,
}

/// Kursene: tre av en vanlig mot én sjelden eller vanlig fra samme tema som
/// er vanskeligere å finne, to sjeldne mot én episk. Alt går oppover (det du
/// får, kan aldri byttes tilbake til mer enn du ga), og ingenting kan selges.
pub const SWAPS: &[Swap] = &[
    // Gravkjelleren
    Swap { id: "tag_divider", give: "tag", n: 3, get: "divider", fee: 4 },
    Swap { id: "badge_divider", give: "badge", n: 3, get: "divider", fee: 4 },
    // Kloakken
    Swap { id: "badge_card", give: "badge", n: 3, get: "card", fee: 6 },
    Swap { id: "button_progress", give: "button", n: 3, get: "progress", fee: 6 },
    Swap { id: "card_alert", give: "card", n: 3, get: "alert", fee: 8 },
    // Gruva
    Swap { id: "divider_label", give: "divider", n: 3, get: "label", fee: 8 },
    Swap { id: "card_help", give: "card", n: 3, get: "help-text", fee: 8 },
    Swap { id: "label_tooltip", give: "label", n: 3, get: "tooltip", fee: 10 },
    // Borgen under jorda
    Swap { id: "label_fieldset", give: "label", n: 3, get: "fieldset", fee: 12 },
    Swap { id: "alert_search", give: "alert", n: 3, get: "search", fee: 12 },
    Swap { id: "alert_upload", give: "alert", n: 3, get: "file-upload", fee: 12 },
    // Den korrupte kjernen
    Swap { id: "fieldset_dialog", give: "fieldset", n: 2, get: "dialog", fee: 20 },
    Swap { id: "search_tabs", give: "search", n: 2, get: "tabs", fee: 20 },
];

pub fn swap(id: &str) -> Option<Swap> {
    SWAPS.iter().copied().find(|s| s.id == id)
}
