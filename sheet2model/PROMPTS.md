# Slik skrives promptene til arkene

Verktøyet er bare så godt som arket. Alt det kan rette, er små avvik: en profil dreid noen grader, en rute tegnet
litt høyere, et redskap i den ene hånda. Det det ikke kan rette, er ark som ikke viser samme figur. Bruk malene
under ordrett og bytt bare ut beskrivelsen av figuren. Promptene er på engelsk fordi bildegeneratorene forstår det
best; alt annet her er på norsk.

Hver figur trenger:

1. et **kroppsark** (`body3`): forfra, fra siden og bakfra på rad,
2. et **hodeark** (`head2x2`): fire ruter,
3. en **rute ovenfra** av hodet (ny, løser den strukne toppen),
4. **redskapet alene**, så det kan modelleres for seg.

Alle fire med samme stilblokk (nederst), så fargene og penselen er like.

## Stilblokken

Lim den inn til slutt i hver prompt. Den første setningen bytter du til stilen i ditt eget prosjekt; resten er
det verktøyet trenger.

```
Style: a high-end stylized 3D game character with hand-painted textures, like the characters in World of
Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, large
hands, strong simple facial features, appealing and believable, never grotesque, never photorealistic, never a
flat cartoon. Hair and beard are sculpted as a few large, smooth, chunky locks with painted strand detail, never
thousands of thin strands. Skin is smooth and softly painted with no pores. Soft painterly shading, NO black
outlines. Flat, even, shadowless light from the front, identical in every view: no cast shadows, no rim light,
no dramatic lighting, no shadow on the background. Plain flat mid-grey background (RGB 130,130,130), no floor,
no ground line, no scenery, no text, no labels, no frame, no guide lines. Orthographic, no perspective. Square
image, 2048 by 2048 pixels.
```

Hvorfor hvert ledd står der:

- *flat, even light, identical in every view*: lyset bakes inn i teksturen. Ulikt lys i rutene blir sømmer.
- *no cast shadows, no shadow on the background*: en skygge på bakgrunnen blir en del av silhuetten.
- *plain flat mid-grey background, no floor, no ground line*: silhuetten finnes som avstand fra bakgrunnsfargen.
  En gulvlinje under føttene må ellers maskes vekk (`floor_line`).
- *no text, no labels, no guide lines*: tekst og hjelpelinjer blir biter av figuren (Apotekerens hodeark hadde to
  vannrette linjer som måtte maskes).
- *orthographic*: formen regnes med parallelle kameraer. Perspektiv gir et for stort hode forfra og for små føtter.
- *NO black outlines*: konturer blir mørke rander i teksturen der visningene møtes.

## Kroppsarket (`body3`)

```
Character turnaround sheet: the same person shown three times side by side at exactly the same scale, standing
straight in an A-pose with the arms held 30 to 40 degrees out from the body so there is clear background between
the arms and the torso, legs slightly apart with clear background between them, hands open and relaxed [or:
the right hand holding a <redskap> loosely, away from the body, the left hand open]. Front view on the left,
side view facing right in the middle, back view on the right, the three figures the same height with the top
of the head and the soles of the feet on the same lines, full body from head to feet with a little space above
and below. <Beskrivelsen av figuren: alder, bygning, ansikt, hår, klær, redskap.>
<Stilblokken>
```

Det vi har lært:

- **Armene 30–40° ut.** Armer inntil kroppen smelter sammen med den i profilen, og hendene blir klumper i
  forkleet. Med luft mellom arm og kropp får armene egen form og egne vekter.
- **Tomme hender, eller redskapet holdt fritt fra kroppen.** Et redskap foran låret i profilen får dybdemodellen
  til å legge det på låret, og det forsvinner eller lager flak. Lange staver (spyd, stav, høygaffel) blir toppen
  av silhuetten og forskyver hele innrettingen: bestill dem **uten** staven i hånda og staven alene (under).
- **Samme skala og samme linjer i alle tre.** Figurer i ulik høyde gir feil dybdeskala. Verktøyet retter noen
  prosent, ikke mer.
- **Profilen vendt mot høyre** (ansiktet mot høyre bildekant). Bare én profil finnes; den andre siden speiles.
  Det som bare finnes på den ene siden (sekk, veske, redskap) oppgis med `tool_box` så det ikke speiles med.
- **Beina litt fra hverandre**, ellers blir de én søyle og knærne kan ikke finnes.

## Hodearket (`head2x2`)

