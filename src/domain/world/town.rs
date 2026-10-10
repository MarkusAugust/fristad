//! Kjøpstaden Fristad rundt 1350, liten og samlet (docs/ny-by.md): ett
//! torg midt i byen avgrenset av husfasader, med Vertshuset, Smia, Apoteket
//! og Skriptoriet langs kantene, og Storgata fra sørporten over torget til
//! nordporten. Husene står i tette rekker langs gata og rundt torget, og
//! det tomme bak dem er sperret av med usynlige vegger, markert med gjerder
//! og vedstabler. Utenfor nordporten ligger kirkegården med stavkirka og
//! krypten, og utenfor muren går veiene ut mellom åsene.
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
pub const SQUARE: [Vec2; 4] = [Vec2::new(-12.0, -9.0), Vec2::new(-12.8, 10.0), Vec2::new(11.5, 11.8), Vec2::new(13.0, -10.6)];
/// Halv bredde på Storgata.
pub const MAIN: f32 = 3.2;
/// Krypten på kirkegården: midten av gravkapellet, med døra mot øst (mot
/// veien).
pub const CRYPT: Vec2 = Vec2::new(-9.0, -52.0);
/// Stavkirka på kirkegården, med portalen mot øst.
const CHURCH: Vec2 = Vec2::new(-22.0, -47.0);
/// Gorveks øvingsplass innenfor nordporten, øst for gata: midten og halve mål.
const YARD: (Vec2, Vec2) = (Vec2::new(9.0, -24.5), Vec2::new(4.0, 3.0));

/// Murverk med fundament under bakken (fot 80 cm ned), som får stå nede i
/// skråninger.
pub const FOUNDED: [&str; 9] = ["town_wall", "wall_tower", "gatehouse", "crypt", "stone_wall", "boulder_a", "boulder_b", "boulder_c", "signpost"];

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
    1.4 * (z * 0.05 + 0.5).sin() + 0.6 * (z * 0.13 + 2.0).sin()
}

