//! Den drivende porten: det nettleseren, vinduet og skjermbildeverktøyet
//! sender inn til spillet, og det de får tilbake. `application::App`
//! implementerer `Play`; adapterne kjenner bare denne fila.

use crate::domain::edit::{LineKind, Prop, Selected};
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
    /// Bytt komponenter hos Buttonsmeden (`loot::SWAPS` etter id). Gir om
    /// det gikk; butikken sendes på nytt med det du har nå.
    Swap(String),
    /// Kjøp trubaduren et krus mjød (1 token), så han synger kvadet om
    /// Gorvek (`Event::Song`). Gir om det gikk.
    BuyMead,
    /// Sangen er spilt til endes i nettlaget (`tavern.mp3` er ferdig).
    SongOver,
    /// Ta på et våpen, skjold eller en rustning du har (fra inventaret).
    Equip(String),
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
    /// Nivåredigeringen (`?nivaa`, `docs/nivaaredigering.md` del 12).
    Level(LevelCommand),
    /// Filmen (`web/film/`): kameraet står der filmen sier, uten armer,
    /// kamp og fysikk, mens byfolket og fiendene lever videre. `None` gir
    /// spillet tilbake.
    Film(Option<Shot>),
    /// Hvor sterkt trubaduren synger nå (0..1, sangens lydstyrke der
    /// sangen er, `assets/music/game/tavern_voice.json`): munnen følger den.
    Voice(f32),
    /// Lysstyrken spilleren har valgt (gamma, 0,6–2; 1 er som malt).
    Gamma(f32),
}

/// Ett bilde i filmen: øyet, retningen og synsfeltet (grader).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub eye: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    /// Krengning (radianer, positiv mot venstre), så kameraet kan legge seg i svingene.
    pub roll: f32,
}

/// Det eieren gjør i nivåredigeringen (`?nivaa`).
#[derive(Clone, Debug, PartialEq)]
pub enum LevelCommand {
    /// En endring av dokumentet (`LevelOp`, som json fra kartet).
    Op(crate::domain::edit::level::LevelOp),
    Undo,
    Redo,
    /// Åpne et annet dokument (historikken tømmes).
    Open(Box<crate::domain::world::crypt::doc::LevelDoc>),
    /// Lagret: telleren over endringer nullstilles.
    Saved,
    /// Det valgte på kartet (modulene og innholdet), markert i 3D.
    Select(Vec<String>),
    /// Arbeidslys (lyst og flatt) eller spillets lys.
    WorkLight(bool),
    /// Gulvet kartet viser (y i dokumentet).
    Floor(f32),
    /// Snittet (S): alt over etasjen med gulvet i y (i dokumentet) kuttes
    /// bort i 3D, så du ser inn i en etasje under en annen; `None` av.
    Cut(Option<f32>),
    /// Kameraet i 3D-vinduet: øyet i dokumentets rom.
    Place { at: Vec3, yaw: f32, pitch: f32 },
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
    /// Gå til steget `n` i historikken (0 er slik det var fra start).
    UndoTo(usize),
    /// Velg nummer `k` i lista over det som lå under pekeren (`add`: Shift).
    SelectUnder {
        k: usize,
        add: bool,
    },
    /// Shift+Tab: det forrige under pekeren.
    CycleBack,
    /// Velg alt med midten innenfor rammen på skjermen (Shift+dra i tomt).
    SelectBox {
        a: (f32, f32),
        b: (f32, f32),
        add: bool,
    },
    /// Fantomkopien av en modell under pekeren før den settes ut (dra fra
    /// katalogen), eller ingen.
    Ghost(Option<(String, (f32, f32))>),
    /// Drei fantomet (R: 90°), radianer.
    GhostTurn(f32),
    /// Fest til rutenettet (0,5 m) og vinkelen (15°): bryteren i panelet,
    /// eller Ctrl holdt under et drag.
    Snap(bool),
    /// Sett en egenskap på det valgte (`Prop.key`, verdien som tekst).
    SetProp {
        key: String,
        value: String,
    },
    /// Lag en gruppe av det valgte med navnet (tomt: «gruppe N»).
    Group(String),
    /// Løs opp gruppene i det valgte.
    Ungroup,
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
    /// Egenskapene til det valgte (den første tingen).
    pub props: Vec<Prop>,
    /// Historikken (eldst først) og hvor mange steg som er gjort.
    pub history: Vec<String>,
    pub step: usize,
    /// Festet til rutenett og vinkel er på.
    pub snap: bool,
    /// Fantomet under pekeren og det som er galt med det der (tomt er grønt).
    pub ghost: Option<Vec<String>>,
    /// Feilen fra siste egenskap som ikke kunne settes.
    pub error: Option<String>,
    /// Gruppen til det valgte, om det er én gruppe.
    pub group: Option<String>,
    /// Punktet på ruta til den valgte innbyggeren som er tatt (Delete
    /// sletter det), nummerert fra 0.
    pub route_point: Option<usize>,
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
    /// En lapp eller bok i krypten er lest (F på `kit_lore`): tittelen og
    /// teksten, vist som et kort midt på skjermen med spillet på pause.
    Lore {
        title: String,
        text: String,
    },
    /// Fienden du sikter på (infopanelet til høyre), sendt når den eller
    /// livet endrer seg; `None` når du ikke sikter på noen.
    Foe(Option<FoeView>),
    /// Du ble truffet: hvor slaget eller skuddet kom fra, som vinkel i
    /// radianer fra blikket (0 rett foran, positiv til høyre, ±π bak).
    /// Nettlaget viser det som en rød bue i den kanten av bildet.
    Hurt {
        angle: f32,
    },
    /// Målet nå (én linje øverst til venstre) og journalen (J): lappene og
    /// bøkene du har lest i krypten, tittel og tekst. Sendes når noe endres.
    Goal {
        text: String,
        journal: Vec<(String, String)>,
    },
    /// Det som fulgte med i kista utenom kistas egen komponent (plan 14),
    /// sendt rett før `Loot`.
    Haul(Vec<String>),
    /// En kiste er åpnet: komponenten som lå i den (eller ingenting).
    Loot(Option<String>),
    Weather(String),
    /// Skjermen går i svart og tilbake (ned i krypten og opp igjen).
    Fade,
    /// Hent bitene til kryptnivå n (1 og oppover): nettleseren laster dem
    /// når økta trenger nivået, og svarer med `add_level`.
    FetchLevel(u32),
    /// Hent disse modellene (navnene; plan del 10a): byens modeller som
    /// ikke trengs til første bilde, når økta trenger dem (utstyret du har
    /// på straks, innsiden og møblene når du nærmer deg, fiendene med
    /// nivået). Nettleseren svarer med `add_models`.
    FetchModels(Vec<String>),
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
        /// Oljen i lykta, 0–100.
        light: u32,
        /// Kampen nå (`Fight`): 0 ingen, 1 en fiende, 2 en boss, 3
        /// Rammeverket. Musikken toner inn kampsporet etter det.
        fight: u8,
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
        /// Byttene (fanen «Bytte», bare hos Buttonsmeden; tom ellers).
        swaps: Vec<SwapView>,
    },
    /// E i skriptoriet: oppskriftene som skjema.
    Workshop {
        recipes: Vec<RecipeView>,
    },
    /// E på trubaduren: tilbudet om et krus mjød (en egen liten dialog).
    Mead(MeadView),
    /// Trubaduren begynner kvadet om Gorvek fra start (`true`, spilles én
    /// gang, ikke i løkke), eller det er over (`false`).
    Song(bool),
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

