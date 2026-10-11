//! Fristad i nettleseren: den drivende adapteren for wasm.
//!
//! JavaScript-laget (`web/main.js`) eier løkka, input og overleggene, og
//! kaller inn hit. Alt går gjennom den drivende porten (`Play`), og
//! hendelsene går tilbake som JSON fra `frame()`.

use crate::adapters::gltf::{model_files, Files};
use crate::adapters::rapier::RapierPhysics;
use crate::adapters::wgpu::Renderer;
use crate::application::{prepare, App};
use crate::domain::weather::{Daylight, WeatherKind};
use crate::ports::driving::{Command, Event, Play, SaveData};
use crate::ports::{Bench, Quality};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn boot() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
}

#[wasm_bindgen]
pub struct Engine {
    app: App<Renderer, RapierPhysics>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    backend: String,
}

async fn connect(canvas: &web_sys::HtmlCanvasElement, webgpu: bool) -> Result<(wgpu::Surface<'static>, wgpu::Adapter, wgpu::Device, wgpu::Queue), String> {
    let instance = if webgpu {
        wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor { backends: wgpu::Backends::BROWSER_WEBGPU, ..Default::default() }).await
    } else {
        wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, ..Default::default() })
    };
    let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone())).map_err(|e| format!("Kunne ikke tegne i canvas: {e}"))?;
    let (adapter, device, queue) = crate::adapters::gpu(&instance, Some(&surface)).await?;
    Ok((surface, adapter, device, queue))
}

/// Start motoren i et canvas, med WebGPU eller WebGL2.
///
/// Et canvas som har gitt ut en WebGPU-kontekst kan ikke gi WebGL etterpå,
/// så JavaScript-laget prøver WebGPU først og gir oss et nytt canvas hvis
/// det må falle tilbake.
/// Filstiene i teksturmanifestet, så nettleseren kan hente dem til `start`.
#[wasm_bindgen]
pub fn asset_paths(manifest_json: &str) -> Vec<String> {
    crate::adapters::layers::paths_in(manifest_json)
}

/// Modellfilene (stier under `models/`, gzip-pakket med `.gz` på slutten
/// av bygget), så nettleseren kan hente dem til `start` i samme rekkefølge.
#[wasm_bindgen]
pub fn model_paths() -> Vec<String> {
    model_files().map(|(_, p)| p.to_string()).collect()
}

#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement, webgpu: bool, manifest_json: Option<String>, names: js_sys::Array, blobs: js_sys::Array, models: js_sys::Array) -> Result<Engine, JsValue> {
    let (surface, adapter, device, queue) = connect(&canvas, webgpu).await.map_err(|e| JsValue::from_str(&e))?;
    let backend = format!("{:?}", adapter.get_info().backend);
    let caps = surface.get_capabilities(&adapter);
    let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: canvas.width().max(1),
        height: canvas.height().max(1),
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    // Ny kjeller for hver økt i nettleseren.
    let seed = (js_sys::Date::now() as u64) ^ 0x5eed;
    let files = model_paths().into_iter().zip(models.iter()).filter(|(_, b)| !b.is_undefined()).map(|(p, b)| (p, js_sys::Uint8Array::new(&b).to_vec())).collect();
    let prepared = prepare(&Files(files), seed);
    let layers = match manifest_json {
        Some(m) => {
            let files: std::collections::HashMap<String, Vec<u8>> = names.iter().zip(blobs.iter()).filter_map(|(n, b)| Some((n.as_string()?, js_sys::Uint8Array::new(&b).to_vec()))).collect();
            crate::adapters::layers::from_files(&m, &files)
        }
        None => vec![],
    };
    let renderer = Renderer::new(device.clone(), queue, &adapter, format, &App::<Renderer, RapierPhysics>::scene(&prepared), &layers, config.width, config.height);
    let app = App::new(prepared, renderer, RapierPhysics::new());
    Ok(Engine { app, surface, config, device, backend })
}