/// Gatene i byen, regnet ut én gang.
pub fn streets() -> &'static [Street] {
    static STREETS: OnceLock<Vec<Street>> = OnceLock::new();
    STREETS.get_or_init(|| {
        let curve = |a: f32, b: f32, n: usize, f: &dyn Fn(f32) -> Vec2| (0..=n).map(|i| f(a + (b - a) * i as f32 / n as f32)).collect::<Vec<_>>();
        vec![
            // Storgata, fra utenfor sørporten til utenfor nordporten.
            Street { pts: curve(48.0, -46.0, 47, &|z| Vec2::new(main_x(z), z)), half: MAIN, cobbled: true },
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
    /// Avstanden fra et punkt til rektangelet (0 innenfor).
    fn dist(&self, p: Vec2) -> f32 {
        let (ax, az) = self.axes();
        let d = p - self.c;
        Vec2::new((d.dot(ax).abs() - self.h.x).max(0.0), (d.dot(az).abs() - self.h.y).max(0.0)).length()
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
            if square_dist(p) < 0.4 {
                return false;
            }
            if streets().iter().any(|st| street_dist(st, p).0 < st.half + 0.25) {
                return false;
            }
        }
        // Hele huset med takskjegget holder seg unna portene, tårnene og
        // øvingsplassen, og fra innsida av muren (som er nesten en meter tykk
        // innenfor omrisset).
        let f = full(r);
        if self.free.iter().any(|(c, rad)| f.dist(*c) < *rad) {
            return false;
        }
        if f.samples().iter().any(|p| terrain::town_edge(p.x, p.y) > -1.3) {
            return false;
        }
        !self.taken.iter().any(|t| full(t).overlaps(&f))
    }
}

/// Bygningene på torget: modell, kant av torget (0 vest, 1 sør, 2 øst,
/// 3 nord, mot klokka fra nordvest), hvor langs kanten (0..1), og grunnflaten
/// som settes av om modellen mangler.
const SPECIALS: [(&str, usize, f32, Vec2); 5] = [
    ("home", 1, 0.24, Vec2::new(8.0, 7.0)),
    ("apothecary", 3, 0.24, Vec2::new(8.0, 7.0)),
    ("tavern", 0, 0.5, Vec2::new(12.0, 9.0)),
    ("forge", 2, 0.5, Vec2::new(9.0, 8.0)),
    ("scriptorium", 3, 0.76, Vec2::new(10.0, 8.0)),
];

/// Husmodellene som står i rekkene.
/// Byhusene: bindingsverk og stein (houses.py).
const HOUSES: [&str; 15] = ["house_a", "house_b", "house_d", "house_f", "house_g", "house_h", "house_i", "house_k", "house_l", "house_m", "house_n", "house_p", "house_r", "house_s", "house_t"];
/// Laftede og bordkledde hus: våningshusene på bondegårdene utenfor muren.
const FARMHOUSES: [&str; 5] = ["house_c", "house_o", "house_j", "house_e", "house_q"];

/// Småbyggene i bakgårdene (tools/models/yard.py), med radius de trenger.
const SMALL: [(&str, f32); 6] = [("shed_a", 2.2), ("shed_b", 2.2), ("privy", 1.0), ("haystack", 1.7), ("garden", 2.6), ("coop", 1.5)];

/// Husets mål: midten av grunnflaten (lokal x, z) og halve mål.
fn footprint(b: &Builder, model: &str, fallback: Vec2) -> (Vec2, Vec2, f32) {
    match b.bounds(model) {
        Some((lo, hi)) => (Vec2::new((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5), (Vec2::new(hi.x - lo.x, hi.z - lo.z) * 0.5 - Vec2::splat(EAVES)).max(Vec2::splat(1.0)), hi.z - EAVES),
        None => (Vec2::ZERO, fallback * 0.5, fallback.y * 0.5),
    }
}

/// Takskjegget: grunnflaten (veggene) er så mye mindre enn hele modellen.
/// Veggene holder avstand til gata og torget; hele huset med taket og litt
/// luft holder avstand til nabohusene og muren.
const EAVES: f32 = 0.7;
const AIR: f32 = 0.3;

/// Hele huset med takskjegg og luft, rundt grunnflaten `r`.
fn full(r: &Rect) -> Rect {
    Rect { h: r.h + Vec2::splat(EAVES + AIR), ..*r }
}

/// Rektangelet til et hus med origo i `base` og yaw.
fn rect_at(base: Vec2, yaw: f32, mid: Vec2, half: Vec2) -> Rect {
    let r = Rect { c: base, h: half, yaw };
    let (ax, az) = r.axes();
    Rect { c: base + ax * mid.x + az * mid.y, ..r }
}

/// En rekke langs en linje: punktene, halv bredde på gata eller torget
/// foran, og bare utover fra torget (`only`).
type Row = (Vec<Vec2>, f32, Option<Vec2>);

/// Rekkene husene står i: langs kantene av torget og på begge sider av
/// Storgata.
fn rows() -> Vec<Row> {
    let mut rows: Vec<Row> = (0..4).map(|i| (vec![SQUARE[i], SQUARE[(i + 1) % 4]], 0.4, Some(square_out(i)))).collect();
    rows.push((streets()[0].pts.clone(), MAIN, None));
    rows
}

/// Et punkt `s` meter langs en linje, og retningen der.
fn along(pts: &[Vec2], s: f32) -> (Vec2, Vec2) {
    let mut acc = 0.0;
    for w in pts.windows(2) {
        let len = w[0].distance(w[1]);
        if s <= acc + len {
            let t = ((s - acc) / len.max(1e-4)).clamp(0.0, 1.0);
            return (w[0].lerp(w[1], t), (w[1] - w[0]).normalize_or_zero());
        }
        acc += len;
    }
    let n = pts.len();
    (pts[n - 1], (pts[n - 1] - pts[n - 2]).normalize_or_zero())
}

fn length(pts: &[Vec2]) -> f32 {
    pts.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// Setter ut husene. Gir stedet foran døra til hver bygning på torget,
/// og hvilken vei den vender.
fn houses(b: &mut Builder) -> Vec<(Vec2, f32)> {
    let mut plan = Plan { taken: vec![], free: vec![] };
    // Porthusene (14 m brede og 6 m dype), tårnene i murhjørnene og
    // øvingsplassen holdes fri.
    for south in [true, false] {
        plan.free.push((gate(south).0, 8.2));
    }
    for c in terrain::wall_corners() {
        plan.free.push((c, 4.9));
    }
    let (yc, yh) = YARD;
    plan.free.push((yc, yh.length()));
    // Bygningene på torget først, med døra mot torget.
    let mut doors = vec![];
    let mut specials = vec![];
    for (model, edge, t, size) in SPECIALS {
        let (a, bb) = (SQUARE[edge], SQUARE[(edge + 1) % 4]);
        let out = square_out(edge);
        let (mid, half, front) = footprint(b, model, size);
        let yaw = facing(-out);
        // Skyves langs kanten til hele bygningen (med takskjegg og
        // støttepilarer) står fri av Storgata.
        let place = |t: f32| {
            let m = a + (bb - a) * t;
            let base = m + out * (front + 0.8);
            (m, base, rect_at(base, yaw, mid, half))
        };
        let clear = |r: &Rect| Rect { h: r.h + Vec2::splat(EAVES), ..*r }.samples().iter().all(|p| street_dist(&streets()[0], *p).0 > MAIN + 0.3);
        let t = (0..40).map(|k| t + (k as f32 * 0.5).ceil() * if k % 2 == 1 { 0.01 } else { -0.01 }).find(|t| clear(&place(*t).2)).unwrap_or(t);
        let (m, base, r) = place(t);
        plan.taken.push(r);
        specials.push(r);
        building(b, model, base, yaw);
        log::info!("{model} ved ({:.1}, {:.1}), yaw {:.2}", base.x, base.y, yaw);
        doors.push((m - out * 1.6, yaw));
    }
    // Så rekkene langs torget og gata: på hvert sted prøves husmodellene i
    // tilfeldig rekkefølge, hver litt dreid for seg (ingen rette vinkler),
    // og den første som får plass med luft rundt blir stående.
    let mut rng = Rng(0x0f12_57ad);
    let mut placed = 0;
    for (pts, half, only) in rows() {
        let len = length(&pts);
        for side in [1.0f32, -1.0] {
            let mut s = rng.range(0.0, 1.0);
            while s < len {
                // De som ikke står noe sted ennå først, så alle husene er med.
                let mut order = HOUSES.to_vec();
                for i in (1..order.len()).rev() {
                    order.swap(i, (rng.next() * (i + 1) as f32) as usize % (i + 1));
                }
                // De største først: i bakgårdene er det trangere enn langs gata.
                let area = |m: &str| footprint(b, m, Vec2::new(7.0, 8.0)).1.element_product();
                order.sort_by(|x, y| b.placements.iter().any(|pl| pl.model == *x).cmp(&b.placements.iter().any(|pl| pl.model == *y)).then(area(y).total_cmp(&area(x))));
                let mut done = None;
                'try_: for model in order {
                    let (mid, half_m, front) = footprint(b, model, Vec2::new(7.0, 8.0));
                    let (p, t) = along(&pts, (s + half_m.x + EAVES).min(len));
                    let n = Vec2::new(-t.y, t.x) * side;
                    if only.is_some_and(|o| n.dot(o) < 0.0) {
                        break;
                    }
                    for back in [0.4f32, 1.0, 1.8] {
                        let base = p + n * (half + back + front);
                        let yaw = facing(-n) + rng.range(-0.18, 0.18);
                        let r = rect_at(base, yaw, mid, half_m);
                        if plan.fits(&r) {
                            plan.taken.push(r);
                            building(b, model, base, yaw);
                            done = Some((half_m.x + EAVES) * 2.0 + AIR);
                            break 'try_;
                        }
                    }
                }
                match done {
                    Some(w) => {
                        placed += 1;
                        s += w + rng.range(0.2, 1.2);
                    }
                    None => s += 0.6,
                }
            }
        }
    }
    // Bakgårdene: hus bak rekkene, ut mot muren, så byen er tett sett
    // ovenfra og fra gata. Du kommer ikke dit (avsperringen står foran).
    let mut back = 0;
    for _ in 0..8000 {
        let a = rng.range(0.0, TAU);
        let r = rng.range(8.0, terrain::wall_radius(a) - 3.0);
        let p = Vec2::new(a.cos(), a.sin()) * r;
        let (d, t) = street_dist(&streets()[0], p);
        if d < MAIN + 2.0 || square_dist(p) < 2.0 {
            continue;
        }
        let n = Vec2::new(-t.y, t.x);
        let toward = if n.dot(Vec2::new(main_x(p.y), p.y) - p) >= 0.0 { n } else { -n };
        let yaw = facing(toward) + if rng.chance(0.3) { FRAC_PI_2 } else { 0.0 } + rng.range(-0.3, 0.3);
        let unused: Vec<&'static str> = HOUSES.iter().copied().filter(|m| !b.placements.iter().any(|pl| pl.model == *m)).collect();
        let model = if unused.is_empty() { rng.pick(&HOUSES) } else { unused[(rng.next() * unused.len() as f32) as usize % unused.len()] };
        let (mid, half_m, _) = footprint(b, model, Vec2::new(7.0, 8.0));
        let rr = rect_at(p, yaw, mid, half_m + Vec2::splat(0.05));
        if plan.fits(&rr) {
            plan.taken.push(rr);
            building(b, model, p, yaw);
            back += 1;
        }
    }
    log::info!("Byen: {placed} hus i rekkene, {back} i bakgårdene");
    // Småbyggene: skur, vedskjul, utedo, høystakker, kjøkkenhager og
    // hønsehus i hullene mellom husene, dreid hver for seg.
    let mut small = 0;
    for _ in 0..6000 {
        let a = rng.range(0.0, TAU);
        let r = rng.range(6.0, terrain::wall_radius(a) - 2.0);
        let p = Vec2::new(a.cos(), a.sin()) * r;
        if street_dist(&streets()[0], p).0 < MAIN + 3.0 || square_dist(p) < 3.0 {
            continue;
        }
        let (model, _) = SMALL[(rng.next() * SMALL.len() as f32) as usize % SMALL.len()];
        let (mid, half_m, _) = footprint(b, model, Vec2::splat(2.0));
        let yaw = rng.range(0.0, TAU);
        let rr = rect_at(p, yaw, mid, half_m);
        if plan.fits(&rr) {
            plan.taken.push(rr);
            building(b, model, p, yaw);
            small += 1;
        }
    }
    log::info!("Byen: {small} småbygg i bakgårdene");
    enclose(b, &plan, &specials, &mut rng);
    street_life(b, &mut plan, &mut rng);
    doors
}

/// Det tomme i byen sperres av: en usynlig vegg langs hver husrekke, litt
/// bak fasadene, så du holder deg på torget og i gata, og gjerder og
/// vedstabler der det er glipe mellom husene, så sperringen synes. Ved
/// bygningene på torget (som du går inn i) og der gata munner ut, står det
/// ingen vegg.
fn enclose(b: &mut Builder, plan: &Plan, specials: &[Rect], rng: &mut Rng) {
    let (mut walls, mut fences) = (0, 0);
    let inside_any = |p: Vec2, rects: &[Rect], grow: f32| {
        rects.iter().any(|r| {
            let (ax, az) = r.axes();
            let d = p - r.c;
            d.dot(ax).abs() <= r.h.x + grow && d.dot(az).abs() <= r.h.y + grow
        })
    };
    for (pts, half, only) in rows() {
        let len = length(&pts);
        for side in [1.0f32, -1.0] {
            let step = 1.0;
            let mut s = 0.0;
            while s < len {
                let (p, t) = along(&pts, s + step * 0.5);
                let n = Vec2::new(-t.y, t.x) * side;
                s += step;
                if only.is_some_and(|o| n.dot(o) < 0.0) {
                    break;
                }
                let line = p + n * (half + 1.1);
                let free = terrain::town_edge(line.x, line.y) > -1.2
                    || square_dist(line) < 0.6
                    || street_dist(&streets()[0], line).0 < MAIN + 0.4
                    || inside_any(line, specials, 0.3)
                    || plan.free.iter().any(|(c, r)| c.distance(line) < *r * 0.6);
                if free {
                    continue;
                }
                let yaw = facing(n);
                // Den usynlige veggen: en meter lang, høy nok til at ingen
                // hopper over.
                b.solid_box(b.on_ground(Vec3::new(line.x, 1.5, line.y)), Vec3::new(step + 0.1, 3.0, 0.3), yaw);
                walls += 1;
                // Gjerdet der det ikke står et hus (hver andre meter, gjerdet
                // er to meter langt).
                let fence = p + n * (half + 0.9);
                // Ikke foran en husvegg (døra står der): da er huset selv
                // sperringen.
                if (s as i32) % 2 == 0 && !inside_any(fence, &plan.taken, 0.6) && !inside_any(fence + n * 2.2, &plan.taken, 0.3) && !inside_any(fence + t * 1.0, &plan.taken, 0.6) && !inside_any(fence - t * 1.0, &plan.taken, 0.6) {
                    let base = b.on_ground(Vec3::new(fence.x, 0.0, fence.y));
                    let model = match rng.next() {
                        x if x < 0.75 => "wattle_fence",
                        x if x < 0.9 => "woodpile",
                        _ => "crate",
                    };
                    b.place(model, at(base, yaw + rng.range(-0.04, 0.04)));
                    fences += 1;
                }
            }
        }
    }
    // Øvingsplassen: gjerde på de tre sidene som ikke vender mot gata.
    let (yc, yh) = YARD;
    for (a, bb) in [(Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0)), (Vec2::new(1.0, -1.0), Vec2::new(1.0, 1.0)), (Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0))] {
        let (p0, p1) = (yc + a * yh, yc + bb * yh);
        let n = (p0.distance(p1) / 2.0).round().max(1.0) as usize;
        let yaw = facing(Vec2::new(-(p1 - p0).y, (p1 - p0).x));
        for k in 0..n {
            let c = p0.lerp(p1, (k as f32 + 0.5) / n as f32);
            let base = b.on_ground(Vec3::new(c.x, 0.0, c.y));
            b.place("wattle_fence", at(base, yaw));
            b.solid_box(base + Vec3::Y * 1.5, Vec3::new(2.1, 3.0, 0.3), yaw);
            fences += 1;
        }
    }
    log::info!("Avsperring: {walls} m usynlig vegg, {fences} gjerder og vedstabler");
}

