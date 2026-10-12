//! Fristad som eget vindu på Mac, Windows og Linux: den drivende adapteren
//! for winit. Klikk i vinduet for å fange musa, Esc for å slippe den.

use crate::adapters::gltf::Embedded;
use crate::adapters::rapier::RapierPhysics;
use crate::adapters::wgpu::Renderer;
use crate::application::{prepare, App, Prepared};
use crate::ports::driving::{Event, Play};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

struct Running {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    app: App<Renderer, RapierPhysics>,
}

struct Host {
    prepared: Option<Prepared>,
    running: Option<Running>,
    last: Instant,
    grabbed: bool,
}

impl Host {
    fn grab(&mut self, on: bool) {
        let Some(r) = &mut self.running else { return };
        if on {
            let ok = r.window.set_cursor_grab(CursorGrabMode::Locked).is_ok()
                || r.window.set_cursor_grab(CursorGrabMode::Confined).is_ok();
            if ok {
                r.window.set_cursor_visible(false);
                self.grabbed = true;
            }
        } else {
            let _ = r.window.set_cursor_grab(CursorGrabMode::None);
            r.window.set_cursor_visible(true);
            self.grabbed = false;
            r.app.input().release_all();
        }
    }
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("Fristad")
                    .with_inner_size(winit::dpi::LogicalSize::new(1280, 760)),
            )
            .expect("vindu"),
        );
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).expect("surface");
        let (adapter, device, queue) =
            pollster::block_on(crate::adapters::gpu(&instance, Some(&surface))).expect("GPU");
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let prepared = self.prepared.take().expect("verden er allerede brukt");
        let renderer = Renderer::new(
            device.clone(),
            queue,
            &adapter,
            format,
            &App::<Renderer, RapierPhysics>::scene(&prepared),
            &crate::adapters::layers::from_dir(std::path::Path::new("assets/external")),
            config.width,
            config.height,
        );
        let app = App::new(prepared, renderer, RapierPhysics::new());
        self.running = Some(Running {
            window,
            surface,
            config,
            device,
            app,
        });
        self.last = Instant::now();
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.grabbed {
                if let Some(r) = &mut self.running {
                    r.app.input().look.0 += delta.0 as f32;
                    r.app.input().look.1 += delta.1 as f32;
                }
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.running {
                    r.config.width = size.width.max(1);
                    r.config.height = size.height.max(1);
                    r.surface.configure(&r.device, &r.config);
                    r.app.resize(r.config.width, r.config.height);
                }
            }
            WindowEvent::Focused(false) => self.grab(false),
            WindowEvent::MouseInput { state, button, .. } => {
                if !self.grabbed {
                    if state == ElementState::Pressed && button == MouseButton::Left {
                        self.grab(true);
                    }
                } else if let Some(r) = &mut self.running {
                    let b = match button {
                        MouseButton::Left => 0,
                        MouseButton::Right => 2,
                        _ => 1,
                    };
                    r.app.input().mouse(b, state == ElementState::Pressed);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let name = format!("{code:?}");
                    if name == "Escape" && event.state == ElementState::Pressed {
                        self.grab(false);
                    } else if let Some(r) = &mut self.running {
                        if !event.repeat || event.state == ElementState::Released {
                            r.app
                                .input()
                                .key(&name, event.state == ElementState::Pressed);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last).as_secs_f32();
                self.last = now;
                let Some(r) = &mut self.running else { return };
                for e in r.app.tick(dt) {
                    match e {
                        Event::Prompt(Some(p)) => r.window.set_title(&format!(
                            "Fristad · {} · F {} · E {}",
                            p.label, p.verb, p.trade
                        )),
                        Event::Prompt(None) => r.window.set_title("Fristad"),
                        Event::Marks(_) | Event::Speech(_) | Event::Ambience(_) => {}
                        other => log::info!("{other:?}"),
                    }
                }
                let frame = match r.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(_) => {
                        r.surface.configure(&r.device, &r.config);
                        return;
                    }
                };
                r.app
                    .renderer
                    .set_target(frame.texture.create_view(&Default::default()));
                r.app.draw(r.config.width as f32 / r.config.height as f32);
                r.window.pre_present_notify();
                frame.present();
                r.window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Frøet til kjelleren: `FRISTAD_SEED`, ellers 7.
pub fn seed() -> u64 {
    std::env::var("FRISTAD_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(7)
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let prepared = prepare(&Embedded, seed());
    let el = EventLoop::new().expect("event loop");
    let mut host = Host {
        prepared: Some(prepared),
        running: None,
        last: Instant::now(),
        grabbed: false,
    };
    el.run_app(&mut host).expect("kjøring");
}
