//! Kantutjevningen i standard grafikk: SMAA 1× etter tonemappingen
//! (`shaders/smaa.wgsl`, oppslagstabellene i `assets/smaa/`). MSAA koster
//! rundt 20 ms per bilde i Chrome på Apple-kort uansett hva som tegnes
//! (eierens måling 11. oktober); SMAA er tre små pass over skjermen.

use super::{sampler_entry, tex_entry, uniform_entry, FILTER};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Mellombildet tonemappingen skriver til (gamma-kodet, 8 bit).
pub const LDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniform {
    metrics: [f32; 4],
    flags: [f32; 4],
}

/// Pipelinene og tabellene, laget én gang.
pub struct Smaa {
    layout: wgpu::BindGroupLayout,
    edges: wgpu::RenderPipeline,
    weights: wgpu::RenderPipeline,
    blend: wgpu::RenderPipeline,
    area: wgpu::TextureView,
    search: wgpu::TextureView,
    linear: wgpu::Sampler,
    point: wgpu::Sampler,
    /// Utbildet er et sRGB-format: blandingen gjør fargen lineær igjen.
    srgb_out: bool,
}

/// Bildene og bindingene for én skjermstørrelse.
pub struct Targets {
    /// Tonemappingen skriver hit.
    pub ldr: wgpu::TextureView,
    edges: wgpu::TextureView,
    weights: wgpu::TextureView,
    bind: [wgpu::BindGroup; 3],
}

fn table(device: &wgpu::Device, queue: &wgpu::Queue, label: &str, w: u32, h: u32, format: wgpu::TextureFormat, data: &[u8]) -> wgpu::TextureView {
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        )
        .create_view(&Default::default())
}

impl Smaa {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, output: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("smaa"),
            entries: &[
                tex_entry(0, FILTER),
                tex_entry(1, FILTER),
                tex_entry(2, FILTER),
                tex_entry(3, FILTER),
                sampler_entry(4, wgpu::SamplerBindingType::Filtering),
                sampler_entry(5, wgpu::SamplerBindingType::Filtering),
                uniform_entry(6, false),
            ],
        });
        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("smaa"), bind_group_layouts: &[&layout], push_constant_ranges: &[] });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("smaa"), source: wgpu::ShaderSource::Wgsl(include_str!("shaders/smaa.wgsl").into()) });
        let pipe = |entry: &str, format: wgpu::TextureFormat| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipe_layout),
                vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(entry), compilation_options: Default::default(), targets: &[Some(format.into())] }),
                multiview: None,
                cache: None,
            })
        };
        let sampler = |label: &str, filter: wgpu::FilterMode| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some(label),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: filter,
                min_filter: filter,
                ..Default::default()
            })
        };
        Self {
            edges: pipe("fs_edges", wgpu::TextureFormat::Rg8Unorm),
            weights: pipe("fs_weights", wgpu::TextureFormat::Rgba8Unorm),
            blend: pipe("fs_blend", output),
            area: table(device, queue, "smaa area", 160, 560, wgpu::TextureFormat::Rg8Unorm, include_bytes!("../../../assets/smaa/area.bin")),
            search: table(device, queue, "smaa search", 64, 16, wgpu::TextureFormat::R8Unorm, include_bytes!("../../../assets/smaa/search.bin")),
            linear: sampler("smaa linear", wgpu::FilterMode::Linear),
            point: sampler("smaa point", wgpu::FilterMode::Nearest),
            layout,
            srgb_out: output.is_srgb(),
        }
    }

    pub fn targets(&self, device: &wgpu::Device, width: u32, height: u32) -> Targets {
        let tex = |label: &str, format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let ldr = tex("smaa ldr", LDR);
        let edges = tex("smaa edges", wgpu::TextureFormat::Rg8Unorm);
        let weights = tex("smaa weights", wgpu::TextureFormat::Rgba8Unorm);
        let u = Uniform { metrics: [1.0 / width as f32, 1.0 / height as f32, width as f32, height as f32], flags: [if self.srgb_out { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0] };
        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("smaa"), contents: bytemuck::bytes_of(&u), usage: wgpu::BufferUsages::UNIFORM });
        let bind = |a: &wgpu::TextureView, b: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("smaa"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(a) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(b) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.area) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.search) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.linear) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&self.point) },
                    wgpu::BindGroupEntry { binding: 6, resource: buf.as_entire_binding() },
                ],
            })
        };
        let bind = [bind(&ldr, &ldr), bind(&edges, &edges), bind(&ldr, &weights)];
        Targets { ldr, edges, weights, bind }
    }

    /// De tre passene, fra `t.ldr` til `out`. `end` er tidsmålingen ved
    /// slutten av det siste.
    pub fn run(&self, enc: &mut wgpu::CommandEncoder, t: &Targets, out: &wgpu::TextureView, end: Option<wgpu::RenderPassTimestampWrites>) {
        let steps: [(&wgpu::RenderPipeline, &wgpu::TextureView, &wgpu::BindGroup); 3] = [(&self.edges, &t.edges, &t.bind[0]), (&self.weights, &t.weights, &t.bind[1]), (&self.blend, out, &t.bind[2])];
        let mut end = end;
        for (k, (pipe, target, bg)) in steps.into_iter().enumerate() {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("smaa"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: if k == 2 { end.take() } else { None },
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
