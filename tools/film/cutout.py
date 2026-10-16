"""Bildene til åpningsfilmen (docs/film/manus.md) gjort klare for nettleseren.

Eieren maler hvert lag for seg (`assets/film/intro/src/<navn>.jpeg`). Det som
står på jevn grønn bakgrunn (#00FF00), klippes ut: alfa etter hvor mye
grønnere pikselen er enn rødt og blått, og grønnskjæret tas av kantene. Alt
skrives som webp med høyst `WIDTH` piksler i bredden til
`assets/film/intro/<navn>.webp`, og målene til `assets/film/intro/sizes.json`
(filmen trenger forholdet før bildet er lastet).

  python3 tools/film/cutout.py [navn ...]
"""

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "assets/film/intro/src"
OUT = ROOT / "assets/film/intro"
WIDTH = 2048
# Grønnere enn rødt og blått med så mye (0..1): helt gjennomsiktig over HI,
# helt dekkende under LO.
LO, HI = 0.10, 0.32
# Et utklipp som når kanten av bildet, ville endt i en rett, skarp kant i
# verden (eieren 16. oktober: «se på hvordan du klipper ting»). Der det når
# kanten, tones det bort over `EDGE` av bildet. Båndene som gjentas sidelengs
# (`TILES`), og bunnen av dem, står i ett og tones ikke.
EDGE = 0.08
TILES = {"aaser_fjerne", "aaser_naere", "forgrunn_gress", "forgrunn_steiner", "veien_natt", "jordlag"}
# Bilder som må beskjæres før de klippes (brøk: venstre, topp, høyre, bunn):
# fasaden har biter av nabohusene i kantene.
CROP = {"skriptoriet_fasade": (0.135, 0.0, 0.915, 1.0)}


def keyed(rgb):
    """Alfa og fargene uten grønnskjær for et bilde på grønn bakgrunn."""
    c = rgb.astype(np.float32) / 255.0
    r, g, b = c[..., 0], c[..., 1], c[..., 2]
    over = g - np.maximum(r, b)
    alpha = 1.0 - np.clip((over - LO) / (HI - LO), 0.0, 1.0)
    # Grønnskjæret: grønt kan ikke være sterkere enn det sterkeste av rødt og blått.
    c[..., 1] = np.minimum(g, np.maximum(r, b) + 0.02)
    return np.dstack([c, alpha])


def green_background(rgb):
    """Står bildet (helt eller delvis) på grønn bakgrunn? Da er en god del av
    pikslene rent grønne (himmelen over åsene, vinduet i fasaden)."""
    c = rgb[::4, ::4].astype(np.float32) / 255.0
    return float(np.mean((c[..., 1] - np.maximum(c[..., 0], c[..., 2])) > 0.35)) > 0.04


def soften(rgba, tile):
    """Toner utklippet bort mot kantene det når."""
    a = rgba[..., 3]
    h, w = a.shape
    n = max(2, int(EDGE * min(w, h)))
    ramp = np.linspace(0.0, 1.0, n, dtype=np.float32)
    ramp = ramp * ramp * (3 - 2 * ramp)
    sides = {"top": a[0], "bottom": a[-1], "left": a[:, 0], "right": a[:, -1]}
    touched = [k for k, e in sides.items() if (e > 0.5).mean() > 0.02 and not (tile and k in ("left", "right", "bottom"))]
    for k in touched:
        if k == "top":
            a[:n] *= ramp[:, None]
        elif k == "bottom":
            a[-n:] *= ramp[::-1, None]
        elif k == "left":
            a[:, :n] *= ramp[None, :]
        else:
            a[:, -n:] *= ramp[None, ::-1]
    return touched


def build(src):
    im = Image.open(src).convert("RGB")
    if src.stem in CROP:
        l, t, r, b = CROP[src.stem]
        im = im.crop((round(l * im.width), round(t * im.height), round(r * im.width), round(b * im.height)))
    if im.width > WIDTH:
        im = im.resize((WIDTH, round(im.height * WIDTH / im.width)), Image.LANCZOS)
    rgb = np.asarray(im)
    soft = []
    if green_background(rgb):
        rgba = keyed(rgb)
        soft = soften(rgba, src.stem in TILES)
        out = Image.fromarray((rgba * 255 + 0.5).astype(np.uint8), "RGBA")
        kind = "cutout"
    else:
        out = im
        kind = "plate"
    out.save(OUT / f"{src.stem}.webp", quality=88, method=6)
    return {"w": out.width, "h": out.height, "kind": kind, **({"soft": soft} if soft else {})}


def main(names):
    sizes_path = OUT / "sizes.json"
    sizes = json.loads(sizes_path.read_text()) if sizes_path.exists() else {}
    for src in sorted(SRC.glob("*.jp*g")) + sorted(SRC.glob("*.png")):
        if names and src.stem not in names:
            continue
        sizes[src.stem] = build(src)
        print(src.stem, sizes[src.stem])
    sizes_path.write_text(json.dumps(dict(sorted(sizes.items())), indent=1) + "\n")


if __name__ == "__main__":
    main(sys.argv[1:])
