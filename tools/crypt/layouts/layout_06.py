"""Utlegget for nivå 6, Krystallgrotta (planen: `levels/level_06.py`).

Gruva bryter gjennom til naturlige grotter. Fra stollmunningen går
krystallgangen (blå krystaller langs berget) ned til nordstranden ved
undergrunnssjøen: en stor hall (`sjoen`) med bunnen 6 m under strendene og
svart vann imellom. Strendene og holmen er rom som står inne i sjøen
(`virtual`, som øya i cisternen): murte blokker fra bunnen opp til
strandhøyden, med steinbroer nord–holme–sør. Faller du i vannet, står du på
bunnen og går til stigene opp på holmen; hullet med boblene i bunnen er
dykket ned til den skjulte kilden (sjakt → kildesjakta → kilden → sprekken
ut i vindeltrappa). Fra sørstranden går Lysgrotta (hvilestedet, under den
rosa krystallen), Ekkogrotta (bakholdet) og Dryppsteinshallen (skog av
drypp- og stalagmitter, de tre resonanskrystallene som åpner Klangkammeret).
Kløfta går ned til vindeltrappa hogd i berget rundt krystallsøyla (galleriet
på −10 m, gulvet på −16 m), forkammeret, og det store grottekammeret der
Dropdownen (en select) venter mellom krystallsøylene. Snarveien er sprekken
fra forkammeret opp til Lysgrotta (slåen nede).

Alle rommene fra planen er justert (`override`) så gangene treffer rette
kanter, og gangene inn i de runde rommene går radielt langs x eller z.
Grottene har berg (`rock`) i murene og hvelvene (`vault_mats`).
"""

import math

import build_level as B
import kit_grav as K
import kit_mine as KM
from shapely.geometry import Point, Polygon
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
VANN = "sjoen_6"  # vannet i sjøen synker aldri (ingen spak åpner det)
KLANG = "hemmelig_6b"
# Sjøen: hallen, bunnen, vannflaten og strendene.
LAKE = dict(x0=29.0, x1=75.0, z0=22.0, z1=56.0, bottom=-9.0, water=-6.3, shore=-3.0)
HOLME = dict(c=(52.0, 39.0), r=6.0)
BRIDGE_W = 2.5
# Hullet med boblene i bunnen: dykket ned til kildesjakta.
HOLE = sbox(69.3, 49.3, 70.7, 50.7)
CAVE = -16.0  # gulvet i vindeltrappa, forkammeret og grottekammeret
GALLERI = -10.0  # hylla øverst i vindeltrappa, der kløfta kommer inn
VINDEL = dict(c=(80.0, 96.0), r=6.0)
HVILE = dict(c=(26.0, 74.0), r=6.0)
KLANGC = (122.0, 35.0)
DOWN = (-4.0, CAVE, 103.6)

KRYSTALL = [(8, -3), (8, 3), (16, 6), (26, 10), (30, 18), (36, 18), (36, 8), (28, 2), (18, -5)]
EKKO = [(22, 46), (22, 60), (14, 64), (6, 60), (2, 50), (8, 42), (16, 40)]
DRYPP = [(80, 20), (80, 32), (84, 40), (84, 50), (88, 56), (100, 56), (108, 50), (112, 42), (112, 28), (104, 18), (94, 16)]
KLOFT = [(88, 66), (98, 66), (101, 76), (97, 88), (89, 88), (86, 76)]
FORKAMMER = [(66, 88), (66, 104), (60, 108), (52, 108), (52, 92), (56, 88)]
DROPDOWN = [(14, 82), (34, 78), (46, 86), (46, 106), (38, 118), (18, 120), (6, 106), (6, 92)]

EXTRA = [
    ("kildesjakt", "Kildesjakta", ("rect", 70, 50, 3, 3), CAVE, 3.0, True),
]

