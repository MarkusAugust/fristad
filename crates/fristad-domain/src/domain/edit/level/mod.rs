//! Nivåredigeringen (`?nivaa`, `docs/nivaaredigering.md` del 12): eieren
//! setter ut moduler på rutenettet, dreier, speiler, flytter og sletter
//! dem, velger slaget for hver åpning, setter ut innhold (fiender, kister,
//! lapper, spaker …), kobler spaker til porter og angrer. Alt er endringer
//! av nivådokumentet (`LevelDoc`), som `Editor` er det for byen: hver
//! handling blir en liste `Change` som kan snus, og dokumentet kompileres
//! på nytt etter hver (`Compiler`, som bare regner cellene der noe er
//! endret). Reglene er kompilatorens og de samme som testene (`rules`).
//! Ingen nettleser, filer eller GitHub her: dokumentet inn, dokumentet ut.

pub mod fixes;
pub mod map;
pub mod rules;
mod templates;

use crate::domain::world::crypt::compose::{self, modules, Compiler, Issue, Report, Severity, CELL};
use crate::domain::world::crypt::doc::{DocLight, DocLink, DocModule, DocOverride, LevelDoc, LevelId, Thing};
use crate::domain::world::crypt::kit::KitDef;
use crate::domain::world::crypt::level::LevelFile;
use crate::domain::world::crypt::stone::StoneRef;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Hodet på dokumentet: det som gjelder hele nivået.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Head {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub level: LevelId,
    /// `utkast` eller `klar`.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stone: Option<StoneRef>,
}

/// En ting i dokumentet en endring gjelder.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Module(DocModule),
    Thing(Thing),
    Link(DocLink),
    Light(DocLight),
    Head(Head),
}

/// Én endring, som kan snus: en ting på plass `index` i sin liste.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Set { index: usize, before: Entry, after: Entry },
    Insert { index: usize, entry: Entry },
    Remove { index: usize, entry: Entry },
}

impl Change {
    fn inverse(&self) -> Change {
        match self.clone() {
            Change::Set { index, before, after } => Change::Set { index, before: after, after: before },
            Change::Insert { index, entry } => Change::Remove { index, entry },
            Change::Remove { index, entry } => Change::Insert { index, entry },
        }
    }

    fn apply(&self, doc: &mut LevelDoc) {
        fn put<T: Clone>(list: &mut Vec<T>, c: &Change, v: T) {
            match c {
                Change::Set { index, .. } => {
                    if let Some(x) = list.get_mut(*index) {
                        *x = v;
                    }
                }
                Change::Insert { index, .. } => list.insert((*index).min(list.len()), v),
                Change::Remove { index, .. } => {
                    if *index < list.len() {
                        list.remove(*index);
                    }
                }
            }
        }
        let e = match self {
            Change::Set { after, .. } => after,
            Change::Insert { entry, .. } | Change::Remove { entry, .. } => entry,
        };
        match e.clone() {
            Entry::Module(m) => put(&mut doc.modules, self, m),
            Entry::Thing(t) => put(&mut doc.things, self, t),
            Entry::Link(l) => put(&mut doc.links, self, l),
            Entry::Light(l) => put(&mut doc.lights, self, l),
            Entry::Head(h) => {
                doc.name = h.name;
                doc.level = h.level;
                doc.status = h.status;
                doc.theme = h.theme;
                doc.seed = h.seed;
                doc.stone = h.stone;
            }
        }
    }
}

