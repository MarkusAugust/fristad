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
mod models;
pub mod lod;
mod smaa;
pub mod texture;
mod timer;

/// Hjørnet slik grafikkortet får det: som `Vertex`, men normalen som 8 bit
/// med fortegn (modellene har dem slik i fila uansett), 40 byte mot 48.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    pos: [f32; 3],
    normal: [i8; 4],
    uv: [f32; 2],
    color: [u8; 4],
    mat: [u8; 4],
    joints: [u8; 4],
    weights: [u8; 4],
}

impl GpuVertex {
    fn from(v: &Vertex) -> Self {
        let q = |x: f32| (x.clamp(-1.0, 1.0) * 127.0).round() as i8;
        Self { pos: v.pos, normal: [q(v.normal[0]), q(v.normal[1]), q(v.normal[2]), 0], uv: v.uv, color: v.color, mat: v.mat, joints: v.joints, weights: v.weights }
    }
}

const _: () = assert!(std::mem::size_of::<GpuVertex>() == 40);

const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<GpuVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Snorm8x4,
        2 => Float32x2,
        3 => Unorm8x4,
        4 => Uint8x4,
        5 => Uint8x4,
        6 => Unorm8x4,
    ],
};
/// Leddpaletten: en flyttallstekstur med fire texels (kolonnene) per
/// matrise, 256 matriser per rad. En tekstur, ikke et lagringsbuffer, så
/// den virker i WebGL2 òg.
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuLight {
    pos_radius: [f32; 4],
    color_intensity: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    sun_vp: [[f32; 4]; 4],
    camera: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    sky_top: [f32; 4],
    sky_horizon: [f32; 4],
    ground_bounce: [f32; 4],
    fog_color: [f32; 4],
    params: [f32; 4],
    weather: [f32; 4],
    screen: [f32; 4],
    /// x vind, y MSAA-prøver, z der skyggen begynner å tone bort, w der
    /// den slutter.
    misc: [f32; 4],
    sky: [f32; 4],
    tex_scale: [[f32; 4]; 16],
    palette: [[f32; 4]; PALETTE],
    lights: [GpuLight; MAX_LIGHTS],
    smoke: [[f32; 4]; MAX_SMOKE],
    /// Rommene i kryptnivået (lyset stoppes av murene): antallet (x, 0
    /// oppe i byen), boksene (x0, z0, x1, z1), gulvet og taket (to rom per
    /// vec4), og rommene hvert lys lyser i (to u32 per lys, to lys per vec4).
    room_info: [f32; 4],
    room_box: [[f32; 4]; MAX_ROOMS],
    room_y: [[f32; 4]; MAX_ROOMS / 2],
    light_rooms: [[u32; 4]; MAX_LIGHTS / 2],
    /// x full teksturtetthet, y flat farge (målingen `?bench`), z skyggen
    /// (0 av, 1 fire prøver, 2 myk), w blandede lag på bakken.
    quality: [f32; 4],
    /// Kaskade nummer to.
    sun_vp2: [[f32; 4]; 4],
    /// x, y en texel i skyggekartet (det har kaskadene side om side), z
    /// antall kaskader, w hvor langt kaskade to når.
    shadow: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostUniform {
    params: [f32; 4],
    texel: [f32; 4],
    /// Tonemappingen: xy sola på skjermen (uv), z styrken på solstrålene,
    /// w den malte graderingen (1) eller den gamle ACES-kurven (0).
    sun: [f32; 4],
    /// x: spillerens lysstyrke (gamma, 1 er som malt).
    extra: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
    pub extra: [f32; 4],
    /// x: plassen i leddpaletten + 1 for skinnede instanser, 0 ellers;
    /// y: flammefargen i utseendet.
    pub extra2: [f32; 4],
}

impl InstanceData {
    pub fn new(t: &Affine3A, extra: [f32; 4]) -> Self {
        Self { model: Mat4::from(*t).to_cols_array_2d(), extra, extra2: [0.0; 4] }
    }
    pub fn skinned(t: &Affine3A, extra: [f32; 4], base: u32) -> Self {
        Self { model: Mat4::from(*t).to_cols_array_2d(), extra, extra2: [base as f32 + 1.0, 0.0, 0.0, 0.0] }
    }
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: INSTANCE_STRIDE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![7 => Float32x4, 8 => Float32x4, 9 => Float32x4, 10 => Float32x4, 11 => Float32x4, 12 => Float32x4],
    };
}

struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    /// 16 bit når meshen har færre enn 65 536 hjørner.
    format: wgpu::IndexFormat,
}

impl GpuMesh {
    fn new(device: &wgpu::Device, mesh: &Mesh, label: &str) -> Self {
        // Tomme mesh (markører) får små buffere og tegnes ikke.
        let (v, i32s): (Vec<GpuVertex>, Vec<u32>) = if mesh.indices.is_empty() {
            (vec![GpuVertex::zeroed()], vec![0u32; 3])
        } else {
            (mesh.vertices.iter().map(GpuVertex::from).collect(), mesh.indices.clone())
        };
        let (format, i): (wgpu::IndexFormat, Vec<u8>) = if v.len() <= u16::MAX as usize {
            // Et jevnt antall, så bufferet er delelig på fire.
            let mut short: Vec<u16> = i32s.iter().map(|k| *k as u16).collect();
            if short.len() % 2 == 1 {
                short.push(0);
            }
            (wgpu::IndexFormat::Uint16, bytemuck::cast_slice(&short).to_vec())
        } else {
            (wgpu::IndexFormat::Uint32, bytemuck::cast_slice(&i32s).to_vec())
        };
        Self {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents: bytemuck::cast_slice(&v), usage: wgpu::BufferUsages::VERTEX }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents: &i, usage: wgpu::BufferUsages::INDEX }),
            format,
        }
    }


    fn bind(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_index_buffer(self.indices.slice(..), self.format);
    }
}

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