HALLS = [
    ("landing", "krystall", HALL, [(4, 0), (8, 0)], 3.0, None, None),
    ("krystall", "nord", HALL, [(33, 18), (33, 22)], 3.0, None, None),
    ("nord", "drypp", HALL, [(75, 24), (80, 24)], 2.5, None, None, {"name": "Hylla langs berget"}),
    ("sor", "drypp", HALL, [(75, 54), (80, 54), (80, 46), (84, 46)], 3.0, None, None),
    ("sor", "hvile", DOOR, [(34, 56), (34, 62), (26, 62), (26, 68.4)], 2.5, None, None),
    ("sor", "ekko", HALL, [(29, 54), (22, 54)], 2.5, None, None),
    ("drypp", "klang", SECRET, [(112, 35), (117.4, 35)], 2.0, None, KLANG, {"name": "Krystallveggen"}),
    ("drypp", "kloft", STAIRS, [(93, 56), (93, 66)], 3.0, None, None),
    ("kloft", "vindel", STAIRS, [(93, 88), (93, 96), (85.6, 96)], 2.5, None, None, {"yb": GALLERI}),
    ("kildesjakt", "kilde", HALL, [(70, 51.5), (70, 60)], 2.0, None, None),
    ("kilde", "vindel", HALL, [(80, 70), (80, 90.4)], 2.0, None, None, {"name": "Sprekken"}),
    ("vindel", "forkammer", DOOR, [(74.4, 96), (66, 96)], 3.0, None, None),
    ("forkammer", "dropdown", GATE, [(52, 98), (46, 98)], 4.0, None, "arenaport_6", {"name": "Porten"}),
    ("dropdown", "nedgang", GATE, [(6, 102), (0, 102)], 3.0, None, "arenaport_6ut", {"name": "Porten ut"}),
    ("forkammer", "hvile", SHORTCUT, [(60, 88), (60, 74), (31.6, 74)], 2.2, (1, 0.0, 28.4), "snarvei_6", {"name": "Sprekken opp til Lysgrotta"}),
]

ROUND = {
    "hvile": dict(dome_rise=2.5, ribs=6, wall_cell=1.5, dome_seg=32, dome_rings=5, wall_mat="rock",
                  torches=[(200, 2.2, 1.2, 8.0, False), (340, 2.2, 1.2, 8.0, False)]),
    "klang": dict(dome_rise=2.0, ribs=6, wall_cell=1.5, dome_seg=28, dome_rings=4, wall_mat="rock"),
    "vindel": dict(
        gallery=dict(y=GALLERI, ring_in=3.8, stair_in=4.0, stair_a0=math.radians(380), stair_a1=math.radians(520), steps=30, segs=36,
                     id="vindelhylle", name="Hylla øverst i vindeltrappa", stair_id="vindeltrapp", stair_name="Vindeltrappa"),
        dome_rise=3.0, ribs=8, wall_cell=1.5, dome_seg=36, dome_rings=5, wall_mat="rock",
        torches=[(250, 2.3, 1.3, 8.0, False), (200, 2.0, 1.2, 8.0, True)],
    ),
}


