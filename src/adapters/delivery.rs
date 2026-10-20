//! Hvordan modellene kommer fram til nettleseren (plan del 10a): bitene til
//! hvert kryptnivå hentes for seg når økta ber om nivået, og teksturene får
//! et tak. Reglene her gjelder filene og stiene, ikke materialene (dem leser
//! `gltf` fra fila).

use super::gltf::{capped, model_files, Caps, Files};
use crate::domain::model::{Image, Later, Model, ModelInfo};
use std::collections::BTreeMap;

/// Taket i nettleseren (plan del 10a): bare datamaskin er et mål (eieren
/// 19. oktober), så fargen får 2048 og normal- og metallkartene 1024. De
/// store fiendene (Rammeverket er 4 m høyt og bakt i 4096, modalen og
/// tabellen bossene i nivå 2 og 4, Navbaren og Dropdownen 5 m i nivå 8 og 6) står en meter eller to unna og fyller
/// bildet: fargen får 4096. Bildene pakkes fortsatt ut én om gangen, så
/// wasm-haugen holder bare ett av dem.
pub fn browser_caps(name: &str) -> Caps {
    const BIG: [&str; 5] = ["framework", "mutant_modal", "mutant_table", "navbar", "dropdown"];
    if BIG.contains(&name) {
        Caps { color: 4096, data: 1024 }
    } else {
        Caps { color: 2048, data: 1024 }
    }
}

/// Det lave taket teksturene lastes opp med først i nettleseren (lat
/// lasting): byen kommer raskt opp, og rendereren oppgraderer dem til
/// `browser_caps` i bakgrunnen, de nærmeste først (`upgrade_pick`).
pub const LOW: Caps = Caps { color: 512, data: 256 };

/// Bilder over denne siden (Rammeverkets 4096) oppgraderes bare når
/// modellen er innen `BIG_NEAR` meter fra kameraet.
pub const BIG_SIDE: u32 = 2048;
pub const BIG_NEAR: f32 = 30.0;

/// Tiden oppgraderingen får per bilde i snitt (ms): ett bilde pakkes ut
/// av gangen, og et som tok lenger, betales av de neste bildene.
pub const UPGRADE_MS: f64 = 4.0;

/// Gir modellens bilder det lave taket først og husker målene de skal
/// oppgraderes til (`Image.later`).
pub fn stage(m: &mut Model, low: Caps) {
    let one = |img: &mut Option<Image>, cap: u32| {
        let Some(i) = img.as_mut() else { return };
        let (w, h) = capped(i.width, i.height, cap);
        if (w, h) != (i.width, i.height) && !i.encoded.is_empty() {
            i.later = Some((i.width, i.height));
            (i.width, i.height) = (w, h);
        }
    };
    one(&mut m.albedo, low.color);
    one(&mut m.normal, low.data);
    one(&mut m.metal_rough, low.data);
    one(&mut m.emissive, low.data);
}

/// Hvilken modell som oppgraderes nå: (avstand fra kameraet til nærmeste
/// instans, største side det skal opp til) per kandidat; den nærmeste
/// først, og de store bare innen `BIG_NEAR`. Modeller uten instanser har
/// uendelig avstand og kommer sist.
pub fn upgrade_pick(cands: impl Iterator<Item = (usize, f32, u32)>) -> Option<usize> {
    cands.filter(|(_, d, side)| *side <= BIG_SIDE || *d <= BIG_NEAR).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(k, _, _)| k)
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

/// Om en modellfil hentes etter start i nettleseren (plan del 10a): det
/// som ikke trengs til første bilde i byen. Innsiden (`…_interior`),
/// møblene, galleriplatene, fiendene, armsettene og våpnene (det du har på
/// hentes straks), og kryptens modeller (byggesettet og det fiendene
/// mister). Bitene til kryptnivåene hentes for seg (`level_of`).
pub fn lazy(name: &str, path: &str) -> bool {
    level_of(path).is_none()
        && (name.ends_with("_interior")
            || name.starts_with("gallery_")
            || ["furniture/", "mobs/", "arms/", "crypt/"].iter().any(|d| path.starts_with(d))
            || ["token", "bolt", "key"].contains(&name))
}

/// Manifestet: det monteringen trenger om hver modell som hentes senere
/// (mål, kollidere og skjelett), laget av glb-filene ved pakkingen
/// (`cargo run --release --bin manifest`, `models/manifest.json`).
pub type Manifest = BTreeMap<String, Later>;

/// Det monteringen trenger om en lastet modell.
pub fn later_of(m: &Model) -> Later {
    let (min, max) = m.mesh.bounds();
    Later { info: ModelInfo { name: m.name.clone(), min, max, solids: m.solids.clone() }, rig: m.rig.clone() }
}

/// Manifestet for modellene `lazy` sier hentes senere, lest fra `source`.
pub fn manifest(source: &dyn crate::ports::ModelSource) -> Manifest {
    model_files().filter(|(n, p)| lazy(n, p)).filter_map(|(n, _)| source.load(n)).map(|m| (m.name.clone(), later_of(&m))).collect()
}

/// Modellfilene nettleseren hentet ved start, og manifestet for dem som
/// kommer senere: monteringen setter dem ut etter manifestet, og `App`
/// ber om dem (`Event::FetchModels`).
pub struct WithManifest<'a>(pub &'a Files, pub Manifest);

impl crate::ports::ModelSource for WithManifest<'_> {
    fn load(&self, name: &str) -> Option<Model> {
        self.0.load(name)
    }
    fn names(&self) -> Vec<String> {
        self.0.names()
    }
    fn world_data(&self) -> crate::domain::world::Data {
        self.0.world_data()
    }
    fn deferred(&self) -> Vec<Later> {
        let have = self.0.names();
        let mut out = self.0.deferred();
        out.extend(self.1.values().filter(|l| !have.contains(&l.info.name)).cloned());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Den nærmeste først, de uten instanser sist, og de store (over
    /// `BIG_SIDE`) bare innen `BIG_NEAR`.
    #[test]
    fn oppgraderingen_tar_det_naermeste_forst() {
        assert_eq!(upgrade_pick([(0, 50.0, 2048), (1, 3.0, 1024), (2, f32::INFINITY, 2048)].into_iter()), Some(1));
        assert_eq!(upgrade_pick([(0, f32::INFINITY, 2048)].into_iter()), Some(0));
        assert_eq!(upgrade_pick([(0, 31.0, 4096), (1, 80.0, 2048)].into_iter()), Some(1), "bossen venter til du er nær");
        assert_eq!(upgrade_pick([(0, 31.0, 4096)].into_iter()), None);
        assert_eq!(upgrade_pick([(0, 12.0, 4096), (1, 5.0, 2048)].into_iter()), Some(1));
        assert_eq!(upgrade_pick([(0, 12.0, 4096), (1, 20.0, 2048)].into_iter()), Some(0));
        assert_eq!(browser_caps("framework").color, 4096);
        assert_eq!(browser_caps("navbar").color, 4096);
        assert_eq!(browser_caps("dropdown").color, 4096);
        assert!(LOW.color < browser_caps("house_a").color && LOW.data < browser_caps("house_a").data);
    }
}
