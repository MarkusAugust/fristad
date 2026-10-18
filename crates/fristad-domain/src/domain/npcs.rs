//! Innbyggerne rundt torget: Buttonsmeden, Skjemakontoret, Apotekeren og
//! Torgils (våpenmesteren; nøkkelen heter fortsatt `gorvek`). Én rigget menneskemodell (`tools/models/npc.py`) i ulike farger,
//! som spillet poserer per bilde (idle, talk, wave). Installasjonen
//! (`installations/npcs.rs`) gir dem props å sikte på og replikkene;
//! her bor animasjonen og samtalen som pågår.

use crate::domain::anim::{Animator, Rig};
use crate::domain::scene::ModelInstance;
use crate::domain::world::cast::Line;
use glam::{Affine3A, Mat4, Quat, Vec3};

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
    pub lines: &'static [Line],
    /// Modellen (figuren til innbyggeren) og skjelettet
    /// med klippene i `Npcs.rigs`.
    pub model: &'static str,
    /// Høyden på figuren (snakkebobla står rett over hodet).
    pub height: f32,
    rig: Option<usize>,
    /// Replikkene du har hørt i denne runden (bit per replikk), den
    /// siste, og hvor mange runder du har hørt alle, så rekkefølgen skifter.
    heard: u64,
    last_line: Option<usize>,
    round: usize,
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
    walking: bool,
    /// Står stille for deg (du er nær, eller hen snakker).
    held: bool,
    /// Hvor lenge hen ikke har kommet nærmere målet (står fast bak noen).
    stuck: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Clips {
    idle: usize,
    talk: usize,
    wave: usize,
    walk: usize,
}

/// Ganghastigheten (m/s) og hvor fort de snur seg.
const WALK_SPEED: f32 = 0.95;
const TURN: f32 = 4.0;
/// Så nær går de aldri hverandre (sentrum til sentrum), og så langt fram
/// ser de etter noen å gå rundt.
pub const NPC_GAP: f32 = 0.9;
const AVOID: f32 = 2.2;
/// Kommer hen ikke nærmere målet på så lenge, går hen videre til neste.
const STUCK_SECS: f32 = 3.0;

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

/// Retningen å gå: mot målet, men rundt de andre som står eller går
/// foran (til den siden de ikke står på, til høyre om de står rett foran,
/// så to som møtes går rundt hverandre).
fn steer(i: usize, pos: Vec3, dir: Vec3, others: &[Vec3]) -> Vec3 {
    let right = Vec3::new(-dir.z, 0.0, dir.x);
    let mut side = 0.0;
    for (j, q) in others.iter().enumerate() {
        let d = (*q - pos).with_y(0.0);
        let len = d.length();
        if j == i || len < 1e-4 || len > AVOID {
            continue;
        }
        let ahead = d.dot(dir) / len;
        if ahead <= 0.0 {
            continue;
        }
        let lateral = d.dot(right);
        let away = if lateral.abs() < 0.15 { 1.0 } else { -lateral.signum() };
        side += away * ahead * (1.0 - len / AVOID) * 2.5;
    }
    (dir + right * side.clamp(-2.0, 2.0)).normalize_or(dir)
}

/// Neste replikk (se `Npcs::talk`).
fn next_line(lines: &[Line], depth: u8, heard: &mut u64, round: &mut usize, last: Option<usize>) -> Option<usize> {
    let len = lines.len().min(64);
    let fits: Vec<usize> = (0..len).filter(|k| lines[*k].fits(depth)).collect();
    if fits.is_empty() {
        return None;
    }
    let pick = |heard: u64, round: usize| {
        fits.iter().copied().filter(|k| heard & (1 << k) == 0 && Some(*k) != last).min_by_key(|k| (std::cmp::Reverse(lines[*k].from), if round == 0 { *k } else { (k * 5 + round * 3) % len }, *k))
    };
    let k = match pick(*heard, *round) {
        Some(k) => k,
        None => {
            // Alle er hørt: en ny runde i en annen rekkefølge.
            *round += 1;
            *heard = 0;
            pick(0, *round).unwrap_or(fits[0])
        }
    };
    *heard |= 1 << k;
    Some(k)
}

