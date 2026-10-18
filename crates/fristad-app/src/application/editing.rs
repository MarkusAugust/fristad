//! Redigeringen i økta (`docs/redigering.md`): kommandoene fra nettlaget går
//! til `Editor` i kjernen, du flyr fritt og gjennom alt, musa velger og drar
//! i håndtakene, og bildet viser det som er flyttet på det nye stedet
//! (`Frame.hidden` og `instances`) og markeringene (`Frame.lines`) til byen
//! bygges på nytt når du trykker P.

use super::session::Session;
use crate::domain::edit::{ghost, rules, Editor};
use crate::domain::scene::ModelInstance;
use crate::domain::world::layout::Ref;
use crate::domain::world::terrain;
use crate::ports::driving::{EditCommand, EditView, Event, Grab, Input};
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3, Vec4};
use std::collections::HashMap;

/// Fart i redigeringen (m/s), og med Shift.
const FLY: f32 = 6.0;
const FLY_FAST: f32 = 24.0;
/// Hvor langt pekeren rekker når en ny ting settes ut.
const REACH: f32 = 120.0;
/// Hvor nær et håndtak pekeren må være (brøkdel av skjermhøyden).
const GRAB: f32 = 0.018;

/// Fargene på markeringene (lyse nok til å synes gjennom tonemappingen).
const SELECTED: Vec3 = Vec3::new(3.0, 2.4, 0.4);
const BREACH: Vec3 = Vec3::new(4.0, 0.3, 0.2);
const DRAFT: Vec3 = Vec3::new(3.0, 3.0, 3.0);
const AXES: [(Vec3, Vec3); 3] = [(Vec3::X, Vec3::new(4.0, 0.4, 0.3)), (Vec3::Y, Vec3::new(0.4, 4.0, 0.4)), (Vec3::Z, Vec3::new(0.4, 0.8, 4.0))];
const RING: Vec3 = Vec3::new(3.5, 2.8, 0.6);
/// Fantomet: grønt når alt er i orden, rødt når en regel brytes.
const GHOST_OK: Vec3 = Vec3::new(0.4, 4.0, 0.6);
const GHOST_BAD: Vec3 = BREACH;

/// Så langt (brøkdel av skjermhøyden) musa må gå før et trykk på en ting
/// blir et drag, så et klikk ikke flytter noe.
const SLACK: f32 = 0.006;

/// Det du drar i.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Handle {
    Axis(usize),
    Ring,
    /// Tingen selv, langs bakken (planet der det valgte står).
    Ground,
    /// Opp og ned med Alt, uansett hvor du drar.
    Lift,
    /// Drei med Ctrl eller Cmd: bortover i bildet dreier.
    Turn,
    /// Et punkt på ruta til den valgte innbyggeren, langs bakken.
    Route(usize),
}

/// Musa og linjeverktøyet i redigeringen.
#[derive(Default)]
pub struct EditUi {
    pub(super) drag: Option<(Handle, (f32, f32))>,
    /// Der knappen ble trykket, til musa har gått `SLACK` (da er det et drag).
    pub(super) pressed: Option<(f32, f32)>,
    /// Trykket på noe som alt var valgt: velges når knappen slippes uten drag.
    pub(super) click: Option<(f32, f32)>,
    pub(super) draft: Vec<Vec3>,
    /// Fantomet: modellen, pekeren, posen og det som er galt der.
    pub(super) ghost: Option<(String, (f32, f32), Affine3A, Vec<String>)>,
    /// Hvor mye fantomet er dreid (R).
    pub(super) ghost_turn: f32,
    /// Fest til rutenett og vinkel.
    pub(super) snap: bool,
    /// Det draget har flyttet og dreid som ikke er festet ennå.
    pub(super) carry: Vec3,
    pub(super) carry_turn: f32,
    /// Det som sto på noe da draget langs bakken begynte (settes ned igjen).
    pub(super) standing: Vec<Ref>,
    /// Feilen fra siste egenskap, vist én gang.
    pub(super) error: Option<String>,
    /// Punktet på ruta som er tatt (Delete sletter det).
    pub(super) route_point: Option<usize>,
}

