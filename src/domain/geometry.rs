//! Geometri: vertex-formatet og en bygger for former. Rene data; hvordan
//! de legges i GPU-buffere er rendererens sak.
//!
//! Byen bygges av bokser, prismer, sylindre, dreide profiler, ekstruderte
//! profiler og kuler her. Hver vertex bærer et materiale: enten et lag i de
//! bakte teksturene (brostein, puss, panel …) som legges på triplanart i
//! shaderen, eller en spesialtype som glass, blad, tekst og lampeglass som
//! bruker uv.

use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// Farge i sRGB, og ruhetsfaktor i alfa.
    pub color: [u8; 4],
    /// Type, palettplass, flagg og okklusjon.
    pub mat: [u8; 4],
    /// Skinning: fire ledd og vektene deres (0..255, summen 255). Alt
    /// null for det som ikke er skinnet.
    pub joints: [u8; 4],
    pub weights: [u8; 4],
}

/// Lagene i de bakte teksturene. Må stemme med `bake.wgsl` og `TEX_SCALE`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Layer {
    Cobble = 0,
    Plaster = 1,
    Cladding = 2,
    Slate = 3,
    Tiles = 4,
    Stone = 5,
    Brick = 6,
    Oak = 7,
    Metal = 8,
    Grass = 9,
    Bark = 10,
    Gravel = 11,
    Concrete = 12,
    /// Høvlet, lyst bygningsvirke (bindingsverk, trelast).
    Timber = 13,
    /// OSB-/sponplater.
    Plywood = 14,
    /// Fjellsider: lagdelt berg i stor skala (30 m per flis).
    Rock = 15,
    /// Snø på toppene.
    Snow = 16,
    /// Tråkket jord uten hjulspor (gårdsplasser).
    Earth = 17,
    /// Halmtak.
    Thatch = 18,
    /// Laftet tømmer (liggende stokker med mose i fugene).
    Logs = 19,
    /// Torvtak.
    Sod = 20,
    /// Ullstoff (klær).
    Wool = 21,
    /// Linstoff (skjorter, hodeplagg, forklær).
    Linen = 22,
    /// Lær (belter, sko, bokbind).
    Leather = 23,
}
pub const LAYERS: u32 = 24;

impl Layer {
    pub const ALL: [Layer; LAYERS as usize] = [
        Layer::Cobble, Layer::Plaster, Layer::Cladding, Layer::Slate, Layer::Tiles, Layer::Stone, Layer::Brick,
        Layer::Oak, Layer::Metal, Layer::Grass, Layer::Bark, Layer::Gravel, Layer::Concrete, Layer::Timber, Layer::Plywood, Layer::Rock, Layer::Snow, Layer::Earth, Layer::Thatch, Layer::Logs, Layer::Sod, Layer::Wool, Layer::Linen, Layer::Leather,
    ];
    /// Navnet i `assets/external/manifest.json`.
    pub fn name(self) -> &'static str {
        match self {
            Layer::Cobble => "cobble",
            Layer::Plaster => "plaster",
            Layer::Cladding => "cladding",
            Layer::Slate => "slate",
            Layer::Tiles => "tiles",
            Layer::Stone => "stone",
            Layer::Brick => "brick",
            Layer::Oak => "oak",
            Layer::Metal => "metal",
            Layer::Grass => "grass",
            Layer::Bark => "bark",
            Layer::Gravel => "gravel",
            Layer::Concrete => "concrete",
            Layer::Timber => "timber",
            Layer::Plywood => "plywood",
            Layer::Rock => "rock",
            Layer::Snow => "snow",
            Layer::Earth => "earth",
            Layer::Thatch => "thatch",
            Layer::Logs => "logs",
            Layer::Sod => "sod",
            Layer::Wool => "wool",
            Layer::Linen => "linen",
            Layer::Leather => "leather",
        }
    }
}

/// Meter per teksturflis for hvert lag.
pub const TEX_SCALE: [f32; LAYERS as usize] = [2.0, 3.0, 2.0, 2.0, 1.5, 2.5, 1.5, 1.5, 1.0, 3.0, 1.2, 2.0, 2.0, 1.2, 2.4, 30.0, 6.0, 2.5, 2.0, 2.0, 2.5, 0.2, 0.35, 0.3];

