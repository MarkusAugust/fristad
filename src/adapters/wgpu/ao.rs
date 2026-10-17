//! Skygge i krokene (`Quality::ao`): okklusjon i skjermrommet i halv
//! oppløsning fra dybden (`shaders/ao.wgsl`), tolv faste prøver uten
//! tilfeldig dreiing (ingen støy og intet flimmer), så en uskarp utjevning
//! som stopper ved kanter i dybden. Tonemappingen ganger HDR-bildet med
//! den. Bare WebGPU og de innebygde: WebGL2 kan ikke lese dybden med
//! `textureLoad`.

use super::{tex_entry, FILTER};

/// Formatet okklusjonen lagres i.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// Pipelinene, laget én gang.
pub struct Ao {
    layout: wgpu::BindGroupLayout,
    ao: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
}

/// Bildene og bindingene for én skjermstørrelse.
pub struct Targets {
    raw: wgpu::TextureView,
    /// Den utjevnede okklusjonen, som tonemappingen leser.
    pub out: wgpu::TextureView,
    bind: [wgpu::BindGroup; 2],
}

impl Ao {
    pub fn new(device: &wgpu::Device, globals_layout: &wgpu::BindGroupLayout, common: &str) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("ao"), entries: &[tex_entry(0, wgpu::TextureSampleType::Depth), tex_entry(1, FILTER)] });
        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ao"), bind_group_layouts: &[globals_layout, &layout], push_constant_ranges: &[] });
        let module = super::shader(device, "ao", &[common, include_str!("shaders/ao.wgsl")]);
        let pipe = |entry: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipe_layout),
                vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(entry), compilation_options: Default::default(), targets: &[Some(FORMAT.into())] }),
                multiview: None,
                cache: None,
            })
        };
        Ao { ao: pipe("fs_ao"), blur: pipe("fs_blur"), layout }
    }

    /// Bildene i halv oppløsning for en skjerm på `width` × `height`.
    pub fn targets(&self, device: &wgpu::Device, depth: &wgpu::TextureView, width: u32, height: u32) -> Targets {
        let tex = |label: &str| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: (width / 2).max(1), height: (height / 2).max(1), depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let raw = tex("ao");
        let out = tex("ao blur");
        let bind = |second: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ao"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(depth) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(second) },
                ],
            })
        };
        // Okklusjonen leser bare dybden (det andre bildet er der for
        // oppsettets skyld); utjevningen leser okklusjonen.
        let bind = [bind(&out), bind(&raw)];
        Targets { raw, out, bind }
    }

    /// Okklusjonen og utjevningen; `begin` er tidsmålingen som begynner her.
    pub fn run(&self, enc: &mut wgpu::CommandEncoder, globals: &wgpu::BindGroup, t: &Targets, begin: Option<wgpu::RenderPassTimestampWrites>) {
        let mut begin = begin;
        for (pipe, target, bg) in [(&self.ao, &t.raw, &t.bind[0]), (&self.blur, &t.out, &t.bind[1])] {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ao"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::WHITE), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: begin.take(),
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, globals, &[]);
            pass.set_bind_group(1, bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
