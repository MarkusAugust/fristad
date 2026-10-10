//! Innbyggerne rundt torget og byfolket: hver sin riggede figur
//! (`tools/models/people.py`, `folk.py`, ellers den felles `npc.py`) med
//! klippene fra `tools/models/folk_motion.py`: idle, walk, talk, wave, og
//! når figuren har dem, stop/stop_r (stopp fra hælnedslaget på venstre og
//! høyre fot), turn_l/turn_r (snu på stedet), look og fidget (småting i
//! hvile). Farten i gangen leses av klippet (`Rig::stride_speed`), så
//! føttene står stille på bakken. Her bor gangen, stoppet, snuingen og
//! samtalen som pågår.

use crate::domain::anim::{Animator, Rig};
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Mat4, Quat, Vec3};

pub const NPC_MODEL: &str = "npc";
pub const MAX_NPCS: usize = 16;
/// Så lenge en replikk står.
const LINE_SECS: f32 = 5.5;
/// Så langt unna kan du være og fortsatt snakke.
const TALK_RANGE: f32 = 4.5;

#[derive(Clone, Debug)]
pub struct Npc {
    pub name: &'static str,
    pub pos: Vec3,
    /// Retningen (yaw i spillets konvensjon: front er (sin, 0, cos)).
    pub yaw: f32,
    pub tint: Vec3,
    pub lines: &'static [&'static str],
    /// Modellen (en egen figur, eller den felles `NPC_MODEL`) og skjelettet
    /// med klippene i `Npcs.rigs`.
    pub model: &'static str,
    rig: Option<usize>,
    /// Hvilken replikk som står for tur.
    next_line: usize,
    anim: Option<Animator>,
    /// Replikken som vises nå, og hvor lenge den har stått.
    pub speaking: Option<(usize, f32)>,
    /// Vinker når du kommer nær første gang.
    waved: bool,
    /// Gjøremålene: punktene innbyggeren går mellom (minst to, ellers står
    /// hen), hvilket som står for tur, og hvor lenge hen blir stående.
    route: Vec<(Vec3, f32)>,
    leg: usize,
    wait: f32,
    gait: Gait,
    /// Fasen i gangen forrige bilde (til å se når en hæl treffer bakken).
    last_phase: f32,
    /// Tida til neste lille gest i hvile (se seg om, rette på beltet).
    fidget: f32,
    fidgets: usize,
}

/// Hva beina gjør: står, starter, går, bremser (stoppklippet, med tida i
/// det og farten det begynte med) eller snur på stedet.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Gait {
    Stand,
    /// Startklippet: akselererer jevnt fra stillstand til full fart.
    Start { t: f32 },
    Walk,
    Stop { t: f32, v0: f32, at_goal: bool },
    /// Snur på stedet: mot deg, eller mot neste mål før hen går (`walk`).
    Turn { walk: bool },
}

#[derive(Clone, Copy, Debug, Default)]
struct Clips {
    idle: usize,
    talk: usize,
    wave: usize,
    walk: usize,
    /// Fra stillstand til gange (ender i gangens fase 0).
    start: Option<usize>,
    /// Stopp fra hælnedslaget på venstre og høyre fot (folk_motion.py).
    stop: Option<(usize, usize)>,
    /// Snu på stedet mot venstre og høyre.
    turn: Option<(usize, usize)>,
    look: Option<usize>,
    fidget: Option<usize>,
    /// Farten gangen er laget for (målt i klippet), og lengden på en
    /// gangsyklus og et snuklipp.
    speed: f32,
    cycle: f32,
    turn_len: f32,
}

/// Ganghastigheten (m/s) når klippet ikke sier noe, og hvor fort de snur seg.
const WALK_SPEED: f32 = 0.95;
const TURN: f32 = 4.0;
/// Andelen av gangsyklusen en fot står; stoppet bremser jevnt over så lang
/// tid (samme tall som `DUTY` i tools/models/folk_motion.py).
const DUTY: f32 = 0.6;
/// Andelen av gangsyklusen starten akselererer over (`START` i folk_motion.py).
const START: f32 = 0.4;
/// Så mye dreier figuren per snuklipp (`TURN_DEG` i folk_motion.py).
const TURN_DEG: f32 = 60.0;
/// Snur på stedet med skritt når den skal dreie mer enn dette (radianer).
const TURN_STEP: f32 = 0.45;

