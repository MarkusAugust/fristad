//! Renderer et bilde av Fristad til en PNG, uten vindu: en drivende adapter
//! som snakker med `ports::driving::Play`.
//!
//!   cargo run --release --bin snapshot -- --out torg.png --night
//!
//! Valg: --out FIL, --size BxH, --night, --eye x,y,z (y med + foran er
//! høyde over terrenget), --yaw grader, --pitch grader, --stance huk|kryp,
//! --weather klart|overskyet|regn|uvær|snø|tåke, --time sek, --descend
//! (ned i krypten først: startstedet i nivå 1), --level N (startstedet i
//! nivå N; med --eye er koordinatene i nivåets eget rom), --portal (ned og hjem med H, så portalen åpner seg), --chest (åpne den første kista), --attack S [--hold], --block, --edit [--edit-move X,Y,Z] (redigeringen, det midt i bildet valgt og flyttet), --film [--fov G] (som i filmen: uten armer), --frames N,
//! --at MODELL X,Y,Z YAW (kamera i modellens rom), --gallery (full gallerivegg), --debug albedo|normal|ao|rough|shadow,
//! --dump-layer N, --shots FIL (mange bilder i én kjøring: én linje per bilde,
//! «ut.png MODELL X,Y,Z YAW PITCH» med kameraet i modellens rom som --at),
//! --model NAVN=FIL.glb (en modell fra fila i stedet for den innebygde, eller
//! en som mangler: en stedfortreder, som trubaduren før figuren kommer),
//! --dump-doc UT.json (nivådokumentet med modulene utvidet, merkene og rommene; med --level-doc),
//! --voice V (hvor sterkt trubaduren synger, 0..1), --level-doc NAVN (nivådokumentet
//! `assets/levels/nivaa_NAVN.json`, eller `brett`: prøvebrettet, kompilert som
//! eneste kryptnivå; med --level 1 og --eye i nivåets rom), --merge (bitene fra
//! byggesettet slått sammen per rute på 16 m, som målingen i fase 0).

use fristad::adapters::gltf::Embedded;
use fristad::adapters::native::seed;
use fristad::adapters::rapier::RapierPhysics;
use fristad::adapters::wgpu::Renderer;
use fristad::application::{prepare, prepare_for_edit, App};
use fristad::domain::weather::{Daylight, WeatherKind};
use fristad::domain::world::Use;
use fristad::ports::driving::{Command, Play};
use fristad::ports::Quality;
use fristad::ports::Renderer as _;
use glam::Vec3;

/// De innebygde modellene, med noen fra filer foran (`--model`), og et
/// nivådokument som eneste kryptnivå (`--level-doc`).
struct WithFiles(Vec<(String, Vec<u8>)>, Option<String>);

