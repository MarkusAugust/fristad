# Nivåredigeringen for krypten

*Plan skrevet 18. oktober 2026, etter eierens beslutninger samme dag (`docs/plan.md`, del 15). Endret 19. oktober: eieren jobber med **moduler**, ikke biter (del 21). Dette er planen i detalj som del 15 sier skal vises eieren før arbeidet starter. Den tar for seg formatet, bitene, sammenføyningen, navigasjonen, reglene, lagringen, skjermbildet, fasene og testene.*

## Kort oppsummering

- **Eieren bygger nivåene selv** på en egen side (`?nivaa`). Til venstre ligger et kart ovenfra, etasje for etasje, og til høyre et 3D-vindu som oppdateres med en gang. Én knapp spiller nivået fra starten med fiender og alt, og Esc tar deg tilbake dit du var.
- **Eieren bygger med moduler, som Lego** (eieren 19. oktober, del 21): ferdige rom i ulike størrelser, ganger (rett, hjørne, T, kryss, blindgang), trapper (halvtrapp, halv etasje, nedgang), rotunde og ferdige rom med innhold (kapellet med døra opp, gravgangen, skattkammeret). En modul dras ut, dreies og speiles, og føyes sammen av seg selv der den møter en annen: **åpning der begge har en åpning på samme kant, mur ellers.** Eieren kan overstyre hver åpning: dør, port, falsk vegg, gang, bue, snarvei eller mur.
- **Inni er modulene ekte Blender-modeller** på et rutenett på 4 × 4 m med høyde i trinn på 1 m: celler (romfelt, gang, trapp …), og murene mellom cellene velges automatisk fra byggesettet (**halvkanter**, én halv mur per side; **kvarthjørner**, søyler og hjørner; **gulv og hvelv**). Bitene og cellene er bare det kompilatoren bruker; eieren ser dem aldri (en «avansert» skuff for Claude). Ingen geometri lages i Rust. Kjernen velger bare hvilke ekte modeller som står hvor, så regelen «alt som synes er en ekte modell» holder.
- **Kildefila er `assets/levels/nivaa_NN.json`.** Den holder modulene med rotasjon, speiling og eierens valg for åpningene, innholdet, koblingene, pynten og lysene. Modulene selv er maler i `assets/levels/moduler/<tema>/<navn>.json`.
- **En ny kompilator i kjernen** (`fristad-domain`, uten fs eller wgpu) gjør fila om til den samme `LevelFile` som motoren bruker i dag: instanser, kollidere, navigasjonsnett, rom, lys og merker. Redigeringen og spillet kjører den samme koden, så de bygger likt. Blender trengs bare når bitene lages, aldri når et nivå bygges.
- **Reglene er de samme som testene.** De kjøres mens eieren bygger og står som en liste med «Gå til» og «Rett». En egen løser går gjennom nivået med nøkler, spaker og fall, og finner dører som ikke går noen vei, spaker uten port, rom du ikke når, nøkler bak sin egen port og steder du blir sittende fast.
- **Utkast og lagring:** utkastet huskes i nettleseren. Lagre legger nivået på grenen `redigering` som byen, med sjekk mot det som ligger der fra før. Et nivå er «utkast» til eieren merker det «klart». Bare klare nivåer erstatter de gamle i spillet, og de gamle står til da.
- **Rekkefølgen:**
  1. kompilatoren med et grått pilotsett,
  2. pilotsettet for gravkjelleren i malt stil, satt sammen til moduler (eieren godkjenner modulene),
  3. siden med modulpaletten og ett lite testnivå som eieren bygger,
  4. mekanismene og løseren,
  5. variasjon og stemning,
  6. full vertikalitet,
  7. ytelsen målt på M2,
  8. de fire andre temaene.

  Etter det bygger eieren nivå 1–10 fra blankt.

---

## 1. Mål og prinsipper

1. **Samme kode i redigeringen og spillet.** Nivået lagres som kildedata. Kjernen kompilerer det til `LevelFile` (`crates/fristad-domain/src/domain/world/crypt/level.rs`) ved lasting, både i spillet, i redigeringen, i `cargo test` og i `snapshot`. Det finnes ingen genererte filer som kan komme ut av takt med kilden.
2. **Alt som synes, er ekte modeller.** Bitene lages i Blender (`tools/crypt/kit_<tema>.py`, med funksjonene fra `kit_grav.py`: `wall`, `plinth`, `groin_bay`, `arch_profile`, `pillar`, `steps`, `parapet` …). Kompilatoren velger og plasserer dem og lager aldri geometri. Når rendereren slår sammen instanser til større biter for å spare tegnekall (del 14), setter den bare sammen ekte modeller.
3. **Heksagonal arkitektur.** Kjernen kjenner bare data:
   - byggesettets beskrivelse (`KitDef`),
   - nivådokumentet (`LevelDoc`),
   - resultatet (`LevelFile` med instanser).

   Adapterne leser glb og json, nettleseren tegner kartet, og GitHub lagrer. `src/architecture.rs` passer på grensene som før.
4. **Det motoren gjør i dag, står.** `Crypt::build`, `nav.rs` (A* og trakt), `map.rs`, `run.rs`, mekanismene, fiendene, `light_mask` og alle testene over `Crypt` brukes uendret. Det nye ender i samme kontrakt (`docs/krypten/format.md`) med ett nytt felt for instanser.
5. **Modulen er enheten eieren jobber med** (del 21). Alt eieren ser i redigeringen er moduler, åpninger og innhold. Cellene, halvkantene og kvarthjørnene er kompilatorens; de vises bare i en avansert skuff.
6. **Faste nivåer.** Nivåene er like hver gang. Frø brukes bare til å velge varianter og pynt, regnes ut av cellens plass og er låst i fila (del 9).

## 2. Hvordan det henger sammen

**Kjeden:**

1. **Blender** (`kit_<tema>.py`) gir hver bit som glb. Kolliderne ligger i `extras.solids` som i dag (`solids.py --models`), og i `extras.kit` står flatene, fotavtrykket, navigasjonsflatene, lysfestene og pyntefestene. I tillegg skrives en liten manifestfil per tema (`assets/models/crypt/kit/<tema>.json`).
2. **Adapterne** (`adapters/data.rs`, `gltf.rs`) leser manifestene og nivådokumentene og gir kjernen `KitDef` og `LevelDoc` gjennom `ModelSource::world_data`.
3. **Kjernen** (ny modul `domain/world/crypt/compose/`) gjør `LevelDoc` + `KitDef` om til en `LevelFile` med instanser, rom, kollidere, nett, lys og merker. Reglene (`domain/world/crypt/check/`) går over både dokumentet og resultatet.
4. **`Crypt::build`** bygger som før, men setter ut `instances` i stedet for bitene `crypt_NN_aa`.
5. **Redigeringen** (`domain/edit/level/`) endrer `LevelDoc` med angre og gjenta, kompilerer det som er endret på nytt og sender et kartbilde og 3D-instanser ut gjennom portene.

**Overgangen fra de gamle nivåene:** `build.rs` velger per nivå. Er `assets/levels/nivaa_NN.json` merket `"status": "klar"`, kompileres den. Ellers brukes dagens `assets/models/crypt/crypt_NN.json`. De gamle står altså til hvert nytt nivå er klart, og så slettes de ett for ett. Når alle ti er nye, slettes `build_level.py`, `layouts/`, `levels/` og `crypt_NN_*.glb`.

**Lasting nivå for nivå** (`adapters/delivery.rs`): bitene hentes per tema, ikke per nivå (`dist/models/crypt/kit/<tema>/`). Nivå n trenger temaene som rommene i nivået bruker. Manifestene og nivådokumentene er små og hentes ved start. Meshene hentes ved `Event::FetchLevel` som i dag.

## 3. Rutenettet og standardmålene

Faste mål gjør at alle temaene passer sammen. De bestemmes én gang i pilotfasen og ligger som konstanter både i kjernen (`compose/grid.rs`) og i `kit_common.py`.

| Mål | Verdi | Hvorfor |
|---|---|---|
| Celle | 4 × 4 m (i, j heltall; x = 4i, z = 4j i nivåets rom) | eieren |
| Gulvhøyde | heltall meter (y) | eieren |
| Kant | 0,8 m tykk mur, delt i to halvkanter på 0,4 m (en per side) | tema og form per side; `WALL_T` 0,4 |
| Kantsone | de ytterste 0,5 m av gulvet langs en kant hører til halvkanten | okklusjonen ved veggfoten og sokkelen stemmer; åpne kanter får gulv uten mørk stripe |
| Åpninger (felles for alle temaer) | dør 1,6 × 2,3 m (`SHORTCUT_H`); port 2,8 × 3,0 (`GATE_W`, `GATE_H`); gang 2,0 og 3,0 m; bue 3,2 × 3,4; helt åpen 4,0 | de to halvdelene av lysningen møtes alltid |
| Standardrom | 4 m til vederlaget og hvelv over; rommet bruker y−0,5 til y+5,5 | under `HEADROOM` (2,9) aldri |
| Gang | 2,6 m til vederlaget; bruker y−0,5 til y+3,6 | |
| Etasje (vanlig) | 6 m mellom gulvene over hverandre | gulvskive og hvelv får plass |
| Trapper | 1 m over én celle (halvtrapp), 2 m over én celle (halv etasje), 4 m over to celler (hel etasje); vindeltrapp 4–6 m i én celle | helningen ≤ 27°, som kapselen tåler |
| Murhøyder over 4 m | stables av moduler på 1 m med topp, som stigen (`ladder_pieces`) | ingen strukket tekstur |
| Uv | i meter; steinbildet gjentar seg hver 2 m (`STONE_TEX_M`) | 4 m er et multiplum, så skjøtene møtes ved flytting, dreiing og speiling |

**Etasjene i kartet** er høydebånd eieren navngir, for eksempel «Hovedplanet» fra 0 til 5 og «Kjelleren» fra −6 til −1. Kartet viser cellene som har gulv i det aktive båndet og de andre svakt. Et gulv kan stå på hvilken som helst heltallshøyde. Båndene er bare en visning.

## 4. Bitene (byggesettet)

*Bitene er det modulene er bygd av (del 21). Eieren setter ut moduler; bitene velges inni og dømmes gjennom modulene på godkjenningssiden.*

### 4.1 Slag

| Slag | Hva | Eksempler |
|---|---|---|
| **Cellebit** | det som fyller en celle: gulv, hvelv og innmat | romfelt med kryssribbet hvelv; gang rett, hjørne, T, kryss og blindgang (i 2,0 og 3,0 m bredde); trapp 1/2/4 m; vindeltrapp; dobbel høyde (8–9 m); sjakt med stigefeste; kløft (ingen bunn, eller bunn langt nede); basseng; brofelt |
| **Halvkant** | en side av en kant, velges automatisk | vegg; vegg med dør, port, gang eller bue; åpen med gjordbue; overmur (der takene er ulike); vindu og gitter (ser gjennom, går ikke gjennom); brystning og rekkverk; trappevange (skrå); falsk vegg (samme mur som veggen, med sprekker); avslutning mot fjell; trinnmur (åpning over et lavere gulv) |
| **Kvarthjørne** | det som står i et hjørne der fire celler møtes, velges automatisk | frittstående søyle; innvendig og utvendig hjørne; pilaster der en vegg slutter; T og kryss; gjordbueanfang; i stabler for høye rom |
| **Sammensatt bit** | flere celler som én modell, med kantene ut mot resten | rotunde 3 × 3 (som beinhuset) med kuppel og galleri, tårn 3 × 3 i flere etasjer, apsis 2 × 1, stor hall 3 × 4 uten søyler, skrå gang 2 × 2 (45°), grotte 2 × 2 og 3 × 3 med ujevn vegg |
| **Mekanisme** | dagens `kit_*`, som motoren setter ut etter merkene | `kit_gate`, `kit_lever`, `kit_shortcut`, `kit_bridge`, `kit_lift`, `kit_water`, `kit_trapdoor`, `kit_ladder` … |
| **Pynt** | fri eller i et pyntefeste | sarkofager, hyller med hodeskaller, kjeder, bein, søppel, fat, røtter, fakler, lysestaker (dagens `PROPS` og nye) |

