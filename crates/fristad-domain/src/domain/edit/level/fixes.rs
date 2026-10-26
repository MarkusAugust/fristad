//! «Rett» (`docs/nivaaredigering.md` del 27): for de vanlige bruddene en
//! retting ved bruddet når den er entydig. Det som endrer dokumentet, er en
//! vanlig `LevelOp` (så det er én `Change` med angre); resten er hjelp i
//! siden (fokuser et felt, velg en modul i paletten, vis et sted).
//! - en åpning eieren har valgt som ikke møter noe → mur (`vegg`),
//! - en spak uten mål → kobles til nærmeste port, bro, sluse eller heis
//!   som ingen spak åpner ennå (ellers den nærmeste),
//! - en tom lapp → tekstfeltet,
//! - en fiende i et for lavt rom → bosshallen i paletten,
//! - en modul ingen kommer til → den av åpningene dens som er nærmest det
//!   man kommer til fra start.

use super::{footprint, LevelOp};
use crate::domain::world::crypt::compose::{check::NEEDED, modules, outward, reach, Issue, CELL, HALF};
use crate::domain::world::crypt::doc::LevelDoc;
use crate::domain::world::crypt::kit::KitDef;
use crate::domain::world::crypt::level::LevelFile;
use glam::Vec3;
use serde::Serialize;

/// En retting: knappen (`label`) og hva den gjør.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Fix {
    pub label: String,
    #[serde(flatten)]
    pub what: FixKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "do", rename_all = "snake_case")]
