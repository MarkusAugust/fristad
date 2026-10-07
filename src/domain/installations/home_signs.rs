//! «Torget»-skiltene og veiviserne ut av byen: lenker (`.fs-link`) på
//! stolper. F på et «Torget»-skilt tar deg med flytur tilbake til torget;
//! veiviserne til Linelord, Cobble og Streamlord peker ut i landskapet.
//! Lenka har de samme tre tilstandene som veiviseren på torget (link.rs):
//! vanlig, besøkt og deaktivert, rendret fra Fristils CSS som egne ark.

use super::link::VARIANTS;
use super::signs;
use super::{Action, Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, OAK, SPAWN, SPAWN_YAW};
use crate::domain::geometry::Mesh;
use crate::domain::palette::slot;
use glam::{Affine3A, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};

/// Skiltene: hvor de står, og hvilken vei de vender (yaw). De står i
/// veikanten, aldri i veibanen, og vender mot dem som kommer fra stedet.
const SIGNS: [(Vec3, f32); 5] = [
    // Ved Tårnene, vendt mot tårnplassen.
    (Vec3::new(6.5, 0.0, -76.0), PI),
    // Ved porten til Tokenparken, sør for veien, vendt inn i parken.
    (Vec3::new(-41.5, 0.0, 5.0), -FRAC_PI_2 - 0.15),
    // Ved Verkstedet.
    (Vec3::new(10.6, 0.0, 71.0), 0.0),
    // Østre gate, sør for veien, vendt mot søknadskontoret og busskuret.
    (Vec3::new(30.4, 0.0, 4.6), FRAC_PI_2),
    // Infotavlene i vest, sør for veien rett utenfor husrekka.
    (Vec3::new(-31.5, 0.0, 5.0), -FRAC_PI_2 - 0.15),
];

