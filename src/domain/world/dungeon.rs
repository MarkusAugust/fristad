//! Legacy-kjelleren: fem etasjer under borgruinen, én per epoke, der de
//! korrupte komponentene holder til. Du kommer ned gjennom døra i krypta
//! (`Use::Descend`) og opp igjen gjennom døra øverst i trappa i
//! startrommet (`Use::Ascend`). Hver etasje er et rutenett av celler
//! på 4 m: rom fra en BSP-deling, ganger som binder dem sammen, noen rom
//! hevet 1,4 m med trapper opp og avsatser du kan hoppe ned fra, og en
//! utgang i øst: en alkove bak en rusten port (`Use::Gate`, nøkkelen
//! bærer elitefienden i rommet) med en trapp ned til neste
//! etasje. Temaet følger epoken: murstein med mose og fakler, serverrom
//! med lysrør og kabler, kloakk med gjørme, vann og lykter, katakomber
//! med stearinlys og hodeskaller, og lavadypet nederst. Fiendene
//! (`domain/mobs.rs`) går på rutenettet; spilleren går på kolliderne.

use super::{Builder, Shape, Use, IRON, OAK, STONE};
use crate::domain::rng::Rng;
use crate::domain::geometry::{flag, kind, Layer, Mat};
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::collections::{HashSet, VecDeque};

/// Gulvets overkant i den første etasjen, under alt i byen.
pub const DEPTH: f32 = -16.0;
pub const CELL: f32 = 4.0;
/// Takhøyden i ganger og i rom.
pub const WALL_H: f32 = 3.0;
pub const ROOM_H: f32 = 4.4;
/// Hvor høyt de hevede rommene ligger.
pub const LEDGE: f32 = 1.4;
/// Høyden mellom etasjene.
pub const FLOOR_DROP: f32 = 12.0;
pub const FLOORS: usize = 5;
/// Rutenettet er N × N celler.
pub const N: usize = 28;
/// Startrommet (3 × 3) ligger rundt denne cellen; i første etasje rett
/// under krypta.
pub const START: (usize, usize) = (4, 14);
/// Portgitteret som sveives opp foran trappa ned (modell, posert av spillet).
pub const PORTCULLIS: &str = "portcullis";
/// Trappa mellom etasjene: så mange celler lang, mot øst.
const SHAFT: i32 = 5;
pub const EPOCHS: [&str; FLOORS] = ["2026: forretningslogikken", "2019: klassekomponentene", "2014: jQuery-kloakken", "2009: tabellayout-katakombene", "1999: blink-dypet"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Rock,
    /// Gulv på nivå 0 eller 1 (hevet `LEDGE`).
    Floor(u8),
    /// Trapp som stiger mot (dx, dz), fra nivå 0 til 1.
    Stair(i32, i32),
}

#[derive(Clone, Debug)]
pub struct Floor {
    pub cells: Vec<Cell>,
    /// Rom (1) eller gang (2) per celle, 0 for fjell.
    kind: Vec<u8>,
    /// Takhøyde per celle.
    ceil: Vec<f32>,
    /// Verdenskoordinatene (x, z) til hjørnet av celle (0, 0).
    pub origin: (f32, f32),
    /// Gulvets overkant på nivå 0.
    pub y: f32,
    pub theme: usize,
    /// Avsatser: (høy celle, lav celle) der veggen mellom er borte over
    /// avsatsen, så du kan hoppe ned.
    ledges: HashSet<((i32, i32), (i32, i32))>,
    /// Veggåpninger som ikke skal tegnes: (celle, retning).
    openings: HashSet<((i32, i32), (i32, i32))>,
    pub start: (i32, i32),
    /// Alkoven med trappa ned (eller skatten nederst).
    pub exit: Option<(i32, i32)>,
    /// Retningen utgangen ligger i (1 øst, −1 vest): etasjene sikksakker
    /// under byen, så kjelleren holder seg innenfor landskapet.
    pub dir: i32,
}

#[derive(Clone, Debug)]
pub struct Dungeon {
    pub floors: Vec<Floor>,
    /// Føttene der du står når du kommer ned trappa fra krypta.
    pub start: Vec3,
    /// Steder fiendene kan starte, alle etasjer.
    pub spawns: Vec<Vec3>,
    /// Elitefiendene som bærer nøklene: en per etasje.
    pub elites: Vec<Vec3>,
    /// De rustne portene: (midten i gulvhøyde, yaw), en per etasje med trapp.
    pub gates: Vec<(Vec3, f32)>,
    /// Tokens som ligger og venter (skatten nederst).
    pub loot: Vec<Vec3>,
    /// Kistene: (føtter, yaw, etasje). F åpner dem (`Use::Chest`), og
    /// innholdet følger etasjen (`craft::LOOT`).
    pub chests: Vec<(Vec3, f32, usize)>,
    /// Der Rammeverket står: midt i utgangsrommet på nederste etasje.
    pub boss: Option<Vec3>,
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

    /// Cellen et verdenspunkt ligger i.
    pub fn cell_of(&self, p: Vec3) -> (i32, i32) {
        (((p.x - self.origin.0) / CELL).floor() as i32, ((p.z - self.origin.1) / CELL).floor() as i32)
    }

    /// Nivået i en celle (0 eller 1); trapper regnes som 0 i den ene enden.
    fn level(&self, ix: i32, iz: i32) -> u8 {
        match self.cell(ix, iz) {
            Cell::Floor(l) => l,
            _ => 0,
        }
    }

    /// Gulvhøyden i et verdenspunkt (absolutt y), trappene medregnet.
    pub fn height_at(&self, p: Vec3) -> f32 {
        let (ix, iz) = self.cell_of(p);
        match self.cell(ix, iz) {
            Cell::Floor(l) => self.y + l as f32 * LEDGE,
            Cell::Stair(dx, dz) => {
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
                self.y + t.clamp(0.0, 1.0) * LEDGE
            }
            Cell::Rock => self.y,
        }
    }

    /// Midten av en celle, på gulvet.
    pub fn center(&self, ix: i32, iz: i32) -> Vec3 {
        let p = Vec3::new(self.origin.0 + (ix as f32 + 0.5) * CELL, 0.0, self.origin.1 + (iz as f32 + 0.5) * CELL);
        Vec3::new(p.x, self.height_at(p), p.z)
    }

