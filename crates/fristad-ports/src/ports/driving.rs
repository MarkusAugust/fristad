//! Den drivende porten: det nettleseren, vinduet og skjermbildeverktøyet
//! sender inn til spillet, og det de får tilbake. `application::App`
//! implementerer `Play`; adapterne kjenner bare denne fila.

use crate::domain::edit::{LineKind, Selected};
use crate::domain::weather::{Daylight, WeatherKind};
use crate::domain::world::layout::Ref;
use glam::{Affine3A, Vec3};

/// Tastene og musa slik de står nå, og handlingene siden sist.
#[derive(Default)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    /// C holdes inne: huk.
    pub crouch: bool,
    /// Høyre musetast holdes inne: blokk med skjoldet.
    pub block: bool,
    pub look: (f32, f32),
    pub actions: Vec<Action>,
}

impl Input {
    /// Tast trykket eller sluppet. `code` er `KeyboardEvent.code` i
    /// nettleseren, og samme navn fra winit native: "KeyW", "Space" osv.
    pub fn key(&mut self, code: &str, down: bool) {
        match code {
            "KeyW" | "ArrowUp" => self.forward = down,
            "KeyS" | "ArrowDown" => self.back = down,
            "KeyA" | "ArrowLeft" => self.left = down,
            "KeyD" | "ArrowRight" => self.right = down,
            "Space" => self.jump = down,
            "ShiftLeft" | "ShiftRight" => {
                self.sprint = down;
                // I kjelleren er Shift med en retning en unnamanøver.
                if down {
                    self.actions.push(Action::Dodge);
                }
            }
            "KeyC" => self.crouch = down,
            "KeyE" if down => self.actions.push(Action::Trade),
            "KeyF" if down => self.actions.push(Action::Use),
            "KeyT" if down => self.actions.push(Action::Daylight),
            "KeyV" if down => self.actions.push(Action::Weather),
            "KeyH" if down => self.actions.push(Action::Home),
            _ => {}
        }
    }

    /// Musetast trykket eller sluppet: 0 er venstre (hugg, hold for tungt),
    /// 2 er høyre (blokk mens den holdes).
    pub fn mouse(&mut self, button: u32, down: bool) {
        match button {
            0 => self.actions.push(if down { Action::Attack } else { Action::AttackRelease }),
            2 => self.block = down,
            _ => {}
        }
    }

    /// Slipp alle taster, for eksempel når vinduet mister fokus.
    pub fn release_all(&mut self) {
        let actions = std::mem::take(&mut self.actions);
        *self = Input { actions, ..Default::default() };
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// F: bruk det du sikter på (snakk, åpne, lås opp, gå ned eller opp).
    Use,
    /// E: handle med innbyggeren du sikter på (butikk eller verksted).
    Trade,
    /// T: dag eller natt.
    Daylight,
    /// V: neste vær.
    Weather,
    /// H: tilbake til sørporten.
    Home,
    /// Venstre musetast ned: hugg (i kjelleren).
    Attack,
    AttackRelease,
    /// Shift trykket: unnamanøver i kjelleren om du går en vei.
    Dodge,
}

/// Det verten ber spillet om utenom tastene.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Smi etter en oppskrift (fra skjemaet i skriptoriet).
    Craft(String),
    /// Kjøp en vare.
    Buy(String),
    /// Les om en komponent i boka (gir liv).
    ReadDoc(String),
    Load(SaveData),
    SetDaylight(Daylight),
    SetWeather(WeatherKind),
    SetPaused(bool),
    /// Tilbake til sørporten.
    Respawn,
    /// Rett ned i kjelleren, som om du gikk inn døra i krypten (til
    /// nettlesertestene).
    Descend,
    /// Flytt kameraet (øyet), til skjermbilder og tester.
    Place {
        eye: Vec3,
        yaw: f32,
        pitch: f32,
    },
    SetQuality(super::Quality),
    SetBench(super::Bench),
    /// Redigeringsmodusen (`docs/redigering.md`; bare med `?edit`).
    Edit(EditCommand),
}

/// Hva et drag med musa gjør (tasten som holdes når knappen trykkes).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Grab {
    /// Håndtakene, ellers dra tingen under pekeren langs bakken.
    #[default]
    Move,
    /// Opp og ned (Alt).
    Lift,
    /// Drei om midten av det valgte (Ctrl eller Cmd).
    Turn,
}

