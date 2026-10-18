//! Komponentene (plan 14): hva en gjennomspilling gir, hvilke oppskrifter
//! det holder til, og byttet hos Buttonsmeden.
//! `cargo test --release komponentene -- --nocapture` skriver tallene.

use crate::domain::craft::{self, Rarity};
use crate::domain::loot::{self, Level, Rules};
use crate::domain::mob_kind::Kind;
use crate::domain::world::crypt::Crypt;
use crate::tests::world;
use glam::Vec3;

/// Et sted på gulvet foran kista.
fn front(c: &Crypt, pos: Vec3, yaw: f32) -> Vec3 {
    let q = pos + Vec3::new(yaw.sin(), 0.0, yaw.cos()) * 0.9;
    match c.floor_under(q + Vec3::Y * 0.3) {
        Some(y) if (y - pos.y).abs() < 1.0 => q.with_y(y),
        _ => c.snap(q + Vec3::Y * 0.3, 2.0).unwrap_or(q),
    }
}

/// Nivåene slik tellingen ser dem. `search`: også kistene bak de falske
/// veggene og de som ellers ikke kan nås fra startstedet (med portene,
/// spakene og snarveiene åpne). Uten leting slår du fiendene på veien
/// (`share` av dem) og åpner kistene langs den.
pub(crate) fn levels(search: bool, share: f32) -> Vec<Level> {
    let c = &world().world.crypt;
    let mut open = c.clone();
    let secret: Vec<usize> = c.secrets.iter().map(|s| s.switch).collect();
    for (i, o) in open.open.iter_mut().enumerate() {
        *o = !secret.contains(&i);
    }
    (0..c.levels.len())
        .map(|k| {
            let start = c.levels[k].start.0;
            let chests = c.chests.iter().filter(|ch| ch.level == k).filter(|ch| search || open.path(start + Vec3::Y * 0.2, front(c, ch.pos, ch.yaw)).is_some()).map(|ch| ch.loot.clone()).collect();
            let spawns = c.spawns.iter().filter(|s| s.level == k);
            let mobs: u32 = spawns.clone().filter(|s| !s.elite && !s.boss && s.label != "tooltip").map(|s| s.n).sum();
            let share = if search { 1.0 } else { share };
            let tokens: f32 = spawns.clone().map(|s| (Kind::from_label(&s.label).stats().tokens * s.n) as f32 * if s.elite || s.boss { 1.0 } else { share }).sum();
            Level {
                chests,
                elites: spawns.clone().filter(|s| s.elite).count() as u32,
                bosses: spawns.clone().filter(|s| s.boss && s.label.to_lowercase() != "rammeverket").count() as u32,
                last_boss: spawns.clone().any(|s| s.boss && s.label.to_lowercase() == "rammeverket"),
                mobs: (mobs as f32 * share).round() as u32,
                tokens: tokens.round() as u32,
            }
        })
        .collect()
}

/// Gjennomsnittet over mange frø: hvor mange oppskrifter per trinn
/// (vanlig, sjelden, episk) du kan smi med byttet, når `purse` av hundre
/// tokens går til komponenter hos Buttonsmeden, og hvor mange komponenter
/// det var.
pub(crate) fn tally(levels: &[Level], rules: &Rules, purse: u32) -> ([f32; 3], f32) {
    const SEEDS: u64 = 200;
    let tier = |r: Rarity| match r {
        Rarity::Common => 0,
        Rarity::Rare => 1,
        Rarity::Epic => 2,
    };
    let mut made = [0.0f32; 3];
    let mut count = 0.0;
    let tokens = levels.iter().map(|l| l.tokens).sum::<u32>() * purse / 100;
    for s in 0..SEEDS {
        let got = loot::haul(levels, rules, 0x9e37_79b9_7f4a_7c15 ^ (s * 0x1000_0001));
        count += got.values().sum::<u32>() as f32 / SEEDS as f32;
        for r in loot::forge(&got, tokens, true) {
            made[tier(r.rarity)] += 1.0 / SEEDS as f32;
        }
    }
    (made, count)
}

