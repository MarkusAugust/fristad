//! Fristad i nettleseren: den drivende adapteren for wasm.
//!
//! JavaScript-laget (`web/main.js`) eier løkka, input og panelene, og kaller
//! inn hit. Rust eier byen, fysikken og rendereren. Hendelser går tilbake
//! som JSON fra `frame()`.

use crate::domain::ports::Renderer as _;
use crate::adapters::assets::MemoryAssets;
use crate::adapters::rapier::RapierPhysics;
use crate::adapters::wgpu::Renderer;
use crate::app::{self, App};
use crate::domain::game::Event;
use crate::domain::installations::WeatherKind;
use crate::domain::tokens::Theme;
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
/// Filstiene nettleseren må hente for et manifest, så den kan gi dem til
/// `start`.
#[wasm_bindgen]
pub fn asset_paths(manifest_json: &str) -> Vec<String> {
    MemoryAssets::paths_in(manifest_json)
}

#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement, webgpu: bool, manifest_json: Option<String>, names: js_sys::Array, blobs: js_sys::Array) -> Result<Engine, JsValue> {
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

    let assets = match manifest_json {
        Some(m) => {
            let files: Vec<(String, Vec<u8>)> = names
                .iter()
                .zip(blobs.iter())
                .filter_map(|(n, b)| Some((n.as_string()?, js_sys::Uint8Array::new(&b).to_vec())))
                .collect();
            MemoryAssets::from_files(&m, files)
        }
        None => MemoryAssets::empty(),
    };
    let prepared = app::prepare(&assets, &assets);
    let renderer = Renderer::new(device.clone(), queue, &adapter, format, &prepared.city, &prepared.models, &prepared.layers, config.width, config.height);
    let app = App::new(prepared, renderer, RapierPhysics::new());
    Ok(Engine { app, surface, config, device, backend })
}

