//! Kjelleren: fem etasjer under borgruinen, én per epoke, der de korrupte
//! komponentene holder til (se docs/kjelleren.md). Du kommer ned gjennom
//! døra i krypten (`Use::Descend`) og opp igjen gjennom døra i startrommet
//! (`Use::Ascend`).
//!
//! Hver etasje settes sammen av håndbygde rom fra Blender
//! (`tools/models/dungeon.py`: hvelvede haller, cisterne, galleri, kapell,
//! skattkamre, portrommet og bossarenaen) og et modulsett av hvelvede
//! ganger. Rommene vokser ut fra startrommet: hvert nytt rom henges på en
//! ledig dør i et rom som står, med en gang imellom (ruta med A* på
//! rutenettet). Hovedveien ender i portrommet, der portgitteret
//! (`Use::Gate`, nøkkelen bærer eliten) stenger vindeltrappa ned til
//! neste etasje; der kommer du ut i trappefoten. Sidegrener går til
//! skattkamre, og en løkke binder hovedveien sammen. Dører ingen gang går
//! til mures igjen. Hvert rom har en json-fil fra Blender med gulv- og
//! takhøyde per celle, dører, kollidere, lys og steder.
//!
//! Fiendene (`domain/mobs.rs`) går på rutenettet (`Floor::walkable`,
//! `next_step`); spilleren går på kolliderne.

use super::{Builder, Shape, Use};
use crate::domain::geometry::Layer;
use crate::domain::model::Look;
use crate::domain::rng::Rng;
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

/// Gulvets overkant i den første etasjen, under alt i byen.
pub const DEPTH: f32 = -16.0;
pub const CELL: f32 = 4.0;
/// Minste høyde fra gulvet til taket: spilleren er 1,8 m og hopper.
pub const HEADROOM: f32 = 2.9;
/// Høyden mellom etasjene (vindeltrappa).
pub const FLOOR_DROP: f32 = 12.0;
pub const FLOORS: usize = 5;
/// Rutenettet i hver etasje er N × N celler.
pub const N: usize = 40;
/// Portgitteret som sveives opp foran vindeltrappa (modell, posert av spillet).
pub const PORTCULLIS: &str = "portcullis";
/// Kistene: kassa står fast, lokket (`CHEST_LID`, origo i hengslet) dreies
/// opp av økta. Hengslet sitter bak på overkanten av kassa.
pub const CHEST: &str = "chest";
pub const CHEST_LID: &str = "chest_lid";
/// Snarveidøra: rammen står fast, døra dreies opp av økta.
pub const SHORTCUT: &str = "dng_shortcut";
pub const SHORTCUT_LEAF: &str = "dng_shortcut_leaf";
/// Hengslet til snarveidøra i rammens rom (x mot venstre stolpe, z midt i
/// muren), og døras bredde og høyde.
pub const SHORTCUT_HINGE: Vec3 = Vec3::new(-1.31, 0.0, 0.3);
pub const SHORTCUT_W: f32 = 2.62;
pub const SHORTCUT_H: f32 = 2.28;
pub const CHEST_HINGE: Vec3 = Vec3::new(0.0, 0.46, -0.28);
pub const EPOCHS: [&str; FLOORS] = ["2026: forretningslogikken", "2019: klassekomponentene", "2014: jQuery-kloakken", "2009: tabellayout-katakombene", "1999: blink-dypet"];
/// Portgitteret: så bredt og høyt, og hvor langt det sveives opp.
pub const GATE_W: f32 = 3.9;
pub const GATE_H: f32 = 2.9;
pub const GATE_LIFT: f32 = 2.7;
/// Halv bredde og takhøyde i gangene.
const HALL_HW: f32 = 1.4;
const HALL_CEIL: f32 = 3.8;
/// Kjelleren holder seg innenfor verdens kant (199 m).
const REACH: f32 = 185.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Rock,
    /// Gulv i denne høyden over etasjens gulv.
    Floor(f32),
    /// Trapp som stiger mot (dx, dz), fra `lo` til `hi`.
    Ramp { dx: i32, dz: i32, lo: f32, hi: f32 },
}

#[derive(Clone, Debug)]
pub struct Floor {
    pub cells: Vec<Cell>,
    /// Halv gangbredde fra cellemidten (2 i rom, 1,4 i gangene).
    hw: Vec<f32>,
    /// Takhøyde over etasjens gulv.
    ceil: Vec<f32>,
    /// Hvilket rom en celle hører til (gangceller er hver for seg, −1).
    unit: Vec<i32>,
    /// Kanter mellom celler i ulike rom eller ganger som er åpne (dører
    /// og gangforløp), og kanter inne i et rom som er stengt.
    links: HashSet<((i32, i32), (i32, i32))>,
    blocked: HashSet<((i32, i32), (i32, i32))>,
    /// Kanter som er stengt av en dør (snarveiene), til den åpnes.
    closed: HashSet<((i32, i32), (i32, i32))>,
    /// Verdenskoordinatene (x, z) til hjørnet av celle (0, 0).
    pub origin: (f32, f32),
    /// Gulvets overkant.
    pub y: f32,
    pub theme: usize,
    /// Cella der du starter i etasjen.
    pub start: (i32, i32),
    /// Tårncella med vindeltrappa ned (ingen nederst).
    pub exit: Option<(i32, i32)>,
}

#[derive(Clone, Debug)]
pub struct Dungeon {
    pub floors: Vec<Floor>,
    /// Føttene der du står når du kommer ned fra krypten.
    pub start: Vec3,
    /// Steder fiendene kan starte, alle etasjer.
    pub spawns: Vec<Vec3>,
    /// Elitefiendene som bærer nøklene: en per etasje.
    pub elites: Vec<Vec3>,
    /// Portgitrene: (midten i gulvhøyde, yaw), en per etasje med trapp.
    pub gates: Vec<(Vec3, f32)>,
    /// Tokens som ligger og venter (skatten nederst).
    pub loot: Vec<Vec3>,
    /// Kistene: (føtter, yaw, etasje). F åpner dem (`Use::Chest`), og
    /// innholdet følger etasjen (`craft::LOOT`).
    pub chests: Vec<(Vec3, f32, usize)>,
    /// Der Rammeverket står: midt i rotunden nederst.
    pub boss: Option<Vec3>,
    /// Snarveiene: dører fra startrommet i en etasje til en gang som går til
    /// et rom lenger inn, med slå på gangsida.
    pub shortcuts: Vec<Shortcut>,
    /// Det som står i rommene (sarkofager, søyler, kister, murer) sett
    /// ovenfra, så fiendene går rundt det i stedet for gjennom.
    obstacles: Vec<(Obstacle, f32, f32)>,
    /// Hindringene som berører hver celle: (etasje, celle) → indekser.
    near: HashMap<(usize, i32, i32), Vec<u32>>,
}

/// En snarveidør: rammen står i celleveggen med lokal +z inn i rommet
/// (`yaw`), og kanten den stenger.
#[derive(Clone, Copy, Debug)]
pub struct Shortcut {
    pub pos: Vec3,
    pub yaw: f32,
    pub floor: usize,
    pub edge: ((i32, i32), (i32, i32)),
}

/// Fotavtrykket til en kollider i kroppshøyde.
#[derive(Clone, Copy, Debug)]
enum Obstacle {
    /// Boks: midten, halve mål og retningen til den lokale x-aksen.
    Box { c: Vec2, half: Vec2, axis: Vec2 },
    Circle { c: Vec2, r: f32 },
}

impl Obstacle {
    /// Avstand fra punktet til kanten (negativ inni).
    fn dist(&self, p: Vec2) -> f32 {
        match *self {
            Obstacle::Box { c, half, axis } => {
                let d = p - c;
                let l = Vec2::new(d.dot(axis), d.perp_dot(axis)).abs() - half;
                l.max(Vec2::ZERO).length() + l.x.max(l.y).min(0.0)
            }
            Obstacle::Circle { c, r } => p.distance(c) - r,
        }
    }

