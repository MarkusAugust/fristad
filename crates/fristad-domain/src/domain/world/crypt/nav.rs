//! Navigasjonsnettet i et kryptnivå: konvekse polygoner på gulv, trapper og
//! ramper, sydd sammen med felles kanter (fra Blender, `crypt_NN.json`).
//! Fiendene finner veien med A* over polygonene (kostnaden måles mellom
//! midtpunktene på kantene de går gjennom) og glatter den med trakten
//! (funnel / «simple stupid funnel»), så de går rett mellom hjørnene.
//! Porter, dører og falske vegger stenger polygonene sine (`Poly.gate`) til
//! de åpnes. Stigene og heisene er lenker utenom nettet (`Hop`, «off-mesh
//! links») mellom to polygoner i hver sin etasje: A* går over dem som over
//! en kant, og veien får lenkens punkter (fiendene klatrer og kjører langs
//! dem, `mob_walk.rs`).

use glam::{Vec2, Vec3};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// Rutene i oppslaget (meter).
const BUCKET: f32 = 4.0;

#[derive(Clone, Debug)]
pub struct Link {
    /// Polygonen på andre siden.
    pub to: u32,
    /// Kanten i denne polygonens omløp: `a` til høyre og `b` til venstre
    /// når du går ut gjennom den.
    pub a: Vec3,
    pub b: Vec3,
}

#[derive(Clone, Debug)]
pub struct Poly {
    /// Hjørnene, mot klokka i (x, z) som matematisk plan.
    pub pts: Vec<Vec3>,
    /// Rommet (indeks i nivåets rom), om noe.
    pub room: Option<usize>,
    /// Bryteren (port, dør, falsk vegg) som stenger polygonen, om noen.
    pub gate: Option<usize>,
    pub center: Vec3,
    pub min: Vec2,
    pub max: Vec2,
    pub links: Vec<Link>,
}

/// En lenke utenom nettet: stigen eller heisen fra polygonen `from` til
/// `to`, begge veier. `pts` er veien langs den (den første i `from`, den
/// siste i `to`).
#[derive(Clone, Debug)]
pub struct Hop {
    pub from: u32,
    pub to: u32,
    pub pts: Vec<Vec3>,
    pub kind: HopKind,
    /// Bryteren som stenger den: fallemmen over stigen, bremsen på heisen.
    pub gate: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HopKind {
    Ladder,
    /// Heis nummer n (`Crypt.lifts`).
    Lift(usize),
}

impl Hop {
    /// Punktene i den retningen du går (`forward`: fra `from` til `to`).
    pub fn oriented(&self, forward: bool) -> Vec<Vec3> {
        let mut p = self.pts.clone();
        if !forward {
            p.reverse();
        }
        p
    }

