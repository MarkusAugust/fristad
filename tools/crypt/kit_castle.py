"""Borgens byggesett (nivå 7 og 8): faner og rustninger. Ren Python på
`kit_grav.MB`; eksporteres av `kit_grav.py` sammen med mekanismene
(`castle()`), og motoren setter dem ut etter `prop`-merkene (`PROPS` i
`crypt/mod.rs`; fanen henger på muren og settes ut med `place_mounted`).

| modell | origo | mål |
|---|---|---|
| `kit_banner` | festet i muren (stanga stikker ut mot +z), duken henger ned | stanga 1,2 m langs x, duken 0,9 × 1,9 m |
| `kit_armor` | på gulvet midt under rustningen | 1,85 m høy, vender mot +z |
"""

import math

from kit_grav import MB, beam, box, cylinder, obox


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


def castle():
    return {
        "kit_banner": kit_banner(),
        "kit_armor": kit_armor(),
    }
