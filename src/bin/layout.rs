//! Skriver `assets/layout/town.json`, `assets/layout/inside/<hus>.json` og
//! `assets/layout/crypt_NN.json` på nytt i fast form (`layout_files`): én
//! ting per linje, rundede tall. Etter at filene er endret for hånd eller
//! skrevet av Blender eller `tools/crypt/build_level.py`.
//!
//!     cargo run --bin layout

use fristad::adapters::data::{world_data, FileStore};
use fristad::domain::world::crypt::level::CryptContents;
use fristad::ports::LayoutStore;

fn main() {
    let data = world_data();
    let layout = data.layout.expect("assets/layout/town.json finnes ikke");
    let crypt: Vec<(u32, CryptContents)> = data
        .levels
        .into_iter()
        .map(|(n, f)| {
            (
                n,
                CryptContents {
                    marks: f.marks,
                    lights: f.lights,
                },
            )
        })
        .collect();
    FileStore(env!("CARGO_MANIFEST_DIR").into())
        .save(&layout, &data.contents, &crypt)
        .expect("kunne ikke skrive");
}
