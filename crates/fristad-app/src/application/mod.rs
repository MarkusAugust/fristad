//! Applikasjonen: bruker kjernen (`domain`) gjennom portene (`ports`), og
//! implementerer den drivende porten (`ports::driving::Play`) som
//! nettleseren, vinduet og skjermbildeverktøyet snakker med.
//!
//! - `assembly`: laster modellene gjennom `ModelSource` og bygger verden.
//! - `session`: økta, som koordinerer reglene i kjernen med fysikken, delt
//!   i `usage`, `prompt`, `fight`, `audio`, `trade`, `goals`, `water` og
//!   `testing`.
//! - `view`: bildet rendereren får.
//! - `editing`: redigeringen i økta (flyging, kommandoene, det flyttede i
//!   bildet).
//! - `app`: `App`, som eier økta og de drevne adapterne.

pub mod app;
pub mod assembly;
mod audio;
mod edit_marks;
mod editing;
mod fight;
mod goals;
mod prompt;
pub mod session;
mod testing;
mod tavern;
mod trade;
mod usage;
pub mod view;
mod water;

pub use app::App;
pub use assembly::{prepare, prepare_for_edit, prepare_with, rigs, EditData, Prepared};
pub use session::Session;
