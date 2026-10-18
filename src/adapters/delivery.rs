//! Hvordan modellene kommer fram til nettleseren (plan del 10a): bitene til
//! hvert kryptnivå hentes for seg når økta ber om nivået, og teksturene får
//! et tak. Reglene her gjelder filene og stiene, ikke materialene (dem leser
//! `gltf` fra fila).

use super::gltf::{model_files, Caps};

/// Taket i nettleseren (plan del 10a): utpakket med mipmaps ga modellene
/// 1,6 GB teksturer, som telefonen ikke tåler. Armene og våpnene står rett
/// foran kameraet og beholder 2048 i fargen; resten får 1024, og normal- og
/// metallkartene halvparten av fargen.
pub fn browser_caps(name: &str) -> Caps {
    const NEAR: [&str; 6] = ["arms", "sword", "axe", "dagger", "shield", "mace"];
    if NEAR.iter().any(|p| name.starts_with(p)) {
        Caps { color: 2048, data: 1024 }
    } else {
        Caps { color: 1024, data: 512 }
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
