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

use crate::domain::world::crypt::compose::{check, Issue, Severity};
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::level::LevelFile;
use crate::domain::world::crypt::nav::Nav;
use crate::domain::world::crypt::HEADROOM;
use glam::Vec3;

/// Innholdet som må kunne nås fra start.
const MUST_REACH: [&str; 13] = ["up", "down", "rest", "chest", "rare", "lever", "lore", "bonus", "mob", "elite", "boss", "gate", "shortcut"];

fn issue(severity: Severity, rule: &'static str, at: Option<[f32; 3]>, msg: String) -> Issue {
    Issue { severity, rule, at, msg }
}

/// Bruddene i nivåfila `file`, kompilert fra `doc`.
pub fn check(doc: &LevelDoc, file: &LevelFile) -> Vec<Issue> {
    let mut out = vec![];
    let _ = doc;
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
    let seen = nav.reachable(a, &all);
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
