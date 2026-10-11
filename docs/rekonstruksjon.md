# Rekonstruksjon fra forbilder (`tools/reconstruct`)

Figurene og tingene lages fra eierens forbildeark: ett bilde med samme figur fra flere kanter (hodet i 2 × 2, kroppen forfra, fra siden og bakfra). Verktøyet gjør det bilde-til-3D-tjenestene gjør, men lokalt og med åpne verktøy.

## Hvorfor

Tidligere forsøk projiserte forbildet på et hode med en annen form enn forbildet viser. Da havnet det innmalte lyset og detaljene feil, og det ble sømmer. Her leses formen fra selve forbildet, så projeksjonen av forbildet stemmer med formen den legges på.

## Hva den gjør

1. **`sheet.py` – visningene.** Arket deles i ruter etter oppsettet (`head2x2`, `body3`, eller rutene i `sheets.json`). Hver rute segmenteres mot den grå bakgrunnen: avstand fra bakgrunnsfargen og metning, største bit, hullene lukkes.
2. **`depth.py` – dybden.** Depth Anything V2 ViT-B som ONNX på CPU (onnxruntime). Vektene (ca. 390 MB) lastes ned første gang fra GitHub-releases (`fabio-sim/Depth-Anything-ONNX`) til `target/reconstruct/` (git-ignorert). Bildet og det speilede bildet tas i snitt.
3. **`fuse.py` – formen.**
   - Visningene rettes inn: ortografisk og vannrett kamera gir samme øverste og nederste punkt i alle visningene, og midten av silhuetten over ørene gir sentrum.
   - Relativ dybde blir metrisk: per rad skal det nærmeste punktet i dybdekartet ligge på ytterkanten av silhuetten fra visningene på tvers (profilene for forfra og bakfra, og omvendt). Skalaen er felles for alle visningene, med egen forskyvning per visning (Huber-vektet minste kvadrat).
   - Volumet (3 px voksler) er minimum over visningene av to ting: signert avstand til silhuetten og avstand bak dybdeflaten. Nær silhuettkanten er dybden upålitelig, så der tar silhuetten over.
   - Valgfritt (`opt.neck` i `sheets.json`): et rent, skrått halskutt og en elliptisk hals bak skjegget der forbildet ikke stemmer med seg selv.
   - Lett gaussisk glatting (0,8 voksel) og marching cubes (skimage).
4. **`texture.py` – teksturen** (Blender som Python-modul).
   - Modellen desimeres til `--tris` (standard 7 000) og får Smart UV med vinkelgrense 40°.
   - Posisjon og normal per teksel bakes med Cycles (EMIT), og normalkartet bakes fra den høye modellen med Blenders tangenter.
   - Hver teksel farges fra visningene: myk dybdetest mot z-bufferen til den høye modellen, vekt etter normalen mot kameraet og mykt mot silhuettkanten.
   - Blandingen skjer i to frekvensbånd: de lave frekvensene blandes bredt (n·c²), detaljene nesten bare fra den beste visningen (n·c⁸). Det gir ingen sømmer og ingen spøkelser.
   - Hull fylles med push-pull. Resultatet er én albedo og ett normalkart på 2048².
5. **`finish.py` – render og eksport.** Lager et ark i samme vinkler som forbildet, ved siden av forbildet, og to mellomvinkler. Lager også en turnaround-GIF og en glb med én tekstur, slanket med `tools/models/slim.py`.

## Bruk

```
pip install -r tools/reconstruct/requirements.txt   # bpy 5.2 for Python 3.13
python3.13 -m tools.reconstruct smith_head          # oppslag i tools/reconstruct/sheets.json
python3.13 -m tools.reconstruct ark.jpeg --layout head2x2 --out modell.glb --tris 7000
python3.13 -m tools.reconstruct smith_head --steps fuse,look   # bare formen, rask kontroll uten Blender
```

