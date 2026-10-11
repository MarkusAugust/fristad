# Hode Z: Buttonsmedens hode rekonstruert fra forbildet

Start 10. okt 17:37 UTC. Kode: `tools/reconstruct/` (pakken, se `docs/rekonstruksjon.md`); `tools/models/hode_z.py` er en tynn innpakning.
Mellomfiler i `target/hode_z/` (git-ignorert, også ONNX-vektene). Ark i `shots/hode_z/`.

## Fremgangsmåte
1. `seg`: de fire flisene av `smith_head.jpeg` (1024² hver), segmentert mot den grå bakgrunnen (avstand + metning,
   største bit, lukk hull).
2. `depth`: Depth Anything V2 ViT-B som ONNX (`fabio-sim/Depth-Anything-ONNX` på GitHub-releases, onnxruntime fra
   PyPI; torch fra pytorch.org er sperret). Snitt av bildet og det speilede bildet.
3. `fuse`: visningene rettes inn etter topp/bunn av silhuetten (ortografisk, samme skala) og midten av silhuetten.
   Relativ dybde -> metrisk: per rad skal det nærmeste punktet i dybdekartet ligge på ytterkanten fra visningene
   på tvers (profilene for forfra/bakfra og omvendt); felles skala for alle fire, egen forskyvning per visning,
   Huber-vektet. Volum 3 px: felt = min over visningene av (signert avstand til silhuetten, avstand bak dybdeflaten),
   marching cubes (skimage).
4. `bake` (Blender 5.2): desimer til 7 000 trekanter, Smart UV, bak posisjon og normal per teksel (EMIT) og
   normalkart fra den høye modellen (Cycles, selected to active), 2048².
5. `project`: hver teksel farges fra de fire bildene: myk dybdetest mot z-buffer av den høye modellen, vekt etter
   normal mot kameraet, mykt mot silhuettkanten, to frekvensbånd (lave blandes med n·c², detaljene med n·c⁸).
   Hull fylles med push-pull.
6. `render`: Cycles 24 samples, ortografisk i samme vinkler som forbildet + to mellomvinkler.

## Logg
- 17:57 **runde_1.png** / `runde_1_mellom.png`: hele kjeden virker på første forsøk. Formen fra bildet stemmer så godt
  at projeksjonen passer: ansiktet, brynene, nesa, skjegglokkene og håret sitter der de skal, også i mellomvinkelen
  (45°, 20° ovenfra). Galt: lyset i renderen er for sterkt (dobbelt lys oppå det innmalte), toppen av håret har en
  trapp/kant (dybdekart som skjærer feil nær silhuettkanten), halskuttet er kantete og bakfra er det striper.
  Neste: slakk på dybden nær silhuettkanten, skrått plan for halskuttet, flatere lys.
- 18:03 **runde_2.png** / `runde_2_mellom.png`: flatere lys (hvit himmel 0,8 + svakt nøkkellys, så det innmalte lyset
  ikke dobles), slakk på dybden nær silhuettkanten (R = 28 px), halsplan. Fargene og kontrasten er nær forbildet;
  mellomvinkelen holder (ansiktet, brynene, nesa, skjegget og grå striper i håret på rett sted, ingen synlige sømmer).
  Galt: halsen er flisete der den møter skjegget (silhuetten forfra skjærer halsen bak skjegget, forbildet er ikke
  enig med seg selv), toppen av håret er strukket (ingen visning ser ovenfra), litt mykere enn forbildet.
  Neste: ren elliptisk hals bak skjegget (målt bakfra og fra profilen), så eksport og trekanttelling.
- 18:08 **runde_3.png** / `runde_3_mellom.png`: ren elliptisk hals bak skjegget (målt bakfra og fra profilen) med bunn
  foran under skjegget, skrått halskutt. Flisene under skjegget er nesten borte. Fortsatt: toppen av håret strukket,
  halsen bakfra kantete (boks med hult kutt).
