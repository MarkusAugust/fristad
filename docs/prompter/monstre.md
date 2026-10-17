# Monstrene i den nye malte stilen

Alle monstrene i spillet, også de som finnes, i samme stil, så hele rollelisten henger sammen. Kanon (`docs/kanon.md`): komponenter fra Fristil som vred seg løs fra sidene og ble til noe som kryper; der de ble spent fast i rammeverket, sitter kongens segl. Bygges med TRELLIS (bildet forfra klippes ut med `python3 tools/models/folk_front.py ARK navn`, kjøres i TRELLIS.2 med Resolution 1536, Decimation Target 500000, Texture Size 4096, og `item3d.py`/`monsters.py` gjør formen spillklar og rigger den) eller med `monsters.py` (`project_sheet`) for dem med mange bein. Teksturarket brukes når TRELLIS-teksturen er grumsete eller for å male om deler.

**Slik gjør du det:** lag dreiearket (a) og lagre som `assets/textures/monsters/ref/<id>_turn.png`; lag teksturarket (b) og lagre som `assets/textures/monsters/ref/<id>_tex.png`. For de som finnes, legg ved det gamle arket (`<id>_body.jpeg`) som forbilde, så formen blir den samme. Send filene her.

| Nivå | Størrelse | Kjennetegn |
|---|---|---|
| Trash | kne- til mannshøy | ett hovedmateriale, enkel silhuett, ett øye |
| Elite | mannshøy eller mer | rustning, eget våpen, en glødende jernnøkkel grodd inn i kroppen (bærer nøkkelen) |
| Boss | 3–5 ganger et menneske | flere deler som beveger seg, glødende kjerne |

## Gravkjelleren (nivå 1–2)

Materialet monstrene har grodd sammen med: gravstein, bein, stearinlys, kaldt hvitt lys.

### 1. `button_wight`: Button Wight

- **Komponent:** Button. **Hvor:** nivå 1–2, gravkjelleren. **Størrelse:** 0,9 m. **Status:** finnes (`mutant_button`).
- **I kampen:** trash; lunter krokrygget, trykker seg selv ned og slår.

(a) Dreieark `button_wight_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A hunched little grave-wight whose whole body is a big rounded push button of pale bone-white stone, pressed half down, with a pale rim like a coffin lid around its waist; short bowed legs, long thin grey arms ending in knuckly hands, a single round glowing eye in the middle of the button face, a few melted candle stubs stuck on its shoulders, wax dripping down. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `button_wight_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: pale bone-white stone with fine cracks; grey ash-skin; dripping candle wax with soot; dark iron clamp with a gold grid seal. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 2. `tooltip_sprite`: Tooltip Sprite

- **Komponent:** Tooltip. **Hvor:** nivå 1–2. **Størrelse:** 0,7 m. **Status:** finnes (`mutant_tooltip`).
- **I kampen:** trash; svever og hopper, stikker med halen.

(a) Dreieark `tooltip_sprite_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A small floating ghost-like sprite shaped like a speech bubble of thin pale parchment, with a sharp pointed tail curling down like a stinger, a single eye and a thin crooked mouth, two tiny clawed arms, faint cold blue light inside the parchment. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `tooltip_sprite_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: thin pale parchment with faded ink; cold translucent ghost glow; tiny dark claws; iron clamp with gold grid seal. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 3. `datepicker_priest`: Datepicker-presten

- **Komponent:** Datepicker. **Hvor:** nivå 1, alteret. **Størrelse:** 2,1 m. **Status:** finnes (`datepicker_priest`).
- **I kampen:** elite; går langsomt og rett, slår med staven, roper datoer.

(a) Dreieark `datepicker_priest_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. ELITE TIER: as tall as a human or taller, extra armour plates and fittings, a glowing iron key grown into its body (it carries the key), a unique weapon or tool, bolder accent colour and more glowing seams than the small ones. A tall gaunt priest-wight in a tattered grey-black monk's robe; its head is an open calendar page of yellowed parchment with a grid of day squares and one square burning magenta, a tall iron staff topped with a ring of dangling calendar pages, a heavy iron key grown into its chest, long bony fingers. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `datepicker_priest_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: tattered grey-black wool robe; yellowed calendar parchment with a grid of day squares; dark forged iron; bone. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 4. `modal`: Modalen

