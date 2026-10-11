//! Data fra Blender som ikke er modeller, lest og tolket her så kjernen får
//! den ferdig (`world::Data`): det husene har med seg (`<navn>_inside.json`:
//! kollidere, lys, lykter, røyk, småting ute, kjeller, seng, galleri) og
//! kryptnivåene (`crypt_NN.json`); `build.rs` finner filene. Filene er små og
//! bygges inn i binæren, også i nettleseren.

use crate::domain::world::{crypt::level::LevelFile, town::Inside, Data};

// `LEVEL_FILES`: nivåene som finnes (laget av build.rs).
include!(concat!(env!("OUT_DIR"), "/crypt_levels.rs"));

// `INSIDE_FILES`: alle `<navn>_inside.json` i `assets/models/town/` (build.rs).
include!(concat!(env!("OUT_DIR"), "/inside_files.rs"));

/// Leser alt. En fil som ikke kan tolkes, meldes og hoppes over (et hus
/// får da en boks rundt seg; krypten slutter ved nivået før).
pub fn world_data() -> Data {
    let insides = INSIDE_FILES
        .iter()
        .filter_map(|(n, json)| match serde_json::from_str::<Inside>(json) {
            Ok(i) => Some((n.to_string(), i)),
            Err(e) => {
                log::warn!("{n}_inside.json: {e}");
                None
            }
        })
        .collect();
    let mut levels = vec![];
    for (n, json) in LEVEL_FILES {
        match serde_json::from_str::<LevelFile>(json) {
            Ok(f) => levels.push((*n, f)),
            Err(e) => {
                log::warn!("Kryptnivå {n}: {e}");
                break;
            }
        }
    }
    Data { insides, levels }
}
