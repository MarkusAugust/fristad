//! `.fs-link`: veiviseren på torget, med armer som peker dit lenkene går.
//! F på en arm følger lenka: du teleporteres dit. Den første armen bytter
//! tilstand med R (vanlig, besøkt, deaktivert).

use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 3] = ["default", "visited", "disabled"];
const POS: Vec3 = Vec3::new(-7.4, 0.0, 12.4);
/// Armene: tekst, retning (yaw, dit armen peker) og høyde.
const ARMS: [(&str, f32, f32); 3] = [("Mine søknader", 0.05, 2.45), ("Torget", 0.53, 2.1), ("Dokumentasjonen", -1.32, 1.75)];
/// Dit lenkene går: føttene, og retningen du ser i når du kommer fram.
const TARGETS: [(Vec3, f32); 3] = [
    (Vec3::new(36.0, 0.0, 5.0), std::f32::consts::PI),
    (Vec3::new(0.0, 0.0, 8.5), 0.0),
    (Vec3::new(9.0, 0.0, 76.0), -std::f32::consts::FRAC_PI_2),
];

#[derive(Default)]
pub struct Link {
    variant: usize,
    arms: [usize; 3],
}

impl Installation for Link {
    fn component(&self) -> Component {
        Component { name: "link", class: ".fs-link" }
    }
    fn variants(&self) -> &'static [&'static str] {
        &VARIANTS
    }
    fn variant(&self) -> usize {
        self.variant
    }
    fn set_variant(&mut self, i: usize) {
        self.variant = i % VARIANTS.len();
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(POS);
        let m = &mut b.mesh;
        m.cylinder(&Affine3A::from_translation(p), 0.07, 0.06, 2.7, 10, &OAK, true);
        m.sphere(&Affine3A::from_translation(p + Vec3::Y * 2.75), Vec3::splat(0.09), 5, 8, &OAK);
        b.solid_box(p + Vec3::Y * 1.3, Vec3::new(0.3, 2.6, 0.3), 0.0);
        for (k, (label, yaw, y)) in ARMS.iter().enumerate() {
            let t = at(p + Vec3::Y * *y, *yaw);
            let tw = signs::width(b.font, label, 16.0) / PX;
            let w = tw + 40.0;
            // Armen: en planke med spiss ende, teksten på begge sider.
            let mut arm = Mesh::default();
            let plank = Mat::plain(0xf2efe8, 0.6).flags(flag::NO_DIRT);
            arm.cuboid(&Affine3A::from_translation(Vec3::new(w * PX * 0.5, 0.0, 0.0)), Vec3::new(w * PX, 0.22, 0.03), &plank);
            arm.cuboid(&(Affine3A::from_translation(Vec3::new(w * PX + 0.078, 0.0, 0.0)) * Affine3A::from_rotation_z(0.785)), Vec3::new(0.155, 0.155, 0.03), &plank);
            let (text, _) = if k == 0 { (LINK_TEXT, LINK_LINE) } else if k == 1 { (slot::VISITED_TEXT, slot::VISITED_TEXT) } else { (slot::ACCENT_TEXT, slot::ACCENT_TEXT) };
            let line = if k == 0 { LINK_LINE } else { text };
            for side in [1.0f32, -1.0] {
                // Teksten og understreken (0,08 em tykk, 0,14 em under
                // grunnlinja) på hver side av planken; baksiden speiles.
                let mut txt = Mesh::default();
                signs::text(&mut txt, b.font, label, Vec3::new(0.0, 0.0, 0.016), 16.0, text);
                txt.cuboid(&Affine3A::from_translation(Vec3::new(0.0, (-0.36 * 16.0 - 0.14 * 16.0 - 0.6) * PX, 0.016)), Vec3::new(tw * PX, 1.3 * PX, 0.004), &Mat::plain(0xffffff, 0.5).slot(line).flags(flag::NO_DIRT | flag::NO_WEATHER));
                if side < 0.0 {
                    txt.transform(&Affine3A::from_rotation_y(std::f32::consts::PI));
                }
                txt.transform(&Affine3A::from_translation(Vec3::new(w * PX * 0.5, 0.0, 0.0)));
                arm.append(&txt);
            }
            arm.transform(&(t * Affine3A::from_translation(Vec3::new(0.06, 0.0, 0.0))));
            let collider = Some((t.transform_point3(Vec3::new(0.06 + w * PX * 0.5, 0.0, 0.0)), Vec3::new(w * PX * 0.5, 0.12, 0.05)));
            self.arms[k] = b.prop("link", arm, Affine3A::IDENTITY, collider);
        }
    }

    fn usable(&self) -> bool {
        true
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        let Some(k) = self.arms.iter().position(|a| Some(*a) == env.aimed_prop) else { return };
        if k == 0 && VARIANTS[self.variant] == "disabled" {
            env.toasts.push("Lenka er deaktivert".into());
            return;
        }
        let (feet, yaw) = TARGETS[k];
        env.teleport = Some((feet, yaw));
        env.toasts.push(format!("Fulgte lenka «{}»", ARMS[k].0));
    }

    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        let (text, line) = match VARIANTS[self.variant] {
            "visited" => (c("visited-text"), c("visited-text")),
            // Deaktivert: dempet tekst uten understrek (streken får platas farge).
            "disabled" => (c("neutral-text-subtle"), crate::domain::tokens::hex("#f2efe8")),
            _ => (c("accent-text"), c("accent-text")),
        };
        p[LINK_TEXT as usize] = text;
        p[LINK_LINE as usize] = line;
        p[slot::VISITED_TEXT as usize] = c("visited-text");
    }
}

pub const LINK_TEXT: u8 = slot::LINK_TEXT;
pub const LINK_LINE: u8 = slot::LINK_LINE;
