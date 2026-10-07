//! Kjernen i Fristad: verden, reglene og figurene, uten noen avhengighet
//! til porter, applikasjon, adaptere, GPU, nettleser, vindu, fysikkmotor
//! eller filformater. Testen i `architecture.rs` passer på det.
//!
//! - `geometry`, `model`, `anim`, `scene`, `rng`: rene data og bygging.
//! - `world`: middelalderbyen, terrenget, naturen og kjelleren, bygd fra
//!   modellene som finnes.
//! - `combat`, `mobs`, `craft`, `npcs`, `herd`: reglene for kampen,
//!   fiendene, smiinga, innbyggerne og dyrene.
//! - `weather`: værtypene og hva de gjør med lyset.

pub mod anim;
pub mod combat;
pub mod craft;
pub mod geometry;
pub mod herd;
pub mod mobs;
pub mod model;
pub mod npcs;
pub mod rng;
pub mod scene;
pub mod tokens;
pub mod weather;
pub mod world;