impl SceneSources {
    fn pipes(&self, device: &wgpu::Device, samples: u32, a2c: bool, hdr: wgpu::TextureFormat) -> ScenePipes {
        let depth = |write: bool, compare: wgpu::CompareFunction| Some(wgpu::DepthStencilState { format: DEPTH, depth_write_enabled: write, depth_compare: compare, stencil: Default::default(), bias: Default::default() });
        let ms = |a2c: bool| wgpu::MultisampleState { count: samples, mask: !0, alpha_to_coverage_enabled: a2c && samples > 1 };
        let target = |blend: Option<wgpu::BlendState>| [Some(wgpu::ColorTargetState { format: hdr, blend, write_mask: wgpu::ColorWrites::ALL })];
        let buffers = [VERTEX_LAYOUT, InstanceData::LAYOUT];
        // Det ugjennomsiktige uten alpha to coverage og uten `discard` (begge
        // skrur av tidlig dybdetest på Apple-GPU-er); alfakortene for seg.
        let world_pipe = |label: &str, entry: &str, a2c: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&self.world_layout),
                vertex: wgpu::VertexState { module: &self.world, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &buffers },
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: depth(true, wgpu::CompareFunction::Less),
                // Alfakortene skriver dekningen i alfa, som blir til MSAA-prøver.
                multisample: ms(a2c),
                fragment: Some(wgpu::FragmentState { module: &self.world, entry_point: Some(entry), compilation_options: Default::default(), targets: &target(None) }),
                multiview: None,
                cache: None,
            })
        };
        let world = world_pipe("world", "fs", false);
        let cutout = world_pipe("cutout", "fs_cutout", a2c);
        let sky = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky"),
            layout: Some(&self.simple_layout),
            vertex: wgpu::VertexState { module: &self.sky, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: Default::default(),
            depth_stencil: depth(false, wgpu::CompareFunction::LessEqual),
            multisample: ms(false),
            fragment: Some(wgpu::FragmentState { module: &self.sky, entry_point: Some("fs"), compilation_options: Default::default(), targets: &target(None) }),
            multiview: None,
            cache: None,
        });
        let particles = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("particles"),
            layout: Some(&self.simple_layout),
            vertex: wgpu::VertexState { module: &self.particles, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, cull_mode: None, ..Default::default() },
            depth_stencil: depth(false, wgpu::CompareFunction::Less),
            multisample: ms(false),
            fragment: Some(wgpu::FragmentState { module: &self.particles, entry_point: Some("fs"), compilation_options: Default::default(), targets: &target(Some(wgpu::BlendState::ALPHA_BLENDING)) }),
            multiview: None,
            cache: None,
        });
        let line_buffers = [wgpu::VertexBufferLayout { array_stride: 24, step_mode: wgpu::VertexStepMode::Vertex, attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3] }];
        let lines = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lines"),
            layout: Some(&self.simple_layout),
            vertex: wgpu::VertexState { module: &self.lines, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &line_buffers },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::LineList, cull_mode: None, ..Default::default() },
            depth_stencil: depth(false, wgpu::CompareFunction::Always),
            multisample: ms(false),
            fragment: Some(wgpu::FragmentState { module: &self.lines, entry_point: Some("fs"), compilation_options: Default::default(), targets: &target(None) }),
            multiview: None,
            cache: None,
        });
        ScenePipes { samples, a2c, hdr, world, cutout, sky, particles, lines }
    }
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

fn shader(device: &wgpu::Device, name: &str, parts: &[&str]) -> wgpu::ShaderModule {
    let full = parts.join("\n");
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(full.into()),
    })
}

fn uniform_entry(binding: u32, dynamic: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dynamic,
            min_binding_size: None,
        },
        count: None,
    }
}

fn tex_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
        count: None,
    }
}

fn sampler_entry(binding: u32, ty: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(ty), count: None }
}

const FILTER: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };

