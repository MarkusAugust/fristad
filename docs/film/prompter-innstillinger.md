# Promptene til de 24 innstillingene

Planen er `docs/film/plan-aaa.md`. Hver innstilling trenger to bilder fra Gemini og ett dybdekart fra Hugging Face:

- **A: maleriet.** Lim inn prompten under som den står. 16:9, så stort som Gemini gir (2752 × 1536 som før). Lagre som `assets/film/shots/NN_navn/A.png` (eller `.jpg`).
- **B: tomt.** Last opp A i samme samtale og skriv nøyaktig:
  `Edit this image: remove every person and creature completely and paint in what is behind them, so the floor, walls, furniture and light continue naturally. Keep absolutely everything else identical: same framing, same colours, same light, same objects, same brush style.`
  Lagre som `B.png`. Innstillinger uten personer trenger ingen B (merket «ingen B»).
- **Dybde:** last A opp i Hugging Face-spacet *Depth Anything V2* (`depth-anything/Depth-Anything-V2`), last ned dybdebildet i gråtoner i full størrelse, lagre som `depth.png`.
- **Oppskalering:** bare der det står «oppskaler»: kjør A (og B) gjennom et Real-ESRGAN- eller SUPIR-space med 2×, lagre som `A2x.png`/`B2x.png`.

Gjentas en figur i flere bilder (mester Ansgar, herolden, den fremmede), står beskrivelsen av figuren likt i hver prompt. Blir en figur annerledes likevel, last opp det første bildet av figuren i samme samtale og skriv «the same man as in this image» foran prompten.

Stilsetningen er den samme i alle promptene, så hele filmen er malt med samme hånd:

> Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image.

Hver akt har sin palett, som står i promptene: Akt I gyllen morgen mot kjølig blå skygge; Akt II kaldt gråblått med dypblått og gull, så rødt og brannoransje; Akt III magenta og cyan mot svart, så blå natt; Akt IV mørk krypt med magenta glør, så gyllen morgen igjen.

---

## Akt I: Den frie boka

### 01_riket (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. A vast landscape at dawn seen from a hilltop: rolling green hills with a wide river winding through them, and on the hilltops five small walled medieval towns far apart, each different (one of timber, one of pale stone, one with a round tower, one by the river, one on a cliff), each with a tall scriptorium building with lit windows and thin smoke rising. Layered depth: dark foreground grass and a few stones at the bottom edge, mid-ground hills in warm green, far hills in misty blue, a dawn sky with soft clouds lit pink and gold from below. Warm golden light from the right against cool blue shadows. No people.
```

### 02_boka (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of a large, well-worn book lying open on a slanted oak writing desk in a shaft of warm morning sunlight, filling the frame. The two pages show simple, clean, beautiful painted drawings of buttons, checkboxes, switches, cards and small tables in gentle colours, laid out in an orderly grid, with soft glowing light rising from the pages. Beside the book a quill in an inkwell and a few loose sheets. Golden light from the upper left, dust floating in the light, the room behind soft and out of focus. No hands, no people.
```

### 03_skriptoriet_ute (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The front of a tall stone scriptorium in a small Nordic medieval town in warm morning sun, seen straight on from the square: carved stone walls, a big arched wooden door, and in the centre of the facade one large arched window with warm light inside and a glimpse of bookshelves through the glass. Ivy on the walls, a hanging wooden sign with a painted quill, doves on the ledge, a cobbled square in the foreground with long blue shadows, half-timbered houses at the edges, green hills and a dawn sky behind the roof. The whole building is inside the image with air around it. No people.
```

### 04_skriptoriet_inne

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The inside of a bright medieval scriptorium in golden morning light: a long vaulted hall with tall arched windows on the left, shafts of sunlight with floating dust falling across rows of slanted oak writing desks, bookshelves and a wooden gallery with a ladder along the back wall, an iron candle wheel hanging from the ceiling. Four monks in brown habits sit at the desks in the middle distance, calm and absorbed, copying from open books; their pages show simple clean coloured drawings of buttons and cards. Scrolls, inkpots and coloured ink jars on the desks. Warm gold against cool blue shadow, deep and inviting.
```

### 05_ansgar (oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Master Ansgar, an old monk with a long white beard, a bald crown with a ring of white hair, kind deep-set eyes and a lined face, wearing a grey-brown woollen habit with a rope belt and a small brass key on a cord around his neck, stands in the right half of the frame in a sunlit scriptorium, seen from the waist up in three-quarter view, holding a large open book against his chest with both hands and looking down at it with a calm, proud smile. The pages glow softly and show simple clean coloured drawings of buttons, checkboxes and cards. Behind him, out of focus, tall arched windows with golden morning light, bookshelves and writing desks. Warm light from the window on the left side of his face, cool blue in the shadows. Nothing in front of him.
```

## Akt II: Kongens bud

### 06_raadssalen (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The council hall of the king's palace in the capital, empty: a high, cold, perfectly symmetrical stone hall seen straight down its length, tall narrow windows letting in pale grey-blue light, a very long dark table in the middle with high-backed chairs, stacks of ledgers and scrolls, and deep blue banners hanging in a straight row on both walls, each with the king's seal: a rigid golden square grid inside a heavy ornate iron frame with a small crown on top. A dark empty throne at the far end. Cold, orderly and lifeless; grey-blue with dark blue and gold.
```

