//! Rendereren. Rekkefølgen per bilde:
//!
//! 1. Skygger fra sola eller månen: tre kaskader inn i ett dybdeatlas.
//! 2. Dybdeprepass som også skriver normalene.
//! 3. SSAO i halv oppløsning, uskarpet med hensyn til dybden.
//! 4. Verden i HDR med dybdetest «lik», så bare synlige piksler skyggelegges,
//!    deretter himmelen og regn eller snø.
//! 5. Tåke med lysstråler fra skyggekartet og lys fra lyktene.
//! 6. TAA: bildet blandes med forrige, reprojisert gjennom dybden.
//! 7. Glød og tonemapping til skjermen.

use crate::domain::geometry::{Chunk, Layer, Mesh, Vertex, LAYERS};
use crate::domain::model::LayerImages;
use crate::ports;
pub use crate::domain::scene::{Frame, Weather};
use crate::domain::scene::{Frustum, GRID, MAX_LIGHTS, MAX_SMOKE, PALETTE};
use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Mat4, Vec3, Vec4};
use texture::Materials;
use wgpu::util::DeviceExt;

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
const NORMAL: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Dybden som fargetekstur, så etterbehandlingen kan lese den i WebGL2.
const DEPTH_LIN: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
/// Bevegelsen på skjermen siden forrige bilde (uv), til TAA.
const VELOCITY: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
const AO: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
const SHADOW_ATLAS: u32 = 4096;
const CASCADE_RES: u32 = SHADOW_ATLAS / 2;
/// Hvor kaskadene slutter, i meter fra kameraet.
pub const CASCADES: [f32; 3] = [14.0, 45.0, 170.0];
const BLOOM_LEVELS: usize = 5;
/// Skarpingen etter TAA (0..1), som tar igjen det TAA visker ut.
const SHARPEN: f32 = 0.6;
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
    prev_view_proj: [[f32; 4]; 4],
    sun_vp: [[[f32; 4]; 4]; 3],
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
    jitter: [f32; 4],
    cascade: [f32; 4],
    sky: [f32; 4],
    tex_scale: [[f32; 4]; 7],
    palette: [[f32; 4]; PALETTE],
    lights: [GpuLight; MAX_LIGHTS],
    smoke: [[f32; 4]; MAX_SMOKE],
    /// Kjellerens rutenett: hjørnet (x, z), gulvhøyden og cellestørrelsen
    /// (0 utenfor kjelleren), og cellene, fire per u32.
    grid_info: [f32; 4],
    grid: [[u32; 4]; GRID * GRID / 16],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostUniform {
    params: [f32; 4],
    texel: [f32; 4],
    /// x: skarpingen etter TAA (bare tonemappingen leser den).
    extra: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
    pub extra: [f32; 4],
    /// x: plassen i leddpaletten + 1 for skinnede instanser, 0 ellers.
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

/// Pass-regioner i instansbufferet: tre kaskader og kameraet.
const REGIONS: u32 = 4;

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
    depth: wgpu::TextureView,
    depth_lin: wgpu::TextureView,
    normal: wgpu::TextureView,
    velocity: wgpu::TextureView,
    hdr: wgpu::TextureView,
    hdr_fog: wgpu::TextureView,
    resolved: [wgpu::TextureView; 2],
    ao: [wgpu::TextureView; 2],
    bloom: Vec<(wgpu::TextureView, u32, u32)>,
    frame_bg: wgpu::BindGroup,
    /// Til skyggepasset, som ikke kan lese atlaset det tegner til.
    shadow_frame_bg: wgpu::BindGroup,
    /// Til etterbehandlingen, som skriver til SSAO-teksturen.
    post_frame_bg: wgpu::BindGroup,
    ssao_bg: (wgpu::BindGroup, wgpu::Buffer),
    blur_bg: [(wgpu::BindGroup, wgpu::Buffer); 2],
    fog_bg: (wgpu::BindGroup, wgpu::Buffer),
    taa_bg: [(wgpu::BindGroup, wgpu::Buffer); 2],
    down0_bg: [(wgpu::BindGroup, wgpu::Buffer); 2],
    down_bg: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
    up_bg: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
    tonemap_bg: [(wgpu::BindGroup, wgpu::Buffer); 2],
}

