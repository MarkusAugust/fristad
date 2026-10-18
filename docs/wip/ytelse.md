# Ytelse: ny renderer (claude/detaljnivaa) mot gammel (master)

Logg for hjelperen som måler fragmentkostnaden. Ikke commit midlertidige shaderendringer.

## Bakgrunn
Eieren: 9 fps (GPU 223 ms) i ny renderer mot 36 fps (GPU 44 ms) på master, Mac, WebGPU, 961×692.
Ny tegner 185k trekanter, gammel 1 229k. Måles her med lavapipe (relativt).

## Oppsett
- Arbeidstre for master: `/home/user/fristad-master` (`assets/external` symlinket), binær i `target-master/release/snapshot`.
- Måling: `RUST_LOG=info snapshot --size 960x540 --frames 8|1`, logglinja «Rendret N bilder på X ms», per bilde = (t8 − t1)/7.

## Lesning av shaderen (ny) – det som koster per fragment
- `terrain()`: 8 × `fbm2` (4 oktaver hver = 32 noise2 = 128 hash21) for varpen av vektene, + 1 fbm2 for gresstonen, + 1 fbm2 i `fs` for albedovariasjon. Opptil 6 lag × 2 oppslag (albedo + nrh), søle = 2 × 2 oppslag, berg = triplanar (opptil 3 × 2). Verste fall ~18 textureSampleGrad + 10 fbm2.
- `triplanar_s()` (alle hus/murer uten UVMAP): opptil 3 × 2 oppslag; berg på avstand (lag 15) dobbelt (×10-flis): 12 oppslag.
- `fs` for lag < K_PLAIN: + `fbm2` for variation, + `fbm2` for skitt nederst på veggene.
- `weather()`: fbm2 bare når regn/snø > 0 (kortsluttet ellers). OK.
- `shade()`: `sun_shadow` 4 × PCF (ut til 40 m), ggx for sol, lysløkke (8 lys standard, 48 avansert) med ggx per lys (pow ×2), `room_of`-løkke (0 oppe), sky_gradient (3 pow).
- `fog()`: pow 1.5, 3 exp, sky_gradient. Bare når density > 0.
- Himmelen (`sky.wgsl`): 2 fbm2 per piksel til skyer (+ aurora-løkke om natta).
- Glød: 5 nivåer ned (13 oppslag) + 4 opp (9 oppslag) + tonemap. Lite i oppløsning.

## Målinger
(fylles inn)

### Baseline (lavapipe, 960×540, ms per bilde = (t8 − t1)/7)
| Oppsett | Torget (2,+1.7,9) | Storgata (4.7,+1.7,28) |
|---|---|---|
| Ny, standard (MSAA 4×) | 963 | 1210 |
| Ny, avansert (MSAA 8×) | 1293 | – |
| Master, full (TAA, SSAO, 3 kaskader) | 4009 | 4986 |
| Master, --reduced | 1988 | – |

**Lavapipe snur bildet:** den nye rendereren er 2–4× *billigere* enn master i programvare. Problemet på Macen er altså ikke ren shader-ALU, men noe som er billig i lavapipe og dyrt på Metal/Mac (se under).

### Rust-side-brytere (ny, standard, torget)
| Variant | ms | andel av 963 |
|---|---|---|
| MSAA av (sample_count 1) | 313 | **−67 %** (lavapipe skygger per sample, så 4× MSAA ≈ 3–4× fragmentarbeid her; på ekte GPU-er er MSAA 4× normalt 10–30 %) |
| Skyggepasset av | 878 | −9 % |
| Glød (bloom) av | 980 | ±0 |
| Himmelen av | 1003 | ±0 |

Statistikk ny torget (`--stats`): skygge 21k verden + 386k modell, 84 kall; hoved 43k + 248k, 70 kall. Sum 635k modelltrekanter, 154 kall.

### Oppløsningen i nettlaget (viktig funn, `web/main.js` `renderHeight`)
Med vinduet 961×692 CSS-px og `devicePixelRatio` 2 (Retina) er `ph` = 1384, og «auto» i standard velger største trinn ≤ min(1384, 1080) = **1080p (1920×1080 ≈ 2,07 Mpx) × MSAA 4×**. Master i redusert tegnet 961×692 = 0,66 Mpx (og dynamisk `scale` 0,4–1,0 ned til 0,1 Mpx), i avansert 1,5 × = 1,5 Mpx uten MSAA. Det er 3× flere piksler før MSAA. CLAUDE.md sier «aldri tettere enn én piksel per CSS-piksel» i standard, men koden gjør det ikke lenger.

### Hva som er likt og ulikt mellom ny og master (lest i koden)
- Likt: `terrain()` (8+1 fbm2, opptil 6 lag), `triplanar`, `sample_plane`, `discard` i `fs`, `cull_mode: None` (ingen backface culling i noen av dem), anisotropi 8, mipmaps på lagene og modellene, rgba16float HDR.
- Ulikt: ny har MSAA 4×/8× (master TAA uten MSAA), ett skyggekart 4096² (master atlas 4096² delt i 2048²-kaskader), bloom alltid på (master: av i redusert), gress som instansierte modellkort (K_MODEL: 4 oppslag + alpha-to-coverage) mot masters K_LEAF-geometri, og oppløsningstrinnene i nettlaget (over).
