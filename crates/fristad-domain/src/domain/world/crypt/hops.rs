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
/// Trinnene i hver bit av stigen (`kit_grav.LADDER_RUNGS`), og tykkelsen
/// på trinnet (radien, `kit_grav.kit_ladder`): foten står oppå det.
pub const LADDER_RUNGS: f32 = 3.0;
const RUNG_RADIUS: f32 = 0.021;
/// Fasen i klatreklippet der venstre fot står nederst på trinnet sitt
/// (`mob_motion.climb`: `limbs`, foten løfter seg i 0,06), og høyre fot en
/// halv syklus (ett trinn) senere. Hver syklus stiger fienden to trinn.
const CLIMB_FOOT_PHASE: f32 = 0.06;

/// Hvor langt klatreklippet er kommet (0..1) når fienden står `height` over
/// stigens fot og trinnene står `rung` fra hverandre: regnet fra høyden, ikke
/// fra klokka, så føttene står på trinnene uansett hvor fienden begynte å
/// klatre (grepene står like langt fra hverandre som trinnene).
pub fn climb_phase(height: f32, rung: f32) -> f32 {
    let rung = rung.max(0.05);
    ((height - RUNG_RADIUS - 0.5 * rung) / (2.0 * rung) + CLIMB_FOOT_PHASE).rem_euclid(1.0)
}

/// Trinnavstanden i en stige på `h` meter (bitene strekkes litt, `ladder_pieces`).
pub fn ladder_rung(h: f32) -> f32 {
    super::ladder_pieces(h).1 / LADDER_RUNGS
}
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
/// skal kjøres når den begynner på steget. `climb` er steget opp eller ned
/// selve stigen (klippet `climb`), og `face` er retningen mot stigen
/// (vannrett), som fienden vender ansiktet mot mens den klatrer. `foot` er
/// høyden på stigens fot og `rung` trinnavstanden (`climb_phase`).
#[derive(Clone, Copy, Debug)]
pub struct HopStep {
    pub to: Vec3,
    pub speed: f32,
    pub ride: Option<usize>,
    pub climb: bool,
    pub face: Vec3,
    pub foot: f32,
    pub rung: f32,
}

/// Retningen mot stigen i lenken: fra toppen av den loddrette delen inn
/// mot toppen av stigen (`ladder_points`: foran foten, rett opp, inn).
fn ladder_face(pts: &[Vec3]) -> Vec3 {
    pts.windows(3)
        .find(|w| {
            let d = w[1] - w[0];
            d.y.abs() > d.with_y(0.0).length()
        })
        .map(|w| if w[1].y > w[0].y { (w[2] - w[1]).with_y(0.0) } else { (w[0] - w[1]).with_y(0.0) })
        .unwrap_or(Vec3::ZERO)
        .normalize_or_zero()
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
                let face = if h.kind == HopKind::Ladder { ladder_face(&h.pts) } else { Vec3::ZERO };
                let (lo, hi) = h.pts.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p.y), hi.max(p.y)));
                let rung = ladder_rung((hi - lo).max(0.5));
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
                            HopStep { to: w[1], speed, ride, climb: up && h.kind == HopKind::Ladder, face, foot: lo, rung }
                        })
                        .collect(),
                );
            }
        }
        None
    }
}
