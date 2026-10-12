//! `Editor`: tilstanden i redigeringen. Dataene (byen og husenes innhold),
//! valget, angre og gjøre om, og byen bygget fra dataene (`town_scene`) til
//! reglene og til å finne tingene. Alt eieren gjør er en endring av
//! dataene; byen bygges på nytt etter hver.

use super::rules::{self, Breach};
use super::{town_scene, Sources};
use crate::domain::world::layout::{Item, Layout, NpcDef, Pose, Ref, SolidDef};
use crate::domain::world::{terrain, Builder, Shape};
use glam::{Affine3A, Quat, Vec3};
use std::collections::{HashMap, HashSet};

/// En ting i dataene: en ting i byen eller et hus, eller en innbygger.
#[derive(Clone, Debug, PartialEq)]
enum Thing {
    Item(Item),
    Npc(NpcDef),
}

/// Én endring, som kan snus.
#[derive(Clone, Debug)]
enum Change {
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

/// Hva en linje blir: en rad av en modell (gjerdebitene etter hverandre,
/// `spacing` meter mellom midtene, eller modellens lengde om 0), eller en
/// usynlig vegg så høy som `height`.
#[derive(Clone, Debug, PartialEq)]
pub enum LineKind {
    Row { model: String, spacing: f32 },
    Wall { height: f32 },
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

/// Tykkelsen på en usynlig vegg.
const WALL_THICK: f32 = 0.3;

pub struct Editor {
    src: Sources,
    pub layout: Layout,
    pub contents: HashMap<String, Vec<Item>>,
    /// Modellene eieren har godkjent (`approvals.json`); uten: ingen sjekk.
    approved: Option<HashSet<String>>,
    pub selection: Vec<Ref>,
    undo: Vec<Vec<Change>>,
    redo: Vec<Vec<Change>>,
    /// Dataene slik de var da redigeringen begynte, til å se hva som er endret.
    original: HashMap<String, usize>,
    scene: Builder,
    breaches: Vec<Breach>,
    /// En gruppe endringer som angres som én (dra med håndtak): `Some(false)`
    /// før første endring, `Some(true)` når den er i gang.
    group: Option<bool>,
    /// Det siste klikket: strålen og alt den traff, nærmest først, så et nytt
    /// klikk på samme sted (eller Tab) velger det neste under pekeren.
    last_ray: Option<(Vec3, Vec3)>,
    under: Vec<Ref>,
}

/// En nøkkel for innholdet i en ting, til å telle like ting.
fn key(list: &str, t: &Thing) -> String {
    format!("{list}|{t:?}")
}

impl Editor {
    pub fn new(src: Sources, layout: Layout, contents: HashMap<String, Vec<Item>>, approved: Option<HashSet<String>>) -> Self {
        let scene = town_scene(&src, &layout, &contents);
        let mut e = Self { src, layout, contents, approved, selection: vec![], undo: vec![], redo: vec![], original: HashMap::new(), scene, breaches: vec![], group: None, last_ray: None, under: vec![] };
        e.original = e.census();
        e.refresh();
        e
    }

    /// Sammenlign med andre data enn dem redigeringen begynte med (dataene i
    /// fila, når spillet er bygget på nytt fra redigeringen med P): det som
    /// er endret siden da, må fortsatt være godkjent.
    pub fn set_baseline(&mut self, layout: Layout, contents: HashMap<String, Vec<Item>>) {
        let base = Editor { layout, contents, ..Self::empty(self.src.clone()) };
        self.original = base.census();
        self.refresh();
    }

