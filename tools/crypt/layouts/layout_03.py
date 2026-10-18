"""Utlegget for nivå 3, Kloakken (planen: `levels/level_03.py`).

Vannstanden er det nye: hovedkanalen og rotunden står fulle av vann til
sluseratten i slusekammeret er dreid (spaken åpner bryteren `vann_1`). Da synker
vannet (`water`-merkene), og bunnen av kanalen og rotunden, vindeltrappa ned i
rotunden, stikkrenna til silkammeret og gangen til rottereiret åpner seg (for
fiendene er navigasjonen der stengt av `vann_1`). Det er vannet selv som holder
deg ute: du vader ikke dypere enn `crypt::WADE`, og faller du i, drar du deg
opp der du sist sto tørt (eieren 18. oktober: ingen usynlige murer).

Gangbanene i kanalen er hyller på konsoller langs murene (`terrace(..., solid=False)`),
så gangene ut fra kanalbunnen går inn under dem. Bunnen i kanalen og rotunden
ligger 3,4 og 3,6 m under gangbanene, så det er takhøyde under hyllene og
galleriet.
"""

import math

import build_level as B
import kit_grav as K
from shapely.geometry import Point
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
GREEN = (0.55, 0.9, 0.5)
VANN = "vann_1"

# Kanalen: bunnen, gangbanene (hyller langs nord- og sørmuren) og bruene.
KANAL = dict(x0=-4.0, x1=80.0, z0=19.0, z1=29.0, bottom=-3.4, walk=2.5, bridges=(18.0, 58.0), bridge_w=3.0)
ROT = dict(c=(78.0, 54.0), r=9.0, bottom=-3.6, a0=math.radians(200), a1=math.radians(290))
# Hullene i gulvene: sluket i rotunden, stigen opp i utløpet.
SLUK = sbox(77.2, 57.7, 78.8, 59.3)
STIGE = sbox(34.4, 79.4, 35.6, 80.6)
DOWN = (30.0, -6.0, 96.6)


def ngon(cx, cz, r, n, rot=0.0):
    return [(round(cx + r * math.cos(rot + 2 * math.pi * i / n), 3), round(cz + r * math.sin(rot + 2 * math.pi * i / n), 3)) for i in range(n)]


# Silkammeret: åttekanten (gangene ender nøyaktig i hjørnenes koordinater).
SIL = ngon(34, 58, 8, 8, math.pi / 8)
SIL_E = SIL[0][0]  # østkanten
SIL_N = SIL[5][1]
SIL_S = SIL[1][1]

EXTRA = [
    ("rorbunn", "Bunnen av sluket", ("rect", 78, 58.5, 4, 4), -8, 3.0, True),
    ("stigerom", "Under utløpet", ("rect", 35, 80, 3, 3), -8.6, 2.9, True),
]

HALLS = [
    ("trappefot", "vokterstue", DOOR, [(-4, 1.0), (-11.5, 1.0)], 2.5, None, None),
    ("trappefot", "kanal", HALL, [(0, 3.5), (0, 19)], 3.0, None, None, {"yb": 0.0}),
    ("kanal", "mosegrotta", STAIRS, [(26, 19), (26, 12)], 2.5, None, None, {"ya": 0.0}),
    ("kanal", "fossen", STAIRS, [(62, 19), (62, 12)], 2.5, None, None, {"ya": 0.0}),
    ("fossen", "basseng", SECRET, [(62, 0), (62, -7.5)], 2.0, None, "hemmelig_3a"),
    ("kanal", "sluse", DOOR, [(80, 20.25), (86, 20.25)], 2.5, None, None, {"ya": 0.0}),
    ("sluse", "rotunden", HALL, [(92, 32), (92, 54), (86.6, 54)], 2.5, None, None, {"yb": 0.0}),
    ("kanal", "kapell", SECRET, [(52, 29), (52, 38)], 2.0, (0, 3.0, 7.0), "hemmelig_3b"),
    ("kanal", "rottereir", HALL, [(4, 29), (4, 37)], 2.5, None, None, {"water": VANN}),
    ("kanal", "silkammer", HALL, [(34, 29), (34, SIL_N)], 2.5, (0, 14.0, 18.0), None, {"water": VANN, "name": "Stikkrenna"}),
    ("rotunden", "silkammer", HALL, [(69.4, 54), (50, 54), (50, 58), (SIL_E, 58)], 2.0, (0, 6.0, 10.0), None, {"water": VANN}),
    ("rorbunn", "underkum", HALL, [(78, 60.5), (78, 78), (68.5, 78)], 2.0, None, None),
    ("underkum", "stigerom", HALL, [(59.5, 80), (36.5, 80)], 2.0, (0, 10.0, 14.0), None),
    ("silkammer", "utlop", STAIRS, [(34, SIL_S), (34, 77)], 3.0, None, None),
    ("utlop", "nedgang", GATE, [(30, 87), (30, 95)], 3.0, (0, 3.0, 7.0), "port_3"),
    ("utlop", "vokterstue", SHORTCUT, [(23, 82), (-24, 82), (-24, 2), (-20.5, 2)], 2.0, (1, 30.0, 52.0), "snarvei_3"),
]

