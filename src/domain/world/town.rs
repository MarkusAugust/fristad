//! Kjøpstaden Fristad rundt 1350, lagt ut slik middelalderbyer vokste:
//! Storgata slynger seg fra sørporten gjennom et skjevt torg til
//! nordporten, Tverrgata bukter seg ut fra torget mot øst og vest, og
//! smale smau fører til kirka og ned mot muren. Husene står i tette rekker
//! langs gatene med gavlen eller langsida mot gata, hver litt skjevt, med
//! ulik avstand til gata og et smau her og der. Bymuren følger et ujevnt
//! omriss (`terrain::wall_radius`) med tårn og porthus. Nord for byen står
//! borgruinen på haugen, og under den ligger krypten.
//!
//! Alt som synes er modeller; byen bestemmer bare hvor de står. Husmodellene
//! har døra mot sin lokale +Z og origo midt på grunnflaten i bakkehøyde, så
//! `yaw` vender døra mot gata.

use super::{at, terrain, Builder, NpcSpec, Shape, Use};
use crate::domain::rng::Rng;
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;

/// Torget: et skjevt firkantet åpent rom (x, z), hjørnene i rekkefølge
/// nordvest, sørvest, sørøst, nordøst.
pub const SQUARE: [Vec2; 4] = [Vec2::new(-13.0, -11.0), Vec2::new(-11.0, 12.5), Vec2::new(16.5, 8.5), Vec2::new(12.0, -13.5)];
/// Halv bredde på Storgata, Tverrgata og smauene.
pub const MAIN: f32 = 3.2;
pub const CROSS: f32 = 2.6;
pub const LANE: f32 = 1.5;
/// Krypten i borgruinen: midten av inngangen.
pub const CRYPT: Vec2 = Vec2::new(0.0, -138.0);
/// Åkrene utenfor muren (midte, radier).
pub const FIELDS: [(f32, f32, f32, f32); 3] = [(-95.0, 40.0, 30.0, 20.0), (90.0, 30.0, 26.0, 34.0), (70.0, -70.0, 24.0, 18.0)];

/// Murverk med fundament under bakken (fot 80 cm ned), som får stå nede i
/// skråninger.
pub const FOUNDED: [&str; 10] = ["town_wall", "wall_tower", "gatehouse", "castle_ruin", "crypt", "stone_wall", "boulder_a", "boulder_b", "boulder_c", "signpost"];

/// Det byen forteller resten av verden.
pub struct Town {
    pub spawn: Vec3,
    pub spawn_yaw: f32,
    /// Der kjelleren ligger under (x, z).
    pub crypt_anchor: Vec2,
    /// Der du står når du kommer opp fra kjelleren.
    pub crypt_exit: (Vec3, f32),
}

// --- Gatene ---------------------------------------------------------------

/// En gate: midtlinja som punkter (x, z), halv bredde, og om den er
/// brolagt (Storgata) eller tråkket jord.
pub struct Street {
    pub pts: Vec<Vec2>,
    pub half: f32,
    pub cobbled: bool,
}

/// Storgata: midtlinja som x for en z, en svak S fra sør til nord.
pub fn main_x(z: f32) -> f32 {
    6.0 * (z * 0.042 + 0.5).sin() + 1.6 * (z * 0.12 + 2.0).sin()
}

/// Gatene i byen, regnet ut én gang.
pub fn streets() -> &'static [Street] {
    static STREETS: OnceLock<Vec<Street>> = OnceLock::new();
    STREETS.get_or_init(|| {
        let curve = |a: f32, b: f32, n: usize, f: &dyn Fn(f32) -> Vec2| (0..=n).map(|i| f(a + (b - a) * i as f32 / n as f32)).collect::<Vec<_>>();
        vec![
            // Storgata, fra utenfor sørporten til utenfor nordporten.
            Street { pts: curve(68.0, -66.0, 67, &|z| Vec2::new(main_x(z), z)), half: MAIN, cobbled: true },
            // Tverrgata vestover fra torget, buktende og litt nedover mot sør.
            Street { pts: curve(-11.0, -62.0, 26, &|x| Vec2::new(x, 3.0 + 3.5 * (x * 0.06 + 1.0).sin() - (x + 11.0) * 0.08)), half: CROSS, cobbled: false },
            // Tverrgata østover, stigende mot nord.
            Street { pts: curve(13.0, 62.0, 25, &|x| Vec2::new(x, -3.0 + 4.0 * (x * 0.07 + 2.5).sin() - (x - 13.0) * 0.1)), half: CROSS, cobbled: false },
            // Smauet fra torget sørøstover ned mot muren.
            Street { pts: curve(0.0, 1.0, 12, &|t| Vec2::new(15.0 + 22.0 * t, 9.5 + 26.0 * t - 6.0 * (t * PI).sin())), half: LANE, cobbled: false },
            // Kirkesmauet fra Storgata vestover til kirkegården.
            Street { pts: curve(0.0, 1.0, 10, &|t| Vec2::new(main_x(-27.0) * (1.0 - t) - 19.0 * t, -27.0 + 4.0 * (t * PI).sin() + 3.0 * t)), half: LANE, cobbled: false },
            // Smauet nordvest fra Tverrgata langs innsida av muren.
            Street { pts: curve(0.0, 1.0, 12, &|t| Vec2::new(-28.0 - 8.0 * t, 1.5 + 26.0 * t + 4.0 * (t * PI).sin())), half: LANE, cobbled: false },
        ]
    })
}

/// Avstand fra (x, z) til midtlinja av gata, og retningen gata går der.
pub fn street_dist(st: &Street, p: Vec2) -> (f32, Vec2) {
    let mut best = (f32::MAX, Vec2::Y);
    for w in st.pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
        let d = (a + ab * t).distance(p);
        if d < best.0 {
            best = (d, ab.normalize_or_zero());
        }
    }
    best
}

/// Som `street_dist`, med koordinatene langs gata: (avstand, fortegnet
/// avstand på tvers, meter langs midtlinja fra starten).
pub fn street_coords(st: &Street, p: Vec2) -> (f32, f32, f32) {
    let mut best = (f32::MAX, 0.0, 0.0);
    let mut run = 0.0;
    for w in st.pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b - a;
        let len = ab.length().max(1e-6);
        let t = ((p - a).dot(ab) / (len * len)).clamp(0.0, 1.0);
        let d = (a + ab * t).distance(p);
        if d < best.0 {
            let across = (p - a).perp_dot(ab) / len;
            best = (d, across, run + (p - a).dot(ab) / len);
        }
        run += len;
    }
    best
}

/// Fortegnet avstand fra (x, z) til kanten av torget (negativ innenfor).
pub fn square_dist(p: Vec2) -> f32 {
    let mut d = f32::MIN;
    for i in 0..4 {
        let (a, b) = (SQUARE[i], SQUARE[(i + 1) % 4]);
        let e = (b - a).normalize();
        // Hjørnene går med klokka i (x, z), så utsida er til venstre.
        let out = Vec2::new(-e.y, e.x);
        d = d.max((p - a).dot(out));
    }
    d
}

/// Utover fra torget langs kant `i`.
fn square_out(i: usize) -> Vec2 {
    let e = (SQUARE[(i + 1) % 4] - SQUARE[i]).normalize();
    Vec2::new(-e.y, e.x)
}

/// Der Storgata møter muren (sør og nord), og retningen ut av byen der.
pub fn gate(south: bool) -> (Vec2, Vec2) {
    let (mut a, mut b) = if south { (0.0, 75.0) } else { (0.0, -75.0) };
    for _ in 0..40 {
        let m = (a + b) * 0.5;
        if terrain::town_edge(main_x(m), m) < 0.0 {
            a = m;
        } else {
            b = m;
        }
    }
    let p = Vec2::new(main_x(a), a);
    (p, p.normalize())
}

// --- Tomtene --------------------------------------------------------------

/// Et rektangel på bakken: midte, halve mål langs husets x og z, og yaw.
#[derive(Clone, Copy)]
struct Rect {
    c: Vec2,
    h: Vec2,
    yaw: f32,
}

