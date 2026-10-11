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
