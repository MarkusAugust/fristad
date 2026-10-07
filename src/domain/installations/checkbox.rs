//! `.fs-checkbox`: avkryssingsboksene på søknadsskjemaet. F krysser av
//! den første, R bytter tilstand (standard, ugyldig, deaktivert, ubestemt).

use super::form::{self, CHECK_ROWS, RIGHT};
use super::signs::{self, Box2};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 4] = ["default", "invalid", "disabled", "indeterminate"];
const SIZE: f32 = 20.0;

#[derive(Default)]
pub struct Checkbox {
    variant: usize,
    checked: bool,
    unchecked: usize,
    checked_box: usize,
    dash: usize,
}

/// Boksen: 20 px, 1 px kant, radius 4.
fn boxed(m: &mut Mesh, fill: u8, border: u8) {
    signs::boxed(m, Vec3::ZERO, &Box2 { w: SIZE, h: SIZE, radius: 4.0, fill, border: Some(border), depth: 0.02 });
}

/// Haken: to streker i aksentens innholdsfarge.
fn check(m: &mut Mesh, color: u8) {
    let mat = Mat::plain(0xffffff, 0.5).slot(color).flags(flag::NO_DIRT | flag::NO_WEATHER);
    // Kort strek ned mot høyre, lang strek opp mot høyre.
    m.cuboid(&(Affine3A::from_translation(Vec3::new(-3.6 * PX, -0.6 * PX, 0.025)) * Affine3A::from_rotation_z(-0.785)), Vec3::new(6.0 * PX, 2.2 * PX, 0.01), &mat);
    m.cuboid(&(Affine3A::from_translation(Vec3::new(1.8 * PX, 0.8 * PX, 0.025)) * Affine3A::from_rotation_z(0.785)), Vec3::new(10.5 * PX, 2.2 * PX, 0.01), &mat);
}

impl Installation for Checkbox {
    fn component(&self) -> Component {
        Component { name: "checkbox", class: ".fs-checkbox" }
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
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        form::office(b);
        let cx = RIGHT + SIZE * 0.5;
        // Den første raden: tre utgaver av boksen, én vises.
        let rest = form::on_sheet(cx, CHECK_ROWS[0] - 12.0, 0.035);
        let mut off = Mesh::default();
        boxed(&mut off, CHECK_FILL, CHECK_BORDER);
        self.unchecked = b.prop("checkbox", off, rest, Some((Vec3::ZERO, Vec3::new(SIZE * PX * 0.5 + 0.02, SIZE * PX * 0.5 + 0.02, 0.04))));
        let mut on = Mesh::default();
        boxed(&mut on, CHECK_ON, CHECK_ON);
        check(&mut on, CHECK_MARK);
        self.checked_box = b.prop("checkbox_checked", on, rest, None);
        let mut ind = Mesh::default();
        boxed(&mut ind, CHECK_ON, CHECK_ON);
        ind.cuboid(&Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.025)), Vec3::new(10.0 * PX, 2.2 * PX, 0.01), &Mat::plain(0xffffff, 0.5).slot(CHECK_MARK).flags(flag::NO_DIRT | flag::NO_WEATHER));
        self.dash = b.prop("checkbox_indeterminate", ind, rest, None);
        // Den andre raden står avkrysset.
        let mut row2 = Mesh::default();
        boxed(&mut row2, slot::ACCENT_FILL, slot::ACCENT_FILL);
        check(&mut row2, slot::ACCENT_CONTENT);
        row2.transform(&form::on_sheet(cx, CHECK_ROWS[1] - 12.0, 0.035));
        b.mesh.append(&row2);
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use && self.variant != 2 {
            self.checked = !self.checked;
            env.toasts.push(if self.checked { "Vilkårene er godtatt".into() } else { "Vilkårene er ikke godtatt".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        let indeterminate = self.variant == 3 && !self.checked;
        let show = |on: bool| if on { rest } else { rest * hidden };
        if prop == self.unchecked {
            return (show(!self.checked && !indeterminate), [0.0; 4]);
        }
        if prop == self.checked_box {
            return (show(self.checked), [0.0; 4]);
        }
        if prop == self.dash {
            return (show(indeterminate), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        let (fill, border, on, mark) = match VARIANTS[self.variant] {
            "invalid" => (c("neutral-canvas"), c("danger-text"), c("accent-fill"), c("accent-content")),
            "disabled" => (c("neutral-raised"), c("neutral-raised"), c("neutral-border"), c("neutral-canvas")),
            _ => (c("neutral-canvas"), c("neutral-border"), c("accent-fill"), c("accent-content")),
        };
        p[CHECK_FILL as usize] = fill;
        p[CHECK_BORDER as usize] = border;
        p[CHECK_ON as usize] = on;
        p[CHECK_MARK as usize] = mark;
    }
}

pub const CHECK_FILL: u8 = slot::CHECK_FILL;
pub const CHECK_BORDER: u8 = slot::CHECK_BORDER;
pub const CHECK_ON: u8 = slot::CHECK_ON;
pub const CHECK_MARK: u8 = slot::CHECK_MARK;
