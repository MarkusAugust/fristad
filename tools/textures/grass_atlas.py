"""Gressatlaset: tar et ark med gresstrå og tuster (fra eierens
bildegenerator) og gjør det til en RGBA-tekstur med ekte alfa, og finner
hvert strå og hver tust i arket.

    python3 tools/textures/grass_atlas.py <ark.jpg>

Arket har et tegnet rutemønster i stedet for gjennomsiktighet (grått 196
og hvitt 255, uten farge), så bakgrunnen skilles ut på fargemetning og
lyshet, med myk kant. Fargen bres ut i det gjennomsiktige så kantene ikke
blir grå når teksturen mipmappes.

Skriver assets/models/trees/grass_atlas.png (1024²) og grass_items.json
med hver bit som [x0, y0, x1, y1] i 0..1 (y nedover, rota nederst), som
tools/models/grass.py bygger tustene av."""

import json
import os
import sys

import numpy as np
from PIL import Image

OUT = "assets/models/trees"
SIZE = 1024


def spans(profile, min_gap, min_len, thresh):
    """Sammenhengende løp der profilen er over terskelen, slått sammen over
    hull kortere enn `min_gap`."""
    on = profile > thresh
    out, start, gap = [], None, 0
    for i, v in enumerate(on):
        if v:
            if start is None:
                start = i
            gap = 0
            end = i
        elif start is not None:
            gap += 1
            if gap >= min_gap:
                if end - start + 1 >= min_len:
                    out.append((start, end + 1))
                start = None
    if start is not None and end - start + 1 >= min_len:
        out.append((start, end + 1))
    return out


def main():
    src = sys.argv[1]
    rgb = np.asarray(Image.open(src).convert("RGB")).astype(np.float32)
    h, w, _ = rgb.shape
    mx, mn = rgb.max(-1), rgb.min(-1)
    chroma = mx - mn
    # Bakgrunnen er lys og fargeløs; plantene har farge eller er mørke.
    alpha = np.maximum(np.clip((chroma - 9.0) / 22.0, 0, 1), np.clip((165.0 - mn) / 45.0, 0, 1))
    # Fjern enkeltprikker: krev naboer i en 5×5-boks.
    m = (alpha > 0.5).astype(np.float32)
    pad = np.pad(m, 2)
    count = sum(pad[dy:dy + h, dx:dx + w] for dy in range(5) for dx in range(5))
    alpha = np.where(count >= 4, alpha, 0.0)
    mask = alpha > 0.5

    items = []
    for y0, y1 in spans(mask.sum(1), 18, 40, 1):
        band = mask[y0:y1]
        for x0, x1 in spans(band.sum(0), 4, 12, 3):
            cell = band[:, x0:x1]
            rows = np.where(cell.any(1))[0]
            items.append([x0 / w, (y0 + rows[0]) / h, x1 / w, (y0 + rows[-1] + 1) / h])

    # Kantene er blandet med den hvite og grå bakgrunnen; bare det helt
    # dekte innerste i plantene beholder fargen, og den bres ut over
    # kantene og det gjennomsiktige (gjentatt snitt av naboer med farge).
    col = rgb.copy()
    known = (alpha > 0.97) & (chroma > 18)
    for _ in range(40):
        if known.all():
            break
        acc = np.zeros_like(col)
        cnt = np.zeros((h, w), np.float32)
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            k = np.roll(known, (dy, dx), (0, 1))
            acc += np.roll(col, (dy, dx), (0, 1)) * k[..., None]
            cnt += k
        grow = (~known) & (cnt > 0)
        col[grow] = acc[grow] / cnt[grow][:, None]
        known = known | grow
    col[~known] = col[known].mean(0)

    # Ekte gress har lav albedo: litt mørkere og mer mettet enn arket,
    # som er lyst fotografert med høylys på stråene.
    grey = col.mean(-1, keepdims=True)
    col = (grey + (col - grey) * 1.35) * 0.78
    out = np.dstack([col, alpha * 255.0]).clip(0, 255).astype(np.uint8)
    os.makedirs(OUT, exist_ok=True)
    Image.fromarray(out, "RGBA").resize((SIZE, SIZE), Image.LANCZOS).save(f"{OUT}/grass_atlas.png", optimize=True)
    with open(f"{OUT}/grass_items.json", "w") as f:
        json.dump({"source": os.path.basename(src), "items": items}, f, indent=1)
    print(f"{OUT}/grass_atlas.png: {len(items)} strå og tuster")


if __name__ == "__main__":
    main()
