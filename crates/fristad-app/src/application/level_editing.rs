//! Nivåredigeringen i økta (`?nivaa`, `docs/nivaaredigering.md` del 12):
//! kommandoene fra kartet går til `LevelEditor` i kjernen, og 3D-vinduet
//! viser nivået slik det er kompilert nå. Etter hver endring bygges nivået
//! på et kladdeark (`crypt::build` med en egen `Builder`, som spillet gjør),
//! og bitene, mekanismene og fiendene tegnes som bevegelige instanser i
//! plassen monteringen satte av (`assembly::prepare_level_editor`), mens de
//! faste instansene i krypten (nivået slik det var da verden ble bygd)
//! skjules. Du flyr fritt med arbeidslys. Prøveturen bygger verden på nytt
//! med dokumentet som nivå 1 (adapteren), og du kan ikke dø (`immortal`).

use super::session::Session;
use crate::domain::edit::level::LevelEditor;
use crate::domain::mob_kind::Kind;
use crate::domain::model::ModelInfo;
use crate::domain::scene::{Frame, Light, ModelInstance, RoomLight, MAX_LIGHTS, MAX_ROOMS};
use crate::domain::world::crypt::{self, compose::CELL};
use crate::domain::world::mechanisms::Mechanisms;
use crate::domain::world::{self, Builder, World};
use crate::ports::driving::{Input, LevelCommand};
use glam::{Affine3A, Vec2, Vec3};

/// Det monteringen gir nivåredigeringen: redigeringen og stedet krypten
/// begynner under (`town.crypt_anchor`).
pub struct LevelSetup {
    pub editor: LevelEditor,
    pub anchor: Vec2,
}

/// Fargene på markeringene i 3D.
const SELECTED: Vec3 = Vec3::new(3.0, 2.4, 0.4);
const ERROR: Vec3 = Vec3::new(4.0, 0.3, 0.2);
const LINK: Vec3 = Vec3::new(0.6, 2.2, 4.0);
const PATROL: Vec3 = Vec3::new(3.2, 1.2, 0.3);

/// Hvor mange av hver bit 3D-vinduet kan vise (plassen monteringen setter av).
pub const LEVEL_RESERVE: usize = 1500;
/// Det samme for mekanismene, pynten og innholdet.
pub const THING_RESERVE: usize = 96;

/// Redigeringen av ett nivå i økta.
pub struct LevelEditing {
    pub editor: LevelEditor,
    anchor: Vec2,
    infos: Vec<ModelInfo>,
    /// De faste instansene i krypten (nivået slik verden ble bygd).
    hidden: Vec<(&'static str, usize)>,
    shown: Vec<ModelInstance>,
    lights: Vec<Light>,
    band: (f32, f32),
    /// Nivåets origo i verden (dokumentets rom + origo = verden).
    pub origin: Vec3,
    start: Option<(Vec3, f32)>,
    /// I redigeringen (ellers på prøvetur).
    pub active: bool,
    /// Arbeidslys: lyst og flatt; ellers spillets lys (faklene og lykta).
    pub work_light: bool,
    /// Det valgte på kartet (modulene og innholdet), markert i 3D.
    pub selected: Vec<String>,
    /// Kameraet i redigeringen før prøveturen.
    pub camera: Option<(Vec3, f32, f32)>,
    /// Gulvet kartet viser (y i dokumentet), til å finne punktet under pekeren.
    pub floor: f32,
    /// Nivået slik det er bygd på kladdearket (rommene til lyset).
    crypt: crypt::Crypt,
}

impl LevelEditing {
    pub fn new(setup: LevelSetup, world: &World, infos: &[ModelInfo]) -> Self {
        // Alt fast under kirkegården hører til nivået slik det var: skjules.
        let hidden = world.placements.iter().flat_map(|p| p.transforms.iter().enumerate().filter(|(_, t)| t.translation.y < crypt::TOP).map(move |(n, _)| (p.model, n))).collect();
        let mut e = LevelEditing {
            editor: setup.editor,
            anchor: setup.anchor,
            infos: infos.to_vec(),
            hidden,
            shown: vec![],
            lights: vec![],
            band: (-80.0, -20.0),
            origin: Vec3::new(0.0, -crypt::LEVEL_DROP, 0.0),
            start: None,
            active: true,
            work_light: true,
            selected: vec![],
            camera: None,
            floor: 0.0,
            crypt: crypt::Crypt::default(),
        };
        e.refresh();
        e
    }

