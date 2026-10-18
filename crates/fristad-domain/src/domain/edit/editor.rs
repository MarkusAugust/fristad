//! `Editor`: tilstanden i redigeringen. Dataene (byen, husenes innhold og
//! det som står i kryptnivåene), valget, angre og gjøre om, og byen og
//! krypten bygget fra dataene (`scene`) til reglene og til å finne tingene.
//! Alt eieren gjør er en endring av dataene; alt bygges på nytt etter hver.

use super::rules::{self, Breach};
use super::{crypt, scene, Sources};
use crate::domain::world::crypt::level::{CryptContents, LightDef, MarkDef};
use crate::domain::world::crypt::Crypt;
use crate::domain::world::layout::{Item, Layout, LightDef as ItemLightDef, NpcDef, Pose, Ref, SolidDef};
use crate::domain::world::{Builder, Shape};
use glam::{Affine3A, Quat, Vec3};
use std::collections::{BTreeMap, HashMap, HashSet};

/// En ting i dataene: en ting i byen eller et hus, en innbygger, eller et
/// merke eller lys i et kryptnivå.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Thing {
    Item(Item),
    Npc(NpcDef),
    Mark(MarkDef),
    Light(LightDef),
    /// Et lys på en ting i byen eller et hus (`Ref::ItemLight`).
    Lamp(ItemLightDef),
}

/// Én endring, som kan snus.
#[derive(Clone, Debug)]
pub(super) enum Change {
    Set { r: Ref, before: Thing, after: Thing },
    Insert { r: Ref, thing: Thing },
    Remove { r: Ref, thing: Thing },
}

impl Change {
    fn inverse(&self) -> Change {
        match self.clone() {
            Change::Set { r, before, after } => Change::Set { r, before: after, after: before },
            Change::Insert { r, thing } => Change::Remove { r, thing },
            Change::Remove { r, thing } => Change::Insert { r, thing },
        }
    }
}

/// Det valgte, slik det vises: tingen, navnet og boksen i verden (midte,
/// halve mål, dreining).
#[derive(Clone, Debug, PartialEq)]
pub struct Selected {
    pub r: Ref,
    pub label: String,
    pub pose: Affine3A,
    pub center: Vec3,
    pub half: Vec3,
}

pub struct Editor {
    pub(super) src: Sources,
    pub layout: Layout,
    pub contents: HashMap<String, Vec<Item>>,
    /// Det som står i kryptnivåene (merkene og lysene), per nivå.
    pub crypt: BTreeMap<u32, CryptContents>,
    /// Krypten slik den står nå, bygget fra dataene.
    pub(super) built: Crypt,
    /// Modellene eieren har godkjent (`approvals.json`); uten: ingen sjekk.
    pub(super) approved: Option<HashSet<String>>,
    pub selection: Vec<Ref>,
    pub(super) undo: Vec<Vec<Change>>,
    pub(super) redo: Vec<Vec<Change>>,
    /// Dataene slik de var da redigeringen begynte, til å se hva som er endret.
    original: HashMap<String, usize>,
    scene: Builder,
    breaches: Vec<Breach>,
    /// En gruppe endringer som angres som én (dra med håndtak): `Some(false)`
    /// før første endring, `Some(true)` når den er i gang.
    group: Option<bool>,
    /// Det siste klikket: strålen og alt den traff, nærmest først, så et nytt
    /// klikk på samme sted (eller Tab) velger det neste under pekeren.
    pub(super) last_ray: Option<(Vec3, Vec3)>,
    pub(super) under: Vec<Ref>,
}

/// En nøkkel for innholdet i en ting, til å telle like ting.
fn key(list: &str, t: &Thing) -> String {
    format!("{list}|{t:?}")
}

