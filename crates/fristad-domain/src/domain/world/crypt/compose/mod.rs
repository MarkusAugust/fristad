//! Kompilatoren for nivåredigeringen (`docs/nivaaredigering.md`, del 5 og
//! 7): gjør nivådokumentet (`LevelDoc`) og byggesettet (`KitDef`) om til den
//! samme `LevelFile` som motoren bruker for de gamle nivåene, med bitene som
//! instanser (`instances`), kolliderne flettet, navigasjonsnettet, rommene,
//! lysene og merkene. Ingen geometri lages her: kompilatoren velger bare
//! hvilke ekte modeller som står hvor. Redigeringen, spillet, `cargo test`
//! og `snapshot` kjører den samme koden.
//!
//! 0. Modulene utvides til celler, rom, innhold og kantvalg, og åpningene
//!    der to moduler møtes, føyes sammen (`modules`, del 21).
//! 1. Cellene slås opp i rutenettet; sidene og gulvene dreies og speiles.
//! 2. Hver kant får en åpning (sidenes ønsker og eierens valg, tabellen i
//!    del 5.1) og en halvkant per side; hvert hjørne et kvarthjørne der en
//!    mur når hjørnet eller fjellet står på skrå, ellers en søyle der fire
//!    romfelt møtes.
//! 3. Kolliderne er bitenes egne, flyttet på plass og flettet (`solids`).
//! 4. Nettet rastres i ruter på 0,2 m per plan og slås sammen til
//!    rektangler; trappene er skrå firkanter (`nav`).
//! 5. Rommene er omrisset av cellene (`rooms`), merkene og lysene kommer
//!    fra innholdet med koblingene.
//!
//! Resultatet er likt byte for byte for samme dokument (tallene avrundes
//! til mm). `Compiler` husker hver celle med naboene: bare cellene der noe i
//! nabolaget er endret, regnes på nytt.

pub mod board;
pub mod check;
mod grid;
mod look;
pub mod modules;
mod nav;
mod rooms;
mod solids;

pub use grid::*;
pub use look::{expand, roll};
pub use nav::reach;
use look::{anchor_point, cell_at, cell_look, cell_variant_id, cell_stone, edge_stone, lights, side_wall, torches, wall_variant_fits};

use super::doc::{DocCell, LevelDoc, Thing};
use super::kit::{KitDef, KitPiece, PieceKind};
use super::level::{ColliderDef, InstanceDef, LevelFile, LightDef, MarkDef};
use glam::{Vec2, Vec3};
use std::collections::{BTreeMap, HashMap};

/// Hvor alvorlig et brudd er: feil stopper «klar», advarsler ingenting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Et brudd på reglene, med sted (i nivåets rom) når det finnes.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub severity: Severity,
    /// `kant`, `tett`, `overlapp`, `bit`, `kobling`, `innhold`, `rom`.
    pub rule: &'static str,
    pub at: Option<[f32; 3]>,
    pub msg: String,
}

impl Issue {
    fn error(rule: &'static str, at: Option<Vec3>, msg: String) -> Self {
        Issue { severity: Severity::Error, rule, at: at.map(mm3), msg }
    }
    pub(super) fn warn(rule: &'static str, at: Option<Vec3>, msg: String) -> Self {
        Issue { severity: Severity::Warning, rule, at: at.map(mm3), msg }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub issues: Vec<Issue>,
    /// Cellene som ble regnet på nytt (resten kom fra minnet).
    pub recomputed: usize,
    pub cells: usize,
    /// Åpningene slik de ble (til kartet i redigeringen): én per kant som
    /// ikke er mur, med gulvet og siden i nivået.
    pub edges: Vec<EdgeOpen>,
}

/// En åpning i nivået etter sammenføyningen: ruta (i, gulvet, j), siden i
/// nivået, slaget (`Open::kit_name`) og bryteren (porten, snarveien eller
/// den falske veggen) om den har en.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeOpen {
    pub at: [i32; 3],
    pub side: usize,
    pub kind: &'static str,
    pub switch: Option<String>,
}

impl Report {
    pub fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(|i| i.severity == Severity::Error)
    }
}

/// Kompiler et dokument fra bunnen av.
pub fn compile(doc: &LevelDoc, kit: &KitDef) -> (LevelFile, Report) {
    Compiler::default().compile(doc, kit)
}

/// Åpningen i en kant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Open {
    Wall,
    Arch,
    Gang,
    Door,
    Gate,
    Secret,
    /// Inne i en sammensatt bit: ingen halvkant, ingen mur.
    Inner,
}

