//! Fristad bygges her, prosedyrisk, fra Fristil-tokens.
//!
//! Målestokken er at 1 rem er en meter: `spacing-4` (16px) er 1 m,
//! `spacing-12` (48px) er 3 m, som er etasjehøyden, og `spacing-6` (24px) er
//! 1,5 m, som er vindushøyden. Endres spacing-skalaen i Fristil, endres husene.
//!
//! Koordinater: X øst, Y opp, Z sør. Torget ligger rundt origo, og porten
//! med den store knappen står i nord.

use crate::mesh::{Glow, Mat, Mesh, Surface};
use crate::tokens::{Theme, Tokens};
use glam::{Affine3A, Quat, Vec3};

/// Palettplassene shaderen henter farge fra. Verdiene settes hvert bilde
/// fra tokens og tema, så husene kan bytte farge uten ny geometri.
pub mod slot {
    pub const BUTTON_BODY: u8 = 1;
    pub const BUTTON_RIM: u8 = 2;
    pub const BUTTON_LABEL: u8 = 3;
    pub const BRAND1: u8 = 4;
    pub const BRAND2: u8 = 5;
    pub const BRAND3: u8 = 6;
    pub const ACCENT: u8 = 7;
    pub const WARNING: u8 = 8;
}

pub enum Shape {
    /// Boks med sentrum, halve mål og rotasjon om Y.
    Cuboid { center: Vec3, half: Vec3, yaw: f32 },
    /// Stående sylinder med sentrum, radius og halv høyde.
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
}

pub struct PointLight {
    pub pos: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
}

/// Noe i verden som flytter seg: knappen og portdørene.
pub struct Prop {
    pub name: &'static str,
    pub mesh: Mesh,
    /// Hvor den står i ro.
    pub rest: Affine3A,
    /// Kollisjon i propens eget koordinatsystem.
    pub collider: Option<(Vec3, Vec3)>,
}

pub struct World {
    pub mesh: Mesh,
    pub colliders: Vec<Shape>,
    pub lights: Vec<PointLight>,
    pub props: Vec<Prop>,
    pub spawn: Vec3,
    pub spawn_yaw: f32,
    /// Grensene for skyggekartet.
    pub extent: f32,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }
    fn pick<T: Copy>(&mut self, v: &[T]) -> T {
        v[((self.next() * v.len() as f32) as usize).min(v.len() - 1)]
    }
}