### 4.2 Det hver bit har med seg (`extras.kit` og manifestet)

- **id og versjon:** `grav/romfelt`, `v`. Det se under 13.4.
- **fotavtrykk:** cellene den dekker, i bitens rom, og **volumet** den bruker i høyden (fra og til, i meter over gulvet). Volumet brukes mot overlapp.
- **flatene:** for hver side av hver celle i fotavtrykket står det
  - hva siden *vil ha*: `lukket`, `åpen`, `gang-2`, `gang-3`, `trapp-topp`, `trapp-bunn`, `vann`, `kløft`, eller en åpning som er fast i biten,
  - gulvhøyden ved kanten (fast eller skrå),
  - takhøyden.

  Over og under står `tak`, `hull` (for sjakt og fallem) og `luft` (for dobbel høyde).
- **gangbare flater:** polygoner på gulv, trapper og ramper, trukket 0,4 m inn fra bitens egne murer. Ved åpne kanter går de helt ut til kanten.
- **kollidere:** `extras.solids`, som følger formen. Gulvene ligger nøyaktig i flukt med heltallshøyden, uten små kanter.
- **lysfester og pyntefester:** fakkelfester på murer, nisjer, kroker i hvelvet og hjørner på gulvet, med slag og retning.
- **speiling:** om biten kan speiles. Har biten tekst eller annet som ikke tåler speiling, peker den på sin egen speilede modell.
- **varianter:** for eksempel intakt, forfall, ras, røtter, mose, glitch, med vekt per variant.
- **godkjent:** dommen står i `approvals.json` som for alle andre modeller.

### 4.3 Hvordan bitene lages

- Bitene lages i Blender med dagens murer (`crypt_wall`, `crypt_floor`, `dressed`, `rib`, `vault` …) og lagene for hvert tema (plan del 12, steg 5).
- **Okklusjonen bakes i sammenheng:** hver bit bakes stående i en standard omgivelse av naboer, så lyset langs kantene går over i nøytralt og skjøtene ikke synes. Kantsonen (tabellen i del 3) sørger for at mørket ved veggfoten hører til veggen.
- **Nær- og fjernversjon** av hver bit, siden rendereren har `near`/`far`.
- **Tetthetssjekken** (som `leak_check.py`) kjøres per bit i Blender: hver flate som sier `lukket`, skal være tett.

### 4.4 Pilotsettet (gravkjelleren, rundt 20 biter)

- romfelt med kryssribbet hvelv, i to varianter,
- gang 3,0: rett, hjørne, T, kryss og blindgang,
- halvtrapp og halv etasje,
- halvkantene vegg, dør, port, gang 3,0, åpen med gjordbue, overmur, rekkverk og falsk vegg,
- kvarthjørnene søyle, innvendig hjørne, utvendig hjørne og pilaster,
- én sammensatt rotunde 3 × 3,
- fakkelfestet,
- de mekanismene vi har.

Resten av settet (dobbel høyde, sjakt, kløft, bro, vindeltrapp, vindu og gitter, flere varianter) kommer i fase 6.

## 5. Sammenføyningen

**Først modulene** (del 21): der to moduler møtes og begge har en åpning på samme kant, blir det en åpning (dør mellom to rom, gangåpning der en gang er med, bue om en av dem vil ha bue, og port, falsk vegg eller snarvei om en av dem ber om det; eierens valg går foran). Møtes de langs flere kanter som passer, blir bare den midterste åpen. Ellers er det mur. Etter det er nivået celler med kantvalg, og reglene under gjelder som før.

### 5.1 Celler, halvkanter og kvarthjørner

Modulene (og den avanserte skuffen) gir **cellebiter** og **sammensatte biter**. Kompilatoren regner ut resten:

1. **Kantene.** For hver kant mellom to celler, eller mellom en celle og tomt fjell, slås de to sidenes ønsker sammen med eierens overstyring (se tabellen). Det gir **åpningen** for kanten. Så velges én **halvkant** for hver side ut fra
   - sidens tema,
   - åpningen,
   - gulv og tak på begge sider (flat mot flat, ulike høyder, skrå mot flat, skrå mot skrå),
   - variant og frø.
2. **Hjørnene.** Hvert hjørne der fire celler møtes, får et **kvarthjørne** per celle. Det velges etter de to kantene cellen har der (som «marching squares»: 16 tilfeller som blir 6 med dreiing og speiling) og etter tema.
3. **Gulv og hvelv** kommer fra cellebiten. Mellom to felt i samme rom med samme tak står en gjordbue på den åpne kanten, så hvelvene møtes riktig.
4. **Tomt er fjell.** Alt som ikke er en celle, er massivt berg. En side mot fjell blir alltid vegg, så ingenting blir stående åpent ut i fjellet.

**Slik slås de to sidene sammen:**

| Side A vil | Side B vil | Eieren har ikke valgt | Om eieren har valgt |
|---|---|---|---|
| `åpen` (rom) | `åpen`, samme rom | åpen med gjordbue (søyler i hjørnene om rommet vil det) | vegg, bue, vindu, gitter … |
| `åpen` | `åpen`, et annet rom | **vegg** (rommene skilles) | dør, port, bue, åpen, vindu, gitter, falsk vegg, snarvei |
| `åpen` | `gang-3` | vegg med gangåpning 3,0 | port, dør (smalere), falsk vegg |
| `gang-3` | `gang-3` (samme akse) | gangen fortsetter (bare en gjordbue) | port, dør, falsk vegg |
| `gang-x` | `lukket` eller fjell | **blindgang**, gangen lukkes | regel: «gangen ender i vegg» om eieren har bedt om åpning |
| `trapp-topp` eller `-bunn` | gulvet passer | åpning som profilen | port, dør |
| `trapp-x` | gulvet passer ikke | **konflikt** | — |
| hva som helst | fjell | vegg | en åpning her er en feil: «dør som ikke går noen vei» |
| `kløft` | `åpen` | rekkverk på rom-siden | åpen (du kan falle) |
| `vann` | `vann` | samme basseng | — |

### 5.2 Høyder

- **Gulvene er ulike over en kant.** Ved 1 m eller mer uten trapp får den lave siden en trinnmur under åpningen og den høye siden rekkverk. Eieren kan velge «åpen kant», så du kan hoppe ned. Det blir en vei bare én vei i løseren (del 10).
- **Takene er ulike.** En overmur fyller fra det lave taket opp til det høye, med buen under.
- **Dobbel høyde.** En celle med tak på 8–9 m. Cellen over i samme kolonne er `luft` og kan ikke brukes til noe annet (det sjekkes mot overlapp).
- **Galleri og balkong.** En celle i et høyere bånd med flaten `luft` under seg, inne i et rom med dobbel høyde. Kanten mot lufta får brystning. Biten har en underside, så du ser hvelv og ikke hull nedenfra. Galleriet hører til salens rom eller er sitt eget rom som overlapper. Overlappet gir delt lys (`overlaps`) som i dag.
- **Rom over rom.** Lov når volumene ikke overlapper (gulvskive og hvelv er med i volumet).

### 5.3 Trapper, sjakter, broer, vann og fall

- **Trapper:** trappebitens topp og bunn skal møte gulv i riktig høyde på riktig side. En trapp som ender i en vegg eller i lufta, er en feil som stopper «klar». Vindeltrappa er én celle med en åpning nede og en oppe.
- **Sjakter:** en søyle av celler med `hull` og `hull`, med et stigefeste eller heis (`lift`) og en avsats oppe og nede. Regelen sjekker at sjakta er rett, at avsatsene finnes og at det er en vei opp (stige eller heis). Fallem (`trapdoor`) er en celle med `hull` og et rom rett under (`to` fylles ut automatisk).
- **Broer over kløfter:** kløftceller på rad. Brobiten eller mekanismen `bridge` får lengden (4 m × antall celler), feste i taket over (`top`) og hvor høyt den henger. En fast bro er en cellebit, en bro som senkes er et merke.
- **Vann:** bassengceller. Kompilatoren lager ett `water`-merke per rektangel i bassenget, alle med samme `id`, og dermed samme bryter. Vannstanden står i bassengets egenskaper (høy og lav). Regelen: hovedveien må kunne gås når vannet står høyt, eller spaken som senker det må nås før.
- **Fall:** åpne kanter, fallemmer og kløfter er veier bare én vei.

### 5.4 Sammensatte biter og store rom

- Rotunder, tårn og haller uten søyler er sammensatte biter. Innenfor er de en ferdig modell med eget omriss (sirkelen blir rommets polygon) og eget nett. Utover har de flater på kvadratet. Bare noen celler har åpninger (for eksempel midten av hver side). Resten er `lukket` med fjell mellom sirkelen og kvadratet.
- Store saler kan også bygges av vanlige romfelt med søyler i hjørnene. Det er slik en ekte krypt er bygd, fag for fag.

### 5.5 Temaoverganger

Hver halvkant og hvert kvarthjørne tar temaet fra sin egen celle. En dør mellom kloakken og gravkjelleren er derfor kloakkmur på den ene siden og gravmur på den andre, og de møtes midt i lysningen. Åpningene har samme mål i alle temaer (tabellen i del 3), så det går alltid opp. Rammen rundt døra står på siden eieren velger (standard er rommet med høyest prioritet i hovedveien). Egne overgangsbiter (en bue med blandet stein) kan komme senere. Steinen følger samme regel (del 22).

### 5.6 Konflikter

- **Bitene overlapper** (fotavtrykk og volum): fantomet blir rødt og plasseringen nektes. Med Alt erstatter den nye biten den gamle, og innholdet på den gamle følger med eller blir løst (det står i angresteget).
- **To sider som ikke passer:** kompilatoren lager likevel noe tett og trygt (vegg) og merker kanten rød med grunnen. Nivået kan fortsatt spilles.
- **Overstyring som ikke går** (port der den ene siden er fjell): vegg og en feil i lista.
- **En bit mangler i byggesettet** (eldre versjon, ukjent id): en grå kasse og en feil. Nivået lastes likevel, like rausing som `level.rs` i dag.

## 6. Dataformatet: `assets/levels/nivaa_NN.json`

Fila skrives i fast form, med én ting per linje sortert etter id (som `layout_json`), så endringene er lette å lese i git. Alle id-er er stabile og byttes ikke når noe annet endres.