    fn bounds(&self) -> (Vec2, Vec2) {
        match *self {
            Obstacle::Box { c, half, axis } => {
                let e = Vec2::new(half.x * axis.x.abs() + half.y * axis.y.abs(), half.x * axis.y.abs() + half.y * axis.x.abs());
                (c - e, c + e)
            }
            Obstacle::Circle { c, r } => (c - Vec2::splat(r), c + Vec2::splat(r)),
        }
    }
}

fn edge(a: (i32, i32), b: (i32, i32)) -> ((i32, i32), (i32, i32)) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

impl Floor {
    fn idx(ix: i32, iz: i32) -> Option<usize> {
        (ix >= 0 && iz >= 0 && (ix as usize) < N && (iz as usize) < N).then(|| iz as usize * N + ix as usize)
    }

    pub fn cell(&self, ix: i32, iz: i32) -> Cell {
        Self::idx(ix, iz).map(|i| self.cells[i]).unwrap_or(Cell::Rock)
    }

    pub fn is_open(&self, ix: i32, iz: i32) -> bool {
        self.cell(ix, iz) != Cell::Rock
    }

    /// Åpne en dør som stengte kanten mellom to celler (en snarvei).
    pub fn open(&mut self, e: ((i32, i32), (i32, i32))) {
        self.closed.remove(&e);
    }

    /// Om to celler hører til samme rom.
    pub fn same_room(&self, a: (i32, i32), b: (i32, i32)) -> bool {
        match (Self::idx(a.0, a.1), Self::idx(b.0, b.1)) {
            (Some(i), Some(j)) => self.unit[i] >= 0 && self.unit[i] == self.unit[j],
            _ => false,
        }
    }

    /// Cellen et verdenspunkt ligger i.
    pub fn cell_of(&self, p: Vec3) -> (i32, i32) {
        (((p.x - self.origin.0) / CELL).floor() as i32, ((p.z - self.origin.1) / CELL).floor() as i32)
    }

    /// Taket over en celle (absolutt y).
    pub fn ceil_at(&self, ix: i32, iz: i32) -> f32 {
        self.y + Self::idx(ix, iz).map(|i| self.ceil[i]).unwrap_or(HALL_CEIL)
    }

    /// Det høyeste gulvet i en celle (toppen av trappa).
    pub fn top_at(&self, ix: i32, iz: i32) -> f32 {
        self.y
            + match self.cell(ix, iz) {
                Cell::Floor(h) => h,
                Cell::Ramp { hi, .. } => hi,
                Cell::Rock => 0.0,
            }
    }

    /// Gulvhøyden i et verdenspunkt (absolutt y), trappene medregnet.
    pub fn height_at(&self, p: Vec3) -> f32 {
        let (ix, iz) = self.cell_of(p);
        match self.cell(ix, iz) {
            Cell::Floor(h) => self.y + h,
            Cell::Ramp { dx, dz, lo, hi } => {
                let lx = (p.x - (self.origin.0 + ix as f32 * CELL)) / CELL;
                let lz = (p.z - (self.origin.1 + iz as f32 * CELL)) / CELL;
                let t = if dx > 0 {
                    lx
                } else if dx < 0 {
                    1.0 - lx
                } else if dz > 0 {
                    lz
                } else {
                    1.0 - lz
                };
                self.y + lo + t.clamp(0.0, 1.0) * (hi - lo)
            }
            Cell::Rock => self.y,
        }
    }

    /// Midten av en celle, på gulvet.
    pub fn center(&self, ix: i32, iz: i32) -> Vec3 {
        let p = Vec3::new(self.origin.0 + (ix as f32 + 0.5) * CELL, 0.0, self.origin.1 + (iz as f32 + 0.5) * CELL);
        Vec3::new(p.x, self.height_at(p), p.z)
    }

    /// Om du kan gå fra celle `a` til nabocellen `b`: begge åpne, samme rom
    /// (og kanten ikke stengt) eller en dør eller gang imellom, og samme
    /// høyde der de møtes (trapper bare langs aksen).
    pub fn passable(&self, a: (i32, i32), b: (i32, i32)) -> bool {
        let (Some(ia), Some(ib)) = (Self::idx(a.0, a.1), Self::idx(b.0, b.1)) else { return false };
        let (ca, cb) = (self.cells[ia], self.cells[ib]);
        if ca == Cell::Rock || cb == Cell::Rock {
            return false;
        }
        let e = edge(a, b);
        let joined = if self.unit[ia] >= 0 && self.unit[ia] == self.unit[ib] { !self.blocked.contains(&e) } else { self.links.contains(&e) };
        if !joined || self.closed.contains(&e) {
            return false;
        }
        let d = (b.0 - a.0, b.1 - a.1);
        // Høyden på hver side av kanten.
        let side = |c: Cell, toward: (i32, i32)| -> Option<f32> {
            match c {
                Cell::Floor(h) => Some(h),
                Cell::Ramp { dx, dz, lo, hi } => {
                    if toward == (dx, dz) {
                        Some(hi)
                    } else if toward == (-dx, -dz) {
                        Some(lo)
                    } else {
                        None
                    }
                }
                Cell::Rock => None,
            }
        };
        match (side(ca, d), side(cb, (-d.0, -d.1))) {
            (Some(x), Some(y)) => (x - y).abs() < 0.3,
            _ => false,
        }
    }

    /// Om et punkt (med margin inn fra murene) er på gulvet: inne i
    /// midtfeltet av cella, eller i en arm mot en nabo du kan gå til.
    pub fn walkable(&self, p: Vec3, margin: f32) -> bool {
        let (ix, iz) = self.cell_of(p);
        let Some(i) = Self::idx(ix, iz) else { return false };
        if self.cells[i] == Cell::Rock {
            return false;
        }
        let hw = self.hw[i];
        let lx = p.x - (self.origin.0 + (ix as f32 + 0.5) * CELL);
        let lz = p.z - (self.origin.1 + (iz as f32 + 0.5) * CELL);
        let lim = hw - margin;
        let arm = |dx: i32, dz: i32| -> f32 {
            let n = (ix + dx, iz + dz);
            if self.passable((ix, iz), n) {
                hw.min(Self::idx(n.0, n.1).map(|j| self.hw[j]).unwrap_or(hw)) - margin
            } else {
                -1.0
            }
        };
        let ok_x = lx.abs() <= lim || (if lx > 0.0 { arm(1, 0) } else { arm(-1, 0) }) >= lz.abs();
        let ok_z = lz.abs() <= lim || (if lz > 0.0 { arm(0, 1) } else { arm(0, -1) }) >= lx.abs();
        ok_x && ok_z
    }

    /// Neste steg på korteste vei fra `from` til `to` (bredde først på
    /// rutenettet), eller `None` om det ikke finnes noen vei.
    pub fn next_step(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let a = self.cell_of(from);
        let b = self.cell_of(to);
        if !self.is_open(a.0, a.1) || !self.is_open(b.0, b.1) {
            return None;
        }
        if a == b {
            return Some(to);
        }
        let idx = |c: (i32, i32)| c.1 as usize * N + c.0 as usize;
        let mut prev = vec![usize::MAX; N * N];
        let mut queue = VecDeque::new();
        prev[idx(a)] = idx(a);
        queue.push_back(a);
        while let Some(c) = queue.pop_front() {
            if c == b {
                break;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + dx, c.1 + dz);
                if Self::idx(n.0, n.1).is_some() && prev[idx(n)] == usize::MAX && self.passable(c, n) {
                    prev[idx(n)] = idx(c);
                    queue.push_back(n);
                }
            }
        }
        if prev[idx(b)] == usize::MAX {
            return None;
        }
        let mut cur = idx(b);
        while prev[cur] != idx(a) {
            cur = prev[cur];
        }
        Some(self.center((cur % N) as i32, (cur / N) as i32))
    }
}

