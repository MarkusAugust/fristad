//! Lydene: hvor en lyd kommer fra sett fra øyet, hva du går på,
//! skrittene til innbyggerne, kirkeklokka, de fjerne lydene i krypten og
//! lydbildet (hvor nær du er torget, smia, kirka, ilden og portalen).

use super::session::Session;
use crate::domain::weather::Daylight;
use crate::domain::world::{terrain, town, World};
use crate::ports::driving::{Ambience, Event, Sound};
use crate::ports::Physics;
use glam::{Vec2, Vec3};

/// Vertshuset: så nær midten du må stå inne for å være i skjenkestua, og
/// hvor langt unna trubaduren høres utenfor.
const TAVERN_INSIDE: f32 = 10.0;
const TAVERN_HEARD: f32 = 22.0;

/// Det lydbildet husker mellom bildene.
pub(super) struct Soundscape {
    /// Der lydene i byen kommer fra: torget, smia og kirka (flatt).
    spots: [Vec2; 3],
    /// Midt i Vertshuset (flatt), der trubaduren synger.
    tavern: Vec2,
    /// Det som lager lyd i løkke der det står: ild (ildsteder og fakler).
    fires: Vec<Vec3>,
    last: Ambience,
    /// Sekunder til lydbildet sendes igjen.
    t: f32,
    /// Sekunder til neste fjerne lyd i krypten, og terningen til dem.
    distant_t: f32,
    seed: u32,
    /// Innendørs (tak over deg og vegger rundt), målt med stråler.
    pub indoor: bool,
    /// Innbyggerne: hvor de sto sist bilde og hvor langt de har gått siden
    /// forrige skritt.
    npc_steps: Vec<(Vec3, f32)>,
}

impl Soundscape {
    pub fn new(world: &World) -> Self {
        let at = |name: &str| world.placements.iter().find(|p| p.model == name).and_then(|p| p.transforms.first()).map(|t| Vec2::new(t.translation.x, t.translation.z)).unwrap_or(Vec2::splat(1.0e4));
        let sq = town::square_center();
        Self {
            spots: [Vec2::new(sq.x, sq.z), at("forge"), at("stave_church")],
            tavern: at("tavern"),
            fires: world.lights.iter().filter(|l| l.always).map(|l| l.pos).collect(),
            last: Ambience::default(),
            t: 0.0,
            distant_t: 6.0,
            seed: 0x9e37_79b9,
            indoor: false,
            npc_steps: vec![],
        }
    }

    fn roll(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed % 10_000) as f32 / 10_000.0
    }
}

impl Session {
    /// En lyd fra et sted i verden, sett fra øyet: x til høyre, y opp, −z
    /// rett fram (slik nettleserens lytter står).
    pub(super) fn sound_at(&self, name: &str, at: Vec3, gain: f32) -> Event {
        let d = at - self.eye();
        let (s, c) = self.yaw.sin_cos();
        Event::Sound(Sound { name: name.into(), at: Some([d.x * c - d.z * s, d.y, d.x * s + d.z * c]), gain })
    }

    /// Kirkeklokka, høyt over kirka.
    pub(super) fn bell(&self) -> Event {
        let church = self.sound.spots[2];
        self.sound_at("bell", Vec3::new(church.x, self.eye().y + 12.0, church.y), 2.5)
    }