/// Det eieren gjør i redigeringen. Med musa: `at` er pekeren på skjermen
/// (0..1 fra øverst til venstre); uten (`None`) gjelder det midten.
#[derive(Clone, Debug, PartialEq)]
pub enum EditCommand {
    /// Inn i redigeringen (fly fritt, gjennom alt) og ut igjen.
    Enter,
    Leave,
    /// Venstre knapp ned: tar tak i et håndtak (pilene og ringen rundt det
    /// valgte), ellers velges det under pekeren og kan dras langs bakken;
    /// `add` (Shift) legger til eller tar bort. `grab` er tasten som holdes
    /// (Alt heiser, Ctrl eller Cmd dreier det valgte, uansett hvor du drar).
    Press {
        at: (f32, f32),
        add: bool,
        grab: Grab,
    },
    /// Musa flyttet med knappen nede: håndtaket drar det valgte.
    Drag {
        to: (f32, f32),
    },
    /// Knappen opp: draget angres som én handling.
    Release,
    /// Velg det under pekeren eller midt i bildet.
    Pick {
        add: bool,
        at: Option<(f32, f32)>,
    },
    /// Tab: det neste under pekeren fra siste klikk.
    Cycle,
    /// Flytt det valgte så mange meter (piltastene, PageUp og PageDown).
    Move(Vec3),
    /// Drei det valgte (Q og E), radianer.
    Rotate(f32),
    /// Sett en ting rett dit.
    SetPose {
        r: Ref,
        pose: Affine3A,
    },
    /// Sett det valgte ned på det som er under.
    Drop,
    /// Kopier det valgte (Ctrl+D).
    Copy,
    /// Slett det valgte (Delete).
    Delete,
    /// Sett ut en ny ting under pekeren (eller midt i bildet), vendt mot deg.
    Place {
        model: String,
        at: Option<(f32, f32)>,
    },
    /// Linjeverktøyet: et punkt på bakken under pekeren, ferdig (gjerde, rad
    /// eller usynlig vegg gjennom punktene), eller avbryt.
    LinePoint {
        at: (f32, f32),
    },
    LineFinish(LineKind),
    LineCancel,
    /// En linje gjennom punktene på bakken.
    Line {
        points: Vec<Vec3>,
        kind: LineKind,
    },
    /// Ny høyde på de valgte usynlige veggene.
    WallHeight(f32),
    Undo,
    Redo,
}

/// Redigeringen sett utenfra: det valgte, reglene som er brutt, og om noe
/// kan angres, gjøres om, er endret og kan lagres.
#[derive(Clone, Debug, PartialEq)]
pub struct EditView {
    pub selected: Vec<Selected>,
    pub breaches: Vec<(Ref, String)>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    /// Ingen regler brutt: Lagre er lov.
    pub clean: bool,
    /// Punktene i linjeverktøyet så langt.
    pub draft: usize,
    /// Det som lå under pekeren ved siste klikk, nærmest først, og om det er
    /// valgt.
    pub under: Vec<(String, bool)>,
}

/// Spillet sett utenfra.
pub trait Play {
    fn input(&mut self) -> &mut Input;
    /// Ett steg. Gir hendelsene til verten.
    fn tick(&mut self, dt: f32) -> Vec<Event>;
    /// Tegn bildet.
    fn draw(&mut self, aspect: f32);
    fn resize(&mut self, width: u32, height: u32);
    /// Gir om det gikk (smiing, kjøp, lesing); resten gir alltid `true`.
    fn command(&mut self, c: Command) -> bool;
    fn save(&self) -> SaveData;
    fn status(&self) -> Status;
}

/// Det verten kan spørre om.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub weather: WeatherKind,
    pub daylight: Daylight,
    pub triangles: u32,
    pub calls: u32,
    /// Millisekunder på grafikkortet per del av bildet (tomt om det ikke
    /// kan måles).
    pub gpu_times: Vec<(&'static str, f32)>,
}

/// Et merke på skjermen (0..1 fra øverst til venstre): helsestolpe over en
/// fiende som er truffet, eller et skadetall som stiger.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    /// "bar" eller "text".
    pub kind: &'static str,
    /// Stolpens fyll, eller tekstens styrke, 0..1.
    pub value: f32,
    pub text: String,
}

/// Det spillet forteller verten.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Hva du sikter på, eller `None`.
    Prompt(Option<Prompt>),
    Daylight(Daylight),
    /// En kort melding.
    Toast(String),
    /// En kiste er åpnet: komponenten som lå i den (eller ingenting).
    Loot(Option<String>),
    Weather(String),
    /// Skjermen går i svart og tilbake (ned i krypten og opp igjen).
    Fade,
    /// Du reiser gjennom portalen (hjem med H, eller tilbake): en virvel
    /// over skjermen.
    Warp,
    /// Liv, utholdenhet, tokens, nøkler, om du er nede i kjelleren, og
    /// bossen om den har sett deg, når noe av det endrer seg.
    Hud {
        hp: u32,
        stamina: u32,
        tokens: u32,
        cellar: bool,
        keys: u32,
        boss: Option<(String, u32)>,
    },
    /// En lyd å spille (navnene står i `assets/sounds/sounds.json`).
    Sound(Sound),
    /// Merkene på skjermen dette bildet (tom liste når de er borte).
    Marks(Vec<Mark>),
    /// Snakkeboblene over innbyggerne dette bildet (tom liste når de er borte).
    Speech(Vec<Bubble>),
    /// Lageret og utstyret, når noe endrer seg (og en gang ved start).
    Inventory(InventoryView),
    /// E på en innbygger med butikk: varene.
    Shop {
        npc: String,
        wares: Vec<WareView>,
    },
    /// E i skriptoriet: oppskriftene som skjema.
    Workshop {
        recipes: Vec<RecipeView>,
    },
    /// Lydbildet der du står, når det endrer seg merkbart.
    Ambience(Ambience),
    /// Kartet over etasjen du er i (minikartet og det store kartet), når
    /// du har oppdaget mer eller bytter etasje; `None` oppe i byen.
    Map(Option<MapView>),
    /// Hvor du står og hvilken vei du ser i kjelleren (til minikartet).
    Where {
        x: f32,
        z: f32,
        yaw: f32,
    },
    /// Åpne dokumentasjonen for en komponent i panelet til høyre (F på en
    /// fylt ramme på galleriveggen).
    OpenDoc(String),
    /// Redigeringen etter hver endring; `None` når du går ut av den.
    Edit(Option<EditView>),
}

