"""Utlegget for nivå 4, Cisternen (planen: `levels/level_04.py`).

Den store søylesalen (64 × 42 m, bunnen på −10 m) står i vann til ventilen i
ventilkammeret er stengt (spaken åpner `ventil_4`: vannet synker fra −8 til −9,5 m,
akvedukten tørker ut, og porten til kanalmunningen går opp). Du går ned i
tre trinn: nordgalleriet (0 m) med buer ut mot salen, gangbroene (−4 m) fra
trappa langs østmuren til øya med hvilestedet og videre til akvedukten, og
den nedre gangbroa langs sørmuren (−8,5 m), som ligger under vann til ventilen
er stengt. Øya og den nedre gangbroa er rom i planen som står inne i salen
(`virtual`): de bygges her, og dørene deres hogges i salens mur.

Faller du i vannet fra galleriet (buen lengst øst har ikke rekkverk), står du
på bunnen og går til en av stigene opp på øya.
"""

import math

import build_level as B
import kit_grav as K
from shapely.geometry import Point, Polygon
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
VANN = "ventil_4"
SAL = dict(x0=8.0, x1=72.0, z0=34.0, z1=76.0, bottom=-10.0, high=-8.0, low=-9.5)
OYA = dict(c=(40.0, 52.0), r=6.0, y=-4.0)
BRO = dict(y=-4.0, w=2.5, east_z=53.0, west_z=52.0)
# Trappa langs østmuren: reposet oppe (0 m), trappa ned mot sør, reposet nede (−4 m).
OST = dict(x0=69.5, x1=72.0, top=(38.75, 41.25), stair=(41.25, 50.75), foot=(50.75, 54.25))
NEDRE = dict(z0=73.0, z1=76.0, y=-8.5)
SOYLELUND = [(78, 56), (90, 56), (96, 68), (88, 82), (78, 80)]
ARENA = (40.0, 114.0)
HOL = sbox(15.4, 29.9, 16.6, 31.1)  # den hule søylen i nordgalleriet
DOWN = (40.0, -12.0, 132.6)

EXTRA = [
    ("sjakt", "Inne i den hule søylen", ("rect", 16, 30.5, 3, 3), -12, 3.0, True),
]

HALLS = [
    ("trappefot", "nordgalleri", DOOR, [(34, 15.5), (34, 29)], 3.0, None, None),
    ("nordgalleri", "cisternen", HALL, [(24, 32), (24, 34)], 2.5, None, None, {"id": "bue_vest", "name": "Buen", "yb": 0.0, "rail_b": True}),
    ("nordgalleri", "cisternen", HALL, [(40, 32), (40, 34)], 2.5, None, None, {"id": "bue_midt", "name": "Buen", "yb": 0.0, "rail_b": True}),
    ("nordgalleri", "cisternen", HALL, [(56, 32), (56, 34)], 2.5, None, None, {"id": "bue_ost", "name": "Buen", "yb": 0.0, "open_b": True}),
    ("nordgalleri", "pumpehus", DOOR, [(72, 30.5), (74, 30.5)], 2.5, None, None),
    ("pumpehus", "cisternen", HALL, [(76, 32), (76, 40), (72, 40)], 2.5, None, None, {"yb": 0.0, "name": "Gangen til østtrappa"}),
    ("pumpehus", "soylelund", STAIRS, [(84, 32), (84, 56)], 2.0, (0, 6.0, 18.0), None),
    ("akvedukt", "cisternen", HALL, [(1, BRO["west_z"]), (8, BRO["west_z"])], 2.5, None, None, {"yb": BRO["y"], "name": "Gangen til vestbroa"}),
    ("akvedukt", "bygg", SECRET, [(-5, 57), (-11.5, 57)], 2.0, None, "hemmelig_4a"),
    ("akvedukt", "munning", GATE, [(-2, 36), (-2, 30)], 2.5, None, VANN, {"name": "Kanalmunningen"}),
    ("akvedukt", "ventil", HALL, [(-2, 72), (-2, 79)], 2.5, None, None),
    ("ventil", "nedregang", STAIRS, [(2, 86), (6, 86), (6, 74.5), (8, 74.5)], 2.5, (1, 1.3, 10.2), None),
    ("nedregang", "soylelund", STAIRS, [(72, 74.5), (78, 74.5)], 2.5, None, None),
    ("nedregang", "filter", DOOR, [(40, 76), (40, 84)], 3.0, None, None),
    ("filter", "arena", GATE, [(40, 92), (40, ARENA[1] - 13.6)], 4.0, (0, 1.2, 8.4), "arenaport_4", {"name": "Porten"}),
    ("arena", "nedgang", GATE, [(40, ARENA[1] + 13.6), (40, 131)], 3.0, (0, 1.5, 3.4), "arenaport_4ut", {"name": "Porten ut"}),
    ("sjakt", "soylefot", HALL, [(16, 29), (16, 21.5)], 2.0, None, None),
]

