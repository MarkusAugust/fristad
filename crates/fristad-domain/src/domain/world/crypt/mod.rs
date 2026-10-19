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

pub mod compose;
pub mod doc;
pub mod kit;
pub mod level;
pub mod map;
pub mod nav;
mod query;
pub mod run;
mod parts;

pub use parts::*;

use super::{Builder, Shape, Use};
use glam::{Affine3A, Vec2, Vec3};
use level::{AmbushDef, Dir, LevelFile};
use nav::{Nav, Poly};

/// Hvor langt under byen hvert nivå ligger: origo i y = −40·n.
pub const LEVEL_DROP: f32 = 40.0;
/// Under denne høyden er du i krypten.
pub const TOP: f32 = -20.0;
/// Så dypt du kan vade (føttene under vannflaten). Dypere vann går du ikke
/// ut i, og faller du i det, drar du deg opp der du sist sto tørt: det er
/// vannet som holder deg ute, ikke en usynlig mur (eieren 18. oktober).
pub const WADE: f32 = 0.6;
/// Minste høyde fra gulvet til taket: spilleren er 1,8 m og hopper.
pub const HEADROOM: f32 = 2.9;
/// Krypten holder seg innenfor verdens kant (199 m).
const REACH: f32 = 185.0;

/// Portgitteret (posert av økta): så bredt og høyt, og hvor langt det
/// sveives opp. Bredere porter skaleres i x.
pub const PORTCULLIS: &str = "kit_gate";
pub const GATE_W: f32 = 2.8;
pub const GATE_H: f32 = 3.0;
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
/// - `kit_gate`: portgitteret (`PORTCULLIS`), `GATE_W` bredt, skaleres i x.
/// - `kit_shortcut` (karmen, midt i døråpningen) og `kit_shortcut_leaf`
///   (døra, origo i hengslet til venstre sett forfra, går mot +x).
/// - `kit_secret_wall`: muren som synker i gulvet (origo nede på midten).
/// - `kit_trapdoor`: lemmen, origo i hengslet på −z-kanten, dreies ned om x.
/// - `kit_ladder`: stigen, origo ved foten, skaleres i y til toppen.
/// - `kit_plate` (trykkplata), `kit_arrow` (pila, peker mot +z).
/// - `kit_lore`, `kit_bonus`, `kit_crack` (sprekk-hintet på muren),
///   `kit_up` (døra opp med tre trinn; døra står 0,9 m inn), `kit_down`
///   (trappa ned med brystvern; origo i åpningen der du går inn).
pub const KIT_REST: &str = "kit_rest";
/// Flammen i skåla på toppen av hvilestedet, vist når det er tent.
pub const KIT_REST_FLAME: &str = "kit_rest_flame";
pub const KIT_LEVER: &str = "kit_lever";
pub const KIT_LEVER_ARM: &str = "kit_lever_arm";
pub const KIT_SHORTCUT: &str = "kit_shortcut";
pub const KIT_SHORTCUT_LEAF: &str = "kit_shortcut_leaf";
pub const KIT_SECRET: &str = "kit_secret_wall";
/// Den falske veggen: hvor langt målet og kollideren står foran murflaten,
/// og hvor dypt de går (flaten er i flukt med muren rundt, z = 0).
pub const SECRET_FRONT: f32 = 0.05;
pub const SECRET_DEPTH: f32 = 0.45;
pub const KIT_TRAPDOOR: &str = "kit_trapdoor";
pub const KIT_LADDER: &str = "kit_ladder";
pub const KIT_LADDER_TOP: &str = "kit_ladder_top";
pub const KIT_PLATE: &str = "kit_plate";
pub const KIT_ARROW: &str = "kit_arrow";
pub const KIT_LORE: &str = "kit_lore";
pub const KIT_BONUS: &str = "kit_bonus";
pub const KIT_CRACK: &str = "kit_crack";
pub const KIT_UP: &str = "kit_up";
pub const KIT_DOWN: &str = "kit_down";
/// Broa over en avgrunn (`kit_bridge`, origo midt på dekket, lengden langs
/// z) og kjettingene den henger i (`kit_chain`, 12 m fra øyet på dekket og
/// opp gjennom hvelvet). Broa henger `Bridge.hang` over der den
/// ligger når den er nede, og senkes når bryteren åpnes.
pub const KIT_BRIDGE: &str = "kit_bridge";
pub const KIT_CHAIN: &str = "kit_chain";
/// Heisen (dekket, posert av økta) og heistårnet med vinsjen (står fast).
pub const KIT_LIFT: &str = "kit_lift";
pub const KIT_WINCH: &str = "kit_winch";
/// Dekket på heisen (`kit_lift`), og høyden på hengebjelkene i heistårnet
/// over gulvet oppe, der kjettingene ender (`kit_winch`).
pub const LIFT_W: f32 = 2.8;
pub const LIFT_HEAD: f32 = 3.45;
/// Vinsjen står så langt fra sjaktmidten (langs −z i heistårnets rom).
pub const LIFT_WINCH_Z: f32 = -2.6;
/// Vannflaten (1 × 1 m, origo midt på), skalert og senket av økta.
pub const KIT_WATER: &str = "kit_water";
/// Målene broa er modellert i (skaleres til merkets `size`).
pub const BRIDGE_W: f32 = 3.6;
pub const BRIDGE_L: f32 = 8.0;
pub const REST_FLAME: Vec3 = Vec3::new(0.0, 1.75, 0.0);
pub const LEVER_PIVOT: Vec3 = Vec3::new(0.0, 0.56, 0.0);
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
    /// Så langt under gulvet rommet når i lyset (sjakta til trappa ned).
    pub sink: f32,
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
    /// Bitene nivået er bygd av (`crypt_NN_aa` …), som nettleseren henter
    /// først når nivået trengs.
    pub pieces: Vec<String>,
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
    pub bridges: Vec<Bridge>,
    pub lifts: Vec<Lift>,
    pub waters: Vec<Water>,
    pub traps: Vec<Trap>,
    pub rests: Vec<Rest>,
    pub lores: Vec<Lore>,
    pub bonuses: Vec<Bonus>,
    pub spawns: Vec<Spawn>,
    pub hints: Vec<Hint>,
    pub pending: Vec<Pending>,
}

