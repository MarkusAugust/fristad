"""Granbar og engblomster fra eierens ark (ensfarget mørk grå bakgrunn).

    python3 tools/textures/sheets.py needles <ark.jpg>
    python3 tools/textures/sheets.py flowers <ark.jpg>

needles: kvistene som ligger vannrett (rota til venstre) dreies så rota
står nederst, og legges side om side og oppå hverandre til et tett
barkort: assets/models/trees/needles_spruce.png (rota ved v = 0, langs
kortet som trees.py legger det).

hair / beard: hele arket med alfa (hår: fire farger × tre lokker, rota
øverst; skjegg: fire farger med helskjegg, kinnskjegg, bart og stubb),
til assets/models/npc/hair_atlas.png og beard_atlas.png, som
tools/models/npc.py legger på kortene rundt hodet.

flowers: arket er fem kolonner (smørblomst, blåklokke, ryllik, rødkløver,
hundekjeks) og tre rader. Hver plante beskjæres og legges i et atlas med
rota nederst: assets/models/trees/flowers_atlas.png og flowers_items.json
(som grass_items.json), som tools/models/grass.py bygger blomstertuster av."""

import json
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from leaf_atlas import cutout  # noqa: E402

OUT = "assets/models/trees"
SIZE = 1024


def load(path):
    rgb = np.asarray(Image.open(path).convert("RGB")).astype(np.float32)
    bg = np.median(np.concatenate([rgb[:8].reshape(-1, 3), rgb[-8:].reshape(-1, 3)]), axis=0)
    return cutout(rgb, bg)


def crop(cell):
    ys, xs = np.where(cell[..., 3] > 128)
    if len(ys) < 200:
        return None
    return Image.fromarray(cell[ys.min():ys.max() + 1, xs.min():xs.max() + 1].clip(0, 255).astype(np.uint8), "RGBA")


def needles(path):
    rgba = load(path)
    h, w = rgba.shape[:2]
    sprigs = []
    # Rad 2 og 3, kolonne 3–4 og hele rad 3: kvistene med rota til venstre.
    for r, cols in ((1, (2, 3)), (2, (0, 1, 2, 3))):
        for c in cols:
            p = crop(rgba[r * h // 3:(r + 1) * h // 3, c * w // 4:(c + 1) * w // 4])
            if p:
                sprigs.append(p.rotate(90, expand=True))
    canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    rng = np.random.default_rng(4)
    for k in range(9):
        p = sprigs[k % len(sprigs)]
        s = SIZE * rng.uniform(0.9, 1.0) / p.height
        q = p.resize((max(1, int(p.width * s)), int(p.height * s)), Image.LANCZOS)
        q = q.rotate(rng.uniform(-8, 8), resample=Image.BICUBIC, expand=True)
        x = int(SIZE / 2 + (k - 4) * SIZE * 0.07 - q.width / 2)
        canvas.alpha_composite(q, (max(0, min(SIZE - q.width, x)), max(0, SIZE - q.height)))
    a = np.asarray(canvas).astype(np.float32)
    known = a[..., 3] > 10
    a[..., :3][~known] = a[..., :3][known].mean(0)
    Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA").save(f"{OUT}/needles_spruce.png", optimize=True)
    print(f"{OUT}/needles_spruce.png av {len(sprigs)} kvister")


def flowers(path):
    rgba = load(path)
    h, w = rgba.shape[:2]
    plants = []
    for c in range(5):
        for r in range(3):
            p = crop(rgba[r * h // 3:(r + 1) * h // 3, c * w // 5:(c + 1) * w // 5])
            if p:
                plants.append((c, p))
    # Atlaset: fem kolonner, tre rader, hver plante skalert inn i ruta.
    cw, ch = SIZE // 5, SIZE // 3
    canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    items = []
    count = [0] * 5
    for c, p in plants:
        r = count[c]
        count[c] += 1
        s = min((cw - 8) / p.width, (ch - 8) / p.height)
        q = p.resize((max(1, int(p.width * s)), max(1, int(p.height * s))), Image.LANCZOS)
        x0 = c * cw + (cw - q.width) // 2
        y0 = r * ch + ch - 4 - q.height
        canvas.alpha_composite(q, (x0, y0))
        items.append({"kind": ["buttercup", "harebell", "yarrow", "clover", "cowparsley"][c], "box": [x0 / SIZE, y0 / SIZE, (x0 + q.width) / SIZE, (y0 + q.height) / SIZE]})
    a = np.asarray(canvas).astype(np.float32)
    known = a[..., 3] > 10
    a[..., :3][~known] = a[..., :3][known].mean(0)
    # Litt lavere albedo, som gresset.
    a[..., :3] *= 0.85
    Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA").save(f"{OUT}/flowers_atlas.png", optimize=True)
    json.dump({"source": path.rsplit("/", 1)[-1], "items": items}, open(f"{OUT}/flowers_items.json", "w"), indent=1)
    print(f"{OUT}/flowers_atlas.png: {len(items)} planter")


def sheet(path, name):
    """Hele arket med alfa, halvert til 1024², fargen bredd ut bak alfaen."""
    rgba = load(path)
    a = rgba.copy()
    known = a[..., 3] > 10
    a[..., :3][~known] = a[..., :3][known].mean(0)
    import os
    os.makedirs("assets/models/npc", exist_ok=True)
    Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA").resize((SIZE, SIZE), Image.LANCZOS).save(f"assets/models/npc/{name}_atlas.png", optimize=True)
    print(f"assets/models/npc/{name}_atlas.png")


if __name__ == "__main__":
    {"needles": needles, "flowers": flowers, "hair": lambda p: sheet(p, "hair"), "beard": lambda p: sheet(p, "beard")}[sys.argv[1]](sys.argv[2])
