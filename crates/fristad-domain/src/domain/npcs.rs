//! Innbyggerne rundt torget: Smeden, Skriptoriet, Apotekeren og
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
/// Sittende står hodet så høyt (brøk av figurens høyde), så snakkebobla og
/// siktemålet står over det.
pub const SEATED_TOP: f32 = 0.74;
/// Så langt åpnes kjeven (radianer) når sangen er på det sterkeste, og så
/// langt løftes hodet i stedet når figuren ikke har noe kjevebein. Leppesømmen
/// er skåret opp og har en mørk munnhule bak seg (`folk3d.open_mouth`,
/// `mouth_cave`, 20. oktober), så kjeven kan åpnes mer enn de 0,12 den hadde
/// da den bare strakk underleppa til en lang, blek hake.
pub const JAW_OPEN: f32 = 0.2;
pub const HEAD_LIFT: f32 = 0.12;

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
    /// Replikken som vises nå (teksten), og hvor lenge den har stått.
    pub speaking: Option<(&'static str, f32)>,
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
    /// Sitter på krakken (trubaduren): går ingen steder og snur seg ikke;
    /// spiller (`play`) og synger bare mens sangen går (`Npcs::set_singing`,
    /// kjeven følger `Npcs::set_voice`), ellers sitter han stille (`sit`).
    pub seated: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct Clips {
    idle: usize,
    talk: usize,
    wave: usize,
    walk: usize,
    /// Sittende: med hendene på lårene (når hen snakker) og spillende.
    sit: usize,
    play: usize,
    /// Kjevebeinet (`jaw`), eller hodet om figuren ikke har noe.
    jaw: Option<usize>,
    head: Option<usize>,
    /// Beinet lutten henger i (`lute`, under hoftene; `folk3d.LUTE`).
    lute: Option<usize>,
}

/// Lutten trubaduren har i fanget (`tools/models/item3d.py lute`), satt etter
/// beinet `lute` i figurens skjelett hvert bilde (`Npcs::instances`).
pub const LUTE_MODEL: &str = "lute";

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
    /// Hvor sterkt det synges nå (0..1, sangens lydstyrke fra nettlaget,
    /// `assets/music/game/tavern_voice.json`): så åpen er munnen på dem som sitter.
    pub voice: f32,
    /// Trubaduren synger (noen har kjøpt ham et krus mjød,
    /// `application/tavern.rs`): de som sitter, spiller.
    pub singing: bool,
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

impl Npc {
    /// Hvor høyt hodet står (sittende er det lavere).
    fn top(&self) -> f32 {
        if self.seated {
            self.height * SEATED_TOP
        } else {
            self.height
        }
    }
}

/// Munnen etter sangen: kjevebeinet åpnes `voice · JAW_OPEN`; har figuren
/// ikke noe kjevebein (stedfortrederen), løftes hodet litt i stedet, så det
/// synes at hen synger. Dreies om sideaksen i figurens rom (figurene ser mot
/// +X med +Y opp, så sideaksen er Z; positivt løfter haka).
fn sing(rig: &Rig, pose: &mut crate::domain::anim::Pose, c: &Clips, voice: f32) {
    let v = voice.clamp(0.0, 1.0);
    let (joint, angle) = match (c.jaw, c.head) {
        (Some(j), _) => (j, -JAW_OPEN * v),
        (None, Some(h)) => (h, HEAD_LIFT * v),
        _ => return,
    };
    let globals = rig.globals(pose);
    let Some(j) = rig.joints.get(joint) else { return };
    let parent = j.parent.map(|p| globals[p].to_scale_rotation_translation().1).unwrap_or(Quat::IDENTITY);
    let q = Quat::from_axis_angle(Vec3::Z, angle);
    if let Some(slot) = pose.0.get_mut(joint) {
        slot.1 = (parent.inverse() * q * parent * slot.1).normalize();
    }
}

impl Npcs {
    /// Hvor sterkt det synges nå (0..1): munnen på dem som sitter og
    /// spiller, følger sangen (nettlaget sender verdien hvert bilde).
    pub fn set_voice(&mut self, v: f32) {
        self.voice = if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 };
    }

    /// Sangen begynner eller er over: de som sitter, tar opp spillet
    /// (`play`) eller legger det fra seg (`sit`), om de ikke snakker.
    pub fn set_singing(&mut self, on: bool) {
        self.singing = on;
        for n in self.npcs.iter_mut().filter(|n| n.seated && n.speaking.is_none()) {
            let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
            if let Some(a) = n.anim.as_mut() {
                a.play(if on { clips.play } else { clips.sit }, true, 0.4);
            }
        }
    }

    /// Klippet innbygger `i` spiller nå (`sit`, `play`, `walk` …).
    pub fn clip_of(&self, i: usize) -> Option<&str> {
        let n = self.npcs.get(i)?;
        let (r, _) = self.rigs.get(n.rig?)?;
        Some(r.clips.get(n.anim.as_ref()?.clip)?.name.as_str())
    }

    /// Innbygger `i` sier `text` (utenom replikkene: takken for mjøden).
    pub fn say(&mut self, i: usize, text: &'static str) -> Option<Speech> {
        let n = self.npcs.get_mut(i)?;
        let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
        n.speaking = Some((text, 0.0));
        if let Some(a) = &mut n.anim {
            a.play(if n.seated { clips.sit } else { clips.talk }, true, 0.15);
        }
        Some(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.top() + 0.1), text: text.to_string() })
    }

    /// `rig_of` gir skjelettet til en modell; hver innbygger har sin.
    pub fn new(rig_of: impl Fn(&str) -> Option<Rig>, cast: Vec<(&'static str, &'static str, Vec3, f32, Vec3, &'static [Line], Vec<(Vec3, f32)>, f32, bool)>) -> Self {
        let mut rigs: Vec<(Rig, Clips)> = vec![];
        let mut by_model: Vec<(&'static str, Option<usize>)> = vec![];
        let mut npcs = vec![];
        for (i, (name, model, pos, yaw, tint, lines, route, height, seated)) in cast.into_iter().enumerate().take(MAX_NPCS) {
            let rig = match by_model.iter().find(|(m, _)| *m == model) {
                Some((_, r)) => *r,
                None => {
                    let r = rig_of(model).map(|r| {
                        let find = |n: &str| r.clip(n).unwrap_or(0);
                        let idle = find("idle");
                        let sit = r.clip("sit").unwrap_or(idle);
                        let c = Clips { idle, talk: find("talk"), wave: find("wave"), walk: r.clip("walk").unwrap_or(idle), sit, play: r.clip("play").unwrap_or(sit), jaw: r.joint("jaw"), head: r.joint("head"), lute: r.joint("lute") };
                        rigs.push((r, c));
                        rigs.len() - 1
                    });
                    by_model.push((model, r));
                    r
                }
            };
            let mut anim = rig.map(|r| Animator::new(if seated { rigs[r].1.sit } else { rigs[r].1.idle }, true));
            if let Some(a) = &mut anim {
                a.time = i as f32 * 0.9;
            }
            npcs.push(Npc { name, pos, yaw, tint, lines, model, height, rig, heard: 0, last_line: None, round: 0, anim, speaking: None, waved: false, route, leg: 0, wait: 2.0 + i as f32 * 1.3, walking: false, held: false, stuck: 0.0, seated });
        }
        Self { npcs, rigs, voice: 0.0, singing: false }
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
        n.speaking = Some((n.lines[line].text, 0.0));
        if let Some(a) = &mut n.anim {
            // Sittende legger hen fra seg spillet og snakker med hendene på lårene.
            a.play(if n.seated { clips.sit } else { clips.talk }, true, 0.15);
        }
        Some(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.top() + 0.1), text: n.lines[line].text.to_string() })
    }

    /// Ett steg: snur seg mot deg når du er nær, vinker første gang, og
    /// replikken står i LINE_SECS. Gir replikkene som står nå.
    pub fn update(&mut self, dt: f32, player: Vec3) -> Vec<Speech> {
        let mut out = vec![];
        // Hvor alle står før steget, så hver kan gå rundt de andre.
        let others: Vec<Vec3> = self.npcs.iter().map(|n| n.pos).collect();
        for (i, n) in self.npcs.iter_mut().enumerate() {
            let clips = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
            let rest = if self.singing { clips.play } else { clips.sit };
            let to = (player - n.pos).with_y(0.0);
            let dist = to.length();
            if let (Some(r), Some(a)) = (n.rig.map(|r| &self.rigs[r].0), n.anim.as_mut()) {
                a.update(dt, r);
                if a.finished {
                    a.play(if n.seated { rest } else { clips.idle }, true, 0.2);
                }
            }
            if n.seated {
                // Sitter på krakken (og spiller mens sangen går): blir
                // sittende, snur seg ikke og vinker ikke; snakker hen, står
                // replikken som ellers.
                if let Some((text, t)) = &mut n.speaking {
                    *t += dt;
                    if *t > LINE_SECS || dist > TALK_RANGE + 2.0 {
                        n.speaking = None;
                        if let Some(a) = n.anim.as_mut() {
                            a.play(rest, true, 0.4);
                        }
                    } else {
                        out.push(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.height * SEATED_TOP + 0.1), text: text.to_string() });
                    }
                }
                continue;
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
            if let Some((text, t)) = &mut n.speaking {
                *t += dt;
                if *t > LINE_SECS || dist > TALK_RANGE + 2.0 {
                    n.speaking = None;
                    if let Some(a) = n.anim.as_mut() {
                        a.play(clips.idle, true, 0.3);
                    }
                } else {
                    out.push(Speech { npc: i, name: n.name, pos: n.pos + Vec3::Y * (n.height + 0.1), text: text.to_string() });
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
                    // Den som sitter på krakken flyttes aldri.
                    if self.npcs[i].seated || self.npcs[j].seated {
                        (wi, wj) = (if self.npcs[i].seated { 0.0 } else { 1.0 }, if self.npcs[j].seated { 0.0 } else { 1.0 });
                        if wi + wj == 0.0 {
                            continue;
                        }
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
                    let mut pose = a.pose(rig);
                    let c = n.rig.map(|r| self.rigs[r].1).unwrap_or_default();
                    if n.seated && self.singing && n.speaking.is_none() {
                        sing(rig, &mut pose, &c, self.voice);
                    }
                    // Beinets rom er lutens rom slik Blender har det (z opp
                    // langs halsen), og glb-fila har y opp: dreid om x.
                    if let Some(j) = c.lute {
                        let at = Affine3A::from_mat4(rig.joint_transform(&pose, j)) * Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2);
                        out.push(ModelInstance { model: LUTE_MODEL, transform: t * at, skin: None, flash: 0.0, tint: None, look: None });
                    }
                    let mats = rig.skin(&pose);
                    let base = skins.len() as u32;
                    skins.extend_from_slice(&mats);
                    out.push(ModelInstance { model: n.model, transform: t, skin: Some((base, mats.len() as u32)), flash: 0.0, tint: Some(n.tint), look: None });
                }
                _ => out.push(ModelInstance { model: n.model, transform: t, skin: None, flash: 0.0, tint: Some(n.tint), look: None }),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::anim::{Clip, Joint};

    /// Et lite skjelett: roten, hodet og (om `jaw`) kjeven, med klippene
    /// figurene har og de sittende.
    fn rig(jaw: bool) -> Rig {
        let joint = |name: &str, parent: Option<usize>, y: f32| Joint { name: name.into(), parent, translation: Vec3::new(0.0, y, 0.0), rotation: Quat::IDENTITY, scale: Vec3::ONE, inverse_bind: Mat4::IDENTITY };
        let mut joints = vec![joint("root", None, 0.0), joint("head", Some(0), 1.5)];
        if jaw {
            joints.push(joint("jaw", Some(1), 0.05));
        }
        let clips = ["idle", "walk", "talk", "wave", "sit", "play"].iter().map(|n| Clip { name: n.to_string(), duration: 1.0, channels: vec![] }).collect();
        Rig { joints, clips, root: Mat4::IDENTITY }
    }

    fn troubadour(jaw: bool) -> Npcs {
        static LINES: &[Line] = crate::domain::world::cast::TROUBADOUR;
        Npcs::new(move |_| Some(rig(jaw)), vec![("Trubaduren", "npc_troubadour", Vec3::new(1.0, 0.6, 2.0), 0.4, Vec3::ONE, LINES, vec![], 1.78, true)])
    }

    fn clip_name(n: &Npcs) -> &str {
        let npc = &n.npcs[0];
        let (r, _) = &n.rigs[npc.rig.unwrap()];
        &r.clips[npc.anim.as_ref().unwrap().clip].name
    }

    /// Trubaduren sitter der han sitter: han går ingen steder, snur seg
    /// ikke mot deg og vinker ikke. Han sitter stille (`sit`) til sangen
    /// begynner, spiller (`play`) mens den går; snakker han, legger han fra
    /// seg spillet og tar det opp igjen etterpå.
    #[test]
    fn trubaduren_sitter_og_spiller() {
        let mut n = troubadour(true);
        assert_eq!(clip_name(&n), "sit", "uten mjød spiller han ikke");
        n.set_singing(true);
        assert_eq!(clip_name(&n), "play");
        let (pos, yaw) = (n.npcs[0].pos, n.npcs[0].yaw);
        for k in 0..600 {
            let player = if k < 300 { Vec3::new(2.0, 0.6, 3.0) } else { Vec3::new(40.0, 0.0, 40.0) };
            n.update(1.0 / 30.0, player);
        }
        assert_eq!((n.npcs[0].pos, n.npcs[0].yaw), (pos, yaw), "trubaduren reiste seg fra krakken");
        assert_eq!(clip_name(&n), "play");
        let said = n.talk(0, 0).expect("trubaduren har replikker");
        assert!(said.pos.y < pos.y + 1.78, "snakkebobla står over hodet hans når han sitter");
        assert_eq!(clip_name(&n), "sit");
        for _ in 0..(LINE_SECS * 30.0) as usize + 30 {
            n.update(1.0 / 30.0, Vec3::new(2.0, 0.6, 3.0));
        }
        assert_eq!(clip_name(&n), "play", "spiller igjen når replikken er over");
    }

    /// Munnen følger sangen: kjeven åpnes etter `voice` (hodet løftes i
    /// stedet om figuren ikke har noe kjevebein), og står stille når han
    /// ikke synger.
    #[test]
    fn munnen_folger_sangen() {
        for jaw in [true, false] {
            let mut n = troubadour(jaw);
            n.set_singing(true);
            n.update(0.1, Vec3::new(40.0, 0.0, 40.0));
            let joint = if jaw { 2 } else { 1 };
            let at = |n: &Npcs| {
                let mut skins = vec![];
                n.instances(&mut skins);
                skins[joint]
            };
            n.set_voice(0.0);
            let shut = at(&n);
            n.set_voice(1.0);
            let open = at(&n);
            n.set_voice(f32::NAN);
            assert_eq!(at(&n), shut, "en ugyldig verdi lukker munnen");
            let turned = (open.to_scale_rotation_translation().1 * shut.to_scale_rotation_translation().1.inverse()).to_axis_angle();
            let want = if jaw { -JAW_OPEN } else { HEAD_LIFT };
            assert!((turned.0.z.signum() * turned.1 - want).abs() < 1e-3, "jaw {jaw}: {turned:?}");
            n.set_singing(false);
            n.set_voice(1.0);
            assert_eq!(at(&n), shut, "jaw {jaw}: munnen står stille når han ikke synger");
        }
    }
}
