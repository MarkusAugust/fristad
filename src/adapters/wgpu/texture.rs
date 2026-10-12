//! Teksturene: materialene bakes på GPU-en ved oppstart til to
//! array-teksturer (albedo og normal/ruhet/høyde) med mipmaps, pluss
//! bladkortet og skriftatlaset. Ingen bildefiler lastes ned for byen; bare
//! modellene har egne teksturer.

use crate::domain::geometry::{Layer, LAYERS, TEX_SCALE};
use crate::domain::model::LayerImages;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const SIZE: u32 = 1024;
const LEAF_SIZE: u32 = 512;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BakeUniform {
    params: [f32; 4],
}

pub struct Materials {
    /// Meter per flis for hvert lag: de bakte verdiene, eller målene til de
    /// fotoskannede materialene som erstatter dem.
    pub scales: [f32; LAYERS as usize],
    /// De bakte lagene, til feilsøking (`snapshot --dump-layer`).
    pub albedo_tex: wgpu::Texture,
    pub nrh_tex: wgpu::Texture,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// For modellenes teksturer: albedo, normal, metall/ruhet, emissiv.
    pub model_layout: wgpu::BindGroupLayout,
    pub dummy_model: wgpu::BindGroup,
    pub repeat: wgpu::Sampler,
}

fn tex2d(binding: u32, array: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: if array {
                wgpu::TextureViewDimension::D2Array
            } else {
                wgpu::TextureViewDimension::D2
            },
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

/// Laster et bilde opp som tekstur med mipmaps generert på GPU-en.
pub fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mipper: &Mipper,
    img: &crate::domain::model::Image,
    srgb: bool,
    label: &str,
) -> wgpu::TextureView {
    let format = if srgb {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    };
    let mips = (img.width.max(img.height) as f32).log2().floor() as u32 + 1;
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(img.width * 4),
            rows_per_image: Some(img.height),
        },
        wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
        },
    );
    mipper.generate(device, queue, &tex, &tex, format, format, 1, mips);
    tex.create_view(&Default::default())
}

fn solid(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rgba: [u8; 4],
    srgb: bool,
) -> wgpu::TextureView {
    let format = if srgb {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    };
    let tex = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("solid"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &rgba,
    );
    tex.create_view(&Default::default())
}

/// Lager mipmaps ved å tegne hvert nivå fra det forrige. Virker for to
/// teksturer samtidig (albedo og nrh) og for enkle teksturer.
pub struct Mipper {
    layout: wgpu::BindGroupLayout,
    layout_array: wgpu::BindGroupLayout,
    pipeline_srgb: wgpu::RenderPipeline,
    pipeline_pair: wgpu::RenderPipeline,
    pipeline_lin: wgpu::RenderPipeline,
    pipeline_array: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
}

impl Mipper {
    fn generate(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        a: &wgpu::Texture,
        b: &wgpu::Texture,
        fa: wgpu::TextureFormat,
        fb: wgpu::TextureFormat,
        layers: u32,
        mips: u32,
    ) {
        let pair = !std::ptr::eq(a, b);
        let pipeline = if pair {
            &self.pipeline_pair
        } else if fa.is_srgb() {
            &self.pipeline_srgb
        } else {
            &self.pipeline_lin
        };
        let _ = fb;
        let array = layers > 1;
        let pipeline = if array {
            &self.pipeline_array
        } else {
            pipeline
        };
        let mut enc = device.create_command_encoder(&Default::default());
        for layer in 0..layers {
            // Laget som uniform, til array-banen.
            let layer_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mip layer"),
                contents: bytemuck::bytes_of(&[layer as f32, 0.0, 0.0, 0.0]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            for level in 1..mips {
                let view = |t: &wgpu::Texture, l: u32| {
                    t.create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: l,
                        mip_level_count: Some(1),
                        base_array_layer: layer,
                        array_layer_count: Some(1),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        ..Default::default()
                    })
                };
                // Kilden: hele arrayet på nivået over (WebGL2 kan ikke lese ett
                // lag som 2D), eller teksturen selv når den ikke er et array.
                let src_view = |t: &wgpu::Texture| {
                    t.create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: level - 1,
                        mip_level_count: Some(1),
                        dimension: Some(if array {
                            wgpu::TextureViewDimension::D2Array
                        } else {
                            wgpu::TextureViewDimension::D2
                        }),
                        ..Default::default()
                    })
                };
                let src_a = src_view(a);
                let src_b = src_view(b);
                let dst_a = view(a, level);
                let dst_b = view(b, level);
                let bg = if array {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("mip array"),
                        layout: &self.layout_array,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: layer_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: wgpu::BindingResource::TextureView(&src_a),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: wgpu::BindingResource::TextureView(&src_b),
                            },
                        ],
                    })
                } else {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("mip"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.uniform.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&src_a),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(&src_b),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    })
                };
                let mut attachments = vec![Some(wgpu::RenderPassColorAttachment {
                    view: &dst_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })];
                if pair {
                    attachments.push(Some(wgpu::RenderPassColorAttachment {
                        view: &dst_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }));
                }
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("mip"),
                    color_attachments: &attachments,
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &bg, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        queue.submit([enc.finish()]);
    }
}

