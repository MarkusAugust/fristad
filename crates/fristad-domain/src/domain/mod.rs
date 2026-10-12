//! Kjernen i Fristad: verden, reglene og figurene, uten noen avhengighet
//! til porter, applikasjon, adaptere, GPU, nettleser, vindu, fysikkmotor
//! eller filformater. Pakken har bare `glam`, `bytemuck`, `serde` og `log`
//! som avhengigheter, og `architecture.rs` passer på det.
//!
//! - `geometry`, `model`, `anim`, `scene`, `rng`: rene data og bygging.
//! - `world`: middelalderbyen, terrenget, naturen og kjelleren, bygd fra
//!   modellene som finnes.
//! - `combat`, `mobs`, `craft`, `npcs`, `herd`: reglene for kampen,
//!   fiendene, smiinga, innbyggerne og dyrene.
//! - `weather`: værtypene, døgnet og himmelen (`Sky`).
//! - `body`, `hero`: spillerens kropp, og livet, lageret og utstyret.
//! - `note`: lydene og meldingene reglene har å si.
//! - `edit`: redigeringsmodusen (velg, flytt, angre) og reglene for byen.

pub mod anim;
pub mod body;
pub mod combat;
pub mod craft;
pub mod edit;
pub mod geometry;
pub mod herd;
pub mod hero;
pub mod mob_kind;
pub mod mobs;
pub mod model;
pub mod note;
pub mod npcs;
pub mod rng;
pub mod scene;
pub mod weather;
pub mod world;
