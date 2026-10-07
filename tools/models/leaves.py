"""Bladteksturene til trærne og buskene: klynger av blader tegnet med PIL,
med alfa utenfor bladene og bladgrønn RGB bak alfaen (så mipnivåene ikke
mørkner mot svart). Skrives til assets/models/trees/. Kjør:

    python3 tools/models/leaves.py
"""

import math
import os
import random

from PIL import Image, ImageDraw, ImageFilter

OUT = "assets/models/trees"
SIZE = 1024


def leaf_polygon(cx, cy, length, width, angle, lobes=0, serrate=False, rng=None):
    """Et blad som polygon: ovalt, med lapper (eik) eller takker (bjørk)."""
    pts = []
    n = 48
    for i in range(n):
        t = i / n
        a = t * 2 * math.pi
        # Grunnform: en oval med spiss ende.
        r = 1.0
        y = -math.cos(a)  # -1 ved stilken, 1 ved spissen
        x = math.sin(a)
        w = width * (0.55 + 0.45 * (1 - abs(y) ** 1.5))
        if lobes:
            w *= 1.0 + 0.35 * math.sin(a * lobes + 0.3) * (1 - abs(y) ** 2)
        if serrate:
            w *= 1.0 + 0.06 * (1 if i % 2 else -1)
        px = x * w
        py = y * length * 0.5 * r
        # Roter.
        ca, sa = math.cos(angle), math.sin(angle)
        pts.append((cx + px * ca - py * sa, cy + px * sa + py * ca))
    return pts


def cluster(name, count, length, width, colors, lobes=0, serrate=False, seed=1, size=SIZE, bg=(70, 120, 45)):
    rng = random.Random(seed)
    img = Image.new("RGBA", (size, size), (*bg, 0))
    draw = ImageDraw.Draw(img)
    # Skygge under hvert blad, så klyngen får dybde.
    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    sd = ImageDraw.Draw(shadow)
    leaves = []
    for _ in range(count):
        cx = rng.uniform(size * 0.08, size * 0.92)
        cy = rng.uniform(size * 0.08, size * 0.92)
        ang = rng.uniform(0, 2 * math.pi)
        ln = length * rng.uniform(0.75, 1.25)
        wd = width * rng.uniform(0.8, 1.2)
        leaves.append((cx, cy, ln, wd, ang))
    for cx, cy, ln, wd, ang in leaves:
        poly = leaf_polygon(cx + 6, cy + 8, ln, wd, ang, lobes, serrate)
        sd.polygon(poly, fill=(0, 0, 0, 110))
    shadow = shadow.filter(ImageFilter.GaussianBlur(5))
    img = Image.alpha_composite(img, shadow)
    draw = ImageDraw.Draw(img)
    for cx, cy, ln, wd, ang in leaves:
        base = rng.choice(colors)
        tone = rng.uniform(0.82, 1.12)
        col = tuple(min(255, int(c * tone)) for c in base)
        poly = leaf_polygon(cx, cy, ln, wd, ang, lobes, serrate)
        draw.polygon(poly, fill=(*col, 255))
        # Midtnerve og sidenerver, litt lysere.
        vein = tuple(min(255, int(c * 1.25 + 10)) for c in col)
        ca, sa = math.cos(ang), math.sin(ang)
        x0, y0 = cx + (ln * 0.48) * sa, cy - (ln * 0.48) * ca
        x1, y1 = cx - (ln * 0.48) * sa, cy + (ln * 0.48) * ca
        draw.line([(x0, y0), (x1, y1)], fill=(*vein, 255), width=max(1, int(wd * 0.06)))
        for k in range(1, 6):
            t = k / 6
            mx, my = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            for s in (-1, 1):
                ex = mx + s * ca * wd * 0.4 * (1 - abs(t - 0.5)) - sa * wd * 0.25
                ey = my + s * sa * wd * 0.4 * (1 - abs(t - 0.5)) + ca * wd * 0.25
                draw.line([(mx, my), (ex, ey)], fill=(*vein, 200), width=1)
    # Bladgrønn RGB bak alfaen: kopier fargen fra nærmeste blad ved å
    # utvide RGB med en uskarp versjon.
    rgb = img.convert("RGB")
    blurred = rgb.filter(ImageFilter.GaussianBlur(12))
    alpha = img.getchannel("A")
    mask = alpha.point(lambda a: 255 if a > 0 else 0)
    merged = Image.composite(rgb, blurred, mask)
    out = Image.merge("RGBA", (*merged.split(), alpha))
    out.save(f"{OUT}/{name}.png")
    print(f"{OUT}/{name}.png")


