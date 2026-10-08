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

flames: åtte flammer på svart (fire lys øverst, fire bål nederst) til
assets/models/props/flames_atlas.png: alfa er lysstyrken, fargen delt på
alfa, så flammen kan blandes additivt eller alfablandes uten svart kant.

door: en hel jernbeslått dør (1:2) til assets/textures/door/ med albedo,
høyde og normalkart i samme format, til dørene i husene og krypten.

flowers: arket er fem kolonner (smørblomst, blåklokke, ryllik, rødkløver,
hundekjeks) og tre rader. Hver plante beskjæres og legges i et atlas med
rota nederst: assets/models/trees/flowers_atlas.png og flowers_items.json
(som grass_items.json), som tools/models/grass.py bygger blomstertuster av."""

import json
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from derive import blur  # noqa: E402
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


def flames(path):
    import os
    rgb = np.asarray(Image.open(path).convert("RGB")).astype(np.float32)
    h, w = rgb.shape[:2]
    canvas = Image.new("RGBA", (SIZE, SIZE // 2), (0, 0, 0, 0))
    cw, ch = SIZE // 4, SIZE // 4
    for r in range(2):
        for c in range(4):
            cell = rgb[r * h // 2:(r + 1) * h // 2, c * w // 4:(c + 1) * w // 4]
            a = np.clip((cell.max(-1) - 12.0) / 200.0, 0, 1)
            col = np.where(a[..., None] > 0.01, cell / np.maximum(a[..., None] * 255.0, 1e-3) * 255.0, 0)
            img = Image.fromarray(np.dstack([col, a * 255]).clip(0, 255).astype(np.uint8), "RGBA")
            ys, xs = np.where(a > 0.05)
            img = img.crop((xs.min(), ys.min(), xs.max() + 1, ys.max() + 1))
            k = min(cw / img.width, ch / img.height) * 0.95
            img = img.resize((max(1, int(img.width * k)), max(1, int(img.height * k))), Image.LANCZOS)
            canvas.alpha_composite(img, (c * cw + (cw - img.width) // 2, r * ch + ch - img.height))
    os.makedirs("assets/models/props", exist_ok=True)
    canvas.save("assets/models/props/flames_atlas.png", optimize=True)
    print("assets/models/props/flames_atlas.png")


def door(path):
    import os
    out = "assets/textures/door"
    os.makedirs(out, exist_ok=True)
    img = Image.open(path).convert("RGB").resize((512, 1024), Image.LANCZOS)
    img.save(f"{out}/albedo.jpg", quality=92)
    lum = (np.asarray(img, dtype=np.float32) / 255.0) @ np.array([0.299, 0.587, 0.114], dtype=np.float32)
    # Jernet stikker opp (lyst og glatt mot det mørke treet er vanskelig å
    # skille på lyshet alene), så høyden er lysheten minus det store.
    hgt = blur(lum, 1.5) - blur(lum, 30.0) * 0.85
    lo, hi = np.percentile(hgt, 1), np.percentile(hgt, 99)
    hgt = np.clip((hgt - lo) / max(hi - lo, 1e-4), 0, 1)
    Image.fromarray((hgt * 255).astype(np.uint8)).save(f"{out}/height.jpg", quality=92)
    dx = (np.roll(hgt, -1, 1) - np.roll(hgt, 1, 1)) * 3.0
    dy = (np.roll(hgt, -1, 0) - np.roll(hgt, 1, 0)) * 3.0
    n = np.dstack([-dx, dy, np.ones_like(hgt)])
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    Image.fromarray(((n * 0.5 + 0.5) * 255).astype(np.uint8)).save(f"{out}/normal.jpg", quality=92)
    print(f"{out}: albedo, høyde, normal (512 × 1024)")


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
    {"needles": needles, "flowers": flowers, "flames": flames, "door": door, "hair": lambda p: sheet(p, "hair"), "beard": lambda p: sheet(p, "beard")}[sys.argv[1]](sys.argv[2])
