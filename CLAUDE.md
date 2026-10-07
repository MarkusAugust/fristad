# Fristad – instruksjoner for Claude

Fristad er dokumentasjonen til designsystemet [Fristil](https://github.com/MarkusAugust/fristil) som en 3D-by du går rundt i. Se README.md for oversikt. Svar eieren på norsk.

## Arkitektur: ports and adapters

- `src/domain/` er kjernen og har **ingen** avhengighet til wgpu, winit, wasm, gltf eller rapier. Der bor tokens, geometri (`geometry.rs`), tekst (`text.rs`), byen (`city/`), komponentene i byen (`installations/`), spillet (`game.rs`) og det rendereren får per bilde (`scene.rs`).
- `src/domain/ports.rs` er portene: `Renderer`, `Physics`, `ModelSource`.
- `src/adapters/` er adapterne: `wgpu/` (renderer og shadere), `rapier.rs` (fysikk), `gltf.rs` (modeller), `web.rs` (nettleser, drivende), `native.rs` (eget vindu, drivende). `src/bin/snapshot.rs` er en drivende adapter for skjermbilder.
- `src/app.rs` setter sammen kjernen og portene. De drivende adapterne snakker bare med `App`.
- **Ny komponent i byen:** én fil i `src/domain/installations/`, implementer `Installation` (bygg geometri og props, `tick`, `act`, `pose`, `palette`), legg den til i `all()`, legg navnet til i `COMPONENTS` i `tools/export-docs.ts`, og eventuelt variantene i `VARIANTS` i `web/main.js`. Resten (sikting, E/R/F, panel, tema) følger av `component()`.
- Målestokk: 1 rem = 1 m for arkitekturen (etasjehøyde fra spacing-skalaen). For komponentene er **1 CSS-piksel = 1 cm** (`installations::PX`), så `.fs-button` med 16px tekst og 8/16px padding blir 42 × 80 cm.
- Terrenget (`city/terrain.rs`) er en høydefunksjon: torget er planert innenfor 21 m, utenfor heller og bølger bakken. Alt som bygges spør `Builder::ground`/`on_ground`, og bakken er et 2 m-rutenett med heightfield-kollider. `city::town_dist` og `path_dist` sier hvor nær bebyggelsen og stiene et punkt er; `ground_mat` velger brostein, grus eller eng etter det.
- Husene varieres av `house::Spec::random` (gavl mot gata, steinsokkel, skodder, blomsterkasser, balkong, etasjehøyde) i rekker som bølger, med smug i mellom. Enga (`nature::meadow`) har løvtrær, busker, steiner, steingjerder, tuster og granskog.

## Arbeidsflyt

- **Bygg minst mulig på Netlify.** Netlify bygger aldri (`ignore = "exit 0"` i netlify.toml). GitHub Actions (`.github/workflows/web.yml`) bygger Wasm og laster opp `dist/` med Netlify CLI. Push til `master` går til https://fristad.netlify.app, push til `claude/**` går til forhåndsvisningen https://claude--fristad.netlify.app. Repoet er privat, så Actions-minutter er begrenset: test lokalt først, push samlet.
- **Test lokalt før push.** `cargo build --bins` og skjermbilder med `snapshot`. Skyen trenger `apt-get install -y mesa-vulkan-drivers` (lavapipe) først.
  `./target/debug/snapshot --out shots/x.png --size 1280x720 --eye 7,1.7,9 --yaw 14.5 [--theme dark] [--weather regn|snø|tåke|uvær|overskyet] [--gate] [--variant button=secondary] [--frames 8] [--debug albedo|normal|ao|rough|shadow]`
  `--dump-layer N` skriver et bakt teksturlag til PNG. Se på bildene før du pusher, og vis eieren skjermbilder underveis.
- **Wasm kan ikke bygges i Claude sitt skymiljø** (static.rust-lang.org er sperret). Wasm-feil vises i Actions: `gh run view <id> --repo MarkusAugust/fristad --log-failed`.
- **Test nettversjonen** i eierens nettleser. `window.fristad.engine` har `place(x,y,z,yaw,pitch)`, `key(code, down)`, `set_paused`, `set_variant(component, variant)`, `set_weather(navn)`, `set_online(bool)`, og `window.fristad.openPanel/closePanel`.
- Tokens: `bun tools/export-tokens.ts`. Dokumentasjonstekst: `bun tools/export-docs.ts [sti-til-fristil-klone]` (henter fra `master` i fristil-repoet). Skriftatlas: `cargo run --release --bin bake_font -- /usr/share/fonts/opentype/inter/Inter-Medium.otf`.
- **Eksterne ressurser (CC0 fra Poly Haven):** `bun tools/fetch-assets.ts` henter fotoskannede materialer per teksturlag og modeller (benk, lykt, tønne, kasse, plantekasse, busk, sykkel, søppelbøtte, kjerre, tre) til `assets/external/` med `manifest.json`. Actions kjører det under byggingen (hurtiglagret), nettleseren henter filene ved oppstart, og motoren bruker dem framfor de bakte lagene og den prosedyriske geometrien når de finnes (`adapters/assets.rs`, portene `TextureSource` og `ModelSource`). Lista over hva som hentes står øverst i skriptet.
- Nettverk i skyen: GitHub (raw-filer) og crates.io er åpne; Poly Haven, ambientCG, Kenney og Quaternius er sperret. Actions legger de hentede ressursene på grenen `external-assets`; `tools/pull-assets.sh` henter dem til `assets/external/` (i .gitignore), og `snapshot --assets assets/external` bruker dem.
- Poly Havens 1k-modeller har JPEG-teksturer uten alfa. Materialer med `alphaMode` BLEND/MASK får `flag::ALPHA`, og rendereren alfatester dem mot svart bakgrunn i alle pass. Busker med bladkort blir likevel for tynne, så buskene er prosedyriske.
- Bladteksturen må ha bladgrønn RGB også utenfor bladene, ellers mørkner mip-nivåene mot svart på avstand. De innebygde modellene (Lantern, Fox) er fra `KhronosGroup/glTF-Sample-Assets`, og teksturene bakes prosedyrisk på GPU-en som reserve (`adapters/wgpu/shaders/bake.wgsl`).
- Pass på at `[hidden]` faktisk skjuler: elementer med egen `display` trenger regelen øverst i `web/style.css`.

## Beslutninger

- Egen motor i Rust + wgpu (WebGPU, WebGL2 som reserve) + Rapier, for maks ytelse og realisme.
- Stil: stilisert realisme, nordisk arkitektur. Realismen kommer fra geometrisk detalj (fasadegrammatikken i `city/house.rs`), bakte PBR-teksturer med normalkart, kaskadeskygger, SSAO, TAA, volumetrisk tåke og vær. Modellfiler (glTF) er greit.
- Lyst tema = dag, mørkt tema = natt. V bytter vær; `connection-status` gir uvær når nettleseren er offline.
- Komponentene skal ikke stå på sokler: hver komponent får en metafor i byen (knappen ved porten, oppslaget på veggen, bryteren for gatelysene, framdriftsstolpen over porten, tagger og badges på oppslagstavla, spinneren på masta, brødsmulene i brosteinen, skeleton-huset under bygging, masta for forbindelsen).
- Hosting: Netlify for den statiske byen. Railway er aktuelt senere for en flerspillerserver.

## Status (7. oktober 2026)

Ferdig: ny renderer (kaskadeskygger, dybdeprepass, SSAO, TAA, tåke med lysstråler, regn, snø, våte flater, snødekke), bakte teksturer med normalkart, fotoskannede materialer og modeller fra Poly Haven, fasadegrammatikk med variasjon, terreng med eng, åser og granskog, trær med bladkort og vind, SDF-tekst i Inter, ti komponenter i byen med panel for hver, Fristils egne toast og connection-status i nettlaget, lasteskjerm og inngang uten pekerlås (dra med musa).

Neste:
1. Tilbakemelding fra eieren på forhåndsvisningen (bybildet, lys, materialer, lesbarhet på skiltene, ytelse i WebGL2).
2. Ytelse: frustum-culling og LOD, lavere oppløsning på svake GPU-er, måling med timestamp-queries.
3. Flere komponenter: dialog som telefonkiosk, tabs som hus med dører, card, accordion; bydelene Verkstedet, Tårnene, Tokenparken.
4. Lyd, kart, touch-kontroller, tilgjengelig 2D-modus.
