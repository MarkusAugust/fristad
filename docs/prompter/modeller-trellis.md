# Modellene fra TRELLIS

Alt som skal lages som ekte modeller med TRELLIS.2, i prioritert rekkefølge: det som synes mest i spillet først. Monstrene står for seg i `monstre.md`, innbyggerne i `docs/prompter-folk-3d.md`.

**Slik gjør du det:**

1. Lag bildet med prompten under. Er det et dreieark (dyr), klipper `python3 tools/models/folk_front.py ARK navn` ut visningen forfra.
2. Kjør bildet gjennom https://huggingface.co/spaces/microsoft/TRELLIS.2 med **Resolution 1536, Decimation Target 500000, Texture Size 4096** (nytt frø om formen blir dårlig).
3. Send glb-fila med navnet i overskriften (`tree_oak.glb` osv.). Jeg gjør den spillklar med `tools/models/item3d.py` (gjenstander, natur, pynt) eller `folk3d.py` (personer), innenfor trekantbudsjettet, og viser deg den før den tas i bruk.

Felles for alle: hele motivet inne i bildet med luft rundt, lys grå bakgrunn, ingen skygge på bakken og ingen tekst.

## A. Det som synes mest

Står i byen eller på vei inn, og ses hver gang du spiller.

### 1. `npc_innkeeper`: Vertinnen

- **Hvor:** Vertshuset, bak disken. **Størrelse:** 1,65 m.

Arket står ferdig i `docs/prompter-folk-3d.md`, nr. 2 (vertinnen). Hun er den eneste innbyggeren som mangler glb fra TRELLIS. Lag arket der, klipp ut forfra med `folk_front.py`, kjør TRELLIS og send glb-fila.

### 2. `tree_oak`: Eik

- **Hvor:** enkeltstående på åsene og ved gårdene. **Størrelse:** 9–12 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A large old broad oak tree with a thick gnarled trunk, wide spreading crown of rounded clumps of dark-green leaves, some roots showing at the base, painted bark with deep furrows. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 3. `tree_birch`: Bjørk

- **Hvor:** i små klynger langs veiene. **Størrelse:** 8–10 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A slender Nordic birch tree with a white bark trunk with black markings, light airy crown of small bright-green leaf clumps, slightly leaning. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 4. `tree_spruce`: Gran

- **Hvor:** i skogkanten mot kanten av verden. **Størrelse:** 12–15 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A tall dark Norway spruce, conical shape, drooping layered branches of dense dark-green needles, reddish-brown scaly bark at the base. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 5. `tree_pine`: Furu

- **Hvor:** på berg og knauser. **Størrelse:** 10–12 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A tall Scots pine with a straight orange-brown upper trunk and grey lower bark, a high flat-topped crown of blue-green needle clumps, a few dead lower branches. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 6. `bush_juniper`: Einer

- **Hvor:** i bakkene utenfor muren. **Størrelse:** 1,5–2 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A dense upright juniper bush, dark blue-green spiky foliage in clumps, a few dusty blue berries, woody base. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 7. `bush_hazel`: Hasselbusk

- **Hvor:** langs gjerder og veier. **Størrelse:** 2–3 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A multi-stemmed hazel bush with rounded soft-green leaves in clumps and a few hazelnuts, thin grey-brown stems from the ground. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 8. `rock_big`: Stor stein

- **Hvor:** på enga og ved veiene. **Størrelse:** 1,5–2,5 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A large grey granite boulder with soft rounded edges, patches of yellow-green lichen and moss on top, a few cracks, half buried at the base. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 9. `rock_group`: Steingruppe

- **Hvor:** spredt på enga. **Størrelse:** 0,3–1 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A small group of five grey fieldstones of different sizes lying together, mossy tops, rounded edges. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 10. `grass_tuft`: Gresstue med blomster

- **Hvor:** overalt på enga. **Størrelse:** 0,4 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A single dense tuft of long meadow grass with a few white and yellow wildflowers and purple clover in it, stems standing up and bending outward. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

## B. Dyrene

Beiter utenfor muren (`herd.rs` venter på modellene). TRELLIS gir en stiv form; riggen og gangen legges på av oss (firbeint, som tabellen).

### 11. `sheep`: Sau

- **Hvor:** på jordene ved gårdene. **Størrelse:** 0,9 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A sturdy Nordic short-tailed sheep with thick creamy-white wool in big soft clumps, a dark grey face and legs, small curled horns. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 12. `cow`: Ku

