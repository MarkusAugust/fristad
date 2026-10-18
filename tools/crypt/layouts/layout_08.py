"""Utlegget for nivå 8, Ridderhallen (planen: `levels/level_08.py`).

Bunnen av temaet «Borgen under jorda»: hjertet i den sunkne borgen. Trappa
fra fangehullet kommer ut i **forhallen** (rustninger og faner) med
**våpenkammeret** (stativer med spyd og skjold, kista, og rustflekkene som
skjuler **smedkammeret**: smedens bok og varig utholdenhet). Veien videre
går gjennom **pendelgangen** (statuer i rekke på begge sider, tre
pendelfeller tvers over gangen; den som vil, går løkka rundt gjennom
**vaktgangen** fra våpenkammeret til kapellet) og ned trappa i **søylesalen**
(30 × 28 m, to rader søyler, hjulkroner i hvelvet og gallerier langs begge
langmurene med bueskyttere, trappene ned langs nordmuren). Vest for salen
ligger **ridderkapellet** med hvilestedet ved alteret, øst for den **Det
runde bord** (rund sal med steinbordet, ridderstatuene rundt, galleri rundt
med vindeltrappa langs muren, og tre statuer som er spaker: de åpner den
falske veggen fra galleriet inn til **skattkammeret** med Avatar). Gangen
sørover fører til **gildehallen** (langbordene, spillemannsgalleriet over
døra med trapp ned langs østmuren, bakholdet under bordene, og lemmen ved
det store karet ned i **vinkjelleren**, med kjellergangen bort til stigen
opp i forværelset). **Forværelset** foran tronsalen har spaken som heiser
gitteret mot søylesalen (snarveien til kapellet) og den tredje pendelen
over døra. **Tronsalen** (26 × 28 m, −4 m) er arenaen: porten slår igjen bak
deg, Navbaren venter mellom søylene med lenkene, tronen står på
forhøyningen i sør, og døra bak tronen åpner seg når den faller:
**nedgangen** med trappa til nivå 9.

Nytt i byggesettet (`kit_castle.py`): `kit_statue` (ridder i hogd stein),
`kit_rack` (våpenstativ), `kit_throne` (tronen) og `kit_chandelier`
(hjulkrona, henger i hvelvet: `MOUNTED_PROPS`).
"""

import math

import build_level as B
import kit_grav as K
from shapely.geometry import Point
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
I = B.INSET
TOP = 0.0  # forhallen, våpenkammeret, smedkammeret, pendelgangen og vaktgangen
HALLY = -2.0  # søylesalen, kapellet, Det runde bord, gildehallen og forværelset
GALS = HALLY + 4.5  # galleriene i søylesalen (planen sa +3; dørbuene under dem er 3,7 m høye)
GALR = 3.0  # galleriet rundt Det runde bord (+5 m) og skattkammeret
GALG = HALLY + 4.5  # spillemannsgalleriet i gildehallen
VIN = -7.0  # vinkjelleren og kjellergangen
THRONE = -4.0  # tronsalen og nedgangen
DAIS = -3.0  # forhøyningen med tronen
RUND = dict(c=(34.0, 46.0), r=10.0)
GITTER, ARENA, ARENA_UT = "gitter_8", "arenaport_8", "arenaport_8ut"
SMED, SKATT = "hemmelig_8a", "hemmelig_8b"
LEM, LEM2 = "lem_8", "lem_8b"
SOYLE = dict(x0=-15.0, x1=15.0, z0=32.0, z1=60.0)
GILDE = dict(x0=21.0, x1=43.0, z0=71.0, z1=85.0)
VINH = 3.4  # vinkjelleren: høy nok til at murene går over hvelvet i kjellergangen
LEMH = sbox(29.3, 82.8, 30.7, 84.2)  # lemmen i gildehallen ned i vinkjelleren
LADH = sbox(3.4, 82.6, 4.6, 83.8)  # hullet i forværelsets gulv over stigen
DOWN = (0.0, THRONE, 129.6)

EXTRA = [
    ("stigebunn", "Under lemmen i forværelset", ("rect", 4, 84, 3, 4), VIN, 3.2, True),
]

HALLS = [
    ("forhall", "vapen", DOOR, [(-6, 0), (-14, 0)], 2.5, None, None),
    ("vapen", "smed", SECRET, [(-28, 1), (-32, 1)], 2.0, None, SMED),
    ("forhall", "pendel", DOOR, [(0, 5), (0, 10)], 3.0, None, None),
    ("pendel", "soylesal", STAIRS, [(0, 26), (0, SOYLE["z0"])], 3.0, (0, 0.75, 5.25), None, {"name": "Trappa ned i søylesalen"}),
    ("vapen", "vakt", DOOR, [(-22, 5), (-22, 7)], 2.0, None, None),
    ("vakt", "kapell", STAIRS, [(-22, 34), (-22, 40)], 2.0, (0, 0.6, 5.4), None, {"name": "Løkka rundt pendlene"}),
    ("soylesal", "kapell", DOOR, [(SOYLE["x0"], 46), (-20, 46)], 3.0, None, None),
    ("soylesal", "rundsal", DOOR, [(SOYLE["x1"], 46), (RUND["c"][0] - RUND["r"] + 0.4, 46)], 3.0, None, None),
    ("rundsal", "skatt", SECRET, [(RUND["c"][0], RUND["c"][1] - RUND["r"] + 0.4), (RUND["c"][0], 30)], 2.0, None, SKATT, {"ya": GALR}),
    ("rundsal", "gilde", HALL, [(RUND["c"][0], RUND["c"][1] + RUND["r"] - 0.4), (RUND["c"][0], GILDE["z0"])], 3.0, None, None, {"name": "Gangen til gildehallen"}),
    ("vin", "stigebunn", HALL, [(24.5, 84.5), (5.5, 84.5)], 2.0, None, None, {"name": "Kjellergangen"}),
    ("gilde", "forvaerelse", DOOR, [(GILDE["x0"], 80), (7, 80)], 3.0, None, None),
    ("forvaerelse", "soylesal", GATE, [(0, 76), (0, SOYLE["z1"])], 3.0, None, GITTER, {"name": "Gitteret"}),
    ("forvaerelse", "tronsal", GATE, [(0, 88), (0, 96)], 4.0, (0, 0.5, 5.0), ARENA, {"name": "Porten til tronsalen"}),
    ("tronsal", "nedgang", GATE, [(0, 124), (0, 128)], 2.5, None, ARENA_UT, {"name": "Døra bak tronen"}),
]