    /// Bygg nivået på kladdearket og samle det som skal tegnes.
    pub fn refresh(&mut self) {
        let mut b = Builder::new(&self.infos);
        let mut file = self.editor.file().clone();
        file.level = 1;
        let c = crypt::build(&mut b, self.anchor, vec![(1, file)]);
        let mut shown = vec![];
        for pl in &b.placements {
            for (n, t) in pl.transforms.iter().enumerate() {
                let look = pl.looks.get(n).copied().flatten();
                let tint = pl.tints.get(n).copied().filter(|t| *t != Vec3::ONE);
                shown.push(ModelInstance { model: pl.model, transform: *t, skin: None, flash: 0.0, tint, look });
            }
        }
        // Mekanismene i hvile (portene nede, veggene og lokkene lukket).
        let mech = Mechanisms::new(&c, 0);
        let posed = |model: &'static str, list: Vec<Affine3A>| list.into_iter().map(move |transform| ModelInstance { model, transform, skin: None, flash: 0.0, tint: None, look: None });
        shown.extend(c.gates.iter().enumerate().map(|(n, g)| ModelInstance { model: crypt::PORTCULLIS, transform: mech.gate_pose(&c, n, world::at(g.pos, g.yaw)), skin: None, flash: 0.0, tint: None, look: None }));
        shown.extend(posed(crypt::KIT_LEVER_ARM, mech.lever_poses(&c)));
        shown.extend(posed(crypt::KIT_SHORTCUT_LEAF, mech.shortcut_poses(&c)));
        shown.extend(posed(crypt::CHEST_LID, c.chests.iter().enumerate().map(|(n, ch)| mech.chest_lid(n, world::at(ch.pos, ch.yaw))).collect()));
        shown.extend(c.secrets.iter().zip(mech.secret_poses(&c, 0.0)).map(|(s, transform)| ModelInstance { model: s.model, transform, skin: None, flash: 0.0, tint: None, look: s.look }));
        shown.extend(posed(crypt::KIT_TRAPDOOR, mech.trapdoor_poses(&c)));
        let (decks, chains) = mech.bridge_poses(&c);
        shown.extend(posed(crypt::KIT_BRIDGE, decks));
        shown.extend(posed(crypt::KIT_CHAIN, chains));
        shown.extend(posed(crypt::KIT_WATER, mech.water_poses(&c)));
        // Fiendene i hvilestillingen.
        for s in &c.spawns {
            let kind = Kind::from_label(&s.label);
            let scale = kind.stats().scale * if s.elite { 1.3 } else { 1.0 };
            shown.push(ModelInstance { model: kind.model(), transform: world::at(s.pos, s.yaw) * Affine3A::from_scale(Vec3::splat(scale)), skin: None, flash: 0.0, tint: None, look: None });
        }
        self.shown = shown;
        self.lights = b.lights.clone();
        if let Some(l) = c.levels.first() {
            self.origin = l.origin;
            self.band = (l.min_y - 9.0, l.max_y + 2.0);
            self.start = Some(l.start);
        }
        self.crypt = c;
    }

    /// Startstedet (føttene i verden, retningen), om nivået har et.
    pub fn start(&self) -> Option<(Vec3, f32)> {
        self.start
    }

    /// Et punkt i dokumentets rom, i verden.
    pub fn to_world(&self, p: Vec3) -> Vec3 {
        p + self.origin
    }

    /// Markeringene i 3D: det valgte, bruddene, koblingene og patruljene.
    fn lines(&self) -> Vec<(Vec3, Vec3, Vec3)> {
        let mut out = vec![];
        let o = self.origin;
        let doc = self.editor.doc();
        for m in doc.modules.iter().filter(|m| self.selected.contains(&m.id)) {
            for c in self.editor.footprint(m) {
                let lo = o + Vec3::new(c[0] as f32 * CELL - 2.0, c[1] as f32 + 0.05, c[2] as f32 * CELL - 2.0);
                let hi = lo + Vec3::new(CELL, 0.0, CELL);
                let corners = [lo, Vec3::new(hi.x, lo.y, lo.z), hi, Vec3::new(lo.x, lo.y, hi.z)];
                for k in 0..4 {
                    out.push((corners[k], corners[(k + 1) % 4], SELECTED));
                    out.push((corners[k], corners[k] + Vec3::Y * 0.6, SELECTED));
                }
            }
        }
        let file = self.editor.file();
        let pos = |id: &str| file.marks.iter().find(|m| m.id.as_deref() == Some(id)).and_then(|m| m.pos).map(|p| o + Vec3::from(p));
        for m in &file.marks {
            let Some(p) = m.pos.map(|p| o + Vec3::from(p)) else { continue };
            let chosen = m.id.as_ref().is_some_and(|id| self.selected.contains(id));
            if chosen {
                out.push((p, p + Vec3::Y * 2.5, SELECTED));
            }
            for t in m.opens.iter().filter_map(|t| pos(t)) {
                out.push((p + Vec3::Y * 1.2, t + Vec3::Y * 1.2, LINK));
            }
            let mut last = p;
            for q in &m.patrol {
                let q = o + Vec3::from(*q) + Vec3::Y * 0.1;
                out.push((last + Vec3::Y * 0.1, q, PATROL));
                last = q;
            }
        }
        for i in self.editor.issues().iter().filter(|i| i.severity == crypt::compose::Severity::Error) {
            if let Some(p) = i.at {
                let p = o + Vec3::from(p);
                out.push((p, p + Vec3::Y * 3.0, ERROR));
            }
        }
        out
    }
}

impl Session {
    /// I nivåredigeringen (ikke på prøvetur).
    pub fn level_active(&self) -> bool {
        self.level.as_ref().is_some_and(|l| l.active)
    }