/// Bondegårdene utenfor muren: midten av tunet. Flate steder på åsene,
/// unna veiene, kirkegården og beitene, med god avstand imellom.
pub fn farms() -> &'static [Vec2] {
    static FARMS: OnceLock<Vec<Vec2>> = OnceLock::new();
    FARMS.get_or_init(|| {
        let mut out: Vec<Vec2> = vec![];
        let herds = crate::domain::herd::HERDS;
        for k in 0..400 {
            if out.len() >= FARMHOUSES.len() {
                break;
            }
            let a = k as f32 * 2.399 + 0.7;
            let r = 66.0 + (k as f32 * 0.37).sin().abs() * 55.0;
            let p = Vec2::new(a.cos(), a.sin()) * r;
            let flat = (0..8).map(|i| {
                let q = p + Vec2::new((i as f32 * 0.785).cos(), (i as f32 * 0.785).sin()) * 8.0;
                terrain::height(q.x, q.y)
            });
            let (lo, hi) = flat.fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)));
            let clear = road_dist(p.x, p.y) > 16.0
                && terrain::town_edge(p.x, p.y) > 28.0
                && herds.iter().all(|(_, c, rad, _)| c.distance(p) > rad + FARM_R + 12.0)
                && out.iter().all(|q| q.distance(p) > 40.0)
                && {
                    let (cx, cz, hx, hz) = terrain::YARD_FLAT;
                    ((p.x - cx).abs() - hx).max((p.y - cz).abs() - hz) > 18.0
                };
            if clear && hi - lo < 2.2 {
                out.push(p);
            }
        }
        out
    })
}

/// Det høyeste punktet på bakken innenfor `r` fra `p` (i en skråning står
/// ting der, og sokkelen eller stolpene tar resten).
fn highest(b: &Builder, p: Vec2, r: f32) -> f32 {
    [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (0.0, 0.0)].iter().map(|(sx, sz)| b.ground(p.x + sx * r, p.y + sz * r)).fold(f32::MIN, f32::max)
}

/// Tunet på en gård: radius på jorda rundt husene.
const FARM_R: f32 = 13.0;