    /// Om du kan gå fra celle `a` til nabocellen `b`: begge åpne, og
    /// samme nivå, eller via en trapp langs dens akse.
    pub fn passable(&self, a: (i32, i32), b: (i32, i32)) -> bool {
        let (ca, cb) = (self.cell(a.0, a.1), self.cell(b.0, b.1));
        if ca == Cell::Rock || cb == Cell::Rock {
            return false;
        }
        let d = (b.0 - a.0, b.1 - a.1);
        // Nivået på hver side av overgangen.
        let side = |c: Cell, dir: (i32, i32), leaving: bool| -> Option<u8> {
            match c {
                Cell::Floor(l) => Some(l),
                Cell::Stair(dx, dz) => {
                    // Trappa kan bare brukes langs aksen sin: høy ende mot
                    // (dx, dz), lav ende motsatt.
                    if dir == (dx, dz) {
                        Some(if leaving { 1 } else { 0 })
                    } else if dir == (-dx, -dz) {
                        Some(if leaving { 0 } else { 1 })
                    } else {
                        None
                    }
                }
                Cell::Rock => None,
            }
        };
        match (side(ca, d, true), side(cb, d, false)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// Om et punkt (med margin inn fra veggene) er på gulvet, og innenfor
    /// rekkevidde av cellen det står i.
    pub fn walkable(&self, p: Vec3, margin: f32) -> bool {
        let (ix, iz) = self.cell_of(p);
        if !self.is_open(ix, iz) {
            return false;
        }
        let lx = p.x - (self.origin.0 + ix as f32 * CELL);
        let lz = p.z - (self.origin.1 + iz as f32 * CELL);
        (lx >= margin || self.passable((ix, iz), (ix - 1, iz)))
            && (lx <= CELL - margin || self.passable((ix, iz), (ix + 1, iz)))
            && (lz >= margin || self.passable((ix, iz), (ix, iz - 1)))
            && (lz <= CELL - margin || self.passable((ix, iz), (ix, iz + 1)))
    }

    /// Neste cellemidte på korteste vei fra `from` til `to` (bredde først
    /// på rutenettet), eller `None` om det ikke finnes noen vei.
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
                if Self::idx(n.0, n.1).is_some() && self.passable(c, n) && prev[idx(n)] == usize::MAX {
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
        self.floors.iter().position(|f| p.y >= f.y - 1.5 && p.y < f.y + FLOOR_DROP - 1.5)
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
}

#[derive(Clone, Copy, Debug)]
struct Room {
    x0: i32,
    z0: i32,
    x1: i32,
    z1: i32,
    level: u8,
}

impl Room {
    fn center(&self) -> (i32, i32) {
        ((self.x0 + self.x1) / 2, (self.z0 + self.z1) / 2)
    }
    fn contains(&self, c: (i32, i32)) -> bool {
        c.0 >= self.x0 && c.0 <= self.x1 && c.1 >= self.z0 && c.1 <= self.z1
    }
}

/// Temaet per epoke: materialer, lys og pynt.
#[derive(Clone, Copy)]
struct Theme {
    wall: Mat,
    floor: Mat,
    ceil: Mat,
    light: LightKind,
    moss: f32,
    mud: f32,
    puddles: f32,
    pillars: bool,
    skulls: f32,
    kegs: f32,
    cables: bool,
    pipes: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum LightKind {
    Torch,
    Tube,
    Lantern,
    Candle,
    Lava,
}

fn theme(k: usize) -> Theme {
    let nw = flag::NO_WEATHER;
    match k % FLOORS {
        0 => Theme {
            wall: Mat::tex(Layer::Brick, 0xffffff).tint(Vec3::new(0.42, 0.36, 0.33)).flags(nw),
            floor: Mat::tex(Layer::Concrete, 0xffffff).tint(Vec3::new(0.35, 0.34, 0.33)).flags(nw),
            ceil: Mat::tex(Layer::Stone, 0xffffff).tint(Vec3::new(0.3, 0.29, 0.28)).flags(nw),
            light: LightKind::Torch,
            moss: 0.45,
            mud: 0.0,
            puddles: 0.15,
            pillars: false,
            skulls: 0.05,
            kegs: 0.2,
            cables: false,
            pipes: false,
        },
        1 => Theme {
            wall: Mat::tex(Layer::Concrete, 0xffffff).tint(Vec3::new(0.46, 0.48, 0.52)).flags(nw),
            floor: Mat::tex(Layer::Concrete, 0xffffff).tint(Vec3::new(0.26, 0.27, 0.3)).flags(nw),
            ceil: Mat::tex(Layer::Metal, 0xffffff).tint(Vec3::new(0.3, 0.32, 0.36)).flags(nw),
            light: LightKind::Tube,
            moss: 0.0,
            mud: 0.0,
            puddles: 0.3,
            pillars: true,
            skulls: 0.0,
            kegs: 0.0,
            cables: true,
            pipes: false,
        },
        2 => Theme {
            wall: Mat::tex(Layer::Stone, 0xffffff).tint(Vec3::new(0.32, 0.35, 0.3)).flags(nw),
            floor: Mat::tex(Layer::Gravel, 0xffffff).tint(Vec3::new(0.24, 0.19, 0.13)).flags(nw),
            ceil: Mat::tex(Layer::Brick, 0xffffff).tint(Vec3::new(0.28, 0.26, 0.24)).flags(nw),
            light: LightKind::Lantern,
            moss: 0.7,
            mud: 0.6,
            puddles: 0.55,
            pillars: false,
            skulls: 0.08,
            kegs: 0.15,
            cables: false,
            pipes: true,
        },
        3 => Theme {
            wall: Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.5, 0.46, 0.42)).flags(nw),
            floor: Mat::tex(Layer::Stone, 0xffffff).tint(Vec3::new(0.38, 0.36, 0.33)).flags(nw),
            ceil: Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.32, 0.3, 0.28)).flags(nw),
            light: LightKind::Candle,
            moss: 0.15,
            mud: 0.0,
            puddles: 0.05,
            pillars: true,
            skulls: 0.45,
            kegs: 0.1,
            cables: false,
            pipes: false,
        },
        _ => Theme {
            wall: Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.24, 0.2, 0.18)).flags(nw),
            floor: Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.2, 0.17, 0.15)).flags(nw),
            ceil: Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.15, 0.13, 0.12)).flags(nw),
            light: LightKind::Lava,
            moss: 0.0,
            mud: 0.0,
            puddles: 0.0,
            pillars: true,
            skulls: 0.25,
            kegs: 0.0,
            cables: false,
            pipes: false,
        },
    }
}

