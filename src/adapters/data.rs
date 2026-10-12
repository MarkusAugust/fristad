//! Data fra Blender som ikke er modeller, lest og tolket her så kjernen får
//! den ferdig (`world::Data`): det husene har med seg (`<navn>_inside.json`:
//! kollidere, lys, lykter, røyk, småting ute, kjeller, seng, galleri) og
//! kryptnivåene (`crypt_NN.json`); `build.rs` finner filene. Filene er små og
//! bygges inn i binæren, også i nettleseren.

use crate::domain::world::{crypt::level::LevelFile, layout::{Item, Layout}, town::Inside, Data};

// `LEVEL_FILES`: nivåene som finnes (laget av build.rs).
include!(concat!(env!("OUT_DIR"), "/crypt_levels.rs"));

// `INSIDE_FILES`: alle `<navn>_inside.json` i `assets/models/town/` (build.rs).
include!(concat!(env!("OUT_DIR"), "/inside_files.rs"));

// `CONTENT_FILES`: det som står inne i husene (build.rs).
include!(concat!(env!("OUT_DIR"), "/content_files.rs"));

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
    let contents = CONTENT_FILES.iter().map(|(n, json)| (n.to_string(), serde_json::from_str::<Vec<Item>>(json).unwrap_or_else(|e| panic!("assets/layout/inside/{n}.json: {e}")))).collect();
    Data { insides, levels, layout, contents, approved: Some(approved()) }
}

/// Dommen fra godkjenningssiden (`assets/catalog/approvals.json`).
pub const APPROVALS: &str = include_str!("../../assets/catalog/approvals.json");

/// Modellene eieren har godkjent (`assets/catalog/approvals.json`, dommen
/// fra godkjenningssiden).
pub fn approved() -> std::collections::HashSet<String> {
    let json: serde_json::Value = serde_json::from_str(APPROVALS).expect("approvals.json");
    json["models"].as_object().map(|m| m.iter().filter(|(_, v)| v["verdict"] == "godkjent").map(|(k, _)| k.clone()).collect()).unwrap_or_default()
}

/// Tallene rundet til 0,01 mm (heltall uten desimaler), og skala 1 utelatt.
fn round(v: &mut serde_json::Value) {
    use serde_json::Value;
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

/// Det som står i et hus som tekst, slik `assets/layout/inside/<hus>.json`
/// skal se ut: én ting per linje, med de samme rundede tallene som byen.
pub fn contents_json(items: &[Item]) -> String {
    let lines: Vec<String> = items
        .iter()
        .map(|it| {
            let mut v = serde_json::to_value(it).unwrap_or_default();
            round(&mut v);
            format!("  {v}")
        })
        .collect();
    format!("[\n{}\n]\n", lines.join(",\n"))
}

/// Husenes innholdsfiler slik de står i repoet (hus, tekst).
pub fn content_files() -> &'static [(&'static str, &'static str)] {
    CONTENT_FILES
}

/// Filene redigeringen lagrer (sti i repoet, tekst): byen og innholdet i
/// hvert hus, sortert etter sti.
pub fn layout_files(layout: &Layout, contents: &std::collections::HashMap<String, Vec<Item>>) -> Vec<(String, String)> {
    let mut out = vec![("assets/layout/town.json".to_string(), layout_json(layout))];
    let mut houses: Vec<_> = contents.iter().collect();
    houses.sort_by(|a, b| a.0.cmp(b.0));
    out.extend(houses.into_iter().map(|(h, items)| (format!("assets/layout/inside/{h}.json"), contents_json(items))));
    out
}

/// Byen som tekst, slik `assets/layout/town.json` skal se ut: én ting og én
/// innbygger per linje, tallene rundet til 0,01 mm, og skala 1 utelatt.
/// Redigeringsmodusen lagrer med denne, så endringene blir små i git.
pub fn layout_json(layout: &Layout) -> String {
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

/// Kategoriene i katalogen i redigeringen, i den rekkefølgen de vises.
pub const CATEGORIES: [&str; 6] = ["Hus", "Gjenstander", "Møbler", "Gjerder", "Mur og porter", "Krypten"];

/// Kategorien til en modell i katalogen (`path` er stien under
/// `assets/models/`, `buildings` modellene som står som hus i byen), eller
/// `None` for det som ikke kan settes ut alene: delene spillet beveger
/// (lokket, vinda, portalen, spaken), platene i galleriet, innsidene av
/// husene og bitene av kryptnivåene.
pub fn category(name: &str, path: &str, buildings: &std::collections::HashSet<String>) -> Option<&'static str> {
    let moving = ["chest_lid", "well_winch", "well_rope", "well_bucket", "kit_lever_arm", "kit_shortcut_leaf"];
    if name.ends_with("_interior") || name.starts_with("gallery_") || name.starts_with("portal_") || moving.contains(&name) || (name.starts_with("crypt_") && name[6..].starts_with(|c: char| c.is_ascii_digit())) {
        return None;
    }
    // Krypten på kirkegården er et bygg, men står som en vanlig ting i byen.
    Some(if buildings.contains(name) || name == "crypt" {
        "Hus"
    } else if matches!(name, "town_wall" | "wall_tower" | "gatehouse") {
        "Mur og porter"
    } else if name.ends_with("_fence") {
        "Gjerder"
    } else if path.starts_with("furniture/") {
        "Møbler"
    } else if path.starts_with("crypt/") {
        "Krypten"
    } else {
        "Gjenstander"
    })
}

/// Modellene som står som hus i byen (`building` i `assets/layout/town.json`).
pub fn buildings() -> std::collections::HashSet<String> {
    world_data().layout.map(|l| l.items.into_iter().filter(|it| it.building).filter_map(|it| it.model).collect()).unwrap_or_default()
}

/// Lagringen til filene i en mappe (repoet): `cargo run --bin layout`, og
/// redigeringen utenom nettleseren.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileStore(pub std::path::PathBuf);

#[cfg(not(target_arch = "wasm32"))]
impl crate::ports::LayoutStore for FileStore {
    fn save(&mut self, layout: &Layout, contents: &[(String, Vec<Item>)]) -> Result<(), String> {
        let map: std::collections::HashMap<String, Vec<Item>> = contents.iter().cloned().collect();
        for (path, text) in layout_files(layout, &map) {
            std::fs::write(self.0.join(&path), text).map_err(|e| format!("{path}: {e}"))?;
        }
        Ok(())
    }
}