def marks(_):
    hx, hz = HOLME["c"]
    hr = HOLME["r"]
    sh = LAKE["shore"]
    return [
        {"kind": "start", "pos": [0, 0, 1.5], "yaw": P},
        {"kind": "up", "pos": [0, 0, -3], "yaw": 0.0},
        {"kind": "rest", "pos": [HVILE["c"][0], -4, HVILE["c"][1] + 1.0], "yaw": P},
        # Krystallgangen.
        {"kind": "mob", "pos": [26, -1.5, 8], "label": "spinner", "n": 2, "patrol": [[12, -1.5, 1], [32, -1.5, 14]]},
        {"kind": "chest", "pos": [18.5, -1.5, -3.4], "yaw": 0.0},
        # Nordstranden, holmen og sørstranden.
        {"kind": "mob", "pos": [42, sh, 24], "label": "tooltip", "n": 3},
        {"kind": "mob", "pos": [64, sh, 24], "label": "table"},
        {"kind": "mob", "pos": [hx - 2.5, sh, hz - 1.0], "label": "accordion"},
        {"kind": "chest", "pos": [hx + 3.2, sh, hz + 2.6], "yaw": P * 0.75},
        {"kind": "water", "id": VANN, "pos": [(LAKE["x0"] + LAKE["x1"]) / 2, LAKE["water"], (LAKE["z0"] + LAKE["z1"]) / 2],
         "size": [LAKE["x1"] - LAKE["x0"], LAKE["z1"] - LAKE["z0"]], "levels": [LAKE["water"], LAKE["water"]]},
        {"kind": "ladder", "pos": [hx + hr + 0.3, LAKE["bottom"], hz], "yaw": P / 2, "top": [hx + hr - 1.0, sh, hz]},
        {"kind": "ladder", "pos": [hx - hr - 0.3, LAKE["bottom"], hz], "yaw": -P / 2, "top": [hx - hr + 1.0, sh, hz]},
        {"kind": "trapdoor", "id": "bobler_6", "label": "shaft", "pos": [70, LAKE["bottom"], 50], "size": [1.4, 1.4], "to": "kildesjakt"},
        {"kind": "hint", "label": "sound", "pos": [70, LAKE["bottom"], 50], "for": "bobler_6"},
        {"kind": "mob", "pos": [46, sh, 54], "label": "accordion", "n": 2},
        {"kind": "mob", "pos": [60, sh, 54], "label": "tooltip", "n": 2},
        # Ekkogrotta: bakholdet, kista og gruvemesterens tavle.
        {"kind": "mob", "pos": [12, -4, 52], "label": "accordion i bakhold", "n": 3, "ambush": "ekko"},
        {"kind": "chest", "pos": [5.0, -4, 51.0], "yaw": -P / 2},
        {"kind": "lore", "pos": [16.5, -4, 61.6], "yaw": P, "label": "Gruvemesterens tavle",
         "text": "Vi brøt gjennom til grottene i den tolvte uka. Krystallene lyser av seg selv, og vannet i sjøen er "
                 "svart og stille. Langs stranda går et spor etter noe tungt som er slept nedover, ledd for ledd. Tre "
                 "av karene gikk ned i kløfta etter det og kom ikke opp. Det som synger der nede, er ikke vind."},
        # Dryppsteinshallen: de tre resonanskrystallene (spakene) åpner krystallveggen når alle er slått an.
        {"kind": "mob", "pos": [86, -5, 27], "label": "accordion i bakhold", "ambush": 4.0},
        {"kind": "mob", "pos": [94, -5, 36], "label": "spinner", "n": 3, "patrol": [[86, -5, 30], [100, -5, 24], [104, -5, 42], [90, -5, 48]]},
        {"kind": "mob", "pos": [100, -5, 46], "label": "modal"},
        {"kind": "lever", "id": "klang_1", "pos": [90.0, -5, 22.0], "yaw": P, "opens": [KLANG]},
        {"kind": "lever", "id": "klang_2", "pos": [109.0, -5, 28.0], "yaw": -P / 2, "opens": [KLANG]},
        {"kind": "lever", "id": "klang_3", "pos": [88.0, -5, 50.0], "yaw": P / 2, "opens": [KLANG]},
        {"kind": "secret_wall", "id": KLANG, "pos": [112, -5, 35], "yaw": -P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [111.95, -5, 36.6], "yaw": -P / 2, "for": KLANG},
        {"kind": "hint", "label": "sound", "pos": [110.5, -5, 35], "for": KLANG},
        {"kind": "rare", "pos": [125.8, -5, 35.0], "yaw": -P / 2, "label": "Progress"},
        # Kløfta: krystallene i taket dirrer og faller.
        {"kind": "trap", "label": "rocks", "id": "krystallras_6a", "pos": [93, -9, 71], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "trap", "label": "rocks", "id": "krystallras_6b", "pos": [92, -9, 82], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "mob", "pos": [93, -9, 77], "label": "tooltip", "n": 3},
        # Den skjulte kilden.
        {"kind": "lore", "pos": [84.5, CAVE, 61.6], "yaw": P, "label": "Den druknede gruvearbeideren",
         "text": "Her kom jeg opp. Luft, og lys fra krystallene. Sprekken i sørveggen fører ut i den store trappa. Der "
                 "nede hører jeg noe folde seg ut, lag på lag, og to stykker jern som leter etter hverandre i mørket. "
                 "De finner hverandre. Jeg går ikke ned igjen."},
        {"kind": "bonus", "pos": [83.0, CAVE, 69.4], "yaw": P, "label": "utholdenhet"},
        # Vindeltrappa og forkammeret.
        {"kind": "mob", "pos": [77, CAVE, 93], "label": "spinner", "n": 2},
        {"kind": "mob", "pos": [59, CAVE, 98], "label": "table", "n": 2},
        {"kind": "shortcut", "id": "snarvei_6", "from": "forkammer", "pos": [60, CAVE, 88], "yaw": 0.0, "width": 1.6},
        # Det store grottekammeret: Dropdownen.
        {"kind": "gate", "id": "arenaport_6", "pos": [50.3, CAVE, 98], "yaw": P / 2, "width": 3.4, "arena": "dropdown"},
        {"kind": "boss", "pos": [26, CAVE, 100], "yaw": 0.0, "label": "dropdown", "arena": "dropdown"},
        {"kind": "chest", "pos": [30.0, CAVE, 115.0], "yaw": P * 0.9},
        {"kind": "gate", "id": "arenaport_6ut", "label": "exit", "pos": [2.0, CAVE, 102], "yaw": P / 2, "width": 2.6, "arena": "dropdown"},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    s = rm["sjoen"]
    s.floor_y = LAKE["bottom"]
    s.floor_mat = "rubble"
    s.nowalk = True
    s.holes.append(HOLE)
    rm["kildesjakt"].flat = HOLE
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))


