# Prompter til åpningsfilmen: male over motoren

Eieren 16. oktober: «Det er et veldig stort gap mellom den visuelle stilen i scenene og selve spillet.» Maleriene fra første runde (`docs/prompter-film.md`) er malt fritt, så husene, rommene og perspektivet i dem finnes ikke i byen. Kameraet kan derfor aldri fly fra verden og inn i maleriet uten at det blir to bilder oppå hverandre.

Nå males maleriene **over skjermbilder fra motoren**, tatt fra nøyaktig det stedet og med det synsfeltet filmen bruker. Da har maleriet og verden samme hus, samme vinduer og samme perspektiv. I filmen males verden over med pensel mens kameraet flyr inn (`Frame.paint` gjør motorens bilde malt, og maleriet legger seg over på de samme flatene).

## Slik lager du dem

1. Last opp grunnbildet (`assets/film/paintover/base/<scene>.jpg`) i bildegeneratoren, som bilde å redigere eller som strukturreferanse med høy styrke (ikke bare som stilreferanse).
2. Lim inn stilen og så scenens prompt.
3. **Perspektivet, horisonten og plasseringen av husene, buene, trappene og vinduene skal være de samme som i grunnbildet.** Legg på detaljer, lys, folk og stemning, men ikke flytt noe som står der. Legg bildene over hverandre (50 % gjennomsiktighet) og se at kantene treffer.
4. Lagre som `assets/film/paintover/<scene>.png`, 16:9, helst 2560 × 1440. Ett bilde per scene, ingen lag.

**Stilen (lim inn først i hver prompt):**

> Repaint this exact image as a hand-painted stylized fantasy illustration. Keep the camera, perspective, horizon and the position and shape of every building, wall, arch, window, stair and piece of furniture exactly as in the image; only repaint the surfaces and add details, light and people. Soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, sharp, high detail, no text, no watermark, no border.

---

## 1. `1_skriptoriet` – Skriptoriet i gullalderen (dag)

> The scriptorium in its golden age on a bright morning: warm sunbeams through the arched windows with dust in the light, the gallery shelves full of leather books, monks in brown habits writing at the desks and reading at the lectern, open illuminated books with glowing golden runes and small floating interface symbols (buttons, checkboxes, sliders) drawn in gold ink, candles lit on the iron candle crown, glass ink jars glowing, calm, orderly and hopeful.

## 2. `2_fallet` – Forretningslogikken lekker inn (samme rom, kveld)

> The same scriptorium at dusk, now in disorder: papers scattered over the floor, ink spilled across the lectern, cold purple light through the windows, cracks in the stone walls glowing with corrupted magenta and cyan light like digital glitches, broken interface symbols (buttons, dropdowns, checkboxes) escaping from the open books as glowing glass-and-iron trinkets with jagged edges, pixel fragments in the air, ominous.

## 3. `3_kirkegarden` – Begravelsen (natt)

> The churchyard at night under moonlight: the Romanesque stone church on the left and the stone crypt entrance on the right with its door open and torches burning, crooked gravestones and wooden crosses in the grass inside the low wall, mist creeping over the ground, the dirt road glistening, a solemn procession of hooded monks with lanterns carrying long wooden coffins bound with iron chains towards the crypt door, faint magenta glow leaking from the coffins, sombre and quiet. The grass is dark blue-green in the moonlight.

## 4. `4_krypten` – Krypten i dag

> The crypt under the church today: the vaulted brick chamber with the iron portcullis in the arch and the torch on the wall, burial niches with skulls and bones in the walls, cobwebs, damp flagstones reflecting the torchlight, cold blue light beyond the portcullis, behind the bars the silhouettes of monsters made of corrupted interface components (a hunched creature shaped like a rusty push-button, a floating tooltip-shaped sprite) with eerie magenta glow, mysterious and dangerous.

## 5. `5_torget` – Fristad, år 1350 (dag)

> The market square of Fristad in 1350 on a sunny afternoon: the same stall, well, church, gate and half-timbered houses, now full of life: townsfolk in small groups whispering and pointing (a burly blacksmith in a leather apron, an innkeeper woman in a red dress, a merchant in a blue fur-trimmed coat, a maid with a braid, a wide-eyed boy), baskets of apples and bread on the stall, smoke from a chimney, pigeons, flags on the gate towers, soft clouds, lively and warm.

---

Grunnbildene lages med `snapshot --film-seq` fra kamerastedene under (verdens koordinater, grader; NATT 1 er natt). Filmen setter maleriene ut på de samme stedene.

| Scene | Øye (x, y, z) | Yaw | Pitch | Synsfelt | Natt |
|---|---|---|---|---|---|
| `1_skriptoriet` | −8,6, 1,8, −11,8 | 25 | 6 | 62 | 0 |
| `2_fallet` | −8,6, 1,8, −11,8 | 25 | 6 | 62 | 1 |
| `3_kirkegarden` | −2, 2,5, −40 | 49 | −4 | 55 | 1 |
| `4_krypten` | −9, −38,3, −14 | 180 | 0 | 55 | 1 |
| `5_torget` | 0, 2, 12 | 0 | −2 | 60 | 0 |
