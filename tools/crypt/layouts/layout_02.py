"""Utlegget for nivå 2, Gravkongens hall (planen: `levels/level_02.py`).

Det planen ikke sier, står her: hvor gangene går (rette strekk på
rutenettet, trapper i dem), rommene som ikke er tegnet (avgrunnen under broa,
sjakta under fallemmen, kammeret under stigen), hva som står i rommene og
merkene. Byggeklossene er felles (`build_level.py`): balkongene
(`terrace`), trappa rundt gravkista (`core_stair`), broene (`stone_bridge`),
tårnet med galleri (`round_room`), sekskanten (`poly_room`) og avgrunnen
(et rom med `floor_y` langt under gulvet og broa som mekanisme).
"""

import math

import build_level as B
import kit_grav as K
from shapely.geometry import Point
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi


def ngon(cx, cz, r, n, rot=0.0):
    return [(round(cx + r * math.cos(rot + 2 * math.pi * i / n), 2), round(cz + r * math.sin(rot + 2 * math.pi * i / n), 2)) for i in range(n)]


# Balsamkammeret: sekskanten dreid så en kant vender mot nisjegangen (gangen
# ender nøyaktig i hjørnenes x, ellers står det en glipe mellom gangen og muren).
HEX = ngon(-46, 54, 7, 6, rot=math.pi / 6)
# Kongesalen: balkongene langs vest og øst, kista midt i salen med trappa
# rundt, og broene ut til balkongene.
SAL = dict(c=(0.0, 54.0), core=2.6, out=4.6, top=2.5, a0=math.radians(200), a1=math.radians(340), steps=23, bridge_w=3.0)
TERRACE_X = 9.0  # fronten på balkongene (|x|)
# Hullene i gulvene: fallemmen i nisjegangen, stigen opp i kongesalen.
FALLEM = sbox(-28.6, 54.4, -27.4, 55.6)
STIGE = sbox(-6.6, 60.4, -5.4, 61.6)
# Avgrunnen: broa fra bruhallen til forhallen, bunnen langt nede.
CHASM = dict(z0=81.5, z1=89.5, bottom=-11.0, deck=-2.0, width=3.6, hang=6.0)
# Trappa ned står i nedgangen, mot sør.
DOWN = (0.0, -5.0, 136.6)

EXTRA = [
    ("avgrunn", "Avgrunnen", ("rect", 0, (CHASM["z0"] + CHASM["z1"]) / 2, 14, CHASM["z1"] - CHASM["z0"]), -2, 9, False),
    ("fallbunn", "Under fallemmen", ("rect", -28, 54, 4, 4), -6, 3.0, False),
    ("stigebunn", "Under stigen", ("rect", -6, 62, 3, 4), -6, 3.0, False),
]

