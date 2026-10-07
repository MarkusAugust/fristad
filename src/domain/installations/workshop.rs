//! Verkstedet: et verkstedbygg sør for byen, ved stien videre fra Sørgata.
//! Under skuret langs vestveggen står tre arbeidsbenker, én for hver
//! veiledningsside i dokumentasjonen: «Kom i gang», «Typesikker bruk» og
//! «Tilgjengelighet». Sikt på benken og trykk E for å lese sida.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, DARK_TRIM, IRON, OAK, STONE, WHITE_TRIM};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

pub const POS: Vec3 = Vec3::new(16.0, 0.0, 76.0);
/// Bygget vender mot vest, der stien går.
pub const YAW: f32 = -std::f32::consts::FRAC_PI_2;
const W: f32 = 15.0;
const D: f32 = 9.0;
const WALL_H: f32 = 4.2;

/// Benkene langs fronten, fra venstre: lokal x.
const BENCH_X: [f32; 3] = [-5.2, -1.6, 2.0];

/// Bygg verkstedet om det ikke finnes.
fn building(b: &mut Builder) {
    if b.props.iter().any(|p| p.name == "workshop") {
        return;
    }
    let p = b.on_ground(POS);
    let t = at(p, YAW);
    let wall = Mat::tex(Layer::Cladding, 0xffffff).tint(Vec3::new(0.55, 0.2, 0.16)).flags(flag::ROT_UV);
    let m = &mut b.mesh;
    // Sokkel, vegger og tak med mønet på langs.
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, -0.75, -D * 0.5))), Vec3::new(W + 0.12, 2.5, D + 0.12), &STONE);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H * 0.5 + 0.5, -D * 0.5))), Vec3::new(W, WALL_H, D), &wall, 1.0);
    for (sx, sz) in [(-1.0, 0.0), (1.0, 0.0), (-1.0, -1.0), (1.0, -1.0)] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(sx * W * 0.5, WALL_H * 0.5 + 0.5, sz * D))), Vec3::new(0.18, WALL_H, 0.18), &WHITE_TRIM, 1.0);
    }
    m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 0.5, -D * 0.5))), W, D, 2.4, 0.5, &Mat::tex(Layer::Tiles, 0xffffff), &wall, &DARK_TRIM);
    // Den store skyvedøra midt på fronten, og to vinduer.
    let door = Mat::tex(Layer::Oak, 0xffffff).flags(flag::ROT_UV);
    let dx = 5.6;
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.5 + 1.5, 0.06))), Vec3::new(3.2, 3.0, 0.1), &door, 1.0);
    for y in [0.9, 1.9, 2.9] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.5 + y, 0.12))), Vec3::new(3.0, 0.1, 0.04), &IRON, 1.0);
    }
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.5 + 3.15, 0.1))), Vec3::new(4.0, 0.08, 0.14), &IRON, 1.0);
    // Vinduene sitter på sideveggene, så de ikke dekker tavlene på fronten.
    let glass = Mat::special(kind::GLASS, 0x1d2630, 0.05);
    for sx in [-1.0f32, 1.0] {
        let wt = t * Affine3A::from_rotation_translation(Quat::from_rotation_y(sx * std::f32::consts::FRAC_PI_2), Vec3::new(sx * (W * 0.5), 0.5 + 2.6, -D * 0.5));
        m.cuboid_ao(&(wt * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0))), Vec3::new(1.4, 1.0, 0.2), &WHITE_TRIM, 1.0);
        m.cuboid(&(wt * Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.11))), Vec3::new(1.2, 0.8, 0.02), &glass);
    }
    // Skuret: et pulttak på stolper langs hele fronten.
    let roof = Mat::tex(Layer::Slate, 0xffffff);
    m.cuboid_ao(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_x(0.28), Vec3::new(0.0, WALL_H + 0.1, 1.4))), Vec3::new(W + 0.6, 0.1, 3.2), &roof, 1.0);
    // Stolpene går 1,5 m ned i bakken, så de står på hellingen.
    for x in [-W * 0.5 + 0.3, -3.4, 0.2, 3.8, W * 0.5 - 0.3] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, (WALL_H + 0.1) * 0.5 - 1.0, 2.8))), Vec3::new(0.16, WALL_H + 1.6, 0.16), &OAK, 1.0);
    }
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H - 0.3, 2.8))), Vec3::new(W + 0.4, 0.14, 0.14), &OAK, 1.0);
    // Skiltet over døra.
    let plate = Mat::plain(0x24303a, 0.6);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 1.4, -0.02))), Vec3::new(4.6, 0.5, 0.1), &plate, 1.0);
    let mut label = Mesh::default();
    b.font.centered(&mut label, "VERKSTEDET", Vec3::ZERO, 26.0, 0.012, &Mat::plain(0xf4efe4, 0.5).flags(flag::GLOW_ALWAYS));
    label.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 1.4, 0.035))));
    b.mesh.append(&label);
    // Esse i hjørnet: en murt kasse med glør, og en ambolt.
    let brick = Mat::tex(Layer::Brick, 0xffffff);
    // Essen og ambolten står på bakken der de er, og essen går litt ned i den.
    let fp = t.transform_point3(Vec3::new(W * 0.5 - 1.6, 0.0, 1.0));
    let gf = b.ground(fp.x, fp.z) - p.y;
    let ap = t.transform_point3(Vec3::new(W * 0.5 - 3.4, 0.0, 1.2));
    let ga = b.ground(ap.x, ap.z) - p.y;
    b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(W * 0.5 - 1.6, gf + 0.2, 1.0))), Vec3::new(1.4, 1.4, 1.2), &brick, 1.0);
    b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(W * 0.5 - 1.6, gf + 0.93, 1.0))), Vec3::new(0.9, 0.06, 0.8), &Mat::special(kind::LAMP, 0xff6a2a, 0.9), 1.0);
    b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(W * 0.5 - 3.4, ga + 0.25, 1.2))), Vec3::new(0.3, 0.9, 0.3), &OAK, 1.0);
    b.mesh.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(W * 0.5 - 3.4, ga + 0.8, 1.2))), Vec3::new(0.7, 0.2, 0.25), &IRON, 1.0);
    let q = Quat::from_rotation_y(YAW);
    b.light(p + q * Vec3::new(W * 0.5 - 1.6, 1.6, 1.0), Vec3::new(1.0, 0.45, 0.15), 2.5, 5.0);
    b.solid_box(p + q * Vec3::new(0.0, WALL_H * 0.5 + 0.5, -D * 0.5), Vec3::new(W, WALL_H, D), YAW);
    b.solid_box(p + q * Vec3::new(W * 0.5 - 1.6, gf + 0.45, 1.0), Vec3::new(1.4, 0.9, 1.2), YAW);
    b.prop("workshop", Mesh::default(), Affine3A::IDENTITY, None);
}

