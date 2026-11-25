//! Kontrakten mot nettlaget: hendelsene (`Event`) som json, slik
//! `web/main.js` leser dem, og kommandoene fra redigeringen (`EditJson`),
//! slik `web/editor.js` sender dem. Formen på hendelsene står i
//! `web/events.json`, og testene her holder de tre sammen: det motoren
//! sender, fila, og det nettlaget leser.
//! Endres en hendelse, skrives fila på nytt med
//! `FRISTAD_SKRIV_KONTRAKT=1 cargo test kontrakten`.

use crate::adapters::gltf::model_files;
use crate::domain::weather::Daylight;
use crate::domain::edit::LineKind;
use crate::ports::driving::{EditCommand, Event, GearView, Grab};

/// Et kort (utstyret i inventaret og galleriet i skriptoriet).
fn gear_json(g: &GearView) -> serde_json::Value {
    use serde_json::json;
    json!({"id": g.id, "name": g.name, "slot": g.slot, "rarity": g.rarity, "special": g.special, "about": g.about, "stats": g.stats.iter().map(|(l, v, t, up)| json!({"label": l, "value": v, "text": t, "up": up})).collect::<Vec<_>>(), "equipped": g.equipped, "owned": g.owned})
}

/// Én hendelse slik nettlaget får den.
pub fn json(e: &Event) -> serde_json::Value {
    use serde_json::json;
    match e {
        Event::Prompt(Some(p)) => {
            json!({"type": "prompt", "label": p.label, "verb": p.verb, "trade": p.trade, "info": p.info})
        }
        Event::Prompt(None) => json!({"type": "prompt"}),
        Event::Daylight(d) => {
            json!({"type": "theme", "theme": if *d == Daylight::Night { "dark" } else { "light" }})
        }
        Event::Toast(t) => json!({"type": "toast", "text": t}),
        Event::Lore { title, text } => json!({"type": "lore", "title": title, "text": text}),
        Event::Foe(None) => json!({"type": "foe", "foe": null}),
        Event::Foe(Some(f)) => json!({"type": "foe", "foe": {"name": f.name, "info": f.info, "rank": f.rank, "hp": f.hp, "max_hp": f.max_hp, "ranged": f.ranged}}),
        Event::Hurt { angle } => json!({"type": "hurt", "angle": angle}),
        Event::Goal { text, journal } => json!({"type": "goal", "text": text, "journal": journal.iter().map(|(t, b)| json!({"title": t, "text": b})).collect::<Vec<_>>()}),
        Event::Loot(c) => json!({"type": "loot", "component": c}),
        Event::Haul(more) => json!({"type": "haul", "components": more}),
        Event::Ambience(a) => {
            json!({"type": "ambience", "square": a.square, "forge": a.forge, "church": a.church, "meadow": a.meadow, "fire": a.fire, "water": a.water, "lava": a.lava, "chasm": a.chasm, "portal": a.portal, "tavern": a.tavern, "rain": a.rain, "wind": a.wind, "night": a.night, "cellar": a.cellar, "indoor": a.indoor})
        }
        Event::Weather(w) => json!({"type": "weather", "weather": w}),
        Event::Fade => json!({"type": "fade"}),
        Event::FetchLevel(n) => json!({"type": "fetch_level", "level": n}),
        Event::FetchModels(names) => {
            let paths: Vec<&str> = names.iter().filter_map(|n| model_files().find(|(m, _)| *m == n.as_str()).map(|(_, p)| *p)).collect();
            json!({"type": "fetch_models", "names": names, "paths": paths})
        }
        Event::Warp => json!({"type": "warp"}),
        Event::Map(None) => json!({"type": "map", "map": null}),
        Event::Map(Some(m)) => json!({"type": "map", "map": {
            "level": m.level, "name": m.name, "rooms": m.rooms,
            "marks": m.marks.iter().map(|k| json!({"kind": k.kind, "x": k.x, "z": k.z})).collect::<Vec<_>>(),
        }}),
        Event::OpenDoc(c) => json!({"type": "open_doc", "component": c}),
        Event::Where { x, z, yaw } => json!({"type": "where", "x": x, "z": z, "yaw": yaw}),
        Event::Hud { hp, stamina, tokens, cellar, keys, boss, light, fight } => {
            json!({"type": "hud", "hp": hp, "stamina": stamina, "tokens": tokens, "cellar": cellar, "keys": keys, "light": light, "fight": fight, "boss": boss.as_ref().map(|(n, f)| json!({"name": n, "hp": f}))})
        }
        Event::Sound(s) => json!({"type": "sound", "name": s.name, "at": s.at, "gain": s.gain}),
        Event::Inventory(v) => {
            json!({"type": "inventory", "stash": v.stash, "carried": v.carried, "weapon": v.weapon, "weapon_special": v.weapon_special, "shield": v.shield, "armor": v.armor, "lantern": v.lantern, "max_hp": v.max_hp, "max_stamina": v.max_stamina, "tokens": v.tokens, "read": v.read, "gear": v.gear.iter().map(gear_json).collect::<Vec<_>>()})
        }
        Event::Shop { npc, wares, swaps } => {
            let swaps: Vec<_> = swaps.iter().map(|s| json!({"id": s.id, "give": s.give, "n": s.n, "have": s.have, "give_rarity": s.give_rarity, "get": s.get, "get_have": s.get_have, "get_rarity": s.get_rarity, "fee": s.fee, "source": s.source, "ok": s.ok})).collect();
            json!({"type": "shop", "npc": npc, "wares": wares.iter().map(|w| json!({"id": w.id, "name": w.name, "price": w.price})).collect::<Vec<_>>(), "swaps": swaps})
        }
        Event::Mead(m) => json!({"type": "mead", "npc": m.npc, "text": m.text, "price": m.price, "singing": m.singing}),
        Event::Song(on) => json!({"type": "song", "on": on}),
        Event::Workshop { recipes } => {
            json!({"type": "workshop", "recipes": recipes.iter().map(|r| json!({"id": r.id, "name": r.name, "rarity": r.rarity, "result": r.result, "needs": r.needs, "requires": r.requires, "sources": r.sources, "ok": r.ok, "item": gear_json(&r.item), "current": r.current.as_ref().map(gear_json)})).collect::<Vec<_>>()})
        }
        Event::Speech(bubbles) => {
            json!({"type": "speech", "bubbles": bubbles.iter().map(|b| json!({"id": b.id, "x": b.x, "y": b.y, "name": b.name, "text": b.text})).collect::<Vec<_>>()})
        }
        Event::Marks(marks) => {
            json!({"type": "marks", "marks": marks.iter().map(|m| json!({"id": m.id, "x": m.x, "y": m.y, "kind": m.kind, "value": m.value, "text": m.text})).collect::<Vec<_>>()})
        }
        Event::Edit(None) => json!({"type": "edit", "edit": null}),
        Event::Edit(Some(v)) => json!({"type": "edit", "edit": {
            "selected": v.selected.iter().map(|s| json!({"ref": s.r, "label": s.label, "pose": s.pose.to_cols_array(), "center": s.center.to_array(), "half": s.half.to_array()})).collect::<Vec<_>>(),
            "breaches": v.breaches.iter().map(|(r, why)| json!({"ref": r, "why": why})).collect::<Vec<_>>(),
            "can_undo": v.can_undo, "can_redo": v.can_redo, "dirty": v.dirty, "clean": v.clean, "draft": v.draft,
            "under": v.under.iter().map(|(l, sel)| json!({"label": l, "selected": sel})).collect::<Vec<_>>(),
            "props": v.props.iter().map(|p| json!({"key": p.key, "label": p.label, "value": p.value, "options": p.options})).collect::<Vec<_>>(),
            "history": v.history, "step": v.step, "snap": v.snap, "ghost": v.ghost, "error": v.error,
            "group": v.group, "route_point": v.route_point,
        }}),
    }
}