/// Redusert grafikk: hvor langt unna modellene tegnes (m), og hvor tett
/// tåka minst er, så kanten ikke synes. Bakken og fjellene tegnes som før.
const REDUCED_FAR: f32 = 160.0;
const REDUCED_FOG: f32 = 0.005;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    encode_srgb: bool,
    globals: wgpu::Buffer,
    /// Én per leddpalett: nå i binding 1, forrige bilde i binding 2.
    globals_bg: [wgpu::BindGroup; 2],
    pass_buf: wgpu::Buffer,
    shadow_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    frame_layout: wgpu::BindGroupLayout,
    post_layout: wgpu::BindGroupLayout,
    materials: Materials,
    shadow_pipe: wgpu::RenderPipeline,
    prepass_pipe: wgpu::RenderPipeline,
    world_pipe: wgpu::RenderPipeline,
    sky_pipe: wgpu::RenderPipeline,
    particle_pipe: wgpu::RenderPipeline,
    ssao_pipe: wgpu::RenderPipeline,
    blur_pipe: wgpu::RenderPipeline,
    fog_pipe: wgpu::RenderPipeline,
    taa_pipe: wgpu::RenderPipeline,
    down_pipe: wgpu::RenderPipeline,
    up_pipe: wgpu::RenderPipeline,
    tonemap_pipe: wgpu::RenderPipeline,
    linear: wgpu::Sampler,
    nearest: wgpu::Sampler,
    black: wgpu::TextureView,
    world: GpuMesh,
    chunks: Vec<Chunk>,
    instances: wgpu::Buffer,
    models: Vec<GpuModel>,
    /// Leddpalettene (se `SKIN_W`), annethvert bilde, så forrige bildes
    /// matriser kan leses til bevegelsesvektorene.
    skins: [wgpu::Texture; 2],
    /// Hvor de bevegelige instansene lå i paletten forrige bilde: (modell,
    /// nummer blant modellens instanser) → (plass for transformen, skinnet).
    prev_moving: std::collections::HashMap<(usize, usize), (u32, Option<u32>)>,
    targets: Targets,
    frame_index: u32,
    prev_view_proj: Mat4,
    history_valid: bool,
    current: Option<wgpu::TextureView>,
    /// Intern oppløsning som andel av vinduet.
    scale: f32,
    /// Kvalitet 0–1: antall prøver i skygger, SSAO og tåkemarsj.
    quality: f32,
    /// Redusert grafikk: to skyggekaskader, ingen SSAO eller glød, færre
    /// lys, grovere detalj tidligere og småting bare nær.
    reduced: bool,
    out_size: (u32, u32),
    /// Trekanter tegnet i hovedpasset sist bilde: verden, props, modeller.
    triangles: std::cell::Cell<(u32, u32, u32)>,
    /// Tiden på grafikkortet per del, når kortet kan måle den.
    timer: Option<timer::GpuTimer>,
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

