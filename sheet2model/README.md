# sheet2model – rigget spillfigur fra et forbildeark

`sheet2model` lager en teksturert, rigget 3D-figur (glb) fra bilder en bildegenerator har laget: et **kroppsark**
(samme figur forfra, fra siden og bakfra) og et **hodeark** (hodet i fire ruter). Verktøyet gjør det bilde-til-3D-
tjenestene gjør, men lokalt, med åpne verktøy, og med en rapport som sier om resultatet holder før et menneske
ser på det. Pakken er frittstående: alt som er spesifikt for ett spill (skjelett, animasjonsklipp, materialnavn,
slanking av fila) ligger i et pluggbart **mål** i `targets/`.

Metoden: dybden leses fra hver visning med en åpen dybdemodell, visningene rettes inn mot hverandre med målbar
feil, smeltes til én form, og forbildet projiseres tilbake på den samme formen. Siden formen kommer fra bildet,
havner det innmalte lyset og detaljene der de skal.

- `PROMPTS.md`: hvordan arkene bestilles fra bildegeneratoren, og sjekklista før et ark brukes.
- `examples/fristad/`: 26 oppslag (13 figurer, hode og kropp) fra spillet Fristad, som eksempel på formatet.

## Hva du trenger

| | |
|---|---|
| Bildegenerator | En som kan lage «character turnaround sheets» (Gemini, Midjourney, SDXL …). Se `PROMPTS.md`. |
| Python 3.13 | `pip install -e sheet2model[blender,test]` (numpy, scipy, scikit-image, opencv, Pillow, onnxruntime, bpy, pytest). |
| bpy | Blender som Python-modul. Brukes til desimering, uv-kart, hull, skjelett og glb-eksport (sekunder), og til kontrollrenderen av arkene (Cycles, CPU). Teksturen regnes i numpy. |
| Dybdevekter | Depth Anything V2 ViT-B som ONNX, ca. 390 MB. Lastes ned første gang fra GitHub-releases (`fabio-sim/Depth-Anything-ONNX`) til `<root>/target/sheet2model/cache/`. Har du dem alt: `SHEET2MODEL_ONNX=/sti/til/depth_anything_v2_vitb.onnx`. |
| Maskin | 4 kjerner og 8 GB minne holder. Én figur (hode + kropp, rigget) tar 4–6 minutter første gang og under 3 minutter når dybden er bufret. |

## Fra idé til figur i spillet

1. **Beskriv figuren** i én setning med alder, kroppsbygning, ansikt, hår, klær og ett redskap i én hånd.
2. **Bestill arkene** etter malene i `PROMPTS.md`: kroppsarket (`body3`), hodearket (`head2x2`), og helst en rute
   ovenfra og redskapet alene. Gå gjennom **sjekklista** der før du bruker et ark; de fleste feilene i resultatet
   kommer fra ark som ikke stemmer med seg selv.
3. **Skriv et oppslag per ark** i `sheets/<navn>.json` (se formatet under). Et minimum:
   ```json
   {"ref": "assets/ref/smith_body.jpeg", "layout": "body3", "head": "smith_head", "target": "fristad",
    "opt": {"floor_line": true, "open_holes": true, "elliptic": true, "scale_per_view": true,
            "tool_side": "R", "tool_box": [[440, 1140, 670, 1240]], "style": "smith"}}
   ```
4. **Bygg hodet**, så **kroppen med hodet på**:
   ```
   python3.13 -m sheet2model smith_head
   python3.13 -m sheet2model smith_body --rig-out assets/models/npc/npc_smith.glb
   ```
   Stegene (`--steps`): `seg` (rutene og silhuettene) → `depth` (dybde, bufret) → `fuse` (innretting,
   kalibrering, volum, marching cubes) → `mesh` (største bit, desimering, uv) → `texture` (projeksjon) →
   `finish` (arkene og glb) for et hode; for en kropp `assemble` (hodet på, redskapet ut, ett uv-kart, én
   tekstur) → `finish` → `rig` (skjelett, vekter, klipp, glb) → `report`.
5. **Les rapporten** (`target/sheet2model/<navn>/rapport.json`, oppsummert på én linje i terminalen). Er den
   OK, ligger glb-fila der `--rig-out` sa. Er den ikke OK, har verktøyet alt prøvd rettelsene (under) og feilet med
   `shots/sheet2model/<navn>/feil.png`, som viser hva som er galt.
6. **Se på mellomvinkelen** (`shots/sheet2model/<navn>/ark_mellom.png` eller `ark_hel_mellom.png`) før du tar
   figuren i bruk, og på gang- og hvileklippene i spillet.

