"""Utlegget for nivå 10, Kjernen (planen: `levels/level_10.py`).

En trakt av plast. Trappa fra nivå 9 ender på **Kanten**, og derfra ser du
rett ned: tre terrasser skrur seg nedover rundt **Rotunden**, som lyser
hvitt i bunnen. Den ytre spiralen går nesten en hel runde med klokka (øst,
sør, vest) fra 0 til −11,6 m; broa i nord fører over til **Den midtre
spiralen** (øst, ned til −16,8 m), og broa i sør ned til **Den indre
spiralen** (vest, ned til −26 m). Fra enden av den indre går den brede
trappa ned gjennom porten i rotundemuren, der Rammeverket venter. Porten
slår igjen bak deg til det faller. Ingen trapp ned: dette er bunnen.

Sidegrenene: **Det glemte forrådet** bak veggen som henger igjen (øst på
den ytre), **Stabelkammeret** med bakholdet (vest), fallemmen i det
sprukne gulvet ned til **Urskriften** (under den ytre spiralen, stigen
opp igjen), trappebroa ned til **Sammenføyningen** (den vestre halvdelen
av den midtre ringen) og sømsperren i sør, som slås opp fra den midtre
spiralen (løkka). De tre sømknutene på den midtre spiralen heiser gitteret
på broa inn til **Knutekammeret** (den østre halvdelen av den indre
ringen). Hvilestedet, **Det siste alteret**, står i enden av den indre
spiralen, like ved trappa ned i rotunden.

Trakten er vertsrommet (et rundt rom med bunnen i avgrunnen, `floor_y`);
terrassene, rotunden og alteret er rom inne i den (`virtual`) på murte
blokker av plast fra bunnen. Avsatsene er flate, og mellom dem går
trappene rundt i buen (trinn på 0,2 m). Faller du utfor, løfter økta deg
opp der du sist sto (bunnen ligger mer enn 3 m under det laveste rommet).

Avvik fra planen (`docs/wip/krypten.md`): lysheisen er ikke laget, så
hvilestedet står nederst ved rotunden i stedet for ved starten; rommene med
moderne navn har fått middelalderske (Cachen → Stabelkammeret, Det glemte
bufferet → Det glemte forrådet, Kildekoden → Urskriften, Kompilatoren →
Sammenføyningen, Nodekammeret → Knutekammeret); Carousel er ikke en
komponent i spillet, så den sjeldne kista har Search. Fiendene er de som
finnes (Card → button, Tabs → accordion, Skeleton loader → breadcrumb,
Popover → tooltip, Grid → table).
"""

import math

import build_level as B
import kit_grav as K
from kit_grav import Cyl, Horizontal, Plane, grid_face
from shapely.geometry import Point, Polygon
from shapely.geometry import box as sbox
from shapely.ops import unary_union

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET = B.DOOR, B.HALL, B.STAIRS, B.GATE, B.SHORTCUT, B.SECRET
P = math.pi
I = B.INSET
C = (0.0, 46.0)  # midten av trakten og rotunden
HOST_R = 41.5  # traktmuren
BOTTOM = -38.0  # bunnen av avgrunnen
HOST_Y = -31.0  # traktens gulv i json (bunnen ligger lenger nede, så du løftes opp om du faller)
HOST_TOP = 6.0  # vederlaget til kuppelen over trakten
RAIL = 0.26  # tykkelsen på rekkverket
ROT_R, ROT_W = 11.0, 0.5  # rotunden: innsida av muren og tykkelsen
ROT_Y, ROT_TOP = -28.5, -23.5
VIOLET = (0.75, 0.4, 1.0)
WHITE = (0.85, 0.92, 1.0)
CAP = "magic_seam"  # dekkhella på rekkverkene lyser og tegner spiralen i mørket
ARENA, KNUTE, SPERRE, FORRAD = "arenaport_10", "knutegitter_10", "somsperren_10", "forradet_10"


def A(d):
    """Vinkelen i xz (radianer, som i byggesettet) for d grader med klokka fra nord."""
    return math.radians(d - 90.0)


def pt(r, d):
    a = A(d)
    return (C[0] + r * math.cos(a), C[1] + r * math.sin(a))


def arcpts(r, d0, d1, step=2.0):
    n = max(1, math.ceil(abs(d1 - d0) / step - 1e-9))
    return [pt(r, d0 + (d1 - d0) * k / n) for k in range(n + 1)]


def sector(r0, r1, d0, d1, step=2.0):
    return Polygon(arcpts(r1, d0, d1, step) + arcpts(r0, d1, d0, step))


def radial(d):
    a = A(d)
    return (math.cos(a), math.sin(a))


def tangent(d):
    a = A(d)
    return (-math.sin(a), math.cos(a))


def back(d):
    """Retningen bakover langs terrassen (mot lavere vinkel)."""
    t = tangent(d)
    return (-t[0], -t[1])


def yaw_of(v):
    return math.atan2(v[0], v[1])


def at(r, d, y):
    x, z = pt(r, d)
    return [round(x, 3), y, round(z, 3)]


# Terrassene: (fra, til, høyde ved start, høyde ved slutt, rom), i grader
# med klokka fra nord (over 360 i den midtre, som går forbi nord). Like
# høyder er avsatser, ulike er trapper med trinn på 0,2 m langs buen.
RINGS = {
    "ytre": dict(r0=35.0, r1=HOST_R, wall=True, open_ends=(), segs=[
        (-4, 24, 0.0, 0.0, "ytre_o"), (24, 34, 0.0, -2.0, "ytre_o"), (34, 80, -2.0, -2.0, "ytre_o"),
        (80, 88, -2.0, -3.6, "ytre_o"), (88, 112, -3.6, -3.6, "ytre_o"), (112, 122, -3.6, -5.6, "ytre_o"),
        (122, 180, -5.6, -5.6, "ytre_o"),
        (180, 200, -5.6, -5.6, "ytre_v"), (200, 210, -5.6, -7.6, "ytre_v"), (210, 240, -7.6, -7.6, "ytre_v"),
        (240, 248, -7.6, -9.2, "ytre_v"), (248, 300, -9.2, -9.2, "ytre_v"), (300, 312, -9.2, -11.6, "ytre_v"),
        (312, 346, -11.6, -11.6, "ytre_v")]),
    "midt": dict(r0=25.0, r1=31.0, wall=False, open_ends=("end",), segs=[
        (330, 372, -11.6, -11.6, "midt"), (372, 384, -11.6, -13.6, "midt"), (384, 430, -13.6, -13.6, "midt"),
        (430, 440, -13.6, -15.2, "midt"), (440, 478, -15.2, -15.2, "midt"), (478, 488, -15.2, -16.8, "midt"),
        (488, 548, -16.8, -16.8, "midt")]),
    "komp": dict(r0=25.0, r1=31.0, wall=False, open_ends=("start",), segs=[
        (556, 590, -16.8, -16.8, "komp"), (590, 604, -16.8, -14.0, "komp"), (604, 628, -14.0, -14.0, "komp"),
        (628, 642, -14.0, -11.2, "komp"), (642, 682, -11.2, -11.2, "komp")]),
    "indre": dict(r0=16.0, r1=21.0, wall=False, open_ends=(), segs=[
        (168, 196, -18.8, -18.8, "indre"), (196, 210, -18.8, -21.2, "indre"), (210, 250, -21.2, -21.2, "indre"),
        (250, 264, -21.2, -23.6, "indre"), (264, 300, -23.6, -23.6, "indre"), (300, 314, -23.6, -26.0, "indre"),
        (314, 344, -26.0, -26.0, "indre"), (344, 358, -26.0, -26.0, "hvile")]),
    "node": dict(r0=16.0, r1=21.0, wall=False, open_ends=(), segs=[(30, 150, -15.2, -15.2, "node")]),
}