#[derive(Default)]
pub struct Npcs {
    pub npcs: Vec<Npc>,
    rigs: Vec<(Rig, Clips)>,
}

/// Det som sies dette bildet: hvem, hvor (over hodet) og hva.
#[derive(Clone, Debug, PartialEq)]
pub struct Speech {
    pub npc: usize,
    pub name: &'static str,
    pub pos: Vec3,
    pub text: String,
}

fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn clips_of(r: &Rig) -> Clips {
    let find = |n: &str| r.clip(n);
    let idle = find("idle").unwrap_or(0);
    let walk = find("walk").unwrap_or(idle);
    let cycle = r.clips.get(walk).map(|c| c.duration).unwrap_or(1.0).max(0.2);
    let speed = if walk != idle { r.stride_speed(walk, &["foot.L", "foot.R"]).filter(|v| (0.3..2.5).contains(v)).unwrap_or(WALK_SPEED) } else { WALK_SPEED };
    let turn = find("turn_l").zip(find("turn_r"));
    let turn_len = turn.and_then(|(l, _)| r.clips.get(l)).map(|c| c.duration).unwrap_or(1.0).max(0.2);
    Clips {
        idle,
        talk: find("talk").unwrap_or(idle),
        wave: find("wave").unwrap_or(idle),
        walk,
        start: find("start"),
        stop: find("stop").zip(find("stop_r")),
        turn,
        look: find("look"),
        fidget: find("fidget"),
        speed,
        cycle,
        turn_len,
    }
}

impl Npc {
    /// Klippet når hen står: prater eller hviler.
    fn rest(&mut self, clips: &Clips, fade: f32) {
        let clip = if self.speaking.is_some() { clips.talk } else { clips.idle };
        if let Some(a) = self.anim.as_mut() {
            a.play(clip, true, fade);
        }
    }

    /// Begynner å stoppe ved hælnedslaget som nettopp var (`half` er hvor
    /// langt i den halve syklusen gangen er kommet, 0..½): stoppklippet for
    /// den foten, spolt like langt fram.
    fn begin_stop(&mut self, clips: &Clips, phase: f32, at_goal: bool) {
        let Some((left, right)) = clips.stop else {
            self.gait = Gait::Stand;
            self.rest(clips, 0.25);
            return;
        };
        let half = phase % 0.5;
        let clip = if phase < 0.5 { left } else { right };
        let t = half * clips.cycle;
        if let Some(a) = self.anim.as_mut() {
            a.play(clip, false, 0.12);
            a.time = t;
        }
        self.gait = Gait::Stop { t, v0: clips.speed, at_goal };
    }
}

impl Npc {
    /// Går mot neste mål: snur seg først om det er langt å dreie, så
    /// startklippet (eller rett i gangen om figuren ikke har det).
    fn set_off(&mut self, clips: &Clips) {
        let Some((goal, _)) = self.route.get(self.leg).copied() else { return };
        let flat = (goal - self.pos).with_y(0.0);
        let diff = wrap(flat.x.atan2(flat.z) - self.yaw);
        if let (Some((left, right)), true) = (clips.turn, diff.abs() > TURN_STEP * 1.5) {
            self.gait = Gait::Turn { walk: true };
            if let Some(a) = self.anim.as_mut() {
                a.play(if diff > 0.0 { left } else { right }, true, 0.25);
            }
            return;
        }
        self.last_phase = 0.0;
        match clips.start {
            Some(c) => {
                self.gait = Gait::Start { t: 0.0 };
                if let Some(a) = self.anim.as_mut() {
                    a.play(c, false, 0.25);
                }
            }
            None => {
                self.gait = Gait::Walk;
                if let Some(a) = self.anim.as_mut() {
                    a.play(clips.walk, true, 0.3);
                }
            }
        }
    }
}