HALLS = [
    ("trappefot", "soylegang", STAIRS, [(0, 3.5), (0, 9)], 3.0, None, None),
    ("soylegang", "likkapell", DOOR, [(4.5, 30), (15.5, 30)], 2.5, None, None),
    ("soylegang", "kongegrav", SECRET, [(-4.5, 27), (-14, 27)], 2.0, (0, 2.0, 8.0), "hemmelig_2a"),
    ("soylegang", "kongesal", STAIRS, [(0, 39), (0, 44)], 4.0, None, None),
    ("kongesal", "nisjegang", DOOR, [(-14, 54), (-18.5, 54)], 2.5, None, None, {"ya": SAL["top"]}),
    ("nisjegang", "balsam", DOOR, [(-37.5, 54), (HEX[0][0], 54)], 2.5, None, None),
    ("kongesal", "spydgang", DOOR, [(14, 50), (19, 50)], 2.5, None, None, {"ya": SAL["top"]}),
    ("spydgang", "gravtarn", STAIRS, [(33, 48.5), (33, 38), (46, 38), (46, 41.4)], 2.5, (1, 2.0, 11.0), None),
    # Gangen bak den falske veggen begynner bak den runde muren, ikke 0,4 m inne i rommet, så ingen ramme røper den (eieren 18. oktober); `secret_reveal` lukker buen mellom muren og gangen.
    ("gravtarn", "tarnnisje", SECRET, [(55.01, 50), (58, 50), (58, 40)], 2.0, None, "hemmelig_2c", {"ya": 6.0}),
    ("kongesal", "bruhall", HALL, [(0, 64), (0, 72)], 4.0, None, None),
    ("bruhall", "avgrunn", HALL, [(0, 80), (0, CHASM["z0"])], 4.0, None, None, {"name": "Brokanten"}),
    ("avgrunn", "forhall", STAIRS, [(0, CHASM["z1"]), (0, 92)], 4.0, None, None),
    ("forhall", "arena", GATE, [(0, 100), (0, 106.4)], 4.0, (0, 2.6, 6.4), "arenaport", {"name": "Steinporten"}),
    ("arena", "nedgang", GATE, [(0, 129.6), (0, 135)], 3.0, (0, 2.2, 5.4), "arenaport_ut", {"name": "Porten ut"}),
    ("forhall", "likkapell", SHORTCUT, [(7, 96), (22, 96), (22, 34)], 2.2, (1, 20.0, 30.0), "snarvei_2a"),
    ("fallbunn", "kongegrav", SHORTCUT, [(-28, 52), (-28, 40), (-18, 40), (-18, 28.5)], 2.0, (0, 3.0, 8.0), "snarvei_2b"),
    ("fallbunn", "kjeller", HALL, [(-28, 56), (-28, 67)], 2.0, None, None),
    ("kjeller", "stigebunn", HALL, [(-22, 69), (-6, 69), (-6, 64)], 2.0, None, None),
    ("kjeller", "kronike", SECRET, [(-38, 74), (-42, 74), (-42, 79), (-46.5, 79)], 2.0, None, "hemmelig_2b"),
]

ROUND = {
    "gravtarn": dict(
        gallery=dict(y=6.0, ring_in=6.0, stair_in=6.2, stair_a0=math.radians(110), stair_a1=math.radians(200), steps=35, segs=24,
                     id="tarngalleri", name="Tårngalleriet", stair_id="tarntrapp", stair_name="Vindeltrappa i tårnet"),
        wall_cell=1.5, dome_seg=32, dome_rings=6,
        torches=[(300, 2.3, 1.8, 9.0, False), (20, 2.3, 1.6, 9.0, False), (240, 2.3, 1.4, 8.0, False),
                 (60, 1.9, 1.6, 8.0, True), (150, 1.9, 1.4, 8.0, True), (330, 1.9, 1.4, 8.0, True)],
    ),
    "arena": dict(
        dome_rise=4.5, ribs=16, wall_cell=1.5, dome_seg=40, dome_rings=6,
        torches=[(250, 2.6, 1.6, 10.0, False), (290, 2.6, 1.6, 10.0, False), (20, 2.6, 1.2, 9.0, False), (160, 2.6, 1.2, 9.0, False)],
    ),
}


