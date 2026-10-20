"""Utlegget for nivå 7, Den sunkne borgen (planen: `levels/level_07.py`).

Nytt tema, «Borgen under jorda»: en hel borg som sank i berget. Trappa fra
krystallgrotta kommer ut på ytterbredden (hvilestedet: leirbålet til hæren
som aldri kom inn) foran vollgrava. Vindebroen (`bridge`-merket, heist til
vinsjen i porthuset senker den) og gitteret under porthuset er stengt, så
veien inn går ned trappa i grava (−7 m), bort til utfallsporten i foten av
vesttårnet, opp vindeltrappa til brystvernet (+4 m), langs vestmuren inn på
brystvernet rundt borggården, til vinsjrommet (spaken senker broen og heiser
gitteret: snarveien tilbake til leirbålet), og ned igjen gjennom østtårnet
til gården. Fra gården: stallen (bakholdet), kapellet (lysestaken åpner
trappa til borgherrens grav) og kastellet, der Kortkapteinen bærer nøkkelen
til kjellerporten ned til fangehullet og nedgangen.

Brystvernet er hyller på konsoller langs alle fire murene i gården
(`terrace(..., solid=False)`, én flate i navigasjonen), som tårnenes
gallerier og vinsjrommet henger på med ganger i høyden +4 (`ya`/`yb`).
Kastellet har et galleri langs nordmuren med en trapp ned langs vestmuren
(`wall_stair` med et repos, som trappa i cisternen). Bueskytterne finnes
ikke i motoren: breadcrumbene på brystvernet og galleriet står for dem.
"""

import math

import build_level as B
import kit_grav as K
from shapely.geometry import Point, Polygon
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
I = B.INSET
WALK = 4.0  # brystvernet
COURT = -1.0  # gården
MOAT = -7.0  # bunnen av vollgrava
TOWER_W = -3.0  # gulvet i vesttårnet (trappa opp til brystvernet stiger 7 m; brattere enn 27° kommer ikke kapselen opp)
GAL = 3.5  # galleriet i kastellet
DUNGEON = -8.0
GITTER, BRO, PORT = "gitter_7", "vindebro_7", "kjellerport_7"
SKATT, GRAV = "hemmelig_7a", "hemmelig_7b"
GARD = dict(x0=24.0, x1=68.0, z0=36.0, z1=80.0)
BRIDGE = dict(x=46.0, z0=19.0, z1=26.0, w=3.6)
VEST = dict(c=(13.0, 42.0), r=5.5)
OST = dict(c=(79.0, 42.0), r=5.5)
WELL = (46.0, 58.0)
KAST = dict(x0=47.0, x1=65.0, z0=82.0, z1=98.0)
DOWN = (79.0, DUNGEON, 121.6)

EXTRA = [
    ("gravvest", "Vollgrava, vestsida", ("rect", 0.5, 42, 5, 28), MOAT, 16.0, False),
    ("osttarn", "Østtårnet", ("circle", OST["c"][0], OST["c"][1], OST["r"]), COURT, 8.5, False),
]