| Felt | Innhold |
|---|---|
| `format` | versjon av formatet (1). Kjernen har migreringer fra hver eldre versjon. |
| `level`, `name`, `status` | nummeret (1–10, eller `test`), navnet, og `utkast` eller `klar` |
| `seed` | nivåets frø. Varianter og pynt regnes av frøet og cellens plass, ikke av rekkefølgen. |
| `kit` | settene og versjonene nivået ble laget med (`grav@3`, `kloakk@1`) |
| `bands` | etasjene i kartet: navn, fra og til (m) |
| `modules` | modulene eieren har satt ut (del 21): `id`, `module` (`grav/kapell`), `at` (i, y, j for modulens (0, 0)), `rot` (0–3), `mirror`, `room` (valgfri: rommet modulen hører til, om flere moduler skal være ett rom), `name`, `mood`, og `overrides` (eierens valg per åpning: `opening`, `kind` og `id` for porten, snarveien og den falske veggen) |
| `cells` | løse celler fra den avanserte skuffen (et nivå av moduler har ingen): `id`, `bit` (`grav/romfelt`), `at` (i, y, j), `rot` (0–3), `mirror`, `room` (rom-id), `variant` (valgfri, låst), `tags` |
| `edges` | overstyringene: kanten (celle og side), `kind` (dør, port, bue, åpen, vindu, gitter, rekkverk, falsk, snarvei, vegg, åpen kant), `owner` (siden med rammen), `variant` |
| `rooms` | `id`, `name`, `theme`, `secret`, `mood` (lysstemning, tåke, lyd, forfall 0–1, pyntetetthet), `priority`. Cellene peker på rommet, ikke omvendt, så en celle aldri står i to rom. |
| `things` | innholdet: `id`, `kind` (som merkene i dag: `start`, `up`, `down`, `rest`, `mob`, `elite`, `boss`, `chest`, `rare`, `lore`, `bonus`, `lever`, `trap`, `hint`, `lift`, `ladder`, `bridge`, `water`, `trapdoor` …), `anchor`, `pos` og `yaw` i ankerets rom, og egenskapene (slag, antall, patrulje, bakhold, innhold, tekst, størrelse …) |
| `anchor` (i `things` og `decor`) | `fri` (nivåets rom), `celle:<id>` (følger biten når den flyttes, dreies eller speiles) eller `kant:<celle>/<side>` (porter, snarveier og falske vegger som *er* kanten) |
| `links` | koblingene: `from`, `to`, `kind`. Slagene er `opens` (spak eller elite til port, bro, vann eller heis), `key` (eliten bærer nøkkelen), `arena` (port til rom), `exit`, `for` (hint til hemmelighet), `to` (fallem til rom), `from` (snarveiens slåside). Målet har `requires`: `én` eller `alle` (flere spaker til én port). |
| `decor` | pynt: `id`, `model`, `anchor`, `pos`, `yaw`, `scale` (smal grense), `layer` (pyntelaget) og `slot` (pyntefestet den står i, eller fri). Pynt som er satt ut automatisk og så endret, lagres her. Det som ikke er endret, regnes ut på nytt. |
| `lights` | lys eieren har satt: fritt, eller i et lysfeste med endrede egenskaper (farge, rekkevidde, styrke, blafring) |
| `view` | bare for redigeringen: siste etasje, kamera og bokmerker (holdes utenfor sammenligningen av endringer) |

**Modulene** (malene) ligger i `assets/levels/moduler/<tema>/<navn>.json` med cellene, rommet, standardinnholdet, lysene, åpningene og murene i modulens eget rom (midtcella i (0, 0), `rot` 0). Se del 21. Rom-malene eieren lager selv (del 9) blir moduler i samme format.

## 7. Fra dokumentet til det motoren trenger (kompilatoren)

Modulene ligger i `crates/fristad-domain/src/domain/world/crypt/compose/`. Ingen fs, klokke eller wgpu. Alt er rene funksjoner som tar `LevelDoc` + `KitDef` og gir `LevelFile` og en rapport.

1. **Rutenettet:** cellene slås opp i en tabell over (i, y, j). Fotavtrykk og volumer regnes ut med rotasjon og speiling.
2. **Kantene og hjørnene** (del 5) gir instansene: modell, sted, dreining og speiling. De går i et nytt felt `instances` i `LevelFile`, som `Crypt::build` setter ut med `place_mounted` (de er murverk med fundament og måles ikke mot gulvet, som bitene i dag). `pieces` står for de gamle nivåene.
3. **Kolliderne:** bitenes `solids`, flyttet på plass. De **flettes**:
   - Gulvene i samme plan blir store rektangler, så kapselen ikke henger seg opp i skjøtene mellom boksene.
   - Murbokser i samme plan og tykkelse slås sammen langs rette murløp.

   Målet er under rundt 15 000 kollidere per nivå (i dag 500–9 500).
4. **Navigasjonsnettet:** bitenes gangbare flater samles per plan. Rundt mekanismene og pynten på gulvet gjøres det hull: fotavtrykket utvidet med 0,4 m tas bort. Så lages konvekse polygoner. Fremgangsmåten er robust og enkel: flatene rastres i ruter på 0,25 m per plan, rutene slås grådig sammen til rektangler, og trapper og ramper tas med som skrå firkanter fra biten. `Nav::new` sveiser og kobler kanter som før. Hver polygon får rom og bryter (`poly_gate` for porter, falske vegger, broer og vann under).
5. **Rommene** (til lyset, kartet, `room_at` og tåkeleggingen):
   - Omrisset er cellenes union (et sammensatt rom tar sitt eget omriss), `y` er det laveste gulvet og `h` til det laveste taket over det gangbare.
   - Et rom med hull (en ring rundt en blokk) deles automatisk i to, fordi `RoomDef.poly` ikke kan ha hull.
   - Forbindelsene for lyset regnes som i dag fra nettet. Vinduer og gitter gir en egen lysforbindelse uten nett.
6. **Merkene:** innholdet blir `MarkDef` i nivåets rom. Koblingene blir `opens`, `arena`, `for`, `to` og `from`. Porter og falske vegger på en kant får sted midt i kanten, retning etter kantens normal og bredde etter åpningen.
   - Den falske veggen blir halvkantens egen variant `falsk`, som er den samme muren som rundt den. Bitene `crypt_NN_secret_<id>` per nivå trengs ikke lenger.
   - Sprekke-hintet legges på veggen som i dag.
7. **Lysene:** lysfestene i bitene og rommets stemning gir fakler og lysestaker (frø og tetthet). Eierens lys går foran. Hvert lys får rom.
8. **Pynten:** pyntefestene fylles etter rommets pyntesett og tetthet, med frø (del 9). Eierens pynt går foran.
9. **Deterministisk:** samme dokument og samme byggesett gir byte for byte samme `LevelFile`, både i wasm og i maskinkode (hjørnene avrundes til mm).
10. **Bare det som er endret:** et lite inngrep regner bare cellene innen én celle fra det som ble endret, nettet i de planene det rører og reglene som gjelder. Løseren og lyslekk-sjekken venter 150 ms etter siste endring. Målene: under 16 ms for en vanlig endring og under 150 ms for et helt nivå på 60 × 60 celler i wasm. Testen krever at det å regne bare det endrede gir det samme som å regne alt på nytt.

**Nivåets plass i verden** er uendret: origo = ankeret under krypten minus `start`, `LEVEL_DROP` × n ned, og flyttet innenfor `REACH` (±185 m). Redigeringen tegner grensen som en rød ramme på kartet etter hvor `start` står.

## 8. Innholdet

### 8.1 Mekanismer og koblinger (koblingsmodus)

- **Spak til port, bro, vann eller heis:** velg spaken, trykk K (eller dra fra den lille kroken på spaken) og slipp på målet. Koblingen tegnes som en bue på kartet og i 3D, i farge etter slag. Et mål med flere spaker velger `én` eller `alle`. I dag er det ikke eksplisitt (broa senkes når «alle spakene» er trukket), så motoren må få støtte for begge, og det testes.
- **Nøkkel:** eliten kobles til porten (`key`).
- **Arena:** et rom merkes som arena. Portene inn kobles (`arena`), og én er `exit`. Bossen settes i rommet.
- **Snarvei:** en kant av slaget snarvei. Siden med slåen velges med en pil på kanten.
- **Hemmelig vegg og rom:** en kant av slaget falsk vegg. Rommet bak merkes hemmelig, og hint (sprekk, trekk, lyd) foreslås og kobles (`for`).
- **Feller** (pil, pigger, steiner, pendel): trykkplata og retningen settes med et håndtak. Flom og gass kommer når motoren har dem.
- **Heis, bro og vann:** som i 5.3, med egenskapene i panelet.
- **Fallem og stige:** fallemmen kobles til rommet under (fylles ut automatisk når det står rett under). Stigen fra bunnen foreslås.

### 8.2 Fiender

- **Slag** fra en liste (`mob_kind.rs`), med antall, og vanlig, elite eller boss.
- **Patrulje:** punkter på kartet som dras. Hvert punkt festes til nettet, og streken går langs veien de faktisk går (A* tegnet).
- **Bakhold:** av, en radius (en sirkel på kartet) eller rommet de vekkes i.
- **Siktet** vises som en kjegle i 3D når fienden er valgt.
- **«Kan ikke nås – med vilje»:** for skyttere på et brystvern man ikke kommer opp på, så regelen ikke klager.

### 8.3 Kister, lapper og bonuser

- **Kister:** innhold fra en liste over komponentene (`craft.rs`), tilfeldig fra nivåets liste, eller sjelden (`rare`).
- **Lapper:** tittel og tekst med forhåndsvisning i Fristil-dialogen. Kanon gjelder (`docs/kanon.md`).
- **Bonuser:** liv eller utholdenhet.

### 8.4 Lys og stemning

- **Per rom:** stemning (varm fakkel, kald blå, likgrønn, krystall, glør), tåke (tetthet og farge) og lyd (drypp, vind, kjeder, vann).
  - Tåka følger rommet du står i og glir over på ett sekund. Det trenger bare noen få verdier i `Frame` og ingen tåke per piksel, siden tåka er felles for hele bildet i dag (`fog_color` i `common.wgsl`).
  - Lyden går gjennom `web/ambience.js`.
- **Forhåndsvisning:** tre brytere: *Spillets lys* (mørket og lykta), *Arbeidslys* (flatt og lyst) og *Bare dette rommets lys*.

## 9. Kreativitet og variasjon

- **Varianter per bit** (intakt, forfall, ras, røtter, mose, glitch): valgt med frøet og cellens plass, vektet med rommets **forfall** (0–1). Eieren kan
  - låse en variant per celle,
  - «kaste på nytt» for det valgte (ett angresteg),
  - male forfall med en pensel over cellene.
- **Pynt i lag** (sarkofager, hyller, kjeder, bein, søppel, røtter, fat):
  - Hvert rom har et *pyntesett* og en tetthet, og pyntefestene i bitene fylles etter det.
  - Hvert lag kan skjules, låses og tømmes.
  - Pynt kan settes **fritt** hvor som helst (fantom, fest og dreiing som i dagens redigering: `edit/ghost.rs`, `snap_pose`), også over kanter, så rutenettet brytes opp.
