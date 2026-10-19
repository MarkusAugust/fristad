//! Data fra Blender som ikke er modeller, lest og tolket her så kjernen får
//! den ferdig (`world::Data`): det husene har med seg (`<navn>_inside.json`:
//! kollidere, lys, lykter, røyk, småting ute, kjeller, seng, galleri) og
//! kryptnivåene (`crypt_NN.json`); `build.rs` finner filene. Filene er små og
//! bygges inn i binæren, også i nettleseren.

use crate::domain::world::{
    crypt::doc::LevelDoc,
    crypt::kit::{KitDef, KitFile},
    crypt::level::{CryptContents, LevelFile},
    layout::{Item, Layout},
    town::Inside,
    Data,
};

// `LEVEL_FILES`: nivåene som finnes (laget av build.rs).
include!(concat!(env!("OUT_DIR"), "/crypt_levels.rs"));

// `INSIDE_FILES`: alle `<navn>_inside.json` i `assets/models/town/` (build.rs).
include!(concat!(env!("OUT_DIR"), "/inside_files.rs"));

// `CONTENT_FILES`: det som står inne i husene (build.rs).
include!(concat!(env!("OUT_DIR"), "/content_files.rs"));

// `CRYPT_CONTENT_FILES`: det som står i kryptnivåene (build.rs).
include!(concat!(env!("OUT_DIR"), "/crypt_content_files.rs"));

// `LAYOUT_FILE`: byen som data, `assets/layout/town.json` (build.rs).
include!(concat!(env!("OUT_DIR"), "/layout_file.rs"));

// `KIT_FILES` og `LEVEL_DOC_FILES`: byggesettet og nivådokumentene (build.rs).
include!(concat!(env!("OUT_DIR"), "/level_docs.rs"));

/// Byggesettet for nivåredigeringen: alle manifestene slått sammen.
pub fn kit() -> KitDef {
    let mut kit = KitDef::default();
    for (theme, json) in KIT_FILES {
        match serde_json::from_str::<KitFile>(json) {
            Ok(f) => kit.add(f),
            Err(e) => log::warn!("assets/models/crypt/kit/{theme}.json: {e}"),
        }
    }
    for (name, json) in MODULE_FILES {
        match serde_json::from_str::<crate::domain::world::crypt::doc::ModuleDef>(json) {
            Ok(mut m) => {
                if m.id.is_empty() {
                    m.id = name.to_string();
                }
                kit.modules.push(m);
            }
            Err(e) => log::warn!("assets/levels/moduler/{name}.json: {e}"),
        }
    }
    kit
}

/// Navnene på nivådokumentene som finnes (`01`, `test`), `brett`
/// (prøvebrettet, laget av byggesettet) og `moduler` (alle modulene).
pub fn level_doc_names() -> Vec<String> {
    let mut out: Vec<String> = LEVEL_DOC_FILES.iter().map(|(n, _)| n.to_string()).collect();
    out.push("brett".into());
    out.push("moduler".into());
    out
}

/// Nivådokumentet `assets/levels/nivaa_<navn>.json`, prøvebrettet
/// (`brett`: hver bit mot hver bit i alle dreininger) eller en modul alene
/// (`modul:<tema>/<navn>`).
pub fn level_doc(name: &str) -> Option<LevelDoc> {
    if name == "brett" {
        return Some(crate::domain::world::crypt::compose::board::board(&kit()));
    }
    // `moduler`: alle modulene side om side.
    if name == "moduler" {
        return Some(crate::domain::world::crypt::compose::modules::gallery(&kit()));
    }
    // `modul:grav/kapell`: modulen alene, med fakler og lys (bildene til eieren).
    if let Some(id) = name.strip_prefix("modul:") {
        return crate::domain::world::crypt::compose::modules::showcase(&kit(), id);
    }
    let (_, json) = LEVEL_DOC_FILES.iter().find(|(n, _)| *n == name)?;
    match serde_json::from_str::<LevelDoc>(json) {
        Ok(d) => Some(d),
        Err(e) => {
            log::warn!("assets/levels/nivaa_{name}.json: {e}");
            None
        }
    }
}

