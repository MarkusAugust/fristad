//! Data fra Blender som ikke er modeller, lest og tolket her så kjernen får
//! den ferdig (`world::Data`): det husene har med seg (`<navn>_inside.json`:
//! kollidere, lys, lykter, røyk, småting ute, kjeller, seng, galleri) og
//! kryptnivåene (`crypt_NN.json`); `build.rs` finner filene. Filene er små og
//! bygges inn i binæren, også i nettleseren.

use crate::domain::world::{crypt::level::LevelFile, layout::Layout, town::Inside, Data};

// `LEVEL_FILES`: nivåene som finnes (laget av build.rs).
include!(concat!(env!("OUT_DIR"), "/crypt_levels.rs"));

// `INSIDE_FILES`: alle `<navn>_inside.json` i `assets/models/town/` (build.rs).
include!(concat!(env!("OUT_DIR"), "/inside_files.rs"));

// `LAYOUT_FILE`: byen som data, `assets/layout/town.json` (build.rs).
include!(concat!(env!("OUT_DIR"), "/layout_file.rs"));

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
    // Kan ikke byen leses, stopper spillet: den er hele byen.
    let layout = LAYOUT_FILE.map(|json| serde_json::from_str::<Layout>(json).unwrap_or_else(|e| panic!("assets/layout/town.json: {e}")));
    Data { insides, levels, layout }
}

/// Byen som tekst, slik `assets/layout/town.json` skal se ut: én ting og én
/// innbygger per linje, tallene rundet til 0,01 mm, og skala 1 utelatt.
/// Redigeringsmodusen lagrer med denne, så endringene blir små i git.
pub fn layout_json(layout: &Layout) -> String {
    use serde_json::Value;
    fn round(v: &mut Value) {
        match v {
            Value::Number(n) if n.is_f64() => {
                let r = (n.as_f64().unwrap_or(0.0) * 1e5).round() / 1e5;
                *v = if r == r.trunc() && r.abs() < 1e9 { Value::from(r as i64) } else { Value::from(r) };
            }
            Value::Array(a) => a.iter_mut().for_each(round),
            Value::Object(o) => {
                o.values_mut().for_each(round);
                if o.get("scale") == Some(&serde_json::json!([1, 1, 1])) {
                    o.remove("scale");
                }
            }
            _ => {}
        }
    }
    let mut v = serde_json::to_value(layout).unwrap_or_default();
    round(&mut v);
    let Some(o) = v.as_object() else { return String::new() };
    let mut out = String::from("{\n");
    for (i, (k, val)) in o.iter().enumerate() {
        let sep = if i + 1 < o.len() { "," } else { "" };
        match val.as_array().filter(|_| k == "items" || k == "npcs") {
            Some(list) => {
                out += &format!("  {k:?}: [\n");
                for (j, x) in list.iter().enumerate() {
                    out += &format!("    {}{}\n", x, if j + 1 < list.len() { "," } else { "" });
                }
                out += &format!("  ]{sep}\n");
            }
            None => out += &format!("  {k:?}: {val}{sep}\n"),
        }
    }
    out + "}\n"
}

/// Byen slik den står i `assets/layout/town.json`.
pub fn layout_file() -> Option<&'static str> {
    LAYOUT_FILE
}
