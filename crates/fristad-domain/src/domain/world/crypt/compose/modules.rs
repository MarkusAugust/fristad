//! Modulene (`docs/nivaaredigering.md`, del 21; eieren 19. oktober: «jeg vil
//! ha moduler som kan settes sammen»): ferdige rom, ganger, trapper og rom
//! med innhold (`ModuleDef`, `assets/levels/moduler/<tema>/<navn>.json`) som
//! eieren setter ut, dreier og speiler som Lego. Før kompileringen utvides
//! hver modul i dokumentet (`DocModule`) til cellene, rommet, innholdet og
//! lysene sine (`flatten`), og der to moduler møtes, velges kanten:
//!
//! - Hver ytre side av en modul som ikke er lukket, er en **åpning**.
//! - Har begge modulene en åpning på samme kant, blir det en åpning: dør
//!   mellom to rom, gangåpning der en gang er med, bue om en av dem vil ha
//!   bue, og port, falsk vegg eller snarvei om en av dem ber om det.
//!   Eierens valg (`overrides`) går foran modulenes eget.
//! - Møtes to moduler langs flere kanter som passer, blir bare den midterste
//!   åpen (med mindre eieren har valgt flere); resten er mur.
//! - Ellers er det mur, også mot en celle utenfor modulene uten åpning.
//!
//! Etter utvidelsen er dokumentet et vanlig dokument med celler og kantvalg,
//! så kompilatoren og reglene er de samme. Bitene og cellene er bare det
//! kompilatoren bruker inni.

use super::grid::*;
use super::{expand, Issue};
use crate::domain::world::crypt::doc::{DocCell, DocEdge, DocLight, DocLink, DocModule, DocRoom, LevelDoc, LevelId, ModuleDef, Mood, Thing};
use crate::domain::world::crypt::kit::KitDef;
use glam::Vec3;
use std::collections::{BTreeMap, HashMap};

/// Slagene en åpning kan ha (`auto` er modulens eget valg).
pub const OPENING_KINDS: [&str; 8] = ["auto", "gang", "bue", "dor", "port", "falsk", "snarvei", "vegg"];

/// En åpning i modulens rom: cella (delcella i en sammensatt bit), ruta
/// (i, j), siden, id-en og slaget.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalOpening {
    pub cell: String,
    pub at: [i32; 2],
    pub y: i32,
    pub side: usize,
    pub id: String,
    pub kind: String,
}

/// Åpningene en modul har i sitt eget rom: hver ytre side som ikke er
/// lukket, utenom `walls`, med slaget og navnet fra `openings`.
pub fn openings(m: &ModuleDef, kit: &KitDef) -> (Vec<LocalOpening>, Vec<Issue>) {
    let mut issues = vec![];
    let temp = LevelDoc { cells: m.cells.clone(), ..Default::default() };
    let subs = expand(&temp, kit);
    let columns: std::collections::HashSet<(i32, i32)> = subs.iter().map(|c| (c.at[0], c.at[2])).collect();
    let mut out = vec![];
    for c in &subs {
        let Some(piece) = kit.piece(&c.bit) else {
            issues.push(Issue::error("modul", None, format!("modulen {}: ukjent bit «{}»", m.id, c.bit)));
            continue;
        };
        for side in 0..4 {
            let (di, dj) = outward(side);
            if columns.contains(&(c.at[0] + di, c.at[2] + dj)) {
                continue;
            }
            let face = piece.faces.get(world_to_local(side, c.rot % 4, c.mirror && piece.mirror)).map(|f| f.kind.as_str()).unwrap_or("closed");
            if face == "closed" || face == "inner" {
                continue;
            }
            let here = |o: &crate::domain::world::crypt::doc::ModuleOpening| o.at == [c.at[0], c.at[2]] && side_index(&o.side) == Some(side);
            if m.walls.iter().any(here) {
                continue;
            }
            let named = m.openings.iter().find(|o| here(o));
            let id = named.map(|o| o.id.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| format!("{},{},{}", c.at[0], c.at[2], SIDES[side]));
            let kind = named.map(|o| o.kind.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| "auto".into());
            out.push(LocalOpening { cell: c.id.clone(), at: [c.at[0], c.at[2]], y: c.at[1], side, id, kind });
        }
    }
    for o in &m.openings {
        if !out.iter().any(|p| p.at == o.at && Some(p.side) == side_index(&o.side)) {
            issues.push(Issue::error("modul", None, format!("modulen {}: åpningen {:?}/{} ligger ikke på en ytre side som kan åpnes", m.id, o.at, o.side)));
        }
        if !o.kind.is_empty() && !OPENING_KINDS.contains(&o.kind.as_str()) {
            issues.push(Issue::error("modul", None, format!("modulen {}: ukjent slag «{}»", m.id, o.kind)));
        }
    }
    (out, issues)
}