ROUND = {
    "arena": dict(dome_rise=4.0, ribs=16, wall_cell=1.5, dome_seg=40, dome_rings=6, wall_mat="wall_dark",
                  torches=[(250, 2.6, 1.4, 10.0, False), (290, 2.6, 1.4, 10.0, False), (20, 2.6, 1.2, 9.0, False), (160, 2.6, 1.2, 9.0, False)]),
}


def ring(c, r, n=24):
    return [(c[0] + r * math.cos(2 * math.pi * k / n), c[1] + r * math.sin(2 * math.pi * k / n)) for k in range(n)]


def marks(_):
    ox, oz = OYA["c"]
    return [
        {"kind": "start", "pos": [34, 0, 13.5], "yaw": P},
        {"kind": "up", "pos": [34, 0, 9.5], "yaw": 0.0},
        {"kind": "rest", "pos": [ox, OYA["y"], oz - 2.0], "yaw": 0.0},
        # Nordgalleriet og den hule søylen.
        {"kind": "mob", "pos": [50, 0, 30.5], "label": "input", "n": 2, "patrol": [[12, 0, 30.5], [68, 0, 30.5]]},
        {"kind": "trapdoor", "id": "hul_soyle", "label": "shaft", "pos": [16, 0, 30.5], "size": [1.2, 1.2], "to": "sjakt"},
        {"kind": "hint", "label": "sound", "pos": [16, 0, 30.5], "for": "hul_soyle"},
        {"kind": "rare", "pos": [16, -12, 15.4], "yaw": 0.0, "label": "Fieldset"},
        {"kind": "ladder", "pos": [19.75, -12, 18.0], "yaw": -P / 2, "top": [31.2, 0, 13.0]},
        # Pumpehuset.
        {"kind": "mob", "pos": [79, 0, 25], "label": "select", "n": 2},
        {"kind": "chest", "pos": [84.6, 0, 23.4], "yaw": -P / 2},
        # Salen: vannet, broene og øya.
        {"kind": "water", "id": VANN, "pos": [40, SAL["high"], 55], "size": [64, 42], "levels": [SAL["high"], SAL["low"]]},
        {"kind": "mob", "pos": [60, BRO["y"], BRO["east_z"]], "label": "toast", "n": 3},
        {"kind": "mob", "pos": [20, BRO["y"], BRO["west_z"]], "label": "toast", "n": 2},
        {"kind": "ladder", "pos": [ox, SAL["bottom"], oz + OYA["r"] + 0.3], "yaw": 0.0, "top": [ox, OYA["y"], oz + OYA["r"] - 1.0]},
        {"kind": "ladder", "pos": [ox, SAL["bottom"], oz - OYA["r"] - 0.3], "yaw": P, "top": [ox, OYA["y"], oz - OYA["r"] + 1.0]},
        # Akvedukten, byggmesterens kammer og kanalmunningen.
        {"kind": "water", "id": VANN, "pos": [-4, -3.95, 54], "size": [2, 36], "levels": [-3.95, -4.4]},
        {"kind": "mob", "pos": [-1, -4, 41], "label": "input", "n": 3, "patrol": [[-1, -4, 38], [-1, -4, 70]]},
        {"kind": "trap", "label": "flood", "id": "vannstot", "pos": [-1, -4, 62], "size": [1.2, 1.2], "dir": [1, 0, 0]},
        {"kind": "secret_wall", "id": "hemmelig_4a", "pos": [-5, -4, 57], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "draft", "pos": [-4.2, -4, 57], "for": "hemmelig_4a"},
        {"kind": "lore", "pos": [-17.5, -4, 59.6], "yaw": P / 2, "label": "Byggmesterens tegninger",
         "text": "Cisternen bærer byen over seg. Førti søyler, og øya i midten for å holde hvelvet. "
                 "Stenges ventilen, synker vannet, og den nedre gangbroa kommer fram."},
        {"kind": "bonus", "pos": [-13.0, -4, 54.6], "yaw": P, "label": "liv"},
        {"kind": "gate", "id": VANN, "pos": [-2, -4, 34.6], "yaw": 0.0, "width": 2.6},
        {"kind": "chest", "pos": [-4.6, -4, 26.0], "yaw": P / 2},
        {"kind": "bonus", "pos": [0.8, -4, 25.4], "yaw": -P / 2, "label": "utholdenhet"},
        # Ventilkammeret.
        {"kind": "lever", "id": "ventilen", "pos": [-8.6, -4, 87.0], "yaw": P / 2, "opens": [VANN]},
        {"kind": "mob", "pos": [-3, -4, 82], "label": "button", "n": 2},
        {"kind": "mob", "pos": [-6, -4, 85], "label": "select"},
        # Den nedre gangbroa (under vann til ventilen er stengt).
        {"kind": "water_block", "id": VANN, "pos": [7.0, NEDRE["y"], 74.5], "yaw": P / 2, "size": [2.6, 2.8, 0.3]},
        {"kind": "water_block", "id": VANN, "pos": [73.0, NEDRE["y"], 74.5], "yaw": P / 2, "size": [2.6, 2.8, 0.3]},
        {"kind": "water_block", "id": VANN, "pos": [40, NEDRE["y"], 77.0], "yaw": 0.0, "size": [3.2, 2.8, 0.3]},
        {"kind": "mob", "pos": [22, NEDRE["y"], 74.5], "label": "button", "n": 3, "patrol": [[10, NEDRE["y"], 74.5], [70, NEDRE["y"], 74.5]]},
        # Søylelunden.
        {"kind": "mob", "pos": [86, -6, 70], "label": "toast i bakhold", "n": 4, "ambush": "soylelund"},
        {"kind": "mob", "pos": [85, -6, 59.5], "label": "input", "n": 2},
        {"kind": "chest", "pos": [89.5, -6, 75.5], "yaw": P},
        # Filterkammeret, heisen opp til øya og arenaen.
        {"kind": "mob", "pos": [36, -7, 87], "label": "select"},
        {"kind": "mob", "pos": [44, -7, 89], "label": "input", "n": 2},
        {"kind": "ladder", "pos": [46.5, -7, 84.25], "yaw": 0.0, "top": [ox + 2.0, OYA["y"], oz + 2.0]},
        {"kind": "gate", "id": "arenaport_4", "pos": [40, -7, 92.6], "yaw": 0.0, "width": 3.4, "arena": "arena"},
        {"kind": "boss", "pos": [ARENA[0], -11, ARENA[1] - 1.0], "yaw": P, "label": "table", "arena": "arena"},
        {"kind": "gate", "id": "arenaport_4ut", "label": "exit", "pos": [40, -11, ARENA[1] + 14.4], "yaw": 0.0, "width": 2.6, "arena": "arena"},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    s = rm["cisternen"]
    s.gate = VANN  # bunnen er under vann
    s.floor_mat = "slab"
    g = rm["nordgalleri"]
    g.holes.append(HOL)
    g.obstacles.append(HOL)
    rm["sjakt"].flat = HOL
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))
    # Søylene i salen (bunnen er nettet når vannet har sunket).
    L.cistern_pillars = []
    for x in (14, 22, 30, 50, 58, 66):
        for z in (39.5, 46.0, 60.0, 67.0):
            if x > 65 and z < 56:
                continue  # trappa langs østmuren
            L.cistern_pillars.append((x, z))
    for x, z in L.cistern_pillars:
        s.obstacles.append(sbox(x - 0.6, z - 0.6, x + 0.6, z + 0.6))
    s.obstacles.append(Point(OYA["c"]).buffer(OYA["r"]))