/// Graver ut og bygger alle etasjene. `seed` gir nye etasjer om byen
/// bygges på nytt; i testene er den fast.
pub fn build(b: &mut Builder, seed: u64, anchor: Vec2) -> Dungeon {
    let mut rng = Rng(seed | 1);
    // Egen tilfeldighet til fiendenes startsteder, så kartet ikke endrer
    // seg når reglene for dem gjør det.
    let mut srng = Rng((seed | 1) ^ 0x5bd1_e995_1234_5678);
    let mut d = Dungeon { floors: vec![], start: Vec3::ZERO, spawns: vec![], elites: vec![], gates: vec![], loot: vec![], chests: vec![], boss: None };
    let mut origin = (anchor.x - (START.0 as f32 + 0.5) * CELL, anchor.y - (START.1 as f32 + 0.5) * CELL);
    let mut y = DEPTH;
    for k in 0..FLOORS {
        let last = k == FLOORS - 1;
        let (floor, rooms) = carve(origin, y, k, &mut rng, last);
        // Startstedet, fiendene og elitefienden i utgangsrommet.
        if k == 0 {
            d.start = floor.center(START.0 as i32, START.1 as i32);
        }
        // Tolv fiender per etasje: de fire første i nærheten av startrommet
        // (5–11 celler unna), resten hvor som helst.
        let mut count = 0;
        let mut tries = 0;
        while count < 12 && tries < 2000 {
            tries += 1;
            let (ix, iz) = ((srng.next() * N as f32) as i32, (srng.next() * N as f32) as i32);
            let far = (ix - floor.start.0).abs() + (iz - floor.start.1).abs();
            let ok = if count < 2 {
                (3..=5).contains(&far)
            } else if count < 5 {
                (5..=11).contains(&far)
            } else {
                far >= 6
            };
            if matches!(floor.cell(ix, iz), Cell::Floor(_)) && ok && floor.exit != Some((ix, iz)) {
                d.spawns.push(floor.center(ix, iz));
                count += 1;
            }
        }
        if let Some(exit_room) = rooms.last() {
            let c = exit_room.center();
            if last {
                d.boss = Some(floor.center(c.0, c.1));
            } else {
                d.elites.push(floor.center(c.0, c.1));
            }
        }
        if let Some(exit) = floor.exit {
            // Porten står i alkovens munning, mot rommet.
            let a = floor.center(exit.0, exit.1);
            let dir = floor.dir as f32;
            if !last {
                // Gitteret står i yz-planet (dreid 90°), med stengene langs z.
                d.gates.push((Vec3::new(a.x - dir * CELL * 0.5, floor.y, a.z), std::f32::consts::FRAC_PI_2));
            } else {
                for i in 0..12 {
                    let ang = i as f32 * 0.52;
                    d.loot.push(a + Vec3::new(ang.cos() * 1.2, 0.0, ang.sin() * 1.2));
                }
            }
            // Neste etasje begynner der trappa ender, og går motsatt vei.
            let next_start_x = if floor.dir > 0 { (N - 1 - START.0) as f32 } else { START.0 as f32 };
            origin = (a.x + dir * (SHAFT as f32 + 2.0) * CELL - (next_start_x + 0.5) * CELL, a.z - (START.1 as f32 + 0.5) * CELL);
        }
        mesh(b, &floor, &mut rng, !last, &mut d.chests);
        let s = floor.center(floor.start.0, floor.start.1);
        let e = floor.exit.map(|e| floor.center(e.0, e.1)).unwrap_or(Vec3::ZERO);
        log::info!("Etasje {k} ({}): start ({:.0}, {:.0}, {:.0}), utgang ({:.0}, {:.0}, {:.0})", EPOCHS[k], s.x, s.y, s.z, e.x, e.y, e.z);
        if log::log_enabled!(log::Level::Debug) {
            // Kartet: . gang, # rom, ^ hevet rom, / trapp, S start, E utgang.
            let mut map = String::new();
            for iz in 0..N as i32 {
                for ix in 0..N as i32 {
                    let ch = if (ix, iz) == floor.start {
                        'S'
                    } else if floor.exit == Some((ix, iz)) {
                        'E'
                    } else {
                        match floor.cell(ix, iz) {
                            Cell::Rock => ' ',
                            Cell::Stair(..) => '/',
                            Cell::Floor(1) => '^',
                            Cell::Floor(_) => {
                                if floor.kind[iz as usize * N + ix as usize] == 1 {
                                    '#'
                                } else {
                                    '.'
                                }
                            }
                        }
                    };
                    map.push(ch);
                }
                map.push('\n');
            }
            log::debug!("Etasje {k}:\n{map}");
        }
        d.floors.push(floor);
        y -= FLOOR_DROP;
    }
    // Det du kan bruke: portene, kistene, og døra opp i startrommet.
    for (i, (p, yaw)) in d.gates.iter().enumerate() {
        let pose = Affine3A::from_rotation_translation(Quat::from_rotation_y(*yaw), *p);
        b.target(Use::Gate(i), pose, Vec3::new(0.0, GATE_H * 0.5, 0.0), Vec3::new(GATE_W * 0.5, GATE_H * 0.5, 0.12));
        gate_frame(b, pose);
    }
    for (i, (p, yaw, _)) in d.chests.iter().enumerate() {
        b.target(Use::Chest(i), Affine3A::from_rotation_translation(Quat::from_rotation_y(*yaw), *p), Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.48, 0.33, 0.33));
    }
    // Trappa opp ligger inntil nordveggen i startrommet, med døra øverst.
    let door = d.start + Vec3::new(0.0, 0.0, -(1.5 * CELL - 0.35));
    b.place("crypt_stair", Affine3A::from_translation(door));
    b.target(Use::Ascend, Affine3A::from_translation(door), Vec3::new(0.0, 1.2, 0.0), Vec3::new(0.75, 1.2, 0.2));
    b.solid_box(door + Vec3::new(0.0, 1.5, -0.1), Vec3::new(1.8, 3.0, 0.3), 0.0);
    log::info!("Kjelleren: {} etasjer, {} fiender, {} porter", d.floors.len(), d.spawns.len(), d.gates.len());
    d
}

/// Portgitteret: så bredt og høyt, og hvor langt det sveives opp.
pub const GATE_W: f32 = 3.9;
pub const GATE_H: f32 = 2.9;
pub const GATE_LIFT: f32 = 2.7;

/// Rammen rundt portgitteret: to stolper og en overligger.
fn gate_frame(b: &mut Builder, rest: Affine3A) {
    let rust = IRON.tint(Vec3::new(0.32, 0.2, 0.13)).flags(flag::NO_WEATHER);
    for sx in [-1.0, 1.0] {
        b.mesh.cuboid(&(rest * Affine3A::from_translation(Vec3::new(sx * GATE_W * 0.5, GATE_H * 0.5 + 0.2, 0.0))), Vec3::new(0.2, GATE_H + 0.4, 0.3), &rust);
    }
    b.mesh.cuboid(&(rest * Affine3A::from_translation(Vec3::new(0.0, GATE_H + 0.35, 0.0))), Vec3::new(GATE_W + 0.2, 0.3, 0.3), &rust);
}

/// Plass til portgitrene, som spillet poserer etter hvor åpne de er.
pub fn reserve(d: &Dungeon) -> Vec<(&'static str, usize)> {
    vec![(PORTCULLIS, d.gates.len())]
}

