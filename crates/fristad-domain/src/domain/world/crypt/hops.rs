//! Stigene og heisene i navigasjonsnettet (`nav::Hop`, «off-mesh links»):
//! fiendene klatrer opp og ned stigen (`CLIMB_SPEED`) og kjører heisen når
//! den står der de er og bremsen er løsnet (spaken, som broene bak
//! bryteren). Fallemmen over en stige stenger den til lemmen er åpnet
//! (nedenfra, av deg); fiendene åpner ingen lemmer. Redigeringens regel og
//! testene bruker `level_nav`, der alle lenkene er åpne.

use super::level::LevelFile;
use super::nav::{Hop, HopKind, Nav};
use super::Crypt;
use crate::domain::world::mechanisms::LIFT_SPEED;
use glam::Vec3;

/// Så fort fiendene klatrer i stigen (m/s).
pub const CLIMB_SPEED: f32 = 1.0;
/// Så langt fra endene lenken kobles til nettet (stigen, heisen: dekket er
/// 2,8 m bredt, og nettet holdes unna sjakta).
const LADDER_REACH: f32 = 2.0;
const LIFT_REACH: f32 = 2.5;
/// Fallemmen så nær toppen av stigen hører til den (som `Use::Ladder`).
const TRAPDOOR_NEAR: f32 = 2.5;

/// Veien langs stigen: foran foten, rett opp, og inn på toppen.
pub fn ladder_points(foot: Vec3, yaw: f32, top: Vec3) -> Vec<Vec3> {
    let front = foot + Vec3::new(yaw.sin(), 0.0, yaw.cos()) * 0.4;
    vec![front, front.with_y(top.y), top]
}

/// Veien med heisen: midt på dekket oppe og nede.
pub fn lift_points(deck: Vec3, drop: f32) -> Vec<Vec3> {
    vec![deck, deck - Vec3::Y * drop]
}

/// Lenkene i nivå `k` (stigene og heisene som allerede er lest inn).
pub(super) fn link(c: &Crypt, k: usize, nav: &mut Nav) {
    for l in c.ladders.iter().filter(|l| l.level == k) {
        let gate = c.trapdoors.iter().find(|t| t.level == k && t.pos.distance(l.top) < TRAPDOOR_NEAR).map(|t| t.switch);
        if !nav.add_hop(HopKind::Ladder, &ladder_points(l.pos, l.yaw, l.top), gate, LADDER_REACH) {
            log::warn!("Stigen ved {:?} når ikke nettet i begge ender", l.pos);
        }
    }
    for (n, l) in c.lifts.iter().enumerate().filter(|(_, l)| l.level == k) {
        if !nav.add_hop(HopKind::Lift(n), &lift_points(l.pos, l.drop), Some(l.switch), LIFT_REACH) {
            log::warn!("Heisen ved {:?} når ikke nettet i begge ender", l.pos);
        }
    }
}

/// Nettet i nivåfila (i fila sitt eget rom) med stigene og heisene som
/// lenker, uten rom og brytere: det redigeringens nåbarhetsregel går over
/// (`compose::nav::reach`).
pub fn level_nav(file: &LevelFile) -> Nav {
    let verts: Vec<Vec3> = file.nav.verts.iter().map(|v| Vec3::from(*v)).collect();
    let n = file.nav.polys.len();
    let mut nav = Nav::new(&verts, &file.nav.polys, &vec![None; n], &vec![None; n]);
    let mut lifts = 0;
    for m in &file.marks {
        let Some(pos) = m.pos.map(Vec3::from) else { continue };
        match m.kind.as_str() {
            "ladder" => {
                let top = m.top.map(Vec3::from).unwrap_or(pos + Vec3::Y * 3.0);
                nav.add_hop(HopKind::Ladder, &ladder_points(pos, m.yaw, top), None, LADDER_REACH);
            }
            "lift" => {
                let drop = m.bottom.map(|b| pos.y - b[1]).unwrap_or(10.0).max(0.5);
                nav.add_hop(HopKind::Lift(lifts), &lift_points(pos, drop), None, LIFT_REACH);
                lifts += 1;
            }
            _ => {}
        }
    }
    nav
}

/// Et steg langs en lenke for fienden: punktet, farten dit, og heisen som
/// skal kjøres når den begynner på steget.
#[derive(Clone, Copy, Debug)]
pub struct HopStep {
    pub to: Vec3,
    pub speed: f32,
    pub ride: Option<usize>,
}

impl Crypt {
    /// Hvor heis `n` står: `Some(nede)` i ro, `None` mens den kjører.
    pub fn lift_rest(&self, n: usize) -> Option<bool> {
        self.lift_rest.get(n).copied().unwrap_or(Some(false))
    }

    /// Om lenken kan brukes i den retningen nå: bryteren er åpen (lemmen,
    /// bremsen), og heisen står i ro i enden du går fra (forover er ned).
    pub fn hop_open(&self, h: &Hop, forward: bool) -> bool {
        h.gate.is_none_or(|g| self.open.get(g).copied().unwrap_or(true))
            && match h.kind {
                HopKind::Ladder => true,
                HopKind::Lift(n) => self.lift_rest(n) == Some(!forward),
            }
    }

    pub(super) fn open_hop<'a>(&'a self) -> impl Fn(&Hop, bool) -> bool + 'a {
        move |h, f| self.hop_open(h, f)
    }

    /// Står fienden i `at` ved starten på en åpen lenke, og er `next` neste
    /// punkt på den? Gir stegene videre langs lenken (gangfarten `walk` bort
    /// til stigen og dekket, stigefarten og heisfarten opp og ned).
    pub fn hop_ahead(&self, at: Vec3, next: Vec3, walk: f32) -> Option<Vec<HopStep>> {
        let l = self.level_at(at)?;
        let nav = &self.levels[l].nav;
        for h in &nav.hops {
            for fwd in [true, false] {
                let pts = h.oriented(fwd);
                if pts.len() < 2 {
                    continue;
                }
                let near = pts[0].with_y(0.0).distance(at.with_y(0.0)) < 0.35 && (pts[0].y - at.y).abs() < 0.7;
                if !near || pts[1].distance(next) > 0.05 || !self.hop_open(h, fwd) {
                    continue;
                }
                return Some(
                    pts.windows(2)
                        .map(|w| {
                            let d = w[1] - w[0];
                            let up = d.y.abs() > d.with_y(0.0).length();
                            let speed = match (up, h.kind) {
                                (false, _) => walk,
                                (true, HopKind::Ladder) => CLIMB_SPEED,
                                (true, HopKind::Lift(_)) => LIFT_SPEED,
                            };
                            let ride = match h.kind {
                                HopKind::Lift(n) if up => Some(n),
                                _ => None,
                            };
                            HopStep { to: w[1], speed, ride }
                        })
                        .collect(),
                );
            }
        }
        None
    }
}
