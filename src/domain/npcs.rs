//! Innbyggerne rundt torget: Knappesmeden, Skjemakontoret, Apotekeren og
//! Gorvek. Én rigget menneskemodell (`tools/models/npc.py`) i ulike farger,
//! som spillet poserer per bilde (idle, talk, wave). Installasjonen
//! (`installations/npcs.rs`) gir dem props å sikte på og replikkene;
//! her bor animasjonen og samtalen som pågår.

use crate::domain::anim::{Animator, Rig};
use crate::domain::scene::ModelInstance;
use glam::{Affine3A, Mat4, Quat, Vec3};

pub const NPC_MODEL: &str = "npc";
pub const MAX_NPCS: usize = 8;
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
    /// Hvilken replikk som står for tur.
    next_line: usize,
    anim: Option<Animator>,
    /// Replikken som vises nå, og hvor lenge den har stått.
    pub speaking: Option<(usize, f32)>,
    /// Vinker når du kommer nær første gang.
    waved: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct Clips {
    idle: usize,
    talk: usize,
    wave: usize,
}

#[derive(Default)]
pub struct Npcs {
    pub npcs: Vec<Npc>,
    rig: Option<Rig>,
    clips: Clips,
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
    pub fn new(rig: Option<Rig>, cast: Vec<(&'static str, Vec3, f32, Vec3, &'static [&'static str])>) -> Self {
        let clips = rig
            .as_ref()
            .map(|r| {
                let find = |n: &str| r.clip(n).unwrap_or(0);
                Clips { idle: find("idle"), talk: find("talk"), wave: find("wave") }
            })
            .unwrap_or_default();
        let mut npcs = vec![];
        for (i, (name, pos, yaw, tint, lines)) in cast.into_iter().enumerate().take(MAX_NPCS) {
            let mut anim = rig.as_ref().map(|_| Animator::new(clips.idle, true));
            if let Some(a) = &mut anim {
                a.time = i as f32 * 0.9;
            }
            npcs.push(Npc { name, pos, yaw, tint, lines, next_line: 0, anim, speaking: None, waved: false });
        }
        Self { npcs, rig, clips }
    }

    pub fn reserve() -> Vec<(&'static str, usize)> {
        vec![(NPC_MODEL, MAX_NPCS)]
    }

    /// F på en innbygger: neste replikk. Gir det som ble sagt.
    pub fn talk(&mut self, i: usize) -> Option<Speech> {
        let clips = self.clips;
        let n = self.npcs.get_mut(i)?;
        if n.lines.is_empty() {
            return None;
        }
        let line = n.next_line % n.lines.len();
        n.next_line = line + 1;
        n.speaking = Some((line, 0.0));
        if let Some(a) = &mut n.anim {
            a.play(clips.talk, true, 0.15);
        }
        Some(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * 2.0, text: n.lines[line].to_string() })
    }

    /// Ett steg: snur seg mot deg når du er nær, vinker første gang, og
    /// replikken står i LINE_SECS. Gir replikkene som står nå.
    pub fn update(&mut self, dt: f32, player: Vec3) -> Vec<Speech> {
        let mut out = vec![];
        let clips = self.clips;
        for (i, n) in self.npcs.iter_mut().enumerate() {
            let to = (player - n.pos).with_y(0.0);
            let dist = to.length();
            if let (Some(r), Some(a)) = (&self.rig, n.anim.as_mut()) {
                a.update(dt, r);
                if a.finished {
                    a.play(clips.idle, true, 0.2);
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
                    out.push(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * 2.0, text: n.lines[*line].to_string() });
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
            match (&self.rig, &n.anim) {
                (Some(rig), Some(a)) => {
                    let mats = rig.skin(&a.pose(rig));
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    out.push(ModelInstance { model: NPC_MODEL, transform: t, skin: Some((base, mats.len() as u32)), flash: 0.0, tint: Some(n.tint) });
                }
                _ => out.push(ModelInstance { model: NPC_MODEL, transform: t, skin: None, flash: 0.0, tint: Some(n.tint) }),
            }
        }
        out
    }
}