```
Head turnaround sheet of the same person's head and neck, four views in a 2 by 2 grid, all at exactly the same
size with the eyes at the same height in both rows and the top of the head at the same height in all four: top
left the face seen straight from the front; top right the head turned so the face looks towards the RIGHT edge
of the image (we see the person's left cheek); bottom left the head turned so the face looks towards the LEFT
edge of the image (we see the person's right cheek); bottom right the back of the head. True 90-degree profiles
with the far eye hidden. Neutral expression, mouth closed, eyes open looking straight ahead, eyebrows clearly
painted. Each view fills most of its quarter, from the top of the hair to the base of the neck, the neck cut
horizontally at the same height in all four. <Beskrivelsen av hodet.>
<Stilblokken>
```

Det vi har lært:

- **Si eksplisitt hvilken vei profilen vender, og hvilken kinn vi ser.** Ti av tretten hodeark fikk profilene
  byttet om; med feil retning projiseres ansiktet på bakhodet. Sjekk alltid på miniatyrer og skriv `views` i
  oppslaget. En «profil» som er 60° i stedet for 90° gir dobbelt ansikt; innrettingen retter opptil 8°.
- **Øynene i samme høyde i begge rader, issen i samme høyde i alle fire.** Vertinnens ark hadde issen og halsen
  ulikt høyt i rutene; det ga dobbelt ansikt til `align_skin` rettet det.
- **Nøytralt uttrykk, øynene åpne.** Fiskerkona myste nesten med lukkede øyne forfra.
- **Hattebremmer og hetter** skjuler issen og forstyrrer kalibreringen (Pilegrimen: `cal_rows`). Be om at
  bremmen ikke skygger for ansiktet, og bestill ruta ovenfra.
- **Ingen to profiler mot samme side** (Fiskerkona). Da brukes bare én, speilet.

## Ruta ovenfra (ny)

```
The same person's head seen straight from above (top-down orthographic view), the face towards the bottom
edge of the image and the back of the head towards the top, the same size as the head in the front view of the
head sheet, showing the hair parting and the top of the hat/headcloth. <Samme beskrivelse av hodet.>
<Stilblokken>
```

Oppgis som `"extra": {"top": "sti/<navn>_top.png"}` i hodeoppslaget. Uten den fylles toppen med en kuppel og
strekk fra de skrå visningene (eller fliser). Med den får toppen både form (silhuett og dybde ovenfra) og tekstur.
Ruta må vise bakhodet opp i bildet; innrettingen finner forskyvning og skala, men ikke en dreining ovenfra.

## Redskapet alene

```
A single <redskap: forging hammer / wooden tankard / bearded axe / pilgrim's staff> lying flat, seen from the
side with the handle horizontal, and the same object seen from the front end, side by side at the same scale,
nothing else in the image. <Materialer og farger.>
<Stilblokken>
```

Redskapet i hånda på kroppsarket er 30–100 px bredt og kan ikke få volum derfra (hammeren ble en skive, øksa
forsvant i låret). Derfor finnes det i kroppsarket bare for å få plasseringen og grepet (`tool_side`,
`tool_box`), og selve redskapet modelleres for seg (`model_tool`, i dag en enkel hammer; et eget ark gir form og
tekstur til flere).

## Gjenstander: `prop2`-arket (heltegjenstander)

Kista, tønna, brønnen, ambolten, hvilestedet og alt annet spilleren står og ser på, får to visninger:

```
Prop turnaround sheet of the same object shown twice side by side at exactly the same scale: on the left the
object seen from the front-left three-quarter view, slightly from above (the camera about 20 degrees above
the object), and on the right the same object seen from the back-right three-quarter view from the same height,
so that between the two views every side of the object is visible. The object stands on nothing: no floor, no
ground shadow, no contact shadow, plain flat mid-grey background. Both views the same height, centred in their
half. <Beskrivelsen av gjenstanden: materialer, slitasje, beslag, hva som er malt på.>
<Stilblokken>
```

Oppslaget: `"layout": "prop2"`, `"opt": {"prior": "box" | "cylinder" | "barrel", "view_pitch": 20}`. Prioren
er den grove formen verktøyet passer til silhuetten; målene finnes selv. `tiles: "auto"` legger den malte flisen
som ligner mest under gjenstanden (ingen ser undersiden i arket).

Det vi har lært: **ingen gulvskygge** (den blir en del av silhuetten og gjør gjenstanden høyere), **samme
høyde på kameraet i begge rutene** (vippen finnes, men bare én felles), og **la gjenstanden stå fritt** (en tønne
på en bukk er to gjenstander: bestill dem hver for seg, eller bruk `erase`).

