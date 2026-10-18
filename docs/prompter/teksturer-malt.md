# Teksturene til den malte stilen

Alle teksturer som trengs for at spillet skal se helhetlig malt ut. Alle de 55 teksturlagene er nå malte (testen `alle_lagene_er_malte`, 17. oktober); dette er det som er underkjent og skal lages på nytt, og det som mangler for krypttemaene, naturen og byen.

**Slik gjør du det:** lag bildet med prompten, sjekk at det er flisbart (ingen synlig skjøt når to ligger ved siden av hverandre) og send det med navnet i overskriften. Det lagres som `assets/textures/painted/<navn>/albedo.png`, og jeg avleder normal, ruhet og høyde med `tools/textures/derive.py` og kjører teksturkontrollen (`python3 -m tools.qa texture <navn>`) før det tas i bruk.

Krav til alle: 2048 × 2048, flisbart, sett rett forfra, uten lys og skygge malt inn, samme palett.

## A. Underkjent, må lages på nytt

Eierens dom i `assets/catalog/approvals.json`; promptene tar opp det som var galt.

### 1. `cobble`: Brostein (torget og Storgata)

*Dom: dårlig horisontal skjøt.*

```
Medieval cobblestone street paving seen from directly above: rounded river cobbles of different sizes, grey with warm brown and faint blue tints, set tightly in packed dark earth, laid in an irregular pattern with no rows and no straight courses at all, so there is no horizontal or vertical line anywhere. The tile covers 1.6 by 1.6 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 2. `stone`: Murstein (husenes grunnmur)

*Dom: skjøten er for dårlig.*

```
Rough rubble masonry wall: irregular grey fieldstones of many sizes fitted together with lime mortar, no regular courses, the stones overlapping the tile edges in all directions so that no seam line can form. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 3. `core_stone`: Kjernestein (nivå 9–10)

*Dom: vertikal skjøt ser ikke bra ut, må tenkes nytt.*

```
Dark castle stone turning into smooth matt plastic: irregular dark grey ashlar blocks whose surfaces are partly melted into smooth seamless grey plastic, with thin cracks between them that glow magenta; the blocks are staggered and of different widths so there is no vertical line through the tile. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 4. `plaster_red`: Rød kalkpuss

*Dom: for mange små flekker, støyende på avstand.*

```
Old lime plaster wall painted with faded red ochre: large soft areas of slightly varying red, a few big gentle patches where the paint is thinner and shows warm beige underneath, very few and only large marks, calm from a distance. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 5. `crypt_plaster`: Kryptpuss

*Dom: stjernetapetet ble et barnerom.*

```
Old cold lime plaster on a crypt wall: pale grey-white plaster with soft damp stains in grey-green, a few large hairline cracks and patches where the plaster has fallen off and shows grey stone behind, no pattern and no decoration. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## B. Krypttemaene

De malte bildene finnes alt (`sewer_brick`, `sewer_floor`, `water_murky`, `mine_rock`, `mine_timber`, `cave_rock`, `crystal`, `keep_ashlar`, `keep_floor`, `core_plastic`, `glitch_seam`) og skal tas inn som lag per tema (plan del 12, steg 5). Det som mangler:

### 6. `sewer_slime`: Slim på kloakkveggen (nivå 3)

```
Wet brick sewer wall covered in a layer of green-brown slime and algae streaks running down, darker wet patches, a few small mushrooms. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 7. `cistern_stone`: Cisternestein (nivå 4)

```
Large smooth Roman-style cistern blocks of pale grey limestone with a dark waterline band across, white lime deposits and green algae below the line. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 8. `mine_floor`: Gruvegulv (nivå 5)

```
Packed dirt mine floor with crushed grey rock, scattered ore chips with copper-green flecks, wheel ruts and footprints, no straight lines. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 9. `cave_floor`: Grottegulv (nivå 6)

```
Wet natural cave floor of smooth flowstone in beige and grey with shallow puddles, small blue crystal shards, no straight lines. The tile covers 2 by 2 metres. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 10. `keep_tiles`: Flislagt gulv i borgen (nivå 7–8)

```
Worn medieval castle floor of square glazed clay tiles in dark red and ochre in a checkered pattern, chipped corners, dust in the joints. The tile covers 2 by 2 metres and holds exactly 8 by 8 tiles so the pattern repeats cleanly. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 11. `banner_cloth`: Fanestoff (borgen og nivå 7–8)