/// Fienden du sikter på: navnet, en kort tekst om hva den er (fra kanon og
/// slaget), om den er bossen eller eliten, og livet.
#[derive(Clone, Debug, PartialEq)]
pub struct FoeView {
    pub name: String,
    pub info: String,
    /// «Boss», «Elite» eller tomt.
    pub rank: String,
    pub hp: u32,
    pub max_hp: u32,
    /// Skyter på avstand (inputen, Toast: slagene med markører).
    pub ranged: bool,
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
    /// Vertshuset: 1 i skjenkestua der trubaduren sitter, `TAVERN_UPPER`
    /// og `TAVERN_CELLAR` i overetasjen og kjelleren, under 0,5 og
    /// avtagende utenfor døra. Inne i huset (≥ 0,6) tones byens spor ned;
    /// synger trubaduren (`Event::Song`), spiller `tavern` så sterkt.
    pub tavern: f32,
    pub rain: f32,
    pub wind: f32,
    pub night: bool,
    pub cellar: bool,
    pub indoor: bool,
}

/// Trubadurens tilbud: det han sier, prisen, og om han synger allerede
/// (da er knappen av, så ingen betaler to ganger).
#[derive(Clone, Debug, PartialEq)]
pub struct MeadView {
    pub npc: String,
    pub text: String,
    pub price: u32,
    pub singing: bool,
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
    /// Alt utstyret du har (det du har på er merket), som kort.
    pub gear: Vec<GearView>,
}

/// Et våpen, skjold, en rustning eller lykta som kort (inventaret og
/// galleriet i skriptoriet). Bildet er `icons/<id>.png`.
#[derive(Clone, Debug, PartialEq)]
pub struct GearView {
    pub id: String,
    pub name: String,
    /// «weapon», «shield», «armor» eller «lantern».
    pub slot: String,
    pub rarity: String,
    pub special: String,
    /// Én linje om tingen etter kanon (`craft::about`).
    pub about: String,
    /// (navn, verdi, tekst, høyere er bedre).
    pub stats: Vec<(String, f32, String, bool)>,
    pub equipped: bool,
    pub owned: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WareView {
    pub id: String,
    pub name: String,
    pub price: u32,
}

/// Et bytte hos Buttonsmeden: gi `n` av `give` (du har `have`) og `fee`
/// tokens, få én `get` (du har `get_have`). Sjeldenhetene, hvor `get` ellers
/// finnes (kort), og om du kan bytte nå.
#[derive(Clone, Debug, PartialEq)]
pub struct SwapView {
    pub id: String,
    pub give: String,
    pub n: u32,
    pub have: u32,
    pub give_rarity: String,
    pub get: String,
    pub get_have: u32,
    pub get_rarity: String,
    pub fee: u32,
    pub source: String,
    pub ok: bool,
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
    /// (komponent, hvor den finnes kort på én linje, bare i krypten) for
    /// alle komponentene over.
    pub sources: Vec<(String, String, bool)>,
    pub ok: bool,
    /// Tingen oppskriften gir, som kort (med om du har den).
    pub item: GearView,
    /// Det du har på i samme spor, til sammenligningen.
    pub current: Option<GearView>,
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
    /// Oljen i lykta (sekunder); mangler i gamle lagringer (full lykt).
    pub oil: Option<f32>,
    /// Det dypeste nivået du har nådd (0 er nivå 1), så målet og portalen
    /// husker det; mangler i gamle lagringer.
    pub deepest: Option<u32>,
    /// Titlene på lappene og bøkene du har lest i krypten (journalen).
    pub lore: Vec<String>,
    /// Alt utstyret du har (id-ene); mangler i gamle lagringer, og da er
    /// det du har på og det du startet med.
    pub owned: Vec<String>,
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