/// En åpning satt ut i nivået.
#[derive(Clone, Debug)]
struct Port {
    inst: usize,
    id: String,
    kind: String,
    explicit: bool,
    thing: Option<String>,
}

/// Hvor sterkt et slag står når to åpninger møtes.
fn rank(kind: &str) -> i32 {
    match kind {
        "vegg" => 7,
        "falsk" => 6,
        "port" => 5,
        "snarvei" => 4,
        "dor" => 3,
        "gang" => 2,
        "bue" => 1,
        _ => 0,
    }
}

/// Slaget når to åpninger møtes: eierens valg først, så det sterkeste av
/// modulenes; to «auto» gir gangåpning om en av sidene er en gang, ellers dør.
fn resolve(a: &Port, b: &Port, gang_face: bool) -> String {
    let chosen = |p: &Port| p.explicit && p.kind != "auto";
    let pool: Vec<&Port> = if chosen(a) || chosen(b) { [a, b].into_iter().filter(|p| chosen(p)).collect() } else { vec![a, b] };
    let best = pool.iter().max_by_key(|p| rank(&p.kind)).unwrap();
    match best.kind.as_str() {
        "vegg" | "falsk" | "port" | "snarvei" | "dor" => best.kind.clone(),
        _ if gang_face || pool.iter().any(|p| p.kind == "gang") => "gang".into(),
        _ if pool.iter().any(|p| p.kind == "bue") => "bue".into(),
        _ => "dor".into(),
    }
}

