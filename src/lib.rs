//! Fristad: dokumentasjonen til Fristil som en by du kan gå rundt i.
//!
//! Hexagonal arkitektur: `domain` er kjernen uten ytre avhengigheter,
//! `domain::ports` er portene, `adapters` er adapterne (wgpu, Rapier, glTF,
//! nettleser, eget vindu) og `app` setter det sammen.

pub mod adapters;
pub mod app;
pub mod domain;