    /// Kostnaden i A* (meter gange): stigen er treg, og heisen må man vente på.
    fn cost(&self) -> f32 {
        let len: f32 = self.pts.windows(2).map(|w| w[0].distance(w[1])).sum();
        match self.kind {
            HopKind::Ladder => 2.0 + len * 2.5,
            HopKind::Lift(_) => 4.0 + len * 1.5,
        }
    }
}

/// Et steg i korridoren: gjennom kanten `links[k]` i polygonen, eller over
/// lenken (og retningen).
#[derive(Clone, Copy, Debug)]
enum Step {
    Edge(u32, usize),
    Hop(usize, bool),
}

#[derive(Clone, Debug, Default)]
pub struct Nav {
    pub polys: Vec<Poly>,
    /// Stigene og heisene (`add_hop`).
    pub hops: Vec<Hop>,
    buckets: HashMap<(i32, i32), Vec<u32>>,
}

fn flat(v: Vec3) -> Vec2 {
    Vec2::new(v.x, v.z)
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

impl Nav {
    /// Bygg nettet av hjørner og polygoner (indekser), med rom og bryter per
    /// polygon. Hjørner som ligger oppå hverandre sveises, og kanter som går
    /// langs hverandre uten felles hjørner kobles også.
    pub fn new(verts: &[Vec3], polys: &[Vec<u32>], rooms: &[Option<usize>], gates: &[Option<usize>]) -> Self {
        // Sveis hjørnene (1 cm).
        let key = |v: Vec3| ((v.x * 100.0).round() as i64, (v.y * 100.0).round() as i64, (v.z * 100.0).round() as i64);
        let mut weld: HashMap<(i64, i64, i64), u32> = HashMap::new();
        let ids: Vec<u32> = verts.iter().enumerate().map(|(i, v)| *weld.entry(key(*v)).or_insert(i as u32)).collect();
        let mut out = Nav::default();
        let mut idx_polys: Vec<Vec<u32>> = vec![];
        for (k, p) in polys.iter().enumerate() {
            let mut vi: Vec<u32> = p.iter().filter(|&&i| (i as usize) < verts.len()).map(|&i| ids[i as usize]).collect();
            vi.dedup();
            if vi.len() >= 2 && vi.first() == vi.last() {
                vi.pop();
            }
            if vi.len() < 3 {
                continue;
            }
            let mut pts: Vec<Vec3> = vi.iter().map(|&i| verts[i as usize]).collect();
            let area: f32 = (0..pts.len()).map(|i| cross(flat(pts[i]), flat(pts[(i + 1) % pts.len()]))).sum();
            if area.abs() < 1e-4 {
                continue;
            }
            if area < 0.0 {
                pts.reverse();
                vi.reverse();
            }
            let center = pts.iter().copied().sum::<Vec3>() / pts.len() as f32;
            let min = pts.iter().fold(Vec2::splat(f32::MAX), |m, p| m.min(flat(*p)));
            let max = pts.iter().fold(Vec2::splat(f32::MIN), |m, p| m.max(flat(*p)));
            out.polys.push(Poly { pts, room: rooms.get(k).copied().flatten(), gate: gates.get(k).copied().flatten(), center, min, max, links: vec![] });
            idx_polys.push(vi);
        }
        // Felles kanter.
        let mut edges: HashMap<(u32, u32), Vec<(u32, usize)>> = HashMap::new();
        for (p, vi) in idx_polys.iter().enumerate() {
            for e in 0..vi.len() {
                let (a, b) = (vi[e], vi[(e + 1) % vi.len()]);
                edges.entry((a.min(b), a.max(b))).or_default().push((p as u32, e));
            }
        }
        let mut linked = vec![vec![false; 0]; out.polys.len()];
        for (p, vi) in idx_polys.iter().enumerate() {
            linked[p] = vec![false; vi.len()];
        }
        for list in edges.values() {
            for &(p, e) in list {
                for &(q, _) in list {
                    if p != q {
                        let n = out.polys[p as usize].pts.len();
                        let (a, b) = (out.polys[p as usize].pts[e], out.polys[p as usize].pts[(e + 1) % n]);
                        out.polys[p as usize].links.push(Link { to: q, a, b });
                        linked[p as usize][e] = true;
                    }
                }
            }
        }
        out.index();
        // Kanter som går langs hverandre (T-skjøter, eller hjørner som ikke
        // er felles): koble der de overlapper.
        for p in 0..out.polys.len() {
            for e in 0..out.polys[p].pts.len() {
                if linked[p][e] {
                    continue;
                }
                let n = out.polys[p].pts.len();
                let (a, b) = (out.polys[p].pts[e], out.polys[p].pts[(e + 1) % n]);
                let (lo, hi) = (flat(a).min(flat(b)) - Vec2::splat(0.05), flat(a).max(flat(b)) + Vec2::splat(0.05));
                let mut found = vec![];
                for q in out.near(lo, hi) {
                    if q as usize == p {
                        continue;
                    }
                    let qp = &out.polys[q as usize];
                    for f in 0..qp.pts.len() {
                        let (c, d) = (qp.pts[f], qp.pts[(f + 1) % qp.pts.len()]);
                        if let Some((s, t)) = overlap(a, b, c, d) {
                            found.push(Link { to: q, a: s, b: t });
                        }
                    }
                }
                out.polys[p].links.extend(found);
            }
        }
        out
    }

    fn index(&mut self) {
        self.buckets.clear();
        for (i, p) in self.polys.iter().enumerate() {
            let (a, b) = (bucket(p.min), bucket(p.max));
            for z in a.1..=b.1 {
                for x in a.0..=b.0 {
                    self.buckets.entry((x, z)).or_default().push(i as u32);
                }
            }
        }
    }

    /// Legg til en lenke langs `pts` (enden i hver etasje; endene kobles
    /// til nettet innen `reach` meter). Gir om den ble lagt til.
    pub fn add_hop(&mut self, kind: HopKind, pts: &[Vec3], gate: Option<usize>, reach: f32) -> bool {
        let all = |_: &Poly| true;
        let (Some(first), Some(last)) = (pts.first(), pts.last()) else { return false };
        let (Some((a, pa)), Some((b, pb))) = (self.nearest(*first, reach, &all), self.nearest(*last, reach, &all)) else { return false };
        if a == b {
            return false;
        }
        let mut full: Vec<Vec3> = vec![pa];
        for p in pts.iter().copied().chain([pb]) {
            if full.last().is_none_or(|q| q.distance(p) > 0.05) {
                full.push(p);
            }
        }
        self.hops.push(Hop { from: a, to: b, pts: full, kind, gate });
        true
    }

    /// Lenkene ut fra polygonen `p`: (lenken, om den går forover).
    fn hops_from(&self, p: u32) -> impl Iterator<Item = (usize, bool)> + '_ {
        self.hops.iter().enumerate().filter_map(move |(h, hop)| if hop.from == p { Some((h, true)) } else if hop.to == p { Some((h, false)) } else { None })
    }

    /// Polygonene som kan berøre rektangelet.
    fn near(&self, lo: Vec2, hi: Vec2) -> Vec<u32> {
        let (a, b) = (bucket(lo), bucket(hi));
        let mut out = vec![];
        for z in a.1..=b.1 {
            for x in a.0..=b.0 {
                if let Some(l) = self.buckets.get(&(x, z)) {
                    for &i in l {
                        let p = &self.polys[i as usize];
                        if p.max.x >= lo.x && p.min.x <= hi.x && p.max.y >= lo.y && p.min.y <= hi.y && !out.contains(&i) {
                            out.push(i);
                        }
                    }
                }
            }
        }
        out
    }

    /// Om punktet (x, z) er inne i polygonen.
    fn inside(&self, i: u32, q: Vec2, tol: f32) -> bool {
        let p = &self.polys[i as usize];
        let n = p.pts.len();
        (0..n).all(|k| {
            let (a, b) = (flat(p.pts[k]), flat(p.pts[(k + 1) % n]));
            let e = b - a;
            cross(e, q - a) >= -tol * e.length()
        })
    }

    /// Gulvhøyden i polygonen under punktet (x, z).
    pub fn height_in(&self, i: u32, q: Vec2) -> f32 {
        let p = &self.polys[i as usize];
        let v0 = p.pts[0];
        let mut best = (f32::MAX, v0.y);
        for k in 1..p.pts.len() - 1 {
            let (v1, v2) = (p.pts[k], p.pts[k + 1]);
            let (a, b, c) = (flat(v0), flat(v1), flat(v2));
            let d = cross(b - a, c - a);
            if d.abs() < 1e-8 {
                continue;
            }
            let w1 = cross(q - a, c - a) / d;
            let w2 = cross(b - a, q - a) / d;
            let w0 = 1.0 - w1 - w2;
            let y = v0.y * w0 + v1.y * w1 + v2.y * w2;
            // Utenfor trekanten: så langt utenfor.
            let out = (-w0).max(-w1).max(-w2).max(0.0);
            if out < best.0 {
                best = (out, y);
            }
        }
        best.1
    }

    /// Polygonen punktet står i: inne i (x, z), og gulvet nærmest under
    /// føttene (eller litt over, opptil `above`).
    pub fn locate(&self, p: Vec3, above: f32) -> Option<u32> {
        let q = flat(p);
        let mut best: Option<(f32, u32)> = None;
        for i in self.near(q, q) {
            if !self.inside(i, q, 0.01) {
                continue;
            }
            let h = self.height_in(i, q);
            let score = if h <= p.y + above { p.y - h } else { 100.0 + h - p.y };
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, i));
            }
        }
        best.filter(|(s, _)| *s < 100.0).map(|(_, i)| i)
    }

    /// Nærmeste punkt på nettet innen `reach` meter (flatt), med gulvhøyden,
    /// og polygonen det er i. Høydeforskjellen teller dobbelt.
    pub fn nearest(&self, p: Vec3, reach: f32, open: &dyn Fn(&Poly) -> bool) -> Option<(u32, Vec3)> {
        let q = flat(p);
        let mut best: Option<(f32, u32, Vec3)> = None;
        for i in self.near(q - Vec2::splat(reach), q + Vec2::splat(reach)) {
            let poly = &self.polys[i as usize];
            if !open(poly) {
                continue;
            }
            let c = if self.inside(i, q, 0.0) {
                q
            } else {
                let n = poly.pts.len();
                (0..n)
                    .map(|k| closest_on_segment(q, flat(poly.pts[k]), flat(poly.pts[(k + 1) % n])))
                    .min_by(|a, b| a.distance_squared(q).total_cmp(&b.distance_squared(q)))
                    .unwrap_or(q)
            };
            let d = c.distance(q);
            if d > reach {
                continue;
            }
            let h = self.height_in(i, c);
            // Litt over føttene teller som avstand (ikke som en fordel: på en
            // trapp vant ellers trinnet tre trinn opp over det du står på).
            let dy = if h <= p.y + 0.6 { (p.y - h).abs() } else { (h - p.y) * 3.0 };
            if dy > 3.0 {
                continue;
            }
            let score = d + dy * 2.0;
            if best.is_none_or(|(s, _, _)| score < s) {
                best = Some((score, i, Vec3::new(c.x, h, c.y)));
            }
        }
        best.map(|(_, i, c)| (i, c))
    }

    /// Veien fra `from` til `to`: A* over polygonene, så trakten. Gir
    /// hjørnene etter startpunktet (det siste er målet), eller `None` om det
    /// ikke finnes noen åpen vei. Alle lenkene er åpne.
    pub fn path(&self, from: Vec3, to: Vec3, open: &dyn Fn(&Poly) -> bool) -> Option<Vec<Vec3>> {
        self.path_with(from, to, open, &|_, _| true)
    }

    /// Som `path`, med `hop_open` som sier om lenken kan brukes i den
    /// retningen (heisen står i den andre enden, fallemmen er stengt). Over
    /// en lenke får veien lenkens punkter.
    pub fn path_with(&self, from: Vec3, to: Vec3, open: &dyn Fn(&Poly) -> bool, hop_open: &dyn Fn(&Hop, bool) -> bool) -> Option<Vec<Vec3>> {
        let (a, start) = self.nearest(from, 1.5, open)?;
        let (b, goal) = self.nearest(to, 2.5, open)?;
        // Står fienden på kanten av nettet (innerkanten av en vindeltrapp),
        // flyttes startpunktet 3 cm inn i polygonen, ellers ligger det på
        // linje med kantene, og trakten kan gi et hjørne bak den.
        let c = self.polys[a as usize].center;
        let d = (c - start).with_y(0.0);
        let start = if d.length() > 1e-4 { start + d / d.length() * d.length().min(0.03) } else { start };
        let corridor = self.corridor(a, start, b, goal, open, hop_open)?;
        // Trakten for hvert strekk på nettet, og lenkens punkter imellom.
        let (mut out, mut here, mut leg) = (vec![], start, vec![]);
        for step in corridor {
            match step {
                Step::Edge(p, k) => leg.push((p, k)),
                Step::Hop(h, fwd) => {
                    let pts = self.hops[h].oriented(fwd);
                    out.extend(self.funnel(here, pts[0], &leg));
                    out.extend(pts[1..].iter().copied());
                    here = *pts.last().unwrap_or(&here);
                    leg.clear();
                }
            }
        }
        out.extend(self.funnel(here, goal, &leg));
        Some(out)
    }

    /// Polygonene på korteste vei og kantene og lenkene imellom.
    fn corridor(&self, a: u32, start: Vec3, b: u32, goal: Vec3, open: &dyn Fn(&Poly) -> bool, hop_open: &dyn Fn(&Hop, bool) -> bool) -> Option<Vec<Step>> {
        if a == b {
            return Some(vec![]);
        }
        let n = self.polys.len();
        let mut g = vec![f32::MAX; n];
        let mut at = vec![start; n];
        let mut prev: Vec<Option<(u32, Step)>> = vec![None; n];
        let mut heap = BinaryHeap::new();
        g[a as usize] = 0.0;
        let key = |f: f32| (f * 1000.0) as u64;
        heap.push(Reverse((key(start.distance(goal)), a)));
        while let Some(Reverse((_, p))) = heap.pop() {
            if p == b {
                break;
            }
            let here = at[p as usize];
            let gp = g[p as usize];
            let edges = self.polys[p as usize].links.iter().enumerate().map(|(k, l)| (l.to, (l.a + l.b) * 0.5, here.distance((l.a + l.b) * 0.5), Step::Edge(p, k)));
            let hops = self.hops_from(p).filter(|&(h, fwd)| hop_open(&self.hops[h], fwd)).map(|(h, fwd)| {
                let hop = &self.hops[h];
                let pts = hop.oriented(fwd);
                (if fwd { hop.to } else { hop.from }, pts[pts.len() - 1], here.distance(pts[0]) + hop.cost(), Step::Hop(h, fwd))
            });
            for (to, mid, step_cost, step) in edges.chain(hops).collect::<Vec<_>>() {
                let q = to as usize;
                if !open(&self.polys[q]) {
                    continue;
                }
                let cost = gp + step_cost + if to == b { mid.distance(goal) } else { 0.0 };
                if cost + 1e-4 < g[q] {
                    g[q] = cost;
                    at[q] = mid;
                    prev[q] = Some((p, step));
                    heap.push(Reverse((key(cost + mid.distance(goal)), to)));
                }
            }
        }
        prev[b as usize]?;
        let mut out = vec![];
        let mut cur = b;
        while let Some((p, step)) = prev[cur as usize] {
            out.push(step);
            cur = p;
            if cur == a {
                break;
            }
        }
        out.reverse();
        Some(out)
    }

    /// Trakten: strammer veien gjennom kantene (venstre og høyre side) til
    /// rette strekk mellom hjørnene.
    fn funnel(&self, start: Vec3, goal: Vec3, corridor: &[(u32, usize)]) -> Vec<Vec3> {
        // Kantene som (venstre, høyre), med start og mål som egne kanter.
        let mut portals: Vec<(Vec3, Vec3)> = vec![(start, start)];
        for &(p, k) in corridor {
            let l = &self.polys[p as usize].links[k];
            portals.push((l.b, l.a));
        }
        portals.push((goal, goal));
        let area = |a: Vec3, b: Vec3, c: Vec3| cross(flat(b) - flat(a), flat(c) - flat(a));
        let same = |a: Vec3, b: Vec3| flat(a).distance_squared(flat(b)) < 1e-6;
        let mut out = vec![];
        let (mut apex, mut left, mut right) = (start, start, start);
        let (mut left_i, mut right_i) = (0usize, 0usize);
        let mut apex_i;
        let mut i = 1;
        while i < portals.len() {
            let (l, r) = portals[i];
            // Høyre side: strammes innover, eller går over venstre.
            if area(apex, right, r) >= 0.0 {
                if same(apex, right) || area(apex, left, r) < 0.0 {
                    right = r;
                    right_i = i;
                } else {
                    out.push(left);
                    apex = left;
                    apex_i = left_i;
                    left = apex;
                    right = apex;
                    left_i = apex_i;
                    right_i = apex_i;
                    i = apex_i + 1;
                    continue;
                }
            }
            if area(apex, left, l) <= 0.0 {
                if same(apex, left) || area(apex, right, l) > 0.0 {
                    left = l;
                    left_i = i;
                } else {
                    out.push(right);
                    apex = right;
                    apex_i = right_i;
                    left = apex;
                    right = apex;
                    left_i = apex_i;
                    right_i = apex_i;
                    i = apex_i + 1;
                    continue;
                }
            }
            i += 1;
        }
        if out.last().is_none_or(|p| !same(*p, goal)) {
            out.push(goal);
        }
        // Høyden langs veien: gulvet under hvert hjørne.
        out.into_iter().filter(|p| !same(*p, start) || corridor.is_empty()).collect()
    }

    /// Polygonene som kan nås fra `a` når `open` sier hvilke som er åpne,
    /// over kantene og alle lenkene (stigene og heisene).
    pub fn reachable(&self, a: u32, open: &dyn Fn(&Poly) -> bool) -> Vec<bool> {
        let mut seen = vec![false; self.polys.len()];
        let mut stack = vec![a];
        seen[a as usize] = true;
        while let Some(p) = stack.pop() {
            let hops = self.hops_from(p).map(|(h, fwd)| if fwd { self.hops[h].to } else { self.hops[h].from });
            for to in self.polys[p as usize].links.iter().map(|l| l.to).chain(hops).collect::<Vec<_>>() {
                if !seen[to as usize] && open(&self.polys[to as usize]) {
                    seen[to as usize] = true;
                    stack.push(to);
                }
            }
        }
        seen
    }
}

