"""Bladteksturene til trærne fra eierens ark med eike- og bjørkekvister
(fire kolonner: to med eik, to med bjørk, på ensfarget mørk grå bakgrunn).

    python3 tools/textures/leaf_atlas.py <ark.jpg>

Hver kvist skilles fra bakgrunnen på avstand fra den grå fargen (myk
kant), og kvistene legges tett sammen, dreid og skalert, i et kvadrat så
hvert bladkort i treet blir en full klynge. Fargen bres ut i det
gjennomsiktige så kantene ikke blir grå i mipnivåene. Skriver
assets/models/trees/leaves_oak.png, leaves_birch.png og leaves_bush.png
(1024², RGBA), som tools/models/trees.py legger på kortene."""

import random
import sys

import numpy as np
from PIL import Image

OUT = "assets/models/trees"
SIZE = 1024


def cutout(rgb, bg):
    """Alfa fra avstanden til bakgrunnsfargen, og fargen bredd ut."""
    d = np.sqrt(((rgb - bg) ** 2).sum(-1))
    alpha = np.clip((d - 14.0) / 22.0, 0, 1)
    col = rgb.copy()
    known = alpha > 0.95
    h, w = alpha.shape
    for _ in range(30):
        acc = np.zeros_like(col)
        cnt = np.zeros((h, w), np.float32)
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            k = np.roll(known, (dy, dx), (0, 1))
            acc += np.roll(col, (dy, dx), (0, 1)) * k[..., None]
            cnt += k
        grow = (~known) & (cnt > 0)
        col[grow] = acc[grow] / cnt[grow][:, None]
        known |= grow
    if known.any():
        col[~known] = col[known].mean(0)
    return np.dstack([col, alpha * 255.0])


def sprigs(rgba, cols):
    """Kvistene i kolonnene `cols` (av fire) og fire rader, hver beskåret
    til der det er blader."""
    h, w = rgba.shape[:2]
    out = []
    for c in cols:
        for r in range(4):
            cell = rgba[r * h // 4:(r + 1) * h // 4, c * w // 4:(c + 1) * w // 4]
            m = cell[..., 3] > 128
            ys, xs = np.where(m)
            if len(ys) < 100:
                continue
            out.append(Image.fromarray(cell[ys.min():ys.max() + 1, xs.min():xs.max() + 1].clip(0, 255).astype(np.uint8), "RGBA"))
    return out


def atlas(pieces, seed, darken=1.0):
    """Kvistene lagt tett i et kvadrat, dreid ut fra midten som i en
    klynge, med den gjennomsnittlige fargen bak alfaen."""
    rng = random.Random(seed)
    canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    n = 11
    for k in range(n):
        p = rng.choice(pieces)
        a = 2 * np.pi * k / n + rng.uniform(-0.3, 0.3)
        s = rng.uniform(0.42, 0.55) * SIZE / max(p.size)
        q = p.resize((int(p.width * s), int(p.height * s)), Image.LANCZOS)
        # Kvisten peker utover fra midten (stilken mot midten).
        q = q.rotate(-np.degrees(a) - 90 + 180, resample=Image.BICUBIC, expand=True)
        r = rng.uniform(0.18, 0.3) * SIZE
        x = int(SIZE / 2 + np.cos(a) * r - q.width / 2)
        y = int(SIZE / 2 + np.sin(a) * r - q.height / 2)
        canvas.alpha_composite(q, (max(0, min(SIZE - q.width, x)), max(0, min(SIZE - q.height, y))))
    for _ in range(3):
        p = rng.choice(pieces)
        s = rng.uniform(0.4, 0.5) * SIZE / max(p.size)
        q = p.resize((int(p.width * s), int(p.height * s)), Image.LANCZOS).rotate(rng.uniform(0, 360), resample=Image.BICUBIC, expand=True)
        canvas.alpha_composite(q, ((SIZE - q.width) // 2, (SIZE - q.height) // 2))
    a = np.asarray(canvas).astype(np.float32)
    known = a[..., 3] > 10
    mean = (a[..., :3][known]).mean(0)
    a[..., :3][~known] = mean
    a[..., :3] *= darken
    return Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA")


def main():
    rgb = np.asarray(Image.open(sys.argv[1]).convert("RGB")).astype(np.float32)
    bg = np.median(np.concatenate([rgb[:8].reshape(-1, 3), rgb[-8:].reshape(-1, 3)]), axis=0)
    rgba = cutout(rgb, bg)
    oak = sprigs(rgba, [0, 1])
    birch = sprigs(rgba, [2, 3])
    atlas(oak, 1).save(f"{OUT}/leaves_oak.png", optimize=True)
    atlas(birch, 2).save(f"{OUT}/leaves_birch.png", optimize=True)
    atlas(birch + oak[:2], 3, darken=0.82).save(f"{OUT}/leaves_bush.png", optimize=True)
    print(f"{OUT}: leaves_oak, leaves_birch, leaves_bush av {len(oak)} eikekvister og {len(birch)} bjørkekvister")


if __name__ == "__main__":
    main()
