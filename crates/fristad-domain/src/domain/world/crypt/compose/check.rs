//! Reglene over resultatet (`docs/nivaaredigering.md`, del 10), regnet ut
//! for seg, uavhengig av valgene kompilatoren gjorde: de samme
//! funksjonene kjører i redigeringen og i `cargo test`.

use super::grid::*;
use super::{Issue, Severity};
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::kit::{KitDef, PieceKind};
use crate::domain::world::crypt::level::LevelFile;
use glam::Vec3;
use std::collections::HashMap;

/// En celle slik reglene ser den: ruta, gulvet, volumet og gulvene ved
/// sidene i nivåets rom.
struct C {
    id: String,
    i: i32,
    j: i32,
    y: f32,
    lo: f32,
    hi: f32,
    floors: [f32; 4],
}

fn cells(doc: &LevelDoc, kit: &KitDef) -> Vec<C> {
    super::expand(&super::modules::flatten(doc, kit).0, kit)
        .iter()
        .filter_map(|c| {
            let p = kit.piece(&c.bit).filter(|p| p.kind == PieceKind::Cell)?;
            let mirror = c.mirror && p.mirror;
            let y = c.at[1] as f32;
            let floors = [0, 1, 2, 3].map(|w| y + p.floors[world_to_local(w, c.rot % 4, mirror)]);
            Some(C { id: c.id.clone(), i: c.at[0], j: c.at[2], y, lo: y + p.bottom, hi: y + p.top.max(p.bottom + 0.5), floors })
        })
        .collect()
}

/// Bitene overlapper i volum: to celler i samme rute med volumer som går inn
/// i hverandre (gulvskive og hvelv er med). To moduler i samme celle er
/// den samme feilen, og meldes med modulene.
pub fn overlaps(doc: &LevelDoc, kit: &KitDef) -> Vec<Issue> {
    let cs = cells(doc, kit);
    let module = |id: &str| id.split_once('/').map(|(m, _)| m).filter(|m| doc.modules.iter().any(|x| x.id == *m)).map(str::to_string);
    let mut out = vec![];
    for a in 0..cs.len() {
        for b in a + 1..cs.len() {
            let (p, q) = (&cs[a], &cs[b]);
            if p.i == q.i && p.j == q.j && p.lo < q.hi - 0.01 && q.lo < p.hi - 0.01 {
                let msg = match (module(&p.id), module(&q.id)) {
                    (Some(x), Some(y)) if x != y => format!("modulene {x} og {y} overlapper i ({}, {}, {})", p.i, p.y, p.j),
                    _ => format!("{} og {} overlapper", p.id, q.id),
                };
                out.push(Issue { severity: Severity::Error, rule: "overlapp", at: Some(mm3(cell_center(p.i, p.y, p.j))), msg });
            }
        }
    }
    out
}

