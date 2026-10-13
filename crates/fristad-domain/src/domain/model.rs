//! En modell med egne teksturer, slik kjernen ser den. Hvor den kommer fra
//! (glTF) er adapterens sak.

use crate::domain::geometry::{Layer, Mesh};

pub struct Image {
    pub width: u32,
    pub height: u32,
    /// RGBA8.
    pub rgba: Vec<u8>,
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
}

/// Hvor en modell står i byen, med alle instansene.
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

/// Et annet utseende på samme modell: kjellerens modulsett bygges med fire
/// roller (`Look::ROLES`: mur, hvelv, kvaderstein og gulv), som hver etasje
/// gir sine egne teksturlag. Tonen (lineær) ganges på rollene, og flammene
/// (materialer som heter `fire…`) får fargen `flame`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub layers: [Layer; 4],
    pub tint: glam::Vec3,
    pub flame: glam::Vec3,
}

impl Look {
    /// Lagene modulsettet er bygd med, som rollene byttes ut fra.
    pub const ROLES: [Layer; 4] = [Layer::Stone, Layer::Ashlar, Layer::Flagstone, Layer::Tiles];
}

/// Et fotoskannet materiale som erstatter et bakt teksturlag.
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
}

/// Modellene systemene venter på: koden for dem står og virker, men
/// modellen er ikke laget (eller er underkjent og slettet) ennå. En modell
/// som brukes, finnes eller står her; en som står her og finnes, skal
/// strykes (testen `alle_modellnavn_finnes_eller_ventes`).
pub const PLANNED: &[(&str, &str)] = &[
    ("mutant_table", "fiende (fase 5)"),
    ("framework", "sluttbossen Rammeverket (fase 5)"),
    ("token", "tokenene fiendene mister (fase 7)"),
    ("key", "nøkkelen eliten mister (fase 7)"),
    ("bolt", "spyttet fra inputen (fase 5)"),
    ("arms", "armene i førsteperson (fase 7)"),
    ("sword", "våpen (fase 7)"),
    ("axe", "våpen (fase 7)"),
    ("dagger", "våpen (fase 7)"),
    ("shield_round", "skjold (fase 7)"),
    ("shield_tower", "skjold (fase 7)"),
    ("sheep_body", "sau (fase 7)"),
    ("sheep_leg", "sau (fase 7)"),
    ("cow_body", "ku (fase 7)"),
    ("cow_leg_front", "ku (fase 7)"),
    ("cow_leg_hind", "ku (fase 7)"),
    ("npc_smith", "figur (fase 5)"),
    ("npc_clerk", "figur (fase 5)"),
    ("npc_apothecary", "figur (fase 5)"),
    ("npc_innkeeper", "figur (fase 5)"),
    ("npc_gorvek", "figur (fase 5)"),
    ("npc_farmer", "figur (fase 5)"),
    ("npc_maid", "figur (fase 5)"),
    ("npc_guard", "figur (fase 5)"),
    ("npc_merchant", "figur (fase 5)"),
    ("npc_fishwife", "figur (fase 5)"),
    ("npc_boy", "figur (fase 5)"),
    ("npc_washerwoman", "figur (fase 5)"),
];