- 18:16 **Pakken `tools/reconstruct/` er ferdig** (beskjed til koordinatoren): `sheet.py`, `depth.py` (vektene lastes ned
  til `target/reconstruct/`), `fuse.py`, `texture.py`, `finish.py` (render, turnaround-GIF, glb via slim.py),
  `job.py` (oppsett, layout `head2x2`/`body3`), `__main__.py` (`python3.13 -m tools.reconstruct smith_head` eller
  `… ark.jpeg --layout head2x2 --out x.glb --tris N`), `sheets.json` (alle 13 figurer, hode og kropp, med avvikene
  fra README; pilegrimens hodeark har egne ruter), `requirements.txt`. `docs/rekonstruksjon.md` skrevet, én linje i
  CLAUDE.md, og `docs/ny-stil.md` og `docs/prompter-folk.md` peker på metoden. Kjørt på nytt på smith_head:
  samme resultat (`shots/hode_z/pakke/ark.png`, `ark_mellom.png`, `smith_head_turnaround.gif`, `smith_head.glb`
  med 7 000 trekanter, 0,7 MB). Smart UV bruker nå 40° etter tipset fra hode_y.
- 18:22 **runde_4.png** / `runde_4_mellom.png` (pakken, `shots/hode_z/pakke/`): toppen glattes til en kuppel
  (øverste 14 %, gauss 3 voksler), og tekslene som vender opp eller ned vektes etter retningen i det vannrette planet,
  så toppen får jevne striper bakover i stedet for flekker. Halsen vider seg ut mot skuldrene (trapes bakfra).
  Renderen er nå nesten flat (hvit himmel 0,85 + svakt nøkkellys, nesten ingen glans, uten støyfjerner), så arket
  viser teksturen slik forbildet er malt. Kontroll: albedoen i ansiktet er lik forbildet i de samme punktene
  (199/141/115 mot 199/141/115). Mellomarket har fire bilder: 45° ovenfra, −40° nedenfra, ovenfra (60°) og skrått
  bakfra. Turnaround: `shots/hode_z/pakke/smith_head_turnaround.gif`. glb: `shots/hode_z/pakke/smith_head.glb`
  (7 000 trekanter, albedo og normalkart 2048², 0,7 MB).
  Gjenstår:
  - Halsen bakfra er fortsatt flat på sidene.
  - Kuttflaten under halsen er mørk (malt slik i forbildet bakfra).
  - Toppen er glatt, men uten de skulpterte lokkene forbildet antyder.
  - Renderen er litt mykere enn forbildet (forbildet er 1024 px per visning).

## Plan for hele kroppen fra `smith_body.jpeg` (samme metode, ikke startet)

**Arket:** tre ruter på rad (forfra, profil med ansiktet mot høyre, bakfra), 2048² delt i tre (`body3` i `job.py`,
`scale` ≈ 0,9 mm per piksel, så figuren blir ca. 1,75 m). Det er bare én profil, så dybden for forfra og bakfra
kalibreres mot én silhuett. Det holder, fordi forfra og bakfra kalibrerer hverandre.

**Endringer i verktøyet:**
1. `sheet.py`: masker bort den tynne gulvlinja under føttene. Ikke fyll alle hull: hull som har bakgrunnsfarge
   (mellom armen og kroppen, mellom beina) skal være åpne, ellers smelter armene fast i kroppen.
2. `fuse.py`: voksler på ca. 4 px (kroppen er ~1 900 px høy, ca. 230 × 480 × 160 voksler).
   - Kalibreringen per rad virker best på brystet og beina. Armene og hammeren står utenfor kroppen, så radene der
     får Huber-vekt.
   - Dybden nær silhuettkanten slakkes som for hodet.
   - Hammeren og hendene synes atskilt bare forfra og bakfra. I profilen ligger de foran låret, så profilens dybde
     må få slakk der (en maske for redskapet, segmentert på farge: jern/treskaft).
3. **Hodet tas fra hodearket** (mye høyere oppløsning) og kroppen fra kroppsarket. `texture.py` får en valgfri maske:
   tekslene over halsen farges ikke fra kroppsarket.
4. `texture.py`: 2048² for kroppen (eget kart) og 2048² for hodet. Forkleet og skjorta får det meste av plassen.
   Undersiden av armene og innsida av beina ses dårlig; der fyller push-pull.

**Skjøten mellom hode og kropp:**
- Halsen på hodet kuttes med samme skrå plan som i dag. Kroppen kuttes med et plan litt under, så halsen på hodet
  går ned i kragen.