## Gjenstander: `prop1` og 3 × 3-arkene (småting)

Småting ses i forbifarten og bygges fra én 3/4-visning. Arkene med ni gjenstander i et 3 × 3-rutenett
(`furniture_b.jpeg`, `clutter_*.jpeg`) brukes rett: oppgi rutenettet og ruta i oppslaget, så deles arket der.

```json
{"ref": "assets/textures/props/ref/furniture_b.jpeg", "layout": "prop1", "grid": [3, 3],
 "views": {"q_front_left": [0, 0]}, "scale": 0.0012, "opt": {"prior": "box", "view_pitch": 20, "tiles": "auto"}}
```

`grid` er rader og kolonner i arket, `views` ruta (rad, kolonne fra 0) og hvilken vei den vender
(`q_front_left`: vi ser forsiden og venstre side; `q_front_right` når gjenstanden vender den andre veien).
Prompten for et nytt 3 × 3-ark:

```
A 3 by 3 grid of nine separate <medieval household / market / dungeon> objects, each one alone in its cell,
all seen from the same front-left three-quarter angle slightly from above, all at a similar size filling most
of the cell, no two objects touching, nothing crossing a cell border, no floor, no ground shadow, no contact
shadow, no labels. Row 1: <tre gjenstander>. Row 2: <tre>. Row 3: <tre>.
<Stilblokken>
```

Baksiden lages ved speiling (gjenstander er nesten alltid symmetriske om midten), så det som bare finnes på
én side (en lås på forsiden) havner også på baksiden; bestill `prop2` når det ikke er greit.

## Sjekkliste før et ark brukes

Se på arket i miniatyr (hele arket på 1 024 px) og i full størrelse rundt hendene og halsen.

- [ ] Samme figur i alle rutene: samme klær, samme hår, samme skjegg, samme hodeplagg, samme redskap i samme hånd.
- [ ] Samme skala: issen og føttene (kroppen) eller issen og halskuttet (hodet) på samme linjer i alle rutene.
- [ ] Profilene vender riktig vei, og de er ekte 90°-profiler (det bakre øyet er skjult).
- [ ] Armene står ut fra kroppen med bakgrunn imellom; beina står fra hverandre.
- [ ] Ingen tekst, ingen rammer, ingen hjelpelinjer, ingen gulvlinje, ingen skygge på bakgrunnen.
- [ ] Bakgrunnen er jevnt grå og figuren rører ikke kanten av ruta.
- [ ] Ingen ekstra deler (en hånd som svever ved siden av, Apotekeren), ingen staver som stikker ut av ruta.
- [ ] Lyset er likt og flatt i alle rutene (ingen rute mørkere eller med skygge under haka).
- [ ] Hodearket: nøytralt uttrykk, øynene åpne, munnen lukket, øynene i samme høyde.

Feiler ett punkt, er det billigere å generere arket på nytt enn å rette modellen etterpå. Avvik du likevel
godtar, skriver du i oppslaget (`merk`) og retter med `opt` (`erase`, `tool_box`, `drop_pole`, `cal_rows`,
`views`).

## Hva som gikk galt hos oss (så du slipper det)

| Figur | Hva | Hvorfor |
|---|---|---|
| Pilegrimen, Vaktmannen, Bonden | To kropper oppå hverandre | Staven/spydet/høygaffelen ble toppen av silhuetten i én rute; innrettingen etter topp og bunn ble feil. Nå: `drop_pole`, og staven bestilles for seg. |
| Pilegrimen | Negativ dybdeskala | Hattebremmen øverst styrte kalibreringen. Nå: `cal_rows`. |
| Fiskerkona, Vertinnen | Dobbelt ansikt | Profilene tegnet i annen høyde/dreining enn forfra; to profiler mot samme side. Nå: innretting med yaw og hudlinjer, én profil speilet. |
| Vertinnen, Gorvek | Krus/øks i begge hender | Den manglende profilen ble speilet med redskapet i. Nå: `tool_box` tar det ut før speilingen. |
| Ti hodeark | Ansiktet på bakhodet | Profilene byttet om i forhold til prompten. Nå: `views` sjekkes på miniatyrer. |
| Apotekeren | Striper i hodet, en ekstra hånd | Hjelpelinjer i arket, en hånd som svevde ved siden av. Nå: `erase`, største bit. |
| Alle | Strukket topp, klumpete hender | Ingen rute ovenfra; 100 px over hånda. Nå: ruta ovenfra; hender fra malen senere. |