impl Dungeon {
    /// Etasjen et punkt er i, etter høyden.
    pub fn floor_of(&self, p: Vec3) -> Option<usize> {
        self.floors.iter().position(|f| p.y >= f.y - 2.5 && p.y < f.y + FLOOR_DROP - 2.5)
    }

    pub fn walkable(&self, p: Vec3, margin: f32) -> bool {
        self.floor_of(p).is_some_and(|f| self.floors[f].walkable(p, margin))
    }

    pub fn next_step(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let f = self.floor_of(from)?;
        if self.floor_of(to) != Some(f) {
            return None;
        }
        self.floors[f].next_step(from, to)
    }

    /// Gulvhøyden under et punkt (absolutt y); punktets y om det er
    /// utenfor kjelleren.
    pub fn height_at(&self, p: Vec3) -> f32 {
        self.floor_of(p).map(|f| self.floors[f].height_at(p)).unwrap_or(p.y)
    }

    /// Rutenettet i etasjen du står i, til lyset (se `scene::CellGrid`).
    pub fn grid_at(&self, p: Vec3) -> Option<crate::domain::scene::CellGrid> {
        let fl = &self.floors[self.floor_of(p)?];
        let mut bits = vec![0u8; N * N];
        // Lyset går også over celler i et rom som ikke er gulv (kløfta).
        let open = |c: (i32, i32)| fl.is_open(c.0, c.1) || fl.same_room(c, c);
        let through = |a: (i32, i32), b: (i32, i32)| open(a) && open(b) && (fl.passable(a, b) || fl.same_room(a, b));
        for iz in 0..N as i32 {
            for ix in 0..N as i32 {
                if !open((ix, iz)) {
                    continue;
                }
                let mut b = 1u8;
                if through((ix, iz), (ix + 1, iz)) {
                    b |= 2;
                }
                if through((ix, iz), (ix, iz + 1)) {
                    b |= 4;
                }
                bits[iz as usize * N + ix as usize] = b;
            }
        }
        Some(crate::domain::scene::CellGrid { origin: Vec2::new(fl.origin.0, fl.origin.1), floor_y: fl.y, cell: CELL, bits })
    }

    /// Om en kropp med radius `r` får plass ved `p` uten å stå i noe som
    /// står i rommet (gulvet og murene i rutenettet sjekker `walkable`).
    pub fn clear(&self, p: Vec3, r: f32) -> bool {
        let Some(f) = self.floor_of(p) else { return true };
        let c = self.floors[f].cell_of(p);
        let Some(list) = self.near.get(&(f, c.0, c.1)) else { return true };
        let q = Vec2::new(p.x, p.z);
        // Det du står oppå (en plattform, et trinn) eller går under, er
        // ikke i veien.
        // I trappene regnes toppen av trappa: massen under trinnene står
        // i kollidere, trinnene selv er rampa.
        let fl = &self.floors[f];
        let ground = match fl.cell(c.0, c.1) {
            Cell::Ramp { hi, .. } => fl.y + hi,
            _ => fl.height_at(p),
        };
        list.iter().all(|&i| {
            let (o, lo, hi) = self.obstacles[i as usize];
            hi < ground + 0.45 || lo > ground + 1.6 || o.dist(q) >= r
        })
    }

    /// Fotavtrykkene til boksene og sylindrene som står i kroppshøyde over
    /// gulvet (ikke gulvplatene, trinnene eller hvelvet), lagt i cellene de
    /// berører.
    fn index_obstacles(&mut self, shapes: &[Shape]) {
        for s in shapes {
            let (o, lo, hi, at) = match s {
                Shape::Cuboid { center, size, yaw } => {
                    let axis = Vec2::new(yaw.cos(), -yaw.sin());
                    (Obstacle::Box { c: Vec2::new(center.x, center.z), half: Vec2::new(size.x, size.z) * 0.5, axis }, center.y - size.y * 0.5, center.y + size.y * 0.5, *center)
                }
                Shape::Cylinder { center, radius, half_height } => (Obstacle::Circle { c: Vec2::new(center.x, center.z), r: *radius }, center.y - half_height, center.y + half_height, *center),
                // Omhyllingene er trinn og skrå trapper, som man går oppå.
                _ => continue,
            };
            let Some(f) = self.floor_of(at) else { continue };
            let fl = &self.floors[f];
            // Gulvplatene og hvelvet tas ikke med; resten måles mot gulvet
            // der kroppen står (`clear`).
            if hi < fl.y + 0.3 || lo > fl.y + 4.0 || (o.bounds().1 - o.bounds().0).min_element() > 3.9 {
                continue;
            }
            let i = self.obstacles.len() as u32;
            self.obstacles.push((o, lo, hi));
            // Cellene den berører, med en meters slingringsmonn for kroppene.
            let (a, z) = o.bounds();
            let ca = fl.cell_of(Vec3::new(a.x - 1.0, 0.0, a.y - 1.0));
            let cz = fl.cell_of(Vec3::new(z.x + 1.0, 0.0, z.y + 1.0));
            for iz in ca.1..=cz.1 {
                for ix in ca.0..=cz.0 {
                    self.near.entry((f, ix, iz)).or_default().push(i);
                }
            }
        }
        log::info!("Kjelleren: {} hindringer i rommene", self.obstacles.len());
    }
}

// --- Rommene fra Blender --------------------------------------------------------

#[derive(serde::Deserialize, Default)]
struct Spots {
    #[serde(default)]
    spawn: Vec<[f32; 3]>,
    #[serde(default)]
    chest: Vec<[f32; 4]>,
    #[serde(default)]
    loot: Vec<[f32; 3]>,
    elite: Option<[f32; 3]>,
    boss: Option<[f32; 3]>,
    start: Option<[f32; 4]>,
    ascend: Option<[f32; 4]>,
    gate: Option<[f32; 4]>,
    tower: Option<[i32; 2]>,
    /// Ledige murflater for pynt: (x, z, normalen inn i rommet).
    #[serde(default)]
    wall: Vec<[f32; 4]>,
    /// Ledige flekker på gulvet for pynt.
    #[serde(default)]
    floor: Vec<[f32; 3]>,
}

/// Et rom eller en gangmodul: fotavtrykket w × d celler og det som står i
/// json-fila ved siden av modellen.
#[derive(serde::Deserialize, Default)]
struct Spec {
    w: i32,
    d: i32,
    floor: Vec<Vec<Option<f32>>>,
    ceil: Vec<Vec<f32>>,
    hw: Vec<Vec<f32>>,
    #[serde(default)]
    ramps: Vec<(i32, i32, i32, i32, f32, f32)>,
    #[serde(default)]
    doors: Vec<(i32, i32, i32, i32)>,
    #[serde(default)]
    blocked: Vec<(i32, i32, i32, i32)>,
    #[serde(default)]
    boxes: Vec<([f32; 3], [f32; 3], f32)>,
    #[serde(default)]
    hulls: Vec<Vec<[f32; 3]>>,
    #[serde(default)]
    lights: Vec<([f32; 3], [f32; 3], f32, f32)>,
    #[serde(default)]
    spots: Spots,
}

macro_rules! specs {
    ($($name:literal),* $(,)?) => {
        const SPECS: &[(&str, &str)] = &[$(($name, include_str!(concat!("../../../assets/models/dungeon/", $name, ".json")))),*];
    };
}

specs!(
    "dng_hall_i",
    "dng_hall_i2",
    "dng_hall_i3",
    "dng_hall_sewer",
    "dng_hall_crypt",
    "dng_hall_lava",
    "dng_hall_l",
    "dng_hall_t",
    "dng_hall_x",
    "dng_hall_end",
    "dng_seal",
    "dng_shortcut",
    "dng_spiral",
    "dng_start",
    "dng_gate",
    "dng_landing",
    "dng_hall",
    "dng_long",
    "dng_cistern",
    "dng_gallery",
    "dng_chasm",
    "dng_chapel",
    "dng_treasury",
    "dng_arena",
    "dngp_shelf",
    "dngp_sacks",
    "dngp_banner",
    "dngp_moss",
    "dngp_puddle",
    "dngp_roots",
    "dngp_skulls",
    "dngp_bonepile",
    "dngp_lava",
    "dngp_embers",
);