/// Tett: hver side av hver celle er enten lukket av en mur (en halvkant som
/// ikke er åpen), eller åpen mot en celle med gulvet i samme høyde; og hvert
/// hjørne der en mur ender eller fjellet står på skrå, har muren i hjørnet.
/// Leser instansene i nivåfila, ikke valgene kompilatoren gjorde.
pub fn tight(doc: &LevelDoc, kit: &KitDef, file: &LevelFile) -> Vec<Issue> {
    let cs = cells(doc, kit);
    let opening: HashMap<&str, &str> = kit
        .pieces
        .iter()
        .filter(|p| p.kind == PieceKind::Edge)
        .flat_map(|p| std::iter::once(p.model.as_str()).chain(p.variants.iter().map(|v| v.model.as_str())).map(move |m| (m, p.opening.as_deref().unwrap_or(""))))
        .collect();
    let corners: Vec<&str> = kit.pieces.iter().filter(|p| p.kind == PieceKind::Corner).map(|p| p.model.as_str()).collect();
    let key = |p: [f32; 3], yaw: f32| {
        ((p[0] * 100.0).round() as i64, (p[1] * 100.0).round() as i64, (p[2] * 100.0).round() as i64, ((yaw.rem_euclid(std::f32::consts::TAU) / std::f32::consts::FRAC_PI_2).round() as i64) % 4)
    };
    let mut halves: HashMap<(i64, i64, i64, i64), &str> = HashMap::new();
    let mut fills: HashMap<(i64, i64, i64, i64), bool> = HashMap::new();
    for inst in &file.instances {
        if let Some(o) = opening.get(inst.model.as_str()) {
            halves.insert(key(inst.pos, inst.yaw), o);
        } else if corners.contains(&inst.model.as_str()) {
            fills.insert(key(inst.pos, inst.yaw), true);
        }
    }
    let mut cols: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (k, c) in cs.iter().enumerate() {
        cols.entry((c.i, c.j)).or_default().push(k);
    }
    let neighbor = |a: usize, w: usize| -> bool {
        let c = &cs[a];
        let (di, dj) = outward(w);
        cols.get(&(c.i + di, c.j + dj)).is_some_and(|v| v.iter().any(|&b| (cs[b].floors[opposite(w)] - c.floors[w]).abs() < 0.01))
    };
    let mut out = vec![];
    let solid = |o: Option<&&str>| o.is_some_and(|o| *o != "arch");
    for (a, c) in cs.iter().enumerate() {
        let center = cell_center(c.i, c.y, c.j);
        for w in 0..4 {
            let o = outward_v(w) * HALF;
            let at = [center.x + o.x, c.floors[w], center.z + o.y];
            let half = halves.get(&key(at, side_yaw(w)));
            let closed = half.is_some_and(|h| *h == "wall");
            if !closed && !neighbor(a, w) {
                let what = half.map(|h| format!("halvkanten «{h}»")).unwrap_or_else(|| "ingen mur".into());
                out.push(Issue { severity: Severity::Error, rule: "tett", at: Some(at), msg: format!("{} er åpen mot {} ({what}) uten noe på andre siden", c.id, SIDES[w]) });
            }
        }
        for k in 0..4 {
            let v = corner_vertex(c.i, c.j, k);
            let p = vertex_pos(v);
            // En mur som ender i hjørnet (i cella selv eller en nabo), eller
            // fjell på skrå.
            let mut need = false;
            for ((i, j), corner) in around_vertex(v) {
                let list: Vec<usize> = cols.get(&(i, j)).map(|l| l.iter().copied().filter(|&b| cs[b].lo < c.hi - 0.01 && c.lo < cs[b].hi - 0.01).collect()).unwrap_or_default();
                if list.is_empty() {
                    need = true;
                }
                for b in list {
                    let cb = &cs[b];
                    let cc = cell_center(cb.i, cb.y, cb.j);
                    for w in corner_sides(corner) {
                        let o = outward_v(w) * HALF;
                        let at = [cc.x + o.x, cb.floors[w], cc.z + o.y];
                        if solid(halves.get(&key(at, side_yaw(w)))) {
                            need = true;
                        }
                    }
                }
            }
            if need && !fills.contains_key(&key([p.x, c.y, p.y], corner_yaw(k))) {
                out.push(Issue { severity: Severity::Error, rule: "tett", at: Some([p.x, c.y, p.y]), msg: format!("{}: muren ender i lufta i hjørnet {k}", c.id) });
            }
        }
    }
    out
}

/// Brudd fra kompileringen som gjelder kantene (to sider som ikke passer,
/// åpning mot fjell, trapp som ender i vegg).
pub fn edges(issues: &[Issue]) -> Vec<&Issue> {
    issues.iter().filter(|i| i.rule == "kant").collect()
}

/// Cella ved et punkt i nivåets rom (til «Gå til»), også i modulene.
pub fn cell_at(doc: &LevelDoc, kit: &KitDef, p: Vec3) -> Option<String> {
    let (i, j) = ((p.x / CELL).round() as i32, (p.z / CELL).round() as i32);
    super::modules::flatten(doc, kit).0.cells.into_iter().find(|c| c.at[0] == i && c.at[2] == j && (p.y - c.at[1] as f32).abs() < 3.0).map(|c| c.id)
}

/// Det en spak kan åpne (bryterne), og hva hver av dem heter i meldingene.
const SWITCHES: [(&str, &str); 7] = [("gate", "porten"), ("bridge", "vindebroa"), ("water", "slusa"), ("lift", "heisen"), ("shortcut", "snarveien"), ("secret_wall", "den falske veggen"), ("trapdoor", "fallemmen")];

