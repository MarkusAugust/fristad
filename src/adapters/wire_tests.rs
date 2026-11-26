//! Kontrakten mellom motoren og nettlaget som tester: `web/events.json` er
//! formen på hver hendelse, regnet ut fra en fullt utfylt prøve av hver
//! variant, og `web/main.js` får bare lese det som står der.

use super::json;
use crate::domain::edit::{Prop, Selected};
use crate::domain::weather::Daylight;
use crate::domain::world::layout::Ref;
use crate::ports::driving::*;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn gear() -> GearView {
    GearView {
        id: "sword".into(),
        name: "Sverd".into(),
        slot: "weapon".into(),
        rarity: "vanlig".into(),
        special: "s".into(),
        about: "a".into(),
        stats: vec![("Skade".into(), 1.0, "1".into(), true)],
        equipped: true,
        owned: true,
    }
}

/// Prøven som kommer etter `prev`, med alt fylt ut (ingen tomme lister og
/// ingen `None`), så formen blir hel. Hver variant peker på den neste: en ny
/// variant i `Event` kompilerer ikke før den står her, og den hektes på ved
/// at den forrige siste peker på den.
fn next(prev: Option<&Event>) -> Option<Event> {
    let s = |t: &str| t.to_string();
    Some(match prev {
        None => Event::Prompt(Some(Prompt { label: s("l"), verb: s("v"), trade: s("t"), info: s("i") })),
        Some(Event::Prompt(_)) => Event::Daylight(Daylight::Night),
        Some(Event::Daylight(_)) => Event::Toast(s("t")),
        Some(Event::Toast(_)) => Event::Lore { title: s("t"), text: s("x") },
        Some(Event::Lore { .. }) => Event::Foe(Some(FoeView { name: s("n"), info: s("i"), rank: s("Boss"), hp: 1, max_hp: 2, ranged: true })),
        Some(Event::Foe(_)) => Event::Hurt { angle: 0.5 },
        Some(Event::Hurt { .. }) => Event::Goal { text: s("t"), journal: vec![(s("t"), s("x"))] },
        Some(Event::Goal { .. }) => Event::Haul(vec![s("button")]),
        Some(Event::Haul(_)) => Event::Loot(Some(s("button"))),
        Some(Event::Loot(_)) => Event::Weather(s("regn")),
        Some(Event::Weather(_)) => Event::Fade,
        Some(Event::Fade) => Event::Warp,
        Some(Event::Warp) => Event::Hud { hp: 1, stamina: 1, tokens: 1, cellar: true, keys: 1, boss: Some((s("b"), 1)), light: 1, fight: 1 },
        Some(Event::Hud { .. }) => Event::Sound(Sound { name: s("step"), at: Some([0.5, 0.5, 0.5]), gain: 0.5 }),
        Some(Event::Sound(_)) => Event::Marks(vec![Mark { id: 1, x: 0.5, y: 0.5, kind: "bar", value: 0.5, text: s("t") }]),
        Some(Event::Marks(_)) => Event::Speech(vec![Bubble { id: 1, x: 0.5, y: 0.5, name: s("n"), text: s("t") }]),
        Some(Event::Speech(_)) => Event::Inventory(InventoryView {
            stash: vec![(s("button"), 1)],
            carried: vec![(s("button"), 1)],
            weapon: s("w"),
            weapon_special: s("s"),
            shield: s("s"),
            armor: s("a"),
            lantern: true,
            max_hp: 1,
            max_stamina: 1,
            tokens: 1,
            read: vec![s("button")],
            gear: vec![gear()],
        }),
        Some(Event::Inventory(_)) => Event::Shop {
            npc: s("n"),
            wares: vec![WareView { id: s("i"), name: s("n"), price: 1 }],
            swaps: vec![SwapView { id: s("i"), give: s("g"), n: 1, have: 1, give_rarity: s("r"), get: s("g"), get_have: 1, get_rarity: s("r"), fee: 1, source: s("s"), ok: true }],
        },
        Some(Event::Shop { .. }) => Event::Workshop {
            recipes: vec![RecipeView {
                id: s("i"),
                name: s("n"),
                rarity: s("r"),
                result: s("r"),
                needs: vec![(s("button"), 1, 1)],
                requires: vec![(s("button"), true)],
                sources: vec![(s("button"), s("kister"), true)],
                ok: true,
                item: gear(),
                current: Some(gear()),
            }],
        },
        Some(Event::Workshop { .. }) => Event::Mead(MeadView { npc: s("n"), text: s("t"), price: 1, singing: true }),
        Some(Event::Mead(_)) => Event::Song(true),
        Some(Event::Song(_)) => Event::Ambience(Ambience {
            square: 0.5,
            forge: 0.5,
            church: 0.5,
            meadow: 0.5,
            fire: 0.5,
            water: 0.5,
            lava: 0.5,
            chasm: 0.5,
            portal: 0.5,
            tavern: 0.5,
            rain: 0.5,
            wind: 0.5,
            night: true,
            cellar: true,
            indoor: true,
        }),
        Some(Event::Ambience(_)) => Event::Map(Some(MapView { level: 1, name: s("n"), rooms: vec![vec![[0.5, 0.5]]], marks: vec![MapMark { kind: s("exit"), x: 0.5, z: 0.5 }] })),
        Some(Event::Map(_)) => Event::Where { x: 0.5, z: 0.5, yaw: 0.5 },
        Some(Event::Where { .. }) => Event::OpenDoc(s("button")),
        Some(Event::OpenDoc(_)) => Event::Edit(Some(EditView {
            selected: vec![Selected { r: Ref::Town(1), label: s("l"), pose: glam::Affine3A::IDENTITY, center: glam::Vec3::ONE, half: glam::Vec3::ONE }],
            breaches: vec![(Ref::Town(1), s("w"))],
            can_undo: true,
            can_redo: true,
            dirty: true,
            clean: true,
            draft: 1,
            under: vec![(s("l"), true)],
            props: vec![Prop { key: s("k"), label: s("l"), value: s("v"), options: vec![s("o")] }],
            history: vec![s("h")],
            step: 1,
            snap: true,
            ghost: Some(vec![s("g")]),
            error: Some(s("e")),
            group: Some(s("g")),
            route_point: Some(1),
        })),
        Some(Event::Edit(_)) => return None,
    })
}