/// Rom, ganger, nivåer, trapper og avsatser for én etasje.
fn carve(origin: (f32, f32), y: f32, k: usize, rng: &mut Rng, last: bool) -> (Floor, Vec<Room>) {
    let n = N as i32;
    let dir: i32 = if k % 2 == 0 { 1 } else { -1 };
    let mut cells = vec![Cell::Rock; N * N];
    let mut kind = vec![0u8; N * N];
    let idx = |ix: i32, iz: i32| iz as usize * N + ix as usize;
    // BSP: del arealet til bladene er små nok, ett rom i hvert blad.
    let mut rooms: Vec<Room> = vec![];
    let mut stack = vec![(1, 1, n - 2, n - 2, 0)];
    while let Some((x0, z0, x1, z1, depth)) = stack.pop() {
        let (w, h) = (x1 - x0 + 1, z1 - z0 + 1);
        let split = (w > 10 || h > 10 || (w > 7 && h > 7 && rng.chance(0.65))) && depth < 7;
        if split {
            let vertical = if w > h * 5 / 4 { true } else if h > w * 5 / 4 { false } else { rng.chance(0.5) };
            if vertical && w >= 8 {
                let cut = x0 + 4 + ((w - 8) as f32 * rng.next()) as i32;
                stack.push((x0, z0, cut - 1, z1, depth + 1));
                stack.push((cut, z0, x1, z1, depth + 1));
                continue;
            } else if !vertical && h >= 8 {
                let cut = z0 + 4 + ((h - 8) as f32 * rng.next()) as i32;
                stack.push((x0, z0, x1, cut - 1, depth + 1));
                stack.push((x0, cut, x1, z1, depth + 1));
                continue;
            }
        }
        // Et rom i bladet, med minst én celle marg.
        let rw = 3.max(((w - 2) as f32 * rng.range(0.55, 0.95)) as i32).min(w - 2).min(7);
        let rh = 3.max(((h - 2) as f32 * rng.range(0.55, 0.95)) as i32).min(h - 2).min(7);
        if rw < 3 || rh < 3 {
            continue;
        }
        let rx = x0 + 1 + ((w - 2 - rw) as f32 * rng.next()) as i32;
        let rz = z0 + 1 + ((h - 2 - rh) as f32 * rng.next()) as i32;
        rooms.push(Room { x0: rx, z0: rz, x1: rx + rw - 1, z1: rz + rh - 1, level: 0 });
    }
    // Startrommet: 3 × 3 rundt START, og rom som overlapper det fjernes.
    let s = (if dir > 0 { START.0 as i32 } else { n - 1 - START.0 as i32 }, START.1 as i32);
    let start_room = Room { x0: s.0 - 1, z0: s.1 - 1, x1: s.0 + 1, z1: s.1 + 1, level: 0 };
    rooms.retain(|r| r.x1 < start_room.x0 - 1 || r.x0 > start_room.x1 + 1 || r.z1 < start_room.z0 - 1 || r.z0 > start_room.z1 + 1);
    rooms.insert(0, start_room);
    // Utgangsrommet: det ytterste i utgangsretningen (ikke startrommet),
    // på nivå 0, sist i lista.
    let exit_i = (1..rooms.len()).max_by_key(|i| (if dir > 0 { rooms[*i].x1 } else { -rooms[*i].x0 }) * 64 + (rooms[*i].z1 - rooms[*i].z0)).unwrap_or(0);
    let exit_room = rooms.remove(exit_i);
    rooms.push(exit_room);
    let exit_room = *rooms.last().unwrap();
    let alcove = (if dir > 0 { exit_room.x1 + 1 } else { exit_room.x0 - 1 }, exit_room.center().1);
    // Trappesjakten videre i utgangsretningen er reservert.
    let reserved: HashSet<(i32, i32)> = (1..=SHAFT + 1).map(|i| (alcove.0 + dir * i, alcove.1)).collect();
    for r in &rooms {
        for iz in r.z0..=r.z1 {
            for ix in r.x0..=r.x1 {
                cells[idx(ix, iz)] = Cell::Floor(0);
                kind[idx(ix, iz)] = 1;
            }
        }
    }
    cells[idx(alcove.0, alcove.1)] = Cell::Floor(0);
    kind[idx(alcove.0, alcove.1)] = 1;
    // Gangene: hvert rom til det neste i bladrekkefølgen, pluss noen løkker.
    let mut links: Vec<(usize, usize)> = (0..rooms.len().saturating_sub(1)).map(|i| (i, i + 1)).collect();
    for _ in 0..3 {
        if rooms.len() > 3 {
            let a = (rng.next() * rooms.len() as f32) as usize % rooms.len();
            let b = (rng.next() * rooms.len() as f32) as usize % rooms.len();
            if a != b {
                links.push((a, b));
            }
        }
    }
    let carve_line = |cells: &mut Vec<Cell>, kind: &mut Vec<u8>, from: (i32, i32), to: (i32, i32), x_first: bool| -> bool {
        let mut path = vec![];
        let (mut x, mut z) = from;
        let step = |v: &mut i32, goal: i32| {
            if *v < goal {
                *v += 1
            } else if *v > goal {
                *v -= 1
            }
        };
        for leg in 0..2 {
            let horizontal = (leg == 0) == x_first;
            loop {
                if horizontal {
                    if x == to.0 {
                        break;
                    }
                    step(&mut x, to.0);
                } else {
                    if z == to.1 {
                        break;
                    }
                    step(&mut z, to.1);
                }
                path.push((x, z));
            }
        }
        if path.iter().any(|c| reserved.contains(c) || c.0 < 1 || c.1 < 1 || c.0 > n - 2 || c.1 > n - 2) {
            return false;
        }
        for (x, z) in path {
            let i = idx(x, z);
            if cells[i] == Cell::Rock {
                cells[i] = Cell::Floor(0);
                kind[i] = 2;
            }
        }
        true
    };
    for (a, b) in links {
        let (ca, cb) = (rooms[a].center(), rooms[b].center());
        let first = rng.chance(0.5);
        if !carve_line(&mut cells, &mut kind, ca, cb, first) {
            carve_line(&mut cells, &mut kind, ca, cb, !first);
        }
    }
    // Hevede rom (ikke start, ikke utgang), og trappene opp til dem: en
    // gangcelle som ligger inntil rommet og fortsetter bakover.
    let last_room = rooms.len() - 1;
    for (i, r) in rooms.iter_mut().enumerate() {
        if i != 0 && i != last_room && rng.chance(0.4) {
            r.level = 1;
        }
    }
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for r in rooms.iter_mut() {
        if r.level == 0 {
            continue;
        }
        let mut stairs = 0;
        for iz in r.z0..=r.z1 {
            for ix in r.x0..=r.x1 {
                for (dx, dz) in dirs {
                    let c = (ix + dx, iz + dz);
                    if r.contains(c) || Floor::idx(c.0, c.1).is_none() || kind[idx(c.0, c.1)] != 2 || cells[idx(c.0, c.1)] != Cell::Floor(0) {
                        continue;
                    }
                    // Fortsetter gangen bakover, og er sidene lukket?
                    let back = (c.0 + dx, c.1 + dz);
                    let sides = [(c.0 + dz, c.1 + dx), (c.0 - dz, c.1 - dx)];
                    let back_open = Floor::idx(back.0, back.1).is_some_and(|i| cells[i] != Cell::Rock);
                    let sides_closed = sides.iter().all(|s| Floor::idx(s.0, s.1).is_none_or(|i| cells[i] == Cell::Rock || kind[i] == 1 && !r.contains(*s)));
                    if back_open && sides_closed {
                        cells[idx(c.0, c.1)] = Cell::Stair(-dx, -dz);
                        stairs += 1;
                    }
                }
            }
        }
        if stairs == 0 {
            r.level = 0;
        }
    }
    for r in &rooms {
        for iz in r.z0..=r.z1 {
            for ix in r.x0..=r.x1 {
                cells[idx(ix, iz)] = Cell::Floor(r.level);
            }
        }
    }
    // Avsatser: der et hevet rom ligger inntil en lav gang uten trapp,
    // åpnes veggen over avsatsen noen steder.
    let mut ledges = HashSet::new();
    for r in &rooms {
        if r.level == 0 {
            continue;
        }
        for iz in r.z0..=r.z1 {
            for ix in r.x0..=r.x1 {
                for (dx, dz) in dirs {
                    let c = (ix + dx, iz + dz);
                    if !r.contains(c) && matches!(cells.get(idx(c.0, c.1).min(N * N - 1)), Some(Cell::Floor(0))) && Floor::idx(c.0, c.1).is_some() && rng.chance(0.3) {
                        ledges.insert(((ix, iz), c));
                    }
                }
            }
        }
    }
    // Takhøyder: rom høyere enn ganger.
    let ceil: Vec<f32> = kind.iter().map(|k| if *k == 1 { ROOM_H } else { WALL_H }).collect();
    let mut openings = HashSet::new();
    if !last {
        openings.insert((alcove, (dir, 0)));
    }
    // Startrommets vegg mot trappa fra etasjen over åpnes (trappa kommer
    // fra den sida utgangen vår ligger på).
    if k > 0 {
        openings.insert(((s.0 + dir, s.1), (dir, 0)));
    }
    let floor = Floor { cells, kind, ceil, origin, y, theme: k, ledges, openings, start: s, exit: Some(alcove), dir };
    (floor, rooms)
}