/// Flest MSAA-prøver (av `wanted`, synkende) som både HDR-fargen og dybden
/// kan ha på dette kortet; 1 om ingen.
fn max_samples(adapter: &wgpu::Adapter, device: &wgpu::Device, wanted: &[u32]) -> u32 {
    let ok = |n: u32| {
        // Annet enn 4 krever at kortets egne formategenskaper er slått på.
        (n == 4 || device.features().contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES))
            && adapter.get_texture_format_features(HDR).flags.sample_count_supported(n)
            && adapter.get_texture_format_features(DEPTH).flags.sample_count_supported(n)
    };
    wanted.iter().copied().find(|n| ok(*n)).unwrap_or(1)
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        adapter: &wgpu::Adapter,
        output: wgpu::TextureFormat,
        scene: &ports::StaticScene,
        external: &[(Layer, LayerImages)],
        width: u32,
        height: u32,
    ) -> Self {
        log::info!("Utdata: {output:?}");
        let (materials, mipper) = Materials::new(&device, &queue, external);

        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Leddpaletten for skinnede modeller, skrevet per bilde.
        let skins = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("skins"),
            size: wgpu::Extent3d { width: SKIN_W, height: SKIN_ROWS, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let skins_view = skins.create_view(&Default::default());
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals"),
            entries: &[
                uniform_entry(0, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
            ],
        });
        let globals_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals"),
            layout: &globals_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&skins_view) },
            ],
        });

        // Per pass for partiklene: regn, snø og røyk.
        let pass_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pass"),
            size: PASS_STRIDE * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_tex = |label: &str, size: u32, width: u32| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width, height: size, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: DEPTH,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let side = device.limits().max_texture_dimension_2d.min(4096);
        let shadow_size = (side, side);
        let shadow_view = depth_tex("shadow", shadow_size.1, shadow_size.0);
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &[tex_entry(0, wgpu::TextureSampleType::Depth), sampler_entry(1, wgpu::SamplerBindingType::Comparison), uniform_entry(2, true)],
        });
        let frame_bg = Self::frame_bg(&device, &frame_layout, &shadow_sampler, &pass_buf, &shadow_view);
        let shadow_frame_bg = Self::frame_bg(&device, &frame_layout, &shadow_sampler, &pass_buf, &depth_tex("dummy shadow", 1, 1));
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[tex_entry(0, FILTER), tex_entry(1, FILTER), sampler_entry(2, wgpu::SamplerBindingType::Filtering), uniform_entry(3, false), tex_entry(4, FILTER)],
        });

        let noise = include_str!("shaders/noise.wgsl");
        let common = include_str!("shaders/common.wgsl");
        let shadow_src = include_str!("shaders/shadow.wgsl");
        let world_mod = shader(&device, "world", &[noise, common, shadow_src, include_str!("shaders/world.wgsl")]);
        let sky_mod = shader(&device, "sky", &[noise, common, include_str!("shaders/sky.wgsl")]);
        let particle_mod = shader(&device, "particles", &[noise, common, include_str!("shaders/particles.wgsl")]);
        let line_mod = shader(&device, "lines", &[noise, common, include_str!("shaders/lines.wgsl")]);
        let post_mod = shader(&device, "post", &[common, include_str!("shaders/post.wgsl")]);

        let world_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("world"),
            bind_group_layouts: &[&globals_layout, &frame_layout, &materials.layout, &materials.model_layout],
            push_constant_ranges: &[],
        });
        let simple_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("simple"),
            bind_group_layouts: &[&globals_layout, &frame_layout],
            push_constant_ranges: &[],
        });
        let post_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post"),
            bind_group_layouts: &[&globals_layout, &post_layout],
            push_constant_ranges: &[],
        });

        let buffers = [VERTEX_LAYOUT, InstanceData::LAYOUT];
        // Det ugjennomsiktige skrives bare som dybde (ingen fragmentshader,
        // så kortet kan forkaste tidlig); alfakortene forkaster i `fs_shadow`.
        let shadow_with = |label: &str, fragment: Option<wgpu::FragmentState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState { module: &world_mod, entry_point: Some("vs_shadow"), compilation_options: Default::default(), buffers: &buffers },
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.5, clamp: 0.0 },
                }),
                multisample: Default::default(),
                fragment,
                multiview: None,
                cache: None,
            })
        };
        let shadow_pipe = shadow_with("shadow", None);
        let shadow_cutout = shadow_with("shadow cutout", Some(wgpu::FragmentState { module: &world_mod, entry_point: Some("fs_shadow"), compilation_options: Default::default(), targets: &[] }));
        let samples = (max_samples(adapter, &device, &[4]), max_samples(adapter, &device, &[8, 4]));
        log::info!("MSAA i målingen: {}× og {}×", samples.0, samples.1);
        let sources = SceneSources { world: world_mod, sky: sky_mod, particles: particle_mod, lines: line_mod, world_layout, simple_layout };
        // Spillet starter i standard grafikk; verten bytter med `set_quality`.
        let scene_pipes = sources.pipes(&device, 1, true, HDR);

        let post_pipe = |entry: &str, format: wgpu::TextureFormat, blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&post_pipe_layout),
                vertex: wgpu::VertexState { module: &post_mod, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &post_mod,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format, blend, write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview: None,
                cache: None,
            })
        };
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
            alpha: wgpu::BlendComponent::REPLACE,
        };
        let down_pipe = post_pipe("fs_down", HDR, None);
        let up_pipe = post_pipe("fs_up", HDR, Some(additive));
        let tonemap_pipe = post_pipe("fs_tonemap", output, None);
        let tonemap_ldr = post_pipe("fs_tonemap", smaa::LDR, None);
        let smaa = smaa::Smaa::new(&device, &queue, output);
        // WebGL2 kan ikke lese dybden med `textureLoad`: ingen skygge i krokene der.
        let ao = (adapter.get_info().backend != wgpu::Backend::Gl).then(|| ao::Ao::new(&device, &globals_layout, common));
        let white = device
            .create_texture_with_data(
                &queue,
                &wgpu::TextureDescriptor {
                    label: Some("white"),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &[255, 255, 255, 255],
            )
            .create_view(&Default::default());

        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let world_mesh = GpuMesh::new(&device, scene.mesh, "world");
        let models = scene.models;
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: INSTANCE_STRIDE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let gpu_models: Vec<GpuModel> = scene
            .placements
            .iter()
            .filter_map(|pl| {
                let m = models.iter().find(|m| m.name == pl.model)?;
                Some(models::gpu_model(&device, &queue, &materials, &mipper, pl, m))
            })
            .collect();
        // Instansene til modellene som kommer senere (kryptnivåene i
        // nettleseren, `add_models`).
        let waiting: Vec<crate::domain::model::Placement> = scene.placements.iter().filter(|pl| !models.iter().any(|m| m.name == pl.model)).cloned().collect();
        log::info!(
            "Verden: {} hjørner, {} trekanter, {} modeller, {} faste instanser i {} ruter, plass til {} bevegelige",
            scene.mesh.vertices.len(),
            scene.mesh.indices.len() / 3,
            gpu_models.len(),
            gpu_models.iter().map(|m| m.statics).sum::<u32>(),
            gpu_models.iter().map(|m| m.cells.len()).sum::<usize>(),
            gpu_models.iter().map(|m| m.reserve).sum::<usize>()
        );

        let targets = Self::make_targets(&device, &post_layout, &linear, 1, HDR, Some(&smaa), None, &white, width, height);
        let chunks = scene.chunks.to_vec();
        let timer = timer::GpuTimer::new(&device, &queue);
        Self {
            device,
            queue,
            encode_srgb: !output.is_srgb(),
            globals,
            globals_bg,
            pass_buf,
            shadow_view,
            shadow_size,
            frame_layout,
            shadow_sampler,
            frame_bg,
            shadow_frame_bg,
            post_layout,
            materials,
            shadow_pipe,
            shadow_cutout,
            world_cutout: scene.mesh.vertices.iter().any(|v| v.mat[2] & flag::ALPHA != 0 || v.mat[0] == kind::LEAF),
            sources,
            scene: scene_pipes,
            down_pipe,
            up_pipe,
            tonemap_pipe,
            tonemap_ldr,
            smaa,
            ao,
            white,
            linear,
            world: world_mesh,
            chunks,
            instances,
            models: gpu_models,
            waiting,
            later: vec![],
            lost: false,
            mipper,
            skins,
            targets,
            current: None,
            quality: Quality::default(),
            bench: Bench::default(),
            line_buf: Default::default(),
            samples,
            triangles: std::cell::Cell::new((0, 0, 0)),
            pass_stats: Default::default(),
            timer,
        }
    }

    /// Bindingene til verden: skyggekartet, sammenligningen og verdiene
    /// per pass.
    fn frame_bg(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, pass_buf: &wgpu::Buffer, shadow: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(shadow) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: pass_buf, offset: 0, size: wgpu::BufferSize::new(16) }) },
            ],
        })
    }

    /// Skyggekartet innstillingene vil ha: sidene per kaskade (så store som
    /// kortet tillater) og kaskadene side om side.
    fn wanted_shadow(&self) -> (u32, u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        let n = if self.quality.far_shadow { 2 } else { 1 };
        let res = if self.quality.shadow { self.quality.shadow_res.clamp(512, 4096).min(max / n) } else { 1 };
        (res * n, res)
    }

    #[allow(clippy::too_many_arguments)]
    fn make_targets(device: &wgpu::Device, post_layout: &wgpu::BindGroupLayout, linear: &wgpu::Sampler, samples: u32, format: wgpu::TextureFormat, smaa: Option<&smaa::Smaa>, ao: Option<&ao::Ao>, white: &wgpu::TextureView, width: u32, height: u32) -> Targets {
        let width = width.max(1);
        let height = height.max(1);
        let tex = |label: &str, w: u32, h: u32, format, samples: u32| {
            let mut usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
            if samples == 1 {
                usage |= wgpu::TextureUsages::TEXTURE_BINDING;
            }
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: samples,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let color = (samples > 1).then(|| tex("color msaa", width, height, format, samples));
        let depth = tex("depth", width, height, DEPTH, samples);
        let hdr = tex("hdr", width, height, format, 1);
        let mut bloom = vec![];
        let (mut w, mut h) = (width, height);
        for i in 0..BLOOM_LEVELS {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            bloom.push((tex(&format!("bloom {i}"), w, h, HDR, 1), w, h));
        }
        let ao = ao.filter(|_| samples == 1).map(|a| a.targets(device, &depth, width, height));
        let bind = |t0: &wgpu::TextureView, t1: &wgpu::TextureView, t4: &wgpu::TextureView| {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("post"),
                size: std::mem::size_of::<PostUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("post"),
                layout: post_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(t0) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(t1) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(linear) },
                    wgpu::BindGroupEntry { binding: 3, resource: buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(t4) },
                ],
            });
            (bg, buf)
        };
        let down0_bg = bind(&hdr, &hdr, white);
        let down_bg = (1..BLOOM_LEVELS).map(|i| bind(&bloom[i - 1].0, &bloom[i - 1].0, white)).collect();
        let up_bg = (1..BLOOM_LEVELS).rev().map(|i| bind(&bloom[i].0, &bloom[i].0, white)).collect();
        let tonemap_bg = bind(&hdr, &bloom[0].0, ao.as_ref().map_or(white, |a| &a.out));
        let smaa = smaa.map(|s| s.targets(device, width, height));
        Targets { width, height, color, depth, hdr, bloom, down0_bg, down_bg, up_bg, tonemap_bg, smaa, ao }
    }

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

    /// Ortografisk projeksjon fra sola som dekker kamerafrustumet mellom
    /// `near` og `far`, låst til hele texels så kantene ikke skimrer.
    fn cascade(&self, f: &Frame, near: f32, far: f32, res: u32) -> Mat4 {
        let inv = (f.proj * f.view).inverse();
        let ndc_z = |d: f32| {
            let c = f.proj * Vec4::new(0.0, 0.0, -d, 1.0);
            c.z / c.w
        };
        let (zn, zf) = (ndc_z(near), ndc_z(far));
        let mut corners = [Vec3::ZERO; 8];
        let mut k = 0;
        for z in [zn, zf] {
            for y in [-1.0f32, 1.0] {
                for x in [-1.0f32, 1.0] {
                    let p = inv * Vec4::new(x, y, z, 1.0);
                    corners[k] = p.truncate() / p.w;
                    k += 1;
                }
            }
        }
        let center = corners.iter().fold(Vec3::ZERO, |a, c| a + *c) / 8.0;
        let radius = corners.iter().map(|c| c.distance(center)).fold(0.0, f32::max) * 1.05;
        let up = if f.sun_dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let view = Mat4::look_at_rh(f.sun_dir * 400.0, Vec3::ZERO, up);
        let texel = 2.0 * radius / res.max(1) as f32;
        let mut c = view.transform_point3(center);
        c.x = (c.x / texel).round() * texel;
        c.y = (c.y / texel).round() * texel;
        // z i lysrommet er negativt fremover. Ta med alt mellom lyset og sonen.
        let proj = Mat4::orthographic_rh(c.x - radius, c.x + radius, c.y - radius, c.y + radius, -c.z - radius - 200.0, -c.z + radius);
        proj * view
    }

    /// Skjul de faste instansene i `Frame.hidden` (flyttet langt ned, så
    /// verken bildet eller skyggen ser dem) og vis de som ikke lenger er
    /// skjult, med dataene de ble lastet opp med.
    fn apply_hidden(&self, f: &Frame) {
        let away = InstanceData::new(&Affine3A::from_translation(Vec3::new(0.0, -1.0e6, 0.0)), [0.0; 4]);
        for m in &self.models {
            let mut want: Vec<u32> = f.hidden.iter().filter(|(name, _)| *name == m.name).filter_map(|(_, n)| m.slot_of.get(*n).copied()).collect();
            let mut now = m.hidden.borrow_mut();
            if want.is_empty() && now.is_empty() {
                continue;
            }
            want.sort_unstable();
            want.dedup();
            for slot in now.iter().filter(|s| !want.contains(s)) {
                self.queue.write_buffer(&m.instances, *slot as u64 * INSTANCE_STRIDE, bytemuck::bytes_of(&m.originals[*slot as usize]));
            }
            for slot in want.iter().filter(|s| !now.contains(s)) {
                self.queue.write_buffer(&m.instances, *slot as u64 * INSTANCE_STRIDE, bytemuck::bytes_of(&away));
            }
            *now = want;
        }
    }

    /// Linjene til redigeringen i bufferet; gir hvor mange hjørner.
    fn upload_lines(&self, f: &Frame) -> u32 {
        if f.lines.is_empty() {
            return 0;
        }
        let data: Vec<[f32; 6]> = f.lines.iter().flat_map(|(a, b, c)| [[a.x, a.y, a.z, c.x, c.y, c.z], [b.x, b.y, b.z, c.x, c.y, c.z]]).collect();
        let bytes = bytemuck::cast_slice::<[f32; 6], u8>(&data);
        let mut buf = self.line_buf.borrow_mut();
        if buf.as_ref().is_none_or(|(_, cap)| *cap < bytes.len() as u64) {
            let cap = (bytes.len() as u64).next_power_of_two().max(4096);
            *buf = Some((self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("lines"), size: cap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }), cap));
        }
        let (b, _) = buf.as_ref().expect("bufferet");
        self.queue.write_buffer(b, 0, bytes);
        data.len() as u32
    }

    pub fn render(&mut self, out: &wgpu::TextureView, f: &Frame) {
        if self.lost {
            return;
        }
        self.apply_hidden(f);
        let line_count = self.upload_lines(f);
        let t = &self.targets;
        let (w, h) = (t.width as f32, t.height as f32);
        let view_proj = f.proj * f.view;
        // Nede i kjelleren når ikke sola ned: ingen skygge, og sollyset er av.
        let q = self.quality;
        let reduced = !q.detail;
        let sunlit = f.rooms.is_none() && f.band.1 >= 0.0;
        let shadows = sunlit && q.shadow && self.bench.shadow;
        let res = self.shadow_size.1;
        let cascades = if shadows && q.far_shadow { 2 } else { 1 };
        let sun_vp = self.cascade(f, 0.1, SHADOW_FAR, res);
        let sun_vp2 = if cascades == 2 { self.cascade(f, 0.1, SHADOW_FAR2, res) } else { sun_vp };
        // Småting kaster skygge bare nær. Med to kaskader tegnes det samme
        // utvalget i begge (det fjerne frustumet rommer det nære).
        let sun_cull = Cull { frustum: Frustum::from_view_proj(sun_vp2), eye: f.camera, fine_within: if reduced { 18.0 } else { 40.0 }, band: f.band };
        let cam_cull = Cull { frustum: Frustum::from_view_proj(view_proj), eye: f.camera, fine_within: if reduced { 35.0 } else { 70.0 }, band: f.band };
        // Kortere sikt: grovere detalj tidligere og småting bare nær.
        let model_near = if reduced { MODEL_NEAR * 0.5 } else { MODEL_NEAR };
        let far = if q.far > 0.0 { q.far } else { f32::INFINITY };
        // Modellinstansene: hvilke som er synlige i hver region, nær og fjern.
        let culls = [sun_cull, cam_cull];
        let mut model_draws: Vec<[ModelDraw; REGIONS as usize]> = Vec::with_capacity(self.models.len());
        for m in &self.models {
            // De bevegelige instansene (figurene, fiendene, dyrene) legges
            // til de faste, innenfor plassen byen satte av.
            let moving: Vec<(InstanceData, Vec3, Vec3)> = f
                .instances
                .iter()
                .filter(|i| i.model == m.name)
                .take(m.reserve)
                .map(|i| {
                    let (lo, hi) = world_aabb(&i.transform, m.bounds.0, m.bounds.1);
                    // Blinket ligger over 1 i markeringen (extra.x).
                    let mut extra = m.extra;
                    extra[0] = if i.flash > 0.0 { 1.0 + i.flash } else { 0.0 };
                    if let Some(t) = i.tint {
                        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
                        extra[1] = (c(t.x) << 16 | c(t.y) << 8 | c(t.z)) as f32;
                    }
                    let inst = match i.skin.map(|(base, _)| base).filter(|base| m.skinned && (*base as usize) < SKIN_MAX) {
                        Some(base) => InstanceData::skinned(&i.transform, extra, base),
                        None => InstanceData::new(&i.transform, extra),
                    };
                    (inst, lo, hi)
                })
                .collect();
            let mut draws: [ModelDraw; REGIONS as usize] = Default::default();
            let mut dynamic: Vec<InstanceData> = Vec::with_capacity(m.reserve * REGIONS as usize);
            for (r, cull) in culls.iter().enumerate() {
                if r == 0 && !shadows {
                    continue;
                }
                // Hvor langt unna noe i boksen er, og om det tegnes nær,
                // fjernt eller ikke.
                let lod = |lo: Vec3, hi: Vec3| -> Option<bool> {
                    let d = cull.eye.clamp(lo, hi).distance(cull.eye);
                    if m.interior && d > INTERIOR_FAR {
                        return None;
                    }
                    // Møblene kaster solskygge bare nær deg (i Smia, som
                    // er åpen mot torget, synes den).
                    if m.indoor && r == 0 && cam_cull.eye.clamp(lo, hi).distance(cam_cull.eye) > INDOOR_SHADOW {
                        return None;
                    }
                    if reduced && m.small && d > if r == 1 { 32.0 } else { 12.0 } {
                        return None;
                    }
                    // Kortere sikt: det som er langt unna tegnes ikke (tåka
                    // er tettere, så kanten glir bort).
                    if d > far {
                        return None;
                    }
                    Some(d < model_near)
                };
                let d = &mut draws[r];
                for c in &m.cells {
                    if !cull.sees(c.lo, c.hi) {
                        continue;
                    }
                    match lod(c.lo, c.hi) {
                        Some(true) => ModelDraw::push(&mut d.near, c.range.clone()),
                        Some(false) => ModelDraw::push(&mut d.far, c.range.clone()),
                        None => {}
                    }
                }
                // De bevegelige: hver region sin plass bak de faste.
                let base = m.statics + (r * m.reserve) as u32;
                let (mut near, mut far) = (vec![], vec![]);
                for (inst, lo, hi) in &moving {
                    if !cull.sees(*lo, *hi) {
                        continue;
                    }
                    match lod(*lo, *hi) {
                        Some(true) => near.push(*inst),
                        Some(false) => far.push(*inst),
                        None => {}
                    }
                }
                dynamic.resize(r * m.reserve, InstanceData::new(&Affine3A::IDENTITY, [0.0; 4]));
                ModelDraw::push(&mut d.near, base..base + near.len() as u32);
                ModelDraw::push(&mut d.far, base + near.len() as u32..base + (near.len() + far.len()) as u32);
                dynamic.extend(near);
                dynamic.extend(far);
            }
            if !dynamic.is_empty() {
                self.queue.write_buffer(&m.instances, m.statics as u64 * INSTANCE_STRIDE, bytemuck::cast_slice(&dynamic));
            }
            model_draws.push(draws);
        }
        let mut lights = [GpuLight::zeroed(); MAX_LIGHTS];
        for (i, l) in f.lights.iter().take(MAX_LIGHTS).enumerate() {
            lights[i] = GpuLight { pos_radius: [l.pos.x, l.pos.y, l.pos.z, l.radius], color_intensity: [l.color.x, l.color.y, l.color.z, l.intensity] };
        }
        let mut smoke = [[0.0f32; 4]; MAX_SMOKE];
        for (i, s) in f.smoke.iter().take(MAX_SMOKE).enumerate() {
            smoke[i] = s.to_array();
        }
        let v4 = |v: Vec3, w: f32| [v.x, v.y, v.z, w];
        let mut palette = [[0.0; 4]; PALETTE];
        for (i, c) in f.palette.iter().enumerate() {
            palette[i] = v4(*c, 1.0);
        }
        let mut tex_scale = [[1.0f32; 4]; 16];
        for (i, s) in self.materials.scales.iter().enumerate().take(LAYERS as usize) {
            tex_scale[i / 4][i % 4] = *s;
        }
        let wx = &f.weather;
        let mut room_info = [0.0f32; 4];
        let mut room_box = [[0.0f32; 4]; MAX_ROOMS];
        let mut room_y = [[0.0f32; 4]; MAX_ROOMS / 2];
        let mut light_rooms = [[u32::MAX; 4]; MAX_LIGHTS / 2];
        if let Some(r) = &f.rooms {
            let n = r.boxes.len().min(MAX_ROOMS);
            room_info[0] = n as f32;
            for (i, (lo, hi, y0, y1)) in r.boxes.iter().take(n).enumerate() {
                room_box[i] = [lo.x, lo.y, hi.x, hi.y];
                room_y[i / 2][(i % 2) * 2] = *y0;
                room_y[i / 2][(i % 2) * 2 + 1] = *y1;
            }
            for (i, m) in r.masks.iter().take(MAX_LIGHTS).enumerate() {
                light_rooms[i / 2][(i % 2) * 2] = *m as u32;
                light_rooms[i / 2][(i % 2) * 2 + 1] = (*m >> 32) as u32;
            }
        }
        let globals = Globals {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            sun_vp: sun_vp.to_cols_array_2d(),
            camera: v4(f.camera, f.time),
            sun_dir: v4(f.sun_dir, if sunlit { f.sun_intensity } else { 0.0 }),
            sun_color: v4(f.sun_color, f.night),
            sky_top: v4(f.sky_top, 0.0),
            sky_horizon: v4(
                f.sky_horizon,
                if !q.fog {
                    0.0
                } else if far.is_finite() && f.fog_density > 0.0 {
                    (f.fog_density * 3.0).max(REDUCED_FOG / far)
                } else {
                    f.fog_density
                },
            ),
            ground_bounce: v4(f.ground_bounce, f.fog_falloff),
            fog_color: v4(f.fog_color, f.windows_lit),
            params: [f.exposure, f.lights.len().min(q.lights as usize).min(MAX_LIGHTS) as f32, 1.0 / res as f32, f.street_lights],
            weather: [wx.wet, wx.snow_cover, wx.rain, wx.snow],
            screen: [w, h, 1.0 / w, 1.0 / h],
            misc: [wx.wind, self.scene.samples as f32, SHADOW_FADE, SHADOW_FAR],
            sky: [wx.cloud, f.aurora, wx.lightning, f.debug as f32],
            tex_scale,
            palette,
            lights,
            smoke,
            room_info,
            room_box,
            room_y,
            light_rooms,
            quality: [
                if q.textures { 1.0 } else { 0.0 },
                if self.bench.flat { 1.0 } else { 0.0 },
                if !shadows {
                    0.0
                } else if q.soft_shadow {
                    2.0
                } else {
                    1.0
                },
                if q.ground { 1.0 } else { 0.0 },
            ],
            sun_vp2: sun_vp2.to_cols_array_2d(),
            shadow: [1.0 / self.shadow_size.0 as f32, 1.0 / self.shadow_size.1 as f32, cascades as f32, SHADOW_FAR2],
        };
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        // Per pass for partiklene: regn, snø og røyk.
        // Den fjerde er kaskade to i skyggepasset (x = 1).
        let mut pass_data = vec![0u8; PASS_STRIDE as usize * 4];
        for (i, v) in [[0.0, wx.rain, 0.0, 0.0], [1.0, wx.snow, 0.0, 0.0], [2.0, 1.0, 0.0, 0.0f32], [1.0, 0.0, 0.0, 0.0]].iter().enumerate() {
            let at = i * PASS_STRIDE as usize;
            pass_data[at..at + 16].copy_from_slice(bytemuck::cast_slice(v));
        }
        self.queue.write_buffer(&self.pass_buf, 0, &pass_data);

        // Den faste verdenen har én instans uten transform.
        let inst = [InstanceData::new(&Affine3A::IDENTITY, [0.0; 4])];
        self.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&inst));

        // Leddpaletten: matrisene kolonne for kolonne, fire texels hver.
        if !f.skins.is_empty() {
            let n = f.skins.len().min(SKIN_MAX);
            if f.skins.len() > SKIN_MAX {
                log::warn!("For mange leddmatriser: {} av {}", f.skins.len(), SKIN_MAX);
            }
            let rows = (n as u32).div_ceil(SKIN_W / 4);
            let mut data = vec![0f32; (rows * SKIN_W * 4) as usize];
            for (i, m) in f.skins.iter().take(n).enumerate() {
                data[i * 16..i * 16 + 16].copy_from_slice(&m.to_cols_array());
            }
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &self.skins, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                bytemuck::cast_slice(&data),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(SKIN_W * 16), rows_per_image: Some(rows) },
                wgpu::Extent3d { width: SKIN_W, height: rows, depth_or_array_layers: 1 },
            );
        }

        if let Some(tm) = &mut self.timer {
            let _ = self.device.poll(wgpu::PollType::Poll);
            tm.collect();
        }
        let ts = |k: usize, begin: bool, end: bool| self.timer.as_ref().filter(|_| self.bench.timers).and_then(|tm| tm.writes(k, begin, end));
        let mut stats = [PassStats::default(); 2];
        let mut enc = self.device.create_command_encoder(&Default::default());

        // 1. Skyggen (kartet tømmes også nede, der den ikke tegnes).
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: ts(0, true, true),
                occlusion_query_set: None,
            });
            if shadows {
                pass.set_bind_group(0, &self.globals_bg, &[]);
                pass.set_bind_group(2, &self.materials.bind_group, &[]);
                pass.set_bind_group(3, &self.materials.dummy_model, &[]);
                for k in 0..cascades {
                    // Hver kaskade i sin del av kartet; pass-verdien x sier
                    // hvilken matrise vertexshaderen bruker.
                    pass.set_viewport((k * res) as f32, 0.0, res as f32, res as f32, 0.0, 1.0);
                    pass.set_bind_group(1, &self.shadow_frame_bg, &[if k == 0 { 0 } else { (3 * PASS_STRIDE) as u32 }]);
                    let s = self.draw_all(&mut pass, &sun_cull, &model_draws, 0, &self.shadow_pipe, &self.shadow_cutout);
                    stats[0].add(&s);
                }
            }
        }

        // 2. Verden, himmel og nedbør, med MSAA løst opp i `hdr`.
        {
            let (view, resolve_target, store) = match &t.color {
                Some(msaa) => (msaa, Some(&t.hdr), wgpu::StoreOp::Discard),
                None => (&t.hdr, None, wgpu::StoreOp::Store),
            };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, resolve_target, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    // Okklusjonen leser dybden etterpå.
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: if t.ao.is_some() { wgpu::StoreOp::Store } else { wgpu::StoreOp::Discard } }),
                    stencil_ops: None,
                }),
                timestamp_writes: ts(1, true, true),
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_bind_group(1, &self.frame_bg, &[0]);
            pass.set_bind_group(2, &self.materials.bind_group, &[]);
            pass.set_bind_group(3, &self.materials.dummy_model, &[]);
            stats[1] = self.draw_all(&mut pass, &cam_cull, &model_draws, 1, &self.scene.world, &self.scene.cutout);
            self.triangles.set((stats[1].world, 0, stats[1].models));
            if self.bench.sky {
                pass.set_pipeline(&self.scene.sky);
                pass.draw(0..3, 0..1);
            }
            pass.set_pipeline(&self.scene.particles);
            let particles = q.particles && self.bench.particles;
            if !f.smoke.is_empty() && particles {
                pass.set_bind_group(1, &self.frame_bg, &[(2 * PASS_STRIDE) as u32]);
                pass.draw(0..4, 0..f.smoke.len() as u32 * SMOKE_PUFFS);
            }
            if wx.rain > 0.0 && particles {
                pass.set_bind_group(1, &self.frame_bg, &[0]);
                pass.draw(0..4, 0..RAIN_COUNT);
            }
            if wx.snow > 0.0 && particles {
                pass.set_bind_group(1, &self.frame_bg, &[PASS_STRIDE as u32]);
                pass.draw(0..4, 0..SNOW_COUNT);
            }
            if line_count > 0 {
                if let Some((b, _)) = self.line_buf.borrow().as_ref() {
                    pass.set_bind_group(1, &self.frame_bg, &[0]);
                    pass.set_pipeline(&self.scene.lines);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..line_count, 0..1);
                }
            }
        }

        // 3. Glød og tonemapping til skjermen.
        // Solstrålene: sola på skjermen, sterkest midt i bildet og borte
        // når den er bak deg, under horisonten eller det er natt.
        let sun_post = {
            let c = view_proj * (f.camera + f.sun_dir * 1000.0).extend(1.0);
            let mut out = [0.5, 0.5, 0.0, if q.grade { 1.0 } else { 0.0 }];
            if q.rays && sunlit && c.w > 0.0 && f.sun_dir.y > 0.0 {
                let ndc = c.truncate() / c.w;
                let off = ndc.x.abs().max(ndc.y.abs());
                let k = (1.0 - ((off - 0.9) / 0.6).clamp(0.0, 1.0)) * (f.sun_dir.y / 0.06).clamp(0.0, 1.0) * (1.0 - f.night).clamp(0.0, 1.0);
                out = [ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, k, out[3]];
            }
            out
        };
        let post = |buf: &wgpu::Buffer, params: [f32; 4], sw: u32, sh: u32| {
            let u = PostUniform { params, texel: [sw as f32, sh as f32, 1.0 / sw as f32, 1.0 / sh as f32], sun: sun_post, extra: [if f.gamma > 0.0 { f.gamma } else { 1.0 }, 0.0, 0.0, 0.0] };
            self.queue.write_buffer(buf, 0, bytemuck::bytes_of(&u));
        };
        let fullscreen = |enc: &mut wgpu::CommandEncoder, label: &str, target: &wgpu::TextureView, pipe: &wgpu::RenderPipeline, bg: &wgpu::BindGroup, load: wgpu::LoadOp<wgpu::Color>, timestamps: Option<wgpu::RenderPassTimestampWrites>| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: timestamps,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_bind_group(1, bg, &[]);
            pass.draw(0..3, 0..1);
        };
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        // Enkel grafikk: ingen glød (tonemappingen får styrke 0).
        let bloom = q.bloom;
        // Skygge i krokene før gløden (tidsmålingen for etterbehandlingen
        // begynner da her).
        let ao_ran = match (&t.ao, &self.ao) {
            (Some(at), Some(a)) => {
                a.run(&mut enc, &self.globals_bg, at, ts(2, true, false));
                true
            }
            _ => false,
        };
        for i in (0..BLOOM_LEVELS).filter(|_| bloom) {
            let (bg, sw, sh) = if i == 0 { (&t.down0_bg, t.width, t.height) } else { (&t.down_bg[i - 1], t.bloom[i - 1].1, t.bloom[i - 1].2) };
            post(&bg.1, [0.0, 0.0, 0.0, if i == 0 { 1.0 } else { 0.0 }], sw, sh);
            fullscreen(&mut enc, "bloom down", &t.bloom[i].0, &self.down_pipe, &bg.0, clear, if i == 0 { ts(2, !ao_ran, false) } else { None });
        }
        for (k, i) in (1..BLOOM_LEVELS).rev().enumerate().filter(|_| bloom) {
            post(&t.up_bg[k].1, [0.0; 4], t.bloom[i].1, t.bloom[i].2);
            fullscreen(&mut enc, "bloom up", &t.bloom[i - 1].0, &self.up_pipe, &t.up_bg[k].0, wgpu::LoadOp::Load, None);
        }
        match &t.smaa {
            // Med SMAA skriver tonemappingen gamma-kodet til mellombildet,
            // og kantutjevningen skriver til skjermen.
            Some(st) => {
                post(&t.tonemap_bg.1, [f.exposure, if bloom { f.bloom } else { 0.0 }, 1.0, f.hurt], t.width, t.height);
                fullscreen(&mut enc, "tonemap", &st.ldr, &self.tonemap_ldr, &t.tonemap_bg.0, clear, ts(2, !bloom && !ao_ran, false));
                self.smaa.run(&mut enc, st, out, ts(2, false, true));
            }
            None => {
                post(&t.tonemap_bg.1, [f.exposure, if bloom { f.bloom } else { 0.0 }, if self.encode_srgb { 1.0 } else { 0.0 }, f.hurt], t.width, t.height);
                fullscreen(&mut enc, "tonemap", out, &self.tonemap_pipe, &t.tonemap_bg.0, clear, ts(2, !bloom && !ao_ran, true));
            }
        }

        if let Some(tm) = &mut self.timer {
            tm.resolve(&mut enc);
        }
        self.pass_stats.set(stats);
        self.queue.submit([enc.finish()]);
        if let Some(tm) = &mut self.timer {
            tm.map();
        }
    }

    /// Tegn verden og modellene. Verden tegnes bit for bit, bare
    /// de bitene som er innenfor synsfeltet, og småtingene bare nær
    /// kameraet. Naboer som begge skal tegnes slås sammen til ett kall.
    /// Returnerer hva som ble tegnet.
    /// Tegner bymeshen og modellene: det ugjennomsiktige med `opaque` først,
    /// så alfakortene med `cutout` (også bymeshen, om den har alfakort).
    fn draw_all(&self, pass: &mut wgpu::RenderPass, cull: &Cull, model_draws: &[[ModelDraw; REGIONS as usize]], region: usize, opaque: &wgpu::RenderPipeline, cutout: &wgpu::RenderPipeline) -> PassStats {
        pass.set_pipeline(if self.world_cutout { cutout } else { opaque });
        let mut calls = 0u32;
        let mut instances = 0u32;
        pass.set_vertex_buffer(1, self.instances.slice(0..INSTANCE_STRIDE));
        self.world.bind(pass);
        let mut indices = 0u32;
        let mut run: Option<std::ops::Range<u32>> = None;
        let mut flush = |run: &mut Option<std::ops::Range<u32>>, pass: &mut wgpu::RenderPass| {
            if let Some(r) = run.take() {
                if !r.is_empty() {
                    indices += r.end - r.start;
                    calls += 1;
                    pass.draw_indexed(r, 0, 0..1);
                }
            }
        };
        for c in &self.chunks {
            if !cull.sees(c.min, c.max) {
                flush(&mut run, pass);
                continue;
            }
            let fine = cull.fine_within > 0.0 && c.distance(cull.eye) < cull.fine_within;
            let range = if fine { c.coarse.start..c.fine.end } else { c.coarse.clone() };
            match &mut run {
                Some(r) if r.end == range.start => r.end = range.end,
                _ => {
                    flush(&mut run, pass);
                    run = Some(range);
                }
            }
        }
        flush(&mut run, pass);
        let world = indices / 3;
        let mut models = 0;
        // Først de ugjennomsiktige, så alfakortene med sin pipeline.
        let order = self.models.iter().zip(model_draws).filter(|(m, _)| !m.cutout).chain(self.models.iter().zip(model_draws).filter(|(m, _)| m.cutout));
        pass.set_pipeline(opaque);
        let mut switched = false;
        for (m, draws) in order {
            let d = &draws[region];
            // Målingen kan ta modellene ut av hovedpasset.
            if (d.near.is_empty() && d.far.is_empty()) || (region == 1 && !self.bench.models) {
                continue;
            }
            if m.cutout && !switched {
                pass.set_pipeline(cutout);
                switched = true;
            }
            pass.set_bind_group(3, &m.textures, &[]);
            pass.set_vertex_buffer(1, m.instances.slice(..));
            m.mesh.bind(pass);
            for r in &d.near {
                pass.draw_indexed(m.near.clone(), 0, r.clone());
                models += (m.near.end - m.near.start) / 3 * r.len() as u32;
                calls += 1;
                instances += r.len() as u32;
            }
            for r in &d.far {
                pass.draw_indexed(m.far.clone(), 0, r.clone());
                models += (m.far.end - m.far.start) / 3 * r.len() as u32;
                calls += 1;
                instances += r.len() as u32;
            }
        }
        PassStats { world, models, calls, instances }
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