fn samples() -> Vec<Event> {
    let mut out: Vec<Event> = vec![];
    while let Some(e) = next(out.last()) {
        assert!(out.len() < 1000, "prøvene går i ring");
        out.push(e);
    }
    out
}

/// Formen på en verdi: typene i stedet for verdiene, og ett element per
/// liste. En tom liste eller `null` gir ingen form, og er en feil i prøven.
fn shape(v: &Value, at: &str) -> Value {
    match v {
        Value::Null => panic!("{at}: prøven har null, fyll den ut"),
        Value::Bool(_) => "bool".into(),
        Value::Number(_) => "tall".into(),
        Value::String(_) => "tekst".into(),
        Value::Array(a) => {
            assert!(!a.is_empty(), "{at}: prøven har en tom liste, fyll den ut");
            Value::Array(vec![shape(&a[0], &format!("{at}[]"))])
        }
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), shape(v, &format!("{at}.{k}")))).collect()),
    }
}

/// Kontrakten: hendelsens `type` → formen på resten.
fn contract() -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    for e in samples() {
        let Value::Object(mut o) = json(&e) else { panic!("en hendelse er et objekt") };
        let Some(Value::String(kind)) = o.remove("type") else { panic!("en hendelse har type") };
        let prev = out.insert(kind.clone(), shape(&Value::Object(o), &kind));
        assert!(prev.is_none(), "to varianter sender typen {kind}");
    }
    out
}

fn written() -> String {
    let map: Map<String, Value> = contract().into_iter().collect();
    serde_json::to_string_pretty(&Value::Object(map)).unwrap() + "\n"
}

/// Det motoren sender, er det `web/events.json` sier.
#[test]
fn hendelsene_folger_kontrakten() {
    let path = root().join("web/events.json");
    let now = written();
    if std::env::var_os("FRISTAD_SKRIV_KONTRAKT").is_some() {
        std::fs::write(&path, &now).unwrap();
    }
    let file = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(file == now, "web/events.json stemmer ikke med hendelsene motoren sender. Se over web/main.js, og skriv fila med FRISTAD_SKRIV_KONTRAKT=1 cargo test kontrakten");
}

fn ident(b: &[u8], mut i: usize) -> (&str, usize) {
    let from = i;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'$') {
        i += 1;
    }
    (std::str::from_utf8(&b[from..i]).unwrap(), i)
}

/// Hvert `navn.felt` og `navn?.felt` i teksten (der `navn` ikke er en del av
/// et lengre navn eller et felt på noe annet), med stedet rett etter feltet.
fn reads<'a>(text: &'a str, name: &str) -> Vec<(&'a str, usize)> {
    let b = text.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while let Some(k) = text[i..].find(name) {
        let at = i + k;
        i = at + name.len();
        let free = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || matches!(b[at - 1], b'_' | b'$' | b'.'));
        let mut j = i;
        if b.get(j) == Some(&b'?') {
            j += 1;
        }
        if !free || b.get(j) != Some(&b'.') {
            continue;
        }
        let (field, end) = ident(b, j + 1);
        if !field.is_empty() {
            out.push((field, end));
        }
    }
    out
}

