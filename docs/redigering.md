# Redigeringsmodusen

Eieren plasserer, flytter og dreier alt i byen og krypten selv, inne og ute, i stedet for at koden setter det ut. Spesifikasjonen bygger på svarene fra eieren 11. oktober. Den bygges etter at husene er rettet (bindingsverket, etasjeskillet, grunnmuren, takene).

## For hvem og hvor

- Bare for eieren: verktøyet åpnes med `?edit` i adressen på forhåndsvisningen (https://claude--fristad.netlify.app/?edit). Spillerne ser det aldri.
- Det gjelder byen, husene inne og ute, torget, gatene, gårdene, naturen og kryptnivåene. Murene og bitene i krypten lages fortsatt med byggesettet; det som står i dem, kan flyttes.

## Hva du gjør

- **Flyge:** i redigeringen flyr du fritt og går gjennom alt, også usynlige vegger. Mellomrom stiger, C synker, Shift går fort.
- **Katalogen:** et Fristil-panel med alle modellene, bilde og eierens dom (godkjent, underkjent, usikker), med filter. Underkjente ting kan plasseres for å prøve, men Lagre nekter dem til de er godkjent.
- **Plassere:** dra en ting fra katalogen rett ut i bildet. Et fantom av modellen følger pekeren langs flaten under (bakke, gulv, bord) med en grønn boks rundt når alt er i orden og en rød når det står inni noe, svever eller ikke er godkjent; panelet sier hvorfor. R dreier fantomet en kvart runde (Shift+R den andre veien), og slipp setter tingen ned med bunnen på flaten. Et klikk i katalogen og så klikk i bildet virker som før (Shift setter ut flere). Tingen får en kollider rundt modellen når den er stor nok til at regelen krever det, så den er grønn med en gang.
- **Fest:** bryteren «Fest» i panelet, eller Ctrl holdt, setter det du setter ut og drar på rutenettet (0,5 m) og vinkelen (15°).
- **Velge i dybden:** klikk på noe for å velge det. Ligger flere ting under pekeren (kirkas boks dekker gjerdet inni den), står de i en liste i panelet, nærmest først, og et klikk i lista velger (Shift legger til). Klikk igjen på samme sted eller Tab går til det neste bakover, Shift+Tab til det forrige, og Alt+klikk (uten å dra) tar det bakerste med en gang.
- **Velge mange:** Shift-klikk velger flere, og Shift+dra i bildet trekker en ramme som velger alt med midten innenfor.
- **Flytte:** det valgte kan:
  - dras med musa (eieren 12. oktober): trykk på tingen og dra, så følger den pekeren langs bakken, og det som sto på noe, settes ned igjen der du slipper (i samme angresteg); hold Alt og dra for å heise den opp og ned, og Ctrl (Cmd på Mac) for å dreie den. Med Alt og Ctrl gjelder draget det valgte uansett hvor i bildet du drar. Pekeren viser hva draget gjør. Hvert drag angres for seg,
  - dras med håndtak (piler i tre farger langs aksene, og en ring for dreiing); står pekeren på tingen, er det tingen du tar, ikke pila som går gjennom den,
  - flere valgt (et gjerde i biter) dreies samlet om midten, som én ting,
  - finjusteres med tastene: piltastene flytter langs bakken, PageUp og PageDown flytter opp og ned, Q og E dreier. Små steg holder du Alt for.
  - kopieres (Ctrl+D) og slettes (Delete).
- **Linje:** klikk punkter på bakken. Langs linja blir det gjerde, usynlig vegg (3 m høy som standard, høyden kan endres) eller en rad av den valgte tingen, og den følger terrenget.
- **Usynlige vegger** vises som gjennomsiktig rødt i redigeringen og kan flyttes og strekkes som andre ting.
- **Ting med funksjon** plasseres som andre ting og virker der de står:
  - lys: lykter, lys og ildsteder lyser og blafrer,
  - ting du bruker: kister, brønner, dører, senga, galleriveggen,
  - innbyggerne: hvor hver står, og punktene på runden deres.
- **Hele hus** flyttes og dreies som andre ting. Det som står inne i et hus, er lagret i husets eget rom og følger med.
- **Egenskaper:** panelet viser egenskapene til det valgte, og du skriver inn verdiene (Enter eller gå ut av feltet setter dem): x, y og z i verden og dreiningen i grader for alt, fargen for tingene og innbyggerne (fargevelger eller #rrggbb), navnet og replikknøkkelen for innbyggerne, og for merkene i krypten det slaget bruker: spakens «åpner» (id-ene, skilt med komma), portens id, bredde og arena, lappens tittel og tekst, kistens innhold, fiendens slag, antall og runde (punktene x,y,z i nivåets rom, skilt med semikolon), snarveiens rom og hemmelige veggers mål. Lysene i krypten har farge, rekkevidde, styrke og om det blafrer. En gal verdi gir en melding og endrer ingenting. Stedet og dreiningen settes på én ting om gangen; resten på alle som er valgt.
- **Angre** med Ctrl+Z og gjøre om med Ctrl+Shift+Z, for alt, også egenskapene. Historikken står som en liste i panelet («Flyttet barrel», «Satte ut bench», «Endret lever (…)»), og et klikk på et steg angrer eller gjør om dit.
- **Kamera:** Ctrl+1–9 husker hvor du står og ser, 1–9 tar deg dit (i nettleseren, `fristad.edit-marks`).
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
- Uten nøkkel laster Ctrl+S ned filene som en zip med stiene som i repoet (også knappen «Last ned zip» i lagringsdialogen), så de kan legges inn for hånd eller sendes til Claude.
- Lagre (Ctrl+S) skriver datafilene på grenen `redigering`, uten spørsmål når nøkkelen er limt inn én gang. Alle lagringene i én økt (én fane) blir én commit: lagrer du igjen, erstattes den. Den grenen bygges ikke, så det koster ingen Actions-minutter.
- Det du gjør, huskes som et utkast i nettleseren ved hver endring og hvert 30. sekund, så ingenting går tapt om fanen lukkes. Neste gang du går inn i redigeringen, fortsetter du fra utkastet, eller fra det som ligger på grenen `redigering` (`engine.open_files`). Et utkast fra en eldre utgave av byen legges bort.
- Når Claude har slått grenen inn i spillet, slettes `redigering`, så neste lagring begynner på den nye byen.
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

- **Steg 1 (12. oktober):** byen er data i `assets/layout/town.json` (`domain/world/layout.rs`: `Layout`, `Item`, `NpcDef`, `apply`). Fila ble skrevet ut fra koden i `town.rs` og prøvd mot den ting for ting (modellene, kolliderne, siktemålene, lysene, røyken, innbyggerne og bakken var like), og så ble koden som plasserte byen fjernet (`town.rs` fra 1705 til 606 linjer). Et hus er én ting (`building`) og tar med seg det som står i `<navn>_inside.json`; det skilles ut i steg 2. Innholdet i krypten er allerede data: merkene (kister, spaker, porter …) i `assets/models/crypt/crypt_NN.json`; redigeringen skriver dem til `assets/layout/crypt_NN.json` (se «Krypten» under). Stiene ut til gårdene følger ennå de faste gårdsstedene (`town::farms`), ikke husene i fila.
- **Steg 2 (12. oktober):** møblene er egne modeller. Rundt møbelkoden i Blender-skriptene står `with town.piece(navn, sted, yaw):`; det som lages i blokka (delene, kolliderne og lysene), tas ut av innsidemodellen og blir `assets/models/furniture/<navn>.glb` med `<navn>_inside.json` i møbelets eget rom, og stedet skrives til `assets/layout/inside/<hus>.json` (bare om fila ikke finnes: etter første gang er fila sannheten; `FRISTAD_CONTENTS=skriv` og `FRISTAD_PIECES=alle` tvinger). Byen setter dem ut med huset (`town::building_at`, `layout::apply_item`), og hver modell tar med seg kolliderne og lysene sine (`town::carry`). Eieren valgte «møbel med innhold»: en hylle med krukkene sine, et bord med lyset sitt, en seng med dyna. 80 møbler i seks hus, alle godkjent av eieren (runde 5–7, 12. oktober). Det som blir stående i husene: vegger, gulv, bjelker, trapper og rekkverk, ildsted og ovn med ilden, galleriveggen, krusifikset, urtebuntene fra bjelkene og verktøyet på ambolten. Senga i spillerens hus har med seg det du sikter på for å sove. Testen `moblene_er_egne_modeller`.
- **Steg 3 (12. oktober):** redigeringen er ren logikk i kjernen (`domain/edit/`) og i økta (`application/editing.rs`), med tester i `src/edit_tests.rs`.
  - **Hvilken ting:** `layout::Ref` (en ting i byen, et møbel i et hus eller en innbygger). `Builder` noterer hvilken ting hver modellinstans og hver kollider kom fra (`origins`, `solid_origins`, `World.origins`).
  - **`Editor`:** velge med en stråle (inne i et hus treffes møblene, ikke huset), flytte, dreie, sette en pose, sette ned på flaten under, kopiere, slette, sette ut en ny ting (inne i et hus lagres den i husets rom og følger huset), linje (en rad av en modell eller en usynlig vegg langs terrenget), høyden på usynlige vegger, og angre og gjøre om alt. Byen bygges på nytt fra dataene etter hver endring (`town_scene`, 17 ms i release).
  - **Reglene** (`edit/rules.rs`), de samme som testene: svever eller står nede (`Builder::ground_findings`, som `audit` bruker), alt stort har kollider (`lacks_collider`, som `alt_stort_har_kollider` bruker), står inni noe (biter som skjøtes, gjerdene og muren, er unntatt), husene holder avstand til hverandre, muren, gatene og torget (`EAVES`, `AIR`, `Rect`, hentet tilbake fra den gamle plasseringskoden), og godkjent (`approvals.json`, bare for det som er endret). Dagens by bryter ingen regler (`dagens_by_folger_reglene`).
  - **Portene:** `Command::Edit(EditCommand)`, `Event::Edit(EditView)`, `Frame.hidden` (det som er flyttet skjules der det sto og tegnes der det står nå) og lagringsporten `LayoutStore`.
  - **Økta:** i redigeringen flyr du gjennom alt. `prepare_for_edit` laster alle modellene med plass til å flytte dem (`EDIT_RESERVE`), og `prepare_with` bygger spillet på nytt fra de redigerte dataene (P).
- **Steg 4 (12. oktober):** redigeringen i nettleseren, med `?edit` i adressen.
  - **Nettlaget** (`web/editor.js`): et Fristil-panel til venstre med tilstanden (grønt eller antall brudd), Angre, Gjør om, Spill (P) og Lagre (kommer i steg 5), det valgte, reglene som brytes, og katalogen. Katalogen har bilde, navn og eierens dom for hver modell, med søk og filter på dom og gruppe. Bildene lages med `tools/qa/thumbs.py` til `assets/catalog/thumbs/` og kopieres til `dist/catalog/`.
  - **Musa er fri:** venstre klikk velger det under pekeren (Shift legger til), eller tar tak i et håndtak og drar. Håndtakene er pilene i rødt, grønt og blått langs aksene og den gule ringen for dreiing, og et drag angres som én handling. Høyre knapp holdt nede ser rundt.
  - **Tastene:**
    - WASD, mellomrom og C flyr, og Shift går fort.
    - Piltastene flytter langs bakken etter hvor du ser, og PageUp og PageDown flytter opp og ned (Alt gir små steg, Shift store).
    - Q og E dreier, og G setter det valgte ned på det som er under.
    - Ctrl+D kopierer, Delete sletter, og Ctrl+Z eller Ctrl+Shift+Z angrer og gjør om.
    - L starter linjeverktøyet: klikk punkter, så gir Enter en rad av modellen valgt i katalogen og V en usynlig vegg.
    - P spiller.
  - **Bildet:** rendereren skjuler de faste instansene i `Frame.hidden` (plassen i bufferet byttes, `slot_of`) og tegner linjene i `Frame.lines` uten dybdetest. Det valgte vises med gule bokser, det som bryter en regel med røde, og håndtakene og linjeutkastet tegnes også.
  - **P:** spillet bygges på nytt fra de redigerte dataene på samme grafikkort (`Engine::play`, `prepare_with`), så kolliderne, lysene og alt stemmer, og du spiller der du sto. P igjen går tilbake i redigeringen. Det som er endret siden fila, må fortsatt være godkjent (`EditData.baseline`).
  - **Testene:** `handtakene_drar` (pila flytter bare langs aksen, draget angres i ett steg, ringen dreier) og `bildet_viser_det_flyttede`. `snapshot --edit [--edit-move X,Y,Z]` viser det valgte og flyttede, og `tools/edit-test.mjs` går gjennom redigeringen i nettleseren.
- **Steg 5 (12. oktober):** Lagre (Ctrl+S) gir filene fra motoren (`engine.save_files()`, `data::layout_files`: byen, hvert hus og hvert kryptnivå i fast form) og gjør dem til én commit på grenen `redigering` med GitHubs Git Data API (`web/github.js`). Nøkkelen er en finkornet personlig nøkkel med «Contents: Read and write» bare på fristad-repoet, og huskes bare i nettleseren. Lagre nekter så lenge en regel er brutt. Claude kjører kontrollene på grenen og slår den inn. Testen `lagringen_gir_filene`.
- **Krypten (12. oktober):** det som står i kryptnivåene (merkene: kister, spaker, porter, hvilesteder, fiender, lapper …, og lysene) ligger i `assets/layout/crypt_NN.json` og går foran det i nivåfila (`data::world_data`; `build_level.py` leser fila og skriver den bare når den mangler, så navigasjonsnettet bygges etter det som er redigert). P nede i krypten redigerer nivået du står i: du flyr fritt i nivået, et klikk velger et merke (`Ref::Crypt`, lysene `Ref::CryptLight`), og det flyttes, dreies, kopieres og slettes som alt annet. En kiste, spak, lapp, bonus, trykkplate, sprekk eller en av kryptens småting fra katalogen blir et nytt merke i nivået (`edit::crypt::mark_for`); resten kan ikke stå der nede. Regelen: et merke på gulvet står inne i et rom i sitt nivå med foten på gulvet (`edit::crypt::breaches`). Testene `krypten_redigeres_og_lagres` og `krypten_redigeres_der_du_star`. Etter en lagring med flyttede merker kjører Claude `python3 tools/crypt/build_level.py N` så nettet og kartet følger.

- **Raskere og enklere (17. oktober):** velge i dybden (lista under pekeren, Shift+Tab, Alt+klikk for det bakerste; `edit/select.rs`), boksvalg (Shift+dra, `Editor::select_box`), dra fra katalogen med fantom og reglene som farge (`edit/ghost.rs`: `ghost_breaches`, `EditCommand::Ghost`), fest til 0,5 m og 15° (`snap_pose`, `EditCommand::Snap`, også under drag), R dreier 90°, automatisk «sett ned» når et drag langs bakken slippes (`Editor::standing`, `settle`), kollider på nye ting (`default_solids`), egenskapspanelet (`edit/props.rs`: `Prop`, `Editor::props`, `set_prop`), historikken som liste (`edit/history.rs`: `history`, `undo_to`), zip uten nøkkel (`makeZip` i `web/editor.js`), utkast hvert 30. sekund og kamerabokmerker. Testene `dybden_velges_fra_lista_og_bakfra`, `egenskapene_settes_og_angres_fra_historikken`, `fantomet_festet_og_slippet` og `rammen_velger_det_innenfor`. Fantomet tegnes som modellen med en farget boks, ikke gjennomsiktig (det trenger et eget pass i rendereren).

## Videre

Det som mangler og rekkefølgen står i `docs/plan.md`, del 11 (17. oktober). Gjort: egenskapspanelet, fest til rutenett, boksvalg, kamerabokmerker og lagring uten nøkkel (zip). Igjen: grupper som velges og flyttes samlet (trenger et felt i dataene), innbyggernes ruter som punkter å dra, lysene i byen, kartet, rommene og gangene i krypten som redigerbare ting, regler som foreslår rettelsen, og et gjennomsiktig fantom.
