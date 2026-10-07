//! Fasadegrammatikken: husene rundt torget og langs gatene.
//!
//! Et hus er sokkel, vegger med hull, vinduer satt inn i hullene med
//! smyg, karm, sprosser og vindusbrett, dør med trapp og rekkverk, gesims,
//! tak med takrenner og nedløp, pipe, og på noen av dem ark, butikkvindu
//! med skilt, hjørnekasser og dørlampe. Målene kommer fra spacing-skalaen.

use super::{Builder, Shape, CONCRETE, DARK_TRIM, IRON, OAK, STONE, WHITE_TRIM};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec2, Vec3};

#[derive(Clone, Copy, PartialEq)]
pub enum Wall {
    Plaster(Vec3),
    /// Liggende panel.
    Cladding(Vec3),
    /// Stående panel.
    Standing(Vec3),
    Brick,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Roof {
    Slate,
    Tiles,
}

pub struct Spec {
    pub front: Vec3,
    pub facing: Vec3,
    pub width: f32,
    pub depth: f32,
    pub floors: u32,
    pub wall: Wall,
    pub roof: Roof,
    pub door_slot: u8,
    pub shop: Option<&'static str>,
    pub dormer: bool,
    pub door_lamp: bool,
    /// Gavlen vender mot gata (mønet på tvers av fasaden).
    pub gable_front: bool,
    /// Første etasje i stein.
    pub stone_base: bool,
    pub shutters: bool,
    pub flower_boxes: bool,
    pub balcony: bool,
    /// Etasjehøyden, litt forskjellig fra hus til hus.
    pub floor_h: f32,
}

impl Spec {
    /// Et hus med tilfeldige valg, gitt plassering og mål.
    pub fn random(b: &mut Builder, front: Vec3, facing: Vec3, width: f32, paint: Vec3, shop: Option<&'static str>) -> Self {
        let wall = match b.rng.next() {
            x if x < 0.38 => Wall::Plaster(paint),
            x if x < 0.66 => Wall::Cladding(paint),
            x if x < 0.84 => Wall::Standing(paint),
            _ => Wall::Brick,
        };
        let gable_front = shop.is_none() && width < 8.0 && b.rng.chance(0.3);
        Self {
            front,
            facing,
            width,
            depth: b.rng.range(7.5, 10.0),
            floors: b.rng.pick(&[2, 3, 3, 4, 2, 3]),
            wall,
            roof: if b.rng.chance(0.6) { Roof::Slate } else { Roof::Tiles },
            door_slot: b.rng.pick(&[slot::BRAND1, slot::BRAND2, slot::BRAND3, slot::ACCENT, slot::WARNING]),
            shop,
            dormer: !gable_front && b.rng.chance(0.45),
            door_lamp: b.rng.chance(0.5),
            gable_front,
            stone_base: !matches!(wall, Wall::Brick) && shop.is_none() && b.rng.chance(0.25),
            shutters: b.rng.chance(0.35),
            flower_boxes: b.rng.chance(0.45),
            balcony: shop.is_none() && width > 7.0 && b.rng.chance(0.3),
            floor_h: b.unit("12") * b.rng.range(0.92, 1.08),
        }
    }
}

const SHOPS: &[&str] = &["Bakeriet", "Smia", "Apoteket", "Bokhandelen", "Kafé Fristil", "Skredderen", "Urmakeren", "Glassblåseriet"];

/// Husrekkene rundt torget og i gatene.
pub fn district(b: &mut Builder) {
    struct Row {
        origin: Vec3,
        along: Vec3,
        facing: Vec3,
        from: f32,
        to: f32,
        gaps: &'static [(f32, f32)],
        square: bool,
    }
    let rows = [
        // Nord, med porten i midten.
        Row { origin: Vec3::new(0.0, 0.0, -18.0), along: Vec3::X, facing: Vec3::Z, from: -29.0, to: 29.0, gaps: &[(-5.5, 5.5)], square: true },
        // Sør, med gata hjem.
        Row { origin: Vec3::new(0.0, 0.0, 18.0), along: Vec3::X, facing: -Vec3::Z, from: -29.0, to: 29.0, gaps: &[(-4.0, 4.0)], square: true },
        // Vest og øst.
        Row { origin: Vec3::new(-20.0, 0.0, 0.0), along: Vec3::Z, facing: Vec3::X, from: -18.0, to: 18.0, gaps: &[(-3.5, 3.5)], square: true },
        Row { origin: Vec3::new(20.0, 0.0, 0.0), along: Vec3::Z, facing: -Vec3::X, from: -18.0, to: 18.0, gaps: &[(-3.5, 3.5)], square: true },
        // Gata bak porten.
        Row { origin: Vec3::new(-5.0, 0.0, 0.0), along: Vec3::Z, facing: Vec3::X, from: -52.0, to: -27.0, gaps: &[], square: false },
        Row { origin: Vec3::new(5.0, 0.0, 0.0), along: Vec3::Z, facing: -Vec3::X, from: -52.0, to: -27.0, gaps: &[], square: false },
        // Gata sørover.
        Row { origin: Vec3::new(-4.0, 0.0, 0.0), along: Vec3::Z, facing: Vec3::X, from: 27.0, to: 48.0, gaps: &[], square: false },
        Row { origin: Vec3::new(4.0, 0.0, 0.0), along: Vec3::Z, facing: -Vec3::X, from: 27.0, to: 48.0, gaps: &[], square: false },
    ];
    let paints = b.paints();
    let mut shop_i = 0;
    for (ri, row) in rows.iter().enumerate() {
        let phase = ri as f32 * 1.7;
        let mut s = row.from;
        while s < row.to - 3.0 {
            if let Some(&(_, end)) = row.gaps.iter().find(|(a, bb)| s >= *a - 0.01 && s < *bb) {
                s = end;
                continue;
            }
            let next_gap = row.gaps.iter().filter(|(a, _)| *a > s).map(|(a, _)| *a).fold(row.to, f32::min);
            // Et smug mellom noen av husene, med et gjerde og et tre bak.
            if b.rng.chance(0.14) && next_gap - s > 9.0 {
                let alley = b.rng.range(1.6, 2.4);
                alley_fence(b, row.origin + row.along * (s + alley * 0.5), row.along, row.facing, alley);
                s += alley;
                continue;
            }
            let mut width = b.rng.range(5.0, 9.5);
            if next_gap - (s + width) < 4.0 {
                width = next_gap - s;
            }
            let center_along = s + width * 0.5;
            // Rekka buer svakt, og hvert hus står litt inn eller ut og litt skjevt.
            let wobble = (center_along * 0.19 + phase).sin() * 0.7 + b.rng.range(-0.45, 0.45);
            let front = row.origin + row.along * center_along - row.facing * wobble;
            let jitter = b.rng.range(-0.035, 0.035);
            let facing = Quat::from_rotation_y(jitter) * row.facing;
            let paint = b.rng.pick(&paints);
            let shop = if row.square && width > 6.5 && b.rng.chance(0.4) && shop_i < SHOPS.len() {
                shop_i += 1;
                Some(SHOPS[shop_i - 1])
            } else {
                None
            };
            let spec = Spec::random(b, front, facing, width - 0.1, paint, shop);
            house(b, &spec);
            s += width;
        }
    }

    // Gatene lukkes ikke av hus: veiene fortsetter buktende ut i landskapet
    // (`path_dist`), med veivisere til stedene utenfor (home_signs.rs).
}

/// Et smug mellom to hus: plankegjerde med port, og et tre bak.
fn alley_fence(b: &mut Builder, center: Vec3, along: Vec3, facing: Vec3, width: f32) {
    let gy = b.ground(center.x, center.z);
    let yaw = facing.x.atan2(facing.z);
    let t = Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), Vec3::new(center.x, gy, center.z));
    let m = &mut b.mesh;
    let fence = Mat::tex(Layer::Oak, 0xffffff).flags(flag::ROT_UV);
    let n = (width / 0.16) as i32;
    for k in 0..n {
        let x = -width * 0.5 + 0.08 + k as f32 * 0.16;
        let h = 1.6 + ((k as f32 * 1.3).sin() * 0.04);
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(x, h * 0.5, 0.0))), Vec3::new(0.12, h, 0.03), &fence);
    }
    for y in [0.4, 1.3] {
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, y, -0.04))), Vec3::new(width, 0.1, 0.04), &fence);
    }
    for sx in [-1.0, 1.0] {
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(sx * (width * 0.5 - 0.06), 0.9, -0.05))), Vec3::new(0.12, 1.8, 0.12), &fence);
    }
    b.solid_box(Vec3::new(center.x, gy + 0.9, center.z), Vec3::new(width, 1.8, 0.2), yaw);
    // Tøy på snor bak gjerdet, mellom husveggene.
    log::debug!("Smug ved ({:.1}, {:.1}), bredde {width:.1}", center.x, center.z);
    if b.rng.chance(0.6) {
        let a = Vec3::new(center.x, gy + 2.4, center.z) - facing * 2.2 - along * (width * 0.5 + 0.05);
        let c = Vec3::new(center.x, gy + 2.3, center.z) - facing * 2.2 + along * (width * 0.5 + 0.05);
        super::street_life::laundry(&mut b.mesh, a, c, &mut b.rng);
    }
    // Treet bak gjerdet.
    let back = center - facing * 4.0;
    let tree_scale = b.rng.range(0.8, 1.2);
    let tree_pos = b.on_ground(back);
    super::nature::tree(b, tree_pos, tree_scale, 3);
}