/// `handle(e)` i `web/main.js`: typen i hver `case` og feltene blokka leser
/// rett på hendelsen (`e.felt`).
fn handled() -> BTreeMap<String, BTreeSet<String>> {
    let js = std::fs::read_to_string(root().join("web/main.js")).unwrap();
    let from = js.find("function handle(e) {").expect("handle(e) finnes i web/main.js");
    let body = &js[from..];
    let body = &body[..body.find("\n}\n").expect("handle(e) slutter")];
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut open: Vec<String> = vec![];
    for line in body.lines() {
        let l = line.trim();
        if let Some(kind) = l.strip_prefix("case \"").and_then(|r| r.strip_suffix("\":")) {
            out.entry(kind.to_string()).or_default();
            open.push(kind.to_string());
            continue;
        }
        for kind in &open {
            out.get_mut(kind).unwrap().extend(reads(l, "e").into_iter().map(|(f, _)| f.to_string()));
        }
        if l == "break;" {
            open.clear();
        }
    }
    out
}

/// Nettlaget tar imot hver hendelse motoren sender og ingen andre, og leser
/// bare felt som finnes. (Der hele hendelsen sendes videre til en funksjon,
/// følger testen ikke med.)
#[test]
fn nettlaget_leser_bare_det_som_sendes() {
    let contract = contract();
    let handled = handled();
    let mut bad = vec![];
    for kind in contract.keys().filter(|k| !handled.contains_key(*k)) {
        bad.push(format!("motoren sender «{kind}», men handle(e) i web/main.js tar ikke imot den"));
    }
    for (kind, fields) in &handled {
        let Some(shape) = contract.get(kind) else {
            bad.push(format!("handle(e) venter på «{kind}», som motoren aldri sender"));
            continue;
        };
        for f in fields.iter().filter(|f| *f != "type" && shape.get(f.as_str()).is_none()) {
            bad.push(format!("handle(e) leser e.{f} i «{kind}», men feltet sendes ikke"));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Det nettlaget kaller på motoren (`engine.navn(…)`), finnes som `pub fn`
/// i nettadapteren.
#[test]
fn nettlaget_kaller_bare_det_motoren_har() {
    let mut have = BTreeSet::new();
    for file in ["src/adapters/web.rs", "src/adapters/web_level.rs"] {
        let text = std::fs::read_to_string(root().join(file)).unwrap();
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("pub fn ").or_else(|| line.trim().strip_prefix("pub async fn ")) {
                have.insert(ident(rest.as_bytes(), 0).0.to_string());
            }
        }
    }
    let mut bad = BTreeSet::new();
    // Alle modulene i `web/` (ikke undermappene).
    let mut files: Vec<_> = std::fs::read_dir(root().join("web")).unwrap().map(|f| f.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "js")).collect();
    files.sort();
    assert!(files.len() > 10, "fant bare {} moduler i web/", files.len());
    for path in files {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        let b = text.as_bytes();
        for (name, end) in reads(&text, "engine") {
            // Bare kall: `engine.navn(` og `engine.navn?.(`.
            let call = b.get(end) == Some(&b'(') || text[end..].starts_with("?.(");
            if call && !have.contains(name) {
                bad.insert(format!("web/{file} kaller engine.{name}(), som motoren ikke har"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.into_iter().collect::<Vec<_>>().join("\n"));
}

/// Kommandoene redigeringen sender (`cmd: "navn"` i `web/editor.js`), er
/// kommandoer motoren kjenner (`EditJson`).
#[test]
fn redigeringen_sender_bare_kommandoer_motoren_kjenner() {
    let text = std::fs::read_to_string(root().join("web/editor.js")).unwrap();
    let mut names = BTreeSet::new();
    for part in text.split("cmd: ").skip(1) {
        // Uttrykket fram til neste felt, også `a ? "x" : "y"`.
        let expr = &part[..part.find([',', '}']).unwrap_or(part.len())];
        names.extend(expr.split('"').skip(1).step_by(2).map(str::to_string));
    }
    assert!(names.len() > 20, "fant bare {} kommandoer i web/editor.js", names.len());
    let unknown: Vec<_> = names
        .into_iter()
        .filter(|n| {
            let err = serde_json::from_str::<super::EditJson>(&format!("{{\"cmd\":\"{n}\"}}")).err().map(|e| e.to_string()).unwrap_or_default();
            err.contains("unknown variant")
        })
        .collect();
    assert!(unknown.is_empty(), "web/editor.js sender kommandoer motoren ikke kjenner: {unknown:?}");
}
