//! Søknadskontoret sør for Østre gate: et lite kontorbygg med skjemaet
//! «Søknad om bostøtte» på fronten. Skjemakomponentene (input, select,
//! radio, checkbox, textarea) er hver sin installasjon og legger feltene
//! sine på arket her. Den første som bygger lager kontoret og arket;
//! etikettene, legenden og sendeknappen står fast.
//!
//! Målene er i piksler relativt til arkets sentrum, y opp. Arket er
//! 784 × 400 px i to kolonner på 360 px, med plass til en feilmelding
//! (`.fs-error-text`) under hvert felt. Skjemaet virker som et skjema: F på
//! et felt gir det tastaturfokus, og «Send søknad» (`submit.rs`) validerer.

use super::signs::{self, Box2};
use super::PX;
use crate::domain::city::{at, Builder, DARK_TRIM, STONE, WHITE_TRIM};
use crate::domain::geometry::{flag, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec3};

pub const POS: Vec3 = Vec3::new(36.0, 0.0, 10.5);
pub const YAW: f32 = std::f32::consts::PI;
const W: f32 = 9.6;
const D: f32 = 5.0;
const WALL_H: f32 = 4.9;

pub const SHEET_W: f32 = 784.0;
pub const SHEET_H: f32 = 400.0;
/// Kolonnenes venstrekant og bredde.
pub const COL_W: f32 = 360.0;
pub const LEFT: f32 = -372.0;
pub const RIGHT: f32 = 12.0;

/// Feltenes plass (overkant i px fra sentrum), fra oppsettet i modulteksten.
pub const INPUT_TOP: f32 = 125.0;
pub const SELECT_TOP: f32 = 33.0;
pub const TEXTAREA_TOP: f32 = -59.0;
pub const RADIO_ROWS: [f32; 3] = [120.0, 88.0, 56.0];
pub const CHECK_ROWS: [f32; 2] = [20.0, -12.0];
pub const FIELD_H: f32 = 42.0;
pub const TEXTAREA_H: f32 = 80.0;
/// Feilmeldingenes overkant: under e-post, fylke, begrunnelse og vilkår.
pub const ERROR_TOPS: [f32; 4] = [80.0, -12.0, -142.0, -40.0];
/// Sendeknappens sentrum.
pub const BUTTON_Y: f32 = -85.0;

/// Vedleggsarket på baksiden av kontoret (mot sør): overskriften,
/// ledeteksten, hjelpeteksten og feilmeldingen, søkefeltet, feltsettet og
/// filopplastingen er hver sin installasjon og legger seg her. Målene er
/// i px relativt til arkets sentrum, y opp, samme kolonner som forsiden.
pub const ANNEX_TITLE_TOP: f32 = 128.0;
/// Kolonnenes overkant under overskriften (mega-størrelsen rekker ned hit).
pub const ANNEX_TOP: f32 = 64.0;
pub const ID_FIELD_TOP: f32 = 39.0;
pub const HELP_TOP: f32 = -7.0;
pub const ERROR_TOP: f32 = -28.0;
pub const SEARCH_LABEL_TOP: f32 = -62.0;
pub const SEARCH_TOP: f32 = -87.0;
pub const SEARCH_W: f32 = 250.0;
pub const UPLOAD_LABEL_TOP: f32 = 28.0;
pub const UPLOAD_TOP: f32 = 3.0;
pub const UPLOAD_H: f32 = 56.0;
pub const FILE_ROW_TOP: f32 = -82.0;
pub const FILE_ROW_H: f32 = 44.0;

/// Vedleggsarkets ramme: sentrum i arket, forsiden mot sør.
pub fn annex_t() -> Affine3A {
    at(POS, YAW) * Affine3A::from_translation(Vec3::new(0.0, 2.6, -D - 0.13)) * Affine3A::from_rotation_y(std::f32::consts::PI)
}

/// Et punkt på vedleggsarket, i px, som transform.
pub fn on_annex(x: f32, y: f32, z: f32) -> Affine3A {
    annex_t() * Affine3A::from_translation(Vec3::new(x * PX, y * PX, z))
}

/// Stiplet 1 px kant (strek 6, mellomrom 4) rundt en flate, som
/// `border: 1px dashed`. Ligger i z.
pub fn dashed(m: &mut Mesh, w: f32, h: f32, slot: u8, z: f32) {
    let mat = Mat::plain(0xffffff, 0.5).slot(slot).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let (dash, gap) = (6.0, 4.0);
    let mut x = -w * 0.5 + 2.0;
    while x + dash <= w * 0.5 - 2.0 {
        for sy in [-1.0f32, 1.0] {
            m.cuboid(&Affine3A::from_translation(Vec3::new((x + dash * 0.5) * PX, sy * (h * 0.5 - 0.5) * PX, z)), Vec3::new(dash * PX, PX, 0.004), &mat);
        }
        x += dash + gap;
    }
    let mut y = -h * 0.5 + 2.0;
    while y + dash <= h * 0.5 - 2.0 {
        for sx in [-1.0f32, 1.0] {
            m.cuboid(&Affine3A::from_translation(Vec3::new(sx * (w * 0.5 - 0.5) * PX, (y + dash * 0.5) * PX, z)), Vec3::new(PX, dash * PX, 0.004), &mat);
        }
        y += dash + gap;
    }
}

