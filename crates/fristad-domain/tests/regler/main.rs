//! Reglene i kjernen, testet uten adapterne: kompilatoren for nivåene
//! (`crypt::compose`), nivåredigeringen (`edit::level`), steinene,
//! mekanismene, modulsettet, byttet og smiinga. `cargo test -p fristad-domain`
//! kjører dem uten wgpu, Rapier og modellene.
//!
//! Testene som trenger modellene, fysikken eller økta står i hovedpakken
//! (`src/*_tests.rs`). Byggesettet og nivådokumentene leses fra `assets/`
//! av `data.rs`; kjernen selv leser aldri filer.

use fristad_domain::domain;

mod craft;
mod data;
mod grid;
mod level_doc;
mod level_editor;
mod loot;
mod mechanism;
mod module_set;
mod stone;
