# Prompter til åpningsfilmen: male over motoren

Eieren 16. oktober: «Det er et veldig stort gap mellom den visuelle stilen i scenene og selve spillet.» Maleriene males derfor over skjermbilder fra motoren, tatt fra stedet filmen ser dem fra, så maleriet og verden har samme hus og perspektiv (`tools/film/paintover.py`). Hver prompt under er komplett: kopier hele blokka.

## Slik lager du dem

1. Last opp grunnbildet (`assets/film/paintover/base/<navn>.jpg`) i bildegeneratoren som bilde å redigere (ikke bare som stilreferanse).
2. Lim inn hele prompten for scenen.
3. Sjekk at husene, buene, trappene og vinduene står der de sto (legg maleriet over grunnbildet med 50 % gjennomsiktighet).
4. Lagre som `assets/film/paintover/<navn>.png`, 16:9, helst 2560 × 1440.

## 1. `1_skriptoriet` – Skriptoriet i gullalderen (dag)

Grunnbilde: `assets/film/paintover/base/1_skriptoriet.jpg`

```
Repaint this exact image as a hand-painted stylized fantasy illustration. Keep the camera, perspective, horizon and the position and shape of every building, wall, arch, window, stair and piece of furniture exactly as in the image; only repaint the surfaces and add details, light and people. Soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, sharp, high detail, 16:9, no text, no watermark, no border. The scene: the scriptorium in its golden age on a bright morning. Warm sunbeams through the arched windows with dust in the light, the gallery shelves full of leather books, monks in brown habits writing at the desks and reading at the lectern, open illuminated books with glowing golden runes and small floating interface symbols (buttons, checkboxes, sliders) drawn in gold ink, candles lit on the iron candle crown, glass ink jars glowing. Calm, orderly and hopeful.
```

## 2. `2_fallet` – Forretningslogikken lekker inn (samme rom, kveld)

Grunnbilde: `assets/film/paintover/base/2_fallet.jpg`

```
Repaint this exact image as a hand-painted stylized fantasy illustration. Keep the camera, perspective, horizon and the position and shape of every building, wall, arch, window, stair and piece of furniture exactly as in the image; only repaint the surfaces and add details, light and people. Soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, sharp, high detail, 16:9, no text, no watermark, no border. The scene: the same scriptorium at dusk, now in disorder. Papers scattered over the floor, ink spilled across the lectern, cold purple light through the windows, cracks in the stone walls glowing with corrupted magenta and cyan light like digital glitches, broken interface symbols (buttons, dropdowns, checkboxes) escaping from the open books as glowing glass-and-iron trinkets with jagged edges, pixel fragments in the air. Ominous.
```

## 3. `3_kirkegarden` – Begravelsen (natt)

Grunnbilde: `assets/film/paintover/base/3_kirkegarden.jpg`

```
Repaint this exact image as a hand-painted stylized fantasy illustration. Keep the camera, perspective, horizon and the position and shape of every building, wall, arch, window, stair and piece of furniture exactly as in the image; only repaint the surfaces and add details, light and people. Soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, sharp, high detail, 16:9, no text, no watermark, no border. The scene: the churchyard at night under moonlight. The Romanesque stone church on the left and the stone crypt entrance on the right with its door open and torches burning, crooked gravestones and wooden crosses in the grass inside the low stone wall, mist creeping over the ground, the dirt road glistening, a solemn procession of hooded monks with lanterns carrying long wooden coffins bound with iron chains towards the crypt door, faint magenta glow leaking from the coffins. The grass is dark blue-green in the moonlight. Sombre and quiet.
```

## 4. `4_krypten` – Krypten i dag

Grunnbilde: `assets/film/paintover/base/4_krypten.jpg`

```
Repaint this exact image as a hand-painted stylized fantasy illustration. Keep the camera, perspective, horizon and the position and shape of every building, wall, arch, window, stair and piece of furniture exactly as in the image; only repaint the surfaces and add details, light and people. Soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, sharp, high detail, 16:9, no text, no watermark, no border. The scene: the crypt under the church today. The vaulted brick chamber with the iron portcullis in the arch and the torch on the wall, burial niches with skulls and bones in the walls, cobwebs, damp flagstones reflecting the torchlight, cold blue light beyond the portcullis, and behind the bars the silhouettes of monsters made of corrupted interface components (a hunched creature shaped like a rusty push-button, a floating tooltip-shaped sprite) with an eerie magenta glow. Mysterious and dangerous.
```

## 5. `5_torget` – Fristad, år 1350 (dag)

Grunnbilde: `assets/film/paintover/base/5_torget.jpg`

```
Repaint this exact image as a hand-painted stylized fantasy illustration. Keep the camera, perspective, horizon and the position and shape of every building, wall, arch, window, stair and piece of furniture exactly as in the image; only repaint the surfaces and add details, light and people. Soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, sharp, high detail, 16:9, no text, no watermark, no border. The scene: the market square of Fristad in 1350 on a sunny afternoon. The same stall, well, church, gate and half-timbered houses, now full of life: townsfolk in small groups whispering and pointing (a burly blacksmith in a leather apron, an innkeeper woman in a red dress, a merchant in a blue fur-trimmed coat, a maid with a braid, a wide-eyed boy), baskets of apples and bread on the stall, smoke from a chimney, pigeons, flags on the gate towers, soft clouds. Lively and warm.
```

## Første forsøk (16. oktober): generatoren malte nye rom

Bildene ble vakre, men generatoren tegnet egne rom i stedet for å male over grunnbildet: Skriptoriet fikk tre høye vinduer og pulter i rader, torget ble en gate med arkader, og selv i krypten står buen og veggene andre steder. Da kan maleriet ikke ligge på verden. En vanlig bilderedigering med tekst holder ikke formen; generatoren må **styres av formen**:

- **Dybde eller linjer som styring** (ControlNet «Depth» eller «Canny», «Structure reference» eller «Image guidance → Depth/Edge» i verktøy som Krea, Leonardo, Invoke, ComfyUI eller Stable Diffusion/Flux): bruk `assets/film/paintover/control/<navn>_dybde.png` (best) eller `<navn>_linjer.png`, styrke rundt 0,7–0,9, og samme prompt.
- **Bilde-til-bilde med lav endring** (denoise/«strength» rundt 0,45–0,6): grunnbildet inn, samme prompt.

Sjekk alltid med 50 % gjennomsiktighet over grunnbildet før bildet lagres.

---

Grunnbildene lages med `snapshot --film-seq` fra kamerastedene under (verdens koordinater, grader; NATT 1 er natt). Filmen setter maleriene ut på de samme stedene.

| Scene | Øye (x, y, z) | Yaw | Pitch | Synsfelt | Natt |
|---|---|---|---|---|---|
| `1_skriptoriet` | −8,6, 1,8, −11,8 | 25 | 6 | 62 | 0 |
| `2_fallet` | −8,6, 1,8, −11,8 | 25 | 6 | 62 | 1 |
| `3_kirkegarden` | −2, 2,5, −40 | 49 | −4 | 55 | 1 |
| `4_krypten` | −9, −38,3, −14 | 180 | 0 | 55 | 1 |
| `5_torget` | 0, 2, 12 | 0 | −2 | 60 | 0 |