    /// Nivåredigeringen, til nettlaget og testene.
    pub fn level_editing(&self) -> Option<&LevelEditing> {
        self.level.as_ref()
    }

    pub fn level_editing_mut(&mut self) -> Option<&mut LevelEditing> {
        self.level.as_mut()
    }

    /// En kommando fra kartet. Gir om den gjorde noe (feilen står i
    /// `level_error`).
    pub fn level_command(&mut self, c: LevelCommand) -> bool {
        let Some(l) = self.level.as_mut() else { return false };
        l.active = true;
        let changed = match c {
            LevelCommand::Op(op) => match l.editor.apply(op) {
                Ok(made) => {
                    self.level_made = made;
                    self.level_error = None;
                    true
                }
                Err(e) => {
                    self.level_error = Some(e);
                    return false;
                }
            },
            LevelCommand::Undo => l.editor.undo(),
            LevelCommand::Redo => l.editor.redo(),
            LevelCommand::Open(doc) => {
                l.editor.open(*doc);
                true
            }
            LevelCommand::Saved => {
                l.editor.saved();
                return true;
            }
            LevelCommand::Select(ids) => {
                l.selected = ids;
                return true;
            }
            LevelCommand::WorkLight(on) => {
                l.work_light = on;
                return true;
            }
            LevelCommand::Floor(y) => {
                l.floor = y;
                return true;
            }
            LevelCommand::Place { at, yaw, pitch } => {
                let eye = l.to_world(at);
                self.place(eye, yaw, pitch);
                return true;
            }
        };
        if changed {
            l.refresh();
        }
        changed
    }

    /// Prøveturen i nivået (verden er nettopp bygd med dokumentet som nivå
    /// 1): fra startstedet, eller der kameraet sto (`here`, med føttene på
    /// gulvet under). `back` er kameraet redigeringen kommer tilbake til.
    /// Du kan ikke dø, og lykta er full.
    pub fn level_play(&mut self, back: (Vec3, f32, f32), here: bool) -> bool {
        let Some(l) = self.level.as_mut() else { return false };
        l.active = false;
        l.camera = Some(back);
        self.immortal = true;
        self.hero.lamp.fill();
        let k = 0;
        if !self.go_to_level(k) {
            return false;
        }
        if here {
            let (eye, yaw, _) = back;
            let all = |_: &crypt::nav::Poly| true;
            if let Some((_, feet)) = self.crypt.levels[k].nav.nearest(eye - Vec3::Y * 1.6, 8.0, &all) {
                self.teleport(feet, yaw);
            }
        }
        true
    }

    /// Ta redigeringen ut av økta (før verden bygges på nytt til prøveturen).
    pub fn take_level_editor(&mut self) -> Option<LevelEditor> {
        self.level.take().map(|l| l.editor)
    }

    /// Tilbake fra prøveturen til redigeringen, med kameraet der det var.
    pub fn level_back(&mut self) -> bool {
        let Some(l) = self.level.as_mut() else { return false };
        l.active = true;
        let cam = l.camera.take();
        self.immortal = false;
        if let Some((eye, yaw, pitch)) = cam {
            self.place(eye, yaw, pitch);
        }
        true
    }

    /// Kameraet (øyet, yaw, pitch) i dokumentets rom.
    pub fn level_camera(&self) -> Option<(Vec3, f32, f32)> {
        let l = self.level.as_ref()?;
        Some((self.eye() - l.origin, self.yaw, self.pitch))
    }

    /// Punktet på gulvet kartet viser under pekeren i 3D-vinduet (0..1 fra
    /// øverst til venstre), i dokumentets rom.
    pub fn level_pick(&self, at: (f32, f32)) -> Option<Vec3> {
        let l = self.level.as_ref()?;
        let (o, d) = self.screen_ray(at);
        let y = l.origin.y + l.floor;
        if d.y.abs() < 1e-4 {
            return None;
        }
        let t = (y - o.y) / d.y;
        (t > 0.0 && t < 400.0).then(|| o + d * t - l.origin)
    }

