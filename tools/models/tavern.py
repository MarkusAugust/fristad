"""Vertshuset i den malte stilen (docs/ny-stil.md), bygd med byggesettet
(kit.py). Kjør:

    python3 tools/models/tavern.py

Gir to modeller: utsida (`tavern.glb`, tegnes alltid) og innsida
(`tavern_interior.glb`, tegnes bare når du er nær), og `tavern_inside.json`
med kolliderne, lysene, lykta, pipene og kjelleren for begge.

Planen (Blender: x til høyre, −y mot torget):
- Steinsokkel og bindingsverk i to etasjer; overetasjen krager ut foran og
  bak. Høyt halmtak som henger litt i midten, med mønerull. Pipe på
  venstre gavl (peisen) og gjennom taket bak til høyre (grua).
- Første etasje: skjenkestua til venstre med peis på gavlveggen, bord og
  benker, og disken langs kjøkkenveggen. Kjøkkenet bak til høyre med grue,
  arbeidsbord, spekeskinker og hyller, og lem med trapp ned til
  ølkjelleren. Trapphallen foran til høyre med trappa opp langs fasaden.
- Kjelleren under huset: tønner på lager, hyller med krukker, lykter.
- Overetasjen: sovesal under taket med senger, kister og vaskefat.

Kjellertrappa ligger midt i huset, så bakken kan tas bort der
(`Builder::hole`) uansett hvordan huset er dreid i byen."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import CARVED, IRON_BAND, ROPE, barrel, bench, box, candle, door, fire, half_timbered, planks, stairs, stick, stool, table, window  # noqa: E402
from town import collide, layer, light, plain  # noqa: E402

OUT = "assets/models/town"

# Mål (m). Grunnflaten er W × D; veggenes midtlinjer ligger T/2 innenfor.
W, D, T = 12.0, 9.0, 0.3
F0 = 0.6  # gulvet i første etasje
C0 = 3.6  # taket i første etasje (undersida av bjelkelaget)
F1 = 3.85  # gulvet oppe
E = 6.6  # takfoten
JETTY = 0.45
PITCH = 52.0
CELLAR = -2.2  # kjellergulvet
X0, X1 = -W / 2 + T / 2, W / 2 - T / 2
Y0, Y1 = -D / 2 + T / 2, D / 2 - T / 2
UY0, UY1 = Y0 - JETTY, Y1 + JETTY

DOOR_X = -1.6
# Kjellertrappa: mellom x 1,0 og 2,0, fra y 1,6 (oppe) ned til −1,6.
CX0, CX1, CY_TOP, CY_BOT = 1.0, 2.0, 1.6, -1.6
# Trappa opp: langs fasaden innenfor, fra x 5,2 (nederst) til 0,9
# (øverst), der du går av mot vest. Trapphullet oppe er HX0..HX1.
SX0, SX1, SY0, SY1 = 0.9, 5.2, -4.1, -3.0
HX0, HX1 = 0.75, 4.0
KITCHEN_X = 0.4  # kjøkkenveggen (vest)
KITCHEN_Y = -1.6  # kjøkkenveggen (sør)


def outside(p, rng):
    """Utsida: sokkel, vegger, tak, piper, skilt, lykt og trinn."""
    # Sokkelen: fire murer av kampestein (hul, så kjelleren får plass).
    sx, sy = W / 2 + 0.1, D / 2 + 0.1
    for lo, hi in (((-sx, -sy, -0.25), (sx, -sy + 0.5, F0)), ((-sx, sy - 0.5, -0.25), (sx, sy, F0)), ((-sx, -sy, -0.25), (-sx + 0.5, sy, F0)), ((sx - 0.5, -sy, -0.25), (sx, sy, F0))):
        box("plinth", lo, hi, p.stone, bevel=0.05)
    # Trinnene opp til døra.
    box("step", (DOOR_X - 0.9, -sy - 0.75, -0.2), (DOOR_X + 0.9, -sy + 0.05, 0.3), p.stone, bevel=0.05)
    box("step", (DOOR_X - 0.85, -sy - 0.35, 0.2), (DOOR_X + 0.85, -sy + 0.05, F0), p.stone, bevel=0.05)
    town.ramp(DOOR_X - 0.85, DOOR_X + 0.85, -sy - 0.95, -sy + 0.05, 0.0, F0)

    # Første etasje.
    gf = (F0, C0)
    fw = (F0 + 0.9, F0 + 2.3)
    front_open = [(DOOR_X - X0, F0, F0 + 2.35, 1.3), (-4.2 - X0, fw[0], fw[1], 1.0), (2.2 - X0, fw[0], fw[1], 1.0), (4.6 - X0, fw[0], fw[1], 1.0)]
    fr, _ = half_timbered(p, "wall_front", (X0, Y0), (X1, Y0), *gf, -1, front_open, rng)
    back_open = [(-3.4 - X0, fw[0], fw[1], 1.0), (2.4 - X0, fw[0], fw[1], 1.0)]
    bk, _ = half_timbered(p, "wall_back", (X0, Y1), (X1, Y1), *gf, 1, back_open, rng)
    left_open = [(-2.7 - Y0, fw[0], fw[1], 1.0), (3.0 - Y0, fw[0], fw[1], 1.0)]
    lf, _ = half_timbered(p, "wall_left", (X0, Y0), (X0, Y1), *gf, 1, left_open, rng)
    right_open = [(2.6 - Y0, fw[0], fw[1], 1.0), (-1.0 - Y0, fw[0], fw[1], 1.0)]
    rt, _ = half_timbered(p, "wall_right", (X1, Y0), (X1, Y1), *gf, -1, right_open, rng)
    for frame, opens, out in ((fr, front_open[1:], -1), (bk, back_open, 1), (lf, left_open, 1), (rt, right_open, -1)):
        for s, za, zb, w in opens:
            window(p, frame, s, (za + zb) / 2, w, zb - za, out, rng=rng)
    door(p, fr, DOOR_X - X0, F0, 1.3, 2.35, -1)

    # Bjelkelaget mellom etasjene: bjelkeender som stikker ut under
    # utkragingen foran og bak, og et bord langs kanten.
    for y0, y1 in ((UY0 - 0.05, Y0 + 0.2), (Y1 - 0.2, UY1 + 0.05)):
        x = X0 + 0.3
        while x < X1:
            stick("joist_end", (x, y0, (C0 + F1) / 2), (x, y1, (C0 + F1) / 2), 0.2, F1 - C0, p.timber, ("timber", rng.randrange(4)), seed=x, bevel=0.0)
            x += 0.75
        yy = y0 if y0 < 0 else y1
        stick("jetty_board", (X0 - 0.1, yy, (C0 + F1) / 2 + 0.05), (X1 + 0.1, yy, (C0 + F1) / 2 + 0.05), 0.08, 0.3, p.trim, ("trim", CARVED))
    for x in (X0, X1):
        box("floor_band", (x - T / 2 - 0.02, Y0 - 0.02, C0), (x + T / 2 + 0.02, Y1 + 0.02, F1), p.timber)

    # Overetasjen.
    uf = (F1, E)
    uw = (F1 + 0.8, F1 + 2.0)
    ufront = [(x - X0, uw[0], uw[1], 0.85) for x in (-4.4, -1.6, 1.4, 4.2)]
    ufr, _ = half_timbered(p, "upper_front", (X0, UY0), (X1, UY0), *uf, -1, ufront, rng)
    uback = [(x - X0, uw[0], uw[1], 0.85) for x in (-3.6, 0.2, 3.0)]
    ubk, _ = half_timbered(p, "upper_back", (X0, UY1), (X1, UY1), *uf, 1, uback, rng)
    uleft = [(0.0 - UY0, uw[0], uw[1], 0.85)]
    ulf, _ = half_timbered(p, "upper_left", (X0, UY0), (X0, UY1), *uf, 1, uleft, rng)
    uright = [(0.0 - UY0, uw[0], uw[1], 0.85)]
    urt, _ = half_timbered(p, "upper_right", (X1, UY0), (X1, UY1), *uf, -1, uright, rng)
    for frame, opens, out in ((ufr, ufront, -1), (ubk, uback, 1), (ulf, uleft, 1), (urt, uright, -1)):
        for s, za, zb, w in opens:
            window(p, frame, s, (za + zb) / 2, w, zb - za, out, rng=rng)

    # Taket og gavlene.
    ridge = kit.gable_roof(p, X0 - T / 2, X1 + T / 2, UY0 - T / 2, UY1 + T / 2, E + 0.28, PITCH, overhang=0.65, gable_over=0.45)
    for x, out in ((X0, -1), (X1, 1)):
        kit.gable_wall(p, x, UY0 - T / 2, UY1 + T / 2, E, ridge, PITCH, out, rng)

    # Pipene: peisen på venstre gavl (utvendig), og grua gjennom taket bak.
    kit.chimney(p, X0 - T / 2 - 0.45, 0.5, -0.2, ridge + 1.2, w=1.0, smoke=0.9)
    roof_at = lambda y: E + (UY1 + T / 2 + 0.65 - abs(y)) * math.tan(math.radians(PITCH)) - 0.9
    kit.chimney(p, 4.85, 3.4, roof_at(3.4) - 0.6, roof_at(3.4) + 1.8, w=0.8, smoke=0.7)

    # Skiltet: smijernsarm og et bord med et ølkrus.
    ax, az = -3.4, C0 - 0.25
    stick("sign_arm", (ax, Y0 - T / 2, az), (ax, Y0 - 1.25, az), 0.06, 0.07, p.iron)
    stick("sign_strut", (ax, Y0 - T / 2, az - 0.5), (ax, Y0 - 0.85, az - 0.02), 0.05, 0.05, p.iron)
    for yy in (Y0 - 0.55, Y0 - 1.15):
        stick("sign_chain", (ax, yy, az), (ax, yy, az - 0.22), 0.015, 0.015, p.iron)
    box("sign_board", (ax - 0.04, Y0 - 1.3, az - 1.0), (ax + 0.04, Y0 - 0.4, az - 0.2), p.door_dark, bevel=0.02)
    stick("sign_frame", (ax, Y0 - 1.32, az - 0.2), (ax, Y0 - 0.38, az - 0.2), 0.1, 0.06, p.trim, ("trim", ROPE))
    stick("sign_frame", (ax, Y0 - 1.32, az - 1.0), (ax, Y0 - 0.38, az - 1.0), 0.1, 0.06, p.trim, ("trim", ROPE))
    # Krusene på begge sider av skiltet.
    for side in (-1, 1):
        box("sign_mug", (ax + side * 0.05 - 0.07, Y0 - 1.0, az - 0.8), (ax + side * 0.05 + 0.07 + side * 0.08, Y0 - 0.7, az - 0.4), p.cladding, bevel=0.04)
        box("sign_mug_band", (ax + side * 0.05 - 0.08, Y0 - 1.01, az - 0.7), (ax + side * 0.05 + 0.08 + side * 0.08, Y0 - 0.69, az - 0.66), p.iron, bevel=0.01)
    # Lykta ved døra, benk og tønner foran.
    town.wall_lantern(p, DOOR_X + 1.05, Y0 - T / 2, F0 + 2.35, -1)
    bench(p, -4.8, Y0 - T / 2 - 0.85, 0.0, 2.2)
    for k, (x, y) in enumerate(((0.6, Y0 - 1.0), (1.2, Y0 - 0.9), (0.85, Y0 - 1.5))):
        barrel(p, x, y, 0.0, yaw=k)
        collide((x, y, 0.45), (0.65, 0.65, 0.9))


def ground_floor(p, rng):
    """Første etasje inne: gulv, kjøkkenvegger, peis, disk, bord, kjøkken."""
    inner = (X0 + T / 2, Y0 + T / 2, X1 - T / 2, Y1 - T / 2)
    ix0, iy0, ix1, iy1 = inner
    # Gulvet med hullet for kjellertrappa, og kolliderne rundt.
    hx0, hx1, hy0, hy1 = CX0 - 0.05, CX1 + 0.05, CY_BOT + 0.3, CY_TOP
    for lo, hi in (((ix0, iy0), (hx0, iy1)), ((hx1, iy0), (ix1, iy1)), ((hx0, iy0), (hx1, hy0)), ((hx0, hy1), (hx1, iy1))):
        planks(p, "floor0", (lo[0], lo[1], F0 - 0.1), (hi[0], hi[1], F0))
        collide(((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, F0 - 0.15), (hi[0] - lo[0], hi[1] - lo[1], 0.3))
    # Taket: bjelker og bordene under gulvet oppe (med trapphullet).
    oy1 = SY1 + 0.1
    for lo, hi in (((ix0, iy0), (HX0, iy1)), ((HX1, iy0), (ix1, iy1)), ((HX0, oy1), (HX1, iy1))):
        planks(p, "floor1", (lo[0], lo[1], C0), (hi[0], hi[1], F1))
        collide(((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, F1 - 0.12), (hi[0] - lo[0], hi[1] - lo[1], 0.25))
    x = ix0 + 0.4
    while x < ix1:
        y_lo = oy1 if HX0 < x < HX1 else iy0
        stick("joist", (x, y_lo, C0 - 0.12), (x, iy1, C0 - 0.12), 0.18, 0.24, p.timber, ("timber", rng.randrange(4)), seed=x)
        x += 0.9
    stick("trimmer", (HX0, oy1 + 0.05, C0 - 0.12), (HX1, oy1 + 0.05, C0 - 0.12), 0.2, 0.24, p.timber, ("timber", 2))
    # Stolper midt i rommet under bjelkene.
    for px, py in ((-2.6, -1.2), (-2.6, 2.2)):
        stick("pillar", (px, py, F0), (px, py, C0 - 0.24), 0.28, 0.28, p.timber, ("timber", rng.randrange(4)), seed=px + py)
        box("pillar_head", (px - 0.5, py - 0.16, C0 - 0.42), (px + 0.5, py + 0.16, C0 - 0.24), p.timber)
        collide((px, py, (F0 + C0) / 2), (0.3, 0.3, C0 - F0))

    # Kjøkkenveggene: mot skjenkestua (vest) og trapphallen (sør), med dører.
    kw = 0.2
    fw, L = kit.wall_frame((KITCHEN_X, KITCHEN_Y), (KITCHEN_X, iy1))
    kit.slab_grid("kitchen_wall_w", fw, L, F0, C0, kw, [(3.2 - KITCHEN_Y - 0.5, 3.2 - KITCHEN_Y + 0.5, F0, F0 + 2.15)], p.plaster)
    kit.wall_colliders(fw, L, F0, C0, kw, [(3.2 - KITCHEN_Y - 0.5, 3.2 - KITCHEN_Y + 0.5, F0, F0 + 2.15)])
    fs, L2 = kit.wall_frame((KITCHEN_X, KITCHEN_Y), (ix1, KITCHEN_Y))
    kit.slab_grid("kitchen_wall_s", fs, L2, F0, C0, kw, [(3.4 - KITCHEN_X - 0.55, 3.4 - KITCHEN_X + 0.55, F0, F0 + 2.15)], p.plaster)
    kit.wall_colliders(fs, L2, F0, C0, kw, [(3.4 - KITCHEN_X - 0.55, 3.4 - KITCHEN_X + 0.55, F0, F0 + 2.15)])
    for frame, s in ((fw, 3.2 - KITCHEN_Y), (fs, 3.4 - KITCHEN_X)):
        for side in (-1, 1):
            stick("kdoor_post", kit.at_wall(frame, s + side * 0.55, 0, F0), kit.at_wall(frame, s + side * 0.55, 0, F0 + 2.25), 0.26, 0.14, p.timber, ("timber", 1))
        stick("kdoor_head", kit.at_wall(frame, s - 0.65, 0, F0 + 2.22), kit.at_wall(frame, s + 0.65, 0, F0 + 2.22), 0.26, 0.16, p.trim, ("trim", kit.DARK_BEAM))
    # Bjelker på kjøkkenveggene.
    for frame, length in ((fw, L), (fs, L2)):
        for s in (0.1, length - 0.1):
            stick("kpost", kit.at_wall(frame, s, -0.13, F0), kit.at_wall(frame, s, -0.13, C0), 0.14, 0.2, p.timber, ("timber", 3))

    # Peisen på venstre gavlvegg.
    hx = ix0
    box("hearth_base", (hx, -0.6, F0), (hx + 0.85, 1.6, F0 + 0.35), p.stone, bevel=0.05)
    box("hearth_back", (hx, -0.4, F0 + 0.35), (hx + 0.25, 1.4, C0 - 0.3), p.soot, bevel=0.03)
    for yy in (-0.6, 1.25):
        box("hearth_jamb", (hx, yy, F0 + 0.35), (hx + 0.85, yy + 0.35, F0 + 1.6), p.stone, bevel=0.05)
    box("hearth_lintel", (hx - 0.02, -0.65, F0 + 1.6), (hx + 0.95, 1.65, F0 + 1.95), p.stone, bevel=0.05)
    stick("mantel", (hx + 0.95, -0.8, F0 + 2.0), (hx + 0.95, 1.8, F0 + 2.0), 0.3, 0.14, p.trim, ("trim", CARVED))
    box("hearth_hood", (hx, -0.4, F0 + 1.95), (hx + 0.7, 1.4, C0 - 0.24), p.stone, bevel=0.04)
    fire(p, hx + 0.45, 0.5, F0 + 0.35, size=0.9, intensity=3.2, radius=10.0)
    collide((hx + 0.45, 0.5, F0 + 1.0), (0.9, 2.2, 2.0))
    # Jerngryte på krok, og ved ved siden av.
    barrel(p, hx + 0.45, 0.5, F0 + 0.75, r=0.2, h=0.3).data.materials[0] = p.iron
    for k in range(4):
        stick("firewood", (hx + 0.2, 1.75 + 0.12 * (k % 2), F0 + 0.08 + 0.13 * (k // 2)), (hx + 0.9, 1.75 + 0.12 * (k % 2), F0 + 0.08 + 0.13 * (k // 2)), 0.11, 0.11, layer("logs"), bevel=0.02)

    # Disken langs kjøkkenveggen, med hyller og tønner bak.
    cx0, cx1, cy0, cy1 = -1.05, -0.5, -0.8, 2.6
    box("counter", (cx0, cy0, F0), (cx1, cy1, F0 + 1.0), p.cladding, bevel=0.03)
    box("counter_top", (cx0 - 0.12, cy0 - 0.1, F0 + 1.0), (cx1 + 0.05, cy1 + 0.1, F0 + 1.08), p.floor, bevel=0.02)
    stick("counter_trim", (cx0 - 0.02, cy0, F0 + 0.95), (cx0 - 0.02, cy1, F0 + 0.95), 0.05, 0.1, p.trim, ("trim", IRON_BAND))
    collide(((cx0 + cx1) / 2, (cy0 + cy1) / 2, F0 + 0.55), (cx1 - cx0 + 0.15, cy1 - cy0 + 0.2, 1.1))
    for z in (F0 + 1.35, F0 + 1.95):
        box("shelf", (KITCHEN_X - 0.42, cy0, z), (KITCHEN_X - 0.1, cy1, z + 0.05), p.floor, bevel=0.01)
        for k in range(9):
            y = cy0 + 0.25 + k * (cy1 - cy0 - 0.5) / 8
            if k % 3 == 2:
                barrel(p, KITCHEN_X - 0.26, y, z + 0.05, r=0.1, h=0.22)
            else:
                box("jug", (KITCHEN_X - 0.33, y - 0.07, z + 0.05), (KITCHEN_X - 0.19, y + 0.07, z + 0.27), layer("plaster_red"), bevel=0.03)
    for k, y in enumerate((-0.3, 0.6)):
        barrel(p, KITCHEN_X - 0.4, y, F0 + 0.0, r=0.3, h=0.7, lying=True, yaw=0)
    for k in range(5):
        x = cx0 + 0.1
        y = cy0 + 0.35 + k * 0.6
        box("mug", (x - 0.05, y - 0.05, F0 + 1.08), (x + 0.05, y + 0.05, F0 + 1.22), p.door_dark, bevel=0.015)

    # Bordene i skjenkestua, med benker og krakker, og lys på bordene.
    for (x, y, yaw, w) in ((-4.0, -2.8, 0.0, 2.2), (-4.0, 2.6, 0.0, 2.2), (-1.9, 0.3, math.pi / 2, 1.8)):
        table(p, x, y, F0, w, 0.9, yaw=yaw)
        c, s_ = math.cos(yaw), math.sin(yaw)
        for side in (-1, 1):
            bx, by = x - s_ * side * 0.75, y + c * side * 0.75
            bench(p, bx, by, F0, w - 0.2, yaw=yaw)
        candle(p, x, y, F0 + 0.78, light_it=True)
    for x, y in ((-5.0, -0.9), (-1.5, 2.9)):
        stool(p, x, y, F0)
        collide((x, y, F0 + 0.25), (0.4, 0.4, 0.5))

    # Kjøkkenet: grua i hjørnet bak til høyre, med hette og pipe.
    gx0, gy0 = 4.0, 3.0
    box("grue", (gx0, gy0, F0), (ix1, iy1, F0 + 0.55), p.stone, bevel=0.05)
    box("grue_back", (gx0, iy1 - 0.25, F0 + 0.55), (ix1, iy1, C0), p.soot, bevel=0.02)
    box("grue_hood", (gx0 - 0.1, gy0 - 0.15, F0 + 2.0), (ix1, iy1, C0), p.stone, bevel=0.05)
    fire(p, (gx0 + ix1) / 2, (gy0 + iy1) / 2 - 0.1, F0 + 0.55, size=0.7, intensity=2.6, radius=7.0)
    collide(((gx0 + ix1) / 2, (gy0 + iy1) / 2, F0 + 0.5), (ix1 - gx0, iy1 - gy0, 1.0))
    # Gryta over ilden i en kjetting.
    pot = barrel(p, (gx0 + ix1) / 2, (gy0 + iy1) / 2 - 0.1, F0 + 1.05, r=0.24, h=0.35)
    pot.data.materials[0] = p.iron
    stick("pot_chain", ((gx0 + ix1) / 2, (gy0 + iy1) / 2 - 0.1, F0 + 1.4), ((gx0 + ix1) / 2, (gy0 + iy1) / 2 - 0.1, F0 + 2.0), 0.02, 0.02, p.iron)
    # Arbeidsbordet med brød, og en krakk.
    table(p, 3.6, 0.9, F0, 1.8, 0.85)
    for k in range(4):
        x = 3.0 + k * 0.38
        box("bread", (x - 0.14, 0.75, F0 + 0.78), (x + 0.14, 1.0, F0 + 0.9), layer("thatch", (0.9, 0.62, 0.35)), bevel=0.05)
    box("dough_board", (3.9, 1.05, F0 + 0.78), (4.4, 1.25, F0 + 0.81), p.floor, bevel=0.005)
    stool(p, 3.6, 1.7, F0)
    # Spekeskinker og pølser fra bjelkene.
    for k, x in enumerate((2.8, 3.3, 3.8, 4.3)):
        y = -0.6 + (k % 2) * 0.3
        stick("ham_rope", (x, y, C0 - 0.24), (x, y, C0 - 0.6), 0.015, 0.015, p.trim, ("trim", ROPE))
        ham = barrel(p, x, y, C0 - 1.15, r=0.13, h=0.55)
        ham.data.materials[0] = layer("leather", (0.75, 0.42, 0.3))
    # Hyller med krukker på bakveggen, og sekker i hjørnet.
    for z in (F0 + 1.4, F0 + 2.0):
        box("kshelf", (1.0, iy1 - 0.35, z), (3.6, iy1, z + 0.05), p.floor, bevel=0.01)
        for k in range(6):
            x = 1.2 + k * 0.42
            box("crock", (x - 0.1, iy1 - 0.28, z + 0.05), (x + 0.1, iy1 - 0.08, z + 0.3), layer("plaster_white", (0.85, 0.8, 0.7)), bevel=0.04)
    for k, (x, y) in enumerate(((5.3, -0.9), (5.3, -0.25), (4.75, -1.0))):
        box("sack", (x - 0.25, y - 0.22, F0), (x + 0.25, y + 0.22, F0 + 0.6), layer("linen", (0.85, 0.78, 0.62)), bevel=0.12)
    collide((5.1, -0.6, F0 + 0.3), (0.9, 1.0, 0.6))

    # Lemmen og rekkverket rundt kjellertrappa.
    rail = p.timber
    for xx in (hx0 - 0.04, hx1 + 0.04):
        kit.stick("crail", (xx, hy0, F0 + 0.95), (xx, hy1, F0 + 0.95), 0.07, 0.08, rail, ("timber", 2))
        for yy in (hy0, (hy0 + hy1) / 2, hy1):
            kit.stick("crail_post", (xx, yy, F0), (xx, yy, F0 + 1.0), 0.09, 0.09, rail, ("timber", 0))
        collide((xx, (hy0 + hy1) / 2, F0 + 0.5), (0.12, hy1 - hy0, 1.0))
    kit.stick("crail", (hx0, hy0 - 0.04, F0 + 0.95), (hx1, hy0 - 0.04, F0 + 0.95), 0.07, 0.08, rail, ("timber", 2))
    collide(((hx0 + hx1) / 2, hy0 - 0.04, F0 + 0.5), (hx1 - hx0, 0.12, 1.0))
    # Lemmen står slått opp mot rekkverket på østsida.
    box("trapdoor", (hx1 + 0.1, hy1 - 1.2, F0), (hx1 + 0.17, hy1, F0 + 1.0), p.door_dark, bevel=0.01)
    stairs(p, CX0, CX1, CY_BOT, CY_TOP, CELLAR, F0, 11, along="y")

    # Trappa opp langs fasaden, med rekkverk mot rommet.
    stairs(p, SY0, SY1, SX1, SX0, F0, F1, 13, along="x", rails=(SY1,))
    # Lys: lykter på stolpene.
    light((-2.6, -1.0, C0 - 0.8), (1.0, 0.68, 0.38), 1.2, 6.0)


def cellar(p, rng):
    """Ølkjelleren under huset: murer, steingulv, bjelketak, tønner og
    hyller, og to lykter."""
    cx0, cy0, cx1, cy1 = -5.3, -3.8, 5.3, 3.8
    z0 = CELLAR
    box("cellar_floor", (cx0, cy0, z0 - 0.2), (cx1, cy1, z0), p.flag, bevel=0.0)
    collide((0, 0, z0 - 0.1), (cx1 - cx0, cy1 - cy0, 0.2))
    for lo, hi in (((cx0 - 0.4, cy0 - 0.4, z0 - 0.2), (cx1 + 0.4, cy0, F0 - 0.1)), ((cx0 - 0.4, cy1, z0 - 0.2), (cx1 + 0.4, cy1 + 0.4, F0 - 0.1)), ((cx0 - 0.4, cy0, z0 - 0.2), (cx0, cy1, F0 - 0.1)), ((cx1, cy0, z0 - 0.2), (cx1 + 0.4, cy1, F0 - 0.1))):
        box("cellar_wall", lo, hi, p.wall_stone, bevel=0.0)
        collide(tuple((a + b) / 2 for a, b in zip(lo, hi)), tuple(b - a for a, b in zip(lo, hi)))
    # Taket under bakken: bord og bjelker, med åpning over trappa.
    top = -0.08
    ox0, ox1, oy0, oy1 = CX0 - 0.1, CX1 + 0.1, CY_BOT - 0.1, CY_TOP + 0.2
    for lo, hi in (((cx0, cy0), (ox0, cy1)), ((ox1, cy0), (cx1, cy1)), ((ox0, cy0), (ox1, oy0)), ((ox0, oy1), (ox1, cy1))):
        planks(p, "cellar_ceiling", (lo[0], lo[1], top), (hi[0], hi[1], top + 0.06))
    x = cx0 + 0.5
    while x < cx1:
        if not (ox0 - 0.15 < x < ox1 + 0.15):
            stick("cellar_joist", (x, cy0, top - 0.12), (x, cy1, top - 0.12), 0.2, 0.22, p.timber, ("timber", rng.randrange(4)), seed=x)
        x += 1.1
    # Stolper med sattel midt i kjelleren.
    for px in (-2.8, 3.6):
        stick("cellar_post", (px, 0.0, z0), (px, 0.0, top - 0.24), 0.26, 0.26, p.timber, ("timber", 1))
        collide((px, 0.0, (z0 + top) / 2), (0.3, 0.3, top - z0))
    # Tønnelager langs bakveggen og venstre vegg, på bukker.
    for k in range(6):
        x = -4.6 + k * 1.0
        stick("rack", (x - 0.45, cy1 - 0.45, z0 + 0.2), (x + 0.45, cy1 - 0.45, z0 + 0.2), 0.12, 0.12, p.timber, ("timber", 0))
        barrel(p, x, cy1 - 0.45, z0 + 0.24, r=0.36, h=0.85, lying=True, yaw=math.pi / 2)
    collide((-2.1, cy1 - 0.45, z0 + 0.5), (6.2, 0.9, 1.0))
    for k in range(4):
        y = -2.6 + k * 1.0
        barrel(p, cx0 + 0.45, y, z0, r=0.34, h=0.9, yaw=k)
    collide((cx0 + 0.45, -1.1, z0 + 0.45), (0.8, 4.0, 0.9))
    # Hyller med krukker på høyre vegg.
    for z in (z0 + 0.5, z0 + 1.1, z0 + 1.7):
        box("cshelf", (cx1 - 0.45, -2.5, z), (cx1, 0.0, z + 0.05), p.floor, bevel=0.01)
        for k in range(5):
            y = -2.25 + k * 0.5
            box("ccrock", (cx1 - 0.35, y - 0.12, z + 0.05), (cx1 - 0.12, y + 0.12, z + 0.38), layer("plaster_red", (0.8, 0.7, 0.6)), bevel=0.05)
    collide((cx1 - 0.22, -1.25, z0 + 1.0), (0.45, 2.5, 2.0))
    # Sekker og kasser foran.
    for k, (x, y) in enumerate(((-3.6, -3.2), (-2.9, -3.3), (3.0, -3.2))):
        box("crate", (x - 0.35, y - 0.3, z0), (x + 0.35, y + 0.3, z0 + 0.6), p.cladding, bevel=0.03)
        collide((x, y, z0 + 0.3), (0.7, 0.6, 0.6))
    # Lyktene.
    for x, y in ((-2.8, 0.25), (3.6, 0.25)):
        box("cellar_lantern", (x - 0.1, y - 0.1, top - 0.85), (x + 0.1, y + 0.1, top - 0.55), plain("glow_horn", (0.3, 0.19, 0.08), 0.5), bevel=0.02)
        light((x, y, top - 0.75), (1.0, 0.62, 0.32), 1.4, 7.0)


def upper_floor(p, rng):
    """Sovesalen under taket: sperrer, senger, kister, vaskefat."""
    ix0, ix1 = X0 + T / 2, X1 - T / 2
    iy0, iy1 = UY0 + T / 2, UY1 - T / 2
    # Gulvet foran over utkragingen (bjelkelaget innenfor er fra første etasje).
    for y0, y1 in ((iy0, Y0 + T / 2), (Y1 - T / 2, iy1)):
        planks(p, "floor_jetty", (ix0, y0, F1 - 0.12), (ix1, y1, F1))
        collide(((ix0 + ix1) / 2, (y0 + y1) / 2, F1 - 0.12), (ix1 - ix0, y1 - y0, 0.25))
    # Sperrene og hanebjelkene inne.
    tan = math.tan(math.radians(PITCH))
    half = UY1 + T / 2
    ridge = E + half * tan
    x = ix0 + 0.6
    while x < ix1:
        for side in (-1, 1):
            stick("rafter", (x, side * (half - 0.3), E + 0.3 * tan - 0.3), (x, 0.0, ridge - 0.55), 0.16, 0.22, p.timber, ("timber", rng.randrange(4)), seed=x * side)
        zc = E + 1.6
        w = half - 1.6 / tan
        stick("collar_tie", (x, -w + 0.4, zc), (x, w - 0.4, zc), 0.14, 0.2, p.timber, ("timber", rng.randrange(4)), seed=x + 9)
        x += 1.6
    # Rekkverket rundt trapphullet.
    oy1 = SY1 + 0.1
    for a, b in (((HX0 + 0.6, oy1), (HX1, oy1)), ((HX1, Y0 + T / 2), (HX1, oy1))):
        stick("srail", (a[0], a[1], F1 + 0.95), (b[0], b[1], F1 + 0.95), 0.07, 0.08, p.timber, ("timber", 2))
        for t in (0.0, 0.5, 1.0):
            px, py = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
            stick("srail_post", (px, py, F1), (px, py, F1 + 1.0), 0.09, 0.09, p.timber, ("timber", 0))
    collide(((HX0 + 0.6 + HX1) / 2, oy1, F1 + 0.5), (HX1 - HX0 - 0.6, 0.12, 1.0))
    collide((HX1, (Y0 + oy1) / 2, F1 + 0.5), (0.12, oy1 - Y0, 1.0))
    # Pipa fra grua går gjennom rommet.
    box("chimney_inside", (4.45, 3.0, F1), (5.25, 3.8, ridge - 2.0), p.stone, bevel=0.04)
    collide((4.85, 3.4, F1 + 1.5), (0.8, 0.8, 3.0))
    # Sengene langs bak- og framveggen.
    wool = [layer("wool", c) for c in ((0.75, 0.3, 0.25), (0.35, 0.45, 0.7), (0.45, 0.6, 0.35), (0.8, 0.65, 0.4))]
    beds = [(-4.6, iy1 - 1.1, 0), (-2.4, iy1 - 1.1, 0), (-0.2, iy1 - 1.1, 0), (2.0, iy1 - 1.1, 0), (-4.6, iy0 + 1.1, 1), (-2.4, iy0 + 1.1, 1), ]
    for k, (x, y, front) in enumerate(beds):
        s = 1 if front else -1
        box("bed_frame", (x - 0.5, y - 1.0, F1), (x + 0.5, y + 1.0, F1 + 0.35), p.timber, bevel=0.03)
        box("bed_straw", (x - 0.43, y - 0.93, F1 + 0.35), (x + 0.43, y + 0.93, F1 + 0.48), p.straw, bevel=0.04)
        box("bed_blanket", (x - 0.45, min(y + s * 0.95, y - s * 0.3), F1 + 0.44), (x + 0.45, max(y + s * 0.95, y - s * 0.3), F1 + 0.55), wool[k % 4], bevel=0.04)
        box("bed_pillow", (x - 0.32, min(y - s * 0.95, y - s * 0.6), F1 + 0.46), (x + 0.32, max(y - s * 0.95, y - s * 0.6), F1 + 0.6), layer("linen"), bevel=0.06)
        collide((x, y, F1 + 0.3), (1.0, 2.0, 0.6))
        # Kista ved fotenden.
        cy = y + s * 1.3
        box("chest", (x - 0.4, cy - 0.25, F1), (x + 0.4, cy + 0.25, F1 + 0.45), p.cladding, bevel=0.03)
        box("chest_band", (x - 0.42, cy - 0.27, F1 + 0.3), (x + 0.42, cy + 0.27, F1 + 0.36), p.iron, bevel=0.01)
        collide((x, cy, F1 + 0.23), (0.8, 0.5, 0.45))
    # Vaskefatet på et stativ, og lys.
    table(p, 4.6, 1.2, F1, 0.6, 0.5, h=0.85)
    barrel(p, 4.6, 1.2, F1 + 0.85, r=0.22, h=0.12).data.materials[0] = layer("plaster_white", (0.8, 0.85, 0.9))
    for x, y in ((-3.5, 0.0), (1.0, 0.0)):
        candle(p, x, y, F1 + 0.01, light_it=False)
    light((-2.0, 0.0, F1 + 2.0), (1.0, 0.68, 0.4), 1.3, 7.0)
    light((2.4, 0.0, F1 + 2.0), (1.0, 0.68, 0.4), 1.1, 6.0)


def build():
    rng = random.Random(1350)
    real_write = town.write_inside
    town.write_inside = lambda path: None
    # Utsida.
    clear()
    town.PIECES.clear()
    p = kit.Paint(plaster="plaster", timber="timber", roof="thatch")
    town.LIT = True
    outside(p, rng)
    town.finish(f"{OUT}/tavern.glb", max_edge=100.0)
    # Innsida.
    clear()
    town.PIECES.clear()
    p = kit.Paint(plaster="plaster", timber="timber", roof="thatch")
    ground_floor(p, rng)
    cellar(p, rng)
    upper_floor(p, rng)
    town.finish(f"{OUT}/tavern_interior.glb", max_edge=100.0)
    town.write_inside = real_write
    real_write(f"{OUT}/tavern.glb")
    # Kjelleren: bakken tas bort over kjellertrappa (motorens akser:
    # x, y, −z), og kjelleren dekker kanten.
    import json
    path = f"{OUT}/tavern_inside.json"
    data = json.load(open(path))
    data["cellar"] = [[0.0, 0.0, 0.0], [5.2, 3.6]]
    data["cellar_floor"] = [[0.0, CELLAR, 0.0], [5.3, 3.8]]
    json.dump(data, open(path, "w"))


if __name__ == "__main__":
    build()
