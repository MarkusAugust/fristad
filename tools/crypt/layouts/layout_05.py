"""Utlegget for nivå 5, Gruva (planen: `levels/level_05.py`).

Nytt tema: ned i etapper. Fra gruveinngangen går hovedstollen (tømmerstiver
hver andre meter, skinner i gulvet) ned til malmhallen, den gamle stollen
videre østover til vinsjhuset, og der står heisen: spaken løsner bremsen,
og plattformen tar deg 14 m ned til sjaktbunnen og heisbunnen. Nede går
løkka heisbunnen → krystallåra → smeltehula → vognbanen → heisbunnen, og
fra smeltehula går gruveporten (låst; formannen i smeltehula har nøkkelen)
til nedgangen. Snarveien er luftesjakta: en lang, slak trapp fra gruveporten
opp til lavehuset (hvilestedet) ved inngangen, med slåen nede.

Rommene fra planen er justert (`override`) så gangene treffer rette kanter,
og krystallkammeret ligger rett under det morkne lokket i den gamle stollen.
Gangene inn i den runde smeltehula går radielt langs x eller z (øst, nord og
sør) og ender 0,4 m innenfor sirkelen: en skrå inngang i den runde muren ga
en glippe mellom gangmuren og døråpningen (`snapshot --leak-scan`).
"""

import math

import build_level as B
import kit_grav as K
from shapely.geometry import Point, Polygon
from shapely.geometry import box as sbox

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
HEIS = "heis_5"
PORT = "port_5"
# Heissjakta: hullet i vinsjhusets gulv (3 × 3 m), sjaktbunnen 14 m under.
HOLE = sbox(78.5, 40.5, 81.5, 43.5)
LIFT_TOP, LIFT_BOTTOM = -3.0, -17.0
# Det morkne lokket i den gamle stollen, over krystallkammeret.
LOKK = sbox(63.3, 39.6, 64.7, 41.0)
DOWN = (13.0, -21.0, 120.4)
MALMHALL = [(4, 32), (26, 30), (36, 34), (36, 46), (20, 52), (4, 46)]
HEISBUNN = [(66, 60), (76, 58), (84, 58), (90, 72), (80, 80), (66, 80)]
AARA = [(38, 54), (52, 54), (58, 58), (58, 72), (36, 72), (36, 62)]
SMELTE = (18.0, 86.0)
OVN = (12.0, 81.0)
LAMP_Y = 2.55  # kroken under overliggeren på stivene

EXTRA = [
    ("sjaktbunn", "Sjaktbunnen", ("rect", 80, 42, 5, 5), LIFT_BOTTOM, 4.0, False),
]

HALLS = [
    ("inngang", "lavehus", DOOR, [(-4, 2), (-11.5, 2)], 2.5, None, None),
    ("inngang", "hovedstoll", HALL, [(0, 3.5), (0, 12)], 3.0, None, None),
    ("hovedstoll", "forseglet", SECRET, [(-2, 26), (-9, 26)], 2.0, None, "hemmelig_5a"),
    ("hovedstoll", "malmhall", HALL, [(0, 30), (0, 38), (4, 38)], 3.0, None, None),
    ("malmhall", "gammelstoll", HALL, [(36, 40), (39, 40)], 2.5, None, None),
    ("gammelstoll", "vinsjhus", DOOR, [(69, 40), (73, 40)], 2.5, None, None),
    ("sjaktbunn", "heisbunn", HALL, [(80, 44.5), (80, 58)], 3.0, None, None),
    ("heisbunn", "krystallaara", HALL, [(66, 66), (58, 66)], 2.5, None, None),
    ("heisbunn", "vognbane", HALL, [(74, 80), (74, 86), (68, 86)], 3.0, None, None),
    ("vognbane", "lagerrom", SECRET, [(40, 88), (40, 93)], 2.0, None, "hemmelig_5c"),
    ("vognbane", "smeltehule", HALL, [(32, 86), (27.6, 86)], 3.0, None, None),
    ("krystallaara", "smeltehule", HALL, [(36, 70), (18, 70), (18, 76.4)], 2.5, None, None),
    ("smeltehule", "gruveport", HALL, [(18, 95.6), (18, 100)], 3.0, None, None),
    ("gruveport", "nedgang", GATE, [(13, 108), (13, 117)], 3.0, (0, 3.0, 7.5), PORT, {"name": "Gruveporten"}),
    ("gruveport", "lavehus", SHORTCUT, [(6, 104), (-26, 104), (-26, 6), (-20.5, 6)], 2.0, (1, 2.0, 96.0), "snarvei_5", {"name": "Luftesjakta"}),
    ("krystallkammer", "krystallaara", SECRET, [(62, 46.5), (62, 50), (46, 50), (46, 54)], 2.0, None, "hemmelig_5b", {"name": "Sprekken bak krystallene"}),
]

