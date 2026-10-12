//! Skriver `assets/layout/town.json` og `assets/layout/inside/<hus>.json`
//! på nytt i fast form (`layout_files`): én ting per linje, rundede tall.
//! Etter at filene er endret for hånd eller skrevet av Blender.
//!
//!     cargo run --bin layout

use fristad::adapters::data::{world_data, FileStore};
use fristad::ports::LayoutStore;

fn main() {
    let data = world_data();
    let layout = data.layout.expect("assets/layout/town.json finnes ikke");
    let contents: Vec<_> = data.contents;
    FileStore(concat!(env!("CARGO_MANIFEST_DIR")).into()).save(&layout, &contents).expect("kunne ikke skrive");
}