def needles(name, seed=3, size=SIZE):
    """En grankvist: en brun stilk langs bildet med sidekvister, tett med
    nåler, som kort for grangreinene (bredden er kvistens lengde)."""
    rng = random.Random(seed)
    bg = (38, 70, 40)
    img = Image.new("RGBA", (size, size // 2), (*bg, 0))
    draw = ImageDraw.Draw(img)
    h = size // 2
    greens = [(52, 104, 50), (44, 92, 46), (68, 122, 58), (38, 80, 42), (80, 134, 64)]
    # Silhuetten: en fylt, ujevn kvistform bak nålene, så kortet holder
    # seg tett på avstand (alfatesten spiser tynne streker).
    body = []
    for i in range(60):
        t = i / 59
        x = size * (0.03 + 0.94 * t)
        w = h * 0.24 * (1 - t * 0.6) * (0.75 + 0.25 * math.sin(t * 40 + 1))
        body.append((x, h * 0.5 - w))
    for i in range(60):
        t = 1 - i / 59
        x = size * (0.03 + 0.94 * t)
        w = h * 0.24 * (1 - t * 0.6) * (0.75 + 0.25 * math.cos(t * 37))
        body.append((x, h * 0.5 + w))
    draw.polygon(body, fill=(34, 66, 38, 255))
    # Hovedstilken.
    draw.line([(size * 0.02, h * 0.5), (size * 0.98, h * 0.5)], fill=(72, 50, 30, 255), width=7)
    # Sidekvister, kortere mot spissen, med nåler på begge sider.
    for k in range(34):
        t = k / 34
        x = size * (0.06 + 0.9 * t)
        ln = h * 0.46 * (1 - t * 0.7) * rng.uniform(0.85, 1.1)
        for s in (-1, 1):
            ang = rng.uniform(0.35, 0.6) * s
            ex = x + math.cos(ang) * ln * 0.6
            ey = h * 0.5 + math.sin(ang) * ln
            draw.line([(x, h * 0.5), (ex, ey)], fill=(70, 48, 28, 255), width=3)
            n = int(ln / 4)
            for i in range(n):
                u = i / max(1, n - 1)
                px = x + (ex - x) * u
                py = h * 0.5 + (ey - h * 0.5) * u
                for side in (-1, 1):
                    a2 = ang + side * rng.uniform(0.9, 1.4)
                    nl = rng.uniform(18, 34)
                    col = rng.choice(greens)
                    draw.line([(px, py), (px + math.cos(a2) * nl, py + math.sin(a2) * nl)], fill=(*col, 255), width=2)
    # Nåler rett på hovedstilken også.
    for i in range(320):
        x = rng.uniform(size * 0.03, size * 0.97)
        for side in (-1, 1):
            a2 = side * rng.uniform(0.7, 1.3) + (0 if rng.random() < 0.5 else math.pi)
            nl = rng.uniform(16, 30)
            col = rng.choice(greens)
            draw.line([(x, h * 0.5), (x + math.cos(a2) * nl, h * 0.5 + math.sin(a2) * nl)], fill=(*col, 255), width=2)
    rgb = img.convert("RGB")
    blurred = rgb.filter(ImageFilter.GaussianBlur(10))
    alpha = img.getchannel("A")
    mask = alpha.point(lambda a: 255 if a > 0 else 0)
    merged = Image.composite(rgb, blurred, mask)
    out = Image.merge("RGBA", (*merged.split(), alpha))
    out.save(f"{OUT}/{name}.png")
    print(f"{OUT}/{name}.png")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    cluster("leaves_oak", 34, 190, 120, [(62, 112, 40), (74, 128, 46), (88, 140, 52), (52, 98, 38)], lobes=5, seed=11)
    cluster("leaves_birch", 60, 120, 90, [(110, 160, 60), (130, 176, 70), (96, 146, 54), (150, 186, 80)], serrate=True, seed=12, bg=(100, 150, 60))
    cluster("leaves_bush", 70, 100, 80, [(56, 110, 44), (70, 124, 50), (46, 96, 40), (84, 136, 56)], seed=13)
    needles("needles_spruce", seed=14)