impl Rect {
    fn axes(&self) -> (Vec2, Vec2) {
        let (s, c) = self.yaw.sin_cos();
        (Vec2::new(c, -s), Vec2::new(s, c))
    }
    fn corners(&self) -> [Vec2; 4] {
        let (ax, az) = self.axes();
        let (x, z) = (ax * self.h.x, az * self.h.y);
        [self.c - x - z, self.c + x - z, self.c + x + z, self.c - x + z]
    }
    /// Punktene som sjekkes mot gater og mur: hjørnene, kantmidtene og midten.
    fn samples(&self) -> [Vec2; 9] {
        let k = self.corners();
        [k[0], k[1], k[2], k[3], (k[0] + k[1]) * 0.5, (k[1] + k[2]) * 0.5, (k[2] + k[3]) * 0.5, (k[3] + k[0]) * 0.5, self.c]
    }
    fn overlaps(&self, o: &Rect) -> bool {
        let (a1, a2) = self.axes();
        let (b1, b2) = o.axes();
        let (ka, kb) = (self.corners(), o.corners());
        for axis in [a1, a2, b1, b2] {
            let (amin, amax) = ka.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p.dot(axis)), hi.max(p.dot(axis))));
            let (bmin, bmax) = kb.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p.dot(axis)), hi.max(p.dot(axis))));
            if amax <= bmin || bmax <= amin {
                return false;
            }
        }
        true
    }
}

/// Yaw som vender husets lokale +Z (døra) i retningen `dir` (x, z).
fn facing(dir: Vec2) -> f32 {
    dir.x.atan2(dir.y)
}

/// Det som er satt av: husene som står, og områder som skal være fri.
struct Plan {
    taken: Vec<Rect>,
    free: Vec<(Vec2, f32)>,
}

impl Plan {
    /// Får rektangelet plass: innenfor muren, utenfor gatene og torget, og
    /// ikke oppå noe annet.
    fn fits(&self, r: &Rect) -> bool {
        for p in r.samples() {
            if terrain::town_edge(p.x, p.y) > -3.0 || square_dist(p) < 0.4 {
                return false;
            }
            if streets().iter().any(|st| street_dist(st, p).0 < st.half + 0.25) {
                return false;
            }
            if self.free.iter().any(|(c, rad)| c.distance(p) < *rad) {
                return false;
            }
        }
        !self.taken.iter().any(|t| t.overlaps(r))
    }
}

/// Bygningene på torget: modell, kant av torget (0 vest, 1 sør, 2 øst,
/// 3 nord, mot klokka fra nordvest), hvor langs kanten (0..1), og grunnflaten
/// som settes av om modellen mangler.
const SPECIALS: [(&str, usize, f32, Vec2); 4] = [
    ("apothecary", 0, 0.22, Vec2::new(8.0, 7.0)),
    ("tavern", 1, 0.28, Vec2::new(12.0, 9.0)),
    ("forge", 2, 0.22, Vec2::new(9.0, 8.0)),
    ("scriptorium", 3, 0.18, Vec2::new(10.0, 8.0)),
];

/// Husmodellene som står i rekkene.
const HOUSES: [&str; 6] = ["house_a", "house_b", "house_c", "house_d", "house_e", "house_f"];

