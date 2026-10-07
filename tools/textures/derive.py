"""Avleder normalkart, høyde og ruhet fra et albedo-bilde (fra en
bildegenerator eller et foto), så et enkelt bilde blir et fullt teksturlag.

    python3 tools/textures/derive.py <lag> <bilde> [--size-m 2.0] [--rough 0.8] [--depth 1.0] [--invert]

Skriver assets/textures/<lag>/albedo.jpg, normal.jpg, height.jpg,
rough.jpg og meta.json (meter per flis). Alt regnes med omslag i kantene,
så flisene møtes uten søm. Høyden er lysheten med de store variasjonene
trukket fra: fugene og sprekkene er mørke og blir lave. `--invert` snur det
for materialer der det mørke stikker opp. Lagnavnene er de i
`Layer::name` (cobble, plaster, oak, tiles, stone, cladding …)."""

import json
import os
import sys

import numpy as np
from PIL import Image

SIZE = 1024


def blur(a, r):
    """Gaussisk uskarphet med omslag i kantene (sirkulær, via FFT), så
    flisene møtes uten søm."""
    h, w = a.shape
    fy = np.fft.fftfreq(h)[:, None]
    fx = np.fft.fftfreq(w)[None, :]
    g = np.exp(-2.0 * (np.pi * r) ** 2 * (fx * fx + fy * fy))
    return np.real(np.fft.ifft2(np.fft.fft2(a) * g)).astype(np.float32)


def main():
    args = sys.argv[1:]
    layer, src = args[0], args[1]
    opt = lambda k, d: float(args[args.index(k) + 1]) if k in args else d
    size_m = opt("--size-m", 2.0)
    rough_base = opt("--rough", 0.8)
    depth = opt("--depth", 1.0)
    invert = "--invert" in args
    out = f"assets/textures/{layer}"
    os.makedirs(out, exist_ok=True)

    img = Image.open(src).convert("RGB").resize((SIZE, SIZE), Image.LANCZOS)
    img.save(f"{out}/albedo.jpg", quality=90)
    rgb = np.asarray(img, dtype=np.float32) / 255.0
    lum = rgb @ np.array([0.299, 0.587, 0.114], dtype=np.float32)
    if invert:
        lum = 1.0 - lum
    # Høyden: lysheten minus det store (belysning og fargeflekker), litt
    # myket opp, og strukket til 0..1.
    h = blur(lum, 2.0) - blur(lum, 40.0) * 0.85
    h = blur(h, 1.2)
    lo, hi = np.percentile(h, 1), np.percentile(h, 99)
    h = np.clip((h - lo) / max(hi - lo, 1e-4), 0, 1)
    Image.fromarray((h * 255).astype(np.uint8)).save(f"{out}/height.jpg", quality=92)

    # Normalen fra høyden (OpenGL-konvensjon, grønt opp), med omslag.
    strength = 6.0 * depth
    dx = (np.roll(h, -1, axis=1) - np.roll(h, 1, axis=1)) * 0.5
    dy = (np.roll(h, -1, axis=0) - np.roll(h, 1, axis=0)) * 0.5
    nx, ny, nz = -dx * strength, dy * strength, np.ones_like(h)
    length = np.sqrt(nx * nx + ny * ny + nz * nz)
    n = np.stack([nx / length, ny / length, nz / length], axis=-1)
    Image.fromarray(((n * 0.5 + 0.5) * 255).astype(np.uint8)).save(f"{out}/normal.jpg", quality=92)

    # Ruheten: grunnverdien, glattere på toppene som slites, ruere i fugene.
    r = np.clip(rough_base + (0.5 - h) * 0.2 + (lum - lum.mean()) * -0.1, 0.05, 1.0)
    Image.fromarray((r * 255).astype(np.uint8)).save(f"{out}/rough.jpg", quality=92)

    with open(f"{out}/meta.json", "w") as f:
        json.dump({"size_m": size_m, "source": os.path.basename(src)}, f)
    print(f"{out}: albedo, normal, høyde, ruhet ({size_m} m per flis)")


if __name__ == "__main__":
    main()
