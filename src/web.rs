//! Fristad i nettleseren.
//!
//! JavaScript-laget (`web/main.js`) eier løkka, input og panelene, og kaller
//! inn hit. Rust eier verden, fysikken og rendereren. Hendelser går tilbake
//! som JSON fra `frame()`.

use crate::game::{Event, Game, Input};
use crate::render::Renderer;
use crate::tokens::{Theme, Tokens};
use crate::world;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn boot() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
}

#[wasm_bindgen]
pub struct Engine {
    game: Game,
    input: Input,
    renderer: Renderer,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    backend: String,
}

async fn connect(
    canvas: &web_sys::HtmlCanvasElement,
    webgpu: bool,
) -> Result<(wgpu::Surface<'static>, wgpu::Adapter, wgpu::Device, wgpu::Queue), String> {
    let instance = if webgpu {
        wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        })
        .await
    } else {
        wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, ..Default::default() })
    };
    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .map_err(|e| format!("Kunne ikke tegne i canvas: {e}"))?;
    let (adapter, device, queue) = crate::gpu(&instance, Some(&surface)).await?;
    Ok((surface, adapter, device, queue))
}

/// Start motoren i et canvas, med WebGPU eller WebGL2.
///
/// Et canvas som har gitt ut en WebGPU-kontekst kan ikke gi WebGL etterpå,
/// så JavaScript-laget prøver WebGPU først og gir oss et nytt canvas hvis
/// det må falle tilbake.
#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement, webgpu: bool) -> Result<Engine, JsValue> {
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

    let tokens = Tokens::load();
    let world = world::build(&tokens);
    let renderer = Renderer::new(device, queue, &adapter, format, &world, config.width, config.height);
    let game = Game::new(tokens, &world);
    Ok(Engine { game, input: Input::default(), renderer, surface, config, backend })
}

fn json(e: &Event) -> serde_json::Value {
    use serde_json::json;
    match e {
        Event::Prompt(Some(p)) => json!({"type": "prompt", "component": p.component, "class": p.class, "variant": p.variant}),
        Event::Prompt(None) => json!({"type": "prompt"}),
        Event::Inspect { component, variant } => json!({"type": "inspect", "component": component, "variant": variant}),
        Event::Theme(t) => json!({"type": "theme", "theme": if *t == Theme::Dark { "dark" } else { "light" }}),
        Event::Variant { component, variant } => json!({"type": "variant", "component": component, "variant": variant}),
    }
}

#[wasm_bindgen]
impl Engine {
    /// "BrowserWebGpu" eller "Gl".
    pub fn backend(&self) -> String {
        self.backend.clone()
    }

    pub fn fristil_version(&self) -> String {
        self.game.fristil_version().to_string()
    }

    pub fn variant(&self) -> String {
        self.game.variant().to_string()
    }

    pub fn key(&mut self, code: &str, down: bool) {
        self.input.key(code, down);
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.input.look.0 += dx;
        self.input.look.1 += dy;
    }

    pub fn release_all(&mut self) {
        self.input.release_all();
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.game.paused = paused;
        if paused {
            self.input.release_all();
        }
    }

    pub fn set_theme(&mut self, dark: bool) {
        self.game.set_theme(if dark { Theme::Dark } else { Theme::Light }, false);
    }

    pub fn set_variant(&mut self, variant: &str) {
        self.game.set_variant(variant);
    }

    /// Flytt kameraet. Brukes av testene og til å lenke til et sted i byen.
    pub fn place(&mut self, x: f32, y: f32, z: f32, yaw_deg: f32, pitch_deg: f32) {
        self.game.place(glam::Vec3::new(x, y, z), yaw_deg.to_radians(), pitch_deg.to_radians());
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let (w, h) = (width.max(1), height.max(1));
        if w == self.config.width && h == self.config.height {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(self.renderer.device(), &self.config);
        self.renderer.resize(w, h);
    }

    /// Ett bilde. Returnerer hendelsene som JSON-liste.
    pub fn frame(&mut self, dt: f32) -> String {
        let events = self.game.update(&mut self.input, dt);
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());
                let data = self.game.frame(self.config.width as f32 / self.config.height as f32);
                self.renderer.render(&view, &data);
                frame.present();
            }
            Err(e) => {
                log::warn!("Fikk ikke bilde fra canvas: {e}");
                self.surface.configure(self.renderer.device(), &self.config);
            }
        }
        serde_json::Value::Array(events.iter().map(json).collect()).to_string()
    }
}