ROUND = {
    "rotunden": dict(
        gallery=dict(y=0.0, ring_in=7.0, stair_in=7.2, stair_a0=ROT["a0"], stair_a1=ROT["a1"], steps=18, segs=24,
                     id="ringgang", name="Ringgangen", stair_id="rotundetrapp", stair_name="Trappa ned i rotunden", stair_gate=VANN),
        wall_cell=1.5, dome_seg=32, dome_rings=6, wall_mat="wall_dark",
        torches=[(30, 1.9, 1.4, 8.0, True), (120, 1.9, 1.4, 8.0, True), (240, 1.9, 1.2, 8.0, True)],
    ),
}


def marks(_):
    kz = (KANAL["z1"] + KANAL["z0"]) / 2
    nz, sz = KANAL["z0"] + KANAL["walk"] / 2, KANAL["z1"] - KANAL["walk"] / 2
    cx, cz = ROT["c"]
    return [
        {"kind": "start", "pos": [0, 0, 1.0], "yaw": P},
        {"kind": "up", "pos": [0, 0, -2.5], "yaw": 0.0},
        {"kind": "rest", "pos": [-18.5, 0, -0.6], "yaw": 0.0},
        # Kanalen.
        {"kind": "mob", "pos": [12, 0, nz], "label": "input", "n": 2, "patrol": [[6, 0, nz], [74, 0, nz]]},
        {"kind": "mob", "pos": [22, 0, sz], "label": "toast", "n": 3},
        {"kind": "mob", "pos": [46, 0, sz], "label": "button", "n": 2},
        {"kind": "water", "id": VANN, "pos": [38, -0.5, kz], "size": [84, 10], "levels": [-0.5, KANAL["bottom"] - 0.05]},
        {"kind": "hint", "label": "draft", "pos": [52, KANAL["bottom"], 28.2], "for": "hemmelig_3b"},
        {"kind": "secret_wall", "id": "hemmelig_3b", "pos": [52, KANAL["bottom"], 29.0], "yaw": P, "width": 2.0, "height": 2.6},
        # Mosegrotta.
        {"kind": "mob", "pos": [26, 0.5, 5], "label": "tooltip", "n": 2},
        {"kind": "chest", "pos": [30.5, 0.5, 9.2], "yaw": P},
        # Fossen og bassenget bak.
        {"kind": "mob", "pos": [60, 2, 8.5], "label": "input", "n": 2},
        {"kind": "secret_wall", "id": "hemmelig_3a", "pos": [62, 2, 0.0], "yaw": 0.0, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [63.8, 2, 0.02], "yaw": 0.0, "for": "hemmelig_3a"},
        {"kind": "rare", "pos": [62, 2, -12.8], "yaw": 0.0, "label": "Search"},
        # Slusekammeret: sluseratten.
        {"kind": "lever", "id": "sluseratt", "pos": [97.4, 0, 22.0], "yaw": -P / 2, "opens": [VANN]},
        {"kind": "mob", "pos": [90, 0, 28], "label": "select", "n": 2},
        {"kind": "chest", "pos": [96.6, 0, 30.6], "yaw": -P / 2},
        {"kind": "trap", "label": "flood", "id": "flodluke", "pos": [92, 0, 40], "size": [1.2, 1.2], "dir": [1, 0, 0]},
        # Rotunden.
        {"kind": "mob", "pos": [cx - 7.9, 0, cz - 1.5], "label": "toast", "n": 3},
        {"kind": "mob", "pos": [cx + 8.0, 0, cz + 0.3], "label": "select", "patrol": [[cx, 0, cz - 8], [cx + 8, 0, cz], [cx, 0, cz + 8], [cx - 8, 0, cz]]},
        {"kind": "water", "id": VANN, "pos": [cx, -0.5, cz], "size": [2 * ROT["r"], 2 * ROT["r"]], "levels": [-0.5, ROT["bottom"] - 0.05]},
        {"kind": "trapdoor", "id": "sluket", "label": "shaft", "pos": [78, ROT["bottom"], 58.5], "size": [1.6, 1.6], "to": "rorbunn"},
        {"kind": "hint", "label": "draft", "pos": [78, ROT["bottom"], 58.5], "for": "sluket"},
        # Underkummen og stigen opp i utløpet.
        {"kind": "bonus", "pos": [62.0, -8, 76.0], "yaw": 0.0, "label": "utholdenhet"},
        {"kind": "chest", "pos": [66.9, -8, 80.4], "yaw": P},
        {"kind": "trapdoor", "id": "fallem_3", "label": "bolted", "pos": [35, -5, 80.0], "size": [1.2, 1.2], "to": "stigerom"},
        {"kind": "ladder", "pos": [35, -8.6, 79.46], "yaw": 0.0, "top": [35, -5, 81.25]},
        # Det druknede kapellet.
        {"kind": "lore", "pos": [49.0, -4, 39.3], "yaw": 0.0, "label": "Slusevokterens dagbok",
         "text": "Skriverne kastet lappene sine i sluket, natt etter natt: omvei på omvei, regel over regel, til ingen "
                 "visste hva som var boka og hva som var lapper. Alt rant hit ned. Nå stiger vannet hver vår, og slusa "
                 "er det eneste som holder kapellet tørt. Den store Selecten i silkammeret har tatt nøkkelen til "
                 "porten ned. Den vil bare at jeg skal velge. Jeg tør ikke gå dit."},
        {"kind": "bonus", "pos": [54.6, -4, 42.6], "yaw": P, "label": "liv"},
        # Rottereiret.
        {"kind": "mob", "pos": [4, KANAL["bottom"], 44], "label": "toast i bakhold", "n": 4, "ambush": "rottereir"},
        {"kind": "chest", "pos": [9.5, KANAL["bottom"], 46.0], "yaw": -P / 2},
        # Silkammeret: eliten med nøkkelen.
        {"kind": "trap", "label": "gas", "id": "giftgass", "pos": [34, KANAL["bottom"], 40], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "elite", "pos": [34, -4, 59.5], "yaw": 0.0, "label": "select", "opens": ["port_3"]},
        {"kind": "mob", "pos": [30, -4, 54], "label": "input", "n": 2},
        # Utløpet og porten ned.
        {"kind": "mob", "pos": [27, -5, 83], "label": "toast", "n": 2},
        {"kind": "mob", "pos": [26.5, -5, 79.0], "label": "button", "n": 2},
        {"kind": "gate", "id": "port_3", "pos": [30, -5, 88.5], "yaw": 0.0, "width": 2.6},
        {"kind": "shortcut", "id": "snarvei_3", "pos": [23, -5, 82], "yaw": P / 2, "width": 1.6, "from": "utlop"},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    k = rm["kanal"]
    k.gate = VANN  # bunnen er under vann
    k.floor_mat = "slab"
    rt = rm["rotunden"]
    rt.gate = VANN
    rt.holes.append(SLUK)
    rt.obstacles.append(SLUK)
    rm["rorbunn"].flat = SLUK
    rm["utlop"].holes.append(STIGE)
    rm["utlop"].obstacles.append(STIGE)
    rm["stigerom"].flat = STIGE
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))


