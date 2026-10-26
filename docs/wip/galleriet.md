# Galleriveggen: bildet av komponenten i rammen

Eieren: det som henger på trofeveggen skal se ut som selve LO-TEK-komponenten i standardvarianten, inne i rammen.

## Gjort

- `tools/gallery/render.mjs`: Playwright tar `demo`-HTML-en fra `web/docs.json` med `lotek.css` og komponent-JS-en fra `dist-web/` (etter `sh tools/pull-dist.sh`), i lyst tema, skåret ned til standardvarianten (første knapp, første tag osv.; tooltip tvinges synlig, dialogen åpnes med `showModal`), 20rem bred, device scale 2, beskåret til innholdet → `target/gallery/<komponent>.png` + `meta.json`.
- `tools/gallery/pack.py`: kvadratiske 512×512 → `assets/textures/components/<komponent>.jpg`, og kontaktarket `shots/galleriet/kontaktark.png`. Etter koordinatorens innspill: skjermbildet tas med gjennomsiktig bakgrunn (device scale 4), innholdet skaleres så den lengste sida er 80 % av platen, og legges på pergament (#f3ead8 med myk flekkete tekstur og mørkere kanter), så platen ser ut som et oppslag.
- `tools/models/home.py`: `gallery_piece(name)` er nå en plate med bildet som albedo (uv 0..1 over forsida, sidene peker i polstringen) og gullkant; `gallery_pieces()` eksporterer `gallery_<komponent>.glb` for hvert bilde (`python3 tools/models/home.py pieces`).

- 15 modeller eksportert: `assets/models/town/gallery_<komponent>.glb` (60 trekanter, 8–52 kB hver); `gallery_piece.glb` er slettet.
- Rust: `craft::GALLERY_MODELS` (erstatter `GALLERY_TINTS`), `Session::gallery_poses` gir `(modell, pose)`, `view.rs` tegner uten tint, `world::build` setter av én plass per modell, `models!`-tabellen har de 15, testen `funnet_henger_i_galleriet` sjekker også at det er riktig modell som henger. `GALLERY_PIECE` er borte. CLAUDE.md oppdatert i avsnittet «Spillerens hus».

- Teksturene er jpeg (kvalitet 90), så hver plate er 45–55 kB (png av pergamentet ga 260 kB); de 15 modellene er 700 kB til sammen.
- Skjermbilder: `shots/galleriet/galleriveggen_1024.png` (hele veggen med alt funnet, `snapshot --gallery --eye -6.9,2.3,15.6 --yaw -90`) og `naer_button_1024.png` (nært, `--eye -5.3,3.0,13.64`): bildet står riktig vei, i riktig ramme, og innholdet fyller rundt 80 % av platen.

## Gjenstår / neste

- Testene `funnet_henger_i_galleriet`, `rammen_apner_boka`, `alt_er_ekte_modeller_med_teksturer`, `alt_stort_har_kollider`, `nettleseren_far_de_samme_modellene` er grønne (første eksport). Modellene eksporteres på nytt med pergamentteksturene; så full `cargo test --release` og skjermbilde inne i huset mot galleriveggen (`snapshot --gallery`, kamera fra `galleriveggen_star`: rammene står ved x ≈ −4,2, z 13,6–17,6, y 1,5/2,25/3,0 og vender mot −X, så øyet står ved −6.8,2.25,15.6 med yaw −90).