/// Bygg vedleggsarket om det ikke finnes: plata og det som står fast
/// (fødselsnummerfeltet, hjelpeteksten under opplastingen).
pub fn annex(b: &mut Builder) {
    office(b);
    if b.props.iter().any(|p| p.name == "annex") {
        return;
    }
    let st = annex_t();
    let mut sheet = Mesh::default();
    signs::boxed(&mut sheet, Vec3::ZERO, &Box2 { w: SHEET_W + 40.0, h: SHEET_H + 40.0, radius: 8.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.04 });
    let z = 0.03;
    // Fødselsnummerfeltet under ledeteksten, tomt, med plassholder.
    let mut f = Mesh::default();
    field(&mut f, COL_W, FIELD_H, slot::NEUTRAL_CANVAS, slot::NEUTRAL_BORDER);
    f.transform(&Affine3A::from_translation(Vec3::new((LEFT + COL_W * 0.5) * PX, (ID_FIELD_TOP - FIELD_H * 0.5) * PX, z)));
    sheet.append(&f);
    signs::text_left(&mut sheet, b.font, "11 siffer", Vec3::new((LEFT + 12.0) * PX, (ID_FIELD_TOP - FIELD_H * 0.5 - 0.36 * 16.0) * PX, z + 0.015), 16.0, slot::NEUTRAL_TEXT_SUBTLE);
    signs::paragraph(&mut sheet, b.font, &["PDF eller bilde, opptil 10 MB per fil."], RIGHT, UPLOAD_TOP - UPLOAD_H - 4.0, 12.0, 1.4, z, slot::NEUTRAL_TEXT_SUBTLE);
    sheet.transform(&st);
    b.mesh.append(&sheet);
    b.prop("annex", Mesh::default(), Affine3A::IDENTITY, None);
}

/// Arkets ramme: sentrum i arket, forsiden mot +Z.
pub fn sheet_t() -> Affine3A {
    at(POS, YAW) * Affine3A::from_translation(Vec3::new(0.6, 2.6, 0.13))
}

/// Et punkt på arket, i px, som transform.
pub fn on_sheet(x: f32, y: f32, z: f32) -> Affine3A {
    sheet_t() * Affine3A::from_translation(Vec3::new(x * PX, y * PX, z))
}

/// Et skjemafelt: 1 px kant, radius 4, med fyll og kant fra palettplasser.
pub fn field(m: &mut Mesh, w: f32, h: f32, fill: u8, border: u8) {
    signs::boxed(m, Vec3::ZERO, &Box2 { w, h, radius: 4.0, fill, border: Some(border), depth: 0.02 });
}

/// En feilmelding (`.fs-error-text`: 12 px medium, linjehøyde 1,4, farefargen)
/// under et felt, brutt til kolonnebredden, som en prop som vises ved feil.
pub fn error_text(b: &mut Builder, text: &str, left: f32, top: f32) -> usize {
    let lines = signs::wrap(b.font, text, 12.0, COL_W);
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mut m = Mesh::default();
    signs::paragraph(&mut m, b.font, &lines, 0.0, 0.0, 12.0, 1.4, 0.0, slot::DANGER_TEXT);
    b.prop("error_text", m, on_sheet(left, top, 0.035), None)
}

