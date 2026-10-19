# TRELLIS: innbyggerne

Alle de fjorten innbyggerne er bygd fra TRELLIS (smeden godkjent 15. oktober). Det som gjenstår, er Torgils, som i dag låner figuren `npc_gorvek` (kanon: Gorvek er aldri i byen).

**Slik gjør du det:**

1. Lag ETT kvadratisk bilde forfra i A-stilling med prompten (ikke ark med side og rygg: TRELLIS dikter dem selv, og vi maler aldri bilder på formen; det ga ansikt i bakhodet). Sjekk at armene står fri av kroppen og at hendene er tomme.
2. Lagre som `assets/textures/people/ref/<navn>_front.png`.
3. Kjør bildet gjennom https://huggingface.co/spaces/microsoft/TRELLIS.2 med **Resolution 1536, Decimation Target 500000, Texture Size 4096** (nytt frø om formen eller ansiktet blir dårlig), og last opp glb-fila som `<navn>.glb`. Den legges i `assets/textures/people/3d/<navn>.glb`.
4. `python3.13 tools/models/folk3d.py <navn>` gjør den spillklar med TRELLIS-teksturen som den er (`FACE_KEEP`), og jeg viser deg den før den tas i bruk.

Blir et ansikt dårlig på en av de andre, brukes samme oppskrift: last opp det gamle bildet forfra (`<navn>_apose_front.png`) som forbilde sammen med denne malen og beskrivelsen av personen.

### 1. `torgils`: Torgils, våpenmesteren

- **Hvor:** øvingsplassen innenfor nordporten, går mellom dukkene og selger trening. **Størrelse:** 1,85 m.
- **Erstatter:** `npc_gorvek` på Torgils (`assets/layout/town.json`); blir `assets/models/npcs/npc_torgils.glb`.

```
Full-body character portrait for 3D modelling: ONE single view straight from the front, the whole person from head to feet centred in the image. A-pose: standing straight and relaxed, both arms held out from the body at about 30 degrees with a clear gap of background between each arm and the body, elbows very slightly bent, hands open and empty with the fingers together, legs straight with the feet a little apart and a clear gap between the legs. Nothing in the hands, nothing carried, nothing hanging in front of the arms, legs or body. Neutral calm face, mouth closed with clearly shaped lips, eyes open looking straight ahead. Torgils, the town's weapons master, a veteran man-at-arms in his fifties, broad and solid but not a giant, short grey-streaked brown hair, a short trimmed grey-brown beard, a calm stern weathered face with a small scar through one eyebrow. He wears a faded padded gambeson in dull ochre over a dark wool tunic, leather bracers, a wide leather belt with an empty sword frog lying flat against the hip, brown wool trousers and sturdy leather boots. Style: a high-end stylized 3D game character with hand-painted textures, like the characters in World of Warcraft (modern) and Fable: clean sculpted forms, slightly exaggerated proportions, big readable shapes, large hands, strong simple facial features with a defined brow, nose and cheekbones, appealing and believable, never grotesque, never photorealistic, never a flat cartoon. Hair and beard are a few large, smooth, chunky locks with painted strand detail, never thin strands, never frizzy. Soft painterly shading, NO black outlines. Historically plausible Scandinavian market town around 1350: wool, linen, leather and fur, no fantasy armour, no modern items. Warm muted Nordic palette, medium saturation. Soft even light from the front, no cast shadows, no shadow on the ground, no rim light. Plain flat light-grey background, no scenery, no floor, no text, no letters, no labels, no frame. Square image, 2048 by 2048 pixels, the subject as large as possible with clear background space on all sides, nothing cut off at the edges.
```