```
Heavy woven wool banner cloth in deep red with a woven border of gold thread along the top and bottom edges of the image, soft folds, faded. The tile covers 1 by 1 metre and repeats only horizontally. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 12. `gilded`: Forgylt kant (tronen, rammene)

```
Gilded carved wood trim: worn gold leaf over carved vine scrolls, dark wood showing where the gold has rubbed off. The tile covers 50 by 50 cm. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 13. `statue_stone`: Statuestein (nivå 7–8)

```
Weathered pale sandstone for statues: smooth carved surface with soft erosion, grey-green lichen patches, darker streaks where rain has run. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 14. `plastic_white`: Hvit plast (nivå 9–10)

```
Smooth matt white plastic surface with very subtle soft panel lines and a faint warm-grey tint, slightly scuffed, clean and uncanny, with a few thin seams that glow faint magenta. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## C. Naturen

Til trærne, buskene og enga (plan del 12, steg 6). Bark finnes (`bark_oak`, `bark_birch`, `bark_spruce`); bladene lages som utklippskort på grønn bakgrunn.

### 15. `leaves_oak`: Eikeløv (kort)

```
Hand-painted foliage card for a stylized game, seen straight on against a plain flat pure green background (#00FF00) with no shadows, the whole shape inside the image with green margin on all sides so it can be cut out cleanly: a dense clump of oak leaves on short twigs, lobed dark-green leaves, a few acorns. Soft painterly brush strokes, chunky simplified leaf clumps, NO black outlines, no lighting painted in, natural Nordic summer greens. Square image, 2048 by 2048 pixels.
```

### 16. `leaves_birch`: Bjørkeløv (kort)

```
Hand-painted foliage card for a stylized game, seen straight on against a plain flat pure green background (#00FF00) with no shadows, the whole shape inside the image with green margin on all sides so it can be cut out cleanly: a hanging spray of small bright-green birch leaves on thin drooping twigs. Soft painterly brush strokes, chunky simplified leaf clumps, NO black outlines, no lighting painted in, natural Nordic summer greens. Square image, 2048 by 2048 pixels.
```

### 17. `needles_spruce`: Grannåler (kort)

```
Hand-painted foliage card for a stylized game, seen straight on against a plain flat pure green background (#00FF00) with no shadows, the whole shape inside the image with green margin on all sides so it can be cut out cleanly: a flat spray of a spruce branch with dense dark-green needles. Soft painterly brush strokes, chunky simplified leaf clumps, NO black outlines, no lighting painted in, natural Nordic summer greens. Square image, 2048 by 2048 pixels.
```

### 18. `needles_pine`: Furunåler (kort)

```
Hand-painted foliage card for a stylized game, seen straight on against a plain flat pure green background (#00FF00) with no shadows, the whole shape inside the image with green margin on all sides so it can be cut out cleanly: a spray of a Scots pine branch with long blue-green needles in tufts. Soft painterly brush strokes, chunky simplified leaf clumps, NO black outlines, no lighting painted in, natural Nordic summer greens. Square image, 2048 by 2048 pixels.
```

### 19. `bark_pine`: Furubark

```
Scots pine bark: thick grey-brown plates low down with deep cracks, turning into thin flaking orange bark, painted with chunky plates. The tile covers 1 by 1 metre and repeats vertically. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 20. `grass_card`: Gress og blomster (kort)

```
Hand-painted foliage card for a stylized game, seen straight on against a plain flat pure green background (#00FF00) with no shadows, the whole shape inside the image with green margin on all sides so it can be cut out cleanly: a wide strip of tall meadow grass blades with white daisies, yellow buttercups and purple clover, the base along the bottom edge. Soft painterly brush strokes, chunky simplified leaf clumps, NO black outlines, no lighting painted in, natural Nordic summer greens. Square image, 2048 by 2048 pixels.
```

### 21. `moss`: Mose

```
Soft green moss carpet with small mounds, a few dry brown strands and tiny stones showing, no straight lines. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## D. Byen

### 22. `awning_striped`: Stripet seil over bodene

```
Woven linen awning cloth with broad faded stripes of madder red and cream, slightly sagging folds, a few stains. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 23. `hay`: Høy

```
Loose golden-brown hay with longer stalks crossing in all directions, a few dry flowers. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```