def features(L):
    mb = L.mb
    s = L.rooms["cisternen"]
    y = BRO["y"]
    # Øya: en murt sokkel fra bunnen opp til −4 m, med rekkverk rundt
    # utenom der broene og stigene kommer inn.
    c, rad = OYA["c"], OYA["r"]
    disc = Polygon(ring(c, rad, 32))
    L.block(disc, SAL["bottom"], y, mat="wall", top_mat="floor")
    ea = math.asin((BRO["east_z"] - c[1]) / rad)
    wa = P - math.asin((BRO["west_z"] - c[1]) / rad)
    hb = math.asin((BRO["w"] / 2 + 0.05) / rad)
    gaps = sorted([(ea - hb, ea + hb), (wa - hb, wa + hb), (P / 2 - 0.12, P / 2 + 0.12), (3 * P / 2 - 0.12, 3 * P / 2 + 0.12)])
    for k, (a0, a1) in enumerate(gaps):
        nxt = gaps[(k + 1) % len(gaps)][0] + (2 * P if k + 1 == len(gaps) else 0.0)
        m = max(2, math.ceil((nxt - a1) * rad / 0.8))
        path = [(c[0] + (rad - 0.13) * math.cos(a1 + (nxt - a1) * q / m), c[1] + (rad - 0.13) * math.sin(a1 + (nxt - a1) * q / m)) for q in range(m + 1)]
        K.parapet(mb, path, y, h=0.95, t=0.26)
        for q0, q1 in zip(path, path[1:]):
            L.wall_box(q0, q1, y, y + 1.0, (0.0, 0.0))
    # Murblokker bak stigene opp på øya (inne i sokkelen; kollideren ved
    # foten av stigen).
    for sz in (1.0, -1.0):
        L.box((c[0], (SAL["bottom"] + y) / 2, c[1] + sz * (rad - 0.25)), (1.2, y - SAL["bottom"], 0.5))
    top_walk = Point(c).buffer(rad - 0.26 - B.INSET, 48)
    o = L.rooms["oya"]
    # Broene: øst fra reposet ved trappa, vest til gangen inn i akvedukten.
    _, be = L.stone_bridge((OST["x0"], BRO["east_z"]), (c[0] + rad - 0.3, BRO["east_z"]), BRO["w"], y, clip=disc)
    _, bw = L.stone_bridge((c[0] - rad + 0.3, BRO["west_z"]), (SAL["x0"], BRO["west_z"]), BRO["w"], y, clip=disc)
    for bx, bz in ((53.0, BRO["east_z"]), (61.0, BRO["east_z"]), (17.0, BRO["west_z"]), (26.0, BRO["west_z"])):
        K.pillar(mb, bx, bz, SAL["bottom"], y - 0.62, size=0.8)
        L.box((bx, (SAL["bottom"] + y) / 2 - 0.3, bz), (1.0, y - SAL["bottom"] - 0.6, 1.0))
        s.obstacles.append(sbox(bx - 0.6, bz - 0.6, bx + 0.6, bz + 0.6))
    # Trappa langs østmuren fra gangen fra pumpehuset ned til broa.
    top = sbox(OST["x0"], OST["top"][0], OST["x1"], OST["top"][1])
    foot = sbox(OST["x0"], OST["foot"][0], OST["x1"], OST["foot"][1])
    L.block(top, SAL["bottom"], 0.0)
    L.block(foot, SAL["bottom"], y)
    xm = (OST["x0"] + OST["x1"]) / 2
    stair = L.wall_stair("cisternen", (xm, OST["stair"][0]), (xm, OST["stair"][1]), OST["x1"] - OST["x0"], 0.0, y, SAL["bottom"], +1)
    s.obstacles += [sbox(OST["x0"], OST["top"][0], OST["x1"], OST["foot"][1])]
    # Reposene: inset fra kanten mot salen.
    top_w = sbox(OST["x0"] + B.INSET, OST["top"][0] + B.INSET, OST["x1"] - B.INSET, OST["top"][1] + 0.01)
    foot_w = sbox(OST["x0"] + B.INSET, OST["foot"][0] - 0.01, OST["x1"] - B.INSET, OST["foot"][1] - B.INSET)
    L.decks.append(dict(room="cisternen", of="cisternen", y=0.0, walk=top_w, ring=top.buffer(0.3), obstacles=[]))
    o.floor_walk = unary_union([top_walk, be.intersection(disc.buffer(1.0)), bw.intersection(disc.buffer(1.0))])
    L.decks.append(dict(room="cisternen", of="cisternen", y=y, walk=unary_union([foot_w, be.difference(disc.buffer(0.5)), bw.difference(disc.buffer(0.5))]),
                        ring=foot.buffer(0.3), obstacles=[]))
    # Den nedre gangbroa langs sørmuren: en murt hylle fra bunnen.
    n = L.rooms["nedregang"]
    shelf = sbox(SAL["x0"], NEDRE["z0"], SAL["x1"], NEDRE["z1"])
    L.block(shelf, SAL["bottom"], NEDRE["y"], mat="dressed", top_mat="slab", cell=2.0)
    n.floor_walk = sbox(SAL["x0"] + B.INSET, NEDRE["z0"] + B.INSET, SAL["x1"] - B.INSET, NEDRE["z1"] - B.INSET)
    n.gate = VANN
    s.obstacles.append(shelf)
    # Søylene i salen, opp til hvelvet.
    for x, z in L.cistern_pillars:
        K.pillar(mb, x, z, SAL["bottom"], B.pillar_top(s, x, z, 0.9), size=0.9)
        L.box((x, (SAL["bottom"] + 10.0) / 2, z), (1.2, 20.0, 1.2))


