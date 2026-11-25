//! Oppsettet: rendereren lages, og målene den tegner til.

use super::*;

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
            upgrade_credit: 0.0,
            upgrade_ms: std::env::var("FRISTAD_UPGRADE_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(crate::adapters::delivery::UPGRADE_MS),
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
    pub(super) fn frame_bg(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, pass_buf: &wgpu::Buffer, shadow: &wgpu::TextureView) -> wgpu::BindGroup {
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
    pub(super) fn wanted_shadow(&self) -> (u32, u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        let n = if self.quality.far_shadow { 2 } else { 1 };
        let res = if self.quality.shadow { self.quality.shadow_res.clamp(512, 4096).min(max / n) } else { 1 };
        (res * n, res)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn make_targets(device: &wgpu::Device, post_layout: &wgpu::BindGroupLayout, linear: &wgpu::Sampler, samples: u32, format: wgpu::TextureFormat, smaa: Option<&smaa::Smaa>, ao: Option<&ao::Ao>, white: &wgpu::TextureView, width: u32, height: u32) -> Targets {
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
}