/// Halvparten av kistene (før de ble doblet 18. oktober, plan 14: de nye
/// står etter de gamle i nivåfilene), så økningen ikke hviler på dem.
fn halved(levels: &[Level]) -> Vec<Level> {
    levels.iter().map(|l| Level { chests: l.chests.iter().take(l.chests.len().div_ceil(2)).cloned().collect(), ..l.clone() }).collect()
}

/// Hvor mange tokens av hundre som går til komponenter hos Buttonsmeden;
/// resten går til olje, drikker, senga og Torgils.
const PURSE: u32 = 25;

/// Tallene før og etter plan 14, og at det holder: en vanlig gjennomspilling
/// (som boten: kistene langs veien, eliten, bossene og tre av fire fiender)
/// gir nok til omtrent hele andre trinn og noe av tredje, også med bare
/// halvparten av kistene, og med leting (de falske veggene og de sjeldne
/// kistene) til omtrent alt.
#[test]
fn komponentene_holder_til_utstyret() {
    let n = {
        let n = |t: Rarity| craft::RECIPES.iter().filter(|r| r.rarity == t).count() as f32;
        [n(Rarity::Common), n(Rarity::Rare), n(Rarity::Epic)]
    };
    let fmt = |t: [f32; 3]| format!("vanlig {:.1}/{} · sjelden {:.1}/{} · episk {:.1}/{}", t[0], n[0], t[1], n[1], t[2], n[2]);
    let normal = levels(false, 0.75);
    let all = levels(true, 1.0);
    let chests = |l: &[Level]| l.iter().map(|l| l.chests.len()).sum::<usize>();
    eprintln!(
        "kister: {} langs veien, {} i alt; fiender langs veien {}; tokens {}, {PURSE} % til komponenter",
        chests(&normal),
        chests(&all),
        normal.iter().map(|l| l.mobs).sum::<u32>(),
        normal.iter().map(|l| l.tokens).sum::<u32>()
    );
    let scenes = [("vanlig", normal.clone()), ("med leting", all.clone()), ("vanlig, halvparten av kistene", halved(&normal)), ("med leting, halvparten av kistene", halved(&all))];
    let mut now = vec![];
    for (name, rules) in [("før", &loot::BEFORE), ("nå", &loot::RULES)] {
        for (what, l) in &scenes {
            let (made, count) = tally(l, rules, PURSE);
            let (bare, _) = tally(l, rules, 0);
            eprintln!("{name}, {what}: {count:.0} komponenter; {} (uten tokens: {})", fmt(made), fmt(bare));
            if name == "nå" {
                now.push(made);
            }
        }
    }
    let [normal, all, half, half_all] = [now[0], now[1], now[2], now[3]];
    for (what, t) in [("vanlig", normal), ("vanlig med halvparten av kistene", half)] {
        assert!(t[0] >= n[0] - 0.2 && t[1] >= n[1] * 0.8 && t[2] >= 1.0, "en {what} gjennomspilling holder ikke til første trinn, omtrent hele andre og noe av tredje: {}", fmt(t));
    }
    assert!(all.iter().zip(n).all(|(t, n)| *t >= n - 0.5), "med leting holder det ikke til alt: {}", fmt(all));
    assert!(half_all.iter().zip(n).all(|(t, n)| *t >= n - 1.0), "med leting og halvparten av kistene holder det ikke til omtrent alt: {}", fmt(half_all));
}