/// Spesialtyper som ikke er teksturlag. Må stemme med `world.wgsl`.
pub mod kind {
    /// Flat farge og ruhet fra vertexen.
    pub const PLAIN: u8 = 200;
    /// Vindusglass med rom bak (interior mapping) og refleksjon.
    pub const GLASS: u8 = 201;
    /// Bladkort med alfa, uv inn i bladteksturen.
    pub const LEAF: u8 = 202;
    /// Lampeglass som lyser om natta.
    pub const LAMP: u8 = 204;
    /// Skeleton-komponenten: grå flate som skimrer.
    pub const SKELETON: u8 = 205;
    /// Vann i en kum eller pytt: speiler himmelen.
    pub const WATER: u8 = 206;
    /// Modell med egne teksturer (albedo, normal, metall/ruhet, emissiv).
    pub const MODEL: u8 = 207;
    /// Bakken: hjørnefargen er vektene til brostein, jord, berg og
    /// skogbunn (gress er resten), uv er tørrhet og søle. Shaderen blander
    /// lagene etter høydekartene.
    pub const TERRAIN: u8 = 209;
}

/// Flagg i `mat.z`. De to nederste bitene er glød.
pub mod flag {
    pub const GLOW_LAMP: u8 = 2;
    pub const GLOW_ALWAYS: u8 = 3;
    /// Svaier i vinden, mer jo høyere opp (uv.y brukes som vekt for blad).
    pub const WIND: u8 = 4;
    /// Teksturen roteres 90° på veggene (stående panel).
    pub const ROT_UV: u8 = 8;
    /// Ikke snø eller regnvåt (undersider, innendørs).
    pub const NO_WEATHER: u8 = 16;
    /// Ingen slitasje nederst på veggen.
    pub const NO_DIRT: u8 = 32;
    /// Modellflater med alfamaske (blader på kort): alfatestes i alle pass.
    pub const ALPHA: u8 = 64;
    /// Modellflater som er metall (krom): metallfaktoren er 1.
    pub const METAL: u8 = 128;
    /// Teksturlag lagt på etter flatens egen uv (i meter) i stedet for
    /// triplanart: modellene fra Blender med «layer_»-materialer, så
    /// årene følger bjelkene og radene følger takflaten. Samme bit som
    /// METAL, som ikke betyr noe for teksturlag.
    pub const UVMAP: u8 = 128;
    /// Modellflater som er lakk: fargen kommer fra instansen (`Placement::tints`).
    /// Samme bit som ROT_UV, som ikke betyr noe for modeller.
    pub const PAINT: u8 = 8;
}

#[derive(Clone, Copy, Debug)]
pub struct Mat {
    pub srgb: [u8; 3],
    /// For teksturlag: faktor på bakt ruhet (0,5 er uendret). Ellers ruheten.
    pub roughness: f32,
    pub kind: u8,
    /// 0 bruker `srgb`. 1–15 henter fargen fra paletten, som styres av
    /// Fristil-tokens og temaet.
    pub slot: u8,
    pub flags: u8,
}

impl Mat {
    const fn rgb(srgb: u32) -> [u8; 3] {
        [(srgb >> 16) as u8, (srgb >> 8) as u8, srgb as u8]
    }
    /// Teksturlag med fargetone. `0xffffff` lar teksturen stå som den er.
    pub const fn tex(layer: Layer, tint: u32) -> Self {
        Self { srgb: Self::rgb(tint), roughness: 0.5, kind: layer as u8, slot: 0, flags: 0 }
    }
    pub const fn plain(srgb: u32, roughness: f32) -> Self {
        Self { srgb: Self::rgb(srgb), roughness, kind: kind::PLAIN, slot: 0, flags: 0 }
    }
    pub const fn special(kind: u8, srgb: u32, roughness: f32) -> Self {
        Self { srgb: Self::rgb(srgb), roughness, kind, slot: 0, flags: 0 }
    }
    pub const fn slot(mut self, slot: u8) -> Self {
        self.slot = slot;
        self
    }
    pub const fn rough(mut self, r: f32) -> Self {
        self.roughness = r;
        self
    }
    pub const fn flags(mut self, f: u8) -> Self {
        self.flags |= f;
        self
    }
    pub fn tint(mut self, c: Vec3) -> Self {
        self.srgb = to_srgb8(c);
        self
    }
}

