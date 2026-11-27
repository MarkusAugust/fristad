//! Krypten i redigeringen, bygget for seg: den er mange ganger større enn
//! byen (ti nivåer med rom og kollidere) og endres sjelden, så den bygges
//! og sjekkes bare når merkene, lysene eller nedgangen er endret, og ikke
//! hver gang noe flyttes i byen (eieren 27. november).

use super::rules::{self, Breach};
use super::{crypt, Sources};
use crate::domain::world::crypt::level::CryptContents;
use crate::domain::world::crypt::Crypt;
use crate::domain::world::layout::Ref;
use crate::domain::world::{Builder, Grid};
use glam::Vec2;
use std::cell::OnceCell;
use std::collections::{BTreeMap, HashSet};

pub(super) struct Below {
    pub scene: Builder,
    pub built: Crypt,
    /// Det reglene fant i krypten sist den ble bygget.
    pub breaches: Vec<Breach>,
    /// Det krypten ble bygget fra: nedgangen og innholdet. `None` når den
    /// må bygges uansett.
    from: Option<(Vec2, BTreeMap<u32, CryptContents>)>,
    grids: OnceCell<(Grid, Grid)>,
}

impl Below {
    pub fn empty() -> Self {
        Self { scene: Builder::new(&[]), built: Crypt::default(), breaches: vec![], from: None, grids: OnceCell::new() }
    }

    /// Glem det som er bygget, så neste `refresh` bygger og sjekker på nytt
    /// (det som regnes som endret, er et annet).
    pub fn forget(&mut self) {
        self.from = None;
    }

    /// Bygg krypten og sjekk reglene, om den ikke alt står slik.
    pub fn refresh(&mut self, src: &Sources, anchor: Vec2, crypt: &BTreeMap<u32, CryptContents>, approved: Option<&HashSet<String>>, touched: &HashSet<Ref>) {
        if self.from.as_ref().is_some_and(|(a, c)| *a == anchor && c == crypt) {
            return;
        }
        (self.scene, self.built) = super::crypt_scene(src, anchor, crypt);
        self.breaches = rules::check_below(&self.scene, approved, touched);
        self.breaches.extend(crypt::breaches(&self.built, crypt));
        self.from = Some((anchor, crypt.clone()));
        self.grids = OnceCell::new();
    }

    /// Rutenettene over gulvene og kolliderne i krypten.
    pub fn grids(&self) -> &(Grid, Grid) {
        self.grids.get_or_init(|| (self.scene.floor_grid(), rules::collider_grid(&self.scene.colliders)))
    }
}
