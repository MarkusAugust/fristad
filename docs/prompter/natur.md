# Naturen: trær, busker og steiner

TRELLIS er svak på løv, så trærne lages i to deler: **stammen og greinene** fra TRELLIS (ett bilde av et nakent tre), og **løvet** som bladkort i Blender med de malte løvarkene. Løvarkene for eik, bjørk, gran og buskene og gresset finnes alt (`assets/textures/atlas/`); det som mangler, står i del B. Bjørka er laget (20. oktober, `tools/models/tree3d.py`: TRELLIS ga den med løv, som skilles ut etter fargen og byttes med kort; samme vei virker om TRELLIS legger løv på de andre også).

**Stammene og steinene:** lag ETT bilde med prompten. Kjør bildet gjennom https://huggingface.co/spaces/microsoft/TRELLIS.2 med **Resolution 1536, Decimation Target 500000, Texture Size 4096** (nytt frø om formen eller ansiktet blir dårlig), og last opp glb-fila som `<id>.glb`. Den legges i `assets/textures/nature/3d/<id>.glb`; jeg gjør den spillklar med `item3d.py`, setter løvkortene på greinene og viser deg treet før det settes ut (plan del 12, steg 6).

**Løvarkene og flisene:** lagre som `assets/textures/atlas/<id>.jpeg` (løv på hvit bakgrunn, skjæres ut med `tools/textures/leaf_atlas.py`) eller `assets/textures/painted/<id>/albedo.png` (fliser, avledet med `tools/textures/derive.py` og sjekket med `python3 -m tools.qa texture <id>`).

## A. Stammer og greiner (TRELLIS)

### 1. `tree_oak`: Eik

- **Hvor:** enkeltstående på åsene og ved gårdene. **Størrelse:** 9–12 m.
- **Løv:** `leaves_oak` (finnes).

```
Single game asset for 3D modelling, ONE single view from the front and slightly above, the whole tree centred and fully inside the image from the roots to the tips of the branches. Late winter: NO leaves, NO needles and NO blossoms anywhere, only the bare trunk and the branches. A large old broad oak: a short thick gnarled trunk with deep furrowed grey-brown bark, roots spreading at the base, a few huge crooked limbs spreading wide and low. The branches are thick, smooth and simplified, tapering to blunt ends; only the main limbs and a second order of branches, never a fine web of thin twigs. The bark is painted, not sculpted. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. A tree of the Nordic countryside around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 3. `tree_spruce`: Gran

- **Hvor:** i skogkanten mot kanten av verden. **Størrelse:** 12–15 m.
- **Bar:** `needles_spruce` (finnes).

```
Single game asset for 3D modelling, ONE single view from the front and slightly above, the whole tree centred and fully inside the image from the roots to the tips of the branches. Late winter: NO leaves, NO needles and NO blossoms anywhere, only the bare trunk and the branches. A tall Norway spruce without needles: a straight reddish-brown scaly trunk to the top, many layers of short thick branches all the way up, drooping a little, getting shorter toward the top so the outline is a cone. The branches are thick, smooth and simplified, tapering to blunt ends; only the main limbs and a second order of branches, never a fine web of thin twigs. The bark is painted, not sculpted. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. A tree of the Nordic countryside around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 4. `tree_pine`: Furu

- **Hvor:** på berg og knauser. **Størrelse:** 10–12 m.
- **Bar:** `needles_pine` (del B).

```
Single game asset for 3D modelling, ONE single view from the front and slightly above, the whole tree centred and fully inside the image from the roots to the tips of the branches. Late winter: NO leaves, NO needles and NO blossoms anywhere, only the bare trunk and the branches. A tall Scots pine without needles: a straight trunk, grey plated bark low down turning orange-brown higher up, bare of branches on the lower half, a high flat-topped crown of a few thick crooked limbs. The branches are thick, smooth and simplified, tapering to blunt ends; only the main limbs and a second order of branches, never a fine web of thin twigs. The bark is painted, not sculpted. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. A tree of the Nordic countryside around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 5. `bush_hazel`: Hasselbusk

- **Hvor:** langs gjerder og veier. **Størrelse:** 2–3 m.
- **Løv:** `leaves_bush` (finnes).

```
Single game asset for 3D modelling, ONE single view from the front and slightly above, the whole tree centred and fully inside the image from the roots to the tips of the branches. Late winter: NO leaves, NO needles and NO blossoms anywhere, only the bare trunk and the branches. A bare multi-stemmed hazel bush: eight to ten thick grey-brown stems rising from one root clump and fanning out, a few side branches near the top. The branches are thick, smooth and simplified, tapering to blunt ends; only the main limbs and a second order of branches, never a fine web of thin twigs. The bark is painted, not sculpted. Style: a high-end stylized 3D game asset with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, never photorealistic, never a flat cartoon. A tree of the Nordic countryside around 1350. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

## B. Løv og fliser (bildegeneratoren)

### 8. `needles_pine`: Furubar (løvark)

- **Lagres:** `assets/textures/atlas/needles_pine.jpeg`.
- **Til:** furua (A4), som de andre løvarkene.

```
A sheet of 8 separate tufts of Scots pine needles, each a chunky brush of long blue-green needles at the end of a short orange-brown twig, seen from the side. Hand-painted stylized foliage like World of Warcraft and Fable trees: chunky, clearly grouped clusters with soft painterly shading and a light top-left, NO black outlines. Each cluster stands alone with clear empty space around it, nothing touching another cluster or the image edge, so each can be cut out. Plain flat pure white background, no shadows on the background, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 9. `bark_pine`: Furubark (flis)

- **Lagres:** `assets/textures/painted/bark_pine/albedo.png`.
- **Til:** furua (A4); eik, bjørk og gran har bark fra før.

```
Scots pine bark: thick grey-brown plates with deep cracks between them, turning into thin flaking orange bark, painted as chunky plates. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```

### 10. `moss`: Mose (flis)

- **Lagres:** `assets/textures/painted/moss/albedo.png`.
- **Til:** steinene, røttene og steingjerdene.

```
Soft green moss carpet with small mounds, a few dry brown strands and tiny stones showing, no straight lines. The tile covers 1 by 1 metre. Style: hand-painted stylized game texture in the style of World of Warcraft and Fable environment art. Soft painterly brush strokes, chunky simplified shapes, gentle value variation, NO black outlines and NO dark contour lines around shapes, edges defined only by a slightly darker and slightly lighter painted tone. Muted warm Nordic medieval palette, medium saturation, never neon (except where glowing is asked for). No lighting and no shadows painted in: even flat light, no cast shadows, no ambient occlusion pools, no highlights. Seen perfectly straight-on (orthographic, flat, no perspective). Seamless tileable: the left edge continues the right edge and the top edge continues the bottom edge exactly, with shapes crossing all four edges, and no long straight line that would show as a seam. The whole image is ONE unique tile: never copy, mirror or repeat any part of the pattern inside the image, never a 2x2 grid, and no small spots or specks that would read as noise when repeated at a distance. Even brightness over the whole image, no vignette, no border, no text, no frame. Square image, 2048 by 2048 pixels.
```