impl Materials {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        external: &[(Layer, LayerImages)],
    ) -> (Self, Mipper) {
        let src = format!(
            "{}\n{}",
            include_str!("shaders/noise.wgsl"),
            include_str!("shaders/bake.wgsl")
        );
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bake"),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
        let bake_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bake"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                tex2d(1, false),
                tex2d(2, false),
                sampler_entry(3),
            ],
        });
        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bake"),
            bind_group_layouts: &[&bake_layout],
            push_constant_ranges: &[],
        });
        let make = |entry: &str, formats: &[wgpu::TextureFormat]| {
            let targets: Vec<Option<wgpu::ColorTargetState>> = formats
                .iter()
                .map(|f| {
                    Some(wgpu::ColorTargetState {
                        format: *f,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                })
                .collect();
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipe_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &targets,
                }),
                multiview: None,
                cache: None,
            })
        };
        let srgb = wgpu::TextureFormat::Rgba8UnormSrgb;
        let lin = wgpu::TextureFormat::Rgba8Unorm;
        let bake_pipe = make("fs_bake", &[srgb, lin]);
        let leaf_pipe = make("fs_leaf", &[srgb]);
        let mip_pair = make("fs_mip", &[srgb, lin]);
        let mip_srgb = make("fs_mip_one", &[srgb]);
        let mip_lin = make("fs_mip_one", &[lin]);
        // Mip for array-teksturene: kilden bindes som array, laget i uniformen.
        let array_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mip array"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let array_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mip array"),
            bind_group_layouts: &[&array_layout],
            push_constant_ranges: &[],
        });
        let mip_array = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fs_mip_array"),
            layout: Some(&array_pipe_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_mip_array"),
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: srgb,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: lin,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mip"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bake"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mipper = Mipper {
            layout: bake_layout.clone(),
            layout_array: array_layout,
            pipeline_srgb: mip_srgb,
            pipeline_pair: mip_pair,
            pipeline_lin: mip_lin,
            pipeline_array: mip_array,
            sampler: sampler.clone(),
            uniform: uniform.clone(),
        };

        let mips = (SIZE as f32).log2() as u32 + 1;
        let array = |label: &str, format| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: SIZE,
                    height: SIZE,
                    depth_or_array_layers: LAYERS,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let albedo = array("albedo", srgb);
        let nrh = array("nrh", lin);
        let dummy = solid(device, queue, [0, 0, 0, 255], false);

        // Bak hvert lag.
        for layer in 0..LAYERS {
            let u = BakeUniform {
                params: [
                    layer as f32,
                    TEX_SCALE[layer as usize],
                    1.0 / SIZE as f32,
                    0.0,
                ],
            };
            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bake"),
                contents: bytemuck::bytes_of(&u),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bake"),
                layout: &bake_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&dummy),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&dummy),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let view = |t: &wgpu::Texture| {
                t.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                })
            };
            let va = view(&albedo);
            let vb = view(&nrh);
            let mut enc = device.create_command_encoder(&Default::default());
            {
                let clear = wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                };
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bake"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &va,
                            resolve_target: None,
                            ops: clear,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &vb,
                            resolve_target: None,
                            ops: clear,
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&bake_pipe);
                pass.set_bind_group(0, &bg, &[]);
                pass.draw(0..3, 0..1);
            }
            queue.submit([enc.finish()]);
        }
        // Fotoskannede materialer skrives over de bakte lagene.
        let mut scales = TEX_SCALE;
        for (layer, imgs) in external {
            // De malte lagene har fargen sin i bildet; modellene toner dem
            // med hvit.
            let (a, n) = pack_external(imgs, false);
            for (tex, data) in [(&albedo, &a), (&nrh, &n)] {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: *layer as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(SIZE * 4),
                        rows_per_image: Some(SIZE),
                    },
                    wgpu::Extent3d {
                        width: SIZE,
                        height: SIZE,
                        depth_or_array_layers: 1,
                    },
                );
            }
            scales[*layer as usize] = imgs.size_m.max(0.1);
            log::info!(
                "Lag {} bruker fotoskannet materiale ({} m per flis)",
                layer.name(),
                imgs.size_m
            );
        }
        mipper.generate(device, queue, &albedo, &nrh, srgb, lin, LAYERS, mips);

        // Bladkortet.
        let leaf_mips = (LEAF_SIZE as f32).log2() as u32 + 1;
        let leaf = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("leaf"),
            size: wgpu::Extent3d {
                width: LEAF_SIZE,
                height: LEAF_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: leaf_mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: srgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("leaf"),
                layout: &bake_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&dummy),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&dummy),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let v = leaf.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let mut enc = device.create_command_encoder(&Default::default());
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("leaf"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &v,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&leaf_pipe);
                pass.set_bind_group(0, &bg, &[]);
                pass.draw(0..3, 0..1);
            }
            queue.submit([enc.finish()]);
        }
        mipper.generate(device, queue, &leaf, &leaf, srgb, srgb, 1, leaf_mips);

        let repeat = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("repeat"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let clamp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("clamp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("materials"),
            entries: &[
                tex2d(0, true),
                tex2d(1, true),
                sampler_entry(2),
                tex2d(4, false),
                sampler_entry(5),
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("materials"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &albedo.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &nrh.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&repeat),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &leaf.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&clamp),
                },
            ],
        });

        let model_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("model"),
            entries: &[
                tex2d(0, false),
                tex2d(1, false),
                tex2d(2, false),
                tex2d(3, false),
            ],
        });
        let white = solid(device, queue, [255, 255, 255, 255], true);
        let flat = solid(device, queue, [128, 128, 255, 255], false);
        let mr = solid(device, queue, [255, 255, 0, 255], false);
        let black = solid(device, queue, [0, 0, 0, 255], true);
        let dummy_model = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("model dummy"),
            layout: &model_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&flat),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mr),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&black),
                },
            ],
        });

        (
            Self {
                scales,
                albedo_tex: albedo,
                nrh_tex: nrh,
                layout,
                bind_group,
                model_layout,
                dummy_model,
                repeat,
            },
            mipper,
        )
    }

    /// Bindingen for en modell med egne teksturer.
    pub fn model_bind_group(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mipper: &Mipper,
        m: &crate::domain::model::Model,
    ) -> wgpu::BindGroup {
        let tex =
            |img: &Option<crate::domain::model::Image>, srgb: bool, fallback: [u8; 4]| match img {
                Some(i) => upload(device, queue, mipper, i, srgb, &m.name),
                None => solid(device, queue, fallback, srgb),
            };
        let albedo = tex(&m.albedo, true, [255, 255, 255, 255]);
        let normal = tex(&m.normal, false, [128, 128, 255, 255]);
        let mr = tex(&m.metal_rough, false, [255, 255, 255, 255]);
        let emissive = tex(&m.emissive, true, [0, 0, 0, 255]);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&m.name),
            layout: &self.model_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&normal),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mr),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&emissive),
                },
            ],
        })
    }
}

