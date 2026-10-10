# Den nye krypten

Bestemt med eieren 10. oktober 2026. Krypten lages på nytt fra bunnen i den malte stilen (`docs/ny-stil.md`). Den gamle genererte kjelleren slettes med en gang. Krypten er bare så dyp som det som er ferdig. Teksturpromptene står i `docs/prompter-krypten.md`.

## Beslutningene

- **Ti faste nivåer**, tegnet for hånd og like hver gang du spiller. Hvert nivå tar en ny spiller 10–15 minutter, så hele krypten tar 2–3 timer. Kistene kan variere litt. Fiendene står fast.
- **Fem temaer à to nivåer**, ovenfra og ned:

  | Nivå | Tema | Preg |
  |---|---|---|
  | 1–2 | Gravkjelleren | stein, stearinlys, beinhus, gravnisjer |
  | 3–4 | Kloakken og cisternene | vann, renner, mose, murstein, slim |
  | 5–6 | Gruva og grottene | berg, tømmerstiver, malm, lysende krystaller |
  | 7–8 | Borgen under jorda | søylesaler, statuer, rust, falmede faner |
  | 9–10 | Den korrupte kjernen | Fristil har tatt over: plast i steinen, glitch, lysende sømmer |

- **Byggesett og tegnet kart:** hvert tema får et byggesett av moduler i Blender, som murer, buer, gulv, trapper, dører, søyler og hvelv, med malte teksturer. Hvert nivå er et Python-skript som setter modulene sammen etter et kart jeg tegner. Bossarenaene og de store salene kan modelleres for seg.
- **Finne fram:** kartet med fog of war som i dag, både minikartet og det store kartet på K. Hovedveien skal være lett å lese uansett: tydelige døråpninger, ingen blindgater uten belønning, og trappa ned ser lik ut i alle nivåer.
- **Vertikalitet og hemmeligheter:**
  - Falske vegger
  - Trapper og balkonger, gallerier over store saler og halvetasjer
  - Fallemmer og sjakter
  - Spaker og mekanismer: bruer som senkes, porter som åpnes langt unna og vannstand som endres
- **Hemmelighetene:**
  - De har subtile hint: sprekker i muren, et kaldt trekk som får fakkelen til å blafre, en annen stein og lyd bak veggen.
  - De gir sjeldne komponenter som bare finnes der, lore (lapper, inskripsjoner og bøker), snarveier og varig styrke (små bonuser til liv eller utholdenhet).
- **Fiendene:**
  - **Navigasjonsnett:** hvert nivå får et navmesh, så fiendene følger deg over trapper og ramper og rundt møbler. En test går gjennom hele nivået og sjekker at ingen kan bli sittende fast.
  - **Oppførsel:** noen patruljerer faste runder, andre sover eller vokter.
  - **Nye slag:** hvert tema får 2–3 egne fiendeslag, og de gamle kommer igjen sterkere.
- **Bossene** står nederst i hvert tema og er store, korrupte Fristil-komponenter. Nivåene uten boss har en elite som bærer nøkkelen til porten ned.

  | Nivå | Boss | Hvordan den kjemper |
  |---|---|---|
  | 2 | Modalen | gravvokter med et lokk som smeller, og et teppe av mørke som legger seg over arenaen |
  | 4 | Tabellen | vannskapning av rader og celler, som deler seg og setter seg sammen igjen |
  | 6 | Dropdownen | gruvekjempe som folder seg ut, med lag på lag av lemmer |
  | 8 | Navbaren | ridder med lenker, som drar deg mellom søylene |
  | 10 | Rammeverket | sluttbossen, som i dag |

- **Feller:** trykkplater, pendler, piler fra veggen og fallende stein, tilpasset hvert tema.
- **Hvilestedene:**
  - Hvert nivå har ett hvilested, et alter med lysestake som du tenner. Alle ser like ut, så du alltid kjenner dem igjen.
  - Hviler du der, får du fullt liv og full utholdenhet, og fiendene kommer tilbake.
  - Dør du, våkner du ved siste hvilested du tente. Det du bar, blir liggende der du døde.
- **Heisen:** inne i krypten på kirkegården står en heis eller trapp ned til begynnelsen av hvert tema du har nådd (nivå 1, 3, 5, 7 og 9). H og portalen virker som i dag.
- **Glitch:** Fristil-korrupsjonen synes litt allerede på nivå 1, med en lysende søm i en mur og en stein som flimrer. Den øker jo dypere du kommer, til kjernen der den har tatt over.
- **Lyset:** malt halvmørke. Varme lyskilder som fakler, glør og stearinlys står mot kalde, blålilla skygger, som i WoWs katakomber. Du ser alltid gulvet og veien, men hjørnene er mørke.

