# Åpningsfilmen i AAA-kvalitet: plan

Eieren 16. oktober, etter andre versjon: «Det er litt bedre, men fortsatt ikke bra. Planlegg hvordan dette kan bli gjennomført i AAA-kvalitet.» Historien er kanon (`docs/kanon.md`), manuset er `docs/film/manus.md`. Dette dokumentet er planen for utførelsen; det som er bestemt med eieren, flyttes inn i manuset når det er bestemt.

## 1. Hvorfor det ikke er bra nå

Sett med kritiske øyne på versjon 2 (scener med rolig kamera):

1. **Figurene er påklistret.** Hvert utklipp er generert for seg, med sitt eget lys, sin egen metning, sin egen penselstil og sin egen skala. Maleriet bak har gyllent sidelys fra venstre; Ansgar er jevnt belyst forfra. Herolden er for liten på podiet, folkemengden for stor. Grønnskjermen gir glorie langs kantene. Dette er hovedfeilen, og den kan ikke rettes med plassering eller kamera.
2. **Overgangene er umotiverte.** Å gli sidelengs fra rådssalen til torget viser to rom side om side med en myk skjøt. Ingen film gjør det.
3. **Én innstilling per scene i 10–15 sekunder.** Ingenting skjer. Film klipper hvert 3–6 sekund: totalbilde, halvnært, detalj.
4. **Teksten slåss med bildet.** Gotisk skrift over travle flater og ansikter, uten stemme som bærer den.
5. **Ingen lyddesign.** Bare musikk. Ingen torden på lynet, ingen lenker som rasler, ingen ild, ingen kirkeklokke, ingen stillhet.
6. **Oppløsningen er for lav** til å klippe inn i et maleri: 2048 px bredt, vist i inntil 1440p med zoom.

Det som er bra og beholdes: historien, kanon, rekkefølgen av scener, musikkens cuer, 16:9-rammen, filmen i sanntid med three.js, overgangen til spillet ved sørporten.

## 2. Hva AAA betyr her

Forbilder i samme sjanger (malte stillbilder med dybde, ikke 3D-animasjon): introen til *Darkest Dungeon*, forhistorien i *Pillars of Eternity*, Diablo III-fortellingene i «Black Soulstone»-stil, *The Banner Saga*, kartsekvensene i *The Witcher 3*. Det de har felles:

- **Ett maleri per innstilling**, malt som én helhet. Lagene er skåret ut av det samme maleriet, aldri hentet fra et annet bilde.
- **Kameraet gjør én tydelig ting per innstilling:** en langsom innzooming, en panorering, en dolly med parallakse. Aldri flere ting samtidig, aldri hopping.
- **Dybde kjennes:** forgrunnen er uskarp eller mørk, det fjerne er disig, lyset ligger mellom lagene (støv i sollyset, røyk foran lykta).
- **Livet i bildet er effekter, ikke figurer som vrir seg:** ild som flakker, regn, gnister, støv, røyk, lyn, et flagg i vinden. Figurene står stille som i et maleri; høyst et pust.
- **Klippene er motivert:** mørke, en overtoning, et match-klipp (et lys til et lys), eller kameraet går gjennom noe (en dør, et vindu, jorda).
- **En stemme bærer fortellingen,** og teksten på skjermen er kort, stor og aldri over et ansikt.
- **Lyden er halve filmen:** rom, foley, stillhet før slaget, musikken med treffpunkter på klippene.
- **Farge og etterbehandling:** en gradering per akt, vignett, glød på lysene, lett dybdeuskarphet.

## 3. Pipeline for bildene (den viktigste endringen)

Figurer og bakgrunn kommer fra **samme maleri**. For hver innstilling:

1. **Maleriet (A).** Eieren genererer hele innstillingen ferdig komponert, med personene på plass, i 16:9 og høyest mulig oppløsning (generatorens 4K-oppskalering, minst 3840 px bredt). Prompten beskriver komposisjonen presist (hvor i bildet figurene står, lysretning, tid på døgnet, palett for akten). Promptene skrives av meg per innstilling, komplette til å lime inn, med den faste stilsetningen for hele filmen.
2. **Den tomme utgaven (B).** Eieren ber generatoren redigere A: «fjern personene, behold alt annet identisk». Det er det moderne redigeringsmodeller gjør best, og det gir bakgrunnen bak figurene malt i samme stil og lys.
3. **Skillet** gjør jeg: differansen mellom A og B gir figurmasken, som renskes (morfologi, kantmyking, alfa fra differansens styrke), og figurene legges som eget lag med premultiplisert alfa. Ingen grønnskjerm, ingen glorie, samme lys som rommet. Der differansen er usikker, finjusterer jeg masken for hånd i en maskefil (`assets/film/shots/<innstilling>/mask.png`) som kan rettes.
4. **Dybden.** Eieren kjører A gjennom et dybdeestimat (Depth Anything v2 på Hugging Face Spaces, samme arbeidsflyt som TRELLIS), og laster opp dybdekartet. Jeg deler maleriet i 3–5 dybdesjikt etter kartet (himmel, bakgrunn, mellomgrunn, figurer, forgrunn) og lager et relieffnett per sjikt, så parallaksen er ekte innenfor hvert lag og ikke bare mellom flate plater. Hullene bak sjiktene fylles fra B og fra en liten utvidelse av kantene (inpainting ved å strekke kantpikslene; det synes ikke ved de små kamerabevegelsene vi bruker).
5. **Effektlag** males av eieren bare der de trengs (et flagg, en flamme, lysstrålen fra vinduet) som egne bilder med gjennomsiktig bakgrunn eller på B-utgaven, og animeres i shaderen (ild med støyforskyvning, flagg med bølge, lysstråle med støv).

Det vi har i dag, brukes ikke på nytt, bortsett fra der et maleri alt er komponert som én helhet uten figurer vi må legge til (himmelen, åsene, jordlaget, krypten). Alt annet genereres på nytt i denne pipelinen. Det er færre bilder enn det høres ut som: rundt 22 innstillinger à to bilder, og eieren har allerede motivene og stilen.

Verktøy i repoet: `tools/film/shots.py` (nytt) tar `A.png`, `B.png` og `depth.png` for hver innstilling og skriver sjiktene, maskene og nettene til `assets/film/shots/<innstilling>/`, med en kontrollside per innstilling (sjiktene side om side, masken over A, en liten prøveparallakse som gif) som jeg ser på før eieren ser noe.

## 4. Innstillingene

Hver scene får 2–3 innstillinger, hver 4–8 sekunder, med én kamerabevegelse hver. Manuset (`docs/film/manus.md`) får en innstillingsliste med: nummer, tid, hva vi ser, kamerabevegelse, lyd, tekst eller replikk. Utkast:

| # | Scene | Innstilling | Kamera | Tid |
|---|---|---|---|---|
| 1 | Riket | Morgen over åsene med byene og elva | Langsom panorering mot høyre, parallakse i åsene | 0–8 |
| 2 | Riket | Mønsterboka åpen på en pult i sola, sider med mønstre | Sakte innzooming | 8–13 |
| 3 | Skriptoriet | Skriptoriet utenfra i morgensol | Dolly inn mot vinduet | 13–18 |
| 4 | Skriptoriet | Salen innenfra, munkene ved pultene i sollyset, støv | Langsom kran ned | 18–25 |
| 5 | Skriptoriet | Ansgar nær, boka i hendene, lyset fra sidene | Sakte innzooming på ansiktet | 25–32 |
| 6 | Rådet | Rådssalen vid, kald, bannerne | Dolly inn langs bordet | 35–40 |
| 7 | Rådet | Rådet nær over bordet, kjøpmennene bak | Panorering langs ansiktene | 40–45 |
| 8 | Rådet | Komponentboken i lenker på pulten | Innzooming, lenkene gløder kaldt | 45–49 |
| 9 | Budet | Torget vid, herolden på podiet, folket | Kran ned mot podiet | 49–55 |
| 10 | Budet | Herolden nær med rullen og seglet | Still, seglet gløder | 55–62 |
| 11 | Inkvisisjonen | Inkvisitorene i rekke foran Skriptoriet i kveldslys | Dolly sidelengs langs rekka | 62–68 |
| 12 | Inkvisisjonen | Bålet med bøkene nær, gnister | Langsom innzooming, gnister stiger | 68–72 |
| 13 | Inkvisisjonen | Komponenten i rammen, lenket | Innzooming til den fyller bildet | 72–76 |
| 14 | Omveiene | Skriptoriet om natta, de tre slitne skriverne | Langsom dolly inn | 76–83 |
| 15 | Omveiene | Lappene i margen nær, et lys som brenner ned | Still, flammen flakker | 83–89 |
| 16 | Bruddet | Samme rom brister: sprekker med magenta, sider i lufta | Rystelse, glitch | 89–95 |
| 17 | Bruddet | Byen i uvær, Rammeverket reiser seg bak murene | Langsom kran opp med lynet | 96–104 |
| 18 | Forseglingen | Kirkegården, munkene med lykter leder skapningene | Dolly sidelengs mot kryptdøra | 104–113 |
| 19 | Forseglingen | Ansgar i døråpningen med lykta, ser tilbake | Sakte innzooming | 113–119 |
| 20 | Forseglingen | Døra mures igjen, siste stein | Still, lyset slukner | 119–124 |
| 21 | Krypten i dag | Ned gjennom jorda til hvelvet | Kran ned | 124–132 |
| 22 | Krypten i dag | Rammeverket sover i mørket, ett øye gløder | Langsom innzooming | 132–140 |
| 23 | Anno 1350 | Veien, den fremmede bakfra, byen i morgenlys | Dolly fram forbi den fremmede | 141–150 |
| 24 | Anno 1350 | Sørporten nær, fra samme sted som spillet begynner | Innzooming, toner over i spillet | 150–155 |

