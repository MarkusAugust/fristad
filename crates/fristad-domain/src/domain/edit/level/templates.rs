//! Malene og malens innhold i nivåredigeringen (`docs/nivaaredigering.md`
//! del 26): et utvalg som mal (`LevelEditor::template`) og satt ut igjen
//! dreid og speilet (`LevelOp::Stamp`), og innhold fra en modulmal som
//! løsnes i ett nivå (`LevelOp::Detach`, `DocModule.hidden`).

use super::{mm, mm3, Change, Entry, LevelEditor};
use crate::domain::world::crypt::compose::{modules, Compiler, Severity, CELL};
use crate::domain::world::crypt::doc::{DocLight, DocLink, DocModule, DocOverride, LevelDoc, LevelId, ModuleDef, Thing};
use glam::Vec3;

impl LevelEditor {
    /// Innholdet `<modul>/<id>` fra en modulmal som står i nivået nå:
    /// modulen (plassen i lista) og malens id.
    pub(super) fn template_thing(&self, id: &str) -> Option<(usize, String)> {
        let (module, local) = id.split_once('/')?;
        let k = self.module_index(module)?;
        let m = &self.doc.modules[k];
        let def = self.kit.module(&m.module)?;
        (def.things.iter().any(|t| t.id == local) && !m.hidden.iter().any(|h| h == local)).then(|| (k, local.to_string()))
    }

    /// Endringene som løsner malens innhold `id`: modulen tar det bort, og
    /// dokumentet får det på samme sted, med samme id. `None`: ikke malens.
    pub(super) fn detach(&self, id: &str, at: usize) -> Result<Option<(Vec<Change>, Thing)>, String> {
        let Some((k, local)) = self.template_thing(id) else { return Ok(None) };
        let flat = self.flat();
        let t = flat.things.iter().find(|t| t.id == id).ok_or_else(|| format!("«{id}» finnes ikke i modulen"))?;
        if t.anchor.starts_with("kant:") {
            return Err(format!("«{id}» står i en kant av modulen og kan ikke flyttes; velg åpningen i stedet"));
        }
        let mark = self.file.marks.iter().find(|m| m.id.as_deref() == Some(id)).ok_or_else(|| format!("«{id}» er ikke satt ut"))?;
        let thing = Thing {
            id: id.to_string(),
            anchor: "fri".into(),
            pos: mark.pos,
            yaw: mm(mark.yaw.to_degrees().rem_euclid(360.0)),
            patrol: mark.patrol.clone(),
            top: mark.top,
            bottom: mark.bottom,
            levels: mark.levels.clone(),
            ..t.clone()
        };
        let m = &self.doc.modules[k];
        let mut after = m.clone();
        after.hidden.push(local);
        Ok(Some((vec![Change::Set { index: k, before: Entry::Module(m.clone()), after: Entry::Module(after) }, Change::Insert { index: at, entry: Entry::Thing(thing.clone()) }], thing)))
    }

    /// Utvalget `ids` som mal (`LevelOp::Stamp`): modulene, innholdet og
    /// lysene som er valgt eller står i dem, og koblingene mellom dem, flyttet
    /// så den første modulen står i (0, 0, 0).
    pub fn template(&self, ids: &[String], name: &str) -> LevelDoc {
        let mods: Vec<&DocModule> = self.doc.modules.iter().filter(|m| ids.contains(&m.id)).collect();
        let mut things = self.things_in(&mods);
        things.extend(ids.iter().filter_map(|id| self.thing_index(id)));
        things.sort();
        things.dedup();
        let mut lights = self.lights_in(&mods);
        lights.extend(ids.iter().filter_map(|id| self.doc.lights.iter().position(|l| &l.id == id)));
        lights.sort();
        lights.dedup();
        let origin =
            mods.first().map(|m| m.at).or_else(|| things.first().and_then(|&k| self.doc.things[k].pos).map(|p| [(p[0] / CELL).round() as i32, p[1].round() as i32, (p[2] / CELL).round() as i32]));
        let o = origin.unwrap_or([0, 0, 0]);
        let d = Vec3::new(o[0] as f32 * CELL, o[1] as f32, o[2] as f32 * CELL);
        let shift = |p: [f32; 3]| mm3(Vec3::from(p) - d);
        let mut doc = LevelDoc { format: 1, level: LevelId::Name("mal".into()), name: name.into(), status: "mal".into(), theme: self.doc.theme.clone(), ..Default::default() };
        doc.modules = mods.iter().map(|m| DocModule { at: [m.at[0] - o[0], m.at[1] - o[1], m.at[2] - o[2]], room: None, ..(*m).clone() }).collect();
        doc.things = things
            .iter()
            .map(|&k| {
                let t = &self.doc.things[k];
                Thing {
                    pos: t.pos.map(shift),
                    patrol: t.patrol.iter().map(|p| shift(*p)).collect(),
                    top: t.top.map(shift),
                    bottom: t.bottom.map(shift),
                    levels: t.levels.iter().map(|l| mm(l - d.y)).collect(),
                    ..t.clone()
                }
            })
            .collect();
        doc.lights = lights.iter().map(|&k| DocLight { pos: shift(self.doc.lights[k].pos), ..self.doc.lights[k].clone() }).collect();
        // Koblingene der begge ender er med i malen.
        let mut inside: Vec<String> = doc.modules.iter().map(|m| m.id.clone()).collect();
        inside.extend(doc.modules.iter().flat_map(|m| m.overrides.iter().filter_map(|v| v.id.clone())));
        inside.extend(doc.things.iter().map(|t| t.id.clone()));
        let within = |s: &str| inside.iter().any(|i| i == s) || doc.modules.iter().any(|m| s.starts_with(&format!("{}/", m.id)));
        doc.links = self.doc.links.iter().filter(|l| within(&l.from) && within(&l.to)).cloned().collect();
        doc
    }