/// Det eieren gjør, som json fra nettlaget (`{"op": "add_module", …}`).
/// Rutene er (i, y, j) (cellene på 4 × 4 m med gulvet i y meter), stedene
/// for innhold er i nivåets rom (meter).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum LevelOp {
    /// Sett ut en modul (`grav/kapell`) med midtcella i `at`.
    AddModule {
        module: String,
        at: [i32; 3],
        #[serde(default)]
        rot: u8,
        #[serde(default)]
        mirror: bool,
    },
    /// Flytt moduler og innhold (rutene; innholdet 4 m per rute og 1 m per trinn).
    Move {
        ids: Vec<String>,
        by: [i32; 3],
    },
    /// Drei det valgte 90° med klokka (`back`: mot) om den første modulen.
    Turn {
        ids: Vec<String>,
        #[serde(default)]
        back: bool,
    },
    /// Speil det valgte langs x om den første modulen.
    Mirror {
        ids: Vec<String>,
    },
    Delete {
        ids: Vec<String>,
    },
    /// Lim inn kopier (nye id-er), flyttet `by`.
    Paste {
        #[serde(default)]
        modules: Vec<DocModule>,
        #[serde(default)]
        things: Vec<Thing>,
        by: [i32; 3],
    },
    /// Bytt ut en modul (samme id) med det egenskapspanelet har endret.
    SetModule {
        module: DocModule,
    },
    /// Slaget for en åpning i en modul (`auto` eller tomt fjerner valget).
    Opening {
        module: String,
        opening: String,
        kind: String,
        #[serde(default)]
        id: Option<String>,
    },
    /// Sett ut innhold (`mob`, `chest`, `lever` …) i nivåets rom.
    AddThing {
        kind: String,
        at: [f32; 3],
        #[serde(default)]
        yaw: f32,
        #[serde(default)]
        label: Option<String>,
    },
    MoveThing {
        id: String,
        to: [f32; 3],
    },
    SetThing {
        thing: Thing,
    },
    Link {
        from: String,
        to: String,
        kind: String,
    },
    Unlink {
        from: String,
        to: String,
    },
    AddLight {
        at: [f32; 3],
        #[serde(default)]
        color: Option<[f32; 3]>,
        #[serde(default)]
        range: Option<f32>,
    },
    SetLight {
        light: DocLight,
    },
    SetHead {
        head: Head,
    },
    /// Løsne innholdet `<modul>/<id>` fra modulmalen: det blir eierens i
    /// dokumentet med samme id og sted (malen tar det bort i dette nivået,
    /// `DocModule.hidden`), så det kan flyttes og endres. `move_thing`,
    /// `set_thing` og `delete` på malens innhold gjør dette selv.
    Detach {
        id: String,
    },
    /// Malens innhold tilbake slik malen har det (det endrede eller fjernede
    /// glemmes).
    Reattach {
        module: String,
        thing: String,
    },
    /// Sett ut en mal eieren har lagret (`LevelEditor::template`,
    /// `assets/levels/maler/<navn>.json`) med den første modulen i `at`,
    /// speilet langs x og så dreid `rot` kvart omdreininger med klokka. Alt
    /// får nye id-er, og koblingene i malen følger med.
    Stamp {
        template: Box<LevelDoc>,
        at: [i32; 3],
        #[serde(default)]
        rot: u8,
        #[serde(default)]
        mirror: bool,
    },
}

/// Slagene en kobling kan ha, og hva de forbinder.
pub const LINK_KINDS: [&str; 6] = ["opens", "key", "arena", "exit", "for", "from"];

/// Redigeringen av ett nivådokument.
pub struct LevelEditor {
    doc: LevelDoc,
    kit: KitDef,
    compiler: Compiler,
    file: LevelFile,
    report: Report,
    issues: Vec<Issue>,
    /// Rettingen for hvert brudd (`fixes::suggest`).
    fixes: Vec<Option<fixes::Fix>>,
    undo: Vec<Vec<Change>>,
    redo: Vec<Vec<Change>>,
    /// Endringer siden sist lagret.
    pub unsaved: usize,
}

impl LevelEditor {
    pub fn new(doc: LevelDoc, kit: KitDef) -> Self {
        let mut e =
            LevelEditor { doc, kit, compiler: Compiler::default(), file: LevelFile::default(), report: Report::default(), issues: vec![], fixes: vec![], undo: vec![], redo: vec![], unsaved: 0 };
        e.rebuild();
        e
    }

    /// Et nytt, tomt nivå: kapellet med døra opp og startstedet, så det
    /// alltid er noe å begynne fra.
    pub fn blank(name: &str, level: u32, theme: &str, kit: &KitDef) -> LevelDoc {
        let mut doc = LevelDoc { format: 1, level: LevelId::N(level), name: name.into(), status: "utkast".into(), seed: level as u64 * 7919 + 1, theme: theme.into(), ..Default::default() };
        let first = [format!("{theme}/kapell"), format!("{theme}/rom_2x2")].into_iter().find(|m| kit.module(m).is_some());
        if let Some(m) = first {
            doc.modules.push(DocModule { id: "kapell".into(), module: m, ..Default::default() });
        }
        doc
    }