ROUND = {
    "rundsal": dict(
        gallery=dict(y=GALR, ring_in=7.5, stair_in=7.7, stair_a0=math.radians(335), stair_a1=math.radians(435), steps=26, segs=48,
                     id="rundsal_galleri", name="Galleriet rundt Det runde bord", stair_id="rundsal_trapp", stair_name="Trappa opp til galleriet"),
        dome_rise=3.5, ribs=12, wall_cell=1.0, dome_seg=48, dome_rings=6,
        torches=[(120, 2.4, 5.0, 12.0, False), (160, 2.4, 5.0, 12.0, False), (200, 2.4, 5.0, 12.0, False), (240, 2.4, 5.0, 12.0, False),
                 (300, 2.4, 5.0, 12.0, False), (100, 2.2, 4.5, 10.0, True), (180, 2.2, 4.5, 10.0, True), (260, 2.2, 4.5, 10.0, True), (320, 2.2, 4.5, 10.0, True)],
    ),
}


def face(x, z, tx, tz):
    """Dreiningen som vender en ting i (x, z) mot (tx, tz)."""
    return math.atan2(tx - x, tz - z)


def marks(_):
    c = RUND["c"]
    return [
        {"kind": "start", "pos": [0, TOP, 1], "yaw": P},
        {"kind": "up", "pos": [0, TOP, -4], "yaw": 0.0},
        # Forhallen: ridderløftet på muren.
        {"kind": "lore", "pos": [-5.6, TOP, -2.0], "yaw": P / 2, "label": "Ridderløftet",
         "text": "Vi sverget kongen å holde komponentboken ren og porten stengt. Da Kontoret skrev sine regler inn i "
                 "den, holdt vi porten. Da Inkvisisjonen brente mønsterbøkene, holdt vi porten. Da den store "
                 "oppgraderingen kom og alt brakk, holdt vi den fortsatt. Vi holder den ennå. Ingen har sagt oss hva "
                 "den skal holde ute."},
        # Våpenkammeret: skjoldbærerne, kista og rustflekkene som renner ut av fugene.
        {"kind": "mob", "pos": [-17, TOP, 3], "label": "button", "n": 2},
        {"kind": "chest", "pos": [-22, TOP, -4.3], "yaw": 0.0},
        {"kind": "secret_wall", "id": SMED, "pos": [-28, TOP, 1], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [-27.98, TOP, 2.75], "yaw": P / 2, "for": SMED},
        # Smedkammeret.
        {"kind": "lore", "pos": [-39.6, TOP, -1.0], "yaw": P / 2, "label": "Smedens bok",
         "text": "Lenkene til Navbaren smidde jeg av det Rådet ga meg: ett ledd for hver regel, og Kontoret sendte nye "
                 "regler hver uke. Den skulle vise alle veien, men bare den ene. Nå er det lenkene som går, og tronen "
                 "som står."},
        {"kind": "bonus", "pos": [-36, TOP, 4.6], "yaw": P, "label": "utholdenhet"},
        # Pendelgangen: statuene står i features; pendlene er feller.
        {"kind": "trap", "label": "pendulum", "id": "pendel_8a", "pos": [0, TOP, 15], "size": [3.0, 0.8], "dir": [0, 0, 1]},
        {"kind": "trap", "label": "pendulum", "id": "pendel_8b", "pos": [0, TOP, 23], "size": [3.0, 0.8], "dir": [0, 0, 1]},
        # Vaktgangen: patruljen.
        {"kind": "mob", "pos": [-22, TOP, 22], "label": "button", "n": 2, "patrol": [[-22, TOP, 10], [-22, TOP, 31]]},
        # Søylesalen: vaktrunden mellom søylene, bueskytterne på galleriene, Modalen.
        {"kind": "mob", "pos": [0, HALLY, 42], "label": "button", "n": 3, "patrol": [[-3, HALLY, 38], [3, HALLY, 38], [3, HALLY, 56], [-3, HALLY, 56]]},
        {"kind": "mob", "pos": [-13.5, GALS, 50], "label": "breadcrumb-bueskytter", "n": 2},
        {"kind": "mob", "pos": [13.5, GALS, 42], "label": "breadcrumb-bueskytter", "n": 2},
        {"kind": "mob", "pos": [0, HALLY, 54], "label": "modal"},
        # Ridderkapellet: hvilestedet ved alteret.
        {"kind": "rest", "pos": [-29.5, HALLY, 46], "yaw": -P / 2},
        # Det runde bord: kistene rundt bordet, de tre statuene som er spaker, døra fra galleriet.
        {"kind": "mob", "pos": [34, HALLY, 40], "label": "accordion", "n": 3},
        {"kind": "lever", "id": "statue_8a", "pos": [27, HALLY, 41], "yaw": face(27, 41, c[0], c[1]), "opens": [SKATT]},
        {"kind": "lever", "id": "statue_8b", "pos": [41, HALLY, 41], "yaw": face(41, 41, c[0], c[1]), "opens": [SKATT]},
        {"kind": "lever", "id": "statue_8c", "pos": [28.34, HALLY, 51.66], "yaw": face(28.34, 51.66, c[0], c[1]), "opens": [SKATT]},
        {"kind": "secret_wall", "id": SKATT, "pos": [c[0], GALR, c[1] - RUND["r"] + 0.2], "yaw": 0.0, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [c[0] + 1.5, GALR, c[1] - RUND["r"] + 0.18], "yaw": 0.0, "for": SKATT},
        # Skattkammeret.
        {"kind": "rare", "pos": [34, GALR, 26], "yaw": 0.0, "label": "Avatar"},
        {"kind": "chest", "pos": [36.8, GALR, 25.0], "yaw": 0.0},
        # Gildehallen: bakholdet under bordene, spillemennene på galleriet, kista og lemmen.
        {"kind": "mob", "pos": [28, HALLY, 79.5], "label": "accordion i bakhold", "n": 3, "ambush": "gilde"},
        {"kind": "mob", "pos": [32, GALG, 72.5], "label": "breadcrumb-bueskytter", "n": 2},
        {"kind": "mob", "pos": [36, HALLY, 79.5], "label": "button"},
        {"kind": "chest", "pos": [36, HALLY, 84.4], "yaw": P},
        {"kind": "trapdoor", "id": LEM, "pos": [30, HALLY, 83.5], "size": [1.4, 1.4], "to": "vin"},
        {"kind": "hint", "label": "sound", "pos": [30, HALLY, 81.8], "for": LEM},
        # Vinkjelleren og stigen opp i forværelset.
        {"kind": "lore", "pos": [25.0, VIN, 87.8], "yaw": P / 2, "label": "Kjellermesterens regnskap",
         "text": "Tolv fat til inkvisitorenes gilde, fire til kapellet, ett til smeden for lenkene. Ingen til vaktene: "
                 "de drikker ikke lenger. Siste linje, med annen hånd: fatene står. Ingen kommer og henter dem."},
        {"kind": "chest", "pos": [33, VIN, 89.0], "yaw": P},
        {"kind": "trapdoor", "id": LEM2, "label": "bolted", "pos": [4, HALLY, 83.2], "size": [1.2, 1.2], "to": "stigebunn"},
        {"kind": "ladder", "pos": [4, VIN, 82.66], "yaw": 0.0, "top": [4, HALLY, 84.45]},
        # Forværelset: spaken til gitteret, pendelen over døra, vaktene.
        {"kind": "lever", "id": "gitterspak_8", "pos": [6.55, HALLY, 78], "yaw": -P / 2, "opens": [GITTER]},
        {"kind": "gate", "id": GITTER, "pos": [0, HALLY, 74.0], "yaw": 0.0, "width": 2.6},
        {"kind": "trap", "label": "pendulum", "id": "pendel_8c", "pos": [0, HALLY, 77.5], "size": [3.0, 0.8], "dir": [0, 0, 1]},
        {"kind": "mob", "pos": [-3, HALLY, 84], "label": "button", "n": 2},
        # Tronsalen: arenaen med Navbaren.
        {"kind": "gate", "id": ARENA, "pos": [0, THRONE, 94.5], "yaw": 0.0, "width": 3.4, "arena": "tronsal"},
        {"kind": "boss", "pos": [0, THRONE, 112], "yaw": 0.0, "label": "modal", "arena": "tronsal"},
        {"kind": "gate", "id": ARENA_UT, "label": "exit", "pos": [0, THRONE, 126], "yaw": 0.0, "width": 2.2, "arena": "tronsal"},
        {"kind": "down", "pos": list(DOWN), "yaw": P},
    ]


