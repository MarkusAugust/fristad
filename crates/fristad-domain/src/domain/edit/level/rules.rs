//! Reglene over det kompilerte nivået som kompilatoren ikke sjekker selv
//! (`docs/nivaaredigering.md` del 10), de samme som testene:
//! - alt viktig kan nås fra start med alle porter åpne
//!   (`kryptnivaaene_kan_gas_gjennom`, `fiendene_finner_fram_i_krypten`),
//! - koblingene henger sammen (`compose::check::links`: en spak åpner noe
//!   som finnes, en port kan åpnes, nøkkelen bæres av eliten …), og
//!   mekanismene står i bitene sine (`check::mechanisms`),
//! - fiendene har takhøyde (`check::headroom`: bossene trenger
//!   bosshallen), og nivået har høyst 64 rom (`check::rooms`),
//! - taket står minst `HEADROOM` over gulvet i hvert rom
//!   (`kryptnivaaene_har_det_de_skal_og_taket_er_hoyt_nok`).
//!
//! Usynlige vegger (`ingen_usynlige_vegger_i_krypten`) trenger fysikken og
//! sjekkes i testene, ikke her.

use crate::domain::world::crypt::compose::{check, modules, Issue, Report, Severity, CELL};
use crate::domain::world::crypt::kit::KitDef;
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::level::LevelFile;
use crate::domain::world::crypt::nav::Nav;
use crate::domain::world::crypt::HEADROOM;
use super::footprint;
use glam::Vec3;

/// Innholdet som må kunne nås fra start.
const MUST_REACH: [&str; 13] = ["up", "down", "rest", "chest", "rare", "lever", "lore", "bonus", "mob", "elite", "boss", "gate", "shortcut"];

fn issue(severity: Severity, rule: &'static str, at: Option<[f32; 3]>, msg: String) -> Issue {
    Issue { severity, rule, at, msg }
}

/// Bruddene i nivåfila `file`, kompilert fra `doc` med byggesettet `kit`
/// (`report`: kompilatorens rapport med åpningene etter sammenføyningen).
pub fn check(doc: &LevelDoc, kit: &KitDef, file: &LevelFile, report: &Report) -> Vec<Issue> {
    let mut out = vec![];
    stairs(doc, kit, report, &mut out);
    // Koblingene, takhøyden for fiendene, mekanismene og høyst 64 rom
    // (kompilatorens regler, de samme som `src/mechanism_tests.rs`).
    out.extend(check::links(file));
    out.extend(check::headroom(file));
    out.extend(check::mechanisms(file));
    out.extend(check::rooms(file));
    for m in &file.marks {
        let id = m.id.clone().unwrap_or_default();
        match m.kind.as_str() {
            "lever" if m.opens.is_empty() => out.push(issue(Severity::Error, "kobling", m.pos, format!("spaken {id} åpner ingenting (koble den til en port, bro eller sluse)"))),
            "elite" if m.opens.is_empty() => out.push(issue(Severity::Warning, "kobling", m.pos, format!("eliten {id} bærer ingen nøkkel"))),
            "lore" if m.text.as_deref().unwrap_or("").trim().is_empty() => out.push(issue(Severity::Error, "innhold", m.pos, format!("lappen {id} er tom"))),
            _ => {}
        }
    }
    // Takhøyden.
    for r in &file.rooms {
        if r.h + 0.01 < HEADROOM {
            let at = r.poly.first().map(|p| [p[0], r.y, p[1]]);
            out.push(issue(Severity::Error, "tak", at, format!("taket i {} er {:.1} m over gulvet (minst {HEADROOM} m)", if r.name.is_empty() { &r.id } else { &r.name }, r.h)));
        }
    }
    // Nåbarheten.
    let verts: Vec<Vec3> = file.nav.verts.iter().map(|v| Vec3::from(*v)).collect();
    let n = file.nav.polys.len();
    let nav = Nav::new(&verts, &file.nav.polys, &vec![None; n], &vec![None; n]);
    let all = |_: &crate::domain::world::crypt::nav::Poly| true;
    let Some(start) = file.marks.iter().find(|m| m.kind == "start").and_then(|m| m.pos) else {
        return out;
    };
    let Some((a, _)) = nav.nearest(Vec3::from(start), 1.5, &all) else {
        out.push(issue(Severity::Error, "naa", Some(start), "startstedet står ikke på gulvet".into()));
        return out;
    };
    // Stigene og heisene kobler etasjene (de er ikke i nettet): det som
    // kan nås i den ene enden, kan nås i den andre.
    let mut seen = nav.reachable(a, &all);
    let ends: Vec<(Vec3, Vec3)> = file
        .marks
        .iter()
        .filter_map(|m| match m.kind.as_str() {
            "ladder" => Some((Vec3::from(m.pos?), Vec3::from(m.top?))),
            "lift" => Some((Vec3::from(m.pos?), Vec3::from(m.bottom?))),
            _ => None,
        })
        .collect();
    let mut grew = true;
    while grew {
        grew = false;
        for (p, q) in &ends {
            let (Some((i, _)), Some((j, _))) = (nav.nearest(*p, 2.0, &all), nav.nearest(*q, 2.0, &all)) else { continue };
            for (from, to) in [(i, j), (j, i)] {
                if seen[from as usize] && !seen[to as usize] {
                    for (k, s) in nav.reachable(to, &all).into_iter().enumerate() {
                        seen[k] |= s;
                    }
                    grew = true;
                }
            }
        }
    }
    // Modulene: hver har minst én rute man kommer til fra start.
    for m in &doc.modules {
        let cells = footprint(m, kit);
        let reached = cells.iter().any(|c| nav.nearest(Vec3::new(c[0] as f32 * CELL, c[1] as f32 + 0.3, c[2] as f32 * CELL), CELL * 0.5, &all).is_some_and(|(k, _)| seen[k as usize]));
        if !reached && !cells.is_empty() {
            let c = cells[0];
            let name = m.name.clone().unwrap_or_else(|| m.id.clone());
            out.push(issue(Severity::Warning, "naa", Some([c[0] as f32 * CELL, c[1] as f32, c[2] as f32 * CELL]), format!("{name} kan ikke nås fra start (ingen åpning, trapp, stige eller heis dit)")));
        }
    }
    for m in file.marks.iter().filter(|m| MUST_REACH.contains(&m.kind.as_str())) {
        let Some(p) = m.pos else { continue };
        let reach = if matches!(m.kind.as_str(), "gate" | "shortcut") { 2.5 } else { 2.0 };
        let id = m.id.clone().unwrap_or_else(|| m.kind.clone());
        match nav.nearest(Vec3::from(p), reach, &all) {
            None => out.push(issue(Severity::Error, "naa", Some(p), format!("{id} ({}) står ikke på gulvet", m.kind))),
            Some((k, _)) if !seen[k as usize] => out.push(issue(Severity::Error, "naa", Some(p), format!("{id} ({}) kan ikke nås fra start", m.kind))),
            _ => {}
        }
        for q in &m.patrol {
            if !nav.nearest(Vec3::from(*q), 1.5, &all).is_some_and(|(k, _)| seen[k as usize]) {
                out.push(issue(Severity::Error, "naa", Some(*q), format!("patruljepunktet til {id} kan ikke nås")));
            }
        }
    }
    out
}