Mellomfilene ligger i `target/sheet2model/<navn>/` (git-ignorert), arkene i `shots/sheet2model/<navn>/`.
Tiden per steg står i `tid.json` og i rapporten.

## Oppslaget (`sheets/*.json`)

| Felt | Hva |
|---|---|
| `ref` | Arket (relativt til `root`). |
| `layout` | `head2x2` (fire ruter: forfra, profil, den andre profilen, bakfra), `body3` (tre figurer på rad, delt ved mellomrommene), `single`. |
| `views` | Hvilken rute som viser hva, når arket avviker fra oppsettet: `{"front": [rad, kolonne], "face_right": …, "face_left": …, "back": …}`. `face_right` er profilen der ansiktet vender mot høyre i bildet. Oppgi alltid denne for hodeark; bildegeneratorer bytter ofte om profilene. |
| `extra` | Egne bilder utenom arket: `{"top": "sti/ovenfra.png"}` for ruta ovenfra. |
| `head` | (kropp) navnet på hodearket som settes på. |
| `target` | Målet i `targets/` (`generic` når det ikke er gitt). |
| `root` | Prosjektrota stiene er relative til (standard: mappa over `sheets/`). |
| `scale` | Meter per piksel i forfra-ruta (standard 0,00028 for hoder, 0,0009 for kropper). |
| `tris` | Trekanter etter desimering (7 000 for hoder, 14 000 for hele figurer, hodet får 4 500 av dem). |
| `limits` | Tersklene i rapporten (standardene står i `config.DEFAULT_LIMITS`). |
| `opt` | Avvikene per ark, se under. |

De viktigste i `opt`:

| Nøkkel | Hva |
|---|---|
| `floor_line`, `open_holes`, `min_hole` | Kroppsark: fjern gulvlinja; la hull med bakgrunnsfarge (mellom arm og kropp) stå åpne. |
| `drop_pole` | Lange staver (spyd, høygaffel) tas ut av silhuetten; staven må modelleres for seg (`PROMPTS.md`). |
| `erase` | Slettebokser per visning: `{"front": [[x0, y0, x1, y1]]}` for rester (høygaffeltinder). |
| `tool_side`, `tool_kind`, `tool_box`, `tool_strip`, `drop_mirrored_tool`, `model_tool` | Redskapet: hvilken hånd, jern eller tre, boksen rundt det i profilen, om det tas ut på farge, om spøkelset i den andre hånda fjernes, og om det byttes med en enkel modell (`"hammer"`). |
| `scale_per_view`, `elliptic`, `elliptic_p`, `edge_slack`, `edge_gain`, `depth_margin` | Formen: egen dybdeskala per visning, superellipse som prior for tverrsnittene (hele figurer), slakk ved silhuettkanten, ekstra dybde rundt redskapet. |
| `align` | Innrettingen: `{"shift": true, "shift_v": false, "scale": false, "yaw": 8, "roll": 3, "rounds": 2}`: sideveis forskyvning, høyde, skala, største dreining og rulling i grader (0 slår av). Høyde og skala står som standard (issen og bunnen gir dem; en tynn hattebrem forsvinner ellers). `align_skin: true` retter profilenes høyde etter hudlinjene først. |
| `same_scale`, `cal_rows`, `back_cut`, `neck`, `dome` | Hoder: rutene i samme målestokk; radene kalibreringen bruker (hattebrem øverst); halskuttet bakfra; ren elliptisk hals; kuppel på toppen når ingen rute ser ovenfra. |
| `edge_fade`, `poisson`, `delight`, `tiles` | Teksturen: toning fra silhuettkanten (px), Poisson-styrke på det lave båndet, lysdemping (0 = av: den malte stilen vil ha lyset), fliser for toppen og undersiden (`{"up": "hair.png", "down": "skin.png", "cm": 8}`). |
| `style`, `grip`, `skirt`, `hips_iron`, `arm_swing`, `arm_out` | Riggen (målet Fristad): stilen og grepet i klippene, skjørt, jern på hofta, armsving. |

## Gjenstander (`prop1`, `prop2`)

Gjenstander har ingen fire visninger, så formen kommer fra en **prior** (`opt.prior`: `box`, `cylinder`,
`barrel`) som passes til silhuetten (`props.fit`: priorens mål, kameraets vipp og dreining og nullpunktet mot
IoU på et grovt rutenett, sekunder), og dybden kalibreres mot prioren. Malingen projiseres på de synlige flatene,
den skjulte siden får speilbildet (gjenstander er symmetriske), undersiden den malte flisen som ligner mest
(`tiles: "auto"` leter i `assets/textures/painted/`), og blandingen er den samme som for figurene. `prop1` er én
3/4-visning (småting, 3 × 3-arkene med `grid` og `views`), `prop2` to (3/4 forfra venstre og 3/4 bakfra høyre).
Ikke laget ennå: `revolve` (radiusprofil fra silhuetten) og en glb-fil som prior (byggesettsmodellen), og
gjenstandene får ingen rigg (glb fra `finish`). Se `PROMPTS.md` for arkene.

    python3.13 -m sheet2model chest     # examples/fristad/chest.json: furniture_b.jpeg rute 1, box
    python3.13 -m sheet2model barrel    # rute 7, barrel