- Begge rekonstrueres i samme målestokk: hodets høyde fra isse til hake måles i kroppsarket og hodearket, og hodet
  skaleres til kroppen.
- Hodet settes på der hodet i kroppen var: plasseringen finnes ved å passe hodets silhuett til kroppsarket forfra
  og fra siden (minste kvadrat på silhuettkanten).
- Fargeovergangen på halsen jevnes med en kort gradient på huden: snittfargen fra begge kartene i en ring på 2 cm.
- Skjegget henger ned over kragen, så skjøten er skjult forfra.

**Skjelettet (18 bein) og grepet om hammeren:**
- Rekonstruksjonen står i A-posen fra arket (armene ned, litt ut). Leddene legges ut fra silhuettene og formen,
  ikke fra MakeHuman: hofter og knær fra beina i front- og profilsilhuetten (smaleste punkt = kne, ankel), skuldre,
  albuer og håndledd fra armene forfra, og hode og nakke fra halskuttet.
- Vektene: `game_bones` fra `folk_motion.py`/`people.py` med de samme 18 navnene, bundet med Blenders
  `ARMATURE_AUTO` (varmediffusjon) på den desimerte modellen.
  - Forkleet og skjorta får vektene glattet langs kroppen, så forkleet ikke rives av lårene.
  - Hodet (egen modell) henger stivt på `head` med en myk overgang i halsen.
- Hammeren skilles ut som egen del: masken på farge forfra og bakfra, pluss formen foran låret, blir en egen bit
  med egen tekstur. Den henger stivt på `hand.R` (`rigid` i `bake_textured`), med `grips={"R": "hang"}` i
  `folk_motion.clips`.
  - Fingrene er alt formet rundt skaftet i forbildet, siden skjelettet ikke har fingre. Klippene holder hånda i
    samme retning som i hvileposen, slik `folk_motion.py` forutsetter.
- Kontroll: `pose_preview.py` med `walk` og `idle`, og mellomvinkel-testen på hele figuren (45°, litt ovenfra).

**Risiko:**
- Kroppsarket er 2048 px for hele figuren, så detaljene blir mykere enn på hodet.
- Profilen viser bare én side: hammeren på høyre side forfra blir projisert inn i låret på den andre siden. Dybdetesten
  og en maske for redskapet hindrer det.
- Armene i A-posen: der armen dekker kroppen i profilen, er siden av brystet usett. Den farges med push-pull fra
  forfra og bakfra.


## Kroppen (fra 18:24)

Utvidelser i pakken (ingen sidekode):
- `job.py`: `body3` deler arket ved mellomrommene mellom figurene (`gap_split`). Figurene står ikke i like
  tredjedeler, og tredjedeler kuttet armen forfra.
- `job.py`: bare én profil finnes, så den andre siden er profilen speilet og brukes bare til formen (`geo_only`,
  aldri til teksturen). Uten den ble siden uten profil en flat vegg.
- `sheet.py`: gulvlinja fjernes (`floor_line`), og hull med bakgrunnsfarge står åpne (`open_holes`).
- `depth.py`: høye ruter får kvadratiske vinduer med overlapp, og hvert vindu tilpasses affint til det som alt er
  satt sammen. Ellers ble figuren klemt til 518 × 518.
- `finish.py`: arket for `body3` er forbildet (samme ruter) over modellen.

- 18:31 **kropp_1.png** / `kropp_1_mellom.png` (`shots/reconstruct/smith_body/`, glb med 14 000 trekanter).
  Bra:
  - Forfra og bakfra leses figuren godt: forkleet, beltet, hanskene, skjorta og støvlene sitter der de skal.

  Galt (ærlig):
  1. Armene er delvis borte, og hammerskaftet mangler. I profilen ligger armen foran kroppen, og dybden skjærer
     den bort. Bare 53 % av tekslene er dekket.
  2. Beina og støvlene er trappete i vannrette skiver, både i mellomvinkelen og i profilen.
  3. Kalibreringen er svak: avviket i medianen er 20–39 px mot 4–9 px på hodet. Dybdemodellen gir lite relieff på en
     hel figur.
  4. Hodet er lavoppløst. Det skal byttes med hodet fra hodearket.

  Neste:
  - Egen kalibrering per visning.
  - Dybden for tynne deler (armer, bein) bare der masken er bred nok, ellers silhuett med avrunding: tverrsnitt som
    ellipse fra bredden forfra og dybden fra profilen (som halsen på hodet).
  - Hammeren som egen bit (maske på farge).
  - Så hodet, riggen, klippene og spillet.