Overgangene: overtoning i mørke mellom aktene, rett klipp innenfor en scene (på musikkens slag), gjennom vinduet (3→4), lynet (16→17), ned gjennom jorda (20→21) og til spillet (24). Ingen sidelengs gliding mellom rom.

Det siste maleriet (24) males fra et skjermbilde av spillet ved sørporten (`snapshot`), så overtoningen til spillet er en ren match. Det er den ene plassen spillets bilde brukes, og bare som forelegg til eieren.

## 5. Kamera og dybde i film.js

- **Ett kamera per innstilling** med en enkel, langsom bevegelse (ease in og ut), aldri sammensatt. Bevegelsene er en liten brøk av bildet (3–8 %), så ingen hull åpner seg i sjiktene.
- **Relieffnett per sjikt** fra dybdekartet (ikke flate plater), så parallaksen ligger i selve maleriet: bordet foran stolene, søylen foran veggen.
- **Dybdeuskarphet** som etterbehandling (dybde per sjikt): forgrunnen uskarp når vi ser på figurene, bakgrunnen uskarp i nærbildene. Lett, aldri plakativ.
- **Atmosfære mellom sjiktene:** en svak dis mot det fjerne med scenens farge, lysstråler som støvet synes i, røyk og tåke som egne lag.
- **Slør og gradering:** én gradering per akt (gyllen, kald blå, rød og svart, magenta, blå natt, gyllen morgen), vignett, glød på lysene, ingen korn.
- Rendereren tegner i enhetens oppløsning opp til 1440p, sjiktene i 4K med mipmaps, premultiplisert alfa.

## 6. Liv i bildet

Bare effekter, aldri figurer som vrir seg:

- ild og lys: flammer med støyforskyvning i et malt flammelag, lyset på omgivelsene pulserer i takt (en lysmaske per lyskilde, malt eller avledet fra A med en radial falloff);
- støv i solstrålene, gnister fra bålet, aske, regn med dråper på forgrunnen, tåke i bakken på kirkegården;
- lyn: hvit blits, så maleriet i lysets kontrast et bilde;
- glitch i bruddet: blokkforskyvning og fargesplitting av hele bildet i korte rykk, ikke en maske som tegner;
- flagg og bannere: bølge i et eget lag;
- det eneste figurene gjør: et pust (0,3 % skalering, umerkelig) og øynene som gløder på Rammeverket.

## 7. Fortellerstemme og tekst

- **Fortellerstemme** gjør mest for AAA-følelsen. Tekstene i manuset er skrevet for å leses høyt. Tre veier: eieren leser selv (beste karakter, trenger en stille mikrofon), en skuespiller, eller en generert stemme som plassholder til en ekte finnes. Dette avgjør eieren (se spørsmålene nederst).
- **Teksten på skjermen** blir kortere når stemmen bærer: overskriftene (Fristad, Inkvisisjonen, Alt som var bygd brakk på én natt, Han kom aldri opp igjen, Fristad anno 1350) og de tre latinske linjene. Resten leses av stemmen og vises som undertekst nederst i rammen (lesbar, med skygge, i Uncial), som kan slås av.
- Overskriftene i UnifrakturMaguntia med gull, plassert i rolige flater som velges per innstilling for hånd (ikke automatisk), alltid i et felt uten ansikt eller lys bak. Teksten kommer med én myk inntoning, ikke ord for ord.
- Uten stemme: samme regel, men hver linje får et tonet felt (parkmentfarget slør) bak seg, så den alltid er lesbar.

## 8. Lyd

