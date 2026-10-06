//! Rendereren: skyggekart, verden, himmel, glød og tonemapping.
//!
//! Rekkefølgen per bilde:
//! 1. Skygger fra sola eller månen inn i et dybdekart.
//! 2. Verden og himmel inn i et HDR-bilde, med 4× MSAA der GPU-en tåler det.
//! 3. Glød: det lyse i bildet krympes i fem trinn og vokser tilbake, uskarpt.
//! 4. Tonemapping fra HDR til skjermen.

use crate::mesh::{Mesh, Vertex};
use crate::world::World;
use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;

pub const MAX_LIGHTS: usize = 32;
const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SHADOW_SIZE: u32 = 2048;
const BLOOM_LEVELS: usize = 5;
const OBJECT_STRIDE: u64 = 256;
const MAX_OBJECTS: usize = 64;

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
    sun_view_proj: [[f32; 4]; 4],
    camera: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    sky_top: [f32; 4],
    sky_horizon: [f32; 4],
    ground_bounce: [f32; 4],
    fog_color: [f32; 4],
    params: [f32; 4],
    palette: [[f32; 4]; 16],
    lights: [GpuLight; MAX_LIGHTS],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Object {
    model: [[f32; 4]; 4],
    extra: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostUniform {
    params: [f32; 4],
    texel: [f32; 4],
}

/// Alt rendereren trenger å vite om et bilde. Lages av spillet.
pub struct Frame {
    pub view: Mat4,
    pub proj: Mat4,
    pub camera: Vec3,
    pub time: f32,
    /// Retning mot sola eller månen.
    pub sun_dir: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f32,
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
    pub palette: [Vec3; 16],
    pub lights: Vec<(Vec3, f32, Vec3, f32)>,
    /// Transform og markering for hver prop, i samme rekkefølge som i verden.
    pub props: Vec<(Affine3A, f32)>,
}

struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
}

impl GpuMesh {
    fn new(device: &wgpu::Device, mesh: &Mesh, label: &str) -> Self {
        Self {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            count: mesh.indices.len() as u32,
        }
    }
}

struct Targets {
    width: u32,
    height: u32,
    hdr_msaa: Option<wgpu::TextureView>,
    hdr: wgpu::TextureView,
    depth: wgpu::TextureView,
    bloom: Vec<(wgpu::TextureView, u32, u32)>,
    /// Bindinger for hvert glødtrinn ned, hvert trinn opp, og tonemappingen.
    down: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
    up: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
    tonemap: (wgpu::BindGroup, wgpu::Buffer),
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    samples: u32,
    encode_srgb: bool,
    globals: wgpu::Buffer,
    globals_bg: wgpu::BindGroup,
    objects: wgpu::Buffer,
    objects_bg: wgpu::BindGroup,
    shadow_view: wgpu::TextureView,
    shadow_bg: wgpu::BindGroup,
    world_pipe: wgpu::RenderPipeline,
    shadow_pipe: wgpu::RenderPipeline,
    sky_pipe: wgpu::RenderPipeline,
    down_pipe: wgpu::RenderPipeline,
    up_pipe: wgpu::RenderPipeline,
    tonemap_pipe: wgpu::RenderPipeline,
    post_layout: wgpu::BindGroupLayout,
    linear: wgpu::Sampler,
    black: wgpu::TextureView,
    world: GpuMesh,
    props: Vec<GpuMesh>,
    extent: f32,
    targets: Targets,
}