fn json(e: &Event) -> serde_json::Value {
    use serde_json::json;
    match e {
        Event::Prompt(Some(p)) => json!({"type": "prompt", "label": p.label, "verb": p.verb, "trade": p.trade, "info": p.info}),
        Event::Prompt(None) => json!({"type": "prompt"}),
        Event::Daylight(d) => json!({"type": "theme", "theme": if *d == Daylight::Night { "dark" } else { "light" }}),
        Event::Toast(t) => json!({"type": "toast", "text": t}),
        Event::Loot(c) => json!({"type": "loot", "component": c}),
        Event::Ambience(a) => json!({"type": "ambience", "square": a.square, "forge": a.forge, "church": a.church, "meadow": a.meadow, "fire": a.fire, "water": a.water, "lava": a.lava, "chasm": a.chasm, "portal": a.portal, "rain": a.rain, "wind": a.wind, "night": a.night, "cellar": a.cellar, "indoor": a.indoor}),
        Event::Weather(w) => json!({"type": "weather", "weather": w}),
        Event::Fade => json!({"type": "fade"}),
        Event::Warp => json!({"type": "warp"}),
        Event::Map(None) => json!({"type": "map", "map": null}),
        Event::Map(Some(m)) => json!({"type": "map", "map": {
            "level": m.level, "name": m.name, "rooms": m.rooms,
            "marks": m.marks.iter().map(|k| json!({"kind": k.kind, "x": k.x, "z": k.z})).collect::<Vec<_>>(),
        }}),
        Event::OpenDoc(c) => json!({"type": "open_doc", "component": c}),
        Event::Where { x, z, yaw } => json!({"type": "where", "x": x, "z": z, "yaw": yaw}),
        Event::Hud { hp, stamina, tokens, cellar, keys, boss } => json!({"type": "hud", "hp": hp, "stamina": stamina, "tokens": tokens, "cellar": cellar, "keys": keys, "boss": boss.as_ref().map(|(n, f)| json!({"name": n, "hp": f}))}),
        Event::Sound(s) => json!({"type": "sound", "name": s.name, "at": s.at, "gain": s.gain}),
        Event::Inventory(v) => json!({"type": "inventory", "stash": v.stash, "carried": v.carried, "weapon": v.weapon, "weapon_special": v.weapon_special, "shield": v.shield, "armor": v.armor, "lantern": v.lantern, "max_hp": v.max_hp, "max_stamina": v.max_stamina, "tokens": v.tokens, "read": v.read}),
        Event::Shop { npc, wares } => json!({"type": "shop", "npc": npc, "wares": wares.iter().map(|w| json!({"id": w.id, "name": w.name, "price": w.price})).collect::<Vec<_>>()}),
        Event::Workshop { recipes } => json!({"type": "workshop", "recipes": recipes.iter().map(|r| json!({"id": r.id, "name": r.name, "rarity": r.rarity, "result": r.result, "needs": r.needs, "requires": r.requires, "sources": r.sources, "ok": r.ok})).collect::<Vec<_>>()}),
        Event::Speech(bubbles) => json!({"type": "speech", "bubbles": bubbles.iter().map(|b| json!({"id": b.id, "x": b.x, "y": b.y, "name": b.name, "text": b.text})).collect::<Vec<_>>()}),
        Event::Marks(marks) => json!({"type": "marks", "marks": marks.iter().map(|m| json!({"id": m.id, "x": m.x, "y": m.y, "kind": m.kind, "value": m.value, "text": m.text})).collect::<Vec<_>>()}),
    }
}

#[wasm_bindgen]
impl Engine {
    /// "BrowserWebGpu" eller "Gl".
    pub fn backend(&self) -> String {
        self.backend.clone()
    }

    /// Versjonen av Fristil som komponentene i spillet hører til
    /// (`assets/tokens.json`, fra `tools/export-tokens.ts`).
    pub fn fristil_version(&self) -> String {
        #[derive(serde::Deserialize)]
        struct Raw {
            fristil: String,
        }
        serde_json::from_str::<Raw>(include_str!("../../assets/tokens.json")).map(|r| r.fristil).unwrap_or_default()
    }

    pub fn weather(&self) -> String {
        self.app.status().weather.name().to_string()
    }

    pub fn key(&mut self, code: &str, down: bool) {
        self.app.input().key(code, down);
    }

    /// Musetast: 0 venstre (hugg), 2 høyre (blokk).
    pub fn mouse(&mut self, button: u32, down: bool) {
        self.app.input().mouse(button, down);
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.app.input().look.0 += dx;
        self.app.input().look.1 += dy;
    }

    pub fn release_all(&mut self) {
        self.app.input().release_all();
    }

    /// Smi etter en oppskrift (id fra skriptoriet). Gir om det gikk.
    pub fn craft(&mut self, recipe: &str) -> bool {
        self.app.command(Command::Craft(recipe.into()))
    }

    /// Kjøp en vare (id fra butikken). Gir om det gikk.
    pub fn buy(&mut self, ware: &str) -> bool {
        self.app.command(Command::Buy(ware.into()))
    }

    /// Boka: lesing av en komponents dokumentasjon gir liv.
    pub fn read_doc(&mut self, component: &str) -> bool {
        self.app.command(Command::ReadDoc(component.into()))
    }