    pub fn doc(&self) -> &LevelDoc {
        &self.doc
    }

    pub fn kit(&self) -> &KitDef {
        &self.kit
    }

    /// Nivåfila slik den er kompilert nå.
    pub fn file(&self) -> &LevelFile {
        &self.file
    }

    pub fn report(&self) -> &Report {
        &self.report
    }

    /// Alle bruddene: kompilatorens og reglene over resultatet.
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Rettingen for bruddet nummer `k` i `issues`, om den er entydig.
    pub fn fix(&self, k: usize) -> Option<&fixes::Fix> {
        self.fixes.get(k).and_then(Option::as_ref)
    }

    /// Kan merkes «klar»: ingen feil.
    pub fn clean(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Lagret: telleren nullstilles.
    pub fn saved(&mut self) {
        self.unsaved = 0;
    }

    /// Kompiler på nytt og regn ut reglene.
    fn rebuild(&mut self) {
        let (file, report) = self.compiler.compile(&self.doc, &self.kit);
        let mut issues = report.issues.clone();
        issues.extend(compose::check::overlaps(&self.doc, &self.kit));
        issues.extend(compose::check::tight(&self.doc, &self.kit, &file));
        issues.extend(rules::check(&self.doc, &self.kit, &file, &report));
        // Det samme bruddet fra to regler meldes én gang.
        let mut seen = std::collections::HashSet::new();
        issues.retain(|i| seen.insert((i.rule, i.msg.clone())));
        issues.sort_by_key(|i| (i.severity != Severity::Error, i.rule));
        self.fixes = fixes::suggest(&self.doc, &self.kit, &file, &issues);
        self.file = file;
        self.report = report;
        self.issues = issues;
    }

    /// Gjør endringene som én handling (ett angresteg).
    fn commit(&mut self, changes: Vec<Change>) -> bool {
        if changes.is_empty() {
            return false;
        }
        for c in &changes {
            c.apply(&mut self.doc);
        }
        self.undo.push(changes);
        self.redo.clear();
        self.unsaved += 1;
        self.rebuild();
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some(changes) = self.undo.pop() else { return false };
        for c in changes.iter().rev() {
            c.inverse().apply(&mut self.doc);
        }
        self.redo.push(changes);
        self.unsaved += 1;
        self.rebuild();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(changes) = self.redo.pop() else { return false };
        for c in &changes {
            c.apply(&mut self.doc);
        }
        self.undo.push(changes);
        self.unsaved += 1;
        self.rebuild();
        true
    }

    /// Bytt hele dokumentet (åpne et annet nivå): historikken tømmes.
    pub fn open(&mut self, doc: LevelDoc) {
        self.doc = doc;
        self.undo.clear();
        self.redo.clear();
        self.unsaved = 0;
        self.rebuild();
    }

    /// En id som ikke er brukt av en modul, et innhold eller et lys.
    pub fn fresh_id(&self, base: &str) -> String {
        let base = base.rsplit('/').next().unwrap_or(base).replace(['/', ' '], "_");
        (1..).map(|n| format!("{base}{n}")).find(|s| !self.used(s)).unwrap()
    }

    /// Om en id er brukt av en modul, et innhold, et lys eller en port.
    fn used(&self, s: &str) -> bool {
        self.doc.modules.iter().any(|m| m.id == s || m.overrides.iter().any(|o| o.id.as_deref() == Some(s))) || self.doc.things.iter().any(|t| t.id == s) || self.doc.lights.iter().any(|l| l.id == s)
    }

    /// Gjør en handling. Gir id-ene til det som ble laget (tomt ellers), eller
    /// hvorfor det ikke gikk.
    pub fn apply(&mut self, op: LevelOp) -> Result<Vec<String>, String> {
        let (changes, made) = self.plan(op)?;
        if !self.commit(changes) && made.is_empty() {
            return Ok(vec![]);
        }
        Ok(made)
    }

    fn module_index(&self, id: &str) -> Option<usize> {
        self.doc.modules.iter().position(|m| m.id == id)
    }

    fn thing_index(&self, id: &str) -> Option<usize> {
        self.doc.things.iter().position(|t| t.id == id)
    }

    /// Rutene (i, y, j) modulen dekker i nivået.
    pub fn footprint(&self, m: &DocModule) -> Vec<[i32; 3]> {
        footprint(m, &self.kit)
    }

    /// Innholdet med fritt anker som står i en av rutene til modulene `ids`
    /// (det følger med når de flyttes, dreies, speiles eller slettes).
    fn things_in(&self, ids: &[&DocModule]) -> Vec<usize> {
        let cells: Vec<[i32; 3]> = ids.iter().flat_map(|m| self.footprint(m)).collect();
        self.doc
            .things
            .iter()
            .enumerate()
            .filter(|(_, t)| free(&t.anchor))
            .filter(|(_, t)| {
                let p = Vec3::from(t.pos.unwrap_or_default());
                let (i, j) = ((p.x / CELL).round() as i32, (p.z / CELL).round() as i32);
                cells.iter().any(|c| c[0] == i && c[2] == j && p.y > c[1] as f32 - 1.5 && p.y < c[1] as f32 + 4.5)
            })
            .map(|(k, _)| k)
            .collect()
    }

    /// Lysene med fritt anker i modulene `ids`.
    fn lights_in(&self, ids: &[&DocModule]) -> Vec<usize> {
        let cells: Vec<[i32; 3]> = ids.iter().flat_map(|m| self.footprint(m)).collect();
        self.doc
            .lights
            .iter()
            .enumerate()
            .filter(|(_, l)| free(&l.anchor))
            .filter(|(_, l)| {
                let p = Vec3::from(l.pos);
                let (i, j) = ((p.x / CELL).round() as i32, (p.z / CELL).round() as i32);
                cells.iter().any(|c| c[0] == i && c[2] == j)
            })
            .map(|(k, _)| k)
            .collect()
    }

    /// Hva en handling endrer, uten å gjøre den.
    fn plan(&self, op: LevelOp) -> Result<(Vec<Change>, Vec<String>), String> {
        let mut out = vec![];
        let mut made = vec![];
        match op {
            LevelOp::AddModule { module, at, rot, mirror } => {
                if self.kit.module(&module).is_none() {
                    return Err(format!("Ukjent modul «{module}»"));
                }
                let id = self.fresh_id(module.rsplit('/').next().unwrap_or("m"));
                let m = DocModule { id: id.clone(), module, at, rot: rot % 4, mirror, ..Default::default() };
                out.push(Change::Insert { index: self.doc.modules.len(), entry: Entry::Module(m) });
                made.push(id);
            }
            LevelOp::Move { ids, by } => {
                let mods: Vec<&DocModule> = self.doc.modules.iter().filter(|m| ids.contains(&m.id)).collect();
                let mut things: Vec<usize> = self.things_in(&mods);
                things.extend(ids.iter().filter_map(|id| self.thing_index(id)).filter(|k| free(&self.doc.things[*k].anchor)));
                things.sort();
                things.dedup();
                let d = Vec3::new(by[0] as f32 * CELL, by[1] as f32, by[2] as f32 * CELL);
                for (k, m) in self.doc.modules.iter().enumerate().filter(|(_, m)| ids.contains(&m.id)) {
                    let after = DocModule { at: [m.at[0] + by[0], m.at[1] + by[1], m.at[2] + by[2]], ..m.clone() };
                    out.push(Change::Set { index: k, before: Entry::Module(m.clone()), after: Entry::Module(after) });
                }
                for k in things {
                    let t = &self.doc.things[k];
                    let after = Thing {
                        pos: t.pos.map(|p| mm3(Vec3::from(p) + d)),
                        patrol: t.patrol.iter().map(|p| mm3(Vec3::from(*p) + d)).collect(),
                        top: t.top.map(|p| mm3(Vec3::from(p) + d)),
                        bottom: t.bottom.map(|p| mm3(Vec3::from(p) + d)),
                        levels: t.levels.iter().map(|l| mm(l + d.y)).collect(),
                        ..t.clone()
                    };
                    out.push(Change::Set { index: k, before: Entry::Thing(t.clone()), after: Entry::Thing(after) });
                }
                for k in self.lights_in(&mods).into_iter().chain(ids.iter().filter_map(|id| self.doc.lights.iter().position(|l| &l.id == id))).collect::<std::collections::BTreeSet<_>>() {
                    let l = &self.doc.lights[k];
                    out.push(Change::Set { index: k, before: Entry::Light(l.clone()), after: Entry::Light(DocLight { pos: mm3(Vec3::from(l.pos) + d), ..l.clone() }) });
                }
            }
            LevelOp::Turn { ids, back } => {
                out = self.reshape(
                    &ids,
                    |p| if back { Vec3::new(p.z, p.y, -p.x) } else { Vec3::new(-p.z, p.y, p.x) },
                    |rot, mirror| ((rot + if back { 3 } else { 1 }) % 4, mirror),
                    |yaw| yaw + if back { 90.0 } else { -90.0 },
                )?
            }
            LevelOp::Mirror { ids } => out = self.reshape(&ids, |p| Vec3::new(-p.x, p.y, p.z), |rot, mirror| ((4 - rot % 4) % 4, !mirror), |yaw| -yaw)?,
            LevelOp::Delete { ids } => {
                let mods: Vec<&DocModule> = self.doc.modules.iter().filter(|m| ids.contains(&m.id)).collect();
                let mut things = self.things_in(&mods);
                things.extend(ids.iter().filter_map(|id| self.thing_index(id)));
                things.sort();
                things.dedup();
                let gone: Vec<String> = things.iter().map(|&k| self.doc.things[k].id.clone()).chain(ids.iter().cloned()).collect();
                let mut lights = self.lights_in(&mods);
                lights.extend(ids.iter().filter_map(|id| self.doc.lights.iter().position(|l| &l.id == id)));
                lights.sort();
                lights.dedup();
                // Bakfra, så indeksene står når det snus.
                let links: Vec<usize> = self
                    .doc
                    .links
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| gone.iter().any(|g| *g == l.from || *g == l.to || l.from.starts_with(&format!("{g}/")) || l.to.starts_with(&format!("{g}/"))))
                    .map(|(k, _)| k)
                    .collect();
                for k in links.into_iter().rev() {
                    out.push(Change::Remove { index: k, entry: Entry::Link(self.doc.links[k].clone()) });
                }
                for k in things.into_iter().rev() {
                    out.push(Change::Remove { index: k, entry: Entry::Thing(self.doc.things[k].clone()) });
                }
                for k in lights.into_iter().rev() {
                    out.push(Change::Remove { index: k, entry: Entry::Light(self.doc.lights[k].clone()) });
                }
                // Malens innhold i moduler som står igjen: modulen tar det bort.
                let mut hide: std::collections::BTreeMap<usize, Vec<String>> = Default::default();
                for id in &ids {
                    if let Some((k, local)) = self.template_thing(id).filter(|(k, _)| !ids.contains(&self.doc.modules[*k].id)) {
                        hide.entry(k).or_default().push(local);
                    }
                }
                for (k, locals) in hide {
                    let m = &self.doc.modules[k];
                    let mut after = m.clone();
                    after.hidden.extend(locals);
                    out.push(Change::Set { index: k, before: Entry::Module(m.clone()), after: Entry::Module(after) });
                }
                for k in (0..self.doc.modules.len()).rev().filter(|k| ids.contains(&self.doc.modules[*k].id)) {
                    out.push(Change::Remove { index: k, entry: Entry::Module(self.doc.modules[k].clone()) });
                }
            }
            LevelOp::Paste { modules, things, by } => {
                let d = Vec3::new(by[0] as f32 * CELL, by[1] as f32, by[2] as f32 * CELL);
                let mut taken: Vec<String> = vec![];
                let mut fresh = |e: &LevelEditor, base: &str| {
                    let base = base.rsplit('/').next().unwrap_or(base).trim_end_matches(|c: char| c.is_ascii_digit());
                    let base = if base.is_empty() { "m" } else { base };
                    let id = (1..).map(|n| format!("{base}{n}")).find(|s| !taken.contains(s) && !e.used(s)).unwrap();
                    taken.push(id.clone());
                    id
                };
                let mut index = self.doc.modules.len();
                for m in modules {
                    if self.kit.module(&m.module).is_none() {
                        continue;
                    }
                    let id = fresh(self, &m.id);
                    let m2 = DocModule {
                        id: id.clone(),
                        at: [m.at[0] + by[0], m.at[1] + by[1], m.at[2] + by[2]],
                        overrides: m.overrides.iter().map(|o| DocOverride { id: None, ..o.clone() }).collect(),
                        room: None,
                        ..m
                    };
                    out.push(Change::Insert { index, entry: Entry::Module(m2) });
                    index += 1;
                    made.push(id);
                }
                let mut index = self.doc.things.len();
                for t in things.into_iter().filter(|t| free(&t.anchor)) {
                    let id = fresh(self, &t.id);
                    let t2 = Thing {
                        id: id.clone(),
                        anchor: "fri".into(),
                        pos: t.pos.map(|p| mm3(Vec3::from(p) + d)),
                        patrol: t.patrol.iter().map(|p| mm3(Vec3::from(*p) + d)).collect(),
                        top: t.top.map(|p| mm3(Vec3::from(p) + d)),
                        bottom: t.bottom.map(|p| mm3(Vec3::from(p) + d)),
                        levels: t.levels.iter().map(|l| mm(l + d.y)).collect(),
                        ..t
                    };
                    out.push(Change::Insert { index, entry: Entry::Thing(t2) });
                    index += 1;
                    made.push(id);
                }
            }
            LevelOp::SetModule { module } => {
                let k = self.module_index(&module.id).ok_or_else(|| format!("Ingen modul «{}»", module.id))?;
                if self.kit.module(&module.module).is_none() {
                    return Err(format!("Ukjent modul «{}»", module.module));
                }
                if self.doc.modules[k] != module {
                    out.push(Change::Set { index: k, before: Entry::Module(self.doc.modules[k].clone()), after: Entry::Module(module) });
                }
            }
            LevelOp::Opening { module, opening, kind, id } => {
                let k = self.module_index(&module).ok_or_else(|| format!("Ingen modul «{module}»"))?;
                if !kind.is_empty() && !modules::OPENING_KINDS.contains(&kind.as_str()) {
                    return Err(format!("Ukjent slag «{kind}»"));
                }
                let m = &self.doc.modules[k];
                let mut after = m.clone();
                after.overrides.retain(|o| o.opening != opening);
                if !kind.is_empty() && kind != "auto" {
                    // Porten, snarveien og den falske veggen får en id koblingene kan peke på.
                    let id = id.filter(|s| !s.is_empty()).or_else(|| {
                        matches!(kind.as_str(), "port" | "snarvei" | "falsk").then(|| {
                            self.fresh_id(match kind.as_str() {
                                "port" => "port",
                                "snarvei" => "snarvei",
                                _ => "falsk",
                            })
                        })
                    });
                    after.overrides.push(DocOverride { opening, kind, id });
                }
                if after != *m {
                    out.push(Change::Set { index: k, before: Entry::Module(m.clone()), after: Entry::Module(after) });
                }
            }
            LevelOp::AddThing { kind, at, yaw, label } => {
                let base = match kind.as_str() {
                    "mob" | "elite" | "boss" => label.clone().unwrap_or_else(|| kind.clone()).replace(' ', "_"),
                    k => k.to_string(),
                };
                let id = self.fresh_id(&base);
                let mut t = Thing { id: id.clone(), kind: kind.clone(), anchor: "fri".into(), pos: Some(mm3(Vec3::from(at))), yaw: mm(yaw), label, ..Default::default() };
                if kind == "lore" && t.text.is_none() {
                    t.label = t.label.or(Some("En lapp".into()));
                    t.text = Some(String::new());
                }
                out.push(Change::Insert { index: self.doc.things.len(), entry: Entry::Thing(t) });
                made.push(id);
            }
            LevelOp::MoveThing { id, to } => {
                let (k, t) = match self.thing_index(&id) {
                    Some(k) => (k, self.doc.things[k].clone()),
                    None => {
                        let at = self.doc.things.len();
                        let (changes, t) = self.detach(&id, at)?.ok_or_else(|| format!("Innholdet «{id}» finnes ikke"))?;
                        out.extend(changes);
                        (at, t)
                    }
                };
                let t = &t;
                if !free(&t.anchor) {
                    return Err(format!("«{id}» er festet ({})", t.anchor));
                }
                let d = Vec3::from(to) - Vec3::from(t.pos.unwrap_or_default());
                let after = Thing {
                    pos: Some(mm3(Vec3::from(to))),
                    patrol: t.patrol.iter().map(|p| mm3(Vec3::from(*p) + d)).collect(),
                    top: t.top.map(|p| mm3(Vec3::from(p) + d)),
                    bottom: t.bottom.map(|p| mm3(Vec3::from(p) + d)),
                    levels: t.levels.iter().map(|l| mm(l + d.y)).collect(),
                    ..t.clone()
                };
                if after != *t {
                    out.push(Change::Set { index: k, before: Entry::Thing(t.clone()), after: Entry::Thing(after) });
                }
            }
            LevelOp::SetThing { thing } => match self.thing_index(&thing.id) {
                Some(k) => {
                    if self.doc.things[k] != thing {
                        out.push(Change::Set { index: k, before: Entry::Thing(self.doc.things[k].clone()), after: Entry::Thing(thing) });
                    }
                }
                None => {
                    // Malens innhold: løsnes og endres i samme steg.
                    let at = self.doc.things.len();
                    let (changes, t) = self.detach(&thing.id, at)?.ok_or_else(|| format!("Innholdet «{}» finnes ikke", thing.id))?;
                    out.extend(changes);
                    if t != thing {
                        out.push(Change::Set { index: at, before: Entry::Thing(t), after: Entry::Thing(thing) });
                    }
                }
            },
            LevelOp::Stamp { template, at, rot, mirror } => {
                let (changes, ids) = self.stamp(&template, at, rot % 4, mirror);
                out.extend(changes);
                made.extend(ids);
            }
            LevelOp::Detach { id } => {
                if self.thing_index(&id).is_none() {
                    let (changes, _) = self.detach(&id, self.doc.things.len())?.ok_or_else(|| format!("«{id}» kommer ikke fra en modul"))?;
                    out.extend(changes);
                }
            }
            LevelOp::Reattach { module, thing } => {
                let k = self.module_index(&module).ok_or_else(|| format!("Ingen modul «{module}»"))?;
                let m = &self.doc.modules[k];
                let full = format!("{module}/{thing}");
                if let Some(t) = self.thing_index(&full) {
                    out.push(Change::Remove { index: t, entry: Entry::Thing(self.doc.things[t].clone()) });
                }
                if m.hidden.contains(&thing) {
                    let mut after = m.clone();
                    after.hidden.retain(|h| *h != thing);
                    out.push(Change::Set { index: k, before: Entry::Module(m.clone()), after: Entry::Module(after) });
                }
            }
            LevelOp::Link { from, to, kind } => {
                if !LINK_KINDS.contains(&kind.as_str()) {
                    return Err(format!("Ukjent kobling «{kind}»"));
                }
                if from == to {
                    return Err("En ting kan ikke kobles til seg selv".into());
                }
                if !self.doc.links.iter().any(|l| l.from == from && l.to == to) {
                    out.push(Change::Insert { index: self.doc.links.len(), entry: Entry::Link(DocLink { from, to, kind }) });
                }
            }
            LevelOp::Unlink { from, to } => {
                for k in (0..self.doc.links.len()).rev().filter(|&k| self.doc.links[k].from == from && self.doc.links[k].to == to) {
                    out.push(Change::Remove { index: k, entry: Entry::Link(self.doc.links[k].clone()) });
                }
            }
            LevelOp::AddLight { at, color, range } => {
                let id = self.fresh_id("lys");
                let l =
                    DocLight { id: id.clone(), anchor: "fri".into(), pos: mm3(Vec3::from(at)), color: color.unwrap_or([1.0, 0.62, 0.3]), range: range.unwrap_or(8.0), flicker: true, intensity: None };
                out.push(Change::Insert { index: self.doc.lights.len(), entry: Entry::Light(l) });
                made.push(id);
            }
            LevelOp::SetLight { light } => {
                let k = self.doc.lights.iter().position(|l| l.id == light.id).ok_or_else(|| format!("Lyset «{}» hører til en modul", light.id))?;
                if self.doc.lights[k] != light {
                    out.push(Change::Set { index: k, before: Entry::Light(self.doc.lights[k].clone()), after: Entry::Light(light) });
                }
            }
            LevelOp::SetHead { head } => {
                if head.status == "klar" && self.doc.status != "klar" && !self.clean() {
                    return Err(format!("{} feil må rettes før nivået kan merkes klart", self.issues.iter().filter(|i| i.severity == Severity::Error).count()));
                }
                let before = self.head();
                if before != head {
                    out.push(Change::Set { index: 0, before: Entry::Head(before), after: Entry::Head(head) });
                }
            }
        }
        Ok((out, made))
    }