fn wall_mat(wall: Wall) -> Mat {
    match wall {
        Wall::Plaster(c) => Mat::tex(Layer::Plaster, 0xffffff).tint(c),
        Wall::Cladding(c) => Mat::tex(Layer::Cladding, 0xffffff).tint(c),
        Wall::Standing(c) => Mat::tex(Layer::Cladding, 0xffffff).tint(c).flags(flag::ROT_UV),
        Wall::Brick => Mat::tex(Layer::Brick, 0xffffff),
    }
}

/// Et hull i en vegg: x langs veggen fra midten, y fra veggens fot.
#[derive(Clone, Copy)]
struct Hole {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// Veggflate i planet z = 0 (forsiden mot +Z) med hull, bygget av striper
/// mellom hullene. `t` plasserer den.
/// Som `wall_with_holes`, med et annet materiale under `base` (første
/// etasje i stein).
fn wall_banded(m: &mut Mesh, t: &Affine3A, width: f32, height: f32, holes: &[Hole], mat: &Mat, base: Option<(f32, Mat)>) {
    let mut edges: Vec<f32> = vec![0.0, height];
    if let Some((bh, _)) = base {
        edges.push(bh);
    }
    for h in holes {
        edges.push(h.y);
        edges.push(h.y + h.h);
    }
    edges.sort_by(|a, b| a.partial_cmp(b).unwrap());
    edges.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    for pair in edges.windows(2) {
        let (y0, y1) = (pair[0], pair[1]);
        if y1 - y0 < 1e-4 {
            continue;
        }
        let mid = (y0 + y1) * 0.5;
        let mat = match base {
            Some((bh, ref bm)) if mid < bh => bm,
            _ => mat,
        };
        let mut cuts: Vec<(f32, f32)> = holes.iter().filter(|h| mid > h.y && mid < h.y + h.h).map(|h| (h.x - h.w * 0.5, h.x + h.w * 0.5)).collect();
        cuts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut x = -width * 0.5;
        for (a, bb) in cuts.iter().chain(std::iter::once(&(width * 0.5, width * 0.5))) {
            if *a - x > 1e-4 {
                let ao_lo = if y0 < 0.01 { 0.7 } else { 1.0 };
                m.quad(t, [Vec3::new(x, y0, 0.0), Vec3::new(*a, y0, 0.0), Vec3::new(*a, y1, 0.0), Vec3::new(x, y1, 0.0)], mat, [ao_lo, ao_lo, 1.0, 1.0]);
            }
            x = *bb;
        }
    }
}

/// Smygene inn i et hull: topp, bunn og sider, `depth` innover (−Z).
fn reveal(m: &mut Mesh, t: &Affine3A, h: Hole, depth: f32, mat: &Mat) {
    let (x0, x1, y0, y1) = (h.x - h.w * 0.5, h.x + h.w * 0.5, h.y, h.y + h.h);
    let d = -depth;
    // Venstre smyg vender mot +X, høyre mot −X, topp ned, bunn opp.
    m.quad(t, [Vec3::new(x0, y0, d), Vec3::new(x0, y0, 0.0), Vec3::new(x0, y1, 0.0), Vec3::new(x0, y1, d)], mat, [0.85, 1.0, 1.0, 0.85]);
    m.quad(t, [Vec3::new(x1, y0, 0.0), Vec3::new(x1, y0, d), Vec3::new(x1, y1, d), Vec3::new(x1, y1, 0.0)], mat, [1.0, 0.85, 0.85, 1.0]);
    m.quad(t, [Vec3::new(x0, y1, 0.0), Vec3::new(x1, y1, 0.0), Vec3::new(x1, y1, d), Vec3::new(x0, y1, d)], mat, [1.0, 1.0, 0.7, 0.7]);
    m.quad(t, [Vec3::new(x0, y0, d), Vec3::new(x1, y0, d), Vec3::new(x1, y0, 0.0), Vec3::new(x0, y0, 0.0)], mat, [0.9, 0.9, 1.0, 1.0]);
}

/// Vindu i et hull: karm, glass, sprosser, vindusbrett, og omramming på trehus.
/// Modeller som skal settes ut etter at fasaden er bygd (vinduer,
/// dørlampe, blomsterkasser), siden meshen er lånt mens huset bygges.
type Models = Vec<(&'static str, Affine3A)>;

fn window(m: &mut Mesh, t: &Affine3A, h: Hole, depth: f32, wall: Wall, panes: (u32, u32), models: Option<&mut Models>) {
    let glass = Mat::special(kind::GLASS, 0x1d2630, 0.05);
    let z = -depth;
    let fw = 0.07;
    let (x0, x1, y0, y1) = (h.x - h.w * 0.5, h.x + h.w * 0.5, h.y, h.y + h.h);
    let frame = |m: &mut Mesh, c: Vec3, s: Vec3| m.cuboid_ao(&(*t * Affine3A::from_translation(c)), s, &WHITE_TRIM, 1.0);
    // Standardvinduet som modell fra Blender, om den finnes: karm, ramme,
    // sprosser og glass i ett. Brettet og omrammingen bygges under uansett.
    let modelled = match models {
        Some(models) if (h.w - 1.0).abs() < 0.01 && (h.h - 1.5).abs() < 0.01 => {
            models.push((if panes.1 >= 3 { "window_23" } else { "window_22" }, *t * Affine3A::from_translation(Vec3::new(h.x, h.y, z))));
            true
        }
        _ => false,
    };
    if !modelled {
    // Karmen: fire lister.
    frame(m, Vec3::new(x0 + fw * 0.5, (y0 + y1) * 0.5, z + 0.04), Vec3::new(fw, h.h, 0.08));
    frame(m, Vec3::new(x1 - fw * 0.5, (y0 + y1) * 0.5, z + 0.04), Vec3::new(fw, h.h, 0.08));
    frame(m, Vec3::new(h.x, y1 - fw * 0.5, z + 0.04), Vec3::new(h.w, fw, 0.08));
    frame(m, Vec3::new(h.x, y0 + fw * 0.5, z + 0.04), Vec3::new(h.w, fw, 0.08));
    // Glasset.
    m.quad(
        t,
        [Vec3::new(x0 + fw, y0 + fw, z), Vec3::new(x1 - fw, y0 + fw, z), Vec3::new(x1 - fw, y1 - fw, z), Vec3::new(x0 + fw, y1 - fw, z)],
        &glass,
        [1.0; 4],
    );
    // Sprosser.
    let bar = 0.035;
    for i in 1..panes.0 {
        let x = x0 + fw + (h.w - 2.0 * fw) * i as f32 / panes.0 as f32;
        frame(m, Vec3::new(x, (y0 + y1) * 0.5, z + 0.025), Vec3::new(bar, h.h - 2.0 * fw, 0.05));
    }
    for j in 1..panes.1 {
        let y = y0 + fw + (h.h - 2.0 * fw) * j as f32 / panes.1 as f32;
        frame(m, Vec3::new(h.x, y, z + 0.025), Vec3::new(h.w - 2.0 * fw, bar, 0.05));
    }
    }
    // Vindusbrettet: en profil som stikker ut.
    let sill = [Vec2::new(0.0, 0.0), Vec2::new(0.12, 0.0), Vec2::new(0.12, -0.05), Vec2::new(0.0, -0.08)];
    let from = t.transform_point3(Vec3::new(x0 - 0.06, y0, 0.0));
    let to = t.transform_point3(Vec3::new(x1 + 0.06, y0, 0.0));
    let out = t.transform_vector3(Vec3::Z);
    let sill_mat = if matches!(wall, Wall::Brick | Wall::Plaster(_)) { STONE } else { WHITE_TRIM };
    m.extrude(&sill, from, to, out, &sill_mat, true);
    // Omramming på trehus: lister rundt hullet på utsiden.
    if matches!(wall, Wall::Cladding(_) | Wall::Standing(_)) {
        let b = 0.11;
        frame(m, Vec3::new(x0 - b * 0.5, (y0 + y1) * 0.5 + 0.03, 0.015), Vec3::new(b, h.h + b + 0.06, 0.03));
        frame(m, Vec3::new(x1 + b * 0.5, (y0 + y1) * 0.5 + 0.03, 0.015), Vec3::new(b, h.h + b + 0.06, 0.03));
        frame(m, Vec3::new(h.x, y1 + b * 0.5 + 0.03, 0.02), Vec3::new(h.w + 2.0 * b + 0.1, b + 0.06, 0.04));
    }
}

pub fn house(b: &mut Builder, s: &Spec) {
    let floor_h = s.floor_h;
    let win_w = b.unit("4");
    let win_h = b.unit("6");
    let yaw = s.facing.x.atan2(s.facing.z);
    let rot = Quat::from_rotation_y(yaw);
    // Huset står på bakkehøyden ved døra, og sokkelen går ned i terrenget.
    let gy = b.ground(s.front.x, s.front.z);
    let center = Vec3::new(s.front.x, gy, s.front.z) - s.facing * (s.depth * 0.5);
    let base = Affine3A::from_rotation_translation(rot, center);
    let local = |p: Vec3| base * Affine3A::from_translation(p);
    let mut solids: Vec<(Vec3, Vec3, f32)> = vec![];
    let mut lights: Vec<(Vec3, Vec3, f32, f32)> = vec![];
    let mut models: Models = vec![];
    let has = (b.has_model("window_22"), b.has_model("flower_box"), b.has_model("door_lamp"));
    let m = &mut b.mesh;

    let plinth = 0.45;
    let wall_h = s.floors as f32 * floor_h + 0.4;
    let fz = s.depth * 0.5;
    let wall = wall_mat(s.wall);
    let (width, depth) = (s.width, s.depth);

    // Sokkel i stein, litt utenfor veggen, og dypt nok til å møte terrenget.
    let sink = 2.0;
    m.cuboid(&local(Vec3::new(0.0, (plinth - sink) * 0.5, 0.0)), Vec3::new(width + 0.12, plinth + sink, depth + 0.12), &STONE);

    // Vinduer og dør: hull i fasaden.
    let bays = ((width - 1.0) / 2.3).floor().max(1.0) as i32;
    let bay_w = (width - 1.0) / bays as f32;
    let door_bay = if s.shop.is_some() { 0 } else { b.rng.range(0.0, bays as f32) as i32 };
    let mut holes = vec![];
    let mut door: Option<Hole> = None;
    let mut shop_hole: Option<Hole> = None;
    let mut balcony: Option<Hole> = None;
    let balcony_bay = if s.balcony && s.floors >= 2 { Some((door_bay + 1) % bays) } else { None };
    let dh = 2.3;
    for f in 0..s.floors {
        for bay in 0..bays {
            let x = -width * 0.5 + 0.5 + bay_w * (bay as f32 + 0.5);
            let floor_y = f as f32 * floor_h;
            if f == 1 && Some(bay) == balcony_bay {
                let h = Hole { x, y: floor_y + 0.02, w: 1.0, h: 2.15 };
                balcony = Some(h);
                holes.push(h);
                continue;
            }
            if f == 0 && bay == door_bay {
                let h = Hole { x, y: 0.0, w: 1.2, h: dh };
                door = Some(h);
                holes.push(h);
                continue;
            }
            if f == 0 && s.shop.is_some() && bay >= 1 {
                // Ett stort butikkvindu over resten av første etasje.
                if bay == 1 {
                    let x0 = -width * 0.5 + 0.5 + bay_w * 1.0 + 0.3;
                    let x1 = width * 0.5 - 0.6;
                    let h = Hole { x: (x0 + x1) * 0.5, y: 0.75, w: x1 - x0, h: 1.95 };
                    shop_hole = Some(h);
                    holes.push(h);
                }
                continue;
            }
            holes.push(Hole { x, y: floor_y + 0.95, w: win_w, h: win_h });
        }
    }
    // Forsiden med hull, de tre andre sidene uten (bakgårdene ser ingen).
    let front_t = local(Vec3::new(0.0, plinth, fz));
    let facade = super::Facade {
        origin: Vec3::new(s.front.x, gy, s.front.z),
        along: rot * Vec3::X,
        facing: s.facing,
        width,
        door: door.map(|d| d.x).unwrap_or(0.0),
        shop: s.shop.is_some(),
    };
    let base_band = if s.stone_base { Some((floor_h - 0.05, STONE)) } else { None };
    wall_banded(m, &front_t, width, wall_h, &holes, &wall, base_band);
    let back_t = local(Vec3::new(0.0, plinth, -fz)) * Affine3A::from_rotation_y(std::f32::consts::PI);
    let mut back_holes = vec![];
    for f in 0..s.floors {
        for bay in 0..bays {
            if b.rng.chance(0.7) {
                let x = -width * 0.5 + 0.5 + bay_w * (bay as f32 + 0.5);
                back_holes.push(Hole { x, y: f as f32 * floor_h + 0.95, w: win_w, h: win_h });
            }
        }
    }
    wall_banded(m, &back_t, width, wall_h, &back_holes, &wall, base_band);
    let side_r = local(Vec3::new(width * 0.5, plinth, 0.0)) * Affine3A::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let side_l = local(Vec3::new(-width * 0.5, plinth, 0.0)) * Affine3A::from_rotation_y(-std::f32::consts::FRAC_PI_2);
    wall_banded(m, &side_r, depth, wall_h, &[], &wall, base_band);
    wall_banded(m, &side_l, depth, wall_h, &[], &wall, base_band);
    // Toppen av veggen under taket.
    m.quad(
        &local(Vec3::new(0.0, plinth + wall_h, 0.0)),
        [Vec3::new(-width * 0.5, 0.0, fz), Vec3::new(width * 0.5, 0.0, fz), Vec3::new(width * 0.5, 0.0, -fz), Vec3::new(-width * 0.5, 0.0, -fz)],
        &wall,
        [1.0; 4],
    );

    // Smyg, vinduer, dør.
    let reveal_depth = if matches!(s.wall, Wall::Brick | Wall::Plaster(_)) { 0.2 } else { 0.1 };
    let reveal_mat = if matches!(s.wall, Wall::Brick) { wall } else { Mat::tex(Layer::Plaster, 0xffffff).tint(match s.wall { Wall::Plaster(c) | Wall::Cladding(c) | Wall::Standing(c) => c, Wall::Brick => Vec3::ONE }) };
    for h in &holes {
        reveal(m, &front_t, *h, reveal_depth, &reveal_mat);
    }
    for h in &back_holes {
        reveal(m, &back_t, *h, reveal_depth, &reveal_mat);
        window(m, &back_t, *h, reveal_depth, s.wall, (2, 2), has.0.then_some(&mut models));
    }
    for h in &holes {
        if Some(*h).map(|x| x.y) == door.map(|d| d.y) && door.map(|d| d.x) == Some(h.x) {
            continue;
        }
        if shop_hole.map(|sh| sh.x) == Some(h.x) && shop_hole.map(|sh| sh.y) == Some(h.y) {
            window(m, &front_t, *h, reveal_depth, s.wall, (3, 1), None);
            continue;
        }
        if balcony.map(|bh| (bh.x, bh.y)) == Some((h.x, h.y)) {
            // Balkongdør med glass, skive og rekkverk i jern.
            let door_mat = Mat::tex(Layer::Cladding, 0xffffff).slot(s.door_slot).flags(flag::ROT_UV | flag::NO_DIRT);
            let dz = -reveal_depth - 0.05;
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x, h.y + h.h * 0.5, dz))), Vec3::new(h.w - 0.1, h.h - 0.05, 0.06), &door_mat, 1.0);
            let glass = Mat::special(kind::GLASS, 0x1d2630, 0.05);
            m.quad(&front_t, [Vec3::new(h.x - 0.3, h.y + 0.9, dz + 0.04), Vec3::new(h.x + 0.3, h.y + 0.9, dz + 0.04), Vec3::new(h.x + 0.3, h.y + 1.9, dz + 0.04), Vec3::new(h.x - 0.3, h.y + 1.9, dz + 0.04)], &glass, [1.0; 4]);
            let bw = h.w + 1.4;
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x, h.y - 0.08, 0.55))), Vec3::new(bw, 0.16, 1.1), &CONCRETE, 1.0);
            for k in 0..=((bw / 0.14) as i32) {
                let x = h.x - bw * 0.5 + k as f32 * 0.14;
                m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(x, h.y + 0.5, 1.05))), Vec3::new(0.02, 1.0, 0.02), &IRON, 1.0);
            }
            for k in 0..=7 {
                let z = k as f32 * 0.15;
                m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x - bw * 0.5, h.y + 0.5, z))), Vec3::new(0.02, 1.0, 0.02), &IRON, 1.0);
                m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x + bw * 0.5, h.y + 0.5, z))), Vec3::new(0.02, 1.0, 0.02), &IRON, 1.0);
            }
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x, h.y + 1.0, 1.05))), Vec3::new(bw + 0.04, 0.04, 0.05), &IRON, 1.0);
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x - bw * 0.5, h.y + 1.0, 0.52))), Vec3::new(0.05, 0.04, 1.06), &IRON, 1.0);
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x + bw * 0.5, h.y + 1.0, 0.52))), Vec3::new(0.05, 0.04, 1.06), &IRON, 1.0);
            solids.push((center + rot * Vec3::new(h.x, plinth + h.y, fz + 0.55), Vec3::new(bw, 0.3, 1.1), yaw));
            continue;
        }
        let panes = if b.rng.chance(0.5) { (2, 2) } else { (2, 3) };
        window(m, &front_t, *h, reveal_depth, s.wall, panes, has.0.then_some(&mut models));
        // Skodder på hver side, i dørfargen.
        if s.shutters {
            let shutter = Mat::tex(Layer::Cladding, 0xffffff).slot(s.door_slot).flags(flag::ROT_UV | flag::NO_DIRT);
            for sx in [-1.0, 1.0] {
                m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x + sx * (h.w * 0.5 + 0.08 + h.w * 0.25), h.y + h.h * 0.5, 0.035))), Vec3::new(h.w * 0.5 - 0.02, h.h + 0.04, 0.05), &shutter, 1.0);
            }
        }
        // Blomsterkasser under vinduene i etasjene over.
        if s.flower_boxes && h.y > floor_h && b.rng.chance(0.6) {
            if has.1 {
                models.push(("flower_box", front_t * Affine3A::from_translation(Vec3::new(h.x, h.y - 0.31, 0.0))));
                continue;
            }
            let bz = 0.16;
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(h.x, h.y - 0.2, bz))), Vec3::new(h.w + 0.1, 0.22, 0.26), &OAK, 1.0);
            for k in 0..6 {
                let fx = h.x - h.w * 0.45 + k as f32 * h.w * 0.18;
                let c = [0xd84a5a, 0xf2c14e, 0xe9e2d0, 0x9b59b6, 0xf07f3c, 0xffffff][k];
                m.sphere(&(front_t * Affine3A::from_translation(Vec3::new(fx, h.y - 0.02, bz + 0.03))), Vec3::splat(0.07), 5, 7, &Mat::plain(c, 0.7));
                m.sphere(&(front_t * Affine3A::from_translation(Vec3::new(fx + 0.05, h.y - 0.1, bz + 0.12))), Vec3::new(0.09, 0.06, 0.09), 4, 6, &Mat::plain(0x3f6b2a, 0.8));
            }
        }
    }

    // Døra: karm, dørblad med fyllinger, håndtak, trapp og rekkverk.
    if let Some(d) = door {
        let door_mat = Mat::tex(Layer::Cladding, 0xffffff).slot(s.door_slot).flags(flag::ROT_UV | flag::NO_DIRT);
        let dz = -reveal_depth - 0.08;
        let fw = 0.08;
        let frame = |m: &mut Mesh, c: Vec3, sz: Vec3| m.cuboid_ao(&(front_t * Affine3A::from_translation(c)), sz, &WHITE_TRIM, 1.0);
        frame(m, Vec3::new(d.x - d.w * 0.5 + fw * 0.5, d.h * 0.5, dz + 0.04), Vec3::new(fw, d.h, 0.1));
        frame(m, Vec3::new(d.x + d.w * 0.5 - fw * 0.5, d.h * 0.5, dz + 0.04), Vec3::new(fw, d.h, 0.1));
        frame(m, Vec3::new(d.x, d.h - fw * 0.5, dz + 0.04), Vec3::new(d.w, fw, 0.1));
        let leaf_w = d.w - 2.0 * fw;
        m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(d.x, (d.h - fw) * 0.5, dz))), Vec3::new(leaf_w, d.h - fw, 0.06), &door_mat, 1.0);
        for py in [0.55, 1.45] {
            m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(d.x, py, dz + 0.045))), Vec3::new(leaf_w - 0.3, 0.6, 0.03), &door_mat, 1.0);
        }
        // Overlys over døra.
        let glass = Mat::special(kind::GLASS, 0x1d2630, 0.05);
        m.quad(
            &front_t,
            [Vec3::new(d.x - leaf_w * 0.5, d.h - fw - 0.3, dz + 0.0), Vec3::new(d.x + leaf_w * 0.5, d.h - fw - 0.3, dz), Vec3::new(d.x + leaf_w * 0.5, d.h - fw, dz), Vec3::new(d.x - leaf_w * 0.5, d.h - fw, dz)],
            &glass,
            [1.0; 4],
        );
        m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(d.x, d.h - fw - 0.3, dz + 0.02))), Vec3::new(leaf_w, 0.04, 0.05), &WHITE_TRIM, 1.0);
        // Håndtak.
        m.cylinder(&(front_t * Affine3A::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), Vec3::new(d.x + leaf_w * 0.5 - 0.12, 1.05, dz + 0.04))), 0.012, 0.012, 0.06, 8, &IRON, true);
        m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(d.x + leaf_w * 0.5 - 0.17, 1.05, dz + 0.1))), Vec3::new(0.12, 0.02, 0.02), &IRON, 1.0);
        // Trappa: to trinn ut fra sokkelen, med rekkverk i jern.
        let steps = 2;
        for k in 0..steps {
            let sh = plinth / steps as f32;
            let out = 0.42 * (steps - k) as f32;
            m.cuboid(&local(Vec3::new(d.x, sh * (k as f32 + 0.5), fz + out * 0.5)), Vec3::new(d.w + 0.6, sh, out), &STONE);
        }
        solids.push((center + rot * Vec3::new(d.x, plinth * 0.5, fz + 0.42), Vec3::new(d.w + 0.6, plinth, 0.84), yaw));
        for sx in [-1.0, 1.0] {
            let rx = d.x + sx * (d.w * 0.5 + 0.25);
            for (zz, hh) in [(0.1, 1.0), (0.8, 0.85)] {
                m.cylinder(&local(Vec3::new(rx, plinth * 0.2, fz + zz)), 0.02, 0.02, hh, 6, &IRON, false);
            }
            let top = [Vec2::new(0.0, 0.0), Vec2::new(0.03, 0.0), Vec2::new(0.03, 0.03), Vec2::new(0.0, 0.03)];
            let from = local(Vec3::new(rx, plinth * 0.2 + 1.0, fz + 0.1)).translation.into();
            let to = local(Vec3::new(rx, plinth * 0.2 + 0.85, fz + 0.8)).translation.into();
            m.extrude(&top, from, to, rot * Vec3::X * sx, &IRON, true);
        }
        // Lampe over døra.
        if s.door_lamp && has.2 {
            let p = Vec3::new(d.x, d.h + 0.4, 0.0);
            models.push(("door_lamp", front_t * Affine3A::from_translation(p)));
            let wp = front_t.transform_point3(p + Vec3::new(0.0, -0.05, 0.3));
            lights.push((wp, Vec3::new(1.0, 0.8, 0.55), 2.2, 6.0));
        } else if s.door_lamp {
            let lamp = Mat::special(kind::LAMP, 0xfff1d6, 0.3);
            let p = Vec3::new(d.x, d.h + 0.35, 0.18);
            m.cuboid_ao(&(front_t * Affine3A::from_translation(p + Vec3::new(0.0, 0.14, -0.12))), Vec3::new(0.06, 0.06, 0.22), &IRON, 1.0);
            m.cuboid_ao(&(front_t * Affine3A::from_translation(p)), Vec3::new(0.2, 0.26, 0.2), &lamp, 1.0);
            m.cuboid_ao(&(front_t * Affine3A::from_translation(p + Vec3::new(0.0, 0.15, 0.0))), Vec3::new(0.26, 0.04, 0.26), &IRON, 1.0);
            let wp = front_t.transform_point3(p + Vec3::new(0.0, -0.1, 0.15));
            lights.push((wp, Vec3::new(1.0, 0.8, 0.55), 2.2, 6.0));
        }
    }

    // Butikken: fascia med skilt over vinduet.
    if let (Some(name), Some(sh)) = (s.shop, shop_hole) {
        let y = sh.y + sh.h + 0.12;
        let board_w = sh.w + 1.0;
        let dark = Mat::plain(0x24303a, 0.6);
        m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(sh.x, y + 0.3, 0.06))), Vec3::new(board_w, 0.6, 0.12), &dark, 1.0);
        m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(sh.x, y + 0.62, 0.1))), Vec3::new(board_w + 0.1, 0.06, 0.2), &WHITE_TRIM, 1.0);
        let label = Mat::plain(0xf4efe4, 0.5).flags(flag::GLOW_ALWAYS);
        let mut text = Mesh::default();
        b.font.centered(&mut text, name, Vec3::new(0.0, 0.0, 0.0), 32.0, 0.012, &label);
        text.transform(&(front_t * Affine3A::from_translation(Vec3::new(sh.x, y + 0.3, 0.135))));
        m.append(&text);
        // Markise.
        let awning = Mat::tex(Layer::Cladding, 0xffffff).tint(Vec3::new(0.55, 0.12, 0.1)).flags(flag::NO_DIRT);
        let aw = Affine3A::from_rotation_x(-0.5);
        m.cuboid_ao(&(front_t * Affine3A::from_translation(Vec3::new(sh.x, y - 0.05, 0.45)) * aw), Vec3::new(sh.w + 0.4, 0.03, 1.0), &awning, 1.0);
    }

    // Hjørnekasser på trehus.
    if matches!(s.wall, Wall::Cladding(_) | Wall::Standing(_)) {
        for (sx, sz) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            m.cuboid_ao(&local(Vec3::new(sx * (width * 0.5), plinth + wall_h * 0.5, sz * fz)), Vec3::new(0.16, wall_h, 0.16), &WHITE_TRIM, 1.0);
        }
    }

    // Gesimsen: en profil langs forsiden og baksiden under takfoten.
    let cornice = [Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.0), Vec2::new(0.1, 0.08), Vec2::new(0.18, 0.08), Vec2::new(0.18, 0.18), Vec2::new(0.26, 0.18), Vec2::new(0.26, 0.26), Vec2::new(0.0, 0.26)];
    let trim = if matches!(s.wall, Wall::Brick | Wall::Plaster(_)) { STONE } else { WHITE_TRIM };
    for side in [1.0f32, -1.0] {
        let y = plinth + wall_h - 0.26;
        let from = local(Vec3::new(-width * 0.5 - 0.1, y, side * fz)).translation.into();
        let to = local(Vec3::new(width * 0.5 + 0.1, y, side * fz)).translation.into();
        m.extrude(&cornice, from, to, rot * Vec3::Z * side, &trim, true);
    }

    // Taket, med mønet langs fasaden.
    let roof_h = b.rng.range(2.6, 4.2);
    let roof_mat = match s.roof {
        Roof::Slate => Mat::tex(Layer::Slate, 0xffffff),
        Roof::Tiles => Mat::tex(Layer::Tiles, 0xffffff),
    };
    let overhang = 0.5;
    let roof_t = local(Vec3::new(0.0, plinth + wall_h, 0.0));
    if s.gable_front {
        // Mønet på tvers: gavlen vender mot gata, med vindskier langs skråtaket.
        let rt = roof_t * Affine3A::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let roof_h = roof_h.max(width * 0.45);
        m.gable_roof(&rt, depth, width, roof_h, overhang, &roof_mat, &wall, &DARK_TRIM);
        m.cylinder(&(rt * Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(depth * 0.5 + overhang, roof_h + 0.1, 0.0))), 0.09, 0.09, depth + 2.0 * overhang, 8, &DARK_TRIM, true);
        // Takrenner langs langsidene (x-sidene).
        let eave_y = -overhang * roof_h / (width * 0.5);
        for side in [1.0f32, -1.0] {
            m.cylinder(&(roof_t * Affine3A::from_rotation_translation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2), Vec3::new(side * (width * 0.5 + overhang + 0.06), eave_y - 0.02, -fz - overhang))), 0.07, 0.07, depth + 2.0 * overhang, 8, &IRON, true);
            let px = side * (width * 0.5 + 0.1);
            m.cylinder(&local(Vec3::new(px, 0.1, fz - 0.3)), 0.045, 0.045, plinth + wall_h + eave_y - 0.2, 8, &IRON, false);
        }
        b.solid(Shape::Cuboid { center: center + Vec3::Y * (plinth + wall_h) * 0.5, size: Vec3::new(width + 0.1, plinth + wall_h, depth + 0.1), yaw });
        for (c, sz, y) in solids {
            b.solid_box(c, sz, y);
        }
        for (p, c, i, r) in lights {
            b.light(p, c, i, r);
        }
        for (n, t) in models {
            b.place(n, t);
        }
        b.facades.push(facade);
        return;
    }
    m.gable_roof(&roof_t, width, depth, roof_h, overhang, &roof_mat, &wall, &DARK_TRIM);
    // Mønekam.
    m.cylinder(&(roof_t * Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(width * 0.5 + overhang, roof_h + 0.1, 0.0))), 0.09, 0.09, width + 2.0 * overhang, 8, &DARK_TRIM, true);
    // Sperrehodene under takskjegget.
    let eave_y = -overhang * roof_h / fz;
    let n_rafters = (width / 0.7) as i32;
    for side in [1.0f32, -1.0] {
        for k in 0..=n_rafters {
            let x = -width * 0.5 + k as f32 * (width / n_rafters as f32);
            m.cuboid_ao(&(roof_t * Affine3A::from_translation(Vec3::new(x, eave_y * 0.5 + 0.02, side * (fz + overhang * 0.5)))), Vec3::new(0.08, 0.14, overhang), &WHITE_TRIM, 1.0);
        }
        // Takrenne og nedløp.
        let gy = eave_y - 0.02;
        m.cylinder(&(roof_t * Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(width * 0.5 + overhang, gy, side * (fz + overhang + 0.06)))), 0.07, 0.07, width + 2.0 * overhang, 8, &IRON, true);
        if side > 0.0 || b.rng.chance(0.5) {
            let px = if side > 0.0 { width * 0.5 - 0.15 } else { -width * 0.5 + 0.15 };
            let pz = side * (fz + 0.1);
            m.cylinder(&local(Vec3::new(px, 0.1, pz)), 0.045, 0.045, plinth + wall_h + eave_y - 0.2, 8, &IRON, false);
            m.sphere(&local(Vec3::new(px, plinth + wall_h + eave_y - 0.1, pz)), Vec3::splat(0.07), 6, 8, &IRON);
            m.cylinder(&(local(Vec3::new(px, plinth + wall_h + eave_y - 0.1, pz)) * Affine3A::from_rotation_x(side * std::f32::consts::FRAC_PI_2)), 0.045, 0.045, overhang - 0.04, 8, &IRON, false);
        }
    }
    // Pipe i mur, med beslag og pipehatt.
    if b.rng.chance(0.7) {
        let x = b.rng.range(-width * 0.3, width * 0.3);
        let zc = -depth * 0.12;
        let top = plinth + wall_h + roof_h * 0.75 + 0.8;
        let bottom = plinth + wall_h + roof_h * 0.2;
        m.cuboid(&local(Vec3::new(x, (top + bottom) * 0.5, zc)), Vec3::new(0.75, top - bottom, 0.75), &Mat::tex(Layer::Brick, 0xffffff));
        m.cuboid(&local(Vec3::new(x, top + 0.05, zc)), Vec3::new(0.9, 0.1, 0.9), &CONCRETE);
        for dx in [-0.18, 0.18] {
            m.cylinder(&local(Vec3::new(x + dx, top + 0.1, zc)), 0.11, 0.1, 0.3, 10, &Mat::tex(Layer::Tiles, 0xffffff), true);
        }
    }
    // Ark på taket.
    if s.dormer && s.floors >= 2 && width > 6.0 {
        let dw = 1.7;
        let dx = b.rng.range(-width * 0.25, width * 0.25);
        // Arken står på forsiden av taket, midt i skråningen.
        let frac = 0.45;
        let dz = fz * (1.0 - frac) - 0.3;
        let dy = plinth + wall_h + roof_h * frac * 0.85 - 0.6;
        let dt = local(Vec3::new(dx, dy, dz));
        m.cuboid_faces(&dt, Vec3::new(dw, 2.0, 1.6), [&wall, &wall, &wall, &wall, &wall, &wall], 1.0);
        m.gable_roof(&(dt * Affine3A::from_translation(Vec3::new(0.0, 1.0, 0.0))), dw, 1.6, 0.8, 0.25, &roof_mat, &wall, &DARK_TRIM);
        let ft = dt * Affine3A::from_translation(Vec3::new(0.0, -1.0, 0.8));
        let hole = Hole { x: 0.0, y: 0.5, w: 0.9, h: 1.2 };
        reveal(m, &ft, hole, 0.1, &reveal_mat);
        window(m, &ft, hole, 0.1, s.wall, (2, 2), None);
    }

    for (c, sz, y) in solids {
        b.solid_box(c, sz, y);
    }
    for (p, c, i, r) in lights {
        b.light(p, c, i, r);
    }
    for (n, t) in models {
        b.place(n, t);
    }
    b.solid(Shape::Cuboid { center: center + Vec3::Y * (plinth + wall_h) * 0.5, size: Vec3::new(width + 0.1, plinth + wall_h, depth + 0.1), yaw });
    b.facades.push(facade);
    let _ = (OAK, Layer::Oak);
}
