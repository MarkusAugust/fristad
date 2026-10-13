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

## Runde 5 (12. oktober)

- **Studiobildene i runde 3 og 4 viste teksturlagene uten fargetonen.** glTF-importen i Blender legger grunnfargen i en MIX-node sammen med hjørnefargen, og `studio.py` leste den ukoblede standardverdien (hvit). Pussen, sokkelen og bordkledningen ble dømt på for lyse bilder. Regel: `studio.py` henter tonen fra MIX-noden, og et nytt verktøy prøves på et kjent tilfelle (et rødt forheng som skal bli rødt) før eieren får bildene. De 23 modellene vises på nytt i runde 5.
- **Et møbel fikk samme navn som en modell i byen** (`weapon_rack` i Smia og på øvingsplassen), og den ene skjulte den andre. Regel: `moblene_er_egne_modeller` krever at alle modellnavn er unike.
- **Uten solskygge mistet tønna i Smia skyggen sin på muren** (Smia er åpen mot torget). Regel: møblene kaster solskygge innen `INDOOR_SHADOW` (10 m) fra kameraet, og før- og etterbilder inne i alle husene sjekkes etter hver endring i rendereren.
- **Tønnene sto i feil høyde i alle husene, og de liggende stakk inn i veggen.** I `kit.barrel` ble `z` (høyden tønna skulle stå i) brukt som løkkevariabel, så hver tønne, krukke, skinke og gryte havnet i høyden `h` over husets origo (kjellertønnene i Vertshuset sto oppe i første etasje). Liggende tønner ble flyttet en halv lengde med `location` før `matrix_world` ble lest, så flyttingen forsvant og tønna stakk 35 cm inn i veggen bak disken. Det er grunnen til «tønne i veggen». Fant det fordi møbelets mål i sitt eget rom (0 til høyden) ikke stemte. Regel: et møbel skal stå med foten i origo i sitt eget rom; studiobildene og målene sjekkes for hvert nytt møbel.

## Runde 6 (12. oktober)

- **Bordbena lå flatt under bordplata.** `stick` setter `location` og rotasjon, og `kit._place` leste `matrix_world` før Blender hadde regnet den ut, så bena ble liggende langs bakken midt under bordet; det eieren så som «lego» var bokser med timber-arket på tvers. Regel: `_place` (og `town.piece`) kaller `view_layer.update()` før `matrix_world` leses, og bena er `stick` med kornet langs. Testen `moblene_star_der_kolliderne_er` krever at ingenting i et møbel går under gulvet.
- **Bindingsverket delte opp åpninger:** felt bredere enn 1,3 m fikk ekstra stolper, også verkstedåpningen i Smia (fire stolper midt i). Regel: et felt som er en åpning, deles ikke (`kit.half_timbered`).
- **Skrå bøker ble dreid om origo** og havnet under gulvet. Regel: noe som lener seg, dreies om sitt eget hjørne.
- **Det som står høyt, ble fotografert fra gulvet** (hjelmhylla, krukkehylla), og flammene lyste ikke i studioet. Regel: `studio.py` sikter på tingen og lar `fire…`/`glow…` lyse.
- **Gjentakelse ser kunstig ut** (eieren om hyllene, disken og verktøyveggen). Regel: hyller og bord fylles fra `tools/models/vessels.py` (`row`) med blandede former og størrelser, og verktøy og ting i rekker er forskjellige.

## Runde 7 (12. oktober)

- **Rad to i tønnestativet svevde 8 cm.** Høyden var regnet som om tønnene under lå tett (r·√3), men de lå en meter fra hverandre. Regel: det som hviler på noe, får høyden regnet fra kontakten med det det hviler på, med de samme målene som plasserer det (`kit.barrel_stack`), ikke fra et tall skrevet for hånd.
- **Kar på en disk i én rad ser oppstilt ut.** Regel: hyller fylles med `vessels.row`, bord og disker med `vessels.spread` (tette grupper av det som hører sammen, spredt over hele flata).
- **Smia-kortet fikk bildene av kassa i Smia**, fordi verktøyet plukket bildene på navnet (`before_forge_*` traff `before_forge_crate_*`). Regel: vinkelnavn har ingen understrek, og `round_page` tar bare dem.
- **Disken ble vist forfra, fra enden**, der det som står oppå ikke synes. Regel: det som ligger oppå en flate, vises også ovenfra (`top`).