impl Open {
    /// Halvkantens åpning i byggesettet (`opening`), og bredden.
    pub fn kit_name(self) -> &'static str {
        match self {
            Open::Wall => "wall",
            Open::Arch => "arch",
            Open::Gang => "gang",
            Open::Door => "door",
            Open::Gate => "gate",
            Open::Secret => "secret",
            Open::Inner => "inner",
        }
    }
    pub fn width(self) -> f32 {
        match self {
            Open::Wall | Open::Arch | Open::Inner => 0.0,
            Open::Gang => GANG_W,
            Open::Door => DOOR_W,
            Open::Gate => GATE_W,
            Open::Secret => SECRET_W,
        }
    }
}

/// En celle med biten slått opp.
#[derive(Clone)]
pub(crate) struct Cell<'a> {
    pub doc: &'a DocCell,
    pub piece: &'a KitPiece,
    pub i: i32,
    pub j: i32,
    pub y: f32,
    pub theme: String,
    /// Lagene cella får (`crypt::stone`).
    pub stone: super::stone::Surfaces,
}

impl Cell<'_> {
    pub fn rot(&self) -> u8 {
        self.doc.rot % 4
    }
    pub fn mirror(&self) -> bool {
        self.doc.mirror && self.piece.mirror
    }
    pub fn center(&self) -> Vec3 {
        cell_center(self.i, self.y, self.j)
    }
    pub fn yaw(&self) -> f32 {
        cell_yaw(self.rot())
    }
    /// Et punkt i bitens rom, i nivåets rom.
    pub fn point(&self, p: Vec3) -> Vec3 {
        transform(self.center(), self.yaw(), self.mirror()).transform_point3(p)
    }
    /// En dreining i bitens rom (radianer), i nivåets rom.
    pub fn turn(&self, yaw: f32) -> f32 {
        (if self.mirror() { -yaw } else { yaw }) + self.yaw()
    }
    pub fn face(&self, w: usize) -> (&str, &str) {
        let l = world_to_local(w, self.rot(), self.mirror());
        self.piece.faces.get(l).map(|f| (f.kind.as_str(), f.profile.as_str())).unwrap_or(("closed", "gang"))
    }
    /// Gulvet ved kanten på siden `w` (absolutt).
    pub fn floor(&self, w: usize) -> f32 {
        self.y + self.piece.floors[world_to_local(w, self.rot(), self.mirror())]
    }
    pub fn volume(&self) -> (f32, f32) {
        (self.y + self.piece.bottom, self.y + self.piece.top.max(self.piece.bottom + 0.5))
    }
}

/// Det én celle bidrar med (det som huskes).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CellOut {
    pub instances: Vec<InstanceDef>,
    pub boxes: Vec<solids::Aabb>,
    pub hulls: Vec<Vec<[f32; 3]>>,
    pub raster: Vec<((i32, i32, i32), nav::Label)>,
    pub slopes: Vec<(Vec<[f32; 3]>, nav::Label)>,
    pub lights: Vec<LightDef>,
    pub issues: Vec<Issue>,
}

/// Kompilatoren med minnet per celle.
#[derive(Default)]
pub struct Compiler {
    kit: Option<KitDef>,
    cache: HashMap<String, (String, CellOut)>,
}

/// Rutenettet og kantvalgene, felles for alle cellene i én kompilering.
pub(crate) struct Grid<'a> {
    pub cells: Vec<Cell<'a>>,
    pub columns: HashMap<(i32, i32), Vec<usize>>,
    /// Eierens valg per (celle, side i nivået): slaget og bryteren (porter,
    /// snarveier og falske vegger som er innhold på kanten).
    pub overrides: HashMap<(usize, usize), (String, Option<String>)>,
    /// Utseendet eieren har låst per (celle, side i nivået).
    pub edge_variants: HashMap<(usize, usize), String>,
    /// Steinen eieren har valgt per (celle, side i nivået).
    pub edge_stones: HashMap<(usize, usize), super::stone::StoneRef>,
    pub kit: &'a KitDef,
    /// Nivåets frø (variantene og faklene).
    pub seed: u64,
    /// Fakkeltettheten per rom.
    pub torches: HashMap<String, f32>,
}

impl<'a> Grid<'a> {
    /// Cella på andre siden av `w` med gulvet i samme høyde.
    pub fn neighbor(&self, a: usize, w: usize) -> Option<usize> {
        let c = &self.cells[a];
        let (di, dj) = outward(w);
        let h = c.floor(w);
        self.columns.get(&(c.i + di, c.j + dj))?.iter().copied().find(|&b| (self.cells[b].floor(opposite(w)) - h).abs() < 0.01)
    }

