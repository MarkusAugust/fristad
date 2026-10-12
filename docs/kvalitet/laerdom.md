# Lærdom: feil som er blitt regler

Hver feil som oppdages (av en kontroll, vurdereren eller eieren) får en årsak og en regel i felles kode med en test, og føres her (`docs/plan.md`, prinsipp 3). Tersklene strammes, aldri slakkes.

## 11. oktober

- **Teksturlag som ikke finnes, ga en stille flat farge** (smia brukte «bark»). Regel: `tools/models/roles.py` stopper eksporten når laget ikke finnes i `assets/textures/`, og motoren laster ikke en modell med ukjent lag (`modellene_har_roller`).
- **Ryddingen fjernet hjelpere byggesettet brukte** (`town.SMOKE`, `LIT`, `RELIEF`), så husene ikke kunne eksporteres. Regel: `tools/models/check_scripts.py` (testen `blenderskriptene_henger_sammen`).
- **Materialrollen ble gjettet ut fra navnet** i motoren. Regel: rollene er data i glb-fila (`extras.fristad`), og motoren sammenligner ingen navn (`lasteren_leser_rollene_og_gjetter_ikke`).
- **Teksturkontrollen (P-T), kalibrert mot bildene:**
  - Skjøten målt som snitt over hele kanten druknet skjøter som bare synes et stykke, og et forholdstall mot nabosteget overdrev på glatte flater (puss). Regel: den verste åttendedelen langs skjøten mot 99-persentilen av de naturlige overgangene i bildet; skjøter synes med øyet fra rundt 1,0 (`MAX_SEAM`).
  - Laplace-variansen straffet glatte materialer. Regel: hvor mye de sterkeste kantene mister ved 1 piksel uskarphet (`MIN_CRISP` 1,40; oppskalerte bilder ligger under).
  - Ramme målt som forskjell mellom kantbånd slo ut på mønstre som kuttes ved kanten (takstein, brostein). Regel: en ramme er en stripe som er jevn langs hele kanten (`MAX_FRAME`).
  - Perspektiv kan ikke skilles fra mønstre som er grovere på én side med vilje (`core_stone`) med enkle mål. Regel: perspektiv vurderes med øyet i den estetiske kontrollen; P-T har ingen perspektivsjekk.
  - Arkene (dører, list, skilt, vinduer) er ikke fliser. Regel: slaget står i `assets/catalog/textures.json`, og P-T sjekker arkene bare for størrelse og skarphet.
- **Kildebildet for kryptpussen var fortsatt stjernetapetet** som eieren underkjente, under samme navn som laget. Regel: katalogen (`docs/katalog.md`) viser kilden, laget og eierens dom side om side.

## Eierens dom over alt som finnes (11. oktober, godkjenningssiden)

209 ting vurdert: av teksturene 75 godkjent, 6 underkjent, 1 usikker; av modellene 40 godkjent, 79 underkjent (28 er bitene i kryptnivå 1), 8 usikker. Grunnene, samlet etter årsak, og regelen hver blir:

- **Bildene var feil for alle modeller med dører, hogd stein og planker.** Hvite dører og «pipemurstein» kom av at de lokale teksturlagene manglet `door`, `door_b`, `ashlar`, `planks`, `iron`, `oak_planed` og `straw_floor`, og et lag uten bilde falt stille tilbake til et prosedyrisk lag. Regel: et lag en modell bruker må ha et bilde, ellers feiler `cargo test` og `snapshot` (`alle_lagene_har_bilde`). Modellene vises eieren på nytt.
- **Glippe mellom etasjene** (hus a, b, g, i, m, n, p, r, s): utkragingen la 0,3 m tom luft mellom etasjene. Regel: felles etasjeskille i `kit` med bjelkelag og svill, som Vertshuset.
- **Bindingsverket ser feil ut, særlig rundt vinduene** (Vertshuset, Apoteket, gjestehuset, hus a, b, d, l): felt med vindu manglet losholt over og under, stolpene overlappet karmene, og skråbåndene var tilfeldige. Regel: bindingsverket bygges etter ekte konstruksjon (stolpe ved hver vindusside, losholt som brystning og overligger, skråbånd symmetrisk ved hjørnene), med kontroll.
- **Hus som svever** (c, e, q, hagen): sokkelen gikk bare 0,25 m ned, og høyden ble målt halvveis ut. Regel: grunnmur ned til terrenget i hvert hjørne, og bakkekontrollen over hele fotavtrykket.
- **Harde overganger uten lister** (alle innsidene unntatt smia, kryptens vegg mot gulv, rundt dører og buer i krypten): to ulike lag møtes i 90° uten noe imellom. Regel: profilerte lister (ikke kvadratiske) med egne teksturer der to lag møtes, og omramming rundt dører; en dør tar aldri med seg vegg (`kit_up`). Gjelder alle bygninger.
- **Innredning plassert dårlig** (tønne i veggen, hyller foran vindu, bord for nær veggen, for nakent): Regel: `group()` med ankre, fri sone foran vinduer og dører, og kontroll på overlapp med vegger.
- **Feil materiale** (stråtak på høystakken, uthustak med strukket tekstur, puss på smias endevegger innvendig, stein på lesepulten i krypten, kista uten beslag): Regel: uv per flate på tak, og hvert slag ting har sitt materiale i byggesettet.
- **Ting som går inn i hverandre** (kjerrehjul, snarveidøras slå, fallemmen, våpenstativet i gjerdet): Regel: kontroll på inntrengning mellom deler (P5) og mellom plasserte ting.
- **Uleselige ting** (torgkorset, sarkofagen, bonusen, stigen som ikke synes): Regel: silhuetten må kunne kjennes igjen; vurdereren og godkjenningsbildet viser tingen tydelig.
- **Porthuset hadde en tverrbjelke midt i portrommet.** Regel: ingenting tvers over en åpning man skal gå gjennom.
- **Skjøter som synes** (`stone`, `cobble`, `crypt_wall`, `core_stone`) og **gjentatte små flekker som blir støy på avstand** (`plaster_red`): Regel: teksturkontrollen skal være enig med eieren (`kontrollen_er_enig_med_eieren`), og den får en måling av gjentakelse på avstand.
- **Spillerens hus:** en vandrer har ikke eget hjem. Regel: det heter Gjestehuset.

