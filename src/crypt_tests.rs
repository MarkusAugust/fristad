//! Krypten med motorens egne øyne og Rapier: ingen usynlige vegger, og
//! kapselen kommer seg mellom nøkkelstedene i hvert nivå uten å bli stående
//! fast (eieren 18. oktober: «man henger seg opp i noen usynlige vegger»
//! i rotunden i kloakken).

use crate::adapters::rapier::RapierPhysics;
use crate::application::Prepared;
use crate::application::Session;
use crate::domain::world::crypt::Crypt;
use crate::ports::driving::Input;
use crate::ports::Physics;
use crate::tests::{doc_world, session, session_in, world};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

/// Trekantene som synes i krypten, i ruter på én meter (x, z).
struct Visible {
    tris: Vec<[Vec3; 3]>,
    cells: HashMap<(i32, i32), Vec<u32>>,
}

const CELL: f32 = 1.0;

impl Visible {
    /// Alt som er satt ut under byen (bitene, byggesettet) og det økta
    /// poserer per bilde (portene, vannet, de falske veggene …).
    fn new(p: &Prepared, game: &Session) -> Self {
        let mut tris = vec![];
        let mut add = |name: &str, t: &glam::Affine3A| {
            let Some(m) = p.models.iter().find(|m| m.name == name) else { return };
            let v = &m.mesh.vertices;
            for f in m.mesh.indices.chunks_exact(3) {
                let tri = [0, 1, 2].map(|k| t.transform_point3(Vec3::from(v[f[k] as usize].pos)));
                if tri.iter().all(|q| q.y < crate::domain::world::crypt::TOP) {
                    tris.push(tri);
                }
            }
        };
        for pl in &p.world.placements {
            for t in &pl.transforms {
                if t.translation.y < crate::domain::world::crypt::TOP + 5.0 {
                    add(pl.model, t);
                }
            }
        }
        for i in game.frame(1.6).instances {
            if i.transform.translation.y < crate::domain::world::crypt::TOP && i.skin.is_none() {
                add(i.model, &i.transform);
            }
        }
        let mut cells: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for (n, t) in tris.iter().enumerate() {
            let lo = t[0].min(t[1]).min(t[2]);
            let hi = t[0].max(t[1]).max(t[2]);
            for x in (lo.x / CELL).floor() as i32..=(hi.x / CELL).floor() as i32 {
                for z in (lo.z / CELL).floor() as i32..=(hi.z / CELL).floor() as i32 {
                    cells.entry((x, z)).or_default().push(n as u32);
                }
            }
        }
        Visible { tris, cells }
    }

    /// Avstanden langs strålen til den nærmeste synlige flaten (begge sider).
    fn ray(&self, o: Vec3, d: Vec3, max: f32) -> Option<f32> {
        let mut seen = std::collections::HashSet::new();
        let mut best: Option<f32> = None;
        let steps = (max / (CELL * 0.5)).ceil() as i32;
        for s in 0..=steps {
            let q = o + d * (s as f32 * CELL * 0.5).min(max);
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let key = ((q.x / CELL).floor() as i32 + dx, (q.z / CELL).floor() as i32 + dz);
                    for &n in self.cells.get(&key).map(|v| v.as_slice()).unwrap_or(&[]) {
                        if !seen.insert(n) {
                            continue;
                        }
                        if let Some(t) = hit(o, d, &self.tris[n as usize]) {
                            if t <= max && best.is_none_or(|b| t < b) {
                                best = Some(t);
                            }
                        }
                    }
                }
            }
        }
        best
    }
}

impl Visible {
    /// Om noe som synes ligger innen `r` fra punktet (et treff på kanten av
    /// en flate, der strålen streifer den, er ikke en usynlig vegg).
    fn touches(&self, q: Vec3, r: f32) -> bool {
        let (x, z) = ((q.x / CELL).floor() as i32, (q.z / CELL).floor() as i32);
        (-1..=1).any(|dx| (-1..=1).any(|dz| self.cells.get(&(x + dx, z + dz)).is_some_and(|v| v.iter().any(|&n| closest(q, &self.tris[n as usize]).distance(q) <= r))))
    }
}

