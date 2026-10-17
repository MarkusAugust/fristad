"""Borgens byggesett (nivå 7 og 8): faner, rustninger, statuer, våpenstativ,
tronen og hjulkrona. Ren Python på
`kit_grav.MB`; eksporteres av `kit_grav.py` sammen med mekanismene
(`castle()`), og motoren setter dem ut etter `prop`-merkene (`PROPS` i
`crypt/mod.rs`; fanen henger på muren og settes ut med `place_mounted`).

| modell | origo | mål |
|---|---|---|
| `kit_banner` | festet i muren (stanga stikker ut mot +z), duken henger ned | stanga 1,2 m langs x, duken 0,9 × 1,9 m |
| `kit_armor` | på gulvet midt under rustningen | 1,85 m høy, vender mot +z |
| `kit_statue` | på gulvet midt under sokkelen | ridder i hogd stein på sokkel, 2,9 m høy, vender mot +z |
| `kit_rack` | på gulvet midt under stativet, ryggen mot −z | våpenstativ 1,8 m bredt med spyd og skjold, 2,3 m høyt |
| `kit_throne` | på gulvet midt under tronen | eiketrone med gyllen kant, 2,5 m høy, vender mot +z |
| `kit_chandelier` | kroken i hvelvet (henger ned) | hjulkrone av eik med lys, 3,1 m fra kroken til hjulet |
"""

import math

from kit_grav import MB, beam, box, candle, circle, cylinder, obox, prism


def kit_banner(w=0.9, drop=1.9):
    """En falmet fane: ei eikestang i to jernringer, og en ullduk med
    svalehale som henger fra stanga."""
    m = MB()
    # Stanga og ringene som holder den mot muren.
    beam(m, (-0.6, 0.0, 0.18), (0.6, 0.0, 0.18), 0.05, 0.05, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 2))
    for x in (-0.5, 0.5):
        box(m, (x - 0.03, -0.04, 0.0), (x + 0.03, 0.04, 0.2), "iron", faces=("+y", "-y", "+x", "-x", "+z"))
    for x in (-0.62, 0.62):
        cylinder(m, (x, 0.18), 0.045, -0.05, 0.05, "iron", n=6, caps=(True, True), smooth=False)
    # Duken: henger fra stanga, med en svalehale nederst. To flater (synlig fra begge sider).
    z = 0.2
    hw = w / 2
    top = -0.03
    bot = top - drop
    pts = [(-hw, top, z), (hw, top, z), (hw, bot, z), (0.0, bot + 0.35, z), (-hw, bot, z)]
    uv = [(x + hw, -(y - top)) for x, y, _ in pts]
    m.poly(pts, uv, "cloth_red", facing=(0.0, 0.0, 1.0))
    m.poly(pts[::-1], uv[::-1], "cloth_red", facing=(0.0, 0.0, -1.0))
    # Bordet øverst og emblemet (et gyllent felt midt på duken).
    box(m, (-hw, top - 0.12, z - 0.004), (hw, top, z + 0.004), "gilt", faces=("+z", "-z", "-y"))
    box(m, (-0.2, top - 1.0, z - 0.003), (0.2, top - 0.5, z + 0.003), "gilt", faces=("+z", "-z"))
    return m


