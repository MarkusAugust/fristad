# Ikonene til utstyret (18. oktober)

Ett malt ikon per våpen, skjold, rustning og lykta, til kortene i inventaret (I) og galleriet i Skriptoriet. Alle i samme stil: én ting alene, malt, nøytral varm grå bakgrunn, 512 × 512.

**Slik gjør du det:** lag bildet med prompten under hver ting og lagre det som `assets/textures/icons/<id>.png` (id-en står i overskriften). Send filene her. Til ikonet finnes, viser kortet et rendret bilde av modellen (`python3.13 tools/models/gear_icons.py`, listet i `assets/textures/icons/placeholders.json`); et ikon du har laget, skrives aldri over. `tools/build-web.sh` legger dem i `dist/icons/`.

Stilen (står i hver prompt, så de kan lages hver for seg):

```
Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

## Våpen

### 1. `rusty_sword`: Rustent sverd

- **Hvor:** vanlig · det du starter med.

```
A plain old one-handed arming sword: straight double-edged steel blade with a shallow fuller, nicked edges and patches of orange rust, a simple straight iron crossguard, a brown leather-wrapped grip, a flat round disc pommel. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the point to the upper right, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 2. `component_sword`: Komponentsverd

- **Hvor:** vanlig · Gravkjelleren (button, divider).

```
A clean, well-kept one-handed arming sword: bright straight steel blade with a fuller, a crossguard with small engraved scroll ends, a dark oxblood leather grip bound with silver wire, a round silver pommel set with one small blue stone. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the point to the upper right, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 3. `tag_dagger`: Taggdolk

- **Hvor:** vanlig · Gravkjelleren (tag, button).

```
A short broad dagger: a leaf-shaped steel blade with a central groove and a little rust, a round steel disc guard, a dark wooden grip with two brass rings, a round steel ball pommel. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the point to the upper right, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 4. `card_axe`: Kortøks

- **Hvor:** sjelden · Kloakken (card, button, badge, alert).

```
A one-handed bearded war axe: a broad grey iron head with a long hooked beard and a slightly worn edge, on a gently curved ash-wood haft with a criss-cross leather wrap near the end. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the head at the top, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 5. `alert_mace`: Alertkølle

- **Hvor:** sjelden · Gruva (alert, progress, card, label).

```
A flanged war mace: a dark iron head with six sharp vertical flanges and a spike on top, an iron collar, a dark wooden haft wrapped in crossed leather straps, a small iron ring pommel. A faint warm orange glow in the grooves between the flanges, like a warning lamp. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the head at the top, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 6. `tooltip_stiletto`: Tooltipstikker

- **Hvor:** sjelden · Gruva (tooltip, help-text, tag, label).

```
A slender long dagger: a narrow tapering blade of patterned (damascus) steel with a deep fuller, a gilded bell-shaped guard with curling scroll ornaments, a dark wooden grip carved with interlaced knotwork, a round steel pommel with a deep blue stone. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the point to the upper right, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 7. `divider_blade`: Dividerklinge

- **Hvor:** episk · Borgen under jorda (divider, fieldset, alert, file-upload, search).

```
A splendid long one-handed sword: a dark patterned steel blade with a thin bright line running down its centre like a dividing rule, a wide gilded crossguard with curling arms and three small glowing blue stones, a gold-wrapped grip, a heavy ornate red-and-gold pommel. A soft cold blue shimmer along the edges. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the point to the upper right, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 8. `tabs_axe`: Tabsøks

- **Hvor:** episk · Den korrupte kjernen (tabs, dialog, card, fieldset).

```
A heavy bearded battle axe: a wide head of grey patterned (damascus) steel with a sweeping hooked beard, a few small notches cut into its spine like the tabs of a book's pages, a long ash haft carved with interlaced knotwork and wrapped in dark red leather at the end. Thin violet light seeps from a seam where the head meets the haft. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view with the head at the top, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

## Skjold

### 9. `old_shield`: Gammelt rundskjold

- **Hvor:** vanlig · det du starter med.

```
An old round wooden shield of vertical planks painted in a red and cream swirl of three curved arms, scratched and faded, a dented iron boss in the middle, a rawhide rim. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view seen from the front, slightly turned, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 10. `card_shield`: Kortskjold