## Rekkefølgen

1. **Tegnet plan for alle ti nivåene.** Kart ovenfra med rommene, hovedveien, sidegrenene, hemmelighetene, hvilestedet, fiendene og fellene. Eieren ser og godkjenner dem.
2. **Motoren:**
   - Nivåformatet
   - Navmesh og A* med trakt for fiendene
   - Hvilestedene, heisen, spakene, de falske veggene, fallemmene og fellene
   - Patruljer
   - Den gamle kjelleren slettes
3. **Piloten:**
   - Byggesettet for gravkjelleren
   - Nivå 1 helt ferdig, med teksturer, lys, fiender, hemmeligheter og test
   - Eieren godkjenner
4. **Nivå 2 og Modalen**, så tema for tema nedover. Hvert tema trenger teksturer fra eieren, så promptene for neste tema sendes mens det forrige bygges.

## Nivåene

Dette er utkast. De tegnede planene kommer i `docs/krypten/` før noe bygges.

### Gravkjelleren

**Nivå 1, Gravkjelleren under kirka.** Du kommer ned trappa fra krypten til et lite gravkapell med hvilestedet.
- **Hovedveien:** gravgangene med nisjer går til beinhuset, en rund sal med hodeskaller stablet i hyller oppover veggene og et galleri rundt.
- **Spaken:** fra galleriet trekker du en spak som åpner gitteret mot dypet.
- **Hemmelighetene:**
  - En falsk vegg bak et alter med en sjelden komponent.
  - En gammel brønnsjakt du kan falle ned i, som fører til et lite gravkammer med en lore-bok og stiger opp igjen.
- **Snarveien:** en eikedør med slå tilbake til kapellet.
- **Fellen:** en trykkplate med piler i en lang gang. Den læres trygt, for pilene treffer gulvet foran deg første gang.
- **Fiendene:** knapper og tooltips.
- **Porten ned:** trappa ned.

**Nivå 2, Gravkongens hall.**
- **Hovedveien:** en lang søylegang med sarkofager, en sal i to etasjer med balkonger og en trapp som går rundt en stor gravkiste.
- **Spaker:** to spaker i hver sin fløy senker en bro over et hull til arenaen.
- **Hemmeligheten:** en snarvei gjennom en sprukken sarkofag.
- **Fallemmen:** du faller ned i en kjeller med fiender, og en stige fører opp igjen.
- **Bossen:** Modalen i et rundt gravkammer.

### Kloakken og cisternene

**Nivå 3, Kloakken.**
- **Vannstanden:** en hovedkanal med gangbaner på hver side og bruer over. En sluse styrer vannstanden. Når vannet senkes, åpnes nye ganger nede i kanalen.
- **Hemmeligheten:** et gitter bak en foss.

**Nivå 4, Den store cisternen.**
- **Salen:** en enorm sal med søyler som står i vann, og gangbroer i flere høyder.
- **Bossen:** Tabellen kjemper på en flåte av plattformer.

### Gruva og grottene

**Nivå 5, Gruva.**
- **Veien ned:** stoller med tømmerstiver, en heis med vinsj og malmvogner på skinner.
- **Fellen:** steinras.

**Nivå 6, Krystallgrotta.**
- **Grotta:** naturlige grotter med lysende krystaller, en undergrunnssjø og en vindeltrapp hogd i berget.
- **Bossen:** Dropdownen i et stort grottekammer.

### Borgen under jorda

**Nivå 7, Den sunkne borgen.**
- **Borgen:** en borggård under et steintak, porthus og en vindebro. Brystvern du kan gå på.
- **Avstandsfiender:** de skyter fra brystvernet.

**Nivå 8, Ridderhallen.**
- **Hallen:** søylesaler med statuer, våpenkammer, kapell og tronsal.
- **Fellene:** pendler.
- **Bossen:** Navbaren i tronsalen.

### Den korrupte kjernen

**Nivå 9, Overgangen.**
- **Korrupsjonen:** steinen går over i plast og lysende sømmer. Rom glitcher mellom to tilstander.
- **Gangene:** de vrir seg og henger i løse luften over et tomrom.

**Nivå 10, Kjernen.**
- **Veien:** en spiral nedover mot rotunden.
- **Bossen:** Rammeverket.

## De tegnede planene