/// Gårdene: våningshuset midt på tunet, skur, vedskjul, høystakk,
/// hønsehus, kjøkkenhage og utedo rundt, og et gjerde av flettverk med
/// åpning mot veien. Alt dreid hver for seg og med luft imellom.
fn farmsteads(b: &mut Builder) {
    let mut rng = Rng(0x0fa7_4d11);
    for (i, c) in farms().iter().enumerate() {
        let model = FARMHOUSES[i % FARMHOUSES.len()];
        // Mot stien ut til veien, litt skjevt.
        let lane = &farm_lanes()[i];
        let toward = (lane[1] - lane[0]).normalize_or(-c.normalize());
        let yaw = facing(toward) + rng.range(-0.4, 0.4);
        let mut taken: Vec<Rect> = vec![];
        let put = |b: &mut Builder, model: &'static str, p: Vec2, yaw: f32, taken: &mut Vec<Rect>| -> bool {
            let (mid, half, _) = footprint(b, model, Vec2::splat(3.0));
            let r = rect_at(p, yaw, mid, half);
            if taken.iter().any(|t| full(t).overlaps(&full(&r))) {
                return false;
            }
            // På det høyeste punktet under foten (sokkelen tar resten).
            let (ax, az) = r.axes();
            let top = [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (0.0, 0.0)].iter().map(|(sx, sz)| {
                let q = r.c + ax * r.h.x * sx * 0.5 + az * r.h.y * sz * 0.5;
                b.ground(q.x, q.y)
            });
            let y = top.fold(f32::MIN, f32::max).max(highest(b, p, 0.7)) - 0.03;
            taken.push(r);
            let t = at(Vec3::new(p.x, y, p.y), yaw);
            if b.place(model, t) {
                place_colliders(b, model, t, yaw);
            }
            true
        };
        put(b, model, *c, yaw, &mut taken);
        let mut extras = vec!["haystack", "shed_b", "coop", "garden", "privy", "shed_a", "haystack"];
        extras.truncate(4 + i % 3);
        for m in extras {
            for _ in 0..40 {
                let a = rng.range(0.0, TAU);
                let d = Vec2::new(a.cos(), a.sin());
                // Ikke i tunet foran døra, der stien kommer inn.
                if d.dot(toward) > 0.8 {
                    continue;
                }
                // Hele uthuset innenfor gjerdet, med en halv meter å gå på.
                let reach = footprint(b, m, Vec2::splat(3.0)).1.length() + EAVES;
                let p = *c + d * rng.range(6.0, (FARM_R * 0.94 - 0.6 - reach).max(6.0));
                if put(b, m, p, rng.range(0.0, TAU), &mut taken) {
                    break;
                }
            }
        }
        // Gjerdet rundt tunet, med en grind der stien går ut. Gjerdene er
        // 2 m lange og står på tvers av radien (lokal x langs gjerdet), så
        // mange som går rundt.
        let rad = |a: f32| (FARM_R + 0.5) * (1.0 + 0.06 * (a * 3.0 + i as f32).sin());
        let n = ((FARM_R + 0.5) * 1.06 * TAU / 1.9).ceil() as usize;
        for k in 0..n {
            let (a0, a1) = (k as f32 / n as f32 * TAU, (k + 1) as f32 / n as f32 * TAU);
            let (p0, p1) = (*c + Vec2::new(a0.sin(), a0.cos()) * rad(a0), *c + Vec2::new(a1.sin(), a1.cos()) * rad(a1));
            let q = (p0 + p1) * 0.5;
            if (q - *c).normalize().dot(toward) > 0.975 {
                continue;
            }
            let along = p1 - p0;
            let fy = (-along.y).atan2(along.x);
            let base = Vec3::new(q.x, highest(b, q, 0.7) - 0.05, q.y);
            let t = at(base, fy) * Affine3A::from_scale(Vec3::new((along.length() + 0.08) / 2.0, 1.0, 1.0));
            if b.place("wattle_fence", t) {
                b.solid_box(base + Vec3::Y * 0.5, Vec3::new(along.length(), 1.0, 0.15), fy);
            }
        }
        log::info!("Gård {i} ved ({:.0}, {:.0}) med {model}, sti fra ({:.0}, {:.0}) til ({:.0}, {:.0})", c.x, c.y, lane[0].x, lane[0].y, lane[12].x, lane[12].y);
    }
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

/// Brønnene: én på torget og én ved veien utenfor nordporten.
fn street_life(b: &mut Builder, plan: &mut Plan, _rng: &mut Rng) {
    // Den ute står øst for veien, et stykke fra porthuset.
    let (g, out) = gate(false);
    let spots = [(Vec2::new(5.5, 4.0), 0.3), (g + out * 9.5 + Vec2::new(-out.y, out.x) * 5.5, FRAC_PI_2)];
    for (i, (p, yaw)) in spots.into_iter().enumerate() {
        plan.taken.push(Rect { c: p, h: Vec2::splat(1.5), yaw });
        // I en skråning står brønnen på det høyeste punktet under foten.
        let top = [(-0.6f32, -0.6f32), (0.6, -0.6), (-0.6, 0.6), (0.6, 0.6)].iter().map(|(dx, dz)| b.ground(p.x + dx, p.y + dz)).fold(f32::MIN, f32::max);
        let base = Vec3::new(p.x, top, p.y);
        if b.place("well", at(base, yaw)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 0.45, radius: 1.0, half_height: 0.5 });
            for sx in [-0.75f32, 0.75] {
                b.solid(Shape::Cylinder { center: at(base, yaw).transform_point3(Vec3::new(sx, 1.1, 0.0)), radius: 0.12, half_height: 1.1 });
            }
            // Vinda, tauet og bøtta flyttes av økta når du heiser.
            for model in WELL_PARTS {
                b.reserve(model, 1);
            }
            b.target(Use::Well(i), at(base, yaw), Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.1, 0.5, 1.1));
        }
        log::info!("Brønn ved ({:.1}, {:.1}), yaw {:.2}", p.x, p.y, yaw);
    }
}

pub fn build(b: &mut Builder) -> Town {
    let doors = houses(b);
    farmsteads(b);
    wall(b);
    square(b, &doors);
    church(b);
    yard(b);
    let crypt_exit = crypt(b);
    signposts(b);
    people(b, &doors);
    let (g, out) = gate(true);
    let spawn = Vec3::new(main_x(g.y - 8.0), 0.0, g.y - 8.0);
    portal(b, spawn);
    wanderers(b, &doors);
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
    /// Kjelleren under huset: midten og halve mål (x, z) i modellens rom.
    /// Bakken tas bort der (`Builder::hole`).
    #[serde(default)]
    cellar: Option<([f32; 3], [f32; 2])>,
    /// Kjellergulvet: midten (y er overkanten) og halve mål (x, z).
    #[serde(default)]
    cellar_floor: Option<([f32; 3], [f32; 2])>,
    /// Spillerens hus: midten av senga (det du sikter på) og halve mål.
    #[serde(default)]
    bed: Option<([f32; 3], [f32; 3])>,
    /// Spillerens hus: rammene på galleriveggen, midten og yaw (rammen
    /// vender ut mot lokal +Z dreid med yaw), i `craft::GALLERY`-rekkefølge.
    #[serde(default)]
    gallery: Vec<([f32; 3], f32)>,
}

/// Husene med en egen innsidemodell, som bare tegnes når du er nær
/// (rendereren kjenner dem på navnet `…_interior`).
const INTERIORS: [(&str, &str); 6] = [("tavern", "tavern_interior"), ("home", "home_interior"), ("forge", "forge_interior"), ("apothecary", "apothecary_interior"), ("scriptorium", "scriptorium_interior"), ("stave_church", "stave_church_interior")];