- 18:40 **kropp_2.png** / `kropp_2_mellom.png` / `kropp_2_turnaround.gif`:
  - Egen dybdeskala per visning (`scale_per_view`): avviket i medianen falt til 13–23 px.
  - Liten slakk ved silhuettkanten for tynne lemmer (`edge_slack` 8).
  - Ny form-prior (`elliptic` i `fuse.py`): hvert tverrsnitt er en superellipse (p = 2,5), med bredden fra biten i
    raden forfra og dybden fra biten i raden i profilen. Dybdemodellen gir for lite relieff på en hel figur, så uten
    denne ble bein og armer firkantede bokser.
  - Den speilede profilen brukes nå også til teksturen på siden uten profil (`mirror_texture`). Dekningen steg fra
    53 til 80 %.

  Bra:
  - Beina, støvlene, forkleet, beltet og skjorta holder i mellomvinkelen.
  - Fargene er som i forbildet.

  Galt:
  1. Underarmene er tynne og brutte forfra.
  2. I profilen er det et hull på innsida av armen, og hammerskaftet er borte. Armen ligger foran kroppen i profilen,
     og den tynne biten forsvinner i desimeringen.
  3. Hodet er lavoppløst. Det skal byttes med hodet fra hodearket.
  4. Hammeren er ikke skilt ut ennå.

  Gjenstår av bestillingen:
  - Armene: egen sylinder-prior langs armen, eller å segmentere armene forfra og gi dem egne biter.
  - Hammeren som egen bit.
  - Hodet satt på etter silhuetten.
  - Riggen med 18 bein og vekter, og klippene.
  - Eksport til `assets/models/npc/npc_smith.glb`, GIF med `motion_sheet.py`, nærbilde av hånda, og skjermbilde
    fra spillet.

### Fortsettelse 11. okt (etter kvotestoppen)
- **kropp_3**: rutenettet var for smalt (fast ±0,33 × rutebredden), så underarmene og hammeren ble kuttet av
  kanten. Nå følger rutenettet silhuettene med marg, og armene er hele.
- **Feil funnet:** `texture.py` satte `J.SCALE = 0.00028` da modulen ble lastet, så hele kroppen ble bygd i
  hodets skala (0,5 m høy). Linja er fjernet. Hodet er ikke berørt, fordi det bruker den skalaen uansett.
- **`assemble.py`** (steget `assemble --head smith_head`):
  - Hodet passes til kroppsarket på bredden av skallen forfra og fra siden. Skalaen ble 0,438 forfra og 0,432 fra
    siden, altså godt samsvar.
  - Kroppen kuttes litt over halskuttet på hodet.
  - Redskapet finnes på farge og form: jernhodet som største grå flekk, skaftet som et bånd fra jernhodet gjennom
    neven, og forfra en loddrett stripe.
  - Spøkelset av hammeren i venstre hånd fra den speilede profilen fjernes.
  - Alt bakes til én tekstur på 2048², der hodet får 2,2 ganger så lange uv-kanter.
  - Hammeren skilles ut igjen som `tool_hammer` og henger stivt på `hand.R`.
- **`rig.py`** (steget `rig`):
  - De 18 beina legges ut fra silhuettene: beina ved kneet og ankelen, armene ved albuen og håndleddet (et halvt
    armmål inn fra kanten der armen ligger inntil kroppen), og ryggen og nakken på midtlinja.
  - Vektene er automatiske (varmediffusjon). Armvektene fjernes fra kroppen, og halve lårvekten mellom beina går
    til hoftene (forkleet).
  - Klippene kommer fra `folk_motion.clips(rig, "smith", {"R": "hang"}, tools=hammeren)`.
