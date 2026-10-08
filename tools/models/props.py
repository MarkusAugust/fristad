"""Tingene på torget, modellert i Blender (bpy) og eksportert til
assets/models/town/. Kjør:

    python3 tools/models/props.py [market_cross market_stall ...]

Torgkorset av hogd stein på trappetrinn, to markedsboder (bordtak og
seilduk), en tohjulskjerre med eikehjul og jernringer, tønne av staver med
jernbånd, kasse av bord med lekter, og en lyktestolpe med smijernslykt.
Samme hjelpere og teksturlag som husene (town.py), origo midt på
grunnflaten, −Y fram."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Vector  # noqa: E402

import town  # noqa: E402
from town import M, PIECES, beam, block, finish, layer, plain  # noqa: E402
from walls import revolve  # noqa: E402

OUT = "assets/models/town"


def mats():
    m = M()
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.canvas = layer("plaster", (0.62, 0.42, 0.22), 0.5)
    m.canvas2 = layer("plaster", (0.38, 0.12, 0.08), 0.5)
    m.glow = plain("glow", (1.0, 0.75, 0.4), 0.4)
    return m


# --- Små ting som brukes flere steder ---------------------------------------


def barrel(m, at=(0, 0, 0), h=0.86, r=0.3, staves=18, rng=None):
    """En tønne av staver med buk, lokk og fire jernbånd."""
    rng = rng or random.Random(1)
    x0, y0, z0 = at
    for k in range(staves):
        a = 2 * math.pi * k / staves
        w = 2 * math.pi * r / staves * 0.96
        # Staven: tre ledd som følger buken.
        pts = []
        for t in (0.0, 0.25, 0.5, 0.75, 1.0):
            bulge = r * (0.86 + 0.14 * math.sin(math.pi * t))
            pts.append(Vector((x0 + bulge * math.cos(a), y0 + bulge * math.sin(a), z0 + h * t)))
        for p, q in zip(pts, pts[1:]):
            beam("stave", p, q, 0.035, w, m.planks, roll=0.0).rotation_mode = "QUATERNION"
    lid_r = r * 0.86
    revolve("lid", [(lid_r - 0.02, z0 + h - 0.05), (lid_r - 0.02, z0 + h - 0.02), (0.0, z0 + h - 0.02)], m.planks, n=staves, at=(x0, y0, 0))
    revolve("bottom", [(lid_r - 0.02, z0 + 0.02), (lid_r - 0.02, z0 + 0.05), (0.0, z0 + 0.05)], m.planks, n=staves, at=(x0, y0, 0))
    for t in (0.08, 0.3, 0.7, 0.92):
        rr = r * (0.86 + 0.14 * math.sin(math.pi * t)) + 0.012
        revolve("hoop", [(rr, z0 + h * t - 0.025), (rr + 0.006, z0 + h * t), (rr, z0 + h * t + 0.025), (rr - 0.01, z0 + h * t + 0.025), (rr - 0.01, z0 + h * t - 0.025)], m.iron, n=staves * 2, at=(x0, y0, 0), cap=False)


def crate(m, at=(0, 0, 0), s=0.7, rng=None):
    """En kasse av bord med glipper, lekter på hjørnene og et lokk."""
    rng = rng or random.Random(2)
    x0, y0, z0 = at
    n = 4
    bw = s / n
    for side in range(4):
        a = side * math.pi / 2
        ux, uy = math.cos(a), math.sin(a)
        nx, ny = -uy, ux
        for k in range(n):
            off = -s / 2 + bw * (k + 0.5)
            c = (x0 + ux * off + nx * (s / 2 - 0.012), y0 + uy * off + ny * (s / 2 - 0.012), z0 + s / 2)
            ob = block("board", (bw - 0.012, 0.022, s - 0.02), m.planks, at=c, rot=(0, 0, a), bevel=0.004)
    for sx in (-1, 1):
        for sy in (-1, 1):
            block("corner", (0.05, 0.05, s), m.planks, at=(x0 + sx * (s / 2 - 0.005), y0 + sy * (s / 2 - 0.005), z0 + s / 2), bevel=0.006)
    for k in range(n):
        block("lid", (bw - 0.012, s, 0.022), m.planks, at=(x0 - s / 2 + bw * (k + 0.5), y0, z0 + s - 0.01), bevel=0.004)
    block("lid_batten", (s, 0.06, 0.025), m.planks, at=(x0, y0 - s * 0.3, z0 + s + 0.012), bevel=0.005)
    block("lid_batten", (s, 0.06, 0.025), m.planks, at=(x0, y0 + s * 0.3, z0 + s + 0.012), bevel=0.005)


# --- Modellene --------------------------------------------------------------


def market_cross():
    m = mats()
    # Tre trappetrinn av hogd stein, åttekantet sokkel, skaft og kors.
    for k, (w, h) in enumerate([(2.5, 0.22), (1.9, 0.22), (1.35, 0.22)]):
        revolve("step", [(w / 2 * 1.08, k * 0.22 - 0.2 if k == 0 else k * 0.22), (w / 2 * 1.08, (k + 1) * 0.22), (0.0, (k + 1) * 0.22)], m.stone, n=8)
    revolve("plinth", [(0.42, 0.66), (0.42, 1.15), (0.32, 1.3), (0.0, 1.3)], m.stone, n=8)
    revolve("shaft", [(0.15, 1.3), (0.13, 3.6), (0.0, 3.6)], m.stone, n=8)
    revolve("ring", [(0.2, 3.0), (0.2, 3.1), (0.14, 3.1)], m.stone, n=8, cap=False)
    block("arm", (0.95, 0.2, 0.2), m.stone, at=(0, 0, 3.75), bevel=0.03, segments=2)
    block("head", (0.2, 0.2, 0.75), m.stone, at=(0, 0, 3.85), bevel=0.03, segments=2)
    for sx in (-1, 1):
        block("arm_end", (0.08, 0.26, 0.26), m.stone, at=(sx * 0.5, 0, 3.75), bevel=0.03, segments=2)
    block("head_end", (0.26, 0.26, 0.08), m.stone, at=(0, 0, 4.25), bevel=0.03, segments=2)
    # En jernring til å binde dyr, og en kunngjøringstavle på skaftet.
    revolve("tie_ring", [(0.09, 0.95), (0.1, 0.96), (0.09, 0.97), (0.08, 0.96)], m.iron, n=16, at=(0, -0.44, 0), cap=False)
    block("notice", (0.4, 0.03, 0.5), m.planks, at=(0, -0.17, 1.9), bevel=0.006)
    return m


def stall(roofed):
    m = mats()
    rng = random.Random(5 if roofed else 6)
    W, D = 2.6, 1.4
    # Disken: bord på bukker, og en skrå hylle bak.
    block("counter", (W, D * 0.55, 0.06), m.planks, at=(0, -D * 0.2, 0.9), bevel=0.008)
    for sx in (-1, 1):
        for sy in (-1, 1):
            beam("leg", (sx * (W / 2 - 0.1), -D * 0.2 + sy * 0.28, 0.0), (sx * (W / 2 - 0.1), -D * 0.2 + sy * 0.2, 0.87), 0.07, 0.07, m.timber)
    block("front_board", (W, 0.03, 0.55), m.planks, at=(0, -D * 0.47, 0.6), bevel=0.006)
    # Stolpene og taket.
    for sx in (-1, 1):
        for sy, top in ((-1, 2.05), (1, 2.45)):
            beam("post", (sx * (W / 2 - 0.06), sy * (D / 2 - 0.06), -0.1), (sx * (W / 2 - 0.06), sy * (D / 2 - 0.06), top), 0.09, 0.09, m.timber)
    for sy, top in ((-1, 2.05), (1, 2.45)):
        beam("plate", (-W / 2 - 0.05, sy * (D / 2 - 0.06), top), (W / 2 + 0.05, sy * (D / 2 - 0.06), top), 0.1, 0.1, m.timber)
    for sx in (-1, 0, 1):
        beam("rafter", (sx * (W / 2 - 0.06), -D / 2 - 0.35, 1.95), (sx * (W / 2 - 0.06), D / 2 + 0.1, 2.5), 0.06, 0.08, m.timber)
    slope = math.atan2(0.55, D + 0.45)
    if roofed:
        n = 13
        for k in range(n):
            x = -W / 2 - 0.1 + (W + 0.2) * (k + 0.5) / n
            block("roof_board", ((W + 0.2) / n - 0.01, D + 0.55, 0.025), m.boards, at=(x, -0.12, 2.27), rot=(slope, 0, 0), bevel=0.004)
    else:
        # Seilduk med bølget frontkant, stripet i to farger.
        n = 8
        for k in range(n):
            x = -W / 2 - 0.1 + (W + 0.2) * (k + 0.5) / n
            block("canvas", ((W + 0.2) / n, D + 0.6, 0.015), m.canvas if k % 2 == 0 else m.canvas2, at=(x, -0.14, 2.29 - 0.015 * (k % 2)), rot=(slope, 0, 0), bevel=0)
            block("valance", ((W + 0.2) / n, 0.015, 0.22), m.canvas if k % 2 == 0 else m.canvas2, at=(x, -D / 2 - 0.42, 1.86), bevel=0)
    # Varer på disken og under.
    if roofed:
        # Kasser med varer, en tønne og to sekker (puss tonet som lerret).
        crate(m, at=(-0.8, -0.2, 0.93), s=0.38, rng=rng)
        crate(m, at=(-0.35, -0.25, 0.93), s=0.34, rng=rng)
        barrel(m, at=(0.95, 0.35, 0.0), h=0.7, r=0.24, staves=14, rng=rng)
        for x in (0.25, 0.65):
            revolve("sack", [(0.0, 0.93), (0.14, 0.95), (0.17, 1.08), (0.13, 1.25), (0.05, 1.32), (0.0, 1.33)], m.canvas, n=12, at=(x, -0.22, 0))
    else:
        # Tøyruller og et fat med brød (rundt), og en kurv.
        for k, x in enumerate((-0.9, -0.6, -0.3)):
            ob = revolve("bolt", [(0.0, -0.3), (0.09, -0.3), (0.09, 0.3), (0.0, 0.3)], m.canvas if k % 2 else m.canvas2, n=14)
            ob.rotation_euler = (math.pi / 2, 0, 0)
            ob.location = (x, -0.2, 1.02)
        revolve("basket", [(0.0, 0.93), (0.2, 0.93), (0.25, 1.1), (0.23, 1.1), (0.18, 0.95), (0.0, 0.95)], m.planks, n=20, at=(0.6, -0.2, 0))
        for k in range(5):
            a = k * 1.3
            revolve("loaf", [(0.0, 0.96), (0.07, 0.97), (0.06, 1.02), (0.0, 1.04)], m.canvas, n=10, at=(0.6 + 0.08 * math.cos(a), -0.2 + 0.08 * math.sin(a), 0.0))
        crate(m, at=(0.9, 0.35, 0.0), s=0.5, rng=rng)
    return m


def market_stall():
    return stall(True)


def market_stall_b():
    return stall(False)


def cart():
    m = mats()
    rng = random.Random(7)
    L, W = 2.2, 1.2
    # Lasteplanet: bunnbord på to bjelker, sidebord med stolper, og draget.
    for sx in (-1, 1):
        beam("frame", (-L / 2, sx * (W / 2 - 0.08), 0.7), (L / 2 + 1.1, sx * 0.22, 0.62), 0.1, 0.1, m.timber)
    for k in range(9):
        x = -L / 2 + L * (k + 0.5) / 9
        block("floor", (L / 9 - 0.015, W, 0.035), m.planks, at=(x, 0, 0.77), bevel=0.004)
    for sy in (-1, 1):
        for z in (0.92, 1.12):
            block("side", (L, 0.03, 0.15), m.planks, at=(0, sy * (W / 2 + 0.01), z), bevel=0.004)
        for k in range(4):
            beam("stake", (-L / 2 + 0.05 + (L - 0.1) * k / 3, sy * (W / 2 + 0.05), 0.72), (-L / 2 + 0.05 + (L - 0.1) * k / 3, sy * (W / 2 + 0.05), 1.22), 0.05, 0.05, m.timber)
    block("back", (0.03, W, 0.3), m.planks, at=(-L / 2 - 0.01, 0, 1.0), bevel=0.004)
    beam("cross", (L / 2 + 1.0, -0.3, 0.63), (L / 2 + 1.0, 0.3, 0.63), 0.06, 0.06, m.timber)
    # Akselen og to hjul med nav, eiker, felger og jernring.
    beam("axle", (0, -W / 2 - 0.25, 0.42), (0, W / 2 + 0.25, 0.42), 0.08, 0.08, m.timber)
    R = 0.45
    for sy in (-1, 1):
        y = sy * (W / 2 + 0.15)
        hub = revolve("hub", [(0.0, -0.12), (0.08, -0.12), (0.1, 0.0), (0.08, 0.12), (0.0, 0.12)], m.timber, n=12)
        hub.rotation_euler = (math.pi / 2, 0, 0)
        hub.location = (0, y, 0.42)
        for k in range(10):
            a = 2 * math.pi * k / 10
            beam("spoke", (0.09 * math.cos(a), y, 0.42 + 0.09 * math.sin(a)), ((R - 0.05) * math.cos(a), y, 0.42 + (R - 0.05) * math.sin(a)), 0.045, 0.035, m.timber)
        for k in range(16):
            a0, a1 = 2 * math.pi * k / 16, 2 * math.pi * (k + 1) / 16
            beam("felloe", ((R - 0.035) * math.cos(a0), y, 0.42 + (R - 0.035) * math.sin(a0)), ((R - 0.035) * math.cos(a1), y, 0.42 + (R - 0.035) * math.sin(a1)), 0.08, 0.06, m.timber)
            beam("tyre", (R * math.cos(a0), y, 0.42 + R * math.sin(a0)), (R * math.cos(a1), y, 0.42 + R * math.sin(a1)), 0.085, 0.014, m.iron)
    # Støttebein foran og litt last: en sekk og en kasse.
    beam("prop", (L / 2 + 0.9, 0, 0.62), (L / 2 + 0.95, 0, 0.0), 0.06, 0.06, m.timber)
    crate(m, at=(-0.5, 0.1, 0.79), s=0.45, rng=rng)
    revolve("sack", [(0.0, 0.79), (0.18, 0.81), (0.22, 0.95), (0.16, 1.12), (0.06, 1.2), (0.0, 1.21)], m.canvas, n=12, at=(0.35, -0.2, 0))
    return m


def barrel_model():
    m = mats()
    barrel(m)
    return m


def crate_model():
    m = mats()
    crate(m)
    return m


def lantern_post():
    m = mats()
    # Stolpen: firkantet eik med avfasede kanter, på en steinfot.
    revolve("foot", [(0.26, -0.15), (0.26, 0.12), (0.18, 0.2), (0.0, 0.2)], m.stone, n=8)
    block("post", (0.14, 0.14, 2.3), m.timber, at=(0, 0, 1.3), bevel=0.02, segments=2)
    block("cap", (0.2, 0.2, 0.05), m.timber, at=(0, 0, 2.47), bevel=0.01)
    # Lykta: smijernsramme, glassruter og flammen inni, med tak og ring.
    z0 = 2.5
    block("lamp_base", (0.24, 0.24, 0.03), m.iron, at=(0, 0, z0 + 0.015), bevel=0.005)
    for sx in (-1, 1):
        for sy in (-1, 1):
            block("lamp_post", (0.018, 0.018, 0.32), m.iron, at=(sx * 0.105, sy * 0.105, z0 + 0.19), bevel=0.003)
    for a in range(4):
        rot = (0, 0, a * math.pi / 2)
        d = Vector((math.cos(a * math.pi / 2 + math.pi / 2), math.sin(a * math.pi / 2 + math.pi / 2), 0)) * 0.1
        block("pane", (0.19, 0.006, 0.28), plain("glow_horn", (0.3, 0.19, 0.08), 0.5), at=(d.x, d.y, z0 + 0.18), rot=rot, bevel=0)
        block("bar", (0.19, 0.012, 0.012), m.iron, at=(d.x * 1.04, d.y * 1.04, z0 + 0.18), rot=rot, bevel=0.002)
    revolve("candle", [(0.0, z0 + 0.03), (0.03, z0 + 0.03), (0.03, z0 + 0.12), (0.0, z0 + 0.12)], m.planks, n=10)
    revolve("flame", [(0.0, z0 + 0.12), (0.022, z0 + 0.15), (0.0, z0 + 0.2)], m.glow, n=8)
    revolve("lamp_roof", [(0.17, z0 + 0.35), (0.0, z0 + 0.48)], m.iron, n=4).rotation_euler = (0, 0, math.pi / 4)
    revolve("lamp_ring", [(0.03, z0 + 0.5), (0.035, z0 + 0.52), (0.03, z0 + 0.54), (0.025, z0 + 0.52)], m.iron, n=10, cap=False)
    return m


def training_dummy():
    """Treningsdukke: en stolpe i bakken med tverrarm, en kropp av halm
    surret med tau, et skjold på armen og hugg i treet."""
    m = mats()
    m.straw = layer("thatch", (0.75, 0.7, 0.55), 0.5)
    m.rope = layer("thatch", (0.5, 0.42, 0.32), 0.5)
    rng = random.Random(9)
    beam("post", (0, 0, -0.2), (0, 0, 1.9), 0.14, 0.14, m.timber, roll=math.pi / 2)
    beam("arm", (-0.65, 0, 1.35), (0.65, 0, 1.35), 0.1, 0.1, m.timber)
    revolve("body", [(0.0, 0.85), (0.24, 0.9), (0.3, 1.15), (0.28, 1.5), (0.18, 1.62), (0.0, 1.64)], m.straw, n=16)
    revolve("head", [(0.0, 1.64), (0.13, 1.7), (0.15, 1.8), (0.1, 1.92), (0.0, 1.95)], m.straw, n=12)
    for z in (0.95, 1.2, 1.45, 1.66):
        r = 0.3 if z < 1.5 else 0.16
        revolve("rope", [(r + 0.01, z - 0.015), (r + 0.025, z), (r + 0.01, z + 0.015)], m.rope, n=16, cap=False)
    shield = revolve("shield", [(0.0, -0.02), (0.28, -0.02), (0.28, 0.02), (0.0, 0.02)], m.planks, n=20)
    shield.rotation_euler = (0, math.pi / 2, 0)
    shield.location = (-0.66, 0, 1.25)
    boss = revolve("shield_boss", [(0.0, 0.02), (0.07, 0.02), (0.0, 0.07)], m.iron, n=12)
    boss.rotation_euler = (0, -math.pi / 2, 0)
    boss.location = (-0.68, 0, 1.25)
    for k in range(5):
        block("cut", (0.16, 0.02, 0.025), m.planks, at=(0.07, 0, 0.3 + k * 0.12 + rng.uniform(-0.03, 0.03)), rot=(0, rng.uniform(-0.4, 0.4), 0), bevel=0)
    for k in range(3):
        a = k * 2.1
        beam("stake", (0, 0, 0.05), (math.cos(a) * 0.4, math.sin(a) * 0.4, -0.05), 0.06, 0.06, m.timber)
    return m


def weapon_rack():
    """Våpenstativ: et A-stativ av bjelker med sverd, økser og spyd, og et
    par skjold lent mot."""
    m = mats()
    for sx in (-1, 1):
        for sy in (-1, 1):
            beam("leg", (sx * 0.9, sy * 0.28, 0.0), (sx * 0.9, 0.0, 1.5), 0.08, 0.08, m.timber)
    for z, sy in ((1.25, 0), (0.45, -0.22), (0.45, 0.22)):
        beam("rail", (-1.0, sy, z), (1.0, sy, z), 0.07, 0.07, m.timber)
    for k in range(7):
        x = -0.75 + k * 0.25
        if k % 3 == 0:
            beam("spear", (x, 0.12, 0.0), (x, 0.0, 1.9), 0.035, 0.035, m.planks)
            revolve("spear_head", [(0.03, 0.0), (0.0, 0.25)], m.iron, n=4, at=(x, 0.0, 1.9))
        elif k % 3 == 1:
            beam("blade", (x, -0.12, 0.25), (x, -0.02, 1.25), 0.05, 0.008, m.iron)
            beam("guard", (x - 0.1, -0.03, 1.27), (x + 0.1, -0.03, 1.27), 0.03, 0.03, m.iron)
            beam("grip", (x, -0.02, 1.28), (x, -0.01, 1.45), 0.03, 0.03, m.planks)
        else:
            beam("haft", (x, 0.12, 0.0), (x, 0.02, 1.3), 0.035, 0.035, m.planks)
            block("axe_head", (0.16, 0.025, 0.12), m.iron, at=(x + 0.05, 0.03, 1.22), bevel=0.01)
    for sx in (-1, 1):
        sh = revolve("shield", [(0.0, -0.02), (0.3, -0.02), (0.3, 0.02), (0.0, 0.02)], m.planks, n=20)
        sh.rotation_euler = (math.pi / 2 - 0.25, 0, 0)
        sh.location = (sx * 1.25, -0.25, 0.32)
    return m


MODELS = {
    "market_cross": market_cross,
    "market_stall": market_stall,
    "market_stall_b": market_stall_b,
    "cart": cart,
    "barrel": barrel_model,
    "crate": crate_model,
    "lantern_post": lantern_post,
    "training_dummy": training_dummy,
    "weapon_rack": weapon_rack,
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb")