- **Komponent:** Modal. **Hvor:** nivå 2, boss i Gravkongens hall. **Størrelse:** 4,5 m. **Status:** finnes (`mutant_modal`).
- **I kampen:** boss; tramper, slår i bakken med lokk-hendene, legger mørke over arenaen.

(a) Dreieark `modal_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. BOSS TIER: huge, three to five times a human, built of several joined parts that move against each other, a glowing exposed core, intimidating and unmistakable from across a hall. A towering box-shaped horror: a giant dialog window made of a heavy stone sarcophagus standing upright, its lid hinged open like a jaw full of grave-candles, two huge stone-slab hands like coffin lids, a dark shroud draped over its back like a backdrop, a small X-shaped iron latch on its brow that glows, the glowing core visible inside the open lid. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `modal_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: grey carved sarcophagus stone with worn reliefs; dark burial shroud cloth; old candle wax and soot; glowing magenta core with seams. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 5. `avatar_skull`: Avatar Skull

- **Komponent:** Avatar. **Hvor:** nivå 2. **Størrelse:** 0,6 m. **Status:** finnes (`avatar_skull`).
- **I kampen:** trash; ruller som et hjul mot deg, hopper opp og biter.

(a) Dreieark `avatar_skull_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A round skull inside a circular iron portrait frame like a medallion, rolling like a wheel; the frame is its body, the skull grins out from the round picture space, a few teeth missing, one glowing eye socket. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `avatar_skull_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: old yellow bone; dark iron medallion frame; faded painted portrait backing; iron clamp with gold seal. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 6. `breadcrumb_crawler`: Breadcrumb Crawler

- **Komponent:** Breadcrumb. **Hvor:** nivå 2. **Størrelse:** 0,5 m høy, 2 m lang. **Status:** finnes (`breadcrumb_crawler`).
- **I kampen:** trash; kryper i en lang kjede etter deg.

(a) Dreieark `breadcrumb_crawler_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A long centipede-like chain of four small parchment-and-stone segments joined by iron chevron-shaped links, each segment on stubby legs, the head segment with a single eye and small mandibles, the links glowing faintly. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `breadcrumb_crawler_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: worn grey stone segments; brown parchment scraps; dark iron chevron links; magenta seams. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 7. `accordion_coffin`: Accordion Coffin

- **Komponent:** Accordion. **Hvor:** nivå 2. **Størrelse:** 0,8 m, 2 m lang. **Status:** finnes (`accordion_coffin`).
- **I kampen:** trash; ligger som en kiste og folder seg ut i bakhold.

(a) Dreieark `accordion_coffin_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A wooden coffin made of folding panels like an accordion, which unfolds into a crawling creature on short spidery legs; the lid is its jaw lined with nail teeth, a glowing eye in the gap between panels. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `accordion_coffin_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: dark weathered coffin wood with iron nails; folding leather hinges; dusty burial cloth; glowing magenta seams. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

## Kloakken og cisternen (nivå 3–4)

Materialet monstrene har grodd sammen med: slim, mose, våt murstein, grønt vann.

### 8. `input_leech`: Input Leech

- **Komponent:** Input. **Hvor:** nivå 3–4. **Størrelse:** 0,4 m høy, 1,2 m lang. **Status:** finnes (`mutant_input`).
- **I kampen:** trash; lusker lavt, biter framover.

(a) Dreieark `input_leech_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A low slimy leech shaped like a long text input field: a flat rounded rectangular body of wet grey-green hide with a single blinking glowing cursor line for an eye at the front, a round sucker mouth with tiny teeth, two thin rusty iron spikes along its back. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `input_leech_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: wet grey-green leech hide with slime; green moss; rusty iron; glowing cursor line. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 9. `toast_frog`: Toast Frog

