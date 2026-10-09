# Prompter til de malte teksturene

Til Gemini: lim inn hele blokken som den er. Lagre bildet som `assets/textures/painted/<navn>/albedo.png`, eller bare send det i chatten med navnet, så legger jeg det inn (jeg retter skjøter og gjentakelser med `tools/textures/quilt_seam.py`).

Lyset står oppe til venstre i alle, så de passer sammen. Har bildet blitt fire like kopier, går det fint: jeg klipper ut én.

Rekkefølgen er den jeg trenger dem i. Allerede ferdig: stone, plaster, timber, thatch, cobble, planks, door, door_b, trim.

## Husene: vegger

### 1. `plaster_white`: kalket hvit puss

```
Hand-painted stylized game texture of a medieval whitewashed lime plaster wall, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of smooth hand-applied plaster with gentle unevenness, soft trowel marks, a few thin hairline cracks and a few small chipped spots, evenly spread over the whole image, no large features, no center focus, no stones, no bricks, no wood. Palette: warm off-white and soft cream limewash with faint pale grey and pale ochre patches, never pure white. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 2. `plaster_red`: rødokker kalkpuss

```
Hand-painted stylized game texture of a medieval lime plaster wall washed with faded red ochre, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of plaster with gentle unevenness, soft brush marks from the limewash, a few thin hairline cracks and small chipped spots showing paler plaster underneath, evenly spread over the whole image, no large features, no center focus, no stones, no bricks, no wood. Warm saturated palette: faded brick red, terracotta and dusty rose, with paler worn patches. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 3. `plaster_old`: forvitret puss med flettverk

```
Hand-painted stylized game texture of an old weathered medieval plaster wall where the plaster has fallen off in places, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with patches crossing the edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of cream plaster; 4 to 6 irregular patches of different sizes, spread evenly (none in the center), where the plaster has broken away and shows the woven hazel wattle and brown clay daub underneath, plus thin cracks running from the patches. Warm saturated palette: cream and light ochre plaster, warm brown clay, honey and grey-brown wattle sticks. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 4. `timber_grey`: grå, forvitrede eikebjelker

```
Hand-painted stylized game texture of silver-grey weathered old oak timber beams, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The image shows 4 long horizontal beams stacked from top to bottom, each beam 512 pixels tall and running the full width of the image: the beams have NO ends inside the image, they continue straight out through the left and right edges. Each beam is one solid piece of wood, not split into planks. Deep weathered wood grain running horizontally, a few knots, long drying cracks and worn rounded edges, all different from beam to beam, no nails, no metal, no plaster. Palette: silver grey, ash grey and warm grey-brown, with a little warm honey colour deep in the cracks. Painterly brush strokes, crisp clean edges, a thin dark line between the beams, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 5. `cladding`: loddrett tjærebredd panel

```
Hand-painted stylized game texture of vertical tarred wooden board-and-batten wall cladding, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 8 wide vertical boards running from the top edge to the bottom edge with NO ends inside the image, each joint covered by a narrow vertical batten, wood grain running vertically, a few knots, small cracks and a few square wooden pegs, all boards slightly different in width and colour, no center focus, no metal. Warm palette: dark tar brown, deep chestnut and warm black-brown, with lighter worn honey-coloured edges. Painterly brush strokes, crisp clean edges, a thin dark line beside each batten, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 6. `weatherboard`: liggende kledning

```
Hand-painted stylized game texture of horizontal overlapping medieval weatherboards on a wall, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 8 horizontal boards stacked from top to bottom, each overlapping the board below with a thin shadowless lip, running the full width with NO ends inside the image, hand-split uneven boards with wood grain running horizontally, a few knots and small cracks, all different, no center focus, no metal. Warm saturated palette: warm honey brown, light chestnut and a few silver-grey weathered boards. Painterly brush strokes, crisp clean edges, a thin dark line under the lower edge of each board, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 7. `logs`: laftet tømmervegg

```
Hand-painted stylized game texture of a Nordic log wall made of horizontal round logs, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The image shows 6 round horizontal pine logs stacked from top to bottom, each log about 340 pixels tall, running the full width with NO log ends inside the image, the bark removed, axe marks and long drying cracks along the logs, moss chinking in the narrow joints between the logs, all logs different. Warm saturated palette: warm amber, honey and reddish pine brown, green-brown moss in the joints. Painterly brush strokes, crisp clean edges, a dark line between the logs, each log gently rounded with lighter top, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 8. `wattle`: flettverk

```
Hand-painted stylized game texture of a woven wattle fence of hazel sticks, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre: thin horizontal hazel rods woven in and out around 4 vertical stakes spread evenly across the image, the rods running through the left and right edges, the stakes running through the top and bottom edges, rods of different thickness with a little bark left, no center focus, no background visible between the rods. Warm saturated palette: grey-brown and warm hazel bark, pale honey where the bark has worn off. Painterly brush strokes, crisp clean edges, a thin dark outline around each rod, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## Husene: tak