fn spec(name: &str) -> &'static Spec {
    static PARSED: OnceLock<HashMap<&'static str, Spec>> = OnceLock::new();
    let all = PARSED.get_or_init(|| {
        SPECS
            .iter()
            .map(|(n, json)| {
                let s: Spec = serde_json::from_str(json).unwrap_or_else(|e| {
                    log::warn!("Kjellerrommet {n}: {e}");
                    Spec::default()
                });
                (*n, s)
            })
            .collect()
    });
    &all[name]
}

/// Rommene på hovedveien, og hvor mange ganger hvert kan brukes per etasje.
const POOL: [(&str, usize); 6] = [("dng_hall", 2), ("dng_long", 1), ("dng_cistern", 1), ("dng_gallery", 1), ("dng_chapel", 1), ("dng_chasm", 1)];

/// Preget til en etasje: lagene og tonen på murverket (`Look`), fargen og
/// styrken på lyset, og pynten som settes ut på murene, gulvet og i hvelvet.
struct Theme {
    look: Look,
    light: Option<Vec3>,
    gain: f32,
    wall: &'static [&'static str],
    floor: &'static [&'static str],
    ceiling: &'static [&'static str],
    /// Hvor mye pynt (0..1).
    density: f32,
}

fn theme(k: usize) -> Theme {
    let look = |layers: [Layer; 4], tint: [f32; 3], flame: [f32; 3]| Look { layers, tint: Vec3::from(tint), flame: Vec3::from(flame) };
    match k % FLOORS {
        // 2026: forretningslogikken. Tørr bruddsteinsmur, hvelv av hogd
        // stein, fakler, arkiv og sekker.
        0 => Theme { look: look([Layer::Stone, Layer::Brick, Layer::Concrete, Layer::Concrete], [1.0, 1.0, 1.0], [1.0, 0.62, 0.26]), light: None, gain: 1.0, wall: &["dngp_shelf", "dngp_sacks"], floor: &[], ceiling: &[], density: 0.45 },
        // 2019: klassekomponentene. Ordnet kvadermur i kaldt grått, skifer
        // på gulvet, blå flammer, bannere og arkivhyller.
        1 => Theme { look: look([Layer::Brick, Layer::Brick, Layer::Slate, Layer::Slate], [0.62, 0.68, 0.8], [0.5, 0.72, 1.0]), light: Some(Vec3::new(0.5, 0.68, 1.0)), gain: 1.1, wall: &["dngp_banner", "dngp_shelf", "dngp_banner"], floor: &[], ceiling: &[], density: 0.6 },
        // 2014: jQuery-kloakken. Fuktig kryptmur med mose, søle på gulvet,
        // gulgrønne lykter, pytter og røtter gjennom hvelvet.
        2 => Theme { look: look([Layer::CryptWall, Layer::CryptWall, Layer::Stone, Layer::Gravel], [0.8, 0.86, 0.74], [0.75, 1.0, 0.45]), light: Some(Vec3::new(0.62, 0.85, 0.4)), gain: 0.55, wall: &["dngp_moss"], floor: &["dngp_puddle", "dngp_puddle"], ceiling: &["dngp_roots"], density: 0.7 },
        // 2009: tabellayout-katakombene. Hogd berg, beinhyller i rutenett og
        // knokkelhauger, bleke stearinlys.
        3 => Theme { look: look([Layer::Rock, Layer::Rock, Layer::Stone, Layer::Stone], [0.62, 0.6, 0.58], [1.0, 0.86, 0.62]), light: Some(Vec3::new(1.0, 0.78, 0.5)), gain: 0.5, wall: &["dngp_skulls"], floor: &["dngp_bonepile"], ceiling: &[], density: 0.75 },
        // 1999: blink-dypet. Sotet mur og svart berg, rødglødende sprekker
        // og slagg.
        _ => Theme { look: look([Layer::Soot, Layer::Rock, Layer::Soot, Layer::Rock], [0.5, 0.42, 0.4], [1.0, 0.38, 0.1]), light: Some(Vec3::new(1.0, 0.24, 0.06)), gain: 0.75, wall: &["dngp_embers"], floor: &["dngp_lava", "dngp_lava", "dngp_embers"], ceiling: &[], density: 0.65 },
    }
}

/// Et rom satt ut på rutenettet: modellen, hjørnet (gx, gz) og dreiningen
/// i kvarte omganger (mot klokka sett ovenfra, som yaw −90° per steg).
#[derive(Clone, Copy, Debug)]
struct Placed {
    name: &'static str,
    gx: i32,
    gz: i32,
    rot: i32,
}

impl Placed {
    fn spec(&self) -> &'static Spec {
        spec(self.name)
    }

    /// Fotavtrykket på rutenettet etter dreiningen.
    fn size(&self) -> (i32, i32) {
        let s = self.spec();
        if self.rot % 2 == 1 {
            (s.d, s.w)
        } else {
            (s.w, s.d)
        }
    }

    /// Et punkt i modellens rom (meter) dreid inn i fotavtrykket (meter fra
    /// hjørnet på rutenettet).
    fn turn(&self, p: Vec2) -> Vec2 {
        let s = self.spec();
        let (w, d) = (s.w as f32 * CELL, s.d as f32 * CELL);
        match self.rot & 3 {
            0 => p,
            1 => Vec2::new(d - p.y, p.x),
            2 => Vec2::new(w - p.x, d - p.y),
            _ => Vec2::new(p.y, w - p.x),
        }
    }

    fn dir(&self, d: (i32, i32)) -> (i32, i32) {
        match self.rot & 3 {
            0 => d,
            1 => (-d.1, d.0),
            2 => (-d.0, -d.1),
            _ => (d.1, -d.0),
        }
    }

    /// Cella på rutenettet for celle (i, j) i modellen.
    fn cell(&self, i: i32, j: i32) -> (i32, i32) {
        let p = self.turn(Vec2::new((i as f32 + 0.5) * CELL, (j as f32 + 0.5) * CELL));
        (self.gx + (p.x / CELL).floor() as i32, self.gz + (p.y / CELL).floor() as i32)
    }

    fn yaw(&self) -> f32 {
        -(self.rot as f32) * std::f32::consts::FRAC_PI_2
    }

    /// Modellens transform i verden.
    fn affine(&self, origin: (f32, f32), y: f32) -> Affine3A {
        let t = self.turn(Vec2::ZERO);
        Affine3A::from_rotation_translation(Quat::from_rotation_y(self.yaw()), Vec3::new(origin.0 + self.gx as f32 * CELL + t.x, y, origin.1 + self.gz as f32 * CELL + t.y))
    }

    /// Dørene: (cella i rommet på rutenettet, retningen ut).
    fn doors(&self) -> Vec<((i32, i32), (i32, i32))> {
        self.spec().doors.iter().map(|&(i, j, dx, dz)| (self.cell(i, j), self.dir((dx, dz)))).collect()
    }
}

/// Planen for en etasje mens den vokser.
struct Plan {
    rooms: Vec<Placed>,
    /// Hvem som eier en celle: rom (≥ 0), gang (−2) eller ingen (−1).
    owner: Vec<i32>,
    links: HashSet<((i32, i32), (i32, i32))>,
    /// Dørene som er i bruk: (rom, dør).
    used: HashSet<(usize, usize)>,
    /// Hvilke celler i verden som er lov (innenfor verdens kant).
    origin: (f32, f32),
}

impl Plan {
    fn owner(&self, c: (i32, i32)) -> i32 {
        Floor::idx(c.0, c.1).map(|i| self.owner[i]).unwrap_or(-3)
    }

