//! Rendereren. Rekkefølgen per bilde:
//!
//! 1. Skygge fra sola eller månen: én kaskade ut til `SHADOW_FAR`, som toner
//!    bort mot kanten (lenger ut gir den bakte okklusjonen i modellene dybden),
//!    og om spilleren vil en kaskade til ut til `SHADOW_FAR2`, side om side i
//!    samme kart.
//! 2. Verden, himmelen og nedbøren i HDR med MSAA (4× i standard grafikk), med
//!    tåka regnet i hvert punkt. Alfakort (gress, blad) bruker alpha to
//!    coverage, så de ikke flimrer. Ingen TAA: bildet blandes aldri med
//!    forrige, så det blir verken uskarpt eller får spøkelser.
//! 3. Glød og tonemapping til skjermen.

use crate::domain::geometry::{flag, kind, Chunk, Layer, Mesh, Vertex, LAYERS};
use crate::ports::{Bench, Quality};
use crate::domain::model::LayerImages;
use crate::ports;
pub use crate::domain::scene::{Frame, Weather};
use crate::domain::scene::{Frustum, MAX_LIGHTS, MAX_ROOMS, MAX_SMOKE, PALETTE};
use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Mat4, Vec3, Vec4};
use texture::Materials;
use wgpu::util::DeviceExt;

mod ao;
mod frame;
mod gpu;
pub mod lod;
mod models;
mod passes;
mod pipes;
mod setup;
mod smaa;
pub mod texture;
mod timer;

use frame::Plan;
use gpu::*;
pub use gpu::InstanceData;
use pipes::*;

const SKIN_W: u32 = 1024;
const SKIN_ROWS: u32 = 16;
const SKIN_MAX: usize = (SKIN_W / 4 * SKIN_ROWS) as usize;
const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Hvor langt ut sola kaster skygge, og hvor den begynner å tone bort.
/// Sidene i kartet per kaskade velges i innstillingene (`Quality::shadow_res`;
/// 4096 gir rundt 2,5 cm per texel nær).
pub const SHADOW_FAR: f32 = 40.0;
const SHADOW_FADE: f32 = 30.0;
/// Kaskade nummer to (`Quality::far_shadow`): grovere, ut til hit (toner
/// bort over den siste femtedelen).
const SHADOW_FAR2: f32 = 110.0;
const BLOOM_LEVELS: usize = 5;
const INSTANCE_STRIDE: u64 = 96;
const PASS_STRIDE: u64 = 256;
const RAIN_COUNT: u32 = 6000;
const SNOW_COUNT: u32 = 4000;
/// Røykdotter per pipe (må stemme med `PUFFS` i particles.wgsl).
const SMOKE_PUFFS: u32 = 24;


/// Hva et pass tegnet: trekanter i verden og i modellene, tegnekall og
/// modellinstanser.
#[derive(Clone, Copy, Default, Debug)]
pub struct PassStats {
    pub world: u32,
    pub models: u32,
    pub calls: u32,
    pub instances: u32,
}

/// Passene i `Renderer::pass_stats`.
pub const PASSES: [&str; 2] = ["skygge", "hoved"];

impl PassStats {
    fn add(&mut self, o: &PassStats) {
        self.world += o.world;
        self.models += o.models;
        self.calls += o.calls;
        self.instances += o.instances;
    }
}

/// Hva et pass skal ha med av verden.
#[derive(Clone, Copy)]
struct Cull {
    frustum: Frustum,
    eye: Vec3,
    /// Småting tegnes bare nærmere enn dette.
    fine_within: f32,
    /// Høydene som kan synes (`Frame.band`).
    band: (f32, f32),
}

impl Cull {
    fn sees(&self, lo: Vec3, hi: Vec3) -> bool {
        hi.y >= self.band.0 && lo.y <= self.band.1 && self.frustum.contains_aabb(lo, hi)
    }
}