const INSIDE: [(&str, &str); 32] = [
    ("apothecary", include_str!("../../../assets/models/town/apothecary_inside.json")),
    ("home", include_str!("../../../assets/models/town/home_inside.json")),
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
    ("house_g", include_str!("../../../assets/models/town/house_g_inside.json")),
    ("house_h", include_str!("../../../assets/models/town/house_h_inside.json")),
    ("house_i", include_str!("../../../assets/models/town/house_i_inside.json")),
    ("house_j", include_str!("../../../assets/models/town/house_j_inside.json")),
    ("house_k", include_str!("../../../assets/models/town/house_k_inside.json")),
    ("house_l", include_str!("../../../assets/models/town/house_l_inside.json")),
    ("house_m", include_str!("../../../assets/models/town/house_m_inside.json")),
    ("house_n", include_str!("../../../assets/models/town/house_n_inside.json")),
    ("house_o", include_str!("../../../assets/models/town/house_o_inside.json")),
    ("house_p", include_str!("../../../assets/models/town/house_p_inside.json")),
    ("house_q", include_str!("../../../assets/models/town/house_q_inside.json")),
    ("house_r", include_str!("../../../assets/models/town/house_r_inside.json")),
    ("house_s", include_str!("../../../assets/models/town/house_s_inside.json")),
    ("house_t", include_str!("../../../assets/models/town/house_t_inside.json")),
    ("shed_a", include_str!("../../../assets/models/town/shed_a_inside.json")),
    ("shed_b", include_str!("../../../assets/models/town/shed_b_inside.json")),
    ("privy", include_str!("../../../assets/models/town/privy_inside.json")),
    ("haystack", include_str!("../../../assets/models/town/haystack_inside.json")),
    ("garden", include_str!("../../../assets/models/town/garden_inside.json")),
    ("coop", include_str!("../../../assets/models/town/coop_inside.json")),
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
    place_colliders(b, model, t, yaw);
}

/// Det et hus har med seg: innsidemodellen, kjelleren, lysene, røyken og
/// kolliderne (fra `<navn>_inside.json`, ellers en boks rundt modellen).
fn place_colliders(b: &mut Builder, model: &'static str, t: Affine3A, yaw: f32) {
    let base = Vec3::from(t.translation);
    if let Some((_, interior)) = INTERIORS.iter().find(|(m, _)| *m == model) {
        // Innsida står på husets gulv, ikke på bakken.
        b.place_mounted(interior, t);
    }
    let inside = inside(model);
    if let Some((c, h)) = inside.and_then(|i| i.cellar) {
        b.hole(t.transform_point3(Vec3::from(c)), Vec2::from(h), yaw);
    }
    if let Some(inside) = inside {
        if let Some((c, h)) = inside.bed {
            b.target(Use::Sleep, t, Vec3::from(c), Vec3::from(h));
        }
        for (i, (c, y)) in inside.gallery.iter().enumerate().take(crate::domain::craft::GALLERY.len()) {
            let pose = t * Affine3A::from_rotation_translation(Quat::from_rotation_y(*y), Vec3::from(*c));
            b.target(Use::Gallery(i), pose, Vec3::new(0.0, 0.0, 0.03), Vec3::new(0.3, 0.3, 0.05));
        }
    }
    if let Some((c, h)) = inside.and_then(|i| i.cellar_floor) {
        let c = t.transform_point3(Vec3::from(c));
        b.floor(c, Vec2::from(h), yaw, c.y);
    }
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

/// Bymuren: rette stykker fra hjørne til hjørne langs det ujevne omrisset
/// (`terrain::WALL_CORNERS`), delt i biter på rundt 6 m, et tårn i hvert
/// hjørne, og porthus der Storgata går gjennom.
fn wall(b: &mut Builder) {
    let seg_len = 6.0;
    let corners = terrain::wall_corners();
    let mut pts: Vec<Vec2> = vec![];
    for (i, &p) in corners.iter().enumerate() {
        let q = corners[(i + 1) % corners.len()];
        let n = (p.distance(q) / seg_len).round().max(1.0) as usize;
        for k in 0..n {
            pts.push(p.lerp(q, k as f32 / n as f32));
        }
    }
    pts.push(corners[0]);
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
    // Tårnene: i hvert hjørne, der muren knekker.
    for p in corners {
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
                b.solid_box(side, Vec3::new(4.0, 10.0, 5.4), yaw);
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
    // Portalen (som står der når du trenger den) holdes fri.
    placed.push((PORTAL_AT, 1.6));
    let c = settle(Vec2::new(-6.5, -5.5), 1.2, &mut placed).unwrap_or(center);
    let cross = b.on_ground(Vec3::new(c.x, 0.0, c.y));
    if b.place("market_cross", at(cross, 0.3)) {
        b.solid(Shape::Cylinder { center: cross + Vec3::Y * 1.5, radius: 0.9, half_height: 1.5 });
    }
    let stalls = [(-5.5, 8.5, 0.1), (8.5, 7.0, -0.2), (8.5, -6.0, PI - 0.15)];
    for (k, (x, z, yaw)) in stalls.into_iter().enumerate() {
        let Some(p) = settle(Vec2::new(x, z), 1.5, &mut placed) else { continue };
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        let model = if k % 2 == 0 { "market_stall" } else { "market_stall_b" };
        if b.place(model, at(base, yaw)) {
            b.solid_box(base + Vec3::Y * 0.5, Vec3::new(2.6, 1.0, 1.4), yaw);
        }
    }
    let props = [("barrel", -10.5, 8.8, 0.0), ("barrel", 10.6, 9.4, 1.0), ("crate", 10.0, 9.8, 0.3), ("crate", 10.8, -8.8, 1.1), ("barrel", -10.6, -8.0, 2.0)];
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
    // Står det et hus, en vedstabel eller et gjerde der, flyttes lykta inn
    // mot midten av torget eller langs gata til den står fritt.
    let mut spots: Vec<Vec2> = SQUARE.iter().map(|c| clear_spot(b, *c * 0.86, -c.normalize(), 0.5)).collect();
    for (z, side) in [(-24.0f32, 1.0f32), (20.0, -1.0), (27.0, -1.0)] {
        let p = Vec2::new(main_x(z) + side * (MAIN + 0.5), z);
        spots.push(clear_spot(b, p, Vec2::Y, 0.5));
    }
    for p in spots {
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        if b.place("lantern_post", at(base, 0.0)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 1.4, radius: 0.15, half_height: 1.4 });
            b.lamp_light(base + Vec3::Y * 2.68, Vec3::new(1.0, 0.68, 0.38), 2.4, 11.0);
        }
    }
}

/// Står noe i kroppshøyde innenfor `r` fra punktet (et hus, en mur, et
/// gjerde, en brønn)?
fn blocked(b: &Builder, p: Vec2, r: f32) -> bool {
    b.colliders.iter().any(|c| match c {
        Shape::Cuboid { center, size, yaw } if size.y > 0.6 && (center.y - b.ground(center.x, center.z)).abs() < 8.0 => {
            let d = p - Vec2::new(center.x, center.z);
            let (sn, cs) = yaw.sin_cos();
            let l = Vec2::new(d.x * cs - d.y * sn, d.x * sn + d.y * cs).abs();
            l.x < size.x * 0.5 + r && l.y < size.z * 0.5 + r
        }
        Shape::Cylinder { center, radius, .. } => Vec2::new(center.x, center.z).distance(p) < radius + r,
        _ => false,
    })
}