/// Det nærmeste punktet på trekanten (Ericson, «Real-Time Collision Detection»).
fn closest(p: Vec3, t: &[Vec3; 3]) -> Vec3 {
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    a + ab * (vb * denom) + ac * (vc * denom)
}

/// Möller–Trumbore, begge sider.
fn hit(o: Vec3, d: Vec3, t: &[Vec3; 3]) -> Option<f32> {
    let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let s = o - t[0];
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t >= 0.0).then_some(t)
}

/// Stedene på gulvet i nivå `k` (et rutenett på `step` meter i hver polygon
/// av navigasjonsnettet), med polygonen.
fn floor_points(c: &Crypt, k: usize, step: f32) -> Vec<Vec3> {
    let nav = &c.levels[k].nav;
    let mut out = vec![];
    for (i, p) in nav.polys.iter().enumerate() {
        let n = p.pts.len();
        let inside = |q: Vec2| (0..n).all(|j| {
            let (a, b) = (p.pts[j], p.pts[(j + 1) % n]);
            (b.x - a.x) * (q.y - a.z) - (b.z - a.z) * (q.x - a.x) >= -1e-4
        });
        // Rutenettet i nivåets eget rom (som `floor_points` i `solids.py`).
        let o = Vec2::new(c.levels[k].origin.x, c.levels[k].origin.z);
        let mut pts = vec![Vec2::new(p.center.x, p.center.z)];
        let (x0, z0) = (((p.min.x - o.x) / step).ceil() as i32, ((p.min.y - o.y) / step).ceil() as i32);
        let (x1, z1) = (((p.max.x - o.x) / step).floor() as i32, ((p.max.y - o.y) / step).floor() as i32);
        for x in x0..=x1 {
            for z in z0..=z1 {
                pts.push(o + Vec2::new(x as f32 * step, z as f32 * step));
            }
        }
        for q in pts.into_iter().filter(|q| inside(*q)) {
            out.push(Vec3::new(q.x, nav.height_in(i as u32, q), q.y));
        }
    }
    out
}

/// Usynlige vegger: fra hvert sted på gulvet ser strålene rundt seg i tre
/// høyder. Treffer en stråle en kollider som er loddrett (to stråler 6 cm
/// over hverandre treffer like langt unna), skal det være en synlig flate
/// innen `TOL` bak treffpunktet, eller like ved det (`EDGE`: strålen
/// streifer kanten av en flate). Gir stedene (gruppert per halvmeter).
fn invisible_walls(game: &Session, physics: &RapierPhysics, vis: &Visible, k: usize) -> Vec<(Vec3, f32, f32, u64, Vec3)> {
    const TOL: f32 = 0.12;
    const REACH: f32 = 1.2;
    // Strålen streifer en flate: treffet ligger så nær noe som synes.
    const EDGE: f32 = 0.03;
    let c = game.crypt_now();
    let mut found: HashMap<(i32, i32, i32), (Vec3, f32, f32, u64, Vec3)> = HashMap::new();
    for p in floor_points(c, k, 0.7) {
        for h in [0.45f32, 0.95, 1.5] {
            for a in 0..16 {
                let ang = a as f32 * std::f32::consts::TAU / 16.0;
                let d = Vec3::new(ang.cos(), 0.0, ang.sin());
                let o = p + Vec3::Y * h;
                let Some(hc) = physics.cast_solid(o, d, REACH) else { continue };
                let Some(h2) = physics.cast_solid(o + Vec3::Y * 0.06, d, REACH + 0.5) else { continue };
                if (h2.distance - hc.distance).abs() > 0.03 || hc.distance < 0.02 {
                    continue; // skrått (rampe, trapp), eller stedet er inni noe: ikke en vegg
                }
                let seen = vis.ray(o, d, hc.distance + 1.0);
                let at = o + d * hc.distance;
                if seen.is_none_or(|s| s > hc.distance + TOL) && !vis.touches(at, EDGE) {
                    let key = ((at.x * 2.0).floor() as i32, (at.y * 2.0).floor() as i32, (at.z * 2.0).floor() as i32);
                    found.entry(key).or_insert((at, hc.distance, seen.unwrap_or(f32::INFINITY), hc.collider.0, o));
                }
            }
        }
    }
    let mut v: Vec<_> = found.into_values().collect();
    v.sort_by(|a, b| a.0.x.total_cmp(&b.0.x).then(a.0.z.total_cmp(&b.0.z)));
    v
}

