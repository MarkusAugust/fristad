# TRELLIS: gjenstander og pynt

Ting som skal være ekte modeller fra TRELLIS, viktigst først: det spilleren tar i hånda, så pynten i krypten (som i dag er enkle modeller fra byggesettet), så byen.

**Slik gjør du det:** lag ETT bilde med prompten, tre kvart forfra og litt ovenfra, med hele tingen inne i bildet. Kjør bildet gjennom https://huggingface.co/spaces/microsoft/TRELLIS.2 med **Resolution 1536, Decimation Target 500000, Texture Size 4096** (nytt frø om formen eller ansiktet blir dårlig), og last opp glb-fila som `<id>.glb`. Den legges i `assets/textures/items/3d/<id>.glb`, og jeg gjør den spillklar med `tools/models/item3d.py` (tabellen `ITEMS`: høyde, trekanter, TRELLIS-teksturen som den er) og viser deg den før den tas i bruk.

## A. Det du tar i hånda

### 1. `oil_flask`: Lampeolja

- **Hvor:** Apotekeren selger den (`WareKind::Oil`). **Størrelse:** 0,2 m.
- **Ny:** til butikken og kistene.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A small round glazed clay flask with a thick cork, a leather cord tied around the neck, a drip of golden oil at the lip. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 2. `potion`: Helsedrikken

- **Hvor:** kistene og Apotekeren. **Størrelse:** 0,18 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A small squat green glass bottle of deep red herbal tincture, a waxed cork, a blank parchment label tied with string. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 3. `lore_book`: Boka med lappene

- **Hvor:** lappene og bøkene i krypten (`kit_lore`). **Størrelse:** 0,3 m.
- **Erstatter:** `kit_lore` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A worn thick leather-bound book lying half open, loose parchment slips sticking out between the pages (no readable letters), a ribbon bookmark, a small iron clasp. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

## B. Krypten

### 4. `kit_bones`: Beinhaug

- **Hvor:** nivå 1–2, beinhuset. **Størrelse:** 0,6 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A low pile of old yellowed human bones and three skulls, chunky and simplified, a little dust. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 4a. `kit_sarcophagus`: Sarkofagen (to utgaver)

- **Hvor:** gravgangen, kapellet og gravkamrene i alle temaene. **Størrelse:** 2,2 m lang, 1 m høy.
- **Erstatter:** `kit_sarcophagus` fra byggesettet (eieren 20. oktober: «denne må ha en egen 3d modell med tekstur … og vi trenger minst to varianter»). Lagres som `kit_sarcophagus.glb` og `kit_sarcophagus_b.glb`.

Utgave A, ridderen:
```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A heavy medieval stone sarcophagus on a low plinth: a thick rectangular chest of weathered grey limestone with carved arches along its sides, and on the lid a carved effigy of a knight lying with hands folded on a sword on his chest, head on a cushion, feet against a small lion; chipped corners, moss in the cracks. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

Utgave B, munken:
```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A plain old stone sarcophagus of dark rough granite with a heavy slightly pushed-aside lid showing a dark gap, a simple carved cross on the lid, a worn inscription band without letters along the side, iron bands at the ends, cracks and lichen. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 5. `kit_candles`: Lysestake med lys

- **Hvor:** gravkjelleren. **Størrelse:** 0,6 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A tall iron floor candelabrum on three feet with five thick half-melted candles, big drips of wax down the arms. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 6. `kit_throne`: Tronen

- **Hvor:** nivå 8, tronsalen. **Størrelse:** 2,5 m.
- **Erstatter:** `kit_throne` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A heavy carved oak throne on a stone step, a tall back with a golden square grid emblem under a small crown, armrests ending in carved beasts, a faded red cushion, dust. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 7. `kit_statue`: Ridderstatuen

- **Hvor:** nivå 7–8, søylesalene. **Størrelse:** 2,9 m.
- **Erstatter:** `kit_statue` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A stone statue of a medieval knight on a square pedestal, both hands resting on the pommel of a thick sword planted point-down in front, a great helm, weathered grey stone with moss in the cracks. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 8. `kit_armor`: Rustning på stativ