fn json(e: &Event) -> serde_json::Value {
    use serde_json::json;
    match e {
        Event::Prompt(Some(p)) => json!({"type": "prompt", "component": p.component, "class": p.class, "variant": p.variant, "variants": p.has_variants, "usable": p.usable, "label": p.label, "inspectable": p.inspectable, "verb": p.verb}),
        Event::Prompt(None) => json!({"type": "prompt"}),
        Event::Inspect { component, variant } => json!({"type": "inspect", "component": component, "variant": variant}),
        Event::Theme(t) => json!({"type": "theme", "theme": if *t == Theme::Dark { "dark" } else { "light" }}),
        Event::Variant { component, variant } => json!({"type": "variant", "component": component, "variant": variant}),
        Event::Toast(t) => json!({"type": "toast", "text": t}),
        Event::Weather(w) => json!({"type": "weather", "weather": w}),
        Event::Hud { hp, stamina, tokens, cellar, keys, boss } => json!({"type": "hud", "hp": hp, "stamina": stamina, "tokens": tokens, "cellar": cellar, "keys": keys, "boss": boss.as_ref().map(|(n, f)| json!({"name": n, "hp": f}))}),
        Event::Sound(name) => json!({"type": "sound", "name": name}),
        Event::Inventory(v) => json!({"type": "inventory", "stash": v.stash, "carried": v.carried, "weapon": v.weapon, "weapon_special": v.weapon_special, "shield": v.shield, "armor": v.armor, "lantern": v.lantern, "max_hp": v.max_hp, "max_stamina": v.max_stamina, "tokens": v.tokens, "read": v.read}),
        Event::Shop { npc, wares } => json!({"type": "shop", "npc": npc, "wares": wares.iter().map(|w| json!({"id": w.id, "name": w.name, "price": w.price})).collect::<Vec<_>>()}),
        Event::Workshop { recipes } => json!({"type": "workshop", "recipes": recipes.iter().map(|r| json!({"id": r.id, "name": r.name, "rarity": r.rarity, "result": r.result, "needs": r.needs, "requires": r.requires, "ok": r.ok})).collect::<Vec<_>>()}),
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

    pub fn fristil_version(&self) -> String {
        self.app.game.fristil_version().to_string()
    }

    pub fn variant(&self, component: &str) -> String {
        self.app.game.variant_of(component)
    }

    pub fn weather(&self) -> String {
        self.app.game.weather().name().to_string()
    }

    pub fn key(&mut self, code: &str, down: bool) {
        self.app.input.key(code, down);
    }

    /// Smi etter en oppskrift (id fra verkstedet). Gir om det gikk.
    pub fn craft(&mut self, recipe: &str) -> bool {
        self.app.game.craft(recipe)
    }

    /// Kjøp en vare (id fra butikken). Gir om det gikk.
    pub fn buy(&mut self, ware: &str) -> bool {
        self.app.game.buy(ware)
    }

    /// Boka: lesing av en komponents dokumentasjon gir liv.
    pub fn read_doc(&mut self, component: &str) -> bool {
        self.app.game.read_doc(component)
    }

    /// Lageret og utstyret som JSON, til localStorage.
    pub fn save_state(&self) -> String {
        let d = self.app.game.save();
        serde_json::json!({"stash": d.stash, "weapon": d.weapon, "shield": d.shield, "armor": d.armor, "lantern": d.lantern, "max_hp": d.max_hp, "max_stamina": d.max_stamina, "coins": d.coins}).to_string()
    }

    pub fn load_state(&mut self, json: &str) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else { return };
        let stash = v["stash"]
            .as_array()
            .map(|a| a.iter().filter_map(|e| Some((e[0].as_str()?.to_string(), e[1].as_u64()? as u32))).collect())
            .unwrap_or_default();
        let d = crate::domain::game::SaveData {
            stash,
            weapon: v["weapon"].as_str().unwrap_or("").into(),
            shield: v["shield"].as_str().unwrap_or("").into(),
            armor: v["armor"].as_str().map(|s| s.to_string()),
            lantern: v["lantern"].as_f64().unwrap_or(1.0) as f32,
            max_hp: v["max_hp"].as_f64().unwrap_or(100.0) as f32,
            max_stamina: v["max_stamina"].as_f64().unwrap_or(100.0) as f32,
            coins: v["coins"].as_u64().unwrap_or(0) as u32,
        };
        self.app.game.load(d);
    }

    /// Musetast: 0 venstre (hugg), 2 høyre (blokk).
    pub fn mouse(&mut self, button: u32, down: bool) {
        self.app.input.mouse(button, down);
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.app.input.look.0 += dx;
        self.app.input.look.1 += dy;
    }

    pub fn release_all(&mut self) {
        self.app.input.release_all();
    }

    /// Et tegn (`KeyboardEvent.key`) til feltet som har fokus.
    pub fn text(&mut self, key: &str) {
        self.app.input.text(key);
    }

    /// Om et skjemafelt har tastaturfokus: da skal tastene skrives, ikke styre.
    pub fn editing(&self) -> bool {
        self.app.game.editing()
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.app.game.paused = paused;
        if paused {
            self.app.input.release_all();
        }
    }

    pub fn set_theme(&mut self, dark: bool) {
        self.app.game.set_theme(if dark { Theme::Dark } else { Theme::Light }, false);
    }

    /// Bruk en komponent (som F), til testene.
    pub fn use_component(&mut self, component: &str) {
        self.app.game.use_component(component);
    }

    pub fn set_variant(&mut self, component: &str, variant: &str) {
        self.app.game.set_variant(component, variant);
    }

    /// klart, overskyet, regn, uvær, snø eller tåke.
    pub fn set_weather(&mut self, name: &str) {
        if let Some(k) = WeatherKind::ALL.iter().copied().find(|k| k.name() == name) {
            self.app.game.set_weather(k, false);
        }
    }

    /// Forbindelsen til serveren, fra `navigator.onLine`.
    pub fn set_online(&mut self, online: bool) {
        self.app.game.set_online(online);
    }

    /// Tilbake til torget, fra knappen i pausemenyen.
    pub fn respawn(&mut self) {
        self.app.game.respawn();
    }

    /// Flytt kameraet. Brukes av testene og til å lenke til et sted i byen.
    pub fn place(&mut self, x: f32, y: f32, z: f32, yaw_deg: f32, pitch_deg: f32) {
        self.app.game.place(glam::Vec3::new(x, y, z), yaw_deg.to_radians(), pitch_deg.to_radians());
    }

    /// Intern oppløsning som andel av vinduet, 0.4–1.0.
    /// Kvalitet 0–1 (skygger, SSAO, tåke).
    pub fn set_quality(&mut self, quality: f32) {
        self.app.renderer.set_quality(quality);
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.app.renderer.set_scale(scale);
    }

    /// Trekanter tegnet sist bilde.
    pub fn triangles(&self) -> u32 {
        self.app.renderer.triangles()
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