/// Kartleggingen av usynlige vegger i alle nivåene (`cargo test --release
/// ingen_usynlige -- --ignored --nocapture`). Kolliderne følger formen
/// (`tools/crypt/solids.py`), og treffene gikk fra 14 484 til 194 (18.
/// oktober: nivå 1–10 har 4, 19, 7, 4, 51, 21, 16, 62, 10 og 0). Det som er
/// igjen er 165 skrog (kantene av trappeskrogene og de trimmede trappene
/// som stikker ut over trinnet, og tynne biter i gitter og rekkverk) og 29
/// bokser; står som feilsøking til de er rettet.
#[test]
#[ignore]
fn ingen_usynlige_vegger_i_krypten() {
    invisible_walls_in(world());
}

/// Det samme i prøvenivået fra nivådokumentet (bitene fra byggesettet og
/// kolliderne deres, flettet av `crypt::compose`): ingen treff.
#[test]
fn ingen_usynlige_vegger_i_provenivaaet() {
    invisible_walls_in(doc_world());
}

fn invisible_walls_in(p: &Prepared) {
    let mut physics = RapierPhysics::new();
    let game = session_in(p, &mut physics);
    let mut input = Input::default();
    let mut game = game;
    game.update(&mut input, 1.0 / 30.0, &mut physics);
    let vis = Visible::new(p, &game);
    let c = &p.world.crypt;
    let w = &p.world;
    // Kolliderne legges inn i rekkefølge (økta: verdens faste, så målene).
    let describe = |id: u64, o: Vec3| -> String {
        let i = id as usize - 1;
        if let Some(sh) = w.colliders.get(i) {
            return match sh {
                crate::domain::world::Shape::Cuboid { center, size, .. } => format!("boks {:.2?} {:.2?}", (*center - o).to_array(), size.to_array()),
                crate::domain::world::Shape::Cylinder { center, radius, .. } => format!("sylinder {:.2?} r {radius:.2}", (*center - o).to_array()),
                crate::domain::world::Shape::Hull { points } => format!("skrog ved {:.2?}", (points[0] - o).to_array()),
                other => format!("{other:?}").chars().take(40).collect(),
            };
        }
        // Så målene, hvert med kolliderne som flyttes med det.
        let mut k = w.colliders.len();
        for (t, parts) in w.targets.iter().zip(&w.target_parts) {
            if i >= k && i <= k + parts.len() {
                return format!("{:?}", t.what);
            }
            k += 1 + parts.len();
        }
        "annet".into()
    };
    let mut report = vec![];
    for k in 0..c.levels.len() {
        let o = c.levels[k].origin;
        let walls = invisible_walls(&game, &physics, &vis, k);
        for (at, dc, dv, id, from) in &walls {
            let l = *at - o;
            let what = describe(*id, o);
            let f = *from - o;
            report.push(format!("nivå {}: usynlig vegg ved ({:.2}, {:.2}, {:.2}) i nivåets rom, sett fra ({:.2}, {:.2}, {:.2}): kollideren {:.2} m unna, det synlige {:.2} m ({what})", k + 1, l.x, l.y, l.z, f.x, f.y, f.z, dc, dv));
        }
    }
    eprintln!("{}", report.join("\n"));
    assert!(report.is_empty(), "{} usynlige vegger:\n{}", report.len(), report.join("\n"));
}

