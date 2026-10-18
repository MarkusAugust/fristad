//! Applikasjonen i Fristad: monteringen (`assembly`), økta (`session`),
//! bildet til rendereren (`view`) og `App`, som implementerer `Play`. Bygger
//! på kjernen og portene, aldri på adapterne.

pub use fristad_domain::domain;
pub use fristad_ports::ports;
pub mod application;