    /// Cellene i ruta (i, j) som overlapper cella `a` i høyden.
    pub fn overlapping(&self, a: usize, at: (i32, i32)) -> Vec<usize> {
        let (lo, hi) = self.cells[a].volume();
        self.columns
            .get(&at)
            .map(|v| {
                v.iter()
                    .copied()
                    .filter(|&b| {
                        b != a && {
                            let (l, h) = self.cells[b].volume();
                            l < hi - 0.01 && lo < h - 0.01
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn override_at(&self, a: usize, w: usize, b: Option<usize>) -> Option<&(String, Option<String>)> {
        self.overrides.get(&(a, w)).or_else(|| b.and_then(|b| self.overrides.get(&(b, opposite(w)))))
    }

    /// Åpningen i kanten på siden `w` av cella `a` (og bryteren for porter,
    /// snarveier og falske vegger), med brudd.
    pub fn opening(&self, a: usize, w: usize) -> (Open, Option<String>, Vec<Issue>) {
        let c = &self.cells[a];
        let b = self.neighbor(a, w);
        let mid = c.center() + Vec3::new(outward_v(w).x, 0.0, outward_v(w).y) * HALF;
        let mut issues = vec![];
        let (fa, pa) = c.face(w);
        // Inne i en sammensatt bit.
        if fa == "inner" && b.is_some_and(|b| self.cells[b].face(opposite(w)).0 == "inner") {
            return (Open::Inner, None, issues);
        }
        if let Some((kind, switch)) = self.override_at(a, w, b) {
            if kind == "vegg" {
                return (Open::Wall, None, issues);
            }
            let Some(b) = b else {
                issues.push(Issue::error("kant", Some(mid), format!("«{kind}» i {} mot {} går ingen vei (fjell eller et annet gulv)", c.doc.id, SIDES[w])));
                return (Open::Wall, None, issues);
            };
            let (fb, pb) = self.cells[b].face(opposite(w));
            if fa == "closed" && fb == "closed" && (kind == "bue" || kind == "aapen" || kind == "gang") {
                issues.push(Issue::error("kant", Some(mid), format!("«{kind}» mellom {} og {}: begge sidene er lukket", c.doc.id, self.cells[b].doc.id)));
                return (Open::Wall, None, issues);
            }
            let open = match kind.as_str() {
                "dor" | "snarvei" => Open::Door,
                "port" => {
                    if pa != "room" || pb != "room" {
                        issues.push(Issue::warn("kant", Some(mid), format!("port mellom {} og {}: porten er 3 m høy og trenger romhvelv på begge sider", c.doc.id, self.cells[b].doc.id)));
                    }
                    Open::Gate
                }
                "falsk" => Open::Secret,
                // Gangåpningen (mellom to moduler: et rom og en gang, eller to ganger).
                "gang" => Open::Gang,
                "bue" | "aapen" => {
                    if fa == "gang" || fb == "gang" {
                        Open::Gang
                    } else {
                        Open::Arch
                    }
                }
                other => {
                    issues.push(Issue::error("kant", Some(mid), format!("ukjent kant «{other}» i {}", c.doc.id)));
                    Open::Wall
                }
            };
            return (open, switch.clone(), issues);
        }
        let Some(b) = b else {
            // En trapp som ender i vegg eller lufta.
            // (Gangen ut fra galleriet i en sammensatt bit er ingen trapp.)
            let stair = c.piece.part_of.is_none() && c.piece.floors.iter().any(|f| (f - c.piece.floors[0]).abs() > 0.01);
            if stair && fa == "gang" {
                issues.push(Issue::error("kant", Some(mid), format!("trappa {} ender i vegg mot {}", c.doc.id, SIDES[w])));
            }
            return (Open::Wall, None, issues);
        };
        let (fb, _) = self.cells[b].face(opposite(w));
        let open = match (fa, fb) {
            ("closed", _) | (_, "closed") | ("inner", _) | (_, "inner") => Open::Wall,
            ("open", "open") => {
                if c.doc.room == self.cells[b].doc.room {
                    Open::Arch
                } else {
                    Open::Wall
                }
            }
            _ => Open::Gang,
        };
        (open, None, issues)
    }
}

impl Compiler {
    pub fn compile(&mut self, doc: &LevelDoc, kit: &KitDef) -> (LevelFile, Report) {
        if self.kit.as_ref() != Some(kit) {
            self.cache.clear();
            self.kit = Some(kit.clone());
        }
        let mut report = Report::default();
        // Modulene først: etter dette er dokumentet celler og kantvalg.
        let (flat, mut issues) = modules::flatten(doc, kit);
        report.issues.append(&mut issues);
        // Eierens egne lys står først (`flatten` beholder dem foran modulenes).
        let own_lights = doc.lights.len();
        let doc = &flat;
        // Rommene: de i dokumentet, og de cellene nevner uten at de står der.
        let mut room_list: Vec<super::doc::DocRoom> = doc.rooms.clone();
        for c in &doc.cells {
            let id = if c.room.is_empty() { c.id.clone() } else { c.room.clone() };
            if !room_list.iter().any(|r| r.id == id) {
                room_list.push(super::doc::DocRoom { id: id.clone(), name: id, ..Default::default() });
            }
        }
        // Cellene.
        let mut cells: Vec<Cell> = vec![];
        let mut cell_docs: Vec<DocCell> = vec![];
        for c in &expand(doc, kit) {
            let mut c = c.clone();
            if c.room.is_empty() {
                c.room = c.id.clone();
            }
            cell_docs.push(c);
        }
        for c in &cell_docs {
            let Some(piece) = kit.piece(&c.bit).filter(|p| p.kind == PieceKind::Cell) else {
                report.issues.push(Issue::error("bit", Some(cell_center(c.at[0], c.at[1] as f32, c.at[2])), format!("{}: ukjent bit «{}»", c.id, c.bit)));
                continue;
            };
            let room = room_list.iter().find(|r| r.id == c.room);
            let theme = room.map(|r| r.theme.clone()).filter(|t| !t.is_empty()).or_else(|| Some(doc.theme.clone()).filter(|t| !t.is_empty())).unwrap_or_else(|| piece.theme.clone());
            // Standardsteinen: nivåets tema, ellers rommets (modulene er laget i sitt).
            let stone = cell_stone(kit, [if doc.theme.is_empty() { &theme } else { &doc.theme }, &piece.theme], [doc.stone.as_ref(), room.and_then(|r| r.stone.as_ref()), c.stone.as_ref()], cell_center(c.at[0], c.at[1] as f32, c.at[2]), &mut report.issues);
            cells.push(Cell { doc: c, piece, i: c.at[0], j: c.at[2], y: c.at[1] as f32, theme, stone });
        }
        let mut columns: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (k, c) in cells.iter().enumerate() {
            columns.entry((c.i, c.j)).or_default().push(k);
        }
        let index: HashMap<&str, usize> = cells.iter().enumerate().map(|(k, c)| (c.doc.id.as_str(), k)).collect();
        // Overlapp i høyden.
        for (k, c) in cells.iter().enumerate() {
            let (lo, hi) = c.volume();
            for &o in &columns[&(c.i, c.j)] {
                if o > k {
                    let (l2, h2) = cells[o].volume();
                    if l2 < hi - 0.01 && lo < h2 - 0.01 {
                        report.issues.push(Issue::error("overlapp", Some(c.center()), format!("{} og {} overlapper i høyden", c.doc.id, cells[o].doc.id)));
                    }
                }
            }
        }
        // Eierens valg på kantene, og innholdet som er kanten.
        let mut overrides: HashMap<(usize, usize), (String, Option<String>)> = HashMap::new();
        let mut edge_variants: HashMap<(usize, usize), String> = HashMap::new();
    let mut edge_stones = HashMap::new();
        for e in &doc.edges {
            let (Some(&a), Some(l)) = (index.get(e.cell.as_str()), side_index(&e.side)) else {
                report.issues.push(Issue::error("kant", None, format!("kanten {}/{} finnes ikke", e.cell, e.side)));
                continue;
            };
            let w = local_to_world(l, cells[a].rot(), cells[a].mirror());
            if let Some(v) = &e.variant {
                edge_variants.insert((a, w), v.clone());
            }
            if let Some(r) = &e.stone {
                if let Err(e) = kit.stones.apply(&mut cells[a].stone.clone(), r) {
                    report.issues.push(Issue::error("stein", Some(cells[a].center()), e));
                }
                edge_stones.insert((a, w), r.clone());
            }
            // Uten slag låser kanten bare utseendet.
            if !e.kind.is_empty() {
                overrides.insert((a, w), (e.kind.clone(), None));
            }
        }
        for t in &doc.things {
            if let Some((a, w)) = edge_anchor(&t.anchor, &index, &cells) {
                let kind = match t.kind.as_str() {
                    "gate" => "port",
                    "shortcut" => "snarvei",
                    "secret_wall" => "falsk",
                    _ => continue,
                };
                overrides.insert((a, w), (kind.to_string(), Some(t.id.clone())));
            }
        }
        let torches = room_list.iter().map(|r| (r.id.clone(), r.mood.torches)).collect();
        let grid = Grid { cells, columns, overrides, edge_variants, edge_stones, kit, seed: doc.seed, torches };

        // Hver celle, fra minnet om ingenting i nabolaget er endret.
        let mut outs: Vec<CellOut> = vec![];
        let mut keep: HashMap<String, (String, CellOut)> = HashMap::new();
        for a in 0..grid.cells.len() {
            let key = neighborhood_key(&grid, a, &room_list);
            let id = grid.cells[a].doc.id.clone();
            let out = match self.cache.get(&id) {
                Some((k, out)) if *k == key => out.clone(),
                _ => {
                    report.recomputed += 1;
                    cell_out(&grid, a)
                }
            };
            keep.insert(id, (key, out.clone()));
            outs.push(out);
        }
        self.cache = keep;
        report.cells = grid.cells.len();
        // Åpningene til kartet, én gang per kant.
        for a in 0..grid.cells.len() {
            for w in 0..4 {
                let (open, switch, _) = grid.opening(a, w);
                if matches!(open, Open::Wall | Open::Inner) || grid.neighbor(a, w).is_some_and(|b| b < a) {
                    continue;
                }
                let c = &grid.cells[a];
                report.edges.push(EdgeOpen { at: [c.i, c.floor(w).round() as i32, c.j], side: w, kind: open.kit_name(), switch });
            }
        }

        // Samlet.
        let mut file = LevelFile { level: doc.level.number().unwrap_or(0), name: doc.name.clone(), theme: grid.cells.first().map(|c| c.theme.clone()).unwrap_or_default(), ..Default::default() };
        let mut boxes = vec![];
        let mut raster: BTreeMap<(i32, i32, i32), nav::Label> = BTreeMap::new();
        let mut slopes = vec![];
        for out in &outs {
            file.instances.extend(out.instances.iter().cloned());
            boxes.extend(out.boxes.iter().cloned());
            for h in &out.hulls {
                file.colliders.push(ColliderDef { cuboid: None, hull: Some(h.clone()) });
            }
            raster.extend(out.raster.iter().cloned());
            slopes.extend(out.slopes.iter().cloned());
            file.lights.extend(out.lights.iter().cloned());
            report.issues.extend(out.issues.iter().cloned());
        }
        let merged = solids::merge(boxes);
        let mut cols: Vec<ColliderDef> = merged.iter().map(|b| b.def()).collect();
        cols.append(&mut file.colliders);
        file.colliders = cols;

        // Merkene og lysene.
        let (marks, mut issues) = marks(doc, &grid, &index, &room_list);
        report.issues.append(&mut issues);
        nav::cut_marks(&mut raster, &marks);
        nav::bridges(&mut raster, &marks, |p| cell_at(&grid, p).map(|b| grid.cells[b].doc.room.clone()));
        nav::drop_slivers(&mut raster);
        file.nav = nav::build(&raster, &slopes);
        file.marks = marks;
        look::clear_torches(&mut file, &grid);
        // Eierens lys først, så faklene i festene.
        let torches = std::mem::take(&mut file.lights);
        file.lights = lights(doc, &grid, &index);
        file.lights.extend(torches);
        // Stemningen i rommet: lysstyrken på alle lysene i det, og fargen på
        // faklene og modulenes lys (eierens egne lys har sin egen farge).
        for (k, l) in file.lights.iter_mut().enumerate() {
            let Some(m) = l.room.as_ref().and_then(|r| room_list.iter().find(|x| &x.id == r)).map(|x| &x.mood) else { continue };
            if m.light != 1.0 {
                l.intensity = Some(mm(l.intensity.unwrap_or(2.0) * m.light.max(0.0)));
            }
            if let Some(c) = m.color.filter(|_| k >= own_lights) {
                l.color = c;
            }
        }
        let (rooms, mut issues) = rooms::rooms(&grid, &room_list);
        report.issues.append(&mut issues);
        file.rooms = rooms;
        // Det et nivå må ha.
        report.issues.extend(check::needed(&file, doc.level.number() == Some(10)));
        (file, report)
    }
}

/// `kant:<celle>/<side>` → (cella, siden i nivået).
fn edge_anchor(anchor: &str, index: &HashMap<&str, usize>, cells: &[Cell]) -> Option<(usize, usize)> {
    let rest = anchor.strip_prefix("kant:")?;
    let (cell, side) = rest.rsplit_once('/')?;
    let a = *index.get(cell)?;
    let l = side_index(side)?;
    Some((a, local_to_world(l, cells[a].rot(), cells[a].mirror())))
}

/// Alt cella `a` bygges av, som tekst: cellene i ruta 5 × 5 rundt, eierens
/// valg på kantene deres og temaene. Lik nøkkel gir likt resultat.
fn neighborhood_key(g: &Grid, a: usize, rooms: &[super::doc::DocRoom]) -> String {
    let c = &g.cells[a];
    let mut parts = vec![format!("frø {}", g.seed)];
    for dj in -2..=2 {
        for di in -2..=2 {
            if let Some(v) = g.columns.get(&(c.i + di, c.j + dj)) {
                for &b in v {
                    let o = &g.cells[b];
                    let room = rooms.iter().find(|r| r.id == o.doc.room);
                    parts.push(format!("{:?}|{}|{:?}|{:?}|{:?}", o.doc, o.theme, room.map(|r| r.secret), room.map(|r| &r.mood), o.stone));
                    let stones = (0..4).filter_map(|w| g.edge_stones.get(&(b, w)).map(|s| format!("{w}s{s:?}")));
                    let mut ov: Vec<_> = (0..4).filter_map(|w| g.overrides.get(&(b, w)).map(|o| format!("{w}:{o:?}"))).chain((0..4).filter_map(|w| g.edge_variants.get(&(b, w)).map(|v| format!("{w}v{v}")))).chain(stones).collect();
                    ov.sort();
                    parts.push(ov.join(","));
                }
            }
        }
    }
    parts.join(";")
}

/// Det cella `a` bidrar med: biten, halvkantene på sidene, kvarthjørnene
/// og søyla, faklene i festene, kolliderne og nettet. Utseendet (varianten)
/// velges med nivåets frø og plassen, om eieren ikke har låst det.
fn cell_out(g: &Grid, a: usize) -> CellOut {
    let c = &g.cells[a];
    let mut out = CellOut::default();
    let center = c.center();
    let theme = c.theme.as_str();
    let at = [c.i as i64, (c.y * 1000.0).round() as i64, c.j as i64];
    // Biten selv, med steinen (`crypt::stone`; halvkantene og hjørnene tar
    // den fra sin egen celle, så overgangen går midt i åpningen).
    let put = |out: &mut CellOut, (model, solids): (&str, &[ColliderDef]), pos: Vec3, yaw: f32, mirror: bool, (theme, to): (&str, &super::stone::Surfaces)| {
        if !model.is_empty() {
            out.instances.push(InstanceDef { model: model.to_string(), pos: mm3(pos), yaw, mirror, stone: g.kit.stones.swap(theme, to) });
        }
        let t = transform(pos, yaw, mirror);
        for s in solids {
            solids::add(&mut out.boxes, &mut out.hulls, s, &t, yaw);
        }
    };
    let look = cell_look(g, c, at);
    let cell_var = cell_variant_id(c, look.0);
    put(&mut out, look, center, c.yaw(), c.mirror(), (&c.piece.theme, &c.stone));
    // Fakkelfestene i biten selv (rotunden).
    if !c.mirror() {
        torches(g, &mut out, c.piece, (center, c.yaw(), false), &c.doc.room, [at[0], at[1], at[2], 9], g.kit.stones.light_of(&c.stone));
    }
    // Murene rundt en sammensatt bit står bak dens egen mur: ingen fakler der.
    let hidden = c.piece.part_of.is_some();
    // Halvkantene.
    let mut opens = [Open::Wall; 4];
    let mut switches: [Option<String>; 4] = Default::default();
    for w in 0..4 {
        let (open, switch, mut issues) = g.opening(a, w);
        out.issues.append(&mut issues);
        opens[w] = open;
        switches[w] = switch;
        let profile_gang = c.face(w).1 == "gang";
        let kit_open = match open {
            Open::Gang if profile_gang => None,
            Open::Inner => None,
            o => Some(o.kit_name()),
        };
        if let Some(name) = kit_open {
            match g.kit.edge(theme, name) {
                Some(piece) => {
                    let o = outward_v(w) * HALF;
                    let pos = Vec3::new(center.x + o.x, c.floor(w), center.z + o.y);
                    // Eierens variant, ellers cellas (tømmeret i sjakta kler murene), ellers
                    // steinens på annenhver mur (plastsømmene, skyteskårene), ellers frøet.
                    let stone = edge_stone(g, a, w);
                    let by_cell = cell_var.filter(|v| piece.variants.iter().any(|x| x.id == *v));
                    let by_stone = stone.wall_variant.as_deref().filter(|v| piece.variants.iter().any(|x| x.id == *v) && roll(g.seed, [at[0], at[1], at[2], 40 + w as i64]) < 0.5);
                    let locked = g.edge_variants.get(&(a, w)).map(|s| s.as_str()).or(by_cell).or(by_stone);
                    let look = piece.pick_where(locked, roll(g.seed, [at[0], at[1], at[2], 1 + w as i64]), |v| wall_variant_fits(c, v));
                    put(&mut out, look, pos, side_yaw(w), false, (&piece.theme, &stone));
                    if !hidden {
                        torches(g, &mut out, piece, (pos, side_yaw(w), side_wall(c, w)), &c.doc.room, [at[0], at[1], at[2], 10 + w as i64], g.kit.stones.light_of(&c.stone));
                    }
                }
                None => out.issues.push(Issue::error("bit", Some(center), format!("byggesettet mangler halvkanten «{name}» ({theme})"))),
            }
        }
    }
    // Hjørnene: kvarthjørnet etter de to sidene cella har der (innvendig,
    // muren går videre, pilaster under en gjordbue, utvendig).
    // Inne i en sammensatt bit står bitens egen mur: som en mur her.
    let solid = |w: usize| match opens[w] {
        Open::Inner => true,
        Open::Arch => false,
        Open::Gang => c.face(w).1 != "gang",
        _ => true,
    };
    let mut fills = [false; 4];
    let mut pillars: Vec<Vec2> = vec![];
    for k in 0..4 {
        let v = corner_vertex(c.i, c.j, k);
        let (touch, missing, all_open) = vertex_state(g, a, v);
        if touch || missing {
            fills[k] = true;
            let [sa, sb] = corner_sides(k);
            let class = match (solid(sa), solid(sb)) {
                (true, true) => "inner",
                (true, false) if opens[sb] == Open::Arch => "pilaster",
                (false, true) if opens[sa] == Open::Arch => "pilaster",
                (true, false) | (false, true) => "wall",
                _ => "outer",
            };
            match g.kit.corner(theme, class) {
                Some(piece) => {
                    let p = vertex_pos(v);
                    put(&mut out, (&piece.model, &piece.solids), Vec3::new(p.x, c.y, p.y), corner_yaw(k), false, (&piece.theme, &c.stone));
                }
                None => out.issues.push(Issue::error("bit", Some(center), format!("byggesettet mangler kvarthjørnet ({theme})"))),
            }
        } else if all_open {
            pillars.push(vertex_pos(v));
            // Søyla settes ut av cella der hjørnet er NV.
            if k == 0 {
                if let Some(piece) = g.kit.of_kind(theme, PieceKind::Pillar) {
                    let p = vertex_pos(v);
                    put(&mut out, (&piece.model, &piece.solids), Vec3::new(p.x, c.y, p.y), 0.0, false, (&piece.theme, &c.stone));
                }
            }
        }
    }
    // Nettet.
    let room = c.doc.room.clone();
    let pillar_r = g.kit.of_kind(theme, PieceKind::Pillar).map(|p| p.radius).unwrap_or(0.5);
    nav::cell(&mut out, c, &opens, &switches, &fills, &pillars, pillar_r, &room);
    out
}

/// Hjørnet `v` sett fra cella `a`: om en mur når hjørnet (en halvkant som
/// ikke er åpen, i en av cellene rundt), om en av rutene rundt er fjell, og
/// om alle fire er romfelt som er åpne mot hverandre (da står det en søyle).
fn vertex_state(g: &Grid, a: usize, v: (i32, i32)) -> (bool, bool, bool) {
    let mut touch = false;
    let mut missing = false;
    let mut all_open = true;
    for ((i, j), corner) in around_vertex(v) {
        let here = g.cells[a].i == i && g.cells[a].j == j;
        let list: Vec<usize> = if here { vec![a] } else { g.overlapping(a, (i, j)) };
        if list.is_empty() {
            missing = true;
            all_open = false;
            continue;
        }
        for b in list {
            for w in corner_sides(corner) {
                let (open, _, _) = g.opening(b, w);
                let profile_gang = g.cells[b].face(w).1 == "gang";
                let solid = match open {
                    Open::Arch | Open::Inner => false,
                    Open::Gang => !profile_gang,
                    _ => true,
                };
                touch |= solid;
                if open != Open::Arch {
                    all_open = false;
                }
            }
        }
    }
    (touch, missing, all_open)
}

/// Merkene fra innholdet og koblingene.
fn marks(doc: &LevelDoc, g: &Grid, index: &HashMap<&str, usize>, rooms: &[super::doc::DocRoom]) -> (Vec<MarkDef>, Vec<Issue>) {
    let mut out: Vec<MarkDef> = vec![];
    let mut issues = vec![];
    for t in &doc.things {
        let Some((pos, yaw, extra)) = place_thing(t, g, index) else {
            issues.push(Issue::error("innhold", None, format!("{} ({}): ankeret «{}» finnes ikke", t.id, t.kind, t.anchor)));
            continue;
        };
        let patrol = t.patrol.iter().filter_map(|p| anchor_point(&t.anchor, Vec3::from(*p), g, index)).map(mm3).collect();
        let mut m = MarkDef {
            kind: t.kind.clone(),
            pos: Some(mm3(pos)),
            yaw: mm(yaw),
            id: Some(t.id.clone()),
            label: t.label.clone(),
            n: t.n,
            patrol,
            ambush: t.ambush.clone(),
            text: t.text.clone(),
            loot: t.loot.clone(),
            width: t.width.or(extra.0),
            height: t.height.or(extra.1),
            size: t.size,
            top: t.top.and_then(|p| anchor_point(&t.anchor, Vec3::from(p), g, index)).map(mm3),
            bottom: t.bottom.and_then(|p| anchor_point(&t.anchor, Vec3::from(p), g, index)).map(mm3),
            levels: t.levels.iter().filter_map(|l| anchor_point(&t.anchor, Vec3::new(0.0, *l, 0.0), g, index)).map(|p| mm(p.y)).collect(),
            ..Default::default()
        };
        if m.kind == "mob" && m.label.is_none() {
            m.label = Some("button".into());
        }
        // Den falske veggen er byggesettets egen (den samme muren som rundt).
        // Steinen er murens på samme side.
        if let Some((a, w)) = edge_anchor(&t.anchor, index, &g.cells).filter(|_| m.kind == "secret_wall") {
            let leaf = g.kit.of_kind(&g.cells[a].theme, PieceKind::Leaf);
            if m.label.is_none() {
                m.label = leaf.map(|p| p.model.clone());
            }
            m.stone = leaf.and_then(|p| g.kit.stones.swap(&p.theme, &edge_stone(g, a, w)));
        }
        // Døra opp og trappa ned er mures av steinen i cella de står i
        // (byggesettets modeller er laget i gravkjellerens stein).
        if matches!(m.kind.as_str(), "up" | "down") {
            if let Some(b) = m.pos.and_then(|p| cell_at(g, Vec3::from(p) + Vec3::Y * 0.1)) {
                m.stone = g.kit.stones.swap("grav", &g.cells[b].stone);
            }
        }
        out.push(m);
    }
    let room_ok = |id: &str| rooms.iter().any(|r| r.id == id);
    for l in &doc.links {
        let Some(from) = out.iter().position(|m| m.id.as_deref() == Some(l.from.as_str())) else {
            issues.push(Issue::error("kobling", None, format!("koblingen {} → {}: «{}» finnes ikke", l.from, l.to, l.from)));
            continue;
        };
        let target_mark = out.iter().any(|m| m.id.as_deref() == Some(l.to.as_str()));
        let m = &mut out[from];
        match l.kind.as_str() {
            "opens" | "key" if target_mark => m.opens.push(l.to.clone()),
            "arena" if room_ok(&l.to) => m.arena = Some(l.to.clone()),
            "exit" if room_ok(&l.to) => {
                m.arena = Some(l.to.clone());
                m.label = Some("exit".into());
            }
            "for" if target_mark => m.for_id = Some(l.to.clone()),
            "to" if room_ok(&l.to) => m.to = Some(l.to.clone()),
            "from" if room_ok(&l.to) => m.from = Some(l.to.clone()),
            other => issues.push(Issue::error("kobling", m.pos.map(Vec3::from), format!("koblingen {} → {} ({other}): målet finnes ikke eller slaget er ukjent", l.from, l.to))),
        }
    }
    (out, issues)
}

/// Stedet, dreiningen (radianer) og (bredde, høyde) for innhold på kanten.
fn place_thing(t: &Thing, g: &Grid, index: &HashMap<&str, usize>) -> Option<(Vec3, f32, (Option<f32>, Option<f32>))> {
    let yaw = t.yaw.to_radians();
    if let Some((a, w)) = edge_anchor(&t.anchor, index, &g.cells) {
        let c = &g.cells[a];
        let o = outward_v(w);
        let mid = Vec3::new(c.center().x + o.x * HALF, c.floor(w), c.center().z + o.y * HALF);
        let face = side_yaw(w);
        return Some(match t.kind.as_str() {
            // Den falske veggen står i murflaten på siden den synes fra.
            "secret_wall" => (mid - Vec3::new(o.x, 0.0, o.y) * (2.0 * WALL_T), face, (Some(SECRET_W), Some(SECRET_H))),
            "gate" => (mid, face + yaw, (Some(GATE_W), None)),
            "shortcut" => (mid, face + yaw, (Some(DOOR_W), None)),
            _ => (mid + Vec3::from(t.pos.unwrap_or_default()), face + yaw, (None, None)),
        });
    }
    if let Some(id) = t.anchor.strip_prefix("celle:") {
        let c = &g.cells[*index.get(id)?];
        let (p, y) = match (t.pos, c.piece.mounts.iter().find(|m| m.kind == t.kind)) {
            (None, Some(m)) => (Vec3::from(m.pos), m.yaw),
            (p, _) => (Vec3::from(p.unwrap_or_default()), yaw),
        };
        return Some((c.point(p), c.turn(y), (None, None)));
    }
    if t.anchor.is_empty() || t.anchor == "fri" {
        return Some((Vec3::from(t.pos.unwrap_or_default()), yaw, (None, None)));
    }
    None
}
