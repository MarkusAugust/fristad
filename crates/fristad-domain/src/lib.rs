//! Kjernen i Fristad (se `src/lib.rs` i hovedpakken for arkitekturen). Egen
//! pakke, så kompilatoren sørger for at kjernen bare bruker det den har lov
//! til: matematikk (`glam`), `serde`, `bytemuck` og `log`.

pub mod domain;