fn halton(index: u32, base: u32) -> f32 {
    let mut f = 1.0;
    let mut r = 0.0;
    let mut i = index;
    while i > 0 {
        f /= base as f32;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        _adapter: &wgpu::Adapter,
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
        let skin_tex = || device.create_texture(&wgpu::TextureDescriptor {
            label: Some("skins"),
            size: wgpu::Extent3d { width: SKIN_W, height: SKIN_ROWS, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let skins = [skin_tex(), skin_tex()];
        let skin_views = [skins[0].create_view(&Default::default()), skins[1].create_view(&Default::default())];
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
            ],
        });
        let globals_bg = [0, 1].map(|k| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("globals"),
                layout: &globals_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&skin_views[k]) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&skin_views[1 - k]) },
                ],
            })
        });

        // Per pass: kaskadenummer for skyggene, regn og snø for partiklene.
        let pass_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pass"),
            size: PASS_STRIDE * 6,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow"),
            size: wgpu::Extent3d { width: SHADOW_ATLAS, height: SHADOW_ATLAS, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_tex.create_view(&Default::default());
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
            entries: &[
                tex_entry(0, wgpu::TextureSampleType::Depth),
                sampler_entry(1, wgpu::SamplerBindingType::Comparison),
                tex_entry(2, FILTER),
                sampler_entry(3, wgpu::SamplerBindingType::Filtering),
                uniform_entry(4, true),
            ],
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                tex_entry(0, FILTER),
                tex_entry(1, FILTER),
                tex_entry(2, FILTER),
                tex_entry(3, wgpu::TextureSampleType::Float { filterable: false }),
                sampler_entry(4, wgpu::SamplerBindingType::Filtering),
                sampler_entry(5, wgpu::SamplerBindingType::NonFiltering),
                uniform_entry(6, false),
            ],
        });

        let noise = include_str!("shaders/noise.wgsl");
        let common = include_str!("shaders/common.wgsl");
        let shadow_src = include_str!("shaders/shadow.wgsl");
        let world_mod = shader(&device, "world", &[noise, common, shadow_src, include_str!("shaders/world.wgsl")]);
        let sky_mod = shader(&device, "sky", &[noise, common, include_str!("shaders/sky.wgsl")]);
        let particle_mod = shader(&device, "particles", &[noise, common, include_str!("shaders/particles.wgsl")]);
        let post_mod = shader(&device, "post", &[noise, common, shadow_src, include_str!("shaders/post.wgsl")]);

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
            bind_group_layouts: &[&globals_layout, &frame_layout, &post_layout],
            push_constant_ranges: &[],
        });

        let depth_state = |write: bool, compare: wgpu::CompareFunction, bias: wgpu::DepthBiasState| {
            Some(wgpu::DepthStencilState { format: DEPTH, depth_write_enabled: write, depth_compare: compare, stencil: Default::default(), bias })
        };
        let buffers = [VERTEX_LAYOUT, InstanceData::LAYOUT];
        let shadow_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&world_layout),
            vertex: wgpu::VertexState { module: &world_mod, entry_point: Some("vs_shadow"), compilation_options: Default::default(), buffers: &buffers },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: depth_state(true, wgpu::CompareFunction::Less, wgpu::DepthBiasState { constant: 2, slope_scale: 2.5, clamp: 0.0 }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &world_mod, entry_point: Some("fs_shadow"), compilation_options: Default::default(), targets: &[] }),
            multiview: None,
            cache: None,
        });
        let prepass_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("prepass"),
            layout: Some(&world_layout),
            vertex: wgpu::VertexState { module: &world_mod, entry_point: Some("vs_prepass"), compilation_options: Default::default(), buffers: &buffers },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: depth_state(true, wgpu::CompareFunction::Less, Default::default()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &world_mod,
                entry_point: Some("fs_prepass"),
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState { format: NORMAL, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    Some(wgpu::ColorTargetState { format: DEPTH_LIN, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    Some(wgpu::ColorTargetState { format: VELOCITY, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                ],
            }),
            multiview: None,
            cache: None,
        });
        let world_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("world"),
            layout: Some(&world_layout),
            vertex: wgpu::VertexState { module: &world_mod, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &buffers },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: depth_state(false, wgpu::CompareFunction::Equal, Default::default()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &world_mod,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: HDR, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });
        let sky_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky"),
            layout: Some(&simple_layout),
            vertex: wgpu::VertexState { module: &sky_mod, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: Default::default(),
            depth_stencil: depth_state(false, wgpu::CompareFunction::LessEqual, Default::default()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sky_mod,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: HDR, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });
        let particle_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("particles"),
            layout: Some(&simple_layout),
            vertex: wgpu::VertexState { module: &particle_mod, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, cull_mode: None, ..Default::default() },
            depth_stencil: depth_state(false, wgpu::CompareFunction::Less, Default::default()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &particle_mod,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: HDR, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

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
        let ssao_pipe = post_pipe("fs_ssao", AO, None);
        let blur_pipe = post_pipe("fs_blur", AO, None);
        let fog_pipe = post_pipe("fs_fog", HDR, None);
        let taa_pipe = post_pipe("fs_taa", HDR, None);
        let down_pipe = post_pipe("fs_down", HDR, None);
        let up_pipe = post_pipe("fs_up", HDR, Some(additive));
        let tonemap_pipe = post_pipe("fs_tonemap", output, None);

        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let black = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("black"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: HDR,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());

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
                let (lo, hi) = m.mesh.bounds();
                let skinned = m.rig.is_some();
                // En skinnet figur strekker seg utenfor hvileposen når den
                // angriper eller faller; boksen får halvannen gang plassen.
                let (lo, hi) = if skinned {
                    let (c, h) = ((lo + hi) * 0.5, (hi - lo) * 0.75);
                    (c - h, c + h)
                } else {
                    (lo, hi)
                };
                // Lakkfargen pakkes som 8 bit per kanal i et heltall (eksakt i f32).
                let packed = |tint: &Vec3| {
                    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
                    (c(tint.x) << 16 | c(tint.y) << 8 | c(tint.z)) as f32
                };
                let placements: Vec<(InstanceData, Vec3, Vec3)> = pl
                    .transforms
                    .iter()
                    .zip(pl.tints.iter().chain(std::iter::repeat(&Vec3::ONE)))
                    .zip(pl.looks.iter().chain(std::iter::repeat(&None)))
                    .map(|((t, tint), look)| {
                        let (wlo, whi) = world_aabb(t, lo, hi);
                        let inst = match look {
                            // Utseendet: lagene for de fire rollene (5 bit hver, bit 20
                            // slår det på), tonen, og flammefargen i extra2.y.
                            Some(l) => {
                                let roles = l.layers.iter().enumerate().fold(1u32 << 20, |acc, (k, layer)| acc | (*layer as u32 & 31) << (5 * k));
                                let mut inst = InstanceData::new(t, [0.0, packed(tint), roles as f32, packed(&l.tint)]);
                                inst.extra2[1] = packed(&l.flame);
                                inst
                            }
                            None => InstanceData::new(t, [0.0, packed(tint), m.roughness, m.metallic]),
                        };
                        (inst, wlo, whi)
                    })
                    .collect();
                // De faste instansene sorteres i ruter (rad for rad) og lastes
                // opp én gang; bak dem står plassen til de bevegelige.
                let key = |(_, lo, hi): &(InstanceData, Vec3, Vec3)| {
                    let c = (*lo + *hi) * 0.5 / CELL_M;
                    (c.z.floor() as i32, c.x.floor() as i32)
                };
                let mut placements = placements;
                placements.sort_by_key(key);
                let mut cells: Vec<InstanceCell> = vec![];
                let mut last = None;
                for (k, p) in placements.iter().enumerate() {
                    let id = key(p);
                    match cells.last_mut() {
                        Some(c) if last == Some(id) => {
                            c.lo = c.lo.min(p.1);
                            c.hi = c.hi.max(p.2);
                            c.range.end = k as u32 + 1;
                        }
                        _ => cells.push(InstanceCell { lo: p.1, hi: p.2, range: k as u32..k as u32 + 1 }),
                    }
                    last = Some(id);
                }
                let statics = placements.len() as u32;
                let mut data: Vec<InstanceData> = placements.iter().map(|p| p.0).collect();
                data.resize(placements.len() + pl.reserve * REGIONS as usize, InstanceData::new(&Affine3A::IDENTITY, [0.0; 4]));
                if data.is_empty() {
                    data.push(InstanceData::new(&Affine3A::IDENTITY, [0.0; 4]));
                }
                // Nær og fjern detalj i samme buffer.
                let size = (hi - lo).max_element();
                // Figurene får også et grovt nivå (leddene og vektene følger med
                // fra hjørnet som blir stående), så de ikke tegnes med full
                // detalj langt unna og i skyggene.
                let far = m.far_mesh.clone().unwrap_or_else(|| m.mesh.clustered(if m.far_cell > 0.0 { m.far_cell } else { size / 70.0 }));
                let mut combined = m.mesh.clone();
                let base = combined.vertices.len() as u32;
                combined.vertices.extend_from_slice(&far.vertices);
                combined.indices.extend(far.indices.iter().map(|i| i + base));
                let near_count = m.mesh.indices.len() as u32;
                log::info!("Modell {}: {} trekanter nær, {} fjern", m.name, near_count / 3, far.indices.len() / 3);
                Some(GpuModel {
                    name: m.name.clone(),
                    mesh: GpuMesh::new(&device, &combined, &m.name),
                    near: 0..near_count,
                    far: near_count..combined.indices.len() as u32,
                    textures: materials.model_bind_group(&device, &queue, &mipper, m),
                    instances: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&m.name),
                        contents: bytemuck::cast_slice(&data),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    }),
                    cells,
                    statics,
                    reserve: pl.reserve,
                    bounds: (lo, hi),
                    extra: [0.0, packed(&Vec3::ONE), m.roughness, m.metallic],
                    skinned,
                    small: !skinned && size < 1.3,
                })
            })
            .collect();
        log::info!(
            "Verden: {} hjørner, {} trekanter, {} modeller, {} faste instanser i {} ruter, plass til {} bevegelige",
            scene.mesh.vertices.len(),
            scene.mesh.indices.len() / 3,
            gpu_models.len(),
            gpu_models.iter().map(|m| m.statics).sum::<u32>(),
            gpu_models.iter().map(|m| m.cells.len()).sum::<usize>(),
            gpu_models.iter().map(|m| m.reserve).sum::<usize>()
        );

        let targets = Self::make_targets(&device, &frame_layout, &post_layout, &shadow_view, &shadow_sampler, &pass_buf, &linear, &nearest, &black, width, height);
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
            shadow_sampler,
            frame_layout,
            post_layout,
            materials,
            shadow_pipe,
            prepass_pipe,
            world_pipe,
            sky_pipe,
            particle_pipe,
            ssao_pipe,
            blur_pipe,
            fog_pipe,
            taa_pipe,
            down_pipe,
            up_pipe,
            tonemap_pipe,
            linear,
            nearest,
            black,
            world: world_mesh,
            chunks,
            instances,
            models: gpu_models,
            skins,
            prev_moving: Default::default(),
            targets,
            frame_index: 0,
            prev_view_proj: Mat4::IDENTITY,
            history_valid: false,
            current: None,
            scale: 1.0,
            quality: 1.0,
            reduced: false,
            out_size: (width, height),
            triangles: std::cell::Cell::new((0, 0, 0)),
            timer,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn make_targets(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        post_layout: &wgpu::BindGroupLayout,
        shadow_view: &wgpu::TextureView,
        shadow_sampler: &wgpu::Sampler,
        pass_buf: &wgpu::Buffer,
        linear: &wgpu::Sampler,
        nearest: &wgpu::Sampler,
        black: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Targets {
        let width = width.max(1);
        let height = height.max(1);
        let tex = |label: &str, w: u32, h: u32, format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let depth = tex("depth", width, height, DEPTH);
        let depth_lin = tex("depth lin", width, height, DEPTH_LIN);
        let normal = tex("normal", width, height, NORMAL);
        let velocity = tex("velocity", width, height, VELOCITY);
        let hdr = tex("hdr", width, height, HDR);
        let hdr_fog = tex("hdr fog", width, height, HDR);
        let resolved = [tex("resolved 0", width, height, HDR), tex("resolved 1", width, height, HDR)];
        let (hw, hh) = ((width / 2).max(1), (height / 2).max(1));
        let ao = [tex("ao 0", hw, hh, AO), tex("ao 1", hw, hh, AO)];
        let mut bloom = vec![];
        let (mut w, mut h) = (width, height);
        for i in 0..BLOOM_LEVELS {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            bloom.push((tex(&format!("bloom {i}"), w, h, HDR), w, h));
        }

        let frame_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame"),
            layout: frame_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(shadow_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(shadow_sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&ao[0]) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(linear) },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: pass_buf, offset: 0, size: wgpu::BufferSize::new(16) }),
                },
            ],
        });

        let dummy_depth = tex("dummy depth", 1, 1, DEPTH);
        let shadow_frame_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow frame"),
            layout: frame_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&dummy_depth) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(shadow_sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&ao[0]) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(linear) },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: pass_buf, offset: 0, size: wgpu::BufferSize::new(16) }),
                },
            ],
        });

        let post_frame_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post frame"),
            layout: frame_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(shadow_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(shadow_sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(black) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(linear) },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: pass_buf, offset: 0, size: wgpu::BufferSize::new(16) }),
                },
            ],
        });

        let bind = |t0: &wgpu::TextureView, t1: &wgpu::TextureView, t2: &wgpu::TextureView| {
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
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(t2) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&depth_lin) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(linear) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(nearest) },
                    wgpu::BindGroupEntry { binding: 6, resource: buf.as_entire_binding() },
                ],
            });
            (bg, buf)
        };
        let ssao_bg = bind(&normal, black, black);
        let blur_bg = [bind(&ao[0], black, black), bind(&ao[1], black, black)];
        let fog_bg = bind(&hdr, black, black);
        let taa_bg = [bind(&hdr_fog, &resolved[1], &velocity), bind(&hdr_fog, &resolved[0], &velocity)];
        let down0_bg = [bind(&resolved[0], black, black), bind(&resolved[1], black, black)];
        let mut down_bg = vec![];
        for i in 1..BLOOM_LEVELS {
            down_bg.push(bind(&bloom[i - 1].0, black, black));
        }
        let mut up_bg = vec![];
        for i in (1..BLOOM_LEVELS).rev() {
            up_bg.push(bind(&bloom[i].0, black, black));
        }
        let tonemap_bg = [bind(&resolved[0], &bloom[0].0, black), bind(&resolved[1], &bloom[0].0, black)];
        Targets { width, height, depth, depth_lin, normal, velocity, hdr, hdr_fog, resolved, ao, bloom, frame_bg, shadow_frame_bg, post_frame_bg, ssao_bg, blur_bg, fog_bg, taa_bg, down0_bg, down_bg, up_bg, tonemap_bg }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.out_size = (width, height);
        let width = ((width as f32 * self.scale).round() as u32).max(1);
        let height = ((height as f32 * self.scale).round() as u32).max(1);
        if width == self.targets.width && height == self.targets.height {
            return;
        }
        self.targets = Self::make_targets(
            &self.device,
            &self.frame_layout,
            &self.post_layout,
            &self.shadow_view,
            &self.shadow_sampler,
            &self.pass_buf,
            &self.linear,
            &self.nearest,
            &self.black,
            width,
            height,
        );
        self.history_valid = false;
    }

    pub fn size(&self) -> (u32, u32) {
        (self.targets.width, self.targets.height)
    }

    /// Trekanter sist bilde: verden, props, modeller.
    /// Millisekunder på grafikkortet per del av bildet (tomt om kortet
    /// ikke kan måle det).
    pub fn pass_times(&self) -> Vec<(&'static str, f32)> {
        self.timer.as_ref().map(|tm| timer::SECTIONS.iter().copied().zip(tm.ms).collect()).unwrap_or_default()
    }

    pub fn stats(&self) -> (u32, u32, u32) {
        self.triangles.get()
    }

    pub fn set_quality(&mut self, quality: f32) {
        self.quality = quality.clamp(0.0, 1.0);
    }

    /// Redusert grafikk for svake maskiner (se `reduced`).
    pub fn set_reduced(&mut self, reduced: bool) {
        self.reduced = reduced;
    }

    pub fn set_scale(&mut self, scale: f32) {
        let scale = scale.clamp(0.4, 1.0);
        if (scale - self.scale).abs() < 1e-3 {
            return;
        }
        self.scale = scale;
        let (w, h) = self.out_size;
        self.resize(w, h);
    }

    /// Ortografisk projeksjon fra sola som dekker kamerafrustumet mellom
    /// `near` og `far`, låst til hele texels så kantene ikke skimrer.
    fn cascade(&self, f: &Frame, near: f32, far: f32) -> Mat4 {
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
        let texel = 2.0 * radius / CASCADE_RES as f32;
        let mut c = view.transform_point3(center);
        c.x = (c.x / texel).round() * texel;
        c.y = (c.y / texel).round() * texel;
        // z i lysrommet er negativt fremover. Ta med alt mellom lyset og sonen.
        let proj = Mat4::orthographic_rh(c.x - radius, c.x + radius, c.y - radius, c.y + radius, -c.z - radius - 200.0, -c.z + radius);
        proj * view
    }

    pub fn render(&mut self, out: &wgpu::TextureView, f: &Frame) {
        let t = &self.targets;
        let (w, h) = (t.width as f32, t.height as f32);
        let taa = f.taa && self.history_valid;
        let jitter = if f.taa {
            let i = self.frame_index % 8 + 1;
            [halton(i, 2) - 0.5, halton(i, 3) - 0.5]
        } else {
            [0.0, 0.0]
        };
        let view_proj = f.proj * f.view;
        let mut sun_vp = [[[0.0; 4]; 4]; 3];
        let mut sun_cull = [Cull { frustum: Frustum::from_view_proj(view_proj), eye: f.camera, fine_within: 0.0, band: f.band }; 3];
        let mut near = 0.1;
        for (i, far) in CASCADES.iter().enumerate() {
            let m = self.cascade(f, near, *far);
            sun_vp[i] = m.to_cols_array_2d();
            // Småting kaster skygge bare i den nærmeste kaskaden.
            sun_cull[i] = Cull { frustum: Frustum::from_view_proj(m), eye: f.camera, fine_within: if i == 0 { if self.reduced { 18.0 } else { 40.0 } } else { 0.0 }, band: f.band };
            near = far * 0.8;
        }
        let cam_cull = Cull { frustum: Frustum::from_view_proj(view_proj), eye: f.camera, fine_within: if self.reduced { 35.0 } else { 70.0 }, band: f.band };
        // Redusert: grovere detalj tidligere, småting bare nær, og bare to
        // skyggekaskader.
        let model_near = if self.reduced { MODEL_NEAR * 0.5 } else { MODEL_NEAR };
        // Nede i kjelleren når ikke sola ned: ingen skyggekaskader (atlaset
        // tømmes bare).
        let shadow_cascades = if f.grid.is_some() || f.band.1 < 0.0 { 0 } else if self.reduced { 2u32 } else { 3 };
        // Modellinstansene: hvilke som er synlige i hver region, nær og fjern.
        let culls = [sun_cull[0], sun_cull[1], sun_cull[2], cam_cull];
        // Paletten: skinnmatrisene først, så transformene til de bevegelige
        // instansene, som neste bilde leser som «forrige» til
        // bevegelsesvektorene.
        let pal = (self.frame_index % 2) as usize;
        let n_skins = f.skins.len().min(SKIN_MAX);
        let mut palette_extra: Vec<Mat4> = vec![];
        let mut moving_now: std::collections::HashMap<(usize, usize), (u32, Option<u32>)> = Default::default();
        let mut model_draws: Vec<[ModelDraw; REGIONS as usize]> = Vec::with_capacity(self.models.len());
        for (mi, m) in self.models.iter().enumerate() {
            // De bevegelige instansene (dyrene) legges til de faste, innenfor
            // plassen byen satte av.
            let moving: Vec<(InstanceData, Vec3, Vec3)> = f
                .instances
                .iter()
                .filter(|i| i.model == m.name)
                .take(m.reserve)
                .enumerate()
                .map(|(k, i)| {
                    let (lo, hi) = world_aabb(&i.transform, m.bounds.0, m.bounds.1);
                    // Blinket ligger over 1 i markeringen (extra.x).
                    let mut extra = m.extra;
                    extra[0] = if i.flash > 0.0 { 1.0 + i.flash } else { 0.0 };
                    if let Some(t) = i.tint {
                        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
                        extra[1] = (c(t.x) << 16 | c(t.y) << 8 | c(t.z)) as f32;
                    }
                    let skin = i.skin.map(|(base, _)| base).filter(|base| m.skinned && (*base as usize) < SKIN_MAX);
                    let mut inst = match skin {
                        Some(base) => InstanceData::skinned(&i.transform, extra, base),
                        None => InstanceData::new(&i.transform, extra),
                    };
                    // Forrige bildes transform og skinn (plass + 1, 0 er ingen).
                    if let Some((slot, prev_skin)) = self.prev_moving.get(&(mi, k)) {
                        inst.extra2[2] = *slot as f32 + 1.0;
                        if skin.is_some() {
                            inst.extra2[3] = prev_skin.map_or(0.0, |b| b as f32 + 1.0);
                        }
                    }
                    let slot = n_skins + palette_extra.len();
                    if slot < SKIN_MAX {
                        palette_extra.push(Mat4::from(i.transform));
                        moving_now.insert((mi, k), (slot as u32, skin));
                    }
                    (inst, lo, hi)
                })
                .collect();
            let mut draws: [ModelDraw; REGIONS as usize] = Default::default();
            let mut dynamic: Vec<InstanceData> = Vec::with_capacity(m.reserve * REGIONS as usize);
            for (r, cull) in culls.iter().enumerate() {
                if r < 3 && r as u32 >= shadow_cascades {
                    continue;
                }
                // Hvor langt unna noe i boksen er, og om det tegnes nær,
                // fjernt eller ikke.
                let lod = |lo: Vec3, hi: Vec3| -> Option<bool> {
                    let d = cull.eye.clamp(lo, hi).distance(cull.eye);
                    if self.reduced && m.small && d > if r == 3 { 32.0 } else if r == 0 { 12.0 } else { 0.0 } {
                        return None;
                    }
                    // Redusert grafikk ser kortere: det som er langt unna
                    // tegnes ikke (tåka er tettere, så kanten glir bort).
                    if self.reduced && d > REDUCED_FAR {
                        return None;
                    }
                    // Skyggekaskadene lenger ute nøyer seg med den fjerne.
                    Some(d < model_near && (r == 3 || r == 0))
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
        let mut tex_scale = [[1.0f32; 4]; 7];
        for (i, s) in self.materials.scales.iter().enumerate().take(LAYERS as usize) {
            tex_scale[i / 4][i % 4] = *s;
        }
        let wx = &f.weather;
        let mut grid = [[0u32; 4]; GRID * GRID / 16];
        let mut grid_info = [0.0f32; 4];
        if let Some(g) = &f.grid {
            grid_info = [g.origin.x, g.origin.y, g.floor_y, g.cell];
            for (i, b) in g.bits.iter().take(GRID * GRID).enumerate() {
                grid[i / 16][(i / 4) % 4] |= (*b as u32) << ((i % 4) * 8);
            }
        }
        let globals = Globals {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            prev_view_proj: self.prev_view_proj.to_cols_array_2d(),
            sun_vp,
            camera: v4(f.camera, f.time),
            // Uten skyggekart (kjelleren) når ikke sollyset ned: murene og
            // byen over ville ellers skygget for det.
            sun_dir: v4(f.sun_dir, if shadow_cascades == 0 { 0.0 } else { f.sun_intensity }),
            sun_color: v4(f.sun_color, f.night),
            sky_top: v4(f.sky_top, 0.0),
            sky_horizon: v4(f.sky_horizon, if self.reduced && f.fog_density > 0.0 { (f.fog_density * 3.0).max(REDUCED_FOG) } else { f.fog_density }),
            ground_bounce: v4(f.ground_bounce, f.fog_falloff),
            fog_color: v4(f.fog_color, f.windows_lit),
            params: [f.exposure, f.lights.len().min(if self.reduced { 8 } else { MAX_LIGHTS }) as f32, 1.0 / CASCADE_RES as f32, f.street_lights],
            weather: [wx.wet, wx.snow_cover, wx.rain, wx.snow],
            screen: [w, h, 1.0 / w, 1.0 / h],
            jitter: [jitter[0], jitter[1], self.frame_index as f32, wx.wind],
            cascade: [CASCADES[0], CASCADES[1], CASCADES[2], self.quality],
            sky: [wx.cloud, f.aurora, wx.lightning, f.debug as f32],
            tex_scale,
            palette,
            lights,
            smoke,
            grid_info,
            grid,
        };
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        // Per-pass-verdier: kaskade 0–2, regn, snø.
        let mut pass_data = vec![0u8; PASS_STRIDE as usize * 6];
        let put = |data: &mut Vec<u8>, i: usize, v: [f32; 4]| {
            let at = i * PASS_STRIDE as usize;
            data[at..at + 16].copy_from_slice(bytemuck::cast_slice(&v));
        };
        for c in 0..3 {
            put(&mut pass_data, c, [c as f32, 0.0, 0.0, 0.0]);
        }
        put(&mut pass_data, 3, [0.0, wx.rain, 0.0, 0.0]);
        put(&mut pass_data, 4, [1.0, wx.snow, 0.0, 0.0]);
        put(&mut pass_data, 5, [2.0, 1.0, 0.0, 0.0]);
        self.queue.write_buffer(&self.pass_buf, 0, &pass_data);

        // Den faste verdenen har én instans uten transform.
        let inst = [InstanceData::new(&Affine3A::IDENTITY, [0.0; 4])];
        self.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&inst));

        // Leddpaletten: matrisene kolonne for kolonne, fire texels hver.
        if !f.skins.is_empty() || !palette_extra.is_empty() {
            if f.skins.len() > SKIN_MAX {
                log::warn!("For mange leddmatriser: {} av {}", f.skins.len(), SKIN_MAX);
            }
            let n = n_skins + palette_extra.len();
            let rows = (n as u32).div_ceil(SKIN_W / 4);
            let mut data = vec![0f32; (rows * SKIN_W * 4) as usize];
            for (i, m) in f.skins.iter().take(n_skins).chain(palette_extra.iter()).enumerate() {
                data[i * 16..i * 16 + 16].copy_from_slice(&m.to_cols_array());
            }
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &self.skins[pal], mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                bytemuck::cast_slice(&data),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(SKIN_W * 16), rows_per_image: Some(rows) },
                wgpu::Extent3d { width: SKIN_W, height: rows, depth_or_array_layers: 1 },
            );
        }

        if let Some(tm) = &mut self.timer {
            let _ = self.device.poll(wgpu::PollType::Poll);
            tm.collect();
        }
        let ts = |k: usize, begin: bool, end: bool| self.timer.as_ref().and_then(|tm| tm.writes(k, begin, end));
        let mut enc = self.device.create_command_encoder(&Default::default());

        // 1. Skygger: tre kaskader i hvert sitt hjørne av atlaset.
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
            pass.set_pipeline(&self.shadow_pipe);
            pass.set_bind_group(0, &self.globals_bg[pal], &[]);
            pass.set_bind_group(2, &self.materials.bind_group, &[]);
            pass.set_bind_group(3, &self.materials.dummy_model, &[]);
            for c in 0..shadow_cascades {
                let (x, y) = ((c % 2) * CASCADE_RES, (c / 2) * CASCADE_RES);
                pass.set_viewport(x as f32, y as f32, CASCADE_RES as f32, CASCADE_RES as f32, 0.0, 1.0);
                pass.set_bind_group(1, &t.shadow_frame_bg, &[(c as u64 * PASS_STRIDE) as u32]);
                self.draw_all(&mut pass, &sun_cull[c as usize], &model_draws, c as usize);
            }
        }

        // 2. Dybde og normaler.
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("prepass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &t.normal,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &t.depth_lin,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::WHITE), store: wgpu::StoreOp::Store },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &t.velocity,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: ts(1, true, true),
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.prepass_pipe);
            pass.set_bind_group(0, &self.globals_bg[pal], &[]);
            pass.set_bind_group(1, &t.frame_bg, &[0]);
            pass.set_bind_group(2, &self.materials.bind_group, &[]);
            pass.set_bind_group(3, &self.materials.dummy_model, &[]);
            self.draw_all(&mut pass, &cam_cull, &model_draws, 3);
        }

        // 3. SSAO i halv oppløsning, så to uskarphetspass.
        let sharpen = if taa { SHARPEN } else { 0.0 };
        let post = |buf: &wgpu::Buffer, params: [f32; 4], sw: u32, sh: u32| {
            let u = PostUniform { params, texel: [sw as f32, sh as f32, 1.0 / sw as f32, 1.0 / sh as f32], extra: [sharpen, 0.0, 0.0, 0.0] };
            self.queue.write_buffer(buf, 0, bytemuck::bytes_of(&u));
        };
        let (hw, hh) = ((t.width / 2).max(1), (t.height / 2).max(1));
        let fullscreen = |enc: &mut wgpu::CommandEncoder, label: &str, target: &wgpu::TextureView, pipe: &wgpu::RenderPipeline, bg: &wgpu::BindGroup, load: wgpu::LoadOp<wgpu::Color>, timestamps: Option<wgpu::RenderPassTimestampWrites>| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: timestamps,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &self.globals_bg[pal], &[]);
            pass.set_bind_group(1, &t.post_frame_bg, &[0]);
            pass.set_bind_group(2, bg, &[]);
            pass.draw(0..3, 0..1);
        };
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        if self.reduced {
            // Ingen SSAO: okklusjonen er hvit (ingenting skygges).
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("no ssao"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &t.ao[0], resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::WHITE), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: ts(2, true, true),
                occlusion_query_set: None,
            });
        } else {
            post(&t.ssao_bg.1, [0.7, 1.1, 0.0, 0.0], t.width, t.height);
            fullscreen(&mut enc, "ssao", &t.ao[0], &self.ssao_pipe, &t.ssao_bg.0, clear, ts(2, true, false));
            post(&t.blur_bg[0].1, [0.0, 0.0, 1.0, 0.0], hw, hh);
            fullscreen(&mut enc, "blur x", &t.ao[1], &self.blur_pipe, &t.blur_bg[0].0, clear, None);
            post(&t.blur_bg[1].1, [0.0, 0.0, 0.0, 1.0], hw, hh);
            fullscreen(&mut enc, "blur y", &t.ao[0], &self.blur_pipe, &t.blur_bg[1].0, clear, ts(2, false, true));
        }

        // 4. Verden, himmel og nedbør.
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.hdr,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: ts(3, true, true),
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.world_pipe);
            pass.set_bind_group(0, &self.globals_bg[pal], &[]);
            pass.set_bind_group(1, &t.frame_bg, &[0]);
            pass.set_bind_group(2, &self.materials.bind_group, &[]);
            pass.set_bind_group(3, &self.materials.dummy_model, &[]);
            let tris = self.draw_all(&mut pass, &cam_cull, &model_draws, 3);
            self.triangles.set(tris);
            pass.set_pipeline(&self.sky_pipe);
            pass.set_bind_group(0, &self.globals_bg[pal], &[]);
            pass.set_bind_group(1, &t.frame_bg, &[0]);
            pass.draw(0..3, 0..1);
            if !f.smoke.is_empty() {
                pass.set_pipeline(&self.particle_pipe);
                pass.set_bind_group(1, &t.frame_bg, &[(5 * PASS_STRIDE) as u32]);
                pass.draw(0..4, 0..f.smoke.len() as u32 * SMOKE_PUFFS);
            }
            if wx.rain > 0.0 || wx.snow > 0.0 {
                pass.set_pipeline(&self.particle_pipe);
                if wx.rain > 0.0 {
                    pass.set_bind_group(1, &t.frame_bg, &[(3 * PASS_STRIDE) as u32]);
                    pass.draw(0..4, 0..RAIN_COUNT);
                }
                if wx.snow > 0.0 {
                    pass.set_bind_group(1, &t.frame_bg, &[(4 * PASS_STRIDE) as u32]);
                    pass.draw(0..4, 0..SNOW_COUNT);
                }
            }
        }

        // 5. Tåke.
        post(&t.fog_bg.1, [0.0; 4], t.width, t.height);
        fullscreen(&mut enc, "fog", &t.hdr_fog, &self.fog_pipe, &t.fog_bg.0, clear, ts(4, true, false));

        // 6. TAA.
        let cur = (self.frame_index % 2) as usize;
        post(&t.taa_bg[cur].1, [if taa { 1.0 } else { 0.0 }, 0.9, 0.0, 0.0], t.width, t.height);
        fullscreen(&mut enc, "taa", &t.resolved[cur], &self.taa_pipe, &t.taa_bg[cur].0, clear, None);

        // 7. Glød (ikke i redusert grafikk).
        for i in (0..BLOOM_LEVELS).filter(|_| !self.reduced) {
            let (bg, sw, sh) = if i == 0 { (&t.down0_bg[cur], t.width, t.height) } else { (&t.down_bg[i - 1], t.bloom[i - 1].1, t.bloom[i - 1].2) };
            post(&bg.1, [0.0, 0.0, 0.0, if i == 0 { 1.0 } else { 0.0 }], sw, sh);
            fullscreen(&mut enc, "bloom down", &t.bloom[i].0, &self.down_pipe, &bg.0, clear, None);
        }
        for (k, i) in (1..BLOOM_LEVELS).rev().enumerate().filter(|_| !self.reduced) {
            post(&t.up_bg[k].1, [0.0; 4], t.bloom[i].1, t.bloom[i].2);
            fullscreen(&mut enc, "bloom up", &t.bloom[i - 1].0, &self.up_pipe, &t.up_bg[k].0, wgpu::LoadOp::Load, None);
        }

        // 8. Tonemapping til skjermen.
        post(&t.tonemap_bg[cur].1, [f.exposure, if self.reduced { 0.0 } else { f.bloom }, if self.encode_srgb { 1.0 } else { 0.0 }, f.hurt], t.width, t.height);
        fullscreen(&mut enc, "tonemap", out, &self.tonemap_pipe, &t.tonemap_bg[cur].0, clear, ts(4, false, true));

        if let Some(tm) = &mut self.timer {
            tm.resolve(&mut enc);
        }
        self.queue.submit([enc.finish()]);
        if let Some(tm) = &mut self.timer {
            tm.map();
        }
        self.prev_view_proj = view_proj;
        self.prev_moving = moving_now;
        self.history_valid = f.taa;
        self.frame_index = self.frame_index.wrapping_add(1);
    }

    /// Tegn verden og modellene. Verden tegnes bit for bit, bare
    /// de bitene som er innenfor synsfeltet, og småtingene bare nær
    /// kameraet. Naboer som begge skal tegnes slås sammen til ett kall.
    /// Returnerer antall trekanter.
    fn draw_all(&self, pass: &mut wgpu::RenderPass, cull: &Cull, model_draws: &[[ModelDraw; REGIONS as usize]], region: usize) -> (u32, u32, u32) {
        pass.set_vertex_buffer(1, self.instances.slice(0..INSTANCE_STRIDE));
        self.world.bind(pass);
        let mut indices = 0u32;
        let mut run: Option<std::ops::Range<u32>> = None;
        let mut flush = |run: &mut Option<std::ops::Range<u32>>, pass: &mut wgpu::RenderPass| {
            if let Some(r) = run.take() {
                if !r.is_empty() {
                    indices += r.end - r.start;
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
        let props = 0;
        let mut models = 0;
        for (m, draws) in self.models.iter().zip(model_draws) {
            let d = &draws[region];
            if d.near.is_empty() && d.far.is_empty() {
                continue;
            }
            pass.set_bind_group(3, &m.textures, &[]);
            pass.set_vertex_buffer(1, m.instances.slice(..));
            m.mesh.bind(pass);
            for r in &d.near {
                pass.draw_indexed(m.near.clone(), 0, r.clone());
                models += (m.near.end - m.near.start) / 3 * r.len() as u32;
            }
            for r in &d.far {
                pass.draw_indexed(m.far.clone(), 0, r.clone());
                models += (m.far.end - m.far.start) / 3 * r.len() as u32;
            }
        }
        (world, props, models)
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}

impl ports::Renderer for Renderer {
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
    fn set_scale(&mut self, scale: f32) {
        Renderer::set_scale(self, scale);
    }
    fn set_quality(&mut self, quality: f32) {
        Renderer::set_quality(self, quality);
    }
    fn set_reduced(&mut self, reduced: bool) {
        Renderer::set_reduced(self, reduced);
    }
    fn triangles(&self) -> u32 {
        let (w, p, m) = self.triangles.get();
        w + p + m
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
}