/// Dokumentet med modulene utvidet til celler, rom, innhold, lys og
/// kantvalg (se modulteksten øverst). Et dokument uten moduler kommer
/// tilbake som det er.
pub fn flatten(doc: &LevelDoc, kit: &KitDef) -> (LevelDoc, Vec<Issue>) {
    let mut issues = vec![];
    if doc.modules.is_empty() {
        return (doc.clone(), issues);
    }
    let mut out = LevelDoc { modules: vec![], ..doc.clone() };
    let mut ports: HashMap<(String, usize), Port> = HashMap::new();
    let mut owner: HashMap<String, usize> = HashMap::new();
    let mut anchors: HashMap<(String, String), (String, usize)> = HashMap::new();
    let mut module_things: Vec<Thing> = vec![];
    let mut module_edges: Vec<DocEdge> = vec![];
    for (n, inst) in doc.modules.iter().enumerate() {
        let Some(m) = kit.module(&inst.module) else {
            issues.push(Issue::error("modul", Some(cell_center(inst.at[0], inst.at[1] as f32, inst.at[2])), format!("{}: ukjent modul «{}»", inst.id, inst.module)));
            continue;
        };
        if doc.modules[..n].iter().any(|o| o.id == inst.id) {
            issues.push(Issue::error("modul", None, format!("to moduler heter «{}»", inst.id)));
        }
        let rot = inst.rot % 4;
        let t = transform(Vec3::ZERO, cell_yaw(rot), inst.mirror);
        let origin = cell_center(inst.at[0], inst.at[1] as f32, inst.at[2]);
        let place = |at: [i32; 3]| {
            let d = t.transform_vector3(Vec3::new(at[0] as f32, 0.0, at[2] as f32));
            [inst.at[0] + d.x.round() as i32, inst.at[1] + at[1], inst.at[2] + d.z.round() as i32]
        };
        let point = |p: [f32; 3]| (t.transform_point3(Vec3::from(p)) + origin).to_array().map(mm);
        let turn = |yaw: f32| {
            let y = if inst.mirror { -yaw } else { yaw } - rot as f32 * 90.0;
            mm(y.rem_euclid(360.0))
        };
        // Rommene: det første er hovedrommet (eller rommet modulen skal høre til).
        let main_local = m.rooms.first().map(|r| r.id.clone()).unwrap_or_default();
        let main = inst.room.clone().unwrap_or_else(|| inst.id.clone());
        let room_of = |local: &str| if local.is_empty() || local == main_local { main.clone() } else { format!("{}/{local}", inst.id) };
        let rooms = if m.rooms.is_empty() { vec![DocRoom::default()] } else { m.rooms.clone() };
        for (k, r) in rooms.iter().enumerate() {
            let id = if k == 0 { main.clone() } else { room_of(&r.id) };
            if out.rooms.iter().any(|x| x.id == id) {
                continue;
            }
            let mut r = DocRoom { id, ..r.clone() };
            if r.theme.is_empty() {
                r.theme = m.theme.clone();
            }
            if k == 0 {
                r.name = inst.name.clone().unwrap_or_else(|| if r.name.is_empty() { m.name.clone() } else { r.name.clone() });
                if let Some(mood) = &inst.mood {
                    r.mood = mood.clone();
                }
            }
            out.rooms.push(r);
        }
        let local_rooms: Vec<String> = rooms.iter().map(|r| r.id.clone()).collect();
        let prefix = |s: &str| format!("{}/{s}", inst.id);
        // Cellene, dreid og speilet med modulen.
        for c in &m.cells {
            let (r, mirror) = if inst.mirror { ((rot + 4 - c.rot % 4) % 4, !c.mirror) } else { ((rot + c.rot) % 4, c.mirror) };
            let id = prefix(&c.id);
            owner.insert(id.clone(), n);
            // Modulens stein (eierens valg i nivået) går foran malens.
            out.cells.push(DocCell { id, at: place(c.at), rot: r, mirror, room: room_of(&c.room), stone: inst.stone.clone().or_else(|| c.stone.clone()), ..c.clone() });
        }
        for e in &m.edges {
            module_edges.push(DocEdge { cell: prefix(&e.cell), ..e.clone() });
        }
        // Innholdet og lysene: ankrene i modulens celler, eller fritt i modulens rom.
        let anchor = |a: &str| {
            if let Some(r) = a.strip_prefix("celle:") {
                format!("celle:{}", prefix(r))
            } else if let Some(r) = a.strip_prefix("kant:") {
                format!("kant:{}", prefix(r))
            } else {
                a.to_string()
            }
        };
        let free = |a: &str| a.is_empty() || a == "fri";
        let ids: Vec<&str> = m.things.iter().map(|t| t.id.as_str()).collect();
        for th in &m.things {
            let mut th2 = Thing { id: prefix(&th.id), anchor: anchor(&th.anchor), ..th.clone() };
            if free(&th.anchor) {
                th2.pos = th.pos.map(point);
                th2.yaw = turn(th.yaw);
                th2.patrol = th.patrol.iter().map(|p| point(*p)).collect();
                th2.top = th.top.map(point);
                th2.bottom = th.bottom.map(point);
            }
            module_things.push(th2);
        }
        for l in &m.links {
            let fix = |s: &str| {
                if ids.contains(&s) || local_rooms.iter().any(|r| r == s) {
                    if local_rooms.first().is_some_and(|r| r == s) {
                        main.clone()
                    } else {
                        prefix(s)
                    }
                } else {
                    s.to_string()
                }
            };
            out.links.push(DocLink { from: fix(&l.from), to: fix(&l.to), kind: l.kind.clone() });
        }
        for l in &m.lights {
            let mut l2 = DocLight { id: prefix(&l.id), anchor: anchor(&l.anchor), ..l.clone() };
            if free(&l.anchor) {
                l2.pos = point(l.pos);
            }
            out.lights.push(l2);
        }
        // Åpningene.
        let (local, mut problems) = openings(m, kit);
        issues.append(&mut problems);
        for o in &local {
            let w = local_to_world(o.side, rot, inst.mirror);
            let cell = prefix(&o.cell);
            let ov = inst.overrides.iter().find(|v| v.opening == o.id);
            let port = Port { inst: n, id: o.id.clone(), kind: ov.map(|v| v.kind.clone()).unwrap_or_else(|| o.kind.clone()), explicit: ov.is_some(), thing: ov.and_then(|v| v.id.clone()) };
            anchors.insert((inst.id.clone(), o.id.clone()), (cell.clone(), w));
            ports.insert((cell, w), port);
        }
        for v in &inst.overrides {
            if !local.iter().any(|o| o.id == v.opening) {
                issues.push(Issue::error("modul", Some(origin), format!("{}: modulen {} har ingen åpning «{}»", inst.id, inst.module, v.opening)));
            }
            if !OPENING_KINDS.contains(&v.kind.as_str()) {
                issues.push(Issue::error("modul", Some(origin), format!("{}: ukjent slag «{}» for åpningen {}", inst.id, v.kind, v.opening)));
            }
        }
    }

    // Alle cellene (delcellene til de sammensatte bitene), med gulvet ved
    // hver side i nivået og modulen de hører til.
    let subs = expand(&out, kit);
    struct Sub<'a> {
        doc: &'a DocCell,
        floors: [f32; 4],
        mirror: bool,
        faces: [String; 4],
        owner: Option<usize>,
    }
    let base = |id: &str| id.rsplit_once('#').map(|(b, _)| b.to_string()).unwrap_or_else(|| id.to_string());
    let info: Vec<Sub> = subs
        .iter()
        .filter_map(|c| {
            let p = kit.piece(&c.bit)?;
            let mirror = c.mirror && p.mirror;
            let y = c.at[1] as f32;
            let floors = [0, 1, 2, 3].map(|w| y + p.floors[world_to_local(w, c.rot % 4, mirror)]);
            let faces = [0, 1, 2, 3].map(|w| p.faces.get(world_to_local(w, c.rot % 4, mirror)).map(|f| f.kind.clone()).unwrap_or_default());
            Some(Sub { doc: c, floors, mirror, faces, owner: owner.get(&base(&c.id)).copied() })
        })
        .collect();
    let mut columns: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (k, s) in info.iter().enumerate() {
        columns.entry((s.doc.at[0], s.doc.at[2])).or_default().push(k);
    }
    let neighbor = |a: usize, w: usize| -> Option<usize> {
        let s = &info[a];
        let (di, dj) = outward(w);
        columns.get(&(s.doc.at[0] + di, s.doc.at[2] + dj))?.iter().copied().find(|&b| (info[b].floors[opposite(w)] - s.floors[w]).abs() < 0.01)
    };
    let local_side = |a: usize, w: usize| SIDES[world_to_local(w, info[a].doc.rot % 4, info[a].mirror)].to_string();
    let mut walls: Vec<(usize, usize)> = vec![];
    let mut pairs: BTreeMap<(usize, usize), Vec<(usize, usize, usize)>> = BTreeMap::new();
    for a in 0..info.len() {
        let Some(na) = info[a].owner else { continue };
        for w in 0..4 {
            let Some(b) = neighbor(a, w) else { continue };
            let nb = info[b].owner;
            if nb == Some(na) {
                continue;
            }
            let pa = ports.get(&(info[a].doc.id.clone(), w));
            match nb {
                None => {
                    if pa.is_none() {
                        walls.push((a, w));
                    }
                }
                Some(nb) => {
                    let pb = ports.get(&(info[b].doc.id.clone(), opposite(w)));
                    if pa.is_some() && pb.is_some() {
                        if na < nb {
                            pairs.entry((na, nb)).or_default().push((a, w, b));
                        }
                    } else {
                        walls.push((a, w));
                    }
                }
            }
        }
    }
    let secret = |room: &str| out.rooms.iter().any(|r| r.id == room && r.secret);
    let mut new_edges: Vec<DocEdge> = vec![];
    let mut new_things: Vec<Thing> = vec![];
    let mut new_links: Vec<DocLink> = vec![];
    for (_, mut group) in pairs {
        let port = |a: usize, w: usize| ports[&(info[a].doc.id.clone(), w)].clone();
        group.sort_by_key(|&(a, w, _)| (info[a].doc.at[1], info[a].doc.at[0], info[a].doc.at[2], w));
        let explicit: Vec<usize> = (0..group.len()).filter(|&k| port(group[k].0, group[k].1).explicit || port(group[k].2, opposite(group[k].1)).explicit).collect();
        let chosen = if explicit.is_empty() { vec![(group.len() - 1) / 2] } else { explicit };
        for (k, &(a, w, b)) in group.iter().enumerate() {
            if !chosen.contains(&k) {
                walls.push((a, w));
                continue;
            }
            let (pa, pb) = (port(a, w), port(b, opposite(w)));
            let gang_face = info[a].faces[w] == "gang" || info[b].faces[opposite(w)] == "gang";
            let kind = resolve(&pa, &pb, gang_face);
            let edge = |cell: usize, side: usize, kind: &str| DocEdge { cell: info[cell].doc.id.clone(), side: local_side(cell, side), kind: kind.into(), ..Default::default() };
            // Modulen som ba om slaget (eierens valg først; ber begge om en
            // falsk vegg, den med det hemmelige rommet). Innholdet på kanten
            // heter det eieren ga det, ellers etter åpningen.
            let asks = |p: &Port| p.kind == kind;
            let by_b = match (asks(&pa), asks(&pb)) {
                (true, true) => {
                    if pa.explicit != pb.explicit {
                        pb.explicit
                    } else {
                        kind == "falsk" && secret(&info[b].doc.room) && !secret(&info[a].doc.room)
                    }
                }
                (false, true) => true,
                _ => false,
            };
            let asker = if by_b { &pb } else { &pa };
            let thing_id = asker.thing.clone().unwrap_or_else(|| format!("{}/{}", doc.modules[asker.inst].id, asker.id));
            match kind.as_str() {
                "dor" | "gang" | "vegg" => new_edges.push(edge(a, w, &kind)),
                "bue" => new_edges.push(edge(a, w, "aapen")),
                "port" | "snarvei" => new_things.push(Thing {
                    id: thing_id,
                    kind: if kind == "port" { "gate" } else { "shortcut" }.into(),
                    anchor: format!("kant:{}/{}", info[a].doc.id, local_side(a, w)),
                    ..Default::default()
                }),
                _ => {
                    // Den falske veggen synes fra modulen som ikke ba om den.
                    let (va, vw) = if by_b { (a, w) } else { (b, opposite(w)) };
                    let l = world_to_local(vw, info[va].doc.rot % 4, info[va].mirror);
                    new_things.push(Thing { id: thing_id.clone(), kind: "secret_wall".into(), anchor: format!("kant:{}/{}", info[va].doc.id, SIDES[l]), ..Default::default() });
                    let o = outward_v(l) * FACE;
                    let hint = format!("{thing_id}_sprekk");
                    new_things.push(Thing {
                        id: hint.clone(),
                        kind: "hint".into(),
                        anchor: format!("celle:{}", info[va].doc.id),
                        pos: Some([mm(o.x), 0.0, mm(o.y)]),
                        yaw: mm(side_yaw(l).to_degrees().rem_euclid(360.0)),
                        label: Some("crack".into()),
                        ..Default::default()
                    });
                    new_links.push(DocLink { from: hint, to: thing_id, kind: "for".into() });
                }
            }
        }
    }
    walls.sort();
    walls.dedup();
    for (a, w) in walls {
        new_edges.push(DocEdge { cell: info[a].doc.id.clone(), side: local_side(a, w), kind: "vegg".into(), ..Default::default() });
    }
    // Kantvalgene: sammenføyningen, så modulenes egne, og eierens i
    // dokumentet sist (det siste går foran).
    let user_edges = std::mem::take(&mut out.edges);
    out.edges = new_edges.into_iter().chain(module_edges).chain(user_edges).collect();
    // Innhold på en åpning (`aapning:<modul>/<åpning>`) står på kanten.
    let mut things: Vec<Thing> = module_things.into_iter().chain(new_things).collect();
    let user: Vec<Thing> = std::mem::take(&mut out.things);
    things.extend(user);
    for th in &mut things {
        if let Some(r) = th.anchor.strip_prefix("aapning:") {
            let key = r.split_once('/').map(|(m, o)| (m.to_string(), o.to_string()));
            match key.and_then(|k| anchors.get(&k)) {
                Some((cell, w)) => {
                    let a = info.iter().position(|s| s.doc.id == *cell).unwrap_or(0);
                    th.anchor = format!("kant:{cell}/{}", local_side(a, *w));
                }
                None => issues.push(Issue::error("innhold", None, format!("{}: åpningen «{r}» finnes ikke", th.id))),
            }
        }
    }
    out.things = things;
    out.links.extend(new_links);
    (out, issues)
}

