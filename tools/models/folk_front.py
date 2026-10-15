"""Bildet forfra av et ark i A-stilling, til bilde-til-3D (TRELLIS.2).

Arket (`assets/textures/people/ref/<navn>_apose.jpeg`) viser figuren forfra,
fra siden og bakfra; figuren lengst til venstre klippes ut med luft rundt
og legges midt i et kvadrat med bakgrunnsfargen (`<navn>_apose_front.png`).

    python3 tools/models/folk_front.py ark.jpeg [navn]
"""

import os
import shutil
import sys

import numpy as np
from PIL import Image

REF = "assets/textures/people/ref"
PAD = 40  # luft rundt figuren (px)
BG = 30  # avstand i farge fra bakgrunnen (0–255) som regnes som figur
GAP = 3  # tomme kolonner som skiller to figurer


def front(path):
    im = np.asarray(Image.open(path).convert("RGB")).astype(float)
    bg = np.median(np.concatenate([im[:8].reshape(-1, 3), im[-8:].reshape(-1, 3)]), 0)
    fg = np.linalg.norm(im - bg, axis=2) > BG
    cols = fg.sum(0) > 0.02 * fg.shape[0]  # en strek under føttene er ikke figur
    runs, a, empty = [], None, 0
    for x, c in enumerate(cols):
        if c:
            a = x if a is None else a
            empty = 0
        elif a is not None:
            empty += 1
            if empty > GAP:
                runs.append((a, x - empty + 1))
                a, empty = None, 0
    if a is not None:
        runs.append((a, len(cols)))
    runs = [r for r in runs if r[1] - r[0] > 0.05 * len(cols)]
    x0, x1 = runs[0]
    rows = np.nonzero(fg[:, x0:x1].any(1))[0]
    y0, y1 = rows.min(), rows.max() + 1
    side = max(x1 - x0, y1 - y0) + 2 * PAD
    out = Image.new("RGB", (side, side), tuple(int(v) for v in bg))
    crop = Image.fromarray(im[y0:y1, x0:x1].astype(np.uint8))
    out.paste(crop, ((side - (x1 - x0)) // 2, (side - (y1 - y0)) // 2))
    return out


if __name__ == "__main__":
    src = sys.argv[1]
    name = sys.argv[2] if len(sys.argv) > 2 else os.path.basename(src).split(".")[0].removesuffix("_apose")
    sheet = os.path.join(REF, f"{name}_apose.jpeg")
    if os.path.abspath(src) != os.path.abspath(sheet):
        shutil.copy(src, sheet)
    front(sheet).save(os.path.join(REF, f"{name}_apose_front.png"))
    print(name, "->", os.path.join(REF, f"{name}_apose_front.png"))
