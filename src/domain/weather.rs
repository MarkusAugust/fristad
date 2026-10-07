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
