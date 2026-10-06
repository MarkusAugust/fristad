//! Geometri: vertex-formatet og en bygger for enkle former.
//!
//! Byen har ingen modellfiler. Alt bygges av bokser, prismer, sylindre og
//! kuler her, og detaljene (brostein, puss, treverk) tegnes i shaderen ut fra
//! materialet hver vertex bærer.

use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    /// Grunnfarge i sRGB, og ruhet i alfa.
    pub color: [u8; 4],
    /// Materiale, palettplass, lysgruppe og okklusjon.
    pub mat: [u8; 4],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Unorm8x4,
            3 => Uint8x4,
        ],
    };
}

/// Materialene shaderen kjenner. Må stemme med `world.wgsl`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Surface {
    Plain = 0,
    Cobble = 1,
    Plaster = 2,
    Wood = 3,
    Glass = 4,
    Slate = 5,
    Stone = 6,
    Grass = 7,
    Metal = 8,
    Leaves = 9,
}

/// Hva som lyser om natta. Må stemme med `world.wgsl`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Glow {
    None = 0,
    /// Vinduer, tent tilfeldig om kvelden.
    Window = 1,
    /// Lykter, alltid tent om natta.
    Lamp = 2,
    /// Lyser svakt hele tiden, som en opplyst knapp.
    Always = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct Mat {
    pub srgb: [u8; 3],
    pub roughness: f32,
    pub surface: Surface,
    /// 0 bruker `srgb`. 1–15 henter fargen fra paletten, som styres av
    /// Fristil-tokens og temaet.
    pub slot: u8,
    pub glow: Glow,
}

impl Mat {
    pub const fn new(srgb: u32, roughness: f32, surface: Surface) -> Self {
        Self {
            srgb: [(srgb >> 16) as u8, (srgb >> 8) as u8, srgb as u8],
            roughness,
            surface,
            slot: 0,
            glow: Glow::None,
        }
    }
    pub const fn slot(mut self, slot: u8) -> Self {
        self.slot = slot;
        self
    }
    pub const fn glow(mut self, glow: Glow) -> Self {
        self.glow = glow;
        self
    }
}

