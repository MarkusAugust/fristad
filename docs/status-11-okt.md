# Status 11. oktober 2026 og plan videre

Skrevet for gjennomgangen med eieren etter sammenslåingen (`claude/detaljnivaa` f48d23e). Målet er fortsatt det fra `docs/ny-stil.md`: en stilisert, håndmalt by og krypt på AAA-nivå (WoW/Fable/Valheim), skarpt bilde, 60 fps i standard på en vanlig maskin og 60 fps i avansert på en god.

## 1. Hvor vi står

### Det som er i ny stil og fungerer
- **Rendereren:** MSAA, én skarp skyggekaskade, malt lys, analytisk tåke, glød. I dag også delt i ugjennomsiktig og utskjæring (alpha-to-coverage og `discard` bare for alfakort).
- **Teksturlagene:** 55 malte lag fra eieren (byen, krypten, klær, materialer), alle avledet og i motoren.
- **Byen:** byggesettet (`kit.py`) med Vertshuset, spillerens hus, tjue hus, småbyggene, Smia, Apoteket, Skriptoriet, bymuren med tårn og porthus, gatelivet (brønner, boder, lykter, skilt, kjerre), kirkegården med krypten, fem bondegårder, åsene. Trapper med hoderom og trapphull skåret ut, utendørs småting som egne plasseringer på bakken, vinduskarmer (under eksport), gjerder i fire varianter langs hele sperrelinjene, Gorveks plass åpen, hul kiste, ny portal, galleriveggen med komponentbildene.
- **Krypten:** motoren (ti faste nivåer, navmesh med A* og trakt, mekanismer, rombasert lys, kart), byggesettet for gravkjelleren og nivå 1, nå tett (tetthetssjekk i eksporten).
- **Gjenstandene:** sheet2model prop2 (geometrien er prioren, arket projiseres): kista, tønna, brønnen, ambolten, hvilestedet, sarkofagen.
- **sheet2model:** frittstående pakke (figurer, gjenstander), regresjonstest, README og PROMPTS.
- **Nettlaget:** innstillingspanel (O) med grafikk og oppløsning i trinn, telleren med GPU-tid per del.

### Det som er tatt ut (usynlig til det er laget på nytt)
- **Figurene** (tretten, `APPROVED_FIGURES` tom): for små (1,2–1,6 m), hodene for store, hetter og hatter smeltet inn i skallen, hender som klumper, redskap borte. Siktemål og samtaler virker.
- **Fiendene og bossene:** modellene slettet; kampen virker mot usynlige fiender.
- **Førstepersonsarmene, våpnene og skjoldene.**
- **Dyrene, trærne, buskene, gresset, blomstene, steinene.**