Mellomfilene havner i `target/reconstruct/<navn>/`, og arkene i `shots/reconstruct/<navn>/` (eller `--shots`). Kjør det med `flock /tmp/fristad-render.lock` og `timeout` (se `docs/agentregler.md`). Hele løpet tar rundt 7 minutter på 4 kjerner.

`sheets.json` har ett oppslag per forbilde: arket, oppsettet, rutene når de avviker (pilegrimens hodeark), skalaen og avvikene fra `assets/textures/people/ref/README.md`.

## Begrensninger

- **Ingen visning ser ovenfra eller nedenfra.** Toppen av hodet og undersiden av haka får formen fra silhuettene og teksturen fra skrå visninger, så den blir strukket. Et forbilde ovenfra løser det.
- **Forbilder som ikke stemmer med seg selv.** Bildegeneratoren tegner ikke den samme figuren nøyaktig likt i alle rutene: profilene kan være dreid litt, og halsen kan stå forskjellig. Formen blir snittet av visningene, og der de er uenige, blir det flis. Det rettes med `opt` per ark (halsen).
- **Lyset er malt inn.** Teksturen bærer lyset fra forbildet. Det er ønsket i den malte stilen, men gir dobbelt lys om spillet lyser sterkt fra en annen kant.
- **Ting som henger fritt** (en hammer ved siden av kroppen, hender med mellomrom) smelter sammen med det bak om de ikke synes atskilt i minst to visninger.

## Slik vurderer du resultatet

Se på **mellomvinkelen**: 45° mellom forfra og profilen, litt ovenfra (`<navn>_mellom.png`). Ingen av forbildene viser den vinkelen, så den viser om formen og projeksjonen virkelig stemmer, eller om det bare ser riktig ut fra kameraene forbildet ble tatt fra. Se også etter sømmer der to visninger møtes (rundt øret og kinnet), strekk på toppen av hodet, og at silhuetten følger forbildet i arket.

## Hele figurer (`body3`) og montering

- **Arket:** `body3` deler kroppsarket ved mellomrommene mellom figurene (`gap_split`). Bare én profil finnes, så
  den andre siden er profilen speilet. Den brukes til formen og til teksturen på den siden (`mirror_texture`).
- **Dybden:** høye ruter får kvadratiske vinduer med overlapp (`depth.py`), og hver visning får sin egen dybdeskala
  (`scale_per_view`).
- **Formen:** dybdemodellen gir for lite relieff på en hel figur, så tverrsnittene får en superellipse som prior
  (`elliptic`): bredden fra biten i raden forfra og dybden fra biten i raden i profilen. Slakken ved silhuettkanten
  er liten (`edge_slack`).
- **`assemble.py`:**
  - Hodet fra hodearket settes på etter silhuetten. Skalaen finnes ved å passe bredden av skallen forfra og fra
    siden.
  - Kroppen kuttes ved halsen.
  - Redskapet finnes på farge og form og skilles ut.
  - Spøkelset fra den speilede profilen fjernes, og hullene lukkes.
  - Alt bakes til én tekstur på 2048², der hodet får mer plass.
- **`rig.py`:**
  - De 18 beina legges ut fra silhuettene, og vektene er automatiske, med armene og hendene ryddet.
  - Klippene kommer fra `folk_motion.clips`, og redskapet henger stivt på `hand.R`.
  - Eksporten er en glb med skinn og animasjoner, slanket med `slim.py`.

```
python3.13 -m tools.reconstruct smith_head
python3.13 -m tools.reconstruct smith_body                         # seg, depth, fuse, bake, project, render
python3.13 -m tools.reconstruct smith_body --steps assemble,render_full,rig --head smith_head \
    --rig-out assets/models/npc/npc_smith.glb
```

**Begrensninger på hele figurer:**
- Venstre side har ingen egen profil og bygger på den speilede.
- Hender som ligger inntil kroppen smelter sammen med den, så skinnet strekkes litt når armene svinger.
- Leddene legges etter faste høydeandeler (`rig.FRAC`) og bør sjekkes for figurer med andre proporsjoner.