    fn in_world(&self, c: (i32, i32)) -> bool {
        let x = self.origin.0 + (c.0 as f32 + 0.5) * CELL;
        let z = self.origin.1 + (c.1 as f32 + 0.5) * CELL;
        x.abs() < REACH && z.abs() < REACH && c.0 >= 1 && c.1 >= 1 && c.0 < N as i32 - 1 && c.1 < N as i32 - 1
    }

    /// Får rommet plass: inne i rutenettet og verden, og en celle fritt
    /// rundt (så gangene kan gå forbi).
    fn fits(&self, p: &Placed) -> bool {
        let (w, d) = p.size();
        for z in p.gz - 1..p.gz + d + 1 {
            for x in p.gx - 1..p.gx + w + 1 {
                let inside = x >= p.gx && x < p.gx + w && z >= p.gz && z < p.gz + d;
                if inside && !self.in_world((x, z)) {
                    return false;
                }
                let o = self.owner((x, z));
                if o >= 0 || (inside && o != -1) {
                    return false;
                }
            }
        }
        true
    }

    fn add(&mut self, p: Placed) -> usize {
        let k = self.rooms.len();
        let (w, d) = p.size();
        for z in p.gz..p.gz + d {
            for x in p.gx..p.gx + w {
                if let Some(i) = Floor::idx(x, z) {
                    self.owner[i] = k as i32;
                }
            }
        }
        self.rooms.push(p);
        k
    }

    fn remove_last(&mut self) {
        let k = self.rooms.len() as i32 - 1;
        for o in self.owner.iter_mut() {
            if *o == k {
                *o = -1;
            }
        }
        self.rooms.pop();
    }

    /// Ledige dører i et rom: (dørnummer, cella utenfor, retningen ut).
    fn free_doors(&self, r: usize) -> Vec<(usize, (i32, i32), (i32, i32))> {
        self.rooms[r]
            .doors()
            .into_iter()
            .enumerate()
            .filter(|(n, _)| !self.used.contains(&(r, *n)))
            .map(|(n, (c, d))| (n, (c.0 + d.0, c.1 + d.1), d))
            .filter(|(_, out, _)| self.owner(*out) == -1 || self.owner(*out) == -2)
            .collect()
    }

    /// Den korteste gangen mellom to celler utenfor rom (A*): rette strekk
    /// er billigere enn svinger, og gamle ganger kan gjenbrukes.
    fn route(&self, a: (i32, i32), b: (i32, i32), max: usize) -> Option<Vec<(i32, i32)>> {
        if !self.in_world(a) || !self.in_world(b) || !matches!(self.owner(a), -1 | -2) || !matches!(self.owner(b), -1 | -2) {
            return None;
        }
        // Tilstand: celle og retningen vi kom fra (for svingene).
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let key = |c: (i32, i32), d: usize| (c.1 as usize * N + c.0 as usize) * 5 + d;
        let mut best = vec![u32::MAX; N * N * 5];
        let mut prev = vec![usize::MAX; N * N * 5];
        let mut heap = BinaryHeap::new();
        let h = |c: (i32, i32)| ((c.0 - b.0).abs() + (c.1 - b.1).abs()) as u32 * 10;
        best[key(a, 4)] = 0;
        heap.push(std::cmp::Reverse((h(a), 0u32, a, 4usize)));
        while let Some(std::cmp::Reverse((_, g, c, from))) = heap.pop() {
            if g > best[key(c, from)] {
                continue;
            }
            if c == b {
                let mut path = vec![c];
                let mut k = key(c, from);
                while prev[k] != usize::MAX {
                    k = prev[k];
                    let cell = k / 5;
                    path.push(((cell % N) as i32, (cell / N) as i32));
                }
                path.reverse();
                return (path.len() <= max).then_some(path);
            }
            for (di, (dx, dz)) in dirs.iter().enumerate() {
                let n = (c.0 + dx, c.1 + dz);
                if !self.in_world(n) || !matches!(self.owner(n), -1 | -2) {
                    continue;
                }
                let step = if self.owner(n) == -2 { 7 } else { 10 } + if from != 4 && from != di { 6 } else { 0 };
                let ng = g + step;
                if ng < best[key(n, di)] {
                    best[key(n, di)] = ng;
                    prev[key(n, di)] = key(c, from);
                    heap.push(std::cmp::Reverse((ng + h(n), ng, n, di)));
                }
            }
        }
        None
    }

    /// Legg gangen og bind dørene til den.
    fn connect(&mut self, ra: usize, da: usize, rb: usize, db: usize, path: &[(i32, i32)]) {
        for c in path {
            if let Some(i) = Floor::idx(c.0, c.1) {
                self.owner[i] = -2;
            }
        }
        for w in path.windows(2) {
            self.links.insert(edge(w[0], w[1]));
        }
        for (r, d) in [(ra, da), (rb, db)] {
            let (c, dir) = self.rooms[r].doors()[d];
            self.links.insert(edge(c, (c.0 + dir.0, c.1 + dir.1)));
            self.used.insert((r, d));
        }
    }

    /// Prøv å binde to rom sammen med en gang (de nærmeste ledige dørene først).
    fn join(&mut self, ra: usize, rb: usize, max: usize) -> bool {
        let mut pairs = vec![];
        for (na, oa, _) in self.free_doors(ra) {
            for (nb, ob, _) in self.free_doors(rb) {
                pairs.push(((oa.0 - ob.0).abs() + (oa.1 - ob.1).abs(), na, oa, nb, ob));
            }
        }
        pairs.sort();
        for (_, na, oa, nb, ob) in pairs.into_iter().take(6) {
            if let Some(path) = self.route(oa, ob, max) {
                self.connect(ra, na, rb, nb, &path);
                return true;
            }
        }
        false
    }

    /// Heng et nytt rom på en ledig dør i rom `from`, med en gang på 1–4
    /// celler imellom. `forward` er retningen etasjen vokser (x).
    fn grow(&mut self, rng: &mut Rng, from: usize, name: &'static str, rot: Option<i32>, forward: i32) -> Option<usize> {
        let mut doors = self.free_doors(from);
        // Fram foretrekkes, så til sidene; aldri bakover.
        doors.retain(|(_, _, d)| d.0 != -forward);
        doors.sort_by_key(|(_, _, d)| if d.0 == forward { 0 } else { 1 });
        for (_, out, d) in doors {
            for _ in 0..24 {
                let r = rot.unwrap_or((rng.next() * 4.0) as i32 % 4);
                let mut cand = Placed { name, gx: 0, gz: 0, rot: r };
                let theirs = cand.doors();
                // En dør i det nye rommet som vender mot oss.
                let facing: Vec<_> = theirs.iter().enumerate().filter(|(_, (_, td))| *td == (-d.0, -d.1)).collect();
                if facing.is_empty() {
                    continue;
                }
                let (_, (tc, td)) = facing[(rng.next() * facing.len() as f32) as usize % facing.len()];
                let gap = 1 + (rng.next() * 3.0) as i32;
                let side = if rng.chance(0.5) { 0 } else { (rng.next() * 5.0) as i32 - 2 };
                // Cella utenfor deres dør ligger `gap` celler fram fra vår, litt til siden.
                let (px, pz) = (-d.1, d.0);
                let their_out = (out.0 + d.0 * gap + px * side, out.1 + d.1 * gap + pz * side);
                let their_door = (their_out.0 - td.0, their_out.1 - td.1);
                cand.gx = their_door.0 - tc.0;
                cand.gz = their_door.1 - tc.1;
                if !self.fits(&cand) {
                    continue;
                }
                let k = self.add(cand);
                let tn = self.rooms[k].doors().iter().position(|x| x.0 == their_door && x.1 == *td).unwrap_or(0);
                let dn = self.rooms[from].doors().iter().position(|(c, dd)| (c.0 + dd.0, c.1 + dd.1) == out && *dd == d).unwrap_or(0);
                if let Some(path) = self.route(out, their_out, 12) {
                    self.connect(from, dn, k, tn, &path);
                    return Some(k);
                }
                self.remove_last();
            }
        }
        None
    }
}

