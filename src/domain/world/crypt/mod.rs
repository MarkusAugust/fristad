//! Krypten: ti faste, håndlagde nivåer under byen (se `docs/krypten-ny.md`
//! og kontrakten `docs/krypten/format.md`). Krypten er bare så dyp som
//! nivåene som finnes: `build.rs` tar med `crypt_NN.json` fra 1 og oppover,
//! det ekte nivået fra Blender (`assets/models/crypt/`) når det finnes, ellers
//! testnivået (`assets/crypt_test/`).
//!
//! Hvert nivå legges på sin plass under byen (origo i y = −40·n, flyttet
//! så startstedet står under krypten på kirkegården og nivået holder seg
//! innenfor verdens kant), med bitene (`crypt_NN_a` …), kolliderne, lysene og
//! modellene for merkene (byggesettet `kit_*`, kistene og portgitteret).
//!
//! Fiendene går på navigasjonsnettet (`nav.rs`: A* over polygonene og trakt).
//! Portene, snarveidørene og de falske veggene er brytere (`Crypt.open`) som
//! stenger polygonene sine til de åpnes. Rommene brukes til kartet, til lyset
//! (et lys lyser i sitt rom og i rom med åpen forbindelse, `light_mask`) og
//! til å finne rommet du står i.

pub mod level;
pub mod nav;

use super::{Builder, Shape, Use};
use glam::{Affine3A, Vec2, Vec3};
use level::{AmbushDef, Dir, LevelFile};
use nav::{Nav, Poly};

// `LEVEL_FILES`: nivåene som finnes (laget av build.rs).
include!(concat!(env!("OUT_DIR"), "/crypt_levels.rs"));

/// Hvor langt under byen hvert nivå ligger: origo i y = −40·n.
pub const LEVEL_DROP: f32 = 40.0;
/// Under denne høyden er du i krypten.
pub const TOP: f32 = -20.0;
/// Minste høyde fra gulvet til taket: spilleren er 1,8 m og hopper.
pub const HEADROOM: f32 = 2.9;
/// Krypten holder seg innenfor verdens kant (199 m).
const REACH: f32 = 185.0;

/// Portgitteret (posert av økta): så bredt og høyt, og hvor langt det
/// sveives opp. Bredere porter skaleres i x.
pub const PORTCULLIS: &str = "portcullis";
pub const GATE_W: f32 = 3.9;
pub const GATE_H: f32 = 2.9;
pub const GATE_LIFT: f32 = 2.7;
/// Kistene: kassa står fast, lokket (`CHEST_LID`, origo i hengslet) dreies
/// opp av økta. Hengslet sitter bak på overkanten av kassa.
pub const CHEST: &str = "chest";
pub const CHEST_LID: &str = "chest_lid";
pub const CHEST_HINGE: Vec3 = Vec3::new(0.0, 0.46, -0.28);

/// Byggesettet motoren setter ut etter merkene (`assets/models/crypt/`).
/// Fronten er +z (der du står når du bruker tingen), origo på gulvet.
/// - `kit_rest`: alteret med lysestaken; lyset står `REST_FLAME` over origo.
/// - `kit_lever` (sokkelen) og `kit_lever_arm` (origo i dreiepunktet
///   `LEVER_PIVOT`, dreies om x fra −0,6 rad oppe til +0,6 nede).
/// - `kit_shortcut` (karmen, midt i døråpningen) og `kit_shortcut_leaf`
///   (døra, origo i hengslet til venstre sett forfra, går mot +x).
/// - `kit_secret_wall`: muren som synker i gulvet (origo nede på midten).
/// - `kit_trapdoor`: lemmen, origo i hengslet på −z-kanten, dreies ned om x.
/// - `kit_ladder`: stigen, origo ved foten, skaleres i y til toppen.
/// - `kit_plate` (trykkplata), `kit_arrow` (pila, peker mot +z).
/// - `kit_lore`, `kit_bonus`, `kit_crack` (sprekk-hintet på muren),
///   `kit_up` (døra opp; `crypt_stair` til den finnes), `kit_down` (trappa ned).
pub const KIT_REST: &str = "kit_rest";
pub const KIT_LEVER: &str = "kit_lever";
pub const KIT_LEVER_ARM: &str = "kit_lever_arm";
pub const KIT_SHORTCUT: &str = "kit_shortcut";
pub const KIT_SHORTCUT_LEAF: &str = "kit_shortcut_leaf";
pub const KIT_SECRET: &str = "kit_secret_wall";
pub const KIT_TRAPDOOR: &str = "kit_trapdoor";
pub const KIT_LADDER: &str = "kit_ladder";
pub const KIT_PLATE: &str = "kit_plate";
pub const KIT_ARROW: &str = "kit_arrow";
pub const KIT_LORE: &str = "kit_lore";
pub const KIT_BONUS: &str = "kit_bonus";
pub const KIT_CRACK: &str = "kit_crack";
pub const KIT_UP: &str = "kit_up";
pub const KIT_DOWN: &str = "kit_down";
pub const REST_FLAME: Vec3 = Vec3::new(0.0, 1.25, 0.0);
pub const LEVER_PIVOT: Vec3 = Vec3::new(0.0, 0.9, 0.0);
pub const SHORTCUT_H: f32 = 2.3;

