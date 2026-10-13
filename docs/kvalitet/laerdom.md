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

## Datepicker-presten – dommen (13. oktober)

- **Et våpen holdes og svinges.** Fingrene pekte rett fram ved siden av skaftet, og angrepet var slaget i bakken, så staven ble slengt flatt med nevene. En hånd med våpen har fingre lagt rundt skaftet (stivt på samme bein som våpenet), og et monster med våpen har et eget angrep der våpenet bærer slaget (`attack_staff`).
- **Våpenet går aldri gjennom kroppen.** `staff_clearance` i `tools/models/monsters.py` måler staven mot kapsler rundt kroppen i hvert bilde av angrepet og stopper eksporten om den går inn. Før regelen gikk skaftet 5 cm inn i kappa i treffet.

## Armene og våpnene (13. oktober)

- **Hugg sees fra kameraet.** En klinge som peker rett fram eller bakover over skulderen blir forkortet til nesten ingenting. Et vannrett hugg legger klinga på tvers av bildet, og opptrekket holder den synlig ut til siden. Hvert nøkkelbilde sier hvor grepet er og hvor klinga peker (`aim`), og grepet holdes innenfor synsfeltet (72°).
- **Del-objekter må ha plassen sin i hjørnene før de flyttes.** `tube` og `ellipsoid` lager objekter med egen plassering; ble bare hjørnene flyttet, ble tommelen hengende igjen og fløy løs i huggene.
- **Lyst metall på grått ark trenger egen maske.** Klingas lyse side ligner bakgrunnen. Dolken bygges av delene langs aksen (konvekst skall for klinga og knappen, rad for rad for grepet med ringer), symmetrisk om aksen.
- **Tynne kropper får ikke omregnede normaler.** På en klinge noen millimeter tykk snudde omregningen normalene innover, og okklusjonen bakte albedoen svart.

## Armene og våpnene – dommen (runde 16, 13. oktober)

- **En arm ender aldri inne i bildet.** Overarmen var et kort erme bygd langs underarmen, og i blokken så det ut som avkappede armer som fløy inn. Overarmen går nå langs selve leddet fra albuen til 15 cm forbi skulderen (`ARM_PAST`). `arms_leave_frame` stopper eksporten om enden av armen havner innenfor synsfeltet i noe bilde av noe klipp.
- **Hånda er én kropp etter omrisset i arket.** Knyttneven var stykker (håndbak, fire ringer, rør til tommel) som stakk ut hver for seg, og fingertuppene vendte mot kameraet. Hånda og underarmen loftes nå etter silhuettene ovenfra og fra tommelsida (`silhouettes`, `arm_mesh`). Detaljene står i det malte arket. Lillefingersida får håndbaken eller håndflaten, ikke tommelsida, som ellers ville synes to ganger.
- **Hånda holder som man holder.** Tommelen ligger over grepet, og knokene peker fram langs eggen. Håndbaken vender ut og opp, ikke håndflaten mot øyet. Våpnene eksporteres i hvileposen (`RIGHT`), så de må bygges på nytt hver gang hvileposen endres.
- **Et våpen bygges av lukkede deler, ikke blåses opp og slankes.** Decimate på den oppblåste klinga ga hull. Parerstanga og knappen ble like tynne som klinga, og de malte hakkene i eggen kom med i omrisset. Hver del (`PARTS`) er nå en lukket kropp med sin egen form. Klinga er rett og symmetrisk om én akse og blir aldri smalere mot parerstanga. Arket speiles fra den hele sida. `solid_report` stopper en del med åpne kanter.
- **Grepet passer hånda.** Arkene tegner grepet 5 cm tykt, mens hånda er laget rundt 3 cm. Grepet skaleres til 3,2 cm der hånda holder, og `GRIP_D` stopper et våpen med grep utenfor 2,8–3,6 cm.
- **Skjoldet vender dit man holder det, ikke dit underarmen peker.** Flaten sto vinkelrett på venstre underarm, som pekte inn mot midten. Retningen er nå fast: rett fram, dreid 10° ut mot venstre (`SHIELD_OUT`). `shield_check` stopper hvile utenfor 0–20° og en blokk som dreier skjoldet inn forbi rett fram.
- **Klinga går heller ikke gjennom skjoldet.** `blade_clearance` måler nå også mot skjoldene. Skjoldet trekkes ned og ut mens klinga går over.

## Armene – dommen (runde 17, 13. oktober)

- **Grepet ligger skrått i neven.** Våpnene sto vinkelrett på fingrenes retning, så klinga sto rett opp fra en vannrett underarm, og håndleddet så knekt ut. I en ekte knyttneve går skaftet på skrå over håndflaten. Våpnene står nå i `WEAPON` (`GRIP_TILT` 20°), og underarmen stiger fram mot hånda (`grip_frame`).
- **Håndleddet bøyes lite og vris aldri.** IK-en valgte albuen bare etter retningen den skulle bøyes, og hånda ble dreid fritt, så håndleddet ble knekt 60–90° og blandingsvektene strakk armen som gummi i skjoldblokken. Nå velges albuen der håndleddet bøyes minst. Underarmen tar hele vridningen, og `aim` velger rullen om klinga som gir minst bøy. `wrist_bend` stopper eksporten over 35° bøy eller 15° vridning, i alle klipp og for begge hender.
- **Et punkt uten dom føres inn likevel.** «Øksa holdes feil vei» kom uten dom og ble hoppet over av `approvals apply`. Nå føres punktet inn, og dommen står.
- **Bladet skal synes.** Øksebladet sto med kanten mot kameraet. `face_visible` måler flaten mot synslinja i hvile (høyst 60°).

## Øksa og brynja – dommen (runde 18, 13. oktober)