/// Koblingene i nivået henger sammen (mekanismene, `docs/nivaaredigering.md`
/// del 10): en spak åpner noe som kan åpnes; en port åpnes av en spak, en
/// nøkkel (eliten) eller er en arenaport; vindebroa, slusa og heisen har en
/// spak; nøkkelen bæres av en elite; et hint peker på en falsk vegg; stigen
/// går opp; trykkplata har en felle.
pub fn links(file: &LevelFile) -> Vec<Issue> {
    let kind_of = |id: &str| file.marks.iter().find(|m| m.id.as_deref() == Some(id)).map(|m| m.kind.as_str());
    let by_lever: Vec<&str> = file.marks.iter().filter(|m| m.kind == "lever").flat_map(|m| m.opens.iter().map(String::as_str)).collect();
    let by_key: Vec<&str> = file.marks.iter().filter(|m| m.kind == "elite").flat_map(|m| m.opens.iter().map(String::as_str)).collect();
    let mut out = vec![];
    let mut err = |at: Option<[f32; 3]>, msg: String| out.push(Issue { severity: Severity::Error, rule: "kobling", at, msg });
    for m in &file.marks {
        let id = m.id.clone().unwrap_or_else(|| m.kind.clone());
        let name = SWITCHES.iter().find(|(k, _)| *k == m.kind).map(|(_, n)| *n);
        match m.kind.as_str() {
            "lever" | "elite" => {
                for t in &m.opens {
                    match kind_of(t) {
                        None => err(m.pos, format!("{id} åpner «{t}», som ikke finnes")),
                        Some(k) if m.kind == "elite" && k != "gate" => err(m.pos, format!("nøkkelen til {id} passer bare i en port, ikke {k} «{t}»")),
                        Some(k) if !SWITCHES.iter().any(|(s, _)| *s == k) => err(m.pos, format!("spaken {id} kan ikke åpne {k} «{t}»")),
                        _ => {}
                    }
                }
            }
            "mob" | "boss" if !m.opens.is_empty() => err(m.pos, format!("{id} bærer en nøkkel, men bare eliten gjør det")),
            "gate" if m.arena.is_none() && !by_lever.contains(&id.as_str()) && !by_key.contains(&id.as_str()) => err(m.pos, format!("porten {id} kan ikke åpnes (ingen spak, nøkkel eller arena)")),
            "bridge" | "water" | "lift" if !by_lever.contains(&id.as_str()) => err(m.pos, format!("{} {id} har ingen spak", name.unwrap_or("bryteren"))),
            "hint" => {
                if let Some(f) = &m.for_id {
                    if kind_of(f) != Some("secret_wall") {
                        err(m.pos, format!("hintet {id} peker på «{f}», som ikke er en falsk vegg"));
                    }
                }
            }
            "ladder" if m.top.zip(m.pos).is_some_and(|(t, p)| t[1] < p[1] + 0.5) => err(m.pos, format!("stigen {id} går ikke opp")),
            "trap" if m.label.as_deref().is_some_and(|l| !["arrows", "spikes", "rocks", "pendulum", "gas", "flood"].contains(&l)) => {
                err(m.pos, format!("trykkplata {id} har ingen kjent felle («{}»)", m.label.as_deref().unwrap_or("")))
            }
            _ => {}
        }
    }
    out
}

/// Lyset følger rommene i en maske på 64 bit (`Crypt::light_mask`): rom
/// nummer 64 og over får ikke lys fra noe. Et nivå med flere rom er en feil
/// (modulgalleriet med alle temaene ble svart bakerst).
pub const MAX_ROOMS: usize = 64;

pub fn rooms(file: &LevelFile) -> Vec<Issue> {
    let n = file.rooms.len();
    (n > MAX_ROOMS)
        .then(|| Issue { severity: Severity::Error, rule: "lys", at: None, msg: format!("nivået har {n} rom; lyset følger bare de første {MAX_ROOMS}") })
        .into_iter()
        .collect()
}

/// Om (x, z) er inne i polygonet.
fn inside(poly: &[[f32; 2]], x: f32, z: f32) -> bool {
    let mut c = false;
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        if (a[1] > z) != (b[1] > z) && x < a[0] + (z - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            c = !c;
        }
    }
    c
}