/// En kommando fra redigeringen i nettlaget, som json med `cmd`. `x` og `y`
/// er pekeren (0..1 fra øverst til venstre).
#[derive(serde::Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum EditJson {
    Enter,
    Leave,
    Press {
        x: f32,
        y: f32,
        #[serde(default)]
        add: bool,
        #[serde(default)]
        lift: bool,
        #[serde(default)]
        turn: bool,
    },
    Drag {
        x: f32,
        y: f32,
    },
    Release,
    Pick {
        #[serde(default)]
        add: bool,
        x: Option<f32>,
        y: Option<f32>,
    },
    Cycle,
    Move {
        x: f32,
        y: f32,
        z: f32,
    },
    Rotate {
        a: f32,
    },
    Drop,
    Copy,
    Delete,
    Place {
        model: String,
        x: Option<f32>,
        y: Option<f32>,
    },
    LinePoint {
        x: f32,
        y: f32,
    },
    /// `kind`: "row" (en rad av `model`) eller "wall" (usynlig vegg, `height`).
    LineFinish {
        kind: String,
        model: Option<String>,
        height: Option<f32>,
    },
    LineCancel,
    WallHeight {
        h: f32,
    },
    Undo,
    Redo,
    UndoTo {
        n: usize,
    },
    SelectUnder {
        k: usize,
        #[serde(default)]
        add: bool,
    },
    CycleBack,
    SelectBox {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        #[serde(default)]
        add: bool,
    },
    /// Fantomet under pekeren; uten `model` tas det bort.
    Ghost {
        model: Option<String>,
        #[serde(default)]
        x: f32,
        #[serde(default)]
        y: f32,
    },
    GhostTurn {
        a: f32,
    },
    Snap {
        on: bool,
    },
    SetProp {
        key: String,
        value: String,
    },
    Group {
        #[serde(default)]
        name: String,
    },
    Ungroup,
}