ROUND = {
    "smeltehule": dict(dome_rise=3.5, ribs=8, wall_cell=1.5, dome_seg=36, dome_rings=5, wall_mat="wall_dark",
                       torches=[(30, 2.4, 1.4, 9.0, False), (150, 2.4, 1.4, 9.0, False), (250, 2.4, 1.4, 9.0, False)]),
}


def marks(_):
    return [
        {"kind": "start", "pos": [0, 0, 1], "yaw": P},
        {"kind": "up", "pos": [0, 0, -3], "yaw": 0.0},
        {"kind": "rest", "pos": [-16, 0, 7.3], "yaw": P},
        # Hovedstollen og den forseglede stollen.
        {"kind": "mob", "pos": [0, -1, 26], "label": "breadcrumb", "n": 2, "patrol": [[0, -1, 14], [0, -1, 29]]},
        {"kind": "secret_wall", "id": "hemmelig_5a", "pos": [-2, -1, 26], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "draft", "pos": [-1.4, -1, 26], "for": "hemmelig_5a"},
        {"kind": "lore", "pos": [-16.6, -1, 23.0], "yaw": P / 2, "label": "Gruvefogdens regnskap",
         "text": "Tredje uka: seks vogner malm, to mann under raset i den gamle stollen. Malmen er ikke malm. Det er "
                 "jernledd, store som en mann, med et gyllent rutenett stemplet på: delene munkene lot bære ned. De lå "
                 "ikke der i dag der de lå i går. Vi murer igjen stollen og går ned med heisen i stedet."},
        {"kind": "bonus", "pos": [-12.0, -1, 28.6], "yaw": P, "label": "liv"},
        # Malmhallen.
        {"kind": "mob", "pos": [18, -3, 40], "label": "button", "n": 3},
        {"kind": "mob", "pos": [27, -3, 42], "label": "accordion", "n": 2},
        {"kind": "mob", "pos": [22, -3, 47], "label": "tooltip", "n": 2},
        {"kind": "chest", "pos": [12, -3, 43.5], "yaw": P / 2},
        # Den gamle stollen: slideren på skinnene, raset og lokket.
        {"kind": "mob", "pos": [42, -3, 40], "label": "avatar", "patrol": [[41, -3, 40], [60, -3, 40]]},
        {"kind": "trap", "label": "rocks", "id": "ras_5", "pos": [53, -3, 40], "size": [1.2, 1.2], "dir": [1, 0, 0]},
        {"kind": "trapdoor", "id": "lokk_5", "pos": [64, -3, 40.3], "size": [1.4, 1.4], "to": "krystallkammer"},
        {"kind": "hint", "label": "sound", "pos": [64, -3, 40.3], "for": "lokk_5"},
        # Krystallkammeret under lokket, og sprekken videre ned til krystallåra.
        {"kind": "rare", "pos": [66.5, -11, 45.8], "yaw": -P / 2, "label": "Tag"},
        {"kind": "secret_wall", "id": "hemmelig_5b", "pos": [62, -11, 46.5], "yaw": 0.0, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "draft", "pos": [62, -11, 46.0], "for": "hemmelig_5b"},
        # Vinsjhuset: spaken løsner bremsen, heisen går ned sjakta.
        {"kind": "lever", "id": "spak_5", "pos": [76.5, -3, 35.45], "yaw": 0.0, "opens": [HEIS]},
        {"kind": "lift", "id": HEIS, "pos": [80, LIFT_TOP, 42], "yaw": 0.0, "size": [2.8, 2.8], "bottom": [80, LIFT_BOTTOM, 42]},
        {"kind": "mob", "pos": [76, -3, 43], "label": "breadcrumb", "n": 3},
        # Heisbunnen.
        {"kind": "mob", "pos": [78, -17, 70], "label": "avatar", "n": 2, "patrol": [[70, -17, 66], [84, -17, 64], [86, -17, 74], [72, -17, 76]]},
        {"kind": "mob", "pos": [74, -17, 76], "label": "tooltip i bakhold", "n": 3, "ambush": "heisbunn"},
        # Krystallåra.
        {"kind": "mob", "pos": [46, -16, 63], "label": "breadcrumb", "n": 3},
        {"kind": "chest", "pos": [52, -16, 69.5], "yaw": P},
        # Vognbanen og kruttkammeret.
        {"kind": "mob", "pos": [51, -18, 86], "label": "accordion", "n": 2},
        {"kind": "trap", "label": "arrows", "id": "vogn_5", "pos": [60, -18, 86], "size": [1.2, 1.2], "dir": [-1, 0, 0]},
        {"kind": "secret_wall", "id": "hemmelig_5c", "pos": [40, -18, 88], "yaw": 0.0, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "sound", "pos": [40, -18, 87.5], "for": "hemmelig_5c"},
        {"kind": "chest", "pos": [37.0, -18, 96.5], "yaw": P / 2},
        {"kind": "bonus", "pos": [42.5, -18, 98.4], "yaw": P, "label": "utholdenhet"},
        # Smeltehula: formannen med nøkkelen.
        {"kind": "elite", "pos": [17, -19, 84], "yaw": P / 2, "label": "accordion", "opens": [PORT]},
        {"kind": "mob", "pos": [22, -19, 78], "label": "breadcrumb", "n": 2},
        {"kind": "mob", "pos": [19, -19, 90], "label": "button", "n": 2},
        # Gruveporten, snarveien og nedgangen.
        {"kind": "mob", "pos": [13, -20, 104], "label": "avatar", "n": 2},
        {"kind": "gate", "id": PORT, "pos": [13, -20, 109.5], "yaw": 0.0, "width": 2.6},
        {"kind": "shortcut", "id": "snarvei_5", "from": "gruveport", "pos": [6, -20, 104], "yaw": P / 2, "width": 1.6},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    v = rm["vinsjhus"]
    v.holes.append(HOLE)
    v.obstacles.append(HOLE)
    rm["sjaktbunn"].flat = HOLE
    g = rm["gammelstoll"]
    g.holes.append(LOKK)
    rm["krystallkammer"].flat = LOKK
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))