/// Husets mål: midten av grunnflaten (lokal x, z) og halve mål.
fn footprint(b: &Builder, model: &str, fallback: Vec2) -> (Vec2, Vec2, f32) {
    match b.bounds(model) {
        Some((lo, hi)) => (Vec2::new((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5), Vec2::new((hi.x - lo.x) * 0.5, (hi.z - lo.z) * 0.5), hi.z),
        None => (Vec2::ZERO, fallback * 0.5, fallback.y * 0.5),
    }
}

/// Rektangelet til et hus med origo i `base` og yaw.
fn rect_at(base: Vec2, yaw: f32, mid: Vec2, half: Vec2) -> Rect {
    let r = Rect { c: base, h: half, yaw };
    let (ax, az) = r.axes();
    Rect { c: base + ax * mid.x + az * mid.y, ..r }
}

/// Setter ut husene. Gir stedet foran døra til hver bygning på torget,
/// og hvilken vei den vender.
fn houses(b: &mut Builder) -> Vec<(Vec2, f32)> {
    let mut plan = Plan { taken: vec![], free: vec![] };
    // Portene, kirkegården og øvingsplassen holdes fri.
    for south in [true, false] {
        plan.free.push((gate(south).0, 9.0));
    }
    plan.free.push((Vec2::new(-29.0, -29.0), 12.5));
    plan.free.push((Vec2::new(32.0, -26.0), 10.0));
    // Bygningene på torget først, med døra mot torget.
    let mut doors = vec![];
    for (model, edge, t, size) in SPECIALS {
        let (a, bb) = (SQUARE[edge], SQUARE[(edge + 1) % 4]);
        let out = square_out(edge);
        let m = a + (bb - a) * t;
        let (mid, half, front) = footprint(b, model, size);
        let yaw = facing(-out);
        let base = m + out * (front + 0.8);
        let r = rect_at(base, yaw, mid, half);
        plan.taken.push(r);
        building(b, model, base, yaw);
        log::info!("{model} ved ({:.1}, {:.1}), yaw {:.2}", base.x, base.y, yaw);
        doors.push((m - out * 1.6, yaw));
    }
    // Så rekkene: langs kantene av torget (bare utover) og langs gatene på
    // begge sider.
    let mut rng = Rng(0x0f12_57ad);
    let mut rows: Vec<(Vec<Vec2>, f32, Option<Vec2>)> = (0..4).map(|i| (vec![SQUARE[i], SQUARE[(i + 1) % 4]], 0.4, Some(square_out(i)))).collect();
    for st in streets() {
        rows.push((st.pts.clone(), st.half, None));
    }
    let mut placed = 0;
    for (pts, half, only) in rows {
        // Lengde langs linja, til å gå med jevne steg.
        let mut acc = vec![0.0];
        for w in pts.windows(2) {
            acc.push(acc.last().unwrap() + w[0].distance(w[1]));
        }
        let len = *acc.last().unwrap();
        let point = |s: f32| {
            let i = acc.partition_point(|&a| a <= s).clamp(1, pts.len() - 1);
            let t = ((s - acc[i - 1]) / (acc[i] - acc[i - 1]).max(1e-4)).clamp(0.0, 1.0);
            (pts[i - 1].lerp(pts[i], t), (pts[i] - pts[i - 1]).normalize_or_zero())
        };
        for side in [1.0f32, -1.0] {
            let mut s = rng.range(0.0, 3.0);
            while s < len {
                let model = rng.pick(&HOUSES);
                let (mid, half_m, front) = footprint(b, model, Vec2::new(7.0, 8.0));
                let (p, t) = point((s + half_m.x).min(len));
                let n = Vec2::new(-t.y, t.x) * side;
                if let Some(o) = only {
                    if n.dot(o) < 0.0 {
                        break;
                    }
                }
                let setback = half + rng.range(0.25, 1.4);
                let base = p + n * (setback + front);
                let yaw = facing(-n) + rng.range(-0.07, 0.07);
                let r = rect_at(base, yaw, mid, half_m + Vec2::splat(0.05));
                if plan.fits(&r) {
                    plan.taken.push(r);
                    building(b, model, base, yaw);
                    placed += 1;
                    // Tett i rekka, men et smau her og der.
                    s += half_m.x * 2.0 + if rng.chance(0.22) { rng.range(1.6, 3.0) } else { rng.range(0.05, 0.5) };
                } else {
                    s += 0.8;
                }
            }
        }
    }
    // Til slutt bakgårdene: hus og uthus der det er plass mellom rekkene,
    // vendt mot nærmeste gate, så kvartalene blir tette som i en ekte by.
    let mut back = 0;
    for _ in 0..900 {
        let a = rng.range(0.0, TAU);
        let r = rng.range(10.0, terrain::wall_radius(a) - 6.0);
        let p = Vec2::new(a.cos(), a.sin()) * r;
        let (mut best, mut dir, mut near) = (f32::MAX, Vec2::Y, p);
        for st in streets() {
            let (d, t) = street_dist(st, p);
            if d < best {
                best = d;
                dir = t;
                // Punktet på gata: rett inn mot midtlinja.
                let n = Vec2::new(-t.y, t.x);
                let side = n.dot(p - st.pts[0]).signum();
                near = p - n * side * d;
            }
        }
        if best > 26.0 {
            continue;
        }
        // Vendt med gavlen eller langsida mot gata, litt skjevt.
        let n = Vec2::new(-dir.y, dir.x);
        let toward = if n.dot(near - p) >= 0.0 { n } else { -n };
        let turn = if rng.chance(0.3) { PI * 0.5 } else { 0.0 };
        let yaw = facing(toward) + turn + rng.range(-0.1, 0.1);
        let model = rng.pick(&HOUSES);
        let (mid, half_m, _) = footprint(b, model, Vec2::new(7.0, 8.0));
        let r = rect_at(p, yaw, mid, half_m + Vec2::splat(0.05));
        if plan.fits(&r) {
            plan.taken.push(r);
            building(b, model, p, yaw);
            back += 1;
        }
    }
    log::info!("Byen: {placed} hus i rekkene, {back} i bakgårdene");
    street_life(b, &mut plan, &mut rng);
    doors
}

/// Brønnvinda: vinda (med sveiv) dreier om x i høyden `WELL_AXLE`, tauet
/// (1 m langt, skaleres) henger fra forkanten av valsen (`WELL_ROPE_Z`) ned
/// til bøyla på bøtta. Bøtta henger i `WELL_REST` når ingen bruker brønnen
/// og senkes ned i vannet til `WELL_DEPTH`.
pub const WELL_PARTS: [&str; 3] = ["well_winch", "well_rope", "well_bucket"];
pub const WELL_AXLE: f32 = 1.55;
pub const WELL_ROPE_Z: f32 = 0.09;
pub const WELL_REST: f32 = 1.05;
pub const WELL_DEPTH: f32 = -0.25;
pub const WELL_DRUM: f32 = 0.08;

/// Gatelivet i hullene mellom husene: en brønn i hver bydel, flettverksgjerder
/// rundt gårdsplasser og vedstabler inntil husveggene, vendt mot nærmeste
/// gate. Plassen settes av i planen så ingenting havner oppå noe annet.
fn street_life(b: &mut Builder, plan: &mut Plan, rng: &mut Rng) {
    // Lyktestolper langs tverrgatene og smauene, annenhver side, omtrent
    // hver 18. meter (Storgata og torget har sine fra før).
    let mut lamps = 0;
    for st in &streets()[1..] {
        let mut acc = 0.0;
        let mut side = 1.0f32;
        let mut next = 6.0;
        for w in st.pts.windows(2) {
            let (a, bb) = (w[0], w[1]);
            let seg = a.distance(bb);
            while next < acc + seg {
                let t = (next - acc) / seg;
                let dir = (bb - a).normalize_or_zero();
                let n = Vec2::new(-dir.y, dir.x) * side;
                for off in [0.7f32, 1.2] {
                    let c = a.lerp(bb, t) + n * (st.half + off);
                    let rect = Rect { c, h: Vec2::splat(0.35), yaw: 0.0 };
                    if plan.fits(&rect) && terrain::town_edge(c.x, c.y) < -2.0 {
                        plan.taken.push(rect);
                        let base = b.on_ground(Vec3::new(c.x, 0.0, c.y));
                        if b.place("lantern_post", at(base, rng.range(0.0, TAU))) {
                            b.solid(Shape::Cylinder { center: base + Vec3::Y * 1.4, radius: 0.15, half_height: 1.4 });
                            b.lamp_light(base + Vec3::Y * 2.68, Vec3::new(1.0, 0.68, 0.38), 2.2, 10.0);
                            lamps += 1;
                        }
                        break;
                    }
                }
                side = -side;
                next += 18.0;
            }
            acc += seg;
        }
    }
    log::info!("Gatelys: {lamps} lyktestolper langs gatene");
    let mut wells: Vec<Vec2> = vec![];
    let (mut fences, mut piles) = (0, 0);
    for _ in 0..1500 {
        let a = rng.range(0.0, TAU);
        let r = rng.range(8.0, terrain::wall_radius(a) - 5.0);
        let p = Vec2::new(a.cos(), a.sin()) * r;
        // Nærmeste gate, og retningen mot den (fram for det som settes ut).
        let (mut best, mut toward) = (f32::MAX, Vec2::Y);
        for st in streets() {
            let (d, t) = street_dist(st, p);
            if d < best {
                best = d;
                let n = Vec2::new(-t.y, t.x);
                let probe = p + n * 0.1;
                toward = if street_dist(st, probe).0 < d { n } else { -n };
            }
        }
        let yaw = facing(toward) + rng.range(-0.08, 0.08);
        // Brønnene: én i hver kvart av byen, nær en gate.
        if wells.len() < 4 && best < 9.0 && wells.iter().all(|w| w.distance(p) > 30.0) {
            let rect = Rect { c: p, h: Vec2::splat(1.5), yaw };
            if plan.fits(&rect) {
                plan.taken.push(rect);
                let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
                if b.place("well", at(base, yaw)) {
                    b.solid(Shape::Cylinder { center: base + Vec3::Y * 0.45, radius: 1.0, half_height: 0.5 });
                    for sx in [-0.75f32, 0.75] {
                        b.solid(Shape::Cylinder { center: at(base, yaw).transform_point3(Vec3::new(sx, 1.1, 0.0)), radius: 0.12, half_height: 1.1 });
                    }
                    // Vinda, tauet og bøtta flyttes av økta når du heiser.
                    for model in WELL_PARTS {
                        b.reserve(model, 1);
                    }
                    b.target(Use::Well(wells.len()), at(base, yaw), Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.1, 0.5, 1.1));
                }
                log::info!("Brønn ved ({:.1}, {:.1}), yaw {:.2}", p.x, p.y, yaw);
                wells.push(p);
                continue;
            }
        }
        // Vedstabler: langs gata, inntil et hus (noe tatt like bak).
        if piles < 14 && best < 8.0 {
            let rect = Rect { c: p, h: Vec2::new(1.4, 0.6), yaw };
            let behind = Rect { c: p - toward * 1.2, h: Vec2::new(0.8, 0.4), yaw };
            if plan.fits(&rect) && !plan.fits(&behind) {
                plan.taken.push(rect);
                let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
                if b.place("woodpile", at(base, yaw)) {
                    b.solid_box(at(base, yaw).transform_point3(Vec3::new(0.0, 0.7, 0.0)), Vec3::new(2.0, 1.4, 0.6), yaw);
                }
                piles += 1;
                continue;
            }
        }
        // Gjerder: tre-fem stykker i rekke langs gata, rundt en gårdsplass.
        if fences < 16 && best > 5.0 && best < 16.0 {
            let n = 3 + (rng.next() * 2.99) as usize;
            let along = Vec2::new(yaw.cos(), -yaw.sin());
            let len = n as f32 * 2.0;
            let rect = Rect { c: p, h: Vec2::new(len * 0.5 + 0.2, 0.3), yaw };
            if plan.fits(&rect) {
                plan.taken.push(rect);
                for k in 0..n {
                    let c = p + along * ((k as f32 + 0.5) * 2.0 - len * 0.5);
                    let base = b.on_ground(Vec3::new(c.x, 0.0, c.y));
                    if b.place("wattle_fence", at(base, yaw + rng.range(-0.03, 0.03))) {
                        b.solid_box(base + Vec3::Y * 0.5, Vec3::new(2.0, 1.0, 0.15), yaw);
                    }
                }
                fences += 1;
            }
        }
    }
    log::info!("Gatelivet: {} brønner, {fences} gjerder, {piles} vedstabler", wells.len());
}

pub fn build(b: &mut Builder) -> Town {
    let doors = houses(b);
    wall(b);
    square(b, &doors);
    church(b);
    yard(b);
    let crypt_exit = castle(b);
    signposts(b);
    people(b, &doors);
    wanderers(b, &doors);
    let (g, out) = gate(true);
    let spawn = Vec3::new(main_x(g.y - 8.0), 0.0, g.y - 8.0);
    Town { spawn, spawn_yaw: facing(out), crypt_anchor: CRYPT, crypt_exit }
}

/// Husene man kan gå inn i: kollidere (midte, mål, yaw) og lys (posisjon,
/// farge, styrke, radius) i modellens rom, fra `tools/models/buildings.py`
/// (`<navn>_inside.json`).
#[derive(serde::Deserialize, Default)]
struct Inside {
    boxes: Vec<([f32; 3], [f32; 3], f32)>,
    lights: Vec<([f32; 3], [f32; 3], f32, f32)>,
    /// Ramper over trinnene (punktene i et konvekst skall).
    #[serde(default)]
    hulls: Vec<Vec<[f32; 3]>>,
    /// Lykter som tennes om natta (vegglykta ved døra).
    #[serde(default)]
    lamps: Vec<([f32; 3], [f32; 3], f32, f32)>,
    /// Pipene: toppen og hvor tykk røyken er.
    #[serde(default)]
    smoke: Vec<([f32; 3], f32)>,
}