/// Gulv, tak, vegger, trapper, kollidere, lys og pynt for én etasje, og
/// trappesjakten ned til neste.
fn mesh(b: &mut Builder, f: &Floor, rng: &mut Rng, shaft: bool, chests: &mut Vec<(Vec3, f32, usize)>) {
    let th = theme(f.theme);
    let open = f.cells.iter().filter(|c| **c != Cell::Rock).count();
    log::debug!("Etasje {}: {open} åpne celler, hjørne ({:.0}, {:.0}), gulv y {:.0}, {} trekanter i bymeshen før", f.theme, f.origin.0, f.origin.1, f.y, b.mesh.indices.len() / 3);
    let (x0, z0) = f.origin;
    let n = N as i32;
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut lights: Vec<(Vec3, Vec3)> = vec![];
    let mut wall_count = 0;
    for iz in 0..n {
        for ix in 0..n {
            let cell = f.cell(ix, iz);
            if cell == Cell::Rock {
                continue;
            }
            let (cx, cz) = (x0 + ix as f32 * CELL, z0 + iz as f32 * CELL);
            let level = f.level(ix, iz);
            let fy = f.y + level as f32 * LEDGE;
            let ceil_y = f.y + f.ceil[iz as usize * N + ix as usize] + level as f32 * LEDGE * 0.5;
            let m = &mut b.mesh;
            // Gulv eller trapp, og tak.
            match cell {
                Cell::Stair(dx, dz) => {
                    // Åtte trinn som stiger mot (dx, dz).
                    let steps = 8;
                    for s in 0..steps {
                        let t0 = s as f32 / steps as f32;
                        let t1 = (s + 1) as f32 / steps as f32;
                        let y1 = f.y + t1 * LEDGE;
                        let y0 = f.y + t0 * LEDGE;
                        // Hjørnene langs trappas akse.
                        let at = |t: f32, side: f32| -> Vec3 {
                            let (ux, uz) = (dx as f32, dz as f32);
                            let (vx, vz) = (-uz, ux);
                            let base = Vec3::new(cx + CELL * 0.5, 0.0, cz + CELL * 0.5);
                            base + Vec3::new(ux, 0.0, uz) * (t - 0.5) * CELL + Vec3::new(vx, 0.0, vz) * side * CELL * 0.5
                        };
                        // Trinnflata (på y1) og opptrinnet (fra y0 til y1).
                        let a = at(t0, -1.0);
                        let bq = at(t0, 1.0);
                        let c = at(t1, 1.0);
                        let d = at(t1, -1.0);
                        m.quad_n([Vec3::new(a.x, y1, a.z), Vec3::new(bq.x, y1, bq.z), Vec3::new(c.x, y1, c.z), Vec3::new(d.x, y1, d.z)], [Vec3::Y; 4], &th.floor, 0.9);
                        let riser_n = Vec3::new(-dx as f32, 0.0, -dz as f32);
                        m.quad_n([Vec3::new(a.x, y0, a.z), Vec3::new(a.x, y1, a.z), Vec3::new(bq.x, y1, bq.z), Vec3::new(bq.x, y0, bq.z)], [riser_n; 4], &th.wall, 0.8);
                        // Flipp om normalen peker feil vei i forhold til vindingen.
                        let _ = riser_n;
                    }
                    // Kollideren: en kile.
                    let (ux, uz) = (dx as f32, dz as f32);
                    let (vx, vz) = (-uz, ux);
                    let base = Vec3::new(cx + CELL * 0.5, f.y, cz + CELL * 0.5);
                    let u = Vec3::new(ux, 0.0, uz) * CELL * 0.5;
                    let v = Vec3::new(vx, 0.0, vz) * CELL * 0.5;
                    let points = vec![
                        base - u - v - Vec3::Y * 0.5,
                        base - u + v - Vec3::Y * 0.5,
                        base + u - v - Vec3::Y * 0.5,
                        base + u + v - Vec3::Y * 0.5,
                        base - u - v,
                        base - u + v,
                        base + u - v + Vec3::Y * LEDGE,
                        base + u + v + Vec3::Y * LEDGE,
                    ];
                    b.colliders.push(Shape::Hull { points });
                }
                _ => {
                    m.quad_n([Vec3::new(cx, fy, cz + CELL), Vec3::new(cx + CELL, fy, cz + CELL), Vec3::new(cx + CELL, fy, cz), Vec3::new(cx, fy, cz)], [Vec3::Y; 4], &th.floor, 1.0);
                    b.colliders.push(Shape::Cuboid { center: Vec3::new(cx + CELL * 0.5, fy - 0.25, cz + CELL * 0.5), size: Vec3::new(CELL, 0.5, CELL), yaw: 0.0 });
                    // Gulvet registreres, så rutinen mot svevende modeller
                    // måler tønnene og kistene mot det, ikke mot terrenget.
                    b.floor(Vec3::new(cx + CELL * 0.5, fy, cz + CELL * 0.5), glam::Vec2::splat(CELL * 0.5), 0.0, fy);
                }
            }
            let m = &mut b.mesh;
            m.quad_n([Vec3::new(cx, ceil_y, cz), Vec3::new(cx + CELL, ceil_y, cz), Vec3::new(cx + CELL, ceil_y, cz + CELL), Vec3::new(cx, ceil_y, cz + CELL)], [-Vec3::Y; 4], &th.ceil, 1.0);
            // Veggene: mot fjell, mot celler på et annet nivå (hele veggen,
            // eller en lav vegg på den lave sida av en avsats), og mot
            // trappers sider.
            for (dx, dz) in dirs {
                let nb = (ix + dx, iz + dz);
                let other = f.cell(nb.0, nb.1);
                let opening = f.openings.contains(&((ix, iz), (dx, dz)));
                let quad = |y_lo: f32, y_hi: f32| -> [Vec3; 4] {
                    match (dx, dz) {
                        (-1, 0) => [Vec3::new(cx, y_lo, cz), Vec3::new(cx, y_lo, cz + CELL), Vec3::new(cx, y_hi, cz + CELL), Vec3::new(cx, y_hi, cz)],
                        (1, 0) => [Vec3::new(cx + CELL, y_lo, cz + CELL), Vec3::new(cx + CELL, y_lo, cz), Vec3::new(cx + CELL, y_hi, cz), Vec3::new(cx + CELL, y_hi, cz + CELL)],
                        (0, -1) => [Vec3::new(cx + CELL, y_lo, cz), Vec3::new(cx, y_lo, cz), Vec3::new(cx, y_hi, cz), Vec3::new(cx + CELL, y_hi, cz)],
                        _ => [Vec3::new(cx, y_lo, cz + CELL), Vec3::new(cx + CELL, y_lo, cz + CELL), Vec3::new(cx + CELL, y_hi, cz + CELL), Vec3::new(cx, y_hi, cz + CELL)],
                    }
                };
                let normal = Vec3::new(-dx as f32, 0.0, -dz as f32);
                let wall_center = |y_lo: f32, y_hi: f32| -> (Vec3, Vec3) {
                    let q = quad(y_lo, y_hi);
                    let center = (q[0] + q[2]) * 0.5 - normal * 0.1;
                    let size = if dx != 0 { Vec3::new(0.2, y_hi - y_lo, CELL) } else { Vec3::new(CELL, y_hi - y_lo, 0.2) };
                    (center, size)
                };
                let full = other == Cell::Rock && !opening || !f.passable((ix, iz), nb) && !f.ledges.contains(&((ix, iz), nb)) && !f.ledges.contains(&(nb, (ix, iz))) && !opening;
                // Trappas egne sider får vegg fra gulvet og opp.
                let (y_lo, y_hi) = (f.y + if matches!(cell, Cell::Stair(..)) { 0.0 } else { level as f32 * LEDGE }, ceil_y);
                if full {
                    let m = &mut b.mesh;
                    m.quad_n(quad(y_lo, y_hi), [normal; 4], &th.wall, 0.85);
                    let (center, size) = wall_center(y_lo, y_hi);
                    b.colliders.push(Shape::Cuboid { center, size, yaw: 0.0 });
                    wall_count += 1;
                    // Lys og pynt på veggene.
                    let mid = (quad(y_lo, y_hi)[0] + quad(y_lo, y_hi)[1]) * 0.5;
                    decorate_wall(b, &th, rng, mid, normal, y_lo, y_hi, &mut lights, wall_count);
                } else if f.ledges.contains(&(nb, (ix, iz))) {
                    // Vi er den lave sida av en avsats: lav vegg opp til den.
                    let hi = f.y + LEDGE;
                    let m = &mut b.mesh;
                    m.quad_n(quad(y_lo, hi), [normal; 4], &th.wall, 0.8);
                    let (center, size) = wall_center(y_lo, hi);
                    b.colliders.push(Shape::Cuboid { center, size, yaw: 0.0 });
                }
                // Over en åpning mot en celle med lavere tak (gang fra rom,
                // trapp fra hevet rom) står veggen fra nabocellens tak til vårt.
                if !full {
                    let other_level = f.level(nb.0, nb.1);
                    let other_ceil = Floor::idx(nb.0, nb.1).map(|i| f.y + f.ceil[i] + other_level as f32 * LEDGE * 0.5).unwrap_or(ceil_y);
                    if other_ceil < ceil_y - 0.05 {
                        let m = &mut b.mesh;
                        m.quad_n(quad(other_ceil, ceil_y), [normal; 4], &th.wall, 0.85);
                    }
                }
            }
            decorate_floor(b, &th, rng, f, ix, iz, fy, ceil_y, &mut lights, chests);
        }
    }
    for (p, c) in lights {
        let _ = (p, c);
    }
    if shaft {
        if let Some(exit) = f.exit {
            shaft_down(b, f, exit, &th);
        }
    }
}