### Det eieren har underkjent (må gjøres om, ikke pyntes på)
- **Figurene** (over).
- **Krypten nivå 1:** store tomme rom, harde overganger mellom ensformige flater, langt under målet (#43).
- **Brønnen** (og dermed teksturleggingen i prop2): strukket tak, skjev projeksjon på stolper (#39).
- **Kryptens dør:** dørbildet strukket, beslag dobbelt; dører og omramming må være én mekanisme som arver veggens lag (#35).
- **Byen er naken:** ingen trær, dyr, skrot, klesvask, kasser, folk i aktivitet.
- **Kirka** matcher ikke krypten i stil (#38).

### Ytelsen (den viktigste åpne saken)
- Eierens Mac, WebGPU, 961 × 692: **9 fps, GPU 223 ms** i standard (185k trekanter), 6 fps i avansert; **master: 36 fps, GPU 44 ms** med 1 229k trekanter.
- CPU per bilde er 0,5 ms (målt). I programvare (lavapipe) er den nye rendereren 2–4× billigere enn master. Altså noe som er billig i programvare og dyrt på Metal.
- Hovedhypotese, rettet og pushet (9815afa): hele world-pipelinen hadde alpha-to-coverage og `discard`, som skrur av fjerningen av skjulte flater på Apple-GPU-er, så all overtegning ble skyggelagt fullt. **Må bekreftes med telleren.**
- Neste kandidater om det ikke holdt: 9 `fbm2` per bakkepiksel og opptil 6 lag i `terrain()`, triplanar (6 oppslag) på alle vegger uten uv, MSAA-oppløsning med rgba16float, lysløkka med GGX per lys, himmelen med 2 `fbm2` per piksel.
- Krav: tre nivåer (enkel for de svakeste, standard ≥ 60 fps på vanlig maskin, avansert ≥ 60 fps på god maskin).

## 2. Hvorfor det blir dårlig, og hva som må endres i måten vi jobber på

1. **For løse spesifikasjoner og for rask godkjenning fra Claude.** Figurene gikk inn med en liste over kjente svakheter; nivå 1 ble eksportert uten tetthetssjekk; brønnen ble tatt inn med «taket er strukket» i rapporten. Regel heretter: ingenting går inn i spillet før det (a) består en automatisk kontroll der en finnes, og (b) eieren har sett det i bilde (og bevegelse for figurer) og sagt ja. Godkjenningspunktene står i planen under.
2. **Håndkoordinater i stedet for regler.** Tønnene på rad, trappa i veggen, gjerdestumpene, husene på høyeste punkt. Alt som plasseres skal gå gjennom felles logikk med krav og tester (innredning, sperringer, fundament, dører, trapper). Det er dyrere første gang og billigere for alltid.
3. **Fire hjelpere på én maskin** gir kø, konflikter i delte filer og sene rettelser. Forslag: egne skyøkter med oppstartsskript for de store, uavhengige jobbene (figurene, monstrene, kryptnivåene), små rettelser i serie her, og én tydelig sak om gangen som trenger eieren.
4. **Metoden for figurer og monstre** (fri smelting av dybde fra arket) gir ikke kontroll på proporsjoner, hodefeste eller påheng. Den må byttes med en mal med kjente mål (steg 4), og det samme prinsippet for monstrene.

## 3. Planen (foreslått rekkefølge)

Hvert punkt har et **godkjenningspunkt (G)** der eieren ser bilder før noe tas videre.

### A. Ytelsen først (denne uka)
1. Telleren fra eieren etter pipeline-delingen. Avgjør om A2 trengs.
2. Om GPU-tiden fortsatt er over ~25 ms: støyen i `terrain()` bakes til en flisbar tekstur (2 oppslag i stedet for 8 `fbm2`), høyst 2–3 lag per bakkepiksel, `weather()`/tåke kortsluttet, triplanar bare der normalen krever det, himmelen uten `fbm2` per piksel (bakt skytekstur). Mål per del i telleren.
3. **Tre grafikknivåer** i innstillingspanelet: enkel (ingen MSAA, 720p/900p, ingen solskygge, ett lag, 4 lys, ingen glød), standard, avansert. G: eieren leser telleren på jobb-PC og Mac.

### B. Spillbarhet og menyer (kort, i serie her)
- #40 pausemeny på Esc med innstillinger, tastatur og musefølsomhet; velkomstskjermen bare ved start; krypemodus bort.
- #41 skadetallene større; drikker og varer med reell, forklart effekt; vanskelighetsgrad per nivå.
- G: eieren prøver preview.

### C. Byggesettets felles regler (grunnlaget for alt videre i byen)
- #35 dørmodulen (proporsjon, beslag, omramming arver veggen), #36 planering og fundament (ingenting svever), #33 innredning med soner og historie per rom, #32 karmer (nesten ferdig), #37 fjellene høyere, #38 kirka som steinkirke og gjerdevarianter (ferdig).
- G: oversiktsbilde av byen og nærbilder av fem fasader, Vertshusets skjenkestue, kirka.

### D. Figurene på nytt (egen skyøkt, steg 4 i `docs/rekonstruksjon-plan.md`)
- Felles grunnkropp med riktige proporsjoner (hodet 1/7–1/8), hals, hender med fingre; målhøyde per figur (menn 1,72–1,85, kvinner 1,60–1,70, gutten 1,35, Gorvek 1,92); hodet fra hodearket skalert etter kroppens hodeboks og festet ved halsen, kroppens hode fjernet; hetter, hatter og redskap som egne deler med prior; teksturen bakt i malens faste uv; automatiske sjekker (ett hode, høyde, hendene).
- G1: Buttonsmeden og Bonden i bilder og bevegelse. G2: alle tretten, én og én inn i `APPROVED_FIGURES`.

### E. Monstrene (egen skyøkt, etter D, samme prinsipp)
- #24 skjelettmaler (tobeint, firbeint, klump, svevende) med kjente mål; arket passes til malen; `mob_motion`-klippene på malen; tydelig forskjell trash/elite/boss; de 8 arkene som finnes først, så de 12 siste fra eieren.
- G: hvert slag i bilder og bevegelse før det settes ut.

### F. Krypten nivå 1 om igjen (egen skyøkt, etter at byggesettets regler i C er satt)
- #43 mindre rom, lavere tak, sokler/gesimser/pilastre, nisjer og gravplater, lag blandet, innredning etter `dng_grav`-arket i grupper, lys med varm kjerne. Ny prompt for kryptpuss levert. Deretter nivå 2–10 tema for tema.
- G: kapellet og korsgangen i bilder før nivå 2.

### G. Gjenstander, armer og våpen
- #39 projeksjon per del etter form (sylindrisk, planar, flis på usette flater), strekk-sjekk; brønnen, ambolten og hvilestedet på nytt. Så de fem siste heltegjenstandene, inventaret etter arkene, armene og 3D-våpen/skjold (#41).
- G: gjenstandene i spillet før de brukes.

### H. Byen levende
- Trær og busker fra løvatlasene, gress, dyrene (egne ark per dyr), skrot og klesvask, folk i aktivitet, #42 historier og rykter.
- G: torget og enga i bilder.

### I. Rydding og utgivelse
- #10 død kode; CLAUDE.md; så spørre eieren før noe går til `master`.

## 4. Spørsmål til eieren
1. Telleren (standard/avansert) etter pipeline-delingen.
2. Er rekkefølgen A → B → C → D riktig, eller skal figurene (D) foran byggesettet (C)?
3. Egne skyøkter for D, E og F: greit at jeg lager oppstartsskriptet og kjører dem slik?
4. Er det noe på TODO-lista som skal strykes?

## 5. Revidert plan etter to runder med kritikk (11. oktober kveld)

Rammer fra eieren: én økt, én maskin, ingen flere skyøkter; Claude gjør alt selv i den rekkefølgen som gir mest mening; eieren godkjenner i bilder.

Det kritikken endret:
- **Måling i spillet** (`?bench`) før alt annet: GPU-ms per del, oppløsning, MSAA og brytere (`msaa`, `a2c`, `shadow`, `flat`) så årsaken til 223 ms kan skilles med én kopierbar tekst, uten rundturer. Per-sample shading fra alpha-to-coverage er hovedmistanken; flat fragmentshader mot full skiller skygging fra rasterisering.
- **Terrenget rettes uansett** (målet er jobb-PC): støyen bakt til flis, høyst 3 lag, søle bare ved vei, triplanar bare på berg, himmel uten støy per piksel, lys per rute, ruter tegnet forfra og bakover med bakken sist, culling bare på terrenget og kryptbitene (byggesettet lar vi være til tallene krever det).
- **Skyggepasset deles** som hovedpasset (ugjennomsiktig uten fragmentshader).
- **«Enkel» først**, og enkel = det WebGL2 får: ingen MSAA, 900p, ingen solskygge, ett lag bakke, 4 lys, ingen glød, 512²-lag (øverste mip hoppes over).
- **Teksturkravet avklares:** standard beholder 1024² per 2 m (0,5 teksel/mm), 1 m fliser der det sees nært; BC7 senere. Skrives inn i ny-stil.md.
- **#39 projeksjon før #43 krypten**, og krypten starter med **ett rom, kapellet** (8 × 6 × 3,4 m, sokkel/gesims/pilaster, bakt AO, lys med varm kjerne og kald utfylling, tre innredningsgrupper) til godkjenning i tre faste bilder før resten av nivå 1.
- **Innredning som minimal `group()`** (anker: vegg, hjørne, peis, bord; overlapp-, dørsone- og gangsonesjekk som feiler eksporten; en test som nekter håndkoordinater utenfor `group()`), først på tønnerekka ved peisen. Ingen «historie»-motor.
- **Figurer og monstre på samme grunnkropp**, bygd for hånd i Blender-skript (6–8k, hode 1/6,5, hals, hender med tre fingre, fast papirdukke-uv, 18 bein, parametre for høyde/bredde/kjønn/alder), tekstur ved planar projeksjon forfra/bakfra fra arket med innretting etter hodeboks og høyde; hodeplagg, hår og redskap som byggesett. Monsteret (Button Wight) først fordi det er ferdig på én dag og beviser metoden; figurene går gjennom samme dør samme dag (oppstillingsark med 13 målhøyder, gang-GIF, Buttonsmeden grovt projisert). Dybdefusjon beholdes bare til gjenstander.
- **`sheet_check.py`** avgjør hvilke av de 21 arkene som kan gjenbrukes (forfra/bakfra, rett, armene fri, hode/høyde 1/5,5–1/8, oppløsning, redskap i én hånd); de som stryker lages på nytt mot malens silhuett som referansebilde.
- **Regresjonsark per push** i Actions (de åtte posisjonene + tre i krypten + dreieark, pikseldiff mot godkjent grunnlinje) og **godkjenningsark i fast format** (figur: forfra/side/bak + gang; gjenstand: 3/4 forfra, 3/4 bakfra, nær; rom: inngang, fokus, hjørne). `Builder.missing` feiler testene for godkjente modeller. `SaveData` får versjon.
- Strykes/slås sammen: #21 og #34 lukkes; #23 + #24 → grunnkroppen; #36 blir en strengere test + planering under gårdene; #37 én time; #41 splittes (skadetall og drikker nå, 3D-våpen etter armene, vanskelighetsgrad etter nivå 2); #42 sist; #10 i første ledige hull.

### Dagsplan
| Dag | Gjøres | Eieren ser |
|---|---|---|
| 1 | `?bench` med bryterne; grafikknivået enkel; skyggepasset delt; død kode om tid | bench-tekst fra fire kjøringer (msaa 1/4 × full/flat) på Mac og jobb-PC; enkel spillbar |
| 2 | terrenget (støyflis, 3 lag), `textureSample` der det er uniformt, lys per rute, himmel, sortering, culling på terreng/krypt, skyggekart 2048²; teksturkravet i ny-stil.md | bench nr. 2 (mål: standard < 16 ms på Macen) og de åtte faste bildene før/etter som ett ark |
| 3 | #39 projeksjon per del med strekk-sjekk (brønn, ambolt, hvilested på nytt); `group()` og tønnerekka; kit_grav-moduler, AO, lysoppskrift; kapellet; godkjenningsark; missing → feil | kapellet, de tre gjenstandene og tønnerekka i godkjenningsark: tre ja/nei |
| 4 | grunnkroppen; `sheet_check` på 21 ark; Button Wight med `mob_motion`; oppstillingsark, gang-GIF, Buttonsmeden grovt; silhuett + prompt for ark som strøk; `tools/regress.sh` i Actions | første fiende i kapellet i bevegelse; oppstillingsarket; Buttonsmeden som retning; lista over ark som må lages på nytt |
| 5 | resten av nivå 1 med modulene og `group()`, fiender etter spawns; Modalen (klump); Buttonsmeden ferdig til godkjenning; `SaveData`-versjon; status for uke 2 | nivå 1 med fiender, Modalen, Buttonsmeden i bevegelse, bench nr. 3 |