    /// Lageret og utstyret som JSON, til localStorage.
    pub fn save_state(&self) -> String {
        let d = self.app.save();
        serde_json::json!({"stash": d.stash, "weapon": d.weapon, "shield": d.shield, "armor": d.armor, "lantern": d.lantern, "max_hp": d.max_hp, "max_stamina": d.max_stamina, "coins": d.coins, "found": d.found, "bonuses": d.bonuses}).to_string()
    }

    pub fn load_state(&mut self, json: &str) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else { return };
        let stash = v["stash"].as_array().map(|a| a.iter().filter_map(|e| Some((e[0].as_str()?.to_string(), e[1].as_u64()? as u32))).collect()).unwrap_or_default();
        let d = SaveData {
            stash,
            weapon: v["weapon"].as_str().unwrap_or("").into(),
            shield: v["shield"].as_str().unwrap_or("").into(),
            armor: v["armor"].as_str().map(|s| s.to_string()),
            lantern: v["lantern"].as_f64().unwrap_or(1.0) as f32,
            max_hp: v["max_hp"].as_f64().unwrap_or(100.0) as f32,
            max_stamina: v["max_stamina"].as_f64().unwrap_or(100.0) as f32,
            coins: v["coins"].as_u64().unwrap_or(0) as u32,
            // Gamle lagringer har ikke galleriet: da teller lageret alene.
            found: v["found"].as_array().map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default(),
            bonuses: v["bonuses"].as_array().map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default(),
        };
        self.app.command(Command::Load(d));
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.app.command(Command::SetPaused(paused));
    }

    pub fn set_theme(&mut self, dark: bool) {
        self.app.command(Command::SetDaylight(if dark { Daylight::Night } else { Daylight::Day }));
    }

    /// klart, overskyet, regn, uvær, snø eller tåke.
    pub fn set_weather(&mut self, name: &str) {
        if let Some(k) = WeatherKind::from_name(name) {
            self.app.command(Command::SetWeather(k));
        }
    }

    /// Rett ned i kjelleren (til nettlesertestene).
    pub fn descend(&mut self) {
        self.app.command(Command::Descend);
    }

    /// Tilbake til sørporten, fra knappen i pausemenyen.
    pub fn respawn(&mut self) {
        self.app.command(Command::Respawn);
    }

    /// Flytt kameraet. Brukes av testene og til å lenke til et sted i byen.
    pub fn place(&mut self, x: f32, y: f32, z: f32, yaw_deg: f32, pitch_deg: f32) {
        self.app.command(Command::Place { eye: glam::Vec3::new(x, y, z), yaw: yaw_deg.to_radians(), pitch: pitch_deg.to_radians() });
    }

    /// Grafikknivået: «enkel», «standard» eller «avansert».
    pub fn set_quality(&mut self, name: &str) {
        if let Some(q) = Quality::from_name(name) {
            self.app.command(Command::SetQuality(q));
        }
    }

    /// Bryterne til målingen (`?bench`): MSAA-prøver (0 er nivåets),
    /// alpha to coverage, solskygge og flat farge.
    pub fn set_bench(&mut self, msaa: u32, a2c: bool, shadow: bool, flat: bool, small_hdr: bool, timers: bool) {
        self.app.command(Command::SetBench(Bench { msaa, a2c, shadow, flat, small_hdr, timers }));
    }

    /// Trekanter tegnet sist bilde.
    pub fn triangles(&self) -> u32 {
        self.app.status().triangles
    }

    /// Millisekunder på grafikkortet per del av bildet, som «skygger 1.2 ·
    /// prepass 0.8 · …» (tomt om nettleseren ikke kan måle det).
    pub fn gpu_times(&self) -> String {
        self.app.status().gpu_times.iter().map(|(n, ms)| format!("{n} {ms:.1}")).collect::<Vec<_>>().join(" · ")
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let (w, h) = (width.max(1), height.max(1));
        if w == self.config.width && h == self.config.height {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.app.resize(w, h);
    }

    /// Ett bilde. Returnerer hendelsene som JSON-liste.
    pub fn frame(&mut self, dt: f32) -> String {
        let events = self.app.tick(dt);
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());
                self.app.renderer.set_target(view);
                self.app.draw(self.config.width as f32 / self.config.height as f32);
                frame.present();
            }
            Err(e) => {
                log::warn!("Fikk ikke bilde fra canvas: {e}");
                self.surface.configure(&self.device, &self.config);
            }
        }
        serde_json::Value::Array(events.iter().map(json).collect()).to_string()
    }
}
