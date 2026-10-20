//! Steinen i murene (`docs/nivaaredigering.md` del 22, eieren 19. oktober:
//! «det må være mulig å skifte type tekstur på mur også»). En modul er en
//! form, og steinen er et valg: bitene i byggesettet er laget med temaets
//! lag for gulvet, muren og den hogde steinen (`layer_crypt_floor`,
//! `layer_crypt_wall`, `layer_crypt_dressed`: søyler, ribber, buer, karmer,
//! kapitéler, konsoller, sokkel og terskler), og hver instans får sine egne
//! lag for gulvet, muren, hvelvet og den hogde steinen (`model::Look`), så
//! ingen nye modeller trengs per tema.
//!
//! Steinene står i `assets/levels/steiner.json` (`StoneFile`): navn → lag
//! for gulv, mur og hvelv, og standardsteinen per tema. Nivådokumentet
//! velger med `stone` på nivået, rommet, modulen, cella og kanten (`StoneRef`);
//! det nærmeste vinner, flate for flate.

use crate::domain::geometry::Layer;
use crate::domain::model::Look;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// En stein: lagene for gulvet, muren, hvelvet og den hogde steinen
/// (`trim`; navnene i `Layer::name`).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct StoneDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub floor: String,
    pub wall: String,
    pub vault: String,
    pub trim: String,
    /// Murvarianten steinen vil ha (`som`: plastsømmene i kjernen), om
    /// halvkanten har den; ellers velger frøet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_variant: Option<String>,
    /// Cellevarianten steinen vil ha (`tommer`: tømmeret i gruvas sjakt),
    /// om cella har den; murene i cella tar varianten med samme navn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_variant: Option<String>,
}

/// `assets/levels/steiner.json`.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct StoneFile {
    #[serde(default)]
    pub stones: Vec<StoneDef>,
    /// Temaet (`grav`, `kloakk` …) → standardsteinen. Byggesettet for et
    /// tema er laget med temaets standardstein.
    #[serde(default)]
    pub themes: BTreeMap<String, String>,
    /// Lyset i rom med mørk stein (lag → faktor): faklene og lysene i et
    /// rom lyser så mye sterkere som murlaget sier, så tegl, berg og kvader
    /// er lesbare uten at gravmuren blir lysere (eieren 20. oktober).
    #[serde(default)]
    pub light: BTreeMap<String, f32>,
}

/// Eierens valg: en stein ved navn (`"tegl"`: gulv, mur og hvelv), eller
/// flatene hver for seg (`{"wall": "sewer_brick"}`), hver et lagnavn eller
/// en stein (flaten fra den). Det som er utelatt, arves.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum StoneRef {
    Named(String),
    Parts(StoneParts),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct StoneParts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim: Option<String>,
}

/// Lagene en celle eller en halvkant får (navnene).
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Surfaces {
    pub floor: String,
    pub wall: String,
    pub vault: String,
    pub trim: String,
    /// Murvarianten (`StoneDef::wall_variant`).
    pub wall_variant: Option<String>,
    /// Cellevarianten (`StoneDef::cell_variant`).
    pub cell_variant: Option<String>,
}

impl Surfaces {
    fn of(s: &StoneDef) -> Self {
        Surfaces { floor: s.floor.clone(), wall: s.wall.clone(), vault: s.vault.clone(), trim: s.trim.clone(), wall_variant: s.wall_variant.clone(), cell_variant: s.cell_variant.clone() }
    }

    fn layers(&self) -> [&String; 4] {
        [&self.floor, &self.wall, &self.vault, &self.trim]
    }
}

impl StoneFile {
    pub fn stone(&self, id: &str) -> Option<&StoneDef> {
        self.stones.iter().find(|s| s.id == id)
    }

    /// Lysfaktoren for flatene (murlaget; 1 for det som ikke står i `light`).
    pub fn light_of(&self, s: &Surfaces) -> f32 {
        self.light.get(&s.wall).copied().unwrap_or(1.0)
    }

    /// Standardsteinen for et tema (lagene byggesettet for temaet er laget med).
    pub fn base(&self, theme: &str) -> Option<Surfaces> {
        Some(Surfaces::of(self.stone(self.themes.get(theme)?)?))
    }

    /// Ett lag: et lagnavn, eller flaten fra en stein.
    fn layer(&self, name: &str, pick: fn(&StoneDef) -> &String) -> Result<String, String> {
        if let Some(s) = self.stone(name) {
            return Ok(pick(s).clone());
        }
        match Layer::from_name(name) {
            Some(_) => Ok(name.to_string()),
            None => Err(format!("ukjent stein eller lag «{name}»")),
        }
    }

    /// Legger eierens valg over `s`, flate for flate.
    pub fn apply(&self, s: &mut Surfaces, r: &StoneRef) -> Result<(), String> {
        match r {
            StoneRef::Named(id) => {
                let st = self.stone(id).ok_or_else(|| format!("ukjent stein «{id}»"))?;
                *s = Surfaces::of(st);
            }
            StoneRef::Parts(p) => {
                if let Some(n) = &p.floor {
                    s.floor = self.layer(n, |s| &s.floor)?;
                }
                if let Some(n) = &p.wall {
                    s.wall = self.layer(n, |s| &s.wall)?;
                    s.wall_variant = self.stone(n).and_then(|st| st.wall_variant.clone());
                    s.cell_variant = self.stone(n).and_then(|st| st.cell_variant.clone());
                }
                if let Some(n) = &p.vault {
                    s.vault = self.layer(n, |s| &s.vault)?;
                }
                if let Some(n) = &p.trim {
                    s.trim = self.layer(n, |s| &s.trim)?;
                }
            }
        }
        Ok(())
    }

    /// Steinbyttet for en bit laget i temaet `theme` som skal ha lagene `to`
    /// (ingenting når det er likt det biten er laget med).
    pub fn swap(&self, theme: &str, to: &Surfaces) -> Option<InstanceStone> {
        let base = self.base(theme)?;
        (base.layers() != to.layers()).then(|| InstanceStone { from: [base.floor, base.wall, base.trim], to: [to.floor.clone(), to.wall.clone(), to.vault.clone(), to.trim.clone()] })
    }

    /// Hver stein og hvert tema peker på lag som finnes (feilene).
    pub fn check(&self) -> Vec<String> {
        let mut bad = vec![];
        for s in &self.stones {
            for l in [&s.floor, &s.wall, &s.vault, &s.trim] {
                if Layer::from_name(l).is_none() {
                    bad.push(format!("steinen {}: laget «{l}» finnes ikke", s.id));
                }
            }
        }
        for (t, id) in &self.themes {
            if self.stone(id).is_none() {
                bad.push(format!("temaet {t}: steinen «{id}» finnes ikke"));
            }
        }
        bad
    }
}

/// Steinbyttet på en instans i nivåfila: lagene biten er laget med (gulv,
/// mur, hogd stein) og lagene den får (gulv, mur, hvelv, hogd stein).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct InstanceStone {
    pub from: [String; 3],
    pub to: [String; 4],
}

impl InstanceStone {
    /// Utseendet for rendereren (ingenting om et lag er ukjent).
    pub fn look(&self) -> Option<Look> {
        let l = |n: &String| Layer::from_name(n);
        Some(Look { floor: (l(&self.from[0])?, l(&self.to[0])?), wall: (l(&self.from[1])?, l(&self.to[1])?), vault: l(&self.to[2])?, trim: (l(&self.from[2])?, l(&self.to[3])?) })
    }
}