/// Aldri blandet gulv i ett rom (eieren 20. oktober: «ikke bland ulike
/// teksturer på gulvet»): cellene i et rom har samme gulvlag, og en variant
/// med gravheller (`own_floor`) står bare på gulvet biten er laget med.
/// Gulvlaget er steinbyttet på instansen, ellers temaets standardstein.
pub fn floor_mix(file: &LevelFile, kit: &KitDef) -> Vec<Issue> {
    let mut out = vec![];
    let mut floors: Vec<(usize, String, [f32; 3])> = vec![];
    for inst in &file.instances {
        let Some((p, v)) = kit.listed().filter(|p| p.kind == PieceKind::Cell).find_map(|p| {
            if p.model == inst.model {
                Some((p, None))
            } else {
                p.variants.iter().find(|v| v.model == inst.model).map(|v| (p, Some(v)))
            }
        }) else {
            continue;
        };
        let Some(base) = kit.stones.base(&p.theme) else { continue };
        let floor = inst.stone.as_ref().map(|s| s.to[0].clone()).unwrap_or(base.floor.clone());
        if v.is_some_and(|v| v.own_floor) && floor != base.floor {
            out.push(Issue { severity: Severity::Error, rule: "gulv", at: Some(inst.pos), msg: format!("{} har gravheller på gulvet {floor}", inst.model) });
        }
        let at = inst.pos;
        if let Some(r) = file.rooms.iter().position(|r| (at[1] - r.y).abs() < 1.5 && inside(&r.poly, at[0], at[2])) {
            floors.push((r, floor, at));
        }
    }
    let mut seen: HashMap<usize, String> = HashMap::new();
    for (r, floor, at) in floors {
        match seen.get(&r) {
            Some(f) if *f != floor => {
                let room = &file.rooms[r];
                out.push(Issue { severity: Severity::Warning, rule: "gulv", at: Some(at), msg: format!("rommet {} har to gulv ({f} og {floor})", if room.name.is_empty() { &room.id } else { &room.name }) });
            }
            Some(_) => {}
            None => {
                seen.insert(r, floor);
            }
        }
    }
    out.sort_by(|a, b| a.msg.cmp(&b.msg));
    out.dedup_by(|a, b| a.msg == b.msg);
    out
}

/// Høyden en fiende trenger: modellene er rundt 2 m ved skala 1
/// (`Kind::stats().scale`; Navbaren og Dropdownen er 5 m, Rammeverket 4–5 m),
/// og 0,5 m over hodet til slaget.
pub fn mob_height(label: &str) -> f32 {
    2.0 * crate::domain::mob_kind::Kind::from_label(label).stats().scale + 0.5
}

/// Fiendene står i et rom med plass til høyden sin (eieren 19. oktober:
/// hjelmen til bossen gikk opp i kuppelen): takhøyden i rommet de står i
/// (`RoomDef.h`) er minst `mob_height`. Bossene trenger bosshallen.
pub fn headroom(file: &LevelFile) -> Vec<Issue> {
    let mut out = vec![];
    for m in file.marks.iter().filter(|m| matches!(m.kind.as_str(), "mob" | "elite" | "boss")) {
        let Some(p) = m.pos else { continue };
        let label = m.label.clone().unwrap_or_else(|| if m.kind == "boss" { "rammeverket".into() } else { "button".into() });
        let need = mob_height(&label);
        let Some(room) = file.rooms.iter().find(|r| (p[1] - r.y).abs() < 1.5 && inside(&r.poly, p[0], p[2])) else { continue };
        if room.h + 0.01 < need {
            let id = m.id.clone().unwrap_or_else(|| m.kind.clone());
            out.push(Issue { severity: Severity::Error, rule: "takhoyde", at: m.pos, msg: format!("{id} ({label}) trenger {need:.1} m under taket, men {} har {:.1} m (bruk bosshallen)", room.name, room.h) });
        }
    }
    out
}

/// Bitene mekanismene står i (modellene i `kit_mech.py`).
pub const CHASM: &str = "grav_kloft";
pub const SHAFT: &str = "grav_sjakt";
pub const SHAFT_TOP: &str = "grav_sjakt_topp";
pub const HATCH: &str = "grav_luke";
pub const HATCH_TOP: &str = "grav_luke_topp";
pub const CANALS: [&str; 2] = ["grav_kanal", "grav_kanal_ende"];