def marks(_):
    """Merkene (format.md), plassert etter planen; noen er flyttet så de
    står fritt (bakholdet i kongesalen står ved siden av kista, Modalen
    foran steinkista i arenaen)."""
    gx, gz = 46.0, 50.0
    return [
        {"kind": "start", "pos": [0, 0, 1.5], "yaw": P},
        {"kind": "up", "pos": [0, 0, -2.5], "yaw": 0.0},
        {"kind": "rest", "pos": [20.0, -1, 26.9], "yaw": 0.0},
        # Søylegangen.
        {"kind": "mob", "pos": [0, -1, 15], "label": "avatar", "n": 2},
        {"kind": "mob", "pos": [1.2, -1, 30], "label": "breadcrumb", "patrol": [[-1.2, -1, 12], [-1.2, -1, 36], [1.2, -1, 36], [1.2, -1, 12]]},
        {"kind": "secret_wall", "id": "hemmelig_2a", "pos": [-4.5, -1, 27], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [-4.48, -1, 28.75], "yaw": P / 2, "for": "hemmelig_2a"},
        {"kind": "hint", "label": "draft", "pos": [-3.6, -1, 27], "for": "hemmelig_2a"},
        # Gravkongens grav.
        {"kind": "rare", "pos": [-19.0, -4, 20.6], "yaw": 0.0, "label": "Divider"},
        {"kind": "chest", "pos": [-14.9, -4, 24.0], "yaw": -P / 2},
        {"kind": "shortcut", "id": "snarvei_2b", "pos": [-28, -6, 52], "yaw": 0.0, "width": 1.6, "from": "fallbunn"},
        # Kongesalen.
        {"kind": "mob", "pos": [6.5, -2, 58.5], "label": "accordion i bakhold", "n": 3, "ambush": 4.5},
        {"kind": "mob", "pos": [-11.5, 2.5, 50], "label": "tooltip", "n": 2, "patrol": [[-11.5, 2.5, 46.5], [-11.5, 2.5, 61.5]]},
        {"kind": "mob", "pos": [6.5, -2, 47.5], "label": "button", "n": 2},
        # Vestfløyen: nisjegangen, fallemmen og balsamkammeret.
        {"kind": "mob", "pos": [-23.5, 2.5, 53.4], "label": "avatar", "n": 2},
        {"kind": "trapdoor", "id": "fallem_2", "pos": [-28, 2.5, 55.0], "size": [1.2, 1.2], "to": "fallbunn"},
        {"kind": "lever", "id": "spak_2a", "pos": [-51.4, 2.5, 54.0], "yaw": P / 2, "opens": ["bro_1"]},
        {"kind": "mob", "pos": [-45, 2.5, 50.5], "label": "button", "n": 3},
        {"kind": "chest", "pos": [-46.0, 2.5, 58.8], "yaw": P},
        # Kjelleren under vestfløyen.
        {"kind": "mob", "pos": [-31, -6, 70], "label": "breadcrumb", "n": 2},
        {"kind": "mob", "pos": [-27, -6, 73], "label": "avatar", "n": 2},
        {"kind": "chest", "pos": [-24.5, -6, 75.9], "yaw": P},
        {"kind": "trapdoor", "id": "fallem_2b", "label": "bolted", "pos": [-6, -2, 61.0], "size": [1.2, 1.2], "to": "stigebunn"},
        {"kind": "ladder", "pos": [-6, -6, 60.46], "yaw": 0.0, "top": [-6, -2, 62.25]},
        {"kind": "secret_wall", "id": "hemmelig_2b", "pos": [-38, -6, 74], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "sound", "pos": [-37.2, -6, 72], "for": "hemmelig_2b"},
        {"kind": "lore", "pos": [-51.5, -6, 77.0], "yaw": P / 2, "label": "Gravkongens krønike",
         "text": "Gravkongen ble lagt i den sprukne kista under søylegangen, med seglet sitt i hendene. Den store "
                 "kista i salen er tom; den ble murt for å lure gravrøverne. Det som ligger i gravkammeret nå, kom ned "
                 "med oss. Det lukker seg om den som går inn, og slipper ikke før det selv vil."},
        {"kind": "bonus", "pos": [-49.0, -6, 80.9], "yaw": P, "label": "liv"},
        # Østfløyen: spydgangen og gravtårnet.
        {"kind": "trap", "label": "arrows", "id": "piler_2", "pos": [27, 2.5, 50], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "mob", "pos": [gx + 4, -1, gz], "label": "accordion", "n": 2},
        {"kind": "mob", "pos": [gx - 4.5, -1, gz], "label": "tooltip", "n": 2, "patrol": [[gx - 4.5, -1, gz], [gx, -1, gz + 4.5], [gx + 4.5, -1, gz], [gx, -1, gz - 4.5]]},
        {"kind": "lever", "id": "spak_2b", "pos": [gx, 6.0, gz + 8.5], "yaw": P, "opens": ["bro_1"]},
        {"kind": "secret_wall", "id": "hemmelig_2c", "pos": [55.0, 6.0, 50.0], "yaw": -P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [54.95, 6.0, 51.9], "yaw": -P / 2, "for": "hemmelig_2c"},
        {"kind": "bonus", "pos": [59.3, 6.0, 37.0], "yaw": P, "label": "utholdenhet"},
        # Bruhallen, broa og forhallen.
        {"kind": "mob", "pos": [0, -2, 76], "label": "avatar", "n": 2},
        {"kind": "bridge", "id": "bro_1", "pos": [0, CHASM["deck"], (CHASM["z0"] + CHASM["z1"]) / 2], "yaw": 0.0,
         "size": [CHASM["width"], CHASM["z1"] - CHASM["z0"]], "height": CHASM["hang"], "top": [0, 7.4, (CHASM["z0"] + CHASM["z1"]) / 2]},
        {"kind": "chest", "pos": [5.2, -3, 97.6], "yaw": -P / 2},
        {"kind": "shortcut", "id": "snarvei_2a", "pos": [7, -3, 96], "yaw": -P / 2, "width": 1.6, "from": "forhall"},
        # Arenaen og nedgangen.
        {"kind": "gate", "id": "arenaport", "pos": [0, -3, 101.3], "yaw": 0.0, "width": 3.4, "arena": "arena"},
        {"kind": "boss", "pos": [0, -4, 112.6], "yaw": P, "label": "modal", "arena": "arena"},
        {"kind": "gate", "id": "arenaport_ut", "label": "exit", "pos": [0, -4, 130.9], "yaw": 0.0, "width": 2.6, "arena": "arena"},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def niche_prof(u, v0, w=1.7, h=0.5):
    """Omrisset av en gravnisje (som `K.niche` gir), til å hogge ut av muren."""
    prof = [(u - w / 2, v0), (u + w / 2, v0), (u + w / 2, v0 + h)]
    for k in range(1, 6):
        a = k / 6
        prof.append((u + w / 2 - w * a, v0 + h + 0.1 * math.sin(math.pi * a)))
    prof.append((u - w / 2, v0 + h))
    return prof


NICHE_X = range(-36, -19, 3)


def niche_spots(r):
    """Nisjene i nisjegangen: (side, u, v0) i begge langmurene."""
    for side in ("n", "s"):
        for j, xc in enumerate(NICHE_X):
            u = (xc - r.x0) if side == "n" else (r.x1 - xc)
            for lvl, v0 in enumerate((r.y + 0.35, r.y + 1.25)):
                yield side, j, lvl, u, v0


def pre(L):
    rm = L.rooms
    ng = rm["nisjegang"]
    ng.wall_holes = {"n": [], "s": []}
    for side, j, lvl, u, v0 in niche_spots(ng):
        ng.wall_holes[side].append(niche_prof(u, v0))
    rm["nisjegang"].holes.append(FALLEM)
    rm["nisjegang"].obstacles.append(FALLEM)
    rm["fallbunn"].flat = FALLEM
    rm["kongesal"].holes.append(STIGE)
    rm["kongesal"].obstacles.append(STIGE)
    rm["stigebunn"].flat = STIGE
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(-1.2, dz, 1.2, dz + 3.8))
    a = rm["avgrunn"]
    a.floor_y = CHASM["bottom"]
    a.floor_mat = "rubble" if "rubble" in K.MATS else "floor"
    a.nowalk = True
    rm["soylegang"].ribs = list(range(13, 38, 4))


