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
pub mod gorvek;
pub mod cellar;
pub mod gate;
pub mod npcs;
pub mod chests;
pub mod skeleton;
pub mod spinner;
pub mod submit;
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
    /// Et tegn skrevet mens feltet har fokus: '\u{8}' er rettetast,
    /// '\n' Enter og '\u{1b}' Escape.
    Type(char),
    /// Fokuset gikk et annet sted.
    Blur,
}

/// Det som står i søknadsskjemaet, delt mellom feltene og sendeknappen.
#[derive(Clone, Debug, Default)]
pub struct FormState {
    pub email: String,
    /// Valgt fylke; 0 er «Velg fylke».
    pub county: usize,
    pub reason: String,
    pub delivery: usize,
    pub terms: bool,
    /// Om «Send søknad» er trykket; da valideres feltene fortløpende.
    pub submitted: bool,
    pub sent: bool,
    /// Feil i e-post, fylke, begrunnelse og vilkår.
    pub errors: [bool; 4],
}

impl FormState {
    pub fn validate(&mut self) -> bool {
        let at = self.email.find('@');
        let email_ok = matches!(at, Some(i) if i > 0 && self.email[i + 1..].contains('.') && !self.email.ends_with('.'));
        self.errors = [!email_ok, self.county == 0, self.reason.trim().chars().count() < 10, !self.terms];
        let ok = !self.errors.iter().any(|e| *e);
        if !ok {
            self.sent = false;
        }
        ok
    }
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
    /// Dit spilleren skal flyttes med én gang, uten flytur (ned i kjelleren
    /// og opp igjen): føttene i verden (absolutt y), og retningen.
    pub drop: Option<(Vec3, f32)>,
    /// Skrifta, til tekst som bygges mens spillet går.
    pub font: Option<std::sync::Arc<crate::domain::text::Font>>,
    /// Props som har fått ny geometri og må lastes opp på nytt.
    pub dirty: Vec<(usize, crate::domain::geometry::Mesh)>,
    /// En installasjon ber om tastaturfokus (true) eller slipper det (false).
    pub edit: Option<bool>,
    pub form: FormState,
    /// Nøklene spilleren bærer (til de rustne portene i kjelleren).
    pub keys: u32,
    /// F på en innbygger: hvem (indeks i rollelista) spillet skal la snakke.
    pub talk_to: Option<usize>,
    /// Innbyggeren du sikter på nå (til E: butikk eller verksted).
    pub aimed_npc: Option<usize>,
    /// F på en kiste: hvilken (indeks i `Dungeon.chests`) som ble åpnet.
    pub open_chest: Option<usize>,
}

impl Default for Env {
    fn default() -> Self {
        Self { time: 0.0, dt: 0.0, night: 0.0, gate_open: false, gate_t: 0.0, street_lights: true, weather: WeatherKind::Clear, online: true, toasts: vec![], aimed_prop: None, teleport: None, drop: None, font: None, dirty: vec![], edit: None, form: FormState::default(), keys: 0, talk_to: None, aimed_npc: None, open_chest: None }
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

    /// Om E åpner et panel: komponentene ja, ting i byen (brønnen) nei.
    fn inspectable(&self) -> bool {
        true
    }

    /// Navnet i siktet for ting som ikke er komponenter (tom: dokumentasjonens tittel).
    fn label(&self) -> &'static str {
        ""
    }

    /// Hva F gjør, slik siktet sier det.
    fn verb(&self) -> &'static str {
        "Bruk"
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
        Box::new(submit::Submit::default()),
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
        Box::new(gorvek::Gorvek::default()),
        Box::new(cellar::Cellar::default()),
        Box::new(gate::Gate::default()),
        Box::new(npcs::Npcs::default()),
        Box::new(chests::Chests::default()),
    ]
}

/// Én CSS-piksel er én centimeter for komponentene i byen.
pub const PX: f32 = 0.01;
