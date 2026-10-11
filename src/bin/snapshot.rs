//! Renderer et bilde av Fristad til en PNG, uten vindu: en drivende adapter
//! som snakker med `ports::driving::Play`.
//!
//!   cargo run --release --bin snapshot -- --out torg.png --night
//!
//! Valg: --out FIL, --size BxH, --night, --eye x,y,z (y med + foran er
//! høyde over terrenget), --yaw grader, --pitch grader, --stance huk|kryp,
//! --weather klart|overskyet|regn|uvær|snø|tåke, --time sek, --descend
//! (ned i krypten først: startstedet i nivå 1), --level N (startstedet i
//! nivå N; med --eye er koordinatene i nivåets eget rom), --portal (ned og hjem med H, så portalen åpner seg), --chest (åpne den første kista), --attack S [--hold], --block, --frames N,
//! --at MODELL X,Y,Z YAW (kamera i modellens rom), --gallery (full gallerivegg), --debug albedo|normal|ao|rough|shadow,
//! --dump-layer N.

use fristad::adapters::gltf::Embedded;
use fristad::adapters::native::seed;
use fristad::adapters::rapier::RapierPhysics;
use fristad::adapters::wgpu::Renderer;
use fristad::application::{prepare, App};
use fristad::domain::weather::{Daylight, WeatherKind};
use fristad::domain::world::Use;
use fristad::ports::driving::{Command, Play};
use fristad::ports::Quality;
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
    // --at MODELL X,Y,Z YAW: kameraet i modellens eget rom (første instans;
    // Blenders akser, −y er fram), yaw i grader der 0 ser mot huset forfra.
    let model_at = args.iter().position(|a| a == "--at").map(|i| {
        let model = args[i + 1].clone();
        let v: Vec<f32> = args[i + 2].split(',').map(|x| x.parse().unwrap()).collect();
        let yaw: f32 = args.get(i + 3).and_then(|y| y.parse().ok()).unwrap_or(0.0);
        let t = prepared.world.placements.iter().find(|pl| pl.model == model).map(|pl| pl.transforms[0]).expect("modellen finnes ikke i byen");
        let eye = t.transform_point3(Vec3::new(v[0], v[2], -v[1]));
        // Retningen mot modellens midte, dreid `yaw` grader.
        let to = t.transform_point3(Vec3::new(0.0, v[2], 0.0)) - eye;
        (eye, (-to.x).atan2(-to.z) + yaw.to_radians())
    });
    let chest = prepared.world.crypt.chests.first().map(|c| (c.pos, c.yaw));
    let level: Option<usize> = arg("--level").map(|s| s.parse::<usize>().expect("--level N").max(1) - 1);
    let level_origin = level.and_then(|k| prepared.world.crypt.levels.get(k)).map(|l| l.origin);
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
    if flag("--descend") || level.is_some() {
        app.session.use_first(|u| *u == Use::Descend);
    }
    if let Some(k) = level {
        assert!(app.session.go_to_level(k), "nivå {} finnes ikke", k + 1);
    }
    // --chest: åpne den første kista i kjelleren og se på den forfra (--time
    // styrer hvor langt lokket har kommet).
    if flag("--chest") {
        if let Some((p, yaw)) = chest {
            let front = Vec3::new(yaw.sin(), 0.0, yaw.cos());
            let eye = p + front * 1.8 + Vec3::Y * 1.5;
            app.command(Command::Place { eye, yaw: (front.x).atan2(front.z), pitch: -0.45 });
            app.session.use_first(|u| matches!(u, Use::Chest(0)));
        }
    }
    // --portal: ned i kjelleren, og hjem igjen med H, så portalen ved
    // sørporten våkner (--time styrer hvor langt den har åpnet seg).
    if flag("--portal") {
        app.session.use_first(|u| *u == Use::Descend);
        for _ in 0..20 {
            app.tick(1.0 / 30.0);
        }
        app.command(Command::Respawn);
    }
    // --gallery: alle komponentene er funnet, så galleriveggen er full.
    if flag("--gallery") {
        for c in fristad::domain::craft::GALLERY {
            app.session.give(c, 1);
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
        // I et kryptnivå (--level) er koordinatene i nivåets eget rom.
        if let Some(o) = level_origin {
            return Vec3::new(x, y, z) + o;
        }
        let y = if parts[1].starts_with('+') { y + fristad::domain::world::terrain::height(x, z) } else { y };
        Vec3::new(x, y, z)
    });
    let eye = eye.or(model_at.map(|(e, _)| e));
    let yaw: f32 = model_at.map(|(_, y)| y.to_degrees()).or(arg("--yaw").map(|s| s.parse().unwrap())).unwrap_or(0.0);
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
    // Noen bilder, så animasjonene og lysene har satt seg.
    let frames: usize = arg("--frames").map(|s| s.parse().unwrap()).unwrap_or(4);
    // Standard grafikk er det spillet starter med; --advanced slår på avansert.
    app.command(Command::SetQuality(if flag("--advanced") { Quality::Advanced } else if flag("--simple") { Quality::Simple } else { Quality::Standard }));
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
    let times = app.renderer.pass_times();
    if !times.is_empty() {
        log::info!("GPU-tid per del: {}", times.iter().map(|(n, ms)| format!("{n} {ms:.2} ms")).collect::<Vec<_>>().join(", "));
    }

    // Hva hvert pass tegnet i siste bilde, til før/etter-tabellene.
    if flag("--stats") {
        let st = app.renderer.pass_stats();
        let mut total = fristad::adapters::wgpu::PassStats::default();
        println!("{:<9} {:>11} {:>11} {:>7} {:>9}", "pass", "trek. verden", "trek. modell", "kall", "instanser");
        for (name, p) in fristad::adapters::wgpu::PASSES.iter().zip(st) {
            println!("{name:<9} {:>11} {:>11} {:>7} {:>9}", p.world, p.models, p.calls, p.instances);
            total.world += p.world;
            total.models += p.models;
            total.calls += p.calls;
            total.instances += p.instances;
        }
        println!("{:<9} {:>11} {:>11} {:>7} {:>9}", "sum", total.world, total.models, total.calls, total.instances);
        println!("STATS {} {} {} {} {}", total.world + total.models, st[1].world + st[1].models, total.calls, st[1].calls, st[1].instances);
    }

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