/// Nærmeste frie sted fra `p` langs `dir` (fram og tilbake, opptil 8 m).
fn clear_spot(b: &Builder, p: Vec2, dir: Vec2, r: f32) -> Vec2 {
    (0..33).map(|k| p + dir * (k as f32 * 0.5).ceil() * 0.5 * if k % 2 == 1 { 1.0 } else { -1.0 }).find(|q| !blocked(b, *q, r)).unwrap_or(p)
}

/// Hvem som står hvor, og hva de sier.
const SMITH: &[&str] = &[
    "Buttonsmeden, til tjeneste. Jeg smir det du bærer opp fra krypten.",
    "Et sverd trenger en klinge og et grep: en divider og en button. Resten er håndlag.",
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
    "Det kommer rare ting opp fra dypet om natta. Buttons med tenner.",
];

/// Stavkirka på kirkegården nordvest i byen, med steingjerde og gravsteiner.
fn church(b: &mut Builder) {
    // Stavkirka med portalen mot øst, mot veien.
    building(b, "stave_church", CHURCH, FRAC_PI_2);
    // Kirkegården: lav steinmur rundt, med åpning mot veien i øst.
    let (cx, cz, hx, hz) = terrain::YARD_FLAT;
    let (w, e, n, so) = (cx - hx + 1.0, cx + hx - 1.0, cz - hz + 1.0, cz + hz - 1.0);
    let wall = |b: &mut Builder, a: Vec2, c: Vec2| super::nature::stone_wall(b, Vec3::new(a.x, 0.0, a.y), Vec3::new(c.x, 0.0, c.y), 0.0);
    // Sørøsthjørnet skrås av, så muren går klar av porthuset ved
    // nordporten (14 m bredt og 3 m ut fra bymuren).
    let (g, _) = gate(false);
    let cut = Vec2::new((g.x - 9.0).min(e - 3.0), so);
    let turn = Vec2::new(e, (g.y - 6.5).min(so));
    wall(b, Vec2::new(w, n), Vec2::new(e, n));
    wall(b, Vec2::new(w, so), cut);
    wall(b, cut, turn);
    wall(b, Vec2::new(w, n), Vec2::new(w, so));
    wall(b, Vec2::new(e, n), Vec2::new(e, cz - 3.0));
    wall(b, Vec2::new(e, cz + 3.0), turn);
    // Gravene i ujevne rader over hele kirkegården, utenom kirka, krypten
    // og stien fra åpningen i muren til kirkedøra.
    let mut rng = Rng(0x0c4e_7e12);
    for row in 0..8 {
        for k in 0..8 {
            if rng.chance(0.25) {
                continue;
            }
            let p = Vec2::new(-27.0 + row as f32 * 2.4 + rng.range(-0.3, 0.3), -58.0 + k as f32 * 2.2 + rng.range(-0.3, 0.3));
            let in_church = (p.x - CHURCH.x).abs() < 8.5 && (p.y - CHURCH.y).abs() < 6.0;
            let at_crypt = p.distance(CRYPT) < 5.5 || (p.x > CRYPT.x - 2.0 && (p.y - CRYPT.y).abs() < 4.0);
            let on_path = (p.y - CHURCH.y).abs() < 2.0 && p.x > CHURCH.x;
            if in_church || at_crypt || on_path || p.x > e - 1.5 || p.y > so - 1.5 {
                continue;
            }
            let g = b.on_ground(Vec3::new(p.x, 0.0, p.y));
            let yaw = FRAC_PI_2 + rng.range(-0.15, 0.15);
            if b.place("gravestone", at(g, yaw)) {
                b.solid_box(g + Vec3::Y * 0.45, Vec3::new(0.6, 0.9, 0.2), yaw);
            }
        }
    }
}

/// Gorveks øvingsplass nordøst i byen: treningsdukker og et våpenstativ.
fn yard(b: &mut Builder) {
    let (c, _) = YARD;
    for (x, z, yaw) in [(-2.0, -2.0, 0.4), (1.5, -3.0, -0.3), (3.0, 1.0, 0.1)] {
        let base = b.on_ground(Vec3::new(c.x + x, 0.0, c.y + z));
        if b.place("training_dummy", at(base, yaw)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 0.9, radius: 0.35, half_height: 0.9 });
        }
    }
    let rack = b.on_ground(Vec3::new(c.x + 4.0, 0.0, c.y + 3.5));
    if b.place("weapon_rack", at(rack, -FRAC_PI_2)) {
        b.solid_box(rack + Vec3::Y * 0.8, Vec3::new(0.6, 1.6, 2.0), 0.0);
    }
}

/// Veiskiltene til krypten: ved torget, innenfor nordporten og der stien
/// begynner å klatre opp borghaugen. Pila (lokal +X) peker videre langs
/// veien.
fn signposts(b: &mut Builder) {
    let (north, _) = gate(false);
    let street = |z: f32| Vec2::new(main_x(z), z);
    // (der skiltet står, punktet det peker mot)
    let signs = [
        (Vec2::new(-3.2, -8.0), north),
        (street(north.y - 6.0) + Vec2::new(MAIN + 0.6, 0.0), CRYPT),
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

/// Krypten på kirkegården: et lite gravkapell med døra mot øst, og trappa
/// ned bak døra. Gir stedet du kommer opp til fra kjelleren.
fn crypt(b: &mut Builder) -> (Vec3, f32) {
    let yaw = FRAC_PI_2;
    let crypt = b.on_ground(Vec3::new(CRYPT.x, 0.0, CRYPT.y));
    let t = at(crypt, yaw);
    if b.place("crypt", t) {
        b.solid_box(t.transform_point3(Vec3::new(0.0, 2.5, -2.0)), Vec3::new(6.0, 5.0, 6.0), yaw);
    }
    // Døra: det du sikter på, i forkant av kapellet.
    b.target(Use::Descend, t, Vec3::new(0.0, 1.2, 1.05), Vec3::new(0.75, 1.2, 0.2));
    b.fire_light(t.transform_point3(Vec3::new(1.3, 2.4, 1.6)), Vec3::new(1.0, 0.6, 0.3), 2.0, 8.0);
    b.fire_light(t.transform_point3(Vec3::new(-1.3, 2.4, 1.6)), Vec3::new(1.0, 0.6, 0.3), 2.0, 8.0);
    // Ut av døra, vendt bort fra den (mot øst).
    (t.transform_point3(Vec3::new(0.0, 0.0, 3.2)), -FRAC_PI_2)
}

/// Innbyggerne foran bygningene sine på torget, og Gorvek på øvingsplassen.
fn people(b: &mut Builder, doors: &[(Vec2, f32)]) {
    let at_door = |i: usize| doors.get(i).copied().unwrap_or((Vec2::ZERO, 0.0));
    // Rekkefølgen i SPECIALS: hjemmet, apotek, vertshus, smie, skriptorium.
    // Figuren: en egen modell (fra tools/models/people.py) om den finnes.
    let cast: [(&'static str, &'static str, (Vec2, f32), Vec3, &'static [&'static str]); 5] = [
        ("Buttonsmeden", "npc_smith", at_door(3), Vec3::new(0.22, 0.1, 0.05), SMITH),
        ("Skriptoriet", "npc_clerk", at_door(4), Vec3::new(0.06, 0.12, 0.42), CLERK),
        ("Apotekeren", "npc_apothecary", at_door(1), Vec3::new(0.06, 0.28, 0.1), APOTHECARY),
        ("Vertinnen", "npc_innkeeper", at_door(2), Vec3::new(0.45, 0.12, 0.1), INNKEEPER),
        ("Gorvek", "npc_gorvek", (YARD.0 + Vec2::new(-4.0, 0.0), -FRAC_PI_2), Vec3::new(0.3, 0.1, 0.04), GORVEK),
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
            vec![p, YARD.0 + Vec2::new(-0.5, -0.5), YARD.0 + Vec2::new(2.5, -2.0), YARD.0 + Vec2::new(0.5, 2.5)]
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
        // Starter midt på ruta, ute i gata (ikke i veien ved en brønn), og
        // ikke der en annen allerede står.
        let half = route.len() / 2;
        route.rotate_left(half);
        let taken: Vec<Vec3> = b.npcs.iter().map(|n| n.pos).collect();
        if let Some(k) = (0..route.len()).find(|&k| taken.iter().all(|q| q.distance(route[k].0) > 1.5)) {
            route.rotate_left(k);
        }
        let (pos, _) = route[0];
        // Hver har sin figur (kvinnene på torget deler kona med skaut og
        // bøtte); den felles `npc` bare om modellen mangler.
        let own = match name {
            "Tjenestejenta" | "Fiskerkona" | "Vaskekona" => "npc_woman",
            "Bonden" => "npc_farmer",
            "Gutten" => "npc_boy",
            "Vaktmannen" => "npc_guard",
            "Kremmeren" => "npc_merchant",
            "Pilegrimen" => "npc_pilgrim",
            _ => crate::domain::npcs::NPC_MODEL,
        };
        let model = if own != crate::domain::npcs::NPC_MODEL && b.has_model(own) {
            b.reserve(own, 1);
            own
        } else {
            crate::domain::npcs::NPC_MODEL
        };
        log::debug!("{name} ({model}) starter ved {:.1}, {:.1}", pos.x, pos.z);
        b.npcs.push(NpcSpec { name, model, pos, yaw: 0.0, tint, lines, route });
        let i = b.npcs.len() - 1;
        b.target(Use::Talk(i), Affine3A::from_translation(pos), Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.3, 0.9, 0.3));
    }
    log::info!("Byen: {} innbyggere går ærender", b.npcs.len() - base);
}