## Slik vurderer du resultatet

Rapporten (`rapport.json`) har tallene; terskel i parentes:

- **IoU per visning** (≥ 0,80): formens silhuett mot forbildets, etter innrettingen. Lav IoU i en profil betyr at
  profilen ikke stemmer med forfra/bakfra (dreid, annen høyde, annen figur).
- **Fotometrisk feil** per par av nabovisninger (≤ 0,11): snittavviket i luminans mellom det to visninger ser
  på de samme tekslene. Et dobbelt ansikt eller to kropper oppå hverandre gir høy feil mellom forfra og profilen.
- **Løse komponenter** (1 per del) og **hull** (≤ 1: halsåpningen): flak og skår etter dybde som ikke stemmer.
- **Trekanter**, **teksler per cm i ansiktet** (≥ 3; hoder får 30–40, hodet på en hel figur rundt 10) og
  **skarphet** (Laplace-varians i ansiktet, ≥ 60): at ansiktet har fått plass og detaljene fra forbildet.
- **Redskapet**: funnet, hvilken hånd, antall flater, om det ble byttet med en modell.
- **Innrettingen**: hva som ble flyttet, skalert og dreid per visning, med IoU før og etter. Store tall her
  betyr at arket ikke stemmer med seg selv, og at det er verdt å bestille det på nytt.
- `masked`: det som ble tatt ut av silhuettene (staver, redskap før speiling, slettebokser), så det kan sjekkes.
- `seconds`: tiden per steg.

**Mellomvinkel-testen:** `ark_mellom.png` viser figuren 45° mellom forfra og profilen litt ovenfra, −40° litt
nedenfra, ovenfra og skrått bakfra. Ingen av forbildene viser disse vinklene, så de avslører om formen og
projeksjonen virkelig stemmer, eller om det bare ser riktig ut fra kameraene forbildet ble tatt fra. Se etter
sømmer der to visninger møtes (rundt øret og kinnet), strekk på toppen, dobbelt ansikt, og at silhuetten følger
forbildet i `ark.png`.

**Automatiske rettelser** (`report.FIXES`, i rekkefølge; verktøyet beholder den beste): hudlinjer for
profilenes høyde → mykere kant i projeksjonen → bare én profil (hoder) → større flak fjernet → slakkere dybde.
`--no-fix` slår dem av.

## Vanlige feil og hva som forårsaker dem

| Feil | Årsak | Rettelse |
|---|---|---|
| To figurer oppå hverandre, dobbelt ansikt | Profilen er tegnet dreid eller i en annen høyde enn forfra; staven eller hattebremmen ble toppen av silhuetten | Innrettingen (yaw, skala, høyde) og `align_skin`; `drop_pole`; `cal_rows`. Høy fotometrisk feil forfra–profil i rapporten. |
| Redskap i begge hender | Bare én profil finnes, og den speiles med redskapet i | `tool_box` i profilen: redskapet tas ut før speilingen og males over; `drop_mirrored_tool` tar resten. |
| Løse flak og skår | Dybde som ikke stemmer mellom visningene; rester etter utskjæring av redskapet | `flake_frac`, `edge_slack`, `depth_margin`; hull lukkes med vifter. |
| Mørk søm over kinnet | Teksturen projiseres fra en visning som ikke stemmer med formen der | `edge_fade: 30`, innrettingen; det lave båndet går gjennom Poisson over sømmene. |
| Mørk eller lys flekk på forkleet der redskapet var | Redskapets piksler havnet på kroppen bak | Redskapets maske utelates fra kroppens tekstur (automatisk når `tool_side` er satt). |
| Hånda er en klump | Forbildet har 100 px over hånda; formen kan ikke leses derfra | Bare malen (under) eller en egen håndmodell løser det. |
| Strukket topp | Ingen visning ser ovenfra | En rute ovenfra (`extra.top`, se `PROMPTS.md`); ellers `dome` og fliser (`tiles`). |
| Figuren lener seg eller er dreid | Alle visningene fikk samme yaw i innrettingen | Trekkes fra automatisk (`yaw_gauge` i rapporten). |
| Strekk ved håndledd og forkle i gange | Automatiske vekter på et nett uten kjente ledd | Vektene ryddes (`rig.clean_weights`); malen gir ferdige vekter. |
| Alt er svart eller grått i spillet | Materialnavnet eller teksturen passer ikke spillets lastekode | `Target.material`, `Target.post_export`. |