## Eierens tid (11. oktober, kveld)

- **Eieren ble bedt om å se på nye bilder der ingenting av det eieren påpekte var rettet** (bare teksturene i bildene var nye). Regel: eieren får bare se en ting igjen når hvert punkt eieren skrev om den er rettet og sjekket på før- og etterbilder; kortet viser punktene, «rettet slik» og før/etter side om side, og det som ikke er endret, vises ikke.
- **Godkjenningssiden arvet forrige grunn**: feltet var fylt med teksten fra forrige runde, og en ny dom ble lagret med den gamle grunnen (hus h, j og k ble «godkjent: hvit dør»). Regel: hver runde har sitt eget felt (`round` i lagringen, `ROUND` i `tools/qa/approvals.py`); forrige dom står bare som «Forrige dom», og `approvals apply` fører en endret dom med uendret grunn inn med tom grunn og den gamle som `before`.
- **«Nytt bilde» tok med ting eieren hadde godkjent.** Regel: bare det som ikke er godkjent, vises igjen.
- **Tastene på siden traff kortet over det eieren så på.** Regel: tastene virker bare på kortet som er merket.
- **Planen ble lagt fram i sin helhet for hver lille endring**, så eieren ikke kunne se hva som var nytt. Regel: eieren får bare endringen; hele planen står i `docs/plan.md`.
- **Runde 2 sa i tillegg:** etasjeskillet og bindingsverket må løses samlet for alle husene; vinduskarmene er for spinkle til det kraftige bindingsverket (Vertshuset); noe stikker opp gjennom taket på uthusene og hus c, og takteksturen er feil; tønnene i veggene og trappa for nær veggen er ikke rettet (Vertshuset, Gjestehuset); Smia skal ha stein helt opp i endeveggene under mønet; kryptdøra har merkelige beslag; stigen og bonusen kan ikke kjennes igjen.

## Runde 3 og 4 (12. oktober)

- **Husene og uthusene er godkjent** etter at feilene ble rettet i felles kode: bindingsverket som ekte konstruksjon (`kit.half_timbered`), bjelkelaget mellom etasjene (`kit.storey_band`), grunnmur ned i bakken med bakkekontroll over hele fotavtrykket, vindskiene under takflaten, pulttak med vegger som følger fallet (`yard.lean_roof`, `slope_wall`).
- **Hønsebrua var vridd på skrå** fordi en bjelke dreid fra x-aksen uten fast retning fikk bredden i en tilfeldig retning. Regel: skrå bjelker får en fast basis (bredden vannrett på tvers), som `kit.wall_member`.
- **Høystakken så bra ut fra noen vinkler, men ikke alle:** flate fasetter og en søm der vinkelen går rundt. Regel: runde former merkes med `town.smooth` (ellers gjør `finish` alt flatt), og uv rundt en form regnes per flate uten hopp.
- **Bildene fra byen var ubrukelige til å sjekke en modell** (kameraet havnet i nabohus). Regel: før- og etterbilder tas med `tools/qa/studio.py` (modellen alene med de ekte teksturlagene, i faste vinkler), og en runde bygges med `approvals round`.
