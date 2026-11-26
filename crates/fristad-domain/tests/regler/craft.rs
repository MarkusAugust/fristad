//! Smiinga og komponentene (`domain/craft.rs`).

/// Skjemaet i Smia sier hvor hver komponent i oppskriftene finnes, og de som
/// ikke kan kjøpes i Skriptoriet merkes «bare i krypten». Alle
/// komponentene i oppskriftene må finnes et sted.
#[test]
fn skjemaet_sier_hvor_komponentene_finnes() {
    use crate::domain::craft;
    let button = craft::source("button");
    assert!(!button.crypt_only() && button.describe().contains("Skriptoriet"), "{}", button.describe());
    let card = craft::source("card");
    assert!(card.crypt_only() && card.describe().starts_with("Finnes bare i krypten"), "{}", card.describe());
    let mut nowhere = vec![];
    for r in craft::RECIPES {
        for c in r.needs.iter().map(|(c, _)| *c).chain(r.requires.iter().copied()) {
            let s = craft::source(c);
            if s.chests.is_empty() && s.elite.is_empty() && !s.boss && s.shop.is_none() {
                nowhere.push(format!("{c} ({})", r.name));
            }
        }
    }
    assert!(nowhere.is_empty(), "finnes ingen steder: {}", nowhere.join(", "));
    // Kort på én linje til skjemaet, og hver ting har sin linje om seg selv.
    assert_eq!(button.brief().lines().count(), 1);
    assert!(button.brief().contains("Skriptoriet"), "{}", button.brief());
    let ids = craft::WEAPONS.iter().map(|w| w.id).chain(craft::SHIELDS.iter().map(|s| s.id)).chain(craft::ARMORS.iter().map(|a| a.id)).chain(craft::RECIPES.iter().map(|r| r.id));
    let silent: Vec<&str> = ids.filter(|id| craft::about(id).is_empty()).collect();
    assert!(silent.is_empty(), "uten beskrivelse: {silent:?}");
}
