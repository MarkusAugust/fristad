//! Pipelinene som tegner verden, og hjelperne til layoutene.

use super::*;

impl SceneSources {
    pub(super) fn pipes(&self, device: &wgpu::Device, samples: u32, a2c: bool, hdr: wgpu::TextureFormat) -> ScenePipes {
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
        let clip = world_pipe("clip", "fs_clip", false);
        // Forhåndsmultiplisert: fargen fra shaderen er glansen og det
        // grumsete, og det bak dempes med alfa.
        let premultiplied = wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent::OVER };
        let water = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water"),
            layout: Some(&self.world_layout),
            vertex: wgpu::VertexState { module: &self.world, entry_point: Some("vs_water"), compilation_options: Default::default(), buffers: &buffers },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: depth(true, wgpu::CompareFunction::Less),
            multisample: ms(false),
            fragment: Some(wgpu::FragmentState { module: &self.world, entry_point: Some("fs_water"), compilation_options: Default::default(), targets: &target(Some(premultiplied)) }),
            multiview: None,
            cache: None,
        });
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
        ScenePipes { samples, a2c, hdr, world, cutout, clip, water, sky, particles, lines }
    }
}

pub(super) fn shader(device: &wgpu::Device, name: &str, parts: &[&str]) -> wgpu::ShaderModule {
    let full = parts.join("\n");
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(full.into()),
    })
}

pub(super) fn uniform_entry(binding: u32, dynamic: bool) -> wgpu::BindGroupLayoutEntry {
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

pub(super) fn tex_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
        count: None,
    }
}

pub(super) fn sampler_entry(binding: u32, ty: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(ty), count: None }
}

pub(super) const FILTER: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };

/// Flest MSAA-prøver (av `wanted`, synkende) som både HDR-fargen og dybden
/// kan ha på dette kortet; 1 om ingen.
pub(super) fn max_samples(adapter: &wgpu::Adapter, device: &wgpu::Device, wanted: &[u32]) -> u32 {
    let ok = |n: u32| {
        // Annet enn 4 krever at kortets egne formategenskaper er slått på.
        (n == 4 || device.features().contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES))
            && adapter.get_texture_format_features(HDR).flags.sample_count_supported(n)
            && adapter.get_texture_format_features(DEPTH).flags.sample_count_supported(n)
    };
    wanted.iter().copied().find(|n| ok(*n)).unwrap_or(1)
}