def dress(L):
    mb = L.mb
    rm = L.rooms
    # Trappefoten.
    t = rm["trappefot"]
    for x in (31.2, 36.8):
        K.candlestick(mb, x, 15.0, t.y, arms=2)
        t.obstacles.append(Point(x, 15.0).buffer(0.35))
        L.box((x, t.y + 0.7, 15.0), (0.5, 1.4, 0.5))
        L.light((x, t.y + 1.55, 15.0), B.CANDLE, 5.0, "trappefot", True, 1.0)
    # Nordgalleriet: fakler mellom buene.
    for x in (12, 32, 48, 64):
        L.wall_torch("nordgalleri", "n", x, 2.2, intensity=1.6, reach=9.0)
    L.shaft(HOL, -12 + 3.0, 0.0)
    # Pumpehuset: pumper og kjettinger.
    ph = rm["pumpehus"]
    for x, z in ((78.0, 24.0), (82.0, 24.0)):
        K.cylinder(mb, (x, z), 0.55, ph.y, ph.y + 1.6, "iron", n=10, caps=(True, False), smooth=False)
        K.cylinder(mb, (x, z), 0.15, ph.y + 1.6, ph.y + 3.0, "iron", n=6, caps=(True, False), smooth=False)
        ph.obstacles.append(Point(x, z).buffer(0.6))
        L.box((x, ph.y + 0.8, z), (1.2, 1.6, 1.2))
    L.wall_torch("pumpehus", "e", 27.0, 2.4, intensity=1.6)
    # Salen: kaldt lys over vannet, fakler ved broene, lysestaker på øya.
    s = rm["cisternen"]
    for x, z in ((24, 45), (56, 45), (24, 65), (56, 65)):
        L.light((x, SAL["high"] + 3.0, z), B.COLD, 14.0, "cisternen", True, 0.7)
    L.torch("cisternen", (OST["x1"] - 0.02, 2.2, 40.0 + 1.8), (-1.0, 0.0), intensity=1.6, reach=9.0)
    L.torch("cisternen", (OST["x1"] - 0.02, BRO["y"] + 2.2, 52.5), (-1.0, 0.0), intensity=1.6, reach=9.0)
    L.torch("cisternen", (SAL["x0"] + 0.02, BRO["y"] + 2.2, 49.6), (1.0, 0.0), intensity=1.6, reach=9.0)
    c = OYA["c"]
    for dx in (-2.0, 2.0):
        K.candlestick(mb, c[0] + dx, c[1] + 1.6, OYA["y"], arms=2)
        rm["oya"].obstacles.append(Point(c[0] + dx, c[1] + 1.6).buffer(0.35))
        L.box((c[0] + dx, OYA["y"] + 0.7, c[1] + 1.6), (0.5, 1.4, 0.5))
        L.light((c[0] + dx, OYA["y"] + 1.55, c[1] + 1.6), B.CANDLE, 9.0, "oya", True, 1.4)
    # Glørfatet på øya: landemerket som synes fra galleriet og broene.
    bx, bz = c[0], c[1] + 3.2
    K.cylinder(mb, (bx, bz), 0.6, OYA["y"], OYA["y"] + 0.8, "iron", n=10, caps=(True, False), smooth=False)
    K.cylinder(mb, (bx, bz), 0.55, OYA["y"] + 0.8, OYA["y"] + 0.86, "fire_torch", n=10, caps=(True, False), smooth=False)
    rm["oya"].obstacles.append(Point(bx, bz).buffer(0.65))
    L.box((bx, OYA["y"] + 0.4, bz), (1.3, 0.8, 1.3))
    L.light((bx, OYA["y"] + 1.8, bz), (1.0, 0.5, 0.2), 18.0, "oya", True, 3.2)
    # Lysestaker ved broendene.
    for x, z, room in ((OST["x1"] - 0.4, OST["foot"][1] - 0.4, "cisternen"), (0.2, BRO["west_z"] - 1.9, "akvedukt")):
        K.candlestick(mb, x, z, BRO["y"], arms=2)
        L.box((x, BRO["y"] + 0.7, z), (0.5, 1.4, 0.5))
        L.light((x, BRO["y"] + 1.55, z), B.CANDLE, 8.0, room, True, 1.3)
        if room == "akvedukt":
            rm["akvedukt"].obstacles.append(Point(x, z).buffer(0.35))
        for d in L.decks:
            if room == "cisternen" and d["room"] == "cisternen" and abs(d["y"] - BRO["y"]) < 0.01:
                d["obstacles"].append(Point(x, z).buffer(0.35))
    for x in range(12, 72, 12):
        L.light((x, NEDRE["y"] + 2.4, NEDRE["z1"] - 0.3), (0.55, 0.9, 0.5), 7.0, "nedregang", True, 0.6)
    # Akvedukten og kamrene.
    for z in (40.0, 56.0, 68.0):
        L.wall_torch("akvedukt", "e", z, 2.4, intensity=1.4, reach=8.0)
    L.light((-14.5, -4 + 2.0, 59.0), (1.0, 0.8, 0.45), 5.0, "bygg", True, 1.0)
    L.light((-2.0, -4 + 2.0, 25.0), B.COLD, 5.0, "munning", True, 0.7)
    L.wall_torch("ventil", "w", 82.0, 2.4, intensity=1.6)
    L.light((-4.0, -4 + 3.0, 86.0), (0.55, 0.9, 0.5), 8.0, "ventil", True, 0.8)
    # Søylelunden: søyler opp til hvelvet, kaldt lys.
    sl = rm["soylelund"]
    for x, z in ((82, 62), (88, 63), (85, 70), (91, 69), (82, 76), (87, 78)):
        K.pillar(mb, x, z, sl.y, L.poly_vault_y(sl, x, z) + 0.15, size=0.7)
        sl.obstacles.append(sbox(x - 0.55, z - 0.55, x + 0.55, z + 0.55))
        L.box((x, sl.y + 4.0, z), (1.0, 8.0, 1.0))
    L.light((84.0, sl.y + 3.0, 66.0), (0.55, 0.9, 0.5), 10.0, "soylelund", True, 0.9)
    # Filterkammeret: jerngitter, og det blå skjæret fra arenaen.
    f = rm["filter"]
    L.wall_torch("filter", "n", 36.0, 2.3, intensity=1.6)
    L.wall_torch("filter", "s", 35.0, 2.3, intensity=1.4)
    # Arenaen: svart vann rundt fire plattformer (lave sokler).
    ar = rm["arena"]
    for px, pz, pr in ((33, ARENA[1] - 6, 2.2), (48, ARENA[1] - 5, 2.0), (34, ARENA[1] + 6, 2.0), (47, ARENA[1] + 6, 2.2)):
        L.block(Polygon(ring((px, pz), pr, 12)), ar.y, ar.y + 0.4, mat="dressed", top_mat="slab")
        ar.obstacles.append(Point(px, pz).buffer(pr))
        L.light((px, ar.y + 1.2, pz), B.BLUE, 8.0, "arena", True, 0.9)
    # Kamrene i søylefoten.
    L.light((18.5, -12 + 2.0, 20.0), (1.0, 0.8, 0.45), 5.0, "soylefot", True, 1.0)
    L.light((16.0, -12 + 2.0, 31.5), B.COLD, 4.0, "sjakt", False, 0.5)
    # Nedgangen.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 3.0)
    L.wall_torch("nedgang", "w", 134.0, 2.4, intensity=1.4)
    # Gangene.
    L.torch("pumpehus-soylelund", (84 - 0.92, -6 + 1.9, 52.0), (1.0, 0.0), intensity=1.0, reach=7.0)
    L.torch("ventil-nedregang", (6 - 1.17, -8.5 + 1.9, 76.5), (1.0, 0.0), intensity=1.0, reach=7.0)


LAYOUT = dict(
    theme="kloakk",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round=ROUND,
    virtual={"oya": "cisternen", "nedregang": "cisternen"},
    override={
        "pumpehus": {"shape": ("rect", 80, 27, 12, 10)},
        "nedregang": {"shape": ("rect", 40, 74.5, 64, 3), "y": NEDRE["y"]},
        "soylelund": {"shape": ("poly", SOYLELUND)},
        "arena": {"shape": ("circle", ARENA[0], ARENA[1], 14), "h": 12.0},
    },
    slab=("akvedukt", "filter"),
    wall_mats={"cisternen": "wall_dark", "soylefot": "wall_dark", "sjakt": "wall_dark", "munning": "wall_dark", "bygg": "wall_dark",
               "soylelund": "wall_dark"},
    cells={"cisternen": 2.0},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
)