- **Hvor:** vanlig · Kloakken (card, badge).

```
A round wooden shield of vertical planks painted in a deep blue and cream swirl of three curved arms, a riveted iron boss, a rim bound in dark blue leather; well kept. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view seen from the front, slightly turned, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 11. `badge_heater`: Badgeskjold

- **Hvor:** sjelden · Gruva (badge, label, card, help-text).

```
A heater shield (flat top, curved sides to a point) with a steel-bound rim, painted royal blue with a rampant golden lion, small dents and scratches. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view seen from the front, slightly turned, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 12. `fieldset_tower`: Fieldset-tårnskjold

- **Hvor:** episk · Borgen under jorda (fieldset, card, label, file-upload).

```
A tall rectangular tower shield of planks painted in vertical red and cream stripes, framed by a riveted iron border with iron corner plates, very heavy. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view seen from the front, slightly turned, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 13. `dialog_heater`: Dialogskjold

- **Hvor:** episk · Den korrupte kjernen (dialog, search, badge).

```
A heater shield with a dark steel-bound rim, painted deep violet with a rampant golden lion; a thin glowing violet seam runs around the inside of the rim like the edge of a window, and the paint has a faint sheen. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view seen from the front, slightly turned, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

## Rustning

### 14. `badge_jerkin`: Badgevams

- **Hvor:** vanlig · Gravkjelleren (badge, tag).

```
A short brown leather jerkin with riveted leather pauldrons, a belt with a brass buckle, and a pair of brown leather bracers lying in front of it. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view on an invisible stand, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 15. `tag_cuirass`: Taggkyrass

- **Hvor:** sjelden · Kloakken (tag, badge, progress).

```
A hardened oxblood-black leather cuirass with rows of small brass studs, shaped shoulder guards, and matching dark leather gauntlets lying in front of it. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view on an invisible stand, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 16. `fieldset_mail`: Fieldset-brynje

- **Hvor:** sjelden · Borgen under jorda (fieldset, badge, tag, file-upload).

```
A knee-length mail hauberk of riveted iron rings with a blue and red cloth surcoat over it, brown leather gloves lying in front of it. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view on an invisible stand, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 17. `search_plate`: Searchharnisk

- **Hvor:** episk · Borgen under jorda (search, fieldset, alert, file-upload, label).

```
A bright steel coat of plates with articulated steel gauntlets and vambraces, rivets and leather straps, polished and heavy. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view on an invisible stand, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

### 18. `dialog_plate`: Dialogharnisk

- **Hvor:** episk · Den korrupte kjernen (dialog, tabs, fieldset, search).

```
A suit of blackened blue-black steel plate with brass rivets and leather straps, articulated gauntlets, and thin violet glowing seams along the plate edges. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view on an invisible stand, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```

## Lys

### 19. `search_lantern`: Søkelykt

- **Hvor:** sjelden · Gruva (search, progress, label, help-text).

```
A belt lantern of dark iron and horn panes with a hinged door and a hook, a bright warm flame inside, and a round polished lens set in the front like a magnifying glass that throws a beam. Style: a single item icon for an inventory in a high-end stylized fantasy game, hand-painted like the item icons in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, soft painterly shading, crisp edges, NO black outlines, rich but not garish colours, never photorealistic, never a flat cartoon. Setting: a Nordic market town around 1350; the item is forged from living pieces of an old book of page patterns, so it may carry small painted page-like details. Composition: the one object alone, centred, filling about 80 % of the square, seen in a clear three-quarter view, soft even light from the upper left, a gentle shadow only on the object itself, no cast shadow on the ground. Plain neutral warm-grey background (#76726A) with a very soft lighter centre, no scenery, no frame, no border, no text, no letters, no runes that read as words, no labels, no hands, no people. Square image, 512 by 512 pixels (or 1024 by 1024, downscaled later).
```