/// Lineær farge til sRGB-byte.
pub fn to_srgb8(c: Vec3) -> [u8; 3] {
    let f = |x: f32| {
        let s = if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        (s.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    };
    [f(c.x), f(c.y), f(c.z)]
}

#[derive(Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// En bit av et mesh med egen boks, så rendereren kan hoppe over det som
/// er utenfor synsfeltet. Indeksene ligger [grov][fin] etter hverandre:
/// de fine trekantene (småting) trengs bare nær kameraet.
#[derive(Clone, Debug)]
pub struct Chunk {
    pub min: Vec3,
    pub max: Vec3,
    /// Indeksområdet for de grove trekantene.
    pub coarse: std::ops::Range<u32>,
    /// Indeksområdet for de fine, rett etter de grove.
    pub fine: std::ops::Range<u32>,
}

impl Chunk {
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Avstanden fra et punkt til boksen (0 innenfor).
    pub fn distance(&self, p: Vec3) -> f32 {
        let q = p.clamp(self.min, self.max);
        p.distance(q)
    }
}

impl Mesh {
    pub fn vert(&mut self, p: Vec3, n: Vec3, uv: Vec2, m: &Mat, ao: f32) -> u32 {
        let i = self.vertices.len() as u32;
        self.vertices.push(Vertex {
            pos: p.to_array(),
            normal: n.normalize_or_zero().to_array(),
            uv: uv.to_array(),
            color: [m.srgb[0], m.srgb[1], m.srgb[2], (m.roughness.clamp(0.0, 1.0) * 255.0) as u8],
            mat: [m.kind, m.slot, m.flags, (ao.clamp(0.0, 1.0) * 255.0) as u8],
            joints: [0; 4],
            weights: [0; 4],
        });
        i
    }

    /// Firkant med hjørnene mot klokka sett forfra, med uv 0..1.
    pub fn quad(&mut self, t: &Affine3A, c: [Vec3; 4], m: &Mat, ao: [f32; 4]) {
        self.quad_uv(t, c, m, ao, [Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)]);
    }

    pub fn quad_uv(&mut self, t: &Affine3A, c: [Vec3; 4], m: &Mat, ao: [f32; 4], uv: [Vec2; 4]) {
        let p: Vec<Vec3> = c.iter().map(|v| t.transform_point3(*v)).collect();
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let b = [
            self.vert(p[0], n, uv[0], m, ao[0]),
            self.vert(p[1], n, uv[1], m, ao[1]),
            self.vert(p[2], n, uv[2], m, ao[2]),
            self.vert(p[3], n, uv[3], m, ao[3]),
        ];
        self.indices.extend_from_slice(&[b[0], b[1], b[2], b[0], b[2], b[3]]);
    }

    /// Firkant med egne normaler per hjørne, til myke flater.
    pub fn quad_n(&mut self, p: [Vec3; 4], n: [Vec3; 4], m: &Mat, ao: f32) {
        let uv = [Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)];
        let b = [
            self.vert(p[0], n[0], uv[0], m, ao),
            self.vert(p[1], n[1], uv[1], m, ao),
            self.vert(p[2], n[2], uv[2], m, ao),
            self.vert(p[3], n[3], uv[3], m, ao),
        ];
        self.indices.extend_from_slice(&[b[0], b[1], b[2], b[0], b[2], b[3]]);
    }

    pub fn tri(&mut self, t: &Affine3A, c: [Vec3; 3], m: &Mat) {
        let p: Vec<Vec3> = c.iter().map(|v| t.transform_point3(*v)).collect();
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let a = self.vert(p[0], n, Vec2::ZERO, m, 1.0);
        let b = self.vert(p[1], n, Vec2::X, m, 1.0);
        let c = self.vert(p[2], n, Vec2::Y, m, 1.0);
        self.indices.extend_from_slice(&[a, b, c]);
    }

    /// Boks med sentrum i origo i `t` og hele mål `size`. Bunnen mørkes litt
    /// nedover for å antyde okklusjon der den møter bakken.
    pub fn cuboid(&mut self, t: &Affine3A, size: Vec3, m: &Mat) {
        self.cuboid_ao(t, size, m, 0.7)
    }

    pub fn cuboid_ao(&mut self, t: &Affine3A, size: Vec3, m: &Mat, foot_ao: f32) {
        self.cuboid_faces(t, size, [m; 6], foot_ao)
    }

    /// Boks med eget materiale per side: +Z, −Z, +X, −X, topp, bunn.
    pub fn cuboid_faces(&mut self, t: &Affine3A, size: Vec3, m: [&Mat; 6], foot_ao: f32) {
        let h = size * 0.5;
        let (x, y, z) = (h.x, h.y, h.z);
        let lo = foot_ao;
        let sides = [
            [Vec3::new(-x, -y, z), Vec3::new(x, -y, z), Vec3::new(x, y, z), Vec3::new(-x, y, z)],
            [Vec3::new(x, -y, -z), Vec3::new(-x, -y, -z), Vec3::new(-x, y, -z), Vec3::new(x, y, -z)],
            [Vec3::new(x, -y, z), Vec3::new(x, -y, -z), Vec3::new(x, y, -z), Vec3::new(x, y, z)],
            [Vec3::new(-x, -y, -z), Vec3::new(-x, -y, z), Vec3::new(-x, y, z), Vec3::new(-x, y, -z)],
        ];
        for (i, s) in sides.into_iter().enumerate() {
            self.quad(t, s, m[i], [lo, lo, 1.0, 1.0]);
        }
        self.quad(t, [Vec3::new(-x, y, z), Vec3::new(x, y, z), Vec3::new(x, y, -z), Vec3::new(-x, y, -z)], m[4], [1.0; 4]);
        self.quad(t, [Vec3::new(-x, -y, -z), Vec3::new(x, -y, -z), Vec3::new(x, -y, z), Vec3::new(-x, -y, z)], m[5], [lo; 4]);
    }

    /// Saltak over en grunnflate `w` × `d` (sentrert, med mønet langs X),
    /// med takfoten i y = 0 og mønet i y = `height`. Takflatene er tykke,
    /// så takfoten og gavlkantene synes.
    pub fn gable_roof(&mut self, t: &Affine3A, w: f32, d: f32, height: f32, overhang: f32, roof: &Mat, gable: &Mat, edge: &Mat) {
        let x = w * 0.5 + overhang;
        let z = d * 0.5 + overhang;
        let gx = w * 0.5;
        let gz = d * 0.5;
        let th = 0.14;
        for side in [1.0f32, -1.0] {
            let eave_y = -overhang * height / gz.max(0.01);
            let a = Vec3::new(-x, eave_y, side * z);
            let b = Vec3::new(x, eave_y, side * z);
            let c = Vec3::new(x, height, 0.0);
            let d2 = Vec3::new(-x, height, 0.0);
            // Tykkelsen vinkelrett på takflaten, oppover og utover.
            let slope = Vec3::new(0.0, height - eave_y, -side * z);
            let mut up = Vec3::X.cross(slope).normalize();
            if up.y < 0.0 {
                up = -up;
            }
            let up = up * th;
            if side > 0.0 {
                self.quad(t, [a + up, b + up, c + up, d2 + up], roof, [1.0; 4]);
                self.quad(t, [b, a, d2, c], edge, [0.6; 4]);
                self.quad(t, [a, b, b + up, a + up], edge, [1.0; 4]);
            } else {
                self.quad(t, [b + up, a + up, d2 + up, c + up], roof, [1.0; 4]);
                self.quad(t, [a, b, c, d2], edge, [0.6; 4]);
                self.quad(t, [b, a, a + up, b + up], edge, [1.0; 4]);
            }
            for ex in [-x, x] {
                let s = ex.signum();
                let p0 = Vec3::new(ex, eave_y, side * z);
                let p1 = Vec3::new(ex, height, 0.0);
                if (s > 0.0) == (side > 0.0) {
                    self.quad(t, [p0, p1, p1 + up, p0 + up], edge, [1.0; 4]);
                } else {
                    self.quad(t, [p1, p0, p0 + up, p1 + up], edge, [1.0; 4]);
                }
            }
        }
        self.tri(t, [Vec3::new(gx, 0.0, gz), Vec3::new(gx, 0.0, -gz), Vec3::new(gx, height, 0.0)], gable);
        self.tri(t, [Vec3::new(-gx, 0.0, -gz), Vec3::new(-gx, 0.0, gz), Vec3::new(-gx, height, 0.0)], gable);
    }

    /// Stående sylinder med bunnen i origo i `t`.
    pub fn cylinder(&mut self, t: &Affine3A, r_bottom: f32, r_top: f32, h: f32, seg: u32, m: &Mat, caps: bool) {
        self.lathe(t, &[Vec2::new(r_bottom, 0.0), Vec2::new(r_top, h)], seg, m, caps, 0.75);
    }

    /// Dreid profil: `profile` er (radius, høyde) nedenfra og opp, rundt Y.
    /// Normalene er myke langs rundingen og harde mellom profilsegmentene.
    pub fn lathe(&mut self, t: &Affine3A, profile: &[Vec2], seg: u32, m: &Mat, caps: bool, foot_ao: f32) {
        let ring = |r: f32, y: f32, i: u32| {
            let a = i as f32 / seg as f32 * std::f32::consts::TAU;
            Vec3::new(a.cos() * r, y, a.sin() * r)
        };
        let n_prof = profile.len();
        for k in 0..n_prof - 1 {
            let (p0, p1) = (profile[k], profile[k + 1]);
            let d = p1 - p0;
            // Normalen i profilplanet, vinkelrett på segmentet, pekende utover.
            let nn = Vec2::new(d.y, -d.x).normalize_or_zero();
            let ao0 = if k == 0 { foot_ao } else { 1.0 };
            for i in 0..seg {
                let a0 = ring(p0.x, p0.y, i);
                let a1 = ring(p0.x, p0.y, i + 1);
                let b0 = ring(p1.x, p1.y, i);
                let b1 = ring(p1.x, p1.y, i + 1);
                let rn = |p: Vec3| {
                    let h = Vec3::new(p.x, 0.0, p.z).normalize_or_zero();
                    if h == Vec3::ZERO { Vec3::Y } else { h * nn.x + Vec3::Y * nn.y }
                };
                let p = [a0, a1, b1, b0].map(|v| t.transform_point3(v));
                let tn = |n: Vec3| t.transform_vector3(n);
                let u0 = i as f32 / seg as f32;
                let u1 = (i + 1) as f32 / seg as f32;
                let i0 = self.vert(p[0], tn(rn(a0)), Vec2::new(u0, 0.0), m, ao0);
                let i1 = self.vert(p[1], tn(rn(a1)), Vec2::new(u1, 0.0), m, ao0);
                let i2 = self.vert(p[2], tn(rn(b1)), Vec2::new(u1, 1.0), m, 1.0);
                let i3 = self.vert(p[3], tn(rn(b0)), Vec2::new(u0, 1.0), m, 1.0);
                self.indices.extend_from_slice(&[i0, i2, i1, i0, i3, i2]);
            }
        }
        if caps {
            let top = profile[n_prof - 1];
            let bot = profile[0];
            for i in 0..seg {
                if top.x > 0.0 {
                    self.tri(t, [Vec3::new(0.0, top.y, 0.0), ring(top.x, top.y, i + 1), ring(top.x, top.y, i)], m);
                }
                if bot.x > 0.0 {
                    self.tri(t, [Vec3::new(0.0, bot.y, 0.0), ring(bot.x, bot.y, i), ring(bot.x, bot.y, i + 1)], m);
                }
            }
        }
    }

    /// Kule med jevne normaler.
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
                self.vert(p, nn, Vec2::new(u / std::f32::consts::TAU, v / std::f32::consts::PI), m, ao);
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

    /// Ring (torus) liggende i XZ-planet med sentrum i origo i `t`.
    pub fn torus(&mut self, t: &Affine3A, radius: f32, tube: f32, seg: u32, sides: u32, m: &Mat) {
        let base = self.vertices.len() as u32;
        for i in 0..=seg {
            let a = i as f32 / seg as f32 * std::f32::consts::TAU;
            let c = Vec3::new(a.cos(), 0.0, a.sin());
            for j in 0..=sides {
                let b = j as f32 / sides as f32 * std::f32::consts::TAU;
                let n = c * b.cos() + Vec3::Y * b.sin();
                let p = c * radius + n * tube;
                self.vert(t.transform_point3(p), t.transform_vector3(n), Vec2::new(i as f32 / seg as f32, j as f32 / sides as f32), m, 1.0);
            }
        }
        let w = sides + 1;
        for i in 0..seg {
            for j in 0..sides {
                let a = base + i * w + j;
                let b = a + w;
                self.indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }
    }

    /// Profil ekstrudert langs en rett linje fra `from` til `to`. Profilen
    /// er (ut, opp) i et plan vinkelrett på linja, der «ut» peker mot
    /// `outward`. Brukes til gesimser, vindusbrett og takrenner.
    pub fn extrude(&mut self, profile: &[Vec2], from: Vec3, to: Vec3, outward: Vec3, m: &Mat, closed_ends: bool) {
        let along = (to - from).normalize_or_zero();
        let out = (outward - along * outward.dot(along)).normalize_or_zero();
        let up = along.cross(out).normalize_or_zero();
        let up = if up.y < 0.0 { -up } else { up };
        let out = up.cross(along).normalize_or_zero();
        let out = if out.dot(outward) < 0.0 { -out } else { out };
        let at = |p: Vec2, base: Vec3| base + out * p.x + up * p.y;
        for k in 0..profile.len() - 1 {
            let (p0, p1) = (profile[k], profile[k + 1]);
            let a = at(p0, from);
            let b = at(p0, to);
            let c = at(p1, to);
            let d = at(p1, from);
            let n = (b - a).cross(d - a);
            // Vend trekantene så normalen peker ut fra profilen.
            let d2 = p1 - p0;
            let want = out * d2.y - up * d2.x;
            if n.dot(want) >= 0.0 {
                self.quad(&Affine3A::IDENTITY, [a, b, c, d], m, [1.0; 4]);
            } else {
                self.quad(&Affine3A::IDENTITY, [b, a, d, c], m, [1.0; 4]);
            }
        }
        if closed_ends && profile.len() >= 3 {
            // Vifte fra første punkt.
            for k in 1..profile.len() - 1 {
                let (a, b, c) = (at(profile[0], from), at(profile[k], from), at(profile[k + 1], from));
                let n = (b - a).cross(c - a);
                if n.dot(along) < 0.0 {
                    self.tri(&Affine3A::IDENTITY, [a, b, c], m);
                } else {
                    self.tri(&Affine3A::IDENTITY, [a, c, b], m);
                }
                let (a, b, c) = (at(profile[0], to), at(profile[k], to), at(profile[k + 1], to));
                let n = (b - a).cross(c - a);
                if n.dot(along) > 0.0 {
                    self.tri(&Affine3A::IDENTITY, [a, b, c], m);
                } else {
                    self.tri(&Affine3A::IDENTITY, [a, c, b], m);
                }
            }
        }
    }

    /// Avrundet rektangel som en plate, forsiden mot +Z, sentrert i `c`.
    pub fn rounded_slab(&mut self, t: &Affine3A, c: Vec3, w: f32, h: f32, d: f32, r: f32, m: &Mat) {
        let r = r.min(w * 0.5).min(h * 0.5);
        // Flere segmenter på store hjørner, så sirkler ikke blir kantete.
        let corner_seg = ((r / 0.02) as usize).clamp(6, 16);
        // Omrisset mot klokka sett forfra.
        let mut outline = vec![];
        for (k, (sx, sy)) in [(1.0f32, -1.0f32), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)].iter().enumerate() {
            let cx = sx * (w * 0.5 - r);
            let cy = sy * (h * 0.5 - r);
            let a0 = k as f32 * std::f32::consts::FRAC_PI_2 - std::f32::consts::FRAC_PI_2;
            for s in 0..=corner_seg {
                let a = a0 + s as f32 / corner_seg as f32 * std::f32::consts::FRAC_PI_2;
                outline.push(Vec2::new(cx + a.cos() * r, cy + a.sin() * r));
            }
        }
        let n = outline.len();
        let front = c + Vec3::Z * (d * 0.5);
        let back = c - Vec3::Z * (d * 0.5);
        // For- og bakside som vifter.
        let fc = self.vert(t.transform_point3(front), t.transform_vector3(Vec3::Z), Vec2::splat(0.5), m, 1.0);
        let bc = self.vert(t.transform_point3(back), t.transform_vector3(-Vec3::Z), Vec2::splat(0.5), m, 1.0);
        let mut fi = vec![];
        let mut bi = vec![];
        for o in &outline {
            let uv = Vec2::new(o.x / w + 0.5, 0.5 - o.y / h);
            fi.push(self.vert(t.transform_point3(front + Vec3::new(o.x, o.y, 0.0)), t.transform_vector3(Vec3::Z), uv, m, 1.0));
            bi.push(self.vert(t.transform_point3(back + Vec3::new(o.x, o.y, 0.0)), t.transform_vector3(-Vec3::Z), uv, m, 1.0));
        }
        for k in 0..n {
            let k2 = (k + 1) % n;
            self.indices.extend_from_slice(&[fc, fi[k], fi[k2], bc, bi[k2], bi[k]]);
            // Siden.
            let o0 = outline[k];
            let o1 = outline[k2];
            let edge = o1 - o0;
            let nn = Vec3::new(edge.y, -edge.x, 0.0).normalize_or_zero();
            let p = [
                front + Vec3::new(o0.x, o0.y, 0.0),
                back + Vec3::new(o0.x, o0.y, 0.0),
                back + Vec3::new(o1.x, o1.y, 0.0),
                front + Vec3::new(o1.x, o1.y, 0.0),
            ];
            let i0 = self.vert(t.transform_point3(p[0]), t.transform_vector3(nn), Vec2::ZERO, m, 1.0);
            let i1 = self.vert(t.transform_point3(p[1]), t.transform_vector3(nn), Vec2::ZERO, m, 1.0);
            let i2 = self.vert(t.transform_point3(p[2]), t.transform_vector3(nn), Vec2::ZERO, m, 1.0);
            let i3 = self.vert(t.transform_point3(p[3]), t.transform_vector3(nn), Vec2::ZERO, m, 1.0);
            self.indices.extend_from_slice(&[i0, i2, i1, i0, i3, i2]);
        }
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    pub fn transform(&mut self, t: &Affine3A) {
        for v in &mut self.vertices {
            v.pos = t.transform_point3(Vec3::from(v.pos)).to_array();
            v.normal = t.transform_vector3(Vec3::from(v.normal)).normalize_or_zero().to_array();
        }
    }

    /// Del meshet i biter på `cell` meter i XZ-planet, sortert i rader, og
    /// legg trekanter mindre enn `fine_radius` sist i hver bit. Hjørnene
    /// beholdes, bare indeksene stokkes om.
    pub fn chunked(&self, cell: f32, fine_radius: f32) -> (Mesh, Vec<Chunk>) {
        use std::collections::BTreeMap;
        let pos = |i: u32| Vec3::from(self.vertices[i as usize].pos);
        // (rad, kolonne, fin) → trekanter.
        let mut buckets: BTreeMap<(i32, i32, bool), Vec<usize>> = BTreeMap::new();
        for t in 0..self.indices.len() / 3 {
            let (a, b, c) = (pos(self.indices[t * 3]), pos(self.indices[t * 3 + 1]), pos(self.indices[t * 3 + 2]));
            let centroid = (a + b + c) / 3.0;
            let r = a.distance(centroid).max(b.distance(centroid)).max(c.distance(centroid));
            let key = ((centroid.z / cell).floor() as i32, (centroid.x / cell).floor() as i32, r < fine_radius);
            buckets.entry(key).or_default().push(t);
        }
        let mut indices = Vec::with_capacity(self.indices.len());
        let mut chunks: Vec<Chunk> = vec![];
        let mut current: Option<(i32, i32)> = None;
        for ((row, col, fine), tris) in buckets {
            let start = indices.len() as u32;
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            for t in tris {
                for k in 0..3 {
                    let i = self.indices[t * 3 + k];
                    indices.push(i);
                    let p = pos(i);
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
            }
            let end = indices.len() as u32;
            if current != Some((row, col)) {
                current = Some((row, col));
                chunks.push(Chunk { min: lo, max: hi, coarse: start..start, fine: start..start });
            }
            let c = chunks.last_mut().unwrap();
            c.min = c.min.min(lo);
            c.max = c.max.max(hi);
            if fine {
                c.fine = start..end;
            } else {
                c.coarse = start..end;
                c.fine = end..end;
            }
        }
        (Mesh { vertices: self.vertices.clone(), indices }, chunks)
    }

    /// Grovere bladverk til bruk på avstand: meshet er kort (fire hjørner
    /// og to trekanter hver, slik `card_mesh` i trees.py lager dem); hvert
    /// `keep`-te kort beholdes og forstørres `grow` ganger om midten sitt,
    /// så krona holder på tettheten med en brøkdel av trekantene.
    pub fn thinned_cards(&self, keep: usize, grow: f32) -> Mesh {
        let mut out = Mesh::default();
        for (k, tri) in self.indices.chunks(6).enumerate() {
            if k % keep != 0 || tri.len() < 6 {
                continue;
            }
            let mut ids: Vec<u32> = tri.to_vec();
            ids.sort_unstable();
            ids.dedup();
            let center = ids.iter().map(|i| Vec3::from(self.vertices[*i as usize].pos)).sum::<Vec3>() / ids.len() as f32;
            let base = out.vertices.len() as u32;
            for i in &ids {
                let mut v = self.vertices[*i as usize];
                v.pos = (center + (Vec3::from(v.pos) - center) * grow).to_array();
                out.vertices.push(v);
            }
            for i in tri {
                let j = ids.iter().position(|x| x == i).unwrap_or(0) as u32;
                out.indices.push(base + j);
            }
        }
        out
    }

    /// En grovere utgave til bruk på avstand: hjørnene snappes til et
    /// rutenett på `cell` meter og slås sammen, og trekanter som faller
    /// sammen fjernes. Posisjon og normal midles i hver rute, uv og
    /// materiale tas fra det første hjørnet.
    pub fn clustered(&self, cell: f32) -> Mesh {
        use std::collections::HashMap;
        // Bare hjørner med samme materiale og omtrent samme retning slås
        // sammen, så puss og bjelker ikke blandes og flatene holder seg flate.
        let mut cells: HashMap<(i32, i32, i32, u8, [u8; 4], [u8; 4]), u32> = HashMap::new();
        let mut remap = vec![0u32; self.vertices.len()];
        let mut out = Mesh::default();
        let mut sums: Vec<(Vec3, Vec3, u32)> = vec![];
        for (i, v) in self.vertices.iter().enumerate() {
            let p = Vec3::from(v.pos);
            let n = Vec3::from(v.normal);
            let axis = (0..3).max_by(|&a, &b| n[a].abs().total_cmp(&n[b].abs())).unwrap_or(1);
            let facing = axis as u8 * 2 + (n[axis] < 0.0) as u8;
            let mat = [v.mat[0], v.mat[1], v.mat[2], 0];
            let key = ((p.x / cell).floor() as i32, (p.y / cell).floor() as i32, (p.z / cell).floor() as i32, facing, mat, v.color);
            let j = *cells.entry(key).or_insert_with(|| {
                out.vertices.push(*v);
                sums.push((Vec3::ZERO, Vec3::ZERO, 0));
                (out.vertices.len() - 1) as u32
            });
            remap[i] = j;
            let s = &mut sums[j as usize];
            s.0 += p;
            s.1 += Vec3::from(v.normal);
            s.2 += 1;
        }
        for (v, (p, n, k)) in out.vertices.iter_mut().zip(&sums) {
            v.pos = (*p / *k as f32).to_array();
            v.normal = n.normalize_or_zero().to_array();
        }
        for t in self.indices.chunks(3) {
            let (a, b, c) = (remap[t[0] as usize], remap[t[1] as usize], remap[t[2] as usize]);
            if a != b && b != c && a != c {
                out.indices.extend_from_slice(&[a, b, c]);
            }
        }
        out
    }

    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for v in &self.vertices {
            lo = lo.min(Vec3::from(v.pos));
            hi = hi.max(Vec3::from(v.pos));
        }
        (lo, hi)
    }
}