- **Lysstemninger** per rom (8.4), og lys som blafrer der det trekker (hint).
- **Vertikalitet:** dobbel høyde, gallerier, balkonger over salen, broer over kløfter, sjakter, vindeltrapper, fall du kan hoppe ned, og avsatser med utsikt.
- **Utsikt gjennom rom:** vindu og gitter på en kant (du ser gjennom, men går ikke gjennom, og de gir lysforbindelse), gitter i gulvet og hull i hvelvet.
- **Hemmeligheter:** falske vegger, hemmelige rom (skjult på kartet til du har vært der), hint foreslått automatisk og sjeldne kister.
- **Vann og fall:** basseng med sluse, fosser (pynt) og fall bare én vei.
- **Temaoverganger** mellom rom (5.5), med pynt som blander temaene, for eksempel slim som kryper inn i gravkjelleren.
- **Rom-maler:** velg celler, kanter, innhold og pynt og lagre dem som en mal med navn og bilde. Malen settes ut med dreiing og speiling som én handling. Id-ene lages nye, og koblingene inne i malen følger med. Koblinger ut av malen slippes og meldes.
- **Kopier, lim inn, speil og gjenta:**
  - Ctrl+C og Ctrl+V.
  - Ctrl+D kopierer et fotavtrykk til siden.
  - X og Shift+X speiler langs en akse.
  - «Gjenta» gir N kopier med et fast steg, for eksempel en gang med nisjer.
- **Løft og senk:** +1 eller −1 m på cellene som er valgt, med forslag om trapp der kanten nå har trinn.
- **Bryte rutenettet:** sammensatte grotter med ujevne vegger, skrå ganger, fri pynt og ras over kanter, og varianter i hver celle. Målet er at du ikke ser rutenettet når du går der (eierens innvending: «for kjedelige og generiske»).

## 10. Reglene (de samme som testene)

Reglene ligger i kjernen (`domain/world/crypt/check/`) og gjelder både dokumentet og resultatet. `cargo test` kjører de samme funksjonene på alle klare nivåer, og de tunge testene med Rapier kjører i tillegg. Hvert brudd har alvor, sted («Gå til») og om mulig en rettelse («Rett»).

- **Feil** stopper «klar», men aldri lagring av et utkast.
- **Advarsel** stopper ingenting.

| Regel | Alvor | Testen den speiler |
|---|---|---|
| Nøyaktig én `start`, `up`, `rest` og `down` (nivå 10 ingen `down`) | feil | `kryptnivaaene_har_det_de_skal_og_taket_er_hoyt_nok` |
| Taket ≥ `HEADROOM` over alt gangbart, også over trapper, under gallerier og broer | feil | samme |
| Bitene overlapper i volum | feil | ny: `bitene_overlapper_ikke` |
| To sider som ikke passer, åpning mot fjell («dør som ikke går noen vei»), trapp som ender i vegg eller lufta | feil | ny: `kantene_passer` |
| Tett: hver side mot fjell har vegg, og hver underside mot luft har underside | feil | `leak-scan` per bit + ny: `nivaaet_er_tett` |
| Høyden: gulv og tak innenfor −12 til +14 m fra origo (så `level_at` med −8/+4 m slakk aldri blander to nivåer), og nivå 1 under `TOP` (−20 m) og kirkegården | feil | ny: `nivaaene_ligger_i_hver_sin_hoyde` |
| Bredden: innenfor ±185 m (`REACH`) fra ankeret | feil | samme |
| Alt kan nås fra start (løseren under), utenom det hemmelige, som må ha en vei ut | feil | `kryptnivaaene_kan_gas_gjennom`, `kapselen_gar_mellom_noekkelstedene_i_hvert_nivaa` |
| Spak uten mål, port uten spak, nøkkel eller arena, kobling til noe som er slettet | feil | ny: `koblingene_henger_sammen` |
| Nøkkel eller spak bak sin egen port, soft-lock, `down` som ikke kan nås | feil | ny: `loseren_finner_ingen_laas` |
| Boss uten arena, arena med en inngang uten arenaport, arena uten `exit` | feil | ny: `arenaene_er_lukket` |
| Hver fiende står på nettet og har vei til start (alle porter åpne), og patruljepunktene kan nås | feil (eller «med vilje») | `fiendene_finner_fram_i_krypten` |
| Pynt eller innhold som sperrer den eneste veien | feil | samme som nåbarhet |
| Ingen usynlige vegger: kolliderne følger formen | feil | `ingen_usynlige_vegger_i_krypten` |
| Merket står på gulvet i et rom | feil | `edit::crypt::breaches` |
| Åpningene minst 1,2 m brede og 2,2 m høye; trapper ikke brattere enn kapselen tåler | feil | `kapselen_proever_en_strekning` |
| Lys lekker gjennom mur: lysets rekkevidde når celler i eget eller tilkoblet rom uten fri sikt (sikt regnet over cellene). Forslag: kortere rekkevidde, eller del rommet | advarsel | `lyset_folger_rommene` |
| Høyst 64 rom (`MAX_ROOMS`, `light_mask` er u64), og høyst rundt 40 lys innen synsvidde (`MAX_LIGHTS` 48) | feil / advarsel | ny |
| Budsjettet: synlige trekanter (anslag), ulike modeller, instanser, kollidere, polygoner og fiender | advarsel | `snapshot --stats` |
| Biter som ikke er godkjent | advarsel (feil for «klar») | `approvals` |
| Tom lapp, ukjent komponent i kista, sjelden komponent to ganger | feil | ny |
| Vann som stenger hovedveien uten at slusa kan nås først | feil | løseren |

**Løseren** er ny i kjernen. Den lager en rettet graf av rommene og nettet:

- Vanlige forbindelser går begge veier. Fall, fallemmer og åpne kanter går bare én vei. En snarvei er stengt fra den ene siden til den er åpnet. En heis krever spaken. Høyt vann stenger til slusa er dreid.
- Løseren starter ved `start` med alle brytere stengt. Den tar det den når (spaker, eliter med nøkler, sluser), åpner og fortsetter til ingenting nytt skjer.
- Til slutt sjekker den at `down` (eller bossen på nivå 10) er nådd, at du fra hvert sted du kan havne kommer til `rest` eller `down` (ingen grop uten vei opp), og den lager rekkefølgen spakene trekkes i. Den rekkefølgen bruker testen som går gjennom nivået med Rapier.

## 11. Grensetilfeller

**Rotasjon og speiling:**
- Innhold festet til en bit følger biten: stedet transformeres, dreiningen blir −yaw ved speiling, og venstre og høyre byttes for snarveiens slå, hengslene og stigens side.
- Biter med tekst eller annen asymmetri har en egen speilet modell, ellers nektes speiling med grunnen.
- Speilede instanser snur omløpet. Rendereren tegner begge sider (`cull_mode: None`), men normalene og tangentene må få riktig fortegn når determinanten er negativ. Testen `speilet_bit_ser_lik_ut` sammenligner et bilde av en speilet bit med den speilede modellen.
- Nettets polygoner snus så de går mot klokka, som `Nav` krever.
- Fire dreininger gir det samme som ingen dreining, og to speilinger det samme som ingen (testet per bit).

**Overlapp og biter i lufta:** volumene sjekkes (5.6). En bit i lufta finnes bare over en luftcelle, og da kreves en underside (tetthetsregelen).

**Kanten av kartet:**
- En dør mot en celle som ikke finnes, er en feil.
- Utenfor ±185 m er en feil.
- Den røde rammen flytter seg når `start` flyttes, fordi origo følger `start`.

**Origo og `LEVEL_DROP`:** høydebåndet −12 til +14 m (tabellen i del 10). Høyere sammensatte biter (tårn) må passe innenfor.

**Taket:** sjekkes per polygon i nettet med det laveste over (hvelvets kurve tas fra bitens eget mål for frihøyde).

**Lys som lekker:** rommene skiller lyset. Inne i ett rom finner sjekken lys som når rundt en blokk (del 10). Gallerier som overlapper, deler lys med vilje.

**Kapselen setter seg fast:**
- Gulvkolliderne flettes, så det ikke blir noen sømmer.
- Kvarthjørnene går ikke inn i åpningene; åpningene måles mot standardmålene.
- Trappene har en rampe som kollider.
- Testene `kapselen_*` går mellom alle viktige steder, og `ingen_usynlige_vegger_i_krypten` skyter strålene som i dag.

**Fiender som ikke når deg:** regelen over, unntatt «med vilje». Fall går bare én vei, også for fiendene, som ikke hopper.

**Nøkkel bak egen port og soft-lock:** løseren.

**Arenaporter:** alle veier inn til arenaen går gjennom en arenaport. Arenaen har en `exit`. Bossen står inne i arenaen.

**Flere spaker per port:** `requires` er `én` eller `alle`, og vises på porten («2 av 2»).

**Angre og gjenta:**
- Hver endring er en `Change` med det motsatte (som `edit/editor.rs`): biter, kanter, rom, innhold, koblinger, egenskaper, varianter, maler, liming og løft/senk. Et drag eller en mal er ett steg.
- Historikken står som en liste (`edit/history.rs`).
- En prøvetur endrer ikke dokumentet, så angre går gjennom den.
- Testen gjør tilfeldige rekker av handlinger, angrer alt og krever at dokumentet er som før.

**Slettet innhold:** innhold som er festet til en bit som slettes, slettes med den i samme steg. Koblinger til noe slettet tas bort i samme steg og står i angreteksten.

**Lim inn:** id-ene lages nye. Koblinger inne i utvalget følger med, koblinger ut slippes.

**Store nivåer:**
- Kompilatoren regner bare det som er endret.
- Kartet tegner bare det som synes.
- 3D-vinduet har et *snitt* som skjuler alt over det aktive båndet (`Frame.hidden`), så du ser ned i etasjen.
- Budsjettmåleren står i panelet.

**Lagring:** se del 13. **Formatet og byggesettet:** se del 13.

**Liten skjerm og mobil:** ikke støttet. Siden sier «Bruk en skjerm som er minst 1100 px bred». Spillet er ikke berørt.

**Spill fra start uten start:** knappen er grå med grunnen. Med feil kan du spille likevel, men lista blinker.

**Flyttall:** wasm og maskinkode avrunder til mm, så resultatet er likt.

**Hemmelige rom:** skjult på kartet til du har vært der. Polygoner uten vei ut regnes ikke som feil når de er hemmelige og har en vei ut (som `build_level.py` i dag).

## 12. Skjermbildet

**Siden:** `?nivaa` på forhåndsvisningen (bare for eieren, som `?edit`). Fristil-komponenter.

```
┌ verktøylinje ─────────────────────────────────────────────────────────────┐
│ Nivå ▾  Utkast/Klar  Angre Gjenta  Velg Bygg Kant Koble Rom Pynt Lys       │
│ Etasje ▾  Snitt  Lys: spill/arbeid/rom   ▶ Spill fra start  Lagre (n)      │
├ palett ────┬ kart ovenfra (2D) ──────────────┬ 3D ──────────────┬ panel ──┤
│ søk        │ rutenett, celler, kanter,        │ orbit/fly, snitt │ egenskaper│
│ Celler     │ innhold, koblinger, patruljer,   │ håndtak, fantom  │ rom/tema  │
│ Sammensatt │ grensen ±185, etasjer svakt      │                  │ koblinger │
│ Innhold    │                                  │                  │ regler    │
│ Pynt       │                       minikart ▢ │                  │ historikk │
│ Maler      │                                  │                  │ budsjett  │
└────────────┴──────────────────────────────────┴──────────────────┴──────────┘
```