HALLS = [
    ("trapp", "ytre", STAIRS, [(0, 4), (0, 7)], 3.0, None, None),
    ("ytre", "grav", HALL, [(BRIDGE["x"], 17), (BRIDGE["x"], BRIDGE["z0"])], 4.0, None, None, {"yb": COURT, "name": "Brokanten"}),
    ("grav", "gard", GATE, [(BRIDGE["x"], BRIDGE["z1"]), (BRIDGE["x"], GARD["z0"])], 4.0, None, GITTER, {"ya": COURT, "name": "Porthuset"}),
    ("ytre", "grav", STAIRS, [(-2, 10), (-9, 10), (-9, 24.5), (-2, 24.5)], 2.5, (1, 1.25, 13.25), None, {"id": "gravtrapp", "name": "Trappa ned i grava"}),
    ("grav", "gravvest", HALL, [(0.5, 26), (0.5, 28)], 3.0, None, None, {"name": "Vollgrava"}),
    ("gravvest", "skatt", SECRET, [(0.5, 56), (0.5, 58)], 2.0, None, SKATT),
    ("grav", "vesttarn", STAIRS, [(21, 26), (21, 42), (VEST["c"][0] + VEST["r"] - 0.4, 42)], 2.5, (0, 1.25, 14.75), None, {"name": "Utfallsporten"}),
    ("vesttarn", "gard", HALL, [(VEST["c"][0], VEST["c"][1] - VEST["r"] + 0.4), (VEST["c"][0], 31), (28, 31), (28, GARD["z0"])], 2.5, None, None, {"ya": WALK, "yb": WALK, "name": "Vestmuren"}),
    ("gard", "porthus", DOOR, [(54, GARD["z0"]), (54, 33)], 2.5, None, None, {"ya": WALK, "name": "Vinsjrommet"}),
    ("gard", "osttarn", DOOR, [(GARD["x1"], 42), (OST["c"][0] - OST["r"] + 0.4, 42)], 2.5, None, None),
    ("osttarn", "gard", HALL, [(OST["c"][0], OST["c"][1] - OST["r"] + 0.4), (OST["c"][0], 31), (64, 31), (64, GARD["z0"])], 2.5, None, None, {"ya": WALK, "yb": WALK, "name": "Østmuren"}),
    ("gard", "stall", DOOR, [(33, GARD["z1"]), (33, 82)], 2.5, None, None),
    ("gard", "kapell", DOOR, [(GARD["x0"], 70), (21, 70)], 2.0, None, None),
    ("kapell", "gravkammer", SECRET, [(16, 75), (16, 88)], 2.0, (0, 1.0, 12.0), GRAV, {"name": "Trappa under alteret"}),
    ("gard", "kastell", DOOR, [(56, GARD["z1"]), (56, KAST["z0"])], 3.0, None, None),
    ("kastell", "fangehull", GATE, [(56, KAST["z1"]), (56, 118)], 2.5, (0, 2.5, 18.0), PORT, {"name": "Kjellerporten"}),
    ("fangehull", "nedgang", DOOR, [(68, 122), (75, 122)], 2.5, None, None),
]

ROUND = {
    "vesttarn": dict(
        gallery=dict(y=WALK, ring_in=2.3, stair_in=2.4, stair_a0=math.radians(20), stair_a1=math.radians(250), steps=45, segs=36,
                     id="vesttarn_galleri", name="Brystvernet i vesttårnet", stair_id="vesttarn_trapp", stair_name="Vindeltrappa i vesttårnet"),
        dome_rise=2.5, ribs=8, wall_cell=1.5, dome_seg=32, dome_rings=5,
        torches=[(100, 2.3, 1.4, 9.0, False), (200, 2.3, 1.4, 9.0, False), (300, 2.2, 1.3, 8.0, True), (340, 2.2, 1.3, 8.0, True)],
    ),
    "osttarn": dict(
        gallery=dict(y=WALK, ring_in=2.3, stair_in=2.4, stair_a0=math.radians(290), stair_a1=math.radians(520), steps=25, segs=36,
                     id="osttarn_galleri", name="Brystvernet i østtårnet", stair_id="osttarn_trapp", stair_name="Vindeltrappa i østtårnet"),
        dome_rise=2.5, ribs=8, wall_cell=1.5, dome_seg=32, dome_rings=5,
        torches=[(30, 2.3, 1.4, 9.0, False), (120, 2.3, 1.4, 9.0, False), (200, 2.2, 1.3, 8.0, True), (250, 2.2, 1.3, 8.0, True)],
    ),
}


