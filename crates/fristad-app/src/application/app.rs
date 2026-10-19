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
    /// Bitene til hvert kryptnivå og om rendereren har dem.
    levels: Vec<(Vec<String>, Pieces)>,
    /// Lysstyrken spilleren har valgt (`Command::Gamma`).
    gamma: f32,
}

/// Om rendereren har bitene til et kryptnivå.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pieces {
    /// Lastet fra starten (utenom nettleseren): blir alltid.
    Always,
    /// Ikke lastet.
    Missing,
    /// Bedt om (`Event::FetchLevel`), ikke kommet ennå.
    Asked,
    /// Kom med `add_level`, og slippes når du er langt unna.
    Loaded,
}

impl<R: Renderer, P: Physics> App<R, P> {
    pub fn new(prepared: Prepared, renderer: R, mut physics: P) -> Self {
        let mut session = Session::new(&prepared.world, &mut physics, &rigs(&prepared.models));
        session.edit_data = prepared.edit;
        let levels = prepared
            .world
            .crypt
            .levels
            .iter()
            .map(|l| {
                let have = l.pieces.iter().all(|p| prepared.models.iter().any(|m| &m.name == p));
                (l.pieces.clone(), if have { Pieces::Always } else { Pieces::Missing })
            })
            .collect();
        Self { session, input: Input::default(), renderer, physics, levels, gamma: 1.0 }
    }

    /// Bitene til kryptnivå n (1 og oppover) er hentet: rendereren får dem.
    pub fn add_level(&mut self, n: u32, models: &[crate::domain::model::Model]) {
        let Some(l) = self.levels.get_mut((n as usize).wrapping_sub(1)) else { return };
        if l.1 != Pieces::Always {
            self.renderer.add_models(models);
            l.1 = Pieces::Loaded;
        }
    }

    /// Hvor langt hvert nivå er kommet (til testene og målingen).
    pub fn level_pieces(&self) -> Vec<Pieces> {
        self.levels.iter().map(|l| l.1).collect()
    }

    /// Ber om nivåene økta trenger og slipper dem du har forlatt.
    fn stream_levels(&mut self, events: &mut Vec<Event>) {
        let wanted = self.session.wanted_levels();
        for (k, (pieces, state)) in self.levels.iter_mut().enumerate() {
            match (*state, wanted.contains(&k)) {
                (Pieces::Missing, true) => {
                    events.push(Event::FetchLevel(k as u32 + 1));
                    *state = Pieces::Asked;
                }
                (Pieces::Loaded, false) => {
                    self.renderer.drop_models(pieces);
                    *state = Pieces::Missing;
                }
                _ => {}
            }
        }
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
        let mut events = self.session.update(&mut self.input, dt, &mut self.physics);
        self.stream_levels(&mut events);
        events
    }

    fn draw(&mut self, aspect: f32) {
        let mut frame = self.session.frame(aspect);
        frame.gamma = self.gamma;
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
            Command::Swap(id) => return self.session.swap(&id),
            Command::BuyMead => return self.session.buy_mead(),
            Command::SongOver => self.session.end_song(),
            Command::Equip(id) => return self.session.equip(&id),
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
            Command::Film(shot) => {
                self.input.release_all();
                self.session.film = shot;
            }
            Command::Edit(e) => {
                self.input.release_all();
                return self.session.edit_command(e);
            }
            Command::Voice(v) => self.session.npcs.set_voice(v),
            Command::Gamma(g) => self.gamma = g.clamp(0.6, 2.0),
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
