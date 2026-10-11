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