/// Bygger nivåene som finnes under krypten i byen (`anchor`: der du går
/// ned), fra nivåfilene adapterne har lest (`world::Data.levels`).
pub fn build(b: &mut Builder, anchor: Vec2, files: Vec<(u32, LevelFile)>) -> Crypt {
    let mut c = Crypt::default();
    for (k, (n, f)) in files.into_iter().enumerate() {
        build_level(b, &mut c, k, n, f, anchor);
    }
    c.open = vec![false; c.switches.len()];
    // Portene til bossarenaene står oppe til bossen stenger dem.
    for g in c.gates.iter().filter(|g| g.arena.is_some()) {
        c.open[g.switch] = true;
    }
    // Det du kan bruke.
    for (i, ch) in c.chests.iter().enumerate() {
        b.target(Use::Chest(i), super::at(ch.pos, ch.yaw), Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.48, 0.33, 0.33));
    }
    // Målene er bare å sikte på; det som stenger veien (stengene i
    // gitteret, dørbladet, den falske veggen) er modellens egne kollidere,
    // som økta flytter med målet (`Builder::target_solids`).
    for (i, g) in c.gates.iter().enumerate() {
        b.target(Use::Gate(i), super::at(g.pos, g.yaw), Vec3::new(0.0, GATE_H * 0.5, 0.0), Vec3::new(g.width * 0.5, GATE_H * 0.5, 0.12));
        let t = b.targets.len() - 1;
        if !b.target_solids(t, PORTCULLIS, Affine3A::from_scale(Vec3::new(g.width / GATE_W, 1.0, 1.0))) {
            b.target_box(t);
        }
    }
    for (i, s) in c.shortcuts.iter().enumerate() {
        b.target(Use::Shortcut(i), super::at(s.pos, s.yaw), Vec3::new(0.0, SHORTCUT_H * 0.5, 0.0), Vec3::new(s.width * 0.5, SHORTCUT_H * 0.5, 0.08));
        let t = b.targets.len() - 1;
        if !b.target_solids(t, KIT_SHORTCUT_LEAF, Affine3A::from_translation(Vec3::new(-s.width * 0.5, 0.0, 0.0))) {
            b.target_box(t);
        }
    }
    for (i, s) in c.secrets.iter().enumerate() {
        // Målet er den falske delen: fra 5 cm foran murflaten (så siktet
        // treffer) og inn i muren, og ikke noe ved siden av. Kollideren er
        // veggen selv, i flukt med muren.
        b.target(Use::Secret(i), super::at(s.pos, s.yaw), Vec3::new(0.0, s.height * 0.5, SECRET_FRONT - SECRET_DEPTH * 0.5), Vec3::new(s.width * 0.5, s.height * 0.5, SECRET_DEPTH * 0.5));
        let t = b.targets.len() - 1;
        if !b.target_solids(t, s.model, Affine3A::IDENTITY) {
            // Kollideren er bare den delen av målet som ligger bak
            // murflaten (de 5 cm foran ga en usynlig vegg på skrå).
            let half = Vec3::new(s.width * 0.5, s.height * 0.5, (SECRET_DEPTH - SECRET_FRONT) * 0.5);
            let shift = Vec3::new(0.0, 0.0, -SECRET_FRONT * 0.5);
            if b.target_parts.len() <= t { b.target_parts.resize(t + 1, vec![]); }
            b.target_parts[t] = vec![(half, Affine3A::from_translation(shift))];
        }
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
    for (i, l) in c.lifts.iter().enumerate() {
        // Styrestangen i hjørnet (+x, +z) på dekket (følger heisen; økta flytter kollideren), og vinsjen oppe.
        b.target(Use::Lift(i), super::at(l.pos, l.yaw), Vec3::new(l.size.x * 0.5 - 0.3, 0.75, l.size.y * 0.5 - 0.3), Vec3::new(0.16, 0.7, 0.16));
        b.target(Use::Winch(i), super::at(l.pos, l.yaw), Vec3::new(0.0, 0.7, LIFT_WINCH_Z), Vec3::new(0.9, 0.6, 0.4));
    }
    // Pulten og skrinet: målet dekker hele modellen (korset på taket også),
    // så siktet treffer det du ser fra alle kanter.
    let whole = |b: &Builder, name: &str, fallback: (Vec3, Vec3)| {
        let (lo, hi) = b.bounds(name).unwrap_or(fallback);
        ((lo + hi) * 0.5, ((hi - lo) * 0.5).max(Vec3::splat(0.1)))
    };
    let (lore_c, lore_h) = whole(b, KIT_LORE, (Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 1.0, 0.3)));
    for (i, l) in c.lores.iter().enumerate() {
        b.target(Use::Lore(i), super::at(l.pos, l.yaw), lore_c, lore_h);
    }
    let (bonus_c, bonus_h) = whole(b, KIT_BONUS, (Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 1.35, 0.3)));
    for (i, l) in c.bonuses.iter().enumerate() {
        b.target(Use::Bonus(i), super::at(l.pos, l.yaw), bonus_c, bonus_h);
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
/// de falske veggene, lemmene, pilene og flammene på hvilestedene.
pub fn reserve(c: &Crypt) -> Vec<(&'static str, usize)> {
    // Hver falsk vegg har sin egen modell (eller byggesettets).
    let mut secrets: Vec<(&'static str, usize)> = vec![];
    for s in &c.secrets {
        match secrets.iter_mut().find(|(m, _)| *m == s.model) {
            Some((_, n)) => *n += 1,
            None => secrets.push((s.model, 1)),
        }
    }
    let mut out = vec![
        (PORTCULLIS, c.gates.len()),
        (CHEST_LID, c.chests.len()),
        (KIT_SHORTCUT_LEAF, c.shortcuts.len()),
        (KIT_LEVER_ARM, c.levers.len()),
        (KIT_TRAPDOOR, c.trapdoors.iter().filter(|t| t.lid).count()),
        (KIT_ARROW, if c.traps.is_empty() { 0 } else { 16 }),
        (KIT_REST_FLAME, c.rests.len()),
        (KIT_BRIDGE, c.bridges.len()),
        (KIT_CHAIN, 4 * c.bridges.len() + 4 * c.lifts.len()),
        (KIT_LIFT, c.lifts.len()),
        (KIT_WATER, c.waters.len()),
    ];
    out.extend(secrets);
    out
}

/// Hvor mange biter på 1 m en stige på `h` meter stables av, og hvor høy
/// hver bit blir (nær 1 m, så trinnene står 30–37 cm fra hverandre).
pub fn ladder_pieces(h: f32) -> (usize, f32) {
    let n = h.round().max(1.0) as usize;
    (n, h / n as f32)
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
        rooms.push(Room { id: r.id.clone(), name: r.name.clone(), poly, y: r.y + origin.y, h: r.h, secret: r.secret, min, max, area, sink: 0.0 });
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
    // Bitene fra byggesettet (nivåer kompilert fra et nivådokument):
    // murverk med fundament, som bitene over.
    for inst in &f.instances {
        let name = crate::domain::world::layout::intern(&inst.model);
        b.place_mounted(name, compose::transform(origin + Vec3::from(inst.pos), inst.yaw, inst.mirror));
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
    for (i, l) in f.lights.iter().enumerate() {
        b.origin = Some(crate::domain::world::layout::Ref::CryptLight { level: n, i });
        let pos = w(l.pos);
        let color = Vec3::from(l.color);
        let intensity = l.intensity.unwrap_or(2.0);
        b.flicker_light(pos, color, intensity, l.range, if l.flicker { 0.2 } else { 0.0 });
    }
    b.origin = None;
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
    let mut level = Level { n, name: f.name.clone(), theme: f.theme.clone(), origin, rooms: vec![], nav, start: (origin, 0.0), up: None, down: None, min_y, max_y, links, overlaps, pieces: f.pieces.clone() };
    let mut bonus_n = 0;
    // Portene eliten bærer nøkkelen til.
    let keyed: Vec<usize> = f.marks.iter().filter(|m| m.kind == "elite").flat_map(|m| m.opens.clone()).collect::<Vec<_>>().iter().map(|g| intern(c, k, g)).collect();
    for (i, m) in f.marks.iter().enumerate() {
        b.origin = Some(crate::domain::world::layout::Ref::Crypt { level: n, i });
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
                if b.place_mounted(KIT_UP, pose) {
                    // Trinnene opp til døra som en rampe, og muren bak.
                    let p = |x: f32, y: f32, z: f32| pose.transform_point3(Vec3::new(x, y, z));
                    let mut ramp = vec![];
                    for x in [-1.1f32, 1.1] {
                        ramp.extend([p(x, 0.0, 0.0), p(x, 0.0, -0.95), p(x, 0.45, -0.95), p(x, 0.02, 0.0)]);
                    }
                    b.solid(Shape::Hull { points: ramp });
                    // Døra, karmen og trinnene etter formen (`solids.py`).
                    if !b.place_solids(KIT_UP, pose) {
                        for k in 0..3 {
                            let h = 0.16 * (k + 1) as f32;
                            b.solid_box(p(0.0, h * 0.5, -0.15 - 0.3 * k as f32), Vec3::new(2.2, h, 0.3), m.yaw);
                        }
                        b.solid_box(p(0.0, 1.9, -0.95), Vec3::new(3.2, 3.8, 0.3), m.yaw);
                    }
                }
            }
            "down" => {
                level.down = Some((pos, m.yaw));
                if b.place_mounted(KIT_DOWN, pose) {
                    // Brystvernet rundt trappa: åpent der du går inn.
                    let p = |x: f32, y: f32, z: f32| pose.transform_point3(Vec3::new(x, y, z));
                    // Brystvernet og portalmuren etter formen (`solids.py`).
                    if !b.place_solids(KIT_DOWN, pose) {
                        b.solid_box(p(-1.35, 0.5, -1.95), Vec3::new(0.4, 1.0, 4.3), m.yaw);
                        b.solid_box(p(1.35, 0.5, -1.95), Vec3::new(0.4, 1.0, 4.3), m.yaw);
                        b.solid_box(p(0.0, 0.5, -3.95), Vec3::new(3.1, 1.0, 0.4), m.yaw);
                        for x in [-1.58f32, 1.58] {
                            b.solid_box(p(x, 2.05, -0.25), Vec3::new(0.75, 4.1, 0.5), m.yaw);
                        }
                    }
                    // Trinnene ned som en rampe, og bunnen.
                    let mut ramp = vec![];
                    for x in [-1.2f32, 1.2] {
                        ramp.extend([p(x, 0.0, 0.0), p(x, -2.4, -3.2), p(x, -2.9, -3.2), p(x, -0.5, 0.0)]);
                    }
                    b.solid(Shape::Hull { points: ramp });
                    b.solid_box(p(0.0, -2.65, -3.5), Vec3::new(2.4, 0.5, 1.0), m.yaw);
                    b.solid_box(p(0.0, -0.25, -0.2), Vec3::new(2.4, 0.5, 0.4), m.yaw);
                }
            }
            "rest" => {
                if b.place(KIT_REST, pose) && !b.place_solids(KIT_REST, pose) {
                    // Alteret (uten trinnene rundt): kjernen av modellens mål.
                    let (lo, hi) = b.bounds(KIT_REST).unwrap_or((Vec3::new(-0.7, 0.0, -0.4), Vec3::new(0.7, 1.0, 0.4)));
                    let size = Vec3::new((hi.x - lo.x) * 0.62, 1.0, (hi.z - lo.z) * 0.5);
                    b.solid_box(pose.transform_point3(Vec3::new(0.0, 0.5, 0.0)), size, m.yaw);
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
                c.spawns.push(Spawn {
                    label,
                    pos,
                    yaw: m.yaw,
                    n: m.n.unwrap_or(1).max(1),
                    patrol: m.patrol.iter().map(|p| w(*p)).collect(),
                    ambush,
                    elite: m.kind == "elite",
                    boss: m.kind == "boss",
                    level: k,
                });
            }
            "chest" | "rare" => {
                let chest = first(b, &["kit_chest", CHEST]);
                if b.place(chest, pose) && !b.place_solids(chest, pose) {
                    b.solid_box(pos + Vec3::Y * 0.3, Vec3::new(0.9, 0.6, 0.6), m.yaw);
                }
                let loot = if m.kind == "rare" { m.label.as_ref().map(|l| l.to_lowercase()) } else { m.loot.clone() };
                c.chests.push(Chest { pos, yaw: m.yaw, level: k, loot });
            }
            "lore" => {
                if b.place(KIT_LORE, pose) && !b.place_solids(KIT_LORE, pose) {
                    b.solid_box(pos + Vec3::Y * 0.5, Vec3::new(0.5, 1.0, 0.4), m.yaw);
                }
                c.lores.push(Lore { pos, yaw: m.yaw, title: m.label.clone().unwrap_or_else(|| "En lapp".into()), text: m.text.clone().unwrap_or_default(), level: k });
            }
            "bonus" => {
                if b.place(KIT_BONUS, pose) && !b.place_solids(KIT_BONUS, pose) {
                    b.solid_box(pos + Vec3::Y * 0.55, Vec3::new(0.5, 1.1, 0.5), m.yaw);
                }
                let label = m.label.clone().unwrap_or_default().to_lowercase();
                let kind = if label.contains("uthold") || label.contains("stamina") { BonusKind::Stamina } else { BonusKind::Life };
                c.bonuses.push(Bonus { pos, yaw: m.yaw, kind, level: k, key: format!("{n}:{bonus_n}") });
                bonus_n += 1;
            }
            "lever" => {
                if b.place(KIT_LEVER, pose) && !b.place_solids(KIT_LEVER, pose) {
                    b.solid_box(pose.transform_point3(Vec3::new(0.0, 0.3, -0.1)), Vec3::new(0.4, 0.6, 0.2), m.yaw);
                }
                let opens = m.opens.iter().map(|o| intern(c, k, o)).collect();
                c.levers.push(Lever { pos, yaw: m.yaw, level: k, opens });
            }
            "gate" => {
                let s = id(c, "gate");
                let arena = m.arena.as_deref().and_then(room_ix);
                let exit = m.label.as_deref() == Some("exit");
                c.gates.push(Gate { pos, yaw: m.yaw, width: m.width.unwrap_or(GATE_W), level: k, switch: s, key: keyed.contains(&s), arena, exit });
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
                if b.place_mounted(KIT_SHORTCUT, pose) && !b.place_solids(KIT_SHORTCUT, pose) {
                    // Buen over døra og vangene (muren rundt er nivåets).
                    let w = m.width.unwrap_or(1.6);
                    b.solid_box(pose.transform_point3(Vec3::new(0.0, SHORTCUT_H + 0.2, 0.0)), Vec3::new(w + 0.6, 0.4, 0.5), m.yaw);
                    for sx in [-1.0f32, 1.0] {
                        b.solid_box(pose.transform_point3(Vec3::new(sx * (w * 0.5 + 0.15), SHORTCUT_H * 0.5, 0.0)), Vec3::new(0.3, SHORTCUT_H, 0.5), m.yaw);
                    }
                }
                c.shortcuts.push(Shortcut { pos, yaw: m.yaw, width: m.width.unwrap_or(1.6), level: k, switch: s, latch });
            }
            "secret_wall" => {
                let s = id(c, "secret");
                // Nivåets egen bit av muren (build_level.py), i flukt med muren
                // rundt og med samme tekstur, så den bare synes på sprekkene.
                let own = format!("crypt_{n:02}_secret_{}", m.id.as_deref().unwrap_or("secret"));
                // Ellers byggesettets falske vegg (nivåer fra et nivådokument,
                // `label`), og til sist reserven.
                let model: &'static str = if b.has_model(&own) {
                    Box::leak(own.into_boxed_str())
                } else {
                    match m.label.as_deref().filter(|l| b.has_model(l)) {
                        Some(l) => crate::domain::world::layout::intern(l),
                        None => KIT_SECRET,
                    }
                };
                // Nivåets egen hentes og slippes med nivået, som bitene.
                if b.has_model(&format!("crypt_{n:02}_secret_{}", m.id.as_deref().unwrap_or("secret"))) {
                    level.pieces.push(model.to_string());
                }
                c.secrets.push(Secret { pos, yaw: m.yaw, width: m.width.unwrap_or(2.0), height: m.height.unwrap_or(2.6), level: k, switch: s, model });
            }
            "trapdoor" => {
                let s = id(c, "trapdoor");
                let (sw, sd) = m.size.map(|s| s.wd()).unwrap_or((1.6, 1.6));
                let lid = !matches!(m.label.as_deref(), Some("shaft") | Some("sjakt") | Some("hole"));
                let bolted = matches!(m.label.as_deref(), Some("bolted") | Some("boltet"));
                c.trapdoors.push(Trapdoor { pos, yaw: m.yaw, size: Vec2::new(sw, sd), level: k, switch: s, to: m.to.as_deref().and_then(room_ix), lid, bolted });
            }
            "bridge" => {
                let s = id(c, "bridge");
                let (bw, bl) = m.size.map(|s| s.wd()).unwrap_or((BRIDGE_W, BRIDGE_L));
                let hang = m.height.unwrap_or(6.0);
                let top = m.top.map(w).map(|t| t.y).unwrap_or(pos.y + hang + 3.0);
                c.bridges.push(Bridge { pos, yaw: m.yaw, size: Vec2::new(bw, bl), hang, top, level: k, switch: s });
            }
            "ladder" => {
                let top = m.top.map(w).unwrap_or(pos + Vec3::Y * 3.0);
                // Bitene på 1 m stables og strekkes bare litt, så trinnene
                // står like tett som på en ekte stige (før ble én bit strukket
                // til hele høyden), og toppen med krokene henger i kanten.
                let h = (top.y - pos.y).max(0.5);
                let (n, s) = ladder_pieces(h);
                // Ingen går gjennom stigen: vangene og trinnene etter formen
                // (`solids.py`), ellers en tynn kollider langs vangene.
                let mut shaped = true;
                for i in 0..n {
                    let piece = pose * Affine3A::from_translation(Vec3::Y * (i as f32 * s)) * Affine3A::from_scale(Vec3::new(1.0, s, 1.0));
                    b.place_mounted(KIT_LADDER, piece);
                    shaped &= b.place_solids(KIT_LADDER, piece);
                }
                b.place_mounted(KIT_LADDER_TOP, pose * Affine3A::from_translation(Vec3::Y * h));
                if shaped {
                    b.place_solids(KIT_LADDER_TOP, pose * Affine3A::from_translation(Vec3::Y * h));
                } else {
                    b.solid_box(pose.transform_point3(Vec3::new(0.0, h * 0.5, 0.0)), Vec3::new(0.6, h, 0.08), m.yaw);
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
                // Hintet til en falsk vegg står på selve veggen, der du skal
                // slå (eieren 18. oktober); veggen har sin egen sprekk og tar
                // den med seg når den synker, så sprekken ved siden av, på
                // muren som aldri gir etter, settes ikke ut.
                let wall = m.for_id.as_deref().and_then(|id| f.marks.iter().find(|o| o.kind == "secret_wall" && o.id.as_deref() == Some(id))).and_then(|o| o.pos.map(w)).filter(|_| kind == HintKind::Crack);
                if kind == HintKind::Crack && wall.is_none() {
                    b.place_mounted(KIT_CRACK, pose);
                }
                c.hints.push(Hint { kind, pos: wall.unwrap_or(pos), level: k });
            }
            "prop" => {
                // Pynt fra byggesettet (sarkofagene): modellen i `label`, kolliderne står i nivåfila.
                let label = m.label.clone().unwrap_or_default();
                match PROPS.iter().find(|p| **p == label) {
                    // Lampene henger i kroken under stivene og fanene på muren, over gulvet.
                    Some(name) if MOUNTED_PROPS.contains(name) => {
                        b.place_mounted(name, pose);
                    }
                    // Pynten på gulvet har kolliderne etter formen (`solids.py`),
                    // i tillegg til dem i nivåfila (møblene fra byen i `<navn>_inside.json`).
                    Some(name) => {
                        if b.place(name, pose) && !b.place_solids(name, pose) {
                            super::town::carry(b, name, pose, false);
                        }
                    }
                    None => log::warn!("Kryptnivå {n}: ukjent pynt «{label}» (kjente: {PROPS:?})"),
                }
            }
            "water" => {
                let s = id(c, "water");
                let (sw, sd) = m.size.map(|s| s.wd()).unwrap_or((4.0, 4.0));
                let (high, low) = match m.levels.as_slice() {
                    [h, l, ..] => (*h + origin.y, *l + origin.y),
                    _ => (pos.y, pos.y - 3.0),
                };
                c.waters.push(Water { pos, yaw: m.yaw, size: Vec2::new(sw, sd), high, low, level: k, switch: s });
            }
            "lift" => {
                let s = id(c, "lift");
                let (lw, ld) = m.size.map(|s| s.wd()).unwrap_or((LIFT_W, LIFT_W));
                let drop = m.bottom.map(w).map(|b| pos.y - b.y).unwrap_or(10.0).max(0.5);
                // Heistårnet med vinsjen står over sjakta; stolpene får kollider.
                // (Ikke kolliderne etter formen: tauet henger ned i sjakta der heisen går.)
                b.place(KIT_WINCH, pose);
                for sx in [-1.0f32, 1.0] {
                    b.solid_box(pose.transform_point3(Vec3::new(sx * (lw * 0.5 + 0.45), 2.0, 0.0)), Vec3::new(0.36, 4.0, 0.36), m.yaw);
                }
                // Bommen og bindbjelken under den, midt over sjakta.
                b.solid_box(pose.transform_point3(Vec3::new(0.0, LIFT_HEAD + 0.5, 0.0)), Vec3::new(lw + 1.4, 1.0, 0.34), m.yaw);
                c.lifts.push(Lift { pos, yaw: m.yaw, size: Vec2::new(lw, ld), drop, level: k, switch: s });
            }
            other => log::warn!("Kryptnivå {n}: ukjent merke «{other}»"),
        }
    }
    b.origin = None;
    // Trekket ved en hemmelighet får lysene nær den til å blafre.
    for h in c.hints.iter().filter(|h| h.level == k && h.kind == HintKind::Draft) {
        for l in b.lights[light_start..].iter_mut() {
            if l.pos.distance(h.pos) < 4.0 {
                l.flicker = l.flicker.max(0.35);
            }
        }
    }
    // Trappa ned står i en sjakt under gulvet: rommet lyser den (ellers er
    // trinnene utenfor alle rom, og bare lykta du bærer lyser dem).
    if let Some((p, _)) = level.down {
        if let Some(r) = rooms.iter_mut().find(|r| r.contains(Vec2::new(p.x, p.z)) && (p.y - r.y).abs() < 0.5) {
            r.sink = 3.2;
        }
    }
    level.rooms = rooms;
    log::info!("Kryptnivå {n} ({}): {} rom, {} polygoner, origo ({:.0}, {:.0}, {:.0})", level.name, level.rooms.len(), level.nav.polys.len(), origin.x, origin.y, origin.z);
    c.levels.push(level);
}