/// Portalen ved sørporten (`tools/models/portal.py`): står ikke der før du
/// har gått hjem fra kjelleren med H (eller dødd nede). Da stiger
/// steinringen opp av bakken på den åpne plassen vest for Storgata, mellom
/// muren og husene, med forsiden mot gata, og virvelen åpner seg. Økta
/// setter ringen, runene, virvlene og splintene (`Session::portal_poses`)
/// og flytter kolliderne inn (`PORTAL_SOLIDS`).
/// Der portalen står: på torget, vest for Storgata.
const PORTAL_AT: Vec2 = Vec2::new(-7.5, 5.0);
pub const PORTAL_RING: &str = "portal_ring";
pub const PORTAL_PARTS: [&str; 3] = ["portal_runes", "portal_swirl", "portal_shard"];
/// Midten av ringen over plattformen, og ringens indre radius.
pub const PORTAL_CENTER: f32 = 1.62;
pub const PORTAL_RADIUS: f32 = 1.12;
/// Kolliderne til ringen i dens eget rom (midte, hele mål): plattformen,
/// sidene og toppen; åpningen er fri, så virvelen kan siktes på.
pub const PORTAL_SOLIDS: [(Vec3, Vec3); 4] = [
    (Vec3::new(0.0, 0.06, 0.0), Vec3::new(4.0, 0.2, 2.0)),
    (Vec3::new(-1.3, 1.4, 0.0), Vec3::new(0.5, 2.8, 0.55)),
    (Vec3::new(1.3, 1.4, 0.0), Vec3::new(0.5, 2.8, 0.55)),
    (Vec3::new(0.0, 3.0, 0.0), Vec3::new(2.4, 0.45, 0.55)),
];

fn portal(b: &mut Builder, spawn: Vec3) {
    if !b.has_model(PORTAL_RING) {
        log::warn!("Modellen {PORTAL_RING} mangler, ingen portal");
        return;
    }
    let _ = spawn;
    let at = PORTAL_AT;
    let base = b.on_ground(Vec3::new(at.x, 0.0, at.y));
    // Forsiden (+z i modellen) mot gata i øst.
    let pose = Affine3A::from_rotation_translation(Quat::from_rotation_y(FRAC_PI_2), base);
    log::debug!("Portalen står ved {:.1}, {:.1}", base.x, base.z);
    b.target(Use::Portal, pose, Vec3::new(0.0, PORTAL_CENTER, 0.0), Vec3::new(PORTAL_RADIUS * 0.85, PORTAL_RADIUS * 0.85, 0.12));
    b.reserve(PORTAL_RING, 1);
    let [runes, swirl, shard] = PORTAL_PARTS;
    b.reserve(runes, 1);
    b.reserve(swirl, 2);
    b.reserve(shard, PORTAL_SHARDS);
}

/// Splintene som svever rundt den åpne portalen.
pub const PORTAL_SHARDS: usize = 8;

/// Avstanden fra et punkt til nærmeste gate (midtlinja).
fn street_gap(p: Vec2) -> f32 {
    streets().iter().map(|s| street_dist(s, p).0).fold(f32::MAX, f32::min)
}

const TOWNSFOLK: &[&str] = &[
    "God dag. Vannet i brønnen er kaldt i dag, kaldt som krypten.",
    "Har du vært nede under borgen? Bror min gikk ned i fjor. Han snakker bare om buttons nå.",
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
    // Gårdsplassene er tråkket jord (eget lag uten hjulspor), i byen og på
    // tunene på bondegårdene.
    let farm = farms().iter().map(|c| c.distance(p) - FARM_R).fold(f32::MAX, f32::min);
    let yards = (inside * soft(0.0, (0.3 - n2) * 12.0, 3.0) * 0.85).max(soft(0.0, farm + (n1 - 0.5) * 3.0, 3.0) * 0.9);
    let dirt = roads.max(yards);
    let earth = (yards / (roads + yards).max(1e-4)).clamp(0.0, 1.0);
    // Berg der det er bratt.
    let slope = 1.0 - terrain::normal(x, z).y;
    let rock = soft(0.0, 0.3 - slope + (n1 - 0.5) * 0.05, 0.05);
    let forest = 0.0f32;
    // Fordel: brostein over jord over berg over skogbunn, gress resten.
    let cobble = cobble.clamp(0.0, 1.0);
    let dirt = (dirt * (1.0 - cobble)).clamp(0.0, 1.0);
    let rock = (rock * (1.0 - cobble - dirt)).clamp(0.0, 1.0);
    let forest = (forest * (1.0 - cobble - dirt - rock)).clamp(0.0, 1.0);
    // Tørt gress på ryggene og i åkrene.
    let dry = (terrain::noise(x * 0.05 + 2.0, z * 0.05) * 1.3 - 0.35).clamp(0.0, 1.0) * 0.6;
    // Søle i kantene av jordveiene og i gatene i byen.
    let wet = (dirt * (1.0 - dirt) * 4.0 * soft(0.0, 0.55 - n2, 0.2)).clamp(0.0, 1.0);
    Ground { cobble, dirt, rock, forest, dry, wet, road: coords, earth }
}