/// Trappene henger sammen (eieren: «trappa ned og opp henger sammen»): en
/// trapp (halvtrapp, halv etasje) går et sted både øverst og nederst, så
/// den kobler to etasjer og ikke ender i en mur.
fn stairs(doc: &LevelDoc, kit: &KitDef, report: &Report, out: &mut Vec<Issue>) {
    const STEP: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    // Kanten mellom ruta og naboen er åpen, sett fra hvilken som helst av sidene.
    let joined = |at: [i32; 2], side: usize| {
        report.edges.iter().any(|e| {
            let (di, dj) = STEP[e.side];
            (e.at[0], e.at[2], e.side) == (at[0], at[1], side) || ((e.at[0] + di, e.at[2] + dj) == (at[0], at[1]) && (e.side + 2) % 4 == side)
        })
    };
    for m in &doc.modules {
        let Some(def) = kit.module(&m.module) else { continue };
        if def.kind != "trapp" || def.things.iter().any(|t| t.kind == "down") {
            continue;
        }
        let openings = modules::placed_openings(m, kit);
        let mut ys: Vec<i32> = openings.iter().map(|o| o.y).collect();
        ys.sort();
        ys.dedup();
        if ys.len() < 2 {
            continue;
        }
        let name = m.name.clone().unwrap_or_else(|| def.name.clone());
        for (k, y) in [(0, ys[0]), (1, ys[ys.len() - 1])] {
            let here: Vec<_> = openings.iter().filter(|o| o.y == y).collect();
            if !here.iter().any(|o| joined(o.at, o.side)) {
                let o = here[0];
                let (di, dj) = STEP[o.side];
                let at = [o.at[0] as f32 * CELL + di as f32 * CELL * 0.5, y as f32, o.at[1] as f32 * CELL + dj as f32 * CELL * 0.5];
                out.push(issue(Severity::Warning, "trapp", Some(at), format!("{} ({}) går ikke noen vei {} ({y} m)", name, m.id, if k == 0 { "nederst" } else { "øverst" })));
            }
        }
    }
}