/// Lys og pynt på en vegg: fakler, lykter, lysrør, mose, rør og kabler.
#[allow(clippy::too_many_arguments)]
fn decorate_wall(b: &mut Builder, th: &Theme, rng: &mut Rng, mid: Vec3, normal: Vec3, y_lo: f32, y_hi: f32, _lights: &mut Vec<(Vec3, Vec3)>, count: usize) {
    let yaw = normal.x.atan2(normal.z);
    let every = match th.light {
        LightKind::Torch => 3,
        LightKind::Lantern => 3,
        LightKind::Tube => 0,
        LightKind::Candle => 0,
        LightKind::Lava => 7,
    };
    if every > 0 && count % every == 0 {
        let p = Vec3::new(mid.x, y_lo + 1.8, mid.z) + normal * 0.02;
        match th.light {
            LightKind::Lantern if b.has_model("door_lamp") => {
                b.place_mounted("door_lamp", Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p));
                b.fire_light(p + normal * 0.3 + Vec3::Y * 0.1, Vec3::new(1.0, 0.75, 0.45), 2.4, 9.0);
            }
            _ => {
                if b.has_model("gorvek_torch") {
                    b.place_mounted("gorvek_torch", Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p));
                } else {
                    b.mesh.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_x(0.4) * Quat::from_rotation_y(yaw), p), 0.03, 0.03, 0.4, 6, &OAK, true);
                }
                let flicker = 0.9 + rng.next() * 0.3;
                b.fire_light(p + normal * 0.25 + Vec3::Y * 0.3, Vec3::new(1.0, 0.55, 0.22), 2.2 * flicker, 9.0);
            }
        }
    }
    // Mose nederst på veggen.
    if th.moss > 0.0 && rng.chance(th.moss) {
        let w = rng.range(1.2, 3.4);
        let h = rng.range(0.5, 1.4);
        let moss = Mat::tex(Layer::Grass, 0xffffff).tint(Vec3::new(0.16, 0.26, 0.09)).flags(flag::NO_WEATHER);
        let along = Vec3::new(normal.z, 0.0, -normal.x);
        let c = Vec3::new(mid.x, y_lo, mid.z) + normal * 0.015 + along * rng.range(-0.8, 0.8);
        let q = [c - along * w * 0.5, c + along * w * 0.5, c + along * w * 0.5 + Vec3::Y * h, c - along * w * 0.5 + Vec3::Y * h];
        b.mesh.quad_n(q, [normal; 4], &moss, 0.7);
        // Glødende mose i mursteinsetasjen: et grønt skjær.
        if th.light == LightKind::Torch && rng.chance(0.3) {
            b.flicker_light(c + normal * 0.3 + Vec3::Y * 0.4, Vec3::new(0.35, 0.9, 0.4), 0.7, 4.0, 0.05);
        }
    }
    // Rør langs taket i kloakken, kabler i serverrommet.
    if (th.pipes || th.cables) && rng.chance(0.5) {
        let along = Vec3::new(normal.z, 0.0, -normal.x);
        let r = if th.pipes { 0.09 } else { 0.025 };
        let mat = if th.pipes { IRON.tint(Vec3::new(0.35, 0.25, 0.18)) } else { Mat::plain(0x101012, 0.6) };
        let c = Vec3::new(mid.x, y_hi - 0.35 - rng.range(0.0, 0.4), mid.z) + normal * (r + 0.02) - along * CELL * 0.5;
        b.mesh.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, along), c), r, r, CELL, 8, &mat, false);
    }
}