fn to_srgb_u32(c: Vec3) -> u32 {
    let f = |x: f32| {
        let s = if x <= 0.0031308 {
            x * 12.92
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        (s.clamp(0.0, 1.0) * 255.0 + 0.5) as u32
    };
    (f(c.x) << 16) | (f(c.y) << 8) | f(c.z)
}

const WHITE_WOOD: Mat = Mat::new(0xf2efe8, 0.6, Surface::Wood);
const SLATE: Mat = Mat::new(0x3d4249, 0.75, Surface::Slate);
const STONE: Mat = Mat::new(0x9a9790, 0.85, Surface::Stone);
const DARK_STONE: Mat = Mat::new(0x6f6c66, 0.9, Surface::Stone);
const COBBLE: Mat = Mat::new(0x8c8984, 0.9, Surface::Cobble);
const GLASS: Mat = Mat::new(0x1d2630, 0.08, Surface::Glass).glow(Glow::Window);
const IRON: Mat = Mat::new(0x26282b, 0.45, Surface::Metal);
const LANTERN: Mat = Mat::new(0xfff1d6, 0.3, Surface::Glass).glow(Glow::Lamp);
const BARK: Mat = Mat::new(0x4a3b2f, 0.9, Surface::Wood);
const LEAVES: Mat = Mat::new(0x4d7344, 0.8, Surface::Leaves);
const GRASS: Mat = Mat::new(0x5d7f45, 0.95, Surface::Grass);
const OAK: Mat = Mat::new(0x8a6440, 0.7, Surface::Wood);

pub fn build(tokens: &Tokens) -> World {
    let unit = |step: &str| tokens.spacing_px(step) / 16.0;
    let floor_h = unit("12");
    let win_w = unit("4");
    let win_h = unit("6");

    // Fasadefargene hentes fra Fristil sine lyse tokens og blandes mot
    // kalkhvitt, slik malte nordiske trehus og pussede murhus ser ut.
    let lime = Vec3::new(0.93, 0.9, 0.84);
    let paints: Vec<Mat> = [
        ("neutral-surface", 0.2),
        ("brand3-fill", 0.82),
        ("warning-fill", 0.45),
        ("accent-border-subtle", 0.3),
        ("brand1-border-subtle", 0.25),
        ("neutral-raised", 0.1),
        ("brand3-fill", 0.62),
        ("warning-border-subtle", 0.55),
    ]
    .iter()
    .map(|(name, amount)| {
        let c = lime.lerp(tokens.color(Theme::Light, name), *amount);
        Mat::new(to_srgb_u32(c), 0.85, Surface::Plaster)
    })
    .collect();

    let mut w = World {
        mesh: Mesh::default(),
        colliders: vec![],
        lights: vec![],
        props: vec![],
        spawn: Vec3::new(7.0, 0.0, 9.0),
        spawn_yaw: 0.25,
        extent: 70.0,
    };
    let mut rng = Rng(0x5eed_f215_7a1d);

    // Bakken.
    w.mesh.quad(
        &Affine3A::IDENTITY,
        [
            Vec3::new(-160.0, 0.0, 160.0),
            Vec3::new(160.0, 0.0, 160.0),
            Vec3::new(160.0, 0.0, -160.0),
            Vec3::new(-160.0, 0.0, -160.0),
        ],
        &COBBLE,
        [1.0; 4],
    );
    w.colliders.push(Shape::Cuboid {
        center: Vec3::new(0.0, -0.5, 0.0),
        half: Vec3::new(160.0, 0.5, 160.0),
        yaw: 0.0,
    });

    // Husrekkene rundt torget. Hver rekke er en linje husfrontene står på,
    // med retningen de vender og åpningene der gatene går ut.
    struct Row {
        origin: Vec3,
        along: Vec3,
        facing: Vec3,
        from: f32,
        to: f32,
        gaps: &'static [(f32, f32)],
    }
    let rows = [
        // Nord, med porten i midten.
        Row { origin: Vec3::new(0.0, 0.0, -18.0), along: Vec3::X, facing: Vec3::Z, from: -29.0, to: 29.0, gaps: &[(-5.5, 5.5)] },
        // Sør, med gata hjem.
        Row { origin: Vec3::new(0.0, 0.0, 18.0), along: Vec3::X, facing: -Vec3::Z, from: -29.0, to: 29.0, gaps: &[(-4.0, 4.0)] },
        // Vest og øst.
        Row { origin: Vec3::new(-20.0, 0.0, 0.0), along: Vec3::Z, facing: Vec3::X, from: -18.0, to: 18.0, gaps: &[(-3.5, 3.5)] },
        Row { origin: Vec3::new(20.0, 0.0, 0.0), along: Vec3::Z, facing: -Vec3::X, from: -18.0, to: 18.0, gaps: &[(-3.5, 3.5)] },
        // Gata bak porten.
        Row { origin: Vec3::new(-5.0, 0.0, 0.0), along: Vec3::Z, facing: Vec3::X, from: -52.0, to: -27.0, gaps: &[] },
        Row { origin: Vec3::new(5.0, 0.0, 0.0), along: Vec3::Z, facing: -Vec3::X, from: -52.0, to: -27.0, gaps: &[] },
        // Gata sørover.
        Row { origin: Vec3::new(-4.0, 0.0, 0.0), along: Vec3::Z, facing: Vec3::X, from: 27.0, to: 48.0, gaps: &[] },
        Row { origin: Vec3::new(4.0, 0.0, 0.0), along: Vec3::Z, facing: -Vec3::X, from: 27.0, to: 48.0, gaps: &[] },
    ];
    for row in &rows {
        let mut s = row.from;
        while s < row.to - 3.0 {
            if let Some(&(_, end)) = row.gaps.iter().find(|(a, b)| s >= *a - 0.01 && s < *b) {
                s = end;
                continue;
            }
            let next_gap = row
                .gaps
                .iter()
                .filter(|(a, _)| *a > s)
                .map(|(a, _)| *a)
                .fold(row.to, f32::min);
            let mut width = rng.range(5.5, 9.5);
            if next_gap - (s + width) < 4.0 {
                width = next_gap - s;
            }
            let center_along = s + width * 0.5;
            let front = row.origin + row.along * center_along;
            let depth = 9.0;
            let floors = rng.pick(&[2, 3, 3, 4, 2, 3]);
            let paint = rng.pick(&paints);
            let door_slot = rng.pick(&[slot::BRAND1, slot::BRAND2, slot::BRAND3, slot::ACCENT, slot::WARNING]);
            house(
                &mut w,
                &mut rng,
                front,
                row.facing,
                width - 0.08,
                depth,
                floors,
                floor_h,
                (win_w, win_h),
                &paint,
                door_slot,
            );
            s += width;
        }
    }

    // Husene som lukker gatene.
    let ends = [
        (Vec3::new(0.0, 0.0, -53.0), Vec3::Z, 10.0),
        (Vec3::new(0.0, 0.0, 49.0), -Vec3::Z, 8.0),
        (Vec3::new(30.0, 0.0, 0.0), -Vec3::X, 7.2),
        (Vec3::new(-30.0, 0.0, 0.0), Vec3::X, 7.2),
    ];
    for (front, facing, width) in ends {
        let paint = rng.pick(&paints);
        house(&mut w, &mut rng, front, facing, width, 9.0, 3, floor_h, (win_w, win_h), &paint, slot::ACCENT);
    }

    gate(&mut w, tokens);
    plaza(&mut w, &mut rng);
    mountains(&mut w, &mut rng);

    // Lykter langs gatene.
    for z in [-34.0, -46.0] {
        lamp(&mut w, Vec3::new(-3.6, 0.0, z));
        lamp(&mut w, Vec3::new(3.6, 0.0, z - 6.0));
    }
    for z in [32.0, 42.0] {
        lamp(&mut w, Vec3::new(3.0, 0.0, z));
    }

    w
}

#[allow(clippy::too_many_arguments)]
fn house(
    w: &mut World,
    rng: &mut Rng,
    front: Vec3,
    facing: Vec3,
    width: f32,
    depth: f32,
    floors: u32,
    floor_h: f32,
    (win_w, win_h): (f32, f32),
    paint: &Mat,
    door_slot: u8,
) {
    let yaw = facing.x.atan2(facing.z);
    let rot = Quat::from_rotation_y(yaw);
    let center = front - facing * (depth * 0.5);
    let base = Affine3A::from_rotation_translation(rot, center);
    let at = |p: Vec3| base * Affine3A::from_translation(p);

    let plinth = 0.45;
    let wall_h = floors as f32 * floor_h + 0.4;
    let fz = depth * 0.5;

    // Sokkel og vegger.
    w.mesh.cuboid(&at(Vec3::new(0.0, plinth * 0.5, 0.0)), Vec3::new(width + 0.1, plinth, depth + 0.1), &DARK_STONE);
    w.mesh.cuboid_ao(&at(Vec3::new(0.0, plinth + wall_h * 0.5, 0.0)), Vec3::new(width, wall_h, depth), paint, 0.8);
    // Gesims under taket.
    w.mesh.cuboid(&at(Vec3::new(0.0, plinth + wall_h - 0.1, 0.0)), Vec3::new(width + 0.2, 0.2, depth + 0.2), &WHITE_WOOD);

    // Tak, med mønet langs fasaden.
    let roof_h = rng.range(2.6, 4.2);
    w.mesh.gable_roof(
        &at(Vec3::new(0.0, plinth + wall_h, 0.0)),
        width,
        depth,
        roof_h,
        0.45,
        &SLATE,
        paint,
    );
    // Pipe på noen av husene.
    if rng.next() < 0.6 {
        let x = rng.range(-width * 0.3, width * 0.3);
        w.mesh.cuboid(
            &at(Vec3::new(x, plinth + wall_h + roof_h * 0.6, -depth * 0.15)),
            Vec3::new(0.7, roof_h * 1.1, 0.7),
            &DARK_STONE,
        );
    }

    w.colliders.push(Shape::Cuboid {
        center: center + Vec3::Y * (plinth + wall_h) * 0.5,
        half: Vec3::new(width * 0.5 + 0.05, (plinth + wall_h) * 0.5, depth * 0.5 + 0.05),
        yaw,
    });

    // Vinduer og dør på fasaden.
    let bays = ((width - 1.0) / 2.2).floor().max(1.0) as i32;
    let bay_w = (width - 1.0) / bays as f32;
    let door_bay = rng.range(0.0, bays as f32) as i32;
    let door_mat = Mat::new(0x777777, 0.6, Surface::Wood).slot(door_slot);
    for f in 0..floors {
        for b in 0..bays {
            let x = -width * 0.5 + 0.5 + bay_w * (b as f32 + 0.5);
            let floor_y = plinth + f as f32 * floor_h;
            if f == 0 && b == door_bay {
                let dh = 2.4;
                w.mesh.cuboid(&at(Vec3::new(x, plinth + dh * 0.5, fz + 0.03)), Vec3::new(1.3, dh + 0.12, 0.08), &WHITE_WOOD);
                w.mesh.cuboid(&at(Vec3::new(x, plinth + dh * 0.5 - 0.02, fz + 0.06)), Vec3::new(1.1, dh - 0.04, 0.06), &door_mat);
                // Dørspeil.
                for py in [0.7, 1.7] {
                    w.mesh.cuboid(&at(Vec3::new(x, plinth + py, fz + 0.1)), Vec3::new(0.8, 0.7, 0.03), &door_mat);
                }
                // Trapp.
                w.mesh.cuboid(&at(Vec3::new(x, plinth * 0.5, fz + 0.45)), Vec3::new(1.8, plinth, 0.8), &STONE);
                w.colliders.push(Shape::Cuboid {
                    center: center + rot * Vec3::new(x, plinth * 0.5, fz + 0.45),
                    half: Vec3::new(0.9, plinth * 0.5, 0.4),
                    yaw,
                });
                continue;
            }
            let sill = floor_y + 0.95;
            let cy = sill + win_h * 0.5;
            // Karm, glass, sprosser og vindusbrett.
            w.mesh.cuboid(&at(Vec3::new(x, cy, fz + 0.04)), Vec3::new(win_w + 0.16, win_h + 0.16, 0.08), &WHITE_WOOD);
            w.mesh.quad(
                &at(Vec3::new(x, cy, fz + 0.095)),
                [
                    Vec3::new(-win_w * 0.5, -win_h * 0.5, 0.0),
                    Vec3::new(win_w * 0.5, -win_h * 0.5, 0.0),
                    Vec3::new(win_w * 0.5, win_h * 0.5, 0.0),
                    Vec3::new(-win_w * 0.5, win_h * 0.5, 0.0),
                ],
                &GLASS,
                [1.0; 4],
            );
            w.mesh.cuboid(&at(Vec3::new(x, cy, fz + 0.1)), Vec3::new(0.06, win_h, 0.04), &WHITE_WOOD);
            w.mesh.cuboid(&at(Vec3::new(x, cy + win_h * 0.12, fz + 0.1)), Vec3::new(win_w, 0.06, 0.04), &WHITE_WOOD);
            w.mesh.cuboid(&at(Vec3::new(x, sill - 0.08, fz + 0.1)), Vec3::new(win_w + 0.3, 0.06, 0.2), &WHITE_WOOD);
        }
    }
}

/// Porten i nord, med den store Fristil-knappen på østre pilar.
fn gate(w: &mut World, tokens: &Tokens) {
    let z = -18.0;
    let pillar = Vec3::new(2.0, 6.2, 2.0);
    for x in [-4.5f32, 4.5] {
        let t = Affine3A::from_translation(Vec3::new(x, pillar.y * 0.5, z - 1.0));
        w.mesh.cuboid(&t, pillar, &STONE);
        w.mesh.cuboid(&Affine3A::from_translation(Vec3::new(x, pillar.y + 0.15, z - 1.0)), Vec3::new(2.3, 0.3, 2.3), &DARK_STONE);
        w.mesh.cuboid(&Affine3A::from_translation(Vec3::new(x, 0.2, z - 1.0)), Vec3::new(2.3, 0.4, 2.3), &DARK_STONE);
        w.colliders.push(Shape::Cuboid {
            center: Vec3::new(x, pillar.y * 0.5, z - 1.0),
            half: pillar * 0.5 + Vec3::splat(0.1),
            yaw: 0.0,
        });
    }
    // Overliggeren.
    w.mesh.cuboid(&Affine3A::from_translation(Vec3::new(0.0, 5.6, z - 1.0)), Vec3::new(7.0, 1.0, 1.6), &STONE);

    // Portdørene, som dreier om hengslene ved pilarene.
    let door_w = 3.5;
    let door_h = 5.1;
    for (name, hinge_x, dir) in [("door_west", -3.5f32, 1.0f32), ("door_east", 3.5, -1.0)] {
        let mut m = Mesh::default();
        let cx = dir * door_w * 0.5;
        m.cuboid(&Affine3A::from_translation(Vec3::new(cx, door_h * 0.5, 0.0)), Vec3::new(door_w, door_h, 0.18), &OAK);
        // Planker og beslag.
        for i in 0..5 {
            let px = dir * (0.35 + i as f32 * 0.7);
            m.cuboid(&Affine3A::from_translation(Vec3::new(px, door_h * 0.5, 0.1)), Vec3::new(0.62, door_h - 0.2, 0.04), &OAK);
        }
        for y in [0.9, door_h - 0.9] {
            m.cuboid(&Affine3A::from_translation(Vec3::new(cx, y, 0.13)), Vec3::new(door_w - 0.2, 0.14, 0.04), &IRON);
        }
        w.props.push(Prop {
            name,
            mesh: m,
            rest: Affine3A::from_translation(Vec3::new(hinge_x, 0.0, z - 1.0)),
            collider: Some((Vec3::new(cx, door_h * 0.5, 0.0), Vec3::new(door_w * 0.5, door_h * 0.5, 0.12))),
        });
    }

    // Knappen. Formen er `.fs-button`: avrundet rektangel, med radius fra
    // `spacing-1` og en ledetekst. Fargene kommer fra paletten, så R kan bytte
    // variant og T kan bytte tema.
    let radius = tokens.spacing_px("1") / 16.0 * 0.6;
    let (bw, bh) = (1.7, 0.7);
    let mut button = Mesh::default();
    let rim = Mat::new(0x777777, 0.4, Surface::Plain).slot(slot::BUTTON_RIM).glow(Glow::Always);
    let body = Mat::new(0x777777, 0.35, Surface::Plain).slot(slot::BUTTON_BODY).glow(Glow::Always);
    let label = Mat::new(0xffffff, 0.4, Surface::Plain).slot(slot::BUTTON_LABEL).glow(Glow::Always);
    rounded_slab(&mut button, Vec3::new(0.0, 0.0, 0.0), bw + 0.08, bh + 0.08, 0.12, radius + 0.04, &rim);
    rounded_slab(&mut button, Vec3::new(0.0, 0.0, 0.08), bw, bh, 0.12, radius, &body);
    block_text(&mut button, "ÅPNE", Vec3::new(0.0, 0.0, 0.155), 0.052, &label);
    // Huset knappen sitter i.
    let housing = Affine3A::from_translation(Vec3::new(4.5, 1.35, z + 0.02));
    w.mesh.cuboid(&housing, Vec3::new(2.0, 1.1, 0.06), &DARK_STONE);
    w.props.push(Prop {
        name: "button",
        mesh: button,
        rest: Affine3A::from_translation(Vec3::new(4.5, 1.35, z + 0.05)),
        collider: Some((Vec3::new(0.0, 0.0, 0.1), Vec3::new(bw * 0.5, bh * 0.5, 0.15))),
    });
}

/// Avrundet rektangel som en tykk plate, forsiden mot +Z.
fn rounded_slab(m: &mut Mesh, c: Vec3, w: f32, h: f32, d: f32, r: f32, mat: &Mat) {
    let t = |p: Vec3| Affine3A::from_translation(c + p);
    m.cuboid_ao(&t(Vec3::ZERO), Vec3::new(w - 2.0 * r, h, d), mat, 1.0);
    m.cuboid_ao(&t(Vec3::ZERO), Vec3::new(w, h - 2.0 * r, d), mat, 1.0);
    let lay = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let p = c + Vec3::new(sx * (w * 0.5 - r), sy * (h * 0.5 - r), -d * 0.5);
        m.cylinder(&Affine3A::from_rotation_translation(lay, p), r, r, d, 10, mat, true);
    }
}