- **Verktøylinja:** valg av nivå, utkast eller klar, angre og gjenta, verktøyene (Velg, Bygg, Kant, Koble, Rom, Pynt, Lys), etasjen, snittet, lysvisningen, **Spill fra start** (Shift: spill herfra) og Lagre med antall endringer.
- **Paletten:** **modulene** med bildet (`tools/crypt/module_shots.py`), dommen og temaet, gruppert som rom, ganger, trapper og ferdige rom, med søk og mest brukte øverst. Dra ut på kartet eller i 3D. R dreier, X speiler, og fantomet er grønt eller rødt med grunnen; åpningene vises som piler, og der fantomet møter en annen modul, vises åpningen som blir. Bitene ligger i en avansert skuff som er lukket.
- **Åpningene:** klikk på en åpning (eller en kant mellom to moduler) og velg dør, port, falsk vegg, gang, bue, snarvei eller mur (`overrides`).
- **Kartet:** cellene tegnet med tema og rom, kantene med symbol (dør, port, bue, vindu …), innholdet som ikoner, koblingene som buer, patruljene som streker og bakhold som sirkler. Musehjulet zoomer, mellomknappen eller mellomrom+dra flytter, og rammen velger. Andre etasjer står svakt. Det røde er konflikter og grensen.
- **Etasjene:** liste med navn og høyde, øye (vis/skjul) og lås. PageUp og PageDown bytter.
- **Panelet:** egenskapene til det valgte (bit, variant med lås, rom, tema, stemning, innholdet), koblingene som lenker, reglene, historikken og budsjettet.
- **Koblingsmodus (K):** pekeren drar en bue fra kilden. Gyldige mål lyser, og ugyldige er grå med grunnen.
- **Valideringslista:** gruppert som feil, advarsel og info. «Gå til» flytter kartet, sentrerer 3D og velger. «Rett» der det går: lukk døra, legg til rekkverk, flytt fienden til nettet, lag en spak til porten, del rommet.
- **Minikartet:** hele nivået med alle etasjene, 3D-kameraets synsfelt tegnet inn, og klikk for å gå dit.
- **3D-kameraet:**
  - orbit rundt det valgte (Alt+dra eller mellomknappen),
  - flyging (høyre knapp + WASD, mellomrom og C, Shift fort, som i dag),
  - F setter fokus på det valgte,
  - klikk i 3D velger og viser det samme i kartet.
- **Spill fra start:** spillet bygges fra det kompilerte nivået i sin plass (`Engine::play` og `prepare_with` som P i dag). Du starter ved `start` med fiender, mekanismer og lykt.
  - Med en «prøvespiller» (utstyr som ventet for nivået, og valgfritt «tål alt») kan nivået prøves uten å ha spilt fram til det.
  - **Esc** går rett tilbake til redigeringen med samme kamera, etasje og valg. Pausemenyen er slått av under prøveturen.
  - Dør du, kan du våkne ved hvilestedet eller gå tilbake.

**Tastene:**

| Tast | Gjør |
|---|---|
| 1–7 | verktøyene (Velg, Bygg, Kant, Koble, Rom, Pynt, Lys) |
| R / Shift+R | dreie 90° med og mot klokka |
| X / Shift+X | speile langs x eller z |
| Ctrl+Z / Ctrl+Shift+Z | angre og gjøre om |
| Ctrl+C / Ctrl+V / Ctrl+D | kopiere, lime inn, kopiere ved siden av |
| Delete | slette |
| Ctrl+G | lagre utvalget som modul |
| PageUp / PageDown | etasjen opp og ned |
| + / − | løfte og senke gulvet 1 m |
| K | koble |
| V | kaste variantene på nytt |
| L | låse varianten |
| H | skjule laget |
| S | snittet av og på |
| T | lysvisningen |
| F | fokus på det valgte |
| Tab / Shift+Tab | blar i det som ligger under pekeren (som i dag) |
| Ctrl+1–9 / 1–9 | bokmerker for kameraet |
| P | spill fra start (Shift+P herfra) |
| Esc | tilbake fra prøveturen, eller avbryt det du holder på med |
| Ctrl+S | lagre |

## 13. Lagring, utkast og versjoner

### 13.1 Utkast

- Utkastet ligger i nettleseren (IndexedDB, én post per nivå). Det har med seg formatversjonen, versjonen av byggesettet og fila det bygger på (blob-sha og innhold), og skrives ved hver endring (etter 2 s) og hvert 30. sekund.
- Når siden åpnes, velger du mellom utkastet og det som ligger på grenen. Begge vises med tid.

### 13.2 Lagre

- Lagre bruker `web/github.js` (`commit`, én commit per økt, uendret) og tar bare med nivåene og malene som er endret.
- **Før commit** sammenlignes fila på grenen med den utkastet bygger på. Er den endret av noen andre (en annen fane, byredigeringen eller Claude), kommer et valg:
  - behold min,
  - ta deres,
  - last ned begge som zip.
- **Uten nøkkel** blir det zip (`makeZip`).
- **Feil:**
  - Nettfeil: utkastet står, og det prøves igjen.
  - 401: lim inn nøkkelen på nytt.
  - Konflikt på ref: hent på nytt og prøv én gang, ellers valget over.
- Fila holdes under 1 MB, som er grensen i Contents-API-et.
- Når Claude har slått inn og slettet `redigering`, begynner neste lagring fra `claude/detaljnivaa`. Er innholdet likt det som ble slått inn, er det ingen konflikt.

### 13.3 Formatet

`format` i fila og migreringer i kjernen fra hver eldre versjon. Testen leser en fil fra hver versjon som ligger i `assets/levels/fixtures/`.

### 13.4 Byggesettet over tid

- **Rent utseende:** en bit som får nytt utseende, får `v+1` med samme id. Alle nivåene blir finere uten å bygges på nytt. Det er den store fordelen med instanser.
- **Ny form:** endres flatene eller fotavtrykket, får biten ny id (`grav/romfelt2`). Den gamle merkes «utgår» og står til ingen nivåer bruker den.
- **Bytt bit:** verktøyet «Bytt bit» bytter alle forekomster med en tabell for dreiningen.
- **Nye varianter** blandes bare inn i nivåer som slår dem på. Låste og regnede varianter flytter seg ikke, fordi den valgte varianten skrives i fila første gang nivået merkes klart.

## 14. Ytelse

**Målet:** 60 bilder i sekundet på eierens Mac mini M2 i nettleseren, under rundt 500 000 trekanter og under 300 tegnekall (CLAUDE.md).

**Tegnekall:**
- Rendereren tegner hver modell med instanser, ett kall per modell og nær/fjern-versjon (`draw_all`).
- Om 40 ulike biter synes, blir det rundt 80 kall i hovedpasset, pluss pynt og fiender.
- **I spillet** slås instansene sammen ved lasting: per bit på 16 m og per materiallag, i adapteren eller rendereren og ikke i kjernen. De blir verdensbiter som i dag (`chunks`, sammenhengende områder blir ett kall). Det er mulig fordi alle kryptbitene bruker de samme teksturlagene.
- **I redigeringen** brukes instansene direkte, så endringer synes med en gang.
- Fase 7 måler begge og bestemmer.

**Trekanter:** et anslag på rundt 2 500 per celle med vegg, hjørner, gulv og hvelv. 100 synlige celler gir rundt 250 000. Fjernversjoner for alle bitene.

**Hva som tegnes:** rommene brukes også til utvalget. Bare rommene du ser inn i (forbindelsene fra rommet du står i, to ledd ut) tegnes. Det er planlagt i `docs/krypten-ny.md` («portaler») og blir naturlig nå som rommene er celler.

**Fysikken:** kolliderne flettes (del 7), under rundt 15 000 per nivå.

**Minnet:** bitene hentes per tema og deles mellom nivåene (to nivåer per tema), så nedlastingen blir mindre enn med dagens bit per nivå.

**Målingene:** `snapshot --level-doc N --stats` og `?bench` i nettleseren. Grunnlinjen skrives i `docs/ny-stil.md`, og budsjettestene i CI stopper et klart nivå som går over.

## 15. Hva som gjenbrukes, og hva som er nytt

| Del | Gjenbrukes | Nytt |
|---|---|---|
| `web/github.js` | hele (`commit`, `branchFiles`, nøkkelen) | sjekken mot blob-sha før commit (del 13.2) |
| `web/editor.js` | `makeZip`, utkast og autolagring, lagringsdialogen, katalogkortene med dom, egenskapspanelet, historikklista, bokmerkene og tastemønsteret. Flyttes til `web/edit-common.js`, så begge sidene bruker det | `web/nivaa.js` (siden), `web/nivaa-plan.js` (kartet i Canvas 2D), `web/nivaa-palette.js` |
| `domain/edit/` | mønsteret med `Change` og angre (`editor.rs`), `history.rs`, `props.rs` (`Prop`), `ghost.rs` (fantomet med reglene), `select.rs`, `group.rs`, `rules::Breach` (får alvor og sted), `crypt.rs` (`model_of`, `default_box`, `breaches`) | `domain/edit/level/`: `LevelEditor` over `LevelDoc` (celler, kanter, rom, innhold, koblinger, pynt, maler, liming, speiling, gjentak) |
| `application/editing.rs` | `screen_ray`, håndtakene, drag, flyging, `edit_lines`, `edit_instances` (`EDIT_RESERVE`), `prepare_with` / P | `application/level_editing.rs`: modusen, utsnittet til kartet, snittet og prøveturen med Esc tilbake |
| Portene | `Command::Edit` og `Event::Edit`, `LayoutStore` | `EditCommand::Level(LevelCommand)`, `Event::LevelEdit(LevelView)` (kartbildet per etasje som endringer), `ModelSource` gir `KitDef` og `LevelDoc` |
| Krypten i kjernen | `LevelFile`, `Crypt::build`, `nav.rs`, `map.rs`, `run.rs`, mekanismene, `light_mask`, `query.rs` | `compose/`, `check/` (reglene og løseren), `doc.rs` (format og migreringer), `kit.rs`; feltet `instances` i `LevelFile`; `requires` (én eller alle) i mekanismene; tåke og lyd per rom i `Frame` |
| Python og Blender | funksjonene i `kit_grav.py`, `solids.py --models`, `export_model` med okklusjon, tanken i `leak_check.py`, `tools/qa` (bilder, godkjenning, «Se i 3D») | `kit_common.py` (standardmålene, flatene, manifestet), `kit_<tema>.py` per tema, prøvebrettet |
| Slettes når alle ti er nye | — | `build_level.py`, `layouts/`, `levels/`, `plan.py`, `crypt_NN_*.glb`, `crypt_NN_secret_*.glb`, `assets/layout/crypt_NN.json` (innholdet bor i `nivaa_NN.json`) |

## 16. Fasene

Hver fase slutter med push til forhåndsvisningen og ett godkjenningsark til eieren.

**Fase 0: Planen og en måling (liten)**
- Denne planen godkjennes, med svarene på de åpne spørsmålene (del 18).
- En rask måling: 400 instanser av grå biter laget med `kit_grav.py`-funksjonene, tegnet med instanser og sammenslått, kall og ms på M2 (`?bench`).
- **Eieren ser:** planen, en skisse av skjermbildet og tallene fra målingen.