def features(L):
    """Kongesalen: balkongene, kista med trappa rundt og broene, og broa
    over avgrunnen som mekanisme."""
    r = L.rooms["kongesal"]
    c, y1 = SAL["c"], SAL["top"]
    hw = SAL["bridge_w"] / 2
    gap = (c[1] - hw, c[1] + hw)
    rect_w, walk_w = L.terrace(r, r.x0, -TERRACE_X, r.z0, r.z1, y1, "e", gaps=[gap])
    rect_e, walk_e = L.terrace(r, TERRACE_X, r.x1, r.z0, r.z1, y1, "w", gaps=[gap])
    ga = math.asin(hw / SAL["out"])
    top, tris = L.core_stair(r, c, SAL["core"], SAL["out"], r.y, y1, SAL["a0"], SAL["a1"], SAL["steps"], gaps=[(-ga, ga), (P - ga, P + ga)])
    disc = Point(c).buffer(SAL["out"], 72)
    _, bw = L.stone_bridge((c[0] - SAL["out"] + 0.3, c[1]), (-TERRACE_X, c[1]), SAL["bridge_w"], y1, clip=disc)
    _, be = L.stone_bridge((c[0] + SAL["out"] - 0.3, c[1]), (TERRACE_X, c[1]), SAL["bridge_w"], y1, clip=disc)
    # Gulvet under broene er salens; flaten oppe er én.
    upper = unary_union([walk_w, walk_e, top, bw, be])
    L.decks.append(dict(room="kongesal", of="kongesal", y=y1, walk=upper, ring=unary_union([rect_w, rect_e, disc]).buffer(0.3), obstacles=[]))
    for tri in tris:
        L.tri_patches.append((tri, "kongesal"))
    # Gravkista oppå.
    K.sarcophagus(L.mb, c[0], c[1], y1, P / 2, L=2.6, W=1.2, H=0.95)
    L.box((c[0], y1 + 0.6, c[1]), (2.9, 1.2, 1.5))
    # Avgrunnen: broa er en mekanisme (`bridge`); her står bare det den
    # hviler på og navigasjonen over den (stengt til broa er nede).
    ch = L.rooms["avgrunn"]
    w = CHASM["width"]
    for zz in (CHASM["z0"], CHASM["z1"]):
        K.box(L.mb, (-w / 2 - 0.2, CHASM["deck"] - 0.6, zz - 0.25), (w / 2 + 0.2, CHASM["deck"] - 0.02, zz + 0.25), "dressed")
    walk = sbox(-w / 2 + B.INSET, CHASM["z0"] + B.INSET, w / 2 - B.INSET, CHASM["z1"] - B.INSET)
    L.decks.append(dict(room="avgrunn", y=CHASM["deck"], walk=walk, gate="bro_1", ring=sbox(-w / 2, CHASM["z0"], w / 2, CHASM["z1"]), obstacles=[]))
    K.rubble(L.mb, -3.5, 84.0, CHASM["bottom"], r=2.2, h=1.0, seed=11)
    K.rubble(L.mb, 3.0, 87.5, CHASM["bottom"], r=1.8, h=0.8, seed=12)