- **Komponent:** Toast. **Hvor:** nivå 3–4. **Størrelse:** 0,6 m. **Status:** finnes (`mutant_toast`).
- **I kampen:** trash; hopper og spytter.

(a) Dreieark `toast_frog_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A fat toad whose back is a small rounded notification panel of wet stone with a little bell grown onto it, bulging eyes, a wide mouth, strong hind legs, warts that glow magenta. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `toast_frog_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: wet olive toad skin with warts; slimy stone panel; small bronze bell; glowing magenta warts. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 10. `select_broodmother`: Select-moren

- **Komponent:** Select. **Hvor:** nivå 3, elite med nøkkelen. **Størrelse:** 1,6 m høy, 3 m bred. **Status:** finnes (`mutant_select`).
- **I kampen:** elite; kryper på åtte bein, føder tooltips når den slås.

(a) Dreieark `select_broodmother_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. ELITE TIER: as tall as a human or taller, extra armour plates and fittings, a glowing iron key grown into its body (it carries the key), a unique weapon or tool, bolder accent colour and more glowing seams than the small ones. A huge bloated spider-like creature whose abdomen is a dropdown list of stacked stone option slabs hanging open, eight jointed legs of wet iron, a small chevron-shaped head with many eyes, a glowing iron key grown into its back, egg sacs of parchment. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `select_broodmother_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: wet dark spider chitin; stacked grey stone option slabs; rusty jointed iron; parchment egg sacs with glow. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 11. `table`: Tabellen

- **Komponent:** Table. **Hvor:** nivå 4, boss i cisternen. **Størrelse:** 1,8 m høy, 8 m lang. **Status:** finnes (`mutant_table`).
- **I kampen:** boss; kryper, deler seg i rader, setter seg sammen igjen.

(a) Dreieark `table_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. BOSS TIER: huge, three to five times a human, built of several joined parts that move against each other, a glowing exposed core, intimidating and unmistakable from across a hall. A colossal centipede-like creature made of a data table: rows of heavy oak table segments joined by iron brackets, each segment with a pair of carved wooden legs, the head a thick table-top with a grid of carved cells and a glowing core in the middle cell, iron rivets, moss and water stains. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `table_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: dark oak table wood with carved grid cells; iron brackets and rivets; green moss and water stains; glowing magenta core. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

## Gruva og grotta (nivå 5–6)

Materialet monstrene har grodd sammen med: berg, malm, tømmerstiver, blå krystaller.

### 12. `spinner`: Spinneren

- **Komponent:** Spinner. **Hvor:** nivå 5–6. **Størrelse:** 0,8 m. **Status:** ny (i dag `breadcrumb`).
- **I kampen:** trash; ruller og snurrer som et bor, graver seg fram.

(a) Dreieark `spinner_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A round drilling creature like a stone wheel with a ring of iron teeth that spin around its rim like a loading spinner, a gaping mouth in the hub, short stumpy legs, ore chunks stuck in its rim. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `spinner_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: grey ore-veined rock; dark iron teeth with wear; copper ore chunks; glowing magenta hub seam. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 13. `chip_swarm`: Chip-svermen

- **Komponent:** Chip. **Hvor:** nivå 6. **Størrelse:** 5–8 små, 0,3 m hver. **Status:** ny (i dag `tooltip`).
- **I kampen:** trash; svermer og stuper.

(a) Dreieark `chip_swarm_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A swarm of small flying creatures each shaped like a rounded pill-shaped tag chip of polished stone with tiny insect wings and a single eye; show three of them at different angles in each view as one group. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `chip_swarm_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: polished pale stone chip shell; thin veined insect wings; tiny dark legs; magenta glow. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 14. `progress_foreman`: Progress-formannen

- **Komponent:** Progress. **Hvor:** nivå 5, elite med nøkkelen. **Størrelse:** 2,2 m. **Status:** ny (i dag `accordion`).
- **I kampen:** elite; tramper, hugger med hakka, lader opp et slag mens stolpen fylles.