**Fase 1: Kompilatoren med et grått pilotsett (stor)**
- `LevelDoc`, `KitDef`, kantene og hjørnene, instansene, kolliderne med fletting, nettet, rommene, merkene og `instances` i `Crypt::build`.
- Rundt 12 grå biter (dagens stein, uten pynt) og et prøvenivå på 8 × 8 celler som Claude skriver for hånd i json.
- Prøvebrettet: hver bit mot hver bit i alle dreininger.
- **Tester:**
  - `nivaadokumentet_kompileres_likt` (to ganger og bare det endrede mot alt på nytt),
  - `bitene_dreies_og_speiles_riktig`,
  - `kantene_passer`,
  - `nivaaet_er_tett`,
  - `bitene_overlapper_ikke`,
  - prøvenivået gjennom de gamle testene (`kryptnivaaene_kan_gas_gjennom`, `ingen_usynlige_vegger_i_krypten`, `fiendene_finner_fram_i_krypten`, `lyset_folger_rommene`).
- **Eieren ser:** skjermbilder og prøvenivået spillbart på forhåndsvisningen (`snapshot --level-doc test`).

**Fase 2: Pilotsettet for gravkjelleren i malt stil (middels)**
- Rundt 20 biter (4.4), med okklusjon bakt i sammenheng, nær- og fjernversjon og pyntefester, satt sammen til **moduler** (del 21).
- **Eieren ser:** modulene på godkjenningssiden, hver ferdig satt sammen innenfra med fakler og lys, med et kart over modulen og åpningene, og «Se i 3D»; og kartet over prøvenivået bygd av moduler. Eieren godkjenner før neste fase.

**Fase 3: Siden, ett tema, ett lite nivå (stor)**
- `?nivaa` med kartet (én etasje), **modulpaletten** (del 21), Bygg, Åpning og Velg, dreiing og speiling, 3D ved siden av, angre og gjenta, reglene med «Gå til», utkastet, Lagre og zip, og **Spill fra start** med Esc tilbake.
- Bare halvtrapp og halv etasje i høyden.
- **Tester:**
  - `nivaaredigeringen_bygger_og_angrer` (Rust),
  - `tools/level-edit-test.mjs`: sett ut biter, se 3D, spill, Esc, last inn utkastet på nytt, lagre zip.
- **Eieren ser og gjør:** bygger et lite testnivå selv (5–8 rom) og spiller det.

**Fase 4: Mekanismene, koblingene og løseren (stor)**
- Koblingsmodus, porter, spaker (én eller alle), nøkkel og elite, arena med `exit`, snarvei, falsk vegg med hint, feller, fallem og stige.
- Fiender med patrulje, bakhold og «med vilje». Kister, lapper og bonuser.
- Løseren og «Rett».
- **Tester:**
  - `loseren_finner_ingen_laas` med nivåer som har feil med vilje (nøkkel bak egen port, spak bak sin port, en grop uten vei opp, en arena med bakdør),
  - `koblingene_henger_sammen`, `arenaene_er_lukket`,
  - `gjennomspilling` på testnivået med rekkefølgen fra løseren.
- **Eieren ser:** testnivået med en hel gåte (spak, port, hemmelighet, elite med nøkkel).

**Fase 5: Variasjon og stemning (middels)**
- Varianter med frø og lås, forfallspensel, pyntelag og pyntesett, fri pynt, lysfester, stemninger, tåke og lyd per rom, lysvisningen og lyslekk-sjekken.
- Maler, kopier, lim inn, speil, gjenta, løft og senk, minikartet.
- **Tester:**
  - `variantene_er_faste` (samme fil gir samme utseende; nye varianter flytter ingenting),
  - `malene_settes_ut_med_koblingene`,
  - `lyset_lekker_ikke` (advarselen finner lekken i et laget eksempel).
- **Eieren ser:** to like rom ved siden av hverandre som ser forskjellige ut, og en mal brukt tre steder.

**Fase 6: Full vertikalitet (stor)**
- Etasjene og snittet, dobbel høyde, gallerier og balkonger, hel etasje og vindeltrapp, sjakt med stige og heis, kløft med bro (fast og senket), basseng med sluse, rotunde og tårn, vindu og gitter.
- **Tester:**
  - `nivaaene_ligger_i_hver_sin_hoyde`,
  - taket under gallerier og broer,
  - heisen og broa i `gjennomspilling`,
  - `kapselen_*` over alle trappene.
- **Eieren ser:** en sal med galleri, en bro over en kløft og et fall ned til et hemmelig rom.

**Fase 7: Ytelsen (middels)**
- Sammenslåing ved lasting, fjernversjonene, utvalg etter rom og fletting av kolliderne. Budsjettestene i CI.
- **Eieren ser:** `?bench`-tall fra M2 på det største testnivået. Målet er 60 bilder i sekundet.

**Fase 8: De fire andre temaene (stor, ett tema om gangen)**
- Kloakken (med vann), gruva og grotta (ujevne grotter, stiver, heis), borgen (søylesaler, brystvern) og kjernen (plast og sømmer).
- Hvert tema går gjennom prøvebrettet og godkjenningssiden, og så overgangene.
- Teksturlagene for hvert tema (plan del 12, steg 5; `LAYERS` mot 64).

**Fase 9: Eieren bygger nivå 1–10**
- Claude slår inn og kjører testene. Hvert nivå som merkes klart, erstatter det gamle.
- Når alle ti står, slettes den gamle kjeden (del 15).
- Nye biter kommer når eieren savner noe.

## 17. Testene samlet

**Kjernen** (`crates/fristad-domain`, raske):
- formatet og migreringene,
- dreining og speiling per bit,
- kanttabellen (begge veier og alle par),
- tetthet og overlapp,
- like resultat (fullt, bare det endrede, wasm og maskinkode),
- løseren mot laget feil,
- angre og gjenta med tilfeldige rekker,
- maler og liming,
- variantene er faste,
- lyslekken.

**Spillet** (`src/tests.rs`, `src/crypt_tests.rs`, med Rapier) på alle klare nivåer og prøvenivået:
- `kryptnivaaene_kan_gas_gjennom`,
- `ingen_usynlige_vegger_i_krypten`,
- `kapselen_gar_mellom_noekkelstedene_i_hvert_nivaa`,
- `fiendene_finner_fram_i_krypten`,
- `fiendene_patruljerer_og_ligger_i_bakhold`,
- `lyset_folger_rommene`,
- `kryptnivaaene_har_det_de_skal_og_taket_er_hoyt_nok`,
- `gjennomspilling`,
- `kryptnivaaene_hentes_nivaa_for_nivaa` (nå per tema),
- nye: `nivaaene_bygges_likt_i_redigeringen_og_spillet` og `reglene_og_testene_er_enige` (feilnivåene gir samme svar i begge).

**Byggesettet** (pytest og eksporten):
- hver bit er tett på de lukkede flatene,
- uv i meter og på 2 m-rutenettet,
- okklusjonen nøytral mot kantene,
- kolliderne i `extras.solids`,
- manifestet likt glb-filene,
- budsjettet for trekanter,
- fjernversjonen finnes.

**Nettleseren:**
- `tools/level-edit-test.mjs`: bygg, koble, spill, Esc, utkast, konflikt (falsk GitHub), zip og under 1100 px,
- `tools/browser-test.mjs` med `?webgl`.

**Ytelsen:** `snapshot --level-doc N --stats` mot budsjettet.

## 18. Eierens svar (18. oktober)

1. **Murene** velges automatisk, med overstyring per kant (dør, port, bue, vindu, gitter, falsk vegg, snarvei, rekkverk, åpen).
2. **Lagring:** Lagre legger alltid utkastet på grenen `redigering`; et nivå kommer først inn i spillet når det merkes «klar» og alle reglene er grønne.
3. **Prøveturen:** udødelig (ingen skade, full olje), så du ser på nivået og går gjennom det. Fiendene er der og angriper, men du tar ingen skade.
4. **Stemningene:** begynn med det temaene har i dag (to–tre lysstemninger og romlydene per tema); flere når eieren savner dem.
5. **Skjermen:** kartet til venstre og 3D til høyre, og en tast eller knapp som viser bare den ene over hele skjermen.

## 19. Status: fase 0 og 1 (19. oktober)

**Gjort:**
- **Kjernen** (`crates/fristad-domain/src/domain/world/crypt/`): `doc.rs` (`LevelDoc`), `kit.rs` (`KitDef`), og kompilatoren i `compose/` (`mod.rs`: kantene, hjørnene og instansene; `grid.rs`: standardmålene; `solids.rs`: kolliderne flettet; `nav.rs`: nettet rastret og slått sammen til rektangler; `rooms.rs`: omrisset av cellene; `check.rs`: reglene `tight`, `overlaps`, `edges`; `board.rs`: prøvebrettet). `LevelFile` har fått feltet `instances`, som `Crypt::build` setter ut. `Data.docs` og `Data::compose_levels` lar et nivådokument merket «klar» erstatte det gamle nivået med samme nummer; ingen er klare ennå, så de ti nivåene er uendret.
- **Det grå pilotsettet** (`tools/crypt/kit_cells.py`, målene i `kit_common.py`): 14 modeller og 17 biter i manifestet `assets/models/crypt/kit/grav.json` (romfelt, gang, gang_blind, gang_hjorne, gang_t, gang_kryss, halvtrapp, trapp2, nedgang; halvkantene vegg, vegg_gang, vegg_dor, vegg_port, vegg_falsk, gjordbue; hjorne og soyle), med dagens stein (`crypt_wall`, `crypt_floor`) uten pynt og uten bakt okklusjon.
- **Prøvenivået** `assets/levels/nivaa_test.json`, skrevet for hånd: 31 celler på et rutenett på 8 × 7 (kapellet 3 × 3 med søyler og døra opp, et hemmelig rom bak en falsk vegg med sprekk, gangen med hjørne og halvtrapp opp til hvelvet 3 × 2 med hvilestedet og spaken, porten til forhallen med trappa ned, en dør til sidegangen med kryss og blindgang, og beinhuset med to fiender på patrulje). Det gir 242 instanser, 59 kollidere (flettet), 92 polygoner og 7 rom.
- **Prøvebrettet** (`compose::board`, `snapshot --level-doc brett`): hver av de 9 cellebitene mot hver av dem med alle 4 × 4 dreininger, 1 296 par.
- **Testene** (`src/level_doc_tests.rs`): `nivaadokumentet_kompileres_likt` (to ganger, og bare det endrede mot alt på nytt), `bitene_dreies_og_speiles_riktig`, `kantene_passer` (tabellen i del 5.1 over hele prøvebrettet, og en dør mot fjellet), `nivaaet_er_tett`, `bitene_overlapper_ikke`, `byggesettet_har_bitene`, `provenivaaet_bygges_av_bitene`; og prøvenivået gjennom de gamle testene: `provenivaaet_kan_gas_gjennom` (boten med ekte fysikk: spaken, porten, trappa ned), `ingen_usynlige_vegger_i_provenivaaet` (ingen treff), `fiendene_finner_fram_i_provenivaaet`, `lyset_folger_rommene_i_provenivaaet` og `provenivaaet_har_det_det_skal_og_taket_er_hoyt_nok`.
- **Hull:** `snapshot --leak-scan` fra hver celle i prøvenivået og 73 tilfeldige par på brettet: ingen. To ting rettet underveis: kvarte dreininger var ikke eksakte (sprekker på én piksel langs gulvskjøtene), og halvkantene og hjørnene fikk bakside og ender, så skjøtene har noe bak seg.
- **Skissen** av skjermbildet: `docs/nivaaredigering-skisse.html` (kartet er tegnet av prøvenivået, 3D-vinduet er et bilde av det).