/// En modul alene i et eget nivå, til bildene og «Se i 3D»: startstedet i
/// midtcella (om modulen ikke har et), fakler i alle festene og et svakt
/// lys midt i, så rommet synes slik det er bygd.
pub fn showcase(kit: &KitDef, id: &str) -> Option<LevelDoc> {
    let m = kit.module(id)?;
    let main = m.rooms.first().cloned().unwrap_or_default();
    let mut doc = LevelDoc { format: 1, level: LevelId::Name(format!("modul:{id}")), name: m.name.clone(), status: "utkast".into(), seed: 1, ..Default::default() };
    doc.modules.push(DocModule { id: "m".into(), module: id.into(), mood: Some(Mood { torches: main.mood.torches.max(0.8) }), ..Default::default() });
    if !m.things.iter().any(|t| t.kind == "start") {
        let first = m.cells.iter().min_by_key(|c| (c.at[0].abs() + c.at[2].abs(), c.at[0], c.at[2]))?;
        doc.things.push(Thing { id: "start".into(), kind: "start".into(), anchor: format!("celle:m/{}", first.id), pos: Some([0.0, 0.0, 0.0]), ..Default::default() });
    }
    let n = m.cells.len().max(1) as f32;
    let (sx, sy, sz) = m.cells.iter().fold((0.0, 0.0, 0.0), |(x, y, z), c| (x + c.at[0] as f32, y + c.at[1] as f32, z + c.at[2] as f32));
    doc.lights.push(DocLight {
        id: "fyll".into(),
        anchor: "fri".into(),
        pos: [mm(sx / n * CELL), mm(sy / n + 3.4), mm(sz / n * CELL)],
        color: [1.0, 0.8, 0.6],
        range: 14.0,
        flicker: false,
        intensity: Some(0.9),
    });
    Some(doc)
}