/// Veiviserne ut av byen: sted, hvor stolpen står og hvilken vei den vender.
/// Som veiskilt flest står de i høyre veikant for den som forlater byen, et
/// stykke forbi siste hus, og vender mot byen med en liten dreining inn mot
/// veien. Avstanden står på arket (`tools/sheets.mjs`).
fn places() -> [(&'static str, Vec3, f32); 3] {
    use crate::domain::city::road_east_z;
    [
        // Nordover: gata er brostein til z −60, så grus. Høyre side er øst.
        ("Linelord", Vec3::new(5.2, 0.0, -62.0), -0.15),
        // Østover: forbi skjelett-huset, der grusveien begynner å slynge seg.
        // Høyre side er sør.
        ("Cobble", Vec3::new(55.0, 0.0, road_east_z(55.0) + 5.0), -FRAC_PI_2 - 0.15),
        // Vestover: ved parkporten, nord for veien (høyre side vestover).
        ("Streamlord", Vec3::new(-41.5, 0.0, -5.0), FRAC_PI_2 - 0.15),
    ]
}

#[derive(Default)]
pub struct HomeSigns {
    variant: usize,
    /// Én prop per variant for hvert skilt; posen gjemmer de andre.
    arms: Vec<[usize; 3]>,
    places: Vec<[usize; 3]>,
}

impl HomeSigns {
    fn hidden() -> Affine3A {
        Affine3A::from_scale(Vec3::splat(0.0001))
    }
}

impl Installation for HomeSigns {
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
    fn usable(&self) -> bool {
        true
    }

    fn build(&mut self, b: &mut Builder) {
        for (pos, yaw) in SIGNS {
            let p = b.on_ground(pos);
            let t = at(p, yaw);
            b.mesh.cylinder(&Affine3A::from_translation(p - Vec3::Y * 0.4), 0.06, 0.05, 2.7, 8, &OAK, true);
            b.solid_box(p + Vec3::Y * 1.2, Vec3::new(0.2, 2.4, 0.2), 0.0);
            // Plakaten: lenka «Torget» fra arket, og avstanden under i
            // dempet tekst, på en canvasplate.
            let dist = ((SPAWN.x - pos.x).powi(2) + (SPAWN.z - pos.z).powi(2)).sqrt();
            let sub = format!("{} m, F for å fly dit", (dist / 10.0).round() as i32 * 10);
            let mut ids = [0usize; 3];
            for (k, v) in VARIANTS.iter().enumerate() {
                let mut arm = Mesh::default();
                let (lw, lh) = signs::sheet(&mut arm, b.sheets, &format!("link:{v}:Torget"), Vec3::new(0.0, 14.0 * PX, 0.0), 0.03, 2.0, slot::NEUTRAL_CANVAS);
                let w = (signs::width(b.font, &sub, 12.0) / PX).max(lw) + 32.0;
                let h = lh + 16.0 + 12.0 + 24.0;
                signs::boxed(&mut arm, Vec3::new(0.0, 0.0, -0.02), &signs::Box2 { w, h, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
                signs::text(&mut arm, b.font, &sub, Vec3::new(0.0, -h * 0.5 * PX + 16.0 * PX, 0.0), 12.0, slot::NEUTRAL_TEXT_SUBTLE);
                let rest = t * Affine3A::from_translation(Vec3::new(0.0, 1.9, 0.08));
                ids[k] = b.prop("link", arm, rest, Some((Vec3::ZERO, Vec3::new(w * PX * 0.5, h * PX * 0.5, 0.05))));
            }
            self.arms.push(ids);
        }
        // Veiviserne ut av byen: lenka med avstanden, rendret fra CSS-en,
        // på en stolpe der gata går over i grusveien.
        for (name, pos, yaw) in places() {
            let p = b.on_ground(pos);
            let t = at(p, yaw);
            b.mesh.cylinder(&Affine3A::from_translation(p - Vec3::Y * 0.4), 0.06, 0.05, 2.6, 8, &OAK, true);
            b.solid_box(p + Vec3::Y * 1.2, Vec3::new(0.2, 2.4, 0.2), 0.0);
            let mut ids = [0usize; 3];
            for (k, v) in VARIANTS.iter().enumerate() {
                let mut m = Mesh::default();
                let (w, h) = signs::sheet(&mut m, b.sheets, &format!("place:{name}:{v}"), Vec3::ZERO, 0.03, 4.0, slot::NEUTRAL_CANVAS);
                // Plata rundt arket, i canvas, litt større.
                signs::boxed(&mut m, Vec3::new(0.0, 0.0, -0.02), &signs::Box2 { w: w + 24.0, h: h + 16.0, radius: 4.0, fill: slot::NEUTRAL_CANVAS, border: Some(slot::NEUTRAL_BORDER_SUBTLE), depth: 0.03 });
                let rest = t * Affine3A::from_translation(Vec3::new(0.0, 2.0, 0.08));
                ids[k] = b.prop("link", m, rest, Some((Vec3::ZERO, Vec3::new((w + 24.0) * PX * 0.5, (h + 16.0) * PX * 0.5, 0.05))));
            }
            self.places.push(ids);
        }
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        let aimed = |ids: &[usize; 3]| ids.iter().any(|a| Some(*a) == env.aimed_prop);
        let place = self.places.iter().position(aimed);
        let arm = self.arms.iter().position(aimed);
        if place.is_none() && arm.is_none() {
            return;
        }
        // Deaktivert lenke: som i nettleseren gjør den ingenting.
        if VARIANTS[self.variant] == "disabled" {
            env.toasts.push("Lenka er deaktivert".into());
            return;
        }
        if let Some(k) = place {
            env.toasts.push(format!("{} ligger langt der ute. Veien dit er ikke bygd ennå.", places()[k].0));
            return;
        }
        env.teleport = Some((SPAWN, SPAWN_YAW));
        env.toasts.push("Flyr til Torget".into());
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        let shown = self.arms.iter().chain(self.places.iter()).any(|ids| ids[self.variant] == prop);
        (if shown { rest } else { Self::hidden() }, [0.0; 4])
    }
}
