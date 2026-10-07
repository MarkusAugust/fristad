//! Plakaten på torget: en stor oppslagstavle ved inngangen som forteller
//! hva Fristad er, hva du finner i byen og hvordan du spiller. Du starter
//! med den i synsfeltet.

use super::{at, Builder, DARK_TRIM, OAK};
use crate::domain::geometry::{Layer, Mat};
use crate::domain::installations::signs::{self, Box2};
use crate::domain::installations::PX;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

const POS: Vec3 = Vec3::new(12.6, 0.0, 3.2);
/// Vender skrått mot nordvest, mot stedet du står når du kommer inn.
const YAW: f32 = -0.74;
/// Platas bredde i piksler (1 px = 1 cm); høyden følger teksten.
const W: f32 = 600.0;
const BOTTOM: f32 = 0.9;

const TITLE: &str = "Velkommen til Fristad";
const LEAD: &str = "Fristil sin dokumentasjon, som en by du kan gå rundt i.";
const FIND: [&str; 2] = [
    "Hver komponent i designsystemet står et sted i byen: knappen ved porten, oppslaget på husveggen, brødsmulene i brosteinen, skjemaet på søknadskontoret og rutetabellen i busskuret.",
    "Tårnene nord for porten viser skriftstørrelsene, Tokenparken i vest fargene og avstandene, og Verkstedet sør for Sørgata veiledningene.",
];
const PLAY: [&str; 3] = [
    "Gå med W, A, S og D, og se deg rundt med musa. Shift løper, mellomrom hopper, C huker og X kryper.",
    "Sikt på en komponent: E åpner dokumentasjonen med den ekte komponenten, R bytter variant og F bruker den.",
    "T bytter dag og natt, V været. Går du deg bort, tar H deg tilbake hit til torget.",
];

pub fn build(b: &mut Builder) {
    let p = b.on_ground(POS);
    let t = at(p, YAW);
    let thin = Mat::plain(0xffffff, 0.5).slot(slot::NEUTRAL_BORDER_SUBTLE).flags(crate::domain::geometry::flag::NO_DIRT | crate::domain::geometry::flag::NO_WEATHER);

    // Teksten først, med overkanten i y = 0, så plata kan få høyde etter den.
    let mut text = crate::domain::geometry::Mesh::default();
    let z = 0.03;
    let margin = 24.0;
    let left = -W * 0.5 + margin;
    let mut top = 0.0;
    top -= signs::paragraph(&mut text, b.font, &[TITLE], left, top, 32.0, 1.25, z, slot::NEUTRAL_TEXT_STRONG);
    top -= 4.0;
    let lead = signs::wrap(b.font, LEAD, 20.0, W - 2.0 * margin);
    let lead: Vec<&str> = lead.iter().map(String::as_str).collect();
    top -= signs::paragraph(&mut text, b.font, &lead, left, top, 20.0, 1.5, z, slot::NEUTRAL_TEXT);
    top -= 16.0;
    // En strek under ingressen, som `.fs-divider`.
    text.cuboid(&Affine3A::from_translation(Vec3::new(0.0, top * PX, z)), Vec3::new((W - 2.0 * margin) * PX, PX, 0.004), &thin);
    top -= 20.0;
    let gutter = 32.0;
    let col = (W - 2.0 * margin - gutter) * 0.5;
    let mut bottom = top;
    for (k, (heading, paras)) in [("Hva du finner", &FIND[..]), ("Slik spiller du", &PLAY[..])].iter().enumerate() {
        let x = left + k as f32 * (col + gutter);
        let mut y = top;
        y -= signs::paragraph(&mut text, b.font, &[heading], x, y, 18.0, 1.4, z, slot::NEUTRAL_TEXT_STRONG);
        y -= 4.0;
        for para in paras.iter() {
            let lines = signs::wrap(b.font, para, 16.0, col);
            let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
            y -= signs::paragraph(&mut text, b.font, &lines, x, y, 16.0, 1.5, z, slot::NEUTRAL_TEXT);
            y -= 8.0;
        }
        bottom = bottom.min(y + 8.0);
    }
    let h_px = -bottom + 2.0 * margin;
    let (w, h) = (W * PX, h_px * PX);
    let cy = BOTTOM + h * 0.5;

    let m = &mut b.mesh;
    // Stolpene står 60 cm ned i bakken, ramma og et lite skifertak over.
    for sx in [-1.0f32, 1.0] {
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(sx * (w * 0.5 - 0.2), (BOTTOM + h + 0.2 - 0.6) * 0.5, -0.1))), Vec3::new(0.16, BOTTOM + h + 0.2 + 0.6, 0.16), &OAK, 1.0);
    }
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, cy, 0.0))), Vec3::new(w + 0.24, h + 0.24, 0.08), &DARK_TRIM, 1.0);
    let slate = Mat::tex(Layer::Slate, 0xffffff);
    m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, BOTTOM + h + 0.2, 0.0)) * Affine3A::from_rotation_x(-0.35)), Vec3::new(w + 0.5, 0.05, 0.55), &slate, 1.0);

    // Plata: canvas med en fin kant, og teksten flyttet opp til overkanten.
    let mut board = crate::domain::geometry::Mesh::default();
    signs::boxed(&mut board, Vec3::ZERO, &Box2 { w: W, h: h_px, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
    text.transform(&Affine3A::from_translation(Vec3::new(0.0, (h_px * 0.5 - margin) * PX, 0.0)));
    board.append(&text);
    board.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, cy, 0.04))));
    b.mesh.append(&board);
    b.solid_box(p + Vec3::new(0.0, cy, 0.0), Vec3::new(w + 0.3, h + 0.3, 0.3), YAW);
    // Litt lys på plata om natta.
    b.light(t.transform_point3(Vec3::new(0.0, BOTTOM + h + 0.1, 0.6)), Vec3::new(1.0, 0.92, 0.8), 3.0, 5.0);
}