/// Bygg kontoret og arket om det ikke finnes.
pub fn office(b: &mut Builder) {
    if b.props.iter().any(|p| p.name == "form") {
        return;
    }
    let p = b.on_ground(POS);
    let t = at(p, YAW);
    b.fits("kontorskiltet", "SØKNADSKONTORET", 26.0, 0.012, 5.2);
    let wall = Mat::tex(Layer::Plaster, 0xffffff).tint(Vec3::new(0.88, 0.86, 0.8));
    let m = &mut b.mesh;
    m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, -0.75, -D * 0.5))), Vec3::new(W + 0.12, 2.5, D + 0.12), &STONE);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H * 0.5 + 0.5, -D * 0.5))), Vec3::new(W, WALL_H, D), &wall, 1.0);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 0.56, -D * 0.5))), Vec3::new(W + 0.5, 0.14, D + 0.5), &WHITE_TRIM, 1.0);
    m.gable_roof(&(t * Affine3A::from_translation(Vec3::new(0.0, WALL_H + 0.62, -D * 0.5))), W, D, 1.5, 0.4, &Mat::tex(Layer::Slate, 0xffffff), &wall, &DARK_TRIM);
    // Døra til venstre, med trinn og lampe.
    let door = Mat::tex(Layer::Cladding, 0xffffff).tint(Vec3::new(0.2, 0.3, 0.45)).flags(flag::ROT_UV | flag::NO_DIRT);
    let dx = -4.1;
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.5 + 1.05, 0.04))), Vec3::new(1.0, 2.1, 0.06), &door, 1.0);
    for sx in [-1.0f32, 1.0] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx + sx * 0.55, 0.5 + 1.05, 0.05))), Vec3::new(0.1, 2.2, 0.16), &WHITE_TRIM, 1.0);
    }
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.5 + 2.15, 0.05))), Vec3::new(1.2, 0.1, 0.16), &WHITE_TRIM, 1.0);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 0.25, 0.35))), Vec3::new(1.6, 0.5, 0.7), &STONE, 1.0);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(dx, 3.0, 0.12))), Vec3::new(0.22, 0.16, 0.16), &Mat::special(crate::domain::geometry::kind::LAMP, 0xfff1d6, 0.3), 1.0);
    // Skiltet over arket.
    let plate = Mat::plain(0x24303a, 0.6);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.6, 5.1, 0.06))), Vec3::new(5.2, 0.5, 0.1), &plate, 1.0);
    let mut label = Mesh::default();
    b.font.centered(&mut label, "SØKNADSKONTORET", Vec3::ZERO, 26.0, 0.012, &Mat::plain(0xf4efe4, 0.5).flags(flag::GLOW_ALWAYS));
    label.transform(&(t * Affine3A::from_translation(Vec3::new(0.6, 5.1, 0.115))));
    m.append(&label);

    // Arket: en lys plate med kant, og teksten som står fast.
    let st = sheet_t();
    let mut sheet = Mesh::default();
    signs::boxed(&mut sheet, Vec3::ZERO, &Box2 { w: SHEET_W + 40.0, h: SHEET_H + 40.0, radius: 8.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.04 });
    let z = 0.03;
    signs::paragraph(&mut sheet, b.font, &["Søknad om bostøtte"], LEFT, 180.0, 20.0, 1.25, z, slot::NEUTRAL_TEXT_STRONG);
    // Etikettene: 14 px, linjehøyde 1,5, med påkrevd-stjerne.
    for (label, top) in [("E-postadresse", 148.0), ("Fylke", 56.0), ("Begrunnelse", -36.0)] {
        signs::paragraph(&mut sheet, b.font, &[label], LEFT, top, 14.0, 1.5, z, slot::NEUTRAL_TEXT_STRONG);
        let lw = signs::width(b.font, label, 14.0) / PX;
        signs::paragraph(&mut sheet, b.font, &["*"], LEFT + lw + 3.0, top, 14.0, 1.5, z, slot::DANGER_TEXT);
    }
    // Legenden med påkrevd-stjerne.
    signs::paragraph(&mut sheet, b.font, &["Hvordan vil du ha svaret?"], RIGHT, 148.0, 16.0, 1.5, z, slot::NEUTRAL_TEXT_STRONG);
    let lw = signs::width(b.font, "Hvordan vil du ha svaret?", 16.0) / PX;
    signs::paragraph(&mut sheet, b.font, &["*"], RIGHT + lw + 4.0, 148.0, 16.0, 1.5, z, slot::DANGER_TEXT);
    for (k, label) in ["I digital postkasse", "Som brev i posten", "Hentes på kontoret"].iter().enumerate() {
        let s = if k == 2 { slot::NEUTRAL_TEXT_SUBTLE } else { slot::NEUTRAL_TEXT_STRONG };
        signs::paragraph(&mut sheet, b.font, &[label], RIGHT + 28.0, RADIO_ROWS[k], 14.0, 1.5, z, s);
    }
    for (k, label) in ["Jeg godtar vilkårene", "Send meg nyhetsbrev"].iter().enumerate() {
        signs::paragraph(&mut sheet, b.font, &[label], RIGHT + 28.0, CHECK_ROWS[k], 14.0, 1.5, z, slot::NEUTRAL_TEXT_STRONG);
    }
    sheet.transform(&st);
    b.mesh.append(&sheet);

    let q = Quat::from_rotation_y(YAW);
    b.solid_box(p + q * Vec3::new(0.0, WALL_H * 0.5 + 0.5, -D * 0.5), Vec3::new(W, WALL_H, D), YAW);
    b.solid_box(p + q * Vec3::new(dx, 0.25, 0.35), Vec3::new(1.6, 0.5, 0.7), YAW);
    b.light(p + q * Vec3::new(dx, 2.9, 0.4), Vec3::new(1.0, 0.85, 0.6), 2.0, 5.0);
    b.prop("form", Mesh::default(), Affine3A::IDENTITY, None);
}