/// Gå langs navigasjonsnettet til `to` med kapselen og fysikken, uten å
/// vike til siden. Gir stedet der den ble stående fast (ingen framgang på
/// `STUCK` sekunder), eller `None` om den kom fram eller det ikke er vei.
fn walk(game: &mut Session, physics: &mut RapierPhysics, input: &mut Input, to: Vec3) -> Result<(), (Vec3, Vec3)> {
    const DT: f32 = 1.0 / 30.0;
    const STUCK: f32 = 2.0;
    let (mut best, mut since, mut t) = (f32::MAX, 0.0f32, 0.0f32);
    loop {
        game.clear_mobs();
        let feet = game.feet();
        if feet.with_y(0.0).distance(to.with_y(0.0)) < 0.5 && (feet.y - to.y).abs() < 1.2 {
            input.key("KeyW", false);
            return Ok(());
        }
        let Some(path) = game.crypt_now().path(feet + Vec3::Y * 0.2, to) else {
            input.key("KeyW", false);
            return Ok(());
        };
        let left = path.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>() + path.first().map_or(0.0, |c| c.distance(feet));
        if left < best - 0.15 {
            best = left;
            since = t;
        }
        // Hjørnene i veien ligger 0,4 m fra murene og kapselen er 0,35 m:
        // den sikter på hjørnet etter når den er en halv meter unna.
        let next = path.iter().copied().find(|c| c.with_y(0.0).distance(feet.with_y(0.0)) > 0.5).unwrap_or(to);
        if t - since > STUCK || t > 240.0 {
            input.key("KeyW", false);
            return Err((feet, next));
        }
        if std::env::var("FRISTAD_WALK_TRACE").is_ok() && (t * 30.0) as u32 % 10 == 0 {
            eprintln!("  [{t:.1}] føttene {feet:?} neste {next:?} igjen {left:.2}");
        }
        let d = next - game.eye();
        game.set_look(-d.x.atan2(-d.z), 0.0);
        input.key("KeyW", true);
        game.update(input, DT, physics);
        t += DT;
    }
}

/// Stedene i nivå `k` du skal kunne gå mellom: startstedet, døra opp,
/// trappa ned, hvilestedene, spakene, kistene, lappene, styrkene, begge
/// sider av portene, snarveiene og de falske veggene, foten av stigene, og
/// midt i hvert rom.
fn key_places(c: &Crypt, k: usize) -> Vec<(String, Vec3)> {
    let front = |pos: Vec3, yaw: f32, d: f32| {
        let q = pos + Vec3::new(yaw.sin(), 0.0, yaw.cos()) * d;
        c.snap(q + Vec3::Y * 0.3, 1.5).unwrap_or(q)
    };
    let l = &c.levels[k];
    let mut v = vec![("startstedet".to_string(), l.start.0)];
    v.extend(l.up.map(|(p, y)| ("døra opp".to_string(), front(p, y, 1.4))));
    v.extend(l.down.map(|(p, _)| ("trappa ned".to_string(), c.snap(p + Vec3::Y * 0.3, 2.0).unwrap_or(p))));
    v.extend(c.rests.iter().filter(|r| r.level == k).map(|r| ("hvilestedet".to_string(), front(r.pos, r.yaw, 1.3))));
    v.extend(c.levers.iter().filter(|r| r.level == k).map(|r| ("spaken".to_string(), front(r.pos, r.yaw, 0.9))));
    v.extend(c.chests.iter().filter(|r| r.level == k).map(|r| ("kista".to_string(), front(r.pos, r.yaw, 0.9))));
    v.extend(c.lores.iter().filter(|r| r.level == k).map(|r| (format!("«{}»", r.title), front(r.pos, r.yaw, 0.9))));
    v.extend(c.bonuses.iter().filter(|r| r.level == k).map(|r| ("styrken".to_string(), front(r.pos, r.yaw, 0.9))));
    v.extend(c.ladders.iter().filter(|r| r.level == k).map(|r| ("stigen".to_string(), front(r.pos, r.yaw, 0.8))));
    for g in c.gates.iter().filter(|g| g.level == k) {
        v.extend([1.4f32, -1.4].map(|d| ("porten".to_string(), front(g.pos, g.yaw, d))));
    }
    for s in c.shortcuts.iter().filter(|s| s.level == k) {
        v.extend([1.4f32, -1.4].map(|d| ("snarveien".to_string(), front(s.pos, s.yaw, d))));
    }
    for s in c.secrets.iter().filter(|s| s.level == k) {
        v.extend([1.2f32, -1.2].map(|d| ("den falske veggen".to_string(), front(s.pos, s.yaw, d))));
    }
    // Midt i hvert rom: midten av den største polygonen der.
    for (r, room) in l.rooms.iter().enumerate() {
        let area = |p: &crate::domain::world::crypt::nav::Poly| (p.max - p.min).x * (p.max - p.min).y;
        if let Some(p) = l.nav.polys.iter().filter(|p| p.room == Some(r)).max_by(|a, b| area(a).total_cmp(&area(b))) {
            v.push((format!("rommet «{}»", room.name), p.center));
        }
    }
    v
}