impl Editor {
    pub fn new(src: Sources, layout: Layout, contents: HashMap<String, Vec<Item>>, approved: Option<HashSet<String>>) -> Self {
        let crypt: BTreeMap<u32, CryptContents> = src.levels.iter().map(|(n, f)| (*n, CryptContents { marks: f.marks.clone(), lights: f.lights.clone() })).collect();
        let (scene, built) = scene(&src, &layout, &contents, &crypt);
        let mut e = Self {
            src,
            layout,
            contents,
            crypt,
            built,
            approved,
            selection: vec![],
            undo: vec![],
            redo: vec![],
            original: HashMap::new(),
            scene,
            breaches: vec![],
            group: None,
            last_ray: None,
            under: vec![],
        };
        e.original = e.census();
        e.refresh();
        e
    }

    /// Sammenlign med andre data enn dem redigeringen begynte med (dataene i
    /// fila, når spillet er bygget på nytt fra redigeringen med P): det som
    /// er endret siden da, må fortsatt være godkjent.
    pub fn set_baseline(&mut self, layout: Layout, contents: HashMap<String, Vec<Item>>, crypt: BTreeMap<u32, CryptContents>) {
        let base = Editor { layout, contents, crypt, ..Self::empty(self.src.clone()) };
        self.original = base.census();
        self.refresh();
    }

    fn empty(src: Sources) -> Self {
        let scene = Builder::new(&[]);
        Self {
            src,
            layout: Layout::default(),
            contents: HashMap::new(),
            crypt: BTreeMap::new(),
            built: Crypt::default(),
            approved: None,
            selection: vec![],
            undo: vec![],
            redo: vec![],
            original: HashMap::new(),
            scene,
            breaches: vec![],
            group: None,
            last_ray: None,
            under: vec![],
        }
    }

    /// Hvor mange av hver ting (etter innhold) dataene har.
    fn census(&self) -> HashMap<String, usize> {
        let mut out = HashMap::new();
        for r in self.refs() {
            if let (Some(t), list) = (self.get(&r), Self::list_of(&r)) {
                *out.entry(key(&list, &t)).or_insert(0) += 1;
            }
        }
        out
    }

    /// Begynn en gruppe endringer som angres som én (dra med et håndtak).
    pub fn begin_group(&mut self) {
        self.group = Some(false);
    }

    pub fn end_group(&mut self) {
        self.group = None;
    }

    pub(super) fn list_of(r: &Ref) -> String {
        match r {
            Ref::Town(_) => "town".into(),
            Ref::Inside { house, .. } => format!("inside/{house}"),
            Ref::Npc(_) => "npcs".into(),
            Ref::Crypt { level, .. } => format!("crypt/{level}"),
            Ref::CryptLight { level, .. } => format!("light/{level}"),
            Ref::ItemLight { item, model, .. } => format!("lamp/{item:?}/{model}"),
        }
    }

    /// Alle tingene i dataene.
    pub fn refs(&self) -> Vec<Ref> {
        let mut out: Vec<Ref> = (0..self.layout.items.len()).map(Ref::Town).collect();
        let mut houses: Vec<&String> = self.contents.keys().collect();
        houses.sort();
        for h in houses {
            out.extend((0..self.contents[h].len()).map(|i| Ref::Inside { house: h.clone(), i }));
        }
        out.extend((0..self.layout.npcs.len()).map(Ref::Npc));
        // Startstedet flytter hele nivået (origo regnes fra det), så det
        // flyttes ikke her.
        for (n, c) in &self.crypt {
            out.extend(c.marks.iter().enumerate().filter(|(_, m)| m.kind != "start" && m.pos.is_some()).map(|(i, _)| Ref::Crypt { level: *n, i }));
            out.extend((0..c.lights.len()).map(|i| Ref::CryptLight { level: *n, i }));
        }
        out.extend(self.lamp_refs());
        out
    }

    /// Det som er nytt eller endret siden redigeringen begynte (det som ikke
    /// finnes likt i dataene fra start).
    pub fn touched(&self) -> HashSet<Ref> {
        let mut left = self.original.clone();
        let mut out = HashSet::new();
        for r in self.refs() {
            let Some(t) = self.get(&r) else { continue };
            match left.get_mut(&key(&Self::list_of(&r), &t)) {
                Some(n) if *n > 0 => *n -= 1,
                _ => {
                    out.insert(r);
                }
            }
        }
        out
    }