### 07_raadet

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. In the cold council hall, seven elderly councillors in fur-trimmed dark red and dark blue robes with heavy gold chains of office sit in a row along the far side of a long dark table, facing the viewer, leaning over ledgers and pointing at rules, stern and self-satisfied; none of them holds a quill. Standing behind them to the left, three merchants of the Office in rich brown coats and fur hats with ledgers, keys and a hand scale, whispering. On the wall behind, blue banners with the king's seal: a rigid golden square grid inside a heavy ornate iron frame with a small crown on top. Pale grey-blue window light from the right, long table in the foreground with scrolls and inkpots. Everyone is fully inside the frame.
```

### 08_komponentboken (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of the Royal Component Book lying on a dark iron lectern in the cold council hall: an enormous, heavy book bound in black iron and dark blue leather, closed, wrapped in thick iron chains with a heavy padlock, the king's seal in gold on the cover: a rigid golden square grid inside a heavy ornate iron frame with a small crown on top. Wax seals and stamped slips of parchment stick out between the pages. A faint cold blue glow leaks from between the pages. Grey-blue window light from the right, the hall dark and out of focus behind. No people.
```

### 09_torget

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The market square of a small Nordic medieval town on a grey, overcast day, half-timbered houses around it, a stone well in the middle distance and a church tower behind. On a wooden platform in the right third of the frame stands the king's herald, a young man in a blue tabard with the king's seal (a rigid golden square grid inside a heavy ornate iron frame with a small crown on top), a feathered hat, holding up a long unrolled parchment with a red wax seal, while a trumpeter beside him raises a long trumpet with a blue banner. In the left two thirds, a crowd of townspeople seen from behind and from the side, listening: a smith, a woman with a child, an old man with a stick, a young monk, a baker, their faces worried. Blue banners with the seal have been hung on the houses. Cool grey light, muted colours, the only strong colour the blue and gold of the king.
```

### 10_herolden (oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of the king's herald, a young man in a blue tabard with the king's seal (a rigid golden square grid inside a heavy ornate iron frame with a small crown on top), a feathered hat, seen from the chest up slightly from below, reading loudly from a long unrolled parchment he holds in both hands; the parchment fills the lower left of the frame, with a large red wax seal pressed with the same grid seal hanging from it, the writing on it only suggested as fine lines, not readable. Behind him grey sky and the top of a half-timbered house out of focus. Cool grey light, blue and gold.
```

### 11_inkvisisjonen

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Evening in front of the stone scriptorium of a small Nordic medieval town, the sky dark orange and purple. Five inquisitors in long dark red robes with black hoods and the king's seal (a rigid golden square grid inside a heavy ornate iron frame with a small crown on top) on their chests stand in a line across the middle of the frame facing the viewer, stern faces half in shadow; one holds a heavy chained book, one an iron staff, one a bundle of chains, one a stamp, one a torch. Behind them the scriptorium door stands open with papers spilling out, blue banners with the seal nailed over the windows, an iron cage cart at the left edge. In the lower left a fire of burning books throws warm orange light on the robes against the cold purple evening. Everyone fully inside the frame.
```

### 12_baalet (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of a pile of old books burning on cobblestones at dusk, filling the frame: pages curling in the flames, and on the pages simple clean coloured drawings of buttons, checkboxes and cards blackening and turning to ash, sparks and embers rising into the dark purple evening sky, a torn page with a drawing of a button lying in the foreground just outside the fire. Strong orange firelight from the centre, cold purple shadows. No people.
```

### 13_i_rammen (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of a single simple glowing button, a soft round pale-green orb of light the size of a plate, clamped inside a heavy ornate iron frame shaped like the king's seal: a rigid golden square grid in front of it, a heavy iron frame around it with a small crown on top, thick chains and a padlock holding it, and stamped parchment slips with red wax seals nailed to the frame all around. The orb's light is squeezed between the bars. It sits on a dark stone table in the scriptorium at dusk, the room dark behind, faint orange firelight from the left. No people.
```

## Akt III: Bruddet

### 14_natta

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The scriptorium late at night, lit only by tallow candles: three exhausted scribes in dark habits at their desks in the middle of the frame, one asleep on his book, one rubbing his eyes, one still writing by candlelight, surrounded by towering stacks of paper, loose slips pinned over the pages and hanging from the desks, crossed-out sheets on the floor. Behind them the chained Royal Component Book sits heavy on a lectern, blue banners with the king's seal (a rigid golden square grid inside a heavy ornate iron frame with a small crown on top) on the dark walls, a narrow window showing a black night sky. Warm candlelight pools against deep blue darkness.
```