def pre(L):
    rm = L.rooms
    dx, dy, dz = DOWN
    rm["nedgang"].holes.append(sbox(dx - 1.2, dz, dx + 1.2, dz + 3.8))
    rm["gilde"].holes.append(LEMH)
    rm["vin"].flat = LEMH
    rm["forvaerelse"].holes.append(LADH)
    rm["stigebunn"].flat = LADH


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


def statue(L, room, x, z, yaw, y=None):
    """En ridderstatue på sokkel (2,9 m): kollideren og hindringen rundt sokkelen."""
    r = L.rooms[room]
    yy = r.y if y is None else y
    L.prop("kit_statue", x, yy, z, yaw)
    L.box((x, yy + 1.45, z), (1.0, 2.9, 1.0), yaw)
    r.obstacles.append(Point(x, z).buffer(0.75))


def rack(L, room, x, z, yaw):
    """Våpenstativet mot en mur (ryggen mot −z før dreiningen)."""
    r = L.rooms[room]
    L.prop("kit_rack", x, r.y, z, yaw)
    L.box((x, r.y + 1.15, z), (2.2, 2.3, 0.6), yaw)
    r.obstacles.append(Point(x, z).buffer(1.2))


def chandelier(L, room, x, z, intensity=9.0, reach=18.0):
    """Hjulkrona i hvelvet over (x, z): kroken under gjordbuen, hjulet 3,1 m
    under, lyset ved lysene."""
    r = L.rooms[room]
    yh = (B.vault_under(r, x, z) if hasattr(r, "vault") else r.y + r.h) - 0.05
    L.prop("kit_chandelier", x, yh, z, 0.0)
    L.box((x, yh - 2.95, z), (1.6, 0.6, 1.6))
    L.light((x, yh - 2.75, z), B.CANDLE, reach, room, True, intensity)