- **Hvor:** på jordene. **Størrelse:** 1,3 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A small old Nordic farm cow, red-brown with white patches, short curved horns, a sturdy body, a bell on a leather collar. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 13. `horse`: Hest

- **Hvor:** ved Vertshuset og kjerrene. **Størrelse:** 1,5 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A sturdy Nordic fjord horse, dun coloured with a dark stripe down the back, a short upright two-tone mane, a simple leather halter. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 14. `goat`: Geit

- **Hvor:** ved gårdene. **Størrelse:** 0,8 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A shaggy grey-brown goat with long backward-curving horns and a beard. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 15. `hen`: Høne

- **Hvor:** på tunene og torget. **Størrelse:** 0,4 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A brown speckled farmyard hen with a red comb, standing. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 16. `dog`: Hund

- **Hvor:** ved porten og gårdene. **Størrelse:** 0,6 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A medium Nordic farm dog, grey and cream wolfish coat, curled tail, pointed ears, alert. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 17. `cat`: Katt

- **Hvor:** på trapper og murer. **Størrelse:** 0,3 m.

```
Animal turnaround sheet for 3D modelling: the same animal three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, camera at body height, the whole animal inside each view with space around it, hooves or feet on the same line, standing square with the legs apart and a clear gap between them, head level, tail hanging free, mouth closed. Soft even light from the front, no cast shadows, no shadow on the ground. Plain flat light-grey background, no scenery, no text. Wide image, 3072 by 2048 pixels, each view as large as possible. A grey tabby cat standing, tail up. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

## C. Byen og torget

Varer, verktøy og småting som gjør gatene levende.

### 18. `market_goods_bread`: Brød og bakst på bodene

- **Hvor:** torget. **Størrelse:** 0,5 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A wooden tray heaped with round dark rye loaves, flat crispbreads and braided buns, flour dust. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 19. `market_goods_fish`: Fisk og tørrfisk

- **Hvor:** torget. **Størrelse:** 0,6 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A wooden crate of salted herring and a bundle of hanging dried stockfish tied with string. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 20. `market_goods_produce`: Grønnsaker i kurver

- **Hvor:** torget. **Størrelse:** 0,5 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. Three woven baskets: one with cabbages, one with turnips and onions, one with red apples. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 21. `market_goods_cheese`: Ost og smør

- **Hvor:** torget. **Størrelse:** 0,4 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A small wooden board with two round wheels of yellow cheese, a wooden tub of butter and a knife. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 22. `haystack`: Høystakk

- **Hvor:** på tunene. **Størrelse:** 3 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A tall haystack built around a wooden pole, golden-brown hay in layers, a little sagging. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 23. `woodpile`: Vedstabel

- **Hvor:** langs husveggene. **Størrelse:** 1,4 m høy, 2 m lang.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A neat stacked woodpile of split birch logs under a small plank roof. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 24. `anvil`: Ambolt

- **Hvor:** i Smia. **Størrelse:** 0,8 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A heavy dark iron blacksmith's anvil on a thick oak stump, a hammer and tongs leaning against it. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 25. `cauldron`: Gryte

- **Hvor:** ved ildsteder og Apoteket. **Størrelse:** 0,7 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A round black iron cauldron on three short legs with a curved handle, soot on the sides. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 26. `grindstone`: Slipestein

- **Hvor:** ved Smia. **Størrelse:** 1 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A round sandstone grindstone in a wooden frame with a crank handle and a water trough beneath. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 27. `washtub`: Vaskebalje

- **Hvor:** ved brønnen. **Størrelse:** 0,5 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A wooden washtub with iron hoops, a washboard leaning in it and some linen. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 28. `sacks`: Sekker

- **Hvor:** ved bodene og Vertshuset. **Størrelse:** 0,8 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A pile of three rough jute grain sacks tied at the top, one slumped open with grain spilling. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 29. `loom`: Vevstol

- **Hvor:** i husene. **Størrelse:** 1,8 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. An upright warp-weighted loom with half-woven striped wool cloth and clay weights hanging at the bottom. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

## D. Krypten

Pynt som i dag er grovt modellert i `tools/crypt/kit_*.py` og blir rikere fra TRELLIS. Byttes én og én; navnet og målet må holdes, så nivåene står som før.

### 30. `kit_statue`: Ridderstatue

- **Hvor:** nivå 7–8, søylesalene. **Størrelse:** 2,9 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A stone statue of a medieval knight standing on a square pedestal, hands resting on the pommel of a long sword planted in front, a great helm, weathered grey stone with moss in the cracks. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 31. `kit_throne`: Tronen

- **Hvor:** nivå 8, tronsalen. **Størrelse:** 2,5 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A heavy carved oak throne on a stone step, a tall back with a golden grid emblem under a small crown, armrests ending in carved beasts, a faded red cushion, dust and cobwebs. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 32. `kit_chandelier`: Hjulkrone

- **Hvor:** nivå 7–8 og kirka. **Størrelse:** 1,6 m bred.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A medieval wheel chandelier of dark oak and iron hanging from chains, with twelve candles, wax drips. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 33. `kit_rack`: Våpenstativ

- **Hvor:** nivå 7–8, våpenkammeret. **Størrelse:** 1,8 m bred.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A wooden weapon rack holding three spears, two swords and a round shield, iron fittings. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 34. `kit_armor`: Rustning på stativ

- **Hvor:** nivå 7–8. **Størrelse:** 1,9 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A full suit of medieval mail and plate armour on a wooden stand, a kettle helmet, a tabard with a faded coat of arms. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 35. `kit_cart`: Malmvogn

- **Hvor:** nivå 5, gruva. **Størrelse:** 1,2 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A wooden mine cart with iron bands on four iron wheels, half full of grey ore with copper flecks. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 36. `kit_lamp`: Gruvelampe

- **Hvor:** nivå 5. **Størrelse:** 0,4 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A small iron miner's oil lamp with a hook and a glowing flame behind a horn window. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 37. `kit_crystal`: Krystallklynge

- **Hvor:** nivå 5–6. **Størrelse:** 1–1,5 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A cluster of tall glowing pale-blue crystals growing from a rock base, sharp facets, cold light inside them. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 38. `kit_stalagmite`: Dryppstein

- **Hvor:** nivå 6, dryppsteinshallen. **Størrelse:** 1,5–3 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A group of three tall cave stalagmites of wet beige-grey limestone with ridged drip layers. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 39. `kit_bones`: Beinhaug

- **Hvor:** nivå 1–2, beinhuset. **Størrelse:** 0,6 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A pile of old human bones and three skulls, yellowed, with a little dust. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 40. `kit_candles`: Lysestake med lys

- **Hvor:** overalt i gravkjelleren. **Størrelse:** 0,6 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A tall iron floor candelabrum with five half-melted candles, wax dripping down. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 41. `kit_grate`: Kloakkrist

- **Hvor:** nivå 3–4. **Størrelse:** 1 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A square rusty iron drain grate set in a stone frame, green slime at the edges. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

## E. Gjenstander

Det du finner og bærer.

### 42. `key`: Nøkkelen

- **Hvor:** eliten mister den. **Størrelse:** 0,15 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A large old iron key with an ornate ring bow shaped like a grid under a small crown, slightly rusty. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 43. `oil_flask`: Lampeolje

- **Hvor:** Apoteket selger den. **Størrelse:** 0,2 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A small round glazed clay flask with a cork, a leather cord around the neck, golden oil visible at the lip. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 44. `potion`: Helsedrikk

- **Hvor:** kister og Apoteket. **Størrelse:** 0,18 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A small green glass bottle of red herbal tincture, waxed cork, a little parchment label with no writing. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

### 45. `lore_book`: Bok med lapper

- **Hvor:** krypten (lappene og bøkene). **Størrelse:** 0,3 m.

```
Single game asset for 3D modelling, shown once in a three-quarter view from the front-left and slightly above, the whole object centred and fully inside the image with clear background space on all sides, nothing cut off at the edges, nothing else in the picture. Soft even light from the front, no cast shadow, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels. Square image, 2048 by 2048 pixels. A worn leather-bound book lying half open with loose handwritten-looking pages sticking out (no readable letters), a ribbon bookmark. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated but natural colours, never photorealistic, never a flat cartoon. Historically plausible Nordic market town around 1350.
```

## F. Sluttbossen

### 46. `framework`: Rammeverket

- **Hvor:** nivå 10, rotunden. **Størrelse:** 4 m.

Står i `docs/prompter/monstre.md` (nr. 27) med dreieark og teksturark. Bruk stillbildet av beistet over byen fra filmen som referansebilde, så det blir den samme skapningen, nå satt sammen med jernsømmer der Gorvek hogg.
