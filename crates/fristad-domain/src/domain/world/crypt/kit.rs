//! Byggesettet for nivåredigeringen (`docs/nivaaredigering.md`, del 4):
//! bitene fra Blender (`tools/crypt/kit_cells.py`), beskrevet i manifestet
//! `assets/models/crypt/kit/<tema>.json`, som adapterne leser og gir
//! kjernen. Kjernen ser bare dataene: hvilke flater en bit vil ha, gulvene,
//! kolliderne og det gangbare. Geometrien er modellen (`model`).
//!
//! Bitenes rom (akser som i spillet, x mot øst, y opp, z mot sør):
//! - **celle**: midten av cella på gulvet; sidene i rekkefølgen N (−z),
//!   Ø (+x), S (+z), V (−x).
//! - **halvkant**: midt på kantlinja; muren går 0,4 m inn i cella mot +z.
//! - **kvarthjørne**: i hjørnet; fyller ruta 0,4 × 0,4 m mot +x og +z.
//! - **søyle**: i hjørnet der fire celler møtes.

use super::level::ColliderDef;
use serde::{Deserialize, Serialize};

/// Alle bitene nivåene kan bygges av (alle temaene slått sammen).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct KitDef {
    #[serde(default)]
    pub pieces: Vec<KitPiece>,
    /// Modulene eieren setter sammen (`assets/levels/moduler/<tema>/`,
    /// `docs/nivaaredigering.md` del 21).
    #[serde(default)]
    pub modules: Vec<super::doc::ModuleDef>,
    /// Steinene og standardsteinen per tema (`assets/levels/steiner.json`).
    #[serde(default)]
    pub stones: super::stone::StoneFile,
}

/// Ett manifest (`<tema>.json`).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct KitFile {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub pieces: Vec<KitPiece>,
}

impl KitDef {
    /// Legger til et manifest. En sammensatt bit (`footprint`) får i tillegg
    /// én delbit per celle (`<id>#<k>`, `part_of`), som kompilatoren setter ut
    /// i stedet for den (`compose::expand`): midtcella bærer modellen,
    /// kolliderne og det gangbare, de andre bare flatene og volumet.
    pub fn add(&mut self, f: KitFile) {
        for p in f.pieces {
            for (k, fc) in p.footprint.iter().enumerate() {
                let center = fc.at == [0, 0];
                self.pieces.push(KitPiece {
                    id: format!("{}#{k}", p.id),
                    model: if center { p.model.clone() } else { String::new() },
                    kind: PieceKind::Cell,
                    theme: p.theme.clone(),
                    v: p.v,
                    faces: fc.faces.clone(),
                    floors: fc.floors.unwrap_or(p.floors),
                    clear: p.clear,
                    bottom: p.bottom,
                    top: p.top,
                    walk: if center { p.walk.clone() } else { vec![] },
                    mounts: if center { p.mounts.clone() } else { vec![] },
                    solids: if center { p.solids.clone() } else { vec![] },
                    mirror: p.mirror,
                    part_of: Some(p.id.clone()),
                    ..Default::default()
                });
            }
            self.pieces.push(p);
        }
    }

    pub fn module(&self, id: &str) -> Option<&super::doc::ModuleDef> {
        self.modules.iter().find(|m| m.id == id)
    }

    pub fn piece(&self, id: &str) -> Option<&KitPiece> {
        self.pieces.iter().find(|p| p.id == id)
    }

    /// Halvkanten for en åpning i et tema (eller i et hvilket som helst tema).
    pub fn edge(&self, theme: &str, opening: &str) -> Option<&KitPiece> {
        let of = |p: &&KitPiece| p.kind == PieceKind::Edge && p.opening.as_deref() == Some(opening);
        self.pieces.iter().filter(of).find(|p| p.theme == theme).or_else(|| self.pieces.iter().find(of))
    }

    /// Kvarthjørnet, søyla, den falske veggen eller fakkelen i et tema.
    pub fn of_kind(&self, theme: &str, kind: PieceKind) -> Option<&KitPiece> {
        self.pieces.iter().filter(|p| p.kind == kind).find(|p| p.theme == theme).or_else(|| self.pieces.iter().find(|p| p.kind == kind))
    }

    /// Kvarthjørnet som passer (`fits`: `inner`, `wall`, `pilaster`,
    /// `outer`), ellers et hvilket som helst kvarthjørne i temaet.
    pub fn corner(&self, theme: &str, class: &str) -> Option<&KitPiece> {
        let fits = |p: &&KitPiece| p.kind == PieceKind::Corner && p.fits.iter().any(|f| f == class);
        self.pieces.iter().filter(fits).find(|p| p.theme == theme).or_else(|| self.pieces.iter().find(fits)).or_else(|| self.of_kind(theme, PieceKind::Corner))
    }

    /// Bitene eieren setter ut eller kompilatoren velger (ikke delbitene til
    /// de sammensatte).
    pub fn listed(&self) -> impl Iterator<Item = &KitPiece> {
        self.pieces.iter().filter(|p| p.part_of.is_none())
    }

