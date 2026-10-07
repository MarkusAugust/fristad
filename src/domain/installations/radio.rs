//! `.fs-radio`: radioknappene på søknadsskjemaet, i en fieldset med legend.
//! F velger neste svaralternativ (det tredje er deaktivert), R bytter
//! tilstand (standard, ugyldig).

use super::form::{self, RADIO_ROWS, RIGHT};
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["default", "invalid"];
const SIZE: f32 = 20.0;

#[derive(Default)]
pub struct Radio {
    variant: usize,
    selected: usize,
    off: [usize; 2],
    on: [usize; 2],
}

/// Ringen: en sirkel med 1 px kant.
fn ring(m: &mut Mesh, fill: u8, border: u8, dot: Option<u8>) {
    let r = Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let bm = Mat::plain(0xffffff, 0.5).slot(border).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let fm = Mat::plain(0xffffff, 0.5).slot(fill).flags(flag::NO_DIRT | flag::NO_WEATHER);
    m.cylinder(&(Affine3A::from_translation(Vec3::new(0.0, 0.0, -0.01)) * r), SIZE * 0.5 * PX, SIZE * 0.5 * PX, 0.02, 24, &bm, true);
    m.cylinder(&(Affine3A::from_translation(Vec3::new(0.0, 0.0, -0.006)) * r), (SIZE * 0.5 - 1.0) * PX, (SIZE * 0.5 - 1.0) * PX, 0.02, 24, &fm, true);
    if let Some(d) = dot {
        let dm = Mat::plain(0xffffff, 0.5).slot(d).flags(flag::NO_DIRT | flag::NO_WEATHER);
        m.cylinder(&(Affine3A::from_translation(Vec3::new(0.0, 0.0, -0.002)) * r), SIZE * 0.32 * PX, SIZE * 0.32 * PX, 0.02, 20, &dm, true);
    }
}

impl Installation for Radio {
    fn component(&self) -> Component {
        Component { name: "radio", class: ".fs-radio" }
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
        for k in 0..2 {
            let rest = form::on_sheet(cx, RADIO_ROWS[k] - 12.0, 0.04);
            let mut off = Mesh::default();
            ring(&mut off, slot::NEUTRAL_CANVAS, RADIO_BORDER, None);
            let collider = if k == 0 { Some((Vec3::ZERO, Vec3::new(SIZE * PX * 0.5 + 0.02, SIZE * PX * 0.5 + 0.02, 0.04))) } else { None };
            self.off[k] = b.prop("radio", off, rest, collider);
            let mut on = Mesh::default();
            ring(&mut on, slot::ACCENT_FILL, slot::ACCENT_FILL, Some(slot::ACCENT_CONTENT));
            self.on[k] = b.prop("radio_checked", on, rest, None);
        }
        // Det tredje alternativet er deaktivert.
        let mut dis = Mesh::default();
        ring(&mut dis, slot::NEUTRAL_RAISED, slot::NEUTRAL_RAISED, None);
        dis.transform(&form::on_sheet(cx, RADIO_ROWS[2] - 12.0, 0.04));
        b.mesh.append(&dis);
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action == Action::Use {
            self.selected = (self.selected + 1) % 2;
            env.toasts.push(if self.selected == 0 { "Svar i digital postkasse".into() } else { "Svar som brev i posten".into() });
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
        for k in 0..2 {
            if prop == self.off[k] {
                return (if k == self.selected { rest * hidden } else { rest }, [0.0; 4]);
            }
            if prop == self.on[k] {
                return (if k == self.selected { rest } else { rest * hidden }, [0.0; 4]);
            }
        }
        (rest, [0.0; 4])
    }

    fn palette(&self, tokens: &Tokens, night: f32, p: &mut [Vec3; PALETTE]) {
        let c = |n: &str| tokens.blend(n, night);
        p[RADIO_BORDER as usize] = if VARIANTS[self.variant] == "invalid" { c("danger-text") } else { c("neutral-border") };
    }
}

pub const RADIO_BORDER: u8 = slot::RADIO_BORDER;
