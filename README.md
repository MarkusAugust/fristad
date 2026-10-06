# Fristad

Dokumentasjonen til [Fristil](https://github.com/MarkusAugust/fristil), som en by du kan gå rundt i.

Fristad er en 3D-verden i nettleseren der komponentene i designsystemet står som ting i byen. Den store knappen ved porten er `.fs-button`: sikt på den og trykk **E**, så åpnes den ekte komponenten med varianter, kode og lenke til dokumentasjonssiden. **R** bytter variant, også på knappen i byen. **F** trykker på den, og porten går opp. **T** bytter mellom lyst og mørkt tema, som i Fristad er dag og natt.

## Kontroller

| Tast | Gjør |
| --- | --- |
| W A S D | Gå |
| Mus | Se |
| Shift | Løp |
| Mellomrom | Hopp |
| E | Inspiser komponenten du sikter på |
| R | Neste variant |
| F | Bruk |
| T | Dag og natt (lyst og mørkt tema) |
| Esc | Pause |

## Hvordan den henger sammen med Fristil

- **Tokens.** `tools/export-tokens.ts` leser `tokens.css` fra `@fristil/designsystem` og skriver fargene for begge temaene og spacing-skalaen til `assets/tokens.json`, som motoren bygger inn. Fasadefargene, dørene og knappen kommer derfra.
- **Målestokk.** 1 rem er en meter. `spacing-12` er etasjehøyden, `spacing-6` vindushøyden og `spacing-4` vindusbredden.
- **Tema.** Lyst tema er dag og mørkt tema er natt. Knappen bruker mørkt temas tokens om natta, akkurat som på en nettside.
- **Dokumentasjon.** `tools/export-docs.ts` henter beskrivelse, ingress og eksempelkode fra MDX-sidene i Fristil til `web/docs.json`.
- **Ekte komponenter.** Panelet bruker `fristil.css` fra pakken, så knappene der er de samme som i produksjon.

## Teknologi

Motoren er skrevet i Rust og kompilert til WebAssembly:

- **wgpu** mot WebGPU, med WebGL2 som reserve.
- **Rapier** for fysikk og karakterkontroller.
- Ingen modeller eller teksturer: byen bygges prosedyrisk i `src/world.rs`, og brostein, puss, mur, skifer og treverk tegnes i shaderne.
- Rendering: skyggekart med myke kanter, PBR-lys, himmellys, punktlys fra lyktene, høydetåke, himmel med sol, stjerner og nordlys, glød og ACES-tonemapping.

```
src/
  world.rs      byen, bygget fra tokens
  game.rs       spilleren, fysikken, døgnet og knappen
  render.rs     rendereren
  shaders/      WGSL
  web.rs        inngangen fra nettleseren
  native.rs     eget vindu på Mac, Windows og Linux
  bin/snapshot.rs  skjermbilde til PNG uten vindu
web/            HTML, JavaScript-laget og panelene
tools/          eksport av tokens og dokumentasjon, og nettbygget
```

## Kjøre lokalt

```sh
bun install && bun run tokens

# Eget vindu:
cargo run --release

# Skjermbilde uten vindu:
cargo run --release --bin snapshot -- --out torg.png --theme dark

# Nettversjonen til dist/:
sh tools/build-web.sh
```

Netlify bygger og publiserer `dist/` ved hver push til `master`.