/// Et rom eller en gang, i verden.
#[derive(Clone, Debug)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub poly: Vec<Vec2>,
    /// Gulvet og taket over det (absolutt y og høyde).
    pub y: f32,
    pub h: f32,
    pub secret: bool,
    pub min: Vec2,
    pub max: Vec2,
    area: f32,
}

impl Room {
    /// Om punktet (x, z) er inne i omrisset.
    pub fn contains(&self, q: Vec2) -> bool {
        if q.x < self.min.x || q.y < self.min.y || q.x > self.max.x || q.y > self.max.y {
            return false;
        }
        let mut inside = false;
        let n = self.poly.len();
        for i in 0..n {
            let (a, b) = (self.poly[i], self.poly[(i + n - 1) % n]);
            if (a.y > q.y) != (b.y > q.y) && q.x < (b.x - a.x) * (q.y - a.y) / (b.y - a.y) + a.x {
                inside = !inside;
            }
        }
        inside
    }
}

#[derive(Clone, Debug)]
pub struct Level {
    /// Nummeret (1–10), navnet og temaet.
    pub n: u32,
    pub name: String,
    pub theme: String,
    /// Nivåets origo i verden.
    pub origin: Vec3,
    pub rooms: Vec<Room>,
    pub nav: Nav,
    /// Startstedet (føttene, retningen), døra opp og trappa ned.
    pub start: (Vec3, f32),
    pub up: Option<(Vec3, f32)>,
    pub down: Option<(Vec3, f32)>,
    /// Laveste gulv og høyeste tak.
    pub min_y: f32,
    pub max_y: f32,
    /// Forbindelsene mellom rommene: (rom, rom, bryteren som stenger den).
    pub links: Vec<(usize, usize, Option<usize>)>,
    /// Rom som overlapper (galleri over en sal): lyset går mellom dem.
    pub overlaps: Vec<(usize, usize)>,
}