/// Åpningene til en modul satt ut i nivået: ruta (i, j) og høyden i
/// nivåets rom, og siden i nivået (kartet og bildene).
pub fn placed_openings(inst: &DocModule, kit: &KitDef) -> Vec<LocalOpening> {
    let Some(m) = kit.module(&inst.module) else { return vec![] };
    let t = transform(Vec3::ZERO, cell_yaw(inst.rot % 4), inst.mirror);
    // Delcellene i de sammensatte bitene har sitt eget gulv (galleriet).
    let subs = expand(&LevelDoc { cells: m.cells.clone(), ..Default::default() }, kit);
    let floors = |o: &LocalOpening| {
        let c = subs.iter().find(|c| c.id == o.cell);
        let p = c.and_then(|c| kit.piece(&c.bit));
        match (c, p) {
            (Some(c), Some(p)) => p.floors[world_to_local(o.side, c.rot % 4, c.mirror && p.mirror)].round() as i32,
            _ => 0,
        }
    };
    modules_openings_or_empty(m, kit)
        .into_iter()
        .map(|o| {
            let d = t.transform_vector3(Vec3::new(o.at[0] as f32, 0.0, o.at[1] as f32));
            LocalOpening {
                cell: format!("{}/{}", inst.id, o.cell),
                at: [inst.at[0] + d.x.round() as i32, inst.at[2] + d.z.round() as i32],
                y: inst.at[1] + o.y + floors(&o),
                side: local_to_world(o.side, inst.rot % 4, inst.mirror),
                ..o
            }
        })
        .collect()
}