    /// Hva du går på: navnet på skrittlyden.
    pub(super) fn surface(&self, feet: Vec3) -> &'static str {
        if self.in_crypt() {
            return "step_rock";
        }
        let ground = terrain::height(feet.x, feet.z);
        if feet.y > ground + 0.3 {
            // Gulv, trapper og galleri i husene (og murkronene).
            return if self.sound.indoor { "step_wood" } else { "step_rock" };
        }
        if self.sky.weather.snow_cover > 0.35 {
            return "step_snow";
        }
        let g = town::ground_weights(feet.x, feet.z);
        let grass = (1.0 - g.cobble - g.dirt - g.rock - g.forest).max(0.0);
        let (mut best, mut name) = (grass, "step_grass");
        for (w, n) in [(g.cobble, "step_rock"), (g.rock, "step_rock"), (g.dirt * (1.0 - g.earth), "step_gravel"), (g.dirt * g.earth, "step_dirt"), (g.forest, "step_dirt")] {
            if w > best {
                best = w;
                name = n;
            }
        }
        name
    }

    /// Skrittene til innbyggerne som går nær deg.
    pub(super) fn npc_steps(&mut self, feet: Vec3, in_crypt: bool, events: &mut Vec<Event>) {
        let n = self.npcs.npcs.len();
        self.sound.npc_steps.resize(n, (Vec3::ZERO, 0.0));
        for i in 0..n {
            let p = self.npcs.npcs[i].pos;
            let (last, stride) = self.sound.npc_steps[i];
            let d = p.with_y(0.0).distance(last.with_y(0.0));
            let mut stride = if d < 1.0 { stride + d } else { 0.0 };
            if stride > 0.8 {
                stride = 0.0;
                if p.distance(feet) < 16.0 && !in_crypt {
                    let surface = self.surface(p + Vec3::Y * 0.05);
                    events.push(self.sound_at(surface, p, 0.45));
                }
            }
            self.sound.npc_steps[i] = (p, stride);
        }
    }

    /// Lydbildet: hvor nær du er torget, smia, kirka og dyrene, sendt fire
    /// ganger i sekundet når det har endret seg.
    pub(super) fn ambience(&mut self, dt: f32, feet: Vec3, in_crypt: bool, physics: &mut dyn Physics, events: &mut Vec<Event>) {
        self.sound.t -= dt;
        if self.sound.t > 0.0 {
            return;
        }
        self.sound.t = 0.25;
        // Fjerne lyder i krypten: noe beveger seg i rommene ved siden av.
        if in_crypt {
            self.sound.distant_t -= 0.25;
            if self.sound.distant_t <= 0.0 {
                let (wait, pick, angle, dist) = (self.sound.roll(), self.sound.roll(), self.sound.roll(), self.sound.roll());
                self.sound.distant_t = 8.0 + wait * 14.0;
                let name = ["rats", "rats", "door_close", "locked", "notice", "windup", "crack"][(pick * 7.0) as usize % 7];
                let a = angle * std::f32::consts::TAU;
                let at = self.eye() + Vec3::new(a.cos(), 0.0, a.sin()) * (12.0 + dist * 14.0);
                events.push(self.sound_at(name, at, if name == "rats" { 0.9 } else { 0.35 }));
            }
        }
        // Innendørs: tak over deg og vegger på minst tre kanter.
        let eye = self.eye();
        let roof = physics.cast_solid(eye, Vec3::Y, 12.0).is_some();
        let walls = [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z].iter().filter(|d| physics.cast_solid(eye, **d, 9.0).is_some()).count();
        self.sound.indoor = !in_crypt && roof && walls >= 3;
        let close = |list: &[Vec3], r: f32| list.iter().map(|q| (1.0 - q.distance(eye) / r).clamp(0.0, 1.0)).fold(0.0, f32::max).powi(2);
        let p = Vec2::new(feet.x, feet.z);
        let near = |at: Vec2, r: f32| (1.0 - p.distance(at) / r).clamp(0.0, 1.0).powi(2);
        let [square, forge, church] = self.sound.spots;
        let meadow = self.herd.positions().map(|a| near(a, 45.0)).fold(0.0, f32::max);
        let a = Ambience {
            square: near(square, 55.0),
            forge: near(forge, 32.0),
            church: near(church, 140.0),
            meadow,
            fire: close(&self.sound.fires, 7.0),
            water: 0.0,
            lava: 0.0,
            chasm: 0.0,
            // Inne i Vertshuset (tak og vegger, nær midten) er det trubaduren;
            // utenfor høres han svakere jo lenger unna du står.
            tavern: match p.distance(self.sound.tavern) {
                _ if in_crypt => 0.0,
                d if self.sound.indoor && d < TAVERN_INSIDE => 1.0,
                d => 0.5 * (1.0 - d / TAVERN_HEARD).clamp(0.0, 1.0).powi(2),
            },
            portal: match (in_crypt, self.portal.dest.is_some(), self.portal_center()) {
                (false, true, Some(c)) => close(&[c], 16.0),
                _ => 0.0,
            },
            rain: if in_crypt { 0.0 } else { self.sky.weather.rain },
            wind: if in_crypt { 0.0 } else { self.sky.weather.wind },
            night: self.sky.daylight == Daylight::Night,
            cellar: in_crypt,
            indoor: self.sound.indoor,
        };
        let last = self.sound.last;
        let moved = [a.tavern - last.tavern, a.square - last.square, a.forge - last.forge, a.church - last.church, a.meadow - last.meadow, a.fire - last.fire, a.water - last.water, a.lava - last.lava, a.chasm - last.chasm, a.portal - last.portal, a.rain - last.rain, a.wind - last.wind]
            .iter()
            .fold(0.0f32, |m, d| m.max(d.abs()));
        if moved > 0.02 || a.night != last.night || a.cellar != last.cellar || a.indoor != last.indoor {
            self.sound.last = a;
            events.push(Event::Ambience(a));
        }
    }
}