def struts(L, room, axis, c, lo, hi, step=2.0, lamps=4, first=0.5, yaw=None):
    """Tømmerstiver hver `step` meter langs stollen (aksen x eller z, midtlinja
    `c`), med gruvelamper på hver `lamps`. stolpene får kollider og holdes fri
    i navigasjonen."""
    r = L.rooms[room]
    k = 0
    s = lo + first
    while s <= hi - 0.3:
        x, z = (s, c) if axis == "x" else (c, s)
        yw = (P / 2 if axis == "x" else 0.0) if yaw is None else yaw
        L.prop("kit_strut", x, r.y, z, yw)
        L.box((x, r.y + 2.76, z), (3.5, 0.28, 0.28), yw)  # overliggeren
        for sx in (-1, 1):
            px, pz = (x, z + sx * 1.55) if axis == "x" else (x + sx * 1.55, z)
            L.box((px, r.y + 1.45, pz), (0.3, 2.9, 0.3))
            r.obstacles.append(sbox(px - 0.16, pz - 0.16, px + 0.16, pz + 0.16))
        if k % lamps == lamps // 2:
            lx, lz = (x, z - 0.9) if axis == "x" else (x + 0.9, z)
            L.prop("kit_lamp", lx, r.y + LAMP_Y, lz, 0.0)
            L.light((lx, r.y + LAMP_Y - 0.3, lz), (1.0, 0.62, 0.3), 7.0, room, True, 1.3)
        k += 1
        s += step


def rails(L, room, axis, c, lo, hi, y=None):
    """Skinner langs stollen: stykker på 2 m."""
    r = L.rooms[room]
    yy = r.y if y is None else y
    s = lo + 1.0
    while s <= hi - 0.9:
        x, z = (s, c) if axis == "x" else (c, s)
        L.prop("kit_rail", x, yy, z, P / 2 if axis == "x" else 0.0)
        s += 2.0