    pub fn dirty(&self) -> bool {
        !self.touched().is_empty() || self.census() != self.original
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn breaches(&self) -> &[Breach] {
        &self.breaches
    }

    /// Byen slik den står nå, bygget fra dataene.
    pub fn scene(&self) -> &Builder {
        &self.scene
    }

    /// Alt er grønt: ingen regler brutt, så det kan lagres.
    pub fn clean(&self) -> bool {
        self.breaches.is_empty()
    }

    pub(super) fn get(&self, r: &Ref) -> Option<Thing> {
        match r {
            Ref::Town(i) => self.layout.items.get(*i).cloned().map(Thing::Item),
            Ref::Inside { house, i } => self.contents.get(house).and_then(|v| v.get(*i)).cloned().map(Thing::Item),
            Ref::Npc(i) => self.layout.npcs.get(*i).cloned().map(Thing::Npc),
            Ref::Crypt { level, i } => self.crypt.get(level).and_then(|c| c.marks.get(*i)).cloned().map(Thing::Mark),
            Ref::CryptLight { level, i } => self.crypt.get(level).and_then(|c| c.lights.get(*i)).cloned().map(Thing::Light),
            Ref::ItemLight { item, model, i } => self.get_lamp(item, *model, *i),
        }
    }

    fn apply(&mut self, c: &Change) {
        match c {
            Change::Set { r, after, .. } => match (r, after) {
                (Ref::Town(i), Thing::Item(it)) => self.layout.items[*i] = it.clone(),
                (Ref::Inside { house, i }, Thing::Item(it)) => self.contents.get_mut(house).expect("huset")[*i] = it.clone(),
                (Ref::Npc(i), Thing::Npc(n)) => self.layout.npcs[*i] = n.clone(),
                (Ref::Crypt { level, i }, Thing::Mark(m)) => self.crypt.get_mut(level).expect("nivået").marks[*i] = m.clone(),
                (Ref::CryptLight { level, i }, Thing::Light(l)) => self.crypt.get_mut(level).expect("nivået").lights[*i] = l.clone(),
                (Ref::ItemLight { item, model, i }, Thing::Lamp(l)) => self.apply_lamp(item, *model, *i, Some(l), false),
                _ => {}
            },
            Change::Insert { r, thing } => match (r, thing) {
                (Ref::Town(i), Thing::Item(it)) => self.layout.items.insert(*i, it.clone()),
                (Ref::Inside { house, i }, Thing::Item(it)) => self.contents.entry(house.clone()).or_default().insert(*i, it.clone()),
                (Ref::Npc(i), Thing::Npc(n)) => self.layout.npcs.insert(*i, n.clone()),
                (Ref::Crypt { level, i }, Thing::Mark(m)) => self.crypt.entry(*level).or_default().marks.insert(*i, m.clone()),
                (Ref::CryptLight { level, i }, Thing::Light(l)) => self.crypt.entry(*level).or_default().lights.insert(*i, l.clone()),
                (Ref::ItemLight { item, model, i }, Thing::Lamp(l)) => self.apply_lamp(item, *model, *i, Some(l), true),
                _ => {}
            },
            Change::Remove { r, .. } => match r {
                Ref::Town(i) => {
                    self.layout.items.remove(*i);
                }
                Ref::Inside { house, i } => {
                    self.contents.get_mut(house).expect("huset").remove(*i);
                }
                Ref::Npc(i) => {
                    self.layout.npcs.remove(*i);
                }
                Ref::Crypt { level, i } => {
                    self.crypt.get_mut(level).expect("nivået").marks.remove(*i);
                }
                Ref::CryptLight { level, i } => {
                    self.crypt.get_mut(level).expect("nivået").lights.remove(*i);
                }
                Ref::ItemLight { item, model, i } => self.apply_lamp(item, *model, *i, None, false),
            },
        }
    }

    /// Gjør endringene som én handling (ett steg å angre).
    pub(super) fn commit(&mut self, changes: Vec<Change>) {
        if changes.is_empty() {
            return;
        }
        for c in &changes {
            self.apply(c);
        }
        match (self.group, self.undo.last_mut()) {
            (Some(true), Some(last)) => last.extend(changes),
            _ => self.undo.push(changes),
        }
        if self.group.is_some() {
            self.group = Some(true);
        }
        self.redo.clear();
        self.refresh();
    }

    pub fn undo(&mut self) -> bool {
        let Some(changes) = self.undo.pop() else {
            return false;
        };
        for c in changes.iter().rev() {
            self.apply(&c.inverse());
        }
        self.redo.push(changes);
        self.selection.clear();
        self.refresh();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(changes) = self.redo.pop() else {
            return false;
        };
        for c in &changes {
            self.apply(c);
        }
        self.undo.push(changes);
        self.selection.clear();
        self.refresh();
        true
    }

    /// Bygg byen på nytt fra dataene og sjekk reglene.
    fn refresh(&mut self) {
        (self.scene, self.built) = scene(&self.src, &self.layout, &self.contents, &self.crypt);
        let touched = self.touched();
        self.breaches = rules::check(&self.scene, &self.layout, self.approved.as_ref(), &touched);
        self.breaches.extend(crypt::breaches(&self.built, &self.crypt));
        // Godkjent: også merkene som er endret i krypten.
        if let Some(ok) = &self.approved {
            for r in &touched {
                if let Some(Thing::Mark(m)) = self.get(r) {
                    if let Some(model) = crypt::model_of(&m).filter(|m| !ok.contains(*m)) {
                        self.breaches.push(Breach { r: r.clone(), why: format!("{model} er ikke godkjent ennå") });
                    }
                }
            }
        }
        self.breaches.sort_by(|a, b| a.r.cmp(&b.r).then(a.why.cmp(&b.why)));
        self.breaches.dedup();
        let alive: HashSet<Ref> = self.refs().into_iter().collect();
        self.selection.retain(|r| alive.contains(r));
    }

    /// Hvor huset med innholdet står (det første med den modellen).
    pub(super) fn house_pose(&self, house: &str) -> Affine3A {
        self.layout.items.iter().find(|it| it.building && it.model.as_deref() == Some(house)).and_then(|it| it.at).map(|p| p.affine()).unwrap_or(Affine3A::IDENTITY)
    }

    /// Hvor tingen står i verden.
    pub fn pose_of(&self, r: &Ref) -> Option<Affine3A> {
        match self.get(r)? {
            Thing::Item(it) => {
                let local = it.at.map(|p| p.affine()).unwrap_or(Affine3A::IDENTITY);
                Some(match r {
                    Ref::Inside { house, .. } => self.house_pose(house) * local,
                    _ => local,
                })
            }
            Thing::Npc(n) => Some(Affine3A::from_rotation_translation(Quat::from_rotation_y(n.yaw), n.pos)),
            Thing::Mark(m) => Some(Affine3A::from_rotation_translation(Quat::from_rotation_y(m.yaw), Vec3::from(m.pos?) + self.crypt_origin(r)?)),
            Thing::Light(l) => Some(Affine3A::from_translation(Vec3::from(l.pos) + self.crypt_origin(r)?)),
            Thing::Lamp(l) => match r {
                Ref::ItemLight { item, .. } => Some(Affine3A::from_translation(self.lamp_pos(item, &l)?)),
                _ => None,
            },
        }
    }

    /// Boksen til tingen i dens eget rom (midte og halve mål): modellens
    /// mål, de usynlige veggene, eller en figur på 1,8 m.
    pub(super) fn local_box(&self, r: &Ref) -> Option<(Vec3, Vec3)> {
        match self.get(r)? {
            Thing::Item(it) => {
                if let Some((lo, hi)) = it.model.as_deref().and_then(|m| self.scene.bounds(m)) {
                    return Some(((lo + hi) * 0.5, (hi - lo) * 0.5));
                }
                let mut lo = Vec3::splat(f32::MAX);
                let mut hi = Vec3::splat(f32::MIN);
                for s in &it.solids {
                    if let SolidDef::Cuboid { center, size, yaw } = s {
                        let q = Quat::from_rotation_y(*yaw);
                        for k in 0..8 {
                            let c = Vec3::new(if k & 1 == 0 { -0.5 } else { 0.5 }, if k & 2 == 0 { -0.5 } else { 0.5 }, if k & 4 == 0 { -0.5 } else { 0.5 }) * *size;
                            let p = *center + q * c;
                            lo = lo.min(p);
                            hi = hi.max(p);
                        }
                    }
                }
                (lo.x <= hi.x).then(|| ((lo + hi) * 0.5, (hi - lo) * 0.5))
            }
            Thing::Npc(_) => Some((Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.3, 0.9, 0.3))),
            Thing::Mark(m) => Some(match crypt::model_of(&m).and_then(|name| self.scene.bounds(name)) {
                Some((lo, hi)) => ((lo + hi) * 0.5, (hi - lo) * 0.5),
                None => crypt::default_box(&m),
            }),
            Thing::Light(_) | Thing::Lamp(_) => Some((Vec3::ZERO, Vec3::splat(0.15))),
        }
    }