/// En modell med to detaljnivåer i samme buffer (nær og fjern), og
/// instansene med hver sin boks, så bare de synlige tegnes.
struct GpuModel {
    name: String,
    mesh: GpuMesh,
    /// Har alfakort (gress, blad): tegnes med `cutout`-pipelinen.
    cutout: bool,
    /// Bare vann (`kit_water`): tegnes gjennomskinnelig etter himmelen med
    /// `water`-pipelinen (dybden til bunnen i instansens høyde).
    water: bool,
    near: std::ops::Range<u32>,
    far: std::ops::Range<u32>,
    textures: wgpu::BindGroup,
    /// Instansbufferet: de faste instansene sortert i ruter (lastet opp én
    /// gang), og så plass til de bevegelige én gang per pass-region.
    instances: wgpu::Buffer,
    /// Rutene med de faste instansene, i bufferets rekkefølge.
    cells: Vec<InstanceCell>,
    /// Antall faste instanser (der de bevegelige begynner).
    statics: u32,
    /// Plass til instanser som kommer med `Frame.instances`.
    reserve: usize,
    /// Modellens boks, til å regne ut boksen for de bevegelige.
    bounds: (Vec3, Vec3),
    /// Ekstradata for de bevegelige: hvit lakk, modellens ruhet og metall.
    extra: [f32; 4],
    /// Skinnet figur: instansene tar leddmatriser fra paletten.
    skinned: bool,
    /// Småting (gress, blomster, krukker, stein): i redusert grafikk tegnes
    /// de bare nær kameraet og kaster skygge bare helt nær.
    small: bool,
    /// Innsiden av et hus (`…_interior`): tegnes bare innen `INTERIOR_FAR`.
    interior: bool,
    /// Møblene inne i husene (`Placement.indoor`): tegnes bare innen
    /// `INTERIOR_FAR` og kaster solskygge bare innen `INDOOR_SHADOW`.
    indoor: bool,
    /// Hvor hver fast instans (nummeret i `Placement`) ligger i bufferet, og
    /// dataene den ble lastet opp med, så redigeringen kan skjule den
    /// (`Frame.hidden`) og vise den igjen; og de som er skjult nå.
    slot_of: Vec<u32>,
    originals: Vec<InstanceData>,
    hidden: std::cell::RefCell<Vec<u32>>,
    /// Teksturene i bindegruppen og målene deres, og bildene som skal
    /// oppgraderes i bakgrunnen (plassen 0–3 og bildet med de endelige
    /// målene; lat lasting i nettleseren, `upgrade_textures`).
    views: [wgpu::TextureView; 4],
    sizes: [(u32, u32); 4],
    pending: Vec<(usize, crate::domain::model::Image)>,
}

/// De faste instansene i en rute på `CELL_M` meter: boksen rundt dem og
/// hvor de ligger i instansbufferet.
struct InstanceCell {
    lo: Vec3,
    hi: Vec3,
    range: std::ops::Range<u32>,
}

/// Rutene de faste instansene sorteres i. Hele ruter kuttes bort, og
/// naboer i samme rad som begge tegnes blir ett kall.
const CELL_M: f32 = 20.0;

/// Hvilke instanser et pass tegner av en modell: nær og fjern detalj, som
/// områder i instansbufferet.
#[derive(Clone, Default)]
struct ModelDraw {
    near: Vec<std::ops::Range<u32>>,
    far: Vec<std::ops::Range<u32>>,
}

impl ModelDraw {
    fn push(list: &mut Vec<std::ops::Range<u32>>, r: std::ops::Range<u32>) {
        if r.is_empty() {
            return;
        }
        match list.last_mut() {
            Some(last) if last.end == r.start => last.end = r.end,
            _ => list.push(r),
        }
    }
}

/// Pass-regioner i instansbufferet: skyggen og kameraet.
const REGIONS: u32 = 2;