def marks(_):
    bx = BRIDGE["x"]
    return [
        {"kind": "start", "pos": [0, 0, 1], "yaw": P},
        {"kind": "up", "pos": [0, 0, -3], "yaw": 0.0},
        # Ytterbredden: leirbålet (hvilestedet) og vaktene.
        {"kind": "rest", "pos": [12, COURT, 11], "yaw": P},
        {"kind": "mob", "pos": [44, COURT, 12], "label": "kortknekt", "n": 2, "patrol": [[30, COURT, 12], [60, COURT, 13]]},
        # Vindebroen og gitteret: vinsjen i porthuset åpner begge.
        {"kind": "bridge", "id": BRO, "pos": [bx, COURT, (BRIDGE["z0"] + BRIDGE["z1"]) / 2], "yaw": 0.0,
         "size": [BRIDGE["w"], BRIDGE["z1"] - BRIDGE["z0"]], "height": 3.2, "top": [bx, 8.5, (BRIDGE["z0"] + BRIDGE["z1"]) / 2]},
        {"kind": "gate", "id": GITTER, "pos": [bx, COURT, 31.0], "yaw": 0.0, "width": 3.4},
        # Vollgrava: tabellene i gjørma, steinene fra skoddehullene, og det rasede hjørnet.
        {"kind": "mob", "pos": [20, MOAT, 22.5], "label": "table", "n": 2},
        {"kind": "trap", "label": "rocks", "id": "skoddehull_7", "pos": [30, MOAT, 22.5], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "mob", "pos": [1, MOAT, 40], "label": "kortknekt"},
        {"kind": "secret_wall", "id": SKATT, "pos": [0.5, MOAT, 56], "yaw": P, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "draft", "pos": [0.5, MOAT, 55.4], "for": SKATT},
        {"kind": "rare", "pos": [0.5, MOAT, 62.6], "yaw": P, "label": "Switch"},
        # Vesttårnet.
        {"kind": "mob", "pos": [15.5, TOWER_W, 40], "label": "accordion", "n": 2},
        # Brystvernet: bueskytterne (breadcrumber) i rekke langs murene.
        {"kind": "mob", "pos": [25.5, WALK, 58], "label": "breadcrumb-bueskytter", "n": 3, "patrol": [[25.5, WALK, 44], [25.5, WALK, 72]]},
        {"kind": "mob", "pos": [40, WALK, 37.5], "label": "breadcrumb-bueskytter", "n": 2},
        {"kind": "mob", "pos": [66.5, WALK, 60], "label": "breadcrumb-bueskytter", "n": 2, "patrol": [[66.5, WALK, 44], [66.5, WALK, 74]]},
        # Vinsjrommet i porthuset.
        {"kind": "lever", "id": "vinsj_7", "pos": [54, WALK, 27.45], "yaw": 0.0, "opens": [GITTER, BRO]},
        {"kind": "mob", "pos": [51, WALK, 31], "label": "accordion"},
        # Borggården: vakter, den tørre brønnen og steinene fra taket.
        {"kind": "mob", "pos": [46, COURT, 50], "label": "kortknekt", "n": 3, "patrol": [[34, COURT, 48], [58, COURT, 48], [58, COURT, 68], [34, COURT, 68]]},
        {"kind": "trap", "label": "rocks", "id": "bronnen_7", "pos": [46, COURT, 61.0], "size": [1.2, 1.2], "dir": [0, 0, -1]},
        # Østtårnet.
        {"kind": "mob", "pos": [76.5, COURT, 39.5], "label": "tooltip", "n": 2},
        # Stallen: bakholdet i båsene, kista i høyet.
        {"kind": "mob", "pos": [33, COURT, 86], "label": "kortknekt i bakhold", "n": 3, "ambush": "stall"},
        {"kind": "chest", "pos": [37.5, COURT, 88.4], "yaw": P},
        # Kapellet: lysestaken åpner steinhella under alteret.
        {"kind": "mob", "pos": [16, COURT, 69], "label": "accordion", "n": 2},
        {"kind": "lever", "id": "lysestaken_7", "pos": [19.3, COURT, 73.6], "yaw": P, "opens": [GRAV]},
        {"kind": "secret_wall", "id": GRAV, "pos": [16, COURT, 75], "yaw": P, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "draft", "pos": [16, COURT, 74.4], "for": GRAV},
        # Borgherrens grav.
        {"kind": "lore", "pos": [12.0, -6, 91], "yaw": P / 2, "label": "Gravskriften",
         "text": "Her hviler Hallvard, herre til borgen som sank. Han ville ikke brenne mønsterboka. Da Inkvisisjonen "
                 "kom i de røde kappene, lot han murene mure og broen heise, og berget tok imot dem alle, inkvisitorer "
                 "og borgfolk. Den som leser dette, har kommet lenger enn kongens menn."},
        {"kind": "bonus", "pos": [20.0, -6, 91], "yaw": -P / 2, "label": "liv"},
        # Kastellet: Kortkapteinen med nøkkelen, skjoldbærerne og bueskytterne på galleriet.
        {"kind": "elite", "pos": [56, COURT, 91], "yaw": 0.0, "label": "modal", "opens": [PORT]},
        {"kind": "mob", "pos": [52, COURT, 88], "label": "kortknekt", "n": 2},
        {"kind": "mob", "pos": [58, GAL, 83.4], "label": "breadcrumb-bueskytter", "n": 2},
        {"kind": "chest", "pos": [64.3, COURT, 95], "yaw": -P / 2},
        {"kind": "gate", "id": PORT, "pos": [56, COURT, 99.6], "yaw": 0.0, "width": 2.2},
        # Fangehullet og nedgangen.
        {"kind": "mob", "pos": [50, DUNGEON, 122], "label": "accordion", "n": 2},
        {"kind": "lore", "pos": [60, DUNGEON, 118.6], "yaw": 0.0, "label": "Rissene i veggen",
         "text": "Dag 40. De satte meg her fordi jeg skrev en Button som bare var en Button, uten Kontorets stempel. "
                 "Nå sier de at Navbaren holder tronsalen under oss, og at ingen kommer forbi den. Jeg har hørt den "
                 "gå. Lenkene mine er løse nå. Det er ikke lenkene som holder meg her."},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    g = rm["grav"]
    g.floor_mat = "mud"
    rm["gravvest"].floor_mat = "mud"
    rm["gard"].holes.append(Point(WELL).buffer(0.85, 16))
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))