# Broene over gapene mellom terrassene: radielle (vinkel d, fra radius ra i
# høyden ya til rb i yb), med trapp når høydene er ulike.
BRIDGES = [
    dict(d=336, ra=35.0, rb=31.0, ya=-11.6, yb=-11.6, w=3.0, room="midt", name="Broa i nord"),
    dict(d=534, ra=25.0, rb=21.0, ya=-16.8, yb=-18.8, w=3.0, room="indre", name="Trappebroa i sør"),
    dict(d=290, ra=35.0, rb=31.0, ya=-9.2, yb=-11.2, w=2.6, room="komp", name="Trappebroa til Sammenføyningen"),
    dict(d=460, ra=25.0, rb=21.0, ya=-15.2, yb=-15.2, w=2.6, room="node", gate=KNUTE, name="Knutebroa"),
    dict(d=341, ra=16.0, rb=12.0, ya=-26.0, yb=ROT_Y, w=3.4, room="rotunde", ext_b=1.4, rail_b=0.5, gate_from=ROT_R + ROT_W + 0.05, gate=ARENA,
         name="Trappa ned i rotunden"),
]
# Sømsperren: en buet bro i sør fra enden av den midtre spiralen til
# Sammenføyningen, med gitteret midt på.
SPERRE_ARC = dict(d0=548, d1=556, r0=26.0, r1=30.0, y=-16.8)

# Urskriften: rommet inne i blokka under den ytre spiralen, med fallemmen
# og stigen fra avsatsen ved 226°.
KILDE_C = pt(38.0, 226)
KILDE_Y = -13.6
TRAP_HOLE = sbox(KILDE_C[0] + 0.3, KILDE_C[1] - 1.5, KILDE_C[0] + 1.5, KILDE_C[1] - 0.3)
LADDER_HOLE = sbox(KILDE_C[0] - 1.5, KILDE_C[1] + 0.3, KILDE_C[0] - 0.3, KILDE_C[1] + 1.5)


def ring_sector(name):
    ring = RINGS[name]
    return sector(ring["r0"], ring["r1"], ring["segs"][0][0], ring["segs"][-1][1])


def room_sector(rid):
    polys = []
    for ring in RINGS.values():
        for d0, d1, y0, y1, r in ring["segs"]:
            if r == rid:
                polys.append(sector(ring["r0"], ring["r1"], d0, d1))
    return unary_union(polys)


def room_levels(rid):
    ys = [y for ring in RINGS.values() for d0, d1, y0, y1, r in ring["segs"] if r == rid for y in (y0, y1)]
    return min(ys), max(ys)


def shape_of(rid):
    poly = room_sector(rid).simplify(0.01)
    return ("poly", [(round(x, 3), round(z, 3)) for x, z in list(poly.exterior.coords)[:-1]])


EXTRA = [
    ("trakten", "Trakten", ("circle", C[0], C[1], HOST_R), HOST_Y, HOST_TOP - HOST_Y, False),
]

HALLS = [
    ("port", "ytre_o", DOOR, [(0, 2), (0, C[1] - HOST_R + 0.4)], 4.0, None, None, {"yb": 0.0, "name": "Utsikten"}),
    ("ytre_o", "buffer", SECRET, [(HOST_R - 0.4, 46), (45, 46)], 2.0, None, FORRAD, {"ya": -3.6, "name": "Veggen som henger igjen"}),
    ("ytre_v", "cache", DOOR, [(-HOST_R + 0.4, 46), (-45, 46)], 2.5, None, None, {"ya": -9.2}),
]