/// Boksen i verden: de åtte hjørnene transformert.
fn world_aabb(t: &Affine3A, lo: Vec3, hi: Vec3) -> (Vec3, Vec3) {
    let (mut wlo, mut whi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for k in 0..8 {
        let c = Vec3::new(if k & 1 == 0 { lo.x } else { hi.x }, if k & 2 == 0 { lo.y } else { hi.y }, if k & 4 == 0 { lo.z } else { hi.z });
        let w = t.transform_point3(c);
        wlo = wlo.min(w);
        whi = whi.max(w);
    }
    (wlo, whi)
}
/// Nærmere enn dette tegnes modellene i full detalj.
const MODEL_NEAR: f32 = 28.0;

struct Targets {
    width: u32,
    height: u32,
    /// Fargen med MSAA (løses opp i `hdr`); `None` uten MSAA.
    color: Option<wgpu::TextureView>,
    depth: wgpu::TextureView,
    hdr: wgpu::TextureView,
    bloom: Vec<(wgpu::TextureView, u32, u32)>,
    down0_bg: (wgpu::BindGroup, wgpu::Buffer),
    down_bg: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
    up_bg: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
    tonemap_bg: (wgpu::BindGroup, wgpu::Buffer),
    /// Kantutjevningen etter tonemappingen (standard grafikk, uten MSAA).
    smaa: Option<smaa::Targets>,
    /// Skygge i krokene (`Quality::ao`).
    ao: Option<ao::Targets>,
}

/// Innsiden av husene (modellene `…_interior`) tegnes bare så nær, så de
/// ikke koster noe ute i byen. Ytterveggene skjuler den til du er ved døra.
const INTERIOR_FAR: f32 = 22.0;
/// Møblene (`Placement.indoor`) kaster solskygge bare så nær kameraet.
const INDOOR_SHADOW: f32 = 10.0;
/// Hvor tett tåka minst er når sikten er kortet ned (`Quality::far`), så
/// kanten ikke synes; satt for 160 m og delt på sikten.
const REDUCED_FOG: f32 = 0.005 * 160.0;

/// Pipelinene som tegner inn i MSAA-bildet, laget for et antall prøver.
struct ScenePipes {
    samples: u32,
    a2c: bool,
    hdr: wgpu::TextureFormat,
    world: wgpu::RenderPipeline,
    /// Modellene med alfakort (gress, blad): alpha to coverage og `discard`.
    cutout: wgpu::RenderPipeline,
    /// Snittet i nivåredigeringen (`Frame.clip`): `discard` over høyden, så
    /// bare når snittet er på (det vanlige passet har ingen `discard`).
    clip: wgpu::RenderPipeline,
    /// Vannflatene: blandet over det ugjennomsiktige (`vs_water`/`fs_water`).
    water: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    particles: wgpu::RenderPipeline,
    /// Linjene redigeringen tegner (`Frame.lines`), uten dybdetest.
    lines: wgpu::RenderPipeline,
}

/// Shaderne og layoutene `ScenePipes` lages av, så de kan lages på nytt når
/// antall prøver byttes.
struct SceneSources {
    world: wgpu::ShaderModule,
    sky: wgpu::ShaderModule,
    particles: wgpu::ShaderModule,
    lines: wgpu::ShaderModule,
    world_layout: wgpu::PipelineLayout,
    simple_layout: wgpu::PipelineLayout,
}


pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    encode_srgb: bool,
    globals: wgpu::Buffer,
    globals_bg: wgpu::BindGroup,
    pass_buf: wgpu::Buffer,
    shadow_view: wgpu::TextureView,
    /// Skyggekartets størrelse (kaskadene side om side) og det som trengs
    /// for å lage det på nytt når innstillingene endres.
    shadow_size: (u32, u32),
    frame_layout: wgpu::BindGroupLayout,
    shadow_sampler: wgpu::Sampler,
    /// Til verden og partiklene: skyggekartet og verdiene per pass.
    frame_bg: wgpu::BindGroup,
    /// Til skyggepasset, som ikke kan lese kartet det tegner til.
    shadow_frame_bg: wgpu::BindGroup,
    post_layout: wgpu::BindGroupLayout,
    materials: Materials,
    /// Skyggen fra det ugjennomsiktige: bare dybde, uten fragmentshader.
    shadow_pipe: wgpu::RenderPipeline,
    /// Skyggen fra alfakortene: `fs_shadow` forkaster det gjennomsiktige.
    shadow_cutout: wgpu::RenderPipeline,
    /// Bymeshen har alfakort (blad, gress) og må tegnes med utskjæring.
    world_cutout: bool,
    sources: SceneSources,
    scene: ScenePipes,
    down_pipe: wgpu::RenderPipeline,
    up_pipe: wgpu::RenderPipeline,
    tonemap_pipe: wgpu::RenderPipeline,
    /// Tonemappingen inn i mellombildet til SMAA.
    tonemap_ldr: wgpu::RenderPipeline,
    smaa: smaa::Smaa,
    /// Skygge i krokene; `None` i WebGL2.
    ao: Option<ao::Ao>,
    /// Et hvitt bilde der okklusjonen ikke regnes.
    white: wgpu::TextureView,
    linear: wgpu::Sampler,
    world: GpuMesh,
    chunks: Vec<Chunk>,
    instances: wgpu::Buffer,
    models: Vec<GpuModel>,
    /// Instansene til modeller som ikke er lastet ennå, og til dem som kom
    /// med `add_models` (så de kan slippes igjen).
    waiting: Vec<crate::domain::model::Placement>,
    later: Vec<crate::domain::model::Placement>,
    /// Tiden oppgraderingen av teksturene har til gode (ms, høyst
    /// `delivery::UPGRADE_MS`; negativ når forrige bilde tok lenger), og
    /// taket per bilde (`set_upgrade_budget`).
    upgrade_credit: f64,
    upgrade_ms: f64,
    /// Enheten er borte (`lose`): ingenting sendes til den mer.
    lost: bool,
    mipper: texture::Mipper,
    /// Leddpaletten (se `SKIN_W`).
    skins: wgpu::Texture,
    targets: Targets,
    current: Option<wgpu::TextureView>,
    /// Grafikkinnstillingene spilleren har valgt.
    quality: Quality,
    /// Bryterne til målingen (`?bench`).
    bench: Bench,
    /// Flest MSAA-prøver kortet kan i standard og avansert grafikk.
    samples: (u32, u32),
    /// Trekanter tegnet i hovedpasset sist bilde: verden, props, modeller.
    triangles: std::cell::Cell<(u32, u32, u32)>,
    /// Hva hvert pass tegnet sist bilde (`PASSES`).
    pass_stats: std::cell::Cell<[PassStats; 2]>,
    /// Tiden på grafikkortet per del, når kortet kan måle den.
    timer: Option<timer::GpuTimer>,
    /// Hjørnene til linjene redigeringen tegner (vokser ved behov).
    line_buf: std::cell::RefCell<Option<(wgpu::Buffer, u64)>>,
}

