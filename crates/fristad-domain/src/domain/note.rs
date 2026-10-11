//! Det reglene har å si: lyder og meldinger. Applikasjonen gjør dem om til
//! hendelser for verten (lyd i hodet eller fra et sted, en melding, svart).

use glam::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub enum Note {
    /// En lyd i hodet.
    Sfx(&'static str),
    /// En lyd fra et sted i verden, med styrke.
    At(&'static str, Vec3, f32),
    /// En melding på skjermen.
    Toast(String),
    /// Skjermen toner til svart og tilbake.
    Fade,
}

impl Note {
    pub fn toast(text: impl Into<String>) -> Self {
        Note::Toast(text.into())
    }
}