def crystal(L, room, x, z, yaw=0.0, reach=12.0, y=None, deck=None):
    """En klynge krystaller fra byggesettet (`kit_crystal`) med blått lys."""
    r = L.rooms[room]
    yy = r.y if y is None else y
    L.prop("kit_crystal", x, yy, z, yaw)
    L.box((x, yy + 0.4, z), (1.1, 0.8, 1.1), yaw)
    (deck["obstacles"] if deck is not None else r.obstacles).append(Point(x, z).buffer(0.65))
    L.light((x, yy + 1.3, z), B.BLUE, reach, room, False, 5.0)


def stalagmite(L, room, x, z, r=0.5, h=1.6, seed=0):
    rr = L.rooms[room]
    KM.cone(L.mb, (x, z), r, rr.y, rr.y + h, mat="rock", n=8, seed=seed)
    L.box((x, rr.y + h * 0.35, z), (r * 1.3, h * 0.7, r * 1.3))
    rr.obstacles.append(Point(x, z).buffer(r * 0.9))


def stalactite(L, room, x, z, r=0.45, h=2.2, seed=0):
    rr = L.rooms[room]
    top = L.poly_vault_y(rr, x, z) - 0.05
    KM.cone(L.mb, (x, z), r, top, top - h, mat="rock", n=8, seed=seed)


