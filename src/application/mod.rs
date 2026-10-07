//! Applikasjonen: bruker kjernen (`domain`) gjennom portene (`ports`), og
//! implementerer den drivende porten (`ports::driving::Play`) som
//! nettleseren, vinduet og skjermbildeverktøyet snakker med.
//!
//! - `assembly`: laster modellene gjennom `ModelSource` og bygger verden.
//! - `session`: økta, spillereglene satt sammen med fysikken.
//! - `view`: bildet rendereren får.
//! - `app`: `App`, som eier økta og de drevne adapterne.

pub mod app;
pub mod assembly;
pub mod session;
pub mod view;

pub use app::App;
pub use assembly::{prepare, rigs, Prepared};
pub use session::Session;