    /// Boksen til en ting i verden: posen, og midten og halve mål i tingens
    /// rom.
    pub fn box_of(&self, r: &Ref) -> Option<(Affine3A, Vec3, Vec3)> {
        let (c, h) = self.local_box(r)?;
        Some((self.pose_of(r)?, c, h))
    }

    /// Det valgte, med boksene, til nettlaget.
    pub fn selected(&self) -> Vec<Selected> {
        self.selection
            .iter()
            .filter_map(|r| {
                let pose = self.pose_of(r)?;
                let (center, half) = self.local_box(r)?;
                Some(Selected { r: r.clone(), label: self.label(r), pose, center, half })
            })
            .collect()
    }

    pub fn label(&self, r: &Ref) -> String {
        match self.get(r) {
            Some(Thing::Item(it)) => it.model.unwrap_or_else(|| "usynlig vegg".into()),
            Some(Thing::Npc(n)) => n.name,
            Some(Thing::Mark(m)) => match (crypt::model_of(&m), &m.label, &m.id) {
                (Some(model), _, _) if m.kind != "prop" => model.to_string(),
                (_, Some(l), _) => format!("{} ({l})", m.kind),
                (_, _, Some(id)) => format!("{} ({id})", m.kind),
                _ => m.kind.clone(),
            },
            Some(Thing::Light(_)) => "lys".into(),
            Some(Thing::Lamp(l)) => if l.always { "ild" } else { "lykt" }.into(),
            None => String::new(),
        }
    }

