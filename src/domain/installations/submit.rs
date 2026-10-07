//! «Send søknad»: sendeknappen (`.fs-button`) på søknadsskjemaet. F
//! validerer feltene: e-post med krøllalfa, valgt fylke, begrunnelse på
//! minst ti tegn og godtatte vilkår. Feilmeldingene står under feltene, og
//! feltene får ugyldig- eller suksesstilstand fortløpende etterpå.

use super::form::{self, BUTTON_Y, RIGHT};
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};

#[derive(Default)]
pub struct Submit {
    button: usize,
    sent: usize,
}

impl Installation for Submit {
    fn component(&self) -> Component {
        Component { name: "button", class: ".fs-button" }
    }
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        form::office(b);
        let bw = signs::width(b.font, "Send søknad", 16.0) / PX + 34.0;
        let rest = form::on_sheet(RIGHT + bw * 0.5, BUTTON_Y, 0.035);
        let mut m = Mesh::default();
        signs::button(&mut m, b.font, "Send søknad", Vec3::ZERO, slot::ACCENT_FILL, slot::ACCENT_FILL, slot::ACCENT_CONTENT);
        self.button = b.prop("submit", m, rest, Some((Vec3::ZERO, Vec3::new(bw * PX * 0.5, 21.0 * PX, 0.04))));
        // Kvitteringen ved siden av knappen når søknaden er sendt.
        let mut s = Mesh::default();
        signs::text_left(&mut s, b.font, "Søknaden er sendt", Vec3::new((bw * 0.5 + 12.0) * PX, -0.36 * 14.0 * PX, 0.025), 14.0, slot::SUCCESS_TEXT);
        self.sent = b.prop("submit_sent", s, rest, None);
    }

    fn tick(&mut self, env: &mut Env) {
        if env.form.submitted {
            env.form.validate();
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        env.form.submitted = true;
        if env.form.validate() {
            env.form.sent = true;
            let how = ["i digital postkasse", "som brev i posten", "på kontoret"][env.form.delivery.min(2)];
            env.toasts.push(format!("Søknaden er sendt. Svaret kommer {how}."));
        } else {
            let n = env.form.errors.iter().filter(|e| **e).count();
            env.toasts.push(format!("Søknaden har {n} feil. Se meldingene under feltene."));
        }
    }

    fn pose(&self, prop: usize, rest: Affine3A, env: &Env) -> (Affine3A, [f32; 4]) {
        if prop == self.sent {
            let hidden = Affine3A::from_scale(Vec3::splat(0.0001));
            return (if env.form.sent { rest } else { rest * hidden }, [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}