def pillar(L, room, x, z, size=1.2):
    r = L.rooms[room]
    top = B.pillar_top(r, x, z, size)
    K.pillar(L.mb, x, z, r.y, top, size=size)
    L.box((x, (r.y + top) / 2, z), (size + 0.3, top - r.y, size + 0.3))
    r.obstacles.append(sbox(x - size / 2 - 0.2, z - size / 2 - 0.2, x + size / 2 + 0.2, z + size / 2 + 0.2))


def block_prop(L, room, x, z, size, yaw, mat):
    """En massiv kloss (bord, benk, ambolt) med kollider og hindring."""
    r = L.rooms[room]
    K.obox(L.mb, (x, r.y, z), size, yaw, mat)
    L.box((x, r.y + size[1] / 2, z), size, yaw)
    r.obstacles.append(sbox(x - size[0] / 2, z - size[2] / 2, x + size[0] / 2, z + size[2] / 2).buffer(0.05) if abs(yaw) < 1e-6 else Point(x, z).buffer(max(size[0], size[2]) / 2))


def barrel(L, room, x, z, r_m=0.45, h=1.0, y=None):
    r = L.rooms[room]
    yy = r.y if y is None else y
    K.cylinder(L.mb, (x, z), r_m, yy, yy + h, "oak", n=10, caps=(True, False), smooth=True)
    L.box((x, yy + h / 2, z), (2 * r_m, h, 2 * r_m))
    r.obstacles.append(Point(x, z).buffer(r_m + 0.1))


