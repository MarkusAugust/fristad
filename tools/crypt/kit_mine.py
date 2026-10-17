"""Gruvas byggesett (nivå 5): tømmerstiver, skinner, malmvogn, gruvelampe,
krystaller, heisen og vinsjen med heistårnet. Ren Python på `kit_grav.MB`;
eksporteres av `kit_grav.py` sammen med mekanismene (`mine()`), og motoren
setter dem ut etter `prop`-merkene (`PROPS` i `crypt/mod.rs`) og heisen
etter `lift`-merket (`KIT_LIFT`, `KIT_WINCH`, kjettingene i `KIT_CHAIN`).

| modell | origo | mål |
|---|---|---|
| `kit_strut` | på gulvet midt mellom stolpene | 3,5 m bred (stolpene ±1,55), 2,9 m høy, overliggeren langs x |
| `kit_rail` | på gulvet midt på | 2 m langs z, sporvidde 1 m |
| `kit_cart` | på gulvet midt under vogna | 1,5 × 0,9 m, 1,1 m høy, langs z |
| `kit_lamp` | kroken (henger ned) | 0,34 m |
| `kit_crystal` | på gulvet midt i klyngen | ~1,2 m bred, opptil 1,4 m høy |
| `kit_lift` | midt på oversida av dekket | 2,8 × 2,8 m (`LIFT_W`), åpen mot −x, styrestangen i hjørnet (+x, +z) |
| `kit_winch` | på gulvet midt over sjakta | heistårnet over hullet (3 m), bommen 4 m oppe, vinsjen 2,6 m mot −z |
"""

import math

from kit_grav import MB, beam, box, cylinder, mul, norm, obox, rubble, strap, sub

LIFT_W = 2.8
HEAD_Y = 4.0  # bommen i heistårnet over gulvet oppe
WINCH_Z = -2.6  # vinsjen står så langt fra sjaktmidten (mot −z)


def hex_prism(m, base, tip, r, mat, taper=0.35):
    """Et sekskantet krystallprisme fra `base` til `tip` (spissen), radius r
    ved foten, smalere mot toppen, med en pyramide ytterst."""
    d = norm(sub(tip, base))
    L = math.dist(base, tip)
    ref = (0.0, 1.0, 0.0) if abs(d[1]) < 0.9 else (1.0, 0.0, 0.0)
    sx = norm((d[1] * ref[2] - d[2] * ref[1], d[2] * ref[0] - d[0] * ref[2], d[0] * ref[1] - d[1] * ref[0]))
    sz = (d[1] * sx[2] - d[2] * sx[1], d[2] * sx[0] - d[0] * sx[2], d[0] * sx[1] - d[1] * sx[0])
    body = L * 0.78
    ring0, ring1 = [], []
    for k in range(6):
        a = 2 * math.pi * k / 6
        off = lambda rr: (sx[0] * rr * math.cos(a) + sz[0] * rr * math.sin(a), sx[1] * rr * math.cos(a) + sz[1] * rr * math.sin(a), sx[2] * rr * math.cos(a) + sz[2] * rr * math.sin(a))
        o0, o1 = off(r), off(r * (1 - taper))
        ring0.append((base[0] + o0[0], base[1] + o0[1], base[2] + o0[2]))
        ring1.append((base[0] + d[0] * body + o1[0], base[1] + d[1] * body + o1[1], base[2] + d[2] * body + o1[2]))
    c = (base[0] + d[0] * body * 0.5, base[1] + d[1] * body * 0.5, base[2] + d[2] * body * 0.5)
    for k in range(6):
        a, b = ring0[k], ring0[(k + 1) % 6]
        cc, dd = ring1[(k + 1) % 6], ring1[k]
        mid = ((a[0] + cc[0]) / 2 - c[0], (a[1] + cc[1]) / 2 - c[1], (a[2] + cc[2]) / 2 - c[2])
        m.quad(a, b, cc, dd, mat, facing=mid, uv=[(0, 0), (r, 0), (r, body), (0, body)])
        m.tri(dd, cc, tip, mat, facing=mid, uv=[(0, 0), (r, 0), (r * 0.5, L - body)])


def kit_strut(w=3.5, h=2.9, post=0.28):
    """Tømmerstiver i stollen: to stolper og en overligger av grovt tømmer,
    med kiler mellom overliggeren og taket, og to skråband."""
    m = MB()
    hx = w / 2 - post / 2
    for sx in (-1, 1):
        beam(m, (sx * hx, 0.0, 0.0), (sx * hx, h - post, 0.0), post, post, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 0 if sx < 0 else 3))
    beam(m, (-w / 2, h - post / 2, 0.0), (w / 2, h - post / 2, 0.0), post, post, "wood", strip=("timber", 1))
    for sx in (-1, 1):
        a = (sx * (hx - post / 2), h - post - 0.9, 0.0)
        b = (sx * (hx - 0.9), h - post, 0.0)
        beam(m, a, b, 0.12, 0.12, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 2))
        # Kilen oppå overliggeren mot berget.
        obox(m, (sx * (hx - 0.3), h, 0.0), (0.4, 0.14, post + 0.06), 0.0, "wood")
    return m


