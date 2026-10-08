//! Renderer et bilde av Fristad til en PNG, uten vindu: en drivende adapter
//! som snakker med `ports::driving::Play`.
//!
//!   cargo run --release --bin snapshot -- --out torg.png --night
//!
//! Valg: --out FIL, --size BxH, --night, --eye x,y,z (y med + foran er
//! høyde over terrenget), --yaw grader, --pitch grader, --stance huk|kryp,
//! --weather klart|overskyet|regn|uvær|snø|tåke, --time sek, --descend
//! (gå ned i krypten først), --chest (åpne den første kista), --attack S [--hold], --block, --frames N,
//! --quality Q, --scale S, --debug albedo|normal|ao|rough|shadow,
//! --dump-layer N.

use fristad::adapters::gltf::Embedded;
use fristad::adapters::native::seed;
use fristad::adapters::rapier::RapierPhysics;
use fristad::adapters::wgpu::Renderer;
use fristad::application::{prepare, App};
use fristad::domain::weather::{Daylight, WeatherKind};
use fristad::domain::world::Use;
use fristad::ports::driving::{Command, Play};
use fristad::ports::Renderer as _;
use glam::Vec3;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let flag = |name: &str| args.iter().any(|a| a == name);

    let out = arg("--out").unwrap_or_else(|| "snapshot.png".into());
    let (width, height) = arg("--size").and_then(|s| s.split_once('x').map(|(w, h)| (w.parse().unwrap(), h.parse().unwrap()))).unwrap_or((1280u32, 720u32));

    let prepared = prepare(&Embedded, seed());
    let chest = prepared.world.dungeon.chests.first().copied();
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
    let (adapter, device, queue) = pollster::block_on(fristad::adapters::gpu(&instance, None)).expect("GPU");
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let renderer = Renderer::new(device.clone(), queue.clone(), &adapter, format, &App::<Renderer, RapierPhysics>::scene(&prepared), &fristad::adapters::layers::from_dir(std::path::Path::new(&arg("--assets").unwrap_or_else(|| "assets/external".into()))), width, height);
    let mut app = App::new(prepared, renderer, RapierPhysics::new());

    if flag("--night") || arg("--theme").as_deref() == Some("dark") {
        app.session.set_daylight(Daylight::Night, true);
    }
    if let Some(w) = arg("--weather") {
        app.session.set_weather(WeatherKind::from_name(&w).unwrap_or_default(), true);
    }
    if flag("--descend") {
        app.session.use_first(|u| *u == Use::Descend);
    }
    // --chest: åpne den første kista i kjelleren og se på den forfra (--time
    // styrer hvor langt lokket har kommet).
    if flag("--chest") {
        if let Some((p, yaw, _)) = chest {
            let front = Vec3::new(yaw.sin(), 0.0, yaw.cos());
            let eye = p + front * 1.8 + Vec3::Y * 1.5;
            app.command(Command::Place { eye, yaw: (front.x).atan2(front.z), pitch: -0.45 });
            app.session.use_first(|u| matches!(u, Use::Chest(0)));
        }
    }
    // --well: heis opp vann i den første brønnen (--time styrer hvor langt).
    if flag("--well") {
        app.session.use_first(|u| matches!(u, Use::Well(_)));
    }
    if let Some(st) = arg("--stance") {
        match st.as_str() {
            "huk" => app.input().key("KeyC", true),
            "kryp" => app.input().key("KeyX", true),
            _ => {}
        }
    }
    if let Some(d) = arg("--debug") {
        app.session.debug = match d.as_str() {
            "albedo" => 1,
            "normal" => 2,
            "ao" => 3,
            "rough" => 4,
            "shadow" => 5,
            _ => d.parse().unwrap_or(0),
        };
    }
    // --eye X,Y,Z; Y med pluss foran («+1.7») er høyde over terrenget.
    let eye = arg("--eye").map(|s| {
        let parts: Vec<&str> = s.split(',').collect();
        let (x, z): (f32, f32) = (parts[0].parse().unwrap(), parts[2].parse().unwrap());
        let y: f32 = parts[1].trim_start_matches('+').parse().unwrap();
        let y = if parts[1].starts_with('+') { y + fristad::domain::world::terrain::height(x, z) } else { y };
        Vec3::new(x, y, z)
    });
    let yaw: f32 = arg("--yaw").map(|s| s.parse().unwrap()).unwrap_or(0.0);
    let pitch: f32 = arg("--pitch").map(|s| s.parse().unwrap()).unwrap_or(4.0);
    if let Some(eye) = eye {
        app.command(Command::Place { eye, yaw: yaw.to_radians(), pitch: pitch.to_radians() });
    }

    // Kjør spillet litt, så animasjoner og fiender står der de skal.
    let time: f32 = arg("--time").map(|s| s.parse().unwrap()).unwrap_or(0.5);
    let steps = (time * 30.0) as usize;
    // --attack S: venstre musetast trykkes S sekunder før bildet (og holdes
    // med --hold), --block holder høyre musetast de siste 0,5 sekundene.
    let attack_at = arg("--attack").map(|s| steps.saturating_sub((s.parse::<f32>().unwrap() * 30.0) as usize));
    let hold = flag("--hold");
    for step in 0..steps.max(1) {
        if attack_at == Some(step) {
            app.input().mouse(0, true);
            if !hold {
                app.input().mouse(0, false);
            }
        }
        if flag("--block") && step + 15 >= steps {
            app.input().mouse(2, true);
        }
        app.tick(1.0 / 30.0);
    }
    if let Some(eye) = eye {
        app.command(Command::Place { eye, yaw: yaw.to_radians(), pitch: pitch.to_radians() });
    }

    if let Some(l) = arg("--dump-layer") {
        let layer: u32 = l.parse().unwrap();
        for (nrh, suffix) in [(false, "albedo"), (true, "nrh")] {
            let (size, data) = app.renderer.dump_layer(layer, nrh);
            let path = format!("{}-{suffix}.png", out.trim_end_matches(".png"));
            write_png(&path, size, size, &data);
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
        app.command(Command::SetScale(scale.parse().unwrap()));
    }
    if let Some(q) = arg("--quality") {
        app.command(Command::SetQuality(q.parse().unwrap()));
    }
    for _ in 0..frames.max(1) {
        app.renderer.set_target(target.create_view(&Default::default()));
        app.draw(width as f32 / height as f32);
    }

    let row = (width * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("readback"), size: (row * height) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
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

    let data = buf.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let at = (y * row) as usize;
        pixels.extend_from_slice(&data[at..at + (width * 4) as usize]);
    }
    write_png(&out, width, height, &pixels);
    println!("{out}");
}

fn write_png(path: &str, width: u32, height: u32, rgba: &[u8]) {
    let file = std::fs::File::create(path).expect("kan ikke skrive fila");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(rgba).unwrap();
}
