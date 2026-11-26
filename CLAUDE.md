# Fristad – instruksjoner for Claude

Fristad er et spill i en nordisk kjøpstad rundt 1350. Under kirkegården ligger krypten, og der nede holder de korrupte komponentene fra designsystemet [LO-TEK](https://github.com/MarkusAugust/lo-tek) til. Komponentene er byttet du vinner, smir til utstyr og leser om i bøkene. Se README.md for oversikt og `docs/spilldesign.md` for spillet. Svar eieren på norsk.

Denne fila har reglene som alltid gjelder. Hvordan hver del er bygd (funksjonene, konstantene, testene og verktøyene) står i `docs/oppslag/`, se tabellen nederst. **Les oppslaget for delen før du endrer den, og oppdater det når du endrer noe det beskriver.** Nye detaljer skrives der, ikke her.

## Regler

- **Kanon: `docs/kanon.md` er historien og kjernen i spillet** (eieren 16. oktober). Alt nytt bygger på den og bruker navnene derfra: filmen, replikkene, lappene og bøkene i krypten, fiendene, nivåene, butikktekstene og slutten. Mønsterboka er LO-TEK (uavhengig av språk og rammeverk), Den kongelige komponentboken på hoffets tunge (*lingua Reactia*, React) er motparten, Rådet og Kontorets forretningsregler korrumperte den, Inkvisisjonen håndhevet den, den store oppgraderingen brakk alt, munkene og mester Ansgar forseglet skapningene og Rammeverket i krypten, og spilleren er en fremmed uten fortid som kommer til porten i 1350. Ingen moderne ord i verden. Kanon endres bare av eieren; det som er åpent, står der og avgjøres av eieren.
- **Alt som synes er en ekte modell.** Hus, murer, møbler, figurer, fiender, våpen, trær: alt lages i Blender av skriptene i `tools/models/`, med teksturer. Ingen bokser rett i bymeshen og ingen prosedyrisk geometri i Rust som erstatning for en modell. `Builder::place` setter ikke ut noe om modellen mangler, og melder det (`Builder.missing`, WARN i loggen). Testen `alt_er_ekte_modeller_med_teksturer` krever bakte teksturer eller teksturlag på alle modellene, ikke flate farger.
- **Ny stil (eieren, 9. oktober): malt by, skarpt bilde, 60 fps på referansemaskinen (eierens Mac mini M2, Chrome og Safari; `docs/kvalitet/ytelse.md`).** Se `docs/ny-stil.md`. Stilisert og håndmalt uten konturer, varm by og kald kjeller. Detaljene (stein, planker, takstein) males inn i teksturene; bare det som synes i silhuetten er geometri (bjelker, takskjegg, piper, karmer, trapper). Budsjett: vanlige hus 3 000–10 000 trekanter, hus med interiør høyst 25 000, rundt 500 000 trekanter og under 300 tegnekall per bilde for alle pass. Teksturene er malte, flisbare og skarpe (rundt 1 teksel per mm nær), laget av eieren med bildegenerator etter prompts fra Claude (`assets/textures/painted/<navn>/albedo.png`, avledet med `tools/textures/derive.py`). Ingen uskarphet, spøkelser, støy, korn eller flimmer: SMAA i stedet for TAA (MSAA var for dyrt på Mac), ingen SSAO i standard grafikk. Bygges om i rekkefølgen pilot (Vertshuset og torget) → byen → figurene → naturen → kjelleren; eieren godkjenner piloten før resten. Til ombyggingen er ferdig, står de gamle modellene og rendereren som beskrevet under.
- **Se på hver modell** med `tools/models/preview.py` før den tas i bruk, og vis eieren skjermbilder. Mål trekanter og tegnekall med `sh tools/stats.sh` (faste kameraposisjoner; grunnlinjen står i `docs/ny-stil.md`).
- **Hexagonal arkitektur.** Se under. Kjernen, portene og applikasjonen er egne pakker i `crates/`, så kompilatoren stopper en forbudt avhengighet. `src/architecture.rs` passer på pakkene: `pakkene_har_bare_lovlige_avhengigheter` leser `Cargo.toml` i hver, og `kjernen_bruker_ikke_filer_miljo_eller_klokke` feiler om de bruker `std::fs`/`env`/`process`/`net`/`thread`/`time` eller leser filer når de kompileres (`include_str!`, `env!` …).

## Arkitektur: porter og adaptere

Detaljene står i `docs/oppslag/arkitektur.md`.

- **Fire pakker, og avhengighetene går bare innover.** `crates/fristad-domain` (kjernen: verden, reglene og rene data; bare `glam`, `bytemuck`, `serde` og `log`, ingen filer, miljøvariabler eller klokke), `crates/fristad-ports` (grensesnittene: `Renderer`, `Physics`, `ModelSource` og den drivende porten `Play` med `Input`, `Command` og `Event`), `crates/fristad-app` (økta og `App`, som bruker kjernen gjennom portene) og hovedpakken `fristad` i `src/` (adapterne: `wgpu/`, `rapier.rs`, `gltf.rs`, `web.rs`, `native.rs`, `wire.rs`, og `src/bin/`).
- **Nye delsystemer eier sin egen tilstand.** De er ikke metoder på økta: de får det de trenger som argument (kameraet, helten, hendelseslista), slik `editing.rs`, `level_editing.rs`, `tavern.rs`, `trade.rs`, `solids.rs` og `hud.rs` gjør.
- **Hendelsene er det som skjer i spillet.** Det verten må hente (nivåer og modeller) er levering og står for seg: `driving::Fetch`, som verten spør etter med `App::fetches`.
- **Kontrakten mot nettlaget er testet.** `wire::json(&Event)` er det nettlaget får, og `web/events.json` er formen. En ny hendelse trenger en prøve i `wire_tests.rs`, og fila skrives med `FRISTAD_SKRIV_KONTRAKT=1 cargo test kontrakten`. Testene krever også at `web/main.js` tar imot hver type og bare leser felt som sendes, og at hvert `engine.navn(…)` i `web/` finnes i nettadapteren.
- **Nettlaget er delt etter oppgave.** Modulene importerer aldri `main.js`, `nivaa.js` eller `editor.js`. Det de deler, står i `shared.js` og `nivaa-shared.js`, og bare eieren setter en verdi (de andre bruker `set…`). En ny fil må inn i `tools/build-web.sh`.
- **Linjetak og breddetak** (`src/architecture.rs`): høyst 800 linjer per kildefil og 200 tegn per linje, for Rust i alle pakkene og JavaScript i `web/`. Det som var over da grensen kom, står i `TOO_LONG` og `TOO_WIDE` med taket sitt og kan bare krympe.
- **Testene.** En test som ikke trenger modellene, fysikken eller økta, legges i `crates/fristad-domain/tests/regler/` (`cargo test -p fristad-domain`, rundt 20 sekunder). Resten står i `src/*_tests.rs` og `src/tests.rs`, som setter sammen hele spillet med de ekte adapterne.

## Verden

Detaljene står i `docs/oppslag/verden.md`.

- **Byen er data.** `assets/layout/town.json` er byen, og `layout::apply` bygger den. Ting flyttes i fila eller i redigeringen (`docs/redigering.md`), ikke i kode. `cargo run --bin layout` skriver filene i fast form.
- **Naturlig og organisk, som i middelalderen (eieren):** ingen rette vinkler mellom husene (hvert hus dreies for seg), muren følger et ujevnt omriss og aldri en perfekt sirkel, og bygningene går aldri inn i hverandre: hele huset med takskjegg og luft (`full`, `EAVES`, `AIR`) holder avstand til nabohusene og muren, mens veggene holder avstand til gata. Se på et bilde ovenfra etter hver endring.
- **Plasser ting som i en ekte by.** Se på et oversiktsbilde (`snapshot --eye X,60,Z --pitch -89`) før du plasserer noe nytt, og etter hver endring. Alt som bygges får kollider.
- **Ingenting skal sveve eller synke.** `Builder::audit` sjekker hver modellinstans mot bakken (eller gulvet registrert med `Builder::floor`); `cargo test` feiler om noe svever mer enn 8 cm eller står 30 cm nede (trestammer og murverket i `town::FOUNDED`, med fundament under bakken, får gå ned i bakken i skråninger). Ting i skråninger bruker `Builder::on_slope`; ting på vegger eller oppå noe `place_mounted`.
- **Ingen usynlige vegger** (eieren 18. oktober): kolliderne følger formen. Målet på en ting du kan bruke skal aldri ligge inni tingens egen kollider.
- **Ingen går gjennom noe.** `alt_stort_har_kollider` krever kollider ved foten av hver modellinstans over 60 cm (eller høyere opp på midtlinja for det som er åpent nede, som portrommet), og `kapselen_stopper_mot_trær_og_kanten` går med Rapier mot eiker og mot kanten av verden (`nature::world_edge`, 199 m).

## Spillet

- **Navnene kommer fra kanon og LO-TEK.** Komponentene heter det de heter i LO-TEK (Button, ikke knapp), og fiendene, nivåene og stedene heter det kanon sier. Rundt boka er spillet norsk; bøkene er LO-TEK-dokumentasjonen slik den er, på engelsk.
- **Krypten er faste, håndlagde nivåer**, like hver gang: ingen generator og intet frø. Den er bare så dyp som nivåene som finnes, og det er ingen øvre grense på hvor mange (eieren 26. november; de gamle var ti).
- **Lyskilder skal bety noe** (eieren 13. oktober): nede er det nesten ikke noe grunnlys, og lyset går ikke gjennom murene.
- **Eieren bygger nivåer med moduler, ikke biter** (19. oktober), og det finnes ett sett moduler der steinen velges i redigeringen (20. oktober).
- **Figurene og de nye fiendene lages fra TRELLIS** med `folk3d.py`, `mob3d.py` og `item3d.py`. Formen og teksturen brukes som de er; den egne figurmotoren er forkastet og slettet.
- **Det skal alltid lønne seg å lese boka**, og spillet skal ikke være for lett (eieren 18. oktober).

## Grafikk

Detaljene står i `docs/oppslag/grafikk.md`.

- **Grafikken endres aldri av seg selv.** Bare spilleren bytter innstillinger; ingen fps-vakt bytter nivå.
- **Ingen TAA, bevegelsesvektorer, volumetrisk tåke, støy eller flimmer.** SMAA etter tonemappingen, faste prøver i skyggene.
- **Materialene får rollen sin som data ved eksporten** (`tools/models/slim.py`, `roles.py`). Lasteren gjetter aldri etter navn.
- **I Blender må alle deler ha uv-laget `UVMap`** (`town.ensure_uv`), ellers blir flatene grå.
- **Valider shaderne mot Dawn før push:** `node tools/wgsl-check.mjs`. `textureSample` må stå i uniform kontrollflyt, og WebGL2 kan ikke lese dybdeteksturer med `textureLoad`.

## Arbeidsflyt

Detaljene står i `docs/oppslag/arbeidsflyt.md`.

- **Eierens tid:** vis eieren en ting igjen bare når hvert punkt eieren skrev om den er rettet og sjekket på før- og etterbilder (punktene, «rettet slik» og før/etter side om side); det som ikke er endret, vises ikke. Når planen endres, får eieren bare endringen; hele planen står i `docs/plan.md`.
- **Katalogen først.** `docs/katalog.md` er oversikten over alt spillet er laget av. Les den før du bruker eller lager noe, og hold `assets/catalog/textures.json` og `models.json` oppdatert. Eierens dom i `assets/catalog/approvals.json` bestemmer bare eieren.
- **Kvalitetsportene** (`tools/qa/`) kan ikke overstyres. Eieren dømmer på godkjenningssiden, og ikke løse deler: bitene dømmes gjennom modulene og mekanismene.
- **Promptene til eieren** står i `docs/prompter/`. Nye legges der, og det som er laget, strykes.
- **Bygg minst mulig på Netlify.** Netlify bygger aldri (`ignore = "exit 0"` i netlify.toml). GitHub Actions (`.github/workflows/web.yml`) bygger Wasm og laster opp `dist/` med Netlify CLI. Push til `master` går til https://fristad.netlify.app, push til `claude/**` går til forhåndsvisningen https://claude--fristad.netlify.app. Repoet er offentlig (eieren 18. oktober), men test lokalt først og push samlet.
- **Kjør aldri `cargo fmt` på hele repoet:** koden står med brede linjer uten `rustfmt.toml`, og `cargo fmt` skriver om titalls filer og sprenger linjetaket (`filene_er_ikke_for_lange`). Skal en fil du har endret formateres, bruk `rustfmt --edition 2021 --config max_width=200,use_small_heuristics=Max,skip_children=true FIL`.
- **Test lokalt før push.** Actions kjører ikke testene (eieren 25. november: de tok 21 av 27 minutter per bygg), så den lokale kjøringen er den eneste. `cargo test --workspace` og `cargo build --bins`, og skjermbilder med `snapshot`. Skyen trenger `apt-get install -y mesa-vulkan-drivers` (lavapipe) først.
  `./target/debug/snapshot --out shots/x.png --size 1280x720 --eye 0,+1.7,44 --yaw 0` (`--eye X,+1.7,Z` er høyde over terrenget; yaw 0 ser mot nord (−Z), 90 mot vest, −90 mot øst) [--night] [--weather regn|snø|tåke|uvær|overskyet] [--descend] [--time S] [--frames 8] [--debug albedo|normal|ao|rough|shadow]. Se på bildene før du pusher, og vis eieren skjermbilder underveis.
- **Wasm kan ikke bygges i Claude sitt skymiljø** (static.rust-lang.org er sperret). Wasm-feil vises i Actions.
- **Test nettversjonen i Chromium uten skjerm:** Actions legger den ferdige nettsiden på grenen `web-dist`. `sh tools/pull-dist.sh` henter den til `dist-web/`, og `PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright node tools/browser-test.mjs dist-web webgl|webgpu UT.png` kjører den med SwiftShader og skriver konsollen og et skjermbilde. `?webgl` i adressen tvinger WebGL2.
- **Modellene lages i Blender som Python-modul** (`pip install bpy`, Python 3.13) og legges i `models!`-tabellen i `adapters/gltf.rs`. Modeller som ventes, står i `model::PLANNED`.
- **Bare datamaskin** (eieren 19. oktober): mobil og iPhone er ikke et mål.
- Pass på at `[hidden]` faktisk skjuler: elementer med egen `display` trenger regelen øverst i `web/style.css`.

## Beslutninger

- Egen motor i Rust + wgpu (WebGPU, WebGL2 som reserve) + Rapier.
- Setting: nordisk senmiddelalder, rundt 1350. Spillet skal føles som The Witcher 2 og det gamle Might and Magic: førsteperson med armer, ekte skjelettanimasjon, kamp med kombo, blokk, parade og unnamanøver.
- Komponentene fra LO-TEK finnes bare som bytte, utstyr og bøker (ikke som ting rundt i byen).
- Lyst tema er dag, mørkt tema er natt. V bytter vær.
- Hosting: Netlify for den statiske byen. Railway er aktuelt senere for en flerspillerserver.

## Status og plan

Planen fra 11. oktober står i `docs/plan.md` (kvalitet før alt, ren kode, målt ytelse, åpnings- og sluttfilm), med fasene 0–7; statusen før planen i `docs/status-11-okt.md`. **Planene fra 17. oktober står i samme fil, del 10–12, og gjelder det som kommer etter kryptnivå 8–10:** del 10 (krypten lastet nivå for nivå, som løser minnekrasjet på iPhone og WebGL2; Rammeverket som sluttboss bygd etter filmen; oppdragene for nivå 9 og 10), del 11 (redigeringen videre: egenskapspanel for merkene, ruter, fest, grupper, lys, kamera, lagring uten nøkkel, rommene i krypten som redigerbare ting) og del 12 (alle modellene i den malte stilen: lagene er alt malte; neste er propsene med bakte teksturer, stilporten, krypttemaene og naturen). Les den delen før du starter på noe av dette. Gjeldende regler fra planen: ingenting går inn i spillet uten å ha bestått de automatiske tekniske og estetiske kontrollene (`tools/qa`, fase 2), ingen kontroll kan overstyres, og hver feil rettes ved roten i felles kode og blir en regel med test.

## Oppslag

| Skal du endre | Les |
|---|---|
| Pakkene, portene, økta, nettlaget, kontrakten, testene | `docs/oppslag/arkitektur.md` |
| Byen, landskapet, kolliderne, ting du kan bruke, portalen | `docs/oppslag/verden.md` |
| Kryptnivåene, mekanismene, lykta og mørket | `docs/oppslag/krypten.md`, `docs/krypten-ny.md`, `docs/krypten/format.md` |
| Fiendene, riggene og bevegelsene | `docs/oppslag/fiendene.md` |
| Kampen, armene og våpnene, smiinga, komponentene, boka, spillerens hus | `docs/oppslag/kampen-og-utstyret.md` |
| Innbyggerne, figurene fra TRELLIS, trubaduren | `docs/oppslag/innbyggerne.md` |
| Kartet, stolpene, journalen, tastene, musikken, lydene, filmen | `docs/oppslag/grensesnitt-og-lyd.md` |
| Nivåredigeringen, modulene, steinen, siden `?nivaa` | `docs/oppslag/nivaaredigering.md`, `docs/nivaaredigering.md` |
| Rendereren, grafikkinnstillingene, teksturlagene, materialene | `docs/oppslag/grafikk.md`, `docs/ny-stil.md` |
| Katalogen, godkjenningssiden, leveringen av modellene, nettversjonen, LO-TEK, Blender | `docs/oppslag/arbeidsflyt.md` |
| Redigeringen av byen (`?edit`, P) | `docs/redigering.md` |
| Historien og navnene | `docs/kanon.md` (bare eieren endrer den) |
| Lærdommen fra feil | `docs/kvalitet/laerdom.md` |