def features(L):
    mb = L.mb
    rm = L.rooms
    s = rm["sjoen"]
    sh, bottom = LAKE["shore"], LAKE["bottom"]
    x0, x1, z0, z1 = LAKE["x0"], LAKE["x1"], LAKE["z0"], LAKE["z1"]
    I = B.INSET
    # Strendene: murte blokker fra bunnen opp til strandhøyden, inn i murene
    # (så det ikke står en spalte mellom blokka og berget), grus på toppen.
    nord_rect = sbox(x0 - 0.5, z0 - 0.5, x1 + 0.5, z0 + 4.0)
    sor_rect = sbox(x0 - 0.5, z1 - 4.0, x1 + 0.5, z1 + 0.5)
    L.block(nord_rect, bottom, sh, mat="rock", top_mat="rubble", cell=2.0)
    L.block(sor_rect, bottom, sh, mat="rock", top_mat="rubble", cell=2.0)
    # Holmen: en rund blokk med rekkverk, utenom broene (nord og sør) og stigene (øst og vest).
    c, rad = HOLME["c"], HOLME["r"]
    disc = Polygon([(c[0] + rad * math.cos(2 * P * k / 32), c[1] + rad * math.sin(2 * P * k / 32)) for k in range(32)])
    L.block(disc, bottom, sh, mat="rock", top_mat="rubble")
    hb = math.asin((BRIDGE_W / 2 + 0.05) / rad)
    gaps = sorted([(P / 2 - hb, P / 2 + hb), (3 * P / 2 - hb, 3 * P / 2 + hb), (-0.12, 0.12), (P - 0.12, P + 0.12)])
    for k, (a0, a1) in enumerate(gaps):
        nxt = gaps[(k + 1) % len(gaps)][0] + (2 * P if k + 1 == len(gaps) else 0.0)
        m = max(2, math.ceil((nxt - a1) * rad / 0.8))
        path = [(c[0] + (rad - 0.13) * math.cos(a1 + (nxt - a1) * q / m), c[1] + (rad - 0.13) * math.sin(a1 + (nxt - a1) * q / m)) for q in range(m + 1)]
        K.parapet(mb, path, sh, h=0.95, t=0.26)
        for q0, q1 in zip(path, path[1:]):
            L.wall_box(q0, q1, sh, sh + 1.0, (0.0, 0.0))
    # Flate murflater bak stigene (kollideren ved foten av stigen).
    for sx in (1.0, -1.0):
        L.box((c[0] + sx * (rad - 0.25), (bottom + sh) / 2, c[1]), (0.5, sh - bottom, 1.2))
    # Broene: steinbroer fra strendene til holmen.
    _, bn = L.stone_bridge((c[0], z0 + 4.0), (c[0], c[1] - rad + 0.3), BRIDGE_W, sh, clip=disc)
    _, bs = L.stone_bridge((c[0], c[1] + rad - 0.3), (c[0], z1 - 4.0), BRIDGE_W, sh, clip=disc)
    for bz in (z0 + 6.5, z1 - 6.5):
        K.pillar(mb, c[0], bz, bottom, sh - 0.62, size=0.8, mat="rock")
        L.box((c[0], (bottom + sh) / 2 - 0.3, bz), (1.0, sh - bottom - 0.6, 1.0))
    # Navigasjonen: strendene og holmen med hver sin del av broene (de
    # overlapper langs brosidene, som øya og broene i cisternen).
    rm["nord"].floor_walk = unary_union([sbox(x0 + I, z0 + I, x1 - I, z0 + 4.0 - I), bn.difference(disc.buffer(0.5))])
    rm["sor"].floor_walk = unary_union([sbox(x0 + I, z1 - 4.0 + I, x1 - I, z1 - I), bs.difference(disc.buffer(0.5))])
    rm["holme"].floor_walk = unary_union([Point(c).buffer(rad - 0.26 - I, 48), bn.intersection(disc.buffer(1.0)), bs.intersection(disc.buffer(1.0))])
    # Den hvite krystallen på holmen: landemerket som lyser over hele sjøen.
    KM.spire(mb, (c[0], sh, c[1]), 4.2, 0.55, mat="magic_white", lean=(0.06, -0.04), seed=5)
    L.box((c[0], sh + 2.0, c[1]), (1.5, 4.0, 1.5))
    rm["holme"].obstacles.append(Point(c).buffer(1.1))
    L.light((c[0], sh + 3.2, c[1]), (0.85, 0.92, 1.0), 60.0, "holme", False, 28.0)
    # Sjakta fra hullet i bunnen ned til kildesjakta.
    L.shaft(HOLE, CAVE + 3.0, bottom)
    # Vindeltrappa: krystallsøyla midt i, fra gulvet opp i kuppelen.
    v = rm["vindel"]
    vc = VINDEL["c"]
    K.cylinder(mb, vc, 1.3, v.y, v.y + v.h + 1.5, "rock", n=12, caps=(False, False), smooth=False)
    L.box((vc[0], v.y + (v.h + 1.5) / 2, vc[1]), (2.6, v.h + 1.5, 2.6))
    v.obstacles.append(Point(vc).buffer(1.5))
    # Foten av trappa: nettet holdes unna enden av det indre rekkverket, så
    # veien ned på gulvet ikke skjærer hjørnet rundt det (boten sto fast der).
    gal = ROUND["vindel"]["gallery"]
    v.obstacles.append(Point(vc[0] + gal["stair_in"] * math.cos(gal["stair_a0"]), vc[1] + gal["stair_in"] * math.sin(gal["stair_a0"])).buffer(0.5))
    for k in range(6):
        a = 2 * P * k / 6 + 0.3
        for yy, hh in ((v.y + 0.1, 1.6), (GALLERI + 0.3, 1.2), (v.y + 4.0, 1.0)):
            base = (vc[0] + 1.3 * math.cos(a), yy, vc[1] + 1.3 * math.sin(a))
            tip = (vc[0] + (1.3 + hh * 0.7) * math.cos(a), yy + hh * 0.7, vc[1] + (1.3 + hh * 0.7) * math.sin(a))
            KM.hex_prism(mb, base, tip, 0.14, "magic_crystal")
    L.light((vc[0], v.y + 3.0, vc[1]), B.BLUE, 16.0, "vindel", False, 3.0)
    L.light((vc[0], GALLERI + 2.5, vc[1]), B.BLUE, 12.0, "vindelhylle", False, 1.6)
    # Det store grottekammeret: krystallsøyler Dropdownen folder seg rundt.
    d = rm["dropdown"]
    for k, (x, z) in enumerate(((18, 92), (36, 90), (16, 110), (34, 112), (26, 88))):
        K.pillar(mb, x, z, d.y, L.poly_vault_y(d, x, z) + 0.15, size=1.0, mat="rock")
        d.obstacles.append(sbox(x - 0.7, z - 0.7, x + 0.7, z + 0.7))
        L.box((x, d.y + 7.0, z), (1.2, 14.0, 1.2))
        for j in range(3):
            a = 2 * P * j / 3 + k
            base = (x + 0.5 * math.cos(a), d.y + 0.8 + 0.9 * j, z + 0.5 * math.sin(a))
            tip = (x + 1.3 * math.cos(a), d.y + 1.5 + 0.9 * j, z + 1.3 * math.sin(a))
            KM.hex_prism(mb, base, tip, 0.13, "magic_crystal")
        L.light((x, d.y + 2.5, z), B.BLUE, 14.0, "dropdown", False, 2.6)
    # Lysgrotta: den store rosa krystallen hengende over alteret.
    h = rm["hvile"]
    hc = HVILE["c"]
    top = h.y + h.h
    KM.hex_prism(mb, (hc[0], top + 1.8, hc[1]), (hc[0] + 0.3, h.y + 3.6, hc[1] + 0.2), 0.55, "magic_pink")
    for k in range(4):
        a = 2 * P * k / 4 + 0.5
        KM.hex_prism(mb, (hc[0] + 1.0 * math.cos(a), top + 1.2, hc[1] + 1.0 * math.sin(a)), (hc[0] + 1.6 * math.cos(a), h.y + 4.3, hc[1] + 1.6 * math.sin(a)), 0.22, "magic_pink")
    L.light((hc[0], h.y + 3.0, hc[1]), (1.0, 0.5, 0.8), 16.0, "hvile", False, 3.2)
    # Dryppsteinshallen: skog av stalagmitter og dryppsteiner.
    for k, (x, z, r, hh) in enumerate(((88, 24, 0.5, 1.8), (96, 22, 0.4, 1.2), (102, 30, 0.6, 2.4), (92, 44, 0.5, 1.6), (104, 48, 0.4, 1.4),
                                      (98, 38, 0.7, 2.8), (88, 36, 0.35, 1.0), (106, 40, 0.5, 2.0), (96, 50, 0.45, 1.3))):
        stalagmite(L, "drypp", x, z, r, hh, seed=k)
    for k, (x, z) in enumerate(((90, 28), (100, 26), (106, 34), (94, 42), (100, 46), (88, 42), (98, 32), (104, 26))):
        stalactite(L, "drypp", x, z, r=0.4 + 0.05 * (k % 3), h=1.8 + 0.4 * (k % 4), seed=10 + k)
    # Kløfta: løse krystaller i taket.
    kl = rm["kloft"]
    for k, (x, z) in enumerate(((91, 70), (96, 74), (90, 80), (95, 85))):
        top = L.poly_vault_y(kl, x, z) - 0.05
        KM.hex_prism(mb, (x, top, z), (x + 0.2, top - 2.0 - 0.3 * k, z - 0.1), 0.3, "magic_crystal")
        L.light((x, top - 2.5, z), B.BLUE, 16.0, "kloft", False, 6.0)