const INSIDE: [(&str, &str); 11] = [
    ("apothecary", include_str!("../../../assets/models/town/apothecary_inside.json")),
    ("house_a", include_str!("../../../assets/models/town/house_a_inside.json")),
    ("house_b", include_str!("../../../assets/models/town/house_b_inside.json")),
    ("house_c", include_str!("../../../assets/models/town/house_c_inside.json")),
    ("house_d", include_str!("../../../assets/models/town/house_d_inside.json")),
    ("house_e", include_str!("../../../assets/models/town/house_e_inside.json")),
    ("house_f", include_str!("../../../assets/models/town/house_f_inside.json")),
    ("forge", include_str!("../../../assets/models/town/forge_inside.json")),
    ("tavern", include_str!("../../../assets/models/town/tavern_inside.json")),
    ("scriptorium", include_str!("../../../assets/models/town/scriptorium_inside.json")),
    ("stave_church", include_str!("../../../assets/models/town/stave_church_inside.json")),
];

fn inside(model: &str) -> Option<&'static Inside> {
    static PARSED: OnceLock<Vec<(&'static str, Inside)>> = OnceLock::new();
    let all = PARSED.get_or_init(|| INSIDE.iter().map(|(n, json)| (*n, serde_json::from_str(json).unwrap_or_default())).collect());
    all.iter().find(|(n, _)| *n == model).map(|(_, i)| i)
}

/// Et hus: modellen på tomta, og kollidere så ingen går gjennom veggene.
/// Hus man kan gå inn i har egne kollidere for vegger med døråpning og
/// møblene, og lys inne; de andre får en boks rundt grunnflaten. Bakken
/// under er planert inne i byen, så huset står i terrenghøyden i midten;
/// sokkelen i modellen tar resten.
fn building(b: &mut Builder, model: &'static str, pos: Vec2, yaw: f32) {
    let base = b.on_ground(Vec3::new(pos.x, 0.0, pos.y));
    let t = at(base, yaw);
    if !b.place(model, t) {
        return;
    }
    let inside = inside(model);
    if let Some(inside) = inside {
        for (p, color, intensity, radius) in &inside.lamps {
            b.lamp_light(t.transform_point3(Vec3::from(*p)), Vec3::from(*color), *intensity, *radius);
        }
        for (p, strength) in &inside.smoke {
            b.smoke(t.transform_point3(Vec3::from(*p)), *strength);
        }
    }
    if let Some(inside) = inside.filter(|i| !i.boxes.is_empty()) {
        for (c, size, y) in &inside.boxes {
            b.solid_box(t.transform_point3(Vec3::from(*c)), Vec3::from(*size), yaw + y);
        }
        for pts in &inside.hulls {
            b.solid(Shape::Hull { points: pts.iter().map(|p| t.transform_point3(Vec3::from(*p))).collect() });
        }
        for (p, color, intensity, radius) in &inside.lights {
            b.fire_light(t.transform_point3(Vec3::from(*p)), Vec3::from(*color), *intensity, *radius);
        }
        return;
    }
    if let Some((lo, hi)) = b.bounds(model) {
        let size = hi - lo;
        let mid = (lo + hi) * 0.5;
        let c = at(base, yaw).transform_point3(Vec3::new(mid.x, size.y * 0.5, mid.z));
        b.solid_box(c, size, yaw);
    }
}

/// Bymuren: segmenter på 6 m langs det ujevne omrisset
/// (`terrain::town_edge` = 0), tårn der muren knekker, og porthus der
/// Storgata går gjennom.
fn wall(b: &mut Builder) {
    // Gå rundt omrisset med små steg, og plukk ut punkter med 6 m mellom.
    let steps = 720;
    let path: Vec<Vec2> = (0..=steps)
        .map(|i| {
            let a = TAU * i as f32 / steps as f32;
            Vec2::new(a.cos(), a.sin()) * terrain::wall_radius(a)
        })
        .collect();
    let seg_len = 6.0;
    let mut pts: Vec<Vec2> = vec![path[0]];
    let mut carry = 0.0;
    for w in path.windows(2) {
        let (a, c) = (w[0], w[1]);
        let d = (c - a).length();
        let mut t = seg_len - carry;
        while t <= d {
            pts.push(a + (c - a) * (t / d));
            t += seg_len;
        }
        carry = d - (t - seg_len);
    }
    let gates = [gate(true), gate(false)];
    // Ved portene kappes murbitene der de går inn i porttårnene (porthuset
    // er 13 m bredt), så muren går helt inn til tårnene uten glipe.
    const GATE_R: f32 = 5.4;
    let mut pieces = vec![];
    for w in pts.windows(2) {
        let (a, c) = (w[0], w[1]);
        let Some(g) = gates.iter().map(|(g, _)| *g).find(|g| (a + c).distance(*g * 2.0) < 2.0 * (GATE_R + 6.0)) else {
            pieces.push((a, c));
            continue;
        };
        // Delene av a–c utenfor sirkelen rundt porten.
        let d = c - a;
        let f = a - g;
        let (qa, qb, qc) = (d.dot(d), 2.0 * f.dot(d), f.dot(f) - GATE_R * GATE_R);
        let disc = qb * qb - 4.0 * qa * qc;
        if disc <= 0.0 {
            pieces.push((a, c));
            continue;
        }
        let (t0, t1) = ((-qb - disc.sqrt()) / (2.0 * qa), (-qb + disc.sqrt()) / (2.0 * qa));
        for (s0, s1) in [(0.0, t0.min(1.0)), (t1.max(0.0), 1.0)] {
            if (s1 - s0) * d.length() > 0.8 {
                pieces.push((a + d * s0, a + d * s1));
            }
        }
    }
    for (a, c) in pieces {
        let mid = (a + c) * 0.5;
        let len = (c - a).length();
        let dir = (c - a) / len;
        // Murens lokale +X går langs muren; utsida er lokal −Z. Kappede
        // biter strekkes eller presses langs muren.
        let yaw = (-dir.y).atan2(dir.x);
        let base = b.on_ground(Vec3::new(mid.x, 0.0, mid.y));
        let pose = at(base, yaw) * Affine3A::from_scale(Vec3::new(len / seg_len, 1.0, 1.0));
        if b.place("town_wall", pose) {
            b.solid_box(base + Vec3::Y * 3.0, Vec3::new(len + 0.2, 6.0, 1.6), yaw);
        }
    }
    // Tårnene: med jevne mellomrom, utenom portene.
    for (k, p) in pts.iter().enumerate() {
        if k % 9 != 4 || gates.iter().any(|(g, _)| p.distance(*g) < 14.0) {
            continue;
        }
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        if b.place("wall_tower", at(base, p.y.atan2(p.x))) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 6.0, radius: 3.4, half_height: 6.0 });
        }
    }
    for (g, out) in gates {
        let base = b.on_ground(Vec3::new(g.x, 0.0, g.y));
        let yaw = facing(out);
        if b.place("gatehouse", at(base, yaw)) {
            // To tårn på sidene og murblokken over portrommet.
            for sx in [-1.0f32, 1.0] {
                let side = at(base, yaw).transform_point3(Vec3::new(sx * 4.6, 5.0, 0.0));
                b.solid_box(side, Vec3::new(4.0, 10.0, 5.0), yaw);
            }
            b.solid_box(at(base, yaw).transform_point3(Vec3::new(0.0, 6.4, 0.0)), Vec3::new(5.6, 6.6, 4.4), yaw);
        }
    }
}