/// Blokkbokstaver bygget av små bokser, sentrert om `c`.
fn block_text(m: &mut Mesh, text: &str, c: Vec3, px: f32, mat: &Mat) {
    let glyph = |ch: char| -> [&'static str; 7] {
        match ch {
            'Å' => ["  #  ", "     ", " ### ", "#   #", "#####", "#   #", "#   #"],
            'P' => ["     ", "     ", "#### ", "#   #", "#### ", "#    ", "#    "],
            'N' => ["     ", "     ", "#   #", "##  #", "# # #", "#  ##", "#   #"],
            'E' => ["     ", "     ", "#####", "#    ", "#### ", "#    ", "#####"],
            _ => ["     "; 7],
        }
    };
    let chars: Vec<char> = text.chars().collect();
    let adv = 6.0;
    let total = chars.len() as f32 * adv - 1.0;
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph(*ch);
        for (row, line) in g.iter().enumerate() {
            for (col, cell) in line.chars().enumerate() {
                if cell != '#' {
                    continue;
                }
                let x = (i as f32 * adv + col as f32 - total * 0.5 + 0.5) * px;
                let y = (4.0 - row as f32) * px;
                m.cuboid_ao(&Affine3A::from_translation(c + Vec3::new(x, y, 0.0)), Vec3::new(px * 1.02, px * 1.02, 0.03), mat, 1.0);
            }
        }
    }
}

