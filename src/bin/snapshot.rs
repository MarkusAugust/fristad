//! Renderer et bilde av Fristad til en PNG, uten vindu.
//!
//! Brukes til å se på byen uten skjerm, i CI og i utvikling:
//!
//!   cargo run --release --bin snapshot -- --out torg.png --theme dark
//!
//! Valg: --out FIL, --size BxH, --theme light|dark, --eye x,y,z,
//! --yaw grader, --pitch grader, --gate open, --variant navn, --time sek,
//! --aim (marker knappen som om du sikter på den).

use fristad::{game::Game, render::Renderer, tokens::{Theme, Tokens}, world};
use glam::Vec3;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let flag = |name: &str| args.iter().any(|a| a == name);

    let out = arg("--out").unwrap_or_else(|| "snapshot.png".into());
    let (width, height) = arg("--size")
        .and_then(|s| s.split_once('x').map(|(w, h)| (w.parse().unwrap(), h.parse().unwrap())))
        .unwrap_or((1280u32, 720u32));
    let theme = if arg("--theme").as_deref() == Some("dark") { Theme::Dark } else { Theme::Light };

    let tokens = Tokens::load();
    let world = world::build(&tokens);
    let mut game = Game::new(tokens, &world);
    game.set_theme(theme, true);
    if let Some(v) = arg("--variant") {
        game.set_variant(&v);
    }
    if flag("--gate") {
        game.set_gate(true, true);
    }
    let eye = arg("--eye")
        .map(|s| {
            let v: Vec<f32> = s.split(',').map(|x| x.parse().unwrap()).collect();
            Vec3::new(v[0], v[1], v[2])
        })
        .unwrap_or(Vec3::new(0.0, 1.7, 11.0));
    let yaw: f32 = arg("--yaw").map(|s| s.parse().unwrap()).unwrap_or(0.0);
    let pitch: f32 = arg("--pitch").map(|s| s.parse().unwrap()).unwrap_or(4.0);
    game.place(eye, yaw.to_radians(), pitch.to_radians());

    // Kjør spillet litt, så animasjoner og markering står der de skal.
    let mut input = fristad::game::Input::default();
    let time: f32 = arg("--time").map(|s| s.parse().unwrap()).unwrap_or(0.5);
    let steps = (time * 30.0) as usize;
    for _ in 0..steps.max(1) {
        game.update(&mut input, 1.0 / 30.0);
    }
    game.place(eye, yaw.to_radians(), pitch.to_radians());

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
    let (adapter, device, queue) = pollster::block_on(fristad::gpu(&instance, None)).expect("GPU");
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = Renderer::new(device.clone(), queue.clone(), &adapter, format, &world, width, height);

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
    let view = target.create_view(&Default::default());
    let frame = game.frame(width as f32 / height as f32);
    let start = std::time::Instant::now();
    renderer.render(&view, &frame);

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
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(height) },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait).expect("poll");
    log::info!("Rendret på {:.0} ms", start.elapsed().as_secs_f32() * 1000.0);

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
}