fn shader(device: &wgpu::Device, name: &str, src: &str) -> wgpu::ShaderModule {
    let full = format!("{}\n{}", include_str!("shaders/common.wgsl"), src);
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

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        adapter: &wgpu::Adapter,
        output: wgpu::TextureFormat,
        world: &World,
        width: u32,
        height: u32,
    ) -> Self {
        let samples = if adapter
            .get_texture_format_features(HDR)
            .flags
            .sample_count_supported(4)
            && adapter
                .get_texture_format_features(DEPTH)
                .flags
                .sample_count_supported(4)
        {
            4
        } else {
            1
        };
        log::info!("MSAA: {samples}×, utdata: {output:?}");

        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals"),
            entries: &[uniform_entry(0, false)],
        });
        let globals_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() }],
        });

        let objects = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("objects"),
            size: OBJECT_STRIDE * MAX_OBJECTS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let objects_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("objects"),
            entries: &[uniform_entry(0, true)],
        });
        let objects_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("objects"),
            layout: &objects_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &objects,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<Object>() as u64),
                }),
            }],
        });

        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow"),
            size: wgpu::Extent3d { width: SHADOW_SIZE, height: SHADOW_SIZE, depth_or_array_layers: 1 },
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
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });
        let shadow_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow"),
            layout: &shadow_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&shadow_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        let world_src = shader(&device, "world", include_str!("shaders/world.wgsl"));
        let sky_src = shader(&device, "sky", include_str!("shaders/sky.wgsl"));
        let post_src = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/post.wgsl").into()),
        });

        let world_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("world"),
            bind_group_layouts: &[&globals_layout, &objects_layout, &shadow_layout],
            push_constant_ranges: &[],
        });
        let shadow_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[&globals_layout, &objects_layout],
            push_constant_ranges: &[],
        });
        let sky_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky"),
            bind_group_layouts: &[&globals_layout],
            push_constant_ranges: &[],
        });

        let ms = wgpu::MultisampleState { count: samples, mask: !0, alpha_to_coverage_enabled: false };
        let world_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("world"),
            layout: Some(&world_layout),
            vertex: wgpu::VertexState {
                module: &world_src,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Vertex::LAYOUT],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: ms,
            fragment: Some(wgpu::FragmentState {
                module: &world_src,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: HDR, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });
        let shadow_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&shadow_pipe_layout),
            vertex: wgpu::VertexState {
                module: &world_src,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &[Vertex::LAYOUT],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
            }),
            multisample: Default::default(),
            fragment: None,
            multiview: None,
            cache: None,
        });
        let sky_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky"),
            layout: Some(&sky_layout),
            vertex: wgpu::VertexState {
                module: &sky_src,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: ms,
            fragment: Some(wgpu::FragmentState {
                module: &sky_src,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: HDR, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                uniform_entry(2, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let post_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post"),
            bind_group_layouts: &[&post_layout],
            push_constant_ranges: &[],
        });
        let post_pipe = |entry: &str, format: wgpu::TextureFormat, blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&post_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &post_src,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &post_src,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format, blend, write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview: None,
                cache: None,
            })
        };
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        };
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

        let world_mesh = GpuMesh::new(&device, &world.mesh, "world");
        let props = world.props.iter().map(|p| GpuMesh::new(&device, &p.mesh, p.name)).collect();
        log::info!(
            "Verden: {} hjørner, {} trekanter, {} props",
            world.mesh.vertices.len(),
            world.mesh.indices.len() / 3,
            world.props.len()
        );

        let targets = Self::make_targets(&device, &post_layout, &linear, &black, samples, width, height);
        Self {
            device,
            queue,
            samples,
            encode_srgb: !output.is_srgb(),
            globals,
            globals_bg,
            objects,
            objects_bg,
            shadow_view,
            shadow_bg,
            world_pipe,
            shadow_pipe,
            sky_pipe,
            down_pipe,
            up_pipe,
            tonemap_pipe,
            post_layout,
            linear,
            black,
            world: world_mesh,
            props,
            extent: 48.0,
            targets,
        }
    }

    fn make_targets(
        device: &wgpu::Device,
        post_layout: &wgpu::BindGroupLayout,
        linear: &wgpu::Sampler,
        black: &wgpu::TextureView,
        samples: u32,
        width: u32,
        height: u32,
    ) -> Targets {
        let width = width.max(1);
        let height = height.max(1);
        let tex = |label: &str, w: u32, h: u32, format, samples: u32, usage| {
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
        let rt = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let rt_sampled = rt | wgpu::TextureUsages::TEXTURE_BINDING;
        let hdr_msaa = (samples > 1).then(|| tex("hdr msaa", width, height, HDR, samples, rt));
        let hdr = tex("hdr", width, height, HDR, 1, rt_sampled);
        let depth = tex("depth", width, height, DEPTH, samples, rt);

        let mut bloom = vec![];
        let (mut w, mut h) = (width, height);
        for i in 0..BLOOM_LEVELS {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            bloom.push((tex(&format!("bloom {i}"), w, h, HDR, 1, rt_sampled), w, h));
        }

        let bind = |src: &wgpu::TextureView, extra: &wgpu::TextureView, sw: u32, sh: u32| {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("post"),
                size: std::mem::size_of::<PostUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let _ = (sw, sh);
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("post"),
                layout: post_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(src) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(linear) },
                    wgpu::BindGroupEntry { binding: 2, resource: buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(extra) },
                ],
            });
            (bg, buf)
        };
        let mut down = vec![bind(&hdr, black, width, height)];
        for i in 1..BLOOM_LEVELS {
            down.push(bind(&bloom[i - 1].0, black, bloom[i - 1].1, bloom[i - 1].2));
        }
        let mut up = vec![];
        for i in (1..BLOOM_LEVELS).rev() {
            up.push(bind(&bloom[i].0, black, bloom[i].1, bloom[i].2));
        }
        let tonemap = bind(&hdr, &bloom[0].0, width, height);
        Targets { width, height, hdr_msaa, hdr, depth, bloom, down, up, tonemap }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == self.targets.width && height == self.targets.height {
            return;
        }
        self.targets = Self::make_targets(
            &self.device,
            &self.post_layout,
            &self.linear,
            &self.black,
            self.samples,
            width,
            height,
        );
    }

    pub fn size(&self) -> (u32, u32) {
        (self.targets.width, self.targets.height)
    }

    fn sun_view_proj(&self, f: &Frame) -> Mat4 {
        // Skyggekartet følger kameraet, men flyttes bare i hele texels, så
        // skyggekantene ikke skimrer når du går.
        let e = self.extent;
        let up = if f.sun_dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let view0 = Mat4::look_at_rh(f.sun_dir * 200.0, Vec3::ZERO, up);
        let texel = 2.0 * e / SHADOW_SIZE as f32;
        let ahead = f.camera + f.view.inverse().transform_vector3(-Vec3::Z) * e * 0.5;
        let mut c = view0.transform_point3(Vec3::new(ahead.x, 0.0, ahead.z));
        c.x = (c.x / texel).round() * texel;
        c.y = (c.y / texel).round() * texel;
        let proj = Mat4::orthographic_rh(c.x - e, c.x + e, c.y - e, c.y + e, 1.0, 600.0);
        proj * view0
    }

    pub fn render(&mut self, out: &wgpu::TextureView, f: &Frame) {
        let view_proj = f.proj * f.view;
        let sun_vp = self.sun_view_proj(f);
        let mut lights = [GpuLight::zeroed(); MAX_LIGHTS];
        for (i, (pos, radius, color, intensity)) in f.lights.iter().take(MAX_LIGHTS).enumerate() {
            lights[i] = GpuLight {
                pos_radius: [pos.x, pos.y, pos.z, *radius],
                color_intensity: [color.x, color.y, color.z, *intensity],
            };
        }
        let v4 = |v: Vec3, w: f32| [v.x, v.y, v.z, w];
        let mut palette = [[0.0; 4]; 16];
        for (i, c) in f.palette.iter().enumerate() {
            palette[i] = v4(*c, 1.0);
        }
        let globals = Globals {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            sun_view_proj: sun_vp.to_cols_array_2d(),
            camera: v4(f.camera, f.time),
            sun_dir: v4(f.sun_dir, f.sun_intensity),
            sun_color: v4(f.sun_color, f.night),
            sky_top: v4(f.sky_top, 0.0),
            sky_horizon: v4(f.sky_horizon, f.fog_density),
            ground_bounce: v4(f.ground_bounce, 0.0),
            fog_color: v4(f.fog_color, f.fog_falloff),
            params: [f.exposure, f.lights.len().min(MAX_LIGHTS) as f32, 1.0 / SHADOW_SIZE as f32, f.windows_lit],
            palette,
            lights,
        };
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        // Objekt 0 er den faste verdenen, resten er propsene.
        let mut objects = vec![0u8; OBJECT_STRIDE as usize * (1 + self.props.len())];
        let mut put = |i: usize, m: Mat4, hl: f32| {
            let o = Object { model: m.to_cols_array_2d(), extra: Vec4::new(hl, 0.0, 0.0, 0.0).to_array() };
            let at = i * OBJECT_STRIDE as usize;
            objects[at..at + std::mem::size_of::<Object>()].copy_from_slice(bytemuck::bytes_of(&o));
        };
        put(0, Mat4::IDENTITY, 0.0);
        for (i, (t, hl)) in f.props.iter().enumerate().take(MAX_OBJECTS - 1) {
            put(i + 1, Mat4::from(*t), *hl);
        }
        self.queue.write_buffer(&self.objects, 0, &objects);

        let mut enc = self.device.create_command_encoder(&Default::default());

        // 1. Skygger.
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.shadow_pipe);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            self.draw_all(&mut pass);
        }

        // 2. Verden og himmel.
        {
            let t = &self.targets;
            let (view, resolve) = match &t.hdr_msaa {
                Some(ms) => (ms, Some(&t.hdr)),
                None => (&t.hdr, None),
            };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: resolve,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: if resolve.is_some() { wgpu::StoreOp::Discard } else { wgpu::StoreOp::Store },
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.world_pipe);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_bind_group(2, &self.shadow_bg, &[]);
            self.draw_all(&mut pass);
            pass.set_pipeline(&self.sky_pipe);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // 3. Glød.
        let t = &self.targets;
        let post = |buf: &wgpu::Buffer, params: [f32; 4], sw: u32, sh: u32| {
            let u = PostUniform {
                params,
                texel: [sw as f32, sh as f32, 1.0 / sw as f32, 1.0 / sh as f32],
            };
            self.queue.write_buffer(buf, 0, bytemuck::bytes_of(&u));
        };
        for i in 0..BLOOM_LEVELS {
            let (sw, sh) = if i == 0 { (t.width, t.height) } else { (t.bloom[i - 1].1, t.bloom[i - 1].2) };
            post(&t.down[i].1, [0.0, 0.0, 0.0, if i == 0 { 1.0 } else { 0.0 }], sw, sh);
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom down"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.bloom[i].0,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.down_pipe);
            pass.set_bind_group(0, &t.down[i].0, &[]);
            pass.draw(0..3, 0..1);
        }
        for (k, i) in (1..BLOOM_LEVELS).rev().enumerate() {
            post(&t.up[k].1, [0.0; 4], t.bloom[i].1, t.bloom[i].2);
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom up"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.bloom[i - 1].0,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.up_pipe);
            pass.set_bind_group(0, &t.up[k].0, &[]);
            pass.draw(0..3, 0..1);
        }

        // 4. Tonemapping til skjermen.
        post(
            &t.tonemap.1,
            [f.exposure, f.bloom, if self.encode_srgb { 1.0 } else { 0.0 }, 0.0],
            t.width,
            t.height,
        );
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tonemap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.tonemap_pipe);
            pass.set_bind_group(0, &t.tonemap.0, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([enc.finish()]);
    }

    fn draw_all(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(1, &self.objects_bg, &[0]);
        pass.set_vertex_buffer(0, self.world.vertices.slice(..));
        pass.set_index_buffer(self.world.indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.world.count, 0, 0..1);
        for (i, m) in self.props.iter().enumerate() {
            pass.set_bind_group(1, &self.objects_bg, &[((i + 1) as u64 * OBJECT_STRIDE) as u32]);
            pass.set_vertex_buffer(0, m.vertices.slice(..));
            pass.set_index_buffer(m.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..m.count, 0, 0..1);
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}