### 15_lappene (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of a writing desk at night, filling the frame: an open page of the heavy book with a simple drawing of a button in the middle, almost buried under layers of small slips of parchment pinned and glued over it and around it, each with scribbled lines and arrows (only suggested as fine strokes, not readable), red wax seals, crossed-out corrections, a tallow candle burned down to a stub guttering in a brass holder at the right, a quill laid down, an overturned inkpot. Warm candlelight from the right, deep blue shadow. No hands, no people.
```

### 16_bruddet (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The same scriptorium at night in the moment everything breaks: the stone walls and vaults are cracking with glowing magenta and cyan light pouring out of the cracks, desks overturned, hundreds of pages flying through the air, ink spilled across the floor glowing cyan, the chained book on the lectern bursting open with light. Small distorted shapes made of glowing frames, buttons and grids crawl and flee out of the frame into the dark. Harsh magenta and cyan light against black, broken square pixel-like fragments floating in the air. No people.
```

### 17_rammeverket (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. A small walled Nordic medieval town at night in a thunderstorm, seen from a hill outside the walls, rain and a lightning bolt splitting the sky. Rising from the middle of the town, taller than the church tower, a colossal living creature made of iron frames, chains and glowing magenta grids: a hulking armoured body of rigid square grids in heavy iron frames, a crown of iron on its head, torn pages stuck in its joints, glowing magenta light inside it, one arm raised. Magenta light from the creature on the wet roofs against the cold blue of the storm and the white of the lightning. The whole creature is inside the frame with sky above it. No people.
```

## Akt III: Forseglingen

### 18_kirkegaarden

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. A churchyard at night under a full moon with drifting mist: a small Romanesque stone church on the left, old gravestones and a low stone wall, bare oak trees, and on the right a stone crypt entrance built into a grassy mound with a heavy iron-bound door standing open and warm light inside. A procession of six monks in brown habits walks from the left towards the crypt door, carrying lanterns and singing, leading on chains three small distorted creatures made of glowing magenta frames, buttons and grids that shuffle after them. Cool blue moonlight, warm lantern light, mist along the ground. Everyone fully inside the frame.
```

### 19_ansgar_ned (oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Master Ansgar, an old monk with a long white beard, a bald crown with a ring of white hair, kind deep-set eyes and a lined face, wearing a grey-brown woollen habit with a rope belt and a small brass key on a cord around his neck, stands in the open stone doorway of a crypt at night, seen from the front from the knees up, in the right half of the frame, holding a lit lantern in one hand and a thin bundle of loose glowing pages tied with string against his chest with the other, looking back over his shoulder at the viewer with a calm, sad farewell. Dark stone steps lead down behind him into warm light. Cool blue moonlight from the left, warm lantern light on his face, mist at his feet.
```

### 20_doren_mures (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Close view of the crypt doorway at night being sealed: the arched stone opening is almost completely filled with freshly laid grey stones and wet mortar, only a small dark gap left at the top through which a thin warm light still shines, a trowel and a bucket of mortar on the ground, a few loose stones, carved into the lintel above a simple cross. Cool blue moonlight from the left, mist, the stone mound and ivy around the arch. No people.
```

## Akt IV: Nå

### 21_jorda (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. A cross-section straight down through the earth under a churchyard: at the very top a thin strip of grass with gravestones and mist against a dark sky, then dark soil with roots, stones and old bones, then at the bottom a stone vault of an underground crypt corridor seen from the side with a single torch burning, and faint magenta light glowing from a crack deeper down. Cool blue above, warm torchlight and magenta below, dark and quiet. No people.
```

### 22_det_som_sover (ingen B; oppskaler)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. Deep in a dark stone crypt, a vaulted chamber with fallen pillars and bones on the floor, a huge creature of iron frames, chains and square grids lies sleeping curled against the far wall, half buried in dust and rubble, torn pages stuck in its joints, a crown of iron tilted on its head; most of it is dark, but one grid on its chest and one eye glow faintly magenta and the light pulses on the wet stone. In the foreground a skull, a rusted sword and a burnt-down candle. Almost black, with magenta glow and a faint cold blue from a shaft above. No people.
```

