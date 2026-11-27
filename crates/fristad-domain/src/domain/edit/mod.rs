//! Redigeringsmodusen som ren logikk (`docs/redigering.md`): eieren
//! velger, flytter, dreier, kopierer og sletter tingene i byen og husene,
//! legger ut linjer og angrer, og reglene (`rules`) sier hva som er galt.
//! Ingen nettleser, GitHub eller filer her: dataene inn, dataene ut.

mod below;
mod crypt;
mod editor;
pub mod ghost;
mod group;
mod history;
mod lamps;
mod lines;
pub mod level;
pub mod route;
pub mod props;
pub mod rules;
mod select;
mod surface;

pub use editor::{Editor, Selected};
pub use lines::LineKind;
pub use props::Prop;

use crate::domain::model::ModelInfo;
use crate::domain::world::crypt::level::{CryptContents, LevelFile};
use crate::domain::world::crypt::{self as crypt_world, Crypt};
use crate::domain::world::layout::{self, Item, Layout};
use crate::domain::world::{town, Builder};
use glam::Vec2;
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
pub fn town_scene(src: &Sources, layout: &Layout, contents: &HashMap<String, Vec<Item>>) -> Builder {
    let mut b = Builder::new(&src.models);
    b.insides = src.insides.iter().cloned().collect();
    b.contents = contents.clone();
    layout::apply(&mut b, layout);
    b
}

/// Bygg bare krypten fra dataene i redigeringen (merkene og lysene i
/// `crypt` går foran dem i nivåfilene), med nedgangen i `anchor`.
pub fn crypt_scene(src: &Sources, anchor: Vec2, crypt: &BTreeMap<u32, CryptContents>) -> (Builder, Crypt) {
    let mut b = Builder::new(&src.models);
    b.insides = src.insides.iter().cloned().collect();
    let c = build_crypt(&mut b, src, anchor, crypt);
    (b, c)
}

fn build_crypt(b: &mut Builder, src: &Sources, anchor: Vec2, crypt: &BTreeMap<u32, CryptContents>) -> Crypt {
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
    crypt_world::build(b, anchor, levels)
}

/// Bygg byen og krypten i ett, slik spillet gjør. Redigeringen bygger dem
/// hver for seg (`town_scene` og `crypt_scene`); denne er fasiten testene
/// sammenligner med.
pub fn scene(src: &Sources, layout: &Layout, contents: &HashMap<String, Vec<Item>>, crypt: &BTreeMap<u32, CryptContents>) -> (Builder, Crypt) {
    let mut b = town_scene(src, layout, contents);
    let c = build_crypt(&mut b, src, layout.crypt_anchor, crypt);
    (b, c)
}