/// Pynt og lys på gulvet i en celle: lysrør, stearinlys, lava, pytter,
/// gjørme, søyler, tønner, kister, hodeskaller.
#[allow(clippy::too_many_arguments)]
fn decorate_floor(b: &mut Builder, th: &Theme, rng: &mut Rng, f: &Floor, ix: i32, iz: i32, fy: f32, ceil_y: f32, _lights: &mut Vec<(Vec3, Vec3)>, chests: &mut Vec<(Vec3, f32, usize)>) {
    let cell = f.cell(ix, iz);
    if matches!(cell, Cell::Stair(..)) {
        return;
    }
    let (cx, cz) = (f.origin.0 + (ix as f32 + 0.5) * CELL, f.origin.1 + (iz as f32 + 0.5) * CELL);
    let center = Vec3::new(cx, fy, cz);
    let is_room = f.kind[iz as usize * N + ix as usize] == 1;
    let is_start = (ix - f.start.0).abs() <= 1 && (iz - f.start.1).abs() <= 1;
    let is_exit = f.exit == Some((ix, iz));
    // Hvilke sider som er vegg (til å stille ting inntil).
    let walls: Vec<Vec3> = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().filter(|(dx, dz)| !f.is_open(ix + dx, iz + dz)).map(|(dx, dz)| Vec3::new(*dx as f32, 0.0, *dz as f32)).collect();
    match th.light {
        LightKind::Tube if is_room || (ix + iz) % 2 == 0 => {
            // Lysrør i taket, som flakker.
            let tube = Mat::plain(0xe8f4ff, 0.3).flags(flag::GLOW_ALWAYS | flag::NO_WEATHER);
            // Sylinderen har foten i origo og går langs lokal Y, så røret
            // starter 0,7 m til sida og går 1,4 m.
            let t = Affine3A::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), Vec3::new(cx + 0.7, ceil_y - 0.08, cz));
            b.mesh.cylinder(&t, 0.03, 0.03, 1.4, 6, &tube, true);
            b.mesh.cuboid(&Affine3A::from_translation(Vec3::new(cx, ceil_y - 0.03, cz)), Vec3::new(1.5, 0.06, 0.14), &IRON.tint(Vec3::new(0.5, 0.5, 0.52)));
            let bad = rng.chance(0.35);
            b.flicker_light(Vec3::new(cx, ceil_y - 0.3, cz), Vec3::new(0.75, 0.9, 1.0), 2.0, 8.0, if bad { 0.7 } else { 0.08 });
        }
        LightKind::Candle if rng.chance(if is_room { 0.22 } else { 0.08 }) && !walls.is_empty() => {
            // En klynge stearinlys inntil veggen.
            let w = rng.pick(&walls);
            let base = center + w * 1.6;
            let wax = Mat::plain(0xe9dcc0, 0.6).flags(flag::NO_WEATHER);
            let flame = Mat::plain(0xffc66a, 0.3).flags(flag::GLOW_ALWAYS | flag::NO_WEATHER);
            let n = 3 + (rng.next() * 3.0) as i32;
            for i in 0..n {
                let a = i as f32 * 1.7;
                let p = base + Vec3::new(a.cos() * 0.25, 0.0, a.sin() * 0.25);
                let h = rng.range(0.06, 0.2);
                b.mesh.cylinder(&Affine3A::from_translation(p), 0.02, 0.018, h, 6, &wax, true);
                b.mesh.sphere(&Affine3A::from_translation(p + Vec3::Y * (h + 0.03)), Vec3::new(0.012, 0.025, 0.012), 4, 6, &flame);
            }
            b.fire_light(base + Vec3::Y * 0.4, Vec3::new(1.0, 0.72, 0.4), 1.4, 6.0);
        }
        LightKind::Lava if is_room && !is_start && !is_exit && rng.chance(0.14) => {
            // En lavapøl: glødende flate litt under gulvet, i et hull.
            let lava = Mat::plain(0xff5a16, 0.9).flags(flag::GLOW_ALWAYS | flag::NO_WEATHER);
            let r = rng.range(0.9, 1.7);
            let mut ring = vec![];
            for i in 0..10 {
                let a = i as f32 / 10.0 * std::f32::consts::TAU;
                let rr = r * (0.8 + 0.2 * ((a * 3.0).sin() * 0.5 + 0.5));
                ring.push(center + Vec3::new(a.cos() * rr, 0.012, a.sin() * rr));
            }
            for i in 0..10 {
                let (p, q) = (ring[i], ring[(i + 1) % 10]);
                b.mesh.quad_n([center + Vec3::Y * 0.012, p, q, center + Vec3::Y * 0.012], [Vec3::Y; 4], &lava, 1.0);
            }
            b.flicker_light(center + Vec3::Y * 0.5, Vec3::new(1.0, 0.45, 0.12), 1.8, 7.0, 0.25);
        }
        _ => {}
    }
    // Pytter: vann speiler lysene.
    if th.puddles > 0.0 && rng.chance(th.puddles * 0.5) && !is_start {
        let water = Mat::special(kind::WATER, 0x203040, 0.02).flags(flag::NO_WEATHER);
        let r = rng.range(0.5, 1.5);
        let c = center + Vec3::new(rng.range(-1.0, 1.0), 0.015, rng.range(-1.0, 1.0));
        let mut ring = vec![];
        for i in 0..8 {
            let a = i as f32 / 8.0 * std::f32::consts::TAU + rng.next() * 0.3;
            ring.push(c + Vec3::new(a.cos() * r * rng.range(0.6, 1.0), 0.0, a.sin() * r * rng.range(0.6, 1.0)));
        }
        for i in 0..8 {
            b.mesh.quad_n([c, ring[i], ring[(i + 1) % 8], c], [Vec3::Y; 4], &water, 1.0);
        }
    }
    // Gjørme: mørke flekker på gulvet.
    if th.mud > 0.0 && rng.chance(th.mud) {
        let mud = Mat::tex(Layer::Gravel, 0xffffff).tint(Vec3::new(0.14, 0.1, 0.07)).flags(flag::NO_WEATHER);
        let w = rng.range(1.0, 3.0);
        let h = rng.range(1.0, 3.0);
        let c = center + Vec3::new(rng.range(-0.6, 0.6), 0.008, rng.range(-0.6, 0.6));
        b.mesh.quad_n([c + Vec3::new(-w * 0.5, 0.0, h * 0.5), c + Vec3::new(w * 0.5, 0.0, h * 0.5), c + Vec3::new(w * 0.5, 0.0, -h * 0.5), c + Vec3::new(-w * 0.5, 0.0, -h * 0.5)], [Vec3::Y; 4], &mud, 0.9);
    }
    if is_start || is_exit {
        return;
    }
    // Søyler i rommene, i cellene som har vegg på to sider (hjørnene).
    if th.pillars && is_room && walls.len() >= 2 && rng.chance(0.6) {
        let p = center + Vec3::new(walls[0].x + walls[1].x, 0.0, walls[0].z + walls[1].z) * 1.3;
        let h = ceil_y - fy;
        b.mesh.cylinder(&Affine3A::from_translation(p), 0.38, 0.32, h, 10, &STONE.tint(Vec3::new(0.5, 0.48, 0.45)), false);
        b.mesh.cylinder(&Affine3A::from_translation(p), 0.5, 0.42, 0.3, 10, &STONE.tint(Vec3::new(0.45, 0.43, 0.4)), true);
        b.colliders.push(Shape::Cylinder { center: p + Vec3::Y * h * 0.5, radius: 0.42, half_height: h * 0.5 });
    }
    // Tønner inntil veggene, og kister med komponenter (fire per etasje,
    // i rom, bare der det er vegg å stå inntil).
    if th.kegs > 0.0 && !walls.is_empty() && rng.chance(th.kegs) && b.has_model("gorvek_keg") {
        let w = rng.pick(&walls);
        let p = center + w * 1.45 + Vec3::new(rng.range(-0.8, 0.8) * w.z.abs(), 0.0, rng.range(-0.8, 0.8) * w.x.abs());
        b.place("gorvek_keg", Affine3A::from_rotation_translation(Quat::from_rotation_y(rng.next() * 6.3), p));
        b.colliders.push(Shape::Cylinder { center: p + Vec3::Y * 0.45, radius: 0.35, half_height: 0.45 });
    }
    let chests_here = chests.iter().filter(|c| c.2 == f.theme).count();
    if is_room && !walls.is_empty() && chests_here < 4 && rng.chance(0.06) && b.has_model("gorvek_chest") {
        let w = rng.pick(&walls);
        let p = center + w * 1.4;
        let yaw = (-w.x).atan2(-w.z);
        b.place("gorvek_chest", Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p));
        b.colliders.push(Shape::Cuboid { center: p + Vec3::Y * 0.3, size: Vec3::new(0.9, 0.6, 0.6), yaw });
        chests.push((p, yaw, f.theme));
    }
    // Hodeskaller og knokler i hjørnene.
    if th.skulls > 0.0 && !walls.is_empty() && rng.chance(th.skulls) {
        let w = rng.pick(&walls);
        let p = center + w * 1.6 + Vec3::new(rng.range(-1.0, 1.0) * w.z.abs(), 0.0, rng.range(-1.0, 1.0) * w.x.abs());
        if b.has_model("gorvek_skull") {
            b.place("gorvek_skull", Affine3A::from_rotation_translation(Quat::from_rotation_y(rng.next() * 6.3), p));
        }
        let bone = Mat::plain(0xd8cdb4, 0.7).flags(flag::NO_WEATHER);
        for i in 0..3 {
            let a = rng.next() * 6.3;
            let q = p + Vec3::new((a + i as f32).cos() * 0.3, 0.02, (a + i as f32).sin() * 0.3);
            b.mesh.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_y(rng.next() * 6.3) * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), q), 0.02, 0.02, rng.range(0.2, 0.4), 5, &bone, true);
        }
    }
}