#[derive(Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    fn vert(&mut self, p: Vec3, n: Vec3, m: &Mat, ao: f32) -> u32 {
        let i = self.vertices.len() as u32;
        self.vertices.push(Vertex {
            pos: p.to_array(),
            normal: n.normalize_or_zero().to_array(),
            color: [
                m.srgb[0],
                m.srgb[1],
                m.srgb[2],
                (m.roughness.clamp(0.0, 1.0) * 255.0) as u8,
            ],
            mat: [
                m.surface as u8,
                m.slot,
                m.glow as u8,
                (ao.clamp(0.0, 1.0) * 255.0) as u8,
            ],
        });
        i
    }

    /// Firkant med hjørnene i rekkefølge mot klokka sett forfra.
    pub fn quad(&mut self, t: &Affine3A, c: [Vec3; 4], m: &Mat, ao: [f32; 4]) {
        let p: Vec<Vec3> = c.iter().map(|v| t.transform_point3(*v)).collect();
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let b = [
            self.vert(p[0], n, m, ao[0]),
            self.vert(p[1], n, m, ao[1]),
            self.vert(p[2], n, m, ao[2]),
            self.vert(p[3], n, m, ao[3]),
        ];
        self.indices
            .extend_from_slice(&[b[0], b[1], b[2], b[0], b[2], b[3]]);
    }

    pub fn tri(&mut self, t: &Affine3A, c: [Vec3; 3], m: &Mat) {
        let p: Vec<Vec3> = c.iter().map(|v| t.transform_point3(*v)).collect();
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let a = self.vert(p[0], n, m, 1.0);
        let b = self.vert(p[1], n, m, 1.0);
        let c = self.vert(p[2], n, m, 1.0);
        self.indices.extend_from_slice(&[a, b, c]);
    }

    /// Boks med sentrum i origo i `t` og hele mål `size`. Bunnen mørkes litt
    /// nedover for å antyde okklusjon der den møter bakken.
    pub fn cuboid(&mut self, t: &Affine3A, size: Vec3, m: &Mat) {
        self.cuboid_ao(t, size, m, 0.55)
    }

    pub fn cuboid_ao(&mut self, t: &Affine3A, size: Vec3, m: &Mat, foot_ao: f32) {
        let h = size * 0.5;
        let (x, y, z) = (h.x, h.y, h.z);
        let lo = foot_ao;
        // Sidene: nederste hjørner får okklusjon.
        let sides = [
            // +Z
            [
                Vec3::new(-x, -y, z),
                Vec3::new(x, -y, z),
                Vec3::new(x, y, z),
                Vec3::new(-x, y, z),
            ],
            // -Z
            [
                Vec3::new(x, -y, -z),
                Vec3::new(-x, -y, -z),
                Vec3::new(-x, y, -z),
                Vec3::new(x, y, -z),
            ],
            // +X
            [
                Vec3::new(x, -y, z),
                Vec3::new(x, -y, -z),
                Vec3::new(x, y, -z),
                Vec3::new(x, y, z),
            ],
            // -X
            [
                Vec3::new(-x, -y, -z),
                Vec3::new(-x, -y, z),
                Vec3::new(-x, y, z),
                Vec3::new(-x, y, -z),
            ],
        ];
        for s in sides {
            self.quad(t, s, m, [lo, lo, 1.0, 1.0]);
        }
        self.quad(
            t,
            [
                Vec3::new(-x, y, z),
                Vec3::new(x, y, z),
                Vec3::new(x, y, -z),
                Vec3::new(-x, y, -z),
            ],
            m,
            [1.0; 4],
        );
        self.quad(
            t,
            [
                Vec3::new(-x, -y, -z),
                Vec3::new(x, -y, -z),
                Vec3::new(x, -y, z),
                Vec3::new(-x, -y, z),
            ],
            m,
            [lo; 4],
        );
    }

    /// Saltak over en grunnflate `w` × `d` (sentrert, med mønet langs X),
    /// med takfoten i y = 0 og mønet i y = `height`.
    pub fn gable_roof(&mut self, t: &Affine3A, w: f32, d: f32, height: f32, overhang: f32, roof: &Mat, gable: &Mat) {
        let x = w * 0.5 + overhang;
        let z = d * 0.5 + overhang;
        let gx = w * 0.5;
        let gz = d * 0.5;
        let th = 0.12;
        // Takflatene, litt tykke så de kaster ordentlig skygge.
        for side in [1.0f32, -1.0] {
            let eave = Vec3::new(0.0, -overhang * height / gz.max(0.01), side * z);
            let ridge = Vec3::new(0.0, height, 0.0);
            let a = Vec3::new(-x, eave.y, eave.z);
            let b = Vec3::new(x, eave.y, eave.z);
            let c = Vec3::new(x, ridge.y, 0.0);
            let d2 = Vec3::new(-x, ridge.y, 0.0);
            let up = Vec3::Y * th;
            if side > 0.0 {
                self.quad(t, [a + up, b + up, c + up, d2 + up], roof, [1.0; 4]);
                self.quad(t, [b, a, d2, c], roof, [0.6; 4]);
                self.quad(t, [a, b, b + up, a + up], roof, [1.0; 4]);
            } else {
                self.quad(t, [b + up, a + up, d2 + up, c + up], roof, [1.0; 4]);
                self.quad(t, [a, b, c, d2], roof, [0.6; 4]);
                self.quad(t, [b, a, a + up, b + up], roof, [1.0; 4]);
            }
            // Endene av takflaten.
            for ex in [-x, x] {
                let s = ex.signum();
                let p0 = Vec3::new(ex, eave.y, eave.z);
                let p1 = Vec3::new(ex, ridge.y, 0.0);
                if (s > 0.0) == (side > 0.0) {
                    self.quad(t, [p0, p1, p1 + up, p0 + up], roof, [1.0; 4]);
                } else {
                    self.quad(t, [p1, p0, p0 + up, p1 + up], roof, [1.0; 4]);
                }
            }
        }
        // Gavlene.
        self.tri(
            t,
            [
                Vec3::new(gx, 0.0, gz),
                Vec3::new(gx, 0.0, -gz),
                Vec3::new(gx, height, 0.0),
            ],
            gable,
        );
        self.tri(
            t,
            [
                Vec3::new(-gx, 0.0, -gz),
                Vec3::new(-gx, 0.0, gz),
                Vec3::new(-gx, height, 0.0),
            ],
            gable,
        );
    }

    /// Stående sylinder med bunnen i origo i `t`.
    pub fn cylinder(&mut self, t: &Affine3A, r_bottom: f32, r_top: f32, h: f32, seg: u32, m: &Mat, caps: bool) {
        let ring = |r: f32, y: f32, i: u32| {
            let a = i as f32 / seg as f32 * std::f32::consts::TAU;
            Vec3::new(a.cos() * r, y, a.sin() * r)
        };
        let slope = (r_bottom - r_top) / h.max(1e-4);
        for i in 0..seg {
            let a0 = ring(r_bottom, 0.0, i);
            let a1 = ring(r_bottom, 0.0, i + 1);
            let b0 = ring(r_top, h, i);
            let b1 = ring(r_top, h, i + 1);
            let n0 = Vec3::new(a0.x, slope * r_bottom.max(r_top), a0.z);
            let n1 = Vec3::new(a1.x, slope * r_bottom.max(r_top), a1.z);
            let p = [a0, a1, b1, b0].map(|v| t.transform_point3(v));
            let tn = |n: Vec3| t.transform_vector3(n);
            let i0 = self.vert(p[0], tn(n0), m, 0.75);
            let i1 = self.vert(p[1], tn(n1), m, 0.75);
            let i2 = self.vert(p[2], tn(n1), m, 1.0);
            let i3 = self.vert(p[3], tn(n0), m, 1.0);
            self.indices.extend_from_slice(&[i0, i2, i1, i0, i3, i2]);
            if caps {
                self.tri(t, [Vec3::new(0.0, h, 0.0), b1, b0], m);
                if r_bottom > 0.0 {
                    self.tri(t, [Vec3::ZERO, a0, a1], m);
                }
            }
        }
    }

    /// Kule med jevne normaler, til trekroner og lyktehoder.
    pub fn sphere(&mut self, t: &Affine3A, r: Vec3, rings: u32, seg: u32, m: &Mat) {
        let base = self.vertices.len() as u32;
        for j in 0..=rings {
            let v = j as f32 / rings as f32 * std::f32::consts::PI;
            for i in 0..=seg {
                let u = i as f32 / seg as f32 * std::f32::consts::TAU;
                let n = Vec3::new(v.sin() * u.cos(), v.cos(), v.sin() * u.sin());
                let p = t.transform_point3(n * r);
                let nn = t.transform_vector3(n / r);
                let ao = 0.55 + 0.45 * (n.y * 0.5 + 0.5);
                self.vert(p, nn, m, ao);
            }
        }
        let w = seg + 1;
        for j in 0..rings {
            for i in 0..seg {
                let a = base + j * w + i;
                let b = a + w;
                self.indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
            }
        }
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }
}