fn modules_openings_or_empty(m: &ModuleDef, kit: &KitDef) -> Vec<LocalOpening> {
    openings(m, kit).0
}

/// Alle modulene side om side med fjell imellom (`?nivaa=moduler`,
/// `snapshot --level-doc moduler`): bildene av modulene tas her i én kjøring.
/// Hver modul står med `rot` 0 og sitt eget rom, med fakler i festene og et
/// lys midt i. Gangene, trappene og de ferdige rommene som ikke er saler får
/// et lite rom (`grav/rom_1x1`) inntil åpningene sine, så bildet viser dem
/// koblet (rom-modulene står alene: åpningene deres er hele omkretsen).
pub fn gallery(kit: &KitDef) -> LevelDoc {
    let mut doc = LevelDoc { format: 1, level: LevelId::Name("moduler".into()), name: "Modulene".into(), status: "utkast".into(), seed: 1, ..Default::default() };
    let mut list: Vec<&ModuleDef> = kit.modules.iter().collect();
    list.sort_by(|a, b| a.id.cmp(&b.id));
    let mut x = 0;
    let mut row_z = 0;
    let mut row_h = 0;
    for m in list {
        let cells = expand(&LevelDoc { cells: m.cells.clone(), ..Default::default() }, kit);
        if cells.is_empty() {
            continue;
        }
        // Gravkjellerens moduler heter det de heter; de andre temaene får
        // temaet foran (`kloakk_rom_2x2`), så navnene ikke kolliderer.
        let (theme, base) = m.id.split_once('/').unwrap_or(("grav", &m.id));
        let name = if theme == "grav" { base.to_string() } else { format!("{theme}_{base}") };
        let host = if kit.module(&format!("{theme}/rom_1x1")).is_some() { format!("{theme}/rom_1x1") } else { "grav/rom_1x1".to_string() };
        let ops = openings(m, kit).0;
        let hosted: Vec<&LocalOpening> = match base {
            _ if kit.module(&host).is_none() => vec![],
            "skattkammer" => ops.iter().filter(|o| o.side == 1).collect(),
            "nedgang" => ops.iter().filter(|o| o.side == 1).collect(),
            _ if matches!(m.kind.as_str(), "gang" | "trapp") || matches!(base, "gravgang" | "fellegang" | "gruvegang" | "riddergang") => ops.iter().collect(),
            _ => vec![],
        };
        let mut spots: Vec<[i32; 3]> = cells.iter().map(|c| c.at).collect();
        let host_at: Vec<[i32; 3]> = hosted
            .iter()
            .map(|o| {
                let (di, dj) = outward(o.side);
                let c = cells.iter().find(|c| c.id == o.cell).unwrap();
                let p = kit.piece(&c.bit).unwrap();
                let lift = p.floors[world_to_local(o.side, c.rot % 4, c.mirror && p.mirror)].round() as i32;
                [o.at[0] + di, o.y + lift, o.at[1] + dj]
            })
            .collect();
        spots.extend(host_at.iter().copied());
        let (i0, i1) = (spots.iter().map(|c| c[0]).min().unwrap(), spots.iter().map(|c| c[0]).max().unwrap());
        let (j0, j1) = (spots.iter().map(|c| c[2]).min().unwrap(), spots.iter().map(|c| c[2]).max().unwrap());
        if x > 24 {
            x = 0;
            row_z += row_h + 2;
            row_h = 0;
        }
        let at = [x - i0, 0, row_z - j0];
        let main = m.rooms.first().cloned().unwrap_or_default();
        doc.modules.push(DocModule { id: name.clone(), module: m.id.clone(), at, mood: Some(Mood { torches: main.mood.torches.max(0.8) }), ..Default::default() });
        for (k, h) in host_at.iter().enumerate() {
            doc.modules.push(DocModule {
                id: format!("{name}_nabo{k}"),
                module: host.clone(),
                at: [at[0] + h[0], h[1], at[2] + h[2]],
                name: Some("Nabo".into()),
                mood: Some(Mood { torches: 0.8 }),
                ..Default::default()
            });
        }
        let n = cells.len() as f32;
        let (sx, sz) = cells.iter().fold((0.0, 0.0), |(a, b), c| (a + c.at[0] as f32, b + c.at[2] as f32));
        doc.lights.push(DocLight {
            id: format!("fyll_{name}"),
            anchor: "fri".into(),
            pos: [mm((sx / n + at[0] as f32) * CELL), 3.6, mm((sz / n + at[2] as f32) * CELL)],
            color: [1.0, 0.82, 0.62],
            range: 16.0,
            flicker: false,
            intensity: Some(1.6),
        });
        x += i1 - i0 + 3;
        row_h = row_h.max(j1 - j0 + 1);
    }
    if !kit.modules.iter().any(|m| m.things.iter().any(|t| t.kind == "start")) {
        if let Some(first) = doc.modules.first() {
            let m = kit.module(&first.module).unwrap();
            doc.things.push(Thing { id: "start".into(), kind: "start".into(), anchor: format!("celle:{}/{}", first.id, m.cells[0].id), pos: Some([0.0, 0.0, 0.0]), ..Default::default() });
        }
    }
    doc
}