def features(L):
    mb = L.mb
    rm = L.rooms
    # Sjaktene: lemmen i gildehallen ned i vinkjelleren, og hullet over stigen i forværelset.
    L.shaft(LEMH, VIN + VINH, HALLY)
    L.shaft(LADH, VIN + 3.2, HALLY)
    # Pendelgangen: statuene i rekke på begge sider.
    for z in (13.0, 17.0, 21.0, 25.0):
        statue(L, "pendel", -2.9, z, P / 2)
        statue(L, "pendel", 2.9, z, -P / 2)
    # Søylesalen: galleriene langs langmurene (+4,5 m) på konsoller, med
    # trappene ned langs nordmuren fra hvert galleri; to rader søyler.
    s = rm["soylesal"]
    x0, x1, z0, z1 = SOYLE["x0"], SOYLE["x1"], SOYLE["z0"], SOYLE["z1"]
    _, gw = L.terrace(s, x0, x0 + 3, z0, z1, GALS, "e", gaps=[(z0, z0 + 3)], solid=False)
    _, ge = L.terrace(s, x1 - 3, x1, z0, z1, GALS, "w", gaps=[(z0, z0 + 3)], solid=False)
    L.wall_stair("soylesal", (x0 + 3, z0 + 1.5), (x0 + 12.5, z0 + 1.5), 3.0, GALS, HALLY, HALLY, 1)
    L.wall_stair("soylesal", (x1 - 3, z0 + 1.5), (x1 - 12.5, z0 + 1.5), 3.0, GALS, HALLY, HALLY, -1)
    # Hindringen slutter INSET før trappefoten, så gulvets nett deler kant med det nederste trinnet.
    s.obstacles.append(sbox(x0 + 3, z0, x0 + 12.5 - I, z0 + 3))
    s.obstacles.append(sbox(x1 - 12.5 + I, z0, x1 - 3, z0 + 3))
    jw = sbox(x0 + 2.0, z0 + I, x0 + 3 + I + 0.01, z0 + 3 - 0.26 - I)
    je = sbox(x1 - 3 - I - 0.01, z0 + I, x1 - 2.0, z0 + 3 - 0.26 - I)
    L.decks.append(dict(room="soylesal", of="soylesal", y=GALS, walk=unary_union([gw, jw]), ring=sbox(x0, z0, x0 + 3, z1).buffer(0.3), obstacles=[]))
    L.decks.append(dict(room="soylesal", of="soylesal", y=GALS, walk=unary_union([ge, je]), ring=sbox(x1 - 3, z0, x1, z1).buffer(0.3), obstacles=[]))
    for k in range(13):
        zc = z0 + 4.0 + 1.8 * k
        K.obox(mb, (x0 + 3 - 0.13, GALS + 0.95, zc), (0.26, 0.4, 0.6), 0.0, "dressed")
        K.obox(mb, (x1 - 3 + 0.13, GALS + 0.95, zc), (0.26, 0.4, 0.6), 0.0, "dressed")
    for x in (-6.0, 6.0):
        for z in (38.0, 44.0, 50.0, 56.0):
            pillar(L, "soylesal", x, z, 1.2)
    for z in (40.0, 48.0, 56.0):
        statue(L, "soylesal", x0 + 1.2, z, P / 2)
        statue(L, "soylesal", x1 - 1.2, z, -P / 2)
    chandelier(L, "soylesal", 0.0, 40.0)
    chandelier(L, "soylesal", 0.0, 52.0)
    # Ridderkapellet: alteret mot vestmuren med lysestakene, benkene i to rekker.
    K.altar(mb, -33.2, 46.0, HALLY, yaw=P / 2, w=2.4, d=1.0, h=1.0)
    L.box((-33.2, HALLY + 0.5, 46.0), (1.1, 1.0, 2.5))
    rm["kapell"].obstacles.append(sbox(-34.0, 44.6, -32.5, 47.4))
    for z in (45.2, 46.8):
        K.candlestick(mb, -33.2, z, HALLY + 1.0, h=0.5, arms=2)
    for x in (-25.8, -22.8):
        for z in (42.8, 49.2):
            block_prop(L, "kapell", x, z, (0.45, 0.45, 2.0), 0.0, "oak")
    # Det runde bord: steinbordet på en sokkel midt i salen, statuene rundt
    # under galleriet (de tre spakene er merker).
    c = RUND["c"]
    K.cylinder(mb, c, 0.9, HALLY, HALLY + 0.72, "dressed", n=12, caps=(False, False), smooth=False)
    K.cylinder(mb, c, 2.4, HALLY + 0.72, HALLY + 0.84, "slab", n=24, caps=(True, True), smooth=False)
    L.box((c[0], HALLY + 0.42, c[1]), (4.8, 0.84, 4.8))
    rm["rundsal"].obstacles.append(Point(c).buffer(2.6))
    K.candlestick(mb, c[0], c[1], HALLY + 0.84, h=0.6, arms=3)
    for deg in (110, 160, 200, 240, 290, 310):
        a = math.radians(deg)
        x, z = c[0] + 8.7 * math.cos(a), c[1] + 8.7 * math.sin(a)
        statue(L, "rundsal", x, z, face(x, z, c[0], c[1]))
    # Gildehallen: spillemannsgalleriet over døra (+4,5 m) med trappa ned
    # langs østmuren, langbordene, tønnene og det store karet ved lemmen.
    g = rm["gilde"]
    gx0, gx1, gz0, gz1 = GILDE["x0"], GILDE["x1"], GILDE["z0"], GILDE["z1"]
    _, gg = L.terrace(g, gx0, gx1, gz0, gz0 + 3, GALG, "s", gaps=[(gx1 - 3, gx1)], solid=False)
    L.wall_stair("gilde", (gx1 - 1.5, gz0 + 3), (gx1 - 1.5, gz0 + 11), 3.0, GALG, HALLY, HALLY, 1)
    g.obstacles.append(sbox(gx1 - 3, gz0 + 3, gx1, gz0 + 11 - I))
    jg = sbox(gx1 - 3 + 0.26 + I, gz0 + 2.0, gx1 - I, gz0 + 3 + I + 0.01)
    L.decks.append(dict(room="gilde", of="gilde", y=GALG, walk=unary_union([gg, jg]), ring=sbox(gx0, gz0, gx1, gz0 + 3).buffer(0.3), obstacles=[]))
    for k in range(9):
        xc = gx0 + 1.5 + 1.8 * k
        K.obox(mb, (xc, GALG + 0.95, gz0 + 3 - 0.13), (0.6, 0.4, 0.26), 0.0, "dressed")
    for z in (77.0, 82.0):
        for x in (27.0, 35.0):
            block_prop(L, "gilde", x, z, (6.0, 0.78, 1.0), 0.0, "oak")
            for dz in (-0.9, 0.9):
                block_prop(L, "gilde", x, z + dz, (5.6, 0.42, 0.3), 0.0, "oak")
    for x, z in ((22.2, 75.0), (22.2, 76.2), (23.0, 84.0)):
        barrel(L, "gilde", x, z)
    barrel(L, "gilde", 32.4, 83.6, r_m=0.9, h=1.4)
    chandelier(L, "gilde", 31.0, 79.5, intensity=8.0)
    # Vinkjelleren: tønnene langs murene.
    for x in (26.2, 27.4, 28.6, 31.0, 32.2, 33.4):
        barrel(L, "vin", x, 83.1)
    for x in (27.0, 28.2, 29.4, 30.6):
        barrel(L, "vin", x, 88.9)
    # Smedkammeret: essen mot nordmuren og ambolten på stabben.
    block_prop(L, "smed", -36.5, -2.4, (1.6, 0.9, 1.1), 0.0, "wall")
    K.cylinder(mb, (-36.5, -2.4), 0.35, TOP + 0.9, TOP + 0.98, "fire_torch", n=10, caps=(True, False), smooth=False)
    K.cylinder(mb, (-36.0, 1.0), 0.25, TOP, TOP + 0.6, "oak", n=8, caps=(False, False), smooth=False)
    K.obox(mb, (-36.0, TOP + 0.6, 1.0), (0.7, 0.28, 0.26), 0.0, "iron")
    L.box((-36.0, TOP + 0.45, 1.0), (0.7, 0.9, 0.5))
    rm["smed"].obstacles.append(Point(-36.0, 1.0).buffer(0.55))
    # Forværelset: rustningene ved porten til tronsalen.
    for x in (-3.0, 3.0):
        armor(L, "forvaerelse", x, 87.3, P)
    # Tronsalen: forhøyningen med trappa foran, tronen, søylene med lenkene,
    # fyrfatene og hjulkronene.
    t = rm["tronsal"]
    dais = sbox(-5.0, 117.0, 5.0, 122.0)
    L.block(dais, THRONE, DAIS)
    t.obstacles.append(dais)
    K.steps(mb, (0.0, 115.6), (0.0, 1.0), (-1.0, 0.0), 6.0, 0.0, 1.4, THRONE, DAIS, 5, mat="dressed")
    L.hull([(-3.0, THRONE, 115.6), (3.0, THRONE, 115.6), (3.0, DAIS, 117.0), (-3.0, DAIS, 117.0), (-3.0, THRONE - 0.3, 115.6), (3.0, THRONE - 0.3, 115.6), (3.0, THRONE - 0.3, 117.0), (-3.0, THRONE - 0.3, 117.0)])
    t.obstacles.append(sbox(-3.0, 115.6, 3.0, 117.0))
    L.prop("kit_throne", 0.0, DAIS, 120.3, P)
    L.box((0.0, DAIS + 1.25, 120.3), (1.6, 2.5, 1.4))
    for x in (-7.0, 7.0):
        for z in (100.0, 106.0, 112.0, 118.0):
            pillar(L, "tronsal", x, z, 1.2)
    for z in (100.0, 106.0, 112.0):
        yc = THRONE + 5.5
        K.beam(mb, (-6.3, yc, z), (0.0, yc - 0.9, z), 0.08, 0.08, "iron", up=(0.0, 0.0, 1.0))
        K.beam(mb, (0.0, yc - 0.9, z), (6.3, yc, z), 0.08, 0.08, "iron", up=(0.0, 0.0, 1.0))
    for x in (-6.0, 6.0):
        K.cylinder(mb, (x, 116.0), 0.45, THRONE, THRONE + 1.0, "iron", n=10, caps=(False, False), smooth=False)
        K.cylinder(mb, (x, 116.0), 0.38, THRONE + 0.9, THRONE + 1.0, "fire_torch", n=10, caps=(True, False), smooth=False)
        L.box((x, THRONE + 0.5, 116.0), (1.0, 1.0, 1.0))
        t.obstacles.append(Point(x, 116.0).buffer(0.7))
        L.light((x, THRONE + 1.6, 116.0), (1.0, 0.5, 0.2), 14.0, "tronsal", True, 6.0)
    chandelier(L, "tronsal", 0.0, 103.0, intensity=10.0, reach=20.0)
    chandelier(L, "tronsal", 0.0, 113.0, intensity=10.0, reach=20.0)
    # Nedgangen: hullet for trappa er sperret for nettet.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    nd.obstacles.append(sbox(dx - 1.6, dz, dx + 1.6, nd.z1))