def cart(L, room, x, z, yaw, y=None):
    r = L.rooms[room]
    yy = r.y if y is None else y
    L.prop("kit_cart", x, yy, z, yaw)
    L.box((x, yy + 0.6, z), (1.0, 1.2, 1.6), yaw)
    along_x = abs(math.sin(yaw)) > 0.7
    r.obstacles.append(sbox(x - 0.8, z - 0.5, x + 0.8, z + 0.5) if along_x else sbox(x - 0.5, z - 0.8, x + 0.5, z + 0.8))


def crystal(L, room, x, z, yaw=0.0, reach=7.0, y=None):
    r = L.rooms[room]
    yy = r.y if y is None else y
    L.prop("kit_crystal", x, yy, z, yaw)
    L.box((x, yy + 0.4, z), (1.1, 0.8, 1.1), yaw)
    r.obstacles.append(Point(x, z).buffer(0.65))
    L.light((x, yy + 1.3, z), B.BLUE, reach, room, False, 1.1)


def features(L):
    mb = L.mb
    rm = L.rooms
    # Heissjakta: murene fra sjaktbunnens tak opp til vinsjhusets gulv.
    L.shaft(HOLE, LIFT_BOTTOM + 4.0, LIFT_TOP)
    # Sjakta under det morkne lokket.
    L.shaft(LOKK, -11 + 3.0, -3.0)
    # Lasteplattformen i malmhallen: en lav murt sokkel med malmrenne, og
    # steinpilarer som holder hvelvet.
    m = rm["malmhall"]
    ramp = sbox(29.0, 36.0, 34.0, 46.0)
    L.block(ramp, m.y, m.y + 0.5, mat="wall_dark", top_mat="floor")
    m.obstacles.append(ramp)
    K.rubble(mb, 31.5, 41.0, m.y + 0.5, r=1.8, h=0.7, seed=7)
    for x, z in ((14, 36), (14, 46), (26, 36), (26, 47)):
        K.pillar(mb, x, z, m.y, L.poly_vault_y(m, x, z) + 0.15, size=1.1, mat="wall_dark")
        m.obstacles.append(sbox(x - 0.75, z - 0.75, x + 0.75, z + 0.75))
        L.box((x, m.y + 4.0, z), (1.3, 8.0, 1.3))
    # Heisbunnen og krystallåra: berget bæres av pilarer.
    h = rm["heisbunn"]
    for x, z in ((74, 66), (82, 70), (74, 74)):
        K.pillar(mb, x, z, h.y, L.poly_vault_y(h, x, z) + 0.15, size=1.0, mat="wall_dark")
        h.obstacles.append(sbox(x - 0.7, z - 0.7, x + 0.7, z + 0.7))
        L.box((x, h.y + 3.0, z), (1.2, 6.0, 1.2))
    a = rm["krystallaara"]
    for x, z in ((44, 60), (50, 66)):
        K.pillar(mb, x, z, a.y, L.poly_vault_y(a, x, z) + 0.15, size=0.9, mat="wall_dark")
        a.obstacles.append(sbox(x - 0.65, z - 0.65, x + 0.65, z + 0.65))
        L.box((x, a.y + 3.0, z), (1.1, 6.0, 1.1))
    # Smelteovnen: en murt kuppelovn med glødende munning og pipe opp i kuppelen.
    s = rm["smeltehule"]
    ox, oz = OVN
    K.cylinder(mb, (ox, oz), 2.2, s.y, s.y + 2.6, "wall_dark", n=14, caps=(False, False), smooth=False)
    K.dome(mb, (ox, oz), 2.2, s.y + 2.6, 1.2, mat="wall_dark", rings=4, seg=14)
    K.cylinder(mb, (ox, oz), 0.7, s.y + 3.6, s.y + s.h + 2.0, "wall_dark", n=10, caps=(False, False), smooth=False)
    # Munningen mot sørøst (mot rommet), med glør.
    ang = math.atan2(SMELTE[1] - oz, SMELTE[0] - ox)
    fx, fz = ox + 2.2 * math.cos(ang), oz + 2.2 * math.sin(ang)
    K.obox(mb, (fx, s.y + 0.3, fz), (1.2, 1.0, 0.3), -ang + P / 2, "fire_torch")
    K.obox(mb, (ox + 1.4 * math.cos(ang), s.y, oz + 1.4 * math.sin(ang)), (1.2, 0.12, 1.6), -ang + P / 2, "rubble")
    s.obstacles.append(Point(ox, oz).buffer(2.4))
    L.box((ox, s.y + 2.0, oz), (4.2, 4.0, 4.2))
    L.light((fx + 0.6 * math.cos(ang), s.y + 1.4, fz + 0.6 * math.sin(ang)), (1.0, 0.42, 0.12), 20.0, "smeltehule", True, 3.6)
    L.light((ox, s.y + 5.0, oz), (1.0, 0.5, 0.2), 14.0, "smeltehule", True, 1.2)
    # Slagghauger.
    for x, z, sd in ((24, 92, 11), (10, 90, 12)):
        K.rubble(mb, x, z, s.y, r=1.6, h=0.6, seed=sd)
        s.obstacles.append(Point(x, z).buffer(1.5))