fn plaza(w: &mut World, rng: &mut Rng) {
    // Plantebedet i midten med det store treet.
    w.mesh.cylinder(&Affine3A::IDENTITY, 4.6, 4.5, 0.55, 48, &STONE, true);
    w.mesh.cylinder(&Affine3A::from_translation(Vec3::Y * 0.55), 4.2, 4.2, 0.02, 48, &GRASS, true);
    w.colliders.push(Shape::Cylinder { center: Vec3::Y * 0.275, radius: 4.6, half_height: 0.275 });
    tree(w, rng, Vec3::new(0.0, 0.55, 0.0), 1.6);

    // Mindre trær i hjørnene.
    for (x, z) in [(-14.0, -12.0), (14.0, -12.0), (-14.0, 12.5), (14.0, 12.5)] {
        w.mesh.cylinder(&Affine3A::from_translation(Vec3::new(x, 0.0, z)), 1.3, 1.25, 0.25, 24, &DARK_STONE, true);
        tree(w, rng, Vec3::new(x, 0.25, z), 1.0);
    }

    // Benker rundt bedet.
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
        let p = Vec3::new(a.sin() * 6.6, 0.0, a.cos() * 6.6);
        let rot = Quat::from_rotation_y(a);
        let t = |q: Vec3| Affine3A::from_rotation_translation(rot, p + rot * q);
        for k in 0..4 {
            w.mesh.cuboid(&t(Vec3::new(0.0, 0.45, -0.24 + k as f32 * 0.16)), Vec3::new(2.0, 0.05, 0.13), &OAK);
        }
        for sx in [-0.85, 0.85] {
            w.mesh.cuboid(&t(Vec3::new(sx, 0.22, 0.0)), Vec3::new(0.08, 0.44, 0.6), &IRON);
        }
        w.colliders.push(Shape::Cuboid { center: p + Vec3::Y * 0.24, half: Vec3::new(1.0, 0.24, 0.32), yaw: a });
    }

    // Lykter rundt torget.
    for (x, z) in [
        (-9.0, -15.5), (9.0, -15.5), (-17.5, -6.0), (17.5, -6.0),
        (-17.5, 6.0), (17.5, 6.0), (-9.0, 15.5), (9.0, 15.5),
    ] {
        lamp(w, Vec3::new(x, 0.0, z));
    }

    // Kantstein langs fasadene.
    for (c, s) in [
        (Vec3::new(0.0, 0.06, -17.6), Vec3::new(58.0, 0.12, 0.8)),
        (Vec3::new(0.0, 0.06, 17.6), Vec3::new(58.0, 0.12, 0.8)),
        (Vec3::new(-19.6, 0.06, 0.0), Vec3::new(0.8, 0.12, 35.2)),
        (Vec3::new(19.6, 0.06, 0.0), Vec3::new(0.8, 0.12, 35.2)),
    ] {
        w.mesh.cuboid(&Affine3A::from_translation(c), s, &STONE);
    }
}