    fn empty(src: Sources) -> Self {
        let scene = Builder::new(&[]);
        Self { src, layout: Layout::default(), contents: HashMap::new(), approved: None, selection: vec![], undo: vec![], redo: vec![], original: HashMap::new(), scene, breaches: vec![], group: None, last_ray: None, under: vec![] }
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

    fn list_of(r: &Ref) -> String {
        match r {
            Ref::Town(_) => "town".into(),
            Ref::Inside { house, .. } => format!("inside/{house}"),
            Ref::Npc(_) => "npcs".into(),
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

    fn get(&self, r: &Ref) -> Option<Thing> {
        match r {
            Ref::Town(i) => self.layout.items.get(*i).cloned().map(Thing::Item),
            Ref::Inside { house, i } => self.contents.get(house).and_then(|v| v.get(*i)).cloned().map(Thing::Item),
            Ref::Npc(i) => self.layout.npcs.get(*i).cloned().map(Thing::Npc),
        }
    }

    fn apply(&mut self, c: &Change) {
        match c {
            Change::Set { r, after, .. } => match (r, after) {
                (Ref::Town(i), Thing::Item(it)) => self.layout.items[*i] = it.clone(),
                (Ref::Inside { house, i }, Thing::Item(it)) => self.contents.get_mut(house).expect("huset")[*i] = it.clone(),
                (Ref::Npc(i), Thing::Npc(n)) => self.layout.npcs[*i] = n.clone(),
                _ => {}
            },
            Change::Insert { r, thing } => match (r, thing) {
                (Ref::Town(i), Thing::Item(it)) => self.layout.items.insert(*i, it.clone()),
                (Ref::Inside { house, i }, Thing::Item(it)) => self.contents.entry(house.clone()).or_default().insert(*i, it.clone()),
                (Ref::Npc(i), Thing::Npc(n)) => self.layout.npcs.insert(*i, n.clone()),
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
            },
        }
    }

    /// Gjør endringene som én handling (ett steg å angre).
    fn commit(&mut self, changes: Vec<Change>) {
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
        let Some(changes) = self.undo.pop() else { return false };
        for c in changes.iter().rev() {
            self.apply(&c.inverse());
        }
        self.redo.push(changes);
        self.selection.clear();
        self.refresh();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(changes) = self.redo.pop() else { return false };
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
        self.scene = town_scene(&self.src, &self.layout, &self.contents);
        self.breaches = rules::check(&self.scene, &self.layout, self.approved.as_ref(), &self.touched());
        self.selection.retain(|r| match r {
            Ref::Town(i) => *i < self.layout.items.len(),
            Ref::Inside { house, i } => self.contents.get(house).is_some_and(|v| *i < v.len()),
            Ref::Npc(i) => *i < self.layout.npcs.len(),
        });
    }

    /// Hvor huset med innholdet står (det første med den modellen).
    fn house_pose(&self, house: &str) -> Affine3A {
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
        }
    }

    /// Boksen til tingen i dens eget rom (midte og halve mål): modellens
    /// mål, de usynlige veggene, eller en figur på 1,8 m.
    fn local_box(&self, r: &Ref) -> Option<(Vec3, Vec3)> {
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
        let k = match (again, self.selection.as_slice()) {
            (true, [cur]) if !add => hits.iter().position(|r| r == cur).map(|k| (k + 1) % hits.len()).unwrap_or(0),
            _ => 0,
        };
        let hit = hits.get(k).cloned();
        self.last_ray = Some((origin, dir));
        self.under = hits;
        match (&hit, add) {
            (Some(r), true) => {
                if let Some(k) = self.selection.iter().position(|s| s == r) {
                    self.selection.remove(k);
                } else {
                    self.selection.push(r.clone());
                }
            }
            (Some(r), false) => self.selection = vec![r.clone()],
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

    /// Alt strålen treffer, nærmest først. Står du inni noe (et hus), er
    /// det tingene inne som treffes, ikke huset rundt deg.
    fn hits(&self, origin: Vec3, dir: Vec3) -> Vec<Ref> {
        let mut hits: Vec<(f32, Ref)> = vec![];
        for r in self.refs() {
            let (Some(pose), Some((c, h))) = (self.pose_of(&r), self.local_box(&r)) else { continue };
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
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        hits.into_iter().map(|(_, r)| r).collect()
    }

    /// Endringen av én ting når posen i verden går fra `t` til `f(t)`.
    fn pose_change(&self, r: &Ref, f: impl Fn(Affine3A) -> Affine3A) -> Option<Change> {
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
            Thing::Npc(mut n) => {
                let p = Pose::of(new);
                n.pos = p.pos;
                n.yaw = p.yaw();
                Thing::Npc(n)
            }
        };
        (after != before).then(|| Change::Set { r: r.clone(), before, after })
    }

    /// Flytt det valgte `delta` meter (i verden).
    pub fn translate(&mut self, delta: Vec3) {
        let changes = self.selection.iter().filter_map(|r| self.pose_change(r, |t| Affine3A::from_translation(delta) * t)).collect();
        self.commit(changes);
    }

    /// Drei det valgte `yaw` radianer om sitt eget origo.
    pub fn rotate(&mut self, yaw: f32) {
        let turn = |t: Affine3A| {
            let p = Vec3::from(t.translation);
            Affine3A::from_translation(p) * Affine3A::from_rotation_y(yaw) * Affine3A::from_translation(-p) * t
        };
        let changes = self.selection.iter().filter_map(|r| self.pose_change(r, turn)).collect();
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
            let (Some(foot), Some(t)) = (self.foot(r), self.pose_of(r)) else { continue };
            let o = Vec3::from(t.translation);
            let delta = Vec3::Y * (self.surface_under(o.x, o.z, foot + 0.3, Some(r)) - foot);
            changes.extend(self.pose_change(r, |t| Affine3A::from_translation(delta) * t));
        }
        self.commit(changes);
    }

    /// Den laveste høyden tingen når (bunnen av boksen i verden).
    fn foot(&self, r: &Ref) -> Option<f32> {
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
        for r in self.selection.clone() {
            let Some(thing) = self.get(&r) else { continue };
            let list = Self::list_of(&r);
            let len = *ends.entry(list).or_insert_with(|| match &r {
                Ref::Town(_) => self.layout.items.len(),
                Ref::Inside { house, .. } => self.contents.get(house).map(|v| v.len()).unwrap_or(0),
                Ref::Npc(_) => self.layout.npcs.len(),
            });
            let at = match &r {
                Ref::Town(_) => Ref::Town(len),
                Ref::Inside { house, .. } => Ref::Inside { house: house.clone(), i: len },
                Ref::Npc(_) => Ref::Npc(len),
            };
            *ends.get_mut(&Self::list_of(&r)).expect("lista") += 1;
            changes.push(Change::Insert { r: at.clone(), thing });
            new_sel.push(at);
        }
        self.commit(changes);
        self.selection = new_sel;
    }

    /// Slett det valgte (Delete).
    pub fn delete(&mut self) {
        let mut sel = self.selection.clone();
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
    /// i husets rom (og følger huset), ellers i byen. Den blir valget.
    pub fn place(&mut self, model: &str, pose: Affine3A) -> Ref {
        let at = Vec3::from(pose.translation);
        let (r, local) = match self.house_at(at) {
            Some(house) => {
                let i = self.contents.get(&house).map(|v| v.len()).unwrap_or(0);
                let local = self.house_pose(&house).inverse() * pose;
                (Ref::Inside { house, i }, local)
            }
            None => (Ref::Town(self.layout.items.len()), pose),
        };
        let mounted = matches!(r, Ref::Inside { .. });
        let item = Item { model: Some(model.to_string()), at: Some(Pose::of(local)), mounted, ..Default::default() };
        self.commit(vec![Change::Insert { r: r.clone(), thing: Thing::Item(item) }]);
        self.selection = vec![r.clone()];
        r
    }

    /// En linje gjennom punktene på bakken: en rad av en modell langs hvert
    /// stykke, eller en usynlig vegg. Følger terrenget; blir valget, og angres
    /// som én handling.
    pub fn line(&mut self, points: &[Vec3], kind: &LineKind) -> Vec<Ref> {
        let mut items = vec![];
        for w in points.windows(2) {
            let (a, b) = (w[0], w[1]);
            let d = glam::Vec2::new(b.x - a.x, b.z - a.z);
            let len = d.length();
            if len < 0.05 {
                continue;
            }
            // Lokal x langs stykket.
            let yaw = (-d.y).atan2(d.x);
            match kind {
                LineKind::Row { model, spacing } => {
                    let step = if *spacing > 0.0 { *spacing } else { self.scene.bounds(model).map(|(lo, hi)| hi.x - lo.x).filter(|s| *s > 0.05).unwrap_or(1.0) };
                    let n = (len / step).round().max(1.0) as usize;
                    for k in 0..n {
                        let f = (k as f32 + 0.5) / n as f32;
                        let (x, z) = (a.x + d.x * f, a.z + d.y * f);
                        let p = Vec3::new(x, terrain::height(x, z), z);
                        items.push(Item { model: Some(model.clone()), at: Some(Pose { pos: p, rot: Quat::from_rotation_y(yaw), scale: Vec3::ONE }), ..Default::default() });
                    }
                }
                LineKind::Wall { height } => {
                    let (x, z) = (a.x + d.x * 0.5, a.z + d.y * 0.5);
                    // Foten i det laveste punktet langs veggen, så den når bakken.
                    let low = (0..=8).map(|k| terrain::height(a.x + d.x * k as f32 / 8.0, a.z + d.y * k as f32 / 8.0)).fold(f32::MAX, f32::min);
                    let p = Vec3::new(x, low, z);
                    let solid = SolidDef::Cuboid { center: Vec3::new(0.0, height * 0.5, 0.0), size: Vec3::new(len, *height, WALL_THICK), yaw: 0.0 };
                    items.push(Item { model: None, at: Some(Pose { pos: p, rot: Quat::from_rotation_y(yaw), scale: Vec3::ONE }), solids: vec![solid], ..Default::default() });
                }
            }
        }
        let start = self.layout.items.len();
        let refs: Vec<Ref> = (start..start + items.len()).map(Ref::Town).collect();
        let changes = items.into_iter().enumerate().map(|(k, it)| Change::Insert { r: Ref::Town(start + k), thing: Thing::Item(it) }).collect();
        self.commit(changes);
        self.selection = refs.clone();
        refs
    }

    /// Endre høyden på det valgte (usynlige vegger).
    pub fn set_wall_height(&mut self, height: f32) {
        let mut changes = vec![];
        for r in self.selection.clone() {
            let Some(Thing::Item(before)) = self.get(&r) else { continue };
            let mut it = before.clone();
            for s in &mut it.solids {
                if let SolidDef::Cuboid { center, size, .. } = s {
                    size.y = height;
                    center.y = height * 0.5;
                }
            }
            if it != before {
                changes.push(Change::Set { r, before: Thing::Item(before), after: Thing::Item(it) });
            }
        }
        self.commit(changes);
    }

    /// Dataene til lagring og til å bygge spillet på nytt.
    pub fn data(&self) -> (&Layout, &HashMap<String, Vec<Item>>) {
        (&self.layout, &self.contents)
    }
}
