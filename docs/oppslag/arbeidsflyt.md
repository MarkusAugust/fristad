# Arbeidsflyten og verktøyene

Promptene, katalogen, kvalitetsportene og godkjenningssiden, bygg og test, leveringen av modellene, nettversjonen, LO-TEK og Blender. Flyttet hit fra `CLAUDE.md` 26. november; reglene som alltid gjelder, står fortsatt der.

## Eierens tid

**Eierens tid:** vis eieren en ting igjen bare når hvert punkt eieren skrev om den er rettet og sjekket på før- og etterbilder (punktene, «rettet slik» og før/etter side om side); det som ikke er endret, vises ikke. Når planen endres, får eieren bare endringen; hele planen står i `docs/plan.md`.

## Promptene til eieren

**Promptene til eieren** står samlet i `docs/prompter/` (ryddet 19. oktober: det som er laget eller bruker forlatte metoder, er slettet): `trellis-monstre.md`, `trellis-folk.md`, `trellis-gjenstander.md`, `trellis-dyr.md` (ETT bilde per modell til TRELLIS.2: folk og tobeinte forfra i A-stilling, gjenstander tre kvart ovenfra, dyr fra siden; teksturen brukes som den er), `natur.md` (nakne stammer til TRELLIS, løvet som bladkort), `teksturer.md`, `ikoner.md` og `film.md`, med den prioriterte rekkefølgen i `README.md`. Nye prompter legges der, og det som er laget, strykes.

## Katalogen først

**Katalogen først.** `docs/katalog.md` er oversikten over alt spillet er laget av: hver tekstur (malt kilde, lag, hva den viser, hvem som bruker den, teksturkontrollen), hver modell (bilde, hva den er, hvor den står, trekanter, teksturlag), forbildene, det som ventes (`model::PLANNED`) og eierens dom. Les den før du bruker eller lager noe. Den lages av `cargo run --release --bin catalog > target/qa/world.json` og `python3 -m tools.qa catalog`; det som ikke kan regnes ut, står i `assets/catalog/textures.json` og `models.json` (hold dem oppdatert, `katalogen_dekker_alt`), og eierens dom i `assets/catalog/approvals.json`, som bare eieren bestemmer.

## Kvalitetsportene og godkjenningssiden

