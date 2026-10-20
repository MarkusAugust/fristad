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
| `kit_lift` | midt på oversida av dekket | 2,8 × 2,8 m (`LIFT_W`), åpen mot −x, bremsespaken i hjørnet (+x, +z) |
| `kit_winch` | på gulvet midt over sjakta | heistårnet over hullet (3 m), bommen 4 m oppe med hengebjelkene i 3,45 m, vinsjen 2,6 m mot −z |
"""

import math

from kit_grav import MB, beam, box, cylinder, mul, norm, obox, rubble, strap, sub

LIFT_W = 2.8
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


def cone(m, c, r, y0, y1, mat="rock", n=8, seed=0):
    """En kjegle av berg fra ringen (radius r ved y0) til spissen ved y1:
    stalagmitt (y1 > y0) eller dryppstein fra taket (y1 < y0). Litt ujevn."""
    import random

    rng = random.Random(seed)
    ring = []
    for k in range(n):
        a = 2 * math.pi * k / n
        rr = r * (0.8 + 0.4 * rng.random())
        ring.append((c[0] + rr * math.cos(a), y0, c[1] + rr * math.sin(a)))
    tip = (c[0] + (rng.random() - 0.5) * 0.3 * r, y1, c[1] + (rng.random() - 0.5) * 0.3 * r)
    up = y1 > y0
    for k in range(n):
        a, b = ring[k], ring[(k + 1) % n]
        mid = ((a[0] + b[0]) / 2 - c[0], 0.0, (a[2] + b[2]) / 2 - c[1])
        if up:
            m.tri(a, b, tip, mat, facing=mid, uv=[(0, 0), (r, 0), (r * 0.5, abs(y1 - y0))])
        else:
            m.tri(b, a, tip, mat, facing=mid, uv=[(0, 0), (r, 0), (r * 0.5, abs(y1 - y0))])


def spire(m, base, h, r, mat="magic_crystal", lean=(0.0, 0.0), seed=0):
    """Ett stort krystallspir (sekskantet prisme) fra `base`, `h` høyt, med
    noen mindre krystaller rundt foten. `lean` (dx, dz) per meter høyde."""
    import random

    rng = random.Random(seed)
    tip = (base[0] + lean[0] * h, base[1] + h, base[2] + lean[1] * h)
    hex_prism(m, base, tip, r, mat)
    for k in range(5):
        a = 2 * math.pi * k / 5 + rng.random() * 0.6
        rr = r * (0.9 + 0.6 * rng.random())
        b = (base[0] + rr * math.cos(a), base[1], base[2] + rr * math.sin(a))
        hh = h * (0.2 + 0.25 * rng.random())
        t = (b[0] + 0.3 * hh * math.cos(a), b[1] + hh, b[2] + 0.3 * hh * math.sin(a))
        hex_prism(m, b, t, r * (0.3 + 0.2 * rng.random()), mat)


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
    """Heisen: et plankedekke med en ramme av tømmer rundt, rekkverk på tre
    sider (åpen mot −x: stolper i hjørnene og én håndlist), jernøyer i
    hjørnene til kjettingene, og bremsespaken (én trestang i et jernbeslag)
    ved hjørnet (+x, +z), der du bruker den (eieren 19. oktober: «heisen er
    veldig merkelig. mye rare stenger og spaker»)."""
    m = MB()
    h = w / 2
    box(m, (-h, -0.1, -h), (h, 0.0, h), "oak")
    # Rammen rundt dekket og to bærebjelker under.
    for z in (-h + 0.09, h - 0.09):
        beam(m, (-h, -0.22, z), (h, -0.22, z), 0.18, 0.24, "wood", strip=("timber", 0))
    for x in (-h + 0.09, h - 0.09):
        beam(m, (x, -0.22, -h + 0.18), (x, -0.22, h - 0.18), 0.18, 0.24, "wood", strip=("timber", 2))
    for x in (-h / 3, h / 3):
        beam(m, (x, -0.2, -h + 0.18), (x, -0.2, h - 0.18), 0.14, 0.18, "wood", strip=("timber", 2))
    # Rekkverket: fire hjørnestolper og håndlista på tre sider.
    c = h - 0.08
    for x, z in ((-c, -c), (c, -c), (c, c), (-c, c)):
        beam(m, (x, 0.0, z), (x, 1.05, z), 0.1, 0.1, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 3))
    for (x0, z0), (x1, z1) in (((-c, -c), (c, -c)), ((c, -c), (c, c)), ((c, c), (-c, c))):
        beam(m, (x0, 1.0, z0), (x1, 1.0, z1), 0.08, 0.08, "wood", strip=("timber", 1))
    # Kjettingøyene.
    for sx in (-1, 1):
        for sz in (-1, 1):
            x, z = sx * (h - 0.2), sz * (h - 0.2)
            box(m, (x - 0.08, 0.0, z - 0.08), (x + 0.08, 0.17, z + 0.08), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    # Bremsespaken: beslaget på dekket og en rett trestang, litt på skrå.
    x, z = h - 0.3, h - 0.3
    box(m, (x - 0.1, 0.0, z - 0.1), (x + 0.1, 0.1, z + 0.1), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    beam(m, (x, 0.1, z), (x - 0.12, 1.2, z - 0.12), 0.07, 0.07, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 1))
    return m


HEAD = 3.45  # undersida av hengebjelkene, der kjettingene ender (`LIFT_HEAD`)


def kit_winch(hole=3.0):
    """Heistårnet og vinsjen (`KIT_WINCH`, origo på gulvet midt over sjakta):
    to kraftige stolper utenfor hullet (langs x) med knestøtter, bommen
    over dem, to høye hengebjelker under bommen der de fire kjettingene
    fra heisen ender, og en bindbjelke mellom dem (`HEAD`, motoren: `LIFT_HEAD`). Vinsjen står `WINCH_Z` mot
    −z: to bukker med trommelen og sveiva, og tauet fra trommelen opp til
    bommen. Ingen trinsehjul eller løse stenger."""
    m = MB()
    px = hole / 2 + 0.35
    hang_t = 0.66  # hengebjelkene er høye tømmer rett under bommen
    top = HEAD + hang_t + 0.34  # oversida av bommen
    for sx in (-1, 1):
        beam(m, (sx * px, 0.0, 0.0), (sx * px, top, 0.0), 0.34, 0.34, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 0 if sx < 0 else 3))
        obox(m, (sx * px, 0.0, 0.0), (0.6, 0.12, 0.6), 0.0, "dressed")
        # Knestøtta mellom stolpen og bommen.
        beam(m, (sx * px, top - 1.0, 0.0), (sx * (px - 0.75), top - 0.3, 0.0), 0.14, 0.14, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 2))
    # Bommen og hengebjelkene under den (langs z, over kjettingene).
    beam(m, (-px - 0.25, top - 0.17, 0.0), (px + 0.25, top - 0.17, 0.0), 0.34, 0.34, "wood", strip=("timber", 1))
    cz = LIFT_W / 2 - 0.2
    for sx in (-1, 1):
        x = sx * cz
        beam(m, (x, HEAD + hang_t / 2, -cz - 0.15), (x, HEAD + hang_t / 2, cz + 0.15), 0.16, hang_t, "wood", strip=("timber", 2))
        for sz in (-1, 1):
            box(m, (x - 0.07, HEAD - 0.06, sz * cz - 0.07), (x + 0.07, HEAD, sz * cz + 0.07), "iron", faces=("-y", "+x", "-x", "+z", "-z"))
    # Bindbjelken mellom hengebjelkene, midt over sjakta.
    beam(m, (-cz + 0.08, HEAD + 0.1, 0.0), (cz - 0.08, HEAD + 0.1, 0.0), 0.16, 0.2, "wood", strip=("timber", 1))
    # Vinsjen: to bukker, trommelen mellom, sveiva på sida.
    wz = WINCH_Z
    for sx in (-1, 1):
        obox(m, (sx * 0.9, 0.0, wz), (0.3, 1.1, 0.5), 0.0, "wood")
    ring = [(0.3 * math.cos(a), 0.3 * math.sin(a)) for a in (2 * math.pi * k / 12 for k in range(12))]
    p0 = [(-0.75, 0.85 + y, wz + z) for y, z in ring]
    p1 = [(0.75, 0.85 + y, wz + z) for y, z in ring]
    for k in range(12):
        a, b, c, d = p0[k], p0[(k + 1) % 12], p1[(k + 1) % 12], p1[k]
        L = math.dist(a, b)
        m.quad(a, b, c, d, "wood", facing=(0.0, (a[1] + c[1]) / 2 - 0.85, (a[2] + c[2]) / 2 - wz), uv=[(0, 0), (L, 0), (L, 1.5), (0, 1.5)], smooth=True)
    for q in (p0, p1):
        m.poly(q, [(p[1], p[2]) for p in q], "wood", facing=(-1 if q is p0 else 1, 0, 0))
    beam(m, (-1.15, 0.85, wz), (1.15, 0.85, wz), 0.07, 0.07, "iron")
    beam(m, (1.15, 0.85, wz), (1.15, 0.5, wz), 0.06, 0.06, "iron", up=(0.0, 0.0, 1.0))
    beam(m, (1.15, 0.5, wz), (1.45, 0.5, wz), 0.07, 0.07, "wood", strip=("timber", 1))
    # Kjettingen fra trommelen opp til bommen: ledd av smijern på skrå.
    a, b = (0.0, 1.15, wz + 0.05), (0.0, top - 0.34, -0.12)
    d = norm(sub(b, a))
    n = int(math.dist(a, b) / 0.2)
    for k in range(n):
        p0 = tuple(a[i] + d[i] * 0.2 * k for i in range(3))
        p1 = tuple(a[i] + d[i] * (0.2 * k + 0.23) for i in range(3))
        w, t = (0.1, 0.03) if k % 2 == 0 else (0.03, 0.1)
        beam(m, p0, p1, w, t, "iron", up=(0.0, 0.0, 1.0) if abs(d[2]) < 0.9 else (0.0, 1.0, 0.0))
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