    /// Modellene bitene bruker, med variantene.
    pub fn models(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.pieces.iter().flat_map(|p| std::iter::once(p.model.as_str()).chain(p.variants.iter().map(|v| v.model.as_str()))).filter(|m| !m.is_empty()).collect();
        out.sort();
        out.dedup();
        out
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PieceKind {
    #[default]
    Cell,
    Edge,
    Corner,
    Pillar,
    /// Den falske veggen som synker (motoren setter den ut etter merket).
    Leaf,
    /// Pynt i et feste (fakkelen).
    Prop,
}

/// Et utseende av en bit (`docs/nivaaredigering.md`, del 9): velges med
/// nivåets frø og plassen, vektet, eller låst i dokumentet (`variant`).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Variant {
    pub id: String,
    pub model: String,
    #[serde(default = "one")]
    pub weight: f32,
    /// Kolliderne når varianten har en annen form (nisjer), ellers bitens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solids: Option<Vec<ColliderDef>>,
}

fn one() -> f32 {
    1.0
}

/// En celle i en sammensatt bit: plassen fra midtcella (i, j) og flatene.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct FootCell {
    pub at: [i32; 2],
    pub faces: Vec<Face>,
    /// Gulvet ved kantene i denne cella, om det er et annet enn bitens
    /// (galleriet: gangen ut i nord går fra galleriet, 5 m oppe).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floors: Option<[f32; 4]>,
}

/// Hva en side av en celle vil ha.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Face {
    /// `open` (rom), `gang` eller `closed`.
    pub kind: String,
    /// Hvelvet ved kanten: `room` eller `gang` (en gangåpning trenger ingen
    /// overmur på en side som har gangprofilen).
    #[serde(default)]
    pub profile: String,
}

/// En gangbar flate i bitens rom: rektangelet (x0, z0, x1, z1) med høyden
/// `y[0]` ved z0 og `y[1]` ved z1 (skrå for trappene).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Walk {
    pub rect: [f32; 4],
    #[serde(default)]
    pub y: [f32; 2],
}

/// Et feste i biten for innhold som hører til den (trappa ned i nedgangen).
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Mount {
    pub kind: String,
    pub pos: [f32; 3],
    /// Radianer.
    #[serde(default)]
    pub yaw: f32,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct KitPiece {
    /// `grav/romfelt`.
    pub id: String,
    pub model: String,
    #[serde(default)]
    pub kind: PieceKind,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub v: u32,
    // Cellene.
    #[serde(default)]
    pub faces: Vec<Face>,
    #[serde(default)]
    pub floors: [f32; 4],
    /// Fri høyde over det gangbare.
    #[serde(default)]
    pub clear: f32,
    /// Volumet biten bruker i høyden (over gulvet), mot overlapp.
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub walk: Vec<Walk>,
    #[serde(default)]
    pub mounts: Vec<Mount>,
    // Halvkantene.
    #[serde(default)]
    pub opening: Option<String>,
    #[serde(default)]
    pub width: f32,
    #[serde(default)]
    pub height: f32,
    // Søyla.
    #[serde(default)]
    pub radius: f32,
    #[serde(default)]
    pub solids: Vec<ColliderDef>,
    #[serde(default)]
    pub mirror: bool,
    /// Utseendene (tom: bare `model`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<Variant>,
    /// Kvarthjørnene: tilfellene hjørnet passer til.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fits: Vec<String>,
    /// Sammensatte biter: cellene de dekker.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footprint: Vec<FootCell>,
    /// Delbitene: den sammensatte biten de hører til.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_of: Option<String>,
}

impl KitPiece {
    /// Modellen for et utseende: det låste (`variant`), ellers et valgt med
    /// `roll` i [0, 1) etter vektene.
    pub fn model_for(&self, locked: Option<&str>, roll: f32) -> &str {
        self.pick(locked, roll).0
    }

    /// Modellen og kolliderne for et utseende (som `model_for`).
    pub fn pick(&self, locked: Option<&str>, roll: f32) -> (&str, &[ColliderDef]) {
        fn of<'a>(v: &'a Variant, own: &'a [ColliderDef]) -> (&'a str, &'a [ColliderDef]) {
            (v.model.as_str(), v.solids.as_deref().unwrap_or(own))
        }
        if self.variants.is_empty() {
            return (&self.model, &self.solids);
        }
        if let Some(v) = locked.and_then(|id| self.variants.iter().find(|v| v.id == id)) {
            return of(v, &self.solids);
        }
        let total: f32 = self.variants.iter().map(|v| v.weight.max(0.0)).sum();
        let mut at = roll * total;
        for v in &self.variants {
            at -= v.weight.max(0.0);
            if at < 0.0 {
                return of(v, &self.solids);
            }
        }
        self.variants.last().map(|v| of(v, &self.solids)).unwrap_or((&self.model, &self.solids))
    }
}