- **kropp_4.png** / `kropp_4_mellom.png`: hel smed med hodet fra hodearket, armene hele og hammeren skilt ut
  (250 flater, stiv på `hand.R`). 12 800 trekanter og én tekstur på 2048².
  - Halsen: kroppen kuttes ved halskuttet innenfor halsens radius. Utenfor halsen (skuldrene, kragen) kuttes det
    først 7 cm høyere, så det ikke blir hull ved skuldrene.
  - Spøkelset i venstre hånd fjernes bare i armbiten forfra. Første forsøk tok også forkleet og ga hull.

  Ærlig vurdering:
  - Forfra, fra siden og bakfra er figuren nær forbildet, og mellomvinklene holder.
  - Galt:
    1. En kort stump av spøkelsesskaftet står igjen i venstre neve.
    2. Hodet er litt mykere enn i hodearket (4 500 trekanter og en mindre del av teksturen).
    3. Undersida av armene og innsida av beina er fylt fra naboene.
- **kropp_5.png** / `kropp_5_mellom.png` (rettet etter koordinatoren):
  1. Venstre neve har ingen egen dybde fra siden, siden profilen viser høyre side. Alt i venstre hånd utenfor
     nevens dybde fjernes, sammen med skaftet i redskapsmasken.
  2. Kuttflaten under halsen på hodet (malt mørk i forbildet bakfra) fjernes, så den ikke stikker opp over kragen.
     Hullene etter kuttene lukkes.
  3. Løse flak under 2 % av modellen fjernes.
  - Ovenfra er det ikke lenger hull.
  - Et lite skaftstykke foran venstre neve kan fortsatt synes skrått fra venstre.
- **Riggen**:
  - Armene: hjørner tett ved armbeina følger bare armen, og hendene er stive.
  - Forkleet: halvparten av lårvekten mellom beina går til hoftene.
  - Klippene kommer fra `folk_motion.clips` med smedens stil, men uten `arm_out`. De rekonstruerte armene henger
    alt et stykke ut, så med stilens ekstra utsving sto armene ut som vinger.
  - `keep_clear` brukes ikke, fordi lårradiusen målt med forkleet sperret armen langt ut.
- **Levert (11. okt, ca. 03:55):**
  - `assets/models/npc/npc_smith.glb`: 12 150 trekanter, én albedo og ett normalkart på 2048², 19 ledd og 11 klipp.
    Gammel modell i `target/reconstruct/npc_smith_old.glb`.
  - Arket: `shots/hode_z/kropp_5.png` og `kropp_5_mellom.png`.
  - Bevegelsene: `shots/hode_z/rig/smith_walk.gif`, `smith_idle.gif` og kontaktark.
  - Nærbilde av hånda: `shots/hode_z/rig/hand_idle.png` og `.gif`.
  - Spillet: `shots/hode_z/spill/smia_2.png` og `smia_3.png`.
  - `cargo test` på `alt_er_ekte_modeller_med_teksturer` og `innbyggerne_gar_og_stopper`: ok.

  Ærlig vurdering:
  - Formen og fargene står godt både på arket og i spillet.
  - Galt:
    1. Hammeren er tynn og flat: jernhodet er en skive, fordi den bare er rekonstruert fra to tynne silhuetter.
    2. Neven er smeltet inn i forkleet, så skinnet strekkes ved hånda når armen svinger i gangen.
    3. Armsvinget er lite.
  - Neste:
    - Hammeren som egen enkel rekonstruksjon eller modell (jernhode og skaft) på samme sted.
    - Skille hånda fra forkleet med en maske for hud forfra.

### Smeden, runde 2 (hånd, hammer, armsving)
- **Hammeren** er nå en enkel modell (`assemble.model_tool`): et rundt skaft på 32 cm og et jernhode på
  13 × 4,5 × 5 cm med fas. Den legges langs hovedaksen til den rekonstruerte hammeren, har malte farger for jern og
  tre med litt støy, bakes inn i den felles teksturen og henger stivt på `hand.R`.
- **Armene** skilles fra kroppen (`assemble.split_arms`): hjørner i armbiten forfra fra armhula til håndleddet, og
  nær armbeina i 3D, merkes med attributtet `arm`. Flatene som binder armen til kroppen fjernes, og hullene lukkes.
  Riggen gir armhjørnene bare armvekter og resten av kroppen ingen armvekter.