def banner(L, room, x, y, z, yaw):
    """En fane på muren (henger; `place_mounted` i motoren), med en tynn kollider over duken."""
    L.prop("kit_banner", x, y, z, yaw)
    fx, fz = math.sin(yaw), math.cos(yaw)
    L.box((x + fx * 0.2, y - 1.0, z + fz * 0.2), (1.0, 2.0, 0.14), yaw)


def armor(L, room, x, z, yaw, y=None):
    r = L.rooms[room]
    yy = r.y if y is None else y
    L.prop("kit_armor", x, yy, z, yaw)
    L.box((x, yy + 0.95, z), (0.7, 1.9, 0.7), yaw)
    r.obstacles.append(Point(x, z).buffer(0.5))


def features(L):
    mb = L.mb
    rm = L.rooms
    # Vollgrava: broa er en mekanisme; her står brokarene den hviler på og
    # navigasjonen over den (stengt til broen er nede).
    g = rm["grav"]
    bx, w = BRIDGE["x"], BRIDGE["w"]
    for zz in (BRIDGE["z0"], BRIDGE["z1"]):
        K.box(mb, (bx - w / 2 - 0.2, COURT - 0.6, zz - 0.25), (bx + w / 2 + 0.2, COURT - 0.02, zz + 0.25), "dressed")
    for zz in (BRIDGE["z0"] + 0.7, BRIDGE["z1"] - 0.7):
        K.pillar(mb, bx, zz, MOAT, COURT - 0.62, size=1.0)
        L.box((bx, (MOAT + COURT) / 2 - 0.3, zz), (1.2, COURT - MOAT - 0.6, 1.2))
        g.obstacles.append(sbox(bx - 0.7, zz - 0.7, bx + 0.7, zz + 0.7))
    walk = sbox(bx - w / 2 + I, BRIDGE["z0"] + I, bx + w / 2 - I, BRIDGE["z1"] - I)
    L.decks.append(dict(room="grav", y=COURT, walk=walk, gate=BRO, ring=sbox(bx - w / 2, BRIDGE["z0"], bx + w / 2, BRIDGE["z1"]), obstacles=[]))
    # Råtne påler og steinur i gjørma.
    for x, z, sd in ((10, 21, 31), (58, 24, 32), (68, 21.5, 33)):
        K.rubble(mb, x, z, MOAT, r=1.3, h=0.5, seed=sd)
        g.obstacles.append(Point(x, z).buffer(1.2))
    for x, z in ((6, 24.5), (14, 20.5), (38, 24.5), (52, 20.5), (64, 24.5)):
        K.cylinder(mb, (x, z), 0.16, MOAT, MOAT + 1.2 + 0.3 * ((x * 7) % 3), "wood", n=7, caps=(True, False), smooth=False)
        L.box((x, MOAT + 0.7, z), (0.35, 1.4, 0.35))
        g.obstacles.append(Point(x, z).buffer(0.3))
    # Borggården: brystvernet som hyller på konsoller langs de fire murene,
    # med rekkverk mot gården; én flate i navigasjonen.
    r = rm["gard"]
    x0, x1, z0, z1 = GARD["x0"], GARD["x1"], GARD["z0"], GARD["z1"]
    # Hjørnene: vest- og østhyllene går helt ut i hjørnene uten rekkverk der
    # (`gaps`), og flaten i hjørnet syr de fire hyllene sammen til én.
    corners = [(z0, z0 + 3), (z1 - 3, z1)]
    _, ww = L.terrace(r, x0, x0 + 3, z0, z1, WALK, "e", gaps=corners, solid=False)
    _, we = L.terrace(r, x1 - 3, x1, z0, z1, WALK, "w", gaps=corners, solid=False)
    _, wn = L.terrace(r, x0 + 3, x1 - 3, z0, z0 + 3, WALK, "s", solid=False)
    _, ws = L.terrace(r, x0 + 3, x1 - 3, z1 - 3, z1, WALK, "n", solid=False)
    joins = [sbox(x0 + I, z0 + I, x0 + 3 + I + 0.05, z0 + 3 - 0.26 - I), sbox(x1 - 3 - I - 0.05, z0 + I, x1 - I, z0 + 3 - 0.26 - I),
             sbox(x0 + I, z1 - 3 + 0.26 + I, x0 + 3 + I + 0.05, z1 - I), sbox(x1 - 3 - I - 0.05, z1 - 3 + 0.26 + I, x1 - I, z1 - I)]
    ring = unary_union([sbox(x0, z0, x0 + 3, z1), sbox(x1 - 3, z0, x1, z1), sbox(x0, z0, x1, z0 + 3), sbox(x0, z1 - 3, x1, z1)])
    L.decks.append(dict(room="gard", of="gard", y=WALK, walk=unary_union([ww, we, wn, ws] + joins), ring=ring.buffer(0.3), obstacles=[]))
    # Tinder oppå rekkverket mot gården.
    for k in range(int((z1 - z0 - 6) / 1.8)):
        zc = z0 + 3.9 + 1.8 * k
        K.obox(mb, (x0 + 3 - 0.13, WALK + 0.95, zc), (0.26, 0.45, 0.6), 0.0, "dressed")
        K.obox(mb, (x1 - 3 + 0.13, WALK + 0.95, zc), (0.26, 0.45, 0.6), 0.0, "dressed")
    for k in range(int((x1 - x0 - 6) / 1.8)):
        xc = x0 + 3.9 + 1.8 * k
        K.obox(mb, (xc, WALK + 0.95, z0 + 3 - 0.13), (0.6, 0.45, 0.26), 0.0, "dressed")
        K.obox(mb, (xc, WALK + 0.95, z1 - 3 + 0.13), (0.6, 0.45, 0.26), 0.0, "dressed")
    # Den tørre brønnen midt i gården.
    K.well(mb, WELL[0], WELL[1], COURT, r_in=0.85, r_out=1.3, curb=0.5, depth=6.0)
    L.box((WELL[0], COURT + 0.25, WELL[1]), (2.6, 0.5, 2.6))
    r.obstacles.append(Point(WELL).buffer(1.35))
    # Søyler som bærer steintaket over gården.
    for x, z in ((34, 46), (58, 46), (34, 70), (58, 70)):
        # Krysshvelvet over gården er flatt (R = 2,6 m): søylen går opp i hvelvet over toppen av det.
        K.pillar(mb, x, z, COURT, r.y + r.h + 2.6 + 0.2, size=1.2)
        L.box((x, (COURT + 13.0) / 2, z), (1.5, 14.0, 1.5))
        r.obstacles.append(sbox(x - 0.8, z - 0.8, x + 0.8, z + 0.8))
    # Kastellet: galleriet langs nordmuren (+4,5 m) med reposet i nordvest og
    # trappa ned langs vestmuren.
    k = rm["kastell"]
    kx0, kx1, kz0, kz1 = KAST["x0"], KAST["x1"], KAST["z0"], KAST["z1"]
    _, gw = L.terrace(k, kx0 + 3, kx1, kz0, kz0 + 3, GAL, "s", solid=False)
    landing = sbox(kx0, kz0, kx0 + 3, kz0 + 3)
    L.block(landing, COURT, GAL)
    k.obstacles.append(landing)
    L.wall_stair("kastell", (kx0 + 1.5, kz0 + 3), (kx0 + 1.5, kz0 + 13), 3.0, GAL, COURT, COURT, -1)
    # Hindringen slutter INSET før trappefoten, så gulvets nett deler kant med det nederste trinnet.
    k.obstacles.append(sbox(kx0, kz0 + 3, kx0 + 3, kz0 + 13 - I))
    lw = sbox(kx0 + I, kz0 + I, kx0 + 3 + I + 0.01, kz0 + 3 + 0.01)
    L.decks.append(dict(room="kastell", of="kastell", y=GAL, walk=unary_union([gw, lw]), ring=sbox(kx0, kz0, kx1, kz0 + 3).buffer(0.3), obstacles=[]))
    for j in range(7):
        xc = kx0 + 4.0 + 1.6 * j
        K.obox(mb, (xc, GAL + 0.95, kz0 + 3 - 0.13), (0.6, 0.4, 0.26), 0.0, "dressed")
    # Kapellet: alteret med lysestakene (den ene er spaken) og benker.
    c = rm["kapell"]
    K.altar(mb, 13.5, 73.4, COURT, yaw=0.0, w=2.2, d=1.0, h=1.0)
    L.box((13.5, COURT + 0.5, 73.4), (2.3, 1.0, 1.1))
    c.obstacles.append(sbox(12.3, 72.8, 14.7, 74.0))
    K.candlestick(mb, 12.6, 73.4, COURT + 1.0, h=0.5, arms=2)
    for z in (67.0, 68.6, 70.2):
        for x in (13.5, 18.5):
            K.obox(mb, (x, COURT, z), (2.0, 0.45, 0.4), 0.0, "oak")
            L.box((x, COURT + 0.22, z), (2.0, 0.45, 0.4))
            c.obstacles.append(sbox(x - 1.0, z - 0.2, x + 1.0, z + 0.2))
    # Borgherrens grav: sarkofagen.
    L.prop("kit_sarcophagus", 16.0, -6, 91.0, 0.0)
    L.box((16.0, -6 + 0.4, 91.0), (1.0, 0.8, 2.1))
    rm["gravkammer"].obstacles.append(sbox(15.4, 89.8, 16.6, 92.2))
    # Stallen: båsene (skillevegger av planker) langs sørmuren, høy i den ene.
    s = rm["stall"]
    for x in (28.5, 31.5, 34.5):
        K.box(mb, (x - 0.06, COURT, 86.5), (x + 0.06, COURT + 1.3, 90.0), "oak", faces=("+x", "-x", "+y", "-z"))
        L.box((x, COURT + 0.65, 88.25), (0.12, 1.3, 3.5))
        s.obstacles.append(sbox(x - 0.1, 86.5, x + 0.1, 90.0))
    K.rubble(mb, 37.5, 88.6, COURT, r=1.4, h=0.35, seed=41)
    # Ytterbredden: leirbålet ved hvilestedet, kasser og tønner.
    y = rm["ytre"]
    fx, fz = 16.0, 10.0
    K.rubble(mb, fx, fz, COURT, r=0.9, h=0.22, seed=51)
    K.cylinder(mb, (fx, fz), 0.45, COURT + 0.18, COURT + 0.24, "fire_torch", n=10, caps=(True, False), smooth=False)
    L.box((fx, COURT + 0.2, fz), (1.8, 0.4, 1.8))
    y.obstacles.append(Point(fx, fz).buffer(1.0))
    L.light((fx, COURT + 1.2, fz), (1.0, 0.5, 0.2), 16.0, "ytre", True, 3.0)
    for x, z, yw in ((20.0, 8.5, 0.3), (21.0, 9.6, 0.9), (8.0, 15.5, 0.2)):
        K.obox(mb, (x, COURT, z), (0.8, 0.7, 0.8), yw, "oak")
        L.box((x, COURT + 0.35, z), (0.8, 0.7, 0.8), yw)
        y.obstacles.append(Point(x, z).buffer(0.65))
    for x, z in ((23.0, 8.6), (24.0, 8.7)):
        K.cylinder(mb, (x, z), 0.38, COURT, COURT + 0.85, "oak", n=10, caps=(True, False), smooth=True)
        L.box((x, COURT + 0.42, z), (0.8, 0.85, 0.8))
        y.obstacles.append(Point(x, z).buffer(0.5))