impl fristad::ports::ModelSource for WithFiles {
    fn load(&self, name: &str) -> Option<fristad::domain::model::Model> {
        match self.0.iter().find(|(n, _)| n == name) {
            Some((_, bytes)) => fristad::adapters::gltf::load(name, &[("model.glb", bytes)], -1.0, 0.0).map_err(|e| log::warn!("Modell {name}: {e}")).ok(),
            None => Embedded.load(name),
        }
    }
    fn names(&self) -> Vec<String> {
        let mut out = Embedded.names();
        out.extend(self.0.iter().map(|(n, _)| n.clone()).filter(|n| !out.contains(n)).collect::<Vec<_>>());
        out
    }
    fn world_data(&self) -> fristad::domain::world::Data {
        match &self.1 {
            Some(name) => fristad::adapters::data::world_data_with_doc(name),
            None => Embedded.world_data(),
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let flag = |name: &str| args.iter().any(|a| a == name);

    let out = arg("--out").unwrap_or_else(|| "snapshot.png".into());
    let (width, height) = arg("--size").and_then(|s| s.split_once('x').map(|(w, h)| (w.parse().unwrap(), h.parse().unwrap()))).unwrap_or((1280u32, 720u32));

    // --model NAVN=FIL: modeller fra filer foran de innebygde.
    let extra: Vec<(String, Vec<u8>)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--model")
        .filter_map(|(i, _)| args.get(i + 1)?.split_once('=').map(|(n, f)| (n.to_string(), std::fs::read(f).unwrap_or_else(|e| panic!("--model {f}: {e}")))))
        .collect();
    // --dump-level UT.json: bitene i nivådokumentet (--level-doc NAVN) som
    // kompilatoren setter dem ut (modell, sted, dreining, speiling), til
    // «Se i 3D» på godkjenningssiden (`tools/qa/review.py`).
    if let (Some(out), Some(name)) = (arg("--dump-level"), arg("--level-doc")) {
        let doc = fristad::adapters::data::level_doc(&name).expect("nivådokumentet finnes ikke");
        let (f, _) = fristad::domain::world::crypt::compose::compile(&doc, &fristad::adapters::data::kit());
        std::fs::write(&out, serde_json::to_string(&f.instances).unwrap()).expect("kan ikke skrive --dump-level");
        println!("{out}: {} biter", f.instances.len());
        return;
    }
    // --dump-doc UT.json: nivådokumentet slik eieren skrev det (modulene),
    // utvidet (cellene, rommene, kantvalgene) og merkene og rommene etter
    // kompileringen, til bildene av modulene og kartet over prøvenivået
    // (`tools/crypt/module_shots.py`).
    if let (Some(out), Some(name)) = (arg("--dump-doc"), arg("--level-doc")) {
        let kit = fristad::adapters::data::kit();
        let doc = fristad::adapters::data::level_doc(&name).expect("nivådokumentet finnes ikke");
        let (flat, _) = fristad::domain::world::crypt::compose::modules::flatten(&doc, &kit);
        let (f, _) = fristad::domain::world::crypt::compose::compile(&doc, &kit);
        let openings: Vec<serde_json::Value> = doc
            .modules
            .iter()
            .flat_map(|m| fristad::domain::world::crypt::compose::modules::placed_openings(m, &kit).into_iter().map(move |o| serde_json::json!({ "module": m.id, "id": o.id, "kind": o.kind, "at": o.at, "y": o.y, "side": o.side })))
            .collect();
        let all = serde_json::json!({ "doc": doc, "flat": flat, "marks": f.marks, "rooms": f.rooms, "instances": f.instances, "openings": openings });
        std::fs::write(&out, serde_json::to_string(&all).unwrap()).expect("kan ikke skrive --dump-doc");
        println!("{out}: {} moduler, {} celler", doc.modules.len(), flat.cells.len());
        return;
    }
    let source = WithFiles(extra, arg("--level-doc"));
    // --level-edit NAVN: nivåredigeringen med dokumentet (`?nivaa`): nivået
    // tegnes slik redigeringen viser det, med arbeidslys; --eye i nivåets rom,
    // --level-op JSON (flere ganger) gjør endringer først, --game-light gir
    // spillets lys.
    let mut prepared = if let Some(name) = arg("--level-edit") {
        let doc = fristad::adapters::data::level_doc(&name).expect("nivådokumentet finnes ikke");
        fristad::application::prepare_level_editor(&source, seed(), fristad::domain::edit::level::LevelEditor::new(doc, fristad::adapters::data::kit()))
    } else if flag("--edit") {
        prepare_for_edit(&source, seed())
    } else {
        prepare(&source, seed())
    };
    // --merge: bitene fra byggesettet slått sammen per rute på 16 m.
    if flag("--merge") {
        let kit = fristad::adapters::data::kit();
        let n = fristad::application::assembly::merge_instances(&mut prepared, &kit.models(), 16.0);
        eprintln!("Bitene slått sammen til {n} ruter");
    }
    // --at MODELL X,Y,Z YAW: kameraet i modellens eget rom (første instans;
    // Blenders akser, −y er fram), yaw i grader der 0 ser mot huset forfra.
    let first: Vec<(String, glam::Affine3A)> = prepared.world.placements.iter().filter_map(|pl| Some((pl.model.to_string(), *pl.transforms.first()?))).collect();
    // `yaw` er grader fra retningen mot modellens midte, eller med `a`
    // foran (`a90`) fast i modellens rom: 0 ser mot modellens +y i Blender
    // (−z i spillet), 90 mot −x (inne i et hus, der midten er bak deg).
    let at = |model: &str, v: &str, yaw: &str| {
        let v: Vec<f32> = v.split(',').map(|x| x.parse().unwrap()).collect();
        let t = first.iter().find(|(m, _)| m == model).map(|(_, t)| *t)?;
        let eye = t.transform_point3(Vec3::new(v[0], v[2], -v[1]));
        let to = match yaw.strip_prefix('a') {
            Some(_) => t.transform_vector3(Vec3::NEG_Z),
            None => t.transform_point3(Vec3::new(0.0, v[2], 0.0)) - eye,
        };
        let deg: f32 = yaw.trim_start_matches('a').parse().unwrap();
        Some((eye, (-to.x).atan2(-to.z) + deg.to_radians()))
    };
    let model_at = args.iter().position(|a| a == "--at").map(|i| {
        let yaw: f32 = args.get(i + 3).and_then(|y| y.parse().ok()).unwrap_or(0.0);
        at(&args[i + 1], &args[i + 2], &yaw.to_string()).expect("modellen finnes ikke i byen")
    });
    let chest = prepared.world.crypt.chests.first().map(|c| (c.pos, c.yaw));
    let level: Option<usize> = arg("--level").map(|s| s.parse::<usize>().expect("--level N").max(1) - 1);
    let level_origin = level.and_then(|k| prepared.world.crypt.levels.get(k)).map(|l| l.origin);
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
    let (adapter, device, queue) = pollster::block_on(fristad::adapters::gpu(&instance, None)).expect("GPU");
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    // Teksturlagene; mangler et lag en modell bruker, stopper bildet her i
    // stedet for å vise det prosedyriske laget (`bun tools/fetch-assets.ts`).
    let layers = fristad::adapters::layers::from_dir(std::path::Path::new(&arg("--assets").unwrap_or_else(|| "assets/external".into())));
    let missing = fristad::adapters::layers::missing(&layers, &fristad::adapters::gltf::used_layers());
    if !missing.is_empty() {
        eprintln!("Teksturlag uten bilde: {} (kjør `bun tools/fetch-assets.ts`)", missing.join(", "));
        std::process::exit(1);
    }
    let renderer = Renderer::new(device.clone(), queue.clone(), &adapter, format, &App::<Renderer, RapierPhysics>::scene(&prepared), &layers, width, height);
    let mut app = App::new(prepared, renderer, RapierPhysics::new());
    let level_origin = level_origin.or_else(|| app.session.level_editing().map(|l| l.origin));
    for op in args.windows(2).filter(|w| w[0] == "--level-op").map(|w| w[1].clone()) {
        use fristad::ports::driving::LevelCommand;
        let o = serde_json::from_str(&op).unwrap_or_else(|e| panic!("--level-op {op}: {e}"));
        if !app.command(Command::Level(LevelCommand::Op(o))) {
            eprintln!("--level-op {op}: {:?}", app.session.level_error);
        }
    }
    if flag("--game-light") {
        app.command(Command::Level(fristad::ports::driving::LevelCommand::WorkLight(false)));
    }
    // --anchors: stedene filmen trenger (øyet der du begynner, og startstedet
    // i hvert kryptnivå), som json, til filmens tester.
    if flag("--anchors") {
        let (eye, yaw) = app.session.spawn_eye();
        let levels: Vec<String> = (0..10).filter_map(|k| app.session.level_start(k)).map(|(p, y)| format!("[{},{},{},{}]", p.x, p.y, p.z, y.to_degrees())).collect();
        println!("{{\"spawn_eye\":[{},{},{},{}],\"levels\":[{}]}}", eye.x, eye.y, eye.z, yaw.to_degrees(), levels.join(","));
        return;
    }

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
    // --voice V: trubaduren spiller og synger så sterkt (munnen åpnes etter det).
    if let Some(v) = arg("--voice") {
        app.session.play_song();
        app.command(Command::Voice(v.parse().expect("--voice 0..1")));
    }
    // --gamma G: lysstyrken fra innstillingene (1 er som malt).
    if let Some(g) = arg("--gamma") {
        app.command(Command::Gamma(g.parse().expect("--gamma 0.6..2")));
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
            // Alt tegnet svart og himmelen magenta: det som er magenta, er hull (`--leak-scan`).
            "void" => 7,
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
    // --edit: i redigeringen, med det midt i bildet valgt; --edit-move X,Y,Z
    // flytter det (det flyttede og markeringene synes i bildet).
    if flag("--edit") {
        use fristad::ports::driving::EditCommand;
        app.command(Command::Edit(EditCommand::Enter));
        app.command(Command::Edit(EditCommand::Pick { add: false, at: None }));
        if let Some(m) = arg("--edit-move") {
            let v: Vec<f32> = m.split(',').map(|x| x.parse().unwrap()).collect();
            app.command(Command::Edit(EditCommand::Move(Vec3::new(v[0], v[1], v[2]))));
        }
        if let Some(e) = app.session.editor() {
            for s in e.selected() {
                log::info!("valgt: {} ({:?})", s.label, s.r);
            }
            for b in e.breaches() {
                log::info!("brudd: {:?} {}", b.r, b.why);
            }
        }
    }

    // --oil BRØK: så mye olje i lykta (0 er sluknet).
    if let Some(o) = arg("--oil") {
        app.session.set_oil(o.parse().unwrap());
    }
    // --drop N: N tokens spretter ut 3 m foran deg.
    if let Some(n) = arg("--drop") {
        app.session.drop_tokens(3.0, n.parse().unwrap());
    }
    // --key: nøkkelen eliten mister ligger og svever 2,5 m foran deg, litt
    // til høyre for skjoldet.
    if flag("--key") {
        app.session.drop_key(2.5, 0.35);
    }
    // --bolt [--bolt-side M] [--bolt-at S]: bare én korrupt input 7 m foran
    // deg (og M til høyre), som lader og spytter; bildet tas S sekunder
    // etter (0,95: markøren midt i lufta; under 0,6 lader den ennå). --time
    // teller ikke.
    if flag("--bolt") {
        let side = arg("--bolt-side").map(|s| s.parse().unwrap()).unwrap_or(0.0);
        assert!(app.session.input_ahead(7.0, side), "ikke fritt gulv foran deg til inputen");
        let at: f32 = arg("--bolt-at").map(|s| s.parse().unwrap()).unwrap_or(0.95);
        for _ in 0..(at * 60.0) as usize {
            app.tick(1.0 / 60.0);
        }
        log::info!("markører i lufta: {}", app.session.bolts_in_air());
    }
    // Kjør spillet litt, så animasjoner og fiender står der de skal.
    let time: f32 = if flag("--bolt") { 0.0 } else { arg("--time").map(|s| s.parse().unwrap()).unwrap_or(0.5) };
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
        // --film [--fov G] [--roll G]: som i filmen, uten armer, og byfolket og fiendene
        // lever i et sekund før bildet tas.
        if flag("--film") {
            let fov: f32 = arg("--fov").map(|s| s.parse().unwrap()).unwrap_or(60.0);
            app.command(Command::Film(Some(fristad::ports::driving::Shot { eye, yaw: yaw.to_radians(), pitch: pitch.to_radians(), fov, roll: arg("--roll").map(|s| s.parse::<f32>().unwrap().to_radians()).unwrap_or(0.0) })));
            for _ in 0..30 {
                app.tick(1.0 / 30.0);
            }
        }
    }
    // --edit: i redigeringen, med det midt i bildet valgt; --edit-move X,Y,Z
    // flytter det (det flyttede og markeringene synes i bildet).
    if flag("--edit") {
        use fristad::ports::driving::EditCommand;
        app.command(Command::Edit(EditCommand::Enter));
        app.command(Command::Edit(EditCommand::Pick { add: false, at: None }));
        if let Some(m) = arg("--edit-move") {
            let v: Vec<f32> = m.split(',').map(|x| x.parse().unwrap()).collect();
            app.command(Command::Edit(EditCommand::Move(Vec3::new(v[0], v[1], v[2]))));
        }
        if let Some(e) = app.session.editor() {
            for s in e.selected() {
                log::info!("valgt: {} ({:?})", s.label, s.r);
            }
            for b in e.breaches() {
                log::info!("brudd: {:?} {}", b.r, b.why);
            }
        }
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
    // --gfx bryter=verdi,… endrer enkeltbrytere i forhåndsvalget (`ports::Quality::set`).
    let mut quality = Quality::from_name(if flag("--advanced") { "avansert" } else if flag("--simple") { "enkel" } else { "standard" }).expect("forhåndsvalget");
    for kv in arg("--gfx").unwrap_or_default().split(',').filter(|s| !s.is_empty()) {
        let (k, v) = kv.split_once('=').unwrap_or((kv, "1"));
        assert!(quality.set(k, v.parse().expect("--gfx bryter=tall")), "ukjent grafikkbryter: {k}");
    }
    app.command(Command::SetQuality(quality));
    // HDR-bildet i 4 byte per prøve (målingen i nettleseren har samme bryter).
    // HDR i 4 byte per prøve, eller standard uten kantutjevning (SMAA av),
    // som bryterne i målingen.
    if flag("--small-hdr") || flag("--no-smaa") {
        app.command(Command::SetBench(fristad::ports::Bench { small_hdr: flag("--small-hdr"), smaa: !flag("--no-smaa"), ..Default::default() }));
    }
    // --leak-scan PUNKTER: fra hvert punkt (x y z i nivåets rom, én linje
    // hver) i 8 retninger rundt og 3 opp og ned, med `--debug void`: skriver
    // hvert bilde der himmelen synes gjennom noe, med antall piksler.
    if let Some(list) = arg("--leak-scan") {
        let o = level_origin.unwrap_or(Vec3::ZERO);
        let mut found = 0;
        for (n, line) in std::fs::read_to_string(&list).expect("kan ikke lese --leak-scan").lines().filter(|l| !l.trim().is_empty()).enumerate() {
            let v: Vec<f32> = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
            let eye = Vec3::new(v[0], v[1], v[2]) + o;
            for k in 0..8 {
                for pitch in [-50.0f32, 0.0, 50.0] {
                    let yaw = k as f32 * 45.0;
                    app.command(Command::Place { eye, yaw: yaw.to_radians(), pitch: pitch.to_radians() });
                    for _ in 0..2 {
                        app.renderer.set_target(target.create_view(&Default::default()));
                        app.draw(width as f32 / height as f32);
                    }
                    let px = grab(&device, &queue, &target, width, height);
                    let hits = px.chunks(4).filter(|c| c[0] > 150 && c[2] > 150 && c[1] < 90).count();
                    if hits > 0 {
                        found += 1;
                        let path = format!("{}_{n}_{k}_{}.png", out.trim_end_matches(".png"), pitch as i32);
                        write_png(&path, width, height, &px);
                        println!("HULL {hits} {} {} {} {yaw} {pitch} {path}", v[0], v[1], v[2]);
                    }
                }
            }
        }
        println!("LEAKSCAN {found}");
        return;
    }
    // --film-seq FIL: filmens bilder i ett løp (verden under filmens
    // overlegg): «UT X Y Z YAW PITCH FOV ROLL NATT DT» per linje, grader,
    // NATT 0 eller 1; spillet går DT sekunder mellom bildene.
    if let Some(list) = arg("--film-seq") {
        let mut night = None;
        for line in std::fs::read_to_string(&list).expect("kan ikke lese --film-seq").lines().filter(|l| !l.trim().is_empty()) {
            let f: Vec<&str> = line.split_whitespace().collect();
            let n: Vec<f32> = f[1..].iter().map(|x| x.parse().unwrap()).collect();
            let dark = n[7] > 0.5;
            if night != Some(dark) {
                app.session.set_daylight(if dark { Daylight::Night } else { Daylight::Day }, true);
                night = Some(dark);
            }
            let shot = fristad::ports::driving::Shot { eye: Vec3::new(n[0], n[1], n[2]), yaw: n[3].to_radians(), pitch: n[4].to_radians(), fov: n[5], roll: n[6].to_radians() };
            app.command(Command::Film(Some(shot)));
            app.tick(n[8].max(1.0 / 120.0));
            app.renderer.set_target(target.create_view(&Default::default()));
            app.draw(width as f32 / height as f32);
            write_png(f[0], width, height, &grab(&device, &queue, &target, width, height));
        }
        return;
    }
    // --shots: ett bilde per linje, med samme spill og renderer.
    if let Some(list) = arg("--shots") {
        for line in std::fs::read_to_string(&list).expect("kan ikke lese --shots").lines().filter(|l| !l.trim().is_empty()) {
            let f: Vec<&str> = line.split_whitespace().collect();
            // Med --level: «UT X,Y,Z YAW [PITCH]» i nivåets eget rom.
            let (eye, yaw, pitch_at) = if let Some(o) = level_origin {
                let v: Vec<f32> = f[1].split(',').map(|x| x.parse().unwrap()).collect();
                (Vec3::new(v[0], v[1], v[2]) + o, f[2].parse::<f32>().unwrap().to_radians(), 3)
            } else {
                let Some((eye, yaw)) = at(f[1], f[2], f[3]) else {
                    eprintln!("{}: står ikke ute", f[1]);
                    continue;
                };
                (eye, yaw, 4)
            };
            let pitch: f32 = f.get(pitch_at).map(|p| p.parse().unwrap()).unwrap_or(4.0);
            app.command(Command::Place { eye, yaw, pitch: pitch.to_radians() });
            // Med --film: uten armer, som i filmen.
            if flag("--film") {
                app.command(Command::Film(Some(fristad::ports::driving::Shot { eye, yaw, pitch: pitch.to_radians(), fov: 70.0, roll: 0.0 })));
                app.tick(1.0 / 30.0);
            }
            for _ in 0..frames.max(1) {
                app.renderer.set_target(target.create_view(&Default::default()));
                app.draw(width as f32 / height as f32);
            }
            write_png(f[0], width, height, &grab(&device, &queue, &target, width, height));
            println!("{}", f[0]);
        }
        return;
    }
    for _ in 0..frames.max(1) {
        app.renderer.set_target(target.create_view(&Default::default()));
        app.draw(width as f32 / height as f32);
    }

    let pixels = grab(&device, &queue, &target, width, height);
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

    write_png(&out, width, height, &pixels);
    println!("{out}");
}

/// Leser bildet tilbake fra kortet (RGBA, rad for rad).
fn grab(device: &wgpu::Device, queue: &wgpu::Queue, target: &wgpu::Texture, width: u32, height: u32) -> Vec<u8> {
    let row = (width * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("readback"), size: (row * height) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(height) } },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait).expect("poll");
    let data = buf.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let at = (y * row) as usize;
        pixels.extend_from_slice(&data[at..at + (width * 4) as usize]);
    }
    pixels
}

fn write_png(path: &str, width: u32, height: u32, rgba: &[u8]) {
    let file = std::fs::File::create(path).expect("kan ikke skrive fila");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(rgba).unwrap();
}
