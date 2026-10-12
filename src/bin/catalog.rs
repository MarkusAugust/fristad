//! Skriver det katalogen trenger å vite om verden (`docs/katalog.md`,
//! `python3 -m tools.qa catalog`): hver modell med navnet byen bruker,
//! fila, hvor mange ganger den står ute og hvor mye plass som er satt av,
//! trekantene, og modellene som mangler eller ventes.
//!
//!     cargo run --release --bin catalog > target/qa/world.json

use fristad::adapters::gltf::{model_files, Embedded};
use fristad::application::prepare;
use fristad::domain::model::PLANNED;
use serde_json::json;

fn main() {
    let p = prepare(&Embedded, 7);
    let models: Vec<_> = model_files()
        .map(|(name, file)| {
            let placed = p.world.placements.iter().find(|q| q.model == *name);
            let tris = p.models.iter().find(|m| m.name == *name).map(|m| m.mesh.indices.len() / 3).unwrap_or(0);
            json!({
                "name": name,
                "file": file,
                "instances": placed.map(|q| q.transforms.len()).unwrap_or(0),
                "reserve": placed.map(|q| q.reserve).unwrap_or(0),
                "triangles": tris,
            })
        })
        .collect();
    let out = json!({
        "models": models,
        "missing": p.world.missing,
        "planned": PLANNED.iter().map(|(n, why)| json!({"name": n, "why": why})).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