impl Npcs {
    /// `rig_of` gir skjelettet til en modell; hver innbygger har sin.
    pub fn new(rig_of: impl Fn(&str) -> Option<Rig>, cast: Vec<(&'static str, &'static str, Vec3, f32, Vec3, &'static [&'static str], Vec<(Vec3, f32)>)>) -> Self {
        let mut rigs: Vec<(Rig, Clips)> = vec![];
        let mut by_model: Vec<(&'static str, Option<usize>)> = vec![];
        let mut npcs = vec![];
        for (i, (name, model, pos, yaw, tint, lines, route)) in cast.into_iter().enumerate().take(MAX_NPCS) {
            let rig = match by_model.iter().find(|(m, _)| *m == model) {
                Some((_, r)) => *r,
                None => {
                    let r = rig_of(model).map(|r| {
                        let c = clips_of(&r);
                        rigs.push((r, c));
                        rigs.len() - 1
                    });
                    by_model.push((model, r));
                    r
                }
            };
            let mut anim = rig.map(|r| Animator::new(rigs[r].1.idle, true));
            if let Some(a) = &mut anim {
                a.time = i as f32 * 0.9;
            }
            npcs.push(Npc {
                name,
                pos,
                yaw,
                tint,
                lines,
                model,
                rig,
                next_line: 0,
                anim,
                speaking: None,
                waved: false,
                route,
                leg: 0,
                wait: 2.0 + i as f32 * 1.3,
                gait: Gait::Stand,
                last_phase: 0.0,
                fidget: 6.0 + (i * 5 % 7) as f32,
                fidgets: i,
            });
        }
        Self { npcs, rigs }
    }

    pub fn reserve() -> Vec<(&'static str, usize)> {
        vec![(NPC_MODEL, MAX_NPCS)]
    }

    /// F på en innbygger: neste replikk. Gir det som ble sagt.
    pub fn talk(&mut self, i: usize) -> Option<Speech> {
        let n = self.npcs.get_mut(i)?;
        let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
        if n.lines.is_empty() {
            return None;
        }
        let line = n.next_line % n.lines.len();
        n.next_line = line + 1;
        n.speaking = Some((line, 0.0));
        // Står hen, begynner praten nå; ellers når hen har stoppet og snudd seg.
        if n.gait == Gait::Stand {
            if let Some(a) = &mut n.anim {
                a.play(clips.talk, true, 0.2);
            }
        }
        Some(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * 2.0, text: n.lines[line].to_string() })
    }

    /// Ett steg: går ruta si, stopper med et ekte stopp når du kommer nær,
    /// snur seg mot deg med små skritt, vinker første gang, gjør små gester
    /// i hvile, og replikken står i LINE_SECS. Gir replikkene som står nå.
    pub fn update(&mut self, dt: f32, player: Vec3) -> Vec<Speech> {
        let mut out = vec![];
        for (i, n) in self.npcs.iter_mut().enumerate() {
            let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
            let rig = n.rig.map(|r| &self.rigs[r].0);
            let to = (player - n.pos).with_y(0.0);
            let dist = to.length();
            let mut finished = false;
            if let (Some(r), Some(a)) = (rig, n.anim.as_mut()) {
                a.update(dt, r);
                finished = a.finished;
            }
            if finished {
                // Et klipp uten løkke er ferdig: starten går rett over i
                // gangen (den ender i fase 0), ellers står hen.
                if let Gait::Start { .. } = n.gait {
                    n.gait = Gait::Walk;
                    n.last_phase = 0.0;
                    if let Some(a) = n.anim.as_mut() {
                        a.play(clips.walk, true, 0.06);
                    }
                } else {
                    if matches!(n.gait, Gait::Stop { .. }) {
                        n.gait = Gait::Stand;
                    }
                    n.rest(&clips, 0.3);
                }
            }
            let busy = dist < TALK_RANGE + 1.0 || n.speaking.is_some();
            let near = dist < TALK_RANGE;
            match n.gait {
                Gait::Start { t } => {
                    // Akselererer jevnt mot målet, som foten som står i startklippet.
                    let ta = START * clips.cycle;
                    let t1 = t + dt;
                    let s = |t: f32| if t < ta { clips.speed * t * t / (2.0 * ta) } else { clips.speed * (ta / 2.0 + t - ta) };
                    let d = s(t1) - s(t);
                    let (goal, _) = n.route[n.leg];
                    let to_goal = goal - n.pos;
                    let flat = to_goal.with_y(0.0);
                    if flat.length() > d && flat.length() > 1e-4 {
                        n.pos += to_goal * (d / flat.length());
                        n.yaw += wrap(flat.x.atan2(flat.z) - n.yaw) * (dt * TURN).min(1.0);
                    }
                    n.gait = Gait::Start { t: t1 };
                    if busy {
                        // Har knapt kommet i gang: blir stående.
                        n.gait = Gait::Stand;
                        n.rest(&clips, 0.3);
                    }
                }
                Gait::Walk => {
                    let phase = match (rig, n.anim.as_ref()) {
                        (Some(r), Some(a)) if a.clip == clips.walk => a.phase(r),
                        _ => 0.0,
                    };
                    // En hæl traff bakken siden forrige bilde (fasen 0 eller ½).
                    let strike = (phase % 0.5) < (n.last_phase % 0.5);
                    n.last_phase = phase;
                    let (goal, pause) = n.route[n.leg];
                    let to_goal = goal - n.pos;
                    let flat = to_goal.with_y(0.0);
                    let v = clips.speed;
                    let brake = v * DUTY * clips.cycle / 2.0;
                    if busy {
                        // Stopper ved neste hælnedslag, eller nå om det var nettopp.
                        if strike || (phase % 0.5) < 0.2 || clips.stop.is_none() {
                            n.begin_stop(&clips, phase, false);
                        }
                    } else if pause > 0.0 && strike && flat.length() <= brake + v * clips.cycle / 4.0 && clips.stop.is_some() {
                        n.begin_stop(&clips, phase, true);
                    }
                    if n.gait == Gait::Walk {
                        let step = v * dt;
                        if flat.length() <= step {
                            n.pos = goal;
                            if pause <= 0.0 || clips.stop.is_none() {
                                n.leg = (n.leg + 1) % n.route.len();
                                if pause > 0.0 {
                                    n.gait = Gait::Stand;
                                    n.wait = pause + ((i * 7 + n.leg * 3) % 3) as f32;
                                    n.rest(&clips, 0.3);
                                }
                            } else {
                                // Kom fram uten å treffe et hælnedslag i tide.
                                n.begin_stop(&clips, phase, true);
                            }
                        } else {
                            n.pos += to_goal * (step / flat.length());
                            let want = flat.x.atan2(flat.z);
                            n.yaw += wrap(want - n.yaw) * (dt * TURN).min(1.0);
                        }
                    }
                }
                Gait::Stop { t, v0, at_goal } => {
                    // Bremser jevnt (som foten som står, i stoppklippet).
                    let td = DUTY * clips.cycle;
                    let t1 = t + dt;
                    let s = |t: f32| if t < td { v0 * (t - t * t / (2.0 * td)) } else { v0 * td / 2.0 };
                    let d = s(t1) - s(t);
                    let fwd = Vec3::new(n.yaw.sin(), 0.0, n.yaw.cos());
                    let (goal, pause) = n.route.get(n.leg).copied().unwrap_or((n.pos, 0.0));
                    let to_goal = goal - n.pos;
                    let flat = to_goal.with_y(0.0);
                    // Følger bakken mot målet; forbi det rett fram.
                    n.pos += if at_goal && flat.length() > d { to_goal * (d / flat.length()) } else { fwd * d };
                    n.gait = Gait::Stop { t: t1, v0, at_goal };
                    if at_goal && t1 >= td && t < td {
                        n.leg = (n.leg + 1) % n.route.len().max(1);
                        n.wait = pause + ((i * 7 + n.leg * 3) % 3) as f32;
                    }
                }
                Gait::Turn { walk } => {
                    // Mot deg, eller mot målet hen skal gå til.
                    let dir = if walk { n.route.get(n.leg).map(|(g, _)| (*g - n.pos).with_y(0.0)).unwrap_or(to) } else { to };
                    let want = dir.x.atan2(dir.z);
                    let diff = wrap(want - n.yaw);
                    let rate = TURN_DEG.to_radians() / clips.turn_len;
                    let done = diff.abs() <= rate * dt * 1.5;
                    if walk && busy {
                        n.gait = Gait::Stand;
                        n.rest(&clips, 0.3);
                    } else if done || (!walk && !near) {
                        if done {
                            n.yaw = want;
                        }
                        n.gait = Gait::Stand;
                        if walk {
                            n.set_off(&clips);
                        } else {
                            n.rest(&clips, 0.3);
                        }
                    } else {
                        n.yaw += diff.signum() * rate * dt;
                    }
                }
                Gait::Stand => {
                    if near {
                        let want = to.x.atan2(to.z);
                        let diff = wrap(want - n.yaw);
                        match clips.turn {
                            // Langt å snu: små skritt på stedet. Større yaw er
                            // mot venstre (modellens +Y).
                            Some((left, right)) if diff.abs() > TURN_STEP => {
                                n.gait = Gait::Turn { walk: false };
                                if let Some(a) = n.anim.as_mut() {
                                    a.play(if diff > 0.0 { left } else { right }, true, 0.2);
                                }
                            }
                            _ => n.yaw += diff * (dt * 3.0).min(1.0),
                        }
                        if n.gait == Gait::Stand && !n.waved && diff.abs() < TURN_STEP && n.speaking.is_none() {
                            n.waved = true;
                            if let Some(a) = n.anim.as_mut() {
                                a.play(clips.wave, false, 0.2);
                            }
                        }
                    } else if !busy && n.route.len() > 1 {
                        n.wait -= dt;
                        if n.wait <= 0.0 {
                            if !matches!(n.route.get(n.leg), Some((g, _)) if g.with_y(0.0).distance(n.pos.with_y(0.0)) > 0.3) {
                                n.leg = (n.leg + 1) % n.route.len();
                            }
                            n.set_off(&clips);
                        }
                    }
                    // Små gester i hvile: ser seg om eller gjør noe med hendene.
                    if n.gait == Gait::Stand && n.speaking.is_none() {
                        n.fidget -= dt;
                        let idle = n.anim.as_ref().map(|a| a.clip == clips.idle).unwrap_or(false);
                        if n.fidget <= 0.0 && idle {
                            n.fidgets += 1;
                            n.fidget = 7.0 + ((n.fidgets * 13 + i * 5) % 9) as f32;
                            let pick = if n.fidgets % 2 == 0 { clips.look } else { clips.fidget.or(clips.look) };
                            if let (Some(c), Some(a)) = (pick, n.anim.as_mut()) {
                                a.play(c, false, 0.45);
                            }
                        }
                    }
                }
            }
            if let Some((line, t)) = &mut n.speaking {
                *t += dt;
                if *t > LINE_SECS || dist > TALK_RANGE + 2.0 {
                    n.speaking = None;
                    if n.gait == Gait::Stand {
                        n.rest(&clips, 0.3);
                    }
                } else {
                    let line = *line;
                    // Prater når hen står (ikke midt i et stopp eller en snuing).
                    if n.gait == Gait::Stand {
                        if let Some(a) = n.anim.as_mut() {
                            if a.clip != clips.talk && a.clip != clips.wave {
                                a.play(clips.talk, true, 0.25);
                            }
                        }
                    }
                    out.push(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * 2.0, text: n.lines[line].to_string() });
                }
            }
        }
        out
    }

    pub fn instances(&self, skins: &mut Vec<Mat4>) -> Vec<ModelInstance> {
        let mut out = vec![];
        for n in &self.npcs {
            let rot = Quat::from_rotation_y(n.yaw - std::f32::consts::FRAC_PI_2);
            let t = Affine3A::from_rotation_translation(rot, n.pos);
            match (n.rig.map(|r| &self.rigs[r].0), &n.anim) {
                (Some(rig), Some(a)) => {
                    let mats = rig.skin(&a.pose(rig));
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    out.push(ModelInstance { model: n.model, transform: t, skin: Some((base, mats.len() as u32)), flash: 0.0, tint: Some(n.tint) });
                }
                _ => out.push(ModelInstance { model: n.model, transform: t, skin: None, flash: 0.0, tint: Some(n.tint) }),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::anim::{Channel, Clip, Joint, Keys};
    use glam::Mat4;

    /// Et skjelett med to føtter og klippene fra folk_motion.py: gangen
    /// (foten som står, glir bakover med 1 m/s), stopp, start og snu.
    fn rig() -> Rig {
        let joint = |name: &str| Joint { name: name.into(), parent: None, translation: Vec3::ZERO, rotation: Quat::IDENTITY, scale: Vec3::ONE, inverse_bind: Mat4::IDENTITY };
        // Venstre fot står 0..0,6 s og svinger fram 0,6..1,0; høyre en halv syklus etter.
        let foot = |off: f32| {
            let mut times = vec![];
            let mut keys = vec![];
            for k in 0..=20 {
                let t = k as f32 * 0.05;
                let u = (t + off).rem_euclid(1.0);
                let p = if u < DUTY { Vec3::new(0.3 - u, 0.0, 0.0) } else { let s = (u - DUTY) / (1.0 - DUTY); Vec3::new(-0.3 + 0.6 * s, 0.1 * (s * std::f32::consts::PI).sin(), 0.0) };
                times.push(t);
                keys.push(p);
            }
            (times, keys)
        };
        let (tl, kl) = foot(0.0);
        let (tr, kr) = foot(0.5);
        let walk = Clip { name: "walk".into(), duration: 1.0, channels: vec![Channel { joint: 0, times: tl, keys: Keys::Translation(kl), step: false }, Channel { joint: 1, times: tr, keys: Keys::Translation(kr), step: false }] };
        let bare = |name: &str, d: f32| Clip { name: name.into(), duration: d, channels: vec![] };
        let clips = vec![bare("idle", 12.0), walk, bare("talk", 8.0), bare("wave", 3.0), bare("start", 0.4), bare("stop", 1.35), bare("stop_r", 1.35), bare("turn_l", 0.83), bare("turn_r", 0.83), bare("look", 4.0), bare("fidget", 5.0)];
        Rig { joints: vec![joint("foot.L"), joint("foot.R")], clips, root: Mat4::IDENTITY }
    }

    fn one(route: Vec<(Vec3, f32)>, yaw: f32) -> Npcs {
        Npcs::new(|_| Some(rig()), vec![("Prøve", "prøve", route[0].0, yaw, Vec3::ONE, &["Hei."], route)])
    }

    #[test]
    fn farten_kommer_fra_klippet() {
        let n = one(vec![(Vec3::ZERO, 1.0)], 0.0);
        let c = n.rigs[0].1;
        assert!((c.speed - 1.0).abs() < 0.05, "{}", c.speed);
        assert!(c.start.is_some() && c.stop.is_some() && c.turn.is_some());
    }

    #[test]
    fn starter_gar_og_stopper_ved_malet() {
        let goal = Vec3::new(0.0, 0.0, 6.0);
        let mut n = one(vec![(Vec3::ZERO, 2.0), (goal, 4.0)], 0.0);
        let far = Vec3::new(100.0, 0.0, 0.0);
        let mut seen = vec![];
        let mut stopped_at = None;
        for k in 0..(14.0 * 30.0) as usize {
            n.update(1.0 / 30.0, far);
            let g = n.npcs[0].gait;
            let tag = match g {
                Gait::Stand => 0,
                Gait::Start { .. } => 1,
                Gait::Walk => 2,
                Gait::Stop { .. } => 3,
                Gait::Turn { .. } => 4,
            };
            if seen.last() != Some(&tag) {
                seen.push(tag);
            }
            if tag == 0 && seen.contains(&3) && stopped_at.is_none() {
                stopped_at = Some((k, n.npcs[0].pos));
            }
        }
        // Står, starter, går, bremser, står.
        assert_eq!(&seen[..5], &[0, 1, 2, 3, 0], "{seen:?}");
        let (_, p) = stopped_at.expect("stoppet aldri");
        // Bremsingen begynner ved et hælnedslag, så hen ender innen et kvart skritt fra målet.
        assert!(p.distance(goal) < 0.3, "stoppet {:.2} m fra målet", p.distance(goal));
    }

    #[test]
    fn stopper_og_snur_seg_mot_deg() {
        let mut n = one(vec![(Vec3::ZERO, 0.5), (Vec3::new(0.0, 0.0, 30.0), 1.0)], 0.0);
        let far = Vec3::new(100.0, 0.0, 0.0);
        for _ in 0..(4.0 * 30.0) as usize {
            n.update(1.0 / 30.0, far);
        }
        assert_eq!(n.npcs[0].gait, Gait::Walk);
        // Du står bak og til siden: hen stopper (innen litt over ett sekund) og snur seg.
        let p = n.npcs[0].pos;
        let player = p + Vec3::new(2.5, 0.0, -1.0);
        for _ in 0..(1.3 * 30.0) as usize {
            n.update(1.0 / 30.0, player);
        }
        let here = n.npcs[0].pos;
        for _ in 0..(4.0 * 30.0) as usize {
            n.update(1.0 / 30.0, player);
        }
        assert!(n.npcs[0].pos.distance(here) < 0.02, "gikk videre etter stoppet");
        let to = (player - n.npcs[0].pos).with_y(0.0);
        let diff = wrap(to.x.atan2(to.z) - n.npcs[0].yaw);
        assert!(diff.abs() < 0.1, "ser ikke mot deg: {diff}");
    }
}