/// Mekanismene står i bitene sine (`docs/nivaaredigering.md` del 25): en
/// kløft har en vindebro over seg (ellers kommer ingen over), og en vindebro
/// står over en kløft; heisen står i en sjakt med bunnen i sjakta under;
/// fallemmen ligger over en luke, og stigen under den når opp til gulvet
/// over; en kanal har vann.
pub fn mechanisms(file: &LevelFile) -> Vec<Issue> {
    let at = |model: &'static str| file.instances.iter().filter(move |i| i.model == model).map(|i| Vec3::from(i.pos));
    let near = |a: Vec3, b: Vec3, r: f32| (a.x - b.x).abs() < r && (a.z - b.z).abs() < r && (a.y - b.y).abs() < 0.6;
    // Om punktet er innenfor merket (målene i `size`, dreid med merket).
    let covers = |m: &crate::domain::world::crypt::level::MarkDef, p: Vec3, dflt: (f32, f32)| {
        let Some(c) = m.pos.map(Vec3::from) else { return false };
        let (w, l) = m.size.map(|s| s.wd()).unwrap_or(dflt);
        let d = glam::Quat::from_rotation_y(-m.yaw) * (p - c);
        d.x.abs() <= w * 0.5 + 0.05 && d.z.abs() <= l * 0.5 + 0.05
    };
    let marks = |kind: &'static str| file.marks.iter().filter(move |m| m.kind == kind);
    let mut out = vec![];
    let mut put = |severity, at: Option<Vec3>, msg: String| out.push(Issue { severity, rule: "mekanisme", at: at.map(mm3), msg });
    for c in at(CHASM) {
        if !marks("bridge").any(|m| covers(m, c, (3.6, 8.0)) && m.pos.is_some_and(|p| (p[1] - c.y).abs() < 0.6)) {
            put(Severity::Warning, Some(c), "kløfta har ingen vindebro over seg; ingen kommer over".into());
        }
    }
    for m in marks("bridge") {
        let id = m.id.clone().unwrap_or_default();
        if !at(CHASM).any(|c| covers(m, c, (3.6, 8.0)) && m.pos.is_some_and(|p| (p[1] - c.y).abs() < 0.6)) {
            put(Severity::Error, m.pos.map(Vec3::from), format!("vindebroa {id} står ikke over en kløft"));
        }
    }
    for m in marks("lift") {
        let id = m.id.clone().unwrap_or_default();
        let Some(p) = m.pos.map(Vec3::from) else { continue };
        let top = at(SHAFT_TOP).any(|c| near(c, p, 0.6));
        let bottom = m.bottom.map(Vec3::from).is_some_and(|b| at(SHAFT).any(|c| near(c, b, 0.6)));
        if !top || !bottom {
            put(Severity::Error, Some(p), format!("heisen {id} står ikke i en sjakt (sjakta nede og sjaktas topp over den, med bunnen i sjakta)"));
        }
    }
    for m in marks("trapdoor") {
        let id = m.id.clone().unwrap_or_default();
        let Some(p) = m.pos.map(Vec3::from) else { continue };
        if !at(HATCH_TOP).any(|c| near(c, p, 0.3)) {
            put(Severity::Error, Some(p), format!("fallemmen {id} ligger ikke over en luke"));
        }
    }
    for m in marks("ladder") {
        let id = m.id.clone().unwrap_or_default();
        let (Some(p), Some(t)) = (m.pos.map(Vec3::from), m.top.map(Vec3::from)) else { continue };
        if let Some(c) = at(HATCH).find(|c| near(*c, p, HALF)) {
            if !at(HATCH_TOP).any(|u| near(u, Vec3::new(c.x, t.y, c.z), 0.3)) {
                put(Severity::Error, Some(p), format!("stigen {id} når ikke opp til luka over"));
            }
        }
    }
    for name in CANALS {
        for c in at(name) {
            if !marks("water").any(|m| covers(m, c, (4.0, 4.0))) {
                put(Severity::Warning, Some(c), "kanalen har ikke vann".into());
            }
        }
    }
    out
}