def dress(L):
    mb = L.mb
    rm = L.rooms
    rng = L.rng
    # Trappefoten: lysestaker ved trappa opp.
    t = rm["trappefot"]
    for x in (-2.8, 2.8):
        K.candlestick(mb, x, 2.4, t.y, arms=2)
        t.obstacles.append(Point(x, 2.4).buffer(0.35))
        L.box((x, t.y + 0.7, 2.4), (0.5, 1.4, 0.5))
        L.light((x, t.y + 1.55, 2.4), B.CANDLE, 5.0, "trappefot", True, 1.1)
    L.wall_torch("trappefot", "w", -1.0, 2.4)
    # Søylegangen: søyler, sarkofager imellom (ikke foran den falske veggen).
    so = rm["soylegang"]
    for zc in range(13, 38, 4):
        for xc in (-2.6, 2.6):
            K.pillar(mb, xc, zc, so.y, B.pillar_top(so, xc, zc, 0.7), size=0.7)
            so.obstacles.append(sbox(xc - 0.55, zc - 0.55, xc + 0.55, zc + 0.55))
            L.box((xc, so.y + so.h / 2, zc), (1.0, so.h, 1.0))
    for zc in (15, 19, 23, 31, 35):
        for xc in (-3.4, 3.4):
            if xc > 0 and abs(zc - 30) < 2.6:
                continue
            L.prop("kit_sarcophagus", xc, so.y, zc, 0.0)
            so.obstacles.append(sbox(xc - 0.6, zc - 1.25, xc + 0.6, zc + 1.25))
            L.box((xc, so.y + 0.5, zc), (1.1, 1.0, 2.4))
    for zc in (17, 29, 37):
        for xc, nx in ((-2.6 + 0.47, 1.0), (2.6 - 0.47, -1.0)):
            L.torch("soylegang", (xc, so.y + 2.6, zc), (nx, 0.0), intensity=1.8, reach=8.0)
    # Likkapellet: hvilestedet (motoren), kalde lys og en sarkofag.
    lk = rm["likkapell"]
    L.wall_torch("likkapell", "w", 33.2, 2.3, intensity=1.6)
    for x in (17.0,):
        K.candlestick(mb, x, 27.4, lk.y, arms=2)
        lk.obstacles.append(Point(x, 27.4).buffer(0.35))
        L.box((x, lk.y + 0.7, 27.4), (0.5, 1.4, 0.5))
        L.light((x, lk.y + 1.5, 27.4), B.CANDLE, 5.0, "likkapell", True, 1.1)
    L.prop("kit_sarcophagus", 22.8, lk.y, 32.0, P / 2)
    lk.obstacles.append(sbox(21.6, 31.4, 24.0, 32.6))
    L.box((22.8, lk.y + 0.5, 32.0), (2.4, 1.0, 1.1))
    # Gravkongens grav: lavt og kaldt, med kongens kiste.
    kg = rm["kongegrav"]
    L.prop("kit_sarcophagus", -18.0, kg.y, 24.0, 0.0)
    kg.obstacles.append(sbox(-18.6, 22.8, -17.4, 25.2))
    L.box((-18.0, kg.y + 0.5, 24.0), (1.1, 1.0, 2.4))
    L.light((-20.5, kg.y + 2.3, 26.5), B.COLD, 6.0, "kongegrav", True, 0.8)
    K.candlestick(mb, -20.6, 21.0, kg.y, arms=2)
    kg.obstacles.append(Point(-20.6, 21.0).buffer(0.35))
    L.box((-20.6, kg.y + 0.7, 21.0), (0.5, 1.4, 0.5))
    L.light((-20.6, kg.y + 1.5, 21.0), B.CANDLE, 5.0, "kongegrav", True, 1.0)
    # Kongesalen: fakler på balkongene og på søylene ved dørene, glør i
    # fat ved foten av trappa.
    ks = rm["kongesal"]
    for x, wall in ((-14, "w"), (14, "e")):
        for z in (47.0, 61.0):
            L.wall_torch("kongesal", wall, z, SAL["top"] - ks.y + 2.2, intensity=1.8, reach=10.0)
    for x, n in ((-TERRACE_X + 0.02, 1.0), (TERRACE_X - 0.02, -1.0)):
        for z in (47.5, 60.5):
            L.torch("kongesal", (x, ks.y + 2.4, z), (n, 0.0), intensity=1.6, reach=9.0)
    L.wall_torch("kongesal", "n", -4.0, 3.0, intensity=1.8, reach=10.0)
    L.wall_torch("kongesal", "n", 4.0, 3.0, intensity=1.8, reach=10.0)
    for x, z in ((-6.5, 49.0), (6.5, 63.0)):
        K.cylinder(mb, (x, z), 0.5, ks.y, ks.y + 0.75, "iron", n=10, caps=(True, False), smooth=False)
        K.cylinder(mb, (x, z), 0.45, ks.y + 0.75, ks.y + 0.8, "fire_torch", n=10, caps=(True, False), smooth=False)
        L.box((x, ks.y + 0.4, z), (1.0, 0.8, 1.0))
        ks.obstacles.append(Point(x, z).buffer(0.55))
        L.light((x, ks.y + 1.4, z), (1.0, 0.45, 0.18), 10.0, "kongesal", True, 2.2)
    # Kullpanner på balkongene ved broene.
    deck = next(d for d in L.decks if d["room"] == "kongesal")
    for x in (-TERRACE_X - 1.2, TERRACE_X + 1.2):
        for z in (51.0, 57.0):
            K.cylinder(mb, (x, z), 0.42, SAL["top"], SAL["top"] + 0.7, "iron", n=10, caps=(True, False), smooth=False)
            K.cylinder(mb, (x, z), 0.38, SAL["top"] + 0.7, SAL["top"] + 0.75, "fire_torch", n=10, caps=(True, False), smooth=False)
            L.box((x, SAL["top"] + 0.35, z), (0.9, 0.7, 0.9))
            deck["obstacles"].append(Point(x, z).buffer(0.45))
            L.light((x, SAL["top"] + 1.3, z), (1.0, 0.45, 0.18), 9.0, "kongesal", True, 1.8)
    L.shaft(STIGE, -3.0, ks.y)
    # Nisjegangen: gravnisjer i begge murene, sjakta under fallemmen.
    ng = rm["nisjegang"]
    for side, j, lvl, u, v0 in niche_spots(ng):
        surf = K.Plane((ng.x0, ng.z0), (1.0, 0.0), (0.0, 1.0)) if side == "n" else K.Plane((ng.x1, ng.z1), (-1.0, 0.0), (0.0, -1.0))
        K.niche(mb, surf, u, v0, w=1.7, h=0.5, depth=0.5, sealed=(j + lvl) % 3 == 1, rng=rng)
    L.light((-33.0, ng.y + 2.3, 52.4), B.CANDLE, 5.0, "nisjegang", True, 1.0)
    L.light((-22.0, ng.y + 2.3, 55.6), B.CANDLE, 5.0, "nisjegang", True, 1.0)
    L.shaft(FALLEM, -3.0, ng.y)
    fb = rm["fallbunn"]
    L.light((-29.2, fb.y + 2.2, 53.0), B.COLD, 5.0, "fallbunn", False, 0.5)
    K.rubble(mb, -26.9, 53.0, fb.y, r=0.7, h=0.3, seed=21)
    fb.obstacles.append(Point(-26.9, 53.0).buffer(0.7))
    # Balsamkammeret: balsameringsbordet midt i, spaken bak.
    bs = rm["balsam"]
    K.altar(mb, -46.0, 54.0, bs.y, P / 2, w=2.4, d=1.0, h=0.95)
    bs.obstacles.append(sbox(-46.65, 52.7, -45.35, 55.3))
    L.box((-46.0, bs.y + 0.5, 54.0), (1.2, 1.0, 2.6))
    for deg in (90, 270):
        a = math.radians(deg)
        L.torch("balsam", (-46 + 6.04 * math.cos(a), bs.y + 2.4, 54 + 6.04 * math.sin(a)), (-math.cos(a), -math.sin(a)), intensity=1.6, reach=8.0)
    # Kjelleren: lav og kald, kister i hyllene.
    kj = rm["kjeller"]
    for zc in (68.6, 70.8):
        for lvl in (0.0, 1.1):
            K.box(mb, (kj.x0, kj.y + lvl + 0.42, zc - 1.0), (kj.x0 + 0.75, kj.y + lvl + 0.5, zc + 1.0), "dressed")
            K.coffin(mb, kj.x0 + 0.38, zc, kj.y + lvl + 0.5, 0.0, L=1.8, W=0.6)
    kj.obstacles.append(sbox(kj.x0, 67.4, kj.x0 + 0.8, 72.0))
    L.box((kj.x0 + 0.4, kj.y + 1.0, 69.7), (0.8, 2.0, 4.6))
    L.light((-30.0, kj.y + 2.6, 68.0), B.COLD, 8.0, "kjeller", True, 0.8)
    L.light((-24.0, kj.y + 2.6, 75.0), B.CANDLE, 6.0, "kjeller", True, 0.9)
    st = rm["stigebunn"]
    L.light((-6.0, st.y + 2.2, 63.4), B.COLD, 5.0, "stigebunn", False, 0.5)
    kr = rm["kronike"]
    L.light((-50.0, kr.y + 2.0, 81.2), (1.0, 0.8, 0.45), 5.0, "kronike", True, 1.0)
    # Spydgangen: pilhullene i nordmuren over trykkplata.
    sg = rm["spydgang"]
    surf = K.Plane((sg.x0, sg.z0), (1.0, 0.0), (0.0, 1.0))
    K.arrow_slots(mb, surf, 27.0 - sg.x0, sg.y + 0.9)
    L.light((21.5, sg.y + 2.2, 51.3), B.COLD, 5.0, "spydgang", True, 0.7)
    L.light((32.5, sg.y + 2.2, 48.7), B.COLD, 5.0, "spydgang", True, 0.7)
    L.torch("spydgang-gravtarn", (34.23, 2.5 + 1.9, 44.0), (-1.0, 0.0), intensity=1.2, reach=7.0)
    # Tårnnisjen.
    tn = rm["tarnnisje"]
    L.light((56.6, tn.y + 1.9, 36.6), (1.0, 0.8, 0.45), 4.5, "tarnnisje", True, 0.8)
    # Bruhallen: kanten, kjettingfestene i taket ses fra salen.
    bh = rm["bruhall"]
    L.wall_torch("bruhall", "s", -6.0, 2.6, intensity=1.8, reach=9.0)
    L.wall_torch("bruhall", "s", 6.0, 2.6, intensity=1.8, reach=9.0)
    L.wall_torch("bruhall", "n", -6.0, 2.6, intensity=1.4, reach=8.0)
    # Avgrunnen: et kaldt skjær nedenfra.
    L.light((0.0, CHASM["bottom"] + 1.5, 85.5), (0.45, 0.35, 0.8), 16.0, "avgrunn", True, 2.2)
    L.light((0.0, CHASM["deck"] + 3.5, 85.5), B.COLD, 8.0, "avgrunn", True, 0.5)
    # Forhallen.
    L.wall_torch("forhall", "n", -4.0, 2.4, intensity=1.8)
    L.wall_torch("forhall", "s", -4.5, 2.4, intensity=1.6)
    L.wall_torch("forhall", "s", 4.5, 2.4, intensity=1.6)
    # Arenaen: den enorme steinkista midt i, lysestaker i kanten.
    ar = rm["arena"]
    K.sarcophagus(mb, 0.0, 118.5, ar.y, 0.0, L=4.0, W=1.8, H=1.2)
    ar.obstacles.append(sbox(-1.1, 116.3, 1.1, 120.7))
    L.box((0.0, ar.y + 0.75, 118.5), (2.2, 1.5, 4.4))
    for deg in (210, 330, 90, 30, 150):
        a = math.radians(deg)
        x, z = 9.8 * math.cos(a), 118 + 9.8 * math.sin(a)
        K.candlestick(mb, x, z, ar.y, arms=0)
        ar.obstacles.append(Point(x, z).buffer(0.35))
        L.box((x, ar.y + 0.7, z), (0.5, 1.4, 0.5))
        L.light((x, ar.y + 1.6, z), B.CANDLE, 7.0, "arena", True, 1.3)
    # Nedgangen: det blå lyset fra trappa ned.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(-1.6, dz, 1.6, nd.z1))
    L.light((0.0, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 3.0)
    L.wall_torch("nedgang", "w", 137.0, 2.4, intensity=1.4)
    # Snarveiene: en fakkel midtveis i den lange.
    L.torch("forhall-likkapell", (20.92, -3 + 1.9, 80.0), (1.0, 0.0), intensity=1.2, reach=8.0)
    L.torch("forhall-likkapell", (23.08, -1 + 1.9, 45.0), (-1.0, 0.0), intensity=1.0, reach=7.0)
    L.light((-28.0, -6 + 1.9, 50.0), B.COLD, 6.0, "fallbunn-kongegrav", True, 0.6)


LAYOUT = dict(
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round=ROUND,
    override={"balsam": {"shape": ("poly", HEX)}, "tarnnisje": {"h": 3.0},
              "gravtarn": {"h": 11.0}, "arena": {"h": 8.0}, "kongesal": {"h": 9.5}},
    cells={"avgrunn": 2.0, "kongesal": 1.5, "bruhall": 1.5, "soylegang": 1.5},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
    slab=("soylegang", "nisjegang", "kongegrav"),
    wall_mats={"kjeller": "wall_dark", "kronike": "wall_dark", "fallbunn": "wall_dark", "stigebunn": "wall_dark", "kongegrav": "wall_dark"},
)
