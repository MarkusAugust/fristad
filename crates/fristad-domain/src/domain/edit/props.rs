//! Egenskapene til det valgte, som panelet viser og eieren skriver inn:
//! stedet (x, y, z i verden), dreiningen i grader og fargen for alt, og for
//! merkene i krypten det slaget bruker (spakens `opens`, portens `id`,
//! lappens tekst, kistens innhold, fiendens slag, antall og runde), for
//! lysene fargen, rekkevidden, styrken og om det blafrer. En endring er en
//! vanlig endring av dataene, så angre gjelder den også.

use super::editor::{Change, Editor, Thing};
use crate::domain::world::crypt::level::MarkDef;
use crate::domain::world::layout::Pose;
use glam::{Affine3A, Vec3};

/// Én egenskap: nøkkelen, navnet i panelet og verdien som tekst.
#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub key: String,
    pub label: String,
    pub value: String,
}

fn prop(key: &str, label: &str, value: String) -> Prop {
    Prop { key: key.into(), label: label.into(), value }
}

/// Et tall med høyst to desimaler, uten unødvendige nuller.
fn num(v: f32) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

/// Fargen som «#rrggbb» (0..1 per kanal).
pub fn hex(c: Vec3) -> String {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", b(c.x), b(c.y), b(c.z))
}

fn parse_hex(s: &str) -> Result<Vec3, String> {
    let t = s.trim().trim_start_matches('#');
    let ok = t.len() == 6 && t.chars().all(|c| c.is_ascii_hexdigit());
    if !ok {
        return Err(format!("«{s}» er ikke en farge (#rrggbb)"));
    }
    let ch = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).map(|v| v as f32 / 255.0).unwrap_or(0.0);
    Ok(Vec3::new(ch(0), ch(2), ch(4)))
}

fn parse_num(s: &str) -> Result<f32, String> {
    s.trim().replace(',', ".").parse::<f32>().ok().filter(|v| v.is_finite()).ok_or_else(|| format!("«{s}» er ikke et tall"))
}

fn opt_text(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn opt_num(s: &str) -> Result<Option<f32>, String> {
    if s.trim().is_empty() {
        Ok(None)
    } else {
        parse_num(s).map(Some)
    }
}

/// En runde som tekst: punktene (x, y, z i nivåets rom) skilt med semikolon.
fn patrol_text(p: &[[f32; 3]]) -> String {
    p.iter().map(|q| format!("{},{},{}", num(q[0]), num(q[1]), num(q[2]))).collect::<Vec<_>>().join("; ")
}

fn parse_patrol(s: &str) -> Result<Vec<[f32; 3]>, String> {
    s.split(';')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| {
            let v: Vec<f32> = t.split(|c: char| c == ',' || c.is_whitespace()).filter(|x| !x.is_empty()).map(parse_num).collect::<Result<_, _>>()?;
            match v.as_slice() {
                [x, y, z] => Ok([*x, *y, *z]),
                [x, z] => Ok([*x, 0.0, *z]),
                _ => Err(format!("«{t}» er ikke et punkt (x, y, z)")),
            }
        })
        .collect()
}

/// Feltene et merke av slaget `kind` bruker (de som er satt, vises også).
fn mark_fields(kind: &str) -> &'static [&'static str] {
    match kind {
        "mob" => &["label", "n", "patrol"],
        "elite" | "boss" => &["label", "arena", "patrol"],
        "rare" | "bonus" => &["label"],
        "chest" => &["loot", "id"],
        "lore" => &["label", "text"],
        "lever" => &["id", "opens"],
        "gate" => &["id", "width", "arena"],
        "shortcut" => &["id", "from", "width"],
        "secret_wall" => &["id", "width", "height"],
        "trap" => &["id", "label"],
        "hint" => &["for", "label"],
        "bridge" | "water" | "water_block" | "trapdoor" | "ladder" => &["id"],
        _ => &["label"],
    }
}

/// Navnet på et felt i panelet.
fn mark_label(kind: &str, key: &str) -> &'static str {
    match (kind, key) {
        ("mob" | "elite" | "boss", "label") => "slag",
        ("lore", "label") => "tittel",
        ("rare" | "bonus", "label") => "gave",
        (_, "label") => "merkelapp",
        (_, "n") => "antall",
        (_, "patrol") => "runde (x,y,z; …)",
        (_, "loot") => "innhold",
        (_, "text") => "tekst",
        (_, "opens") => "åpner (id, …)",
        (_, "width") => "bredde",
        (_, "height") => "høyde",
        (_, "from") => "fra rommet",
        (_, "for") => "for (id)",
        (_, "arena") => "arena",
        _ => "id",
    }
}

