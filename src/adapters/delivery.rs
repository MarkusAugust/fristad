//! Hvordan modellene kommer fram til nettleseren (plan del 10a): bitene til
//! hvert kryptnivå hentes for seg når økta ber om nivået, og teksturene får
//! et tak. Reglene her gjelder filene og stiene, ikke materialene (dem leser
//! `gltf` fra fila).

use super::gltf::{model_files, Caps};

/// Taket i nettleseren (plan del 10a): bare datamaskin er et mål (eieren
/// 19. oktober), så fargen får 2048 og normal- og metallkartene 1024. De
/// store fiendene (Rammeverket er 4 m høyt og bakt i 4096, modalen og
/// tabellen bossene i nivå 2 og 4) står en meter eller to unna og fyller
/// bildet: fargen får 4096. Bildene pakkes fortsatt ut én om gangen, så
/// wasm-haugen holder bare ett av dem.
pub fn browser_caps(name: &str) -> Caps {
    const BIG: [&str; 3] = ["framework", "mutant_modal", "mutant_table"];
    if BIG.contains(&name) {
        Caps { color: 4096, data: 1024 }
    } else {
        Caps { color: 2048, data: 1024 }
    }
}

/// Kryptnivået en modellfil er en bit av (`crypt/crypt_NN_aa.glb`), eller
/// ingen. Bitene hentes for seg i nettleseren (plan del 10a).
pub fn level_of(path: &str) -> Option<u32> {
    let rest = path.strip_prefix("crypt/crypt_")?;
    let (n, tail) = rest.split_at_checked(2)?;
    tail.starts_with('_').then_some(())?;
    n.parse().ok()
}

/// Stien i nettsiden (`models/<sti>.gz`): bitene til hvert kryptnivå ligger
/// i en egen mappe, `crypt/NN/`, som `tools/pack-models.sh` skriver.
pub fn dist_path(path: &str) -> String {
    match level_of(path) {
        Some(n) => format!("crypt/{n:02}/{}", path.trim_start_matches("crypt/")),
        None => path.to_string(),
    }
}

/// Bitene til kryptnivå n: (navn, sti).
pub fn level_files(n: u32) -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    model_files().filter(move |(_, p)| level_of(p) == Some(n))
}