### 23_veien

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. A dirt road over green rolling hills in golden morning light leading to a small walled Nordic medieval town in the distance with a stone gatehouse with two round towers, orange roofs and a church tower. In the left third of the frame, close to the viewer, a lone traveller seen from behind walks along the road towards the town: a hooded grey-green cloak, worn and dusty, a walking staff, a leather satchel, a short sword at the belt; no face visible. A weathered wooden signpost with a lantern at the roadside on the far left, meadow flowers and tall grass in the foreground, sheep on a far hill. Warm gold from the right, long cool shadows, soft morning haze over the town.
```

### 24_porten (ingen B)

```
Hand-painted stylized fantasy illustration in the style of painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, Northern Europe around 1350, cinematic composition, high detail, sharp, no text, no letters, no symbols that look like writing, no watermark, no border. Wide 16:9 image. The south gatehouse of a small walled Nordic medieval town seen from the road just outside it in golden morning light: a stone gatehouse with a pointed arch, a raised iron portcullis, two round towers with conical orange roofs, the town wall running off to both sides, a glimpse of a cobbled street and half-timbered houses through the arch, a dirt road and grass in the foreground. Warm gold from the right, cool blue shadows under the arch. No people.
```

(For 24 sender jeg deg et skjermbilde fra spillet ved sørporten å laste opp sammen med prompten, med «paint this view in the style described», så overtoningen til spillet stemmer.)

---

## Manuset til stemmen

Les langsomt, med pauser ved hvert linjeskift; de frittstående linjene («Han kom aldri opp.») får en lang pause foran og bak. Fortelleren er en gammel mann i byen som bryter tausheten og betror seg til den som hører på. Eldre, rolig mannsstemme på bokmål; latinen uttales som kirkelatin (*Omnia lingua Reactia* → «omnia lingva re-aktsia»). ElevenLabs: velg en norsk stemme, lim inn hele teksten under i én fil, last ned som mp3 og legg den i `assets/film/voice/fortelleren.mp3`. Jeg klipper den i replikker etter tidene.

```
[slow, low, intimate, like an old man confiding a secret]
Spør noen i Fristad hva som ligger under kirka, og de skifter emne.

[pause] Jeg skal fortelle deg det likevel.

[warm, unhurried]
Før var riket fullt av byer, og ingen av dem talte likt. Men de delte én bok. En mønsterbok. Du åpnet den, tok det du trengte og lot resten ligge. Den ba ikke om noe. Den sto aldri i veien.

Her holdt mester Ansgar den. [softly] Han sa at en god bok kjennes på det den lar deg slippe.

[colder, measured]
I hovedstaden satt Rådet. Menn som aldri hadde holdt en fjærpenn. Ved siden av dem Kontoret, som visste hva alt kostet. De så ut over riket og fant bare uorden. Så skrev de en bok selv. Tung som en kiste. Lenket som en fange. Kongen satte segl på den.

[solemn, slow, each line separate]
Omnia lingua Reactia. [pause] Alt på hoffets tunge.
Omnia una compage. [pause] Alt med ett rammeverk.
Nulla exceptio. [pause] Ingen avvik.

[grim]
Inkvisisjonen kom i skumringen. Mønsterbøkene brant på torgene. Hver komponent ble spent fast i rammeverket. En knapp kunne ikke lenger bare være en knapp. [dry] Den måtte spørre om lov.

[tired, wry]
Skriverne gjorde det skrivere gjør. De skrev rundt. Omvei på omvei. Lapp på lapp. Til ingen husket hva boka hadde sagt fra først av.

[pause] Og boka ble tyngre.

[long pause] [quiet, ominous] Så kom den store oppgraderingen.

[short, hard sentences]
Én natt. Alt som var bygd, brakk. Komponentene vred seg løs fra sidene og ble til noe som krøp. [rising] Og over dem reiste rammeverket seg selv, høyere enn kirketårnet, og så seg om etter mer å holde.

[pause] Ingenting spurte om lov lenger.

[hushed, reverent]
Munkene her gikk ut med lykter og sang, og lokket skapningene ned i krypten under kirka. Én etter én. Ansgar gikk sist, med de siste sidene under kappen. De murte igjen bak ham.

[long pause] [very quiet] Han kom aldri opp.

[pause] [matter-of-fact, weary]
Det er lenge siden. Byen gjorde det til rykte, og ryktet til eventyr.

[pause] [lowered voice] Men i vinter så klokkeren lys under kryptdøra. [pause] Ingen hadde tent det.

[pause] [gentle, open]
Og på veien sørfra, i morgenlyset, går en fremmed som ikke vet hvor han kommer fra.

[long pause] [quiet, direct] Det er deg.
```

Tittelen «Fristad · Anno 1350» leses ikke; den står i gull på skjermen over det siste bildet før overtoningen til spillet. Taggene i klammer er Eleven v3 sine lydtagger (velg modellen Eleven v3, stabilitet Natural eller Robust).