Alle ti står i `docs/krypten/nivaa_01.png` … `nivaa_10.png`. Dataene ligger i `tools/crypt/levels/level_NN.py`, og verktøyet `tools/crypt/plan.py` tegner dem og sjekker dem (`python3 tools/crypt/plan.py`). Byggeskriptet i Blender leser de samme dataene senere.

| Nivå | Navn | Rom | Min | Slutt |
|---|---|---|---|---|
| 1 | Gravkjelleren under kirka | 13 | 11 | spaken åpner gitteret |
| 2 | Gravkongens hall | 16 | 17 | Modalen |
| 3 | Kloakken | 14 | 15 | eliten (Select) |
| 4 | Den store cisternen | 15 | 17 | Tabellen |
| 5 | Gruva | 15 | 14 | eliten (Progress-formannen) |
| 6 | Krystallgrotta | 16 | 17 | Dropdownen |
| 7 | Den sunkne borgen | 13 | 14 | eliten (Kortkapteinen) |
| 8 | Ridderhallen | 14 | 17 | Navbaren |
| 9 | Overgangen | 13 | 14 | eliten (Tvillingvokteren) |
| 10 | Kjernen | 12 | 18 | Rammeverket |

Minuttene er et anslag fra planen (gange, sidegrener, kamp, feller og bosser); til sammen rundt 2,5 timer.

**Fiendene per tema** (nye slag; de eldre kommer igjen sterkere):

| Tema | Nye slag |
|---|---|
| Gravkjelleren | Button, Tooltip, Avatar (hodeskaller som ruller), Breadcrumb (beinkjede som kryper), Accordion (kister i bakhold) |
| Kloakken og cisternen | Input, Select, Toast, Pagination |
| Gruva og grotta | Spinner, Progress, Slider, Chip-svermer |
| Borgen | Card (skjoldbærer), Tabs (vakt); Breadcrumb kommer igjen som bueskytter |
| Kjernen | Grid, Skeleton loader, Popover |

## Teknisk

- **Nivåformatet:**
  - Skriptene ligger i `tools/models/crypt/`: ett byggesett per tema (`kit_grav.py` osv.) og ett skript per nivå (`level_01.py` osv.).
  - Hvert nivåskript setter modulene på et rutenett på 1 m (rom, ganger, trapper) og eksporterer nivået i biter på 16 m (`crypt_01_a.glb` …). Bitene lages så rendereren kan kutte det som ikke synes.
  - Ved siden av eksporteres `crypt_01.json` med kolliderne, lysene, merkene og navmeshen.
  - **Merkene:** start, hvilested, trapp ned, fiender med patruljeruter, kister, hemmeligheter, spaker, feller og snarveier.
- **Navmesh:**
  - Hver modul i byggesettet har gangbare flater (polygoner på gulv, trapper og ramper).
  - Nivåskriptet syr dem sammen til ett navmesh med naboer og lagrer det i json-fila.
  - Fiendene finner veien med A* over polygonene og glatter den med trakt (funnel). Mot hverandre styres de med en enkel avstandsregel.
  - Dører, gitter, broer og vannstand slår polygoner av og på.
- **Testene:**
  - Hele nivået kan gås fra start til trappa ned med ekte fysikk.
  - Hver fiende har vei til spilleren fra hvert rom på hovedveien.
  - Ingen polygon er avskåret, uten at den er hemmelig og har en vei ut.
  - Taket står høyt nok overalt.
  - Ingenting svever.
- **Lyset:** lysene stoppes av murene. Rutenettet i `grid_lit` byttes mot lysvolumer per rom (portaler mellom rommene).
- **Ytelse:** budsjettet i `docs/ny-stil.md` gjelder også nede. Et nivå tegner bare rommene som kan synes fra rommet du står i (portaler), med under 300 tegnekall.
- **Teksturlagene:** `LAYERS` er nå 46 av 48 mulige. Når den gamle kjelleren slettes, forsvinner noen gamle lag (`cryptwall`, `concrete`, `soot`). Grensen heves til 64 (`tex_scale` til 16 × 4), så alle fem temaene får plass.

## Det som slettes

- `dungeon.rs` sin generator (`plan_floor`, `Plan::grow`, gangmodulene).
- `tools/models/dungeon.py` og alle `dng_*`-modellene.
- Testene som tester generatoren: `alle_etasjer_kan_gas_gjennom`, `fiendene_gar_rundt_det_som_star` og `hodet_gar_ikke_gjennom_taket` i dagens form. De skrives om for de faste nivåene.
- `gjennomspilling` skrives om for de faste nivåene.
