//! Komponentene slik de står i byen. Hver Fristil-komponent er en
//! `Installation`: den bygger sin egen geometri og sine props, sier hva du
//! kan gjøre med den, animerer propsene sine og setter palettfarger.
//!
//! Ny komponent: lag en fil her, implementer `Installation`, og legg den
//! til i `all()`. Resten (sikting, E/R/F, panelet i nettleseren, tema)
//! håndteres av spillet og nettleserlaget ut fra `component()`.

use crate::domain::city::Builder;
use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::{Affine3A, Vec3};

pub mod accordion;
pub mod alert;
pub mod avatar;
pub mod badge;
pub mod breadcrumbs;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod connection_status;
pub mod dialog;
pub mod divider;
pub mod error_text;
pub mod fieldset;
pub mod file_upload;
pub mod form;
pub mod heading;
pub mod help_text;
pub mod home_signs;
pub mod input;
pub mod label;
pub mod link;
pub mod progress;
pub mod radio;
pub mod search;
pub mod select;
pub mod signs;
pub mod skeleton;
pub mod spinner;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod tag;
pub mod textarea;
pub mod tooltip;
pub mod workshop;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Component {
    /// Navnet i dokumentasjonen, for eksempel `button`.
    pub name: &'static str,
    /// CSS-klassen, for eksempel `.fs-button`.
    pub class: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// F: bruk den.
    Use,
}

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
}

/// Tilstanden i byen som installasjonene deler. Knappen åpner porten,
/// framdriftsstolpen viser hvor åpen den er, bryteren tenner gatelysene og
/// forbindelsesstatusen styrer været.
#[derive(Debug)]
pub struct Env {
    pub time: f32,
    pub dt: f32,
    /// 0 dag, 1 natt.
    pub night: f32,
    pub gate_open: bool,
    /// 0 lukket, 1 åpen.
    pub gate_t: f32,
    pub street_lights: bool,
    pub weather: WeatherKind,
    /// Forbindelsen til serveren, slik nettleseren melder den.
    pub online: bool,
    /// Korte meldinger som skal vises som toasts.
    pub toasts: Vec<String>,
    /// Propen spilleren sikter på nå, om noen (som hover).
    pub aimed_prop: Option<usize>,
    /// Dit spilleren skal flyttes: føttene på bakken, og retningen (yaw).
    /// Lenker og brødsmuler setter den; spillet tar den.
    pub teleport: Option<(Vec3, f32)>,
}

impl Default for Env {
    fn default() -> Self {
        Self { time: 0.0, dt: 0.0, night: 0.0, gate_open: false, gate_t: 0.0, street_lights: true, weather: WeatherKind::Clear, online: true, toasts: vec![], aimed_prop: None, teleport: None }
    }
}

pub trait Installation {
    fn component(&self) -> Component;

    /// Variantene R blar gjennom. Tom liste betyr at R ikke gjør noe.
    fn variants(&self) -> &'static [&'static str] {
        &[]
    }
    fn variant(&self) -> usize {
        0
    }
    fn set_variant(&mut self, _i: usize) {}

    /// Bygg geometrien, propsene, kolliderne og lysene. Propsene som skal
    /// kunne siktes på registreres med `Builder::prop`, som gir dem
    /// eieren tilbake.
    fn build(&mut self, b: &mut Builder);

    fn tick(&mut self, _env: &mut Env) {}

    /// Om F gjør noe med den.
    fn usable(&self) -> bool {
        false
    }

    fn act(&mut self, _action: Action, _env: &mut Env) {}

    /// Hvor en av propsene står nå, med ekstra verdier til shaderen
    /// (x markering settes av spillet, y animasjon).
    fn pose(&self, _prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        (rest, [0.0; 4])
    }

    /// Palettfarger som avhenger av varianten.
    fn palette(&self, _tokens: &Tokens, _night: f32, _p: &mut [Vec3; PALETTE]) {}

    /// Lys som avhenger av tilstanden (for eksempel gatelysene).
    fn lights_on(&self, _env: &Env) -> f32 {
        1.0
    }
}

/// Alle komponentene i byen, i den rekkefølgen de bygges.
pub fn all() -> Vec<Box<dyn Installation>> {
    vec![
        Box::new(button::Button::default()),
        Box::new(switch::Switch::default()),
        Box::new(progress::Progress::default()),
        Box::new(alert::Alert::default()),
        Box::new(tag::Tag::default()),
        Box::new(badge::Badge::default()),
        Box::new(spinner::Spinner::default()),
        Box::new(breadcrumbs::Breadcrumbs::default()),
        Box::new(skeleton::Skeleton::default()),
        Box::new(connection_status::ConnectionStatus::default()),
        Box::new(dialog::Dialog::default()),
        Box::new(tabs::Tabs::default()),
        Box::new(card::Card::default()),
        Box::new(accordion::Accordion::default()),
        Box::new(input::Input::default()),
        Box::new(select::Select::default()),
        Box::new(radio::Radio::default()),
        Box::new(checkbox::Checkbox::default()),
        Box::new(textarea::Textarea::default()),
        Box::new(table::Table::default()),
        Box::new(tooltip::Tooltip::default()),
        Box::new(link::Link::default()),
        Box::new(avatar::Avatar::default()),
        Box::new(divider::Divider::default()),
        Box::new(heading::Heading::default()),
        Box::new(label::Label::default()),
        Box::new(help_text::HelpText::default()),
        Box::new(error_text::ErrorText::default()),
        Box::new(fieldset::Fieldset::default()),
        Box::new(file_upload::FileUpload::default()),
        Box::new(search::Search::default()),
        Box::new(home_signs::HomeSigns::default()),
        Box::new(workshop::GetStarted),
        Box::new(workshop::TypeSafe),
        Box::new(workshop::Accessibility),
    ]
}

/// Én CSS-piksel er én centimeter for komponentene i byen.
pub const PX: f32 = 0.01;