fn mark_value(m: &MarkDef, key: &str) -> String {
    match key {
        "label" => m.label.clone().unwrap_or_default(),
        "id" => m.id.clone().unwrap_or_default(),
        "n" => m.n.map(|n| n.to_string()).unwrap_or_default(),
        "patrol" => patrol_text(&m.patrol),
        "loot" => m.loot.clone().unwrap_or_default(),
        "text" => m.text.clone().unwrap_or_default(),
        "opens" => m.opens.join(", "),
        "width" => m.width.map(num).unwrap_or_default(),
        "height" => m.height.map(num).unwrap_or_default(),
        "from" => m.from.clone().unwrap_or_default(),
        "for" => m.for_id.clone().unwrap_or_default(),
        "arena" => m.arena.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

const MARK_KEYS: [&str; 12] = ["label", "id", "n", "patrol", "loot", "text", "opens", "width", "height", "from", "for", "arena"];

fn set_mark(m: &mut MarkDef, key: &str, v: &str) -> Result<(), String> {
    match key {
        "label" => m.label = opt_text(v),
        "id" => m.id = opt_text(v),
        "n" => m.n = if v.trim().is_empty() { None } else { Some(v.trim().parse::<u32>().map_err(|_| format!("«{v}» er ikke et antall"))?) },
        "patrol" => m.patrol = parse_patrol(v)?,
        "loot" => m.loot = opt_text(v),
        "text" => m.text = opt_text(v),
        "opens" => m.opens = v.split(',').filter_map(opt_text).collect(),
        "width" => m.width = opt_num(v)?,
        "height" => m.height = opt_num(v)?,
        "from" => m.from = opt_text(v),
        "for" => m.for_id = opt_text(v),
        "arena" => m.arena = opt_text(v),
        _ => return Err(format!("{} har ikke «{key}»", m.kind)),
    }
    Ok(())
}

impl Editor {
    /// Egenskapene til den første valgte tingen (tomt uten valg).
    pub fn props(&self) -> Vec<Prop> {
        let Some(r) = self.selection.first() else { return vec![] };
        let (Some(t), Some(pose)) = (self.get(r), self.pose_of(r)) else { return vec![] };
        let p = Pose::of(pose);
        let mut out = vec![prop("x", "x", num(p.pos.x)), prop("y", "y", num(p.pos.y)), prop("z", "z", num(p.pos.z))];
        match t {
            Thing::Item(it) => {
                out.push(prop("yaw", "dreining (°)", num(p.yaw().to_degrees())));
                if it.model.is_some() {
                    out.push(prop("tint", "farge", it.tint.map(hex).unwrap_or_default()));
                }
            }
            Thing::Npc(n) => {
                out.push(prop("yaw", "dreining (°)", num(n.yaw.to_degrees())));
                out.push(prop("name", "navn", n.name.clone()));
                out.push(prop("lines", "replikker", n.lines.clone()));
                out.push(prop("tint", "farge", hex(n.tint)));
            }
            Thing::Mark(m) => {
                out.push(prop("yaw", "dreining (°)", num(m.yaw.to_degrees())));
                out.push(prop("kind", "slag merke", m.kind.clone()));
                let wanted = mark_fields(&m.kind);
                for k in MARK_KEYS {
                    let v = mark_value(&m, k);
                    if wanted.contains(&k) || !v.is_empty() {
                        out.push(prop(k, mark_label(&m.kind, k), v));
                    }
                }
            }
            Thing::Light(l) => {
                out.push(prop("color", "farge", hex(Vec3::from(l.color))));
                out.push(prop("range", "rekkevidde (m)", num(l.range)));
                out.push(prop("intensity", "styrke", num(l.intensity.unwrap_or(2.0))));
                out.push(prop("flicker", "blafrer", if l.flicker { "ja" } else { "nei" }.into()));
            }
        }
        out
    }

    /// Sett en egenskap på det valgte (stedet og dreiningen bare med én ting
    /// valgt, resten på alle som har den). Angres som ett steg.
    pub fn set_prop(&mut self, key: &str, value: &str) -> Result<(), String> {
        let sel = self.selection.clone();
        if sel.is_empty() {
            return Err("Ingenting er valgt.".into());
        }
        let mut changes = vec![];
        match key {
            "x" | "y" | "z" | "yaw" => {
                let [r] = sel.as_slice() else {
                    return Err("Stedet settes på én ting om gangen.".into());
                };
                let v = parse_num(value)?;
                let k = ["x", "y", "z"].iter().position(|a| *a == key);
                changes.extend(self.pose_change(r, |t| match k {
                    Some(k) => {
                        let mut p = Vec3::from(t.translation);
                        p[k] = v;
                        Affine3A::from_translation(p - Vec3::from(t.translation)) * t
                    }
                    // Dreies om sitt eget origo, så en skrå ting forblir skrå.
                    None => {
                        let p = Vec3::from(t.translation);
                        let d = v.to_radians() - Pose::of(t).yaw();
                        Affine3A::from_translation(p) * Affine3A::from_rotation_y(d) * Affine3A::from_translation(-p) * t
                    }
                }));
            }
            _ => {
                for r in &sel {
                    let Some(before) = self.get(r) else { continue };
                    let after = Self::with_prop(&before, key, value)?;
                    if after != before {
                        changes.push(Change::Set { r: r.clone(), before, after });
                    }
                }
            }
        }
        self.commit(changes);
        Ok(())
    }

    fn with_prop(t: &Thing, key: &str, v: &str) -> Result<Thing, String> {
        let mut t = t.clone();
        match (&mut t, key) {
            (Thing::Item(it), "tint") => it.tint = if v.trim().is_empty() { None } else { Some(parse_hex(v)?) },
            (Thing::Npc(n), "tint") => n.tint = parse_hex(v)?,
            (Thing::Npc(n), "name") => n.name = opt_text(v).ok_or("Navnet kan ikke være tomt.")?,
            (Thing::Npc(n), "lines") => n.lines = v.trim().to_string(),
            (Thing::Mark(m), "kind") => m.kind = opt_text(v).ok_or("Slaget kan ikke være tomt.")?,
            (Thing::Mark(m), k) => set_mark(m, k, v)?,
            (Thing::Light(l), "color") => l.color = parse_hex(v)?.to_array(),
            (Thing::Light(l), "range") => l.range = parse_num(v)?.max(0.1),
            (Thing::Light(l), "intensity") => l.intensity = Some(parse_num(v)?.max(0.0)),
            (Thing::Light(l), "flicker") => l.flicker = matches!(v.trim().to_lowercase().as_str(), "ja" | "true" | "1" | "j" | "yes"),
            _ => {}
        }
        Ok(t)
    }
}
