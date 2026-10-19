//! En modell med egne teksturer, slik kjernen ser den. Hvor den kommer fra
//! (glTF) er adapterens sak.

use crate::domain::geometry::{Layer, Mesh};

#[derive(Clone)]
pub struct Image {
    /// Målene bildet har når det er pakket ut (etter `encoded` er skalert ned).
    pub width: u32,
    pub height: u32,
    /// RGBA8, eller tom når bildet ennå er kodet.
    pub rgba: Vec<u8>,
    /// Bildefila (png eller jpeg) når den pakkes ut først når rendereren
    /// laster den opp, én om gangen, så alle modellenes teksturer aldri står
    /// utpakket i minnet samtidig. Det er adapteren som pakker ut og skalerer
    /// til `width` × `height`. Delt, så rendereren kan beholde fila til
    /// den oppgraderer bildet uten å kopiere den.
    pub encoded: std::sync::Arc<[u8]>,
    /// Målene bildet får når rendereren oppgraderer det i bakgrunnen
    /// (nettleseren laster først et lavt tak, `delivery::stage`), eller
    /// ingen: `width` × `height` er det endelige.
    pub later: Option<(u32, u32)>,
}

impl Image {
    pub fn rgba(width: u32, height: u32, rgba: Vec<u8>) -> Self {
        Self { width, height, rgba, encoded: Vec::new().into(), later: None }
    }
}

pub struct Model {
    pub name: String,
    pub mesh: Mesh,
    pub albedo: Option<Image>,
    pub normal: Option<Image>,
    /// Ruhet i G, metall i B, som i glTF.
    pub metal_rough: Option<Image>,
    pub emissive: Option<Image>,
    pub roughness: f32,
    pub metallic: f32,
    /// Cellestørrelsen for det grove detaljnivået (meter); 0 lar rendereren
    /// velge etter modellens størrelse.
    pub far_cell: f32,
    /// Et ferdig grovt detaljnivå (bladverk som tynnede kort); ellers
    /// lager rendereren det med `Mesh::clustered`.
    pub far_mesh: Option<Mesh>,
    /// Skjelettet og klippene, for modeller som er skinnet (figurene).
    pub rig: Option<crate::domain::anim::Rig>,
    /// Kolliderne som følger formen, i modellens rom (`extras.solids`,
    /// `tools/crypt/solids.py`); tom når modellen ikke har dem.
    pub solids: Vec<crate::domain::world::Shape>,
}

/// Hvor en modell står i byen, med alle instansene.
#[derive(Clone)]
pub struct Placement {
    pub model: &'static str,
    pub transforms: Vec<glam::Affine3A>,
    /// Lakkfargen per instans (lineær RGB), for flater med flagget PAINT.
    pub tints: Vec<glam::Vec3>,
    /// Et annet utseende per instans (kjelleretasjene), eller ingen.
    pub looks: Vec<Option<Look>>,
    /// Plass til instanser spillet poserer per bilde (`Frame.instances`).
    pub reserve: usize,
    /// Står inne i et hus (møblene): tegnes bare nær og kaster ikke
    /// solskygge (huset skygger for dem).
    pub indoor: bool,
}

/// Steinbyttet per instans (`crypt::stone`, `docs/nivaaredigering.md` del
/// 22): bitene er laget med temaets lag for gulvet og muren, og instansen
/// bytter dem (og den hogde steinen). Flatene av murlaget som vender nedover i modellens rom
/// (normalen under `VAULT_NY`: hvelvene, buene og overliggerne) får
/// hvelvlaget. Rendereren gjør det samme i vertexshaderen (`look` i
/// `world.wgsl`), og sammenslåingen av bitene med `layer`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// (laget biten er laget med, laget den får)
    pub floor: (Layer, Layer),
    pub wall: (Layer, Layer),
    pub vault: Layer,
    /// Den hogde steinen (søyler, ribber, buer, karmer, sokkel): egen rolle,
    /// aldri hvelv.
    pub trim: (Layer, Layer),
}

impl Look {
    pub const VAULT_NY: f32 = -0.3;

    /// Laget en flate med laget `layer` og normalen `normal_y` (modellens rom) får.
    pub fn layer(&self, layer: u8, normal_y: f32) -> u8 {
        if layer == self.floor.0 as u8 {
            self.floor.1 as u8
        } else if layer == self.trim.0 as u8 {
            self.trim.1 as u8
        } else if layer == self.wall.0 as u8 {
            if normal_y < Self::VAULT_NY {
                self.vault as u8
            } else {
                self.wall.1 as u8
            }
        } else {
            layer
        }
    }
}

/// Bildene til et teksturlag (eierens malte, avledet med `tools/textures/derive.py`), som erstatter det bakte laget.
pub struct LayerImages {
    pub diffuse: Image,
    pub normal: Option<Image>,
    pub rough: Option<Image>,
    pub height: Option<Image>,
    pub ao: Option<Image>,
    /// Hvor mange meter én flis dekker.
    pub size_m: f32,
}

/// Det byen trenger å vite om en modell før den bygges: at den finnes, og
/// hvor stor den er (føttene i y = 0, sentrert i x og z).
#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub name: String,
    pub min: glam::Vec3,
    pub max: glam::Vec3,
    /// Kolliderne som følger formen (`Model.solids`).
    pub solids: Vec<crate::domain::world::Shape>,
}

/// Modellene systemene venter på: koden for dem står og virker, men
/// modellen er ikke laget (eller er underkjent og slettet) ennå. En modell
/// som brukes, finnes eller står her; en som står her og finnes, skal
/// strykes (testen `alle_modellnavn_finnes_eller_ventes`).
pub const PLANNED: &[(&str, &str)] = &[
    ("key", "nøkkelen eliten mister (fase 7)"),
    ("sheep_body", "sau (fase 7)"),
    ("sheep_leg", "sau (fase 7)"),
    ("cow_body", "ku (fase 7)"),
    ("cow_leg_front", "ku (fase 7)"),
    ("cow_leg_hind", "ku (fase 7)"),
];
