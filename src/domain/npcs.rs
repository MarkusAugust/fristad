//! Innbyggerne rundt torget: Buttonsmeden, Skjemakontoret, Apotekeren og
//! Gorvek. Én rigget menneskemodell (`tools/models/npc.py`) i ulike farger,
//! som spillet poserer per bilde (idle, talk, wave). Installasjonen
//! (`installations/npcs.rs`) gir dem props å sikte på og replikkene;
//! her bor animasjonen og samtalen som pågår.

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
    /// Høyden på figuren (snakkebobla står rett over hodet).
    pub height: f32,
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
    walking: bool,
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

impl Npcs {
    /// `rig_of` gir skjelettet til en modell; hver innbygger har sin.
    pub fn new(rig_of: impl Fn(&str) -> Option<Rig>, cast: Vec<(&'static str, &'static str, Vec3, f32, Vec3, &'static [&'static str], Vec<(Vec3, f32)>, f32)>) -> Self {
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
            npcs.push(Npc { name, pos, yaw, tint, lines, model, height, rig, next_line: 0, anim, speaking: None, waved: false, route, leg: 0, wait: 2.0 + i as f32 * 1.3, walking: false });
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
        if let Some(a) = &mut n.anim {
            a.play(clips.talk, true, 0.15);
        }
        Some(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.height + 0.1), text: n.lines[line].to_string() })
    }

    /// Ett steg: snur seg mot deg når du er nær, vinker første gang, og
    /// replikken står i LINE_SECS. Gir replikkene som står nå.
    pub fn update(&mut self, dt: f32, player: Vec3) -> Vec<Speech> {
        let mut out = vec![];
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
            let busy = dist < TALK_RANGE + 1.0 || n.speaking.is_some();
            if busy {
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
                        if let Some(a) = n.anim.as_mut() {
                            a.play(clips.walk, true, 0.3);
                        }
                    }
                } else {
                    let (goal, pause) = n.route[n.leg];
                    let to_goal = goal - n.pos;
                    let flat = to_goal.with_y(0.0);
                    let step = WALK_SPEED * dt;
                    if flat.length() <= step {
                        n.pos = goal;
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
                        n.pos += to_goal * (step / flat.length());
                        let want = flat.x.atan2(flat.z);
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
                    out.push(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.height + 0.1), text: n.lines[*line].to_string() });
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
