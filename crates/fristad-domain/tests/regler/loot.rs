//! Byttet i Skriptoriet (plan 14, `loot::SWAPS`). Det en gjennomspilling
//! gir, står i hovedpakken (`src/loot_tests.rs`), som har nivåene.

use crate::domain::craft::{self, Rarity};
use crate::domain::loot;

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
        let price = |c: &str| craft::SCRIPTORIUM_WARES.iter().find(|w| matches!(w.what, craft::WareKind::Component(x) if x == c)).map(|w| w.price);
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
    for w in craft::SCRIPTORIUM_WARES.iter().chain(craft::APOTHECARY_WARES).chain(craft::GORVEK_WARES) {
        assert!(w.price > 0, "{} er gratis", w.id);
    }
}