(a) Dreieark `progress_foreman_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. ELITE TIER: as tall as a human or taller, extra armour plates and fittings, a glowing iron key grown into its body (it carries the key), a unique weapon or tool, bolder accent colour and more glowing seams than the small ones. A broad heavy mine foreman built like an ore cart on legs, with a long horizontal progress bar of glass across its chest that fills with glowing magenta light as it charges, a huge iron pickaxe, a miner's lamp on a leather cap, a glowing iron key grown into its belt. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `progress_foreman_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: dark mine timber and leather; rusty riveted iron; glass bar with glowing fill; ore-dusted rock. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 15. `slider`: Slideren

- **Komponent:** Slider. **Hvor:** nivå 5. **Størrelse:** 0,7 m høy, 1,4 m lang. **Status:** ny (i dag `avatar`).
- **I kampen:** trash; glir langs skinnene og veggene.

(a) Dreieark `slider_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A low creature that slides on a rail like a slider control: a long iron track grown into its belly with a round knob-shaped head that moves along it, short claws gripping the rail on both sides, sparks at the contact. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `slider_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: rusty iron rail; worn bronze knob; mine timber; glowing magenta seam. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 16. `dropdown`: Dropdownen

- **Komponent:** Dropdown. **Hvor:** nivå 6, boss i grottekammeret. **Størrelse:** 5 m. **Status:** ny (i dag `select`).
- **I kampen:** boss; tramper og vagger, folder seg ut i lag, slår med mange armer.

(a) Dreieark `dropdown_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. BOSS TIER: huge, three to five times a human, built of several joined parts that move against each other, a glowing exposed core, intimidating and unmistakable from across a hall. A mountainous folding giant: a dropdown menu standing upright, its body a tall stack of hinged stone menu panels that fall open downward like a cascade of layers, each panel with an arm, a chevron-shaped head with a glowing eye, blue crystals growing out of its shoulders, a glowing core behind the top panel. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `dropdown_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: grey cave rock panels with hinges; blue crystals; dark iron hinges; glowing magenta core. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

## Borgen under jorda (nivå 7–8)

Materialet monstrene har grodd sammen med: mørk stein, rust, faner, rustning.

### 17. `card_squire`: Kortknekten

- **Komponent:** Card. **Hvor:** nivå 7–8. **Størrelse:** 1,5 m. **Status:** ny (i dag `button`).
- **I kampen:** trash; går bredbeint bak skjoldet, slår med klubba.

(a) Dreieark `card_squire_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A squat knight-squire whose body is a heavy rectangular card panel like a shield with rounded corners, a small helmeted head peeking over the top edge, short legs and a club; a faded heraldic picture area on the card front. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `card_squire_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: faded painted heraldic card panel; rusty armour iron; worn leather straps; dark wood club. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 18. `breadcrumb_archer`: Breadcrumb-bueskytteren

- **Komponent:** Breadcrumb. **Hvor:** nivå 7–8. **Størrelse:** 1,6 m. **Status:** ny (i dag `breadcrumb`).
- **I kampen:** trash; står på brystvernet og skyter, trekker seg bakover.

(a) Dreieark `breadcrumb_archer_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A thin archer made of a chain of chevron-shaped stone segments stacked as a body, a longbow of dark wood, a quiver of crooked arrows, a narrow helmet with a single glowing eye slit. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `breadcrumb_archer_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: grey stone chevron segments; dark yew bow wood; worn leather quiver; rusty helmet iron. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 19. `tabs`: Fane-vakten

- **Komponent:** Tabs. **Hvor:** nivå 7–8. **Størrelse:** 1,7 m. **Status:** ny (i dag `accordion`).
- **I kampen:** trash; går i flokk, bytter hvilken side som er pansret.

(a) Dreieark `tabs_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A foot soldier whose torso is a row of three hanging cloth banner tabs on an iron rod across its shoulders, one tab always raised like a shield, a spear, a kettle helmet. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `tabs_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: red and blue heraldic wool banner cloth; dark iron rod and helmet; leather; spear wood. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 20. `card_captain`: Kortkapteinen