    /// Klikk: tingen strålen treffer først, og klikker du igjen på samme sted,
    /// den neste under pekeren (en stor boks, som kirka, skjuler ellers det
    /// som står inni den). Med `add` legges den til i valget (Shift), ellers
    /// blir den valget alene.
    pub fn pick(&mut self, origin: Vec3, dir: Vec3, add: bool) -> Option<Ref> {
        let hits = self.hits(origin, dir);
        // Samme sted som sist: det neste under pekeren (kirka, så gjerdet
        // inni boksen dens, så det bak), rundt og rundt.
        let again = self.last_ray.is_some_and(|(o, d)| o.distance(origin) < 0.05 && d.dot(dir) > 0.9999);
        let k = match (again, self.single()) {
            (true, Some(cur)) if !add => hits.iter().position(|r| r == cur).map(|k| (k + 1) % hits.len()).unwrap_or(0),
            _ => 0,
        };
        let hit = hits.get(k).cloned();
        self.last_ray = Some((origin, dir));
        self.under = hits;
        // En ting i en gruppe velges med hele gruppen.
        match (&hit, add) {
            (Some(r), true) => self.toggle(r),
            (Some(r), false) => self.selection = self.members(r),
            (None, false) => self.selection.clear(),
            (None, true) => {}
        }
        hit
    }

    /// Tab: det neste under pekeren fra siste klikk.
    pub fn cycle(&mut self) -> Option<Ref> {
        let (o, d) = self.last_ray?;
        self.pick(o, d, false)
    }

    /// Det som lå under pekeren ved siste klikk, nærmest først.
    pub fn under(&self) -> &[Ref] {
        &self.under
    }

