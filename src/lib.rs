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
//! Kjernen, portene og applikasjonen er egne pakker (`crates/`), så
//! kompilatoren hindrer dem i å bruke det de ikke skal; `architecture.rs`
//! sjekker avhengighetene mellom pakkene. Her er adapterne og vertene.

pub use fristad_app::application;
pub use fristad_domain::domain;
pub use fristad_ports::ports;

pub mod adapters;

#[cfg(test)]
mod architecture;
#[cfg(test)]
mod edit_tests;
#[cfg(test)]
mod tests;