/// Dataene med ett nivådokument som eneste kryptnivå (nivå 1): prøvenivået
/// i `snapshot --level-doc`, testene og `?nivaa=` i nettleseren.
pub fn world_data_with_doc(name: &str) -> Data {
    let mut data = world_data();
    if let Some(doc) = level_doc(name) {
        data.levels.clear();
        data.docs = vec![(1, doc)];
    } else {
        log::warn!("Fant ikke nivådokumentet «{name}»");
    }
    data
}

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
            Ok(mut f) => {
                // Merkene og lysene fra `assets/layout/crypt_NN.json` (det
                // redigeringen endrer) går foran dem i nivåfila.
                if let Some((_, text)) = CRYPT_CONTENT_FILES.iter().find(|(k, _)| k == n) {
                    let c: CryptContents = serde_json::from_str(text).unwrap_or_else(|e| panic!("assets/layout/crypt_{n:02}.json: {e}"));
                    f.marks = c.marks;
                    f.lights = c.lights;
                }
                levels.push((*n, f))
            }
            Err(e) => {
                log::warn!("Kryptnivå {n}: {e}");
                break;
            }
        }
    }
    // Kan ikke byen leses, stopper spillet: den er hele byen.
    let layout = LAYOUT_FILE.map(|json| serde_json::from_str::<Layout>(json).unwrap_or_else(|e| panic!("assets/layout/town.json: {e}")));
    let contents = CONTENT_FILES.iter().map(|(n, json)| (n.to_string(), serde_json::from_str::<Vec<Item>>(json).unwrap_or_else(|e| panic!("assets/layout/inside/{n}.json: {e}")))).collect();
    // Nivådokumentene som er merket «klar», erstatter de gamle nivåene med
    // samme nummer (`Data::compose_levels`).
    let docs = LEVEL_DOC_FILES
        .iter()
        .filter_map(|(name, _)| {
            let d = level_doc(name)?;
            let n = d.level.number()?;
            d.ready().then_some((n, d))
        })
        .collect();
    Data { insides, levels, layout, contents, approved: Some(approved()), kit: kit(), docs }
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

/// Kryptnivåenes innholdsfiler slik de står i repoet (nivå, tekst).
pub fn crypt_content_files() -> &'static [(u32, &'static str)] {
    CRYPT_CONTENT_FILES
}

/// Det som står i et kryptnivå som tekst, slik `assets/layout/crypt_NN.json`
/// skal se ut: ett merke og ett lys per linje, rundede tall.
pub fn crypt_json(c: &CryptContents) -> String {
    let line = |v: serde_json::Value| {
        let mut v = v;
        round(&mut v);
        format!("    {v}")
    };
    let marks: Vec<String> = c.marks.iter().map(|m| line(serde_json::to_value(m).unwrap_or_default())).collect();
    let lights: Vec<String> = c.lights.iter().map(|l| line(serde_json::to_value(l).unwrap_or_default())).collect();
    format!("{{\n  \"marks\": [\n{}\n  ],\n  \"lights\": [\n{}\n  ]\n}}\n", marks.join(",\n"), lights.join(",\n"))
}

/// Husenes innholdsfiler slik de står i repoet (hus, tekst).
pub fn content_files() -> &'static [(&'static str, &'static str)] {
    CONTENT_FILES
}

/// Filene redigeringen lagrer (sti i repoet, tekst): byen og innholdet i
/// hvert hus, sortert etter sti.
pub fn layout_files(layout: &Layout, contents: &std::collections::HashMap<String, Vec<Item>>, crypt: &std::collections::BTreeMap<u32, CryptContents>) -> Vec<(String, String)> {
    let mut out = vec![("assets/layout/town.json".to_string(), layout_json(layout))];
    let mut houses: Vec<_> = contents.iter().collect();
    houses.sort_by(|a, b| a.0.cmp(b.0));
    out.extend(houses.into_iter().map(|(h, items)| (format!("assets/layout/inside/{h}.json"), contents_json(items))));
    out.extend(crypt.iter().map(|(n, c)| (format!("assets/layout/crypt_{n:02}.json"), crypt_json(c))));
    out
}

/// Byen som tekst, slik `assets/layout/town.json` skal se ut: én ting og én
/// innbygger per linje, tallene rundet til 0,01 mm, og skala 1 utelatt.
/// Redigeringsmodusen lagrer med denne, så endringene blir små i git.
pub fn layout_json(layout: &Layout) -> String {
    let mut v = serde_json::to_value(layout).unwrap_or_default();
    round(&mut v);
    let Some(o) = v.as_object() else {
        return String::new();
    };
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
    if name.ends_with("_interior")
        || name.starts_with("gallery_")
        || name.starts_with("portal_")
        || moving.contains(&name)
        || (name.starts_with("crypt_") && name[6..].starts_with(|c: char| c.is_ascii_digit()))
    {
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
    fn save(&mut self, layout: &Layout, contents: &[(String, Vec<Item>)], crypt: &[(u32, CryptContents)]) -> Result<(), String> {
        let map: std::collections::HashMap<String, Vec<Item>> = contents.iter().cloned().collect();
        for (path, text) in layout_files(layout, &map, &crypt.iter().cloned().collect()) {
            std::fs::write(self.0.join(&path), text).map_err(|e| format!("{path}: {e}"))?;
        }
        Ok(())
    }
}