    /// Ett steg i redigeringen: du flyr fritt (som i byens redigering).
    pub(super) fn level_tick(&mut self, input: &mut Input, dt: f32) {
        self.yaw -= input.look.0 * super::session::SENSITIVITY;
        self.pitch = (self.pitch - input.look.1 * super::session::SENSITIVITY).clamp(-1.5, 1.5);
        input.look = (0.0, 0.0);
        input.actions.clear();
        self.fly(input, dt);
    }

    /// Bildet i redigeringen: nivået slik det er nå i stedet for det faste,
    /// uten himmel og tåke, med faklene og arbeidslyset.
    pub fn level_frame(&self, f: &mut Frame) {
        let Some(l) = self.level.as_ref().filter(|l| l.active) else { return };
        f.instances = l.shown.clone();
        f.skins.clear();
        f.hidden = l.hidden.clone();
        f.band = l.band;
        f.fog_density = 0.0;
        f.sun_intensity = 0.0;
        f.sky_top = Vec3::splat(0.002);
        f.sky_horizon = Vec3::splat(0.003);
        f.hurt = 0.0;
        let eye = f.camera;
        // Arbeidslyset lyser gjennom murene; spillets lys følger rommene som
        // nede i krypten (`Crypt::light_mask`).
        let mut lights: Vec<Light> = if l.work_light { l.lights.iter().map(|x| Light { room: None, ..*x }).collect() } else { l.lights.clone() };
        lights.sort_by(|a, b| a.pos.distance_squared(eye).total_cmp(&b.pos.distance_squared(eye)));
        if l.work_light {
            // Arbeidslyset: et stort, varmt lys over deg og litt mer lys fra
            // bakken, så alt synes (himmelen er svart, som nede i krypten).
            lights.insert(0, Light { pos: eye + Vec3::Y * 1.5, radius: 30.0, color: Vec3::new(1.0, 0.9, 0.78), intensity: 1.8, always: true, flicker: 0.0, room: None });
            f.ground_bounce = Vec3::splat(0.03);
            f.exposure = 0.8;
        } else {
            lights.insert(0, Light { pos: eye - Vec3::Y * 0.9, radius: 11.0, color: Vec3::new(1.0, 0.7, 0.4), intensity: 2.4, always: true, flicker: 0.0, room: None });
            f.ground_bounce = Vec3::splat(0.004);
            f.exposure = 0.75;
        }
        lights.truncate(MAX_LIGHTS);
        // Rommene til rendereren: da er himmelen svart (ingen skyer når
        // kameraet står utenfor nivået) og bildet gradert som i kjelleren.
        // Med arbeidslyset er det én rute langt borte, og alle lys lyser overalt.
        f.rooms = Some(match (l.work_light, l.crypt.levels.first()) {
            (false, Some(level)) => RoomLight {
                boxes: level.rooms.iter().take(MAX_ROOMS).map(|r| (r.min, r.max, r.y - r.sink, r.y + r.h + r.rise)).collect(),
                masks: lights.iter().map(|x| x.room.map_or(u64::MAX, |(lv, r)| if lv == 0 { l.crypt.light_mask(0, r as usize) } else { 0 })).collect(),
            },
            _ => RoomLight { boxes: vec![(Vec2::splat(1.0e6), Vec2::splat(1.0e6 + 1.0), 1.0e6, 1.0e6 + 1.0)], masks: vec![u64::MAX; lights.len()] },
        });
        f.lights = lights;
        // Med spillets lys: tåka i rommet kameraet står i (stemningen).
        if let Some((lv, r)) = l.crypt.room_at(eye - Vec3::Y * 1.4).filter(|_| !l.work_light) {
            f.fog_density = l.crypt.levels[lv].rooms[r].fog * 0.09;
            f.fog_falloff = 1.0e-4;
            f.fog_color = Vec3::new(0.035, 0.042, 0.045);
        }
        f.lines = l.lines();
    }
}

/// Modellene redigeringen må ha plass til (bitene, mekanismene og pynten):
/// navnet og hvor mange.
pub fn level_reserves(kit: &crate::domain::world::crypt::kit::KitDef, infos: &[ModelInfo]) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = kit.models().into_iter().map(|m| (m.to_string(), LEVEL_RESERVE)).collect();
    let extra = infos.iter().map(|i| i.name.as_str()).filter(|n| n.starts_with("kit_")).chain(crypt::PROPS.iter().copied()).chain([crypt::CHEST, crypt::CHEST_LID, crypt::PORTCULLIS]);
    for n in extra {
        if !out.iter().any(|(m, _)| m == n) {
            out.push((n.to_string(), THING_RESERVE));
        }
    }
    out
}
