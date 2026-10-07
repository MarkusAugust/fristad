//! Kjernen i Fristad: byen, komponentene og spillet, uten noen avhengighet
//! til GPU, nettleser, vindu, fysikkmotor eller filformater. Alt det bor i
//! `adapters/`, og snakker med kjernen gjennom portene i `ports.rs`.
//!
//! Lagene, innenfra og ut:
//!
//! - `tokens`, `geometry`, `text`, `model`, `palette`: rene data og bygging.
//! - `city`: byen bygges prosedyrisk fra tokens, med fasader, trær og
//!   gatemøbler.
//! - `installations`: hver Fristil-komponent slik den står i byen, med
//!   geometri, hva den gjør når du bruker den, og hvordan den animeres.
//!   Nye komponenter legges til som en fil her og én linje i `all()`.
//! - `game`: spilleren, døgnet, været og samspillet mellom installasjonene.
//! - `scene`: det rendereren får per bilde.

pub mod anim;
pub mod city;
pub mod combat;
pub mod game;
pub mod geometry;
pub mod herd;
pub mod installations;
pub mod mobs;
pub mod model;
pub mod npcs;
pub mod palette;
pub mod ports;
pub mod scene;
pub mod sheets;
pub mod text;
pub mod tokens;
