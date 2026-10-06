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
}

pub fn prepare(models: &dyn ModelSource, textures: &dyn TextureSource) -> Prepared {
    let tokens = Tokens::load();
    let font = Font::load();
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
    let city = city::build(&tokens, &font, &infos, &mut installations);
    log::info!("Byen: {} hjørner, {} props, {} modeller tilgjengelig", city.mesh.vertices.len(), city.props.len(), infos.len());
    let models: Vec<Model> = loaded.into_iter().filter(|m| city.placements.iter().any(|p| p.model == m.name)).collect();
    let layers: Vec<(Layer, LayerImages)> = Layer::ALL.iter().filter_map(|l| textures.layer(l.name()).map(|i| (*l, i))).collect();
    if !layers.is_empty() {
        log::info!("Fotoskannede materialer: {}", layers.iter().map(|(l, _)| l.name()).collect::<Vec<_>>().join(", "));
    }
    Prepared { tokens, city, models, layers, installations }
}

pub struct App<R: Renderer, P: Physics> {
    pub game: Game,
    pub input: Input,
    pub renderer: R,
    pub physics: P,
}

impl<R: Renderer, P: Physics> App<R, P> {
    pub fn new(prepared: Prepared, renderer: R, mut physics: P) -> Self {
        let game = Game::new(prepared.tokens, &prepared.city, prepared.installations, &mut physics);
        Self { game, input: Input::default(), renderer, physics }
    }

    /// Ett steg av spillet. Gir hendelsene til verten.
    pub fn tick(&mut self, dt: f32) -> Vec<Event> {
        self.game.update(&mut self.input, dt, &mut self.physics)
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
