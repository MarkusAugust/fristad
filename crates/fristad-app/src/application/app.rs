//! `App`: økta og de drevne adapterne (renderer og fysikk) satt sammen,
//! bak den drivende porten `Play`.

use super::assembly::{rigs, Prepared};
use super::session::Session;
use crate::ports::driving::{Command, Event, Input, Play, SaveData, Status};
use crate::ports::{Physics, Renderer, StaticScene};

pub struct App<R: Renderer, P: Physics> {
    pub session: Session,
    pub input: Input,
    pub renderer: R,
    pub physics: P,
}

impl<R: Renderer, P: Physics> App<R, P> {
    pub fn new(prepared: Prepared, renderer: R, mut physics: P) -> Self {
        let session = Session::new(&prepared.world, &mut physics, &rigs(&prepared.models));
        Self { session, input: Input::default(), renderer, physics }
    }

    /// Det rendereren trenger for å lages, fra det monteringen ga.
    pub fn scene(prepared: &Prepared) -> StaticScene<'_> {
        StaticScene { mesh: &prepared.world.mesh, chunks: &prepared.world.chunks, placements: &prepared.world.placements, models: &prepared.models }
    }
}

impl<R: Renderer, P: Physics> Play for App<R, P> {
    fn input(&mut self) -> &mut Input {
        &mut self.input
    }

    fn tick(&mut self, dt: f32) -> Vec<Event> {
        self.session.update(&mut self.input, dt, &mut self.physics)
    }

    fn draw(&mut self, aspect: f32) {
        let frame = self.session.frame(aspect);
        self.renderer.render(&frame);
        self.session.settle();
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    fn command(&mut self, c: Command) -> bool {
        match c {
            Command::Craft(id) => return self.session.craft(&id),
            Command::Buy(id) => return self.session.buy(&id),
            Command::ReadDoc(c) => return self.session.read_doc(&c),
            Command::Load(d) => self.session.load(d),
            Command::SetDaylight(d) => self.session.set_daylight(d, false),
            Command::SetWeather(w) => self.session.set_weather(w, false),
            Command::SetPaused(p) => {
                self.session.paused = p;
                if p {
                    self.input.release_all();
                }
            }
            Command::Respawn => self.session.go_home_now(),
            Command::Descend => {
                let events = self.session.use_first(|u| *u == crate::domain::world::Use::Descend);
                self.session.queue(events);
            }
            Command::Place { eye, yaw, pitch } => self.session.place(eye, yaw, pitch),
            Command::SetQuality(q) => self.renderer.set_quality(q),
            Command::SetBench(b) => self.renderer.set_bench(b),
        }
        true
    }

    fn save(&self) -> SaveData {
        self.session.save()
    }

    fn status(&self) -> Status {
        Status { weather: self.session.weather_kind(), daylight: self.session.daylight(), triangles: self.renderer.triangles(), calls: self.renderer.calls(), gpu_times: self.renderer.gpu_times() }
    }
}
