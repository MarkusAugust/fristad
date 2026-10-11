//! Det rendereren får per bilde: kamera, lys, vær, palett og hvor propsene
//! står. Rene tall, ingen GPU-typer.

use glam::{Affine3A, Mat4, Vec2, Vec3, Vec4};

pub const PALETTE: usize = 64;
pub const MAX_LIGHTS: usize = 48;
/// Pipene som ryker i hvert bilde (de nærmeste kameraet).
pub const MAX_SMOKE: usize = 32;
/// Rommene i kryptnivået som rendereren får (lyset stoppes av murene).
pub const MAX_ROOMS: usize = 64;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Weather {
    /// Hvor våte flatene er, 0..1.
    pub wet: f32,
    /// Snø på bakken og takene, 0..1.
    pub snow_cover: f32,
    /// Regn i lufta, 0..1.
    pub rain: f32,
    /// Snø i lufta, 0..1.
    pub snow: f32,
    /// Skydekke, 0..1.
    pub cloud: f32,
    pub wind: f32,
    pub lightning: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub pos: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub intensity: f32,
    /// Brenner hele døgnet (ildsteder, fakler), ikke bare når lampene er på.
    pub always: bool,
    /// Flakker: 0 rolig, 1 slukker i rykk (lysrør som er i ferd med å ryke).
    pub flicker: f32,
    /// Rommet i krypten lyset står i (nivå, rom), om noe.
    pub room: Option<(u16, u16)>,
}

/// En modellinstans som flytter seg (dyrene): tegnes sammen med de faste
/// instansene av samme modell. Byen må ha satt av plass (`Placement.reserve`).
#[derive(Clone, Copy, Debug)]
pub struct ModelInstance {
    pub model: &'static str,
    pub transform: Affine3A,
    /// Skinnmatrisene i `Frame.skins` (start, antall) for en skinnet figur.
    pub skin: Option<(u32, u32)>,
    /// Blinker hvitt (0..1), rett etter et treff.
    pub flash: f32,
    /// Lakkfargen (lineær RGB) på flatene merket «paint», om modellen har dem.
    pub tint: Option<Vec3>,
}

pub struct Frame {
    pub view: Mat4,
    pub proj: Mat4,
    pub camera: Vec3,
    pub time: f32,
    /// Retning mot sola eller månen.
    pub sun_dir: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f32,
    /// 0 dag, 1 natt.
    pub night: f32,
    pub sky_top: Vec3,
    pub sky_horizon: Vec3,
    pub ground_bounce: Vec3,
    pub fog_color: Vec3,
    pub fog_density: f32,
    pub fog_falloff: f32,
    pub exposure: f32,
    pub bloom: f32,
    pub windows_lit: f32,
    /// Gatelysene, 0..1.
    pub street_lights: f32,
    pub aurora: f32,
    pub weather: Weather,
    pub palette: [Vec3; PALETTE],
    pub lights: Vec<Light>,
    /// Røyken fra pipene: toppen av pipa (xyz) og tykkelsen (w), de
    /// nærmeste først.
    pub smoke: Vec<Vec4>,
    /// Modellinstansene som flytter seg (dyrene), per bilde.
    pub instances: Vec<ModelInstance>,
    /// Leddmatrisene (hjørne i meshen → posert) for de skinnede
    /// instansene, som `ModelInstance.skin` peker inn i.
    pub skins: Vec<Mat4>,
    /// Rødt flash langs kantene (0..1) når spilleren er truffet.
    pub hurt: f32,
    /// Feilsøkingsvisning: 0 vanlig, 1 albedo, 2 normaler, 3 okklusjon, 4 ruhet, 5 skygge.
    pub debug: u32,
    /// Rommene i kryptnivået du er i, så lyset ikke går gjennom murene
    /// (`None` oppe i byen).
    pub rooms: Option<RoomLight>,
    /// Høydene (y fra, til) som kan synes: byen ser ikke krypten under
    /// seg, og et nivå i krypten ser ikke byen eller de andre nivåene.
    /// Det som ligger helt utenfor tegnes ikke, heller ikke i skyggene.
    pub band: (f32, f32),
}

/// Rommene til lyset: hvert rom som en boks (hjørnene i x og z, gulvet og
/// taket), og per lys i `Frame.lights` rommene det lyser i (én bit per rom).
/// Et punkt hører til den minste boksen det er inne i; utenfor alle lyser
/// bare lysene med alle bitene satt (lykta du bærer).
#[derive(Clone, Debug, Default)]
pub struct RoomLight {
    pub boxes: Vec<(Vec2, Vec2, f32, f32)>,
    pub masks: Vec<u64>,
}

/// Synsfeltet som seks plan, til å hoppe over biter som er utenfor.
/// Fungerer for både perspektiv og ortografisk projeksjon med dybde 0..1.
#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_view_proj(m: Mat4) -> Self {
        let r = |i: usize| m.row(i);
        let planes = [r(3) + r(0), r(3) - r(0), r(3) + r(1), r(3) - r(1), r(2), r(3) - r(2)];
        Self { planes }
    }

    /// Om boksen er (delvis) innenfor.
    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for p in &self.planes {
            // Hjørnet lengst i planets retning.
            let corner = Vec3::new(
                if p.x >= 0.0 { max.x } else { min.x },
                if p.y >= 0.0 { max.y } else { min.y },
                if p.z >= 0.0 { max.z } else { min.z },
            );
            if p.x * corner.x + p.y * corner.y + p.z * corner.z + p.w < 0.0 {
                return false;
            }
        }
        true
    }
}
