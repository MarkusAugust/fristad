# Ytelse: målingene

Referansemaskinen er eierens Mac mini M2 (WebGPU i Chrome og Safari, dpr 1, lerret rundt 950 × 1080). Målet er minst 60 fps i standard og avansert grafikk. Målingen er `?bench` (`BENCH_RUNS` i `web/main.js`); fps er det som teller, for delene (verden ≈ etterbehandling) er ikke til å stole på i nettleseren. Trekanter og kall er i hovedpasset.

## 11. oktober: MSAA ut, SMAA inn

Runde 1–3 (Chrome) fant årsaken til 45–60 ms i standard: MSAA i seg selv.

| Kjøring, torget | GPU ms | fps |
|---|---|---|
| Standard med MSAA 4× | 44 | 44 |
| Uten MSAA | 9,4 | 120 (skjermtaket) |
| Flat farge med MSAA | 23 | 79 |
| Uten modeller (26k △, 23 kall), med MSAA | 47 | 42 |
| Uten røyk / uten himmel, med MSAA | 41 / 45 | 43 / 44 |
| Halv oppløsning med / uten MSAA | 24 / 7 | 82 / 120 |

Utelukket: HDR-formatet (`Rg11b10Ufloat`), tidsmålingen, lagring av MSAA-målene, flere pass, alfakort, skyggepasset, røyken, himmelen og geometrien. Safari på samme maskin: MSAA 81 ms mot 14,5 uten. Beslutning (eieren): SMAA 1× i standard og avansert.

Runde 4 (SMAA):

| Kjøring | Chrome | Safari |
|---|---|---|
| Standard, SMAA | 9,4–10,3 ms, 120 fps | 13–15,5 ms, 60 fps |
| Standard uten kantutjevning | 9,9 ms | 15,2 ms |
| Standard med MSAA 4× | 37 ms, 45 fps | 71 ms, 26 fps |
| Enkel | 7–8 ms, 120 fps | 6–7 ms, 60 fps |
| Avansert, SMAA | 23–28 ms, 87–101 fps | 20–25 ms, 60 fps |
| Avansert med MSAA | 53–59 ms, 18–22 fps | 127–141 ms, 11–13 fps |

Hovedpasset: 172k–266k trekanter og 61–79 kall (standard), 211k–311k og 65–86 (avansert).

## 11. oktober: MacBook Pro M3 (dpr 2)

| Kjøring | Chrome (Edge), 1395×720 / 2231×1152 | Safari, 1630×900 / 2087×1152 |
|---|---|---|
| Standard, SMAA | 5,5–6,4 ms, 120 fps | 9,6–9,9 ms, 60 fps |
| Standard med MSAA 4× | 15 ms, 112 fps | 31 ms, 57 fps |
| Enkel | 6 ms, 120 fps | 6–7,6 ms, 60 fps |
| Avansert, SMAA | 8,1–8,5 ms, 120 fps | 12,7–13,4 ms, 60 fps |
| Avansert med MSAA 8× | 34–38 ms, 42–47 fps | 54–62 ms, 30–34 fps |

Standard tegnet bare 720p på Retina-skjermen (én piksel per CSS-piksel). Med god plass ble grensen hevet til 1,5 piksler per CSS-piksel, fortsatt høyst 1080p.

## 17. oktober: grafikkbryterne og løftet mot den malte stilen

Grafikken er nå én bryter per ting (`ports::Quality`), og enkel, standard og avansert er forhåndsvalg som setter dem. Nytt: myke skygger (tolv faste prøver), skyggeoppløsning, skygge langt unna (kaskade to til 110 m), skygge i krokene (halv oppløsning, bare WebGPU), solstråler, den malte tonemappingen med gradering etter sted og tid, og oppløsningsskala.

| Bryter | Enkel | Standard | Avansert |
|---|---|---|---|
| Solskygge, oppløsning | av | 4096 | 4096 |
| Myke skygger | av | på | på |
| Skygge langt unna | av | av | på |
| Skygge i krokene | av | av | på (ikke WebGL2) |
| Glød, SMAA | av | på | på |
| Solstråler | av | på | på |
| Malt gradering, tåke, røyk og nedbør | på | på | på |
| Sikt, lys | 160 m, 4 | 160 m, 8 | hele verden, 48 |
| Teksturtetthet, blandet bakke | halv, av | full, på | full, på |

Enkel har nå sollys uten skygge (før var sola av når skyggen var av, så enkel var lyst bare av himmelen).

**Målt i skyen** (lavapipe, programvare på delt CPU, 1280 × 720, torget mot sola, minste av tre kjøringer). Tallene er ms per bilde på CPU og sier bare hva som koster mye i forhold til hverandre; delene varierte opptil 50 % mellom like kjøringer fordi andre jobber delte maskinen.

| Kjøring | skygge | verden | etterbehandling |
|---|---|---|---|
| Standard | 307 | 526 | 188 |
| Uten myke skygger | 209 | 289 | 126 |
| Skygge 2048 | 202 | 528 | 227 |
| Skygge langt unna | 490 | 384 | 180 |
| Skygge i krokene | 233 | 372 | 223 |
| Uten solstråler | 364 | 607 | 109 |
| Gammel tonemapping | 237 | 353 | 155 |
| Enkel | 0 | 446 | 28 |
| Avansert | 635 | 754 | 229 |

Det som tydelig koster: skygge langt unna (skyggepasset tegnes to ganger, rundt +60 %), solstrålene (24 oppslag i fullt HDR-bilde per piksel; den største delen av etterbehandlingen) og skygge i krokene (to pass i halv oppløsning). Trekanter og kall er uendret i standard (samme utvalg; tabellen fra `tools/stats.sh` under). Med skygge langt unna dobles kallene i skyggepasset.

**Gjenstår på referansemaskinen:** `?bench` har fått kjøringer for hver ny bryter på torget (`gfx` i `BENCH_RUNS`): uten myke skygger, uten solstråler, med skygge i krokene, med skygge langt unna og uten malt gradering. Koster solstrålene eller de myke skyggene mer enn rundt 1 ms på M2 i standard, skal de av i standard; skygge i krokene skal av i standard om den koster mer enn 1,5 ms (den er av i standard nå).

`tools/stats.sh` (standard, 960 × 540), likt før og etter:

```
posisjon       trek. alle     kall  trek. hoved    kall instanser
sorporten         1235899      171       642663      89       276
storgata          1285787      168       625887      89       296
torget            1197379      168       497537      77       248
vertshuset         912045      140       210015      49        66
borghaugen        1031775      123       677310      82       291
utenfor           1055712      146       652930      83       272
oversikt           965695      128       693902      81       308
kjelleren          174945       50       174945      50        88
```