**Valg og avvik fra planen:**
- Cellas midte står i (4i, y, 4j); sidene og ankrene (`kant:<celle>/<side>`, `edges`) er i cellens eget rom; `yaw` i dokumentet er grader.
- Det gangbare kommer fra manifestet (`walk`: rektangler, skrå for trappene), ikke fra `extras.kit` i glb. Manifestet skrives av Blender-skriptet ved siden av modellene.
- Nettet rastres i ruter på 0,2 m (ikke 0,25), så murflaten (1,6), nettets kant (1,2), døra (±0,4) og porten (±1,0) går opp i rutene.
- Kvarthjørnet settes der en mur (en halvkant som ikke er åpen) når hjørnet i en av cellene rundt, eller en av rutene rundt er fjell; søyla der fire romfelt møtes, åpne mot hverandre. Det er færre tilfeller enn «marching squares» i del 5.1, men det gir det samme.
- Trappene har et skrått tønnehvelv som følger trinnene, så begge endene har gangprofilen og ingen overmur trengs mot en gang.
- Porten står i kantlinja og vender inn i cella den er festet til; den falske veggen står i murflaten på den siden (og synker som før). Testen `lyset_folger_rommene` tåler nå at porten står på grensen mellom to rom.
- «Bare det endrede» er et minne per celle med nabolaget på 5 × 5 som nøkkel; kolliderne, nettet og rommene settes sammen på nytt hver gang (det er raskt).
- Et rom med hull deles ikke ennå (det gir en advarsel).

**Målingen (fase 0)**, `snapshot --level-doc test --stats` i skyen (lavapipe, ikke M2; ms gir bare mening på M2). Instansene er det som tegnes i hovedpasset (også armene og fiendene); krypten har ingen solskygge.

| Sted | Instanser | Kall | Trekanter | Slått sammen per 16 m: kall | trekanter |
|---|---|---|---|---|---|
| Kapellet | 130 | 13 | 20 152 | 9 | 23 892 |
| Hvelvet | 84 | 16 | 25 336 | 8 | 23 117 |
| Gangen | 236 | 24 | 38 151 | 17 | 36 626 |
| Prøvebrettet (langs en rad) | 2 831 | 46 | 155 946 | 77 (158 ruter) | 153 338 |

Med instanser blir det ett kall per modell som synes, uansett hvor mange instanser; med få ulike biter (14) er det allerede få kall. Sammenslått gir færre kall i ett rom, men flere når mange ruter synes. Eieren måler ms på M2 med `?nivaa=test&bench` og `?nivaa=test&slaa&bench` på forhåndsvisningen når Actions har bygd.

**Gjenstår til fase 2:** bitene i malt stil med okklusjon bakt i sammenheng, nær- og fjernversjon og pyntefester; `extras.kit` i glb og testen «manifestet likt glb-filene»; lysfester (lysene står i dokumentet ennå); å dele rom med hull; trinnmur, rekkverk og ulike tak over en kant (fase 6); tetthetssjekken per bit i Blender.


## 20. Status: fase 2 (19. oktober)

**Gjort:**
- **Pilotsettet for gravkjelleren i malt stil** (`tools/crypt/kit_cells.py`, eksporten i `kit_bake.py`): 24 modeller og 22 biter i `assets/models/crypt/kit/grav.json`, med lagene fra nivå 1 (`crypt_wall` som mur, hogd stein og hvelv, `crypt_floor`, `grave_slab`, `ossuary`, `plaster_old` som mørtel) og detaljene derfra: sokkel langs foten, karm av hogde steiner rundt døra, karm rundt porten, buestav rundt gangåpningen, ribber og gjordbuer, gravnisjer.
  - Celler: romfelt (a, og b med gravheller i gulvet og ribber i hvelvet), gang (a, og b med gjordbue på konsoller og gravheller), gangkryss, halvtrapp, halv etasje, nedgang, og **rotunden 3 × 3** som sammensatt bit (kuppel med ribber og sluttstein, gesims, fire nisjer med fakkelfester, en gang ut midt på hver side).
  - Halvkanter: mur (a, og b med to gravnisjer, en åpen med knokler og en lukket med helle), overmur over gang, dør, port, falsk vegg, gjordbue, overmur over et lavere hvelv og rekkverk (de to siste tas i bruk i fase 6).
  - Kvarthjørner: innvendig (og der muren går videre), utvendig (avfaset over sokkelen), pilaster under gjordbuen; søyla; fakkelen; den falske veggen som synker (`grav_falsk`, samme mur og sokkel som rundt, sprekk i fugene).
- **Okklusjonen bakt i sammenheng:** hver bit bakes blant naboene den vanligvis har (`kit_bake.context`), og går over i det nøytrale innen 0,35 m fra skjøtene (`SEAMS`): flater opp og ned får 1, loddrette flater mørket en rett mur har i samme høyde (`wall_profile`, bakt først). Skjøtene har derfor samme mørke på begge sider.
- **Nær- og fjernversjon:** noden `far` i hver glb (uten sokkel, karmer, ribber, nisjer; grovere hvelv med de samme hjørnene langs kantene), som `gltf.rs` legger i det grove detaljnivået i stedet for å regne det ut.
- **Festene:** `mounts` med `fakkel` (midt på muren, og over nisjene i rotunden), `pynt_gulv` og `pynt_vegg` (fylles i fase 5). Kompilatoren setter fakler i festene etter rommets `mood.torches` (0–1, med frøet), hver med et lys ved flammen i rommets eget rom; ikke på murene bak en sammensatt bit.
- **Variantene** (del 9): `variants` per bit med vekt, valgt med `compose::roll(frø, plass)` eller låst med `variant` i cella/kanten; en variant med en annen form har egne kollidere (nisjene).
- **Kvarthjørnet etter tilfellet** (`fits`): de to sidene cella har i hjørnet gir innvendig, mur som går videre, pilaster (mur mot gjordbue) eller utvendig.
- **Testene** (`src/level_doc_tests.rs`): `manifestet_er_likt_glb_filene` (`extras.kit` i hver glb er manifestet for bitene som bruker modellen), `bitene_har_fjernversjon_og_holder_budsjettet` (fjern ≤ nær, høyst 1 200 trekanter per celle for én bit), `variantene_er_faste`, `faklene_star_i_festene`, `hjornene_passer_til_murene`, `rotunden_settes_sammen`, og de fra fase 1. **Tetthetssjekken per bit** er `kit_cells.py --check` (Blender, stråler mot mur, hjørne, gulv, hvelv og kuppel; åpningene skal være åpne), og eksporten stopper på en bit som lekker.
- **Prøvenivået** er bygd om: beinhuset er rotunden (to fiender på patrulje rundt midten, gangen ut mot sidegangen), et gravkammer sør for den med kista, et L-formet forhall (utvendig hjørne), fakler i rommene i stedet for lysene midt i rommet. **Prøvebrettet** bruker de nye bitene.
- **Hull:** `snapshot --leak-scan` fra hver celle i prøvenivået (480 × 270 og 960 × 540) og 160 punkter på brettet: null. Rettet underveis: trinnene og kvarthjørnene hadde færre hjørner langs skjøten enn naboen (T-kryss, hull på én piksel), muren i rotunden fulgte ikke ringen i kuppelen, og gulvet i rotundens ganger møtte naboens gulv med andre hjørner.
- **Rettet etter bildene:** hjørnesteinene i det utvendige hjørnet var for urolige (nå en avfasing), pilasteren var en mørk strek (nå hogde steiner), gravhella i gulvet viste biter av flere heller (nå hele bildet), og rotundens gang lå oppå kvarthjørnene (z-kamp).
- **Andre runde (etter at arbeidet ble tatt opp igjen):** kuppelen i rotunden hadde uv skåret skjevt rundt kuppelen (u var buelengden på hver ring for seg, så bildet ble strukket til striper som så ut som tre); nå er v buelengden opp langs kuppelen og u buelengden rundt ved midten av hver ring, så steinene ligger i skift. Enkeltsteinene i murbildet (`kit_grav.STONE_PX`) var målt i det forrige bildet, så hver karmstein viste biter av flere steiner; de er målt på nytt. Pilasteren er ett skaft i forbandt med muren (enkeltsteinene speilet seg i kantlinja mellom de to halvdelene), med sida i kantlinja der naboen har en åpning. Gjordbuene, buestavene, ribbene og kuppelribbene legges som kilesteiner (`band(..., stones=)`, med `half` for gjordbuen som naboen speiler) i stedet for murbildet strukket langs buen. `grid_face` setter inn hjørnene der omrisset krysser en rutelinje i naboruta (T-krysset i gulvet i rotunden der gangen møter sirkelen slapp gjennom to piksler).
- **Budsjettet (del 14):** de vanlige cellene 204–720 trekanter (romfelt b med ribber og gravheller er størst), halvkantene 28–586 (døra med karm), hjørnene 80–144, rotunden 4 836 for ni celler (537 per celle); fjernversjonen er mindre for hver bit (`bitene_har_fjernversjon_og_holder_budsjettet`).

**Valg og avvik fra planen:**
- Okklusjonen på gulvet er nøytral langs alle kanter; mørket ved veggfoten ligger på muren og sokkelen, ikke i en kantsone på 0,5 m av gulvet (en egen gulvbit per veggkombinasjon ble for mange modeller nå).
- Festene og flatene står i manifestet og i `extras.kit`; kjernen leser manifestet (som i fase 1), og testen holder dem like.
- Rotunden er en sammensatt bit allerede nå (planen hadde den i fase 6): kompilatoren deler den i ni delceller med flatene `inner` innvendig.
- Overmuren over et lavere hvelv og rekkverket er laget, men kompilatoren bruker dem først i fase 6 (ulike tak og gulv).
- Nettet tar bort smale striper (under 0,4 m) etter at innholdet er skåret ut (`drop_slivers`).

**Gjenstår til fase 3:** eierens dom over bitene på godkjenningssiden (kortene og bildene er laget, `python3 -m tools.qa approvals review UT`, ikke publisert); `grav_gang_b`, `grav_trapp2`, `grav_falsk`, `grav_overmur` og `grav_rekkverk` står ikke i prøvenivået og har bare studiobildene; rotunden er mørk under kuppelen (bare faklene i nisjene lyser); siden `?nivaa` (kartet, paletten, Bygg/Kant/Velg, dreiing og speiling, 3D, angre, reglene, utkast, lagring og Spill fra start); pynt i festene og flere varianter (fase 5); ulike tak og gulv over en kant (fase 6).

---

### De viktigste filene