impl Npcs {
    /// `rig_of` gir skjelettet til en modell; hver innbygger har sin.
    pub fn new(rig_of: impl Fn(&str) -> Option<Rig>, cast: Vec<(&'static str, &'static str, Vec3, f32, Vec3, &'static [Line], Vec<(Vec3, f32)>, f32)>) -> Self {
        let mut rigs: Vec<(Rig, Clips)> = vec![];
        let mut by_model: Vec<(&'static str, Option<usize>)> = vec![];
        let mut npcs = vec![];
        for (i, (name, model, pos, yaw, tint, lines, route, height)) in cast.into_iter().enumerate().take(MAX_NPCS) {
            let rig = match by_model.iter().find(|(m, _)| *m == model) {
                Some((_, r)) => *r,
                None => {
                    let r = rig_of(model).map(|r| {
                        let find = |n: &str| r.clip(n).unwrap_or(0);
                        let idle = find("idle");
                        let c = Clips { idle, talk: find("talk"), wave: find("wave"), walk: r.clip("walk").unwrap_or(idle) };
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
            npcs.push(Npc { name, pos, yaw, tint, lines, model, height, rig, heard: 0, last_line: None, round: 0, anim, speaking: None, waved: false, route, leg: 0, wait: 2.0 + i as f32 * 1.3, walking: false, held: false, stuck: 0.0 });
        }
        Self { npcs, rigs }
    }

    /// F på en innbygger: neste replikk. `depth` er hvor mange nivåer i
    /// krypten du har nådd (0 før du har vært nede), så replikkene følger
    /// framgangen. Det nyeste (høyest `Line.from`) som ikke er hørt kommer
    /// først, ellers de andre i tur, aldri den samme to ganger på rad.
    /// Gir det som ble sagt.
    pub fn talk(&mut self, i: usize, depth: u8) -> Option<Speech> {
        let n = self.npcs.get_mut(i)?;
        let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
        let line = next_line(n.lines, depth, &mut n.heard, &mut n.round, n.last_line)?;
        n.last_line = Some(line);
        n.speaking = Some((line, 0.0));
        if let Some(a) = &mut n.anim {
            a.play(clips.talk, true, 0.15);
        }
        Some(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.height + 0.1), text: n.lines[line].text.to_string() })
    }

    /// Ett steg: snur seg mot deg når du er nær, vinker første gang, og
    /// replikken står i LINE_SECS. Gir replikkene som står nå.
    pub fn update(&mut self, dt: f32, player: Vec3) -> Vec<Speech> {
        let mut out = vec![];
        // Hvor alle står før steget, så hver kan gå rundt de andre.
        let others: Vec<Vec3> = self.npcs.iter().map(|n| n.pos).collect();
        for (i, n) in self.npcs.iter_mut().enumerate() {
            let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
            let to = (player - n.pos).with_y(0.0);
            let dist = to.length();
            if let (Some(r), Some(a)) = (n.rig.map(|r| &self.rigs[r].0), n.anim.as_mut()) {
                a.update(dt, r);
                if a.finished {
                    a.play(clips.idle, true, 0.2);
                }
            }
            // Gjøremålene: går mellom punktene på ruta og blir stående litt
            // ved hvert; står stille og snur seg mot deg når du er nær.
            n.held = dist < TALK_RANGE + 1.0 || n.speaking.is_some();
            if n.held {
                if n.walking {
                    n.walking = false;
                    if let Some(a) = n.anim.as_mut() {
                        a.play(clips.idle, true, 0.25);
                    }
                }
            } else if n.route.len() > 1 {
                if !n.walking {
                    n.wait -= dt;
                    if n.wait <= 0.0 {
                        n.leg = (n.leg + 1) % n.route.len();
                        n.walking = true;
                        n.stuck = 0.0;
                        if let Some(a) = n.anim.as_mut() {
                            a.play(clips.walk, true, 0.3);
                        }
                    }
                } else {
                    let (goal, pause) = n.route[n.leg];
                    let flat = (goal - n.pos).with_y(0.0);
                    let step = WALK_SPEED * dt;
                    // Står noen der hen skal, er det godt nok å komme nær.
                    let taken = others.iter().enumerate().any(|(j, q)| j != i && (*q - goal).with_y(0.0).length() < NPC_GAP);
                    let arrived = flat.length() <= step || (taken && flat.length() < NPC_GAP * 1.5) || n.stuck > STUCK_SECS;
                    if arrived {
                        if flat.length() <= step {
                            n.pos = goal;
                        }
                        n.stuck = 0.0;
                        if pause <= 0.0 {
                            // Et punkt langs gata: rett videre til neste.
                            n.leg = (n.leg + 1) % n.route.len();
                        } else {
                            n.walking = false;
                            n.wait = pause + ((i * 7 + n.leg * 3) % 3) as f32;
                            if let Some(a) = n.anim.as_mut() {
                                a.play(clips.idle, true, 0.3);
                            }
                        }
                    } else {
                        let dir = steer(i, n.pos, flat / flat.length(), &others);
                        let before = flat.length();
                        n.pos += dir * step;
                        n.pos.y += (goal.y - n.pos.y) * (step / before).min(1.0);
                        let gained = before - (goal - n.pos).with_y(0.0).length();
                        n.stuck = if gained < step * 0.3 { n.stuck + dt } else { (n.stuck - dt).max(0.0) };
                        let want = dir.x.atan2(dir.z);
                        let diff = (want - n.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                        n.yaw += diff * (dt * TURN).min(1.0);
                    }
                }
            }
            if dist < TALK_RANGE {
                // Snur seg mot deg, mykt.
                let want = to.x.atan2(to.z);
                let diff = (want - n.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                n.yaw += diff * (dt * 3.0).min(1.0);
                if !n.waved {
                    n.waved = true;
                    if let Some(a) = n.anim.as_mut() {
                        a.play(clips.wave, false, 0.15);
                    }
                }
            }
            if let Some((line, t)) = &mut n.speaking {
                *t += dt;
                if *t > LINE_SECS || dist > TALK_RANGE + 2.0 {
                    n.speaking = None;
                    if let Some(a) = n.anim.as_mut() {
                        a.play(clips.idle, true, 0.3);
                    }
                } else {
                    out.push(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.height + 0.1), text: n.lines[*line].text.to_string() });
                }
            }
        }
        self.keep_apart();
        out
    }

    /// Ingen to står oppå hverandre: kommer to nærmere enn `NPC_GAP`,
    /// skyves de fra hverandre, og den som går viker (den som står for
    /// deg eller venter ved et gjøremål, blir stående).
    fn keep_apart(&mut self) {
        for _ in 0..3 {
            for i in 0..self.npcs.len() {
                for j in i + 1..self.npcs.len() {
                    let d = (self.npcs[i].pos - self.npcs[j].pos).with_y(0.0);
                    let len = d.length();
                    if len >= NPC_GAP {
                        continue;
                    }
                    let away = if len > 1e-4 { d / len } else { Vec3::new((i as f32).cos(), 0.0, (i as f32).sin()) };
                    let moves = |n: &Npc| if n.walking && !n.held { 1.0 } else { 0.0 };
                    let (mut wi, mut wj) = (moves(&self.npcs[i]), moves(&self.npcs[j]));
                    if wi + wj == 0.0 {
                        (wi, wj) = (1.0, 1.0);
                    }
                    let push = (NPC_GAP - len) / (wi + wj);
                    self.npcs[i].pos += away * push * wi;
                    self.npcs[j].pos -= away * push * wj;
                }
            }
        }
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