/// Torget: torgkorset, markedsbodene, kjerrer, tønner og kasser, og
/// lykter. Hver ting skyves inn mot midten av torget til den står fri av
/// Storgata og har plass foran dørene til bygningene langs kanten.
fn square(b: &mut Builder, doors: &[(Vec2, f32)]) {
    let center = SQUARE.iter().copied().sum::<Vec2>() * 0.25;
    let mut placed = vec![];
    let clear = |p: Vec2, r: f32, placed: &[(Vec2, f32)]| {
        placed.iter().all(|(q, rq)| q.distance(p) > r + rq + 0.6) && street_dist(&streets()[0], p).0 > MAIN + r + 0.4 && doors.iter().all(|(d, _)| d.distance(p) > 4.5 + r) && square_dist(p) < -r - 0.3
    };
    let settle = |p: Vec2, r: f32, placed: &mut Vec<(Vec2, f32)>| {
        // Nærmeste ledige plass i ringer rundt der tingen er tenkt.
        for ring in 0..24 {
            let rad = ring as f32 * 0.5;
            let n = if ring == 0 { 1 } else { 12 + ring * 2 };
            for k in 0..n {
                let a = TAU * k as f32 / n as f32;
                let q = p + Vec2::new(a.cos(), a.sin()) * rad;
                if clear(q, r, placed) {
                    placed.push((q, r));
                    return Some(q);
                }
            }
        }
        None
    };
    let c = settle(Vec2::new(-5.5, -4.0), 1.2, &mut placed).unwrap_or(center);
    let cross = b.on_ground(Vec3::new(c.x, 0.0, c.y));
    if b.place("market_cross", at(cross, 0.3)) {
        b.solid(Shape::Cylinder { center: cross + Vec3::Y * 1.5, radius: 0.9, half_height: 1.5 });
    }
    let stalls = [(-9.0, 6.5, 0.25), (-5.0, 8.0, 0.1), (9.0, 5.5, -0.2), (10.0, -6.5, PI - 0.15), (6.5, -9.0, PI + 0.1)];
    for (k, (x, z, yaw)) in stalls.into_iter().enumerate() {
        let Some(p) = settle(Vec2::new(x, z), 1.5, &mut placed) else { continue };
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        let model = if k % 2 == 0 { "market_stall" } else { "market_stall_b" };
        if b.place(model, at(base, yaw)) {
            b.solid_box(base + Vec3::Y * 0.5, Vec3::new(2.6, 1.0, 1.4), yaw);
        }
    }
    let props = [("cart", -7.0, -9.0, 1.2), ("barrel", -7.6, 9.0, 0.0), ("barrel", -6.9, 9.6, 1.0), ("crate", 10.6, 7.2, 0.3), ("crate", 11.2, 6.4, 1.1), ("barrel", 11.6, -8.4, 2.0)];
    for (model, x, z, yaw) in props {
        let Some(p) = settle(Vec2::new(x, z), if model == "cart" { 1.5 } else { 0.5 }, &mut placed) else { continue };
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        if b.place(model, at(base, yaw)) {
            if let Some((lo, hi)) = b.bounds(model) {
                let size = hi - lo;
                b.solid_box(base + Vec3::Y * size.y * 0.5, Vec3::new(size.x, size.y, size.z), yaw);
            }
        }
    }
    // Lykter i hjørnene av torget og langs Storgata, som lyser om natta.
    let mut spots: Vec<Vec2> = SQUARE.iter().map(|c| *c * 0.86).collect();
    for (k, z) in [-44.0f32, -30.0, 24.0, 38.0].into_iter().enumerate() {
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        spots.push(Vec2::new(main_x(z) + side * (MAIN + 0.5), z));
    }
    for p in spots {
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        if b.place("lantern_post", at(base, 0.0)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 1.4, radius: 0.15, half_height: 1.4 });
            b.lamp_light(base + Vec3::Y * 2.68, Vec3::new(1.0, 0.68, 0.38), 2.4, 11.0);
        }
    }
}

/// Hvem som står hvor, og hva de sier.
const SMITH: &[&str] = &[
    "Knappesmeden, til tjeneste. Jeg smir det du bærer opp fra krypten.",
    "Et sverd trenger en klinge og et grep: en divider og en knapp. Resten er håndlag.",
    "Kom tilbake med komponenter, så får vi se hva stålet duger til.",
];
const CLERK: &[&str] = &[
    "Skriptoriet. Ingenting smis uten et korrekt utfylt skjema.",
    "Oppskriftene er skjemaer. Fyll ut feltene med komponentene du har vunnet, og send.",
    "Bøkene her er dokumentasjonen. Les om det du bærer, så leges sårene.",
];
const APOTHECARY: &[&str] = &[
    "Apoteket. Drikker mot det meste, tokens mot alt.",
    "Les i bøkene når du er skadet. Kunnskap leger.",
    "Hold deg unna lavaen nederst i dypet. Den står ikke i noen bok.",
];
const GORVEK: &[&str] = &[
    "Gorvek. Jeg lærer deg trekk, om du har tokens og tålmodighet.",
    "Blokk i det bittet kommer. Da står de lamslått, og du hugger dobbelt.",
    "Unna med Shift. Det koster pust, men sparer blod.",
];
const INNKEEPER: &[&str] = &[
    "Velkommen til Vertshuset. Øl er det nok av, nyheter også.",
    "De sier krypten under borgen går dypere enn noen har gått.",
    "Det kommer rare ting opp fra dypet om natta. Knapper med tenner.",
];

/// Stavkirka på kirkegården nordvest i byen, med steingjerde og gravsteiner.
fn church(b: &mut Builder) {
    let c = Vec2::new(-30.0, -30.0);
    // Portalen mot øst, mot byen; boksen rundt modellens mål er kollideren.
    building(b, "stave_church", c, FRAC_PI_2);
    // Gravene i to ujevne rader sør for kirka, med haugen mot øst.
    let mut rng = Rng(0x0c4e_7e12);
    for row in 0..2 {
        for k in 0..6 {
            if rng.chance(0.15) {
                continue;
            }
            let p = Vec2::new(-36.0 + k as f32 * 1.9 + rng.range(-0.3, 0.3), -22.6 + row as f32 * 2.6 + rng.range(-0.3, 0.3));
            let g = b.on_ground(Vec3::new(p.x, 0.0, p.y));
            let yaw = FRAC_PI_2 + rng.range(-0.12, 0.12);
            if b.place("gravestone", at(g, yaw)) {
                b.solid_box(g + Vec3::Y * 0.45, Vec3::new(0.6, 0.9, 0.2), yaw);
            }
        }
    }
}

/// Gorveks øvingsplass nordøst i byen: treningsdukker og et våpenstativ.
fn yard(b: &mut Builder) {
    for (x, z, yaw) in [(28.0, -30.0, 0.4), (33.0, -32.0, -0.3), (37.0, -28.0, 0.1)] {
        let base = b.on_ground(Vec3::new(x, 0.0, z));
        if b.place("training_dummy", at(base, yaw)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 0.9, radius: 0.35, half_height: 0.9 });
        }
    }
    let rack = b.on_ground(Vec3::new(32.0, 0.0, -24.0));
    if b.place("weapon_rack", at(rack, 0.0)) {
        b.solid_box(rack + Vec3::Y * 0.8, Vec3::new(2.0, 1.6, 0.6), 0.0);
    }
}

/// Veiskiltene til krypten: ved torget, innenfor nordporten og der stien
/// begynner å klatre opp borghaugen. Pila (lokal +X) peker videre langs
/// veien.
fn signposts(b: &mut Builder) {
    let (north, _) = gate(false);
    let path = |t: f32| Vec2::new(north.x * (1.0 - t) + CRYPT.x * t + (t * PI).sin() * 6.0, north.y + (CRYPT.y - north.y) * t);
    let street = |z: f32| Vec2::new(main_x(z), z);
    // (der skiltet står, punktet det peker mot)
    let signs = [
        (street(-11.5) - Vec2::new(MAIN + 1.0, 0.0), street(-30.0)),
        (street(north.y + 6.0) + Vec2::new(MAIN - 0.5, 0.0), north),
        (path(0.25) + Vec2::new(2.6, 0.0), path(0.45)),
    ];
    for (at, to) in signs {
        let d = to - at;
        let pos = b.on_ground(Vec3::new(at.x, 0.0, at.y));
        let pose = Affine3A::from_rotation_translation(Quat::from_rotation_y((-d.y).atan2(d.x)), pos);
        log::info!("Veiskilt ved ({:.1}, {:.1}) mot ({:.1}, {:.1})", at.x, at.y, to.x, to.y);
        if b.place("signpost", pose) {
            b.solid_box(pos + Vec3::Y * 1.3, Vec3::new(0.2, 2.6, 0.2), 0.0);
        }
    }
}