/// Graver ut og bygger alle etasjene. `seed` gir nye etasjer om byen
/// bygges på nytt; i testene er den fast.
pub fn build(b: &mut Builder, seed: u64, anchor: Vec2) -> Dungeon {
    let mut rng = Rng(seed | 1);
    let mut d = Dungeon { floors: vec![], start: Vec3::ZERO, spawns: vec![], elites: vec![], gates: vec![], loot: vec![], chests: vec![], boss: None, shortcuts: vec![], obstacles: vec![], near: HashMap::new() };
    let c0 = b.colliders.len();
    let mut y = DEPTH;
    // Tårnet fra etasjen over: cella i verden (midten) og dreiningen.
    let mut tower: Option<(Vec2, i32)> = None;
    let mut forward = 1;
    for k in 0..FLOORS {
        let last = k == FLOORS - 1;
        let floor = plan_floor(b, &mut d, &mut rng, k, y, anchor, tower, forward, last);
        tower = floor.1;
        d.floors.push(floor.0);
        forward = -forward;
        y -= FLOOR_DROP;
    }
    d.index_obstacles(&b.colliders[c0..]);
    // Det du kan bruke: portene og kistene.
    for (i, (p, yaw)) in d.gates.iter().enumerate() {
        let pose = Affine3A::from_rotation_translation(Quat::from_rotation_y(*yaw), *p);
        b.target(Use::Gate(i), pose, Vec3::new(0.0, GATE_H * 0.5, 0.0), Vec3::new(GATE_W * 0.5, GATE_H * 0.5, 0.12));
    }
    for (i, sc) in d.shortcuts.iter().enumerate() {
        let pose = Affine3A::from_rotation_translation(Quat::from_rotation_y(sc.yaw), sc.pos);
        b.target(Use::Shortcut(i), pose, Vec3::new(0.0, SHORTCUT_H * 0.5, SHORTCUT_HINGE.z), Vec3::new(SHORTCUT_W * 0.5, SHORTCUT_H * 0.5, 0.08));
    }
    for (i, (p, yaw, _)) in d.chests.iter().enumerate() {
        b.target(Use::Chest(i), Affine3A::from_rotation_translation(Quat::from_rotation_y(*yaw), *p), Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.48, 0.33, 0.33));
    }
    log::info!("Kjelleren: {} etasjer, {} fiender, {} porter, {} kister", d.floors.len(), d.spawns.len(), d.gates.len(), d.chests.len());
    d
}

/// Plass til portgitrene, som spillet poserer etter hvor åpne de er.
pub fn reserve(d: &Dungeon) -> Vec<(&'static str, usize)> {
    vec![(PORTCULLIS, d.gates.len()), (CHEST_LID, d.chests.len()), (SHORTCUT_LEAF, d.shortcuts.len())]
}