/// Byttet hos Buttonsmeden: avslår uten nok av komponenten eller tokenene
/// (og endrer ingenting da), og går det, trekkes det du ga og avgiften, du
/// får én av den dyrere, og butikken kommer på nytt med de nye tallene.
#[test]
fn byttet_virker_og_avslar_uten_nok() {
    use crate::adapters::rapier::RapierPhysics;
    use crate::ports::driving::{Event, Input};
    let mut physics = RapierPhysics::new();
    let mut game = crate::tests::session(&mut physics);
    let mut input = Input::default();
    let s = loot::swap("badge_card").expect("byttet badge_card finnes ikke");
    let count = |game: &crate::application::Session, c: &str| game.inventory_view().stash.iter().find(|(k, _)| k == c).map(|(_, n)| *n).unwrap_or(0);
    game.set_coins(s.fee + 5);
    game.give(s.give, s.n - 1);
    assert!(!game.swap(s.id), "byttet gikk med for få {}", s.give);
    assert_eq!((count(&game, s.give), count(&game, s.get), game.coins_now()), (s.n - 1, 0, s.fee + 5), "et avslått bytte endret lageret");
    game.give(s.give, 1);
    game.set_coins(s.fee - 1);
    assert!(!game.swap(s.id), "byttet gikk uten nok tokens");
    assert_eq!((count(&game, s.give), count(&game, s.get), game.coins_now()), (s.n, 0, s.fee - 1), "et avslått bytte endret lageret");
    game.set_coins(s.fee + 5);
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    assert!(game.swap(s.id), "byttet gikk ikke med nok av alt");
    assert_eq!((count(&game, s.give), count(&game, s.get), game.coins_now()), (0, 1, 5), "byttet trakk feil");
    let events = game.update(&mut input, 1.0 / 30.0, &mut physics);
    let shop = events.iter().find_map(|e| if let Event::Shop { npc, swaps, .. } = e { Some((npc.clone(), swaps.clone())) } else { None }).expect("butikken kom ikke på nytt etter byttet");
    assert_eq!(shop.0, "Buttonsmeden");
    let v = shop.1.iter().find(|v| v.id == s.id).expect("byttet står ikke i butikken");
    assert_eq!((v.have, v.get_have, v.ok), (0, 1, false), "butikken viser ikke det du har nå: {v:?}");
    assert!(game.found().contains(&s.get.to_string()), "det du byttet til, skal henge i galleriet");
    assert!(!game.swap(s.id), "byttet gikk to ganger");
    assert!(!game.swap("finnes_ikke"));
}

/// Byttet kan ikke misbrukes: det går bare oppover (aldri til noe
/// vanligere enn det du gir), ingen kjede av bytter fører tilbake dit den
/// begynte, du gir alltid flere enn du får og en avgift, og ingenting i
/// byen gir tokens tilbake for en komponent.
#[test]
fn byttet_kan_ikke_misbrukes() {
    let rank = |c: &str| match loot::rarity(c) {
        Rarity::Common => 0,
        Rarity::Rare => 1,
        Rarity::Epic => 2,
    };
    for s in loot::SWAPS {
        assert!(craft::GALLERY.contains(&s.give) && craft::GALLERY.contains(&s.get), "{}: ukjent komponent", s.id);
        assert!(s.n >= 2 && s.fee > 0 && s.give != s.get, "{}: du må gi flere enn du får, og en avgift", s.id);
        assert!(rank(s.get) >= rank(s.give), "{}: byttet går nedover", s.id);
        // Kan den kjøpes, er byttet aldri billigere enn å kjøpe den.
        let price = |c: &str| craft::SMITH_WARES.iter().find(|w| matches!(w.what, craft::WareKind::Component(x) if x == c)).map(|w| w.price);
        if let (Some(get), Some(give)) = (price(s.get), price(s.give)) {
            assert!(give * s.n + s.fee > get, "{}: billigere å bytte enn å kjøpe", s.id);
        }
    }
    // Ingen ring: fra hver komponent kommer du aldri tilbake til den.
    fn reach(from: &str, to: &str, depth: usize) -> bool {
        depth < 32 && loot::SWAPS.iter().filter(|s| s.give == from).any(|s| s.get == to || reach(s.get, to, depth + 1))
    }
    for s in loot::SWAPS {
        assert!(!reach(s.get, s.give, 0), "{}: byttene går i ring tilbake til {}", s.id, s.give);
    }
    // Byen kjøper ingenting: varene koster tokens og gir aldri tokens.
    for w in craft::SMITH_WARES.iter().chain(craft::APOTHECARY_WARES).chain(craft::GORVEK_WARES) {
        assert!(w.price > 0, "{} er gratis", w.id);
    }
}