    /// Endringene som setter ut malen `t` (se `LevelOp::Stamp`), og de nye
    /// id-ene.
    pub(super) fn stamp(&self, t: &LevelDoc, at: [i32; 3], rot: u8, mirror: bool) -> (Vec<Change>, Vec<String>) {
        let flip = |x: f32, z: f32| {
            let mut p = (if mirror { -x } else { x }, z);
            for _ in 0..rot {
                p = (-p.1, p.0);
            }
            p
        };
        let base = Vec3::new(at[0] as f32 * CELL, at[1] as f32, at[2] as f32 * CELL);
        let point = |p: [f32; 3]| {
            let (x, z) = flip(p[0], p[2]);
            mm3(Vec3::new(x, p[1], z) + base)
        };
        // Nye id-er for alt som har en.
        let mut taken: Vec<String> = vec![];
        let mut map: Vec<(String, String)> = vec![];
        let mut fresh = |e: &LevelEditor, old: &str| {
            let base = old.rsplit('/').next().unwrap_or(old).trim_end_matches(|c: char| c.is_ascii_digit());
            let base = if base.is_empty() { "m" } else { base };
            let id = (1..).map(|n| format!("{base}{n}")).find(|s| !taken.contains(s) && !e.used(s)).unwrap();
            taken.push(id.clone());
            map.push((old.to_string(), id.clone()));
            id
        };
        for m in &t.modules {
            fresh(self, &m.id);
            for v in m.overrides.iter().filter_map(|v| v.id.as_ref()) {
                fresh(self, v);
            }
        }
        for th in t.things.iter().filter(|th| !th.id.contains('/')) {
            fresh(self, &th.id);
        }
        for l in &t.lights {
            fresh(self, &l.id);
        }
        let rename = |s: &str| -> String {
            if let Some((_, n)) = map.iter().find(|(o, _)| o == s) {
                return n.clone();
            }
            match s.split_once('/') {
                Some((m, rest)) => map.iter().find(|(o, _)| o == m).map(|(_, n)| format!("{n}/{rest}")).unwrap_or_else(|| s.to_string()),
                None => s.to_string(),
            }
        };
        let mut out = vec![];
        let mut made = vec![];
        for (k, m) in t.modules.iter().filter(|m| self.kit.module(&m.module).is_some()).enumerate() {
            let (x, z) = flip(m.at[0] as f32, m.at[2] as f32);
            let r = m.rot % 4;
            let (r2, mirror2) = if mirror { ((rot + 4 - r) % 4, !m.mirror) } else { ((rot + r) % 4, m.mirror) };
            let id = rename(&m.id);
            let overrides = m.overrides.iter().map(|v| DocOverride { id: v.id.as_deref().map(rename), ..v.clone() }).collect();
            let m2 = DocModule {
                id: id.clone(),
                at: [at[0] + x.round() as i32, at[1] + m.at[1], at[2] + z.round() as i32],
                rot: r2,
                mirror: mirror2,
                overrides,
                room: m.room.as_deref().map(rename),
                ..m.clone()
            };
            out.push(Change::Insert { index: self.doc.modules.len() + k, entry: Entry::Module(m2) });
            made.push(id);
        }
        for (k, th) in t.things.iter().enumerate() {
            let id = rename(&th.id);
            let yaw = (if mirror { -th.yaw } else { th.yaw }) - rot as f32 * 90.0;
            let t2 = Thing {
                id: id.clone(),
                anchor: "fri".into(),
                pos: th.pos.map(point),
                yaw: mm(yaw.rem_euclid(360.0)),
                patrol: th.patrol.iter().map(|p| point(*p)).collect(),
                top: th.top.map(point),
                bottom: th.bottom.map(point),
                levels: th.levels.iter().map(|l| mm(l + base.y)).collect(),
                ..th.clone()
            };
            out.push(Change::Insert { index: self.doc.things.len() + k, entry: Entry::Thing(t2) });
            made.push(id);
        }
        for (k, l) in t.lights.iter().enumerate() {
            let id = rename(&l.id);
            out.push(Change::Insert { index: self.doc.lights.len() + k, entry: Entry::Light(DocLight { id: id.clone(), anchor: "fri".into(), pos: point(l.pos), ..l.clone() }) });
            made.push(id);
        }
        for (k, l) in t.links.iter().enumerate() {
            out.push(Change::Insert { index: self.doc.links.len() + k, entry: Entry::Link(DocLink { from: rename(&l.from), to: rename(&l.to), kind: l.kind.clone() }) });
        }
        (out, made)
    }
}

