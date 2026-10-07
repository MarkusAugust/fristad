//! Fristad: et spill i en middelalderby, der komponentene fra designsystemet
//! Fristil er byttet du vinner i kjelleren under borgen.
//!
//! Hexagonal arkitektur (porter og adaptere):
//!
//! - `domain`: kjernen. Verden, reglene og figurene, uten avhengigheter
//!   utover matematikk og serde.
//! - `ports`: grensesnittene. Drevne porter (`Renderer`, `Physics`,
//!   `ModelSource`) og den drivende porten (`driving::Play`).
//! - `application`: bruker kjernen gjennom portene, og implementerer den
//!   drivende porten (`App`).
//! - `adapters`: wgpu, Rapier, glTF, nettleseren og vinduet.
//!
//! `architecture.rs` er en test som leser kildefilene og feiler om et lag
//! bruker noe det ikke skal.

pub mod adapters;
pub mod application;
pub mod domain;
pub mod ports;

#[cfg(test)]
mod architecture;
#[cfg(test)]
mod tests;
