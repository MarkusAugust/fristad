# Prompter til åpningsfilmen (malte lag)

Filmen bygges av malte 2D-bilder i lag på ulik dybde (`web/film/`, docs/plan.md del 8). Kameraet glir sakte, så lagene forskyves mot hverandre og gir dybde. Hver scene har tre bilder:

- **bakgrunn** (`bg.png`): hele bildet, det som er lengst unna;
- **mellomgrunn** (`mid.png`): det som står midt i bildet, på ensfarget grønn bakgrunn (klippes ut);
- **forgrunn** (`fg.png`): det som er nærmest kameraet, på ensfarget grønn bakgrunn (klippes ut).

Lag alle i 16:9, så stort som bildegeneratoren tillater (helst 2560 × 1440 eller større). Legg dem i `assets/film/opening/<scene>/` med navnene over (scenenavnene står i overskriftene).

**Stilen (lim inn først i hver prompt):**

> Hand-painted stylized fantasy illustration, the same style as a painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, slightly exaggerated proportions, Northern European medieval town around 1350, half-timbered houses and stone, cinematic composition, wide 16:9, high detail, sharp, no text, no watermark, no border.

For **mellomgrunn** og **forgrunn**, legg til på slutten:

> Isolated elements only, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.

---

## 1. `gullalderen` – Skriptoriet i gullalderen

**bg:**
> Interior of a medieval scriptorium in its golden age, tall stone hall with arched windows pouring warm morning sunlight, dust in the light beams, wooden gallery with bookshelves along the back wall, rows of writing desks with open illuminated books, glowing glass jars of ink, hanging iron candle crown, calm and orderly, hopeful atmosphere, seen from the entrance at eye height.

**mid:**
> Three monks in brown habits writing at slanted wooden desks, seen from the side, quills in hand, open books with glowing coloured runes and small floating interface symbols (buttons, checkboxes, sliders) drawn in golden ink above the pages.

**fg:**
> A large slanted lectern in the near foreground with an open illuminated manuscript, a candle and an inkwell, seen close and slightly from below, left side of the image.

## 2. `fallet` – Forretningslogikken lekker inn

**bg:**
> The same medieval scriptorium at dusk, now in disorder: papers scattered, ink spilled, cold purple light through the windows, cracks in the walls glowing with corrupted magenta and cyan light like digital glitches, pixelated fragments floating in the air, ominous atmosphere.

**mid:**
> Swarm of corrupted magical interface symbols escaping from open books: broken buttons, tangled dropdown menus, error-red checkboxes, twisted sliders, rendered as glowing glass-and-iron trinkets with jagged glitchy edges, magenta and cyan glow.

**fg:**
> A toppled ink pot and a torn book page in the near foreground, the ink spreading into a glowing glitchy puddle with square pixel fragments, close-up, bottom left.

## 3. `begravelsen` – Begravelsen under kirka

**bg:**
> Small Romanesque stone church and a stone crypt entrance in a hillside churchyard at night, moonlight, low stone wall, crooked gravestones, a few lit lanterns, mist creeping over the grass, distant town walls with lit windows, sombre and quiet.

**mid:**
> A solemn procession of hooded monks carrying long wooden coffins bound with iron chains, seen from the side walking left to right, lanterns held high, faint magenta glow leaking through the cracks of the coffins.

**fg:**
> Dark silhouettes of old gravestones and a leaning wooden cross with grass in the near foreground, bottom of the image, moonlit edges.

## 4. `krypten` – Krypten i dag

**bg:**
> Deep underground crypt corridor under a church, vaulted brick ceiling, stone walls with burial niches, cold blue light from far away, cobwebs, damp floor reflecting faint light, a heavy iron portcullis at the end, mysterious and dangerous.

**mid:**
> Monsters made of corrupted user interface components lurking in the crypt: a hunched undead creature whose body is a rusty old push-button with a glowing label (Button Wight), a floating small sprite shaped like a tooltip speech bubble with sharp tail, and a towering armoured figure shaped like a dialog window with a heavy frame and a glowing close-X (the Modal), eerie magenta glow.

**fg:**
> Scattered bones, a broken skull and an old rusty sword on stone flagstones in the near foreground, a guttering candle, bottom of the image.

## 5. `fristad1350` – Fristad, år 1350

**bg:**
> The market square of a small Nordic medieval trading town in 1350 on a sunny afternoon, cobblestones, a stone well in the middle, half-timbered houses with steep tiled and thatched roofs around the square, a tavern with a hanging sign, a smithy with smoke, a stone church tower and town gate in the distance, blue sky with soft clouds, lively and warm.

**mid:**
> Townsfolk gathered in small groups whispering and pointing: a burly blacksmith in a leather apron, a stout innkeeper woman in a red dress and apron, a merchant in a blue fur-trimmed coat, a young maid with a braid, a boy listening wide-eyed, seen from the front and side, medieval clothing.

**fg:**
> A market stall corner with baskets of apples and bread, a wooden barrel and a hanging cloth awning edge, near foreground on the right side of the image.

## 6. `sorporten` – Veien mot sørporten

**bg:**
> A dirt road winding over green rolling hills towards the south gate of a walled medieval town, stone gatehouse with two towers and orange conical roofs, town walls, roofs and the church tower behind the walls, late afternoon golden light, long shadows, inviting.

**mid:**
> A lone hooded traveller with a walking staff and a travel bag, seen from behind, walking along the road towards the town, medieval clothing, small in the image.

**fg:**
> Tall grass, wild flowers and a weathered wooden signpost at the roadside in the near foreground, left side of the image.
