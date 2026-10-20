//! Portene: grensesnittene mellom applikasjonen og omverdenen.
//!
//! Drevne porter (applikasjonen kaller ut, adapterne implementerer):
//!
//! - `Renderer` tegner et `Frame`. Adapter: wgpu.
//! - `Physics` holder kolliderne, flytter spilleren og skyter stråler.
//!   Adapter: Rapier.
//! - `ModelSource` gir modellene ved navn. Adapter: glTF-filene fra
//!   `tools/models/` som er bygd inn i binæren.
//! - `LayoutStore` lagrer byen og husenes innhold fra redigeringen.
//!   Adapter: GitHub fra nettleseren (`docs/redigering.md`, steg 5).
//!
//! Den drivende porten (`driving`) er det nettleseren, vinduet og
//! skjermbildeverktøyet sender inn (`Input`, `Command`) og får tilbake
//! (`Event`). Applikasjonen (`application::App`) implementerer den.

pub mod driving;

use crate::domain::geometry::Mesh;
use crate::domain::model::Model;
use crate::domain::scene::Frame;
use crate::domain::world::Shape;
use glam::{Affine3A, Vec3};

/// Det rendereren får én gang: den faste geometrien og modellene som står
/// i verden, med instansene.
pub struct StaticScene<'a> {
    pub mesh: &'a Mesh,
    pub chunks: &'a [crate::domain::geometry::Chunk],
    pub placements: &'a [crate::domain::model::Placement],
    pub models: &'a [Model],
}

/// Grafikkinnstillingene spilleren har valgt, bryter for bryter. Nivåene
/// enkel, standard og avansert er bare forhåndsvalg som setter bryterne
/// (`Quality::preset`); endrer spilleren én, er innstillingene egendefinerte.
/// Byttes aldri av seg selv. Oppløsningen velges i verten (lerretet).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quality {
    /// Solskyggen.
    pub shadow: bool,
    /// Sidene i skyggekartet per kaskade (1024, 2048 eller 4096).
    pub shadow_res: u32,
    /// Myke skyggekanter: tolv faste prøver i stedet for fire.
    pub soft_shadow: bool,
    /// Kaskade nummer to: skygge videre ut (`SHADOW_FAR2`), grovere.
    pub far_shadow: bool,
    /// Gløden rundt lys, ild og vinduer.
    pub bloom: bool,
    /// Kantutjevningen SMAA.
    pub smaa: bool,
    /// Avstandståka.
    pub fog: bool,
    /// Hvor langt unna modellene tegnes (m); 0 er så langt verden går.
    pub far: f32,
    /// Småting og fin detalj også langt unna.
    pub detail: bool,
    /// Lys som tegnes samtidig (lykter, ild, vinduer).
    pub lights: u32,
    /// Full teksturtetthet (av: ett mipnivå ned).
    pub textures: bool,
    /// Lagene på bakken blandet med støy i overgangene (av: ett lag).
    pub ground: bool,
    /// Røyk, regn og snø.
    pub particles: bool,
    /// Solstråler gjennom lufta.
    pub rays: bool,
    /// Skygge i krokene (skjermrommet, halv oppløsning; bare WebGPU).
    pub ao: bool,
    /// Den malte tonemappingen med fargegradering etter stedet og tida
    /// (av: den gamle ACES-kurven).
    pub grade: bool,
}

/// Forhåndsvalgene, i rekkefølge fra det letteste.
pub const PRESETS: [&str; 3] = ["enkel", "standard", "avansert"];

impl Quality {
    /// Et forhåndsvalg etter navnet (`PRESETS`).
    pub fn from_name(name: &str) -> Option<Self> {
        let base = Quality {
            shadow: true,
            shadow_res: 4096,
            soft_shadow: true,
            far_shadow: false,
            bloom: true,
            smaa: true,
            fog: true,
            far: 160.0,
            detail: false,
            lights: 8,
            textures: true,
            ground: true,
            particles: true,
            rays: true,
            ao: false,
            grade: true,
        };
        match name {
            "enkel" => Some(Quality { shadow: false, shadow_res: 2048, soft_shadow: false, bloom: false, smaa: false, lights: 4, textures: false, ground: false, rays: false, ..base }),
            "standard" => Some(base),
            "avansert" => Some(Quality { far_shadow: true, far: 0.0, detail: true, lights: 48, ao: true, ..base }),
            _ => None,
        }
    }

