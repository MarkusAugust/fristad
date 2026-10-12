//! Været og døgnet: værtypene V blar gjennom, og hvor mye skyer, regn,
//! snø, vind og tåke hver av dem gir.

use crate::domain::scene::Weather;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeatherKind {
    #[default]
    Clear,
    Overcast,
    Rain,
    Storm,
    Snow,
    Fog,
}

impl WeatherKind {
    pub const ALL: [WeatherKind; 6] = [Self::Clear, Self::Overcast, Self::Rain, Self::Storm, Self::Snow, Self::Fog];

    pub fn name(self) -> &'static str {
        match self {
            Self::Clear => "klart",
            Self::Overcast => "overskyet",
            Self::Rain => "regn",
            Self::Storm => "uvær",
            Self::Snow => "snø",
            Self::Fog => "tåke",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.name() == name)
    }

    /// Været det går mot, og hvor mye tettere tåka blir.
    pub fn target(self) -> (Weather, f32) {
        match self {
            Self::Clear => (Weather { cloud: 0.12, wind: 0.2, ..Default::default() }, 1.0),
            Self::Overcast => (Weather { cloud: 0.85, wind: 0.45, ..Default::default() }, 1.4),
            Self::Rain => (Weather { cloud: 0.92, rain: 0.75, wet: 1.0, wind: 0.6, ..Default::default() }, 1.8),
            Self::Storm => (Weather { cloud: 1.0, rain: 1.0, wet: 1.0, wind: 1.3, ..Default::default() }, 2.2),
            Self::Snow => (Weather { cloud: 0.75, snow: 0.8, snow_cover: 1.0, wind: 0.3, ..Default::default() }, 1.6),
            Self::Fog => (Weather { cloud: 0.6, wet: 0.35, wind: 0.1, ..Default::default() }, 6.0),
        }
    }
}

/// Dag eller natt. T bytter, og nettlaget følger med lyst og mørkt tema.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Daylight {
    #[default]
    Day,
    Night,
}

/// Himmelen over byen: døgnet og været slik de er nå, og hva de går mot.
#[derive(Clone, Debug)]
pub struct Sky {
    pub kind: WeatherKind,
    pub daylight: Daylight,
    /// 0 er dag, 1 er natt, glir mellom.
    pub night: f32,
    pub weather: Weather,
    pub fog_mul: f32,
    lightning_timer: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Self { kind: WeatherKind::Clear, daylight: Daylight::Day, night: 0.0, weather: Weather { wind: 0.2, ..Default::default() }, fog_mul: 1.0, lightning_timer: 0.0 }
    }
}

impl Sky {
    pub fn set_daylight(&mut self, d: Daylight, instant: bool) {
        self.daylight = d;
        if instant {
            self.night = if d == Daylight::Night { 1.0 } else { 0.0 };
        }
    }

    pub fn set_weather(&mut self, kind: WeatherKind, instant: bool) {
        self.kind = kind;
        if instant {
            let (target, fog) = kind.target();
            self.weather = target;
            self.fog_mul = fog;
        }
    }

    /// Dag blir natt og natt blir dag.
    pub fn turn(&mut self) -> Daylight {
        self.daylight = if self.daylight == Daylight::Day { Daylight::Night } else { Daylight::Day };
        self.daylight
    }

    /// Neste vær (V).
    pub fn next_weather(&mut self) -> WeatherKind {
        let all = WeatherKind::ALL;
        let i = all.iter().position(|w| *w == self.kind).unwrap_or(0);
        self.kind = all[(i + 1) % all.len()];
        self.kind
    }

    /// Døgnet og været glir mot målet. `time` er klokka i økta (til lynet).
    /// Gir om det lynte (tordenen kommer etter).
    pub fn tick(&mut self, dt: f32, time: f32) -> bool {
        let goal = if self.daylight == Daylight::Night { 1.0 } else { 0.0 };
        let step = dt / 2.8;
        self.night = if self.night < goal { (self.night + step).min(goal) } else { (self.night - step).max(goal) };

        let mut flash = false;
        let (t, fog) = self.kind.target();
        let w = &mut self.weather;
        let ease = |v: &mut f32, goal: f32, rate: f32| *v += (goal - *v) * (dt * rate).min(1.0);
        ease(&mut w.cloud, t.cloud, 0.4);
        ease(&mut w.rain, t.rain, 0.6);
        ease(&mut w.snow, t.snow, 0.6);
        ease(&mut w.wind, t.wind, 0.3);
        let wet_rate = if t.wet > w.wet { 0.25 } else { 0.05 };
        ease(&mut w.wet, t.wet, wet_rate);
        let snow_rate = if t.snow_cover > w.snow_cover { 0.12 } else { 0.06 };
        ease(&mut w.snow_cover, t.snow_cover, snow_rate);
        ease(&mut self.fog_mul, fog, 0.3);
        // Lyn i uvær: korte blink med tilfeldige mellomrom.
        if self.kind == WeatherKind::Storm {
            self.lightning_timer -= dt;
            if self.lightning_timer <= 0.0 {
                w.lightning = 1.0;
                flash = true;
                self.lightning_timer = 3.0 + ((time * 7.3).sin() * 0.5 + 0.5) * 9.0;
            }
        }
        w.lightning *= (1.0 - dt * 9.0).max(0.0);
        flash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natta_kommer_paa_under_tre_sekunder_og_uvaer_lyner() {
        let mut sky = Sky::default();
        sky.turn();
        sky.set_weather(WeatherKind::Storm, false);
        let mut flashes = 0;
        for i in 0..300 {
            flashes += sky.tick(0.01, i as f32 * 0.01) as u32;
        }
        assert_eq!(sky.night, 1.0);
        assert!(flashes >= 1 && sky.weather.rain > 0.8);
    }
}
