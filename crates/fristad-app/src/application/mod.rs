//! Applikasjonen: bruker kjernen (`domain`) gjennom portene (`ports`), og
//! implementerer den drivende porten (`ports::driving::Play`) som
//! nettleseren, vinduet og skjermbildeverktøyet snakker med.
//!
//! - `assembly`: laster modellene gjennom `ModelSource` og bygger verden.
//! - `session`: økta, som koordinerer reglene i kjernen med fysikken, delt
//!   i `usage`, `prompt`, `fight`, `audio`, `trade` og `testing`.
//! - `view`: bildet rendereren får.
//! - `app`: `App`, som eier økta og de drevne adapterne.

pub mod app;
pub mod assembly;
mod audio;
mod fight;
mod prompt;
pub mod session;
mod testing;
mod trade;
mod usage;
pub mod view;

pub use app::App;
pub use assembly::{prepare, rigs, Prepared};
pub use session::Session;
