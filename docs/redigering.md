# Redigeringsmodusen

Eieren plasserer, flytter og dreier alt i byen og krypten selv, inne og ute, i stedet for at koden setter det ut. Spesifikasjonen bygger på svarene fra eieren 11. oktober. Den bygges etter at husene er rettet (bindingsverket, etasjeskillet, grunnmuren, takene).

## For hvem og hvor

- Bare for eieren: verktøyet åpnes med `?edit` i adressen på forhåndsvisningen (https://claude--fristad.netlify.app/?edit). Spillerne ser det aldri.
- Det gjelder byen, husene inne og ute, torget, gatene, gårdene, naturen og kryptnivåene. Murene og bitene i krypten lages fortsatt med byggesettet; det som står i dem, kan flyttes.

## Hva du gjør

- **Flyge:** i redigeringen flyr du fritt og går gjennom alt, også usynlige vegger. Mellomrom stiger, C synker, Shift går fort.
- **Katalogen:** et Fristil-panel med alle modellene, bilde og eierens dom (godkjent, underkjent, usikker), med filter. Underkjente ting kan plasseres for å prøve, men Lagre nekter dem til de er godkjent.
- **Plassere:** velg en ting i katalogen. Den følger siktekorset og legger seg på flaten under: bakke, gulv, bord, eller mot en vegg. Klikk setter den fra deg.
- **Velge og flytte:** klikk på noe som står for å velge det. Shift-klikk velger flere. Det valgte kan:
  - dras med håndtak (piler i tre farger langs aksene, og en ring for dreiing),
  - finjusteres med tastene: piltastene flytter langs bakken, PageUp og PageDown flytter opp og ned, Q og E dreier. Små steg holder du Alt for.
  - kopieres (Ctrl+D) og slettes (Delete).
- **Linje:** klikk punkter på bakken. Langs linja blir det gjerde, usynlig vegg (3 m høy som standard, høyden kan endres) eller en rad av den valgte tingen, og den følger terrenget.
- **Usynlige vegger** vises som gjennomsiktig rødt i redigeringen og kan flyttes og strekkes som andre ting.
- **Ting med funksjon** plasseres som andre ting og virker der de står:
  - lys: lykter, lys og ildsteder lyser og blafrer,
  - ting du bruker: kister, brønner, dører, senga, galleriveggen,
  - innbyggerne: hvor hver står, og punktene på runden deres.
- **Hele hus** flyttes og dreies som andre ting. Det som står inne i et hus, er lagret i husets eget rom og følger med.
- **Angre** med Ctrl+Z og gjøre om med Ctrl+Shift+Z, for alt.
- **Spill med en gang:** én tast (P) bytter mellom redigering og vanlig spill på samme sted, så du kan gå rundt og kjenne på det.

## Reglene

Kontrollene er de samme som i `cargo test`:
- svever eller står nede i bakken, over hele fotavtrykket,
- står inni en vegg eller går inn i noe annet,
- husene holder avstand til hverandre og til muren,
- alt stort har kollider,
- gjenstanden er godkjent.

En ting som bryter en regel, lyser rødt med grunnen mens du flytter den. Lagre nekter til alt er grønt, så det som lagres, alltid går gjennom testene.

## Lagring

- Første gang limer du inn en GitHub-nøkkel som bare gjelder fristad-repoet. Den huskes bare i din nettleser.
- Lagre skriver datafilene som én commit på grenen `redigering`. Den grenen bygges ikke, så det koster ingen Actions-minutter.
- Jeg kjører kontrollene og slår endringene inn i `claude/detaljnivaa` samlet, og da bygges forhåndsvisningen én gang.
- Dataene er tekst i repoet, én fil per område:
  - `assets/layout/town.json`: byen, med husene, gatelivet, naturen, de usynlige veggene og innbyggerne,
  - `assets/layout/inside/<hus>.json`: det som står inne i hvert hus, i husets eget rom,
  - `assets/layout/crypt_NN.json`: det som står i hvert kryptnivå.

## Arkitekturen (heksagonal)

- **Kjernen** (`fristad-domain`):
  - byen som data: `Layout` med plasseringer (modell, sted, dreining, forelder: verden eller et hus), usynlige vegger og innbyggerruter,
  - redigeringen som ren logikk: velg, flytt, dreie, kopier, linje, angre,
  - reglene som funksjoner over `Layout` og terrenget.

  Ingen nettleser, GitHub eller filer i kjernen. `town.rs` slutter å plassere ting: dagens by skrives ut én gang som data, og koden som satte den ut, fjernes.
- **Portene:**
  - `Command::Edit(...)` inn og `Event::Edit(...)` ut (valget, reglene som brytes, angre-stabelen),
  - `ModelSource::world_data` gir også `Layout`,
  - en ny drevet port `LayoutStore` lagrer.
- **Applikasjonen:** en redigeringsmodus i økta. Den tar imot kommandoene, oppdaterer `Layout`, bygger verden på nytt der det trengs og sender tilstanden ut. Flyging uten kollisjon gjelder bare i den modusen.
- **Adapterne:**
  - nettlaget: Fristil-panelet, håndtakene tegnet over bildet, tastene,
  - `LayoutStore` for GitHub i nettleseren,
  - filene i `adapters/data.rs`,
  - rendereren tegner håndtak og røde markeringer fra `Frame`.

## Rekkefølgen

1. Byen og kryptens innhold skrives ut som data, og koden som plasserte dem, fjernes. Spillet ser likt ut før og etter, og testene viser det.
2. Møblene og gjenstandene skilles ut av innsidemodellene som egne modeller i katalogen. Husene står igjen med vegger, gulv, ildsted og trapper.
3. Redigeringsmodusen i kjernen og applikasjonen, med tester: velg, flytt, regler, angre.
4. Nettlaget: flyging, katalogen, plassering på flaten, taster, håndtak, linje, P for å spille.
5. Lagring til GitHub.
6. Eieren prøver i Vertshuset og på torget.

## Status

- **Steg 1 (12. oktober):** byen er data i `assets/layout/town.json` (`domain/world/layout.rs`: `Layout`, `Item`, `NpcDef`, `apply`). Fila ble skrevet ut fra koden i `town.rs` og prøvd mot den ting for ting (modellene, kolliderne, siktemålene, lysene, røyken, innbyggerne og bakken var like), og så ble koden som plasserte byen fjernet (`town.rs` fra 1705 til 606 linjer). Et hus er én ting (`building`) og tar med seg det som står i `<navn>_inside.json`; det skilles ut i steg 2. Innholdet i krypten er allerede data: merkene (kister, spaker, porter …) i `assets/models/crypt/crypt_NN.json`, som redigeringen skriver til i stedet for en egen fil. Stiene ut til gårdene følger ennå de faste gårdsstedene (`town::farms`), ikke husene i fila.
- **Steg 2 (12. oktober):** møblene er egne modeller. Rundt møbelkoden i Blender-skriptene står `with town.piece(navn, sted, yaw):`; det som lages i blokka (delene, kolliderne og lysene), tas ut av innsidemodellen og blir `assets/models/furniture/<navn>.glb` med `<navn>_inside.json` i møbelets eget rom, og stedet skrives til `assets/layout/inside/<hus>.json` (bare om fila ikke finnes: etter første gang er fila sannheten; `FRISTAD_CONTENTS=skriv` og `FRISTAD_PIECES=alle` tvinger). Byen setter dem ut med huset (`town::building_at`, `layout::apply_item`), og hver modell tar med seg kolliderne og lysene sine (`town::carry`). Eieren valgte «møbel med innhold»: en hylle med krukkene sine, et bord med lyset sitt, en seng med dyna. 81 møbler i seks hus. Det som blir stående i husene: vegger, gulv, bjelker, trapper og rekkverk, ildsted og ovn med ilden, galleriveggen, krusifikset, urtebuntene fra bjelkene og verktøyet på ambolten. Senga i spillerens hus har med seg det du sikter på for å sove. Testen `moblene_er_egne_modeller`.