impl Renderer {
    pub fn resize(&mut self, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if self.lost || width == self.targets.width && height == self.targets.height {
            return;
        }
        self.targets = Self::make_targets(&self.device, &self.post_layout, &self.linear, self.scene.samples, self.scene.hdr, self.smaa_on(self.scene.samples).then_some(&self.smaa), self.ao_on(), &self.white, width, height);
    }

    /// Millisekunder på grafikkortet per del av bildet (tomt om kortet
    /// ikke kan måle det).
    pub fn pass_times(&self) -> Vec<(&'static str, f32)> {
        self.timer.as_ref().filter(|_| self.bench.timers).map(|tm| timer::SECTIONS.iter().copied().zip(tm.ms).collect()).unwrap_or_default()
    }

    /// Hva hvert pass tegnet sist bilde, navngitt etter `PASSES`.
    pub fn pass_stats(&self) -> [PassStats; 2] {
        self.pass_stats.get()
    }

    /// Grafikkinnstillingene. Lager skyggekartet og målene på nytt om de
    /// trengs i en annen størrelse eller med et annet sett pass.
    pub fn set_quality(&mut self, quality: Quality) {
        self.quality = quality;
        self.apply();
    }

    /// Bryterne til målingen (`?bench`).
    pub fn set_bench(&mut self, bench: Bench) {
        self.bench = bench;
        self.apply();
    }

    /// MSAA-prøvene: 1 på alle nivåer (standard og avansert glatter kantene
    /// med SMAA; MSAA kostet 20–110 ms per bilde på eierens M2). Målingen
    /// kan be om 4× eller 8×.
    fn wanted_samples(&self) -> u32 {
        match self.bench.msaa {
            1 => 1,
            4 => self.samples.0,
            8 => self.samples.1,
            _ => 1,
        }
    }