**Kvalitetsportene** (`tools/qa/`, `docs/plan.md` del 3): `python3 -m tools.qa texture [lag]` (P-T: kildens størrelse, ramme og lys; lagets skjøt og skarphet; tersklene kalibrert mot bildene). Lærdommen fra feil står i `docs/kvalitet/laerdom.md`. **Eierens dom** gis på godkjenningssiden på eierens maskin (26. november: den lokale siden tok over for siden på claude.ai, https://claude.ai/artifact/TKUwttgEocYfCZuwWf2sVo, som ikke oppdateres lenger). `sh tools/dom.sh` bygger den i `target/qa/dom/` (`python3 -m tools.qa approvals lokal`, `review.build(local=True)`; `--raskt` hopper over bildene av modulene og mekanismene; første gang lages `target/qa/venv` med Pythonen i Blender.app og `bpy`, Pillow og numpy), og `node tools/rediger.mjs` viser den på http://localhost:1350/dom/. Lokalt har alle kortene bilde, også det som ligger hos meg og det godkjente (bunkene foldes ut, og en dom kan endres når som helst), og dommene over modulene i de gamle temaene vises ikke. Hver dom føres straks inn i `approvals.json` (`POST /lokal/dom` → `python3 -m tools.qa approvals svar`, `approvals.answer`): siden lagrer mens eieren skriver, så svaret føres hver gang inn på det som sto før det første svaret (`base` i `target/qa/dom/svar.json`), som om bare det siste var gitt. Claude bygger siden på nytt etter hver retting og sier fra; siden viser når den ble bygd, og byggingen rydder `svar.json` (`prune_answers`). Det som står under om publisering, asset-lageret og samlingen `dom`, gjelder den gamle siden. Siden viser tilstanden, ikke runder: hver dom gjelder versjonen eieren så (git-blobben, `seen` i `approvals.json`), og alt står i faner per kategori med det som venter på eieren først (ikke vurdert, eller endret siden eieren så det, med før og etter), så det som ligger hos meg (underkjent og ikke rettet) og det godkjente. Ting som er bygd på nytt uten å se annerledes ut, beholder dommen. «Rettet slik» står i `assets/catalog/fixes.json` per versjon. Hver modell som venter, kan ses i 3D («Se i 3D», `tools/qa/viewer.js` med three.js: teksturlagene, før og etter, og kryptbitene inne i hele nivået). Siden bygges med `python3 -m tools.qa approvals review UT` (`tools/qa/review.py`, ett samlet bilde per kort som venter, bare de publiseres), `check UT` stopper en side der noe som venter mangler, og testen `dommen_mister_ingenting` kjører `approvals audit`. Svarene i samlingen `dom` hentes med ArtifactData (`out_dir`) og føres inn med `python3 -m tools.qa approvals apply SVAR`. Siden har plass til 511 filer per versjon, så bildene ligger i sidens asset-lager (siden erklærer `assets`): last dem opp med `asset: true` i bunker på 25 (bare de som mangler), skriv sti → `/_blob/<id>` i en json og kjør `python3 -m tools.qa approvals assets UT MAP`, som peker data.json og glTF-filene dit og sletter bildene; resten (siden, data.json, viewer.js og glTF-filene) publiseres som filer, høyst 255 og 64 MB per publisering, og gamle stier fjernes med `null`. Armene og våpnene vises med `fp_preview.py`/`item_preview.py` som bilder i `fixes.json` (studiobildet utenfra treffer dem ikke). Eieren dømmer ikke løse deler (19. oktober: «Det er 394 deler til godkjenning fortsatt. De fleste løse.»): i `assets/catalog/modules.json` står bitene modulene er bygd av (`gjennom_moduler`), byggesettets mekanismer og pynt (`kit_*`) som dømmes satt sammen på seks kort (`mekanismer`, `u_mek_<kort>`, versjonen er blobene til modellene; bildene fra prøvenivået `assets/levels/nivaa_mekanismer.json` og de gamle nivåene der det trengs en kløft, kanal eller sjakt, med studiobilde eller nærbilde av hver ting under, `python3 tools/crypt/mechanism_shots.py`), og det som utgår (`utgaar`: bitene i de gamle kryptnivåene og reserveveggen `kit_secret_wall`), som står sammenfoldet nederst i Krypten med dommene som er gitt.

## Bygg og Netlify

**Bygg minst mulig på Netlify.** Netlify bygger aldri (`ignore = "exit 0"` i netlify.toml). GitHub Actions (`.github/workflows/web.yml`) bygger Wasm og laster opp `dist/` med Netlify CLI. Push til `master` går til https://fristad.netlify.app, push til `claude/**` går til forhåndsvisningen https://claude--fristad.netlify.app. Repoet er offentlig (eieren 18. oktober), men test lokalt først og push samlet.

## `cargo fmt`

**Kjør aldri `cargo fmt` på hele repoet:** koden står med brede linjer uten `rustfmt.toml`, og `cargo fmt` skriver om titalls filer og sprenger linjetaket (`filene_er_ikke_for_lange`). Skal en fil du har endret formateres, bruk `rustfmt --edition 2021 --config max_width=200,use_small_heuristics=Max,skip_children=true FIL`.

## Test lokalt før push

**Test lokalt før push.** Actions kjører ikke testene (eieren 25. november: de tok 21 av 27 minutter per bygg), så den lokale kjøringen er den eneste. `cargo test --workspace` og `cargo build --bins`, og skjermbilder med `snapshot`. Skyen trenger `apt-get install -y mesa-vulkan-drivers` (lavapipe) først.

`./target/debug/snapshot --out shots/x.png --size 1280x720 --eye 0,+1.7,44 --yaw 0` (`--eye X,+1.7,Z` er høyde over terrenget; yaw 0 ser mot nord (−Z), 90 mot vest, −90 mot øst) [--night] [--weather regn|snø|tåke|uvær|overskyet] [--descend] [--time S] [--frames 8] [--debug albedo|normal|ao|rough|shadow]. Se på bildene før du pusher, og vis eieren skjermbilder underveis.

## Modellene slankes etter eksport

**Modellene slankes etter eksport** (`tools/models/slim.py`, kalles fra alle eksportene): normalene lagres som 8 bit med fortegn (KHR_mesh_quantization, pakkes ut i `packed_normals` i `adapters/gltf.rs`) og hjørnefargen som 8 bit. Nedlastingen er rundt 75 MB gzip; resten er ekte detaljer (Vertshuset er 220 000 trekanter).

## Lydene

**Lydene** bygges med `python3 tools/sounds/build.py` (trenger ffmpeg og numpy; kildene klones til `target/sound-src`). Nye lyder legges til i `SOUNDS` der, med lisensen i `CREDITS.md`, og navnet brukes fra økta. `tools/build-web.sh` kopierer dem til `dist/sounds/`.

## Leveringen: modellene, nivåene og teksturene lastes etter behov

**Modellene er ikke i wasm-fila, og krypten lastes nivå for nivå** (plan del 10a; før krasjet iPhone og WebGL2 stoppet med `unreachable` da wasm-haugen nådde 4 GB). `tools/pack-models.sh` pakker `assets/models/` med gzip til `dist/models/<sti>.gz`, bitene til hvert kryptnivå i en egen mappe `crypt/NN/` (`adapters/delivery.rs`: `level_of`, `dist_path`, `model_url`), og `sizes.json` har dem under `levels` per nivå. `web/main.js` henter ved start alt utenom bitene (`start_paths()`; med `?edit` alt), og `start` får dem (`Files`). Bitene står likevel i verden (`ModelSource::deferred`), og rendereren holder instansene til modellen kommer (`Renderer::add_models`/`drop_models`, `wgpu/models.rs`). Økta sier hvilke nivåer den trenger (`Session::wanted_levels`: det første oppe i byen og det portalen tar deg til, nede nivået du står i og nivåene over og under), og `App` ber verten hente dem. Det er levering og ikke noe som skjer i spillet, så det står ikke blant hendelsene (26. november): verten spør med `App::fetches` (`driving::Fetch::Level(n)` og `Fetch::Models`; i nettleseren `engine.fetches()` etter hvert bilde, `deliver` i `web/loading.js`). Nettlaget henter `level_paths(n)` og svarer med `engine.add_level(n, filer)`, og `App` slipper nivåer du har forlatt (`Pieces`). Utenom nettleseren (`Embedded`, `snapshot`, testene) er alt lastet fra start. Byens modeller som ikke trengs til første bilde hentes også etter start (`delivery::lazy`: innsiden `…_interior`, møblene, galleriplatene, fiendene, armsettene og våpnene, kryptens byggesett og det fiendene mister): `cargo run --release --bin manifest` (i `tools/build-web.sh` og `pull-dist.sh`) skriver `models/manifest.json.gz` med mål, kollidere og skjelett per modell (`delivery::Manifest`, `model::Later`), monteringen bygger verden fra det (`ModelSource::deferred`, `delivery::WithManifest`, `Prepared.rigs`/`later`), og `App` ber om modellene med `Fetch::Models` når de trengs (`app::Lazy`: utstyret du har på straks, innsiden, møblene og galleriet innen `LAZY_NEAR` (30 m), fiendene bare med et nivå som har et merke med slaget (eller nivådokumentet i redigeringen; prøveturen bygger verden med dokumentet som nivå 1), resten med det første kryptnivået); nettlaget henter dem (`lazy_paths`, `fetchLazy`) og svarer med `engine.add_models`. Mangler manifestet, hentes alt ved start. Testene `manifestet_er_likt_glb_filene`, `verden_fra_manifestet_er_lik_verden_fra_glb`, `modellene_hentes_naar_okta_trenger_dem` og `ny_fiende_i_et_nivaa_hentes_med_nivaaet`. Teksturene i glb pakkes ut først når rendereren laster dem opp, én om gangen (`Image.encoded`, `gltf::pixels`), og i nettleseren med tak (`delivery::browser_caps`, hevet 19. oktober da mobil ikke lenger er et mål: fargen 2048, de store fiendene `framework`, `mutant_modal` og `mutant_table` og brønnen `well` 4096, normal- og metallkartene 1024; `FRISTAD_BROWSER_CAPS=1` gir `snapshot` det samme taket). Teksturene lastes lat: først det lave taket `delivery::LOW` (512, data 256; `delivery::stage`, `Image.later`), og etter hvert bilde oppgraderer rendereren ett bilde av gangen til taket over, de nærmeste modellene først (faste og bevegelige instanser) og de uten instanser sist, innenfor `UPGRADE_MS` (4 ms) per bilde i snitt (det som gikk over, betales av de neste bildene), og den lave kopien slippes (`Renderer::upgrade_textures`, `teksturene_oppgraderes_naermest_forst`); bilder over `BIG_SIDE` (Rammeverkets 4096) oppgraderes bare innen `BIG_NEAR` (30 m). `FRISTAD_STAGED=1` gir `snapshot` det samme, og `FRISTAD_UPGRADE_MS=0 … --frames 1` viser det lave, `FRISTAD_UPGRADE_MS=1000000 … --frames 2` det oppgraderte. `tools/browser-test.mjs` skriver wasm-minnet. Testene `nettleseren_far_de_samme_modellene`, `kryptnivaaene_hentes_nivaa_for_nivaa` og `teksturene_har_tak_i_nettleseren`.

## Wasm i skymiljøet

**Wasm kan ikke bygges i Claude sitt skymiljø** (static.rust-lang.org er sperret). Wasm-feil vises i Actions.

## Nettversjonen i Chromium uten skjerm

**Test nettversjonen i Chromium uten skjerm:** Actions legger den ferdige nettsiden på grenen `web-dist`. `sh tools/pull-dist.sh` henter den til `dist-web/`, og `PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright node tools/browser-test.mjs dist-web webgl|webgpu UT.png` kjører den med SwiftShader og skriver konsollen og et skjermbilde. `?webgl` i adressen tvinger WebGL2.

## Shaderne mot Dawn

**Valider shaderne mot Dawn før push:** `node tools/wgsl-check.mjs` (samme PLAYWRIGHT_DIR). Dawn er strengere enn naga: `textureSample` må stå i uniform kontrollflyt, så gradientene tas øverst i fragmentpasset og sendes inn til `textureSampleGrad`. WebGL2 kan ikke lese dybdeteksturer med `textureLoad` og kan ikke binde ett lag av en array som 2D.

## Bare datamaskin

**Bare datamaskin** (eieren 19. oktober: mobil og iPhone er ikke et mål): på en telefon eller et nettbrett (`onPhone` i `web/main.js`: bare grov peker, eller iPhone/iPad/Android) lastes ingenting, og startkortet sier at Fristad spilles på datamaskin (`#touch-note`); `?mobil` laster likevel.

## Nettversjonen og `window.fristad`

**Redigeringen lokalt** (eieren 26. november): `node tools/rediger.mjs --bygg` bygger nettversjonen på Mac (`tools/build-web.sh` henter wasm-bindgen for maskinen) og viser `dist/` på http://localhost:1350, der Lagre i `?edit` og `?nivaa` skriver rett i `assets/layout/` og `assets/levels/` i repoet (`LOCAL` i `web/github.js`). Etter en endring i Rust eller `web/` må det bygges på nytt.

**Nettversjonen:** med `?edit` åpner spillet i redigeringen (`web/editor.js`; `engine.edit(json)`, `catalog()`, `play()`, `view()`, `can_edit()`; testen `tools/edit-test.mjs`). `window.fristad.engine` har `place(x,y,z,yaw,pitch)`, `descend()` (rett ned i krypten), `key(code, down)`, `mouse(knapp, ned)`, `set_paused`, `set_weather(navn)`, `set_theme(mørk)`, `set_quality(navn)`, `set_bench(json)`, `triangles()`, `calls()`, `respawn`, `craft`, `buy`, `read_doc`, `save_state`/`load_state`, og `window.fristad.setQuality(navn)`, `setGraphics(redusert)` og `bench()`.

## Musa i nettleseren

**Musa i nettleseren:** med pekerlås går hvert museknappetrykk rett til spillet (også i HUD-en over bildet, ikke i dialogene). Uten pekerlås (noen nettlesere og jobb-PC-er nekter) drar venstre tast kameraet, et kort klikk uten å dra er et hugg, og høyre tast blokker; `pointerlockerror` gir en melding om dette. Butikken og verkstedet lukkes med «Tilbake» eller E og fanger musa igjen med en gang; Esc gir ingen brukerhandling å låse på, så da kommer en melding om å klikke i bildet.

## LO-TEK

**LO-TEK** (før Fristil, eieren 26. oktober: «Komponentene starter med lt- ikke fs-»): klassene, elementene og variablene heter `lt-…`/`--lt-…`, stilarket er `lotek.css`, og web-komponentene ligger i `dist/lotek/components/frame|standalone/` (`defineLtTabs` …). Pakken `@lo-tek/interfaces` er ikke publisert på npm ennå, så den ligger bygd i `vendor/lo-tek/` (`sh tools/vendor-lotek.sh [klone]`, `"file:./vendor/lo-tek"` i package.json); når den er på npm, byttes avhengigheten og mappa slettes. Fanene er lenker i en liste (`<ul class="lt-tabs__list"><li><a href="#panel">`), og komponentenes standardtekster er engelske, så spillet setter de norske (`label`/`close-label` på `<lt-toast>`, `--lt-label-required-text` i `style.css` og `docs-flate.css`). Tokens: `bun tools/export-tokens.ts` (bare LO-TEK-versjonen brukes i motoren, `engine.lotek_version()`). Bøkene er LO-TEK-dokumentasjonen slik den er, på engelsk (eieren 26. oktober: «i sync med nye utgivelser»): `tools/export-docs.ts` leser sidene fra commiten pakken er bygd fra (`gitHead` i `vendor/lo-tek/package.json`, ellers taggen `v<versjon>`) og skriver `web/docs.json` og `web/docs-flate.css`, og `tools/build-web.sh` kjører det. Rundt boka er spillet norsk; LO-TEKs egne merketekster («(required)») gjelder i eksemplene (`FLATE_EXTRA`).

## Modellene lages i Blender

**Modellene lages i Blender som Python-modul** (`pip install bpy`, Python 3.13). Felles hjelpere i `tools/models/common.py` (`material`, `weather`, `box`, `rounded_box`, `cylinder`, `tube`, `spin`, `bake_textured`, rigging). Hver modell eksporteres til `assets/models/<gruppe>/<navn>.glb` og legges til i `models!`-tabellen i `adapters/gltf.rs` (navnet byen bruker → fila). `tools/models/preview.py modell.glb ut.png` rendrer en forhåndsvisning med Cycles på CPU. Nettverk i skyen: GitHub og crates.io er åpne, det meste annet er sperret.

## Figurer og ting fra forbilder

**Figurer og ting fra forbilder** lages med pakken `sheet2model/` (`python3.13 -m sheet2model <oppslag>`; `sheet2model/README.md` og `PROMPTS.md`; oppslag i `sheet2model/examples/fristad/`; dybdebufferen og modellvektene i `target/sheet2model/cache/`; test `python3.13 -m pytest sheet2model/tests`). Gjenstandene bygges med prioren som geometri og arket projisert (`docs/wip/gjenstander.md`). `tools/reconstruct`, `people.py`, `npc.py` og MakeHuman er slettet.

## Modeller som ventes

**Modeller som ventes** (fiendene, figurene, våpnene, dyrene, plukkene) står i `model::PLANNED` med hvorfor; `place`, `reserve` og `town::figure` melder manglende modeller i `World.missing`, og `alle_modellnavn_finnes_eller_ventes` krever at hvert navn som brukes finnes eller står der (og at det strykes når modellen kommer). `blenderskriptene_henger_sammen` kjører `tools/models/check_scripts.py` (hvert `town.X`, `kit.X` … finnes).

## `[hidden]`

Pass på at `[hidden]` faktisk skjuler: elementer med egen `display` trenger regelen øverst i `web/style.css`.