pub enum FixKind {
    /// Gjør handlingen (med angre).
    Op { op: LevelOp },
    /// Velg innholdet og fokuser feltet (`text`).
    Focus { thing: String, field: &'static str },
    /// Velg modulen i paletten, klar til å settes ut.
    Palette { module: String },
    /// Gå til stedet og vis det (åpningen i modulen `module`).
    Show { at: [f32; 3], module: String },
    /// Ta tingen (`kind`, som `start` eller `rest`) fra paletten, klar til å
    /// settes ut.
    Arm { kind: &'static str },
}

/// Det lever kan åpne (`compose::check::SWITCHES` utenom snarveien og den
/// falske veggen, som åpnes uten spak), nærmest først blant dem uten spak.
const LEVER_TARGETS: [&str; 4] = ["gate", "bridge", "water", "lift"];

fn near(a: Option<[f32; 3]>, b: [f32; 3], d: f32) -> bool {
    a.is_some_and(|a| (Vec3::from(a) - Vec3::from(b)).length() < d)
}

/// Rettingen for hvert brudd (samme rekkefølge som `issues`).
pub fn suggest(doc: &LevelDoc, kit: &KitDef, file: &LevelFile, issues: &[Issue]) -> Vec<Option<Fix>> {
    let mut reached: Option<Option<(crate::domain::world::crypt::nav::Nav, Vec<bool>)>> = None;
    issues
        .iter()
        .map(|i| {
            if i.rule == "innhold" && i.msg.starts_with("Nivået mangler") {
                let (kind, ..) = NEEDED.iter().find(|(_, name, _)| i.msg.ends_with(name))?;
                return Some(Fix { label: "Sett ut".into(), what: FixKind::Arm { kind } });
            }
            let at = i.at?;
            match i.rule {
                "kant" | "aapning" => wall_up(doc, kit, at),
                "kobling" => lever(file, at),
                "innhold" => file
                    .marks
                    .iter()
                    .find(|m| m.kind == "lore" && near(m.pos, at, 0.05) && m.text.as_deref().unwrap_or("").trim().is_empty())
                    .and_then(|m| Some(Fix { label: "Skriv teksten".into(), what: FixKind::Focus { thing: m.id.clone()?, field: "text" } })),
                "takhoyde" => {
                    let theme = if doc.theme.is_empty() { "grav" } else { doc.theme.as_str() };
                    let module = [format!("{theme}/bosshall"), "grav/bosshall".to_string()].into_iter().find(|m| kit.module(m).is_some())?;
                    Some(Fix { label: "Velg bosshallen".into(), what: FixKind::Palette { module } })
                }
                "naa" if i.msg.contains("kan ikke nås fra start (ingen åpning") => {
                    let r = reached.get_or_insert_with(|| file.marks.iter().find(|m| m.kind == "start").and_then(|m| m.pos).and_then(|s| reach(file, Vec3::from(s))));
                    nearest_opening(doc, kit, at, r.as_ref())
                }
                _ => None,
            }
        })
        .collect()
}

/// Åpningen med et valgt slag i midten av kanten `at` blir mur.
fn wall_up(doc: &LevelDoc, kit: &KitDef, at: [f32; 3]) -> Option<Fix> {
    for m in &doc.modules {
        for o in modules::placed_openings(m, kit) {
            let (di, dj) = outward(o.side);
            let mid = Vec3::new(o.at[0] as f32 * CELL + di as f32 * HALF, o.y as f32, o.at[1] as f32 * CELL + dj as f32 * HALF);
            if (mid - Vec3::from(at)).length() > 0.3 {
                continue;
            }
            let chosen = m.overrides.iter().find(|v| v.opening == o.id)?;
            if chosen.kind == "vegg" || chosen.kind == "auto" {
                return None;
            }
            return Some(Fix { label: "Gjør til mur".into(), what: FixKind::Op { op: LevelOp::Opening { module: m.id.clone(), opening: o.id.clone(), kind: "vegg".into(), id: None } } });
        }
    }
    None
}

/// En spak uten mål kobles til nærmeste bryter den kan åpne.
fn lever(file: &LevelFile, at: [f32; 3]) -> Option<Fix> {
    let l = file.marks.iter().find(|m| m.kind == "lever" && m.opens.is_empty() && near(m.pos, at, 0.05))?;
    let from = l.id.clone()?;
    let p = Vec3::from(l.pos?);
    let opened: Vec<&str> = file.marks.iter().filter(|m| m.kind == "lever").flat_map(|m| m.opens.iter().map(String::as_str)).collect();
    let mut targets: Vec<(bool, f32, &str, &str)> = file
        .marks
        .iter()
        .filter(|m| LEVER_TARGETS.contains(&m.kind.as_str()))
        .filter_map(|m| {
            let id = m.id.as_deref()?;
            Some((opened.contains(&id), (Vec3::from(m.pos?) - p).length(), id, m.kind.as_str()))
        })
        .collect();
    targets.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let (_, _, to, kind) = targets.first()?;
    let name = match *kind {
        "gate" => "porten",
        "bridge" => "broa",
        "water" => "slusa",
        _ => "heisen",
    };
    Some(Fix { label: format!("Koble til {name} {to}"), what: FixKind::Op { op: LevelOp::Link { from, to: to.to_string(), kind: "opens".into() } } })
}

/// Åpningen i modulen ved `at` som er nærmest det man kommer til fra start.
fn nearest_opening(doc: &LevelDoc, kit: &KitDef, at: [f32; 3], reached: Option<&(crate::domain::world::crypt::nav::Nav, Vec<bool>)>) -> Option<Fix> {
    let cell = [(at[0] / CELL).round() as i32, at[1].round() as i32, (at[2] / CELL).round() as i32];
    let m = doc.modules.iter().find(|m| footprint(m, kit).contains(&cell))?;
    let seen: Vec<Vec3> = reached.map(|(nav, seen)| nav.polys.iter().zip(seen).filter(|(_, s)| **s).map(|(p, _)| p.center).collect()).unwrap_or_default();
    let best = modules::placed_openings(m, kit)
        .into_iter()
        .map(|o| {
            let (di, dj) = outward(o.side);
            let mid = Vec3::new(o.at[0] as f32 * CELL + di as f32 * HALF, o.y as f32, o.at[1] as f32 * CELL + dj as f32 * HALF);
            let d = seen.iter().map(|c| (*c - mid).length()).fold(f32::INFINITY, f32::min);
            (d, mid)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))?;
    Some(Fix { label: "Vis nærmeste åpning".into(), what: FixKind::Show { at: best.1.to_array(), module: m.id.clone() } })
}