- **Partituret** får treffpunkter på klippene og aktenes vendepunkter (budet, bålet, bruddet, forseglingen), og stillhet der den virker (før lynet, når døra mures).
- **Rom per innstilling** (`soundscape.js`): salen med etterklang, torget med folk, kveld med vind, natt med en ugle, uværet, krypten med drypp.
- **Foley synkronisert med bildet:** fjærpenn på pergament, lenker, pergament som rulles ut, trompet, bålet, lyn med torden forsinket etter avstanden, munkenes sang (munkekoret fra prøvene), stein mot stein når døra mures, lykta som slukner, skritt på veien til slutt.
- Miksen: musikken senkes under stemmen (ducking), foley i rommet med etterklang.

## 9. Prosess (dette er det som mangler mest)

1. **Innstillingslista** (del 4) godkjennes av eieren først. Billig å endre.
2. **Storyboard:** for hver innstilling lager eieren maleriet A etter min prompt. Jeg setter de 24 maleriene sammen til et storyboard (én side med alle, i rekkefølge, med tid og tekst) og eieren godkjenner komposisjonene som stillbilder, før noen bevegelse er laget. Det som ikke holder, genereres på nytt med rettet prompt.
3. **Animatikk:** maleriene klippet etter tidene med musikken og stemmen (eller en plassholderstemme), uten parallakse. Eieren godkjenner rytmen.
4. **Pilot:** én scene (Skriptoriet, innstilling 3–5) gjøres helt ferdig: B-utgaver, dybde, sjikt, kamera, effekter, lyd, gradering. Eieren godkjenner piloten før resten.
5. **Resten** scene for scene, med en video per akt til eieren.
6. **Mix og gradering** til slutt, og én gjennomgang på eierens Mac i Chrome og Safari.

Hvert steg vises eieren bare når mine egne kontroller er bestått (del 10).

## 10. Automatiske kontroller

- Ingen figur svever: foten i figurmasken står på gulvet i maleriet (målt mot B der gulvet er).
- Ingen tekst over et ansikt eller et lys: tekstfeltet testes mot figurmasken og lysmaskene i hver innstilling.
- Kontrast: teksten måles mot bakgrunnen den står på i hele tiden den vises (WCAG-kontrast over 4,5).
- Oppløsning: ingen sjikt vises med mer enn 1 skjermpiksel per teksel når kameraet er nærmest.
- Ingen hull: parallaksen ved ytterpunktene av hver kamerabevegelse åpner ikke udekkede piksler (testes ved å tegne hver innstilling i ytterstillingene og lete etter bakgrunnsfargen).
- Kantene: ingen glorie (maskens kant har ingen piksler lysere eller grønnere enn begge sider).
- Lyd: alle foley-lyder finnes i manifestet med kreditering; ingen klipping i miksen.
- Tiden: hele filmen under 160 s, ingen innstilling under 3 s.

Alt dette kjøres av `tools/film/check.py` og testen `filmen_holder_malene`.

## 11. Rekkefølge og omfang

| Steg | Hva | Hvem | Omfang |
|---|---|---|---|
| 1 | Innstillingslista godkjent | eieren | en halvtime |
| 2 | 24 prompter (A) skrevet | jeg | en økt |
| 3 | 24 malerier generert og oppskalert | eieren | en kveld |
| 4 | Storyboard godkjent, nye prompter for det som ikke holder | begge | en runde |
| 5 | B-utgaver og dybdekart | eieren | en kveld |
| 6 | `shots.py`, sjikt og nett, kontrollsider | jeg | en økt |
| 7 | Animatikk med stemme og musikk | jeg (stemmen: se spørsmål) | en økt |
| 8 | Pilot: Skriptoriet ferdig | jeg | to økter |
| 9 | Resten av scenene | jeg | tre–fire økter |
| 10 | Lyd, mix, gradering, Mac-test | jeg | en økt |

## 12. Spørsmål til eieren

1. **Stemme:** leser du selv, skal vi finne en skuespiller, eller en generert stemme som plassholder? Språk: norsk (med latinen på latin).
2. **Generatoren din:** kan den redigere et bilde («fjern personene, behold alt annet») og oppskalere til 4K? Hvis ikke, må B males ved å beskrive samme scene tom, og jeg må justere lys i etterkant (dårligere).
3. **Dybdekart:** er det greit med samme arbeidsflyt som TRELLIS (Hugging Face Spaces, Depth Anything v2, last opp bildet, last ned kartet)? Jeg kan ikke kjøre modellen i skyen.
4. **Format:** 16:9 som nå, eller bredere filmramme (2,39:1, med svarte felt oppe og nede også på skjermen)? Bredere gir mer filmfølelse og enklere komposisjoner for panoreringer.
5. **Lengde:** 2,5 minutter står. Med stemme kan noen tekster forsvinne fra skjermen; er det greit?