def marks(_):
    b6 = BRIDGES[4]
    gate6 = pt(ROT_R + ROT_W / 2, b6["d"])
    b5 = BRIDGES[3]
    gate5 = pt(23.0, b5["d"])
    kx, kz = KILDE_C
    tc = TRAP_HOLE.centroid
    lc = LADDER_HOLE.centroid
    rest = pt(18.5, 352)
    boss_yaw = yaw_of(radial(b6["d"]))
    return [
        {"kind": "start", "pos": [0, 0, -1.0], "yaw": 0.0},
        {"kind": "up", "pos": [0, 0, -5], "yaw": 0.0},
        # Den ytre spiralen, øst: rutenettene patruljerer avsatsene under kanten.
        {"kind": "mob", "pos": at(38, 12, 0.0), "label": "table", "n": 2,
         "patrol": [at(38, 6, 0.0), at(38, 20, 0.0), at(38, 50, -2.0), at(38, 74, -2.0)]},
        {"kind": "mob", "pos": at(38, 100, -3.6), "label": "button", "n": 2},
        {"kind": "trap", "label": "spikes", "id": "sommene_10", "pos": at(37.8, 140, -5.6), "size": [1.2, 1.2], "dir": [0, 0, 1]},
        # Det glemte forrådet: veggen som henger igjen.
        {"kind": "secret_wall", "id": FORRAD, "pos": [HOST_R - 0.4, -3.6, 46], "yaw": P / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [round(math.sqrt(HOST_R ** 2 - 2.0 ** 2) - 0.05, 3), -3.6, 48.0], "yaw": -P / 2, "for": FORRAD},
        {"kind": "rare", "pos": [51.5, -3.6, 46.0], "yaw": -P / 2, "label": "Search"},
        {"kind": "lore", "pos": [52.4, -3.6, 43.4], "yaw": -P / 2, "label": "Lappen i forrådet",
         "text": "Her la de det som ble til overs når boka skulle skrives om: sider som ikke passet i "
                 "rammen, mønstre ingen regel hadde plass til. Veggen henger igjen fordi ingen orket å "
                 "rive den. Ta det som ligger her. Det var aldri feil, bare glemt."},
        # Den ytre spiralen, vest.
        {"kind": "mob", "pos": at(38, 195, -5.6), "label": "breadcrumb", "n": 2},
        {"kind": "trapdoor", "id": "fallem_10", "pos": [round(tc.x, 3), -7.6, round(tc.y, 3)], "size": [1.2, 1.2], "to": "kilde"},
        {"kind": "ladder", "pos": [round(lc.x, 3), KILDE_Y, round(lc.y - 0.54, 3)], "yaw": 0.0, "top": [round(lc.x, 3), -7.6, round(lc.y + 1.25, 3)]},
        {"kind": "mob", "pos": at(38, 285, -9.2), "label": "breadcrumb", "n": 3,
         "patrol": [at(38, 256, -9.2), at(38, 296, -9.2), at(38, 330, -11.6)]},
        # Stabelkammeret: bakholdet.
        {"kind": "mob", "pos": [-50, -9.2, 46], "label": "breadcrumb i bakhold", "n": 3, "ambush": "cache"},
        {"kind": "chest", "pos": [-53.6, -9.2, 48.4], "yaw": P / 2},
        # Urskriften.
        {"kind": "lore", "pos": [round(kx + 1.55, 3), KILDE_Y, round(kz + 1.1, 3)], "yaw": -P / 2, "label": "Urskriften",
         "text": "Veggene her er skrevet fulle, lag på lag, Kontorets hånd over munkenes. Øverst står "
                 "kongens regler for hvordan en knapp skal være, og under dem de første sidene av "
                 "mønsterboka, skrapt ut, men ikke borte. Rammeverket ble ikke født. Det ble skrevet, "
                 "regel for regel, av et råd som aldri hadde holdt en penn i et skriptorium."},
        {"kind": "bonus", "pos": [round(kx - 1.4, 3), KILDE_Y, round(kz - 1.3, 3)], "yaw": P / 2, "label": "liv"},
        # Sammenføyningen: samlebåndet der rutenett og kort settes sammen.
        {"kind": "mob", "pos": at(26.8, 300 + 360, -11.2), "label": "table", "n": 2,
         "patrol": [at(26.8, 650, -11.2), at(26.8, 676, -11.2), at(28, 615, -14.0), at(28, 570, -16.8)]},
        {"kind": "mob", "pos": at(28, 612, -14.0), "label": "accordion", "n": 2},
        {"kind": "chest", "pos": at(28.0, 678, -11.2), "yaw": yaw_of(back(678))},
        # Den midtre spiralen: tooltips ved broa, glitchflisene, de tre
        # sømknutene som heiser gitteret til Knutekammeret.
        {"kind": "mob", "pos": at(28, 362, -11.6), "label": "tooltip", "n": 2},
        {"kind": "trap", "label": "spikes", "id": "glitchflisene_10", "pos": at(28, 405, -13.6), "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "lever", "id": "knute_10a", "pos": at(25.7, 420, -13.6), "yaw": yaw_of(radial(420)), "opens": [KNUTE]},
        {"kind": "lever", "id": "knute_10b", "pos": at(25.7, 470, -15.2), "yaw": yaw_of(radial(470)), "opens": [KNUTE]},
        {"kind": "lever", "id": "knute_10c", "pos": at(25.7, 510, -16.8), "yaw": yaw_of(radial(510)), "opens": [KNUTE]},
        {"kind": "gate", "id": KNUTE, "pos": [round(gate5[0], 3), -15.2, round(gate5[1], 3)], "yaw": yaw_of(radial(b5["d"] + 180)), "width": 2.0},
        {"kind": "mob", "pos": at(28, 500, -16.8), "label": "accordion", "n": 2},
        {"kind": "mob", "pos": at(28, 525, -16.8), "label": "modal"},
        {"kind": "lever", "id": "sperrespak_10", "pos": at(25.7, 543, -16.8), "yaw": yaw_of(radial(543)), "opens": [SPERRE]},
        {"kind": "gate", "id": SPERRE, "pos": at(28.0, 552, -16.8), "yaw": yaw_of(tangent(552)), "width": 3.4},
        # Knutekammeret.
        {"kind": "bonus", "pos": at(18.5, 70, -15.2), "yaw": yaw_of(tangent(70)), "label": "utholdenhet"},
        {"kind": "chest", "pos": at(18.5, 115, -15.2), "yaw": yaw_of(back(115))},
        {"kind": "lore", "pos": at(18.5, 140, -15.2), "yaw": yaw_of(back(140)), "label": "Knutene",
         "text": "Tre knuter holder sømmen sammen, og sømmen holder hjertet. Munkene slo dem løs én etter "
                 "én for å komme hit inn og se hva det var de hadde lenket. Det de så, skrev de ikke ned."},
        # Den indre spiralen.
        {"kind": "mob", "pos": at(18.5, 240, -21.2), "label": "button", "n": 2},
        {"kind": "mob", "pos": at(18.5, 285, -23.6), "label": "tooltip", "n": 2},
        # Det siste alteret, i enden av den indre spiralen.
        {"kind": "rest", "pos": [round(rest[0], 3), -26.0, round(rest[1], 3)], "yaw": yaw_of(back(352))},
        # Rotunden: porten slår igjen bak deg til Rammeverket faller.
        {"kind": "gate", "id": ARENA, "pos": [round(gate6[0], 3), ROT_Y, round(gate6[1], 3)], "yaw": yaw_of(radial(b6["d"] + 180)), "width": 3.0, "arena": "rotunde"},
        {"kind": "boss", "pos": [C[0], ROT_Y, C[1] + 1.0], "yaw": boss_yaw, "label": "Rammeverket", "arena": "rotunde"},
        {"kind": "lore", "pos": at(ROT_R - 1.2, 160, ROT_Y), "yaw": yaw_of(radial(160 + 180)), "label": "Mester Ansgars siste side",
         "text": "Jeg har båret de siste sidene så langt ned det går. Over meg murer de igjen, og jeg hører "
                 "Gorveks menn gå. Hjertet ligger her og banker i takt med noe jeg ikke ser, og det samler "
                 "delene sine, én etter én. Jeg kan ikke stanse det. Jeg kan bare skrive. Mønsterboka var "
                 "aldri sterk. Den var fri, og det var nok. Du som finner dette: les sidene, bygg av dem det "
                 "du trenger, og la ingen spenne dem fast igjen. Jeg legger meg ned ved siste side. Ansgar."},
    ]


def pre(L):
    rm = L.rooms
    t = rm["trakten"]
    t.floor_y = BOTTOM
    t.nowalk = True
    rm["kilde"].flat = unary_union([TRAP_HOLE, LADDER_HOLE])
    for rid in ("port", "buffer", "cache"):
        rm[rid].floor_mat = "plastic"


def seam_cyl(mb, r, d, y0, y1, inward, w=0.09):
    """En lysende søm loddrett på en buet flate (terrassene, traktmuren)."""
    h = w / 2 / r
    a = A(d)
    off = -0.03 if inward else 0.03
    rr = r + off
    p = lambda ang, y: (C[0] + rr * math.cos(ang), y, C[1] + rr * math.sin(ang))
    n = (-math.cos(a), 0.0, -math.sin(a)) if inward else (math.cos(a), 0.0, math.sin(a))
    mb.quad(p(a - h, y0), p(a + h, y0), p(a + h, y1), p(a - h, y1), "magic_seam", facing=n)


def profile(segs, r, d_start, stair_top=True):
    """Toppen av en terrasse i flatens (u, v): avsatsene flate, trappene i trinn."""
    u = lambda d: math.radians(d - d_start) * r
    pts = []
    for d0, d1, y0, y1, _ in segs:
        if abs(y1 - y0) < 1e-6:
            pts += [(u(d0), y0), (u(d1), y0)]
        else:
            n = max(1, round(abs(y1 - y0) / 0.2))
            dd, rise = (d1 - d0) / n, (y1 - y0) / n
            for k in range(n):
                yk = y0 + (k + 1) * rise
                pts += [(u(d0 + k * dd), y0 + k * rise), (u(d0 + k * dd), yk), (u(d0 + (k + 1) * dd), yk)]
    out = [pts[0]]
    for p in pts[1:]:
        if abs(p[0] - out[-1][0]) > 1e-6 or abs(p[1] - out[-1][1]) > 1e-6:
            out.append(p)
    return out


def build_ring(L, name, cut, holes, hollow, gaps, walks, tris, landings):
    """En terrasse: blokka fra bunnen, avsatsene og trappene oppå,
    rekkverk mot gapene, kolliderne og navigasjonen."""
    mb = L.mb
    ring = RINGS[name]
    r0, r1, segs = ring["r0"], ring["r1"], ring["segs"]
    ds, de = segs[0][0], segs[-1][1]
    tan_pad = lambda r: math.degrees((RAIL + I) / r)
    qi = r0 + RAIL + I
    qo = r1 - I if ring["wall"] else r1 - RAIL - I
    hole_all = unary_union(holes) if holes else None
    for d0, d1, y0, y1, rid in segs:
        if abs(y1 - y0) < 1e-6:
            landings.append((rid, y0, d0, sector(r0, r1, d0, d1)))
            top = sector(r0, r1, d0, d1).difference(cut)
            if hole_all is not None:
                top = top.difference(hole_all)
            grid_face(mb, Horizontal(y0), top, "plastic")
            # Kolliderne: hele blokka i biter på høyst 8°, eller bare et
            # gulv over et hulrom (Urskriften under fallemmen).
            m = max(1, math.ceil((d1 - d0) / 8.0))
            for k in range(m):
                a0, a1 = d0 + (d1 - d0) * k / m, d0 + (d1 - d0) * (k + 1) / m
                piece = sector(r0, r1, a0, a1)
                if hollow is not None and piece.intersects(hollow):
                    floor = piece.difference(hole_all) if hole_all is not None else piece
                    for q in K._pieces(floor):
                        L.floor_collider(q, y0)
                    for rr, out in ((r0, 1.0),) + (((r1, -1.0),) if not ring["wall"] else ()):
                        for p0, p1 in zip(arcpts(rr, a0, a1), arcpts(rr, a0, a1)[1:]):
                            mid = radial((a0 + a1) / 2)
                            L.wall_box(p0, p1, BOTTOM - 0.3, y0 - 0.5, (mid[0] * out, mid[1] * out))
                    continue
                pts = list(piece.exterior.coords)[:-1]
                L.hull([(x, y0, z) for x, z in pts] + [(x, BOTTOM - 0.3, z) for x, z in pts])
            # Navigasjonen på avsatsen.
            w0 = d0 + (tan_pad(qi) if d0 == ds and "start" not in ring["open_ends"] else 0.0)
            w1 = d1 - (tan_pad(qi) if d1 == de and "end" not in ring["open_ends"] else 0.0)
            walk = sector(qi, qo, w0, w1, step=1.0)
            if hole_all is not None:
                walk = walk.difference(hole_all.buffer(I, join_style="mitre"))
            walks.setdefault((rid, y0), []).append(walk)
        else:
            n = max(1, round(abs(y1 - y0) / 0.2))
            dd, rise = (d1 - d0) / n, (y1 - y0) / n
            for k in range(n):
                aa, ab = d0 + k * dd, d0 + (k + 1) * dd
                yk = y0 + (k + 1) * rise
                tread = sector(r0, r1, aa, ab, step=1.0)
                grid_face(mb, Horizontal(yk), tread, "plastic")
                # Opptrinnet, synlig fra den lave sida.
                lo, hi = min(y0 + k * rise, yk), max(y0 + k * rise, yk)
                tg = tangent(aa)
                f = (tg[0], 0.0, tg[1]) if rise < 0 else (-tg[0], 0.0, -tg[1])
                pa, pb = pt(r0, aa), pt(r1, aa)
                mb.quad((pa[0], lo, pa[1]), (pb[0], lo, pb[1]), (pb[0], hi, pb[1]), (pa[0], hi, pa[1]), "plastic", facing=f)
                pts = list(tread.exterior.coords)[:-1]
                L.hull([(x, yk, z) for x, z in pts] + [(x, BOTTOM - 0.3, z) for x, z in pts])
                # To trekanter per trinn i navigasjonen (en rampe over trinnene).
                ya, yb = y0 + k * rise, yk
                A_, B_, C_, D_ = pt(qi, aa), pt(qo, aa), pt(qo, ab), pt(qi, ab)
                tris.append(([(A_[0], ya, A_[1]), (B_[0], ya, B_[1]), (C_[0], yb, C_[1])], rid))
                tris.append(([(A_[0], ya, A_[1]), (C_[0], yb, C_[1]), (D_[0], yb, D_[1])], rid))
    # Flatene mot gapene (og mot traktmuren er det ingen): innsida, og
    # utsida der det er et gap.
    faces = [(r0, True)] + ([] if ring["wall"] else [(r1, False)])
    for rr, inward in faces:
        prof = profile(segs, rr, ds)
        u_end = math.radians(de - ds) * rr
        shape = Polygon(prof + [(u_end, BOTTOM), (0.0, BOTTOM)]).buffer(0)
        grid_face(mb, Cyl(C, rr, a0=A(ds), inward=inward), shape, "plastic", cell=2.0)
    # Endene: loddrette flater fra bunnen opp til avsatsen.
    for d, y, sgn, key in ((ds, segs[0][2], -1.0, "start"), (de, segs[-1][3], 1.0, "end")):
        tg = tangent(d)
        rd = radial(d)
        surf = Plane(pt(r0, d), rd, (tg[0] * sgn, tg[1] * sgn))
        grid_face(mb, surf, sbox(0.0, BOTTOM - 0.05, r1 - r0, y), "plastic", cell=2.0)
        if key not in ring["open_ends"]:
            e0, e1 = pt(r0 + 0.02, d - sgn * math.degrees(0.13 / r0)), pt(r1 - 0.02, d - sgn * math.degrees(0.13 / r1))
            K.parapet(mb, [e0, e1], y, h=0.95, t=RAIL, mat="plastic", cap=CAP)
            L.wall_box(e0, e1, y, y + 1.0, (0.0, 0.0))
    # Rekkverket mot gapene, utenom broene.
    def height(d):
        for d0, d1, y0, y1, _ in segs:
            if d0 - 1e-6 <= d <= d1 + 1e-6:
                if abs(y1 - y0) < 1e-6:
                    return y0
                n = max(1, round(abs(y1 - y0) / 0.2))
                dd, rise = (d1 - d0) / n, (y1 - y0) / n
                k = min(n - 1, max(0, int((d - d0) / dd + 1e-6)))
                # Den laveste av trinnene ved grensen, så rekkverket står i trinnene.
                return min(y0 + (k + 1) * rise, y0 + k * rise) if rise > 0 else y0 + (k + 1) * rise
        return segs[-1][3]
    rails = [(r0 + 0.13, gaps.get((name, "in"), []))]
    if not ring["wall"]:
        rails.append((r1 - 0.13, gaps.get((name, "out"), [])))
    for rr, gp in rails:
        spans, at_d = [], ds
        for g0, g1 in sorted(gp):
            if g0 > at_d:
                spans.append((at_d, g0))
            at_d = max(at_d, g1)
        if at_d < de:
            spans.append((at_d, de))
        for s0, s1 in spans:
            if (s1 - s0) * rr * P / 180 < 0.4:
                continue
            # Punktene: knekkene i profilen og hver 2°.
            ds_pts = sorted(set([s0, s1] + [d for seg in segs for d in (seg[0], seg[1]) if s0 < d < s1]
                                + [s0 + (s1 - s0) * k / max(1, math.ceil((s1 - s0) / 2.0)) for k in range(1, max(1, math.ceil((s1 - s0) / 2.0)))]))
            # Trappene: ett punkt per trinngrense.
            extra = []
            for d0, d1, y0, y1, _ in segs:
                if abs(y1 - y0) > 1e-6:
                    n = max(1, round(abs(y1 - y0) / 0.2))
                    extra += [d0 + (d1 - d0) * k / n for k in range(n + 1) if s0 < d0 + (d1 - d0) * k / n < s1]
            ds_pts = sorted(set(ds_pts + extra))
            path = [pt(rr, d) for d in ds_pts]
            base = [height(d) for d in ds_pts]
            K.parapet(mb, path, base[0], h=0.95, t=RAIL, mat="plastic", cap=CAP, base_y=base)
            for (p0, p1), b0, b1 in zip(zip(path, path[1:]), base, base[1:]):
                L.wall_box(p0, p1, min(b0, b1), max(b0, b1) + 1.0, (0.0, 0.0))


def bridge(L, b, walks, cut_out):
    """En radiell bro over gapet: flat eller med trapp, en tynn plate med
    rekkverk, som går 0,3 m inn i terrassene i begge ender (toppen der er
    skåret ut av terrassene)."""
    mb = L.mb
    d, ra, rb, ya, yb, w = b["d"], b["ra"], b["rb"], b["ya"], b["yb"], b["w"]
    u = radial(d)
    dirv = (-u[0], -u[1]) if rb < ra else u
    lat = (-dirv[1], dirv[0])
    pa = pt(ra, d)
    Lb = abs(ra - rb)
    ext_a, ext_b = 0.3, b.get("ext_b", 0.3)
    hw = w / 2
    S = lambda s, l: (pa[0] + dirv[0] * s + lat[0] * l, pa[1] + dirv[1] * s + lat[1] * l)
    rect = lambda s0, s1, h=hw: Polygon([S(s0, -h), S(s1, -h), S(s1, h), S(s0, h)])
    cut_out.append(rect(-ext_a, Lb + ext_b))
    th = 0.6
    stairs = abs(yb - ya) > 1e-6
    # Toppen: flatt i endene, trinn imellom.
    grid_face(mb, Horizontal(ya), rect(-ext_a, 0.0), "plastic")
    grid_face(mb, Horizontal(yb), rect(Lb, Lb + ext_b), "plastic")
    if stairs:
        n = max(2, round(abs(yb - ya) / 0.2))
        K.steps(mb, pa, dirv, lat, w, 0.0, Lb, ya, yb, n, mat="plastic")
    else:
        grid_face(mb, Horizontal(ya), rect(0.0, Lb), "plastic")
    # Sidene og undersida: platen følger trappa.
    yat = lambda s: ya if s <= 0 else (yb if s >= Lb else ya + (yb - ya) * s / Lb)
    ss = [-ext_a, 0.0, Lb, Lb + ext_b]
    for s0, s1 in zip(ss, ss[1:]):
        a0, a1 = S(s0, -hw), S(s1, -hw)
        b0, b1 = S(s0, hw), S(s1, hw)
        y0, y1 = yat(s0) - th, yat(s1) - th
        mb.quad((a0[0], y0, a0[1]), (b0[0], y0, b0[1]), (b1[0], y1, b1[1]), (a1[0], y1, a1[1]), "plastic", facing=(0, -1, 0))
    for l, sgn in ((-hw, -1.0), (hw, 1.0)):
        if stairs:
            n = max(2, round(abs(yb - ya) / 0.2))
            t = Lb / n
            for k in range(n):
                sa, sb2 = k * t, (k + 1) * t
                yk = ya + (yb - ya) * (k + 1) / n
                q = [S(sa, l), S(sb2, l)]
                mb.quad((q[0][0], yat(sa) - th, q[0][1]), (q[1][0], yat(sb2) - th, q[1][1]), (q[1][0], yk, q[1][1]), (q[0][0], yk, q[0][1]), "plastic",
                        facing=(lat[0] * sgn, 0, lat[1] * sgn))
            segs = [(-ext_a, 0.0, ya), (Lb, Lb + ext_b, yb)]
        else:
            segs = [(-ext_a, Lb + ext_b, ya)]
        for s0, s1, y in segs:
            q = [S(s0, l), S(s1, l)]
            mb.quad((q[0][0], y - th, q[0][1]), (q[1][0], y - th, q[1][1]), (q[1][0], y, q[1][1]), (q[0][0], y, q[0][1]), "plastic",
                    facing=(lat[0] * sgn, 0, lat[1] * sgn))
    # Kollideren: platen (rampa).
    hpts = []
    for s in (-ext_a, 0.0, Lb, Lb + ext_b):
        for l in (-hw, hw):
            x, z = S(s, l)
            hpts += [(x, yat(s) + (0.02 if stairs and 0 < s < Lb else 0.0), z), (x, yat(s) - th, z)]
    L.hull(hpts[:8])
    L.hull(hpts[4:12])
    L.hull(hpts[8:])
    # Rekkverket langs begge sider over gapet.
    rail_b = b.get("rail_b", 0.0)
    for l in (-hw + 0.13, hw - 0.13):
        path = [S(0.0, l), S(Lb + rail_b, l)]
        base = [ya, yb]
        K.parapet(mb, path, ya, h=0.95, t=RAIL, mat="plastic", cap=CAP, base_y=base)
        L.hull([(path[0][0], ya, path[0][1]), (path[1][0], yb, path[1][1]), (path[1][0], yb + 1.0, path[1][1]), (path[0][0], ya + 1.0, path[0][1]),
                (path[0][0] + lat[0] * 0.1, ya, path[0][1] + lat[1] * 0.1), (path[1][0] + lat[0] * 0.1, yb, path[1][1] + lat[1] * 0.1),
                (path[1][0] + lat[0] * 0.1, yb + 1.0, path[1][1] + lat[1] * 0.1), (path[0][0] + lat[0] * 0.1, ya + 1.0, path[0][1] + lat[1] * 0.1)])
    # Navigasjonen: broa (planet langs trappa), og hakket inn i
    # terrassene i hver ende (der rekkverket er åpent), som hører til avsatsen.
    iw = hw - RAIL - I
    k = (yb - ya) / Lb
    plane = (k * dirv[0], k * dirv[1], ya - k * (pa[0] * dirv[0] + pa[1] * dirv[1]))
    gate = b.get("gate")
    g_from = b.get("gate_from")
    if g_from is not None:
        # Rotunden: trappa er åpen, porten og flaten gjennom muren er porten sin.
        sg = ra - g_from
        L.patch(rect(0.0, Lb, iw), plane, b["room"])
        L.patch(rect(Lb, sg, iw), (0.0, 0.0, yb), b["room"])
        L.patch(rect(sg, Lb + ext_b, iw), (0.0, 0.0, yb), b["room"], gate)
    else:
        L.patch(rect(0.0, Lb, iw), plane, b["room"], gate)
    return rect(-2.0, 0.0, iw), rect(Lb, Lb + 2.0, iw), (dirv, lat, pa, Lb, hw)


def features(L):
    mb = L.mb
    rm = L.rooms
    # Det som skjæres ut av toppen på terrassene: gangene inn fra muren og broene.
    cut = [h.poly for h in L.halls]
    notches = []
    for b in BRIDGES:
        notches.append((b, bridge(L, b, None, cut)))
    sp = SPERRE_ARC
    cut_all = unary_union(cut)
    # Åpningene i rekkverkene der broene går inn (vinkler).
    gaps = {}
    for b, _ in notches:
        for rr, ring_end in ((b["ra"], "a"), (b["rb"], "b")):
            ring = next((nm for nm, rg in RINGS.items() if (abs(rg["r0"] - rr) < 1e-6 or abs(rg["r1"] - rr) < 1e-6)
                         and any(d0 <= b["d"] + 360 * j <= d1 for d0, d1, *_ in rg["segs"] for j in (-1, 0, 1))), None)
            if ring is None:
                continue
            side = "in" if abs(RINGS[ring]["r0"] - rr) < 1e-6 else "out"
            half = math.degrees(math.asin(min(0.99, (b["w"] / 2) / rr)))
            dd = next(b["d"] + 360 * j for j in (-1, 0, 1) for d0, d1, *_ in RINGS[ring]["segs"] if d0 <= b["d"] + 360 * j <= d1)
            gaps.setdefault((ring, side), []).append((dd - half, dd + half))
    walks, tris, landings = {}, [], []
    holes = [TRAP_HOLE, LADDER_HOLE]
    hollow = sbox(KILDE_C[0] - 2.4, KILDE_C[1] - 2.4, KILDE_C[0] + 2.4, KILDE_C[1] + 2.4)
    for name in RINGS:
        build_ring(L, name, cut_all, holes if name == "ytre" else [], hollow if name == "ytre" else None, gaps, walks, tris, landings)
    # Avsatsene over gulvet i rommet sitt står som egne rom i json (som
    # galleriene), så det som står på dem har gulvet sitt der.
    for rid, y, d0, poly in landings:
        r = rm[rid]
        if abs(y - r.y) > 1e-6:
            coords = [[B.rnd(x), B.rnd(z)] for x, z in list(poly.simplify(0.01).exterior.coords)[:-1]]
            L.json_rooms.append({"id": f"{rid}_{round(d0) % 360}", "name": r.name, "poly": coords, "y": y, "h": 4.5, "secret": r.secret})
    for tri, rid in tris:
        L.tri_patches.append((tri, rid))
    # Hakkene inn i terrassene ved broene: de hører til avsatsen der.
    for b, (na, nb, _) in notches:
        for notch, y in ((na, b["ya"]), (nb, b["yb"])):
            for (rid, yy), polys in walks.items():
                if abs(yy - y) < 1e-6 and any(p.intersects(notch) for p in polys):
                    polys.append(notch.intersection(room_sector(rid).buffer(0.01)))
                    break
    # Sømsperren: den buede broa med gitteret.
    y = sp["y"]
    top = sector(sp["r0"], sp["r1"], sp["d0"], sp["d1"], step=1.0)
    grid_face(mb, Horizontal(y), top, "plastic")
    grid_face(mb, Horizontal(y - 0.6, up=False), top, "plastic")
    for rr, inward in ((sp["r0"], True), (sp["r1"], False)):
        grid_face(mb, Cyl(C, rr, a0=A(sp["d0"]), inward=inward), sbox(0.0, y - 0.6, math.radians(sp["d1"] - sp["d0"]) * rr, y), "plastic")
        path = arcpts(rr + (0.13 if inward else -0.13), sp["d0"], sp["d1"], 1.0)
        K.parapet(mb, path, y, h=0.95, t=RAIL, mat="plastic", cap=CAP)
        for p0, p1 in zip(path, path[1:]):
            L.wall_box(p0, p1, y, y + 1.0, (0.0, 0.0))
    pts = list(top.exterior.coords)[:-1]
    L.hull([(x, y, z) for x, z in pts] + [(x, y - 0.6, z) for x, z in pts])
    L.patch(sector(sp["r0"] + RAIL + I, sp["r1"] - RAIL - I, sp["d0"], sp["d1"], step=1.0), (0.0, 0.0, y), "komp", SPERRE)
    # Stolpene og overliggeren rundt gitterne på broene.
    for gp, gy, gw, dirv in ((pt(23.0, 460), -15.2, 2.6, radial(460)), (pt(28.0, 552), -16.8, 4.0, tangent(552))):
        lat = (-dirv[1], dirv[0])
        for sgn in (-1.0, 1.0):
            px, pz = gp[0] + lat[0] * sgn * (gw / 2 - 0.13), gp[1] + lat[1] * sgn * (gw / 2 - 0.13)
            K.obox(mb, (px, gy + 1.9, pz), (0.36, 3.8, 0.36), yaw_of(dirv), "plastic")
            L.box((px, gy + 1.9, pz), (0.36, 3.8, 0.36), yaw_of(dirv))
        K.obox(mb, (gp[0], gy + 3.95, gp[1]), (0.4, 0.3, gw), yaw_of(dirv), "magic_seam")
    # Rotunden: blokka under gulvet, muren rundt med porten, gulvet.
    rot = rm["rotunde"]
    a_g = A(BRIDGES[4]["d"])
    disc = Point(C).buffer(ROT_R + ROT_W, 72)
    floor = Point(C).buffer(ROT_R, 72).difference(cut_all)
    grid_face(mb, Horizontal(ROT_Y), floor, "plastic")
    pts = list(disc.exterior.coords)[:-1:2]
    L.hull([(x, ROT_Y, z) for x, z in pts] + [(x, BOTTOM - 0.3, z) for x, z in pts])
    gate_w, gate_h = 3.0, 3.4
    seam_a = a_g + P
    for rr, inward, y0 in ((ROT_R, True, ROT_Y), (ROT_R + ROT_W, False, BOTTOM)):
        half = math.asin(gate_w / 2 / rr)
        u_g = ((a_g - seam_a) % (2 * P)) * rr
        hole = sbox(u_g - half * rr, ROT_Y - 0.05, u_g + half * rr, ROT_Y + gate_h)
        circ = 2 * P * rr
        shape = sbox(0.0, y0 - 0.05, circ, ROT_TOP).difference(hole)
        grid_face(mb, Cyl(C, rr, a0=seam_a, inward=inward), shape, "plastic", cell=1.5)
    # Toppen av muren og åpningen gjennom den.
    ring_top = Point(C).buffer(ROT_R + ROT_W, 72).difference(Point(C).buffer(ROT_R, 72))
    grid_face(mb, Horizontal(ROT_TOP), ring_top, "plastic")
    u = radial(BRIDGES[4]["d"])
    lat = (-u[1], u[0])
    corner = lambda rr, l: (C[0] + u[0] * math.sqrt(rr * rr - l * l) + lat[0] * l, C[1] + u[1] * math.sqrt(rr * rr - l * l) + lat[1] * l)
    for l, sgn in ((-gate_w / 2, 1.0), (gate_w / 2, -1.0)):
        a0, a1 = corner(ROT_R, l), corner(ROT_R + ROT_W, l)
        mb.quad((a0[0], ROT_Y, a0[1]), (a1[0], ROT_Y, a1[1]), (a1[0], ROT_Y + gate_h, a1[1]), (a0[0], ROT_Y + gate_h, a0[1]), "plastic",
                facing=(lat[0] * sgn, 0, lat[1] * sgn))
    q = [corner(ROT_R, -gate_w / 2), corner(ROT_R + ROT_W, -gate_w / 2), corner(ROT_R + ROT_W, gate_w / 2), corner(ROT_R, gate_w / 2)]
    mb.poly([(x, ROT_Y + gate_h, z) for x, z in q], [(x, z) for x, z in q], "plastic", facing=(0, -1, 0))
    # Murkolliderne rundt, og over porten.
    n = 72
    for k in range(n):
        aa, ab = 2 * P * k / n, 2 * P * (k + 1) / n
        mid = (aa + ab) / 2
        p0 = (C[0] + ROT_R * math.cos(aa), C[1] + ROT_R * math.sin(aa))
        p1 = (C[0] + ROT_R * math.cos(ab), C[1] + ROT_R * math.sin(ab))
        off = abs(((mid - a_g + P) % (2 * P)) - P)
        lo = ROT_Y + gate_h if off * ROT_R < gate_w / 2 + 0.3 else ROT_Y - 0.5
        L.wall_box(p0, p1, lo, ROT_TOP + 0.3, (math.cos(mid), math.sin(mid)))
    # Kanten i lyset: ringer av sømmer i gulvet rundt Rammeverket.
    for rr in (3.0, 6.5):
        ring = Point(C).buffer(rr + 0.06, 64).difference(Point(C).buffer(rr - 0.06, 64)).difference(cut_all)
        grid_face(mb, Horizontal(ROT_Y + 0.01), ring, "magic_white")
    for k in range(8):
        dd = k * 45 + 22.5
        p0, p1 = pt(3.06, dd), pt(6.44, dd)
        lat2 = (-radial(dd)[1] * 0.06, radial(dd)[0] * 0.06)
        mb.quad((p0[0] - lat2[0], ROT_Y + 0.01, p0[1] - lat2[1]), (p1[0] - lat2[0], ROT_Y + 0.01, p1[1] - lat2[1]),
                (p1[0] + lat2[0], ROT_Y + 0.01, p1[1] + lat2[1]), (p0[0] + lat2[0], ROT_Y + 0.01, p0[1] + lat2[1]), "magic_white", facing=(0, 1, 0))
    for dd in range(0, 360, 30):
        if abs(((dd - BRIDGES[4]["d"] + 180) % 360) - 180) > 14:
            seam_cyl(mb, ROT_R, dd, ROT_Y + 0.2, ROT_TOP - 0.1, True)
            seam_cyl(mb, ROT_R + ROT_W, dd + 15, BOTTOM + 2.0, ROT_TOP - 0.1, False)
    rot.floor_walk = Point(C).buffer(ROT_R - I, 72).difference(cut_all.buffer(-0.001))
    # Kanten under kuppelen: kuppelen er så flat over trakten at kanten av
    # den (72 rette stykker) står 4 cm innenfor muren, og gjennom spalten
    # synes himmelen over murkronen (`snapshot --leak-scan`). En ring rett
    # over kuppelkanten og bak muren lukker den.
    rim = Point(C).buffer(HOST_R + 0.3, 144).difference(Point(C).buffer(HOST_R - 0.5, 144))
    grid_face(mb, Horizontal(HOST_TOP + 0.2, up=False), rim, "plastic")
    # Urskriften: sjaktene fra taket opp gjennom avsatsen.
    kt = KILDE_Y + rm["kilde"].h
    L.shaft(TRAP_HOLE, kt, -7.6, mat="fresco")
    L.shaft(LADDER_HOLE, kt, -7.6, mat="fresco")
    # Sømmene i terrassene og traktmuren: lange fiolette streker ned i avgrunnen.
    for name, ring in RINGS.items():
        segs = ring["segs"]
        ds, de = segs[0][0], segs[-1][1]
        dd = ds + 6
        k = 0
        while dd < de - 4:
            yt = min(y for d0, d1, y0, y1, _ in segs if d0 <= dd <= d1 for y in (y0, y1))
            seam_cyl(mb, ring["r0"], dd, BOTTOM + 1.0, yt - 0.4 - (k % 3) * 1.5, True)
            if not ring["wall"]:
                seam_cyl(mb, ring["r1"], dd + 9, BOTTOM + 1.0, yt - 0.6 - (k % 2) * 2.0, False)
            dd += 19 if name == "ytre" else 23
            k += 1
    for k, dd in enumerate(range(10, 360, 26)):
        seam_cyl(mb, HOST_R, dd, BOTTOM + 3.0 + (k % 4) * 2.0, HOST_TOP - 1.0 - (k % 3) * 3.0, True)
    # Løse biter av plast som henger i lufta i gapene.
    for k, (r, dd, yy, s) in enumerate(((33.0, 60, -14.0, 1.4), (33.0, 200, -18.0, 1.0), (23.0, 300, -24.0, 1.6), (23.0, 130, -26.0, 1.2),
                                         (14.0, 40, -32.0, 1.0), (33.0, 250, -26.0, 2.0), (14.0, 220, -30.0, 1.4))):
        x, z = pt(r, dd)
        K.box(mb, (x - s / 2, yy - s / 2, z - s / 2), (x + s / 2, yy + s / 2, z + s / 2), "plastic")
        seam_cyl(mb, r - s / 2 + 0.01, dd, yy - s / 2, yy + s / 2, True, w=0.06)
    # Navigasjonen på terrassene: avsatsene i gulvhøyden til rommet er
    # rommets gulv, de andre er flater over det.
    for (rid, y), polys in walks.items():
        walk = unary_union(polys)
        r = rm[rid]
        if abs(y - r.y) < 1e-6:
            prev = getattr(r, "floor_walk", None)
            r.floor_walk = walk if prev is None else unary_union([prev, walk])
        else:
            L.decks.append(dict(room=rid, of=rid, y=y, walk=walk, ring=walk.buffer(0.7), obstacles=[]))
    for rid in ("ytre_o", "ytre_v", "midt", "komp", "node", "indre", "hvile"):
        if getattr(rm[rid], "floor_walk", None) is None:
            rm[rid].floor_walk = Polygon()


def dress(L):
    rm = L.rooms
    mb = L.mb
    # Kanten: plast med de første sømmene, og et kaldt lys over utsikten.
    for x in (-4.98, 4.98):
        seam_wall = (-1.0 if x > 0 else 1.0, 0.0)
        for z in (-4.0, 0.5):
            K.box(mb, (x - 0.02 + seam_wall[0] * 0.03, 0.3, z - 0.045), (x + 0.02 + seam_wall[0] * 0.03, 4.2, z + 0.045), "magic_seam")
    L.light((0.0, 3.2, -1.0), VIOLET, 8.0, "port", True, 1.0)
    # Trakten: rotunden lyser hvitt opp gjennom hele trakten, og sømmene
    # kaster fiolett lys på terrassene.
    L.light((C[0], ROT_Y + 4.0, C[1]), WHITE, 50.0, "trakten", False, 9.0)
    L.light((C[0], ROT_Y + 1.5, C[1]), WHITE, 14.0, "rotunde", True, 3.0)
    for d, r, y, rng, inten in ((30, 38, 2.5, 14.0, 2.2), (100, 38, -1.0, 14.0, 2.0), (160, 38, -3.0, 14.0, 2.0), (220, 38, -5.0, 14.0, 2.0),
                                (280, 38, -6.6, 14.0, 2.0), (336, 33, -9.0, 12.0, 2.2), (40, 28, -9.5, 12.0, 2.0), (110, 28, -12.6, 12.0, 2.0),
                                (165, 28, -14.0, 12.0, 2.0), (250, 28, -9.0, 12.0, 1.6), (180, 18.5, -16.2, 10.0, 1.8), (260, 18.5, -19.0, 10.0, 1.8),
                                (320, 18.5, -23.4, 10.0, 1.8), (90, 18.5, -12.6, 10.0, 1.4)):
        x, z = pt(r, d)
        L.light((x, y, z), VIOLET, rng, "trakten", True, inten)
    for d in (60, 180, 300):
        x, z = pt(30.0, d)
        L.light((x, BOTTOM + 3.0, z), VIOLET, 18.0, "trakten", False, 3.0)
    # Sidegrenene.
    L.light((49.0, -3.6 + 2.4, 46.0), (0.8, 0.85, 1.0), 7.0, "buffer", False, 1.2)
    L.light((-50.0, -9.2 + 3.0, 46.0), VIOLET, 9.0, "cache", True, 1.2)
    L.light((KILDE_C[0], KILDE_Y + 2.2, KILDE_C[1]), (1.0, 0.68, 0.32), 6.0, "kilde", True, 1.4)
    # Stabelkammeret: stabler av halvferdige komponenter langs murene.
    ca = rm["cache"]
    for k, (x, z, s, h) in enumerate(((-54.0, 43.0, 1.2, 1.6), (-52.6, 43.0, 1.0, 0.9), (-46.4, 49.0, 1.1, 1.3), (-54.0, 46.0, 0.9, 2.1), (-47.8, 43.1, 1.0, 0.8))):
        K.box(mb, (x - s / 2, ca.y, z - s / 2), (x + s / 2, ca.y + h, z + s / 2), "plastic")
        L.box((x, ca.y + h / 2, z), (s, h, s))
        ca.obstacles.append(sbox(x - s / 2, z - s / 2, x + s / 2, z + s / 2))
        if k % 2 == 0:
            K.box(mb, (x - s / 2 - 0.01, ca.y + h * 0.3, z - 0.03), (x + s / 2 + 0.01, ca.y + h * 0.3 + 0.05, z + 0.03), "magic_seam")
    # Sammenføyningen: samlebåndet langs ytterkanten, med kort og rutenett på.
    for dd in range(646, 676, 6):
        x, z = pt(29.9, dd)
        yaw = yaw_of(tangent(dd))
        K.obox(mb, (x, -11.2 + 0.35, z), (0.7, 0.7, 2.6), yaw, "plastic")
        K.obox(mb, (x, -11.2 + 0.78, z), (0.5, 0.16, 0.7), yaw + 0.2 * ((dd // 6) % 3 - 1), "magic_seam" if (dd // 6) % 2 else "iron")
        L.box((x, -11.2 + 0.4, z), (0.7, 0.8, 2.6), yaw)
    walk_cut = unary_union([Point(pt(29.9, dd)).buffer(1.5) for dd in range(646, 676, 6)])
    for dk in L.decks:
        if dk["room"] == "komp" and abs(dk["y"] + 11.2) < 1e-6:
            dk["obstacles"].append(walk_cut)
    # Knutene: et lysende hode på rekkverket bak hver spak.
    for dd, y in ((420, -13.6), (470, -15.2), (510, -16.8), (543, -16.8)):
        x, z = pt(25.13, dd)
        K.box(mb, (x - 0.16, y + 0.95, z - 0.16), (x + 0.16, y + 1.45, z + 0.16), "magic_seam")
        L.light((x, y + 1.6, z), VIOLET, 5.0, "midt", True, 0.9)


LAYOUT = dict(
    theme="kjerne",
    extra=EXTRA,
    halls=HALLS,
    marks=marks,
    pre=pre,
    features=features,
    dress=dress,
    round={"trakten": dict(wall_mat="plastic", wall_cell=2.0, floor_mat="glitch", floor_cell=2.0, dome_rise=4.2, dome_rings=10, dome_seg=72, ribs=0)},
    virtual={"ytre_o": "trakten", "ytre_v": "trakten", "midt": "trakten", "komp": "trakten", "node": "trakten",
             "indre": "trakten", "hvile": "trakten", "rotunde": "trakten"},
    override={
        "port": {"shape": ("rect", 0, -2, 10, 8), "y": 0.0, "h": 5.0},
        "ytre_o": {"shape": shape_of("ytre_o"), "y": room_levels("ytre_o")[0], "h": room_levels("ytre_o")[1] - room_levels("ytre_o")[0] + 4.5},
        "ytre_v": {"shape": shape_of("ytre_v"), "y": room_levels("ytre_v")[0], "h": room_levels("ytre_v")[1] - room_levels("ytre_v")[0] + 4.5},
        "midt": {"shape": shape_of("midt"), "y": room_levels("midt")[0], "h": room_levels("midt")[1] - room_levels("midt")[0] + 4.5},
        "komp": {"shape": shape_of("komp"), "y": room_levels("komp")[0], "h": room_levels("komp")[1] - room_levels("komp")[0] + 4.5, "name": "Sammenføyningen"},
        "node": {"shape": shape_of("node"), "y": -15.2, "h": 4.5, "name": "Knutekammeret"},
        "indre": {"shape": shape_of("indre"), "y": room_levels("indre")[0], "h": room_levels("indre")[1] - room_levels("indre")[0] + 4.5},
        "hvile": {"shape": shape_of("hvile"), "y": -26.0, "h": 4.5},
        "rotunde": {"shape": ("circle", C[0], C[1], ROT_R), "y": ROT_Y, "h": 6.0},
        "buffer": {"shape": ("rect", 49, 46, 8, 7), "y": -3.6, "h": 3.5, "name": "Det glemte forrådet"},
        "cache": {"shape": ("rect", -50, 46, 10, 8), "y": -9.2, "h": 4.0, "name": "Stabelkammeret"},
        "kilde": {"shape": ("rect", round(KILDE_C[0], 3), round(KILDE_C[1], 3), 4, 4), "y": KILDE_Y, "h": 3.0, "name": "Urskriften"},
    },
    wall_mats={"port": "plastic", "buffer": "plastic", "cache": "plastic", "kilde": "fresco"},
    vault_mats={"port": "plastic", "buffer": "plastic", "cache": "plastic", "trakten": "plastic", "kilde": "fresco"},
    hall_flat_step=2.0,
    nav_grid=4.0,
    # Bitene er 32 × 32 m (under `LOD_MIN_TRIS`, så de får ikke grovt detaljnivå): trakten ses nesten hel fra overalt, så små biter
    # sparer ingenting å kule, og hver bit er et kort på siden for dommen.
    chunk=32.0,
    round_seam_free=True,
)