## Runde 8 (12. oktober)

- **Byggesettet i krypten brukte dørbilder som tre** (`oak` = `bronze_door`, `wood` = `door_b`): lemmen ble svart, stigen fikk hengsler og nøkkelhull malt på trinnene, kister og fakler i nivået likeså. Regel: tre er planker (`planks`) og bjelker (`timber`); lag som er ett helt bilde av en ting (`WHOLE_IMAGES`) brukes bare på et dørblad med uv 0–1, og eksporten stopper ellers (`check_whole_images`).
- **Projiserte gjenstander fra ark** (kista, brønnen) ble bokser med et bilde av tingen på, og teksturen passet ikke formen. Regel: gjenstander bygges av ekte geometri med teksturlagene, som resten av byen.
- **Steingavler som overlapper murkronen** gir to flater i samme plan og svarte striper (krypten, kirka, Skriptoriet). Regel: `parapet` begynner på murkronen.
- **Tønner i veggene og hyller over vinduet** (Vertshuset, spillerens hus), og trappa over to vinduer. Regel: hver vegg fra `kit.slab_grid` registreres, og hvert møbel og hver trapp sjekkes mot veggene og vinduene (`town.check_piece`, `check_points`); eksporten stopper om noe går inn i en vegg eller står foran et vindu.
- **Lasten i vogna og hjulet mot kassa.** Regel: det som ligger i noe, sjekkes mot det (`check_clear`, `check_apart` i `streetlife.py`).
- **En ting i et gjerde ble ikke funnet** av reglene i redigeringen, fordi gjerder og murdeler var unntatt. Regel: en ting kan ikke stå inni noens kollider, heller ikke et gjerdes.
- **`cargo fmt` skrev om 67 filer.** Regel: kjør det aldri på hele repoet (CLAUDE.md).

## Runde 8 – dommen (12. oktober)

- **Hule ting skal være hule.** Kista ble bygd som en massiv boks med jern utenpå. Regel: `check_hollow` i `streetlife.py` skyter stråler ned i kassa og opp under lokket.
- **Bevegelige deler følger tingen sin i redigeringen.** Vinda og bøtta sto igjen når brønnen ble flyttet, fordi de ble satt etter verdenen spillet startet med. Nå hentes målene fra redigeringens verden (`view.rs::target_poses`, `live_crypt`). Testen er `bevegelige_deler_folger_med_i_redigeringen`.
- **Gavler av stein følger taket.** Dekkhellene ble satt fast over murkronen, ikke etter takflaten, og sto opptil 50 cm over taket. Regler:
  - `roof_parapet` regner gavllinja ut fra mønet og takvinkelen.
  - `check_parapet` måler med stråler at dekkhella ligger 0–30 cm over taket.
  - `check_crosses` sjekker at korsene står på noe.
- **Ikke grønt øverst.** Mose på mønet og flekker på taket så ut som feil. Mønet er hogd stein (`ridge_saddle`).
- **En sprekk må ha mørk farge.** Revnene var tegnet med `wall_dark`, som er lys murtekstur, og forsvant. De har nå sitt eget materiale (`crack`), er 3–5 cm brede og har lyse kanter.
- **Steinene i en bue følger buen.** Murbildet lagt flatt over karmsteinene ga vannrette skift som ble kappet av hver stein, og åpne fuger så ut som hull. Nå:
  - Hver kilestein får én hel stein fra bildet langs seg (`stone_uv`, `STONE_PX`).
  - Bak fugene ligger mørtel.
- **Vis det eieren spør om.** Trappene ble vist uten vinkel fra foten. Kortene skal ha vinkler som svarer på punktene, og målene skal stå i teksten.

## Runde 9 – dommen (12. oktober)