def dress(L):
    mb = L.mb
    rm = L.rooms
    # Inngangen: kaldt skjær fra døra opp, gruvelamper på veggen.
    L.light((0.0, 2.0, -3.0), B.COLD, 6.0, "inngang", False, 0.8)
    L.wall_torch("inngang", "e", 0.0, 2.3, intensity=1.5)
    L.wall_torch("inngang", "w", 0.0, 2.3, intensity=1.5)
    rails(L, "inngang", "z", 0.0, -3.5, 3.5)
    # Lavehuset: lampetavla (lamper på rad langs nordveggen) og hyller.
    lv = rm["lavehus"]
    for x in (-19.0, -17.5, -16.0, -14.5, -13.0):
        L.prop("kit_lamp", x, lv.y + 2.0, lv.z0 + 0.35, 0.0)
    L.light((-16.0, lv.y + 1.7, lv.z0 + 0.5), (1.0, 0.62, 0.3), 9.0, "lavehus", True, 1.8)
    # Hyller av planker langs vestveggen (unna døra i sør), med kasser.
    for y in (0.5, 1.1, 1.7):
        K.box(mb, (lv.x0, lv.y + y, lv.z0 + 0.4), (lv.x0 + 0.4, lv.y + y + 0.05, lv.z0 + 4.4), "oak", faces=("+y", "-y", "+x", "+z", "-z"))
    for x, z in ((lv.x0 + 0.5, lv.z0 + 1.0), (lv.x0 + 0.5, lv.z0 + 3.6)):
        K.obox(mb, (x, lv.y, z), (0.5, 0.45, 0.5), 0.2, "oak")
    L.box((lv.x0 + 0.3, lv.y + 1.0, lv.z0 + 2.4), (0.6, 2.0, 4.0))
    lv.obstacles.append(sbox(lv.x0, lv.z0, lv.x0 + 0.8, lv.z0 + 4.6))
    # Hovedstollen: stiver, lamper og skinner.
    struts(L, "hovedstoll", "z", 0.0, 12.0, 30.0, first=1.5)
    rails(L, "hovedstoll", "z", 0.0, 12.0, 30.0)
    # Den forseglede stollen: to stiver, lampe.
    fs = rm["forseglet"]
    for x in (-15.5, -11.0):
        L.prop("kit_strut", x, fs.y, 26.0, P / 2)
        L.box((x, fs.y + 2.76, 26.0), (3.5, 0.28, 0.28), P / 2)
        for sz in (-1, 1):
            L.box((x, fs.y + 1.45, 26.0 + sz * 1.55), (0.3, 2.9, 0.3))
            fs.obstacles.append(sbox(x - 0.16, 26.0 + sz * 1.55 - 0.16, x + 0.16, 26.0 + sz * 1.55 + 0.16))
    L.prop("kit_lamp", -13.0, fs.y + LAMP_Y, 25.0, 0.0)
    L.light((-13.0, fs.y + 2.2, 25.0), (1.0, 0.62, 0.3), 6.0, "forseglet", True, 1.2)
    # Malmhallen: skinner tvers gjennom, sidespor med vogner, lamper på pilarene.
    rails(L, "malmhall", "x", 39.0, 5.0, 35.0)
    rails(L, "malmhall", "x", 34.0, 15.0, 27.0)
    cart(L, "malmhall", 20.0, 34.0, P / 2)
    cart(L, "malmhall", 23.5, 34.0, P / 2)
    cart(L, "malmhall", 8.0, 41.5, 0.0)
    for x, z in ((14, 36), (14, 46), (26, 36), (26, 47)):
        L.prop("kit_lamp", x + 0.75, rm["malmhall"].y + 2.6, z, 0.0)
        L.light((x + 0.9, rm["malmhall"].y + 2.3, z), (1.0, 0.62, 0.3), 9.0, "malmhall", True, 1.6)
    L.light((20.0, rm["malmhall"].y + 6.0, 41.0), B.COLD, 16.0, "malmhall", False, 0.5)
    # Den gamle stollen: morkne stiver, raset og lokket.
    struts(L, "gammelstoll", "x", 40.0, 39.0, 69.0, first=1.5, lamps=4)
    rails(L, "gammelstoll", "x", 40.0, 39.0, 69.0)
    g = rm["gammelstoll"]
    K.rubble(mb, 55.5, 38.6, g.y, r=0.9, h=0.5, seed=21, lean=(0.0, -1.0))
    g.obstacles.append(Point(55.5, 38.6).buffer(0.8))
    # Vinsjhuset: fakler, og lyset ned i sjakta.
    L.wall_torch("vinsjhus", "n", 83.0 - 1.5, 2.4, intensity=1.6)
    L.wall_torch("vinsjhus", "s", 74.5, 2.4, intensity=1.6)
    L.light((80.0, LIFT_TOP + 3.0, 42.0), (1.0, 0.62, 0.3), 9.0, "vinsjhus", True, 1.4)
    L.light((80.0, LIFT_BOTTOM + 3.0, 42.0), (1.0, 0.62, 0.3), 7.0, "sjaktbunn", True, 1.2)
    # Krystallkammeret: krystaller rundt, blått lys.
    crystal(L, "krystallkammer", 61.5, 40.5, 0.3)
    crystal(L, "krystallkammer", 66.3, 42.0, 1.2)
    crystal(L, "krystallkammer", 61.6, 45.2, 2.1)
    # Heisbunnen: lagerhyller (kasser), vogner, lamper.
    h = rm["heisbunn"]
    for x, z in ((68.0, 62.5), (69.5, 62.0)):
        K.obox(mb, (x, h.y, z), (1.0, 0.8, 1.0), 0.3, "oak")
        L.box((x, h.y + 0.4, z), (1.0, 0.8, 1.0), 0.3)
        h.obstacles.append(Point(x, z).buffer(0.75))
    cart(L, "heisbunn", 85.5, 66.0, P / 2 + 0.2)
    rails(L, "heisbunn", "z", 80.0, 59.0, 78.0)
    for x, z in ((74.6, 66), (82.6, 70), (74.6, 74)):
        L.prop("kit_lamp", x, h.y + 2.6, z, 0.0)
        L.light((x + 0.2, h.y + 2.3, z), (1.0, 0.62, 0.3), 9.0, "heisbunn", True, 1.6)
    # Krystallåra: krystallene langs bergveggene, kaldt blått lys.
    for x, z, yw in ((39.0, 56.5, 0.4), (55.5, 58.5, 1.1), (56.8, 61.5, 2.0), (52.0, 70.6, 0.7), (38.0, 70.5, 1.6), (37.5, 62.5, 2.5), (47.0, 57.0, 0.2)):
        crystal(L, "krystallaara", x, z, yw, reach=8.0)
    L.light((47.0, rm["krystallaara"].y + 4.0, 63.0), B.BLUE, 16.0, "krystallaara", False, 0.6)
    # Vognbanen: stiver, skinner, den lastede vogna i øst.
    struts(L, "vognbane", "x", 86.0, 32.0, 68.0, first=1.5)
    rails(L, "vognbane", "x", 86.0, 32.0, 68.0)
    cart(L, "vognbane", 36.5, 85.0, P / 2)
    # Kruttkammeret: tønner (kasser) og lampe.
    lg = rm["lagerrom"]
    for x, z in ((38.0, 94.0), (42.5, 94.2), (43.0, 97.5)):
        K.cylinder(mb, (x, z), 0.42, lg.y, lg.y + 0.9, "oak", n=10, caps=(True, False), smooth=True)
        L.box((x, lg.y + 0.45, z), (0.9, 0.9, 0.9))
        lg.obstacles.append(Point(x, z).buffer(0.5))
    L.prop("kit_lamp", 40.0, lg.y + 2.4, 96.0, 0.0)
    L.light((40.0, lg.y + 2.1, 96.0), (1.0, 0.62, 0.3), 7.0, "lagerrom", True, 1.3)
    # Smeltehula: skinnene inn fra vognbanen og mot gruveporten.
    rails(L, "smeltehule", "x", 86.0, 20.0, 27.0)
    rails(L, "smeltehule", "z", 18.0, 88.0, 95.0)
    cart(L, "smeltehule", 23.0, 80.5, 0.4)
    # Gruveporten: fakler, lamper, skinnene som slutter.
    gp = rm["gruveport"]
    L.wall_torch("gruveport", "e", 104.0, 2.4, intensity=1.6)
    L.wall_torch("gruveport", "w", 101.5, 2.4, intensity=1.4)
    rails(L, "gruveport", "z", 18.0, 100.0, 106.0)
    L.light((13.0, gp.y + 2.5, 107.5), B.COLD, 6.0, "gruveport", False, 0.6)
    # Nedgangen.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 3.0)
    L.wall_torch("nedgang", "w", 119.0, 2.4, intensity=1.4)
    # Gangene: fakler i de lange.
    L.torch("inngang-hovedstoll", (1.5 - 0.02, -0.4 + 1.9, 8.0), (-1.0, 0.0), intensity=1.0, reach=7.0)
    L.torch("hovedstoll-malmhall", (-1.5 + 0.02, -2.0 + 1.9, 35.0), (1.0, 0.0), intensity=1.0, reach=7.0)
    L.torch("sjaktbunn-heisbunn", (81.5 - 0.02, LIFT_BOTTOM + 1.9, 51.0), (-1.0, 0.0), intensity=1.0, reach=7.0)
    L.torch("heisbunn-vognbane", (75.5 - 0.02, -17.5 + 1.9, 83.0), (-1.0, 0.0), intensity=1.0, reach=7.0)
    L.torch("krystallaara-smeltehule", (30.0, -17.0 + 1.9, 71.25 - 0.02), (0.0, -1.0), intensity=1.0, reach=7.0)
    L.torch("smeltehule-gruveport", (19.5 - 0.02, -19.5 + 1.9, 97.8), (-1.0, 0.0), intensity=1.0, reach=7.0)
    for z in (20.0, 44.0, 68.0, 92.0):
        y = -20.0 + 20.0 * min(1.0, max(0.0, (102.0 - z) / 94.0))
        L.torch("gruveport-lavehus", (-25.0 - 0.02, y + 1.9, z), (-1.0, 0.0), intensity=1.0, reach=7.0)


LAYOUT = dict(
    theme="gruve",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round=ROUND,
    override={
        "lavehus": {"shape": ("rect", -16, 4, 9, 8)},
        "hovedstoll": {"shape": ("rect", 0, 21, 4, 18), "h": 3.4},
        "malmhall": {"shape": ("poly", MALMHALL), "h": 7.0},
        "vinsjhus": {"h": 6.0},
        "krystallkammer": {"shape": ("rect", 64, 43, 7, 7)},
        "heisbunn": {"shape": ("poly", HEISBUNN), "h": 5.0},
        "krystallaara": {"shape": ("poly", AARA), "h": 5.0},
        "vognbane": {"shape": ("rect", 50, 86, 36, 4), "h": 3.4},
        "smeltehule": {"shape": ("circle", SMELTE[0], SMELTE[1], 10), "h": 6.0},
        "gruveport": {"shape": ("rect", 13, 104, 14, 8), "h": 5.0},
        "nedgang": {"shape": ("rect", 13, 121, 8, 8)},
    },
    wall_mats={r: "wall_dark" for r in ("malmhall", "gammelstoll", "krystallkammer", "sjaktbunn", "heisbunn", "krystallaara", "vognbane", "lagerrom", "smeltehule", "gruveport")},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
)