### 9. `shingles`: takspon

```
Hand-painted stylized game texture of a medieval wooden shingle roof, seen perfectly straight-on onto the roof surface (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shingles crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre: 12 horizontal rows of narrow split wooden shingles with rounded lower ends, each row offset by half a shingle from the row above and overlapping it, shingles slightly different in width and colour, a few cracked or missing corners, evenly spread, no center focus, no moss patches. Warm saturated palette: honey brown, warm chestnut and some silver-grey weathered shingles. Painterly brush strokes, crisp clean edges, a thin dark line under the lower edge of each row, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 10. `shingles_tar`: tjærebredd skjellspon (stavkirka)

```
Hand-painted stylized game texture of a tarred wooden dragon-scale shingle roof of a Norwegian stave church, seen perfectly straight-on onto the roof surface (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shingles crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre: 10 horizontal rows of pointed scale-shaped shingles like dragon scales, each row offset by half a shingle and overlapping the row below, wood grain visible under the tar, a few lighter worn tips, evenly spread, no center focus. Palette: deep tar black-brown, warm dark chestnut and amber glints on the worn edges, never pure black. Painterly brush strokes, crisp clean edges, a thin dark line under the lower edge of each row, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 11. `sod`: torvtak

```
Hand-painted stylized game texture of a Nordic turf roof covered in grass, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of thick turf: dense green and yellow grass in soft clumps, a few small white and yellow wildflowers and a few bare patches of brown earth, all spread evenly with no center focus, no stones, no wood. Warm saturated palette: fresh green, olive, straw yellow and warm brown earth. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 12. `slate`: skifertak

```
Hand-painted stylized game texture of a medieval slate roof, seen perfectly straight-on onto the roof surface (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with slates crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre: 10 horizontal rows of irregular hand-split slates of different widths, each row offset from the row above and overlapping it, a few chipped corners and a little yellow lichen on some slates, evenly spread, no center focus. Palette: blue-grey, dark slate grey, a few rust-brown and green-grey slates, small yellow lichen spots. Painterly brush strokes, crisp clean edges, a thin dark line under the lower edge of each row, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 13. `tiles`: takstein av tegl

```
Hand-painted stylized game texture of a medieval red clay tile roof (monk and nun tiles), seen perfectly straight-on onto the roof surface (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with tiles crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre: 8 vertical rows of curved half-round clay tiles running from top to bottom, overlapping rows of tiles, every tile slightly different in colour, a few chipped tiles, evenly spread, no center focus, no moss patches. Warm saturated palette: brick red, terracotta, burnt orange and a few dark brown tiles. Painterly brush strokes, crisp clean edges, a thin dark line along each overlap, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## Kirka, muren og borgen

### 14. `ashlar`: hogd stein

```
Hand-painted stylized game texture of a medieval cut stone ashlar wall, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with blocks crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 8 horizontal courses of rectangular cut stone blocks, each block 30 to 70 cm wide, the joints of each course offset from the course above, chisel marks, slightly worn edges and a few chipped corners, evenly spread, no center focus. Warm saturated palette: sandy beige, warm grey and pale ochre stone, thin light grey mortar joints. Painterly brush strokes, crisp clean edges, a thin dark outline around each block, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 15. `wall_stone`: bymuren

```
Hand-painted stylized game texture of a thick medieval town wall of rough rubble masonry, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with stones crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 20 to 30 large roughly squared stones laid in uneven horizontal courses, mostly 30 to 60 cm wide, with small wedge stones in the gaps, thick grey mortar, a few dark weathering streaks, evenly spread, no center focus. Palette: cool grey granite, warm grey and a few rust-brown stones, light grey mortar. Painterly brush strokes, crisp clean edges, a thin dark outline around each stone, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 16. `ruin`: forvitret borgmur med mose

```
Hand-painted stylized game texture of an old weathered castle ruin wall with moss, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with stones crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 25 to 35 irregular weathered stones, many with rounded worn edges and a few broken stones, crumbling mortar, green moss and yellow lichen growing on the top edges of stones and in the joints, evenly spread, no center focus, no ivy, no plants hanging down. Palette: warm grey and grey-brown stone, deep green moss, small yellow lichen spots. Painterly brush strokes, crisp clean edges, a thin dark outline around each stone, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 17. `church_boards`: stavkirkas veggplanker

```
Hand-painted stylized game texture of the vertical tongue-and-groove wall planks of a Norwegian stave church, dark with tar, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 7 wide vertical planks running from the top edge to the bottom edge with NO ends inside the image, wood grain running vertically, a few knots and long cracks, all planks slightly different, no center focus, no metal. Palette: deep tar brown, warm chestnut and amber glints where the tar has worn thin, never pure black. Painterly brush strokes, crisp clean edges, a thin dark line between the planks, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## Bakken

### 18. `earth`: tråkket jord på gårdsplassene

```
Hand-painted stylized game texture of trampled earth in a medieval farmyard, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of packed brown earth with scattered pieces of straw, small pebbles, a few hoof prints and footprints and a few tiny grass tufts, all spread evenly with no center focus, no puddles. Warm saturated palette: warm brown, ochre, dusty grey-brown and golden straw. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 19. `mud`: jordvei med hjulspor

```
Hand-painted stylized game texture of a muddy medieval dirt road with wheel ruts, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of road: two parallel wheel ruts run straight from the top edge to the bottom edge, at about one quarter and three quarters of the width, with darker wet mud and small puddles in the ruts, drier packed earth with hoof prints and pebbles between and beside them, a little dry grass at the left and right edges. Warm saturated palette: warm brown, dark chocolate mud, ochre dry earth, a little yellow-green grass. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 20. `gravel`: grus

```
Hand-painted stylized game texture of a gravel path, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre of small gravel stones 1 to 4 cm and a few larger pebbles on packed earth, evenly spread with no center focus, no grass. Warm saturated palette: warm grey, sand beige, ochre and a few rust-red pebbles. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 21. `meadow`: eng

```
Hand-painted stylized game texture of a flowering meadow, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of short meadow grass in soft clumps with scattered small wildflowers (white, yellow, purple) and a few clover patches, evenly spread with no center focus, no bare earth, no paths. Warm saturated palette: fresh green, yellow-green, olive and the small flower colours. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 22. `forest_floor`: skogbunn

```
Hand-painted stylized game texture of a Nordic forest floor, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres of brown spruce needles, a few small cones, twigs, small patches of green moss and blueberry leaves, evenly spread with no center focus, no mushrooms, no large branches. Warm saturated palette: rust brown needles, warm brown, deep green moss and olive leaves. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 23. `rock`: granittberg

```
Hand-painted stylized game texture of a grey granite rock face, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 4 by 4 metres of rough granite cliff with angular facets, cracks running in different directions and small patches of yellow and grey lichen, evenly spread with no center focus, no plants, no sky. Palette: cool grey, warm grey, pinkish granite flecks and small yellow lichen spots. Painterly brush strokes, crisp clean edges, crisp dark lines in the cracks, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## Inne

### 24. `flagstone`: steinheller på gulvet

```
Hand-painted stylized game texture of a medieval stone flag floor, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly, with slabs crossing all four edges. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 2 by 2 metres: 12 to 18 large irregular flat stone slabs of different sizes, worn smooth in places, a few cracks, narrow dark joints with a little dirt, evenly spread, no center focus. Warm saturated palette: warm grey, sandy beige and a few ochre slabs. Painterly brush strokes, crisp clean edges, a thin dark outline around each slab, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 25. `straw_floor`: halm på gulvet

```
Hand-painted stylized game texture of loose straw strewn on a stable or tavern floor, seen perfectly straight down from above (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 1 by 1 metre of loose straw lying in all directions, a few gaps showing dark packed earth underneath, evenly spread with no center focus. Warm saturated palette: golden straw, honey yellow, pale ochre and dark brown earth. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 26. `iron`: smijern

```
Hand-painted stylized game texture of hand-forged black iron plate, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 50 by 50 cm of wrought iron with hammer marks, a few round rivet heads in a loose grid, small dents and a little orange rust in the edges of the dents, evenly spread with no center focus. Palette: blue-black, dark steel grey, lighter grey on the worn highlights, small rust orange spots, never pure black. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 27. `wool`: ullstoff

```
Hand-painted stylized game texture of coarse hand-woven wool cloth, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 20 by 20 cm of coarse wool in a visible tabby weave, slightly uneven threads, a little fuzz, evenly spread with no center focus, no folds, no seams. Warm saturated palette: muted warm grey-brown wool with slightly varying thread colours (to be tinted in game). Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 28. `linen`: linstoff

```
Hand-painted stylized game texture of rough natural linen cloth, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 20 by 20 cm of rough linen in a visible tabby weave, slubby uneven threads, evenly spread with no center focus, no folds, no seams. Palette: warm natural flax, cream and pale beige. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 29. `leather`: lær

```
Hand-painted stylized game texture of worn brown leather, seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable texture: the left edge continues the right edge and the top edge continues the bottom edge exactly. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image. The tile covers 30 by 30 cm of leather with fine grain, a few creases and scratches and slightly lighter worn spots, evenly spread with no center focus, no seams, no stitching. Warm saturated palette: warm chestnut, saddle brown and lighter tan on the worn spots. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). No cast shadows, no ambient occlusion gradients, even brightness across the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

## Ark (ikke flisbare): vinduer, skilt, kirkedør og steinlister

### 30. `windows`: vinduer, fire i ett ark

```
Hand-painted stylized game texture sheet with 4 medieval windows arranged in a 2 by 2 grid, each window filling its quarter of the image on a plain flat dark grey background, not tileable. Top left: a small square window with two open wooden shutters with iron hinges and a horn pane. Top right: a leaded glass window with small diamond panes of slightly green, uneven glass in a wooden frame. Bottom left: a round-arched stone window with a deep stone frame and dark leaded glass. Bottom right: a window closed by wooden shutters with a crossbar. Warm saturated palette: honey and chestnut wood, black iron, warm grey stone, green-tinted glass. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). Every part is seen perfectly straight-on (orthographic, flat, no perspective). No cast shadows, no ambient occlusion gradients, even brightness, no vignette, no text, no letters, no watermark. Square image, 2048 by 2048 pixels.
```

### 31. `signs`: butikkskilt, fire i ett ark

```
Hand-painted stylized game texture sheet with 4 medieval hanging shop signs arranged in a 2 by 2 grid, each sign filling its quarter of the image on a plain flat dark grey background, not tileable, no letters. Each sign is a wooden board with a black iron frame and a painted symbol: top left an anvil and hammer (smithy), top right a frothing wooden tankard (tavern), bottom left a mortar and pestle with herbs (apothecary), bottom right an open book with a quill (scriptorium). Warm saturated palette: honey and chestnut wood, black iron, bright painted symbols in red, gold, green and blue. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). Every part is seen perfectly straight-on (orthographic, flat, no perspective). No cast shadows, no ambient occlusion gradients, even brightness, no vignette, no text, no letters, no watermark. Square image, 2048 by 2048 pixels.
```

### 32. `door_church`: kirkedøra

```
Hand-painted stylized game texture of a tall round-arched door of a Norwegian stave church, the door filling the whole image, not tileable. Vertical tarred oak planks with long decorative black iron hinges ending in curling scroll and dragon-head shapes, a heavy iron ring handle, and a carved wooden portal frame around the door with intertwined Urnes-style dragons and vines. Warm saturated palette: deep tar brown and chestnut wood, black iron, amber highlights on the carvings. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). Every part is seen perfectly straight-on (orthographic, flat, no perspective). No cast shadows, no ambient occlusion gradients, even brightness, no vignette, no text, no letters, no watermark. Square image, 2048 by 2048 pixels.
```

### 33. `trim_stone`: steinlister i ett ark

```
Hand-painted stylized game texture trim sheet of medieval stone mouldings, made of 7 horizontal strips of different heights stacked from top to bottom, each strip running the full width of the image and seamless left to right, separated by thin black lines, not tileable top to bottom. Strips from top: a profiled cornice moulding, a band of carved rope pattern, a row of arch voussoir stones, a plain smooth sill stone, a band of carved interlaced Nordic knotwork, a rough plinth of large blocks, a band of small carved runes-like ornaments (no real letters). Warm saturated palette: sandy beige, warm grey and pale ochre stone. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). Every part is seen perfectly straight-on (orthographic, flat, no perspective). No cast shadows, no ambient occlusion gradients, even brightness, no vignette, no text, no letters, no watermark. Square image, 2048 by 2048 pixels.
```

### 34. `carving`: utskåret drakeportal

```
Hand-painted stylized game texture of a carved wooden panel in Norwegian stave church Urnes style, filling the whole image, seamless left to right only: long intertwined dragons and vines biting each other, deep carved relief, the pattern flowing evenly over the whole image with no center focus. Warm saturated palette: honey and chestnut oak with darker tarred hollows and amber highlights. Painterly brush strokes, crisp clean edges, a gentle highlight on the top-left edges (light from the top-left). Every part is seen perfectly straight-on (orthographic, flat, no perspective). No cast shadows, no ambient occlusion gradients, even brightness, no vignette, no text, no letters, no watermark. Square image, 2048 by 2048 pixels.
```