/// Avstand fra et punkt til veiene og stiene utenfor muren.
pub fn path_dist(x: f32, z: f32) -> f32 {
    paths(x, z).into_iter().map(|(d, _, _)| d).fold(f32::MAX, f32::min)
}

/// Avstand fra et punkt til de to veiene (uten stiene til gårdene).
fn road_dist(x: f32, z: f32) -> f32 {
    roads(x, z).into_iter().map(|(d, _, _)| d).fold(f32::MAX, f32::min)
}

/// Midtlinja til veien ut av porten (sør eller nord) for en z.
fn road_x(south: bool, z: f32) -> f32 {
    let (from, _) = gate(south);
    let sign = if south { 1.0 } else { -1.0 };
    let k = (((z - from.y) * sign) / 50.0).clamp(0.0, 1.0);
    let n = terrain::noise(z * 0.02, if south { 9.0 } else { 3.0 });
    from.x * (1.0 - k) + (n - 0.5) * 40.0 * k
}

/// Stiene fra veiene inn til gårdene: en svak bue fra der stien tar av
/// fra veien til grinda i gjerdet, i samme rekkefølge som `farms()`.
/// Første punkt er inne på tunet, siste ute på veien.
pub fn farm_lanes() -> &'static [Vec<Vec2>] {
    static LANES: OnceLock<Vec<Vec<Vec2>>> = OnceLock::new();
    LANES.get_or_init(|| {
        let (south, _) = gate(true);
        let (north, _) = gate(false);
        // Punkter langs begge veiene.
        let mut road: Vec<Vec2> = vec![];
        for k in 0..90 {
            let z = south.y + 4.0 + k as f32 * 2.0;
            road.push(Vec2::new(road_x(true, z), z));
            let z = north.y - 4.0 - k as f32 * 2.0;
            road.push(Vec2::new(road_x(false, z), z));
        }
        let (cx, cz, hx, hz) = terrain::YARD_FLAT;
        let farms = farms();
        // Nærmest veien først; de andre kan ta av fra en sti som alt går
        // ut, så to stier ikke går side om side.
        let mut order: Vec<usize> = (0..farms.len()).collect();
        order.sort_by(|a, b| road_dist(farms[*a].x, farms[*a].y).total_cmp(&road_dist(farms[*b].x, farms[*b].y)));
        let mut lanes: Vec<Vec<Vec2>> = vec![vec![]; farms.len()];
        for i in order {
            let c = &farms[i];
            let lane = {
                // Fri vei: unna byen, kirkegården og de andre gårdene.
                let clear = |q: Vec2| {
                    terrain::town_edge(q.x, q.y) > 14.0 && ((q.x - cx).abs() - hx).max((q.y - cz).abs() - hz) > 6.0 && farms.iter().enumerate().all(|(j, f)| j == i || f.distance(q) > FARM_R + 3.0)
                };
                // Buen: midten skjøvet litt til siden, bort fra byen.
                let curve = |end: Vec2| -> Vec<Vec2> {
                    let dir = (end - *c).normalize_or(Vec2::X);
                    let start = *c + dir * (FARM_R - 4.0);
                    let mut side = Vec2::new(-dir.y, dir.x) * (start.distance(end) * 0.08);
                    if side.dot((start + end) * 0.5) < 0.0 {
                        side = -side;
                    }
                    (0..=12)
                        .map(|k| {
                            let t = k as f32 / 12.0;
                            start.lerp(end, t) + side * (4.0 * t * (1.0 - t))
                        })
                        .collect()
                };
                // Tar stien av fra en annen sti, er det på skrå, ikke langs den.
                let mut options = road.clone();
                for l in &lanes {
                    for k in 3..l.len().saturating_sub(1) {
                        let along = (l[k + 1] - l[k - 1]).normalize_or_zero();
                        if (l[k] - *c).normalize_or_zero().dot(along).abs() < 0.87 {
                            options.push(l[k]);
                        }
                    }
                }
                options.sort_by(|a, b| a.distance(*c).total_cmp(&b.distance(*c)));
                let fine = |lane: &Vec<Vec2>| lane.windows(2).all(|w| (0..4).all(|k| clear(w[0].lerp(w[1], k as f32 / 4.0))));
                options.into_iter().map(curve).find(fine).unwrap_or_else(|| curve(*c + Vec2::X))
            };
            lanes[i] = lane;
        }
        lanes
    })
}

/// Veiene og stiene utenfor muren som gjelder for punktet: (avstand til
/// midtlinja, fortegnet avstand på tvers, meter langs veien). Stiene til
/// gårdene er smalere enn veiene, så avstanden regnes litt lenger.
fn paths(x: f32, z: f32) -> Vec<(f32, f32, f32)> {
    let mut out = roads(x, z);
    let p = Vec2::new(x, z);
    for lane in farm_lanes() {
        let (lo, hi) = lane.iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), q| (lo.min(*q), hi.max(*q)));
        if p.x < lo.x - 6.0 || p.y < lo.y - 6.0 || p.x > hi.x + 6.0 || p.y > hi.y + 6.0 {
            continue;
        }
        let st = Street { pts: lane.clone(), half: 0.0, cobbled: false };
        let (d, across, along) = street_coords(&st, p);
        out.push((d + 0.6, across, along));
    }
    out
}

/// De to veiene utenfor muren: sørover fra sørporten og nordover fra
/// nordporten forbi kirkegården. Begge går omtrent nord–sør, så midtlinja
/// er en x for hver z.
fn roads(x: f32, z: f32) -> Vec<(f32, f32, f32)> {
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
    // Sørover fra sørporten og nordover fra nordporten forbi kirkegården,
    // svingende ut mellom åsene.
    for (from, sign) in [(south, 1.0f32), (north, -1.0)] {
        if (z - from.y) * sign < -2.0 {
            continue;
        }
        let cx = move |z: f32| road_x(sign > 0.0, z);
        let (u, v) = along(&cx, from.y, z);
        out.push(((x - cx(z)).abs(), u, v * sign));
    }
    out
}

/// Avstand fra et punkt til det bygde: byen innenfor muren, og kirkegården.
/// Der holder enga, trærne og dyrene seg unna.
pub fn town_dist(x: f32, z: f32) -> f32 {
    let (cx, cz, hx, hz) = terrain::YARD_FLAT;
    let yard = ((x - cx).abs() - hx).max((z - cz).abs() - hz);
    let farm = farms().iter().map(|c| c.distance(Vec2::new(x, z)) - FARM_R - 1.0).fold(f32::MAX, f32::min);
    terrain::town_edge(x, z).min(yard).min(farm)
}

/// Hvor torget er, til testene.
pub fn square_center() -> Vec3 {
    terrain::at(Vec3::ZERO)
}

