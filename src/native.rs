//! Fristad som eget vindu på Mac, Windows og Linux.
//!
//! Klikk i vinduet for å fange musa, Esc for å slippe den.

use crate::game::{Event, Game, Input};
use crate::render::Renderer;
use crate::tokens::Tokens;
use crate::world;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

struct Gpu {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
}

struct App {
    game: Game,
    world: world::World,
    gpu: Option<Gpu>,
    input: Input,
    last: Instant,
    grabbed: bool,
}

impl App {
    fn grab(&mut self, on: bool) {
        let Some(g) = &self.gpu else { return };
        if on {
            let ok = g.window.set_cursor_grab(CursorGrabMode::Locked).is_ok()
                || g.window.set_cursor_grab(CursorGrabMode::Confined).is_ok();
            if ok {
                g.window.set_cursor_visible(false);
                self.grabbed = true;
            }
        } else {
            let _ = g.window.set_cursor_grab(CursorGrabMode::None);
            g.window.set_cursor_visible(true);
            self.grabbed = false;
            self.input.release_all();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let window = Arc::new(
            el.create_window(Window::default_attributes().with_title("Fristad").with_inner_size(winit::dpi::LogicalSize::new(1280, 760)))
                .expect("vindu"),
        );
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).expect("surface");
        let (adapter, device, queue) = pollster::block_on(crate::gpu(&instance, Some(&surface))).expect("GPU");
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
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
        let renderer = Renderer::new(device, queue, &adapter, format, &self.world, config.width, config.height);
        self.gpu = Some(Gpu { window, surface, config, renderer });
        self.last = Instant::now();
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.grabbed {
                self.input.look.0 += delta.0 as f32;
                self.input.look.1 += delta.1 as f32;
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => {
                if let Some(g) = &mut self.gpu {
                    g.config.width = size.width.max(1);
                    g.config.height = size.height.max(1);
                    g.surface.configure(g.renderer.device(), &g.config);
                    g.renderer.resize(g.config.width, g.config.height);
                }
            }
            WindowEvent::Focused(false) => self.grab(false),
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if !self.grabbed {
                    self.grab(true);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let name = format!("{code:?}");
                    if name == "Escape" && event.state == ElementState::Pressed {
                        self.grab(false);
                        self.game.paused = false;
                    } else if !event.repeat || event.state == ElementState::Released {
                        self.input.key(&name, event.state == ElementState::Pressed);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last).as_secs_f32();
                self.last = now;
                for e in self.game.update(&mut self.input, dt) {
                    match e {
                        Event::Prompt(Some(p)) => {
                            if let Some(g) = &self.gpu {
                                g.window.set_title(&format!("Fristad · {} ({}) · E inspiser · R variant · F trykk", p.class, p.variant));
                            }
                        }
                        Event::Prompt(None) => {
                            if let Some(g) = &self.gpu {
                                g.window.set_title("Fristad");
                            }
                        }
                        Event::Inspect { component, variant } => {
                            log::info!("Inspiser {component} ({variant}). Panelet finnes i nettleserversjonen.");
                        }
                        other => log::info!("{other:?}"),
                    }
                }
                let Some(g) = &mut self.gpu else { return };
                let frame = match g.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(_) => {
                        g.surface.configure(g.renderer.device(), &g.config);
                        return;
                    }
                };
                let view = frame.texture.create_view(&Default::default());
                let data = self.game.frame(g.config.width as f32 / g.config.height as f32);
                g.renderer.render(&view, &data);
                g.window.pre_present_notify();
                frame.present();
                g.window.request_redraw();
            }
            _ => {}
        }
    }
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let tokens = Tokens::load();
    let world = world::build(&tokens);
    let game = Game::new(tokens, &world);
    let el = EventLoop::new().expect("event loop");
    let mut app = App { game, world, gpu: None, input: Input::default(), last: Instant::now(), grabbed: false };
    el.run_app(&mut app).expect("kjøring");
}
