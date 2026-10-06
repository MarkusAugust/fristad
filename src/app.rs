//! Applikasjonslaget: setter sammen kjernen med portene, og er det de
//! drivende adapterne (nettleser, eget vindu, skjermbilde) snakker med.

use crate::domain::city::{self, City};
use crate::domain::game::{Event, Game, Input};
use crate::domain::installations;
use crate::domain::model::Model;
use crate::domain::ports::{ModelSource, Physics, Renderer};
use crate::domain::text::Font;
use crate::domain::tokens::Tokens;

/// Alt som må være klart før rendereren kan lages: byen og modellene.
pub struct Prepared {
    pub tokens: Tokens,
    pub city: City,
    pub models: Vec<Model>,
    pub installations: Vec<Box<dyn installations::Installation>>,
}

pub fn prepare(models: &dyn ModelSource) -> Prepared {
    let tokens = Tokens::load();
    let font = Font::load();
    let mut installations = installations::all();
    let city = city::build(&tokens, &font, &mut installations);
    let text_verts = city.mesh.vertices.iter().filter(|v| v.mat[0] == 203).count() + city.props.iter().flat_map(|p| p.mesh.vertices.iter()).filter(|v| v.mat[0] == 203).count();
    log::info!("Byen: {} hjørner, {} props, {} tekst-hjørner", city.mesh.vertices.len(), city.props.len(), text_verts);
    let models: Vec<Model> = city.placements.iter().filter_map(|p| models.load(p.model)).collect();
    Prepared { tokens, city, models, installations }
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
