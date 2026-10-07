//! `.fs-dialog`: telefonkiosken på torget. F åpner døra, og da står
//! dialogen der inne med tittel, tekst og knapperad, med en mørk flate bak
//! (`::backdrop`). Lukk den igjen med F, som Escape.

use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, IRON};
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

#[derive(Default)]
pub struct Dialog {
    open: bool,
    t: f32,
    door: usize,
    panel: usize,
    backdrop: usize,
}

const POS: Vec3 = Vec3::new(-14.6, 0.0, 7.0);
/// Kiosken vender mot torget (østover).
const YAW: f32 = std::f32::consts::FRAC_PI_2;
const W: f32 = 3.2;
const D: f32 = 1.7;
const H: f32 = 2.9;

impl Installation for Dialog {
    fn component(&self) -> Component {
        Component { name: "dialog", class: ".fs-dialog" }
    }

    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        let p = b.on_ground(POS);
        let t = at(p, YAW);
        let red = Mat::plain(0xb3202b, 0.4).flags(flag::NO_DIRT);
        let glass = Mat::plain(0x9fb5c0, 0.05).flags(flag::NO_DIRT | flag::NO_WEATHER);
        let m = &mut b.mesh;
        // Sokkel, hjørnestolper, tak med kuppel.
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.1, 0.0))), Vec3::new(W + 0.1, 0.2, D + 0.1), &red, 1.0);
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(sx * (W * 0.5 - 0.06), H * 0.5, sz * (D * 0.5 - 0.06)))), Vec3::new(0.12, H, 0.12), &red, 1.0);
        }
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, H + 0.08, 0.0))), Vec3::new(W + 0.2, 0.16, D + 0.2), &red, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, H + 0.25, 0.0))), Vec3::new(W - 0.3, 0.18, D - 0.3), &red, 1.0);
        m.sphere(&(t * Affine3A::from_translation(Vec3::new(0.0, H + 0.34, 0.0))), Vec3::new(W * 0.32, 0.3, D * 0.32), 6, 14, &red);
        // Skiltet «Telefon» over døra, og sprosser i glasset.
        let plate = Mat::plain(0x2a2a2e, 0.5).flags(flag::NO_DIRT);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(0.0, H - 0.22, D * 0.5 + 0.02))), Vec3::new(W - 0.3, 0.34, 0.04), &plate, 1.0);
        let mut label = Mesh::default();
        b.font.centered(&mut label, "TELEFON", Vec3::ZERO, 20.0, 0.012, &Mat::plain(0xf4efe4, 0.5).flags(flag::GLOW_ALWAYS));
        label.transform(&(t * Affine3A::from_translation(Vec3::new(0.0, H - 0.22, D * 0.5 + 0.045))));
        m.append(&label);
        // Sidene og bakveggen i glass, med tverrsprosser i rødt.
        for side in [-1.0f32, 1.0] {
            m.cuboid(&(t * Affine3A::from_translation(Vec3::new(side * (W * 0.5 - 0.02), 0.2 + (H - 0.5) * 0.5, 0.0))), Vec3::new(0.02, H - 0.5, D - 0.3), &glass);
            for y in [1.0, 1.9] {
                m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(side * (W * 0.5 - 0.03), y, 0.0))), Vec3::new(0.06, 0.05, D - 0.2), &red, 1.0);
            }
        }
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(0.0, 0.2 + (H - 0.5) * 0.5, -D * 0.5 + 0.02))), Vec3::new(W - 0.3, H - 0.5, 0.02), &glass);
        // Fronten: fast glassfelt til venstre, døra til høyre.
        let door_w = 1.1;
        let fixed_w = W - 0.24 - door_w;
        m.cuboid(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.12 + fixed_w * 0.5, 0.2 + (H - 0.5) * 0.5, D * 0.5 - 0.02))), Vec3::new(fixed_w, H - 0.5, 0.02), &glass);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.12 + fixed_w, H * 0.5, D * 0.5 - 0.05))), Vec3::new(0.08, H, 0.1), &red, 1.0);
        // Telefonen på venstre sidevegg: apparat og rør på gaffel.
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.15, 1.45, -0.2))), Vec3::new(0.14, 0.42, 0.28), &plate, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.3, 1.6, -0.2))), Vec3::new(0.08, 0.06, 0.24), &plate, 1.0);
        m.cylinder(&(t * Affine3A::from_rotation_translation(glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), Vec3::new(-W * 0.5 + 0.32, 1.66, -0.32))), 0.035, 0.035, 0.24, 8, &plate, true);
        for dz in [0.0, 0.24] {
            m.sphere(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.32, 1.66, -0.32 + dz))), Vec3::splat(0.05), 5, 8, &plate);
        }
        // Mynttelefonens hylle og en telefonkatalog.
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.25, 1.05, -0.2))), Vec3::new(0.36, 0.03, 0.5), &plate, 1.0);
        m.cuboid_ao(&(t * Affine3A::from_translation(Vec3::new(-W * 0.5 + 0.25, 1.09, -0.15))), Vec3::new(0.22, 0.05, 0.3), &Mat::plain(0xd9c27a, 0.7), 1.0);
        // Lys i taket.
        b.light(p + glam::Quat::from_rotation_y(YAW) * Vec3::new(0.0, H - 0.2, 0.0), Vec3::new(1.0, 0.9, 0.75), 2.5, 5.0);

        // Døra: ramme i rødt med glass, hengslet i høyre stolpe.
        let hinge = Vec3::new(W * 0.5 - 0.12, 0.0, D * 0.5 - 0.05);
        let mut door = Mesh::default();
        door.cuboid_ao(&Affine3A::from_translation(Vec3::new(-door_w * 0.5, H * 0.5, 0.0)), Vec3::new(door_w, H - 0.1, 0.05), &glass, 1.0);
        for y in [0.3, 1.0, 1.9, H - 0.3] {
            door.cuboid_ao(&Affine3A::from_translation(Vec3::new(-door_w * 0.5, y, 0.0)), Vec3::new(door_w, 0.06, 0.08), &red, 1.0);
        }
        for x in [0.0, -door_w] {
            door.cuboid_ao(&Affine3A::from_translation(Vec3::new(x, H * 0.5, 0.0)), Vec3::new(0.08, H - 0.1, 0.08), &red, 1.0);
        }
        door.cuboid_ao(&Affine3A::from_translation(Vec3::new(-door_w + 0.15, 1.1, 0.06)), Vec3::new(0.03, 0.22, 0.04), &IRON, 1.0);
        self.door = b.prop("dialog_door", door, t * Affine3A::from_translation(hinge), Some((Vec3::new(-door_w * 0.5, H * 0.5, 0.0), Vec3::new(door_w * 0.5, H * 0.5, 0.05))));

        // Bakteppet: flaten bak dialogen, i overlegg-fargen.
        let mut back = Mesh::default();
        back.cuboid(&Affine3A::IDENTITY, Vec3::new(W - 0.32, H - 0.52, 0.01), &Mat::plain(0xffffff, 0.8).slot(DIALOG_OVERLAY).flags(flag::NO_DIRT | flag::NO_WEATHER));
        self.backdrop = b.prop("dialog_backdrop", back, t * Affine3A::from_translation(Vec3::new(0.0, 0.2 + (H - 0.5) * 0.5, -D * 0.5 + 0.05)), None);

        // Dialogen: 280 px bred, padding 20, tittel 20 px, tekst 16 px, knapperad.
        let mut panel = Mesh::default();
        let w = 280.0;
        let pad = 20.0;
        let title_h = 20.0 * 1.25;
        let body_h = 3.0 * 16.0 * 1.5;
        let h = pad + title_h + 8.0 + body_h + 16.0 + 42.0 + pad;
        signs::boxed(&mut panel, Vec3::ZERO, &Box2 { w, h, radius: 8.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
        let left = -w * 0.5 + pad;
        let mut top = h * 0.5 - pad;
        signs::paragraph(&mut panel, b.font, &["Slette søknaden?"], left, top, 20.0, 1.25, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= title_h + 8.0;
        signs::paragraph(&mut panel, b.font, &["Søknaden og vedleggene blir", "borte. Dette kan ikke", "angres."], left, top, 16.0, 1.5, 0.03, slot::NEUTRAL_TEXT_STRONG);
        top -= body_h + 16.0;
        // Knappene høyrejustert: «Avbryt» (secondary) og «Slett søknaden» (danger).
        let cy = top - 21.0;
        let danger_w = signs::width(b.font, "Slett søknaden", 16.0) / PX + 34.0;
        let right = w * 0.5 - pad;
        signs::button(&mut panel, b.font, "Slett søknaden", Vec3::new((right - danger_w * 0.5) * PX, cy * PX, 0.03), slot::DANGER_SURFACE, slot::DANGER_TEXT, slot::DANGER_TEXT);
        let cancel_w = signs::width(b.font, "Avbryt", 16.0) / PX + 34.0;
        signs::button(&mut panel, b.font, "Avbryt", Vec3::new((right - danger_w - 8.0 - cancel_w * 0.5) * PX, cy * PX, 0.03), slot::NEUTRAL_CANVAS, slot::ACCENT_BORDER, slot::ACCENT_TEXT);
        self.panel = b.prop("dialog", panel, t * Affine3A::from_translation(Vec3::new(0.0, 1.55, -D * 0.5 + 0.12)), Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.04))));

        // Kolliderne: sokkel og vegger, ikke døråpningen.
        b.solid_box(p + Vec3::Y * 0.1, Vec3::new(D + 0.1, 0.2, W + 0.1), YAW);
        let q = glam::Quat::from_rotation_y(YAW);
        b.solid_box(p + q * Vec3::new(0.0, H * 0.5, -D * 0.5), Vec3::new(W, H, 0.1), YAW);
        for side in [-1.0f32, 1.0] {
            b.solid_box(p + q * Vec3::new(side * W * 0.5, H * 0.5, 0.0), Vec3::new(0.1, H, D), YAW);
        }
        b.solid_box(p + q * Vec3::new(-W * 0.5 + 0.12 + fixed_w * 0.5, H * 0.5, D * 0.5), Vec3::new(fixed_w, H, 0.1), YAW);
    }

    fn tick(&mut self, env: &mut Env) {
        let goal = if self.open { 1.0 } else { 0.0 };
        self.t += (goal - self.t) * (env.dt * 5.0).min(1.0);
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            self.open = !self.open;
            env.toasts.push(if self.open { "Dialogen er åpen: showModal()".into() } else { "Dialogen er lukket".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let ease = self.t * self.t * (3.0 - 2.0 * self.t);
        if prop == self.door {
            return (rest * Affine3A::from_rotation_y(ease * 1.9), [0.0; 4]);
        }
        if prop == self.panel || prop == self.backdrop {
            // Dialogen og bakteppet finnes bare mens den er åpen.
            let s = if self.t > 0.5 { 1.0 } else { 0.0001 };
            return (rest * Affine3A::from_scale(Vec3::splat(s)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &crate::domain::tokens::Tokens, night: f32, p: &mut [Vec3; crate::domain::scene::PALETTE]) {
        // `--fs-color-overlay` er halvgjennomsiktig svart; her blandes den
        // mot kioskens mørke interiør.
        let _ = tokens;
        p[DIALOG_OVERLAY as usize] = Vec3::splat(0.12 - night * 0.06);
    }
}

pub const DIALOG_OVERLAY: u8 = slot::DIALOG_OVERLAY;