    fn apply(&mut self) {
        if self.lost {
            return;
        }
        let shadow = self.wanted_shadow();
        if shadow != self.shadow_size {
            self.shadow_size = shadow;
            self.shadow_view = self
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("shadow"),
                    size: wgpu::Extent3d { width: shadow.0, height: shadow.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: DEPTH,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default());
            self.frame_bg = Self::frame_bg(&self.device, &self.frame_layout, &self.shadow_sampler, &self.pass_buf, &self.shadow_view);
        }
        let samples = self.wanted_samples();
        let hdr = if self.bench.small_hdr && self.device.features().contains(wgpu::Features::RG11B10UFLOAT_RENDERABLE) { wgpu::TextureFormat::Rg11b10Ufloat } else { HDR };
        let smaa = self.smaa_on(samples);
        let ao = self.ao_on().filter(|_| samples == 1).is_some();
        if samples != self.scene.samples || self.bench.a2c != self.scene.a2c || hdr != self.scene.hdr || smaa != self.targets.smaa.is_some() || ao != self.targets.ao.is_some() {
            if samples != self.scene.samples || self.bench.a2c != self.scene.a2c || hdr != self.scene.hdr {
                self.scene = self.sources.pipes(&self.device, samples, self.bench.a2c, hdr);
            }
            self.targets = Self::make_targets(&self.device, &self.post_layout, &self.linear, samples, hdr, smaa.then_some(&self.smaa), self.ao_on(), &self.white, self.targets.width, self.targets.height);
        }
    }

    /// SMAA når bildet tegnes uten MSAA, unntatt i enkel grafikk (målingen
    /// kan slå den av).
    fn smaa_on(&self, samples: u32) -> bool {
        self.quality.smaa && samples == 1 && self.bench.smaa
    }

    /// Skygge i krokene, der kortet kan lese dybden (ikke WebGL2).
    fn ao_on(&self) -> Option<&ao::Ao> {
        self.ao.as_ref().filter(|_| self.quality.ao)
    }


    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}

impl ports::Renderer for Renderer {
    fn add_models(&mut self, models: &[crate::domain::model::Model]) {
        Renderer::add_models(self, models);
    }
    fn drop_models(&mut self, names: &[String]) {
        Renderer::drop_models(self, names);
    }
    fn render(&mut self, frame: &Frame) {
        let out = match self.current.take() {
            Some(v) => v,
            None => return,
        };
        Renderer::render(self, &out, frame);
    }
    fn resize(&mut self, width: u32, height: u32) {
        Renderer::resize(self, width, height);
    }
    fn set_quality(&mut self, quality: Quality) {
        Renderer::set_quality(self, quality);
    }
    fn set_bench(&mut self, bench: Bench) {
        Renderer::set_bench(self, bench);
    }
    fn triangles(&self) -> u32 {
        let (w, p, m) = self.triangles.get();
        w + p + m
    }
    fn calls(&self) -> u32 {
        self.pass_stats()[1].calls
    }
    fn gpu_times(&self) -> Vec<(&'static str, f32)> {
        self.pass_times()
    }
}

impl Renderer {
    /// Leser ut et bakt teksturlag (albedo eller normal/ruhet/høyde) som
    /// RGBA8, til feilsøking.
    pub fn dump_layer(&self, layer: u32, nrh: bool) -> (u32, Vec<u8>) {
        let tex = if nrh { &self.materials.nrh_tex } else { &self.materials.albedo_tex };
        let size = tex.width();
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dump"),
            size: (size * size * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: 0, y: 0, z: layer }, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size * 4), rows_per_image: Some(size) } },
            wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
        );
        self.queue.submit([enc.finish()]);
        buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::Wait).expect("poll");
        let data = buf.slice(..).get_mapped_range().to_vec();
        (size, data)
    }

    /// Bildet neste `ports::Renderer::render` skal tegne til. Settes av
    /// verten rett før `App::draw`.
    pub fn set_target(&mut self, view: wgpu::TextureView) {
        self.current = Some(view);
    }

    /// Enheten er borte (nettleseren mistet grafikkortet: `device.lost`).
    /// Da tegnes ingenting mer, og ingenting nytt lages på den: i Chromium
    /// kaster `createBuffer` på en tapt enhet, og wgpu gjør det til en
    /// panikk som stopper hele motoren (først i `add_models` når et
    /// kryptnivå kom etter tapet). Spillet går videre uten bilde.
    pub fn lose(&mut self, why: &str) {
        if !self.lost {
            log::error!("Grafikkortet er borte: {}. Last siden på nytt for å tegne igjen.", why.trim_end_matches('.'));
        }
        self.lost = true;
        self.current = None;
    }

    /// Om enheten er borte.
    pub fn is_lost(&self) -> bool {
        self.lost
    }
}
