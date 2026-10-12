//! Redigeringsmodusen som ren logikk (`docs/redigering.md`): eieren
//! velger, flytter, dreier, kopierer og sletter tingene i byen og husene,
//! legger ut linjer og angrer, og reglene (`rules`) sier hva som er galt.
//! Ingen nettleser, GitHub eller filer her: dataene inn, dataene ut.

mod editor;
pub mod rules;

pub use editor::{Editor, LineKind, Selected};

use crate::domain::model::ModelInfo;
use crate::domain::world::layout::{self, Item, Layout};
use crate::domain::world::{town, Builder};
use std::collections::HashMap;

/// Det byen bygges av utenom selve plasseringene: modellene med mål og det
/// husene og møblene har med seg (`<navn>_inside.json`).
#[derive(Clone, Default)]
pub struct Sources {
    pub models: Vec<ModelInfo>,
    pub insides: Vec<(String, town::Inside)>,
}

/// Bygg bare byen (uten bakke, landskap og krypt) fra dataene, til reglene
/// og til å finne tingene: rask nok til å gjøres etter hver endring.
pub fn town_scene(src: &Sources, layout: &Layout, contents: &HashMap<String, Vec<Item>>) -> Builder {
    let mut b = Builder::new(&src.models);
    b.insides = src.insides.iter().cloned().collect();
    b.contents = contents.clone();
    layout::apply(&mut b, layout);
    b
}
