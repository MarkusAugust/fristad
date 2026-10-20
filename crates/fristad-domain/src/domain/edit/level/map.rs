//! Kartet ovenfra til nettlaget (`web/nivaa.js`): modulene som ruter med
//! navn, åpningene etter sammenføyningen, innholdet med koblingene og
//! patruljene, lysene, reglene med sted og etasjene. Alt i nivåets rom
//! (meter; rutene er (i, y, j) med midten i (4i, y, 4j)). Byggesettet til
//! paletten står i `Palette`.

use super::{footprint, LevelEditor};
use crate::domain::world::crypt::compose::{modules, Severity, SIDES};
use crate::domain::world::crypt::doc::{DocOverride, LevelId, Mood};
use crate::domain::world::crypt::kit::KitDef;
use crate::domain::world::crypt::level::AmbushDef;
use crate::domain::world::crypt::stone::StoneRef;
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LevelMap {
    pub name: String,
    /// Nummeret (`1`–`10`) eller navnet.
    pub level: String,
    pub status: String,
    pub theme: String,
    pub seed: u64,
    pub stone: Option<StoneRef>,
    pub modules: Vec<MapModule>,
    pub edges: Vec<MapEdge>,
    pub things: Vec<MapThing>,
    pub links: Vec<MapLink>,
    pub lights: Vec<MapLight>,
    pub issues: Vec<MapIssue>,
    /// Gulvene som finnes (y i meter), lavest først: etasjene i kartet.
    pub floors: Vec<i32>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub unsaved: usize,
    /// Ingen feil: kan merkes «klar».
    pub clean: bool,
    pub stats: Stats,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Stats {
    pub cells: usize,
    pub recomputed: usize,
    pub instances: usize,
    pub colliders: usize,
    pub polys: usize,
    pub rooms: usize,
    pub lights: usize,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapModule {
    pub id: String,
    pub module: String,
    /// Navnet på rommet (eierens, ellers malens).
    pub name: String,
    pub kind: String,
    pub at: [i32; 3],
    pub rot: u8,
    pub mirror: bool,
    pub room: Option<String>,
    /// Stemningen som gjelder: eierens, ellers malens hovedrom.
    pub mood: Mood,
    /// Eieren har endret stemningen.
    pub own_mood: bool,
    pub stone: Option<StoneRef>,
    /// Rutene (i, y, j) modulen dekker.
    pub cells: Vec<[i32; 3]>,
    pub overrides: Vec<DocOverride>,
    pub openings: Vec<MapOpening>,
    /// Malens innhold eieren har fjernet i dette nivået (malens id-er), så
    /// det kan settes tilbake.
    pub removed: Vec<String>,
}

/// En åpning i en modul satt ut: ruta, gulvet, siden i nivået (0 N, 1 Ø,
/// 2 S, 3 V), malens slag og eierens valg.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapOpening {
    pub id: String,
    pub at: [i32; 2],
    pub y: i32,
    pub side: usize,
    pub kind: String,
    pub chosen: Option<String>,
}

/// En kant som ikke er mur etter sammenføyningen.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapEdge {
    pub at: [i32; 3],
    pub side: usize,
    /// `arch`, `gang`, `door`, `gate`, `secret`.
    pub kind: String,
    pub switch: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapThing {
    pub id: String,
    pub kind: String,
    pub label: Option<String>,
    pub pos: [f32; 3],
    /// Grader.
    pub yaw: f32,
    pub n: Option<u32>,
    pub patrol: Vec<[f32; 3]>,
    /// `av`, radius i meter, eller rommet.
    pub ambush: Option<String>,
    pub loot: Option<String>,
    pub text: Option<String>,
    /// I dokumentet (kan endres og flyttes); ellers fra en modul.
    pub own: bool,
    /// Modulen det kom fra (de som ikke er eierens).
    pub module: Option<String>,
    /// Den andre enden av en stige (toppen) eller heis (bunnen), som kobler
    /// etasjene.
    pub other_end: Option<[f32; 3]>,
    /// Malens innhold eieren har flyttet eller endret: modulen det hører til
    /// (så det kan settes tilbake slik malen har det).
    pub template: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapLink {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub own: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapLight {
    pub id: String,
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub range: f32,
    /// Styrken (2 om den ikke er satt) og om den blafrer som en flamme.
    pub intensity: f32,
    pub flicker: bool,
    pub own: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MapIssue {
    /// `feil` eller `advarsel`.
    pub severity: &'static str,
    pub rule: &'static str,
    pub msg: String,
    pub at: Option<[f32; 3]>,
    /// «Rett»: rettingen når den er entydig (`fixes::suggest`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<super::fixes::Fix>,
}

impl LevelEditor {
    /// Kartet slik nivået står nå.
    pub fn map(&self) -> LevelMap {
        let doc = self.doc();
        let kit = self.kit();
        let file = self.file();
        let level = match &doc.level {
            LevelId::N(n) => n.to_string(),
            LevelId::Name(s) => s.clone(),
        };
        let mods: Vec<MapModule> = doc
            .modules
            .iter()
            .map(|m| {
                let def = kit.module(&m.module);
                let name = m.name.clone().or_else(|| def.map(|d| d.name.clone())).unwrap_or_else(|| m.id.clone());
                let openings = modules::placed_openings(m, kit)
                    .into_iter()
                    .map(|o| MapOpening { chosen: m.overrides.iter().find(|v| v.opening == o.id).map(|v| v.kind.clone()), id: o.id, at: o.at, y: o.y, side: o.side, kind: o.kind })
                    .collect();
                MapModule {
                    id: m.id.clone(),
                    module: m.module.clone(),
                    name,
                    kind: def.map(|d| d.kind.clone()).unwrap_or_default(),
                    at: m.at,
                    rot: m.rot % 4,
                    mirror: m.mirror,
                    room: m.room.clone(),
                    mood: m.mood.clone().or_else(|| def.and_then(|d| d.rooms.first()).map(|r| r.mood.clone())).unwrap_or_default(),
                    own_mood: m.mood.is_some(),
                    stone: m.stone.clone(),
                    cells: footprint(m, kit),
                    overrides: m.overrides.clone(),
                    openings,
                    removed: m.hidden.iter().filter(|h| !doc.things.iter().any(|t| t.id == format!("{}/{h}", m.id))).cloned().collect(),
                }
            })
            .collect();
        let edges = self.report().edges.iter().map(|e| MapEdge { at: e.at, side: e.side, kind: e.kind.to_string(), switch: e.switch.clone() }).collect();
        let own_thing = |id: &str| doc.things.iter().any(|t| t.id == id);
        let things = file
            .marks
            .iter()
            .filter_map(|m| {
                let id = m.id.clone()?;
                let own = own_thing(&id);
                let module = (!own).then(|| id.split('/').next().unwrap_or("").to_string()).filter(|s| doc.modules.iter().any(|x| x.id == *s));
                Some(MapThing {
                    kind: m.kind.clone(),
                    label: m.label.clone(),
                    pos: m.pos.unwrap_or_default(),
                    yaw: m.yaw.to_degrees(),
                    n: m.n,
                    patrol: m.patrol.clone(),
                    ambush: m.ambush.as_ref().map(|a| match a {
                        AmbushDef::On(b) => if *b { "ja" } else { "av" }.to_string(),
                        AmbushDef::Radius(r) => format!("{r}"),
                        AmbushDef::Room(r) => r.clone(),
                    }),
                    loot: m.loot.clone(),
                    text: m.text.clone(),
                    own,
                    module,
                    other_end: match m.kind.as_str() {
                        "ladder" => m.top,
                        "lift" => m.bottom,
                        _ => None,
                    },
                    template: id.split_once('/').filter(|(m, l)| own && doc.modules.iter().any(|x| x.id == *m && x.hidden.iter().any(|h| h == l))).map(|(m, _)| m.to_string()),
                    id,
                })
            })
            .collect();
        let flat = self.flat();
        let links = flat.links.iter().map(|l| MapLink { from: l.from.clone(), to: l.to.clone(), kind: l.kind.clone(), own: doc.links.contains(l) }).collect();
        let lights = file
            .lights
            .iter()
            .zip(flat.lights.iter().map(|l| (l.id.clone(), doc.lights.iter().any(|d| d.id == l.id))).chain(std::iter::repeat((String::new(), false))))
            .map(|(l, (id, own))| MapLight { id, pos: l.pos, color: l.color, range: l.range, intensity: l.intensity.unwrap_or(2.0), flicker: l.flicker, own })
            .collect();
        let issues = self
            .issues()
            .iter()
            .enumerate()
            .map(|(k, i)| MapIssue { severity: if i.severity == Severity::Error { "feil" } else { "advarsel" }, rule: i.rule, msg: i.msg.clone(), at: i.at, fix: self.fix(k).cloned() })
            .collect();
        let mut floors: Vec<i32> = mods.iter().flat_map(|m: &MapModule| m.cells.iter().map(|c| c[1])).collect();
        floors.sort();
        floors.dedup();
        let r = self.report();
        LevelMap {
            name: doc.name.clone(),
            level,
            status: doc.status.clone(),
            theme: doc.theme.clone(),
            seed: doc.seed,
            stone: doc.stone.clone(),
            modules: mods,
            edges,
            things,
            links,
            lights,
            issues,
            floors,
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
            unsaved: self.unsaved,
            clean: self.clean(),
            stats: Stats {
                cells: r.cells,
                recomputed: r.recomputed,
                instances: file.instances.len(),
                colliders: file.colliders.len(),
                polys: file.nav.polys.len(),
                rooms: file.rooms.len(),
                lights: file.lights.len(),
            },
        }
    }
}

/// Byggesettet til paletten: modulene med cellene og åpningene i sitt eget
/// rom, steinene, slagene for åpningene, innholdet og fiendene.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Palette {
    pub modules: Vec<PaletteModule>,
    pub stones: Vec<(String, String)>,
    pub themes: Vec<String>,
    pub openings: Vec<(&'static str, &'static str)>,
    pub things: Vec<(&'static str, &'static str)>,
    pub mobs: Vec<(String, String)>,
    pub loot: Vec<String>,
    pub links: Vec<(&'static str, &'static str)>,
    /// Gruppene i paletten (`GROUPS`).
    pub groups: Vec<(&'static str, &'static str)>,
    /// Lydene et rom kan ha (`doc::SOUNDS`).
    pub sounds: Vec<(&'static str, &'static str)>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PaletteModule {
    pub id: String,
    pub name: String,
    pub theme: String,
    pub kind: String,
    /// Gruppen i paletten: `rom`, `gang`, `trapp`, `ferdig`, `mekanisme`
    /// eller `bosshall` (`palette_group`).
    pub group: &'static str,
    pub about: String,
    pub cells: Vec<[i32; 3]>,
    pub openings: Vec<MapOpening>,
}

/// Gruppene i paletten i rekkefølge, med navnet eieren ser.
pub const GROUPS: [(&str, &str); 6] = [("rom", "Rom"), ("gang", "Ganger"), ("trapp", "Trapper"), ("ferdig", "Ferdigrom"), ("mekanisme", "Mekanismer"), ("bosshall", "Bosshall")];

/// Innholdet som gjør et ferdig rom til en mekanisme (spaker, porter med
/// nøkkel, feller, snarveier, stiger, broer, heiser, sluser og hvilesteder).
const MECHANISM: [&str; 10] = ["lever", "trap", "shortcut", "ladder", "trapdoor", "bridge", "lift", "water", "rest", "elite"];

/// Gruppen en modul står i: bosshallen har en boss, mekanismene har noe som
/// virker (`MECHANISM`), ellers modulens slag.
pub fn palette_group(m: &crate::domain::world::crypt::doc::ModuleDef) -> &'static str {
    if m.things.iter().any(|t| t.kind == "boss") {
        return "bosshall";
    }
    if m.things.iter().any(|t| MECHANISM.contains(&t.kind.as_str())) {
        return "mekanisme";
    }
    GROUPS.iter().map(|(g, _)| *g).find(|g| *g == m.kind).unwrap_or("ferdig")
}

/// Slagene for åpningene med navnet eieren ser.
pub const OPENING_NAMES: [(&str, &str); 8] =
    [("auto", "Automatisk"), ("dor", "Dør"), ("gang", "Gang"), ("bue", "Bue"), ("port", "Port"), ("falsk", "Falsk vegg"), ("snarvei", "Snarvei"), ("vegg", "Mur")];

/// Innholdet eieren kan sette ut.
pub const THING_NAMES: [(&str, &str); 14] = [
    ("start", "Startstedet"),
    ("up", "Døra opp"),
    ("down", "Trappa ned"),
    ("rest", "Hvilestedet"),
    ("mob", "Fiende"),
    ("elite", "Elite (bærer nøkkelen)"),
    ("boss", "Boss"),
    ("chest", "Kiste"),
    ("rare", "Sjelden kiste"),
    ("lore", "Lapp"),
    ("bonus", "Bonus"),
    ("lever", "Spak"),
    ("trap", "Felle (trykkplate)"),
    ("hint", "Hint"),
];

/// Koblingene med navnet eieren ser.
pub const LINK_NAMES: [(&str, &str); 6] = [("opens", "Åpner"), ("key", "Nøkkel til"), ("arena", "Arena (rommet)"), ("exit", "Utgang fra arenaen"), ("for", "Hint til"), ("from", "Slåen mot rommet")];

/// Paletten fra byggesettet; `mobs` er (nøkkel, navn) for fiendeslagene og
/// `loot` komponentene en kiste kan gi.
pub fn palette(kit: &KitDef, mobs: Vec<(String, String)>, loot: Vec<String>) -> Palette {
    let mut list: Vec<PaletteModule> = kit
        .modules
        .iter()
        .map(|m| {
            let inst = crate::domain::world::crypt::doc::DocModule { id: "m".into(), module: m.id.clone(), ..Default::default() };
            let openings = modules::placed_openings(&inst, kit).into_iter().map(|o| MapOpening { id: o.id, at: o.at, y: o.y, side: o.side, kind: o.kind, chosen: None }).collect();
            PaletteModule {
                id: m.id.clone(),
                name: m.name.clone(),
                theme: m.theme.clone(),
                kind: m.kind.clone(),
                group: palette_group(m),
                about: m.about.clone(),
                cells: footprint(&inst, kit),
                openings,
            }
        })
        .collect();
    let order = |k: &str| GROUPS.iter().position(|(x, _)| *x == k).unwrap_or(9);
    list.sort_by(|a, b| (order(a.group), &a.id).cmp(&(order(b.group), &b.id)));
    let mut themes: Vec<String> = kit.modules.iter().map(|m| m.theme.clone()).filter(|t| !t.is_empty()).collect();
    themes.sort();
    themes.dedup();
    Palette {
        modules: list,
        stones: kit.stones.stones.iter().map(|s| (s.id.clone(), if s.name.is_empty() { s.id.clone() } else { s.name.clone() })).collect(),
        themes,
        openings: OPENING_NAMES.to_vec(),
        things: THING_NAMES.to_vec(),
        mobs,
        loot,
        links: LINK_NAMES.to_vec(),
        groups: GROUPS.to_vec(),
        sounds: crate::domain::world::crypt::doc::SOUNDS.to_vec(),
    }
}

/// Siden som bokstav (til meldinger).
pub fn side_name(w: usize) -> &'static str {
    SIDES[w % 4]
}
