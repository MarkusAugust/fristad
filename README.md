# Fristad

Dokumentasjonen til [Fristil](https://github.com/MarkusAugust/fristil), som en by du kan gå rundt i.

Fristad er en 3D-verden i nettleseren der komponentene i designsystemet står som ting i byen: knappen ved porten er `.fs-button`, oppslaget på veggen er `.fs-alert`, bryteren for gatelysene er `.fs-switch`, framdriftsstolpen over porten er `.fs-progress`, merkelappene og statusmerkene på oppslagstavla er `.fs-tag` og `.fs-badge`, ringen på masta er `.fs-spinner`, messingplatene i brosteinen er `.fs-breadcrumbs`, huset under bygging er `.fs-skeleton` og masta ved torget er `.fs-connection-status`. Telefonkiosken er `.fs-dialog`, huset med tre dører ved Østre gate er `.fs-tabs`, infotavla og oppslagsskapet ved Vestre gate er `.fs-card` og `.fs-accordion`, søknadskontoret har skjemafeltene (`.fs-input`, `.fs-select`, `.fs-checkbox`, `.fs-radio`, `.fs-textarea`) med `.fs-tooltip`, `.fs-avatar` og `.fs-divider` rundt seg, rutetabellen i busskuret er `.fs-table` og veiviseren på torget er `.fs-link`. Vedleggsarket på baksiden av søknadskontoret har `.fs-heading`, `.fs-label`, `.fs-help-text`, `.fs-error-text`, `.fs-fieldset`, `.fs-file-upload` og `.fs-search`. Lenker, brødsmuler og «Torget»-skiltene utenfor torget tar deg dit med flytur: løftet over hustakene, fløyet bort og dalt ned i fallskjerm. Sikt på en av dem og trykk **E**, så åpnes den ekte komponenten med eksempel, varianter, kode og lenke til dokumentasjonssiden. **R** bytter variant, også i byen. **F** bruker den: knappen åpner porten, bryteren tenner gatelysene. **T** bytter mellom lyst og mørkt tema, som i Fristad er dag og natt, og **V** bytter vær.

Utenfor byen ligger tre bydeler: **Tokenparken** vest for torget, der fargepalettene står som blomsterbed og avstandsskalaen som steinblokker; **Tårnene** nord for Nordgata, åtte tårn med høyde etter skriftstørrelsene; og **Verkstedet** sør for Sørgata, der veiledningssidene «Kom i gang», «Typesikker bruk» og «Tilgjengelighet» ligger på arbeidsbenkene.

## Kontroller

| Tast | Gjør |
| --- | --- |
| W A S D | Gå |
| Mus | Se |
| Shift | Løp |
| Mellomrom | Hopp |
| C | Huk (hold inne) |
| X | Kryp av og på |
| H | Tilbake til torget med flytur (også knappen i pausemenyen) |
| E | Inspiser komponenten du sikter på |
| R | Neste variant |
| F | Bruk |
| T | Dag og natt (lyst og mørkt tema) |
| V | Vær: klart, overskyet, regn, uvær, snø, tåke |
| Esc | Pause |

## Hvordan den henger sammen med Fristil

- **Tokens.** `tools/export-tokens.ts` leser `tokens.css` fra `@fristil/designsystem` og skriver fargene for begge temaene og spacing-skalaen til `assets/tokens.json`, som motoren bygger inn. Fasadefargene, dørene og knappen kommer derfra.
- **Målestokk.** 1 rem er en meter. `spacing-12` er etasjehøyden, `spacing-6` vindushøyden og `spacing-4` vindusbredden.
- **Tema.** Lyst tema er dag og mørkt tema er natt. Knappen bruker mørkt temas tokens om natta, akkurat som på en nettside.
- **Dokumentasjon.** `tools/export-docs.ts` henter beskrivelse, ingress og eksempelkode fra MDX-sidene i Fristil til `web/docs.json`.
- **Ekte komponenter.** Panelet bruker `fristil.css` fra pakken, så knappene der er de samme som i produksjon.

## Teknologi

Motoren er skrevet i Rust og kompilert til WebAssembly, med hexagonal arkitektur (ports and adapters):

- `src/domain/` er kjernen, uten avhengighet til GPU, nettleser eller fysikkmotor: tokens, geometri, tekst, byen, komponentene i byen og spillet.
- `src/domain/ports.rs` er portene (`Renderer`, `Physics`, `ModelSource`), og `src/adapters/` er adapterne: wgpu (WebGPU, med WebGL2 som reserve), Rapier, glTF, nettleser og eget vindu.
- Rendering: tre kaskader med skyggekart, dybdeprepass, SSAO, PBR-lys med normalkart, himmellys, lykter, vinduer med rom bak glasset, volumetrisk tåke med lysstråler, regn og snø, våte flater og snødekke, TAA, glød og ACES-tonemapping.
- Materialene bakes prosedyrisk på GPU-en ved oppstart (brostein, puss, panel, skifer, takstein, stein, mur, eik, metall, gress, bark, grus, betong) som teksturer med normal-, ruhets- og høydekart. Byen laster ingen bildefiler; bare modellene (lyktene og reven, CC0 fra Khronos) har egne teksturer.
- Teksten på skiltene er Inter som SDF-atlas, så den er skarp uansett avstand. Én CSS-piksel er én centimeter for komponentene.

```
src/
  domain/          kjernen
    city/          byen: fasadegrammatikk, trær, gatemøbler
    installations/ komponentene i byen, én fil per komponent
    game.rs        spilleren, døgnet, været
    ports.rs       portene
  adapters/        wgpu, rapier, gltf, web, native
  app.rs           setter sammen kjernen og portene
  bin/snapshot.rs  skjermbilde til PNG uten vindu
  bin/bake_font.rs SDF-atlas av Inter
web/               HTML, JavaScript-laget og panelene
tools/             eksport av tokens og dokumentasjon, og nettbygget
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

GitHub Actions bygger og publiserer `dist/` ved hver push: `master` til fristad.netlify.app, `claude/**` til forhåndsvisningen claude--fristad.netlify.app.