/// Borgruinen på haugen, og krypten med døra ned. Gir stedet du kommer
/// opp til fra kjelleren.
fn castle(b: &mut Builder) -> (Vec3, f32) {
    let (hx, hz, _, _) = terrain::HILL;
    let keep = b.on_ground(Vec3::new(hx, 0.0, hz));
    if b.place("castle_ruin", at(keep, 0.0)) {
        // Murene i ruinen: fire løp rundt borggården, med åpning i sør.
        for (c, size) in [(Vec3::new(0.0, 4.0, -20.0), Vec3::new(40.0, 8.0, 2.4)), (Vec3::new(-20.0, 4.0, 0.0), Vec3::new(2.4, 8.0, 40.0)), (Vec3::new(20.0, 4.0, 0.0), Vec3::new(2.4, 8.0, 40.0)), (Vec3::new(-12.5, 4.0, 20.0), Vec3::new(15.0, 8.0, 2.4)), (Vec3::new(12.5, 4.0, 20.0), Vec3::new(15.0, 8.0, 2.4))] {
            b.solid_box(keep + c, size, 0.0);
        }
        // Hjørnetårnene, porttårnene, hovedtårnet i nordvest og den raste
        // brønnen midt i borggården.
        for (x, z) in [(-20.0, -20.0), (20.0, -20.0), (-20.0, 20.0), (20.0, 20.0)] {
            b.solid(Shape::Cylinder { center: keep + Vec3::new(x, 4.0, z), radius: 3.3, half_height: 5.0 });
        }
        for x in [-6.6f32, 6.6] {
            b.solid_box(keep + Vec3::new(x, 3.5, 20.0), Vec3::new(3.4, 7.0, 3.4), 0.0);
        }
        for (c, size) in [(Vec3::new(-11.0, 6.0, -14.5), Vec3::new(7.0, 12.0, 1.4)), (Vec3::new(-14.5, 6.0, -11.0), Vec3::new(1.4, 12.0, 7.0)), (Vec3::new(-7.5, 3.6, -11.0), Vec3::new(1.4, 7.2, 7.0))] {
            b.solid_box(keep + c, size, 0.0);
        }
        b.solid(Shape::Cylinder { center: keep + Vec3::Y * 0.5, radius: 1.6, half_height: 0.8 });
    }
    // Krypten: et lite gravkapell med døra mot sør, og trappa ned bak døra.
    let crypt = b.on_ground(Vec3::new(CRYPT.x, 0.0, CRYPT.y));
    if b.place("crypt", at(crypt, 0.0)) {
        b.solid_box(crypt + Vec3::new(0.0, 2.5, -2.0), Vec3::new(6.0, 5.0, 6.0), 0.0);
    }
    // Døra: det du sikter på. Står i forkant av kapellet, vendt mot sør.
    b.target(Use::Descend, at(crypt, 0.0), Vec3::new(0.0, 1.2, 1.05), Vec3::new(0.75, 1.2, 0.2));
    b.fire_light(crypt + Vec3::new(1.3, 2.4, 1.6), Vec3::new(1.0, 0.6, 0.3), 2.0, 8.0);
    b.fire_light(crypt + Vec3::new(-1.3, 2.4, 1.6), Vec3::new(1.0, 0.6, 0.3), 2.0, 8.0);
    (crypt + Vec3::new(0.0, 0.0, 3.2), PI)
}

/// Innbyggerne foran bygningene sine på torget, og Gorvek på øvingsplassen.
fn people(b: &mut Builder, doors: &[(Vec2, f32)]) {
    let at_door = |i: usize| doors.get(i).copied().unwrap_or((Vec2::ZERO, 0.0));
    // Rekkefølgen i SPECIALS: apotek, vertshus, smie, skriptorium.
    // Figuren: en egen modell (fra tools/models/people.py) om den finnes.
    let cast: [(&'static str, &'static str, (Vec2, f32), Vec3, &'static [&'static str]); 5] = [
        ("Knappesmeden", "npc_smith", at_door(2), Vec3::new(0.22, 0.1, 0.05), SMITH),
        ("Skriptoriet", "npc_clerk", at_door(3), Vec3::new(0.06, 0.12, 0.42), CLERK),
        ("Apotekeren", "npc_apothecary", at_door(0), Vec3::new(0.06, 0.28, 0.1), APOTHECARY),
        ("Vertinnen", "npc_innkeeper", at_door(1), Vec3::new(0.45, 0.12, 0.1), INNKEEPER),
        ("Gorvek", "npc_gorvek", (Vec2::new(31.0, -18.5), 0.15), Vec3::new(0.3, 0.1, 0.04), GORVEK),
    ];
    for (i, (name, own, (p, yaw), tint, lines)) in cast.into_iter().enumerate() {
        let pos = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        log::info!("Innbygger {name} ved ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
        let model = if b.has_model(own) {
            b.reserve(own, 1);
            own
        } else {
            crate::domain::npcs::NPC_MODEL
        };
        // Ruta: fram og tilbake langs fasaden foran døra (torget holdes fritt
        // 4,5 m ut fra dørene), og for Gorvek mellom dukkene.
        let side = Vec2::new(yaw.cos(), -yaw.sin());
        let pts: Vec<Vec2> = if name == "Gorvek" {
            vec![p, Vec2::new(31.0, -23.5), Vec2::new(34.5, -25.5), Vec2::new(30.0, -26.0)]
        } else {
            vec![p, p + side * 2.4, p, p - side * 2.4]
        };
        let route = pts.iter().map(|q| (b.on_ground(Vec3::new(q.x, 0.0, q.y)), 3.0)).collect();
        b.npcs.push(NpcSpec { name, model, pos, yaw, tint, lines, route });
        b.target(Use::Talk(i), Affine3A::from_translation(pos), Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.3, 0.9, 0.3));
    }
}