/// Trappesjakten fra alkoven ned til neste etasjes startrom: trinn,
/// vegger, tak og kilekollidere.
fn shaft_down(b: &mut Builder, f: &Floor, exit: (i32, i32), th: &Theme) {
    let a = f.center(exit.0, exit.1);
    let dir = f.dir as f32;
    let top = Vec3::new(a.x + dir * CELL * 0.5, f.y, a.z);
    // Lengden langs x har fortegn etter retningen.
    let len = dir * SHAFT as f32 * CELL;
    let drop = FLOOR_DROP;
    let steps = (len / 0.5) as i32;
    let half_w = CELL * 0.5;
    for s in 0..steps {
        let t0 = s as f32 / steps as f32;
        let t1 = (s + 1) as f32 / steps as f32;
        let (x0, x1) = (top.x + t0 * len, top.x + t1 * len);
        let (y0, y1) = (top.y - t0 * drop, top.y - t1 * drop);
        let m = &mut b.mesh;
        // Trinnet (flatt på y1 fra x0 til x1) og opptrinnet ved x0.
        m.quad_n([Vec3::new(x0, y1, a.z + half_w), Vec3::new(x1, y1, a.z + half_w), Vec3::new(x1, y1, a.z - half_w), Vec3::new(x0, y1, a.z - half_w)], [Vec3::Y; 4], &th.floor, 0.9);
        m.quad_n([Vec3::new(x0, y1, a.z - half_w), Vec3::new(x0, y1, a.z + half_w), Vec3::new(x0, y0, a.z + half_w), Vec3::new(x0, y0, a.z - half_w)], [Vec3::X * dir; 4], &th.wall, 0.8);
    }
    // Kollidere: kiler per celle.
    for c in 0..SHAFT {
        let t0 = c as f32 / SHAFT as f32;
        let t1 = (c + 1) as f32 / SHAFT as f32;
        let (x0, x1) = (top.x + t0 * len, top.x + t1 * len);
        let (y0, y1) = (top.y - t0 * drop, top.y - t1 * drop);
        let points = vec![
            Vec3::new(x0, y0, a.z - half_w),
            Vec3::new(x0, y0, a.z + half_w),
            Vec3::new(x1, y1, a.z - half_w),
            Vec3::new(x1, y1, a.z + half_w),
            Vec3::new(x0, y0 - 1.0, a.z - half_w),
            Vec3::new(x0, y0 - 1.0, a.z + half_w),
            Vec3::new(x1, y1 - 1.0, a.z - half_w),
            Vec3::new(x1, y1 - 1.0, a.z + half_w),
        ];
        b.colliders.push(Shape::Hull { points });
        // Vegger og tak langs hellingen.
        let h = WALL_H;
        let m = &mut b.mesh;
        for (zs, normal) in [(a.z - half_w, Vec3::Z), (a.z + half_w, -Vec3::Z)] {
            let q = if normal.z > 0.0 {
                [Vec3::new(x0, y0, zs), Vec3::new(x1, y1, zs), Vec3::new(x1, y1 + h, zs), Vec3::new(x0, y0 + h, zs)]
            } else {
                [Vec3::new(x1, y1, zs), Vec3::new(x0, y0, zs), Vec3::new(x0, y0 + h, zs), Vec3::new(x1, y1 + h, zs)]
            };
            m.quad_n(q, [normal; 4], &th.wall, 0.8);
            let center = Vec3::new((x0 + x1) * 0.5, (y0 + y1) * 0.5 + h * 0.5, zs - normal.z * 0.1);
            b.colliders.push(Shape::Cuboid { center, size: Vec3::new(CELL * 1.1, h + drop / SHAFT as f32 + 1.0, 0.2), yaw: 0.0 });
        }
        let m = &mut b.mesh;
        m.quad_n([Vec3::new(x0, y0 + h, a.z - half_w), Vec3::new(x1, y1 + h, a.z - half_w), Vec3::new(x1, y1 + h, a.z + half_w), Vec3::new(x0, y0 + h, a.z + half_w)], [-Vec3::Y; 4], &th.ceil, 0.9);
        // En fakkel halvveis.
        if c == SHAFT / 2 {
            let p = Vec3::new((x0 + x1) * 0.5, (y0 + y1) * 0.5 + 1.8, a.z - half_w + 0.02);
            if b.has_model("gorvek_torch") {
                b.place_mounted("gorvek_torch", Affine3A::from_rotation_translation(Quat::from_rotation_y(0.0), p));
            }
            b.fire_light(p + Vec3::Z * 0.25 + Vec3::Y * 0.3, Vec3::new(1.0, 0.55, 0.22), 2.0, 9.0);
        }
    }
    // Den siste cellen før startrommet: et flatt stykke i neste etasjes høyde.
    let x_end = top.x + len;
    let y_end = top.y - drop;
    let m = &mut b.mesh;
    let x_far = x_end + dir * CELL;
    m.quad_n([Vec3::new(x_end, y_end, a.z + half_w), Vec3::new(x_far, y_end, a.z + half_w), Vec3::new(x_far, y_end, a.z - half_w), Vec3::new(x_end, y_end, a.z - half_w)], [Vec3::Y; 4], &th.floor, 0.9);
    b.colliders.push(Shape::Cuboid { center: Vec3::new((x_end + x_far) * 0.5, y_end - 0.25, a.z), size: Vec3::new(CELL, 0.5, CELL), yaw: 0.0 });
    let m = &mut b.mesh;
    for (zs, normal) in [(a.z - half_w, Vec3::Z), (a.z + half_w, -Vec3::Z)] {
        let q = if normal.z > 0.0 {
            [Vec3::new(x_end, y_end, zs), Vec3::new(x_far, y_end, zs), Vec3::new(x_far, y_end + WALL_H, zs), Vec3::new(x_end, y_end + WALL_H, zs)]
        } else {
            [Vec3::new(x_far, y_end, zs), Vec3::new(x_end, y_end, zs), Vec3::new(x_end, y_end + WALL_H, zs), Vec3::new(x_far, y_end + WALL_H, zs)]
        };
        m.quad_n(q, [normal; 4], &th.wall, 0.8);
        b.colliders.push(Shape::Cuboid { center: Vec3::new((x_end + x_far) * 0.5, y_end + WALL_H * 0.5, zs - normal.z * 0.1), size: Vec3::new(CELL, WALL_H, 0.2), yaw: 0.0 });
    }
    let m = &mut b.mesh;
    m.quad_n([Vec3::new(x_end, y_end + WALL_H, a.z - half_w), Vec3::new(x_far, y_end + WALL_H, a.z - half_w), Vec3::new(x_far, y_end + WALL_H, a.z + half_w), Vec3::new(x_end, y_end + WALL_H, a.z + half_w)], [-Vec3::Y; 4], &th.ceil, 0.9);
}