def kit_armor():
    """En tom rustning på et stativ: hjelm, brynje og benskinner av jern
    over en lærkledd dukke, med stativfoten i eik."""
    m = MB()
    # Stativet.
    box(m, (-0.3, 0.0, -0.3), (0.3, 0.08, 0.3), "oak")
    beam(m, (0.0, 0.08, 0.0), (0.0, 1.5, 0.0), 0.07, 0.07, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 1))
    # Beina (benskinner) og støvlene.
    for sx in (-1, 1):
        box(m, (sx * 0.11 - 0.07, 0.0, -0.1), (sx * 0.11 + 0.07, 0.25, 0.14), "leather")
        beam(m, (sx * 0.11, 0.25, 0.0), (sx * 0.11, 0.85, 0.0), 0.15, 0.15, "iron", up=(0.0, 0.0, 1.0))
    # Brynja og brystplata, med beltet.
    box(m, (-0.24, 0.85, -0.14), (0.24, 1.45, 0.14), "iron")
    box(m, (-0.25, 0.98, -0.15), (0.25, 1.06, 0.15), "leather", faces=("+x", "-x", "+z", "-z"))
    # Skuldrene og armene som henger.
    for sx in (-1, 1):
        obox(m, (sx * 0.3, 1.3, 0.0), (0.18, 0.16, 0.24), 0.0, "iron")
        beam(m, (sx * 0.3, 1.3, 0.02), (sx * 0.33, 0.8, 0.08), 0.11, 0.11, "iron", up=(0.0, 0.0, 1.0))
        box(m, (sx * 0.33 - 0.06, 0.66, 0.02), (sx * 0.33 + 0.06, 0.8, 0.14), "leather")
    # Hjelmen med nesebeskytter.
    box(m, (-0.13, 1.5, -0.13), (0.13, 1.72, 0.13), "iron")
    for k in range(6):
        y0 = 1.72 + 0.022 * k
        s = 0.13 * math.sqrt(max(0.0, 1 - (k / 6) ** 2))
        box(m, (-s, y0, -s), (s, y0 + 0.022, s), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    box(m, (-0.02, 1.4, 0.13), (0.02, 1.62, 0.16), "iron")
    return m


def slab(m, pts, z, t, mat, facing=(0.0, 0.0, 1.0)):
    """En tynn plate i xy-planet (skjoldene): forsida ved z + t/2, baksida
    ved z − t/2 og kanten rundt. To sammenfallende flater med hver sin
    normal ble svarte i okklusjonsbakingen."""
    fz = 1.0 if facing[2] >= 0 else -1.0
    front = [(x, y, z + fz * t / 2) for x, y in pts]
    back = [(x, y, z - fz * t / 2) for x, y in pts]
    uv = [(x - pts[0][0], y - pts[0][1]) for x, y in pts]
    m.poly(front, uv, mat, facing=(0.0, 0.0, fz))
    m.poly(back[::-1], uv[::-1], mat, facing=(0.0, 0.0, -fz))
    n = len(pts)
    for k in range(n):
        a, b = front[k], front[(k + 1) % n]
        c, d = back[(k + 1) % n], back[k]
        m.quad(a, b, c, d, mat, facing=(b[1] - a[1], -(b[0] - a[0]), 0.0))


def disc(m, c, r, z, mat, facing, n=12, t=0.04):
    """Et rundskjold: en flat skive i xy-planet med tykkelse."""
    cx, cy = c
    slab(m, [(cx + r * math.cos(2 * math.pi * k / n), cy + r * math.sin(2 * math.pi * k / n)) for k in range(n)], z, t, mat, facing)


def kit_statue():
    """En ridder i hogd stein på en sokkel: skjoldet på venstre arm, sverdet
    med spissen i sokkelen, hjelm med kam. Alt i den hogde steinen
    (`dressed`), sokkelen i murstein."""
    m = MB()
    S = "dressed"
    box(m, (-0.45, 0.0, -0.45), (0.45, 0.5, 0.45), "wall")
    box(m, (-0.5, 0.5, -0.5), (0.5, 0.6, 0.5), S)
    # Beina og kjortelen.
    for sx in (-1, 1):
        box(m, (sx * 0.14 - 0.1, 0.6, -0.12), (sx * 0.14 + 0.1, 1.3, 0.12), S)
    box(m, (-0.3, 1.25, -0.2), (0.3, 1.6, 0.2), S)
    # Brystet med skulderplatene.
    box(m, (-0.3, 1.55, -0.22), (0.3, 2.2, 0.22), S)
    for sx in (-1, 1):
        obox(m, (sx * 0.38, 2.0, 0.0), (0.2, 0.22, 0.3), 0.0, S)
    # Venstre arm med skjoldet (en spiss skjoldform i to flater).
    beam(m, (-0.4, 2.0, 0.0), (-0.46, 1.35, 0.12), 0.14, 0.14, S, up=(0.0, 0.0, 1.0))
    slab(m, [(-0.76, 1.9), (-0.26, 1.9), (-0.26, 1.4), (-0.51, 0.95), (-0.76, 1.4)], 0.22, 0.06, S)
    # Høyre arm med sverdet, spissen i sokkelen.
    beam(m, (0.4, 2.0, 0.0), (0.46, 1.35, 0.2), 0.14, 0.14, S, up=(0.0, 0.0, 1.0))
    box(m, (0.4, 1.2, 0.14), (0.52, 1.45, 0.3), S)
    beam(m, (0.46, 1.6, 0.22), (0.46, 0.6, 0.22), 0.07, 0.03, S, up=(0.0, 0.0, 1.0))
    box(m, (0.3, 1.42, 0.19), (0.62, 1.47, 0.25), S)
    # Hodet med hjelmen og kammen.
    box(m, (-0.14, 2.2, -0.14), (0.14, 2.52, 0.14), S)
    box(m, (-0.16, 2.34, -0.16), (0.16, 2.56, 0.16), S)
    box(m, (-0.03, 2.5, -0.2), (0.03, 2.72, 0.14), S)
    box(m, (-0.02, 2.3, 0.14), (0.02, 2.46, 0.17), S)
    return m


def kit_rack():
    """Våpenstativ av eik mot en mur: to stolper med en øvre og en nedre
    tverrslå, fem spyd på skrå i stativet og to rundskjold hengt på den øvre
    slåa. Ryggen mot −z."""
    m = MB()
    for x in (-0.85, 0.85):
        box(m, (x - 0.25, 0.0, -0.18), (x + 0.25, 0.08, 0.18), "oak")
        beam(m, (x, 0.08, 0.0), (x, 1.75, 0.0), 0.1, 0.1, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 1))
    beam(m, (-0.9, 1.6, 0.0), (0.9, 1.6, 0.0), 0.08, 0.1, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 2))
    beam(m, (-0.9, 0.3, 0.05), (0.9, 0.3, 0.05), 0.08, 0.08, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 2))
    # Spydene: skaft i tre, spiss i jern.
    for k in range(5):
        x = -0.4 + 0.2 * k
        foot = (x, 0.09, 0.2)
        top = (x + 0.03, 2.05, -0.06)
        beam(m, foot, top, 0.035, 0.035, "wood", up=(0.0, 0.0, 1.0))
        beam(m, top, (x + 0.034, 2.3, -0.093), 0.045, 0.012, "iron", up=(0.0, 0.0, 1.0))
    # Rundskjoldene på slåa, med jernbule.
    for x in (-0.65, 0.65):
        disc(m, (x, 1.25), 0.32, 0.1, "oak", (0.0, 0.0, 1.0))
        box(m, (x - 0.06, 1.19, 0.1), (x + 0.06, 1.31, 0.16), "iron")
        box(m, (x - 0.02, 1.45, 0.0), (x + 0.02, 1.65, 0.1), "leather")
    return m


