//! `.fs-radio`: radioknappene på søknadsskjemaet, i en fieldset med legend.
//! F velger neste svaralternativ (det tredje er deaktivert), R bytter
//! tilstand (standard, ugyldig). Knappene er ark fra Fristils CSS, ett per
//! tilstand og valgt/uvalgt.

use super::form::{self, RADIO_ROWS, RIGHT};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

pub const VARIANTS: [&str; 2] = ["default", "invalid"];
const SIZE: f32 = 20.0;

#[derive(Default)]
pub struct Radio {
    variant: usize,
    selected: usize,
    /// [rad][variant][av, på]
    rings: [[[usize; 2]; 2]; 2],
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
            for (v, name) in VARIANTS.iter().enumerate() {
                for (s, state) in ["off", "on"].iter().enumerate() {
                    let mut m = Mesh::default();
                    let (w, h) = signs::sheet(&mut m, b.sheets, &format!("radio:{name}:{state}"), Vec3::ZERO, 0.02, 999.0, slot::NEUTRAL_CANVAS);
                    self.rings[k][v][s] = b.prop("radio", m, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5 + 0.02, h * PX * 0.5 + 0.02, 0.04))));
                }
            }
        }
        // Det tredje alternativet er deaktivert: en kulisse.
        let mut dis = Mesh::default();
        signs::sheet(&mut dis, b.sheets, "radio:disabled:off", Vec3::ZERO, 0.02, 999.0, slot::NEUTRAL_RAISED);
        dis.transform(&form::on_sheet(cx, RADIO_ROWS[2] - 12.0, 0.04));
        b.mesh.append(&dis);
    }

    fn tick(&mut self, env: &mut Env) {
        env.form.delivery = self.selected;
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
            for v in 0..2 {
                for s in 0..2 {
                    if prop == self.rings[k][v][s] {
                        let shown = v == self.variant && (s == 1) == (k == self.selected);
                        return (if shown { rest } else { rest * hidden }, [0.0; 4]);
                    }
                }
            }
        }
        (rest, [0.0; 4])
    }
}
