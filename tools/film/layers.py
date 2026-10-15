"""Lagene i filmene (`web/film/`): malte bilder fra eieren, klippet og
pakket til `assets/film/<film>/<scene>/<lag>.webp`.

    python3 tools/film/layers.py key MAPPE [film]   # eierens bilder inn
    python3 tools/film/layers.py placeholders       # plassholdere til utkastet

Eierens bilder heter `<scene>_<lag>.png|jpg` (lag: bg, mid, fg; promptene i
`docs/prompter-film.md`). Bakgrunnen (`bg`) er hele bildet; mellomgrunnen og
forgrunnen er malt på ensfarget grønt (#00FF00), som her blir gjennomsiktig,
med det grønne skjæret i kantene tatt bort.

Plassholderne (til eieren har laget bildene): bakgrunnen er et bilde av
stedet fra spillet (`snapshot --film`, `target/film/p<n>.png`), og figurene
klippes ut av arkene i `assets/textures/people/ref/` og
`assets/textures/monsters/ref/`. De står som `placeholder` i manuset, så de
aldri tas for ferdige.
"""

import os
import sys

import numpy as np
from PIL import Image, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "assets", "film")
WIDTH = 2560  # bredden lagene lagres i
QUALITY = 86


def save(img, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if img.width > WIDTH:
        img = img.resize((WIDTH, round(img.height * WIDTH / img.width)), Image.LANCZOS)
    img.save(path, "WEBP", quality=QUALITY, method=6)
    print(os.path.relpath(path, ROOT), img.size)


def key_green(img):
    """Grønt blir gjennomsiktig: hvor mye grønnere pikselen er enn de andre
    kanalene, myket i kanten, og det grønne skjæret tatt ut av kantene."""
    a = np.asarray(img.convert("RGB")).astype(np.float32) / 255.0
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    excess = g - np.maximum(r, b)
    alpha = np.clip(1.0 - (excess - 0.12) / 0.25, 0.0, 1.0)
    # Despill: grønt over det største av rødt og blått kappes.
    g2 = np.minimum(g, np.maximum(r, b) + 0.02)
    out = np.dstack([r, g2, b, alpha])
    im = Image.fromarray((out * 255).astype(np.uint8), "RGBA")
    # Kanten myknes litt (alfa bare), så klippet ikke blir taggete.
    al = im.getchannel("A").filter(ImageFilter.GaussianBlur(0.8))
    im.putalpha(al)
    return im


# Etterarbeid per lag: et utsnitt av bildet (brøk av bredden og høyden)
# satt på et tomt lerret med foten der (brøk) og høyden (brøk av lerretet).
# Bildegeneratoren ga tre vandrere der prompten ba om én.
FIT = {
    ("opening", "sorporten", "mid"): dict(crop=(0.38, 0.0, 0.62, 1.0), at=(0.47, 0.97), h=0.36),
}
# Lag som kom som et helt bilde i stedet for på grønt: brukes uklippet.
WHOLE = {("opening", "begravelsen", "mid")}


def fit(img, crop, at, h):
    w0, h0 = img.size
    part = img.crop((int(crop[0] * w0), int(crop[1] * h0), int(crop[2] * w0), int(crop[3] * h0)))
    box = part.getbbox()
    if box:
        part = part.crop(box)
    canvas = Image.new("RGBA", (w0, h0), (0, 0, 0, 0))
    hh = int(h0 * h)
    part = part.resize((max(1, round(part.width * hh / part.height)), hh), Image.LANCZOS)
    canvas.alpha_composite(part, (int(w0 * at[0] - part.width / 2), int(h0 * at[1] - hh)))
    return canvas


def key(src, film="opening"):
    for name in sorted(os.listdir(src)):
        stem, ext = os.path.splitext(name)
        if ext.lower() not in (".png", ".jpg", ".jpeg", ".webp") or "_" not in stem:
            continue
        scene, layer = stem.rsplit("_", 1)
        img = Image.open(os.path.join(src, name))
        if layer in ("mid", "fg") and (film, scene, layer) not in WHOLE:
            img = key_green(img)
        else:
            img = img.convert("RGB")
        if (film, scene, layer) in FIT:
            img = fit(img, **FIT[(film, scene, layer)])
        # Klippede lag kuttes til innholdet; manuset sier hvor de står.
        if img.mode == "RGBA":
            box = img.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
            if box:
                img = img.crop(box)
        save(img, os.path.join(OUT, film, scene, f"{layer}.webp"))


# --- Plassholdere ------------------------------------------------------------------


def cutout(path, which=0, bg_dist=30):
    """Figur nr. `which` fra venstre på et ark, med bakgrunnen (fargen i
    hjørnene) gjennomsiktig."""
    im = np.asarray(Image.open(path).convert("RGB")).astype(np.float32)
    bg = np.median(np.concatenate([im[:8].reshape(-1, 3), im[-8:].reshape(-1, 3)]), 0)
    fg = np.linalg.norm(im - bg, axis=2) > bg_dist
    cols = fg.sum(0) > 0.02 * fg.shape[0]
    runs, a, gap = [], None, 0
    for x, c in enumerate(cols):
        if c:
            a = x if a is None else a
            gap = 0
        elif a is not None:
            gap += 1
            if gap > 3:
                runs.append((a, x - gap + 1))
                a, gap = None, 0
    if a is not None:
        runs.append((a, len(cols)))
    runs = [r for r in runs if r[1] - r[0] > 0.05 * len(cols)]
    x0, x1 = runs[min(which, len(runs) - 1)]
    rows = np.nonzero(fg[:, x0:x1].any(1))[0]
    y0, y1 = rows.min(), rows.max() + 1
    from scipy import ndimage

    m = ndimage.binary_fill_holes(fg[y0:y1, x0:x1])
    alpha = ndimage.gaussian_filter(m.astype(np.float32), 1.0)
    rgba = np.dstack([im[y0:y1, x0:x1], alpha * 255]).astype(np.uint8)
    return Image.fromarray(rgba, "RGBA")


def compose(figures, size=(2560, 1440)):
    """Figurene på et gjennomsiktig lerret: (bilde, midten x 0–1, foten y
    0–1, høyde 0–1, mørkere 0–1)."""
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    for img, cx, foot, h, dark in figures:
        hh = int(size[1] * h)
        img = img.resize((max(1, round(img.width * hh / img.height)), hh), Image.LANCZOS)
        if dark:
            arr = np.asarray(img).astype(np.float32)
            arr[..., :3] *= 1.0 - dark
            img = Image.fromarray(arr.astype(np.uint8), "RGBA")
        canvas.alpha_composite(img, (int(size[0] * cx - img.width / 2), int(size[1] * foot - hh)))
    return canvas


def placeholders():
    people = os.path.join(ROOT, "assets", "textures", "people", "ref")
    monsters = os.path.join(ROOT, "assets", "textures", "monsters", "ref")
    plates = os.path.join(ROOT, "target", "film")
    p = lambda n: os.path.join(people, f"{n}_apose.jpeg")
    m = lambda n: os.path.join(monsters, f"{n}_body.jpeg")
    film = os.path.join(OUT, "opening")
    for k, scene in enumerate(["gullalderen", "fallet", "begravelsen", "krypten", "fristad1350", "sorporten"], 1):
        plate = os.path.join(plates, f"p{k}.png")
        if os.path.exists(plate):
            save(Image.open(plate).convert("RGB"), os.path.join(film, scene, "bg.webp"))
    save(compose([(cutout(p("clerk")), 0.78, 0.98, 0.72, 0.0)]), os.path.join(film, "gullalderen", "mid.webp"))
    save(compose([(cutout(m("button_wight")), 0.25, 0.95, 0.42, 0.0), (cutout(m("tooltip_sprite")), 0.72, 0.62, 0.3, 0.0)]), os.path.join(film, "fallet", "mid.webp"))
    save(compose([(cutout(p("clerk")), 0.3, 0.97, 0.5, 0.75), (cutout(p("pilgrim")), 0.45, 0.97, 0.5, 0.75), (cutout(p("apothecary")), 0.6, 0.97, 0.5, 0.75)]),
         os.path.join(film, "begravelsen", "mid.webp"))
    save(compose([(cutout(m("button_wight")), 0.3, 0.98, 0.5, 0.15), (cutout(m("modal")), 0.68, 0.98, 0.78, 0.15)]), os.path.join(film, "krypten", "mid.webp"))
    save(compose([(cutout(p("smith")), 0.12, 1.0, 0.72, 0.0), (cutout(p("innkeeper")), 0.27, 1.0, 0.64, 0.0), (cutout(p("maid")), 0.72, 1.0, 0.62, 0.0),
                  (cutout(p("boy")), 0.83, 1.0, 0.5, 0.0), (cutout(p("merchant")), 0.93, 1.0, 0.66, 0.0)]),
         os.path.join(film, "fristad1350", "mid.webp"))
    save(compose([(cutout(p("pilgrim"), which=2), 0.5, 0.92, 0.42, 0.1)]), os.path.join(film, "sorporten", "mid.webp"))


if __name__ == "__main__":
    if sys.argv[1:2] == ["key"]:
        key(sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else "opening")
    elif sys.argv[1:2] == ["placeholders"]:
        placeholders()
    else:
        print(__doc__)