## Begrensninger

- **Forbilder som ikke stemmer med seg selv** gir fortsatt flis der visningene er uenige: innrettingen retter
  dreining, skala og høyde, ikke en annen frisyre i profilen.
- **Lyset er malt inn.** Teksturen bærer lyset fra forbildet (ønsket i en malt stil). `delight` demper den
  retningsbestemte delen, men ikke penselstrøkene.
- **Bare én profil** (`body3`): venstre side er høyre speilet, og profilen kan ikke rettes inn sideveis (ingen
  uavhengig visning på tvers), så startinnrettingen etter topp og bunn står.
- **Ting som henger fritt** (en hammer ved siden av kroppen) smelter sammen med det bak om de ikke synes atskilt i
  minst to visninger; derfor bestilles redskapet også alene og byttes med en modell.
- **Hender og fingre** kommer ikke fra 100 px i et ark.

## Malen (steg 4 i planen): hva som finnes og hva som gjenstår

Planen er at en felles grunnkropp med kjent topologi, uv-kart, skjelett og vekter tilpasses forbildet, slik spill
gjør det, i stedet for en ny form fra bunnen per figur. Det gir hender med fingre, ferdige vekter uten strekk, og
en tekstur som alltid lander i samme kart. `template.py` har:

- grensesnittet `Template` (`v`, `f`, `uv`, `weights`, `bones`, `parts`) og `fit(template, views)`;
- `from_makehuman(base.obj)`: MakeHumans grunnmodell (CC0) forenklet til rundt 8 000 trekanter med tettere hode;
- en første `fit`: skalering per akse mot silhuettene og noen runder der malens silhuettkant dras mot forbildets,
  glattet over nettet.

Gjenstår før den kan brukes i løpet: uv-kartet på malen (fast, med ansiktet stort), vektene fra MakeHumans rigg
overført til målets skjelett, klær som skall (kjortel, forkle, skjørt, hette) og redskap som egen kategori,
dybdetermen i tilpasningen (dybdekartet som mål, ikke bare silhuetten), og at `cli.build` bruker `fit` i stedet
for `fuse` + `mesh` når oppslaget har `"template": "…"`. Teksturen (`texture.project`) virker alt på et vilkårlig
nett med uv, så den trenger ingen endring.

## Et nytt prosjekt

1. Kopier mappa `sheet2model/` inn i prosjektet og `pip install -e sheet2model[blender]`.
2. Lag `sheet2model/targets/<spill>.py` fra `generic.py`: skjelettet (`bones`), retningen (`to_game`), klippene
   (`clips`) og etterarbeidet (`post_export`). Fristads mål (`fristad.py`) viser alle fire.
3. Lag `sheets/` med ett oppslag per ark, pek på målet med `"target"`, og sett `SHEET2MODEL_SHEETS=sheets`
   (eller `--sheets`).
4. Legg `target/` i `.gitignore`: dybdebufferen og mellomfilene er store.

## Oppbygging

```
config.py    oppsettet (Job, oppslagene, rutene, standardverdiene og tersklene)
sheet.py     rutene og silhuettene; staver og redskap maskeres; speiling med maske
depth.py     Depth Anything V2 (ONNX), bufret per bilde
geom.py      kameraene: basis med yaw/pitch/roll, verden -> bilde
fuse.py      kalibrering av dybden mot det visuelle skallet, volum, marching cubes
align.py     innretting per visning mot leave-one-out-IoU
mesh.py      største bit, desimering, Smart UV (bpy)
texture.py   projeksjon per teksel i numpy: id-kart, z-buffer, robust vekting, to frekvensbånd,
             screened Poisson på det lave båndet, lysdemping, fliser, normalkart
assemble.py  hodet på kroppen, redskapet ut (eller modellert), ett uv-kart, én tekstur
rig.py       leddene fra silhuettene, skjelettet fra målet, vekter, klipp, glb
finish.py    arkene (Cycles eller punktsplatting), turnaround, glb for ett ark
report.py    rapport.json, tersklene, rettelsene
template.py  malen (grensesnitt og første versjon)
cli.py       byggeløpet
targets/     base.py, generic.py, fristad.py
tests/       regresjonstesten (tre hoder fra bufrede dybdekart)
```

Tester: `python3.13 -m pytest sheet2model/tests` (trenger bufret dybde for smeden, Vertinnen og Pilegrimen, ellers
hoppes de over).