/// Kapselen går mellom nøkkelstedene i hvert nivå med alt åpnet (portene,
/// snarveiene, de falske veggene, vannet og broene; ikke fallemmene og
/// heisene), fra det nærmeste til det neste, og blir aldri stående fast.
#[test]
fn kapselen_gar_mellom_noekkelstedene_i_hvert_nivaa() {
    let c = &world().world.crypt;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    let only: Option<usize> = std::env::var("FRISTAD_LEVEL").ok().and_then(|s| s.parse::<usize>().ok()).map(|n| n - 1);
    let mut stuck = vec![];
    for k in (0..c.levels.len()).filter(|k| only.is_none_or(|o| o == *k)) {
        game.go_to_level(k);
        let shut: Vec<usize> = c.trapdoors.iter().map(|t| t.switch).chain(c.lifts.iter().map(|l| l.switch)).collect();
        for sw in (0..c.switches.len()).filter(|sw| c.switches[*sw].level == k && !shut.contains(sw)) {
            game.open_now(sw);
        }
        for _ in 0..240 {
            game.clear_mobs();
            game.update(&mut input, 1.0 / 30.0, &mut physics);
        }
        let o = c.levels[k].origin;
        let mut left = key_places(c, k);
        while !left.is_empty() {
            let feet = game.feet();
            let dist = |q: Vec3| game.crypt_now().path(feet + Vec3::Y * 0.2, q).map(|p| p.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>());
            let Some((i, _)) = left.iter().enumerate().filter_map(|(i, (_, q))| dist(*q).map(|d| (i, d))).min_by(|a, b| a.1.total_cmp(&b.1)) else {
                // Resten kan ikke nås herfra (bak en fallem): rett dit.
                let (_, q) = left.remove(0);
                game.place_feet(q + Vec3::Y * 0.05);
                continue;
            };
            let (what, q) = left.remove(i);
            if let Err((at, next)) = walk(&mut game, &mut physics, &mut input, q) {
                let (a, n) = (at - o, next - o);
                stuck.push(format!("nivå {}: sto fast på vei til {what} ved ({:.1}, {:.1}, {:.1}) i nivåets rom, ved ({:.2}, {:.2}, {:.2}), neste hjørne ({:.2}, {:.2}, {:.2})", k + 1, (q - o).x, (q - o).y, (q - o).z, a.x, a.y, a.z, n.x, n.y, n.z));
                eprintln!("{}", stuck.last().unwrap());
                game.place_feet(q + Vec3::Y * 0.05);
                game.update(&mut input, 1.0 / 30.0, &mut physics);
            }
        }
    }
    assert!(stuck.is_empty(), "kapselen sto fast {} ganger:\n{}", stuck.len(), stuck.join("\n"));
}

/// Til feilsøking: `FRISTAD_PROBE="nivå x,y,z x,y,z"` (nivåets eget rom)
/// setter kapselen ned i det første punktet og går mot det andre.
#[test]
#[ignore]
fn kapselen_proever_en_strekning() {
    let Ok(spec) = std::env::var("FRISTAD_PROBE") else { return };
    let parts: Vec<&str> = spec.split_whitespace().collect();
    let k: usize = parts[0].parse::<usize>().unwrap() - 1;
    let v = |s: &str| {
        let f: Vec<f32> = s.split(',').map(|x| x.parse().unwrap()).collect();
        Vec3::new(f[0], f[1], f[2])
    };
    let c = &world().world.crypt;
    let o = c.levels[k].origin;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.go_to_level(k);
    if std::env::var("FRISTAD_OPEN").is_ok() {
        for sw in (0..c.switches.len()).filter(|sw| c.switches[*sw].level == k) {
            game.open_now(sw);
        }
        for _ in 0..240 {
            game.update(&mut input, 1.0 / 30.0, &mut physics);
        }
    }
    let (a, b) = (v(parts[1]) + o, v(parts[2]) + o);
    game.place_feet(a);
    for i in 0..150 {
        game.clear_mobs();
        let d = b - game.eye();
        game.set_look(-d.x.atan2(-d.z), 0.0);
        input.key("KeyW", true);
        game.update(&mut input, 1.0 / 30.0, &mut physics);
        if i % 3 == 0 {
            let f = game.feet() - o;
            eprintln!("{:.1} s: føttene ({:.2}, {:.2}, {:.2})", i as f32 / 30.0, f.x, f.y, f.z);
        }
    }
}