def kit_throne():
    """Tronen: en høy eikestol på en pall, med rød pute og rygg, gyllen kant
    rundt ryggen og knotter på armlenene. Vender mot +z."""
    m = MB()
    box(m, (-0.8, 0.0, -0.65), (0.8, 0.14, 0.75), "oak")
    box(m, (-0.5, 0.14, -0.45), (0.5, 0.52, 0.4), "oak")
    box(m, (-0.46, 0.52, -0.4), (0.46, 0.62, 0.36), "cloth_red")
    # Ryggen med spissen, den gylne kanten og det røde feltet.
    box(m, (-0.5, 0.14, -0.6), (0.5, 2.0, -0.45), "oak")
    pts = [(-0.5, 2.0, -0.6), (0.5, 2.0, -0.6), (0.0, 2.5, -0.6)]
    back = [(-0.5, 2.0, -0.45), (0.5, 2.0, -0.45), (0.0, 2.5, -0.45)]
    uv = [(x + 0.5, y - 2.0) for x, y, _ in pts]
    m.poly(back, uv, "oak", facing=(0.0, 0.0, 1.0))
    m.poly(pts, uv, "oak", facing=(0.0, 0.0, -1.0))
    for a, b in ((pts[0], pts[2]), (pts[2], pts[1])):
        m.quad(a, (a[0], a[1], -0.45), (b[0], b[1], -0.45), b, "oak", facing=(b[1] - a[1], -(b[0] - a[0]), 0.0))
    for x in (-0.5, 0.5):
        box(m, (x - 0.04, 0.14, -0.61), (x + 0.04, 2.0, -0.44), "gilt")
    box(m, (-0.4, 0.75, -0.46), (0.4, 1.85, -0.44), "cloth_red", faces=("+z", "+x", "-x", "+y", "-y"))
    box(m, (-0.12, 1.1, -0.45), (0.12, 1.5, -0.435), "gilt", faces=("+z",))
    # Armlenene med gylne knotter.
    for sx in (-1, 1):
        box(m, (sx * 0.5 - 0.06, 0.52, -0.45), (sx * 0.5 + 0.06, 0.9, 0.4), "oak")
        cylinder(m, (sx * 0.5, 0.36), 0.07, 0.9, 0.98, "gilt", n=8, caps=(True, False))
    return m