- **Én type strek.** En mørk revne med en lys kant ved siden av ble en hard strektegning. En sprekk er én sammenhengende strek som smalner av, og greinene går ut fra den (`crack_web`).
- **Svaien må passe til mønet.** 32 cm svai med en stiv mønestein ga en hengende bue. Mønesteinen tåler høyst 12 cm, som på kirka.
- **Plass ved foten.** 1,05 m foran nederste trinn var for lite når hjørnet ved foten er møblert. Skriptoriet har 1,6 m.
- **Regler gjelder også det som er godkjent.** Pultene i Skriptoriet dekket vinduene, men ble ikke fanget før huset ble eksportert på nytt. Etter en ny regel skal alle husene eksporteres på nytt.
- **En kodefeil får også et bilde.** Når eieren skal vurdere en kodefeil (brønnen i redigeringen), tas bildet fra nettleseren med den ferdige nettsiden.

## Runde 10 – dommen (12. oktober)

- **Det som ligger under, må ikke stikke opp.** En kile under mønet fulgte ikke svaien og stakk opp gjennom mønesteinen. Regelen `check_ridge` skyter stråler ned langs mønet, og det første de treffer skal være mønesteinen.
- **En sprekk i kvadermur går i fugene.** Den går trinnvis, opp og bortover, aldri tvers gjennom steinene. Fugene er målt i murbildet (`WALL_BEDS`, `WALL_HEADS`), og bildet står opp ned i meter, fordi glTF har v nedover.
- **En trapp står langs en vegg.** Det som står mellom trappa og veggen, flyttes. Vinduer som trappa eller galleriet ville stå foran, mures igjen eller flyttes, og utsida vises på nytt.

## Runde 11 – dommen (12. oktober)

- **Rett feilen overalt der den kan oppstå.** Vannbordene ble rettet på kirka og kryptinngangen, men ikke på Skriptoriet, som brukte den samme gamle funksjonen. Når en feil rettes ved roten, slettes den gamle funksjonen (`parapet`). Da må alle som brukte den, gå over til den nye (`roof_parapet`) med reglene.

## Monstrene fra arkene (12. oktober)

- **Arket legges på fra den visningen flaten vender mot.** `project_sheet` i `tools/models/monsters.py` gir hver flate uv inn i forfra-, bakfra- eller sidebildet. Sidebildet brukes bare når flaten vender mer enn 63° til siden; ellers drar det inn det som står foran i sidebildet (armen, en mørk glipe).
- **Bakgrunnen fylles fra innsida av figuren.** Der en flate stikker litt utenfor omrisset, tar `filled_sheet` fargen fra nærmeste piksel inni figuren, ikke fra den mørke streken rundt. Hull i masken fylles først: grå partier inni kappa ligner bakgrunnen, og ble ellers flekker.
- **`ellipsoid` tar diametre, ikke radier.** Skulderplatene og hetta til presten ble halvparten så store.
- **Faste deler henger på sitt bein.** Hansker, støvler, kabler og staven settes stivt på hånda, foten eller brystet (`rigid`). Ellers vektes de etter avstand og dras med av leggen ved siden av.
- **Skrittet følger beinlengden.** Med korte bein (Modalen) må tråkket være raskere og skrittet kortere, og hvileposen kan bare senkes litt. Ellers knekker knærne sammen. Armen med stav står i hvileposen (`hold`), så staven står loddrett.

## Kryptnivå 1 – dommen (13. oktober)

- **Et hull er det du kan se gjennom.** Trinnene i trappa ned var tynne plater med to sider, så bakgrunnen synes mellom dem som grå striper. Tetthetssjekken fant det ikke, fordi den skyter fra gulvet i rommene og ikke fra innsida av en modell. Trinn er massive ned til bunnen.
- **Det som ligger under gulvet, må høre til et rom.** Lyset følger rommene (`room_of`), og utenfor alle rom lyser bare lykta du bærer. Sjakta til trappa ned lå under gulvet og ble utbrent i lykteskinnet. Rommet når nå ned i sjakta (`Room.sink`).
- **To flater i samme plan blinker.** Det gjaldt terskelen 3 mm under gulvet, enden av brystvernet mot portalmuren, og innsida av brystvernet mot lysningen i buen. En flate skal enten stå inne i den andre eller minst 1 cm unna.
- **Samme stein overalt i et rom.** Hvelvet, veggene og kapellet mures av den samme steinen (`crypt_wall`). En annen stein brukes bare der den betyr noe (gravheller, beinmuren).