def dress(L):
    mb = L.mb
    rm = L.rooms
    sh = LAKE["shore"]
    # Stollmunningen: de siste stiverne fra gruva, fakler.
    la = rm["landing"]
    for z in (-1.5, 1.5):
        L.prop("kit_strut", 0.0, la.y, z, 0.0)  # stolpene ved veggene (x = ±1,6), ikke midt i veien
        L.box((0.0, la.y + 2.76, z), (3.5, 0.28, 0.28), P / 2)
        for sx in (-1, 1):
            L.box((sx * 1.55, la.y + 1.45, z), (0.3, 2.9, 0.3))
            la.obstacles.append(sbox(sx * 1.55 - 0.16, z - 0.16, sx * 1.55 + 0.16, z + 0.16))
    L.wall_torch("landing", "w", 0.0, 2.3, intensity=1.5)
    L.wall_torch("landing", "e", 1.5, 2.3, intensity=1.5)
    L.light((0.0, 2.0, -3.0), B.COLD, 6.0, "landing", False, 0.8)
    # Krystallgangen: krystallene langs berget lyser veien.
    for x, z, yw in ((10.0, -1.5, 0.3), (15.0, 4.6, 1.1), (21.0, -2.0, 2.0), (27.0, 8.6, 0.7), (31.0, 4.5, 1.6), (34.8, 12.0, 2.5)):
        crystal(L, "krystall", x, z, yw, reach=12.0)
    # Strendene: krystaller i bergveggen, et kaldt skjær over vannet.
    for x, yw in ((38.0, 0.4), (47.0, 1.3), (61.0, 2.2), (70.0, 0.9)):
        crystal(L, "nord", x, LAKE["z0"] + 0.9, yw, reach=8.0, y=sh)
    for x, yw in ((41.0, 0.6), (58.0, 1.5), (68.0, 2.4)):
        crystal(L, "sor", x, LAKE["z1"] - 0.9, yw, reach=8.0, y=sh)
    for x, z in ((40, 32), (64, 32), (40, 46), (64, 46)):
        L.light((x, LAKE["water"] + 5.0, z), B.COLD, 26.0, "sjoen", False, 6.0)
    for x in (36, 52, 68):
        L.light((x, sh + 3.0, LAKE["z0"] + 2.0), B.COLD, 16.0, "nord", False, 5.0)
        L.light((x, sh + 3.0, LAKE["z1"] - 2.0), B.COLD, 16.0, "sor", False, 5.0)
    L.light((HOLME["c"][0], LAKE["bottom"] + 1.5, HOLME["c"][1] + 9.0), (0.3, 0.5, 0.8), 12.0, "sjoen", False, 0.9)
    L.light((70.0, LAKE["bottom"] + 1.2, 50.0), (0.5, 0.8, 1.0), 6.0, "sjoen", True, 1.2)
    # Lysgrotta: krystaller rundt alteret.
    hc = HVILE["c"]
    for k in range(3):
        a = 2 * P * k / 3 + 1.2
        crystal(L, "hvile", hc[0] + 4.6 * math.cos(a), hc[1] + 4.6 * math.sin(a), a, reach=12.0)
    # Ekkogrotta: få og fiolette.
    for x, z, yw in ((20.5, 48.0, 0.2), (7.0, 57.0, 1.4), (8.0, 44.0, 2.3)):
        crystal(L, "ekko", x, z, yw, reach=12.0)
    L.light((12.0, -4 + 3.5, 52.0), (0.6, 0.4, 1.0), 20.0, "ekko", False, 7.0)
    # Dryppsteinshallen: resonanskrystallene gløder der spakene står, ellers kaldt.
    for x, z, yw in ((82.0, 22.0, 0.3), (110.0, 40.0, 1.2), (86.0, 52.0, 2.1), (102.0, 54.0, 0.8)):
        crystal(L, "drypp", x, z, yw, reach=12.0)
    L.light((96.0, -5 + 6.0, 36.0), B.COLD, 30.0, "drypp", False, 9.0)
    L.light((88.0, -5 + 5.0, 26.0), B.COLD, 20.0, "drypp", False, 5.0)
    L.light((102.0, -5 + 5.0, 48.0), B.COLD, 20.0, "drypp", False, 5.0)
    for x, z in ((90.0, 22.0), (109.0, 28.0), (88.0, 50.0)):
        L.light((x, -5 + 1.6, z), (0.5, 0.9, 1.0), 6.0, "drypp", True, 1.2)
    # Klangkammeret.
    kc = KLANGC
    for k in range(4):
        a = 2 * P * k / 4 + 0.8
        crystal(L, "klang", kc[0] + 3.6 * math.cos(a), kc[1] + 3.6 * math.sin(a), a, reach=12.0)
    L.light((kc[0], -5 + 3.0, kc[1]), (0.6, 0.8, 1.0), 12.0, "klang", False, 2.4)
    # Kløfta.
    for x, z, yw in ((89.0, 68.0, 0.5), (99.0, 72.0, 1.7), (88.5, 79.0, 2.6)):
        crystal(L, "kloft", x, z, yw, reach=12.0)
    # Den skjulte kilden: en liten kulp (vannflate) og krystaller.
    ki = rm["kilde"]
    K.box(mb, (76.0, ki.y + 0.01, 62.0), (82.0, ki.y + 0.02, 65.0), "water", faces=("+y",))
    for x, z, yw in ((68.5, 61.0, 0.4), (84.5, 65.0, 1.3), (70.0, 69.0, 2.2)):
        crystal(L, "kilde", x, z, yw, reach=12.0)
    L.light((76.0, ki.y + 2.5, 65.0), B.BLUE, 14.0, "kilde", False, 5.0)
    L.light((70.0, CAVE + 2.4, 50.0), B.COLD, 5.0, "kildesjakt", False, 0.6)
    # Vindeltrappa: fakler står i ROUND; krystallene på søyla lyser.
    # Forkammeret: den gamle vinsjen fra gruva (vogn og tau), fakler.
    fk = rm["forkammer"]
    L.prop("kit_cart", 63.5, fk.y, 92.0, 0.3)
    L.box((63.5, fk.y + 0.6, 92.0), (1.0, 1.2, 1.6), 0.3)
    fk.obstacles.append(Point(63.5, 92.0).buffer(0.9))
    L.prop("kit_lamp", 57.0, fk.y + 2.5, 105.0, 0.0)
    L.light((57.0, fk.y + 2.2, 105.0), (1.0, 0.62, 0.3), 8.0, "forkammer", True, 1.5)
    L.prop("kit_lamp", 64.0, fk.y + 2.5, 100.0, 0.0)
    L.light((64.0, fk.y + 2.2, 100.0), (1.0, 0.62, 0.3), 8.0, "forkammer", True, 1.5)
    L.light((58.0, fk.y + 4.0, 98.0), (0.6, 0.7, 0.9), 16.0, "forkammer", False, 5.0)
    # Grottekammeret: krystaller langs berget, det blå skjæret fra nedgangen.
    for x, z, yw in ((12.0, 86.0, 0.4), (44.0, 90.0, 1.3), (38.0, 112.0, 2.2), (14.0, 110.0, 0.9), (24.0, 118.0, 1.7), (8.0, 98.0, 2.6)):
        crystal(L, "dropdown", x, z, yw, reach=12.0)
    L.light((26.0, CAVE + 8.0, 100.0), B.COLD, 36.0, "dropdown", False, 12.0)
    L.light((14.0, CAVE + 5.0, 108.0), B.COLD, 20.0, "dropdown", False, 6.0)
    L.light((38.0, CAVE + 5.0, 90.0), B.COLD, 20.0, "dropdown", False, 6.0)
    # Nedgangen.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 3.0)
    L.wall_torch("nedgang", "n", -6.0, 2.4, intensity=1.4)
    # Gangene: et par fakler i de lange, krystaller i sprekkene.
    L.torch("sor-drypp", (78.5 + 0.02, -4.0 + 1.9, 50.0), (1.0, 0.0), intensity=1.0, reach=12.0)
    L.torch("drypp-kloft", (91.5 + 0.02, -7.0 + 1.9, 61.0), (1.0, 0.0), intensity=1.0, reach=12.0)
    L.light((93.0, -9.5 + 2.0, 92.0), B.BLUE, 7.0, "kloft-vindel", False, 0.8)
    L.light((80.0, CAVE + 1.8, 80.0), B.BLUE, 7.0, "kilde-vindel", False, 0.8)
    L.light((70.0, CAVE + 1.8, 56.0), B.BLUE, 6.0, "kildesjakt-kilde", False, 0.7)
    for z in (80.0,):
        L.torch("forkammer-hvile", (61.1 - 0.02, CAVE + 4.0 + 1.9, z), (-1.0, 0.0), intensity=1.0, reach=12.0)
    L.light((46.0, -7.0 + 1.9, 74.0), B.BLUE, 7.0, "forkammer-hvile", False, 0.8)
    L.torch("vindel-forkammer", (70.0, CAVE + 1.9, 97.5 - 0.02), (0.0, -1.0), intensity=1.0, reach=12.0)
    L.light((30.0, sh - 0.5 + 1.9, 60.0), B.COLD, 6.0, "sor-hvile", False, 0.6)


