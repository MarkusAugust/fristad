# Filmen: det som mangler

Åpningsfilmen (`tools/film/timeline.py`) har Veo-klipp for elleve innstillinger. Seks står fortsatt som stillbilde med langsom glidning, og to bruker maleriene fra den gamle filmen. Det som mangler, viktigst først:

**Veo-klippene:** last opp bildet under «Første bilde» som første bilde i Veo, lim inn bevegelsen, og last ned klippet som `assets/film/veo/<navn>.mp4`. Er klippet kortere enn innstillingen, saktes det i `cut.py`. Så klipper jeg filmen på nytt (`python3 tools/film/cut.py`) og publiserer den (`sh tools/film/publish.sh`).

**Maleriene** (del B): lim inn prompten i bildegeneratoren, 16:9 så stort som den gir, og lagre som `assets/film/shots/<navn>.png`. Så lager du Veo-klippet fra det som over.

Stilsetningen er den samme i alle maleriene, så filmen er malt med samme hånd.

## A. Veo-klipp fra stillbildene som finnes

### 1. `05_ansgar`: Mester Ansgar med boka i skriptoriet

- **Første bilde:** `assets/film/shots/05_ansgar.png`. **Lengde i filmen:** 9 s.
- **Erstatter:** stillbildet med glidning.

```
Very slow push-in on the old monk. He breathes calmly, lifts his eyes from the open book with a quiet proud smile and looks down again; the pages glow softly and pulse once; dust motes drift slowly in the golden window light. Nothing new appears, no one else enters, the camera never cuts.
```

### 2. `07_raadet`: Rådet og kjøpmennene fra Kontoret

- **Første bilde:** `assets/film/shots/07_raadet.png`. **Lengde i filmen:** 14 s.
- **Erstatter:** stillbildet med glidning.

```
Very slow lateral drift from left to right along the long table. The councillors lean over their ledgers, one taps a rule with a stiff finger and nods, another turns a page; behind them the merchants whisper to each other and one weighs a coin on the hand scale; the blue banners with the seal stir faintly. Cold steady light. Nothing new appears.
```

### 3. `11_inkvisisjonen`: Inkvisisjonen foran skriptoriet

- **Første bilde:** `assets/film/shots/11_inkvisisjonen.png`. **Lengde i filmen:** 9 s.
- **Erstatter:** stillbildet med glidning.

```
Slow push-in toward the line of inquisitors. The book fire in the lower left flares, sparks and burning scraps of paper rise into the purple sky, the torch flickers, the red robes stir in the wind; the inquisitors stand still and stare straight ahead, and the one with the stamp slowly raises it. Nothing new appears.
```

### 4. `13_i_rammen`: Knappen spent fast i rammen

- **Første bilde:** `assets/film/shots/13_i_rammen.png`. **Lengde i filmen:** 5 s.
- **Erstatter:** stillbildet med glidning.

```
Very slow push-in. The glowing orb pulses and strains against the bars, its light squeezing out between them; the chains tighten with a small jolt and the parchment slips on the frame flutter; the firelight from the left flickers. Nothing else moves.
```

### 5. `14_natta`: Skriverne om natta

- **Første bilde:** `assets/film/shots/14_natta.png`. **Lengde i filmen:** 10 s.
- **Erstatter:** stillbildet med glidning.

```
Slow push-in between the desks. The candle flames flicker, the sleeping scribe stirs, one scribe rubs his eyes and goes back to writing, the third pins one more slip of paper over the page; loose slips flutter in a draught. Behind them the chained book stays still. Nothing new appears.
```

### 6. `17_rammeverket`: Rammeverket reiser seg over byen

- **Første bilde:** `assets/film/shots/17_rammeverket.png`. **Lengde i filmen:** 14 s.
- **Erstatter:** stillbildet med glidning.

```
Slow push-in from the hill. Rain streaks down, lightning flashes twice and lights the colossus; it slowly straightens to its full height, turns its crowned head toward the camera and lifts its raised fist higher; its magenta grids pulse brighter with each flash, loose pages tear free from its joints and blow away in the storm, magenta light shimmers on the wet roofs. The town stays still. Nothing new appears.
```

## B. Nye malerier for innstillingene som låner fra den gamle filmen

### 7. `00_doren`: Døra under kirka i dag (åpningen, «Spør noen i Fristad hva som ligger under kirka …»)

- **Lagres:** `assets/film/shots/00_doren.png`, så Veo-klippet `assets/film/veo/00_doren.mp4`.
- **Erstatter maleriet fra den gamle filmen (`opening/begravelsen/bg.webp`).**

Maleriet:

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. A quiet churchyard at night in the present day of the story, generations after the sealing: in the middle of the frame a low stone crypt entrance built into a grassy mound, its arched doorway closed by a heavy old iron-bound oak door, shut and overgrown at the edges with ivy and moss, an old simple cross carved into the lintel, a faint thin magenta glow seeping from the gap under the door. A small Romanesque stone church stands dark on the left, leaning gravestones and a low stone wall, a bare oak, a full moon behind thin cloud, mist along the ground. Cool blue moonlight, the tiny magenta glow the only colour. Still and ominous. No people.
```

Veo:

```
Very slow push-in toward the shut door. Mist drifts along the ground, the ivy stirs in a light wind, the thin magenta glow under the door pulses once like a heartbeat. Nothing else moves, nothing new appears.
```

### 8. `20_doren_mures`: Døra mures igjen («Han kom aldri opp.»)

- **Lagres:** `assets/film/shots/20_doren_mures.png`, så Veo-klippet `assets/film/veo/20_doren_mures.mp4`.
- **Erstatter maleriet fra den gamle filmen (`opening/begravelsen/bg.webp`).**

Maleriet:

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of the crypt doorway at night being sealed: the arched stone opening is almost completely filled with freshly laid grey stones and wet mortar, only a small dark gap left at the top through which a thin warm light still shines, a trowel and a bucket of mortar on the ground, a few loose stones, a simple cross carved into the lintel above. Cool blue moonlight from the left, mist, the grassy mound and ivy around the arch. No people.
```

Veo:

```
Very slow push-in. The thin warm light in the gap at the top flickers and then slowly fades out; mist drifts past; a little mortar trickles down between the new stones. Nothing else moves, no hands, no people appear.
```
