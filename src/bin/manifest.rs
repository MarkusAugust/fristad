//! Skriver manifestet over modellene som hentes etter start i nettleseren
//! (plan del 10a, `delivery::lazy`): mål, kollidere og skjelett for hver,
//! lest fra glb-filene, så verden kan bygges før filene er kommet.
//! `tools/build-web.sh` og `tools/pull-dist.sh` legger det i
//! `models/manifest.json`.
//!
//!     cargo run --release --bin manifest -- dist/models/manifest.json

fn main() {
    let out = std::env::args().nth(1).expect("bruk: manifest UT.json");
    let manifest = fristad::adapters::delivery::manifest(&fristad::adapters::gltf::Embedded);
    let json = serde_json::to_string(&manifest).expect("manifestet");
    std::fs::write(&out, &json).expect("kunne ikke skrive manifestet");
    println!("{out}: {} modeller, {} kB", manifest.len(), json.len() / 1024);
}