- **Nevene** skilles ut (`assemble.split_hands`): hud forfra eller bakfra nær hånda. De henger stivt på
  `hand.L`/`hand.R`, og lokket der neven satt fast får hudfarge.
- **Armsvinget** er 1,6 ganger stilens (`arm_swing`).
- **Leveranser:** `shots/hode_z/kropp_6.png`, `kropp_6_mellom.png`, `rig/smith2_walk.gif` og
  `rig/hand2_walk.gif`/`hand2_idle.gif`.
- **Ærlig:**
  - Gangen er tydeligere, og hammeren har volum.
  - Nærbildet av hånda er fortsatt ikke rent: det er takkete skinn på underarmen, og forkleet ved neven er
    revet. Grunnen er at nevene, underarmen og forkleet er én sammensmeltet form i rekonstruksjonen. Det løses bare
    helt med egne modeller for hendene, eller et forbilde der armene står ut fra kroppen (A-positur).

### Vertinnen og Gorvek (11. okt)
- **Feil i hodearkene:** de fleste har oppe til høyre en profil som ser *mot venstre*, motsatt av smeden. Med feil
  retning ble ansiktet projisert på bakhodet. `sheets.json` har nå riktige ruter for ti hodeark etter en kontroll
  på miniatyrer. Fiskerkona har to profiler mot høyre, så bare én brukes.
- **`job.gap_split`:** gulvlinja står høyere i noen ark, så grensen er senket til 86 %. Der figurene overlapper i
  kolonnene (Gorveks øks), kuttes det der det er minst forgrunn nær tredjedelene.
- **`assemble`** er gjort generell:
  - Redskapshånda (`tool_side`) og redskapsslaget (`tool_kind`: jern eller tre for kruset) er valgbare, og
    skaftbåndet (`tool_handle`) kan slås av.
  - Redskapet søkes under brystet, så Gorveks grå pels ikke tas for jern.
  - Hammeren modelleres bare for smeden (`model_tool`).
  - Jernet på hofta (Vertinnens nøkler) merkes som stivt på hoftene (`hips_iron`).
- **`rig`:** stilen og grepet hentes fra konfigen (`innkeeper` med `L: hang`, `gorvek` med `R: hang`). Skjørtet
  (`skirt`) blandes mellom hoftene og lårene etter høyden og siden.
- **Feil funnet:** hodeskalaen ble søkt bare opp til 0,7. Hodearkene med ruter på 512 px trenger rundt 0,87, så
  hodene ble for små (Gorveks hår ble en smal flette). Søket går nå til 1,6.
- **Vertinnens hode** får dobbelt ansikt i mellomvinkelen, fordi bildegeneratoren har tegnet issen og halsen ulikt
  høyt i rutene. Ny innretting (`align: skin`): høyden og skalaen per profil finnes ved å passe andelen hud per rad
  (pannen, ansiktet, halsen) til visningen forfra.

### 11. okt, ca. 05:30: arkene rettet, tre figurer levert
- **Tilbakegangen i kropp_6** (mørke rifter oppover underarmene, flak ved hendene) kom fra `split_arms` og
  `split_hands`. Begge er nå av som standard, så armene er like rene som i kropp_4. Hullet der redskapet satt,
  lukkes. Hammerhodet hadde fått trefarge (feil materialrekkefølge), og skaftet stakk ut gjennom toppen. Begge
  deler er rettet.
- **Nærbildet** (`finish.closeup`, steget `closeup`): Cycles fra 3/4 forfra med 85 mm og mykt lys. Se
  `shots/hode_z/smith/hand_R.png`. Tommelen og fingrene fra forbildet synes rundt et skaftstykke, men det er malte
  fingre på en klump. Ekte fingre rundt skaftet krever en egen hånd, fordi kroppsarket bare har rundt 100 px
  over hånda.
- **Levert per figur** (`shots/hode_z/<smith|innkeeper|gorvek>/`): `ark.png`, `mellom.png`, `hand_*.png`,
  `<navn>_walk.gif`/`.png`, `<navn>_idle.gif`/`.png` og `spill.png`. Modellene ligger i `assets/models/npc/`
  (`npc_smith.glb`, `npc_innkeeper.glb`, `npc_gorvek.glb`, hver på rundt 12 000 trekanter med én tekstur på
  2048²). De gamle ligger i `target/reconstruct/npc_*_old.glb`. `cargo test` på `alt_er_ekte_modeller_med_teksturer`
  og `innbyggerne_gar_og_stopper`: ok.