def kit_chandelier():
    """Hjulkrone: et hjul av eik med jernbeslag i tre kjettinger fra kroken
    i hvelvet, med tolv lys rundt kanten (`fire_candle` lyser). Origo i
    kroken; hjulet henger 3,1 m under."""
    m = MB()
    Y = -3.1
    cylinder(m, (0.0, 0.0), 0.05, -0.12, 0.0, "iron", n=6, caps=(False, True), smooth=False)
    for a in (math.pi / 2, math.pi / 2 + 2.094, math.pi / 2 + 4.189):
        end = (0.62 * math.cos(a), Y + 0.08, 0.62 * math.sin(a))
        n = 22
        for k in range(n):
            t0, t1 = k / n, (k + 1) / n
            p0 = (end[0] * t0, -0.12 + (end[1] + 0.12) * t0, end[2] * t0)
            p1 = (end[0] * t1, -0.12 + (end[1] + 0.12) * t1, end[2] * t1)
            up = (0.0, 0.0, 1.0) if k % 2 else (math.cos(a), 0.0, math.sin(a))
            beam(m, p0, p1, 0.05 if k % 2 else 0.02, 0.02 if k % 2 else 0.05, "iron", up=up)
    ring = circle((0.0, 0.0), 0.75, 16)
    prism(m, ring, Y - 0.08, Y, "oak", cap_top=True, cap_bottom=True, smooth=False)
    cylinder(m, (0.0, 0.0), 0.76, Y - 0.09, Y + 0.01, "iron", n=16, caps=(False, False), smooth=False)
    for k in range(12):
        a = 2 * math.pi * k / 12
        x, z = 0.62 * math.cos(a), 0.62 * math.sin(a)
        cylinder(m, (x, z), 0.055, Y, Y + 0.03, "iron", n=6, caps=(True, False), smooth=False)
        candle(m, x, z, Y + 0.03, h=0.24, r=0.035)
    return m


def castle():
    return {
        "kit_banner": kit_banner(),
        "kit_armor": kit_armor(),
        "kit_statue": kit_statue(),
        "kit_rack": kit_rack(),
        "kit_throne": kit_throne(),
        "kit_chandelier": kit_chandelier(),
    }