LAYOUT = dict(
    theme="grotte",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round=ROUND,
    virtual={"nord": "sjoen", "sor": "sjoen", "holme": "sjoen"},
    override={
        "landing": {"shape": ("rect", 0, 0, 8, 8), "y": 0, "h": 4.0},
        "krystall": {"shape": ("poly", KRYSTALL), "y": -1.5, "h": 5.0},
        "sjoen": {"shape": ("rect", (LAKE["x0"] + LAKE["x1"]) / 2, (LAKE["z0"] + LAKE["z1"]) / 2, LAKE["x1"] - LAKE["x0"], LAKE["z1"] - LAKE["z0"]), "y": LAKE["bottom"], "h": 17.0},
        "nord": {"shape": ("rect", (LAKE["x0"] + LAKE["x1"]) / 2, LAKE["z0"] + 2.0, LAKE["x1"] - LAKE["x0"], 4.0), "y": LAKE["shore"], "h": 11.0},
        "sor": {"shape": ("rect", (LAKE["x0"] + LAKE["x1"]) / 2, LAKE["z1"] - 2.0, LAKE["x1"] - LAKE["x0"], 4.0), "y": LAKE["shore"], "h": 11.0},
        "holme": {"shape": ("circle", HOLME["c"][0], HOLME["c"][1], HOLME["r"]), "y": LAKE["shore"], "h": 11.0},
        "hvile": {"shape": ("circle", HVILE["c"][0], HVILE["c"][1], HVILE["r"]), "y": -4, "h": 6.0},
        "ekko": {"shape": ("poly", EKKO), "y": -4, "h": 7.0},
        "drypp": {"shape": ("poly", DRYPP), "y": -5, "h": 12.0},
        "klang": {"shape": ("circle", KLANGC[0], KLANGC[1], 5), "y": -5, "h": 6.0},
        "kloft": {"shape": ("poly", KLOFT), "y": -9, "h": 16.0},
        "kilde": {"shape": ("rect", 76, 65, 20, 10), "y": CAVE, "h": 3.5},
        "vindel": {"shape": ("circle", VINDEL["c"][0], VINDEL["c"][1], VINDEL["r"]), "y": CAVE, "h": 14.0},
        "forkammer": {"shape": ("poly", FORKAMMER), "y": CAVE, "h": 6.0},
        "dropdown": {"shape": ("poly", DROPDOWN), "y": CAVE, "h": 14.0},
        "nedgang": {"shape": ("rect", -4, 104, 8, 8), "y": CAVE, "h": 5.0},
    },
    wall_mats={r: "rock" for r in ("krystall", "sjoen", "hvile", "ekko", "drypp", "klang", "kloft", "kildesjakt", "kilde", "vindel", "forkammer", "dropdown")},
    vault_mats={r: "rock" for r in ("krystall", "sjoen", "hvile", "ekko", "drypp", "klang", "kloft", "kildesjakt", "kilde", "vindel", "forkammer", "dropdown")},
    cells={"sjoen": 2.0, "kilde": 1.5},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
    round_seam_free=True,
)