    /// Navnet på forhåndsvalget bryterne svarer til, eller `None`
    /// (egendefinert).
    pub fn preset(&self) -> Option<&'static str> {
        PRESETS.into_iter().find(|n| Quality::from_name(n).as_ref() == Some(self))
    }

    /// Sett én bryter etter navnet (som i nettlaget og `snapshot --gfx`).
    /// Sant om navnet finnes.
    pub fn set(&mut self, key: &str, v: f32) -> bool {
        let on = v > 0.5;
        match key {
            "shadow" => self.shadow = on,
            "shadow_res" => self.shadow_res = (v.max(256.0) as u32).next_power_of_two().min(4096),
            "soft_shadow" => self.soft_shadow = on,
            "far_shadow" => self.far_shadow = on,
            "bloom" => self.bloom = on,
            "smaa" => self.smaa = on,
            "fog" => self.fog = on,
            "far" => self.far = v.max(0.0),
            "detail" => self.detail = on,
            "lights" => self.lights = v.clamp(0.0, 64.0) as u32,
            "textures" => self.textures = on,
            "ground" => self.ground = on,
            "particles" => self.particles = on,
            "rays" => self.rays = on,
            "ao" => self.ao = on,
            "grade" => self.grade = on,
            _ => return false,
        }
        true
    }
}

impl Default for Quality {
    fn default() -> Self {
        Quality::from_name("standard").expect("standard finnes")
    }
}

/// Brytere til målingen i nettleseren (`?bench`): slå av deler av bildet
/// én om gangen, så tiden per del kan skilles fra hverandre.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bench {
    /// MSAA-prøver i stedet for nivåets (0: nivåets).
    pub msaa: u32,
    /// Alpha to coverage på alfakortene.
    pub a2c: bool,
    /// Solskyggen.
    pub shadow: bool,
    /// Flat farge i stedet for skyggeleggingen (måler rasterisering og
    /// overtegning alene).
    pub flat: bool,
    /// HDR-bildet i 4 byte per prøve (`Rg11b10Ufloat`) i stedet for 8, der
    /// kortet kan (måler om MSAA-målet sprenger flisminnet).
    pub small_hdr: bool,
    /// Tidsmålingen på grafikkortet (måler hva den selv koster).
    pub timers: bool,
    /// Himmelen (et fullskjermspass bak alt).
    pub sky: bool,
    /// Røyken, regnet og snøen (blandede firkanter, blandes per MSAA-prøve).
    pub particles: bool,
    /// Modellene i hovedpasset (av: bare bymeshen og terrenget).
    pub models: bool,
    /// Kantutjevningen SMAA i standard grafikk (når bildet er uten MSAA).
    pub smaa: bool,
}

impl Default for Bench {
    fn default() -> Self {
        Bench { msaa: 0, a2c: true, shadow: true, flat: false, small_hdr: false, timers: true, sky: true, particles: true, models: true, smaa: true }
    }
}