    /// Det nærmeste strålen treffer, uten å endre valget.
    pub fn hit(&self, origin: Vec3, dir: Vec3) -> Option<Ref> {
        self.hits(origin, dir).into_iter().next()
    }

    /// Alt strålen treffer, nærmest først. Står du inni noe (et hus), er
    /// det tingene inne som treffes, ikke huset rundt deg.
    pub(super) fn hits(&self, origin: Vec3, dir: Vec3) -> Vec<Ref> {
        let mut hits: Vec<(f32, Ref)> = vec![];
        for r in self.refs() {
            let (Some(pose), Some((c, h))) = (self.pose_of(&r), self.local_box(&r)) else {
                continue;
            };
            let inv = pose.inverse();
            let (o, d) = (inv.transform_point3(origin) - c, inv.transform_vector3(dir));
            if o.abs().cmple(h).all() {
                continue;
            }
            let (mut t0, mut t1) = (0.0f32, f32::MAX);
            for k in 0..3 {
                if d[k].abs() < 1e-9 {
                    if o[k].abs() > h[k] {
                        t1 = -1.0;
                    }
                    continue;
                }
                let (a, b) = ((-h[k] - o[k]) / d[k], (h[k] - o[k]) / d[k]);
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
            if t0 <= t1 {
                hits.push((t0, r));
            }
        }
        // Et lys kommer etter tingen det står i (lyktestolpen, lysestaken).
        let at: HashMap<Ref, f32> = hits.iter().map(|(t, r)| (r.clone(), *t)).collect();
        for (t, r) in &mut hits {
            if let Ref::ItemLight { item, .. } = r {
                *t = t.max(at.get(item.as_ref()).map_or(*t, |p| p + 1e-3));
            }
        }
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        hits.into_iter().map(|(_, r)| r).collect()
    }

    /// Endringen av én ting når posen i verden går fra `t` til `f(t)`.
    pub(super) fn pose_change(&self, r: &Ref, f: impl Fn(Affine3A) -> Affine3A) -> Option<Change> {
        let (before, world) = (self.get(r)?, self.pose_of(r)?);
        let new = f(world);
        let after = match before.clone() {
            Thing::Item(mut it) => {
                let parent = match r {
                    Ref::Inside { house, .. } => self.house_pose(house),
                    _ => Affine3A::IDENTITY,
                };
                let scale = it.at.map(|p| p.scale).unwrap_or(Vec3::ONE);
                let mut p = Pose::of(parent.inverse() * new);
                p.scale = scale;
                it.at = Some(p);
                Thing::Item(it)
            }
            // En innbygger tar med seg ruta si.
            Thing::Npc(mut n) => {
                let p = Pose::of(new);
                let d = p.pos - n.pos;
                n.pos = p.pos;
                n.yaw = p.yaw();
                for (q, _) in &mut n.route {
                    *q += d;
                }
                Thing::Npc(n)
            }
            // I krypten i nivåets rom; en fiende tar med seg runden sin.
            Thing::Mark(mut m) => {
                let o = self.crypt_origin(r)?;
                let p = Pose::of(new);
                let old = Vec3::from(m.pos?);
                let local = p.pos - o;
                let d = local - old;
                m.pos = Some(local.to_array());
                m.yaw = p.yaw();
                for q in &mut m.patrol {
                    *q = (Vec3::from(*q) + d).to_array();
                }
                Thing::Mark(m)
            }
            Thing::Light(mut l) => {
                l.pos = (Vec3::from(new.translation) - self.crypt_origin(r)?).to_array();
                Thing::Light(l)
            }
            Thing::Lamp(l) => match r {
                Ref::ItemLight { item, .. } => Thing::Lamp(self.lamp_moved(item, l, Vec3::from(new.translation))?),
                _ => return None,
            },
        };
        (after != before).then(|| Change::Set { r: r.clone(), before, after })
    }

    /// Flytt det valgte `delta` meter (i verden).
    pub fn translate(&mut self, delta: Vec3) {
        let changes = self.moving().iter().filter_map(|r| self.pose_change(r, |t| Affine3A::from_translation(delta) * t)).collect();
        self.commit(changes);
    }

    /// Drei det valgte `yaw` radianer om midten av det valgte: én ting om
    /// sitt eget origo, flere (et gjerde i biter) samlet, som én ting
    /// (eieren 12. oktober).
    pub fn rotate(&mut self, yaw: f32) {
        let moving = self.moving();
        let at: Vec<Vec3> = moving.iter().filter_map(|r| self.pose_of(r)).map(|t| Vec3::from(t.translation)).collect();
        if at.is_empty() {
            return;
        }
        let p = at.iter().sum::<Vec3>() / at.len() as f32;
        let turn = |t: Affine3A| Affine3A::from_translation(p) * Affine3A::from_rotation_y(yaw) * Affine3A::from_translation(-p) * t;
        let changes = moving.iter().filter_map(|r| self.pose_change(r, turn)).collect();
        self.commit(changes);
    }

    /// Sett posen til én ting (håndtakene i nettlaget).
    pub fn set_pose(&mut self, r: &Ref, pose: Affine3A) {
        let changes = self.pose_change(r, |_| pose).into_iter().collect();
        self.commit(changes);
    }

    /// Sett det valgte rett ned på det som er under (bakken, et gulv eller
    /// en flate), uten å røre x og z.
    pub fn drop_to_surface(&mut self) {
        let mut changes = vec![];
        for r in &self.selection {
            let (Some(foot), Some(t)) = (self.foot(r), self.pose_of(r)) else {
                continue;
            };
            let o = Vec3::from(t.translation);
            let delta = Vec3::Y * (self.surface_under(o.x, o.z, foot + 0.3, Some(r)) - foot);
            changes.extend(self.pose_change(r, |t| Affine3A::from_translation(delta) * t));
        }
        self.commit(changes);
    }

    /// Den laveste høyden tingen når (bunnen av boksen i verden).
    pub(super) fn foot(&self, r: &Ref) -> Option<f32> {
        let t = self.pose_of(r)?;
        let (c, h) = self.local_box(r)?;
        Some(rules::world_box(c - h, c + h, &t).0.y)
    }

    /// Høyden på det noe kan stå på i (x, z) under `below`: bakken, et gulv,
    /// eller toppen av en kollider (et bord, en kasse), men ikke tingens egne.
    pub fn surface_under(&self, x: f32, z: f32, below: f32, own: Option<&Ref>) -> f32 {
        let mut best = self.scene.stand_height(x, z, below);
        for (shape, o) in self.scene.colliders.iter().zip(&self.scene.solid_origins) {
            if own.is_some() && o.as_ref() == own {
                continue;
            }
            if let Shape::Cuboid { center, size, yaw } = shape {
                let d = Quat::from_rotation_y(-*yaw) * (Vec3::new(x, center.y, z) - *center);
                let top = center.y + size.y * 0.5;
                if d.x.abs() <= size.x * 0.5 && d.z.abs() <= size.z * 0.5 && top <= below && top > best {
                    best = top;
                }
            }
        }
        best
    }

    /// Kopier det valgte (Ctrl+D): kopiene står der originalene står og blir
    /// valget, så de kan flyttes.
    pub fn duplicate(&mut self) {
        let mut changes = vec![];
        let mut new_sel = vec![];
        let mut ends: HashMap<String, usize> = HashMap::new();
        // En kopi av en gruppe blir en ny gruppe.
        let mut groups: HashMap<String, String> = HashMap::new();
        for r in self.selection.clone() {
            let Some(mut thing) = self.get(&r) else { continue };
            if let Thing::Item(Item { group: Some(g), .. }) = &mut thing {
                let taken: Vec<String> = groups.values().cloned().collect();
                *g = groups.entry(g.clone()).or_insert_with(|| self.fresh_group(g, &taken)).clone();
            }
            // Lysene kopieres inn i tingens egne.
            let list = match &r {
                Ref::ItemLight { item, .. } => Self::list_of(&Ref::ItemLight { item: item.clone(), model: false, i: 0 }),
                _ => Self::list_of(&r),
            };
            let len = *ends.entry(list.clone()).or_insert_with(|| match &r {
                Ref::Town(_) => self.layout.items.len(),
                Ref::Inside { house, .. } => self.contents.get(house).map(|v| v.len()).unwrap_or(0),
                Ref::Npc(_) => self.layout.npcs.len(),
                Ref::Crypt { level, .. } => self.crypt.get(level).map(|c| c.marks.len()).unwrap_or(0),
                Ref::CryptLight { level, .. } => self.crypt.get(level).map(|c| c.lights.len()).unwrap_or(0),
                Ref::ItemLight { item, .. } => self.item_of(item).map(|it| it.lights.len()).unwrap_or(0),
            });
            let at = match &r {
                Ref::Town(_) => Ref::Town(len),
                Ref::Inside { house, .. } => Ref::Inside { house: house.clone(), i: len },
                Ref::Npc(_) => Ref::Npc(len),
                Ref::Crypt { level, .. } => Ref::Crypt { level: *level, i: len },
                Ref::CryptLight { level, .. } => Ref::CryptLight { level: *level, i: len },
                // En kopi av et lys blir tingens eget.
                Ref::ItemLight { item, .. } => Ref::ItemLight { item: item.clone(), model: false, i: len },
            };
            *ends.get_mut(&list).expect("lista") += 1;
            changes.push(Change::Insert { r: at.clone(), thing });
            new_sel.push(at);
        }
        self.commit(changes);
        self.selection = new_sel;
    }

    /// Slett det valgte (Delete).
    pub fn delete(&mut self) {
        let mut sel = self.moving();
        // Bakfra, så numrene foran ikke flytter seg.
        sel.sort();
        sel.dedup();
        let mut changes = vec![];
        for r in sel.into_iter().rev() {
            if let Some(thing) = self.get(&r) {
                changes.push(Change::Remove { r, thing });
            }
        }
        self.selection.clear();
        self.commit(changes);
    }

    /// Huset med innhold som punktet står inni (innenfor veggene), om noe.
    pub fn house_at(&self, p: Vec3) -> Option<String> {
        self.layout.items.iter().find_map(|it| {
            let m = it.model.as_deref()?;
            if !it.building || !self.contents.contains_key(m) {
                return None;
            }
            let (lo, hi) = self.scene.bounds(m)?;
            let local = it.at?.affine().inverse().transform_point3(p);
            let e = rules::EAVES;
            (local.x > lo.x + e && local.x < hi.x - e && local.z > lo.z + e && local.z < hi.z - e && local.y > lo.y - 0.5 && local.y < hi.y).then(|| m.to_string())
        })
    }

    /// Sett ut en ny ting med posen `pose` i verden: inne i et hus lagres den
    /// i husets rom (og følger huset), nede i krypten som et merke i nivået
    /// (`None` for det som ikke kan stå der), ellers i byen. Den blir valget.
    pub fn place(&mut self, model: &str, pose: Affine3A) -> Option<Ref> {
        let at = Vec3::from(pose.translation);
        if at.y < crate::domain::world::crypt::TOP {
            return self.place_in_crypt(model, pose);
        }
        let (r, local) = match self.house_at(at) {
            Some(house) => {
                let i = self.contents.get(&house).map(|v| v.len()).unwrap_or(0);
                let local = self.house_pose(&house).inverse() * pose;
                (Ref::Inside { house, i }, local)
            }
            None => (Ref::Town(self.layout.items.len()), pose),
        };
        let mounted = matches!(r, Ref::Inside { .. });
        let item = Item { model: Some(model.to_string()), at: Some(Pose::of(local)), mounted, solids: self.default_solids(model), ..Default::default() };
        self.commit(vec![Change::Insert { r: r.clone(), thing: Thing::Item(item) }]);
        self.selection = vec![r.clone()];
        Some(r)
    }

    /// Dataene til lagring og til å bygge spillet på nytt.
    pub fn data(&self) -> (&Layout, &HashMap<String, Vec<Item>>) {
        (&self.layout, &self.contents)
    }
}