- **Hvor:** nivå 7–8. **Størrelse:** 1,9 m.
- **Erstatter:** `kit_armor` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A suit of mail and plate armour on a plain wooden stand, a kettle helmet on top, a faded red tabard with a simple coat of arms. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 9. `kit_rack`: Våpenstativet

- **Hvor:** nivå 7–8, våpenkammeret. **Størrelse:** 1,8 m bred.
- **Erstatter:** `kit_rack` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A sturdy wooden weapon rack holding three thick spears, two swords and a round shield, iron fittings. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 10. `kit_chandelier`: Hjulkrona

- **Hvor:** nivå 7–8 og kirka. **Størrelse:** 1,6 m bred.
- **Erstatter:** `kit_chandelier` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A medieval wheel chandelier of dark oak and iron with twelve thick candles, hanging from three short thick chains that meet in a ring at the top. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 11. `kit_cart`: Malmvogna

- **Hvor:** nivå 5, gruva. **Størrelse:** 1,2 m.
- **Erstatter:** `kit_cart` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A wooden mine cart with iron bands on four iron wheels, half full of grey ore with copper-green flecks. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 12. `kit_lamp`: Gruvelampa

- **Hvor:** nivå 5. **Størrelse:** 0,4 m.
- **Erstatter:** `kit_lamp` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A small iron miner's oil lamp with a thick hook on top and a warm flame behind a horn window. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 13. `kit_crystal`: Krystallklyngen

- **Hvor:** nivå 5–6. **Størrelse:** 1–1,5 m.
- **Erstatter:** `kit_crystal` fra byggesettet.

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A cluster of thick glowing pale-blue crystals growing from a rock base, big blunt facets, cold light inside them. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 14. `kit_stalagmite`: Dryppstein

- **Hvor:** nivå 6, dryppsteinshallen. **Størrelse:** 1,5–3 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A group of three tall thick cave stalagmites of wet beige-grey limestone with ridged drip layers, on a shared rocky base. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 15. `kit_grate`: Kloakkristen

- **Hvor:** nivå 3–4. **Størrelse:** 1 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A square rusty iron drain grate with thick bars set in a stone frame, green slime along the edges. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

## C. Byen og torget

### 16. `anvil`: Ambolten

- **Hvor:** Smia. **Størrelse:** 0,8 m.
- **Ny** (Smia har i dag bare benken og stativene).

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A heavy dark iron blacksmith's anvil on a thick oak stump, nothing leaning against it. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 17. `cauldron`: Gryta

- **Hvor:** ildstedene og Apoteket. **Størrelse:** 0,7 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A round black iron cauldron on three short legs with a thick curved handle, soot on the sides. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 18. `grindstone`: Slipesteinen

- **Hvor:** ved Smia. **Størrelse:** 1 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A round sandstone grindstone in a sturdy wooden frame with a crank handle and a water trough beneath. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 19. `washtub`: Vaskebaljen

- **Hvor:** ved brønnen. **Størrelse:** 0,5 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A wooden washtub with iron hoops, a washboard leaning inside it and a folded linen cloth over the rim. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 20. `market_goods_bread`: Brød på bodene

- **Hvor:** torget. **Størrelse:** 0,5 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A wooden tray heaped with round dark rye loaves, flat crispbreads and braided buns, flour dust. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 21. `market_goods_fish`: Fisk

- **Hvor:** torget. **Størrelse:** 0,6 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A wooden crate of salted herring with a bundle of dried stockfish lying across the top. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 22. `market_goods_produce`: Grønnsaker i kurver

- **Hvor:** torget. **Størrelse:** 0,5 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. Three woven baskets side by side: one with cabbages, one with turnips and onions, one with red apples. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 23. `market_goods_cheese`: Ost og smør

- **Hvor:** torget. **Størrelse:** 0,4 m.
- **Ny.**

```
Single game asset for 3D modelling, ONE single view in three-quarter from the front-left and slightly above, the whole object centred and fully inside the image, nothing else in the picture. A small wooden board with two round wheels of yellow cheese and a wooden tub of butter. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```