pub trait Renderer {
    fn render(&mut self, frame: &Frame);
    fn resize(&mut self, width: u32, height: u32);
    fn set_quality(&mut self, quality: Quality);
    /// Bryterne til målingen; `Bench::default()` er vanlig spill.
    fn set_bench(&mut self, _bench: Bench) {}
    /// Trekanter tegnet i hovedpasset sist bilde.
    fn triangles(&self) -> u32;
    /// Tegnekall i hovedpasset sist bilde.
    fn calls(&self) -> u32 {
        0
    }
    /// Millisekunder på grafikkortet per del av bildet, om det kan måles.
    fn gpu_times(&self) -> Vec<(&'static str, f32)> {
        vec![]
    }
    /// Legg til modeller som ble lastet etter at rendereren ble laget (bitene
    /// til et kryptnivå i nettleseren). Instansene deres står allerede i
    /// `StaticScene.placements`.
    fn add_models(&mut self, _models: &[Model]) {}
    /// Slipp modellene igjen (et nivå du har forlatt); instansene venter til
    /// modellene kommer på nytt.
    fn drop_models(&mut self, _names: &[String]) {}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ColliderId(pub u64);

pub struct Hit {
    pub collider: ColliderId,
    pub distance: f32,
}

pub trait Physics {
    /// Legg til en fast form i verden.
    fn add(&mut self, shape: &Shape) -> ColliderId;
    /// Legg til en boks som kan flyttes etterpå (porter, innbyggerne).
    fn add_movable(&mut self, half: Vec3, pose: Affine3A) -> ColliderId;
    /// Legg til en boks som bare er å sikte på (et mål): `cast_ray` treffer
    /// den, men kapselen går gjennom og `cast_solid` ser den ikke. Den
    /// flyttes med `set_pose`.
    fn add_aim(&mut self, half: Vec3, pose: Affine3A) -> ColliderId;
    fn set_pose(&mut self, id: ColliderId, pose: Affine3A);
    /// Flytt en kapsel (halv høyde `half`, radius `radius`) med `delta`,
    /// med kollisjon og trappesteg. Gir faktisk flytting og om den står på
    /// bakken.
    fn move_capsule(&mut self, pos: Vec3, half: f32, radius: f32, delta: Vec3, dt: f32) -> (Vec3, bool);
    /// Første kollider eller mål langs strålen (siktet).
    fn cast_ray(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<Hit>;
    /// Første kollider langs strålen, uten målene (murer, ting i veien).
    fn cast_solid(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<Hit>;
}

pub trait ModelSource {
    fn load(&self, name: &str) -> Option<Model>;
    /// Navnene på modellene som finnes.
    fn names(&self) -> Vec<String>;
    /// Det husene har med seg og kryptnivåene, ferdig lest.
    fn world_data(&self) -> crate::domain::world::Data;
    /// Modellene som finnes, men lastes senere i nettleseren (bitene til
    /// kryptnivåene, og byens modeller som ikke trengs til første bilde):
    /// monteringen setter dem ut etter målene, kolliderne og skjelettet her,
    /// og rendereren får dem med `Renderer::add_models` når økta ber om dem
    /// (`Event::FetchLevel`, `Event::FetchModels`).
    fn deferred(&self) -> Vec<crate::domain::model::Later> {
        vec![]
    }
}

/// Lagringen fra redigeringen: byen (`assets/layout/town.json`), det som
/// står i husene (`assets/layout/inside/<hus>.json`) og i kryptnivåene
/// (`assets/layout/crypt_NN.json`). Applikasjonen lagrer bare når reglene er
/// grønne (`Editor::clean`).
pub trait LayoutStore {
    fn save(
        &mut self,
        layout: &crate::domain::world::layout::Layout,
        contents: &[(String, Vec<crate::domain::world::layout::Item>)],
        crypt: &[(u32, crate::domain::world::crypt::level::CryptContents)],
    ) -> Result<(), String>;
}

#[cfg(test)]
mod quality_tests {
    use super::{Quality, PRESETS};

    /// Hvert forhåndsvalg kjenner seg igjen, en endret bryter er
    /// egendefinert, og bryternavnene i nettlaget finnes.
    #[test]
    fn forhandsvalgene_er_brytere() {
        for name in PRESETS {
            let q = Quality::from_name(name).expect("forhåndsvalget finnes");
            assert_eq!(q.preset(), Some(name));
        }
        let mut q = Quality::from_name("standard").unwrap();
        assert!(q.set("shadow", 0.0));
        assert_eq!(q.preset(), None);
        assert!(!q.set("finnes_ikke", 1.0));
        for key in ["shadow", "shadow_res", "soft_shadow", "far_shadow", "bloom", "smaa", "fog", "far", "detail", "lights", "textures", "ground", "particles", "rays", "ao", "grade"] {
            assert!(Quality::default().set(key, 1.0), "{key}");
        }
        assert_eq!(Quality::default().preset(), Some("standard"));
    }
}