def dress(L):
    rm = L.rooms
    # Trappefoten og ytterbredden.
    L.light((0.0, 2.0, -3.0), B.COLD, 6.0, "trapp", False, 1.6)
    L.wall_torch("trapp", "e", 0.0, 2.3, intensity=4.2)
    for x in (4.0, 30.0, 56.0):
        L.wall_torch("ytre", "n", x, 2.4, intensity=4.5, reach=10.0)
    L.light((34.0, COURT + 6.0, 12.0), B.COLD, 40.0, "ytre", False, 16.0)
    L.light((60.0, COURT + 3.0, 12.0), B.COLD, 16.0, "ytre", False, 6.0)
    # Vollgrava: kaldt lys ovenfra og fakler på borgmuren.
    for x in (10.0, 30.0, 50.0, 70.0):
        L.light((x, MOAT + 6.0, 22.5), (0.4, 0.5, 0.75), 18.0, "grav", False, 8.0)
    for x in (12.0, 34.0, 58.0):
        L.wall_torch("grav", "s", x, 5.5, intensity=4.8, reach=12.0)
    L.light((0.5, MOAT + 4.0, 40.0), (0.4, 0.5, 0.75), 16.0, "gravvest", False, 7.0)
    L.wall_torch("gravvest", "e", 50.0, 2.4, intensity=4.2)
    L.light((0.5, MOAT + 2.0, 61.0), (1.0, 0.8, 0.45), 6.0, "skatt", True, 2.4)
    # Borggården: fakler på murene under brystvernet, lys oppe på muren, og
    # fyll-lys under steintaket.
    x0, x1, z0, z1 = GARD["x0"], GARD["x1"], GARD["z0"], GARD["z1"]
    for z in (44.0, 58.0, 72.0):
        L.wall_torch("gard", "w", z, 2.4, intensity=4.8, reach=10.0)
        L.wall_torch("gard", "e", z, 2.4, intensity=4.8, reach=10.0)
    for x in (32.0, 60.0):
        L.wall_torch("gard", "n", x, 2.4, intensity=4.8, reach=10.0)
        L.wall_torch("gard", "s", x, 2.4, intensity=4.8, reach=10.0)
    for z in (40.0, 52.0, 64.0, 76.0):
        L.wall_torch("gard", "w", z, WALK - COURT + 2.2, intensity=4.5, reach=10.0)
        L.wall_torch("gard", "e", z, WALK - COURT + 2.2, intensity=4.5, reach=10.0)
    for x in (40.0, 52.0):
        L.wall_torch("gard", "n", x, WALK - COURT + 2.2, intensity=4.5, reach=10.0)
        L.wall_torch("gard", "s", x, WALK - COURT + 2.2, intensity=4.5, reach=10.0)
    L.light((46.0, COURT + 9.0, 58.0), B.COLD, 44.0, "gard", False, 28.0)
    L.light((34.0, COURT + 5.0, 46.0), (0.6, 0.65, 0.8), 20.0, "gard", False, 10.0)
    L.light((58.0, COURT + 5.0, 70.0), (0.6, 0.65, 0.8), 20.0, "gard", False, 10.0)
    # Fanene på murene i gården (under hyllene).
    for z in (48.0, 62.0):
        banner(L, "gard", x0 + 0.05, 3.4, z, P / 2)
        banner(L, "gard", x1 - 0.05, 3.4, z, -P / 2)
    for x in (46.0, 64.0):
        banner(L, "gard", x, 3.4, z0 + 0.05, 0.0)
    banner(L, "gard", 40.0, 3.4, z1 - 0.05, P)
    # Vinsjrommet.
    L.wall_torch("porthus", "w", 30.0, 2.3, intensity=4.8)
    L.wall_torch("porthus", "e", 30.0, 2.3, intensity=4.8)
    # Stallen.
    L.wall_torch("stall", "n", 29.0, 2.3, intensity=4.2)
    L.light((33.0, COURT + 2.5, 86.0), (1.0, 0.75, 0.45), 10.0, "stall", True, 3.2)
    # Kapellet og graven.
    L.wall_torch("kapell", "w", 68.0, 2.4, intensity=4.5)
    L.wall_torch("kapell", "e", 68.0, 2.4, intensity=4.5)
    L.light((13.5, COURT + 1.6, 73.4), B.CANDLE, 8.0, "kapell", True, 3.6)
    L.light((16.0, COURT + 4.5, 70.0), (0.55, 0.6, 0.85), 12.0, "kapell", False, 4.8)
    L.light((16.0, -6 + 2.2, 91.0), (1.0, 0.8, 0.45), 7.0, "gravkammer", True, 2.8)
    # Kastellet: faklene, det varme lyset i hallen, galleriet.
    L.wall_torch("kastell", "e", 86.0, 2.4, intensity=4.8, reach=10.0)
    L.wall_torch("kastell", "e", 94.0, 2.4, intensity=4.8, reach=10.0)
    L.wall_torch("kastell", "s", 50.0, 2.4, intensity=4.8, reach=10.0)
    L.wall_torch("kastell", "s", 62.0, 2.4, intensity=4.8, reach=10.0)
    L.wall_torch("kastell", "n", 60.0, GAL - COURT + 2.0, intensity=4.2)
    L.light((56.0, COURT + 6.0, 90.0), (1.0, 0.8, 0.55), 26.0, "kastell", True, 14.0)
    banner(L, "kastell", KAST["x1"] - 0.05, 2.6, 90.0, -P / 2)
    banner(L, "kastell", 52.0, 2.6, KAST["z1"] - 0.05, P)
    banner(L, "kastell", 60.0, 2.6, KAST["z1"] - 0.05, P)
    for x in (52.5, 59.5):
        armor(L, "kastell", x, 96.8, 0.0)
    # Fangehullet: faklene langs midtgangen og rustningene i cellene.
    for x in (48.0, 56.0, 64.0):
        L.wall_torch("fangehull", "n", x, 2.3, intensity=4.5, reach=9.0)
        L.wall_torch("fangehull", "s", x, 2.3, intensity=4.5, reach=9.0)
    for x, z, yw in ((46.0, 119.0, P * 0.9), (52.0, 125.2, 0.1), (60.0, 125.2, -0.2), (66.0, 119.0, P * 1.1)):
        armor(L, "fangehull", x, z, yw)
    # Nedgangen.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 6.0)
    L.wall_torch("nedgang", "n", 77.0, 2.4, intensity=4.2)
    # Gangene.
    L.torch("gravtrapp", (-10.25 + 0.02, -3.5 + 1.9, 16.0), (1.0, 0.0), intensity=3.0, reach=8.0)
    L.torch("grav-vesttarn", (22.25 - 0.02, -5.0 + 1.9, 34.0), (-1.0, 0.0), intensity=3.0, reach=8.0)
    L.torch("vesttarn-gard", (20.0, WALK + 1.9, 29.75 + 0.02), (0.0, 1.0), intensity=3.0, reach=8.0)
    L.torch("osttarn-gard", (72.0, WALK + 1.9, 29.75 + 0.02), (0.0, 1.0), intensity=3.0, reach=8.0)
    L.torch("grav-gard", (44.0 + 0.02, COURT + 1.9, 28.5), (1.0, 0.0), intensity=3.6, reach=9.0)
    L.torch("grav-gard", (48.0 - 0.02, COURT + 1.9, 33.5), (-1.0, 0.0), intensity=3.6, reach=9.0)
    L.torch("kastell-fangehull", (54.75 + 0.02, -4.4 + 1.9, 108.0), (1.0, 0.0), intensity=3.0, reach=8.0)
    L.torch("kapell-gravkammer", (15.0 + 0.02, -3.0 + 1.9, 80.5), (1.0, 0.0), intensity=3.0, reach=7.0)


