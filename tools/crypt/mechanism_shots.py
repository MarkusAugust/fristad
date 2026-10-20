"""Bildene av byggesettets mekanismer og pynt til eieren, satt sammen
(eieren 19. oktober: «Det er 394 deler til godkjenning fortsatt. De
fleste løse.»; han dømmer det han ser satt sammen, ikke løse deler).

Kortene står i `assets/catalog/modules.json` under `mekanismer`: navnet,
modellene og kameraene. Et kamera med `nivaa` «mekanismer» står i
prøvenivået `assets/levels/nivaa_mekanismer.json` (moduler fra
gravkjelleren med tingene satt ut), et tall i det gamle kryptnivået (der
tingen trenger en kløft, en kanal eller en sjakt). Hvert kort blir ett
bilde: bildene fra nivået øverst og hver modell for seg fra studioet
under, med navnet, så det synes hva som er hva (eller et nærbilde fra
nivået, `naer`, for det som henger under festet og vannet).

    python3 tools/crypt/mechanism_shots.py [kort ...]

Skriver `target/qa/review/mekanismer/<kort>.png`.
"""

import json
import os
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, ROOT)
OUT = os.path.join(ROOT, "target", "qa", "review", "mekanismer")
SNAP = os.path.join(ROOT, "target", "release", "snapshot")
SIZE = (800, 560)


def cards():
    with open(os.path.join(ROOT, "assets", "catalog", "modules.json")) as f:
        return json.load(f)["mekanismer"]["kort"]


def font(h):
    for f in ("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(f, h)
        except OSError:
            pass
    return ImageFont.load_default()


def render(todo):
    """Alle bildene, én kjøring av snapshot per nivå."""
    by_level = {}
    for card in todo:
        named = [(f"{card['id']}_{k}", s) for k, s in enumerate(card["bilder"])] + [(f"naer_{m}", s) for m, s in card.get("naer", {}).items()]
        for stem, s in named:
            f = os.path.join(OUT, "skudd", f"{stem}.png")
            by_level.setdefault(s["nivaa"], []).append(f"{f} {s['eye'][0]},{s['eye'][1]},{s['eye'][2]} {s['yaw']} {s['pitch']}")
    os.makedirs(os.path.join(OUT, "skudd"), exist_ok=True)
    for level, lines in by_level.items():
        shots = os.path.join(OUT, f"shots_{level}.txt")
        with open(shots, "w") as f:
            f.write("\n".join(lines) + "\n")
        where = ["--level-doc", level, "--level", "1"] if isinstance(level, str) else ["--level", str(level)]
        subprocess.run([SNAP, *where, "--film", "--size", f"{SIZE[0]}x{SIZE[1]}", "--frames", "2", "--gfx", "lights=16", "--shots", shots], cwd=ROOT, check=True, capture_output=True)


def studio(name):
    """Modellen fra studioet (bufret av godkjenningssiden, `review.shots`)."""
    from tools.qa import review

    path = review.locate(name, "modell")
    files = review.shots("modell", name, path, review.version(path)) if path else []
    pick = [f for f in files if f.endswith("corner.png")] or files
    return pick[0] if pick else None


def sheet(card, out):
    """Tittelen, bildene fra nivået (tre per rad) og studiobildene under."""
    ims = [Image.open(os.path.join(OUT, "skudd", f"{card['id']}_{k}.png")).convert("RGB") for k in range(len(card["bilder"]))]
    per = 3 if len(ims) != 4 else 2
    w, h = SIZE
    if per == 2:
        w, h = 1208 // 2 - 4, round((1208 // 2 - 4) * SIZE[1] / SIZE[0])
    rows = (len(ims) + per - 1) // per
    width = per * w + (per - 1) * 8
    tw, th = 228, 160
    # Det som henger under festet (fanene, lysekrona, lampa) og vannet
    # synes ikke i studioet: et nærbilde fra nivået i stedet (`naer`).
    near = lambda n: os.path.join(OUT, "skudd", f"naer_{n}.png")
    thumbs = [(n, near(n) if n in card.get("naer", {}) else studio(n)) for n in card["modeller"]]
    tper = max(1, (width + 8) // (tw + 8))
    trows = (len(thumbs) + tper - 1) // tper
    head = 60
    img = Image.new("RGB", (width, head + rows * (h + 8) + trows * (th + 34 + 8)), (24, 26, 30))
    d = ImageDraw.Draw(img)
    d.text((14, 14), card["name"], fill=(240, 236, 226), font=font(30))
    for k, im in enumerate(ims):
        img.paste(im.resize((w, h)), ((k % per) * (w + 8), head + (k // per) * (h + 8)))
    y0 = head + rows * (h + 8)
    for k, (name, f) in enumerate(thumbs):
        x, y = (k % tper) * (tw + 8), y0 + (k // tper) * (th + 34 + 8)
        if f:
            t = Image.open(f).convert("RGB")
            if t.width / t.height > tw / th:
                t = t.crop(((t.width - round(t.height * tw / th)) // 2, 0, (t.width + round(t.height * tw / th)) // 2, t.height))
            t.thumbnail((tw, th))
            img.paste(t, (x + (tw - t.width) // 2, y))
        d.text((x + 6, y + th + 6), name, fill=(220, 214, 200), font=font(18))
    img.save(out)


def main():
    only = sys.argv[1:]
    todo = [c for c in cards() if not only or c["id"] in only]
    os.makedirs(OUT, exist_ok=True)
    render(todo)
    for c in todo:
        out = os.path.join(OUT, f"{c['id']}.png")
        sheet(c, out)
        print(out)


if __name__ == "__main__":
    main()