/// Skalerer et bilde til lagstørrelsen.
fn fit(img: &crate::domain::model::Image) -> image::RgbaImage {
    let src = image::RgbaImage::from_raw(img.width, img.height, img.rgba.clone()).expect("bilde");
    if img.width == SIZE && img.height == SIZE {
        src
    } else {
        image::imageops::resize(&src, SIZE, SIZE, image::imageops::FilterType::Triangle)
    }
}

/// Pakker et fotoskannet materiale til lagformatene: albedo (med AO bakt
/// inn) og normal xy, ruhet, høyde.
fn pack_external(imgs: &LayerImages, normalize: bool) -> (Vec<u8>, Vec<u8>) {
    let diffuse = fit(&imgs.diffuse);
    let normal = imgs.normal.as_ref().map(fit);
    let rough = imgs.rough.as_ref().map(fit);
    let height = imgs.height.as_ref().map(fit);
    let ao = imgs.ao.as_ref().map(fit);
    // Gjennomsnittlig lyshet, så tonede lag kan skaleres mot 0,85.
    let mut gain = 1.0f32;
    if normalize {
        let d = diffuse.as_raw();
        let occl = |i: usize| {
            ao.as_ref()
                .map(|m| 0.35 + 0.65 * m.as_raw()[i * 4] as f32 / 255.0)
                .unwrap_or(1.0)
        };
        let n = d.len() / 4;
        let mean = (0..n)
            .map(|i| {
                (d[i * 4] as f32 * 0.3 + d[i * 4 + 1] as f32 * 0.59 + d[i * 4 + 2] as f32 * 0.11)
                    / 255.0
                    * occl(i)
            })
            .sum::<f32>()
            / n as f32;
        gain = (0.92 / mean.max(0.05)).clamp(0.5, 4.0);
    }
    let n = (SIZE * SIZE) as usize;
    let mut a = Vec::with_capacity(n * 4);
    let mut nrh = Vec::with_capacity(n * 4);
    for i in 0..n {
        let d = diffuse.as_raw();
        let occl = ao
            .as_ref()
            .map(|m| m.as_raw()[i * 4] as f32 / 255.0)
            .unwrap_or(1.0);
        // AO i sRGB-rom er en grov tilnærming, men bra nok for skygger i fugene.
        let k = (0.35 + 0.65 * occl) * gain;
        let c = |v: u8| (v as f32 * k).min(255.0) as u8;
        a.extend_from_slice(&[c(d[i * 4]), c(d[i * 4 + 1]), c(d[i * 4 + 2]), 255]);
        // Poly Haven-normalene er i OpenGL-konvensjon, som de bakte lagene
        // (prøvd med en syntetisk kule: lyset faller riktig uten snuing).
        let (nx, ny) = normal
            .as_ref()
            .map(|m| (m.as_raw()[i * 4], m.as_raw()[i * 4 + 1]))
            .unwrap_or((128, 128));
        let r = rough.as_ref().map(|m| m.as_raw()[i * 4]).unwrap_or(200);
        let h = height.as_ref().map(|m| m.as_raw()[i * 4]).unwrap_or(128);
        nrh.extend_from_slice(&[nx, ny, r, h]);
    }
    (a, nrh)
}