/// Noe som åpnes: en port, en snarveidør, en falsk vegg, en fallem.
#[derive(Clone, Debug)]
pub struct Switch {
    pub level: usize,
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct Chest {
    pub pos: Vec3,
    pub yaw: f32,
    pub level: usize,
    /// En bestemt komponent (sjelden), ellers fra nivåets liste.
    pub loot: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Gate {
    pub pos: Vec3,
    pub yaw: f32,
    pub width: f32,
    pub level: usize,
    pub switch: usize,
    /// Åpnes med nøkkelen eliten bærer (ellers med en spak).
    pub key: bool,
}

#[derive(Clone, Debug)]
pub struct Lever {
    pub pos: Vec3,
    pub yaw: f32,
    pub level: usize,
    pub opens: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Shortcut {
    pub pos: Vec3,
    pub yaw: f32,
    pub width: f32,
    pub level: usize,
    pub switch: usize,
    /// Retningen (flatt) fra døra mot sida der slåen sitter.
    pub latch: Vec3,
}

#[derive(Clone, Debug)]
pub struct Secret {
    pub pos: Vec3,
    pub yaw: f32,
    pub width: f32,
    pub height: f32,
    pub level: usize,
    pub switch: usize,
}

#[derive(Clone, Debug)]
pub struct Trapdoor {
    pub pos: Vec3,
    pub yaw: f32,
    /// Bredde og dybde.
    pub size: Vec2,
    pub level: usize,
    pub switch: usize,
    /// Rommet du havner i.
    pub to: Option<usize>,
    /// En lem som faller opp når du står på den (ellers et åpent hull).
    pub lid: bool,
}

#[derive(Clone, Debug)]
pub struct Ladder {
    pub pos: Vec3,
    pub yaw: f32,
    pub top: Vec3,
    pub level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapKind {
    Arrows,
    Spikes,
    Rocks,
    Pendulum,
    Gas,
    Flood,
}

#[derive(Clone, Debug)]
pub struct Trap {
    pub kind: TrapKind,
    /// Trykkplata (midten på gulvet) og målene.
    pub pos: Vec3,
    pub size: Vec2,
    /// Retningen pilene flyr (flatt).
    pub dir: Vec3,
    pub level: usize,
}

#[derive(Clone, Debug)]
pub struct Rest {
    pub pos: Vec3,
    pub yaw: f32,
    pub level: usize,
}

#[derive(Clone, Debug)]
pub struct Lore {
    pub pos: Vec3,
    pub yaw: f32,
    pub title: String,
    pub text: String,
    pub level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusKind {
    Life,
    Stamina,
}

#[derive(Clone, Debug)]
pub struct Bonus {
    pub pos: Vec3,
    pub yaw: f32,
    pub kind: BonusKind,
    pub level: usize,
    /// Navnet i lagringen (nivå og nummer), så den bare tas én gang.
    pub key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ambush {
    /// Vekkes når du kommer så nær.
    Near(f32),
    /// Vekkes når du går inn i rommet.
    Room(usize),
}

/// Fiender som står et sted: slaget (`label`, se `mobs::Kind::from_label`),
/// hvor mange, ruta de går, og om de ligger i bakhold.
#[derive(Clone, Debug)]
pub struct Spawn {
    pub label: String,
    pub pos: Vec3,
    pub yaw: f32,
    pub n: u32,
    pub patrol: Vec<Vec3>,
    pub ambush: Option<Ambush>,
    pub elite: bool,
    pub boss: bool,
    pub level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintKind {
    Draft,
    Sound,
    Crack,
}

#[derive(Clone, Debug)]
pub struct Hint {
    pub kind: HintKind,
    pub pos: Vec3,
    pub level: usize,
}

/// Det som ennå ikke gjør noe (heiser, vannstand): tatt vare på, så det
/// er klart når mekanismen kommer.
#[derive(Clone, Debug)]
pub struct Pending {
    pub kind: String,
    pub id: Option<String>,
    pub pos: Vec3,
    pub level: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Crypt {
    pub levels: Vec<Level>,
    pub switches: Vec<Switch>,
    /// Hvilke brytere som er åpne nå (økta har sin egen kopi).
    pub open: Vec<bool>,
    pub chests: Vec<Chest>,
    pub gates: Vec<Gate>,
    pub levers: Vec<Lever>,
    pub shortcuts: Vec<Shortcut>,
    pub secrets: Vec<Secret>,
    pub trapdoors: Vec<Trapdoor>,
    pub ladders: Vec<Ladder>,
    pub traps: Vec<Trap>,
    pub rests: Vec<Rest>,
    pub lores: Vec<Lore>,
    pub bonuses: Vec<Bonus>,
    pub spawns: Vec<Spawn>,
    pub hints: Vec<Hint>,
    pub pending: Vec<Pending>,
}

impl Crypt {
    /// Føttene og retningen der du kommer ned fra krypten i byen.
    pub fn start(&self) -> Option<(Vec3, f32)> {
        self.levels.first().map(|l| l.start)
    }

    /// Nivået et punkt er i, etter høyden.
    pub fn level_at(&self, p: Vec3) -> Option<usize> {
        self.levels.iter().position(|l| p.y >= l.min_y - 8.0 && p.y <= l.max_y + 4.0)
    }

    /// Bryteren med denne id-en i nivået.
    pub fn switch(&self, level: usize, id: &str) -> Option<usize> {
        self.switches.iter().position(|s| s.level == level && s.id == id)
    }

    fn open_poly<'a>(&'a self) -> impl Fn(&Poly) -> bool + 'a {
        move |p: &Poly| p.gate.is_none_or(|g| self.open.get(g).copied().unwrap_or(true))
    }

    /// Polygonen under føttene (åpen eller ikke).
    fn locate(&self, p: Vec3) -> Option<(usize, u32)> {
        let l = self.level_at(p)?;
        Some((l, self.levels[l].nav.locate(p, 0.6)?))
    }

    /// Om et punkt står på nettet (et åpent gulv der fiendene kan gå),
    /// innen en meter over eller under gulvet.
    pub fn walkable(&self, p: Vec3) -> bool {
        let Some((l, i)) = self.locate(p) else { return false };
        let nav = &self.levels[l].nav;
        (self.open_poly())(&nav.polys[i as usize]) && (nav.height_in(i, Vec2::new(p.x, p.z)) - p.y).abs() < 1.0
    }

    /// Gulvhøyden under et punkt (absolutt y); punktets y utenfor nettet.
    pub fn height_at(&self, p: Vec3) -> f32 {
        match self.locate(p) {
            Some((l, i)) => self.levels[l].nav.height_in(i, Vec2::new(p.x, p.z)),
            None => p.y,
        }
    }

    /// Gulvet på nettet under et punkt (også bak stengte porter), om noe.
    pub fn floor_under(&self, p: Vec3) -> Option<f32> {
        let (l, i) = self.locate(p)?;
        Some(self.levels[l].nav.height_in(i, Vec2::new(p.x, p.z)))
    }

    /// Nærmeste punkt på det åpne nettet innen `reach` meter.
    pub fn snap(&self, p: Vec3, reach: f32) -> Option<Vec3> {
        let l = self.level_at(p)?;
        self.levels[l].nav.nearest(p, reach, &self.open_poly()).map(|(_, c)| c)
    }

    /// Veien fra `from` til `to` gjennom åpne polygoner: hjørnene etter
    /// startpunktet, det siste er målet.
    pub fn path(&self, from: Vec3, to: Vec3) -> Option<Vec<Vec3>> {
        let l = self.level_at(from)?;
        if self.level_at(to) != Some(l) {
            return None;
        }
        self.levels[l].nav.path(from, to, &self.open_poly())
    }

    /// Neste hjørne på veien.
    pub fn next_step(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        self.path(from, to)?.into_iter().find(|c| c.with_y(0.0).distance(from.with_y(0.0)) > 0.05)
    }

    /// Rommet et punkt står i (det minste som rommer det), i nivået.
    pub fn room_at(&self, p: Vec3) -> Option<(usize, usize)> {
        let l = self.level_at(p)?;
        let q = Vec2::new(p.x, p.z);
        self.levels[l]
            .rooms
            .iter()
            .enumerate()
            .filter(|(_, r)| p.y >= r.y - 1.0 && p.y <= r.y + r.h && r.contains(q))
            .min_by(|a, b| a.1.area.total_cmp(&b.1.area))
            .map(|(i, _)| (l, i))
    }

    /// Rommene et lys i rom `room` lyser i: sitt eget, de med åpen
    /// forbindelse (en dør, en åpen port) og de som overlapper (galleriet
    /// over salen). Én bit per rom i nivået.
    pub fn light_mask(&self, level: usize, room: usize) -> u64 {
        let Some(l) = self.levels.get(level) else { return u64::MAX };
        let bit = |r: usize| if r < 64 { 1u64 << r } else { 0 };
        let mut m = bit(room);
        for &(a, b, sw) in &l.links {
            let open = sw.is_none_or(|s| self.open.get(s).copied().unwrap_or(true));
            if open && (a == room || b == room) {
                m |= bit(a) | bit(b);
            }
        }
        for &(a, b) in &l.overlaps {
            if a == room || b == room {
                m |= bit(a) | bit(b);
            }
        }
        m
    }
}

/// Bygger nivåene som finnes under krypten i byen (`anchor`: der du går
/// ned).
pub fn build(b: &mut Builder, anchor: Vec2) -> Crypt {
    let mut c = Crypt::default();
    let mut files: Vec<(u32, LevelFile)> = vec![];
    for (n, json) in LEVEL_FILES {
        match serde_json::from_str::<LevelFile>(json) {
            Ok(f) => files.push((*n, f)),
            Err(e) => {
                log::warn!("Kryptnivå {n}: {e}");
                break;
            }
        }
    }
    for (k, (n, f)) in files.into_iter().enumerate() {
        build_level(b, &mut c, k, n, f, anchor);
    }
    c.open = vec![false; c.switches.len()];
    // Det du kan bruke.
    for (i, ch) in c.chests.iter().enumerate() {
        b.target(Use::Chest(i), super::at(ch.pos, ch.yaw), Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.48, 0.33, 0.33));
    }
    for (i, g) in c.gates.iter().enumerate() {
        b.target(Use::Gate(i), super::at(g.pos, g.yaw), Vec3::new(0.0, GATE_H * 0.5, 0.0), Vec3::new(g.width * 0.5, GATE_H * 0.5, 0.12));
    }
    for (i, s) in c.shortcuts.iter().enumerate() {
        b.target(Use::Shortcut(i), super::at(s.pos, s.yaw), Vec3::new(0.0, SHORTCUT_H * 0.5, 0.0), Vec3::new(s.width * 0.5, SHORTCUT_H * 0.5, 0.08));
    }
    for (i, s) in c.secrets.iter().enumerate() {
        b.target(Use::Secret(i), super::at(s.pos, s.yaw), Vec3::new(0.0, s.height * 0.5, 0.0), Vec3::new(s.width * 0.5, s.height * 0.5, 0.25));
    }
    for (i, l) in c.levers.iter().enumerate() {
        b.target(Use::Lever(i), super::at(l.pos, l.yaw), Vec3::new(0.0, 0.85, 0.05), Vec3::new(0.3, 0.45, 0.2));
    }
    for (i, r) in c.rests.iter().enumerate() {
        b.target(Use::Rest(i), super::at(r.pos, r.yaw), Vec3::new(0.0, 0.6, 0.0), Vec3::new(0.8, 0.6, 0.5));
    }
    for (i, l) in c.ladders.iter().enumerate() {
        b.target(Use::Ladder(i), super::at(l.pos, l.yaw), Vec3::new(0.0, 1.1, 0.0), Vec3::new(0.45, 1.0, 0.12));
    }
    for (i, l) in c.lores.iter().enumerate() {
        b.target(Use::Lore(i), super::at(l.pos, l.yaw), Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.3, 0.5, 0.3));
    }
    for (i, l) in c.bonuses.iter().enumerate() {
        b.target(Use::Bonus(i), super::at(l.pos, l.yaw), Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.3, 0.5, 0.3));
    }
    for (k, l) in c.levels.iter().enumerate() {
        if let Some((p, yaw)) = l.up {
            b.target(Use::Ascend(k), super::at(p, yaw), Vec3::new(0.0, 1.2, 0.0), Vec3::new(0.75, 1.2, 0.2));
        }
    }
    log::info!(
        "Krypten: {} nivåer, {} fiender, {} kister, {} porter, {} spaker, {} hvilesteder",
        c.levels.len(),
        c.spawns.iter().map(|s| s.n).sum::<u32>(),
        c.chests.len(),
        c.gates.len(),
        c.levers.len(),
        c.rests.len()
    );
    c
}

/// Plass til det økta poserer: portgitrene, kistelokkene, dørene, spakene,
/// de falske veggene, lemmene og pilene.
pub fn reserve(c: &Crypt) -> Vec<(&'static str, usize)> {
    vec![
        (PORTCULLIS, c.gates.len()),
        (CHEST_LID, c.chests.len()),
        (KIT_SHORTCUT_LEAF, c.shortcuts.len()),
        (KIT_LEVER_ARM, c.levers.len()),
        (KIT_SECRET, c.secrets.len()),
        (KIT_TRAPDOOR, c.trapdoors.iter().filter(|t| t.lid).count()),
        (KIT_ARROW, if c.traps.is_empty() { 0 } else { 16 }),
    ]
}

/// Den første modellen i lista som finnes (`missing` får den første).
fn first(b: &Builder, names: &[&'static str]) -> &'static str {
    names.iter().copied().find(|n| b.has_model(n)).unwrap_or(names[0])
}

fn intern(c: &mut Crypt, level: usize, id: &str) -> usize {
    match c.switch(level, id) {
        Some(i) => i,
        None => {
            c.switches.push(Switch { level, id: id.to_string() });
            c.switches.len() - 1
        }
    }
}

fn build_level(b: &mut Builder, c: &mut Crypt, k: usize, n: u32, f: LevelFile, anchor: Vec2) {
    let v3 = |a: [f32; 3]| Vec3::from(a);
    // Startstedet står under krypten i byen, og nivået holdes innenfor
    // verdens kant.
    let start = f.marks.iter().find(|m| m.kind == "start");
    let s = start.and_then(|m| m.pos).map(v3).unwrap_or(Vec3::ZERO);
    let mut origin = Vec3::new(anchor.x - s.x, -LEVEL_DROP * n as f32, anchor.y - s.z);
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for r in &f.rooms {
        for p in &r.poly {
            lo = lo.min(Vec2::from(*p));
            hi = hi.max(Vec2::from(*p));
        }
    }
    if lo.x <= hi.x {
        for (axis, (l, h)) in [(0, (lo.x, hi.x)), (1, (lo.y, hi.y))] {
            let o = if axis == 0 { &mut origin.x } else { &mut origin.z };
            if *o + h > REACH {
                *o -= *o + h - REACH;
            }
            if *o + l < -REACH {
                *o += -REACH - (*o + l);
            }
        }
    }
    let w = |p: [f32; 3]| v3(p) + origin;

    // Rommene.
    let mut rooms = vec![];
    for r in &f.rooms {
        let poly: Vec<Vec2> = r.poly.iter().map(|p| Vec2::new(p[0] + origin.x, p[1] + origin.z)).collect();
        let min = poly.iter().fold(Vec2::splat(f32::MAX), |m, p| m.min(*p));
        let max = poly.iter().fold(Vec2::splat(f32::MIN), |m, p| m.max(*p));
        let area = (0..poly.len()).map(|i| poly[i].perp_dot(poly[(i + 1) % poly.len()])).sum::<f32>().abs() * 0.5;
        rooms.push(Room { id: r.id.clone(), name: r.name.clone(), poly, y: r.y + origin.y, h: r.h, secret: r.secret, min, max, area });
    }
    let room_ix = |id: &str| rooms.iter().position(|r| r.id == id);

    // Navigasjonsnettet, med rom og brytere per polygon.
    let verts: Vec<Vec3> = f.nav.verts.iter().map(|p| w(*p)).collect();
    let poly_rooms: Vec<Option<usize>> = (0..f.nav.polys.len()).map(|i| f.nav.poly_room.get(i).cloned().flatten().and_then(|r| room_ix(&r))).collect();
    let poly_gates: Vec<Option<usize>> = (0..f.nav.polys.len()).map(|i| f.nav.poly_gate.get(i).cloned().flatten().map(|g| intern(c, k, &g))).collect();
    let nav = Nav::new(&verts, &f.nav.polys, &poly_rooms, &poly_gates);

    // Forbindelsene mellom rommene, og rom som overlapper.
    let mut links: Vec<(usize, usize, Option<usize>)> = vec![];
    for p in &nav.polys {
        for l in &p.links {
            let q = &nav.polys[l.to as usize];
            if let (Some(a), Some(bb)) = (p.room, q.room) {
                if a != bb {
                    let e = (a.min(bb), a.max(bb), p.gate.or(q.gate));
                    if !links.contains(&e) {
                        links.push(e);
                    }
                }
            }
        }
    }
    let mut overlaps = vec![];
    for i in 0..rooms.len() {
        for j in i + 1..rooms.len() {
            let (a, bb) = (&rooms[i], &rooms[j]);
            let xz = a.min.x < bb.max.x - 0.5 && bb.min.x < a.max.x - 0.5 && a.min.y < bb.max.y - 0.5 && bb.min.y < a.max.y - 0.5;
            let y = a.y < bb.y + bb.h - 0.2 && bb.y < a.y + a.h - 0.2;
            if xz && y {
                overlaps.push((i, j));
            }
        }
    }

    // Bitene og kolliderne. Bitene er murverk med fundament: de måles ikke
    // mot gulvet.
    for piece in &f.pieces {
        let name: &'static str = Box::leak(piece.clone().into_boxed_str());
        b.place_mounted(name, Affine3A::from_translation(origin));
    }
    for col in &f.colliders {
        if let Some(bx) = &col.cuboid {
            b.solid_box(w(bx.center), v3(bx.size), bx.yaw);
        }
        if let Some(h) = &col.hull {
            if h.len() >= 4 {
                b.solid(Shape::Hull { points: h.iter().map(|p| w(*p)).collect() });
            }
        }
    }
    // Gulvene til rutinen mot svevende modeller: rommene og de flate
    // polygonene i nettet (galleriene).
    for r in &rooms {
        b.floor(Vec3::new((r.min.x + r.max.x) * 0.5, r.y, (r.min.y + r.max.y) * 0.5), (r.max - r.min) * 0.5, 0.0, r.y);
    }
    for p in &nav.polys {
        let (ylo, yhi) = p.pts.iter().fold((f32::MAX, f32::MIN), |(a, z), v| (a.min(v.y), z.max(v.y)));
        if yhi - ylo < 0.05 {
            b.floor(Vec3::new((p.min.x + p.max.x) * 0.5, yhi, (p.min.y + p.max.y) * 0.5), (p.max - p.min) * 0.5 + Vec2::splat(0.6), 0.0, yhi);
        }
    }

    // Lysene, med rommet de står i (global indeks lagres i lyset).
    let light_start = b.lights.len();
    for l in &f.lights {
        let pos = w(l.pos);
        let color = Vec3::from(l.color);
        let intensity = l.intensity.unwrap_or(2.0);
        b.flicker_light(pos, color, intensity, l.range, if l.flicker { 0.2 } else { 0.0 });
    }
    let level_room = |p: Vec3, id: &Option<String>| -> Option<usize> {
        if let Some(r) = id.as_deref().and_then(room_ix) {
            return Some(r);
        }
        let q = Vec2::new(p.x, p.z);
        rooms.iter().enumerate().filter(|(_, r)| p.y >= r.y - 1.0 && p.y <= r.y + r.h + 0.5 && r.contains(q)).min_by(|a, bb| a.1.area.total_cmp(&bb.1.area)).map(|(i, _)| i)
    };
    for (i, l) in f.lights.iter().enumerate() {
        let room = level_room(w(l.pos), &l.room);
        if let Some(light) = b.lights.get_mut(light_start + i) {
            light.room = room.map(|r| (k as u16, r as u16));
        }
    }

    let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    for r in &rooms {
        min_y = min_y.min(r.y);
        max_y = max_y.max(r.y + r.h);
    }
    for v in &verts {
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y + HEADROOM);
    }
    if min_y > max_y {
        min_y = origin.y;
        max_y = origin.y + 4.0;
    }

    // Merkene.
    let mut level = Level { n, name: f.name.clone(), theme: f.theme.clone(), origin, rooms: vec![], nav, start: (origin, 0.0), up: None, down: None, min_y, max_y, links, overlaps };
    let mut bonus_n = 0;
    // Portene eliten bærer nøkkelen til.
    let keyed: Vec<usize> = f.marks.iter().filter(|m| m.kind == "elite").flat_map(|m| m.opens.clone()).collect::<Vec<_>>().iter().map(|g| intern(c, k, g)).collect();
    for m in &f.marks {
        let Some(pos) = m.pos.map(w) else {
            log::warn!("Kryptnivå {n}: merket «{}» har ingen pos", m.kind);
            continue;
        };
        let pose = super::at(pos, m.yaw);
        let id = |c: &mut Crypt, fallback: &str| intern(c, k, m.id.as_deref().unwrap_or(fallback));
        match m.kind.as_str() {
            "start" => level.start = (pos, m.yaw),
            "up" => {
                level.up = Some((pos, m.yaw));
                b.place_mounted(first(b, &[KIT_UP, "crypt_stair"]), pose);
            }
            "down" => {
                level.down = Some((pos, m.yaw));
                if b.has_model(KIT_DOWN) {
                    b.place_mounted(KIT_DOWN, pose);
                }
            }
            "rest" => {
                if b.place(KIT_REST, pose) {
                    b.solid_box(pose.transform_point3(Vec3::new(0.0, 0.5, 0.0)), Vec3::new(1.4, 1.0, 0.8), m.yaw);
                }
                c.rests.push(Rest { pos, yaw: m.yaw, level: k });
            }
            "mob" | "elite" | "boss" => {
                let ambush = m.ambush.as_ref().and_then(|a| match a {
                    AmbushDef::On(true) => Some(Ambush::Near(4.0)),
                    AmbushDef::On(false) => None,
                    AmbushDef::Radius(r) => Some(Ambush::Near(*r)),
                    AmbushDef::Room(id) => room_ix(id).map(Ambush::Room),
                });
                let label = m.label.clone().unwrap_or_else(|| if m.kind == "boss" { "rammeverket".into() } else { "button".into() });
                c.spawns.push(Spawn { label, pos, yaw: m.yaw, n: m.n.unwrap_or(1).max(1), patrol: m.patrol.iter().map(|p| w(*p)).collect(), ambush, elite: m.kind == "elite", boss: m.kind == "boss", level: k });
            }
            "chest" | "rare" => {
                if b.place(first(b, &["kit_chest", CHEST]), pose) {
                    b.solid_box(pos + Vec3::Y * 0.3, Vec3::new(0.9, 0.6, 0.6), m.yaw);
                }
                let loot = if m.kind == "rare" { m.label.as_ref().map(|l| l.to_lowercase()) } else { m.loot.clone() };
                c.chests.push(Chest { pos, yaw: m.yaw, level: k, loot });
            }
            "lore" => {
                b.place(KIT_LORE, pose);
                c.lores.push(Lore { pos, yaw: m.yaw, title: m.label.clone().unwrap_or_else(|| "En lapp".into()), text: m.text.clone().unwrap_or_default(), level: k });
            }
            "bonus" => {
                b.place(KIT_BONUS, pose);
                let label = m.label.clone().unwrap_or_default().to_lowercase();
                let kind = if label.contains("uthold") || label.contains("stamina") { BonusKind::Stamina } else { BonusKind::Life };
                c.bonuses.push(Bonus { pos, yaw: m.yaw, kind, level: k, key: format!("{n}:{bonus_n}") });
                bonus_n += 1;
            }
            "lever" => {
                if b.place(KIT_LEVER, pose) {
                    b.solid_box(pose.transform_point3(Vec3::new(0.0, 0.3, -0.1)), Vec3::new(0.4, 0.6, 0.2), m.yaw);
                }
                let opens = m.opens.iter().map(|o| intern(c, k, o)).collect();
                c.levers.push(Lever { pos, yaw: m.yaw, level: k, opens });
            }
            "gate" => {
                let s = id(c, "gate");
                c.gates.push(Gate { pos, yaw: m.yaw, width: m.width.unwrap_or(GATE_W), level: k, switch: s, key: keyed.contains(&s) });
            }
            "shortcut" => {
                let s = id(c, "shortcut");
                // Slåen sitter på sida der `from`-rommet er.
                let front = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
                let latch = match m.from.as_deref().and_then(room_ix) {
                    Some(r) => {
                        let rc = (rooms[r].min + rooms[r].max) * 0.5;
                        let to = Vec3::new(rc.x - pos.x, 0.0, rc.y - pos.z);
                        if to.dot(front) >= 0.0 {
                            front
                        } else {
                            -front
                        }
                    }
                    None => front,
                };
                b.place_mounted(KIT_SHORTCUT, pose);
                c.shortcuts.push(Shortcut { pos, yaw: m.yaw, width: m.width.unwrap_or(1.6), level: k, switch: s, latch });
            }
            "secret_wall" => {
                let s = id(c, "secret");
                c.secrets.push(Secret { pos, yaw: m.yaw, width: m.width.unwrap_or(2.0), height: m.height.unwrap_or(2.6), level: k, switch: s });
            }
            "trapdoor" => {
                let s = id(c, "trapdoor");
                let (sw, sd) = m.size.map(|s| s.wd()).unwrap_or((1.6, 1.6));
                let lid = !matches!(m.label.as_deref(), Some("shaft") | Some("sjakt") | Some("hole"));
                c.trapdoors.push(Trapdoor { pos, yaw: m.yaw, size: Vec2::new(sw, sd), level: k, switch: s, to: m.to.as_deref().and_then(room_ix), lid });
            }
            "ladder" => {
                let top = m.top.map(w).unwrap_or(pos + Vec3::Y * 3.0);
                if let Some((lo, hi)) = b.bounds(KIT_LADDER) {
                    let s = ((top.y - pos.y) / (hi.y - lo.y).max(0.1)).max(0.2);
                    b.place_mounted(KIT_LADDER, pose * Affine3A::from_scale(Vec3::new(1.0, s, 1.0)));
                } else {
                    b.place_mounted(KIT_LADDER, pose);
                }
                c.ladders.push(Ladder { pos, yaw: m.yaw, top, level: k });
            }
            "trap" => {
                let kind = match m.label.as_deref().unwrap_or("arrows") {
                    "spikes" => TrapKind::Spikes,
                    "rocks" => TrapKind::Rocks,
                    "pendulum" => TrapKind::Pendulum,
                    "gas" => TrapKind::Gas,
                    "flood" => TrapKind::Flood,
                    _ => TrapKind::Arrows,
                };
                let (sw, sd) = m.size.map(|s| s.wd()).unwrap_or((1.2, 1.2));
                let dir = match m.dir {
                    Some(Dir::Yaw(y)) => Vec3::new(y.sin(), 0.0, y.cos()),
                    Some(Dir::Vec2([x, z])) => Vec3::new(x, 0.0, z),
                    Some(Dir::Vec3([x, _, z])) => Vec3::new(x, 0.0, z),
                    None => Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos()),
                }
                .normalize_or(Vec3::X);
                b.place(KIT_PLATE, pose);
                c.traps.push(Trap { kind, pos, size: Vec2::new(sw, sd), dir, level: k });
            }
            "hint" => {
                let kind = match m.label.as_deref().unwrap_or("crack") {
                    "draft" => HintKind::Draft,
                    "sound" => HintKind::Sound,
                    _ => HintKind::Crack,
                };
                if kind == HintKind::Crack {
                    b.place_mounted(KIT_CRACK, pose);
                }
                c.hints.push(Hint { kind, pos, level: k });
            }
            "lift" | "water" => {
                log::info!("Kryptnivå {n}: {} ({:?}) gjør ingenting ennå", m.kind, m.id);
                c.pending.push(Pending { kind: m.kind.clone(), id: m.id.clone(), pos, level: k });
            }
            other => log::warn!("Kryptnivå {n}: ukjent merke «{other}»"),
        }
    }
    // Trekket ved en hemmelighet får lysene nær den til å blafre.
    for h in c.hints.iter().filter(|h| h.level == k && h.kind == HintKind::Draft) {
        for l in b.lights[light_start..].iter_mut() {
            if l.pos.distance(h.pos) < 4.0 {
                l.flicker = l.flicker.max(0.35);
            }
        }
    }
    level.rooms = rooms;
    log::info!("Kryptnivå {n} ({}): {} rom, {} polygoner, origo ({:.0}, {:.0}, {:.0})", level.name, level.rooms.len(), level.nav.polys.len(), origin.x, origin.y, origin.z);
    c.levels.push(level);
}