def kit_rail(L=2.0, gauge=1.0):
    """Skinner på sviller: et stykke på 2 m langs z."""
    m = MB()
    for k in range(4):
        z = -L / 2 + L / 8 + L / 4 * k
        box(m, (-gauge / 2 - 0.18, 0.0, z - 0.09), (gauge / 2 + 0.18, 0.1, z + 0.09), "wood", faces=("+y", "+x", "-x", "+z", "-z"))
    for sx in (-1, 1):
        x = sx * gauge / 2
        box(m, (x - 0.035, 0.1, -L / 2), (x + 0.035, 0.19, L / 2), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    return m


def kit_cart(L=1.5, W=0.9, H=1.1):
    """Malmvogn: en jernbeslått trekasse på fire hjul, med malm i."""
    m = MB()
    # Hjulene (sylindre lagt på siden: bygd som korte prismer langs x).
    for sz in (-1, 1):
        for sx in (-1, 1):
            cx, cz = sx * (W / 2 + 0.02), sz * (L / 2 - 0.3)
            ring = [(0.19 * math.cos(a), 0.19 * math.sin(a)) for a in (2 * math.pi * k / 10 for k in range(10))]
            pts0 = [(cx - sx * 0.05, 0.19 + y, cz + z) for y, z in ring]
            pts1 = [(cx + sx * 0.05, 0.19 + y, cz + z) for y, z in ring]
            for k in range(10):
                a, b, c, d = pts0[k], pts0[(k + 1) % 10], pts1[(k + 1) % 10], pts1[k]
                mid = ((a[1] + c[1]) / 2 - 0.19, (a[2] + c[2]) / 2 - cz)
                m.quad(a, b, c, d, "iron", facing=(0.0, mid[0], mid[1]), smooth=True)
            m.poly(pts1 if sx > 0 else pts0[::-1], [(p[1], p[2]) for p in (pts1 if sx > 0 else pts0[::-1])], "iron", facing=(sx, 0, 0))
    beam(m, (-W / 2 - 0.08, 0.19, -L / 2 + 0.3), (W / 2 + 0.08, 0.19, -L / 2 + 0.3), 0.05, 0.05, "iron")
    beam(m, (-W / 2 - 0.08, 0.19, L / 2 - 0.3), (W / 2 + 0.08, 0.19, L / 2 - 0.3), 0.05, 0.05, "iron")
    # Kassen: planker, litt videre oppe.
    y0, y1 = 0.3, H
    lo, hi = (-W / 2, y0, -L / 2), (W / 2, y1, L / 2)
    box(m, lo, hi, "oak", faces=("+x", "-x", "+z", "-z", "-y"))
    for z in (-L / 2 + 0.03, L / 2 - 0.03):
        strap(m, (-W / 2, 0.42, z), (W / 2, 0.42, z), 0.07, 0.012, (0.0, 0.0, 1.0 if z > 0 else -1.0))
        strap(m, (-W / 2, H - 0.1, z), (W / 2, H - 0.1, z), 0.07, 0.012, (0.0, 0.0, 1.0 if z > 0 else -1.0))
    for x in (-W / 2 - 0.006, W / 2 + 0.006):
        strap(m, (x, 0.42, -L / 2), (x, 0.42, L / 2), 0.07, 0.012, (1.0 if x > 0 else -1.0, 0.0, 0.0))
        strap(m, (x, H - 0.1, -L / 2), (x, H - 0.1, L / 2), 0.07, 0.012, (1.0 if x > 0 else -1.0, 0.0, 0.0))
    # Malmen: en haug som stikker opp av kassen.
    rubble(m, 0.0, 0.0, H - 0.22, r=0.5, h=0.32, seed=4)
    # Innsida av kassen over malmen.
    box(m, (-W / 2 + 0.04, H - 0.25, -L / 2 + 0.04), (W / 2 - 0.04, H, L / 2 - 0.04), "oak", faces=("+x", "-x", "+z", "-z"))
    return m


def kit_lamp():
    """Gruvelampe: en jernlykt med glødende glass, hengende i en krok (origo
    i kroken, lykta henger ned)."""
    m = MB()
    beam(m, (0.0, 0.0, 0.0), (0.0, -0.08, 0.0), 0.02, 0.02, "iron", up=(0.0, 0.0, 1.0))
    # Toppen (hette), hjørnestolper og bunnen; glasset innenfor.
    box(m, (-0.1, -0.12, -0.1), (0.1, -0.08, 0.1), "iron")
    for sx in (-1, 1):
        for sz in (-1, 1):
            box(m, (sx * 0.085 - 0.012, -0.34, sz * 0.085 - 0.012), (sx * 0.085 + 0.012, -0.12, sz * 0.085 + 0.012), "iron", faces=("+x", "-x", "+z", "-z"))
    box(m, (-0.1, -0.38, -0.1), (0.1, -0.34, 0.1), "iron")
    box(m, (-0.075, -0.335, -0.075), (0.075, -0.125, 0.075), "fire_lamp", faces=("+x", "-x", "+z", "-z"))
    return m


def kit_crystal(seed=3):
    """En klynge lysende krystaller på en knaus av berg."""
    import random

    rng = random.Random(seed)
    m = MB()
    rubble(m, 0.0, 0.0, 0.0, r=0.7, h=0.25, seed=seed)
    for k in range(7):
        a = 2 * math.pi * k / 7 + rng.random() * 0.5
        rr = 0.1 + 0.3 * rng.random()
        base = (rr * math.cos(a), 0.05, rr * math.sin(a))
        h = 0.5 + 0.9 * rng.random() if k < 3 else 0.3 + 0.4 * rng.random()
        lean = 0.15 + 0.35 * rng.random()
        tip = (base[0] + lean * h * math.cos(a), base[1] + h, base[2] + lean * h * math.sin(a))
        hex_prism(m, base, tip, 0.07 + 0.08 * rng.random(), "magic_crystal")
    return m


def kit_lift(w=LIFT_W):
    """Heisen: plankedekke med jernramme, tverrbjelker under, rekkverk på
    tre sider (åpen mot −x), jernøyer i hjørnene til kjettingene, og en
    styrestang (bremsen) i hjørnet (+x, +z) som du bruker."""
    m = MB()
    h = w / 2
    box(m, (-h, -0.1, -h), (h, 0.0, h), "oak")
    for x in (-h + 0.2, h - 0.2):
        beam(m, (x, -0.22, -h), (x, -0.22, h), 0.2, 0.24, "wood", strip=("timber", 0))
    for z in (-h + 0.2, 0.0, h - 0.2):
        beam(m, (-h, -0.17, z), (h, -0.17, z), 0.16, 0.14, "wood", strip=("timber", 2))
    # Jernrammen rundt kanten.
    for z in (-h, h):
        strap(m, (-h, 0.005, z - (0.0 if z < 0 else 0.0)), (h, 0.005, z), 0.12, 0.012, (0.0, 1.0, 0.0))
    for x in (-h, h):
        strap(m, (x, 0.005, -h), (x, 0.005, h), 0.12, 0.012, (0.0, 1.0, 0.0))
    # Rekkverket: stolper i hjørnene og midt på, håndlist og knelist.
    posts = [(-h + 0.08, -h + 0.08), (h - 0.08, -h + 0.08), (h - 0.08, h - 0.08), (-h + 0.08, h - 0.08), (h - 0.08, 0.0)]
    for x, z in posts:
        beam(m, (x, 0.0, z), (x, 1.05, z), 0.09, 0.09, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 3))
    rails = [((-h + 0.08, -h + 0.08), (h - 0.08, -h + 0.08)), ((h - 0.08, -h + 0.08), (h - 0.08, h - 0.08)), ((h - 0.08, h - 0.08), (-h + 0.08, h - 0.08))]
    for (x0, z0), (x1, z1) in rails:
        for y in (0.55, 1.0):
            beam(m, (x0, y, z0), (x1, y, z1), 0.06, 0.07, "wood", strip=("timber", 1))
    # Kjettingøyene.
    for sx in (-1, 1):
        for sz in (-1, 1):
            x, z = sx * (h - 0.2), sz * (h - 0.2)
            box(m, (x - 0.08, 0.0, z - 0.08), (x + 0.08, 0.17, z + 0.08), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    # Styrestangen: en jernstang med trehåndtak ved stolpen i hjørnet (+x, +z).
    x, z = h - 0.3, h - 0.3
    box(m, (x - 0.1, 0.0, z - 0.1), (x + 0.1, 0.12, z + 0.1), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    beam(m, (x, 0.12, z), (x - 0.25, 1.15, z - 0.25), 0.04, 0.04, "iron", up=(0.0, 0.0, 1.0))
    beam(m, (x - 0.25, 1.15, z - 0.25), (x - 0.31, 1.4, z - 0.31), 0.07, 0.07, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 1))
    return m


def kit_winch(hole=3.0):
    """Heistårnet og vinsjen (`KIT_WINCH`, origo på gulvet midt over sjakta):
    to stolper utenfor hullet (langs x), bommen `HEAD_Y` oppe med
    trinsehjulet under, og vinsjen med trommel og sveiv `WINCH_Z` mot −z.
    Kjettingene fra heisen går opp til bommen."""
    m = MB()
    px = hole / 2 + 0.35
    for sx in (-1, 1):
        beam(m, (sx * px, 0.0, 0.0), (sx * px, HEAD_Y, 0.0), 0.32, 0.32, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 0 if sx < 0 else 3))
        # Skråstøtter mot −z.
        beam(m, (sx * px, 0.0, -1.6), (sx * px, HEAD_Y - 0.6, -0.16), 0.16, 0.16, "wood", up=(1.0, 0.0, 0.0), strip=("timber", 2))
        obox(m, (sx * px, 0.0, 0.0), (0.6, 0.12, 0.6), 0.0, "dressed")
    beam(m, (-px - 0.3, HEAD_Y + 0.17, 0.0), (px + 0.3, HEAD_Y + 0.17, 0.0), 0.34, 0.34, "wood", strip=("timber", 1))
    # Trinsehjulet under bommen: en skive med aksel langs x.
    ring = [(0.5 * math.cos(a), 0.5 * math.sin(a)) for a in (2 * math.pi * k / 16 for k in range(16))]
    for x0, x1, mat, r in ((-0.06, 0.06, "iron", 1.0), (-0.3, 0.3, "iron", 0.08)):
        p0 = [(x0, HEAD_Y - 0.55 + y * r, z * r) for y, z in ring]
        p1 = [(x1, HEAD_Y - 0.55 + y * r, z * r) for y, z in ring]
        for k in range(16):
            a, b, c, d = p0[k], p0[(k + 1) % 16], p1[(k + 1) % 16], p1[k]
            m.quad(a, b, c, d, mat, facing=(0.0, (a[1] + c[1]) / 2 - (HEAD_Y - 0.55), (a[2] + c[2]) / 2), smooth=True)
        m.poly(p1, [(p[1], p[2]) for p in p1], mat, facing=(1, 0, 0))
        m.poly(p0[::-1], [(p[1], p[2]) for p in p0[::-1]], mat, facing=(-1, 0, 0))
    # Vinsjen: to bukker, trommelen mellom, sveiva på sida.
    wz = WINCH_Z
    for sx in (-1, 1):
        obox(m, (sx * 0.9, 0.0, wz), (0.3, 1.1, 0.5), 0.0, "wood")
    ring = [(0.3 * math.cos(a), 0.3 * math.sin(a)) for a in (2 * math.pi * k / 12 for k in range(12))]
    p0 = [(-0.75, 0.85 + y, wz + z) for y, z in ring]
    p1 = [(0.75, 0.85 + y, wz + z) for y, z in ring]
    for k in range(12):
        a, b, c, d = p0[k], p0[(k + 1) % 12], p1[(k + 1) % 12], p1[k]
        m.quad(a, b, c, d, "wood", facing=(0.0, (a[1] + c[1]) / 2 - 0.85, (a[2] + c[2]) / 2 - wz), uv=[(0, 0), (0.16, 0), (0.16, 1.5), (0, 1.5)], smooth=True)
    beam(m, (-1.15, 0.85, wz), (1.15, 0.85, wz), 0.07, 0.07, "iron")
    beam(m, (1.15, 0.85, wz), (1.15, 0.85, wz + 0.45), 0.05, 0.05, "iron", up=(0.0, 1.0, 0.0))
    beam(m, (1.15, 0.85, wz + 0.45), (1.45, 0.85, wz + 0.45), 0.06, 0.06, "wood", strip=("timber", 1))
    # Kjettingen fra trommelen opp til bommen og over trinsa.
    for k in range(int((HEAD_Y - 1.0) / 0.2)):
        y0 = 1.1 + k * 0.2
        a, b = (0.05, 0.016) if k % 2 == 0 else (0.016, 0.05)
        box(m, (-a, y0, wz - b), (a, y0 + 0.23, wz + b), "iron", faces=("+x", "-x", "+z", "-z"))
    return m


def mine():
    return {
        "kit_strut": kit_strut(),
        "kit_rail": kit_rail(),
        "kit_cart": kit_cart(),
        "kit_lamp": kit_lamp(),
        "kit_crystal": kit_crystal(),
        "kit_lift": kit_lift(),
        "kit_winch": kit_winch(),
    }