fn bucket(p: Vec2) -> (i32, i32) {
    ((p.x / BUCKET).floor() as i32, (p.y / BUCKET).floor() as i32)
}

fn closest_on_segment(q: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let ab = b - a;
    let t = ((q - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
    a + ab * t
}

/// Der to kanter går langs hverandre (innen 3 cm, og i samme høyde innen
/// 30 cm): den felles biten, i retningen til den første.
fn overlap(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Option<(Vec3, Vec3)> {
    let (fa, fb, fc, fd) = (flat(a), flat(b), flat(c), flat(d));
    let ab = fb - fa;
    let len = ab.length();
    if len < 1e-3 {
        return None;
    }
    let dir = ab / len;
    let off = |p: Vec2| cross(dir, p - fa).abs();
    if off(fc) > 0.03 || off(fd) > 0.03 {
        return None;
    }
    let (tc, td) = ((fc - fa).dot(dir), (fd - fa).dot(dir));
    let (lo, hi) = (tc.min(td).max(0.0), tc.max(td).min(len));
    if hi - lo < 0.2 {
        return None;
    }
    let at = |t: f32| a.lerp(b, t / len);
    // Samme høyde der de møtes.
    let other = |t: f32| {
        let s = ((fa + dir * t - fc).dot(fd - fc) / (fd - fc).length_squared().max(1e-9)).clamp(0.0, 1.0);
        c.lerp(d, s).y
    };
    if (at(lo).y - other(lo)).abs() > 0.3 || (at(hi).y - other(hi)).abs() > 0.3 {
        return None;
    }
    Some((at(lo), at(hi)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// En L-formet gang av tre firkanter: veien rundt hjørnet skal gå rett
    /// til det indre hjørnet, og så rett til målet.
    #[test]
    fn trakten_gar_rundt_hjornet() {
        // (0..2, 0..2), (2..4, 0..2), (2..4, 2..6) i (x, z).
        let v = |x: f32, z: f32| Vec3::new(x, 0.0, z);
        let verts = vec![v(0.0, 0.0), v(2.0, 0.0), v(4.0, 0.0), v(4.0, 2.0), v(2.0, 2.0), v(0.0, 2.0), v(4.0, 6.0), v(2.0, 6.0)];
        let polys = vec![vec![0, 1, 4, 5], vec![1, 2, 3, 4], vec![4, 3, 6, 7]];
        let nav = Nav::new(&verts, &polys, &[None; 3], &[None; 3]);
        let all = |_: &Poly| true;
        let path = nav.path(v(0.5, 1.0), v(3.0, 5.5), &all).expect("vei");
        assert_eq!(path.len(), 2, "{path:?}");
        assert!(path[0].distance(v(2.0, 2.0)) < 1e-3, "hjørnet: {path:?}");
        // Og andre veien.
        let back = nav.path(v(3.0, 5.5), v(0.5, 1.0), &all).expect("vei");
        assert_eq!(back.len(), 2, "{back:?}");
        // Stengt midt i: ingen vei.
        let shut = |p: &Poly| p.center.x < 2.0 || p.center.z > 2.0;
        assert!(nav.path(v(0.5, 1.0), v(3.0, 5.5), &shut).is_none());
        // Rett fram i samme rom: bare målet.
        assert_eq!(nav.path(v(0.5, 0.5), v(1.5, 1.5), &all).unwrap().len(), 1);
    }

    /// En rampe: høyden følger skråningen.
    #[test]
    fn hoyden_folger_rampa() {
        let verts = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 4.0), Vec3::new(0.0, 1.0, 4.0)];
        let nav = Nav::new(&verts, &[vec![0, 1, 2, 3]], &[None], &[None]);
        let i = nav.locate(Vec3::new(1.0, 2.0, 2.0), 0.5).expect("på rampa");
        assert!((nav.height_in(i, Vec2::new(1.0, 2.0)) - 0.5).abs() < 1e-4);
    }
}