/// En pose som nøkkel (millimeter), til å se hva som står likt.
fn pose_key(model: &str, t: &Affine3A) -> (String, [i32; 12]) {
    let a = t.to_cols_array();
    (model.to_string(), a.map(|v| (v * 1000.0).round() as i32))
}

/// De tolv kantene av en boks (midte og halve mål i rommet til `t`).
fn box_lines(t: &Affine3A, c: Vec3, h: Vec3, color: Vec3, out: &mut Vec<(Vec3, Vec3, Vec3)>) {
    let corner = |k: usize| t.transform_point3(c + h * Vec3::new(if k & 1 == 0 { -1.0 } else { 1.0 }, if k & 2 == 0 { -1.0 } else { 1.0 }, if k & 4 == 0 { -1.0 } else { 1.0 }));
    for (a, b) in [(0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (1, 3), (4, 6), (5, 7), (0, 4), (1, 5), (2, 6), (3, 7)] {
        out.push((corner(a), corner(b), color));
    }
}

impl Session {
    pub fn editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Redigeringen, til testene og nettlaget.
    pub fn editor(&self) -> Option<&Editor> {
        self.edit.as_ref()
    }

    /// Øyet og retningen (yaw, pitch), så spillet kan bygges på nytt der
    /// du står (P).
    pub fn view_pose(&self) -> (Vec3, f32, f32) {
        (self.eye(), self.yaw, self.pitch)
    }

    /// Kameraet slik bildet tegnes (uten kameraspark i redigeringen).
    fn view_proj(&self) -> Mat4 {
        let aspect = self.aspect.get().max(0.1);
        Mat4::perspective_rh(self.fov().to_radians(), aspect, 0.1, 2500.0) * Mat4::look_to_rh(self.eye(), self.forward(), Vec3::Y)
    }

    /// Strålen gjennom et punkt på skjermen (0..1 fra øverst til venstre).
    pub fn screen_ray(&self, at: (f32, f32)) -> (Vec3, Vec3) {
        let inv = self.view_proj().inverse();
        let ndc = Vec2::new(at.0 * 2.0 - 1.0, 1.0 - at.1 * 2.0);
        let near = inv * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
        let far = inv * Vec4::new(ndc.x, ndc.y, 1.0, 1.0);
        let (a, b) = (near.truncate() / near.w, far.truncate() / far.w);
        (self.eye(), (b - a).normalize())
    }

    /// Et punkt i verden på skjermen, i enheter av skjermhøyden (så x og y
    /// er like lange), eller `None` bak kameraet.
    fn on_screen(&self, p: Vec3) -> Option<Vec2> {
        let c = self.view_proj() * p.extend(1.0);
        if c.w <= 1e-4 {
            return None;
        }
        let n = c.truncate() / c.w;
        Some(Vec2::new((n.x + 1.0) * 0.5 * self.aspect.get().max(0.1), (1.0 - n.y) * 0.5))
    }

    /// Et punkt i verden på skjermen (0..1 fra øverst til venstre).
    pub fn to_screen(&self, p: Vec3) -> Option<(f32, f32)> {
        self.on_screen(p).map(|v| (v.x / self.aspect.get().max(0.1), v.y))
    }

    /// Midten av det valgte og lengden på håndtakene (til testene og
    /// nettlaget).
    pub fn handles(&self) -> Option<(Vec3, f32)> {
        self.pivot()
    }

    pub(super) fn screen_point(&self, at: (f32, f32)) -> Vec2 {
        Vec2::new(at.0 * self.aspect.get().max(0.1), at.1)
    }

    /// Hvor langt punktet `m` på skjermen er fra streken a–b i verden.
    pub(super) fn screen_distance(&self, m: Vec2, a: Vec3, b: Vec3) -> Option<f32> {
        let (a, b) = (self.on_screen(a)?, self.on_screen(b)?);
        let ab = b - a;
        let t = ((m - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
        Some((m - (a + ab * t)).length())
    }

    /// En kommando fra redigeringen. Gir om den gjorde noe.
    pub fn edit_command(&mut self, c: EditCommand) -> bool {
        if let EditCommand::Enter = c {
            let Some(d) = self.edit_data.clone() else {
                return false;
            };
            let mut e = Editor::new(d.sources, d.layout, d.contents, d.approved);
            if let Some((l, c, k)) = d.baseline {
                e.set_baseline(l, c, k);
            }
            self.edit = Some(e);
            self.edit_ui = EditUi::default();
            self.push_edit_view();
            return true;
        }
        if self.edit.is_none() {
            return false;
        }
        let done = match c {
            EditCommand::Enter => true,
            EditCommand::Leave => {
                self.edit = None;
                self.pending.push(Event::Edit(None));
                return true;
            }
            EditCommand::Press { at, add, grab } => {
                self.edit_ui.carry = Vec3::ZERO;
                self.edit_ui.carry_turn = 0.0;
                self.edit_ui.standing.clear();
                // Ruta til den valgte innbyggeren: en stolpe tas, et trykk
                // på streken legger til et punkt der (og det dras).
                // Det nye punktet og draget angres som ett steg.
                let point = match (grab, self.route_point_at(at), self.route_npc()) {
                    (Grab::Move, Some(k), _) => {
                        self.editor_mut().begin_group();
                        Some(k)
                    }
                    // Ikke når pekeren står på innbyggeren selv (den dras).
                    (Grab::Move, None, Some(r)) if !self.on_selected(at) => self.route_insert_at(at).and_then(|(k, p)| {
                        self.editor_mut().begin_group();
                        self.editor_mut().route_insert(&r, k, p)
                    }),
                    _ => None,
                };
                self.edit_ui.route_point = point;
                if let Some(k) = point {
                    self.edit_ui.drag = Some((Handle::Route(k), at));
                    self.edit_ui.pressed = Some(at);
                    self.push_edit_view();
                    return true;
                }
                let handle = match grab {
                    Grab::Lift => Some(Handle::Lift),
                    Grab::Turn => Some(Handle::Turn),
                    // Pilene går gjennom tingen: står pekeren på det
                    // valgte, er det tingen du tar, ikke pila.
                    Grab::Move => self.handle_at(at).filter(|h| {
                        let (o, d) = self.screen_ray(at);
                        let e = self.edit.as_ref().expect("i redigeringen");
                        *h == Handle::Ring || !e.hit(o, d).is_some_and(|r| e.selection.contains(&r))
                    }),
                };
                match handle {
                    // Med Alt eller Ctrl gjelder draget det valgte; er
                    // ingenting valgt, velges det under pekeren først.
                    Some(h) => {
                        if self.editor_mut().selection.is_empty() {
                            self.pick_at(Some(at), add);
                        }
                        self.edit_ui.drag = Some((h, at));
                        self.edit_ui.pressed = (h == Handle::Lift || h == Handle::Turn).then_some(at);
                        self.editor_mut().begin_group();
                        true
                    }
                    // Trykk på en ting: den velges (er den alt valgt, blir
                    // valget som det er, så flere kan dras sammen), og den
                    // dras langs bakken når musa går.
                    None => {
                        let (o, d) = self.screen_ray(at);
                        let e = self.editor_mut();
                        let first = e.hit(o, d);
                        let keep = !add && first.as_ref().is_some_and(|r| e.selection.contains(r));
                        // Er den alt valgt, venter valget til knappen slippes
                        // uten drag (klikk igjen gir det neste under pekeren).
                        self.edit_ui.click = keep.then_some(at);
                        let changed = keep || self.pick_at(Some(at), add);
                        if !add && first.is_some() && !self.editor_mut().selection.is_empty() {
                            self.edit_ui.drag = Some((Handle::Ground, at));
                            self.edit_ui.pressed = Some(at);
                            self.edit_ui.standing = self.editor_mut().standing();
                            self.editor_mut().begin_group();
                        }
                        changed
                    }
                }
            }
            EditCommand::Drag { to } => self.drag_to(to),
            EditCommand::Release => {
                let click = self.edit_ui.click.take().filter(|_| self.edit_ui.pressed.is_some());
                // Alt+klikk uten drag: det bakerste under pekeren.
                let deepest = match (self.edit_ui.drag, self.edit_ui.pressed) {
                    (Some((Handle::Lift, _)), Some(at)) => Some(at),
                    _ => None,
                };
                // Det som sto på noe, settes ned igjen etter et drag langs
                // bakken (i samme angresteg som draget).
                let standing = std::mem::take(&mut self.edit_ui.standing);
                if matches!(self.edit_ui.drag, Some((Handle::Ground, _))) && self.edit_ui.pressed.is_none() && !standing.is_empty() {
                    self.editor_mut().settle(&standing);
                }
                self.edit_ui.drag = None;
                self.edit_ui.pressed = None;
                self.editor_mut().end_group();
                if let Some(at) = click {
                    self.pick_at(Some(at), false);
                }
                if let Some(at) = deepest {
                    let (o, d) = self.screen_ray(at);
                    self.editor_mut().pick_deepest(o, d);
                }
                true
            }
            EditCommand::Pick { add, at } => self.pick_at(at, add),
            EditCommand::Cycle => self.editor_mut().cycle().is_some(),
            EditCommand::Move(d) => {
                self.editor_mut().translate(d);
                true
            }
            EditCommand::Rotate(a) => {
                self.editor_mut().rotate(a);
                true
            }
            EditCommand::SetPose { r, pose } => {
                self.editor_mut().set_pose(&r, pose);
                true
            }
            EditCommand::Drop => {
                self.editor_mut().drop_to_surface();
                true
            }
            EditCommand::Copy => {
                self.editor_mut().duplicate();
                true
            }
            // Er et punkt på ruta tatt, er det punktet som slettes.
            EditCommand::Delete => match (self.edit_ui.route_point.take(), self.route_npc()) {
                (Some(k), Some(r)) => self.editor_mut().route_remove(&r, k),
                _ => {
                    self.editor_mut().delete();
                    true
                }
            },
            EditCommand::Group(name) => match self.editor_mut().make_group(&name) {
                Ok(_) => true,
                Err(why) => {
                    self.edit_ui.error = Some(why);
                    false
                }
            },
            EditCommand::Ungroup => self.editor_mut().ungroup() > 0,
            EditCommand::Place { model, at } => {
                self.edit_ui.ghost = None;
                match self.ghost_pose(&model, at) {
                    Some(pose) => self.editor_mut().place(&model, pose).is_some(),
                    None => false,
                }
            }
            EditCommand::Ghost(g) => {
                self.edit_ui.ghost = g.and_then(|(model, at)| {
                    let pose = self.ghost_pose(&model, Some(at))?;
                    let bad = self.edit.as_ref()?.ghost_breaches(&model, &pose);
                    Some((model, at, pose, bad))
                });
                true
            }
            EditCommand::GhostTurn(a) => {
                self.edit_ui.ghost_turn = (self.edit_ui.ghost_turn + a).rem_euclid(std::f32::consts::TAU);
                if let Some((model, at, _, _)) = self.edit_ui.ghost.clone() {
                    return self.edit_command(EditCommand::Ghost(Some((model, at))));
                }
                true
            }
            EditCommand::Snap(on) => {
                self.edit_ui.snap = on;
                true
            }
            EditCommand::SetProp { key, value } => match self.editor_mut().set_prop(&key, &value) {
                Ok(()) => true,
                Err(why) => {
                    self.edit_ui.error = Some(why);
                    false
                }
            },
            EditCommand::UndoTo(n) => self.editor_mut().undo_to(n),
            EditCommand::SelectUnder { k, add } => self.editor_mut().select_under(k, add).is_some(),
            EditCommand::CycleBack => self.editor_mut().cycle_back().is_some(),
            EditCommand::SelectBox { a, b, add } => {
                let mut e = self.edit.take().expect("i redigeringen");
                let n = e.select_box(self.eye(), REACH, a, b, add, |p| self.to_screen(p));
                self.edit = Some(e);
                n > 0
            }
            EditCommand::LinePoint { at } => match self.ground_at(Some(at)) {
                Some(p) => {
                    self.edit_ui.draft.push(p);
                    true
                }
                None => false,
            },
            EditCommand::LineFinish(kind) => {
                let points = std::mem::take(&mut self.edit_ui.draft);
                !self.editor_mut().line(&points, &kind).is_empty()
            }
            EditCommand::LineCancel => {
                self.edit_ui.draft.clear();
                true
            }
            EditCommand::Line { points, kind } => !self.editor_mut().line(&points, &kind).is_empty(),
            EditCommand::WallHeight(h) => {
                self.editor_mut().set_wall_height(h);
                true
            }
            EditCommand::Undo => self.editor_mut().undo(),
            EditCommand::Redo => self.editor_mut().redo(),
        };
        self.push_edit_view();
        done
    }

    /// Pekeren står på noe som er valgt.
    fn on_selected(&self, at: (f32, f32)) -> bool {
        let (o, d) = self.screen_ray(at);
        self.edit.as_ref().is_some_and(|e| e.hit(o, d).is_some_and(|r| e.selection.contains(&r)))
    }

    fn editor_mut(&mut self) -> &mut Editor {
        self.edit.as_mut().expect("i redigeringen")
    }

    fn pick_at(&mut self, at: Option<(f32, f32)>, add: bool) -> bool {
        let (o, d) = match at {
            Some(at) => self.screen_ray(at),
            None => (self.eye(), self.forward()),
        };
        self.editor_mut().pick(o, d, add).is_some()
    }

    fn push_edit_view(&mut self) {
        let Some(e) = &self.edit else { return };
        let view = EditView {
            selected: e.selected(),
            breaches: e.breaches().iter().map(|b| (b.r.clone(), b.why.clone())).collect(),
            can_undo: e.can_undo(),
            can_redo: e.can_redo(),
            dirty: e.dirty(),
            clean: e.clean(),
            draft: self.edit_ui.draft.len(),
            under: e.under().iter().map(|r| (e.label(r), e.selection.contains(r))).collect(),
            props: e.props(),
            history: e.history().0,
            step: e.history().1,
            snap: self.edit_ui.snap,
            ghost: self.edit_ui.ghost.as_ref().map(|g| g.3.clone()),
            error: self.edit_ui.error.take(),
            group: e.single_group(),
            route_point: self.edit_ui.route_point.filter(|_| self.route_npc().is_some()),
        };
        self.pending.push(Event::Edit(Some(view)));
    }

    /// Der strålen treffer bakken eller noe å stå på.
    fn ground_hit(&self, eye: Vec3, dir: Vec3) -> Option<Vec3> {
        let e = self.edit.as_ref()?;
        let mut last = eye;
        let mut s = 0.0;
        while s < REACH {
            s += 0.1;
            let p = eye + dir * s;
            let ground = e.surface_under(p.x, p.z, last.y.max(p.y) + 0.01, None);
            if p.y <= ground {
                return Some(Vec3::new(p.x, ground, p.z));
            }
            last = p;
        }
        None
    }

    /// Der pekeren (eller midten av bildet) treffer bakken eller noe å stå på.
    pub fn ground_at(&self, at: Option<(f32, f32)>) -> Option<Vec3> {
        let (o, d) = match at {
            Some(at) => self.screen_ray(at),
            None => (self.eye(), self.forward()),
        };
        self.ground_hit(o, d)
    }

    /// Der en ny `model` settes under pekeren: vendt mot deg (modellene har
    /// forsida mot lokal +Z) og dreid med R, festet om festet er på, og
    /// satt ned så bunnen av modellen står på det som er under.
    pub fn ghost_pose(&self, model: &str, at: Option<(f32, f32)>) -> Option<Affine3A> {
        let e = self.edit.as_ref()?;
        let p = self.ground_at(at)?;
        let mut pose = Affine3A::from_rotation_translation(Quat::from_rotation_y(self.yaw + self.edit_ui.ghost_turn), p);
        if self.edit_ui.snap {
            pose = ghost::snap_pose(pose);
        }
        let o = Vec3::from(pose.translation);
        let ground = e.surface_under(o.x, o.z, p.y + 0.3, None);
        let foot = e.scene().bounds(model).map(|(lo, hi)| rules::world_box(lo, hi, &pose).0.y).unwrap_or(o.y);
        pose.translation.y += ground - foot;
        Some(pose)
    }

    /// Der siktekorset treffer (midt i bildet).
    pub fn aim_point(&self) -> Option<Vec3> {
        self.ground_at(None)
    }

    /// Midten av det valgte og hvor lange håndtakene er.
    fn pivot(&self) -> Option<(Vec3, f32)> {
        let e = self.edit.as_ref()?;
        let sel = e.selected();
        if sel.is_empty() {
            return None;
        }
        let p = sel.iter().map(|s| Vec3::from(s.pose.translation)).sum::<Vec3>() / sel.len() as f32;
        Some((p, (p.distance(self.eye()) * 0.12).clamp(0.4, 4.0)))
    }

    fn ring_points(p: Vec3, len: f32) -> Vec<Vec3> {
        (0..=32).map(|k| p + Quat::from_rotation_y(k as f32 / 32.0 * std::f32::consts::TAU) * Vec3::X * len * 0.8).collect()
    }

    /// Håndtaket under pekeren, om noe.
    fn handle_at(&self, at: (f32, f32)) -> Option<Handle> {
        let (p, len) = self.pivot()?;
        let m = self.screen_point(at);
        let near = |a: Vec3, b: Vec3| self.screen_distance(m, a, b);
        let mut best: Option<(f32, Handle)> = None;
        for (k, (axis, _)) in AXES.iter().enumerate() {
            if let Some(d) = near(p, p + *axis * len) {
                if d < GRAB && best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, Handle::Axis(k)));
                }
            }
        }
        for w in Self::ring_points(p, len).windows(2) {
            if let Some(d) = near(w[0], w[1]) {
                if d < GRAB && best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, Handle::Ring));
                }
            }
        }
        best.map(|(_, h)| h)
    }

    /// Dra det valgte med håndtaket fra forrige punkt til `to`.
    fn drag_to(&mut self, to: (f32, f32)) -> bool {
        let (Some((handle, from)), Some((p, _))) = (self.edit_ui.drag, self.pivot()) else {
            return false;
        };
        // Et trykk blir først et drag når musa har gått et stykke.
        if let Some(at) = self.edit_ui.pressed {
            if (self.screen_point(to) - self.screen_point(at)).length() < SLACK {
                return false;
            }
            self.edit_ui.pressed = None;
        }
        self.edit_ui.drag = Some((handle, to));
        let on_plane = |at: (f32, f32)| {
            let (o, d) = self.screen_ray(at);
            (d.y.abs() > 1e-4).then(|| o + d * ((p.y - o.y) / d.y)).filter(|_| ((p.y - o.y) / d.y) > 0.0)
        };
        match handle {
            Handle::Route(k) => {
                let (Some(r), Some(at)) = (self.route_npc(), self.ground_at(Some(to))) else {
                    return false;
                };
                let at = if self.edit_ui.snap { Vec3::new(ghost::snap(at.x, ghost::GRID), at.y, ghost::snap(at.z, ghost::GRID)) } else { at };
                self.editor_mut().route_move(&r, k, at);
            }
            Handle::Ground => {
                let (Some(a), Some(b)) = (on_plane(from), on_plane(to)) else {
                    return false;
                };
                self.translate_snapped(p, Vec3::new(b.x - a.x, 0.0, b.z - a.z));
            }
            Handle::Lift => {
                let (Some(a), Some(b)) = (self.on_screen(p), self.on_screen(p + Vec3::Y)) else {
                    return false;
                };
                let s = b - a;
                let moved = (self.screen_point(to) - self.screen_point(from)).dot(s) / s.length_squared().max(1e-9);
                self.translate_snapped(p, Vec3::Y * moved);
            }
            Handle::Turn => {
                // En bildebredde bortover er en hel runde; mot høyre dreier
                // med klokka sett ovenfra.
                let turn = -(to.0 - from.0) * std::f32::consts::TAU;
                self.rotate_snapped(turn);
            }
            Handle::Axis(k) => {
                let axis = AXES[k].0;
                let (Some(a), Some(b)) = (self.on_screen(p), self.on_screen(p + axis)) else {
                    return false;
                };
                let s = b - a;
                let moved = (self.screen_point(to) - self.screen_point(from)).dot(s) / s.length_squared().max(1e-9);
                self.translate_snapped(p, axis * moved);
            }
            Handle::Ring => {
                // Vinkelen rundt midten i planet der det valgte står.
                let (Some(a), Some(b)) = (on_plane(from), on_plane(to)) else {
                    return false;
                };
                let angle = |v: Vec3| (-(v.z - p.z)).atan2(v.x - p.x);
                let mut turn = angle(b) - angle(a);
                if turn > std::f32::consts::PI {
                    turn -= std::f32::consts::TAU;
                } else if turn < -std::f32::consts::PI {
                    turn += std::f32::consts::TAU;
                }
                self.rotate_snapped(turn);
            }
        }
        true
    }

    /// Flytt det valgte `delta`; med festet på samles flyttingen til midten
    /// `p` havner på rutenettet (bare langs aksene som flyttes).
    fn translate_snapped(&mut self, p: Vec3, delta: Vec3) {
        if !self.edit_ui.snap {
            self.editor_mut().translate(delta);
            return;
        }
        self.edit_ui.carry += delta;
        let target = p + self.edit_ui.carry;
        let mut step = Vec3::ZERO;
        for k in 0..3 {
            if self.edit_ui.carry[k] != 0.0 {
                step[k] = ghost::snap(target[k], ghost::GRID) - p[k];
            }
        }
        if step != Vec3::ZERO {
            self.edit_ui.carry -= step;
            self.editor_mut().translate(step);
        }
    }

    /// Drei det valgte `turn`; med festet på i hele steg på 15°.
    fn rotate_snapped(&mut self, turn: f32) {
        if !self.edit_ui.snap {
            self.editor_mut().rotate(turn);
            return;
        }
        self.edit_ui.carry_turn += turn;
        let n = (self.edit_ui.carry_turn / ghost::ANGLE).round();
        if n != 0.0 {
            self.edit_ui.carry_turn -= n * ghost::ANGLE;
            self.editor_mut().rotate(n * ghost::ANGLE);
        }
    }

    /// Markeringene: boksene rundt det valgte (gule) og det som bryter en
    /// regel (røde), håndtakene, og linjeutkastet.
    pub(super) fn edit_lines(&self) -> Vec<(Vec3, Vec3, Vec3)> {
        let Some(e) = &self.edit else { return vec![] };
        let mut out = vec![];
        let mut seen = std::collections::HashSet::new();
        for b in e.breaches() {
            if seen.insert(b.r.clone()) {
                if let Some((t, c, h)) = e.box_of(&b.r) {
                    box_lines(&t, c, h, BREACH, &mut out);
                }
            }
        }
        for s in e.selected() {
            box_lines(&s.pose, s.center, s.half * 1.01, SELECTED, &mut out);
        }
        if let Some((p, len)) = self.pivot() {
            for (axis, color) in AXES {
                out.push((p, p + axis * len, color));
            }
            for w in Self::ring_points(p, len).windows(2) {
                out.push((w[0], w[1], RING));
            }
        }
        if let Some((model, _, pose, bad)) = &self.edit_ui.ghost {
            if let Some((lo, hi)) = e.scene().bounds(model) {
                box_lines(pose, (lo + hi) * 0.5, (hi - lo) * 0.5, if bad.is_empty() { GHOST_OK } else { GHOST_BAD }, &mut out);
            }
        }
        for w in self.edit_ui.draft.windows(2) {
            out.push((w[0] + Vec3::Y * 0.05, w[1] + Vec3::Y * 0.05, DRAFT));
        }
        for p in &self.edit_ui.draft {
            out.push((*p, *p + Vec3::Y * 1.5, DRAFT));
        }
        self.mark_lines(&mut out);
        out
    }

    /// Flyging i redigeringen: rett dit du ser, gjennom alt; mellomrom
    /// stiger, C synker, Shift går fort.
    pub(super) fn fly(&mut self, input: &Input, dt: f32) {
        let fwd = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(-fwd.z, 0.0, fwd.x);
        let mut wish = Vec3::ZERO;
        for (on, d) in [(input.forward, fwd), (input.back, -fwd), (input.right, right), (input.left, -right), (input.jump, Vec3::Y), (input.crouch, -Vec3::Y)] {
            if on && !self.paused {
                wish += d;
            }
        }
        let speed = if input.sprint { FLY_FAST } else { FLY };
        let mut pos = self.body.pos + wish.normalize_or_zero() * speed * dt;
        // Ikke under bakken oppe i byen (krypten nås med døra, ikke ved å
        // fly ned); nede i krypten flyr du fritt i nivået.
        if self.body.pos.y > crate::domain::world::crypt::TOP {
            pos.y = pos.y.max(terrain::height(pos.x, pos.z) - 0.5);
        }
        self.body.pos = pos;
        self.body.vel_y = 0.0;
    }

    /// Det redigeringen har endret, til bildet: de faste instansene som ikke
    /// lenger står slik (skjules), og instansene som står et nytt sted.
    pub(super) fn edit_instances(&self) -> (Vec<(&'static str, usize)>, Vec<ModelInstance>) {
        let (Some(e), Some(d)) = (&self.edit, &self.edit_data) else {
            return (vec![], vec![]);
        };
        let mut base: HashMap<(String, [i32; 12]), Vec<(&'static str, usize)>> = HashMap::new();
        for (m, n, t, _) in &d.base {
            base.entry(pose_key(m, t)).or_default().push((*m, *n));
        }
        let mut shown = vec![];
        let scene = e.scene();
        for pl in &scene.placements {
            for (t, tint) in pl.transforms.iter().zip(&pl.tints) {
                match base.get_mut(&pose_key(pl.model, t)).and_then(|v| v.pop()) {
                    Some(_) => {}
                    None => shown.push(ModelInstance { model: pl.model, transform: *t, skin: None, flash: 0.0, tint: (*tint != Vec3::ONE).then_some(*tint) }),
                }
            }
        }
        // Fantomet, med den grønne eller røde boksen rundt (`edit_lines`).
        if let Some((model, _, pose, _)) = &self.edit_ui.ghost {
            if scene.bounds(model).is_some() {
                shown.push(ModelInstance { model: crate::domain::world::layout::intern(model), transform: *pose, skin: None, flash: 0.0, tint: None });
            }
        }
        let hidden = base.into_values().flatten().collect();
        (hidden, shown)
    }
}
