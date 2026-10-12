//! Skriver byen ut som data (`assets/layout/town.json`) fra koden i
//! `town.rs`. Brukes én gang for å flytte byen over i data; etterpå endres
//! dataene (redigeringsmodusen, `docs/redigering.md`). Tallene rundes til
//! 0,01 mm, og skala 1 skrives ikke, så fila er lett å lese.
//!
//!     cargo run --release --bin layout > assets/layout/town.json

use fristad::adapters::gltf::{Embedded, FromCode};
use fristad::application::prepare;
use serde_json::Value;

fn round(v: &mut Value) {
    match v {
        Value::Number(n) => {
            if let Some(f) = n.as_f64().filter(|_| !n.is_i64() && !n.is_u64()) {
                let r = (f * 1e5).round() / 1e5;
                *v = if r == r.trunc() && r.abs() < 1e9 { Value::from(r as i64) } else { Value::from(r) };
            }
        }
        Value::Array(a) => a.iter_mut().for_each(round),
        Value::Object(o) => {
            o.values_mut().for_each(round);
            if o.get("scale") == Some(&serde_json::json!([1, 1, 1])) {
                o.remove("scale");
            }
        }
        _ => {}
    }
}

fn main() {
    let p = prepare(&FromCode(&Embedded), 7);
    let layout = p.world.layout.expect("byen ble bygd av koden");
    let mut v = serde_json::to_value(&layout).unwrap();
    round(&mut v);
    // Én ting og én innbygger per linje.
    let o = v.as_object().unwrap();
    let mut out = String::from("{\n");
    for (i, (k, val)) in o.iter().enumerate() {
        let sep = if i + 1 < o.len() { "," } else { "" };
        match val.as_array().filter(|_| k == "items" || k == "npcs") {
            Some(list) => {
                out += &format!("  {k:?}: [\n");
                for (j, x) in list.iter().enumerate() {
                    out += &format!("    {}{}\n", x, if j + 1 < list.len() { "," } else { "" });
                }
                out += &format!("  ]{sep}\n");
            }
            None => out += &format!("  {k:?}: {val}{sep}\n"),
        }
    }
    out += "}";
    println!("{out}");
}