- **Komponent:** Card. **Hvor:** nivå 7, elite med nøkkelen. **Størrelse:** 2,3 m. **Status:** ny (i dag `modal`).
- **I kampen:** elite; tramper, sveiper med sverdet, skjoldet er et kort.

(a) Dreieark `card_captain_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. ELITE TIER: as tall as a human or taller, extra armour plates and fittings, a glowing iron key grown into its body (it carries the key), a unique weapon or tool, bolder accent colour and more glowing seams than the small ones. A tall armoured captain whose great shield is a heraldic card with rounded corners and a picture area, a long sword, a crested helmet, a cloak of banner cloth, a glowing iron key grown into the shield boss. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `card_captain_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: polished dark plate steel; heraldic painted card shield; red wool cloak; gold trim. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 21. `navbar`: Navbaren

- **Komponent:** Navbar. **Hvor:** nivå 8, boss i tronsalen. **Størrelse:** 5 m. **Status:** ny (i dag `modal`).
- **I kampen:** boss; tramper, drar deg med lenkene, bytter våpen.

(a) Dreieark `navbar_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. BOSS TIER: huge, three to five times a human, built of several joined parts that move against each other, a glowing exposed core, intimidating and unmistakable from across a hall. A gigantic armoured knight-king built from a navigation bar: a long horizontal iron beam across its shoulders with five hanging menu-plaques like heraldic shields on chains, each plaque a different weapon it can swing, a crowned great-helm with a glowing visor, a heavy cloak, the glowing core in its breastplate. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `navbar_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: dark plate steel with gold trim; heraldic plaques; heavy chains; glowing magenta core. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

## Den korrupte kjernen (nivå 9–10)

Materialet monstrene har grodd sammen med: glatt plast, lysende sømmer, pikselkuber, ingen stein igjen.

### 22. `skeleton_loader`: Skeleton Loader

- **Komponent:** Skeleton. **Hvor:** nivå 9–10. **Størrelse:** 1,8 m. **Status:** ny (i dag `breadcrumb`).
- **I kampen:** trash; lusker, flimrer, slår med stavarmene.

(a) Dreieark `skeleton_loader_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A thin flickering figure made of blank grey placeholder bars stacked into the shape of a skeleton: rounded grey bars for bones, a rounded grey block for a skull with no face, a soft shimmer running across it like loading. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `skeleton_loader_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: smooth matt grey plastic bars; pale shimmer gradient; glowing magenta seams; faint pixel noise. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 23. `popover_wisp`: Popover-irrbloss

- **Komponent:** Popover. **Hvor:** nivå 9. **Størrelse:** 0,9 m. **Status:** ny (i dag `tooltip`).
- **I kampen:** trash; svever rundt deg, stikker med pila.

(a) Dreieark `popover_wisp_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A floating rounded panel of smooth glossy plastic with a sharp arrow point underneath like a stinger, one large eye in the panel, a drifting trail of small glowing cubes behind it. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `popover_wisp_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: glossy white plastic; glowing magenta seams; small pixel cubes; dark eye lens. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 24. `grid_warden`: Grid-vokteren

- **Komponent:** Grid. **Hvor:** nivå 9. **Størrelse:** 2,2 m. **Status:** ny (i dag `table`).
- **I kampen:** trash/elite; går stivt, rekker ut rutenettarmer, flytter paneler.

(a) Dreieark `grid_warden_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. ELITE TIER: as tall as a human or taller, extra armour plates and fittings, a glowing iron key grown into its body (it carries the key), a unique weapon or tool, bolder accent colour and more glowing seams than the small ones. A tall symmetrical guardian built from a lattice of smooth plastic grid cells, arms of extending grid rows, a head that is a single square cell with a glowing eye, panels sliding in and out of its body. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `grid_warden_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: smooth matt plastic grid; glowing magenta seams; dark lens; pale core light. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 25. `switch`: Bryteren