impl LevelEditor {
    /// Utvalget `ids` som en ny modul eieren kan sette ut som de andre
    /// (`docs/nivaaredigering.md` del 27; Ctrl+G er en mal, dette en modul):
    /// modulene utvidet til cellene sine (`modules::flatten`), med kantene der
    /// de møttes, rommene, innholdet, lysene og koblingene, flyttet så den
    /// første modulen står i (0, 0, 0). Åpningene er de ytre sidene som ikke
    /// er lukket, som i alle modulene. `slug` er filnavnet
    /// (`assets/levels/moduler/<tema>/<slug>.json`).
    pub fn as_module(&self, ids: &[String], name: &str, slug: &str) -> Result<ModuleDef, String> {
        if slug.is_empty() || !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(format!("Navnet på fila «{slug}» kan bare ha små bokstaver a–z, tall og _"));
        }
        let tpl = self.template(ids, name);
        let first = tpl.modules.first().ok_or("Velg minst én modul først")?;
        let theme = if self.doc.theme.is_empty() { self.kit.module(&first.module).map(|m| m.theme.clone()).unwrap_or_else(|| "grav".into()) } else { self.doc.theme.clone() };
        // I samme mappe som modulene den er laget av (settet, `grav/`).
        let set = self.kit.module(&first.module).and_then(|m| m.id.split_once('/')).map(|(s, _)| s.to_string()).unwrap_or_else(|| theme.clone());
        let id = format!("{set}/{slug}");
        if self.kit.module(&id).is_some_and(|m| !m.about.starts_with(OWN)) {
            return Err(format!("Modulen «{id}» finnes i byggesettet; velg et annet navn"));
        }
        let (flat, issues) = modules::flatten(&tpl, &self.kit);
        if let Some(i) = issues.iter().find(|i| i.severity == Severity::Error) {
            return Err(i.msg.clone());
        }
        let names: Vec<String> = tpl.modules.iter().map(|m| m.name.clone().or_else(|| self.kit.module(&m.module).map(|d| d.name.clone())).unwrap_or_else(|| m.id.clone())).collect();
        Ok(ModuleDef {
            id,
            name: name.to_string(),
            theme,
            kind: "ferdig".into(),
            about: format!("{OWN} {}.", names.join(", ")),
            rooms: flat.rooms,
            cells: flat.cells,
            openings: vec![],
            walls: vec![],
            edges: flat.edges,
            things: flat.things,
            links: flat.links,
            lights: flat.lights,
        })
    }

    /// Ta en modul inn i byggesettet (eierens egen, lagret eller husket i
    /// nettleseren): den står i paletten og kan settes ut som de andre. En
    /// med samme id byttes ut.
    pub fn add_module(&mut self, def: ModuleDef) {
        self.kit.modules.retain(|m| m.id != def.id);
        self.kit.modules.push(def);
        self.compiler = Compiler::default();
        self.rebuild();
    }
}

/// Begynnelsen på beskrivelsen av eierens egne moduler (`as_module`).
pub const OWN: &str = "Eierens modul, satt sammen av";
