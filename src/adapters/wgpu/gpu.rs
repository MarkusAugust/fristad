//! Dataene slik grafikkortet får dem: hjørnene, lysene, verdiene per
//! bilde og per instans, og en mesh i buffere.

use super::*;

/// Hjørnet slik grafikkortet får det: som `Vertex`, men normalen som 8 bit
/// med fortegn (modellene har dem slik i fila uansett), 40 byte mot 48.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct GpuVertex {
    pub(super) pos: [f32; 3],
    pub(super) normal: [i8; 4],
    pub(super) uv: [f32; 2],
    pub(super) color: [u8; 4],
    pub(super) mat: [u8; 4],
    pub(super) joints: [u8; 4],
    pub(super) weights: [u8; 4],
}

impl GpuVertex {
    pub(super) fn from(v: &Vertex) -> Self {
        let q = |x: f32| (x.clamp(-1.0, 1.0) * 127.0).round() as i8;
        Self { pos: v.pos, normal: [q(v.normal[0]), q(v.normal[1]), q(v.normal[2]), 0], uv: v.uv, color: v.color, mat: v.mat, joints: v.joints, weights: v.weights }
    }
}

const _: () = assert!(std::mem::size_of::<GpuVertex>() == 40);

pub(super) const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct GpuLight {
    pub(super) pos_radius: [f32; 4],
    pub(super) color_intensity: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Globals {
    pub(super) view_proj: [[f32; 4]; 4],
    pub(super) inv_view_proj: [[f32; 4]; 4],
    pub(super) sun_vp: [[f32; 4]; 4],
    pub(super) camera: [f32; 4],
    pub(super) sun_dir: [f32; 4],
    pub(super) sun_color: [f32; 4],
    pub(super) sky_top: [f32; 4],
    pub(super) sky_horizon: [f32; 4],
    pub(super) ground_bounce: [f32; 4],
    pub(super) fog_color: [f32; 4],
    pub(super) params: [f32; 4],
    pub(super) weather: [f32; 4],
    pub(super) screen: [f32; 4],
    /// x vind, y MSAA-prøver, z der skyggen begynner å tone bort, w der
    /// den slutter.
    pub(super) misc: [f32; 4],
    pub(super) sky: [f32; 4],
    pub(super) tex_scale: [[f32; 4]; 16],
    pub(super) palette: [[f32; 4]; PALETTE],
    pub(super) lights: [GpuLight; MAX_LIGHTS],
    pub(super) smoke: [[f32; 4]; MAX_SMOKE],
    /// Rommene i kryptnivået (lyset stoppes av murene): antallet (x, 0
    /// oppe i byen), boksene (x0, z0, x1, z1), gulvet og taket (to rom per
    /// vec4), og rommene hvert lys lyser i (to u32 per lys, to lys per vec4).
    pub(super) room_info: [f32; 4],
    pub(super) room_box: [[f32; 4]; MAX_ROOMS],
    pub(super) room_y: [[f32; 4]; MAX_ROOMS / 2],
    pub(super) light_rooms: [[u32; 4]; MAX_LIGHTS / 2],
    /// x full teksturtetthet, y flat farge (målingen `?bench`), z skyggen
    /// (0 av, 1 fire prøver, 2 myk), w blandede lag på bakken.
    pub(super) quality: [f32; 4],
    /// Kaskade nummer to.
    pub(super) sun_vp2: [[f32; 4]; 4],
    /// x, y en texel i skyggekartet (det har kaskadene side om side), z
    /// antall kaskader, w hvor langt kaskade to når.
    pub(super) shadow: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PostUniform {
    pub(super) params: [f32; 4],
    pub(super) texel: [f32; 4],
    /// Tonemappingen: xy sola på skjermen (uv), z styrken på solstrålene,
    /// w den malte graderingen (1) eller den gamle ACES-kurven (0).
    pub(super) sun: [f32; 4],
    /// x: spillerens lysstyrke (gamma, 1 er som malt).
    pub(super) extra: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
    pub extra: [f32; 4],
    /// x: plassen i leddpaletten + 1 for skinnede instanser, 0 ellers;
    /// z og w: steinbyttet (`with_look`).
    pub extra2: [f32; 4],
}

impl InstanceData {
    pub fn new(t: &Affine3A, extra: [f32; 4]) -> Self {
        Self { model: Mat4::from(*t).to_cols_array_2d(), extra, extra2: [0.0; 4] }
    }
    pub fn skinned(t: &Affine3A, extra: [f32; 4], base: u32) -> Self {
        Self { model: Mat4::from(*t).to_cols_array_2d(), extra, extra2: [base as f32 + 1.0, 0.0, 0.0, 0.0] }
    }
    /// Steinbyttet (`model::Look`), 6 bit per lag, eksakt i f32: z er gulvet
    /// (fra, til) og muren (fra, til), w hvelvet, bit 6 som slår det på og
    /// den hogde steinen (fra, til) i bit 7–18.
    pub fn with_look(mut self, look: Option<&crate::domain::model::Look>) -> Self {
        if let Some(l) = look {
            let b = |l: crate::domain::geometry::Layer| l as u32 & 63;
            self.extra2[2] = (b(l.floor.0) | b(l.floor.1) << 6 | b(l.wall.0) << 12 | b(l.wall.1) << 18) as f32;
            self.extra2[3] = (b(l.vault) | 1 << 6 | b(l.trim.0) << 7 | b(l.trim.1) << 13) as f32;
        }
        self
    }
    pub(super) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: INSTANCE_STRIDE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![7 => Float32x4, 8 => Float32x4, 9 => Float32x4, 10 => Float32x4, 11 => Float32x4, 12 => Float32x4],
    };
}

pub(super) struct GpuMesh {
    pub(super) vertices: wgpu::Buffer,
    pub(super) indices: wgpu::Buffer,
    /// 16 bit når meshen har færre enn 65 536 hjørner.
    pub(super) format: wgpu::IndexFormat,
}

impl GpuMesh {
    pub(super) fn new(device: &wgpu::Device, mesh: &Mesh, label: &str) -> Self {
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


    pub(super) fn bind(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_index_buffer(self.indices.slice(..), self.format);
    }
}