/// Planlegg og bygg én etasje. Gir etasjen og tårnet ned (midten i verden
/// og dreiningen) om det er et.
#[allow(clippy::too_many_arguments)]
fn plan_floor(b: &mut Builder, d: &mut Dungeon, rng: &mut Rng, k: usize, y: f32, anchor: Vec2, from_above: Option<(Vec2, i32)>, forward: i32, last: bool) -> (Floor, Option<(Vec2, i32)>) {
    let n = N as i32;
    // Startrommet ved kanten bakerst, midt på i z.
    let first = match from_above {
        None => Placed { name: "dng_start", gx: 0, gz: 0, rot: 0 },
        Some((_, rot)) => Placed { name: "dng_landing", gx: 0, gz: 0, rot },
    };
    let (w0, d0) = first.size();
    let first = Placed { gx: if forward > 0 { 2 } else { n - 2 - w0 }, gz: n / 2 - d0 / 2, ..first };
    // Opphavet: startstedet under krypten, eller tårncella under tårnet over.
    let origin = match from_above {
        None => {
            let s = first.spec().spots.start.unwrap_or([6.0, 0.0, 6.0, 0.0]);
            let p = first.turn(Vec2::new(s[0], s[2]));
            (anchor.x - first.gx as f32 * CELL - p.x, anchor.y - first.gz as f32 * CELL - p.y)
        }
        Some((at, _)) => {
            let t = first.spec().spots.tower.unwrap_or([1, 3]);
            let c = first.cell(t[0], t[1]);
            (at.x - (c.0 as f32 + 0.5) * CELL, at.y - (c.1 as f32 + 0.5) * CELL)
        }
    };
    // Rutenettet flyttes så startrommet havner på plass (hele celler).
    let mut plan = Plan { rooms: vec![], owner: vec![-1; N * N], links: HashSet::new(), used: HashSet::new(), origin };
    plan.add(first);

    // Hovedveien: 5–6 rom, så portrommet (eller rotunden nederst).
    let mut uses: HashMap<&str, usize> = HashMap::new();
    let target = 5 + (rng.next() * 2.0) as usize;
    let mut main = vec![0usize];
    let mut tries = 0;
    while main.len() < target + 1 && tries < 60 {
        tries += 1;
        let opts: Vec<&str> = POOL.iter().filter(|(nm, max)| uses.get(nm).copied().unwrap_or(0) < *max && plan.rooms.last().map(|r| r.name) != Some(nm)).map(|(nm, _)| *nm).collect();
        if opts.is_empty() {
            break;
        }
        let name = opts[(rng.next() * opts.len() as f32) as usize % opts.len()];
        // Fra det siste rommet, eller et tidligere om det står fast.
        let from = if tries % 4 == 0 && main.len() > 1 { main[main.len() - 2] } else { *main.last().unwrap() };
        if let Some(r) = plan.grow(rng, from, name, None, forward) {
            *uses.entry(name).or_default() += 1;
            main.push(r);
        }
    }
    // Utgangen: portrommet med tårnet fram (lokal +z mot `forward`), eller rotunden.
    let exit_name = if last { "dng_arena" } else { "dng_gate" };
    let exit_rot = if last { None } else { Some(if forward > 0 { 3 } else { 1 }) };
    let mut exit = None;
    for &from in main.iter().rev() {
        if let Some(r) = plan.grow(rng, from, exit_name, exit_rot, forward) {
            exit = Some(r);
            break;
        }
    }
    if exit.is_none() {
        log::warn!("Etasje {k}: fant ikke plass til {exit_name}");
    }
    // Sidegrener til skattkamre, og en løkke på hovedveien.
    for _ in 0..2 {
        let from = main[(rng.next() * main.len() as f32) as usize % main.len()];
        let way = if rng.chance(0.5) { forward } else { -forward };
        plan.grow(rng, from, "dng_treasury", None, way);
    }
    'loop_: for i in 0..main.len() {
        for j in (i + 2..main.len()).rev() {
            if plan.join(main[i], main[j], 16) {
                break 'loop_;
            }
        }
    }
    // Snarveien: en gang fra et rom langt inne tilbake til startrommet, med
    // en dør inn til startrommet som bare kan åpnes fra gangsida.
    let mut shortcut = None;
    for j in (3..main.len()).rev() {
        let before = plan.used.clone();
        if plan.join(main[0], main[j], 26) {
            shortcut = plan.used.difference(&before).find(|(r, _)| *r == main[0]).copied();
            break;
        }
    }

    // Cellene i etasjen.
    let mut cells = vec![Cell::Rock; N * N];
    let mut hw = vec![2.0; N * N];
    let mut ceil = vec![HALL_CEIL; N * N];
    let mut unit = vec![-1; N * N];
    let mut blocked = HashSet::new();
    for (r, p) in plan.rooms.iter().enumerate() {
        let s = p.spec();
        for j in 0..s.d {
            for i in 0..s.w {
                let c = p.cell(i, j);
                let Some(ix) = Floor::idx(c.0, c.1) else { continue };
                unit[ix] = r as i32;
                ceil[ix] = s.ceil[j as usize][i as usize];
                hw[ix] = s.hw[j as usize][i as usize];
                if let Some(h) = s.floor[j as usize][i as usize] {
                    cells[ix] = Cell::Floor(h);
                }
            }
        }
        for &(i, j, dx, dz, lo, hi) in &s.ramps {
            let c = p.cell(i, j);
            let (dx, dz) = p.dir((dx, dz));
            if let Some(ix) = Floor::idx(c.0, c.1) {
                cells[ix] = Cell::Ramp { dx, dz, lo, hi };
            }
        }
        for &(i, j, dx, dz) in &s.blocked {
            let c = p.cell(i, j);
            let (dx, dz) = p.dir((dx, dz));
            blocked.insert(edge(c, (c.0 + dx, c.1 + dz)));
        }
    }
    let mut halls = vec![];
    for (i, o) in plan.owner.iter().enumerate() {
        if *o == -2 {
            cells[i] = Cell::Floor(0.0);
            hw[i] = HALL_HW;
            ceil[i] = HALL_CEIL;
            halls.push(((i % N) as i32, (i / N) as i32));
        }
    }
    let start_room = &plan.rooms[0];
    let start_spot = start_room.spec().spots.start.unwrap_or([6.0, 0.0, 6.0, 0.0]);
    let sp = start_room.turn(Vec2::new(start_spot[0], start_spot[2]));
    let start_world = Vec3::new(origin.0 + start_room.gx as f32 * CELL + sp.x, y, origin.1 + start_room.gz as f32 * CELL + sp.y);
    let mut floor = Floor { cells, hw, ceil, unit, links: plan.links.clone(), blocked, closed: HashSet::new(), origin, y, theme: k, start: (0, 0), exit: None };
    floor.start = floor.cell_of(start_world);
    if k == 0 {
        d.start = start_world;
    }
    // Snarveidøra i døråpningen til startrommet; kanten er stengt til den
    // åpnes fra gangsida.
    if let Some((r, n)) = shortcut {
        let (c, dir) = plan.rooms[r].doors()[n];
        let e = edge(c, (c.0 + dir.0, c.1 + dir.1));
        floor.closed.insert(e);
        let mid = floor.center(c.0, c.1);
        let pos = Vec3::new(mid.x + dir.0 as f32 * CELL * 0.5, y + floor_height(&floor, c), mid.z + dir.1 as f32 * CELL * 0.5);
        let yaw = (-dir.0 as f32).atan2(-dir.1 as f32);
        put(b, SHORTCUT, Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), pos), &theme(k));
        d.shortcuts.push(Shortcut { pos, yaw, floor: k, edge: e });
        log::info!("Etasje {k}: snarvei fra startrommet ved {c:?}");
    }

    // Modellene, kolliderne, lysene og stedene i rommene.
    let mut tower_down = None;
    let mut candidates: Vec<Vec3> = vec![];
    let mut chest_spots: Vec<(Vec3, f32)> = vec![];
    for (r, p) in plan.rooms.iter().enumerate() {
        let t = p.affine(origin, y);
        let th = theme(k);
        put(b, p.name, t, &th);
        let (w, dd) = p.size();
        log::debug!("  {} ved ({:.1}, {:.1}, {:.1})", p.name, origin.0 + (p.gx as f32 + w as f32 * 0.5) * CELL, y, origin.1 + (p.gz as f32 + dd as f32 * 0.5) * CELL);
        let s = p.spec();
        let at = |q: [f32; 3]| t.transform_point3(Vec3::from(q));
        for q in &s.spots.spawn {
            if r != 0 {
                candidates.push(at(*q));
            }
        }
        for q in &s.spots.chest {
            chest_spots.push((at([q[0], q[1], q[2]]), q[3] + p.yaw()));
        }
        if let Some(q) = s.spots.elite {
            d.elites.push(at(q));
        }
        if let Some(q) = s.spots.boss {
            d.boss = Some(at(q));
        }
        for q in &s.spots.loot {
            d.loot.push(at(*q));
        }
        if let Some(q) = s.spots.gate {
            d.gates.push((at([q[0], q[1], q[2]]), q[3] + p.yaw()));
        }
        if let (Some(q), true) = (s.spots.ascend, k == 0) {
            let pose = Affine3A::from_rotation_translation(Quat::from_rotation_y(q[3] + p.yaw()), at([q[0], q[1], q[2]]));
            b.place("crypt_stair", pose);
            b.target(Use::Ascend, pose, Vec3::new(0.0, 1.2, 0.0), Vec3::new(0.75, 1.2, 0.2));
            b.solid_box(pose.transform_point3(Vec3::new(0.0, 1.8, -0.15)), Vec3::new(3.0, 3.6, 0.3), q[3] + p.yaw());
        }
        if let (Some(c), true) = (s.spots.tower, p.name == "dng_gate") {
            // Vindeltrappa står midt i tårncella, dreid som rommet.
            let cell = p.cell(c[0], c[1]);
            let mid = Vec3::new(origin.0 + (cell.0 as f32 + 0.5) * CELL, y, origin.1 + (cell.1 as f32 + 0.5) * CELL);
            put(b, "dng_spiral", Affine3A::from_rotation_translation(Quat::from_rotation_y(p.yaw()), mid), &th);
            tower_down = Some((Vec2::new(mid.x, mid.z), p.rot));
            floor.exit = Some(cell);
        }
        // Pynten på de ledige stedene i rommet.
        for q in &s.spots.wall {
            if !th.wall.is_empty() && rng.chance(th.density) {
                let name = th.wall[(rng.next() * th.wall.len() as f32) as usize % th.wall.len()];
                let n = t.transform_vector3(Vec3::new(q[2], 0.0, q[3]));
                put_deco(b, name, Affine3A::from_rotation_translation(Quat::from_rotation_y(n.x.atan2(n.z)), at([q[0], 0.0, q[1]])), &th, false);
            }
        }
        for q in &s.spots.floor {
            if !th.floor.is_empty() && rng.chance(th.density) {
                let name = th.floor[(rng.next() * th.floor.len() as f32) as usize % th.floor.len()];
                put_deco(b, name, Affine3A::from_rotation_translation(Quat::from_rotation_y(rng.next() * 6.3), at(*q)), &th, false);
            }
        }
        // Dørene ingen gang går til, mures igjen.
        for (n, (c, dir)) in p.doors().into_iter().enumerate() {
            if plan.used.contains(&(r, n)) {
                continue;
            }
            let mid = floor.center(c.0, c.1);
            let at = Vec3::new(mid.x + dir.0 as f32 * CELL * 0.5, y + floor_height(&floor, c), mid.z + dir.1 as f32 * CELL * 0.5);
            put(b, "dng_seal", Affine3A::from_rotation_translation(Quat::from_rotation_y((-dir.0 as f32).atan2(-dir.1 as f32)), at), &th);
        }
    }
    // Gangene: modulen etter hvilke naboer cella er bundet til.
    for c in &halls {
        let mut arms = vec![];
        for (dx, dz) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            if floor.links.contains(&edge(*c, (c.0 + dx, c.1 + dz))) {
                arms.push((dx, dz));
            }
        }
        let (name, rot) = hall_module(&arms, k, rng);
        let p = Placed { name, gx: c.0, gz: c.1, rot };
        let th = theme(k);
        put(b, name, p.affine(origin, y), &th);
        // Pynt i de rette gangene uten fakkel: på muren ved siden av
        // pilasteren, på gulvet og i hvelvet.
        if name == "dng_hall_i" {
            let mid = floor.center(c.0, c.1);
            let along = Vec3::new(p.dir((0, 1)).0 as f32, 0.0, p.dir((0, 1)).1 as f32);
            let side = Vec3::new(-along.z, 0.0, along.x) * if rng.chance(0.5) { 1.0 } else { -1.0 };
            if !th.wall.is_empty() && rng.chance(th.density * 0.5) {
                let name = th.wall[(rng.next() * th.wall.len() as f32) as usize % th.wall.len()];
                let at = mid + side * HALL_HW + along * 1.05;
                put_deco(b, name, Affine3A::from_rotation_translation(Quat::from_rotation_y((-side.x).atan2(-side.z)), at), &th, false);
            }
            if !th.floor.is_empty() && rng.chance(th.density * 0.4) {
                let name = th.floor[(rng.next() * th.floor.len() as f32) as usize % th.floor.len()];
                let at = if name == "dngp_puddle" || name == "dngp_lava" { mid - along * 0.9 } else { mid - along * 1.0 - side * 0.8 };
                put_deco(b, name, Affine3A::from_rotation_translation(Quat::from_rotation_y(rng.next() * 6.3), at), &th, false);
            }
            if !th.ceiling.is_empty() && rng.chance(th.density * 0.5) {
                let name = th.ceiling[(rng.next() * th.ceiling.len() as f32) as usize % th.ceiling.len()];
                put_deco(b, name, Affine3A::from_rotation_translation(Quat::from_rotation_y(rng.next() * 6.3), mid + along * 1.0 + Vec3::Y * HALL_CEIL), &th, true);
            }
        }
        if rng.chance(0.12) {
            candidates.push(floor.center(c.0, c.1));
        }
    }
    // Hjørnene i rommene som fiendene ikke går i (utenfor rotunden), har
    // også gulv: pynten ved murene står der.
    for p in plan.rooms.iter().filter(|p| p.name != "dng_gate" && p.name != "dng_landing") {
        let s = p.spec();
        for j in 0..s.d {
            for i in 0..s.w {
                let c = p.cell(i, j);
                if s.floor[j as usize][i as usize].is_none() && Floor::idx(c.0, c.1).is_some() {
                    let m = Vec3::new(origin.0 + (c.0 as f32 + 0.5) * CELL, y, origin.1 + (c.1 as f32 + 0.5) * CELL);
                    b.floor(m, Vec2::splat(CELL * 0.5), 0.0, y);
                }
            }
        }
    }
    // Gulvene til rutinen mot svevende modeller (kistene står på dem).
    for iz in 0..n {
        for ix in 0..n {
            if let Cell::Floor(h) = floor.cell(ix, iz) {
                let c = floor.center(ix, iz);
                b.floor(Vec3::new(c.x, y + h, c.z), Vec2::splat(CELL * 0.5), 0.0, y + h);
            }
        }
    }
    // Kistene: opptil fire per etasje.
    let mut placed_chests = 0;
    for (p, yaw) in chest_spots {
        if placed_chests >= 4 || !rng.chance(0.7) {
            continue;
        }
        if b.place(CHEST, Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p)) {
            b.solid_box(p + Vec3::Y * 0.3, Vec3::new(0.9, 0.6, 0.6), yaw);
            d.chests.push((p, yaw, k));
            placed_chests += 1;
        }
    }
    // Tolv fiender: to nær start (3–5 celler), tre litt lenger unna, resten
    // hvor som helst.
    let dist = |p: &Vec3| {
        let c = floor.cell_of(*p);
        (c.0 - floor.start.0).abs() + (c.1 - floor.start.1).abs()
    };
    for (lo, hi, want) in [(2, 6, 2), (5, 12, 3), (6, 99, 7)] {
        let mut pool: Vec<Vec3> = candidates.iter().copied().filter(|p| (lo..=hi).contains(&dist(p))).collect();
        let mut got = 0;
        while got < want && !pool.is_empty() {
            let i = (rng.next() * pool.len() as f32) as usize % pool.len();
            d.spawns.push(pool.swap_remove(i));
            got += 1;
        }
    }
    let s = start_world;
    log::info!("Etasje {k} ({}): {} rom, {} gangceller, start ({:.0}, {:.0}, {:.0})", EPOCHS[k], plan.rooms.len(), halls.len(), s.x, s.y, s.z);
    if log::log_enabled!(log::Level::Debug) {
        let mut map = String::new();
        for iz in 0..n {
            for ix in 0..n {
                let o = plan.owner[iz as usize * N + ix as usize];
                map.push(match o {
                    -2 => '.',
                    -1 => ' ',
                    r => (b'A' + (r as u8 % 26)) as char,
                });
            }
            map.push('\n');
        }
        log::debug!("Etasje {k}:\n{map}");
    }
    (floor, tower_down)
}