/// Kartet over et kryptnivå: omrisset (x, z) av rommene du har sett (fog
/// of war), og merkene i dem.
#[derive(Clone, Debug, PartialEq)]
pub struct MapView {
    /// Nivået (1–10) og navnet.
    pub level: u32,
    pub name: String,
    pub rooms: Vec<Vec<[f32; 2]>>,
    pub marks: Vec<MapMark>,
}

/// Et merke på kartet: «exit» (trappa ned), «ascend» (trappa opp), «gate»
/// og «gate_open», «chest», «shortcut», «lever» og «lever_pulled», «rest» og
/// «rest_lit», «boss».
#[derive(Clone, Debug, PartialEq)]
pub struct MapMark {
    pub kind: String,
    pub x: f32,
    pub z: f32,
}

/// En lyd: navnet (i `assets/sounds/sounds.json`), hvor den kommer fra sett
/// fra deg (meter; x til høyre, y opp, −z rett fram), eller `None` for lyder
/// i hodet (dine egne skritt, hugg og stønn), og hvor sterk den er (1 vanlig).
#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub name: String,
    pub at: Option<[f32; 3]>,
    pub gain: f32,
}

impl Sound {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), at: None, gain: 1.0 }
    }
}

impl PartialEq<&str> for Sound {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

/// Hvor sterkt hvert lydlag skal høres der du står (0–1): folk på torget,
/// hamring i smia, kirka (klokka), dyrene på enga, ild (ildsteder og
/// fakler), rennende vann, lava, vinden fra kløftene, regn og vind; om det er natt, om du er
/// i kjelleren (da er byen stille) eller innendørs (rommet gir etterklang).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ambience {
    pub square: f32,
    pub forge: f32,
    pub church: f32,
    pub meadow: f32,
    pub fire: f32,
    pub water: f32,
    pub lava: f32,
    pub chasm: f32,
    /// Den åpne portalen ved sørporten summer.
    pub portal: f32,
    pub rain: f32,
    pub wind: f32,
    pub night: bool,
    pub cellar: bool,
    pub indoor: bool,
}

/// Siktet: navnet på det du sikter på, hva F gjør, og hva E gjør (tom om
/// ingenting).
#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    pub label: String,
    pub verb: String,
    pub trade: String,
    /// En kort forklaring av hva dette er og hva du kan gjøre (vises i
    /// Fristil-tooltipen over siktet).
    pub info: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InventoryView {
    /// Trygt i byen, og det du bærer på turen (mistes om du dør).
    pub stash: Vec<(String, u32)>,
    pub carried: Vec<(String, u32)>,
    pub weapon: String,
    pub weapon_special: String,
    pub shield: String,
    pub armor: String,
    pub lantern: bool,
    pub max_hp: u32,
    pub max_stamina: u32,
    pub tokens: u32,
    /// Komponentene du har lest om i boka denne turen.
    pub read: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WareView {
    pub id: String,
    pub name: String,
    pub price: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecipeView {
    pub id: String,
    pub name: String,
    pub rarity: String,
    pub result: String,
    /// (komponent, trengs, har).
    pub needs: Vec<(String, u32, u32)>,
    /// (komponent, har).
    pub requires: Vec<(String, bool)>,
    /// (komponent, hvor den finnes, bare i krypten) for alle komponentene
    /// over.
    pub sources: Vec<(String, String, bool)>,
    pub ok: bool,
}

/// Det som lagres i nettleseren mellom øktene.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SaveData {
    pub stash: Vec<(String, u32)>,
    pub weapon: String,
    pub shield: String,
    pub armor: Option<String>,
    pub lantern: f32,
    pub max_hp: f32,
    pub max_stamina: f32,
    pub coins: u32,
    /// Hver komponent du noen gang har fått i byen (galleriveggen). Mangler
    /// i gamle lagringer, og da er det bare lageret som teller.
    pub found: Vec<String>,
    /// Den varige styrken du har tatt i krypten («nivå:nummer»), så den
    /// bare kan tas én gang.
    pub bonuses: Vec<String>,
}

/// En snakkeboble: over hvem (id), hvor på skjermen (0..1), navn og tekst.
#[derive(Clone, Debug, PartialEq)]
pub struct Bubble {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub text: String,
}