/// Folk i byen som går ærender: til brønnene, over torget, til portene og
/// dørene på torget, langs gatene. Veiene er et nett av midtlinjene i
/// gatene og kanten av torget, uten kanter som går gjennom noe, og hver
/// innbygger går korteste vei mellom ærendene sine og blir stående litt
/// ved hvert.
fn wanderers(b: &mut Builder, doors: &[(Vec2, f32)]) {
    // Fritt for kollidere i kroppshøyde, med en halv meter å gå på.
    let shapes: Vec<(Vec2, Vec2, f32, f32)> = b
        .colliders
        .iter()
        .filter_map(|c| match c {
            Shape::Cuboid { center, size, yaw } if size.y > 0.6 && center.y > -5.0 => Some((Vec2::new(center.x, center.z), Vec2::new(size.x, size.z) * 0.5, *yaw, 0.0)),
            Shape::Cylinder { center, radius, .. } if center.y > -5.0 => Some((Vec2::new(center.x, center.z), Vec2::ZERO, 0.0, *radius)),
            _ => None,
        })
        .collect();
    let free = |p: Vec2| {
        terrain::town_edge(p.x, p.y) < -1.5
            && shapes.iter().all(|(c, half, yaw, r)| {
                let d = p - *c;
                if *r > 0.0 {
                    return d.length() > r + 0.45;
                }
                let (sn, cs) = yaw.sin_cos();
                let l = Vec2::new(d.x * cs - d.y * sn, d.x * sn + d.y * cs).abs();
                l.x > half.x + 0.45 || l.y > half.y + 0.45
            })
    };
    let clear = |a: Vec2, c: Vec2| {
        let n = ((a.distance(c) / 0.4).ceil() as usize).max(1);
        (0..=n).all(|k| free(a.lerp(c, k as f32 / n as f32)))
    };
    // Nodene: punktene langs gatene og kanten av torget.
    let mut nodes: Vec<Vec2> = vec![];
    let mut edges: Vec<Vec<(usize, f32)>> = vec![];
    let link = |nodes: &Vec<Vec2>, edges: &mut Vec<Vec<(usize, f32)>>, i: usize, j: usize| {
        let d = nodes[i].distance(nodes[j]);
        if i != j && clear(nodes[i], nodes[j]) && !edges[i].iter().any(|(k, _)| *k == j) {
            edges[i].push((j, d));
            edges[j].push((i, d));
        }
    };
    for st in streets() {
        let start = nodes.len();
        // Tettere enn polylinjen, så folk følger svingene.
        // Står det noe på midtlinja (en lykt, et skilt, en brønn), flyttes
        // punktet sideveis til nærmeste ledige sted i gata.
        let mut put = |p: Vec2, dir: Vec2| {
            let across = Vec2::new(-dir.y, dir.x);
            if let Some(q) = [0.0, 0.8, -0.8, 1.5, -1.5, 2.2, -2.2].iter().map(|o| p + across * *o).find(|q| free(*q)) {
                nodes.push(q);
            }
        };
        for w in st.pts.windows(2) {
            let n = ((w[0].distance(w[1]) / 3.0).ceil() as usize).max(1);
            let dir = (w[1] - w[0]).normalize_or_zero();
            for k in 0..n {
                put(w[0].lerp(w[1], k as f32 / n as f32), dir);
            }
        }
        let last = st.pts.len() - 1;
        put(st.pts[last], (st.pts[last] - st.pts[last - 1]).normalize_or_zero());
        edges.resize(nodes.len(), vec![]);
        for i in start..nodes.len() - 1 {
            link(&nodes, &mut edges, i, i + 1);
        }
    }
    let center = SQUARE.iter().copied().sum::<Vec2>() * 0.25;
    let ring_start = nodes.len();
    for i in 0..4 {
        let (a, c) = (SQUARE[i], SQUARE[(i + 1) % 4]);
        for k in 0..4 {
            let p = a.lerp(c, k as f32 / 4.0);
            let inward = (center - p).normalize();
            if let Some(q) = [3.0, 2.0, 4.5, 6.0].iter().map(|d| p + inward * *d).find(|q| free(*q)) {
                nodes.push(q);
            }
        }
    }
    edges.resize(nodes.len(), vec![]);
    for i in ring_start..nodes.len() {
        let j = if i + 1 == nodes.len() { ring_start } else { i + 1 };
        link(&nodes, &mut edges, i, j);
    }
    // Der gatene møtes hverandre og torget.
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            if nodes[i].distance(nodes[j]) < 6.5 {
                link(&nodes, &mut edges, i, j);
            }
        }
    }
    let path = |from: usize, to: usize| -> Option<Vec<usize>> {
        let mut dist = vec![f32::MAX; nodes.len()];
        let mut prev = vec![usize::MAX; nodes.len()];
        let mut done = vec![false; nodes.len()];
        dist[from] = 0.0;
        loop {
            let u = (0..nodes.len()).filter(|&i| !done[i] && dist[i] < f32::MAX).min_by(|a, b| dist[*a].total_cmp(&dist[*b]))?;
            if u == to {
                break;
            }
            done[u] = true;
            for &(v, w) in &edges[u] {
                if dist[u] + w < dist[v] {
                    dist[v] = dist[u] + w;
                    prev[v] = u;
                }
            }
        }
        let mut out = vec![to];
        while *out.last().unwrap() != from {
            out.push(prev[*out.last().unwrap()]);
        }
        out.reverse();
        Some(out)
    };
    // Ærendene: brønnene, torget, dørene på torget og innsiden av portene.
    let mut errands: Vec<(Vec2, f32)> = vec![];
    for t in &b.targets {
        if let Use::Well(_) = t.what {
            let p = Vec2::new(t.pose.translation.x, t.pose.translation.z);
            // Ved siden av brønnen, mot nærmeste gate.
            if let Some(q) = (0..8).map(|k| p + Vec2::from_angle(k as f32 * 0.785) * 1.7).filter(|q| free(*q)).min_by(|a, b| street_gap(*a).total_cmp(&street_gap(*b))) {
                errands.push((q, 8.0));
            }
        }
    }
    for (i, p) in nodes.iter().enumerate().skip(ring_start).step_by(3) {
        let _ = i;
        errands.push((*p, 6.0));
    }
    for (p, yaw) in doors {
        let side = Vec2::new(yaw.cos(), -yaw.sin());
        errands.push((*p + side * 3.5, 5.0));
    }
    for south in [true, false] {
        let (g, out) = gate(south);
        errands.push((g - out * 9.0, 4.0));
    }
    // Hvert ærend kobles til nærmeste node det er fri vei til.
    let anchor: Vec<Option<usize>> = errands
        .iter()
        .map(|(p, _)| (0..nodes.len()).filter(|&i| nodes[i].distance(*p) < 12.0 && clear(nodes[i], *p)).min_by(|a, c| nodes[*a].distance(*p).total_cmp(&nodes[*c].distance(*p))))
        .collect();
    let usable: Vec<usize> = (0..errands.len()).filter(|&i| anchor[i].is_some() && free(errands[i].0)).collect();
    log::info!("Byen: veinett med {} punkter, {} ærender", nodes.len(), usable.len());
    // Bare ærender i den største sammenhengende delen av nettet (et smau
    // som ender blindt bak et hus, er ikke med).
    let mut comp = vec![usize::MAX; nodes.len()];
    let mut sizes = vec![];
    for s0 in 0..nodes.len() {
        if comp[s0] != usize::MAX {
            continue;
        }
        let mut stack = vec![s0];
        comp[s0] = sizes.len();
        let mut n = 0;
        while let Some(u) = stack.pop() {
            n += 1;
            for &(v, _) in &edges[u] {
                if comp[v] == usize::MAX {
                    comp[v] = sizes.len();
                    stack.push(v);
                }
            }
        }
        sizes.push(n);
    }
    let main = (0..sizes.len()).max_by_key(|&k| sizes[k]).unwrap_or(0);
    let usable: Vec<usize> = usable.into_iter().filter(|&i| comp[anchor[i].unwrap()] == main).collect();
    if usable.len() < 3 {
        return;
    }
    let folk: [(&'static str, Vec3, &'static [&'static str]); 8] = [
        ("Bonden", Vec3::new(0.35, 0.28, 0.12), TOWNSFOLK),
        ("Tjenestejenta", Vec3::new(0.5, 0.12, 0.12), TOWNSFOLK),
        ("Vaktmannen", Vec3::new(0.12, 0.14, 0.3), GUARD),
        ("Kremmeren", Vec3::new(0.4, 0.3, 0.05), TOWNSFOLK),
        ("Fiskerkona", Vec3::new(0.15, 0.3, 0.32), TOWNSFOLK),
        ("Gutten", Vec3::new(0.45, 0.42, 0.2), TOWNSFOLK),
        ("Pilegrimen", Vec3::new(0.25, 0.22, 0.2), PILGRIM),
        ("Vaskekona", Vec3::new(0.55, 0.5, 0.42), TOWNSFOLK),
    ];
    let base = b.npcs.len();
    for (k, (name, tint, lines)) in folk.into_iter().enumerate() {
        // Tre ærender hver, fordelt over byen.
        let pick: Vec<usize> = (0..3).map(|j| usable[(k * 5 + j * 7 + j * k) % usable.len()]).collect();
        let mut route: Vec<(Vec3, f32)> = vec![];
        let mut ok = true;
        for j in 0..pick.len() {
            let (a, c) = (pick[j], pick[(j + 1) % pick.len()]);
            if a == c {
                continue;
            }
            route.push((b.on_ground(Vec3::new(errands[a].0.x, 0.0, errands[a].0.y)), errands[a].1));
            let Some(p) = path(anchor[a].unwrap(), anchor[c].unwrap()) else {
                ok = false;
                break;
            };
            for n in p {
                let q = nodes[n];
                route.push((b.on_ground(Vec3::new(q.x, 0.0, q.y)), 0.0));
            }
        }
        if !ok || route.len() < 2 {
            continue;
        }
        // Starter midt på ruta, ute i gata (ikke i veien ved en brønn).
        let half = route.len() / 2;
        route.rotate_left(half);
        let (pos, _) = route[0];
        b.npcs.push(NpcSpec { name, model: crate::domain::npcs::NPC_MODEL, pos, yaw: 0.0, tint, lines, route });
        let i = b.npcs.len() - 1;
        b.target(Use::Talk(i), Affine3A::from_translation(pos), Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.3, 0.9, 0.3));
    }
    log::info!("Byen: {} innbyggere går ærender", b.npcs.len() - base);
}

/// Avstanden fra et punkt til nærmeste gate (midtlinja).
fn street_gap(p: Vec2) -> f32 {
    streets().iter().map(|s| street_dist(s, p).0).fold(f32::MAX, f32::min)
}