LAYOUT = dict(
    theme="borg",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round=ROUND,
    override={
        "trapp": {"shape": ("rect", 0, 0, 10, 8), "y": 0, "h": 4.0},
        "ytre": {"shape": ("rect", 34, 12, 72, 10), "y": COURT, "h": 12.0},
        "grav": {"shape": ("rect", 36, 22.5, 76, 7), "y": MOAT, "h": 16.0},
        "skatt": {"shape": ("rect", 0.5, 61, 7, 6), "y": MOAT, "h": 3.0},
        "vesttarn": {"shape": ("circle", VEST["c"][0], VEST["c"][1], VEST["r"]), "y": TOWER_W, "h": 12.5},
        "gard": {"shape": ("rect", 46, 58, 44, 44), "y": COURT, "h": 14.0},
        "porthus": {"shape": ("rect", 54, 30, 10, 6), "y": WALK, "h": 4.5},
        "stall": {"shape": ("rect", 33, 86, 14, 8), "y": COURT, "h": 5.0},
        "kapell": {"shape": ("rect", 16, 70, 10, 10), "y": COURT, "h": 7.0},
        "gravkammer": {"shape": ("rect", 16, 91, 9, 6), "y": -6, "h": 3.2},
        "kastell": {"shape": ("rect", 56, 90, 18, 16), "y": COURT, "h": 9.0},
        "fangehull": {"shape": ("rect", 56, 122, 24, 8), "y": DUNGEON, "h": 3.5},
        "nedgang": {"shape": ("rect", 79, 122, 8, 8), "y": DUNGEON, "h": 5.0},
    },
    wall_mats={"ytre": "rock", "skatt": "rock", "trapp": "rock"},
    vault_mats={"ytre": "rock", "grav": "rock", "gravvest": "rock", "skatt": "rock", "trapp": "rock"},
    cells={"gard": 2.0, "grav": 2.0, "ytre": 2.0, "gravvest": 2.0},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
    round_seam_free=True,
)
