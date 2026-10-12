//! Redigeringsmodusen som ren logikk (`docs/redigering.md`): eieren
//! velger, flytter, dreier, kopierer og sletter tingene i byen og husene,
//! legger ut linjer og angrer, og reglene (`rules`) sier hva som er galt.
//! Ingen nettleser, GitHub eller filer her: dataene inn, dataene ut.

mod crypt;
mod editor;
pub mod rules;

pub use editor::{Editor, LineKind, Selected};

use crate::domain::model::ModelInfo;
use crate::domain::world::crypt::level::{CryptContents, LevelFile};
use crate::domain::world::crypt::{self as crypt_world, Crypt};
use crate::domain::world::layout::{self, Item, Layout};
use crate::domain::world::{town, Builder};
use std::collections::{BTreeMap, HashMap};

/// Det byen bygges av utenom selve plasseringene: modellene med mål, det
/// husene og møblene har med seg (`<navn>_inside.json`), og kryptnivåene
/// (rommene, kolliderne og nettet; merkene og lysene er det redigeringen
/// endrer).
#[derive(Clone, Default)]
pub struct Sources {
    pub models: Vec<ModelInfo>,
    pub insides: Vec<(String, town::Inside)>,
    pub levels: Vec<(u32, LevelFile)>,
}

/// Bygg bare byen (uten bakke, landskap og krypt) fra dataene, til reglene
/// og til å finne tingene: rask nok til å gjøres etter hver endring.
pub fn town_scene(
    src: &Sources,
    layout: &Layout,
    contents: &HashMap<String, Vec<Item>>,
) -> Builder {
    let mut b = Builder::new(&src.models);
    b.insides = src.insides.iter().cloned().collect();
    b.contents = contents.clone();
    layout::apply(&mut b, layout);
    b
}

/// Bygg byen og krypten fra dataene i redigeringen (merkene og lysene i
/// `crypt` går foran dem i nivåfilene).
pub fn scene(
    src: &Sources,
    layout: &Layout,
    contents: &HashMap<String, Vec<Item>>,
    crypt: &BTreeMap<u32, CryptContents>,
) -> (Builder, Crypt) {
    let mut b = Builder::new(&src.models);
    b.insides = src.insides.iter().cloned().collect();
    b.contents = contents.clone();
    let town = layout::apply(&mut b, layout);
    let levels = src
        .levels
        .iter()
        .map(|(n, f)| {
            let mut f = f.clone();
            if let Some(c) = crypt.get(n) {
                f.marks = c.marks.clone();
                f.lights = c.lights.clone();
            }
            (*n, f)
        })
        .collect();
    let c = crypt_world::build(&mut b, town.crypt_anchor, levels);
    (b, c)
}
