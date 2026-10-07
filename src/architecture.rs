//! Arkitekturen som test: hvert lag får bare bruke lagene innenfor seg og
//! sine egne avhengigheter. Testen leser kildefilene og leter etter stier
//! og crates som ikke hører hjemme.
//!
//! - `domain` får ikke bruke `ports`, `application` eller `adapters`, og
//!   ingen motor, nettleser, filsystem eller miljø.
//! - `ports` får bare bruke `domain`.
//! - `application` får bruke `domain` og `ports`, men ingen adaptere eller
//!   motorer.
//! - `adapters` får bruke alt.

use std::path::Path;

/// Det ingen av de indre lagene får nevne.
const ENGINES: &[&str] = &["wgpu", "winit", "rapier3d", "gltf::", "wasm_bindgen", "web_sys", "js_sys", "image::", "pollster", "fontdue", "png::", "std::fs", "std::env", "std::time::Instant"];

fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Linjene som er kode (ikke kommentarer), med linjenummer.
fn code(path: &Path) -> Vec<(usize, String)> {
    let src = std::fs::read_to_string(path).unwrap();
    src.lines().enumerate().filter(|(_, l)| !l.trim_start().starts_with("//")).map(|(i, l)| (i + 1, l.to_string())).collect()
}

fn check(layer: &str, forbidden: &[&str]) -> Vec<String> {
    let mut all = vec![];
    files(&Path::new("src").join(layer), &mut all);
    let mut problems = vec![];
    for f in all {
        for (n, line) in code(&f) {
            for bad in forbidden {
                if line.contains(bad) {
                    problems.push(format!("{}:{n} bruker «{bad}»: {}", f.display(), line.trim()));
                }
            }
        }
    }
    problems
}

#[test]
fn lagene_holder_seg_innenfor() {
    let mut problems = vec![];
    let inner: Vec<&str> = ["crate::ports", "crate::application", "crate::adapters"].into_iter().chain(ENGINES.iter().copied()).collect();
    problems.extend(check("domain", &inner));
    let ports: Vec<&str> = ["crate::application", "crate::adapters"].into_iter().chain(ENGINES.iter().copied()).collect();
    problems.extend(check("ports", &ports));
    let app: Vec<&str> = ["crate::adapters"].into_iter().chain(ENGINES.iter().copied()).collect();
    problems.extend(check("application", &app));
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