/// Til feilsøking: som `kapselen_proever_en_strekning`, men med Rapier alene
/// (ingen økt): `FRISTAD_PROBE="nivå x,y,z x,y,z"`.
#[test]
#[ignore]
fn kapselen_proever_en_strekning_i_rapier() {
    let Ok(spec) = std::env::var("FRISTAD_PROBE") else { return };
    let parts: Vec<&str> = spec.split_whitespace().collect();
    let k: usize = parts[0].parse::<usize>().unwrap() - 1;
    let v = |s: &str| {
        let f: Vec<f32> = s.split(',').map(|x| x.parse().unwrap()).collect();
        Vec3::new(f[0], f[1], f[2])
    };
    let p = world();
    let o = p.world.crypt.levels[k].origin;
    let mut physics = RapierPhysics::new();
    for c in &p.world.colliders {
        physics.add(c);
    }
    let (half, radius) = (0.55f32, 0.35f32);
    let (a, b) = (v(parts[1]) + o, v(parts[2]) + o);
    let mut pos = a + Vec3::Y * (half + radius + 0.08);
    for i in 0..60 {
        let d = (b - pos).with_y(0.0).normalize_or_zero() * 4.0 / 30.0 - Vec3::Y * 0.1;
        let (moved, grounded) = physics.move_capsule(pos, half, radius, d, 1.0 / 30.0);
        pos += moved;
        if i % 3 == 0 {
            let f = pos - o;
            eprintln!("{i}: ({:.2}, {:.2}, {:.2}) på bakken {grounded} flyttet {moved:?}", f.x, f.y - half - radius, f.z);
        }
    }
}

/// Vindeltrappa i rotunden i kloakken (nivå 3, eierens feil 18. oktober):
/// før slusa er dreid stopper vannet deg der det når til knærne (ikke en
/// usynlig mur over vannet), siktet sier hva det er, og når vannet har
/// sunket går du helt ned til bunnen.
#[test]
fn vindeltrappa_i_rotunden_gaar_ned_i_vannet() {
    let c = &world().world.crypt;
    let k = c.levels.iter().position(|l| l.n == 3).expect("nivå 3");
    let o = c.levels[k].origin;
    let mut physics = RapierPhysics::new();
    let mut game = session(&mut physics);
    let mut input = Input::default();
    game.go_to_level(k);
    // Øverst i trappa (ved 290°) og ned langs den mot foten (ved 200°).
    let at = |deg: f32, y: f32| o + Vec3::new(78.0 + 8.1 * deg.to_radians().cos(), y, 54.0 + 8.1 * deg.to_radians().sin());
    let down = |game: &mut Session, physics: &mut RapierPhysics, input: &mut Input| {
        game.place_feet(at(292.0, 0.05));
        for i in 0..300 {
            game.clear_mobs();
            let deg = (285.0 - i as f32 * 0.5).max(200.0);
            let d = at(deg, 0.0) - game.eye();
            game.set_look(-d.x.atan2(-d.z), 0.0);
            input.key("KeyW", true);
            game.update(input, 1.0 / 30.0, physics);
        }
        input.key("KeyW", false);
        game.feet() - o
    };
    let feet = down(&mut game, &mut physics, &mut input);
    assert!(feet.y < -0.75 && feet.y > -1.3, "vannet (−0,5 m) skal stoppe deg til knærne i trappa, men føttene sto på {:.2}", feet.y);
    let sw = c.waters.iter().find(|w| w.level == k).map(|w| w.switch).expect("vannet i nivå 3");
    game.open_now(sw);
    for _ in 0..240 {
        game.update(&mut input, 1.0 / 30.0, &mut physics);
    }
    let feet = down(&mut game, &mut physics, &mut input);
    assert!(feet.y < -3.3, "når vannet har sunket, skal trappa gå til bunnen (−3,6 m), men føttene sto på {:.2}", feet.y);
}
