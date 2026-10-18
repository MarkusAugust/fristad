"""Utlegget for nivå 9, Overgangen (planen: `levels/level_09.py`).

Her går borgen over i Fristil. Trappa fra tronsalen ender i **Sømporten**
(vanlig stein, men fugene lyser), og **Sprekkgangen** fører ned til
**Speilsalen**: glatt, lys plast i gulvet og to rader søyler som speiler
hverandre. Rustflekkene i vestmuren skjuler **Det andre rommet** (varig
liv). Fra speilsalens østmur går trappa ut på **Brohodet**, en hylle langs
kanten av **Tomrommet**: en stor, mørk hall med bunnen 16 m under broene,
der bare sømmene i bunnen lyser. Broene går over tomrommet til **Den siste
steinen** (holmen med hvilestedet under den hvite lyssøyla), sørover til
**Borgfragmentet** (en løs bit av borgen med kista og fanen) og østover til
**Østkanten**. Faller du, står du på bunnen og klatrer opp stigene til
holmen eller østkanten. Fra Østkanten går trappa ned i **Plastkammeret**
(Arkivet med den siste lesbare boka ved siden av), og videre ned til
**Kuben**, der sømmen i veggen er spaken som heiser gitteret mot
**Tvillingsalen**. Snarveien fra Kuben går tilbake til Østkanten (slåen på
kubesida). I Tvillingsalen venter Tvillingvokteren (en modal, eliten med
nøkkelen) med to tabeller, og porten bak den åpner **Nedgangen**.

Tomrommet er vertsrommet (`nowalk`, som sjøen i nivå 6); strendene, holmen
og fragmentet er rom inne i det (`virtual`) på murte blokker fra bunnen.

Ikke laget ennå (`docs/wip/krypten.md`): at speilsalen glitcher mellom to
tilstander, at kuben vrir seg, og et eget fallhull fra plastkammeret.
Fiendene er de som finnes (Card → button, Tabs → accordion, Skeleton loader
→ breadcrumb, Popover → tooltip, Grid → table, Tvillingvokteren → modal).
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
I = B.INSET
# Tomrommet: hallen, bunnen og høyden broene og plattformene står i.
VOID = dict(x0=15.0, x1=71.0, z0=36.0, z1=84.0, bottom=-20.0, deck=-4.0)
BRO = dict(x0=15.0, x1=20.0, z0=48.0, z1=68.0)  # brohodet langs vestmuren
OST = dict(x0=66.0, x1=71.0, z0=40.0, z1=80.0)  # østkanten langs østmuren
OYA = dict(c=(43.0, 60.0), r=7.0)
FRAG = dict(c=(43.0, 77.5), r=4.0)
BRIDGE_W = 2.5
FRAG_W = 1.8
SPEIL, KUBE = -2.0, -8.0
PLAST = -6.0
TVILLING = -9.0
VIOLET = (0.75, 0.4, 1.0)
WHITE = (0.85, 0.92, 1.0)
PORT, GITTER, SNARVEI, BAK = "tvillingport_9", "somgitter_9", "snarvei_9", "hemmelig_9"
DOWN = (30.0, TVILLING, 95.6)

PLASTP = [(78, 26), (96, 22), (104, 30), (104, 50), (78, 50)]

EXTRA = [
    ("ostkant", "Østkanten", ("rect", (OST["x0"] + OST["x1"]) / 2, (OST["z0"] + OST["z1"]) / 2, OST["x1"] - OST["x0"], OST["z1"] - OST["z0"]), VOID["deck"], 10.0, False),
]

HALLS = [
    ("port", "sprekk", DOOR, [(0, 4), (0, 8)], 3.0, None, None),
    ("sprekk", "speil", STAIRS, [(0, 32), (0, 39)], 3.0, None, None, {"name": "Trappa ned i speilsalen"}),
    ("speil", "bakrom", SECRET, [(-11, 48), (-14.5, 48)], 2.0, None, BAK, {"name": "Rustflekkene"}),
    ("speil", "brohode", STAIRS, [(11, 54), (15, 54)], 3.0, None, None, {"name": "Trappa ut på brohodet"}),
    ("ostkant", "plast", STAIRS, [(OST["x1"], 44), (78, 44)], 3.0, None, None, {"name": "Trappa ned i plastkammeret"}),
    ("plast", "arkiv", DOOR, [(104, 33), (108, 33)], 2.0, None, None),
    ("plast", "kube", STAIRS, [(96, 50), (96, 69)], 2.5, (0, 6.0, 13.0), None, {"name": "Trappa ned i kuben"}),
    ("kube", "tvilling", GATE, [(95, 83), (95, 96), (62, 96)], 2.5, (1, 6.0, 12.0), GITTER, {"name": "Sømgitteret"}),
    ("kube", "ostkant", SHORTCUT, [(88, 76), (OST["x1"], 76)], 2.2, (0, 4.0, 12.0), SNARVEI, {"name": "Snarveien tilbake til Østkanten"}),
    ("tvilling", "nedgang", GATE, [(42, 96), (34, 96)], 3.0, None, PORT, {"name": "Tvillingporten"}),
]


def marks(_):
    hx, hz = OYA["c"]
    hr = OYA["r"]
    d = VOID["deck"]
    return [
        {"kind": "start", "pos": [0, 0, 1.0], "yaw": P},
        {"kind": "up", "pos": [0, 0, -3], "yaw": 0.0},
        {"kind": "hint", "label": "sound", "pos": [4, 0, -2]},
        # Sprekkgangen: knappene, og fragmenter som løsner fra taket.
        {"kind": "mob", "pos": [0, 0, 14], "label": "button", "n": 3},
        {"kind": "trap", "label": "rocks", "id": "fragmenter_9", "pos": [0, 0, 24], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        # Speilsalen.
        {"kind": "mob", "pos": [0, SPEIL, 46], "label": "accordion"},
        {"kind": "mob", "pos": [-5, SPEIL, 44], "label": "breadcrumb", "n": 3},
        {"kind": "mob", "pos": [6, SPEIL, 52], "label": "breadcrumb", "n": 2, "patrol": [[6, SPEIL, 42], [6, SPEIL, 54]]},
        {"kind": "secret_wall", "id": BAK, "pos": [-11, SPEIL, 48], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [-10.95, SPEIL, 49.6], "yaw": P / 2, "for": BAK},
        {"kind": "bonus", "pos": [-20.4, SPEIL, 48.0], "yaw": P / 2, "label": "liv"},
        {"kind": "lore", "pos": [-18.0, SPEIL, 44.9], "yaw": 0.0, "label": "Lappen i det andre rommet",
         "text": "Rommet her er likt det store, bare mindre, som et speilbilde noen glemte å rive. Rammeverket bygger "
                 "alt to ganger; det klarer ikke å la noe være slik det er. Jeg sto lenge og ventet på at det skulle "
                 "bli til noe. Det ble det ikke. Ta med deg det som er her, og gå. A."},
        # Brohodet og broene.
        {"kind": "mob", "pos": [17.5, d, 62], "label": "tooltip"},
        {"kind": "mob", "pos": [28, d, 60], "label": "tooltip", "n": 2},
        # Den siste steinen: hvilestedet under lyssøyla.
        {"kind": "rest", "pos": [hx - 2.6, d, hz - 2.6], "yaw": P * 0.75},
        {"kind": "ladder", "pos": [hx, VOID["bottom"], hz - hr - 0.3], "yaw": P, "top": [hx, d, hz - hr + 1.0]},
        {"kind": "ladder", "pos": [OST["x0"] - 0.3, VOID["bottom"], 52.0], "yaw": -P / 2, "top": [OST["x0"] + 1.0, d, 52.0]},
        # Borgfragmentet.
        {"kind": "mob", "pos": [FRAG["c"][0], d, FRAG["c"][1]], "label": "breadcrumb", "n": 3},
        {"kind": "chest", "pos": [FRAG["c"][0] + 1.8, d, FRAG["c"][1] + 1.6], "yaw": P * 0.8},
        {"kind": "lore", "pos": [FRAG["c"][0] - 2.4, d, FRAG["c"][1] + 1.2], "yaw": P / 2, "label": "Fanen på borgfragmentet",
         "text": "Fanen fra ridderhallen henger her, på en bit av muren som ikke lenger står på noe. Rammeverket tar "
                 "til seg alt som ble bygd i det, bit for bit, og glemmer å ta resten. Slik vokser det seg helt igjen: "
                 "av det vi lot ligge. A."},
        # Østkanten.
        {"kind": "mob", "pos": [68.5, d, 66], "label": "table"},
        {"kind": "shortcut", "id": SNARVEI, "from": "kube", "pos": [88, KUBE, 76], "yaw": P / 2, "width": 1.6},
        # Plastkammeret: tabellene patruljerer, accordion vokter kista.
        {"kind": "mob", "pos": [86, PLAST, 34], "label": "table", "n": 3, "patrol": [[82, PLAST, 30], [98, PLAST, 28], [100, PLAST, 44], [84, PLAST, 46]]},
        {"kind": "mob", "pos": [84, PLAST, 40], "label": "accordion"},
        {"kind": "chest", "pos": [100.5, PLAST, 33.0], "yaw": -P / 2},
        {"kind": "hint", "label": "sound", "pos": [99, PLAST, 46]},
        # Arkivet.
        {"kind": "mob", "pos": [112, PLAST, 35], "label": "breadcrumb", "n": 2},
        {"kind": "lore", "pos": [112, PLAST, 28.6], "yaw": 0.0, "label": "Den siste lesbare boka",
         "text": "Dette er den siste boka i arkivet som står i mønsterbokas eget mål, det som kunne leses på alle "
                 "tungemål. De andre ble skrevet om til hoffets tunge, og så om igjen etter oppgraderingen, til ingen "
                 "kunne lese dem. Mønsteret er ikke borte. Det er bare glemt. Den som husker hvordan en Button skal "
                 "være, husker resten. Ansgar."},
        # Kuben: bakholdet, sømmen er spaken.
        {"kind": "mob", "pos": [95, KUBE, 76], "label": "breadcrumb i bakhold", "n": 4, "ambush": "kube"},
        {"kind": "lever", "id": "sommen_9", "pos": [101.4, KUBE, 72.0], "yaw": -P / 2, "opens": [GITTER]},
        {"kind": "rare", "pos": [101.4, KUBE, 80.0], "yaw": -P / 2, "label": "Tabs"},
        {"kind": "lore", "pos": [88.6, KUBE, 81.5], "yaw": P / 2, "label": "Nettets stemme",
         "text": "Herfra hører du det: en stemme bak veggene som teller. Én, to, tre, og så på nytt. Det er hjertet, "
                 "den delen Gorvek aldri fikk delt. Det sover ikke. Det teller delene som kommer hjem. Jeg går ned for "
                 "å se hvor mange de er. Ansgar."},
        {"kind": "gate", "id": GITTER, "pos": [95, KUBE, 85.0], "yaw": 0.0, "width": 2.2},
        # Tvillingsalen: vokteren med nøkkelen.
        {"kind": "mob", "pos": [58, TVILLING, 92], "label": "tooltip", "n": 2},
        {"kind": "elite", "pos": [52, TVILLING, 96], "yaw": -P / 2, "label": "modal", "opens": [PORT]},
        {"kind": "mob", "pos": [46, TVILLING, 100], "label": "table", "n": 3},
        {"kind": "gate", "id": PORT, "pos": [40.3, TVILLING, 96], "yaw": P / 2, "width": 2.6},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    t = rm["tomrom"]
    t.floor_y = VOID["bottom"]
    t.floor_mat = "glitch"
    t.nowalk = True
    rm["speil"].floor_mat = "plastic"
    rm["plast"].floor_mat = "plastic"
    rm["kube"].floor_mat = "plastic"
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))


def disc(c, r, n=32):
    return Polygon([(c[0] + r * math.cos(2 * P * k / n), c[1] + r * math.sin(2 * P * k / n)) for k in range(n)])


def ring_rail(L, c, rad, y, gaps):
    """Rekkverk rundt en rund plattform, utenom åpningene (vinkler i radianer)."""
    gaps = sorted(gaps)
    for k, (a0, a1) in enumerate(gaps):
        nxt = gaps[(k + 1) % len(gaps)][0] + (2 * P if k + 1 == len(gaps) else 0.0)
        m = max(2, math.ceil((nxt - a1) * rad / 0.8))
        path = [(c[0] + (rad - 0.13) * math.cos(a1 + (nxt - a1) * q / m), c[1] + (rad - 0.13) * math.sin(a1 + (nxt - a1) * q / m)) for q in range(m + 1)]
        K.parapet(L.mb, path, y, h=0.95, t=0.26)
        for q0, q1 in zip(path, path[1:]):
            L.wall_box(q0, q1, y, y + 1.0, (0.0, 0.0))


def seam(L, room, a, b, y0, y1, out, light=True):
    """En lysende søm i muren: en smal, fiolett stripe fra y0 til y1 langs
    veggen fra a til b, litt ut fra muren (`out` er normalen inn i rommet)."""
    mb = L.mb
    (ax, az), (bx, bz) = a, b
    ox, oz = out[0] * 0.03, out[1] * 0.03
    lo = (min(ax, bx) + ox - 0.02, y0, min(az, bz) + oz - 0.02)
    hi = (max(ax, bx) + ox + 0.02, y1, max(az, bz) + oz + 0.02)
    K.box(mb, lo, hi, "magic_seam")
    if light:
        L.light(((ax + bx) / 2 + out[0] * 0.6, (y0 + y1) / 2, (az + bz) / 2 + out[1] * 0.6), VIOLET, 7.0, room, True, 1.2)


def features(L):
    mb = L.mb
    rm = L.rooms
    d, bottom = VOID["deck"], VOID["bottom"]
    c, rad = OYA["c"], OYA["r"]
    fc, fr = FRAG["c"], FRAG["r"]
    # Strendene: murte blokker fra bunnen opp til broene, inn i murene.
    bro_rect = sbox(BRO["x0"] - 0.5, BRO["z0"], BRO["x1"], BRO["z1"])
    ost_rect = sbox(OST["x0"], OST["z0"], OST["x1"] + 0.5, OST["z1"])
    L.block(bro_rect, bottom, d, mat="wall_dark", top_mat="floor", cell=2.0)
    L.block(ost_rect, bottom, d, mat="wall_dark", top_mat="plastic", cell=2.0)
    # Holmen og fragmentet: runde blokker med rekkverk utenom broene og stigen.
    hole = disc(c, rad)
    frag = disc(fc, fr, 24)
    L.block(hole, bottom, d, mat="wall_dark", top_mat="floor")
    L.block(frag, bottom, d, mat="wall_dark", top_mat="floor")
    hb = math.asin((BRIDGE_W / 2 + 0.05) / rad)
    hf = math.asin((FRAG_W / 2 + 0.05) / rad)
    ring_rail(L, c, rad, d, [(-hb, hb), (P - hb, P + hb), (P / 2 - hf, P / 2 + hf), (3 * P / 2 - 0.12, 3 * P / 2 + 0.12)])
    hff = math.asin((FRAG_W / 2 + 0.05) / fr)
    ring_rail(L, fc, fr, d, [(3 * P / 2 - hff, 3 * P / 2 + hff)])
    # Flat murflate bak stigen på holmen (kollideren ved foten av stigen).
    L.box((c[0], (bottom + d) / 2, c[1] - rad + 0.25), (1.2, d - bottom, 0.5))
    L.box((OST["x0"] + 0.25, (bottom + d) / 2, 52.0), (0.5, d - bottom, 1.2))
    # Broene: brohodet–holmen, holmen–østkanten, holmen–fragmentet.
    _, bw = L.stone_bridge((BRO["x1"], c[1]), (c[0] - rad + 0.3, c[1]), BRIDGE_W, d, clip=hole)
    _, be = L.stone_bridge((c[0] + rad - 0.3, c[1]), (OST["x0"], c[1]), BRIDGE_W, d, clip=hole)
    _, bf = L.stone_bridge((c[0], c[1] + rad - 0.3), (fc[0], fc[1] - fr + 0.3), FRAG_W, d, clip=unary_union([hole, frag]))
    for bx in (27.5, 58.5):
        K.pillar(mb, bx, c[1], bottom, d - 0.62, size=0.8, mat="wall_dark")
        L.box((bx, (bottom + d) / 2 - 0.3, c[1]), (1.0, d - bottom - 0.6, 1.0))
    # Navigasjonen: hver plattform med sin del av broene (de overlapper ved endene).
    rm["brohode"].floor_walk = unary_union([sbox(BRO["x0"] + I, BRO["z0"] + I, BRO["x1"] - I, BRO["z1"] - I), bw.difference(hole.buffer(0.5))])
    rm["ostkant"].floor_walk = unary_union([sbox(OST["x0"] + I, OST["z0"] + I, OST["x1"] - I, OST["z1"] - I), be.difference(hole.buffer(0.5))])
    rm["fragment"].floor_walk = unary_union([Point(fc).buffer(fr - 0.26 - I, 32), bf.difference(hole.buffer(0.5))])
    rm["oya"].floor_walk = unary_union([Point(c).buffer(rad - 0.26 - I, 48), bw.intersection(hole.buffer(1.0)), be.intersection(hole.buffer(1.0)), bf.intersection(hole.buffer(1.0))])
    # Lyssøyla på Den siste steinen: landemerket som synes over hele tomrommet.
    KM.spire(mb, (c[0], d, c[1]), 4.6, 0.55, mat="magic_white", lean=(0.04, -0.03), seed=9)
    L.box((c[0], d + 2.2, c[1]), (1.5, 4.4, 1.5))
    rm["oya"].obstacles.append(Point(c).buffer(1.1))
    L.light((c[0], d + 3.4, c[1]), WHITE, 60.0, "oya", False, 28.0)
    # Sømmene i bunnen av tomrommet: lange fiolette sprekker i glitchgulvet.
    for (x0, z0), (x1, z1) in (((20, 40), (34, 52)), ((52, 44), (66, 38)), ((24, 72), (36, 82)), ((54, 68), (64, 80)), ((30, 60), (36, 66))):
        L.mb.quad((x0 - 0.35, bottom + 0.02, z0), (x0 + 0.35, bottom + 0.02, z0), (x1 + 0.35, bottom + 0.02, z1), (x1 - 0.35, bottom + 0.02, z1), "magic_seam", facing=(0, 1, 0))
        L.light(((x0 + x1) / 2, bottom + 2.5, (z0 + z1) / 2), VIOLET, 24.0, "tomrom", False, 6.0)
    # Skjæret fra sømmene oppover: de murte blokkene og bitene av plast får en fiolett kant.
    for x, z in ((30.0, 46.0), (56.0, 46.0), (30.0, 74.0), (56.0, 74.0)):
        L.light((x, bottom + 8.0, z), VIOLET, 28.0, "tomrom", False, 5.0)
    # Løse biter av plast som henger i lufta over bunnen.
    for k, (x, y, z, s) in enumerate(((24, -10, 42, 1.6), (60, -13, 50, 2.2), (34, -8, 80, 1.2), (52, -15, 70, 2.6), (64, -9, 72, 1.0), (30, -16, 54, 1.8))):
        K.box(mb, (x - s / 2, y - s / 2, z - s / 2), (x + s / 2, y + s / 2, z + s / 2), "plastic")
        seam(L, "tomrom", (x - s / 2, z + s / 2 + 0.01), (x + s / 2, z + s / 2 + 0.01), y - s / 2, y + s / 2, (0, 1), light=k % 2 == 0)
    # Speilsalen: to rader søyler som speiler hverandre, sømmer i murene.
    sp = rm["speil"]
    for x in (-6.0, 6.0):
        for z in (43.0, 48.0, 53.0):
            K.pillar(mb, x, z, sp.y, B.pillar_top(sp, x, z, 0.8), size=0.8, mat="dressed")
            sp.obstacles.append(sbox(x - 0.5, z - 0.5, x + 0.5, z + 0.5))
            L.box((x, sp.y + 4.0, z), (0.9, 8.0, 0.9))
    for z in (42.0, 52.0):
        seam(L, "speil", (-10.98, z), (-10.98, z), SPEIL + 0.2, SPEIL + 5.5, (1, 0))
        seam(L, "speil", (10.98, z - 2.0), (10.98, z - 2.0), SPEIL + 0.2, SPEIL + 5.5, (-1, 0))
    # Kuben: plast med sømmer i alle fire murene, og sømmen som er spaken.
    ku = rm["kube"]
    for x in (91.0, 99.0):
        seam(L, "kube", (x, 69.02), (x, 69.02), KUBE + 0.1, KUBE + 7.0, (0, 1))
        seam(L, "kube", (x, 82.98), (x, 82.98), KUBE + 0.1, KUBE + 7.0, (0, -1))
    for z in (74.0, 79.0):
        seam(L, "kube", (88.02, z), (88.02, z), KUBE + 0.1, KUBE + 7.0, (1, 0))
    seam(L, "kube", (101.98, 72.0), (101.98, 72.0), KUBE + 0.1, KUBE + 7.0, (-1, 0))
    L.light((95.0, KUBE + 6.0, 76.0), (0.8, 0.7, 1.0), 20.0, "kube", False, 6.0)
    # Plastkammeret: sømmer og et kaldt lys.
    for (a, b, out) in (((80.0, 26.0), (80.0, 26.0), (0.2, 1)), ((103.98, 40.0), (103.98, 40.0), (-1, 0)), ((90.0, 49.98), (90.0, 49.98), (0, -1))):
        seam(L, "plast", a, b, PLAST + 0.1, PLAST + 5.5, out)
    L.light((92.0, PLAST + 5.0, 38.0), (0.75, 0.8, 1.0), 24.0, "plast", False, 7.0)
    # Tvillingsalen: to like søyler, en på hver side av vokteren.
    tv = rm["tvilling"]
    for z in (92.5, 99.5):
        K.pillar(mb, 52.0, z, tv.y, B.pillar_top(tv, 52.0, z, 0.9), size=0.9, mat="dressed")
        tv.obstacles.append(sbox(51.4, z - 0.6, 52.6, z + 0.6))
        L.box((52.0, tv.y + 3.5, z), (1.0, 7.0, 1.0))
        seam(L, "tvilling", (52.0, z + 0.47), (52.0, z + 0.47), TVILLING + 0.2, TVILLING + 4.0, (0, 1), light=False)
    L.light((52.0, TVILLING + 5.0, 96.0), (0.8, 0.7, 1.0), 22.0, "tvilling", False, 7.0)


def dress(L):
    rm = L.rooms
    d = VOID["deck"]
    # Sømporten: vanlig stein, fakler, og de første sømmene i fugene.
    L.wall_torch("port", "w", 0.0, 2.3, intensity=1.5)
    L.wall_torch("port", "e", 0.0, 2.3, intensity=1.5)
    seam(L, "port", (4.98, -2.5), (4.98, -2.5), 0.4, 3.2, (-1, 0))
    seam(L, "port", (-4.98, 2.5), (-4.98, 2.5), 0.4, 3.2, (1, 0))
    # Sprekkgangen: sømmene blir flere.
    for z in (12.0, 20.0, 28.0):
        seam(L, "sprekk", (1.98, z), (1.98, z), 0.3, 3.0, (-1, 0))
    L.wall_torch("sprekk", "w", -6.0, 2.2, intensity=1.2)
    L.wall_torch("sprekk", "w", 6.0, 2.2, intensity=1.2)
    # Speilsalen: kaldt, jevnt lys.
    L.light((0.0, SPEIL + 6.0, 48.0), (0.8, 0.85, 1.0), 26.0, "speil", False, 8.0)
    # Det andre rommet: ett svakt lys.
    L.light((-18.0, SPEIL + 2.2, 48.0), (0.8, 0.7, 1.0), 6.0, "bakrom", True, 1.2)
    # Strendene og fragmentet.
    for z in (52.0, 64.0):
        L.light((17.5, d + 3.0, z), (0.8, 0.85, 1.0), 14.0, "brohode", False, 4.0)
    for z in (46.0, 60.0, 74.0):
        L.light((68.5, d + 3.0, z), (0.8, 0.85, 1.0), 14.0, "ostkant", False, 4.0)
    L.light((FRAG["c"][0], d + 2.5, FRAG["c"][1]), (1.0, 0.62, 0.3), 10.0, "fragment", True, 2.5)
    # Arkivet og nedgangen.
    L.light((112.0, PLAST + 2.6, 33.0), (1.0, 0.68, 0.32), 8.0, "arkiv", True, 1.6)
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 3.0)
    # Gangene.
    L.light((95.0, KUBE + 2.0, 90.0), VIOLET, 8.0, "kube-tvilling", False, 1.0)
    L.light((80.0, PLAST + 2.0, 76.0), VIOLET, 8.0, "kube-ostkant", False, 0.8)
    L.light((96.0, PLAST + 1.0, 60.0), VIOLET, 8.0, "plast-kube", False, 1.0)


LAYOUT = dict(
    theme="kjerne",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round={},
    virtual={"brohode": "tomrom", "oya": "tomrom", "fragment": "tomrom", "ostkant": "tomrom"},
    override={
        "tomrom": {"shape": ("rect", (VOID["x0"] + VOID["x1"]) / 2, (VOID["z0"] + VOID["z1"]) / 2, VOID["x1"] - VOID["x0"], VOID["z1"] - VOID["z0"]), "y": VOID["bottom"], "h": 30.0},
        "brohode": {"shape": ("rect", (BRO["x0"] + BRO["x1"]) / 2, (BRO["z0"] + BRO["z1"]) / 2, BRO["x1"] - BRO["x0"], BRO["z1"] - BRO["z0"]), "y": VOID["deck"], "h": 10.0},
        "oya": {"shape": ("circle", OYA["c"][0], OYA["c"][1], OYA["r"]), "y": VOID["deck"], "h": 10.0},
        "fragment": {"shape": ("circle", FRAG["c"][0], FRAG["c"][1], FRAG["r"]), "y": VOID["deck"], "h": 10.0},
        "speil": {"y": SPEIL, "h": 9.0},
        "plast": {"shape": ("poly", PLASTP), "y": PLAST, "h": 7.0},
        "arkiv": {"shape": ("rect", 112, 33, 8, 10), "y": PLAST, "h": 4.0},
        "kube": {"shape": ("rect", 95, 76, 14, 14), "y": KUBE, "h": 10.0},
        "tvilling": {"y": TVILLING, "h": 8.0},
        "nedgang": {"shape": ("rect", 30, 96, 8, 8), "y": TVILLING, "h": 5.0},
    },
    wall_mats={"plast": "plastic", "kube": "plastic", "arkiv": "plastic", "tomrom": "wall_dark"},
    vault_mats={"plast": "plastic", "kube": "plastic", "tomrom": "wall_dark"},
    cells={"tomrom": 2.0},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
    round_seam_free=True,
)