/// En arbeidsbenk under skuret med en tavle på veggen over.
fn bench(b: &mut Builder, name: &'static str, x: f32, title: &str, lines: &[&str]) -> usize {
    building(b);
    let p = b.on_ground(POS);
    let t = at(p, YAW);
    // Benken står på bakken der den er, ikke på sokkelens høyde: bakken
    // heller langs fronten. Beina går litt ned i grusen.
    let foot = t.transform_point3(Vec3::new(x, 0.0, 1.3));
    let g = b.ground(foot.x, foot.z) - p.y;
    let m = &mut b.mesh;
    let top = Mat::tex(Layer::Oak, 0xffffff).flags(flag::ROT_UV);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, g + 0.86, 1.3))), Vec3::new(2.6, 0.08, 0.8), &top, 1.0);
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x + sx * 1.2, g + 0.36, 1.3 + sz * 0.32))), Vec3::new(0.08, 1.0, 0.08), &OAK, 1.0);
    }
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, g + 0.3, 1.3))), Vec3::new(2.4, 0.04, 0.6), &OAK, 1.0);
    // Verktøy: en skrustikke, en hammer og noen bord.
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x - 1.0, g + 1.0, 1.55))), Vec3::new(0.22, 0.2, 0.3), &IRON, 1.0);
    m.cylinder(&(t * Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(x + 0.9, g + 0.92, 1.2))), 0.014, 0.014, 0.3, 6, &OAK, true);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x + 0.6, g + 0.93, 1.2))), Vec3::new(0.1, 0.06, 0.06), &IRON, 1.0);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(x, g + 0.34, 1.3))), Vec3::new(1.6, 0.05, 0.2), &OAK, 1.0);
    b.solid_box(p + Quat::from_rotation_y(YAW) * Vec3::new(x, g + 0.45, 1.3), Vec3::new(2.6, 0.9, 0.8), YAW);
    // Tavla på veggen: tittel i 20 px og tre linjer i 16.
    let (bw, bh) = (300.0, 16.0 + 25.0 + 8.0 + 24.0 * lines.len() as f32 + 16.0);
    let mut board = Mesh::default();
    signs::boxed(&mut board, Vec3::ZERO, &Box2 { w: bw, h: bh, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
    signs::paragraph(&mut board, b.font, &[title], -bw * 0.5 + 16.0, bh * 0.5 - 16.0, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
    signs::paragraph(&mut board, b.font, lines, -bw * 0.5 + 16.0, bh * 0.5 - 16.0 - 25.0 - 8.0, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT);
    let rest = t * Affine3A::from_translation(Vec3::new(x, 0.5 + 2.2, 0.03));
    b.prop(name, board, rest, Some((Vec3::ZERO, Vec3::new(bw * PX * 0.5, bh * PX * 0.5, 0.05))))
}

#[derive(Default)]
pub struct GetStarted;
#[derive(Default)]
pub struct TypeSafe;
#[derive(Default)]
pub struct Accessibility;

impl Installation for GetStarted {
    fn component(&self) -> Component {
        Component { name: "kom-i-gang", class: "" }
    }
    fn build(&mut self, b: &mut Builder) {
        bench(b, "kom-i-gang", BENCH_X[0], "Kom i gang", &["npm install @fristil/designsystem", "Importer tokens.css og", "komponentens CSS, så er du i gang."]);
    }
    fn pose(&self, _p: usize, rest: Affine3A, _e: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }
}

impl Installation for TypeSafe {
    fn component(&self) -> Component {
        Component { name: "typesikker-bruk", class: "" }
    }
    fn build(&mut self, b: &mut Builder) {
        bench(b, "typesikker-bruk", BENCH_X[1], "Typesikker bruk", &["fs.button({ variant: \"secondary\" })", "gir klassen og attributtene,", "med autofullføring og feil i editoren."]);
    }
    fn pose(&self, _p: usize, rest: Affine3A, _e: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }
}

impl Installation for Accessibility {
    fn component(&self) -> Component {
        Component { name: "tilgjengelighet", class: "" }
    }
    fn build(&mut self, b: &mut Builder) {
        bench(b, "tilgjengelighet", BENCH_X[2], "Tilgjengelighet", &["Kontrast, fokus og roller er løst", "sentralt og kontrollert av tester.", "Teksten og rekkefølgen er din jobb."]);
    }
    fn pose(&self, _p: usize, rest: Affine3A, _e: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }
}