def features(L):
    """Gangbanene langs kanalen og bruene over."""
    r = L.rooms["kanal"]
    w, y = KANAL["walk"], 0.0
    hw = KANAL["bridge_w"] / 2
    gaps = [(bx - hw, bx + hw) for bx in KANAL["bridges"]]
    rn, wn = L.terrace(r, r.x0, r.x1, r.z0, r.z0 + w, y, "s", gaps=gaps, solid=False)
    rs, ws = L.terrace(r, r.x0, r.x1, r.z1 - w, r.z1, y, "n", gaps=gaps, solid=False)
    walks = [wn, ws]
    for bx in KANAL["bridges"]:
        _, bw = L.stone_bridge((bx, r.z0 + w), (bx, r.z1 - w), KANAL["bridge_w"], y)
        walks.append(bw)
    L.decks.append(dict(room="kanal", of="kanal", y=y, walk=unary_union(walks), ring=unary_union([rn, rs]).buffer(0.2), obstacles=[]))


def dress(L):
    mb = L.mb
    rm = L.rooms
    # Trappefoten.
    t = rm["trappefot"]
    for x in (-2.8, 2.8):
        K.candlestick(mb, x, 2.4, t.y, arms=2)
        t.obstacles.append(Point(x, 2.4).buffer(0.35))
        L.box((x, t.y + 0.7, 2.4), (0.5, 1.4, 0.5))
        L.light((x, t.y + 1.55, 2.4), B.CANDLE, 5.0, "trappefot", True, 1.1)
    # Slusevokterens stue: køya og hvilestedet (motoren).
    v = rm["vokterstue"]
    K.box(mb, (v.x1 - 2.1, v.y, v.z1 - 0.95), (v.x1 - 0.1, v.y + 0.45, v.z1 - 0.1), "oak")
    v.obstacles.append(sbox(v.x1 - 2.1, v.z1 - 0.95, v.x1 - 0.1, v.z1 - 0.1))
    L.box((v.x1 - 1.1, v.y + 0.22, v.z1 - 0.52), (2.0, 0.45, 0.85))
    L.wall_torch("vokterstue", "s", -17.0, 2.2, intensity=1.6)
    # Kanalen: grønnlige lykter langs den nordre gangbanen, kaldt lys over vannet.
    k = rm["kanal"]
    for x in range(2, 80, 12):
        L.torch("kanal", (x, 0.0 + 2.2, k.z0 + 0.02), (0.0, 1.0), intensity=1.4, reach=9.0)
        L.lights[-1]["color"] = list(GREEN)
    for x in range(8, 80, 16):
        L.light((x, 0.0 + 2.6, k.z1 - 0.3), B.COLD, 8.0, "kanal", True, 0.7)
    # Mosegrotta: rasmasse og kaldt lys.
    mg = rm["mosegrotta"]
    K.rubble(mb, 33.0, 6.0, mg.y, r=1.3, h=0.6, seed=31)
    mg.obstacles.append(Point(33.0, 6.0).buffer(1.2))
    L.box((33.0, mg.y + 0.3, 6.0), (2.0, 0.6, 2.0))
    L.light((24.0, mg.y + 2.4, 4.0), GREEN, 8.0, "mosegrotta", True, 0.9)
    # Fossen: vannet fra røret i nordmuren.
    f = rm["fossen"]
    mb.quad((57.0, f.y + 0.03, 1.0), (67.0, f.y + 0.03, 1.0), (67.0, f.y + 0.03, 4.0), (57.0, f.y + 0.03, 4.0), "water", facing=(0, 1, 0))
    L.light((65.5, f.y + 2.4, 1.2), B.COLD, 8.0, "fossen", True, 0.9)
    L.wall_torch("fossen", "w", 8.0, 2.4, intensity=1.4)
    bs = rm["basseng"]
    mb.quad((bs.x0 + 1.5, bs.y + 0.03, bs.z0 + 1.2), (bs.x1 - 1.5, bs.y + 0.03, bs.z0 + 1.2), (bs.x1 - 1.5, bs.y + 0.03, bs.z1 - 2.5), (bs.x0 + 1.5, bs.y + 0.03, bs.z1 - 2.5), "water", facing=(0, 1, 0))
    L.light((64.5, bs.y + 2.2, -9.0), (1.0, 0.8, 0.45), 6.0, "basseng", True, 1.0)
    # Slusekammeret: rattet er spaken; fakler.
    sl = rm["sluse"]
    L.wall_torch("sluse", "n", 92.0, 2.6, intensity=1.8, reach=10.0)
    L.wall_torch("sluse", "s", 88.5, 2.6, intensity=1.6)
    L.light((95.0, sl.y + 3.0, 24.0), GREEN, 9.0, "sluse", True, 1.0)
    # Rotunden: kaldt lys over sluket.
    L.light((78.0, ROT["bottom"] + 2.0, 54.0), B.COLD, 10.0, "rotunden", True, 0.9)
    # Silkammeret: jerngitter som siler vannet.
    sk = rm["silkammer"]
    for k2 in range(3):
        x = 30.0 + 4.0 * k2
        for z in (52.5, 63.5):
            for b in range(5):
                xx = x - 0.8 + 0.4 * b
                K.box(mb, (xx - 0.03, sk.y, z - 0.03), (xx + 0.03, sk.y + 2.4, z + 0.03), "iron")
            K.box(mb, (x - 1.0, sk.y + 2.4, z - 0.05), (x + 1.0, sk.y + 2.5, z + 0.05), "iron")
            sk.obstacles.append(sbox(x - 1.0, z - 0.1, x + 1.0, z + 0.1))
            L.box((x, sk.y + 1.25, z), (2.0, 2.5, 0.2))
    L.light((34.0, sk.y + 3.2, 58.0), GREEN, 10.0, "silkammer", True, 1.2)
    # Underkummen, kammeret under utløpet, bunnen av sluket.
    L.light((60.5, -8 + 2.0, 76.0), B.COLD, 6.0, "underkum", False, 0.6)
    L.light((36.0, -8.6 + 2.0, 81.0), B.COLD, 5.0, "stigerom", False, 0.5)
    L.light((79.4, -8 + 2.0, 57.2), B.COLD, 5.0, "rorbunn", False, 0.5)
    L.shaft(STIGE, -8.6 + 2.9, rm["utlop"].y)
    L.shaft(SLUK, -8 + 3.0, ROT["bottom"])
    # Utløpet og nedgangen.
    L.wall_torch("utlop", "e", 84.5, 2.4, intensity=1.6)
    L.wall_torch("utlop", "s", 26.0, 2.4, intensity=1.6)
    L.light((27.0, rm["utlop"].y + 3.0, 81.0), GREEN, 8.0, "utlop", True, 0.8)
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 3.0)
    L.wall_torch("nedgang", "w", 97.0, 2.4, intensity=1.4)
    # Kapellet og rottereiret.
    kp = rm["kapell"]
    L.light((55.0, kp.y + 2.0, 39.0), (1.0, 0.8, 0.45), 5.0, "kapell", True, 1.0)
    rr = rm["rottereir"]
    K.rubble(mb, 1.0, 46.0, rr.y, r=1.2, h=0.5, seed=33)
    rr.obstacles.append(Point(1.0, 46.0).buffer(1.1))
    L.box((1.0, rr.y + 0.25, 46.0), (1.8, 0.5, 1.8))
    L.light((6.0, rr.y + 2.2, 40.0), GREEN, 7.0, "rottereir", True, 0.7)
    # Snarveien: fakler i den lange gangen.
    L.torch("utlop-vokterstue", (-24 + 0.92, -5 + 1.9, 70.0), (1.0, 0.0), intensity=1.0, reach=8.0)
    L.torch("utlop-vokterstue", (-24 - 0.92, 0 + 1.9, 15.0), (-1.0, 0.0), intensity=1.0, reach=8.0)
    L.torch("sluse-rotunden", (92 + 1.23, 0 + 1.9, 44.0), (-1.0, 0.0), intensity=1.2, reach=7.0)


LAYOUT = dict(
    theme="kloakk",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round=ROUND,
    override={
        "vokterstue": {"shape": ("rect", -16, 2, 9, 7)},
        "kanal": {"y": KANAL["bottom"], "h": 8.4},
        "rotunden": {"y": ROT["bottom"], "h": 9.0},
        "rottereir": {"shape": ("poly", [(-4, 37), (10, 37), (14, 45), (7, 52), (-3, 49)]), "y": KANAL["bottom"]},
        "silkammer": {"shape": ("poly", SIL)},
    },
    slab=("kanal", "utlop"),
    wall_mats={"kanal": "wall_dark", "rottereir": "wall_dark", "silkammer": "wall_dark", "underkum": "wall_dark",
               "utlop": "wall_dark", "kapell": "wall_dark", "rorbunn": "wall_dark", "stigerom": "wall_dark", "mosegrotta": "wall_dark"},
    cells={"kanal": 1.5, "sluse": 1.5},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
)
