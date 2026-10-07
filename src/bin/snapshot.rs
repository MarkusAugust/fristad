//! Renderer et bilde av Fristad til en PNG, uten vindu.
//!
//! Brukes til å se på byen uten skjerm, i CI og i utvikling:
//!
//!   cargo run --release --bin snapshot -- --out torg.png --theme dark
//!
//! Valg: --out FIL, --size BxH, --theme light|dark, --eye x,y,z,
//! --yaw grader, --pitch grader, --gate, --use komponent, --stance huk|kryp, --variant komponent=navn,
//! --weather klart|overskyet|regn|uvær|snø|tåke, --time sek,
//! --frames N (TAA-bilder som blandes, standard 12), --lights off.

use fristad::adapters::assets::MemoryAssets;
use fristad::adapters::rapier::RapierPhysics;
use fristad::adapters::wgpu::Renderer;
use fristad::app::{self, App};
use fristad::domain::installations::WeatherKind;
use fristad::domain::ports::Renderer as _;
use fristad::domain::tokens::Theme;
use glam::Vec3;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let flag = |name: &str| args.iter().any(|a| a == name);

    let out = arg("--out").unwrap_or_else(|| "snapshot.png".into());
    let (width, height) = arg("--size")
        .and_then(|s| s.split_once('x').map(|(w, h)| (w.parse().unwrap(), h.parse().unwrap())))
        .unwrap_or((1280u32, 720u32));
    let theme = if arg("--theme").as_deref() == Some("dark") { Theme::Dark } else { Theme::Light };

    // Eksterne ressurser fra assets/external/ (eller --assets KATALOG), om de finnes.
    let assets = MemoryAssets::from_dir(std::path::Path::new(&arg("--assets").unwrap_or_else(|| "assets/external".into())));
    let prepared = app::prepare(&assets, &assets);
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
    let (adapter, device, queue) = pollster::block_on(fristad::adapters::gpu(&instance, None)).expect("GPU");
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let renderer = Renderer::new(device.clone(), queue.clone(), &adapter, format, &prepared.city, &prepared.models, &prepared.layers, width, height);
    let mut app = App::new(prepared, renderer, RapierPhysics::new());

    app.game.set_theme(theme, true);
    if let Some(w) = arg("--weather") {
        let kind = WeatherKind::ALL.iter().copied().find(|k| k.name() == w).unwrap_or(WeatherKind::Clear);
        app.game.set_weather(kind, true);
    }
    // --variant kan gis flere ganger: én per komponent.
    for (i, a) in args.iter().enumerate() {
        if a != "--variant" {
            continue;
        }
        let Some(v) = args.get(i + 1) else { continue };
        if let Some((c, n)) = v.split_once('=') {
            app.game.set_variant(c, n);
        } else {
            app.game.set_variant("button", v);
        }
    }
    if flag("--gate") {
        app.game.set_gate(true, true);
    }
    // --use komponent og --type tekst i den rekkefølgen de står: F på
    // komponenten (navn#n velger den n-te), og skriving i feltet som har fokus.
    for (i, a) in args.iter().enumerate() {
        match a.as_str() {
            "--use" => app.game.use_component(args.get(i + 1).map(String::as_str).unwrap_or("")),
            "--type" => app.game.type_text(args.get(i + 1).map(String::as_str).unwrap_or("")),
            _ => {}
        }
        if a == "--use" || a == "--type" {
            // Handlingene virker først etter et steg.
            app.tick(1.0 / 30.0);
        }
    }
    // --fly X,Z: start en flytur dit fra --eye, og la --time bestemme hvor
    // langt turen er kommet når bildet tas.
    if let Some(f) = arg("--fly") {
        let v: Vec<f32> = f.split(',').map(|s| s.parse().unwrap()).collect();
        app.game.fly_to(glam::Vec3::new(v[0], 0.0, v[1]), 0.0);
    }
    if let Some(st) = arg("--stance") {
        match st.as_str() {
            "huk" => app.input.key("KeyC", true),
            "kryp" => app.input.key("KeyX", true),
            _ => {}
        }
    }
    if let Some(d) = arg("--debug") {
        app.game.debug = match d.as_str() {
            "albedo" => 1,
            "normal" => 2,
            "ao" => 3,
            "rough" => 4,
            "shadow" => 5,
            _ => d.parse().unwrap_or(0),
        };
    }
    if arg("--lights").as_deref() == Some("off") {
        app.game.set_online(true);
    }
    // --eye X,Y,Z; Y med pluss foran («+1.7») er høyde over terrenget.
    let eye = arg("--eye")
        .map(|s| {
            let parts: Vec<&str> = s.split(',').collect();
            let (x, z): (f32, f32) = (parts[0].parse().unwrap(), parts[2].parse().unwrap());
            let y: f32 = parts[1].trim_start_matches('+').parse().unwrap();
            let y = if parts[1].starts_with('+') { y + fristad::domain::city::terrain::height(x, z) } else { y };
            Vec3::new(x, y, z)
        })
        .unwrap_or(Vec3::new(0.0, 1.7, 11.0));
    let yaw: f32 = arg("--yaw").map(|s| s.parse().unwrap()).unwrap_or(0.0);
    let pitch: f32 = arg("--pitch").map(|s| s.parse().unwrap()).unwrap_or(4.0);
    app.game.place(eye, yaw.to_radians(), pitch.to_radians());

    // Kjør spillet litt, så animasjoner og markering står der de skal.
    let time: f32 = arg("--time").map(|s| s.parse().unwrap()).unwrap_or(0.5);
    let steps = (time * 30.0) as usize;
    for _ in 0..steps.max(1) {
        app.tick(1.0 / 30.0);
    }
    // Med --fly beholdes stedet spillet har flyttet deg til; bare kameraet snus.
    if arg("--fly").is_some() {
        app.game.set_look(yaw.to_radians(), pitch.to_radians());
    } else {
        app.game.place(eye, yaw.to_radians(), pitch.to_radians());
    }

    if let Some(l) = arg("--dump-layer") {
        let layer: u32 = l.parse().unwrap();
        for (nrh, suffix) in [(false, "albedo"), (true, "nrh")] {
            let (size, data) = app.renderer.dump_layer(layer, nrh);
            let path = format!("{}-{suffix}.png", out.trim_end_matches(".png"));
            let file = std::fs::File::create(&path).expect("kan ikke skrive fila");
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), size, size);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&data).unwrap();
            println!("{path}");
        }
        return;
    }

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("snapshot"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let start = std::time::Instant::now();
    // Flere bilder med TAA gir et supersamplet, rolig bilde.
    let frames: usize = arg("--frames").map(|s| s.parse().unwrap()).unwrap_or(12);
    if let Some(scale) = arg("--scale") {
        app.renderer.set_scale(scale.parse().unwrap());
    }
    if let Some(q) = arg("--quality") {
        app.renderer.set_quality(q.parse().unwrap());
    }
    for _ in 0..frames.max(1) {
        app.renderer.set_target(target.create_view(&Default::default()));
        app.draw(width as f32 / height as f32);
    }

    let row = (width * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(height) } },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait).expect("poll");
    log::info!("Rendret {frames} bilder på {:.0} ms, {} trekanter i hovedpasset", start.elapsed().as_secs_f32() * 1000.0, app.renderer.triangles());
    let (w, p, m) = app.renderer.stats();
    log::info!("Verden {w}, props {p}, modeller {m} trekanter");

    let data = buf.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let at = (y * row) as usize;
        pixels.extend_from_slice(&data[at..at + (width * 4) as usize]);
    }
    let file = std::fs::File::create(&out).expect("kan ikke skrive fila");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&pixels).unwrap();
    println!("{out}");
    let _ = app.renderer.size();
}