const TOWNSFOLK: &[&str] = &[
    "God dag. Vannet i brønnen er kaldt i dag, kaldt som krypten.",
    "Har du vært nede under borgen? Bror min gikk ned i fjor. Han snakker bare om knapper nå.",
    "Torgdag i dag. Alt er dyrere enn i går.",
    "Kirkeklokka ringte feil i natt. Tretten slag.",
];
const GUARD: &[&str] = &[
    "Hold deg unna borghaugen etter mørkets frembrudd.",
    "Portene stenges ikke, men jeg holder øye med hvem som går ut.",
    "Ser du noe som kryper opp fra krypten, rop.",
];
const PILGRIM: &[&str] = &[
    "Jeg har gått fra Nidaros for å se det tapte systemet. De sier det passer i alle rammeverk.",
    "Fristil. Navnet alene gjør meg rolig.",
];

/// Hva bakken er dekket av i et punkt, som vekter mellom 0 og 1: brostein
/// på torget og i Storgata, tråkket jord i Tverrgata, på veiene og rundt
/// husene, berg i bratte skråninger og på borghaugen, skogbunn under
/// granskogen, og gress i resten. `dry` gjør gresset gult (åkrene,
/// ryggene), `wet` gir søle i kantene av veiene.
pub struct Ground {
    pub cobble: f32,
    pub dirt: f32,
    pub rock: f32,
    pub forest: f32,
    pub dry: f32,
    pub wet: f32,
    /// Koordinatene langs nærmeste jordvei i meter: (på tvers fra
    /// midtlinja, langs veien). Søla med hjulspor legges etter dem, så
    /// sporene følger veien og bøyer seg i svingene.
    pub road: Vec2,
    /// Hvor mye av jorda som er tråkket jord (gårdsplassene) i stedet
    /// for veisøle med hjulspor.
    pub earth: f32,
}

fn soft(edge: f32, d: f32, width: f32) -> f32 {
    let t = ((edge - d) / width + 0.5).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn ground_weights(x: f32, z: f32) -> Ground {
    let n1 = terrain::noise(x * 0.35, z * 0.35);
    let n2 = terrain::noise(x * 0.09 + 5.0, z * 0.09);
    let inside = soft(0.0, terrain::town_edge(x, z) + 1.0, 2.0);
    // Brosteinen: torget og Storgata, med ujevn kant der jorda tar over.
    let p = Vec2::new(x, z);
    let off = (1.0 - inside) * 50.0;
    let st = streets();
    let main = street_dist(&st[0], p).0 - MAIN + off;
    let cobble = soft(0.0, square_dist(p).min(main) + (n1 - 0.5) * 1.2, 1.4);
    // Jorda: Tverrgata og smauene, veiene ut, og gårdsplassene rundt husene.
    let mut lanes = f32::MAX;
    let mut coords = Vec2::ZERO;
    for s in st.iter().filter(|s| !s.cobbled) {
        let (d, across, along) = street_coords(s, p);
        if d - s.half < lanes {
            lanes = d - s.half;
            coords = Vec2::new(across, along);
        }
    }
    let mut road = f32::MAX;
    for (d, across, along) in paths(x, z) {
        if d - 1.6 < road {
            road = d - 1.6;
            if road < lanes + off {
                coords = Vec2::new(across, along);
            }
        }
    }
    let roads = soft(0.0, (lanes + off).min(road) + (n1 - 0.5) * 1.6, 1.8);
    // Gårdsplassene er tråkket jord (eget lag uten hjulspor).
    let yards = inside * soft(0.0, (0.3 - n2) * 12.0, 3.0) * 0.85;
    let dirt = roads.max(yards);
    let earth = (yards / (roads + yards).max(1e-4)).clamp(0.0, 1.0);
    // Berg der det er bratt, og litt i toppen av borghaugen.
    let slope = 1.0 - terrain::normal(x, z).y;
    let (hx, hz, hr, _) = terrain::HILL;
    let hill = (((x - hx).powi(2) + (z - hz).powi(2)).sqrt() / hr).clamp(0.0, 1.0);
    // Borghaugen er gress opp lia (berg bare i flekker i borggården).
    let on_hill = 1.0 - ((hill - 0.95) / 0.05).clamp(0.0, 1.0);
    let rock = (soft(0.0, 0.06 - slope + (n1 - 0.5) * 0.05, 0.05) * (1.0 - on_hill)).max(soft(0.0, hill - 0.45 + (n2 - 0.5) * 0.3, 0.1) * 0.4 * (1.0 - soft(0.0, hill - 0.25, 0.1)));
    // Skogbunn der granskogen står tett, langt ute.
    let far = (x * x + z * z).sqrt();
    let forest = soft(0.0, 115.0 - far + (n2 - 0.5) * 40.0, 20.0) * (1.0 - inside);
    // Fordel: brostein over jord over berg over skogbunn, gress resten.
    let cobble = cobble.clamp(0.0, 1.0);
    let dirt = (dirt * (1.0 - cobble)).clamp(0.0, 1.0);
    let rock = (rock * (1.0 - cobble - dirt)).clamp(0.0, 1.0);
    let forest = (forest * (1.0 - cobble - dirt - rock)).clamp(0.0, 1.0);
    // Tørt gress på ryggene og i åkrene.
    let mut dry = (terrain::noise(x * 0.05 + 2.0, z * 0.05) * 1.3 - 0.35).clamp(0.0, 1.0) * 0.6;
    for (cx, cz, rx, rz) in FIELDS {
        let e = ((x - cx) / rx).powi(2) + ((z - cz) / rz).powi(2);
        dry = dry.max(soft(1.0, e, 0.25));
    }
    // Søle i kantene av jordveiene og i gatene i byen.
    let wet = (dirt * (1.0 - dirt) * 4.0 * soft(0.0, 0.55 - n2, 0.2)).clamp(0.0, 1.0);
    Ground { cobble, dirt, rock, forest, dry, wet, road: coords, earth }
}

/// Avstand fra et punkt til veiene utenfor muren: sørover fra sørporten,
/// og opp til borgen fra nordporten.
pub fn path_dist(x: f32, z: f32) -> f32 {
    paths(x, z).into_iter().map(|(d, _, _)| d).fold(f32::MAX, f32::min)
}

/// Veiene utenfor muren som gjelder for punktet: (avstand til midtlinja,
/// fortegnet avstand på tvers, meter langs veien). Begge går omtrent
/// nord–sør, så midtlinja er en x for hver z.
fn paths(x: f32, z: f32) -> Vec<(f32, f32, f32)> {
    let mut out = vec![];
    let (south, _) = gate(true);
    let (north, _) = gate(false);
    // Lengden langs veien regnes med helningen til midtlinja.
    let along = |cx: &dyn Fn(f32) -> f32, z0: f32, z: f32| -> (f32, f32) {
        let c = cx(z);
        let slope = (cx(z + 0.5) - cx(z - 0.5)).abs();
        let stretch = (1.0 + slope * slope).sqrt();
        ((x - c) / stretch, (z - z0) * stretch)
    };
    if z > south.y - 2.0 {
        let cx = |z: f32| {
            let n = terrain::noise(z * 0.03, 9.0);
            let k = ((z - south.y) / 60.0).clamp(0.0, 1.0);
            south.x * (1.0 - k) + (n - 0.5) * 30.0 * k
        };
        let (u, v) = along(&cx, south.y, z);
        out.push(((x - cx(z)).abs(), u, v));
    }
    if z < north.y + 2.0 && z > CRYPT.y - 4.0 {
        // Svinger litt mens den klatrer, og ender ved krypten.
        let cx = |z: f32| {
            let t = ((north.y - z) / (north.y - CRYPT.y)).clamp(0.0, 1.0);
            north.x * (1.0 - t) + CRYPT.x * t + (t * PI).sin() * 6.0
        };
        let (u, v) = along(&cx, north.y, z);
        out.push(((x - cx(z)).abs(), u, -v));
    }
    out
}

/// Avstand fra et punkt til det bygde: byen innenfor muren, og borghaugen.
/// Der holder enga, trærne og dyrene seg unna.
pub fn town_dist(x: f32, z: f32) -> f32 {
    let (hx, hz, _, _) = terrain::HILL;
    let castle = ((x - hx).powi(2) + (z - hz).powi(2)).sqrt() - terrain::KEEP_FLAT - 2.0;
    terrain::town_edge(x, z).min(castle)
}

/// Stigen opp fra startrommet i kjelleren ligger i dungeon.rs; her er bare
/// hvor torget er, til testene.
pub fn square_center() -> Vec3 {
    terrain::at(Vec3::ZERO)
}