Ærlig vurdering:
- **Vertinnen:** skautet, kjolen, forkleet, beltet og nøklene (stive på hoftene) står bra.
  - Kruset er speilet over i den tomme høyre hånda.
  - Det er en mørk søm over kinnet, der profilene ikke stemmer med visningen forfra (bildegeneratoren har tegnet
    dem i ulik høyde).
  - Det er en flekk nederst på kjolen i profilen.
- **Gorvek:** kroppen, pelsen, beltene og støvlene er nær forbildet.
  - Øksa er speilet over i venstre hånd.
  - Det lange håret bak er tynt.
  - Det er løse flak på buksene ved øksa.
- **Felles svakhet:** den speilede profilen dupliserer det som bare den ene hånda holder. Spøkelsesfjerningen
  virker for hammeren, men ikke for kruset og øksa. Neste steg er å fjerne redskapet fra den speilede profilen før
  sammensmeltingen (maske i `sheet.py`), ikke etterpå.

### 11. okt, ca. 05:45: speilingsfeilen og de ti andre figurene
- **Speilingsfeilen:**
  - Redskapet tas nå ut av kildeprofilen før speilingen (`sheet.strip_tool`), innenfor en boks per ark i
    `sheets.json` (`tool_box`). Det som stikker ut forbi kroppen, tas ut av silhuetten. Det som ligger foran
    kroppen, males over fra kroppen rundt (inpaint).
  - Dybden i den speilede profilen interpoleres over redskapet (`depth.py`).
  - Viktigst: den speilede profilen får bare skjære i sin egen halvdel (`fuse.py`). Ellers skjærer den bort
    redskapet i den andre hånda.
  - Fargevarianten (`tool_strip: "color"`) brukes for Gorveks øks, der boksen dekker låret.
- **Øksa forsvant**, fordi dybdemodellen la øksebladet på låret bak, så dybden fra forfra og bakfra skar det bort.
  `depth_margin` (60 px for Gorvek, 30 for Vertinnen) slakker dybden. Silhuettene og ellipsen holder formen.
- **Vertinnens søm over kinnet:** projeksjonen toner nå over 30 px fra silhuettkanten (`edge_fade`) i stedet for
  10.
- **De tre konene** får hver sin modell (`npc_maid`, `npc_fishwife`, `npc_washerwoman`). `town.rs` faller tilbake
  på `npc_woman` om en mangler.
- **Lange staver** (høygaffel, spyd, stav) tas ut av silhuetten (`drop_pole`), ellers blir toppen av staven
  issen i innrettingen. Figurene står i grepet `long`, men uten stav. En enkel stavmodell som hammeren er neste steg.
- Batchen kjøres med `ten.sh`, med sikkerhetskopi etter hver figur.

### 11. okt, ca. 07:30: de fire doble figurene rettet
- **Vaktmannen og Bonden:**
  - Stavene (spyd, høygaffel) ble tatt med i silhuetten. Nå tas bare lange, loddrette, tynne biter (over 140 px)
    ut av silhuetten (`drop_pole`).
  - Til slutt beholdes bare den største sammenhengende biten.
  - Høygaffelens tinder får egne slettebokser (`erase` i `sheets.json`).
  - Innrettingen er nå riktig (lik topp og skala i alle visningene).
- **Fiskerkona:** hodearket har bare én brukbar profil. Den andre siden ble malt med visningen forfra på skrå, så
  det så ut som et ekstra ansikt. `job.setup` speiler nå den manglende profilen og bruker den også til teksturen
  (`mirror_texture` slås på automatisk).
- **Pilegrimen:** dybdekalibreringen ga negativ skala, fordi hattebremmen øverst forstyrret. Kalibreringen bruker
  nå bare radene 35–85 % (`cal_rows`), og en negativ skala faller tilbake på egen skala per visning.
- Bonden og Vaktmannen ble bygd på nytt 06:54–07:04, Fiskerkona og Pilegrimen 07:10–07:25.
  `npc_fishwife` står i `gltf.rs` og `town.rs`.