- **Eggen måles fra skaftet der hodet sitter.** Skaftet er bøyd. Målt fra midtlinja ved grepet så nakken ut som den lengste sida av hodet, så eggen ble snudd bakover. Midtlinja til skaftet ble i tillegg skalert sammen med tykkelsen, så skaftet gikk skjevt gjennom hodet. Nå måles sidene fra skaftets egen midtlinje i radene der hodet sitter, og bare tykkelsen skaleres. `edge_forward` måler på modellen at eggen peker mot fienden midt i bildet i hvile (høyst 40°).
- **Eggen går først i hugget.** Rullen om klinga velges slik at knokene og eggen vender i retningen spissen beveger seg på tvers av klinga, uten å bryte håndleddsregelen. Det gjelder ikke etter treffet. `edge_leads` krever cos ≥ 0,5 i hvert treff.
- **Det synlige av armen skal vise rustningen.** Underarmen i `arms_mail` ble strukket jevnt over radene i arket, så hanskens mansjett dekket det meste og brynja bare litt nederst. `FOREARM_ROWS` legger nå hver del av underarmen på sine rader, og `mail_coverage` krever at brynja dekker minst 70 % av underarmen.

## Dommen – eieren (13. oktober)

- **Siden viser tilstanden, ikke den siste runden.** Hver runde byttet ut hele siden med det jeg nettopp hadde endret. Et kort eieren ikke rakk å svare på, forsvant derfor (rundskjoldet i runde 18), og 26 kryptbiter ble stående med en dom fra 11. oktober. Nå gjelder en dom versjonen eieren så. Alt som ikke er vurdert, eller som er endret siden, står i «Venter på deg» til eieren har svart. `check` og testen `dommen_mister_ingenting` stopper en side der noe mangler.

## Krypten og dolken – dommen (runde 20, 13. oktober)

- **Hull ses best med motorens egne øyne.** Lekkasjesjekken med voksler (8 cm) fant ingenting, men eieren så kiler av himmel ved konsollene. `snapshot --leak-scan PUNKTER --debug void` tegner alt svart og himmelen magenta, og ser fra hvert punkt over gulvet (`tools/crypt/leak_points.py`) i 24 retninger. Det som er magenta, er hull, også de på noen få centimeter og de som bare finnes langt unna.
- **En mur under et hvelv går opp bak hvelvet.** Veggen under krysshvelvet fulgte buen med 17 rette biter og sto under hvelvkanten der buen er bratt ved vederlaget. Det ga kiler av himmel på begge sider av hver konsoll. Murene går nå `VAULT_LAP` (35 cm) opp bak hvelvflaten.
- **Det grove detaljnivået åpnet sprekker.** Lenger unna enn 28 m (14 m i standard grafikk) tegnet motoren en forenklet utgave der hjørnene ble slått sammen i ruter på 1/70 av modellen. En smal murrute (11,5 cm der den runde muren i Beinhuset slutter) forsvant, og skjøtene ble ulike i to naboer. Modeller under `LOD_MIN_TRIS` (12 000 trekanter) tegnes nå helt på alle avstander, og testen `kryptmurene_forenkles_ikke_pa_avstand` holder kryptbitene der.
- **En trapp begynner utenfor hjørnefaget.** Trappa i snarveien begynte midt i faget der gangen svinger, under gulvet der, og gjennom spalten under hjørnegulvet så man ut under gangen. `Hall` flytter nå trappa ut av hjørnefagene og stopper bygget om den ikke får plass. Murene i gangene går også `VAULT_LAP` opp bak det trappete hvelvet.
- **Under jorda er det jord, ikke himmel (eieren).** Nede i krypten tegnes himmelen sort (`sky.wgsl`, når nivået har rom). En sprekk som slipper gjennom, blir da en mørk strek og aldri en lys kile. Sprekkesjekken ser fortsatt himmelen (`--debug void`).
- **En søyle bærer noe.** Søylene i Søylegangen sluttet ved vederlaget, 2 m under tønnehvelvet. Nå står gjordbuene over søyleradene, og kapitélet går opp til buen (`pillar_top`).
- **En dolk er spiss.** Spissen var en halvsirkel. Klinga smalner nå jevnt til et punkt, og `tip_check` stopper en klinge som 2 cm fra spissen er bredere enn halvparten av sin største bredde.
- **Store ting dømmes i 3D.** Eieren kunne ikke dømme 16 m store kryptbiter fra to små bilder. Dommen har nå «Se i 3D» (`tools/qa/viewer.js`): modellen med teksturlagene, før og etter, og for kryptbitene hele nivået rundt. Man kan rotere, zoome og dobbeltklikke seg inn i rommene.

## Lyset, tokenene og huggene – eieren (runde 21, 13. oktober)

- **Lys har bare betydning i mørke.** Krypten hadde et blått grunnlys fra «himmelen» og kalde fyllys uten kilde, og øyet ble skrudd opp 2,5 ganger, så alt var synlig uten lykt. Nå er grunnlyset nesten borte, de kalde lysene er et svakt skjær, og spilleren bærer lykta i beltet med olje som brenner ned. Uten olje ser du bare faklene og lysene.
- **Lyset fra beltet blender ikke det du holder.** Et lys midt mellom hendene gjorde skjoldet hvitt. Lykta står foran deg ved hofta, og et svakt lys når opp på hendene.
- **En modell som ventes, er usynlig.** Tokenene ble sluppet og plukket opp, men modellen `token` sto i `PLANNED`, så ingen så dem. Når spillet bruker noe synlig, lages modellen før det vises eieren.
- **Et Python-skript heter ikke som en modul i standardbiblioteket.** `token.py` krasjet Blender (`token` er Pythons egen); skriptet heter `design_token.py`.
