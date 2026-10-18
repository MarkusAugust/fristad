"""Gjør skjermbildene fra tools/gallery/render.mjs til kvadratiske
teksturer (512×512 jpeg: komponenten skalert så den lengste sida er 80 % av
platen, midt på en svakt varm pergamentflate med litt tekstur, så platen ser
ut som et oppslag og ikke et skjermbilde) og et kontaktark. Kjør:

    python3 tools/gallery/pack.py target/gallery assets/textures/components shots/galleriet/kontaktark.png
"""

import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

SIZE = 512
FILL = 0.8  # lengste side av innholdet, andel av platen
PARCHMENT = (0xF3, 0xEA, 0xD8)


def parchment(rng):
    """Pergamenttonen med svak, myk flekkete tekstur og litt mørkere kanter."""
    noise = rng.normal(0.0, 1.0, (SIZE // 8, SIZE // 8)).astype(np.float32)
    n = Image.fromarray(np.clip(128 + noise * 40, 0, 255).astype(np.uint8), "L").resize((SIZE, SIZE), Image.BICUBIC).filter(ImageFilter.GaussianBlur(3))
    fine = rng.normal(0.0, 1.0, (SIZE, SIZE)).astype(np.float32)
    yy, xx = np.mgrid[0:SIZE, 0:SIZE]
    edge = np.hypot((xx - SIZE / 2) / SIZE, (yy - SIZE / 2) / SIZE)  # 0 i midten, 0,71 i hjørnet
    shade = 1.0 + (np.asarray(n, dtype=np.float32) - 128) / 128 * 0.05 + fine * 0.012 - np.clip(edge - 0.3, 0, 1) * 0.12
    base = np.array(PARCHMENT, dtype=np.float32)[None, None, :]
    return Image.fromarray(np.clip(base * shade[..., None], 0, 255).astype(np.uint8), "RGB")

src, dst, sheet = sys.argv[1:4]
os.makedirs(dst, exist_ok=True)
meta = json.load(open(os.path.join(src, "meta.json")))
thumbs = []
rng = np.random.default_rng(1350)
for name, m in meta.items():
    im = Image.open(os.path.join(src, f"{name}.png")).convert("RGBA")
    # Bort med den gjennomsiktige luften rundt, så innholdet fyller platen.
    bbox = im.getchannel("A").getbbox()
    if bbox:
        im = im.crop(bbox)
    k = SIZE * FILL / max(im.width, im.height)
    im = im.resize((max(1, round(im.width * k)), max(1, round(im.height * k))), Image.LANCZOS)
    out = parchment(rng)
    out.paste(im, ((SIZE - im.width) // 2, (SIZE - im.height) // 2), im)
    # JPEG: glTF-eksporten beholder formatet, så modellen blir rundt 40 kB i
    # stedet for 260 kB med png av den teksturerte flaten.
    out.save(os.path.join(dst, f"{name}.jpg"), quality=90, optimize=True)
    thumbs.append((name, out))
cols, cell = 5, 200
rows = (len(thumbs) + cols - 1) // cols
ark = Image.new("RGB", (cols * cell, rows * (cell + 18)), (200, 200, 200))
d = ImageDraw.Draw(ark)
for i, (name, im) in enumerate(thumbs):
    x, y = (i % cols) * cell, (i // cols) * (cell + 18)
    ark.paste(im.resize((cell - 4, cell - 4), Image.LANCZOS), (x + 2, y + 2))
    d.text((x + 4, y + cell), name, fill=(0, 0, 0))
os.makedirs(os.path.dirname(sheet) or ".", exist_ok=True)
ark.save(sheet)
print(f"{len(thumbs)} teksturer i {dst}, kontaktark {sheet} ({ark.width}×{ark.height})")
