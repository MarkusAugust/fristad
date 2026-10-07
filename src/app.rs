//! Applikasjonslaget: setter sammen kjernen med portene, og er det de
//! drivende adapterne (nettleser, eget vindu, skjermbilde) snakker med.

use crate::domain::city::{self, City};
use crate::domain::game::{Event, Game, Input};
use crate::domain::installations;
use crate::domain::model::{LayerImages, Model, ModelInfo};
use crate::domain::ports::{ModelSource, Physics, Renderer, TextureSource};
use crate::domain::geometry::Layer;
use crate::domain::text::Font;
use crate::domain::tokens::Tokens;

/// Alt som må være klart før rendereren kan lages: byen og modellene.
pub struct Prepared {
    pub tokens: Tokens,
    pub city: City,
    pub models: Vec<Model>,
    /// Fotoskannede materialer per teksturlag, der de finnes.
    pub layers: Vec<(Layer, LayerImages)>,
    pub installations: Vec<Box<dyn installations::Installation>>,
    pub font: Font,
}

pub fn prepare(models: &dyn ModelSource, textures: &dyn TextureSource) -> Prepared {
    let tokens = Tokens::load();
    let font = Font::load();
    let sheets = crate::domain::sheets::Sheets::load();
    let mut installations = installations::all();
    // Modellene lastes først, så byen vet hvilke som finnes og hvor store de er.
    let loaded: Vec<Model> = models.names().iter().filter_map(|n| models.load(n)).collect();
    let infos: Vec<ModelInfo> = loaded
        .iter()
        .map(|m| {
            let (min, max) = m.mesh.bounds();
            ModelInfo { name: m.name.clone(), min, max }
        })
        .collect();
    let city = city::build(&tokens, &font, &sheets, &infos, &mut installations);
    log::info!("Byen: {} hjørner, {} props, {} modeller tilgjengelig", city.mesh.vertices.len(), city.props.len(), infos.len());
    let models: Vec<Model> = loaded.into_iter().filter(|m| city.placements.iter().any(|p| p.model == m.name)).collect();
    // Panelet holdes bakt: malte bord skal være rene, og de skannede plankene
    // er for værbitte til å tones med malingsfargen.
    let layers: Vec<(Layer, LayerImages)> = Layer::ALL.iter().filter(|l| **l != Layer::Cladding).filter_map(|l| textures.layer(l.name()).map(|i| (*l, i))).collect();
    if !layers.is_empty() {
        log::info!("Fotoskannede materialer: {}", layers.iter().map(|(l, _)| l.name()).collect::<Vec<_>>().join(", "));
    }
    Prepared { tokens, city, models, layers, installations, font }
}

#[cfg(test)]
mod tests {
    /// Rutinen mot svevende modeller: byen bygges med de innebygde
    /// modellene (og de eksterne om de er hentet), og ingen instans får
    /// sveve over eller stå nede i terrenget. `cargo test` før push.
    #[test]
    fn modellene_star_pa_bakken() {
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let prepared = super::prepare(&assets, &assets);
        assert!(prepared.city.audit.is_empty(), "\n{}", prepared.city.audit.join("\n"));
    }

    /// Variantene skal virke 1:1 med designsystemet: alle installasjonene
    /// som er samme komponent må ha de samme variantene (ellers bytter R og
    /// panelet bare noen av dem), og det du sikter på må endre seg for hver
    /// variant: for hver prop som kan siktes på, må de synlige propsene
    /// innen 2,5 m (hvilke, og hvor de står) være ulike for alle variantene.
    /// Palettfarger teller ikke: variantene skal være egne ark fra CSS-en.
    #[test]
    fn variantene_endrer_det_du_sikter_pa() {
        use crate::domain::installations::Env;
        let assets = crate::adapters::assets::MemoryAssets::from_dir(std::path::Path::new("assets/external"));
        let mut prepared = super::prepare(&assets, &assets);
        let mut env = Env::default();
        env.dt = 0.5;
        let mut problems = vec![];
        let names: Vec<String> = prepared.installations.iter().map(|i| i.component().name.to_string()).collect();
        for (i, inst) in prepared.installations.iter_mut().enumerate() {
            let name = &names[i];
            let variants = inst.variants().to_vec();
            if variants.is_empty() {
                continue;
            }
            let mine: Vec<usize> = (0..prepared.city.props.len()).filter(|p| prepared.city.props[*p].owner == Some(i)).collect();
            // Posene per variant, etter at animasjonene har fått roe seg.
            let mut poses: Vec<Vec<(usize, glam::Affine3A)>> = vec![];
            for k in 0..variants.len() {
                inst.set_variant(k);
                for _ in 0..20 {
                    inst.tick(&mut env);
                }
                poses.push(mine.iter().map(|p| (*p, inst.pose(*p, prepared.city.props[*p].rest, &env).0)).filter(|(_, pose)| pose.matrix3.x_axis.length_squared() > 1e-6).collect());
            }
            inst.set_variant(0);
            // Hver prop som kan siktes på: det synlige rundt den må endre seg.
            for p in &mine {
                if prepared.city.props[*p].part {
                    continue;
                }
                let Some((offset, _)) = prepared.city.props[*p].collider else { continue };
                let at = prepared.city.props[*p].rest.transform_point3(offset);
                let mut sigs: Vec<(String, String)> = vec![];
                for (k, v) in variants.iter().enumerate() {
                    let mut sig = String::new();
                    for (q, pose) in &poses[k] {
                        if pose.translation.distance(at.into()) > 2.5 {
                            continue;
                        }
                        let m = pose.to_cols_array();
                        sig += &format!("{q}:{:?};", m.iter().map(|x| (x * 1000.0).round() as i64).collect::<Vec<_>>());
                    }
                    if let Some((other, _)) = sigs.iter().find(|(_, s)| *s == sig) {
                        problems.push(format!("{name} (installasjon {i}, propen «{}» ved ({:.1}, {:.1}, {:.1})): varianten «{v}» ser ut som «{other}»", prepared.city.props[*p].name, at.x, at.y, at.z));
                    }
                    sigs.push((v.to_string(), sig));
                }
            }
        }
        // Samme komponent, samme varianter.
        let lists: Vec<Vec<&str>> = prepared.installations.iter().map(|i| i.variants().to_vec()).collect();
        for i in 0..names.len() {
            for j in 0..i {
                if names[i] == names[j] && lists[i] != lists[j] {
                    problems.push(format!("{}: installasjon {j} har variantene {:?}, installasjon {i} har {:?}", names[i], lists[j], lists[i]));
                }
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }
}

pub struct App<R: Renderer, P: Physics> {
    pub game: Game,
    pub input: Input,
    pub renderer: R,
    pub physics: P,
}

impl<R: Renderer, P: Physics> App<R, P> {
    pub fn new(prepared: Prepared, renderer: R, mut physics: P) -> Self {
        let game = Game::new(prepared.tokens, prepared.font, &prepared.city, prepared.installations, &mut physics);
        Self { game, input: Input::default(), renderer, physics }
    }

    /// Ett steg av spillet. Gir hendelsene til verten.
    pub fn tick(&mut self, dt: f32) -> Vec<Event> {
        let events = self.game.update(&mut self.input, dt, &mut self.physics);
        // Tekst som ble skrevet: propsene får ny geometri.
        for (i, mesh) in self.game.take_dirty() {
            self.renderer.update_prop(i, &mesh);
        }
        events
    }

    /// Tegn bildet.
    pub fn draw(&mut self, aspect: f32) {
        let frame = self.game.frame(aspect);
        self.renderer.render(&frame);
        self.game.settle();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }
}