- `crates/fristad-domain/src/domain/world/crypt/level.rs`: `LevelFile` og `MarkDef`, kontrakten kompilatoren skal skrive til (nytt felt `instances`)
- `crates/fristad-domain/src/domain/world/crypt/mod.rs`: `Crypt::build` og `build_level` (origo, `LEVEL_DROP`, `REACH`, merkene, lysene, rommene)
- `crates/fristad-domain/src/domain/world/crypt/nav.rs`: `Nav::new` (sveising og kobling av kanter), som det sammensatte nettet må passe til
- `crates/fristad-domain/src/domain/edit/editor.rs` og `crates/fristad-domain/src/domain/edit/crypt.rs`: mønsteret for angre, `Change` og reglene for merkene, som `domain/edit/level/` bygger på
- `crates/fristad-app/src/application/editing.rs`: økta i redigeringen, `prepare_with`/P, stråler, håndtak og instanser, som skal få en egen nivåmodus
- `web/editor.js` og `web/github.js`: utkastet, zip og lagringen til grenen `redigering`, som flyttes til felles moduler for `web/nivaa.js`
- `tools/crypt/kit_grav.py`: geometrifunksjonene og eksporten (okklusjon, `extras.solids`), som `kit_common.py` og `kit_<tema>.py` bygger på
- `tools/crypt/build_level.py`: dagens regler for murer, nett, tetthet og hemmeligheter, som skal bli kjernekode og så slettes
- `src/adapters/wgpu/mod.rs` (`draw_all`, instanser og biter) og `src/adapters/delivery.rs` (lasting per tema)
- `src/tests.rs` og `src/crypt_tests.rs`: testene reglene skal speile


## 21. Moduler (eieren 19. oktober)

**Hvorfor:** eieren fikk de 24 bitene fra fase 2 på godkjenningssiden og skrev: «jeg er veldig usikker på alt det som ligger fra krypten. Jeg vil ha moduler som kan settes sammen. her er det bare masse rare løse deler? … mener du virkelig at det er slik jeg skal jobbe i editoren? masse små rare biter?» Eieren valgte moduler: ferdige rom og ganger han setter sammen som Lego. Bitene og cellene er bare det kompilatoren bruker inni.

**En modul** er en mal i `assets/levels/moduler/<tema>/<navn>.json` (`ModuleDef` i `crypt/doc.rs`, lest av `adapters/data.rs` til `KitDef.modules`):
- `id`, `name`, `theme`, `kind` (`rom`, `gang`, `trapp`, `ferdig`; paletten grupperer etter det) og `about`;
- `cells` i modulens eget rom (midtcella i (0, 0), id `c<kolonne><rad>`), `rooms` (det første er hovedrommet med stemningen; `secret` for skattkammeret);
- standardinnholdet (`things`, `links`, `lights`, ankret i modulens celler);
- **åpningene:** hver ytre side av en celle som ikke er lukket, er en åpning (`auto`), med id `i,j,S` (cella og siden i modulens rom). `walls` lukker en side (kapellet har mur der døra opp står), og `openings` gir en åpning et annet slag eller navn (`bue` i nedgangen, `falsk` i skattkammeret).

**I dokumentet** står modulene i `modules` (del 6). Før kompileringen utvides de (`compose::modules::flatten`): cellene får id-en `<modul>/<celle>` og dreies og speiles med modulen (samme `transform` som de sammensatte bitene), rommet heter som modulen (eller `room`, så flere moduler blir ett rom, som gangen i prøvenivået av fire moduler), og innholdet, koblingene og lysene følger med (`celle:` og `kant:` får modulens id foran; frie steder flyttes og dreies). Innhold i dokumentet kan ankres i en modul (`celle:hvelv/c20`) eller på en åpning (`aapning:<modul>/<åpning>`).

**Sammenføyningen** (der to moduler møtes, med gulvet i samme høyde):
| Modul A | Modul B | Kanten |
|---|---|---|
| åpning | åpning | åpning: eierens valg om det finnes (det sterkeste om begge har valgt), ellers port, falsk vegg, snarvei eller dør om en modul ber om det, gangåpning om en av sidene er en gang, bue om en av dem vil ha bue, ellers dør |
| åpning | ingen åpning (eller lukket side) | mur |
| flere åpninger langs samme møte | | bare den midterste er åpen (med mindre eieren har valgt flere) |
| åpning | løs celle (avansert) | cellereglene (del 5.1) |
| ingen åpning | løs celle | mur |
| hva som helst | fjell | mur (en åpning mot ingenting er bare lukket) |

Porten, snarveien og den falske veggen blir innhold med id-en eieren gir (`overrides[].id`) eller `<modul>/<åpning>`, så koblingene kan peke på dem. Den falske veggen står i muren på siden til modulen som *ikke* ba om den (skattkammeret ber om den, så veggen synes fra rommet ved siden av), med en sprekk koblet til den (`for`). Etter utvidelsen er dokumentet celler med kantvalg, så kompilatoren, reglene og testene er de samme (`"gang"` er et nytt kantvalg: gangåpning mellom to romfelt).

**Reglene:** to moduler i samme celle er en feil (`check::overlaps`, «modulene A og B overlapper»), en ukjent modul eller åpning er en feil, og en åpning på en lukket side i en modulfil er en feil.

**Modulsettet for gravkjelleren** (18, `tools/crypt/modules_grav.py` skrev dem; de kan endres for hånd):
- rom: `rom_1x1`, `rom_1x2`, `rom_2x2`, `rom_3x2`, `rom_3x3` (fire søyler), `rotunde`;
- ganger: `gang_1`, `gang_2`, `gang_hjorne`, `gang_t`, `gang_kryss`, `blindgang`;
- trapper: `halvtrapp` (1 m), `halv_etasje` (2 m), `nedgang` (trappa ned; bue mot rommet ved siden av);
- ferdige rom: `kapell` (3 × 3 med døra opp og startstedet, et frittstående alter med lys midt under hvelvet og benkerader vendt mot det), `gravgang` (12 m med gravheller, sarkofager annenhver side med lys på lokket og en knapp på vakt), `skattkammer` (hemmelig rom med kiste bak en falsk vegg, og skrin, kiste, sekker og en rustning på stativ i hjørnene). Innholdet er `prop` med modellen i `label` (byggesettets pynt eller møblene fra byen i `crypt::PROPS`, med kolliderne etter formen eller fra `<navn>_inside.json`) og fotavtrykket i `size` som hull i nettet; det står aldri foran en åpning (alteret står fritt, ikke mot en vegg som kan få en dør).

**Prøvenivået** (`assets/levels/nivaa_test.json`) er bygd om av 14 moduler og har nøyaktig de samme bitene på de samme stedene som før (266 instanser), de samme rommene (gangen og sidegangen er flere moduler med `room`), porten som eierens valg på hvelvets åpning (`port1`), og den falske veggen fra skattkammeret.

**Bildene til eieren** (`python3 tools/crypt/module_shots.py`): alle modulene står side om side i nivået `moduler` (`compose::modules::gallery`, `snapshot --level-doc moduler`, `?nivaa=moduler`); gangene, trappene og de ferdige rommene som ikke er saler får et lite naborom ved åpningene, så bildet viser dem koblet. Hver modul får ett bilde med kartet over modulen (grønne piler: åpningene; lilla: falsk vegg; blå: bue) og to bilder innenfra med faklene og lyset. `oversikt_test.png` er kartet ovenfra over prøvenivået, tegnet av det kompilatoren lager (`snapshot --dump-doc`). En modul alene er `snapshot --level-doc modul:grav/kapell`.

**Godkjenningssiden:** modulene er kortene (`u_<tema>_<navn>`, kategorien Krypten, «Se i 3D» viser modulen med naboene), og versjonen er modulfila sammen med bitene i temaet. Bitene står i `assets/catalog/modules.json` (`gjennom_moduler`), vises ikke som egne kort (bare i en sammenfoldet liste) og dømmes gjennom modulene.

**Testene** (`src/level_doc_tests.rs`): `modulene_utvides_til_celler` (settet finnes, hver modul alene og alle side om side bygges tett og uten feil, prøvenivået er moduler), `modulene_dreies_og_speiles` (hver modul i alle fire dreininger og speilet: bitene står likt dreid tilbake, åpningene følger med, tett), `modulene_fores_sammen` (dør mellom rom, gangåpning mot en gang, mur mot sida av en gang, gangen fortsetter, bare én dør langs to kanter, eierens mur, bue og port, skattkammeret med den falske veggen og sprekken, og modul mot løs celle), `to_moduler_i_samme_celle_er_feil`; og prøvenivået gjennom de gamle testene som før.

**Gjenstår:** siden `?nivaa` (fase 3) bygges med modulpaletten, åpningene som piler og valget per åpning; eierens egne moduler (lagre et utvalg som modul, del 9); moduler i flere etasjer og med galleri (fase 6); modulsett for de fire andre temaene (fase 8); `overmur` og `rekkverk` i moduler (fase 6).

## 22. Steinen (eieren 19. oktober)

«Det må være mulig å skifte type tekstur på mur også, eller ha moduler med forskjellige teksturer ut ifra nivået vi er på.» En modul er en form, og steinen er et valg.

- **Steinene** står i `assets/levels/steiner.json`: `id`, `name` og lagene for `floor`, `wall` og `vault`, og `themes` med standardsteinen per tema (byggesettet for et tema er laget med den: gravkjelleren med `crypt_floor`/`crypt_wall`). Ti steiner: gravmur, gravtegl (teglhvelv), beinhus, kalkmaleri, tegl, berg, tømmer, kvader, kjernestein og plast. Lagene for kloakken, gruva, borgen og kjernen er tatt inn (lag 55–62; plass til ett til).
- **Valget** (`stone` i dokumentet): på nivået (sammen med `theme`, som gir standardsteinen), rommet, modulen (går foran malens celler), cella og kanten (`edges` med `cell`, `side` og `stone`, uten `kind`). Verdien er et steinnavn (`"tegl"`) eller flatene hver for seg (`{"wall": "sewer_brick", "vault": "gravtegl"}`: et lagnavn, eller flaten fra en stein). Det nærmeste vinner, flate for flate. Ukjent stein eller lag er en feil (`stein`) med sted.
- **Kompilatoren** setter `stone` på hver instans (lagene biten er laget med → lagene den får). Cella, halvkantene og hjørnene den setter ut tar dens stein, så overgangen mellom to steiner går midt i åpningen (del 5.5); kanten overstyrer bare cellas halvdel av muren. Den falske veggen får murens stein (merket `secret_wall`). Faklene byttes ikke.
- **Motoren** gjør det til `model::Look`, og vertexshaderen bytter laget per instans (`look` i `world.wgsl`): gulvlaget blir gulv, murlaget mur, og murlaget som vender nedover i modellens rom (normalen under −0,3) hvelv. Sammenslåingen per 16 m baker det samme inn i hjørnene. Instansene med samme modell tegnes fortsatt i ett kall: nivået `steiner` har 21 kall med ti steiner og 21 uten.
- **Bildene:** `snapshot --level-doc steiner` (`assets/levels/nivaa_steiner.json`: rom 2 × 2 og ganger i alle ti steinene, med overganger i dørene og gangene).
- **Det som ikke passer ennå:** søylene, ribbene, gjordbuene og karmene er laget med murlaget og får murens stein (berg på en hogd søyle, tømmer på ribbene, plast på gjordbuene); en egen rolle for hogd stein (`.dressed`) trenger et eget lag i bitene. Gravhellene og mørtelen (`grave_slab`, `plaster_old`) byttes ikke. Kalkmaleriet ser ut som tapet som før. Hvelvene er mørke, så hvelvsteinen synes lite.