impl EditJson {
    fn command(self) -> Option<EditCommand> {
        let at = |x: Option<f32>, y: Option<f32>| x.zip(y);
        Some(match self {
            EditJson::Enter => EditCommand::Enter,
            EditJson::Leave => EditCommand::Leave,
            EditJson::Press { x, y, add, lift, turn } => EditCommand::Press {
                at: (x, y),
                add,
                grab: if turn {
                    Grab::Turn
                } else if lift {
                    Grab::Lift
                } else {
                    Grab::Move
                },
            },
            EditJson::Drag { x, y } => EditCommand::Drag { to: (x, y) },
            EditJson::Release => EditCommand::Release,
            EditJson::Pick { add, x, y } => EditCommand::Pick { add, at: at(x, y) },
            EditJson::Cycle => EditCommand::Cycle,
            EditJson::Move { x, y, z } => EditCommand::Move(glam::Vec3::new(x, y, z)),
            EditJson::Rotate { a } => EditCommand::Rotate(a),
            EditJson::Drop => EditCommand::Drop,
            EditJson::Copy => EditCommand::Copy,
            EditJson::Delete => EditCommand::Delete,
            EditJson::Place { model, x, y } => EditCommand::Place { model, at: at(x, y) },
            EditJson::LinePoint { x, y } => EditCommand::LinePoint { at: (x, y) },
            EditJson::LineFinish { kind, model, height } => EditCommand::LineFinish(match kind.as_str() {
                "wall" => LineKind::Wall { height: height.unwrap_or(3.0) },
                _ => LineKind::Row { model: model?, spacing: 0.0 },
            }),
            EditJson::LineCancel => EditCommand::LineCancel,
            EditJson::WallHeight { h } => EditCommand::WallHeight(h),
            EditJson::Undo => EditCommand::Undo,
            EditJson::Redo => EditCommand::Redo,
            EditJson::UndoTo { n } => EditCommand::UndoTo(n),
            EditJson::SelectUnder { k, add } => EditCommand::SelectUnder { k, add },
            EditJson::CycleBack => EditCommand::CycleBack,
            EditJson::SelectBox { x0, y0, x1, y1, add } => EditCommand::SelectBox { a: (x0, y0), b: (x1, y1), add },
            EditJson::Ghost { model, x, y } => EditCommand::Ghost(model.map(|m| (m, (x, y)))),
            EditJson::GhostTurn { a } => EditCommand::GhostTurn(a),
            EditJson::Snap { on } => EditCommand::Snap(on),
            EditJson::SetProp { key, value } => EditCommand::SetProp { key, value },
            EditJson::Group { name } => EditCommand::Group(name),
            EditJson::Ungroup => EditCommand::Ungroup,
        })
    }
}

/// En kommando fra redigeringen i nettlaget (json med `cmd`), eller `None`
/// om den ikke kan leses.
pub fn edit_command(json: &str) -> Option<EditCommand> {
    serde_json::from_str::<EditJson>(json).ok().and_then(EditJson::command)
}

#[cfg(test)]
#[path = "wire_tests.rs"]
mod tests;
