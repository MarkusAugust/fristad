//! Innbyggerne som props: en usynlig kapsel å sikte på ved hver av dem, så
//! F snakker (`Env.talk_to`). Selve figurene poseres av spillet
//! (`domain/npcs.rs`); her bestemmes hvor de står og hva de sier.

use super::{Action, Component, Env, Installation};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use glam::{Affine3A, Vec3};

/// Hvem som står hvor: navn, føtter (y 0 er bakken), yaw, tunikafarge og
/// replikker. Rollene står ved dørene til butikkhusene rundt torget når
/// husene har dem; ellers på faste steder.
pub struct Cast {
    pub name: &'static str,
    pub pos: Vec3,
    pub yaw: f32,
    pub tint: Vec3,
    pub lines: &'static [&'static str],
}

const SMITH: &[&str] = &[
    "Knappesmeden, til tjeneste. Jeg smir det du bærer opp fra kjelleren.",
    "Et sverd trenger en klinge og et grep: en divider og en knapp. Resten er håndlag.",
    "Kom tilbake med komponenter, så får vi se hva stålet duger til.",
];
const CLERK: &[&str] = &[
    "Skjemakontoret. Ingenting smis uten et korrekt utfylt skjema.",
    "Oppskriftene er skjemaer. Fyll ut feltene med komponentene du har vunnet, og send.",
    "Avansert utstyr krever avanserte skjemaer: fieldset, file-upload, search.",
];
const APOTHECARY: &[&str] = &[
    "Apoteket. Drikker mot det meste, tokens mot alt.",
    "Les dokumentasjonen i boka når du er skadet. Kunnskap leger.",
    "Hold deg unna lavaen nederst. Den er ikke i noen token.",
];
const GORVEK: &[&str] = &[
    "Gorvek. Jeg lærer deg trekk, om du har tokens og tålmodighet.",
    "Blokk i det bittet kommer. Da står de lamslått, og du hugger dobbelt.",
    "Unna med Shift. Det koster pust, men sparer blod.",
];

#[derive(Default)]
pub struct Npcs {
    props: Vec<usize>,
    pub cast: Vec<Cast>,
    aimed: Option<usize>,
}

impl Npcs {
    /// Rollene med plass: ved butikkdørene rundt torget, og Gorvek ved huset sitt.
    fn place(b: &Builder) -> Vec<Cast> {
        let mut out = vec![];
        let shops: Vec<_> = b.facades.iter().filter(|f| f.shop).collect();
        // Butikkfasadene kommer i rekkefølgen fra `house::SHOPS`: Bakeriet,
        // Smia, Apoteket, Bokhandelen …; smeden står ved Smia, apotekeren
        // ved Apoteket og skjemakontoristen ved Bokhandelen.
        let roles: [(&'static str, usize, Vec3, &'static [&'static str]); 3] = [
            ("Knappesmeden", 1, Vec3::new(0.22, 0.1, 0.05), SMITH),
            ("Skjemakontoret", 3, Vec3::new(0.06, 0.12, 0.42), CLERK),
            ("Apotekeren", 2, Vec3::new(0.06, 0.28, 0.1), APOTHECARY),
        ];
        let fallback = [Vec3::new(6.0, 0.0, -16.0), Vec3::new(-14.0, 0.0, 14.0), Vec3::new(15.0, 0.0, 12.0)];
        for (i, (name, shop, tint, lines)) in roles.into_iter().enumerate() {
            let (pos, yaw) = match shops.get(shop) {
                Some(f) => {
                    // Et skritt ut fra døra, litt til sida, vendt mot gata.
                    let p = f.origin + f.along * (f.door + 1.1) + f.facing * 1.2;
                    (b.on_ground(Vec3::new(p.x, 0.0, p.z)), f.facing.x.atan2(f.facing.z))
                }
                None => {
                    let p = fallback[i];
                    (b.on_ground(p), (-p.x).atan2(-p.z))
                }
            };
            out.push(Cast { name, pos, yaw, tint, lines });
        }
        // Gorvek utenfor huset sitt, vendt mot veien.
        let g = b.on_ground(Vec3::new(13.0, 0.0, -71.5));
        out.push(Cast { name: "Gorvek", pos: g, yaw: std::f32::consts::FRAC_PI_2, tint: Vec3::new(0.3, 0.1, 0.04), lines: GORVEK });
        out
    }
}

impl Installation for Npcs {
    fn component(&self) -> Component {
        Component { name: "npc", class: "" }
    }
    fn usable(&self) -> bool {
        true
    }
    /// E åpner butikken eller verkstedet.
    fn inspectable(&self) -> bool {
        true
    }
    fn label(&self) -> &'static str {
        self.aimed.and_then(|i| self.cast.get(i)).map(|c| c.name).unwrap_or("Innbygger")
    }
    fn verb(&self) -> &'static str {
        "Snakk"
    }

    fn build(&mut self, b: &mut Builder) {
        self.cast = Self::place(b);
        for c in &self.cast {
            log::info!("Innbygger {} ved ({:.1}, {:.1}, {:.1})", c.name, c.pos.x, c.pos.y, c.pos.z);
            b.npcs.push(crate::domain::city::NpcSpec { name: c.name, pos: c.pos, yaw: c.yaw, tint: c.tint, lines: c.lines });
            // En tom prop med kapselboks rundt kroppen, så du kan sikte på dem.
            let i = b.prop("npc", Mesh::default(), Affine3A::from_translation(c.pos), Some((Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.3, 0.9, 0.3))));
            self.props.push(i);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        self.aimed = env.aimed_prop.and_then(|p| self.props.iter().position(|q| *q == p));
        env.aimed_npc = self.aimed;
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        if let Some(i) = self.aimed {
            env.talk_to = Some(i);
        }
    }
}
