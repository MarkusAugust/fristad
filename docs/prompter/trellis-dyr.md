# TRELLIS: dyrene

Dyrene som beiter utenfor muren (`domain/herd.rs`, som står og venter på modellene) og dyrene i byen. Sauen og kua først: de står i `model::PLANNED`.

**Slik gjør du det:** lag ETT bilde med prompten, rett fra siden og litt skrått, med alle bein synlige og fra hverandre og halen ut fra kroppen. Kjør bildet gjennom https://huggingface.co/spaces/microsoft/TRELLIS.2 med **Resolution 1536, Decimation Target 500000, Texture Size 4096** (nytt frø om formen eller ansiktet blir dårlig), og last opp glb-fila som `<id>.glb`. Den legges i `assets/textures/animals/3d/<id>.glb`; jeg gjør den spillklar med TRELLIS-teksturen som den er, rigger beina, halsen og halen og lager gange, beiting og hvile, og viser deg den før den tas i bruk.

### 1. `sheep`: Sau

- **Hvor:** jordene ved gårdene, i flokk. **Størrelse:** 0,9 m.
- **Erstatter:** `sheep_body`/`sheep_leg` i `model::PLANNED`.

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A sturdy old Nordic short-tailed sheep: a thick rounded fleece of creamy off-white wool in big soft clumps, a dark grey-brown face and legs, small curled horns, a short tail. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 2. `cow`: Ku

- **Hvor:** jordene. **Størrelse:** 1,3 m.
- **Erstatter:** `cow_body`/`cow_leg_front`/`cow_leg_hind` i `model::PLANNED`.

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A small sturdy old Nordic cow of a red-and-white landrace, a red-brown coat with white patches on the belly and face, short curved horns, a long tail with a tuft held a little out from the legs. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 3. `horse`: Hest

- **Hvor:** ved Vertshuset og kjerrene. **Størrelse:** 1,5 m.
- **Ny.**

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A stocky Nordic fjord horse, dun-coloured with a dark stripe down the back, a short upright two-coloured mane, a thick tail, sturdy legs with a little feathering. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 4. `goat`: Geit

- **Hvor:** ved gårdene. **Størrelse:** 0,8 m.
- **Ny.**

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A Nordic goat with a shaggy grey-and-white coat, backward-curving horns, a small beard, a short upturned tail. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 5. `dog`: Hund

- **Hvor:** ved porten og gårdene. **Størrelse:** 0,6 m.
- **Ny.**

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A Nordic farm dog like a Norwegian buhund: wheaten-yellow coat, pointed upright ears, a curled tail held up over the back. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 6. `hen`: Høne

- **Hvor:** tunene og torget. **Størrelse:** 0,4 m.
- **Ny.**

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A plump speckled brown-and-black Nordic landrace hen with a red comb and wattles, standing on two yellow legs, feathers in a few big smooth clumps. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```

### 7. `cat`: Katt

- **Hvor:** trapper og murer. **Størrelse:** 0,3 m.
- **Ny.**

```
Animal portrait for 3D modelling: ONE single view from the side, turned about 30 degrees toward the viewer, the whole animal centred in the image. Standing still on all four legs: all four legs clearly visible and set apart with background between them, the tail held a little out from the body, head up looking forward, mouth closed, eyes open, nothing overlapping the body. A grey tabby cat with darker stripes and a white chest, standing on all four legs, tail held up and a little out. Every part is bold and chunky enough to read from across a dark hall: no thin spikes, wires, antennae, strands or loose straps; anything long is thick and simple. Style: a high-end stylized 3D game animal with hand-painted textures, like World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated readable shapes, soft painterly shading, NO black outlines, fur and wool painted as a few large smooth clumps, never thin hairs, natural but rich colours, never photorealistic, never a flat cartoon. A farm animal of a Nordic market town around 1350: sturdy old landrace breed, no modern features, no harness, no rope, no bell. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```
