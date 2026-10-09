# Fristad

Et spill i en nordisk kjøpstad rundt 1350, i nettleseren. Under borgruinen nord for byen ligger krypten, og der nede holder de korrupte komponentene fra designsystemet [Fristil](https://github.com/MarkusAugust/fristil) til: buttons med tenner, tooltips i flokk, inputer som spytter markører, modaler med knappenever, tabellen som vokter nøkkelen, og nederst Rammeverket.

Du går inn sørporten, gjennom Storgata til torget med markedsbodene, og opp til borgen. Komponentene du vinner i krypten smis til våpen, skjold og rustning hos Buttonsmeden og i Skriptoriet, der oppskriftene er Fristil-skjemaer. Bøkene om komponentene er dokumentasjonen, og å lese dem leger sårene. Dør du, er det du bar igjen nede.

Alt du ser er egne modeller laget i Blender med håndlagde teksturer, og motoren er skrevet fra bunnen i Rust.

## Kontroller

| Tast | Gjør |
| --- | --- |
| W A S D | Gå |
| Mus | Se |
| Shift | Løp, og med en retning i krypten: unnamanøver |
| Mellomrom | Hopp |
| C | Huk (hold inne) |
| X | Kryp av og på |
| F | Bruk det du sikter på: snakk, åpne, lås opp, gå ned eller opp |
| E | Handle med innbyggeren du sikter på |
| Venstre mus | Hugg, hold for tungt hugg |
| Høyre mus | Blokk, parade i det bittet kommer |
| H | Tilbake til sørporten |
| T | Dag og natt |
| V | Vær: klart, overskyet, regn, uvær, snø, tåke |
| Esc | Pause |

## Teknologi

Motoren er skrevet i Rust og kompilert til WebAssembly, med hexagonal arkitektur (porter og adaptere):

- `src/domain/`: kjernen. Byen, terrenget, naturen, krypten, kampen, fiendene, smiinga og innbyggerne, uten avhengigheter til motor eller nettleser.
- `src/ports/`: grensesnittene. Rendereren, fysikken og modellkilden som applikasjonen bruker, og den drivende porten (`Play`) som nettleseren, vinduet og skjermbildeverktøyet bruker.
- `src/application/`: setter kjernen sammen med portene.
- `src/adapters/`: wgpu (WebGPU, med WebGL2 som reserve), Rapier (fysikk), glTF (modellene), nettleseren og eget vindu.

En test (`src/architecture.rs`) leser kildefilene og feiler om et lag bruker noe det ikke skal.

Rendereren har kaskadeskygger, SSAO, TAA, volumetrisk tåke med lysstråler, regn, snø, våte flater og ekte skjelettanimasjon. Modellene og teksturene lages med skriptene i `tools/models/` i Blender (som Python-modul) og bygges inn i binæren.

## Utvikling

```sh
cargo test                     # testene, også arkitekturen
cargo run --release            # eget vindu
cargo run --release --bin snapshot -- --out bilde.png --eye 0,+1.7,44
sh tools/build-web.sh          # nettversjonen til dist/
python3 tools/models/mobs.py   # fiendene i Blender
```

Se `CLAUDE.md` for detaljene og `docs/spilldesign.md` for spillet.