    pub fn head(&self) -> Head {
        let d = &self.doc;
        Head { name: d.name.clone(), level: d.level.clone(), status: d.status.clone(), theme: d.theme.clone(), seed: d.seed, stone: d.stone.clone() }
    }

    /// Drei eller speil modulene og innholdet `ids` om den første modulen
    /// (eller det første innholdet): `turn` flytter et punkt i forhold til
    /// midten, `shape` gir den nye (rot, speiling), `yaw` den nye dreiningen
    /// (grader) for innholdet.
    fn reshape(&self, ids: &[String], turn: impl Fn(Vec3) -> Vec3, shape: impl Fn(u8, bool) -> (u8, bool), yaw: impl Fn(f32) -> f32) -> Result<Vec<Change>, String> {
        let mods: Vec<&DocModule> = self.doc.modules.iter().filter(|m| ids.contains(&m.id)).collect();
        let pivot = match mods.first() {
            Some(m) => m.at,
            None => {
                let t = ids.iter().find_map(|id| self.thing_index(id)).map(|k| &self.doc.things[k]).ok_or("Ingenting valgt")?;
                let p = Vec3::from(t.pos.unwrap_or_default());
                [(p.x / CELL).round() as i32, 0, (p.z / CELL).round() as i32]
            }
        };
        let centre = Vec3::new(pivot[0] as f32 * CELL, 0.0, pivot[2] as f32 * CELL);
        let mut out = vec![];
        for (k, m) in self.doc.modules.iter().enumerate().filter(|(_, m)| ids.contains(&m.id)) {
            let d = turn(Vec3::new((m.at[0] - pivot[0]) as f32, 0.0, (m.at[2] - pivot[2]) as f32));
            let (rot, mirror) = shape(m.rot % 4, m.mirror);
            let after = DocModule { at: [pivot[0] + d.x.round() as i32, m.at[1], pivot[2] + d.z.round() as i32], rot, mirror, ..m.clone() };
            out.push(Change::Set { index: k, before: Entry::Module(m.clone()), after: Entry::Module(after) });
        }
        let mut things = self.things_in(&mods);
        things.extend(ids.iter().filter_map(|id| self.thing_index(id)).filter(|k| free(&self.doc.things[*k].anchor)));
        things.sort();
        things.dedup();
        let point = |p: [f32; 3]| mm3(turn(Vec3::from(p) - centre) + centre);
        for k in things {
            let t = &self.doc.things[k];
            let after = Thing {
                pos: t.pos.map(point),
                patrol: t.patrol.iter().map(|p| point(*p)).collect(),
                top: t.top.map(point),
                bottom: t.bottom.map(point),
                yaw: mm(yaw(t.yaw).rem_euclid(360.0)),
                ..t.clone()
            };
            out.push(Change::Set { index: k, before: Entry::Thing(t.clone()), after: Entry::Thing(after) });
        }
        for k in self.lights_in(&mods) {
            let l = &self.doc.lights[k];
            out.push(Change::Set { index: k, before: Entry::Light(l.clone()), after: Entry::Light(DocLight { pos: point(l.pos), ..l.clone() }) });
        }
        Ok(out)
    }

    /// Det kompilerte nivået med modulene utvidet (til kartet).
    pub fn flat(&self) -> LevelDoc {
        modules::flatten(&self.doc, &self.kit).0
    }
}

fn free(anchor: &str) -> bool {
    anchor.is_empty() || anchor == "fri"
}

fn mm(v: f32) -> f32 {
    compose::mm(v)
}

fn mm3(v: Vec3) -> [f32; 3] {
    compose::mm3(v)
}

/// Rutene (i, y, j) en modul dekker i nivået (delcellene til de sammensatte bitene med).
pub fn footprint(m: &DocModule, kit: &KitDef) -> Vec<[i32; 3]> {
    let doc = LevelDoc { modules: vec![m.clone()], ..Default::default() };
    let flat = modules::flatten(&doc, kit).0;
    compose::expand(&flat, kit).into_iter().map(|c| c.at).collect()
}