fn floor_height(f: &Floor, c: (i32, i32)) -> f32 {
    match f.cell(c.0, c.1) {
        Cell::Floor(h) => h,
        _ => 0.0,
    }
}

/// Gangmodulen og dreiningen for armene (retninger ut av cella).
fn hall_module(arms: &[(i32, i32)], k: usize, rng: &mut Rng) -> (&'static str, i32) {
    let canon: &[(&str, &[(i32, i32)])] = &[
        ("dng_hall_i", &[(0, -1), (0, 1)]),
        ("dng_hall_l", &[(0, -1), (1, 0)]),
        ("dng_hall_t", &[(0, -1), (1, 0), (0, 1)]),
        ("dng_hall_x", &[(0, -1), (1, 0), (0, 1), (-1, 0)]),
        ("dng_hall_end", &[(0, 1)]),
    ];
    let mut want: Vec<(i32, i32)> = arms.to_vec();
    want.sort();
    for (name, local) in canon {
        if local.len() != want.len() {
            continue;
        }
        for rot in 0..4 {
            let p = Placed { name, gx: 0, gz: 0, rot };
            let mut got: Vec<(i32, i32)> = local.iter().map(|d| p.dir(*d)).collect();
            got.sort();
            if got == want {
                let name = if *name == "dng_hall_i" {
                    let r = rng.next();
                    // Etasjene med eget tema får sine egne rette ganger:
                    // kloakkrenne, gravnisjer og glødende sprekker.
                    let own = match k % FLOORS {
                        2 => Some("dng_hall_sewer"),
                        3 => Some("dng_hall_crypt"),
                        4 => Some("dng_hall_lava"),
                        _ => None,
                    };
                    if let Some(own) = own.filter(|_| r < 0.55) {
                        own
                    } else if r < 0.2 {
                        "dng_hall_i2"
                    } else if r < 0.3 {
                        "dng_hall_i3"
                    } else {
                        "dng_hall_i"
                    }
                } else {
                    name
                };
                return (name, rot);
            }
        }
    }
    ("dng_hall_x", 0)
}

/// Sett ut en kjellermodell med etasjens utseende, og kolliderne og lysene
/// fra json-fila. Murverket går ned under gulvet, så rutinen mot svevende
/// modeller hopper over det.
fn put(b: &mut Builder, name: &'static str, t: Affine3A, th: &Theme) {
    if !b.place_mounted_look(name, t, Some(th.look)) {
        return;
    }
    extras(b, name, t, th);
}

/// Pynt: står på gulvet (rutinen mot svevende modeller måler den) eller
/// henger i hvelvet (`hanging`).
fn put_deco(b: &mut Builder, name: &'static str, t: Affine3A, th: &Theme, hanging: bool) {
    // Bannere henger på muren, røtter i hvelvet.
    let placed = if hanging || name == "dngp_banner" { b.place_mounted(name, t) } else { b.place(name, t) };
    if placed {
        extras(b, name, t, th);
    }
}

/// Kolliderne og lysene fra json-fila, med lyset i etasjens farge.
fn extras(b: &mut Builder, name: &'static str, t: Affine3A, th: &Theme) {
    let s = spec(name);
    let yaw = {
        let x = t.transform_vector3(Vec3::X);
        (-x.z).atan2(x.x)
    };
    for (c, size, y) in &s.boxes {
        b.solid_box(t.transform_point3(Vec3::from(*c)), Vec3::from(*size), yaw + y);
    }
    for pts in &s.hulls {
        b.solid(Shape::Hull { points: pts.iter().map(|p| t.transform_point3(Vec3::from(*p))).collect() });
    }
    for (p, color, intensity, radius) in &s.lights {
        let color = th.light.unwrap_or(Vec3::from(*color));
        b.fire_light(t.transform_point3(Vec3::from(*p)), color, *intensity * th.gain, *radius);
    }
}
