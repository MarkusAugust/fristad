//! Skriver `assets/layout/town.json` på nytt i fast form (`layout_json`):
//! én ting per linje, rundede tall. Etter at fila er endret for hånd.
//!
//!     cargo run --bin layout

use fristad::adapters::data::{layout_file, layout_json};
use fristad::domain::world::layout::Layout;

fn main() {
    let text = layout_file().expect("assets/layout/town.json finnes ikke");
    let layout: Layout = serde_json::from_str(text).expect("assets/layout/town.json kan ikke leses");
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/layout/town.json"), layout_json(&layout)).unwrap();
}