def dress(L):
    rm = L.rooms
    # Forhallen: rustningene og fanene langs murene.
    # Ridderløftet henger mellom rustningene på vestmuren.
    for zw, ze in ((-3.7, -2.0), (3.0, 3.0)):
        armor(L, "forhall", -5.3, zw, P / 2)
        armor(L, "forhall", 5.3, ze, -P / 2)
    banner(L, "forhall", -6 + 0.05, 3.6, 3.0, P / 2)
    banner(L, "forhall", 6 - 0.05, 3.6, 3.0, -P / 2)
    banner(L, "forhall", -3.0, 3.6, 5 - 0.05, P)
    banner(L, "forhall", 3.0, 3.6, 5 - 0.05, P)
    L.wall_torch("forhall", "n", -4.0, 2.6, intensity=5.0, reach=10.0)
    L.wall_torch("forhall", "n", 4.0, 2.6, intensity=5.0, reach=10.0)
    L.wall_torch("forhall", "s", -4.5, 2.6, intensity=5.0, reach=10.0)
    L.wall_torch("forhall", "s", 4.5, 2.6, intensity=5.0, reach=10.0)
    L.light((0.0, TOP + 4.5, 0.0), B.WARM, 16.0, "forhall", True, 12.0)
    # Våpenkammeret: stativene langs nord- og sørmuren, rustningene ved østmuren.
    rack(L, "vapen", -26.0, -4.6, 0.0)
    rack(L, "vapen", -18.0, -4.6, 0.0)
    rack(L, "vapen", -26.0, 4.6, P)
    rack(L, "vapen", -17.0, 4.6, P)
    armor(L, "vapen", -14.7, -2.8, -P / 2)
    armor(L, "vapen", -14.7, 2.8, -P / 2)
    L.wall_torch("vapen", "n", -22.0, 2.5, intensity=5.0, reach=10.0)
    L.wall_torch("vapen", "s", -21.5, 2.5, intensity=5.0, reach=10.0)
    L.wall_torch("vapen", "w", -2.5, 2.5, intensity=5.0, reach=10.0)
    L.light((-21.0, TOP + 3.5, 0.0), B.WARM, 16.0, "vapen", True, 10.0)
    # Smedkammeret: glørne i essen og en fakkel.
    L.light((-36.5, TOP + 1.3, -2.4), (1.0, 0.45, 0.15), 10.0, "smed", True, 6.0)
    L.wall_torch("smed", "e", 3.0, 2.2, intensity=4.5, reach=9.0)
    # Pendelgangen: fakler mellom statuene og lys oppe i hvelvet.
    for z in (15.0, 23.0):
        L.wall_torch("pendel", "w", z, 2.8, intensity=5.0, reach=10.0)
        L.wall_torch("pendel", "e", z, 2.8, intensity=5.0, reach=10.0)
    L.light((0.0, TOP + 5.0, 18.0), B.COLD, 22.0, "pendel", False, 12.0)
    # Vaktgangen.
    for z in (12.0, 22.0, 32.0):
        L.wall_torch("vakt", "w", z, 2.3, intensity=4.5, reach=9.0)
    # Søylesalen: fakler over trappene og på sørmuren, over galleriene, det
    # store kalde lyset i hvelvet og varme fyll-lys mellom søylene.
    x0, x1, z0, z1 = SOYLE["x0"], SOYLE["x1"], SOYLE["z0"], SOYLE["z1"]
    for x in (-8.0, 8.0):
        L.wall_torch("soylesal", "n", x, 5.2, intensity=5.0, reach=12.0)
        L.wall_torch("soylesal", "s", x, 2.6, intensity=5.0, reach=12.0)
    for z in (38.0, 46.0, 54.0):
        L.wall_torch("soylesal", "w", z, GALS - HALLY + 2.0, intensity=5.0, reach=12.0)
        L.wall_torch("soylesal", "e", z, GALS - HALLY + 2.0, intensity=5.0, reach=12.0)
    L.light((0.0, HALLY + 9.0, 46.0), B.COLD, 44.0, "soylesal", False, 26.0)
    L.light((-9.0, HALLY + 4.0, 40.0), (1.0, 0.75, 0.5), 20.0, "soylesal", True, 10.0)
    L.light((9.0, HALLY + 4.0, 52.0), (1.0, 0.75, 0.5), 20.0, "soylesal", True, 10.0)
    for z in (42.0, 50.0):
        banner(L, "soylesal", x0 + 0.05, HALLY + 3.4, z, P / 2)
        banner(L, "soylesal", x1 - 0.05, HALLY + 3.4, z, -P / 2)
    # Ridderkapellet: lysene på alteret, faklene og det kalde lyset under hvelvet.
    L.light((-33.0, HALLY + 1.9, 46.0), B.CANDLE, 12.0, "kapell", True, 6.0)
    L.wall_torch("kapell", "n", -30.0, 2.4, intensity=5.0, reach=10.0)
    L.wall_torch("kapell", "s", -30.0, 2.4, intensity=5.0, reach=10.0)
    L.wall_torch("kapell", "s", -24.0, 2.4, intensity=5.0, reach=10.0)
    L.light((-27.0, HALLY + 4.5, 46.0), (0.55, 0.6, 0.85), 18.0, "kapell", False, 8.0)
    banner(L, "kapell", -34 + 0.05, HALLY + 3.6, 43.5, P / 2)
    banner(L, "kapell", -34 + 0.05, HALLY + 3.6, 48.5, P / 2)
    # Det runde bord: lysene på bordet, kuppelen og galleriet (faklene står i ROUND).
    c = RUND["c"]
    L.light((c[0], HALLY + 2.6, c[1]), B.CANDLE, 16.0, "rundsal", True, 7.0)
    L.light((c[0], HALLY + 8.0, c[1]), B.COLD, 40.0, "rundsal", False, 24.0)
    chandelier(L, "rundsal", c[0], c[1], intensity=10.0, reach=16.0)
    L.light((c[0], GALR + 2.5, c[1]), (0.6, 0.65, 0.85), 16.0, "rundsal_galleri", False, 7.0)
    # Skattkammeret.
    L.light((34.0, GALR + 2.2, 27.0), (1.0, 0.8, 0.45), 9.0, "skatt", True, 4.0)
    # Gildehallen: faklene, det varme lyset over bordene, fanene.
    for x in (26.0, 32.0, 38.0):
        L.wall_torch("gilde", "s", x, 2.5, intensity=5.0, reach=10.0)
    for z in (75.0, 83.5):
        L.wall_torch("gilde", "w", z, 2.5, intensity=5.0, reach=10.0)
    L.wall_torch("gilde", "e", 78.0, GALG - HALLY + 1.5, intensity=5.0, reach=10.0)
    for x in (27.0, 37.0):
        L.wall_torch("gilde", "n", x, GALG - HALLY + 2.0, intensity=5.0, reach=10.0)
    L.light((32.0, HALLY + 5.0, 79.0), B.WARM, 24.0, "gilde", True, 14.0)
    banner(L, "gilde", 26.0, HALLY + 3.4, 85 - 0.05, P)
    banner(L, "gilde", 38.0, HALLY + 3.4, 85 - 0.05, P)
    # Vinkjelleren og stigebunnen.
    L.wall_torch("vin", "e", 86.0, 2.2, intensity=4.5, reach=10.0)
    L.light((30.0, VIN + 2.3, 86.0), (1.0, 0.75, 0.45), 14.0, "vin", True, 5.0)
    L.light((4.0, VIN + 2.6, 84.0), B.COLD, 7.0, "stigebunn", False, 3.0)
    # Forværelset: faklene, fyll-lyset og det røde skjæret fra tronsalen.
    L.wall_torch("forvaerelse", "w", 80.0, 2.5, intensity=5.0, reach=10.0)
    L.wall_torch("forvaerelse", "e", 84.0, 2.5, intensity=5.0, reach=10.0)
    L.light((0.0, HALLY + 4.0, 82.0), B.WARM, 18.0, "forvaerelse", True, 10.0)
    L.light((0.0, HALLY + 2.5, 87.0), (0.9, 0.3, 0.2), 10.0, "forvaerelse", True, 5.0)
    banner(L, "forvaerelse", -7 + 0.05, HALLY + 3.4, 84.0, P / 2)
    banner(L, "forvaerelse", 7 - 0.05, HALLY + 3.4, 84.0, -P / 2)
    # Tronsalen: fakler på langmurene, det kalde lyset i hvelvet og det røde ved tronen.
    for z in (100.0, 110.0, 120.0):
        L.wall_torch("tronsal", "w", z, 3.0, intensity=5.0, reach=12.0)
        L.wall_torch("tronsal", "e", z, 3.0, intensity=5.0, reach=12.0)
    L.light((0.0, THRONE + 9.0, 110.0), B.COLD, 44.0, "tronsal", False, 26.0)
    L.light((0.0, DAIS + 2.5, 119.5), (1.0, 0.35, 0.2), 22.0, "tronsal", True, 14.0)
    for x in (-9.0, 9.0):
        banner(L, "tronsal", x, THRONE + 4.0, 124 - 0.05, P)
    banner(L, "tronsal", -13 + 0.05, THRONE + 4.0, 105.0, P / 2)
    banner(L, "tronsal", 13 - 0.05, THRONE + 4.0, 105.0, -P / 2)
    # Nedgangen.
    nd = rm["nedgang"]
    dx, dy, dz = DOWN
    L.light((dx, nd.y + 0.4, dz + 1.9), B.BLUE, 10.0, "nedgang", False, 6.0)
    L.wall_torch("nedgang", "w", 129.0, 2.4, intensity=4.5, reach=9.0)
    # Gangene.
    L.torch("forhall-vapen", (-10.0, TOP + 1.9, -1.25 + 0.02), (0.0, 1.0), intensity=3.6, reach=8.0)
    L.torch("forhall-pendel", (-1.5 + 0.02, TOP + 1.9, 7.5), (1.0, 0.0), intensity=3.6, reach=8.0)
    L.torch("soylesal-kapell", (-17.5, HALLY + 1.9, 44.5 + 0.02), (0.0, 1.0), intensity=3.6, reach=8.0)
    L.torch("soylesal-rundsal", (19.5, HALLY + 1.9, 44.5 + 0.02), (0.0, 1.0), intensity=3.6, reach=9.0)
    L.torch("rundsal-gilde", (35.5 - 0.02, HALLY + 1.9, 63.0), (-1.0, 0.0), intensity=3.6, reach=9.0)
    L.torch("vin-stigebunn", (15.0, VIN + 1.9, 83.5 + 0.02), (0.0, 1.0), intensity=3.6, reach=9.0)
    L.torch("vin-stigebunn", (9.0, VIN + 1.9, 85.5 - 0.02), (0.0, -1.0), intensity=3.6, reach=9.0)
    L.torch("gilde-forvaerelse", (14.0, HALLY + 1.9, 78.5 + 0.02), (0.0, 1.0), intensity=3.6, reach=9.0)
    L.torch("forvaerelse-soylesal", (-1.5 + 0.02, HALLY + 1.9, 68.0), (1.0, 0.0), intensity=3.6, reach=9.0)
    L.torch("forvaerelse-soylesal", (1.5 - 0.02, HALLY + 1.9, 64.0), (-1.0, 0.0), intensity=3.6, reach=9.0)
    L.light((0.0, THRONE + 2.6, 92.0), (0.9, 0.3, 0.2), 10.0, "forvaerelse-tronsal", True, 5.0)
    L.torch("tronsal-nedgang", (-1.25 + 0.02, THRONE + 1.9, 126.0), (1.0, 0.0), intensity=3.6, reach=8.0)


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
        "forhall": {"shape": ("rect", 0, 0, 12, 10), "y": TOP, "h": 7.0},
        "vapen": {"shape": ("rect", -21, 0, 14, 10), "y": TOP, "h": 5.0},
        "smed": {"shape": ("rect", -36, 1, 8, 8), "y": TOP, "h": 3.5},
        "pendel": {"shape": ("rect", 0, 18, 7, 16), "y": TOP, "h": 9.0},
        "vakt": {"shape": ("rect", -22, 20.5, 6, 27), "y": TOP, "h": 3.5},
        "soylesal": {"shape": ("rect", 0, 46, 30, 28), "y": HALLY, "h": 12.0},
        "kapell": {"shape": ("rect", -27, 46, 14, 12), "y": HALLY, "h": 9.0},
        "rundsal": {"shape": ("circle", RUND["c"][0], RUND["c"][1], RUND["r"]), "y": HALLY, "h": 12.0},
        "skatt": {"shape": ("rect", 34, 27, 8, 6), "y": GALR, "h": 3.0},
        "gilde": {"shape": ("rect", 32, 78, 22, 14), "y": HALLY, "h": 8.0},
        "vin": {"shape": ("rect", 30, 86, 11, 7), "y": VIN, "h": VINH},
        "forvaerelse": {"shape": ("rect", 0, 82, 14, 12), "y": HALLY, "h": 6.0},
        "tronsal": {"shape": ("rect", 0, 110, 26, 28), "y": THRONE, "h": 16.0},
        "nedgang": {"shape": ("rect", 0, 131, 8, 6), "y": THRONE, "h": 5.0},
    },
    wall_mats={"smed": "wall_dark", "vin": "wall_dark", "stigebunn": "wall_dark"},
    vault_mats={"vin": "rock", "stigebunn": "rock", "smed": "rock"},
    cells={"soylesal": 2.0, "tronsal": 2.0, "gilde": 2.0},
    hall_vault_n=8,
    hall_flat_step=2.0,
    nav_grid=4.0,
    round_seam_free=True,
)