- **Komponent:** Switch. **Hvor:** nivå 9–10. **Størrelse:** 1,0 m. **Status:** ny.
- **I kampen:** trash; slår av og på, blir usynlig i av-stillingen.

(a) Dreieark `switch_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. TRASH TIER: knee to waist high, one main material, one simple silhouette, a single eye or lens, no armour. A pill-shaped creature lying on four short legs like a toggle switch, a round knob-head that slides from one end to the other, glowing magenta when on and dull grey when off; show it in the on position. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `switch_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: smooth glossy plastic shell; round knob; glowing magenta track; dark legs. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 26. `twin_warden`: Tvillingvokteren

- **Komponent:** Card + Tabs. **Hvor:** nivå 9, elite med nøkkelen. **Størrelse:** 2,4 m. **Status:** ny.
- **I kampen:** elite; står i to rom samtidig, slår du den ene blinker den andre.

(a) Dreieark `twin_warden_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. ELITE TIER: as tall as a human or taller, extra armour plates and fittings, a glowing iron key grown into its body (it carries the key), a unique weapon or tool, bolder accent colour and more glowing seams than the small ones. A tall guardian made of two mirrored halves joined down the middle by a glowing seam, one half old dark castle stone and armour, the other half smooth white plastic, each half with its own arm and weapon, a split face with one stone eye and one glowing lens, a glowing iron key at the seam. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `twin_warden_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: dark castle stone with armour iron; smooth white plastic; glowing magenta seam; gold key. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```

### 27. `framework`: Rammeverket

- **Komponent:** Rammeverket (kongens komponentbok). **Hvor:** nivå 10, sluttboss. **Størrelse:** 4 m i rotunden. **Status:** finnes uten rigg; bygges på nytt.
- **I kampen:** boss; tramper og vagger, slår i bakken, brøler, kaller på småtroll.

(a) Dreieark `framework_turn.png`:

```
Creature turnaround sheet for 3D modelling: the same creature shown three times side by side at exactly the same scale: front view on the left, side view facing right in the middle, back view on the right. Orthographic, no perspective, camera at mid-body height, the whole creature inside each view with clear background space around it, feet or base on the same line. Neutral standing pose: limbs held clear of the body with background showing between, legs apart, nothing crossing the body in the side view, mouth closed, eyes open. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Wide image, 3072 by 2048 pixels (or the widest the generator allows), each view as large as possible. BOSS TIER: huge, three to five times a human, built of several joined parts that move against each other, a glowing exposed core, intimidating and unmistakable from across a hall. The final boss, the same creature as the colossus that rose over the town in the film, now its heart put back together: a hulking body of rigid square iron frames and glowing magenta grids, a crown of iron on its head, torn pages stuck in its joints, and crude fresh iron seams and clamps where severed arms and legs have been fastened back on (an axe once cut them off); a huge king's seal (a golden grid in a heavy iron frame under a crown) on its chest over the glowing core, chains hanging from its wrists. Use the film still of the colossus as a reference image. Style: a high-end stylized 3D game creature with hand-painted textures, like the creatures in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, NO black outlines, rich saturated colours, appealing and readable from across a dark hall, never photorealistic, never a flat cartoon, never gory. Setting: a crypt under a Nordic market town around 1350; the creature is a living piece of an old book of page patterns that has twisted into something that crawls. Somewhere on its body sits an iron clamp stamped with the king's seal (a stiff golden grid in a heavy iron frame under a small crown) where it was once bolted fast, and thin seams in its body glow magenta.
```

(b) Teksturark `framework_tex.png`:

```
Hand-painted texture sheet for a stylized game creature in the same painterly style: the surface materials of this creature laid out flat as four separate square tiles in a 2 by 2 grid with thin grey gaps between them: dark riveted iron frames; glowing magenta grid cells; crude fresh welds and iron clamps; torn parchment pages. Each tile seen straight on, seamless and tileable on all four edges, no lighting, no shadows and no highlights painted in, no perspective, even colour, crisp painted detail, no black outlines, no text, no letters. Square image, 2048 by 2048 pixels.
```
