//! `App`: økta og de drevne adapterne (renderer og fysikk) satt sammen,
//! bak den drivende porten `Play`.

use super::assembly::Prepared;
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
    /// Byens modeller som hentes etter start (nettleseren), og om
    /// rendereren har dem.
    lazy: Vec<Lazy>,
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

/// Hvor nær (m) en modell som hentes senere må stå før den hentes: innsiden
/// tegnes innen `INTERIOR_FAR` (22 m), så den kommer noen skritt før.
pub const LAZY_NEAR: f32 = 30.0;

/// En modell som hentes etter start, og når den trengs.
#[derive(Clone, Debug)]
pub struct Lazy {
    pub name: String,
    /// Utstyr (armsettet, våpenet, skjoldet): straks du har det på.
    gear: bool,
    /// En fiende (`Kind::model`): bare med et nivå (eller nivådokumentet i
    /// redigeringen) som har et merke med slaget.
    mob: bool,
    /// Kryptnivåene den står eller går i (fiendene etter `Crypt.spawns`):
    /// når økta ber om nivået.
    levels: Vec<usize>,
    /// Der den står oppe (innsiden, møblene, galleriet): innen `LAZY_NEAR`.
    spots: Vec<glam::Vec3>,
    pub state: Pieces,
}

impl Lazy {
    fn wanted(&self, eye: glam::Vec3, levels: &[usize], gear: &[&str], editing: &[&str]) -> bool {
        if self.gear {
            return gear.contains(&self.name.as_str());
        }
        let near = self.spots.iter().any(|p| p.distance(eye) <= LAZY_NEAR);
        let level = self.levels.iter().any(|l| levels.contains(l));
        // En fiende som ingen nivåer har ennå, venter til et nivå (eller
        // nivådokumentet i redigeringen) setter den ut.
        let doc = self.mob && editing.contains(&self.name.as_str());
        // Det som verken står fast eller går i et nivå (tokenene, nøkkelen,
        // markøren), kommer med det første nivået.
        let rest = !self.mob && self.spots.is_empty() && self.levels.is_empty() && !levels.is_empty();
        near || level || doc || rest
    }
}

/// Når hver modell som hentes senere trengs (`Prepared.later`, utenom
/// bitene til kryptnivåene).
fn lazy_models(prepared: &Prepared) -> Vec<Lazy> {
    use crate::domain::craft::{ARMORS, GALLERY_MODELS, SHIELDS, WEAPONS};
    use crate::domain::world::Use;
    let world = &prepared.world;
    let crypt = &world.crypt;
    let pieces: std::collections::HashSet<&String> = crypt.levels.iter().flat_map(|l| l.pieces.iter()).collect();
    let gear: Vec<&str> = crate::domain::combat::ARMS_MODELS.iter().copied().chain(WEAPONS.iter().map(|w| w.model)).chain(SHIELDS.iter().map(|s| s.model)).chain(ARMORS.iter().map(|a| a.arms)).collect();
    let gallery: Vec<glam::Vec3> = world.targets.iter().filter(|t| matches!(t.what, Use::Gallery(_))).map(|t| (t.pose * glam::Affine3A::from_translation(t.center)).translation.into()).collect();
    prepared
        .later
        .iter()
        .filter(|n| !pieces.contains(n))
        .map(|name| {
            let mut levels: Vec<usize> = crypt.spawns.iter().filter(|s| crate::domain::mob_kind::Kind::from_label(&s.label).model() == name).map(|s| s.level).collect();
            let mut spots = vec![];
            for t in world.placements.iter().filter(|p| p.model == name.as_str()).flat_map(|p| p.transforms.iter()) {
                let at: glam::Vec3 = t.translation.into();
                match crypt.level_at(at) {
                    Some(k) => levels.push(k),
                    None => spots.push(at),
                }
            }
            if GALLERY_MODELS.contains(&name.as_str()) {
                spots.extend(gallery.iter().copied());
            }
            levels.sort_unstable();
            levels.dedup();
            let mob = crate::domain::mob_kind::Kind::ALL.iter().any(|k| k.model() == name.as_str());
            Lazy { name: name.clone(), gear: gear.contains(&name.as_str()), mob, levels, spots, state: Pieces::Missing }
        })
        .collect()
}

impl<R: Renderer, P: Physics> App<R, P> {
    pub fn new(prepared: Prepared, renderer: R, mut physics: P) -> Self {
        let mut session = Session::new(&prepared.world, &mut physics, &prepared.rigs);
        let lazy = lazy_models(&prepared);
        session.edit_data = prepared.edit;
        session.level = prepared.level.map(|s| super::level_editing::LevelEditing::new(s, &prepared.world, &prepared.infos));
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
        Self { session, input: Input::default(), renderer, physics, levels, lazy, gamma: 1.0 }
    }

    /// Modellene som ble bedt om (`Event::FetchModels`) er hentet:
    /// rendereren får dem.
    pub fn add_models(&mut self, models: &[crate::domain::model::Model]) {
        for m in models {
            if let Some(l) = self.lazy.iter_mut().find(|l| l.name == m.name) {
                l.state = Pieces::Loaded;
            }
        }
        // Rendereren tar bare dem som har instanser som venter.
        self.renderer.add_models(models);
    }

    /// Modellene som hentes etter start, og hvor langt hver er kommet (til
    /// testene og målingen).
    pub fn lazy_models(&self) -> &[Lazy] {
        &self.lazy
    }

    /// Ber om modellene økta trenger nå: utstyret du har på, det som står
    /// nær, og det som hører til nivåene den ber om.
    fn stream_models(&mut self, events: &mut Vec<Event>) {
        if self.lazy.iter().all(|l| l.state != Pieces::Missing) {
            return;
        }
        let eye = self.session.eye();
        let levels = self.session.wanted_levels();
        let c = &self.session.combat;
        let gear = [c.arms_model(), c.gear.weapon.model, c.gear.shield.model];
        // Fiendene i nivådokumentet som redigeres (merkene, slik de er nå).
        let marks = self.session.level_editing().filter(|l| l.active).map(|l| l.editor.file().marks.as_slice()).unwrap_or_default();
        let editing: Vec<&str> = marks
            .iter()
            .filter(|m| matches!(m.kind.as_str(), "mob" | "elite" | "boss"))
            .map(|m| crate::domain::mob_kind::Kind::from_label(m.label.as_deref().unwrap_or(if m.kind == "boss" { "rammeverket" } else { "button" })).model())
            .collect();
        // Utstyret først (i rekkefølgen armene, våpenet, skjoldet), så resten.
        let (mut ask, mut rest) = (vec![], vec![]);
        for l in self.lazy.iter_mut().filter(|l| l.state == Pieces::Missing) {
            if l.wanted(eye, &levels, &gear, &editing) {
                l.state = Pieces::Asked;
                if l.gear { &mut ask } else { &mut rest }.push(l.name.clone());
            }
        }
        ask.sort_by_key(|n| gear.iter().position(|g| g == n));
        ask.extend(rest);
        if !ask.is_empty() {
            events.push(Event::FetchModels(ask));
        }
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
        self.stream_models(&mut events);
        events
    }

    fn draw(&mut self, aspect: f32) {
        let mut frame = self.session.frame(aspect);
        frame.gamma = self.gamma;
        self.session.level_frame(&mut frame);
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
            Command::Level(c) => return self.session.level_command(c),
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