fn tree(w: &mut World, rng: &mut Rng, base: Vec3, scale: f32) {
    let h = 3.2 * scale;
    w.mesh.cylinder(&Affine3A::from_translation(base), 0.22 * scale, 0.14 * scale, h, 10, &BARK, false);
    w.colliders.push(Shape::Cylinder { center: base + Vec3::Y * h * 0.5, radius: 0.3 * scale, half_height: h * 0.5 });
    let blobs = 7 + (rng.next() * 4.0) as i32;
    for k in 0..blobs {
        let a = k as f32 * 2.4 + rng.range(0.0, 0.6);
        let spread = rng.range(0.6, 1.5);
        let o = Vec3::new(a.cos() * spread, rng.range(-0.4, 1.1), a.sin() * spread) * scale;
        let r = rng.range(0.85, 1.35) * scale;
        w.mesh.sphere(
            &Affine3A::from_translation(base + Vec3::Y * (h + 0.6 * scale) + o),
            Vec3::new(r, r * 0.85, r),
            10,
            16,
            &LEAVES,
        );
    }
}

fn lamp(w: &mut World, p: Vec3) {
    let t = |q: Vec3| Affine3A::from_translation(p + q);
    w.mesh.cylinder(&t(Vec3::ZERO), 0.16, 0.12, 0.5, 10, &IRON, true);
    w.mesh.cylinder(&t(Vec3::Y * 0.5), 0.06, 0.05, 3.3, 8, &IRON, false);
    w.mesh.cuboid(&t(Vec3::Y * 3.85), Vec3::new(0.42, 0.06, 0.42), &IRON);
    w.mesh.cuboid(&t(Vec3::Y * 4.35), Vec3::new(0.5, 0.08, 0.5), &IRON);
    w.mesh.cuboid(&t(Vec3::Y * 4.1), Vec3::new(0.34, 0.44, 0.34), &LANTERN);
    w.mesh.cylinder(&t(Vec3::Y * 4.39), 0.2, 0.0, 0.25, 8, &IRON, true);
    w.colliders.push(Shape::Cylinder { center: p + Vec3::Y * 2.0, radius: 0.16, half_height: 2.0 });
    w.lights.push(PointLight {
        pos: p + Vec3::Y * 4.1,
        color: Vec3::new(1.0, 0.72, 0.45),
        intensity: 9.0,
        radius: 14.0,
    });
}

/// Fjell i det fjerne, så byen ligger i et landskap og ikke i et tomrom.
fn mountains(w: &mut World, rng: &mut Rng) {
    let rock = Mat::new(0x5d6670, 0.95, Surface::Stone);
    let snow = Mat::new(0xf4f6f8, 0.6, Surface::Plain);
    let n = 34;
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU + rng.range(-0.06, 0.06);
        let dist = rng.range(330.0, 480.0);
        let h = rng.range(70.0, 190.0);
        let r = rng.range(110.0, 190.0);
        let p = Vec3::new(a.sin() * dist, -4.0, a.cos() * dist);
        w.mesh.cylinder(&Affine3A::from_translation(p), r, 0.0, h, 7, &rock, false);
        if h > 120.0 {
            let k = 0.22;
            w.mesh.cylinder(&Affine3A::from_translation(p + Vec3::Y * (h * (1.0 - k) + 0.3)), r * k * 1.03, 0.0, h * k, 7, &snow, false);
        }
    }
}
