"""Bygger et kryptnivå etter planen (`tools/crypt/levels/level_NN.py`) med
byggesettet (`kit_grav.py`), og skriver det motoren trenger
(`docs/krypten/format.md`):

- `assets/models/crypt/crypt_NN_<bit>.glb`: geometrien i biter på 16 × 16 m,
- `assets/models/crypt/crypt_NN.json`: rommene, kolliderne, navigasjonsnettet
  (konvekse polygoner 0,4 m fra murene, sydd sammen), lysene og merkene,
- `docs/krypten/nav_NN.png`: navigasjonsnettet tegnet over planen.

    python3 tools/crypt/build_level.py 1            # json, bilde og tall (uten Blender)
    python3.13 tools/crypt/build_level.py 1 --export  # og glb-bitene (Blender)

Planen sier hvilke rom som finnes, hvor de står og hvordan de henger sammen.
Hvor gangene går (rette strekk på rutenettet), hvor trappene ligger i dem og
hva som står i rommene, er bestemt her per nivå (`LAYOUTS`), så planen
forblir tegningen eieren godkjente.

Akser som i spillet: x mot øst, y opp, z mot sør, i meter, i nivåets eget
koordinatsystem (motoren flytter nivået på plass under byen).
"""

import importlib.util
import json
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "levels"))

import kit_grav as K  # noqa: E402
from kit_grav import MB, Cyl, Horizontal, Plane, arch_profile, grid_face  # noqa: E402
from shapely import constrained_delaunay_triangles  # noqa: E402
from shapely.geometry import LineString, Point, Polygon  # noqa: E402
from shapely.geometry import box as sbox  # noqa: E402
from shapely.ops import unary_union  # noqa: E402

OUT = os.path.join(ROOT, "assets", "models", "crypt")
DOCS = os.path.join(ROOT, "docs", "krypten")

INSET = 0.4  # navigasjonsnettet holder seg så langt fra murene
SPRING = 2.2  # høyden på gangmurene (vederlaget), over gulvet
STAIR_IN = 0.1  # trapper og ramper inntil en mur går så langt inn i murblokka (aldri tangent)
WALL_T = 0.4  # tykkelsen på murkolliderne
CHUNK = 16.0  # bitene

WARM = (1.0, 0.55, 0.24)
CANDLE = (1.0, 0.68, 0.36)
COLD = (0.42, 0.52, 0.85)
BLUE = (0.3, 0.5, 1.0)


def load_plan(n):
    path = os.path.join(HERE, "levels", f"level_{n:02d}.py")
    spec = importlib.util.spec_from_file_location(f"level_{n:02d}", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.LEVEL


def rnd(v, k=3):
    return round(float(v), k)


# --- Rom og ganger --------------------------------------------------------------


class Room:
    """Et rom fra planen: rektangel eller sirkel, gulvet (y) og høyden opp
    til vederlaget (h; hvelvet går over)."""

    def __init__(self, rid, name, shape, y, h, secret):
        self.id, self.name, self.shape, self.y, self.h, self.secret = rid, name, shape, y, h, secret
        if shape[0] == "rect":
            _, cx, cz, w, d = shape
            self.x0, self.x1, self.z0, self.z1 = cx - w / 2, cx + w / 2, cz - d / 2, cz + d / 2
            self.poly = sbox(self.x0, self.z0, self.x1, self.z1)
        else:
            _, cx, cz, r = shape
            self.c, self.r = (cx, cz), r
            self.poly = Polygon([(cx + r * math.cos(2 * math.pi * k / 72), cz + r * math.sin(2 * math.pi * k / 72)) for k in range(72)])
        self.doors = []  # (side, u, bredde, gulv, kind) — åpningene i murene
        self.obstacles = []  # shapely-polygoner der det står noe
        self.holes = []  # hull i gulvet
        self.ceil = y + h  # laveste tak over det gangbare (til json)
        self.flat = None  # flatt tak (med dette hullet) i stedet for hvelv
        self.walk = None

    @property
    def rect(self):
        return self.shape[0] == "rect"


class Hall:
    """En gang mellom to rom: rette strekk (punktene i planen, fra muren i
    rommet `a` til muren i rommet `b`), bredden, og gulvet: flatt i høyden
    til `a` til trappa, ned eller opp over trappestrekket, og flatt videre.
    `stairs` er (strekk, fra, til) i meter langs strekket."""

    def __init__(self, hid, a, b, kind, pts, w, ya, yb, stairs=None, gate=None, name="", secret=False):
        self.id, self.a, self.b, self.kind, self.pts, self.w = hid, a, b, kind, pts, w
        self.ya, self.yb, self.gate, self.name, self.secret = ya, yb, gate, name, secret
        self.stairs = stairs
        if stairs is None and abs(ya - yb) > 1e-6:
            # Hele det lengste strekket er trapp.
            seg = max(range(len(pts) - 1), key=lambda i: math.dist(pts[i], pts[i + 1]))
            self.stairs = (seg, 0.0, math.dist(pts[seg], pts[seg + 1]))
        self.segs = []
        for i in range(len(pts) - 1):
            (x0, z0), (x1, z1) = pts[i], pts[i + 1]
            L = math.dist(pts[i], pts[i + 1])
            d = ((x1 - x0) / L, (z1 - z0) / L)
            self.segs.append((pts[i], pts[i + 1], d, L))
        # Fotavtrykket: strekkene, med hjørnefeltet tatt med i det første.
        rects = []
        for i, (p0, p1, d, L) in enumerate(self.segs):
            e0 = self.w / 2 if i > 0 else 0.0
            e1 = self.w / 2 if i < len(self.segs) - 1 else 0.0
            rects.append(self.seg_rect(i, -e0, L + e1))
        self.poly = unary_union(rects)

    def seg_rect(self, i, s0, s1, w=None):
        p0, p1, d, L = self.segs[i]
        hw = (w or self.w) / 2
        lat = (-d[1], d[0])
        pts = [(p0[0] + d[0] * s + lat[0] * l, p0[1] + d[1] * s + lat[1] * l) for s, l in ((s0, -hw), (s1, -hw), (s1, hw), (s0, hw))]
        return Polygon(pts)

    def floor_s(self, i, s):
        """Gulvhøyden i strekk i, s meter inn."""
        if self.stairs is None:
            return self.ya
        si, a, b = self.stairs
        if i < si:
            return self.ya
        if i > si:
            return self.yb
        if s <= a:
            return self.ya
        if s >= b:
            return self.yb
        return self.ya + (self.yb - self.ya) * (s - a) / (b - a)

    def locate(self, x, z):
        """(strekk, s) for punktet: nærmeste strekk."""
        best = None
        for i, (p0, p1, d, L) in enumerate(self.segs):
            s = (x - p0[0]) * d[0] + (z - p0[1]) * d[1]
            sc = min(max(s, 0.0), L)
            q = (p0[0] + d[0] * sc, p0[1] + d[1] * sc)
            dist = math.dist(q, (x, z))
            if best is None or dist < best[0] - 1e-6:
                best = (dist, i, s)
        return best[1], best[2]

    def floor_at(self, x, z):
        i, s = self.locate(x, z)
        return self.floor_s(i, s)


# --- Utlegget for nivå 1 --------------------------------------------------------

DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET, LADDER, FALL = "door", "hall", "stairs", "gate", "shortcut", "secret", "ladder", "fall"

# Ekstra rom som ikke står i planen: bunnen av brønnsjakta og kammeret
# under fallemmen i sidegraven (begge hører til hemmeligheten).
EXTRA_ROOMS_1 = [
    ("sjaktbunn", "Bunnen av brønnen", ("rect", -36, 42, 4, 4), -8, 3.2, True),
    ("stigebunn", "Under fallemmen", ("rect", -12, 23, 3, 6), -8, 3.2, True),
]

# Gangene: (a, b, slag, punkter fra mur til mur, bredde, trapp, port).
HALLS_1 = [
    ("kapell", "gravgang", DOOR, [(0, 4), (0, 8)], 3.0, None, None),
    ("gravgang", "sidegrav", DOOR, [(-2, 20), (-6, 20)], 2.0, None, None),
    ("gravgang", "krysset", STAIRS, [(0, 36), (0, 38.5)], 3.0, None, None),
    ("krysset", "pilgang", DOOR, [(-5.5, 44), (-10, 44)], 2.5, None, None),
    ("pilgang", "bronn", DOOR, [(-28, 44), (-30.5, 44)], 2.5, None, None),
    ("krysset", "beinhus", STAIRS, [(5.5, 44), (19.4, 44)], 3.5, (0, 3.0, 9.0), None),
    ("krysset", "soylegang", GATE, [(0, 49.5), (0, 53)], 3.4, None, "gitter_1"),
    ("soylegang", "alter", DOOR, [(0, 87), (0, 89.5)], 4.0, None, None),
    ("soylegang", "kistekjeller", STAIRS, [(-4.5, 72), (-11, 72)], 2.5, None, None),
    ("alter", "skatt", SECRET, [(0, 102.5), (0, 104.5)], 2.0, None, "hemmelig_1"),
    ("alter", "nedgang", STAIRS, [(7.5, 98), (17.5, 98)], 3.0, (0, 3.5, 6.5), None),
    ("nedgang", "beinhus", SHORTCUT, [(22, 93.5), (22, 60), (30, 60), (30, 54.8)], 2.2, (2, 0.0, 4.4), "snarvei_1"),
    ("sjaktbunn", "dypgrav", HALL, [(-36, 40), (-36, 24), (-32.5, 24)], 2.0, None, None),
    ("dypgrav", "stigebunn", HALL, [(-23.5, 24), (-13.5, 24)], 2.0, None, None),
]

# Beinhuset: galleriet rundt (ring), trappa langs muren opp til det.
BEINHUS = dict(c=(30.0, 44.0), r=11.0, gallery_y=2.0, ring_in=8.0, stair_a0=math.radians(300), stair_a1=math.radians(375), stair_in=8.2)

# Merkene (format.md). Plassert etter planen; noen er flyttet litt så de
# står fritt (kista i sidegraven, spaken og kista på galleriet).
def marks_1(y):
    P = math.pi
    gx, gz = BEINHUS["c"]
    gal = lambda deg, r: (gx + r * math.cos(math.radians(deg)), gz + r * math.sin(math.radians(deg)))
    lever = gal(270, 10.1)
    chest_g = gal(40, 10.1)
    chest_g_yaw = math.atan2(gx - chest_g[0], gz - chest_g[1])
    return [
        {"kind": "start", "pos": [0, 0, 2], "yaw": P},
        {"kind": "up", "pos": [0, 0, -3.0], "yaw": 0.0},
        {"kind": "rest", "pos": [3.0, 0, -2.6], "yaw": 0.0},
        {"kind": "mob", "pos": [0, 0, 14], "label": "button", "n": 2, "patrol": [[0, 0, 11], [0, 0, 33]]},
        {"kind": "chest", "pos": [-8.2, 0, 18.2], "yaw": P / 2},
        {"kind": "trapdoor", "id": "fallem_1", "label": "bolted", "pos": [-12, 0, 21.2], "size": [1.2, 1.2], "to": "stigebunn"},
        {"kind": "mob", "pos": [2, -1, 44], "label": "tooltip"},
        {"kind": "trap", "label": "arrows", "id": "piler_1", "pos": [-19, -1, 44], "size": [1.2, 1.2], "dir": [0, 0, 1]},
        {"kind": "mob", "pos": [-36, -1, 47.5], "label": "button", "n": 2},
        {"kind": "chest", "pos": [-40, -1, 40.2], "yaw": P / 4},
        {"kind": "trapdoor", "id": "bronn_1", "label": "shaft", "pos": [-36, -1, 42], "size": [1.6, 1.6], "to": "sjaktbunn"},
        {"kind": "hint", "label": "draft", "pos": [-36, -1, 42], "for": "bronn_1"},
        {"kind": "lore", "pos": [-30.5, -8, 21.2], "yaw": 0.0, "label": "Gravskriveren",
         "text": "Den som faller ned i brønnen, er ikke den første. Gå mot øst, under sidegraven er det en stige, og lemmen er boltet herfra."},
        {"kind": "bonus", "pos": [-26, -8, 26.5], "yaw": P, "label": "liv"},
        {"kind": "ladder", "pos": [-12, -8, 20.66], "yaw": 0.0, "top": [-12, 0, 22.45]},
        {"kind": "mob", "pos": [30, -3, 47], "label": "tooltip", "n": 3},
        {"kind": "mob", "pos": [26, -3, 40], "label": "button", "n": 2, "patrol": [[25, -3, 40], [35, -3, 40], [35, -3, 48], [25, -3, 48]]},
        {"kind": "lever", "id": "spak_1", "pos": [rnd(lever[0]), 2.0, rnd(lever[1])], "yaw": 0.0, "opens": ["gitter_1"]},
        {"kind": "chest", "pos": [rnd(chest_g[0]), 2.0, rnd(chest_g[1])], "yaw": rnd(chest_g_yaw)},
        {"kind": "gate", "id": "gitter_1", "pos": [0, -1, 51.25], "yaw": 0.0, "width": 2.8},
        {"kind": "mob", "pos": [0, -1, 62], "label": "button", "n": 2, "patrol": [[-1.2, -1, 56], [-1.2, -1, 84], [1.2, -1, 84], [1.2, -1, 56]]},
        {"kind": "mob", "pos": [0, -1, 78], "label": "tooltip", "n": 2},
        {"kind": "mob", "pos": [-17, -3.5, 73], "label": "knapp i bakhold", "n": 4, "ambush": "kistekjeller"},
        {"kind": "chest", "pos": [-21.5, -3.5, 75.6], "yaw": P},
        {"kind": "mob", "pos": [0, -1, 94], "label": "button", "n": 3},
        {"kind": "secret_wall", "id": "hemmelig_1", "pos": [0, -1, 102.5], "yaw": P, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [1.7, -1, 102.48], "yaw": P, "for": "hemmelig_1"},
        {"kind": "hint", "label": "draft", "pos": [0, -1, 101.6], "for": "hemmelig_1"},
        {"kind": "rare", "pos": [0, -1, 107.6], "yaw": P, "label": "Badge"},
        {"kind": "shortcut", "id": "snarvei_1", "pos": [22, -2, 93.5], "yaw": 0.0, "width": 1.6, "from": "nedgang"},
        {"kind": "down", "pos": [20.4, -2, 98], "yaw": -P / 2},
    ]


LAYOUTS = {1: dict(extra=EXTRA_ROOMS_1, halls=HALLS_1, marks=marks_1)}


# --- Nivået ---------------------------------------------------------------------


class Level:
    def __init__(self, n):
        self.n = n
        self.plan = load_plan(n)
        lay = LAYOUTS[n]
        self.rooms = {}
        for r in self.plan.rooms:
            self.rooms[r.id] = Room(r.id, r.name, r.shape, r.y, r.h, r.secret)
        for rid, name, shape, y, h, secret in lay["extra"]:
            self.rooms[rid] = Room(rid, name, shape, y, h, secret)
        self.halls = []
        for a, b, kind, pts, w, stairs, gate in lay["halls"]:
            ra, rb = self.rooms[a], self.rooms[b]
            hid = f"{a}-{b}"
            secret = ra.secret or rb.secret or kind == SECRET
            name = {GATE: "Gitteret", SECRET: "Den skjulte gangen", SHORTCUT: "Snarveien", STAIRS: "Trappa"}.get(kind, "Gangen")
            self.halls.append(Hall(hid, a, b, kind, pts, w, ra.y, rb.y, stairs, gate, name, secret))
        self.marks = lay["marks"](None)
        self.mb = MB()
        self.colliders = []
        self.lights = []
        self.patches = []  # (polygon, plan (a, b, c) så y = a·x + b·z + c, rom, port)
        self.tri_patches = []  # ferdige trekanter (stigen opp i beinhuset)
        self.rng = random.Random(n * 7919)

    # -- Kolliderne --

    def box(self, center, size, yaw=0.0):
        self.colliders.append({"box": {"center": [rnd(c) for c in center], "size": [rnd(s) for s in size], "yaw": rnd(yaw, 4)}})

    def hull(self, pts):
        self.colliders.append({"hull": [[rnd(c) for c in p] for p in pts]})

    def prop(self, model, x, y, z, yaw=0.0):
        """Pynt fra byggesettet som egen modell (sarkofagen fra arket, med bakt
        tekstur): motoren setter den ut etter merket `prop`. Kolliderne og
        nav-hindringen legges der den kalles fra, som før."""
        self.marks.append({"kind": "prop", "label": model, "pos": [rnd(x), rnd(y), rnd(z)], "yaw": rnd(yaw, 4)})

    def wall_box(self, a, b, y0, y1, out):
        """Murkollider langs (a → b) i planen, bak flaten (mot `out`)."""
        L = math.dist(a, b)
        if L < 0.05 or y1 - y0 < 0.05:
            return
        dx, dz = (b[0] - a[0]) / L, (b[1] - a[1]) / L
        c = ((a[0] + b[0]) / 2 + out[0] * WALL_T / 2, (y0 + y1) / 2, (a[1] + b[1]) / 2 + out[1] * WALL_T / 2)
        self.box(c, (L + WALL_T, y1 - y0, WALL_T), math.atan2(-dz, dx))

    def light(self, pos, color, rng_m, room, flicker=True, intensity=2.0):
        self.lights.append({"pos": [rnd(p) for p in pos], "color": list(color), "range": rng_m, "flicker": flicker, "intensity": intensity, "room": room})

    def torch(self, room, p, n, intensity=2.2, reach=9.0):
        f = K.torch(self.mb, p, n)
        self.light((f[0], f[1] + 0.1, f[2]), WARM, reach, room, True, intensity)

    def patch(self, poly, plane, room, gate=None):
        """En gangbar flate. Småbiter (flisene bak noe som står inntil
        muren) tas ikke med."""
        if poly.is_empty:
            return
        keep = [p for p in K._pieces(poly) if p.area > 0.4]
        if keep:
            self.patches.append((unary_union(keep), plane, room, gate))

    # -- Bygging --

    def build(self):
        self.attach_doors()
        self.pre()
        for r in self.rooms.values():
            if r.rect:
                self.rect_room(r)
        self.beinhus(self.rooms["beinhus"])
        for h in self.halls:
            self.hall(h)
        self.dress()
        self.room_patches()
        for h in self.halls:
            self.hall_patches(h)

    def pre(self):
        """Hullene i gulvene og de flate takene, før rommene bygges."""
        rm = self.rooms
        rm["sidegrav"].holes.append(sbox(-12.6, 20.6, -11.4, 21.8))
        rm["bronn"].holes.append(Point(-36.0, 42.0).buffer(0.85, 16))
        rm["nedgang"].holes.append(sbox(20.4, 96.8, 24.2, 99.2))
        rm["sjaktbunn"].flat = Point(-36.0, 42.0).buffer(0.85, 16)
        rm["stigebunn"].flat = sbox(-12.6, 20.6, -11.4, 21.8)

    def attach_doors(self):
        """Åpningene i rommene der gangene går inn."""
        for h in self.halls:
            for rid, p, q, y in ((h.a, h.pts[0], h.pts[1], h.ya), (h.b, h.pts[-1], h.pts[-2], h.yb)):
                r = self.rooms[rid]
                if r.rect:
                    if abs(p[1] - r.z0) < 0.05:
                        side, u = "n", p[0]
                    elif abs(p[1] - r.z1) < 0.05:
                        side, u = "s", p[0]
                    elif abs(p[0] - r.x0) < 0.05:
                        side, u = "w", p[1]
                    elif abs(p[0] - r.x1) < 0.05:
                        side, u = "e", p[1]
                    else:
                        raise ValueError(f"gangen {h.id} treffer ikke muren i {rid} ({p})")
                else:
                    side, u = "arc", math.atan2(p[1] - r.c[1], p[0] - r.c[0])
                w = h.w
                kind = h.kind
                if kind == SHORTCUT and rid == h.a:
                    w = 1.6
                if kind == SECRET and rid == h.a:
                    w = 2.0
                r.doors.append((side, u, w, y, kind if rid == h.a else (DOOR if kind in (SHORTCUT, SECRET) else kind), h))

    def opening(self, w, y, kind):
        """Omrisset av en åpning (u fra midten, v absolutt)."""
        if kind == SECRET:
            return K.rect_profile(0.0, w, y, 2.6)
        if kind == SHORTCUT:
            return arch_profile(0.0, w, y, 1.5, n=10)
        return arch_profile(0.0, w, y, SPRING, n=12)

    def rect_room(self, r):
        mb = self.mb
        top = r.y + r.h
        # Gulvet, med hullene.
        floor = r.poly.difference(unary_union(r.holes)) if r.holes else r.poly
        mat = "slab" if r.id in ("gravgang", "soylegang", "dypgrav") else "floor"
        grid_face(mb, Horizontal(r.y), floor, mat)
        for piece in K._pieces(floor):
            self.floor_collider(piece, r.y)
        # Murene, med åpningene, og hvelvet over.
        span_x, span_z = r.x1 - r.x0, r.z1 - r.z0
        along_x = span_x >= span_z
        a = (span_z if along_x else span_x) / 2
        R = min(a * 0.75, 2.6)
        square = max(span_x, span_z) / min(span_x, span_z) < 1.45
        flat = r.flat is not None
        if flat:
            R = 0.0
        sides = {
            "n": (Plane((r.x0, r.z0), (1.0, 0.0), (0.0, 1.0)), span_x, r.x0, (0.0, -1.0)),
            "s": (Plane((r.x1, r.z1), (-1.0, 0.0), (0.0, -1.0)), span_x, r.x1, (0.0, 1.0)),
            "w": (Plane((r.x0, r.z1), (0.0, -1.0), (1.0, 0.0)), span_z, r.z1, (-1.0, 0.0)),
            "e": (Plane((r.x1, r.z0), (0.0, 1.0), (-1.0, 0.0)), span_z, r.z0, (1.0, 0.0)),
        }
        for side, (surf, length, origin_u, out) in sides.items():
            sgn = 1.0 if side in ("n", "e") else -1.0
            to_u = lambda v, sgn=sgn, origin_u=origin_u: (v - origin_u) * sgn
            openings, lintels = [], []
            for (ds, u, w, y, kind, h) in r.doors:
                if ds != side:
                    continue
                prof = [(to_u(u) + du, v) for du, v in self.opening(w, y, kind)]
                openings.append(Polygon(prof))
                if kind not in (SECRET, SHORTCUT):
                    K.portal(mb, surf, prof, mat="dressed", band=0.3, depth=0.12)
                apex = max(v for _, v in prof)
                lintels.append((to_u(u) - w / 2, to_u(u) + w / 2, apex, y))
            # Endeveggene under et tønnehvelv går opp i buen (lunetten).
            end_wall = (not square) and ((along_x and side in ("w", "e")) or ((not along_x) and side in ("n", "s")))
            if flat:
                K.wall(mb, surf, length, r.y - 0.05, top, openings, mat=self.wall_mat(r))
            elif end_wall:
                cw = a
                fn = lambda u, cw=cw, a=a, R=R, top=top: top + R * math.sqrt(max(0.0, 1 - ((u - cw) / a) ** 2))
                K.wall(mb, surf, length, r.y - 0.05, top, openings, mat=self.wall_mat(r), top_fn=fn, top_samples=[length * k / 16 for k in range(17)])
            elif square:
                fn = lambda u, length=length, R=R, top=top: top + R * math.sqrt(max(0.0, 1 - ((u - length / 2) / (length / 2)) ** 2))
                K.wall(mb, surf, length, r.y - 0.05, top, openings, mat=self.wall_mat(r), top_fn=fn, top_samples=[length * k / 16 for k in range(17)])
            else:
                K.wall(mb, surf, length, r.y - 0.05, top, openings, mat=self.wall_mat(r))
            # Kolliderne: murbitene mellom åpningene, og over dem.
            cuts = sorted(lintels)
            at = -WALL_T
            p_of = lambda u: surf.point(u, 0.0)
            for u0, u1, apex, y in cuts:
                if u0 > at:
                    a3, b3 = p_of(at), p_of(u0)
                    self.wall_box((a3[0], a3[2]), (b3[0], b3[2]), r.y - 0.5, top + R + 0.5, out)
                a3, b3 = p_of(u0), p_of(u1)
                self.wall_box((a3[0], a3[2]), (b3[0], b3[2]), apex, top + R + 0.5, out)
                if y > r.y + 0.05:
                    self.wall_box((a3[0], a3[2]), (b3[0], b3[2]), r.y - 0.5, y, out)
                at = u1
            a3, b3 = p_of(at), p_of(length + WALL_T)
            self.wall_box((a3[0], a3[2]), (b3[0], b3[2]), r.y - 0.5, top + R + 0.5, out)
        # Hvelvet (eller et flatt tak med hull for sjakta).
        if flat:
            grid_face(mb, Horizontal(top, up=False), r.poly.difference(r.flat), "vault")
        elif square:
            K.groin_bay(mb, r.x0, r.x1, r.z0, r.z1, top, R, mat="vault")
            K.groin_ribs(mb, r.x0, r.x1, r.z0, r.z1, top, R)
        elif along_x:
            cz = (r.z0 + r.z1) / 2
            K.barrel(mb, "x", r.x0, span_x, cz, top, R, a, mat="vault", step=1.0)
            nb = max(1, round(span_x / 4))
            for k in range(1, nb):
                K.barrel_rib(mb, "x", r.x0 + span_x * k / nb, cz, top, R, a)
        else:
            cx = (r.x0 + r.x1) / 2
            K.barrel(mb, "z", r.z0, span_z, cx, top, R, a, mat="vault", step=1.0)
            nb = max(1, round(span_z / 4))
            for k in range(1, nb):
                K.barrel_rib(mb, "z", r.z0 + span_z * k / nb, cx, top, R, a)
        r.ceil = top

    def wall_mat(self, r):
        return {"kapell": "fresco", "alter": "fresco", "kistekjeller": "wall_dark", "dypgrav": "wall_dark", "sjaktbunn": "wall_dark", "stigebunn": "wall_dark"}.get(r.id, "wall")

    def floor_collider(self, poly, y):
        """Gulvet som bokser (rektangler) eller konvekse omhyllinger."""
        x0, z0, x1, z1 = poly.bounds
        if abs(poly.area - (x1 - x0) * (z1 - z0)) < 1e-3:
            self.box(((x0 + x1) / 2, y - 0.25, (z0 + z1) / 2), (x1 - x0, 0.5, z1 - z0))
            return
        for c in convex_parts(poly):
            pts = list(c.exterior.coords)[:-1]
            self.hull([(x, y, z) for x, z in pts] + [(x, y - 0.5, z) for x, z in pts])

    def hall(self, h):
        mb = self.mb
        a = h.w / 2
        n = len(h.segs)
        # Gulvet og trappene.
        for i, (p0, p1, d, L) in enumerate(h.segs):
            lat = (-d[1], d[0])
            e0 = a if i > 0 else 0.0
            e1 = a if i < n - 1 else 0.0
            pieces = [(-e0, L + e1)]
            if h.stairs and h.stairs[0] == i:
                _, sa, sb = h.stairs
                pieces = [(-e0, sa), (sa, sb), (sb, L + e1)]
            for s0, s1 in pieces:
                if s1 - s0 < 1e-3:
                    continue
                y0, y1 = h.floor_s(i, s0 + 1e-4), h.floor_s(i, s1 - 1e-4)
                rect = h.seg_rect(i, s0, s1)
                if abs(y1 - y0) < 1e-3:
                    grid_face(mb, Horizontal(y0), rect, "floor")
                    self.floor_collider(rect, y0)
                else:
                    steps = max(2, round(abs(y1 - y0) / 0.2))
                    # Trinnene går STAIR_IN inn i murene på begge sider (aldri tangent: da
                    # blir det glipper med lys gjennom mellom trinn og mur).
                    K.steps(mb, p0, d, lat, h.w + 2 * STAIR_IN, s0, s1, y0, y1, steps, mat="dressed")
                    # Rampa under trinnene.
                    pts = []
                    for s, y in ((s0, y0), (s1, y1)):
                        for l in (-a, a):
                            x, z = p0[0] + d[0] * s + lat[0] * l, p0[1] + d[1] * s + lat[1] * l
                            pts += [(x, y + (0.02 if s == s0 and y1 < y0 else 0.0), z), (x, min(y0, y1) - 0.5, z)]
                    self.hull(pts)
                    # Undersida av trinnene (synlig ingen steder), og sidene.
        # Murene langs sidene: kantene av fotavtrykket som ikke er endene.
        r0 = h.seg_rect(0, 0.0, 0.001).exterior.coords
        r1 = h.seg_rect(n - 1, h.segs[-1][3] - 0.001, h.segs[-1][3]).exterior.coords
        ends = [LineString([r0[0], r0[3]]).buffer(0.02), LineString([r1[1], r1[2]]).buffer(0.02)]
        ring = list(h.poly.exterior.coords)
        for k in range(len(ring) - 1):
            pa, pb = ring[k], ring[k + 1]
            seg = LineString([pa, pb])
            if seg.length < 1e-3 or any(e.contains(seg) for e in ends):
                continue
            mid = ((pa[0] + pb[0]) / 2, (pa[1] + pb[1]) / 2)
            dx, dz = (pb[0] - pa[0]) / seg.length, (pb[1] - pa[1]) / seg.length
            nrm = (-dz, dx)
            if not h.poly.contains(Point(mid[0] + nrm[0] * 0.05, mid[1] + nrm[1] * 0.05)):
                nrm = (dz, -dx)
            surf = Plane(pa, (dx, dz), nrm)
            L = seg.length
            us = set([0.0, L] + [L * q / max(1, math.ceil(L / 0.5)) for q in range(math.ceil(L / 0.5) + 1)])
            # Muren går opp til vederlaget til hvelvet, som er trappet i skiver på 1 m der
            # gulvet stiger (ikke til gulvet + SPRING, som er en rampe: mellom rampa og det
            # trappete vederlaget sto det kiler av himmel over hvert trinn). Prøv tett ved
            # skivegrensene, så toppen følger trinnene.
            for i, (p0, p1, d, Ls) in enumerate(h.segs):
                m = max(1, math.ceil(Ls - 1e-6))
                for q in range(m + 1):
                    px, pz = p0[0] + d[0] * Ls * q / m, p0[1] + d[1] * Ls * q / m
                    u = (px - pa[0]) * dx + (pz - pa[1]) * dz
                    for uu in (u - 1e-3, u + 1e-3):
                        if 0.0 < uu < L:
                            us.add(uu)
                if i < n - 1:  # buen i lunetten i hjørnefaget: prøver hver 10 cm
                    uc = (p1[0] - pa[0]) * dx + (p1[1] - pa[1]) * dz
                    for kq in range(-int(a / 0.1) - 1, int(a / 0.1) + 2):
                        uu = uc + 0.1 * kq
                        if 0.0 < uu < L:
                            us.add(uu)
            us = sorted(us)
            fl = [self.hall_floor(h, surf.point(u, 0.0, 0.3)) for u in us]
            tops = [self.hall_spring(h, surf.point(u, 0.0, 0.3)) + 0.03 for u in us]
            outline = Polygon([(u, f - 0.3) for u, f in zip(us, fl)] + [(u, t) for u, t in reversed(list(zip(us, tops)))])
            grid_face(mb, surf, outline, "wall")
            lo = min(fl) - 0.5
            hi = max(fl) + SPRING + a + 0.4
            self.wall_box(pa, pb, lo, hi, (-nrm[0], -nrm[1]))
        # Hvelvet: tønne langs hvert strekk, i skiver på 1 m der gulvet
        # stiger, og krysshvelv i hjørnene.
        for i, (p0, p1, d, L) in enumerate(h.segs):
            axis = "x" if abs(d[0]) > 0.5 else "z"
            cw = p0[1] if axis == "x" else p0[0]
            s_lo, s_hi = (0.0, L)
            m = max(1, math.ceil(L - 1e-6))
            # Krysshvelvet i hjørnet før strekket står på gulvet ved starten; er første
            # skive høyere (trappa begynner i den), må trinnet imellom også lukkes.
            prev = h.floor_s(i, 1e-4) + SPRING if i > 0 else None
            for q in range(m):
                s0, s1 = s_lo + L * q / m, s_lo + L * (q + 1) / m
                S = max(h.floor_s(i, s0 + 1e-4), h.floor_s(i, s1 - 1e-4)) + SPRING
                g0 = (p0[0] + d[0] * s0, p0[1] + d[1] * s0)
                g1 = (p0[0] + d[0] * s1, p0[1] + d[1] * s1)
                c0 = min(g0[0], g1[0]) if axis == "x" else min(g0[1], g1[1])
                K.barrel(mb, axis, c0, abs(s1 - s0), cw, S, a, a, mat="vault", step=1.0)
                if prev is not None and abs(prev - S) > 1e-3:
                    self.vault_step(axis, c0 if (d[0] + d[1]) > 0 else c0 + abs(s1 - s0), cw, prev, S, a)
                prev = S
            if i < n - 1:
                cx, cz = p1
                S = h.floor_s(i, L) + SPRING
                if prev is not None and abs(prev - S) > 1e-3:  # siste skive mot krysshvelvet i hjørnet
                    g_end = (p0[0] + d[0] * L, p0[1] + d[1] * L)
                    self.vault_step(axis, g_end[0] if axis == "x" else g_end[1], cw, prev, S, a)
                K.groin_bay(mb, cx - a, cx + a, cz - a, cz + a, S, a, mat="vault")
                K.groin_ribs(mb, cx - a, cx + a, cz - a, cz + a, S, a)
        # Muren i enden der åpningen er smalere enn gangen (døra, den
        # falske veggen), sett fra gangen.
        if h.kind in (SHORTCUT, SECRET):
            p0, p1, d, L = h.segs[0]
            lat = (-d[1], d[0])
            surf = Plane((p0[0] - lat[0] * a, p0[1] - lat[1] * a), lat, d)
            section = Polygon([(u + a, v) for u, v in arch_profile(0.0, h.w, h.ya, SPRING, n=12)])
            ow = 1.6 if h.kind == SHORTCUT else 2.0
            hole = Polygon([(u + a, v) for u, v in self.opening(ow, h.ya, h.kind)])
            grid_face(mb, surf, section.difference(hole), "wall")

    def hall_patches(self, h):
        """Navigasjonen i gangen: fotavtrykket forlenget 0,8 m inn i rommene
        i endene, 0,4 m inn fra murene, uten det som alt er rommenes, og delt
        der gulvet skifter."""
        a = h.w / 2
        n = len(h.segs)
        p0, p1, d, L = h.segs[0]
        ext = [h.seg_rect(0, -2 * INSET, 0.0)]
        q0, q1, d2, L2 = h.segs[-1]
        ext.append(h.seg_rect(n - 1, L2, L2 + 2 * INSET))
        walk = unary_union([h.poly] + ext).buffer(-INSET, join_style="mitre")
        for rid in (h.a, h.b):
            if self.rooms[rid].walk is not None:
                walk = walk.difference(self.rooms[rid].walk)
        for i, (p0, p1, d, L) in enumerate(h.segs):
            s0_all = a if i > 0 else -2 * INSET
            s1_all = L + a if i < n - 1 else L + 2 * INSET
            cuts = [(s0_all, s1_all)]
            if h.stairs and h.stairs[0] == i:
                _, sa, sb = h.stairs
                cuts = [(s0_all, sa), (sa, sb), (sb, s1_all)]
            for s0, s1 in cuts:
                if s1 - s0 < 1e-3:
                    continue
                part = walk.intersection(h.seg_rect(i, s0, s1, h.w + 1.0))
                y0, y1 = h.floor_s(i, s0 + 1e-4), h.floor_s(i, s1 - 1e-4)
                if abs(y1 - y0) < 1e-3:
                    plane = (0.0, 0.0, y0)
                else:
                    k = (y1 - y0) / (s1 - s0)
                    # y = y0 + k·((x − x0)·dx + (z − z0)·dz − s0)
                    plane = (k * d[0], k * d[1], y0 - k * (p0[0] * d[0] + p0[1] * d[1] + s0))
                gate = h.gate if h.kind in (GATE, SECRET) else None
                if h.kind == SHORTCUT and i == 0:
                    # Bare den første meteren ved døra stenges.
                    near = part.intersection(h.seg_rect(0, -2.0, 1.0, h.w + 1.0))
                    rest = part.difference(h.seg_rect(0, -2.0, 1.0, h.w + 1.0))
                    self.patch(near, plane, h.id, h.gate)
                    self.patch(rest, plane, h.id, None)
                    continue
                self.patch(part, plane, h.id, gate)

    def vault_step(self, axis, s, cw, S0, S1, a, n=12):
        lo, hi = min(S0, S1), max(S0, S1)
        ts = K.tsamples(n)
        for t0, t1 in zip(ts, ts[1:]):
            ya0 = math.sqrt(max(0.0, 1 - t0 * t0)) * a
            ya1 = math.sqrt(max(0.0, 1 - t1 * t1)) * a
            P = lambda t, y: (s, y, cw + a * t) if axis == "x" else (cw + a * t, y, s)
            pts = [P(t0, lo + ya0), P(t1, lo + ya1), P(t1, hi + ya1), P(t0, hi + ya0)]
            f = (1.0, 0.0, 0.0) if axis == "x" else (0.0, 0.0, 1.0)
            self.mb.poly(pts, [(cw + a * t0, lo), (cw + a * t1, lo), (cw + a * t1, hi), (cw + a * t0, hi)], "vault", facing=f)
            self.mb.poly(pts[::-1], [(0, 0)] * 4, "vault", facing=(-f[0], -f[1], -f[2]))

    def hall_floor(self, h, p):
        return h.floor_at(p[0], p[2])

    def hall_spring(self, h, p):
        """Vederlaget til hvelvet over punktet p i gangen: tønnehvelvet ligger i skiver
        på 1 m med høyden etter det høyeste gulvet i skiva (som i `hall`), og
        krysshvelvene i hjørnene etter gulvet ved enden av strekket. Murene går
        opp hit, så det aldri står luft mellom mur og hvelv."""
        px, pz = p[0], p[2]
        best = None
        for i, (p0, p1, d, L) in enumerate(h.segs):
            s = (px - p0[0]) * d[0] + (pz - p0[1]) * d[1]
            sc = min(max(s, 0.0), L)
            dist = math.hypot(px - (p0[0] + d[0] * sc), pz - (p0[1] + d[1] * sc))
            if best is None or dist < best[0] - 1e-6:
                best = (dist, i, s, L)
        _, i, s, L = best
        a = h.w / 2
        n = len(h.segs)
        # I krysshvelvets fag i et hjørne (±a om knekkpunktet) er de lukkede sidene
        # lunetter: muren går helt opp til buen, S + a·sqrt(1 − t²), ellers står det
        # en halvsirkel av luft mellom vederlaget og hvelvet.
        bay = None
        if s >= L - a - 1e-6 and i < n - 1:
            bay = (h.segs[i][1], h.floor_s(i, L - 1e-4) + SPRING)
        elif s <= a + 1e-6 and i > 0:
            bay = (h.segs[i][0], h.floor_s(i, 1e-4) + SPRING)
        if bay is not None:
            (cx, cz), S = bay
            t = (pz - cz) / a if abs(abs(px - cx) - a) < abs(abs(pz - cz) - a) else (px - cx) / a
            return S + a * math.sqrt(max(0.0, 1 - min(1.0, t * t) ** 1))
        if s <= 1e-6:
            return h.floor_s(i, 1e-4) + SPRING
        if s >= L - 1e-6:
            return h.floor_s(i, L - 1e-4) + SPRING
        m = max(1, math.ceil(L - 1e-6))
        q = min(m - 1, int(s * m / L))
        s0, s1 = L * q / m, L * (q + 1) / m
        return max(h.floor_s(i, s0 + 1e-4), h.floor_s(i, s1 - 1e-4)) + SPRING

    # -- Beinhuset: rund sal med galleri og trapp langs muren --

    def beinhus(self, r):
        mb = self.mb
        B = BEINHUS
        c, R, gy = B["c"], B["r"], B["gallery_y"]
        a0, a1 = B["stair_a0"], B["stair_a1"]
        top = r.y + r.h
        # Gulvet.
        grid_face(mb, Horizontal(r.y), r.poly, "floor")
        self.hull([(x, r.y, z) for x, z in list(r.poly.exterior.coords)[:-1:3]] + [(x, r.y - 0.5, z) for x, z in list(r.poly.exterior.coords)[:-1:3]])
        # Muren med åpningene, beinhusteksturen og hyllene.
        cyl = Cyl(c, R, a0=0.0, inward=True)
        circ = 2 * math.pi * R
        openings, cuts = [], []
        for (side, ang, w, y, kind, h) in r.doors:
            u = (ang % (2 * math.pi)) * R
            prof = [(u + du, v) for du, v in self.opening(w, y, kind)]
            openings.append(Polygon(prof))
            K.portal(mb, cyl, prof, mat="dressed", band=0.3, depth=0.12)
            cuts.append((u - w / 2 - 0.1, u + w / 2 + 0.1, max(v for _, v in prof)))
        K.wall(mb, cyl, circ, r.y - 0.05, top, openings, mat="bones", cell=1.0)
        # Hyllene med hodeskaller, i to rader under galleriet og over det.
        shelf_cyl = Cyl(c, R, a0=0.0, inward=True)
        segs = 72
        for k in range(segs):
            u0, u1 = circ * k / segs, circ * (k + 1) / segs
            if any(u0 < cu1 and u1 > cu0 for cu0, cu1, _ in cuts):
                continue
            am = ((u0 + u1) / 2 / R) % (2 * math.pi)
            on_stair = am >= a0 or am <= a1 - 2 * math.pi
            for lv in (r.y + 0.9, r.y + 2.0, r.y + 3.2):
                if lv < gy - 0.4 and not on_stair:
                    K.ossuary_shelves(mb, Cyl(c, R, a0=u0 / R, inward=True), u1 - u0, r.y, top, [lv], step=u1 - u0)
            for lv in (gy + 1.4, gy + 2.6):
                K.ossuary_shelves(mb, Cyl(c, R, a0=u0 / R, inward=True), u1 - u0, gy, top, [lv], step=u1 - u0)
        # Murkolliderne rundt (utenom åpningene).
        for k in range(segs):
            u0, u1 = circ * k / segs, circ * (k + 1) / segs
            pa, pb = cyl.point(u0, 0.0), cyl.point(u1, 0.0)
            hit = [cu for cu in cuts if u0 < cu[1] and u1 > cu[0]]
            mid = ((pa[0] + pb[0]) / 2, (pa[2] + pb[2]) / 2)
            out = ((mid[0] - c[0]) / R, (mid[1] - c[1]) / R)
            if hit:
                self.wall_box((pa[0], pa[2]), (pb[0], pb[2]), hit[0][2], top + 4.5, out)
            else:
                self.wall_box((pa[0], pa[2]), (pb[0], pb[2]), r.y - 0.5, top + 4.5, out)
        # Kuppelen.
        hfn = K.dome(mb, c, R, top, top + 3.6, mat="vault", rings=8, seg=48)
        K.dome_ribs(mb, c, R, hfn, count=12)
        # Galleriet: ringen fra a1 rundt til a0, med rekkverk, og trappa.
        ri = B["ring_in"]
        g0, g1 = a1 - 2 * math.pi, a0  # 15° → 300°
        steps = 48
        ring_pts = lambda q, aa, ab, n: [(c[0] + q * math.cos(aa + (ab - aa) * k / n), c[1] + q * math.sin(aa + (ab - aa) * k / n)) for k in range(n + 1)]
        outer = ring_pts(R, g0, g1, steps)
        inner = ring_pts(ri, g0, g1, steps)
        gal = Polygon(outer + inner[::-1])
        grid_face(mb, Horizontal(gy), gal, "floor")
        under = Horizontal(gy - 0.3, up=False)
        grid_face(mb, under, gal, "dressed")
        for k in range(steps):
            q = [inner[k], inner[k + 1]]
            mb.quad((q[0][0], gy - 0.3, q[0][1]), (q[1][0], gy - 0.3, q[1][1]), (q[1][0], gy, q[1][1]), (q[0][0], gy, q[0][1]), "dressed",
                    facing=(c[0] - q[0][0], 0, c[1] - q[0][1]))
            pa, pb = outer[k], outer[k + 1]
            pc, pd = inner[k + 1], inner[k]
            quad = [pa, pb, pc, pd]
            self.hull([(x, gy, z) for x, z in quad] + [(x, gy - 0.3, z) for x, z in quad])
            # Konsollene under galleriet.
            if k % 4 == 0:
                aa = g0 + (g1 - g0) * k / steps
                p = (c[0] + (ri + 0.6) * math.cos(aa), c[1] + (ri + 0.6) * math.sin(aa))
                K.obox(mb, (p[0], gy - 0.9, p[1]), (0.35, 0.6, 1.2), math.pi / 2 - aa, "dressed")
        K.parapet(mb, inner, gy, h=0.95, t=0.26)
        # Rekkverket over enden av galleriet, over foten av trappa.
        end = [(c[0] + ri * math.cos(g1), c[1] + ri * math.sin(g1)), (c[0] + (R - 0.1) * math.cos(g1), c[1] + (R - 0.1) * math.sin(g1))]
        K.parapet(mb, end, gy, h=0.95, t=0.26)
        self.wall_box(end[0], end[1], gy, gy + 1.0, (0.0, 0.0))
        for k in range(steps):
            self.wall_box(inner[k], inner[k + 1], gy, gy + 1.0, (0.0, 0.0))
        # Trappa langs muren fra gulvet (ved a0) opp til galleriet (ved a1).
        si = B["stair_in"]
        n = 25
        yfn = K.spiral_stair(mb, c, si, R, a0, a1, r.y, gy, n, mat="dressed", inner_wall=True)
        sp = ring_pts(si, a0, a1, n)
        K.parapet(mb, sp, r.y, h=0.95, t=0.22, base_y=[r.y + (gy - r.y) * k / n for k in range(n + 1)])
        for k in range(n):
            aa, ab = a0 + (a1 - a0) * k / n, a0 + (a1 - a0) * (k + 1) / n
            ya, yb = r.y + (gy - r.y) * k / n, r.y + (gy - r.y) * (k + 1) / n
            P = lambda q, ang: (c[0] + q * math.cos(ang), c[1] + q * math.sin(ang))
            quad = [P(si, aa), P(R, aa), P(R, ab), P(si, ab)]
            self.hull([(x, yb if j >= 2 else ya + 0.02, z) for j, (x, z) in enumerate(quad)] + [(x, r.y - 0.2, z) for x, z in quad])
            self.wall_box(P(si, aa), P(si, ab), r.y, yb + 1.0, (0.0, 0.0))
            # Navigasjonen på trappa: to trekanter per trinn.
            qi, qo = si + INSET, R - INSET
            A, Bq, C, D = P(qi, aa), P(qo, aa), P(qo, ab), P(qi, ab)
            self.tri_patches.append(([(A[0], ya, A[1]), (Bq[0], ya, Bq[1]), (C[0], yb, C[1])], "beintrapp"))
            self.tri_patches.append(([(A[0], ya, A[1]), (C[0], yb, C[1]), (D[0], yb, D[1])], "beintrapp"))
        # Navigasjonen: gulvet (uten feltet under trappa) og galleriet.
        under_stair = Polygon(ring_pts(si - INSET, a0, a1, n) + ring_pts(R + 1, a0, a1, n)[::-1])
        floor = Point(c).buffer(R - INSET, 72).difference(under_stair)
        r.floor_walk = floor
        g1w = g1 - INSET / (ri + 1.0)
        gwalk = Polygon(ring_pts(R - INSET, g0, g1w, steps) + ring_pts(ri + INSET, g0, g1w, steps)[::-1])
        self.patch(gwalk, (0.0, 0.0, gy), "galleri")
        self.gallery_room = Polygon(ring_pts(R, 0.0, 2 * math.pi, 72) + ring_pts(ri - 0.05, 0.0, 2 * math.pi, 72)[::-1])
        self.stair_room = Polygon(ring_pts(R, a0, a1, 16) + ring_pts(si, a0, a1, 16)[::-1])
        # Lysene: fakler rundt muren og glør i et fat midt i salen.
        for deg in (150, 210, 30, 110):
            ang = math.radians(deg)
            p = (c[0] + (R - 0.02) * math.cos(ang), r.y + 2.2, c[1] + (R - 0.02) * math.sin(ang))
            self.torch("beinhus", p, (-math.cos(ang), -math.sin(ang)), intensity=2.0, reach=10.0)
        for deg in (240, 330, 60, 170):
            ang = math.radians(deg)
            p = (c[0] + (R - 0.02) * math.cos(ang), gy + 1.9, c[1] + (R - 0.02) * math.sin(ang))
            self.torch("beinhus", p, (-math.cos(ang), -math.sin(ang)), intensity=1.6, reach=8.0)
        # Fatet med glør.
        K.cylinder(mb, c, 0.55, r.y, r.y + 0.75, "iron", n=10, caps=(True, False), smooth=False)
        K.cylinder(mb, c, 0.5, r.y + 0.75, r.y + 0.8, "fire_torch", n=10, caps=(True, False), smooth=False)
        self.box((c[0], r.y + 0.4, c[1]), (1.1, 0.8, 1.1))
        r.obstacles.append(Point(c).buffer(0.6))
        self.light((c[0], r.y + 1.4, c[1]), (1.0, 0.45, 0.18), 9.0, "beinhus", True, 2.4)
        r.ceil = gy - 0.3  # under galleriet

    # -- Det som står i rommene --

    def dress(self):
        mb = self.mb
        rm = self.rooms
        rng = self.rng
        # Kapellet: alteret til hvilestedet settes av motoren. Fakler ved
        # trappa opp, kalkmalt mur.
        k = rm["kapell"]
        self.torch("kapell", (-4.98, 2.4, -1.0), (1.0, 0.0))
        self.torch("kapell", (4.98, 2.4, 1.5), (-1.0, 0.0), intensity=1.6)
        for x, z in ((-3.6, 2.8),):
            K.candlestick(mb, x, z, k.y, arms=3)
            k.obstacles.append(Point(x, z).buffer(0.35))
            self.box((x, k.y + 0.7, z), (0.5, 1.4, 0.5))
            self.light((x, k.y + 1.55, z), CANDLE, 5.0, "kapell", True, 1.2)
        # Gravgangen: nisjer i begge murene, stearinlys i hver tredje.
        g = rm["gravgang"]
        for side, xw, nrm in (("w", g.x0, (1.0, 0.0)), ("e", g.x1, (-1.0, 0.0))):
            surf = Plane((xw, g.z1), (0.0, -1.0), nrm) if side == "w" else Plane((xw, g.z0), (0.0, 1.0), nrm)
            length = g.z1 - g.z0
            for j, zc in enumerate(range(10, 35, 3)):
                if abs(zc - 20) < 2.5:
                    continue
                u = (g.z1 - zc) if side == "w" else (zc - g.z0)
                for lvl, v0 in enumerate((g.y + 0.35, g.y + 1.25)):
                    K.niche(mb, surf, u, v0, w=1.7, h=0.5, depth=0.5, sealed=(j + lvl) % 3 == 1, rng=rng)
                if j % 3 == 0:
                    p = surf.point(u, g.y + 2.15, 0.12)
                    K.candle(mb, p[0], p[2], g.y + 2.1)
                    box_c = surf.point(u, g.y + 2.05, 0.12)
                    K.box(mb, (box_c[0] - 0.12, g.y + 2.0, box_c[2] - 0.12), (box_c[0] + 0.12, g.y + 2.1, box_c[2] + 0.12), "dressed")
                    self.light((p[0], g.y + 2.45, p[2]), CANDLE, 4.5, "gravgang", True, 1.0)
        # Sidegraven: en sarkofag, kald.
        s = rm["sidegrav"]
        self.prop("kit_sarcophagus", -9.5, s.y, 22.3, math.pi / 2)
        s.obstacles.append(sbox(-10.6, 21.8, -8.4, 22.8))
        self.box((-9.5, s.y + 0.5, 22.3), (2.2, 1.0, 1.0))
        s.obstacles.append(sbox(-12.6, 20.6, -11.4, 21.8))
        self.light((-13.8, 1.8, 18.0), COLD, 6.0, "sidegrav", True, 0.8)
        # Krysset: fakler på søylene ved gangene.
        x = rm["krysset"]
        self.torch("krysset", (-5.48, x.y + 2.4, 41.0), (1.0, 0.0))
        self.torch("krysset", (5.48, x.y + 2.4, 41.0), (-1.0, 0.0))
        self.torch("krysset", (-2.5, x.y + 2.4, 49.48), (0.0, -1.0), intensity=1.8)
        K.rubble(mb, 4.0, 47.8, x.y, r=0.9, h=0.4, seed=3)
        x.obstacles.append(Point(4.0, 47.8).buffer(0.9))
        self.box((4.0, x.y + 0.2, 47.8), (1.6, 0.4, 1.6))
        # Pilgangen: pilhullene i nordmuren over trykkplata.
        pg = rm["pilgang"]
        surf = Plane((pg.x0, pg.z0), (1.0, 0.0), (0.0, 1.0))
        K.arrow_slots(mb, surf, -19.0 - pg.x0, pg.y + 0.9)
        self.light((-14.0, pg.y + 2.3, 45.4), COLD, 5.0, "pilgang", True, 0.7)
        # Brønnrommet: brønnen, rasmasse, kaldt trekk.
        b = rm["bronn"]
        K.well(mb, -36.0, 42.0, b.y, r_in=0.85, r_out=1.3, curb=0.5, depth=b.y - (-8 + 3.2))
        b.obstacles.append(Point(-36.0, 42.0).buffer(1.3, 16))
        for k2 in range(8):
            ang = 2 * math.pi * k2 / 8
            self.box((-36.0 + 1.07 * math.cos(ang), b.y + 0.25, 42.0 + 1.07 * math.sin(ang)), (0.46, 0.5, 0.6), -ang)
        K.rubble(mb, -40.0, 48.0, b.y, r=1.2, h=0.6, seed=7)
        b.obstacles.append(Point(-40.0, 48.0).buffer(1.1))
        self.box((-40.0, b.y + 0.3, 48.0), (1.8, 0.6, 1.8))
        self.light((-31.0, b.y + 2.6, 39.0), COLD, 7.0, "bronn", True, 0.9)
        self.torch("bronn", (-30.52, b.y + 2.3, 47.0), (-1.0, 0.0), intensity=1.2, reach=7.0)
        # Brønnsjakta ned: muren rundt sjakta fra taket i sjaktbunnen.
        sb = rm["sjaktbunn"]
        sb.water = True
        mb.quad((-37.5, sb.y + 0.02, 40.5), (-34.5, sb.y + 0.02, 40.5), (-34.5, sb.y + 0.02, 43.5), (-37.5, sb.y + 0.02, 43.5), "water", facing=(0, 1, 0))
        self.light((-36.0, sb.y + 2.4, 41.0), COLD, 5.0, "sjaktbunn", False, 0.6)
        for k2 in range(16):
            ang0, ang1 = 2 * math.pi * k2 / 16, 2 * math.pi * (k2 + 1) / 16
            pa = (-36.0 + 0.85 * math.cos(ang0), 42.0 + 0.85 * math.sin(ang0))
            pb = (-36.0 + 0.85 * math.cos(ang1), 42.0 + 0.85 * math.sin(ang1))
            mid = ((pa[0] + pb[0]) / 2 + 36.0, (pa[1] + pb[1]) / 2 - 42.0)
            ln = math.hypot(*mid)
            self.wall_box(pa, pb, sb.y + 3.2, b.y + 0.5, (mid[0] / ln, mid[1] / ln))
        # Den glemte graven: sarkofager og stearinlys.
        d = rm["dypgrav"]
        for xx in (-29.0, -26.5):
            self.prop("kit_sarcophagus", xx, d.y, 23.5, 0.0)
            d.obstacles.append(sbox(xx - 0.55, 22.4, xx + 0.55, 24.6))
            self.box((xx, d.y + 0.5, 23.5), (1.0, 1.0, 2.2))
        K.candlestick(mb, -31.6, 26.8, d.y, arms=2)
        d.obstacles.append(Point(-31.6, 26.8).buffer(0.35))
        self.box((-31.6, d.y + 0.7, 26.8), (0.5, 1.4, 0.5))
        self.light((-31.6, d.y + 1.5, 26.8), CANDLE, 6.0, "dypgrav", True, 1.0)
        self.light((-24.2, d.y + 2.6, 21.0), COLD, 6.0, "dypgrav", True, 0.7)
        K.glitch_stone(mb, Plane((d.x0, d.z1), (0.0, -1.0), (1.0, 0.0)), 3.0, d.y + 1.5)
        # Under fallemmen: sjakta opp til sidegraven.
        u = rm["stigebunn"]
        for (pa, pb, out) in (((-12.6, 20.6), (-11.4, 20.6), (0, -1)), ((-11.4, 20.6), (-11.4, 21.8), (1, 0)), ((-11.4, 21.8), (-12.6, 21.8), (0, 1)), ((-12.6, 21.8), (-12.6, 20.6), (-1, 0))):
            nrm = (-out[0], -out[1])
            surf = Plane(pa, ((pb[0] - pa[0]) / 1.2, (pb[1] - pa[1]) / 1.2), nrm)
            grid_face(mb, surf, sbox(0.0, u.y + 3.2, 1.2, 0.0), "wall_dark")
            self.wall_box(pa, pb, u.y + 3.2, 0.0, out)
        self.light((-12.0, u.y + 2.2, 24.5), COLD, 5.0, "stigebunn", False, 0.5)
        # Søylegangen: to rader søyler, sarkofager imellom, fakler på søylene.
        so = rm["soylegang"]
        for zc in range(57, 86, 4):
            for xc in (-2.6, 2.6):
                K.pillar(mb, xc, zc, so.y, so.y + so.h, size=0.7)
                so.obstacles.append(sbox(xc - 0.55, zc - 0.55, xc + 0.55, zc + 0.55))
                self.box((xc, so.y + so.h / 2, zc), (1.0, so.h, 1.0))
        for zc in (59, 63, 67, 77, 81):
            for xc in (-3.4, 3.4):
                if xc < 0 and 69 <= zc <= 75:
                    continue
                self.prop("kit_sarcophagus", xc, so.y, zc, 0.0)
                so.obstacles.append(sbox(xc - 0.6, zc - 1.25, xc + 0.6, zc + 1.25))
                self.box((xc, so.y + 0.5, zc), (1.1, 1.0, 2.4))
        for zc in (61, 73, 85):
            for xc, nx in ((-2.6 + 0.47, 1.0), (2.6 - 0.47, -1.0)):
                self.torch("soylegang", (xc, so.y + 2.6, zc), (nx, 0.0), intensity=1.8, reach=8.0)
        # Kistekjelleren: hyller med likkister langs murene.
        kk = rm["kistekjeller"]
        for zc in (68.6, 70.8, 73.2, 75.4):
            for lvl in (0.0, 1.1):
                K.box(mb, (kk.x0, kk.y + lvl + 0.42, zc - 1.0), (kk.x0 + 0.75, kk.y + lvl + 0.5, zc + 1.0), "dressed")
                K.coffin(mb, kk.x0 + 0.38, zc, kk.y + lvl + 0.5, 0.0, L=1.8, W=0.6)
        kk.obstacles.append(sbox(kk.x0, 67.5, kk.x0 + 0.8, 76.5))
        self.box((kk.x0 + 0.4, kk.y + 1.0, 72.0), (0.8, 2.0, 9.0))
        for xc in (-19.5, -14.5):
            K.coffin(mb, xc, 76.0, kk.y, 0.2 if xc < -17 else -0.15, L=1.9, W=0.6)
            kk.obstacles.append(sbox(xc - 0.4, 75.0, xc + 0.4, 77.0))
            self.box((xc, kk.y + 0.25, 76.0), (0.7, 0.5, 2.0))
        self.light((-12.5, kk.y + 2.5, 68.0), COLD, 7.0, "kistekjeller", True, 0.8)
        # Alterkapellet: det store alteret mot sør med lysestaker, kalkmalt.
        al = rm["alter"]
        K.altar(mb, 0.0, 99.4, al.y, math.pi, w=2.6, d=1.1)
        al.obstacles.append(sbox(-1.85, 98.3, 1.85, 100.5))
        self.box((0.0, al.y + 0.55, 99.4), (2.9, 1.1, 1.3))
        for xc in (-2.4, 2.4):
            K.candlestick(mb, xc, 99.9, al.y, arms=3)
            al.obstacles.append(Point(xc, 99.9).buffer(0.35))
            self.box((xc, al.y + 0.7, 99.9), (0.5, 1.4, 0.5))
            self.light((xc, al.y + 1.6, 99.9), CANDLE, 6.0, "alter", True, 1.4)
        self.torch("alter", (-7.48, al.y + 2.6, 93.0), (1.0, 0.0))
        self.torch("alter", (7.48, al.y + 2.6, 93.0), (-1.0, 0.0))
        # Skattkammeret bak den falske veggen: lite lys.
        self.light((2.6, rm["skatt"].y + 2.0, 108.6), (1.0, 0.8, 0.45), 5.0, "skatt", True, 1.0)
        # Nedgangen: det blå lyset fra trappa ned og en fakkel ved døra.
        ng = rm["nedgang"]
        ng.obstacles.append(sbox(20.4, 96.1, 25.0, 99.9))
        self.light((22.6, ng.y + 0.4, 98.0), BLUE, 10.0, "nedgang", False, 3.0)
        self.torch("nedgang", (17.52, ng.y + 2.4, 95.0), (1.0, 0.0), intensity=1.6)
        self.torch("nedgang", (25.0, ng.y + 2.4, 93.52), (0.0, 1.0), intensity=1.4)
        # Snarveien: en fakkel midtveis.
        self.torch("nedgang-beinhus", (20.92, -2 + 1.9, 77.0), (1.0, 0.0), intensity=1.2, reach=8.0)
        self.torch("gravgang-krysset", (1.48, 0.0 + 1.9, 37.0), (-1.0, 0.0), intensity=1.0, reach=6.0)
        # Ting motoren setter ut, holdes fri i navigasjonen.
        for m in self.marks:
            p = m.get("pos")
            if m["kind"] in ("rest",):
                self.obstacle_at(p, sbox(p[0] - 1.25, p[2] - 1.0, p[0] + 1.25, p[2] + 0.9))
            elif m["kind"] in ("chest", "rare"):
                self.obstacle_at(p, Point(p[0], p[2]).buffer(0.55))
            elif m["kind"] in ("lever", "lore", "bonus"):
                self.obstacle_at(p, Point(p[0], p[2]).buffer(0.35))
            elif m["kind"] == "up":
                self.obstacle_at(p, sbox(p[0] - 1.6, p[2] - 1.2, p[0] + 1.6, p[2] + 0.05))

    def obstacle_at(self, p, poly):
        for r in self.rooms.values():
            if r.poly.buffer(0.5).contains(Point(p[0], p[2])) and abs(p[1] - r.y) < 1.0:
                r.obstacles.append(poly)
                return
        if self.rooms["beinhus"].poly.contains(Point(p[0], p[2])):
            self.gallery_obstacles = getattr(self, "gallery_obstacles", []) + [poly]

    def room_patches(self):
        for r in self.rooms.values():
            if r.rect:
                walk = r.poly.buffer(-INSET, join_style="mitre")
            else:
                walk = r.floor_walk
            if r.obstacles:
                walk = walk.difference(unary_union([o.buffer(INSET, join_style="mitre") for o in r.obstacles]))
            r.walk = walk
            self.patch(walk, (0.0, 0.0, r.y), r.id)
        go = getattr(self, "gallery_obstacles", [])
        if go:
            gy = BEINHUS["gallery_y"]
            for k, (poly, plane, room, gate) in enumerate(self.patches):
                if room == "galleri":
                    self.patches[k] = (poly.difference(unary_union([o.buffer(INSET, join_style="mitre") for o in go])), plane, room, gate)

    # -- Navigasjonsnettet --

    def nav(self):
        verts, index, polys, prooms, pgates = [], {}, [], [], []

        def v(x, y, z):
            k = (round(x, 3), round(y, 3), round(z, 3))
            if k not in index:
                index[k] = len(verts)
                verts.append([k[0], k[1], k[2]])
            return index[k]

        for poly, plane, room, gate in self.patches:
            for c in convex_parts(poly):
                pts = list(c.exterior.coords)[:-1]
                if c.area < 0.02:
                    continue
                y = lambda x, z: plane[0] * x + plane[1] * z + plane[2]
                polys.append([v(x, y(x, z), z) for x, z in pts])
                prooms.append(room)
                pgates.append(gate)
        for tri, room in self.tri_patches:
            polys.append([v(*p) for p in tri])
            prooms.append(room)
            pgates.append(None)
        # Biter som ikke henger sammen med resten (flisene rundt noe som står
        # inntil muren) tas bort, utenom i de hemmelige rommene.
        keep = self.reachable(verts, polys, prooms)
        secret = {r.id for r in self.rooms.values() if r.secret} | {h.id for h in self.halls if h.secret}
        idx = [i for i in range(len(polys)) if keep[i] or prooms[i] in secret]
        dropped = len(polys) - len(idx)
        if dropped:
            print(f"  {dropped} polygoner uten forbindelse tatt bort")
        polys = [polys[i] for i in idx]
        prooms = [prooms[i] for i in idx]
        pgates = [pgates[i] for i in idx]
        return {"verts": verts, "polys": polys, "poly_room": prooms, "poly_gate": pgates}

    def reachable(self, verts, polys, prooms):
        """Hvilke polygoner som kan nås fra startstedet (alle porter åpne):
        naboer deler en kant, eller deler et stykke av en kant (samme høyde)."""
        from shapely.strtree import STRtree

        geoms = [Polygon([(verts[k][0], verts[k][2]) for k in p]) for p in polys]
        tree = STRtree(geoms)
        ys = [[verts[k][1] for k in p] for p in polys]
        edges = [{(min(p[k], p[(k + 1) % len(p)]), max(p[k], p[(k + 1) % len(p)])) for k in range(len(p))} for p in polys]
        start = next(m for m in self.marks if m["kind"] == "start")["pos"]
        seen = [False] * len(polys)
        first = [i for i, g in enumerate(geoms) if g.buffer(0.05).contains(Point(start[0], start[2])) and abs(min(ys[i]) - start[1]) < 0.6]
        stack = first[:1]
        for i in stack:
            seen[i] = True
        while stack:
            i = stack.pop()
            for j in tree.query(geoms[i].buffer(0.01)):
                j = int(j)
                if seen[j]:
                    continue
                # En felles kant (samme hjørner), eller et felles stykke av
                # en kant på minst 0,2 m, som motoren krever.
                shared = edges[i] & edges[j]
                if shared:
                    a, b = next(iter(shared))
                    c = Point((verts[a][0] + verts[b][0]) / 2, (verts[a][2] + verts[b][2]) / 2)
                else:
                    c = edge_overlap(verts, polys[i], polys[j])
                if c is None:
                    continue
                hi = height_in(verts, polys[i], c.x, c.y)
                hj = height_in(verts, polys[j], c.x, c.y)
                if abs(hi - hj) < 0.3:
                    seen[j] = True
                    stack.append(j)
        return seen

    # -- Rommene i json --

    def rooms_json(self):
        out = []
        for r in self.rooms.values():
            coords = [[rnd(x), rnd(z)] for x, z in list(r.poly.exterior.coords)[:-1]]
            if not r.rect:
                coords = coords[::2]
            out.append({"id": r.id, "name": r.name, "poly": coords, "y": r.y, "h": rnd(min(r.h, r.ceil - r.y)), "secret": r.secret})
        B = BEINHUS
        top = self.rooms["beinhus"].y + self.rooms["beinhus"].h
        coords = [[rnd(x), rnd(z)] for x, z in list(self.gallery_room.exterior.coords)[:-1]]
        out.append({"id": "galleri", "name": "Galleriet", "poly": coords, "y": B["gallery_y"], "h": rnd(top - B["gallery_y"]), "secret": False})
        coords = [[rnd(x), rnd(z)] for x, z in list(self.stair_room.exterior.coords)[:-1]]
        ry = self.rooms["beinhus"].y
        out.append({"id": "beintrapp", "name": "Trappa opp til galleriet", "poly": coords, "y": ry, "h": rnd(top - ry), "secret": False})
        for h in self.halls:
            # Taket over det gangbare: buen 0,4 m fra muren, over det høyeste
            # gulvet i gangen.
            a = h.w / 2
            low = SPRING + a * math.sqrt(max(0.0, 1 - ((a - INSET) / a) ** 2))
            lo, hi = min(h.ya, h.yb), max(h.ya, h.yb)
            coords = [[rnd(x), rnd(z)] for x, z in list(h.poly.exterior.coords)[:-1]]
            out.append({"id": h.id, "name": h.name, "poly": coords, "y": lo, "h": rnd(hi - lo + low - 0.05), "secret": h.secret})
        return out


def edge_overlap(verts, p, q):
    """Som `overlap` i nav.rs: to kanter på linje (innen 3 cm) som overlapper
    minst 0,2 m. Gir midtpunktet av overlappen."""
    for k in range(len(p)):
        a, b = verts[p[k]], verts[p[(k + 1) % len(p)]]
        ax, az, bx, bz = a[0], a[2], b[0], b[2]
        L = math.hypot(bx - ax, bz - az)
        if L < 1e-3:
            continue
        dx, dz = (bx - ax) / L, (bz - az) / L
        for m in range(len(q)):
            c, d = verts[q[m]], verts[q[(m + 1) % len(q)]]
            off = lambda x, z: abs(dx * (z - az) - dz * (x - ax))
            if off(c[0], c[2]) > 0.03 or off(d[0], d[2]) > 0.03:
                continue
            tc = (c[0] - ax) * dx + (c[2] - az) * dz
            td = (d[0] - ax) * dx + (d[2] - az) * dz
            lo, hi = max(min(tc, td), 0.0), min(max(tc, td), L)
            if hi - lo >= 0.2:
                t = (lo + hi) / 2
                return Point(ax + dx * t, az + dz * t)
    return None


def height_in(verts, poly, x, z):
    """Høyden i en (plan) polygon i punktet, fra de tre første hjørnene."""
    a, b, c = (verts[k] for k in poly[:3])
    d = (b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2])
    if abs(d) < 1e-9:
        return a[1]
    w1 = ((x - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (z - a[2])) / d
    w2 = ((b[0] - a[0]) * (z - a[2]) - (x - a[0]) * (b[2] - a[2])) / d
    return a[1] * (1 - w1 - w2) + b[1] * w1 + c[1] * w2


def convex_parts(poly):
    """Deler en polygon (med hull) i konvekse biter: trianguler, og slå
    sammen naboer så lenge resultatet er konvekst (Hertel–Mehlhorn)."""
    out = []
    for p in K._pieces(poly):
        if p.area < 1e-4:
            continue
        p = p.simplify(0.002)
        coords = list(p.exterior.coords)[:-1]
        if not p.interiors and K._convex(coords) and len(coords) <= 12:
            out.append(Polygon(coords))
            continue
        tris = [list(t.exterior.coords)[:-1] for t in K._pieces(constrained_delaunay_triangles(p)) if t.area > 1e-6]
        out += [Polygon(c) for c in merge_convex(tris)]
    return out


def merge_convex(tris):
    key = lambda q: (round(q[0], 4), round(q[1], 4))
    polys = [[key(q) for q in t] for t in tris]
    for pl in polys:
        if signed_area(pl) < 0:
            pl.reverse()
    alive = [True] * len(polys)
    changed = True
    while changed:
        changed = False
        edges = {}
        for i, pl in enumerate(polys):
            if not alive[i]:
                continue
            for k in range(len(pl)):
                a, b = pl[k], pl[(k + 1) % len(pl)]
                edges.setdefault((min(a, b), max(a, b)), []).append(i)
        for e, owners in edges.items():
            if len(owners) != 2:
                continue
            i, j = owners
            if not (alive[i] and alive[j]) or i == j:
                continue
            m = join(polys[i], polys[j], e)
            if m and convex(m) and len(m) <= 10:
                polys[i] = m
                alive[j] = False
                changed = True
    return [p for p, a in zip(polys, alive) if a]


def signed_area(p):
    return sum(p[i][0] * p[(i + 1) % len(p)][1] - p[(i + 1) % len(p)][0] * p[i][1] for i in range(len(p))) / 2


def join(p, q, e):
    a, b = e
    n, m = len(p), len(q)
    for i in range(n):
        if {p[i], p[(i + 1) % n]} == {a, b}:
            break
    else:
        return None
    for j in range(m):
        if {q[j], q[(j + 1) % m]} == {a, b}:
            break
    else:
        return None
    # p: ... p[i] → p[i+1] ...; q går motsatt vei over kanten.
    out = [p[(i + 1 + k) % n] for k in range(n)]  # starter i p[i+1], slutter i p[i]
    qq = [q[(j + 2 + k) % m] for k in range(m - 2)]  # q uten kanten
    res = out + qq
    # Fjern hjørner på rette linjer.
    clean = []
    for k in range(len(res)):
        a0, b0, c0 = res[k - 1], res[k], res[(k + 1) % len(res)]
        cr = (b0[0] - a0[0]) * (c0[1] - b0[1]) - (b0[1] - a0[1]) * (c0[0] - b0[0])
        if abs(cr) > 1e-7:
            clean.append(b0)
    return clean if len(clean) >= 3 else None


def convex(p):
    n = len(p)
    for k in range(n):
        a, b, c = p[k], p[(k + 1) % n], p[(k + 2) % n]
        if (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]) < -1e-9:
            return False
    return True


# --- Bitene ---------------------------------------------------------------------


def chunks(mb, n):
    """Flatene fordelt på biter på 16 × 16 m etter midtpunktet."""
    xs = [p[0] for f in mb.faces for p in f[0]]
    zs = [p[2] for f in mb.faces for p in f[0]]
    x0, z0 = min(xs), min(zs)
    groups = {}
    for f in mb.faces:
        cx = sum(p[0] for p in f[0]) / len(f[0])
        cz = sum(p[2] for p in f[0]) / len(f[0])
        k = (math.floor((cx - x0) / CHUNK), math.floor((cz - z0) / CHUNK))
        groups.setdefault(k, MB()).faces.append(f)
    out = {}
    for i, k in enumerate(sorted(groups)):
        name = f"crypt_{n:02d}_{chr(ord('a') + i // 26)}{chr(ord('a') + i % 26)}"
        out[name] = groups[k]
    return out


# --- Bildet av nettet -----------------------------------------------------------


def draw(level, data, path):
    from PIL import Image, ImageDraw

    rooms = data["rooms"]
    xs = [p[0] for r in rooms for p in r["poly"]]
    zs = [p[1] for r in rooms for p in r["poly"]]
    x0, x1, z0, z1 = min(xs) - 4, max(xs) + 4, min(zs) - 4, max(zs) + 4
    s = min(1000 / (x1 - x0), 1000 / (z1 - z0))
    W, H = int((x1 - x0) * s), int((z1 - z0) * s)
    img = Image.new("RGB", (W, H), (24, 26, 32))
    d = ImageDraw.Draw(img, "RGBA")
    T = lambda x, z: ((x - x0) * s, (z - z0) * s)
    for r in rooms:
        col = (70, 70, 110, 255) if r["secret"] else (90, 84, 74, 255)
        d.polygon([T(*p) for p in r["poly"]], fill=col, outline=(200, 190, 170, 255))
    nav = data["nav"]
    for i, p in enumerate(nav["polys"]):
        pts = [nav["verts"][k] for k in p]
        y = sum(q[1] for q in pts) / len(pts)
        t = max(0.0, min(1.0, (y + 8) / 10))
        col = (int(60 + 160 * t), int(170 - 40 * t), int(200 - 150 * t), 110)
        if nav["poly_gate"][i]:
            col = (230, 70, 60, 150)
        d.polygon([T(q[0], q[2]) for q in pts], fill=col, outline=(240, 240, 240, 160))
    for l in data["lights"]:
        x, z = T(l["pos"][0], l["pos"][2])
        c = tuple(int(255 * min(1, v)) for v in l["color"])
        d.ellipse((x - 3, z - 3, x + 3, z + 3), fill=c + (255,))
    cols = {"start": (90, 220, 120), "rest": (250, 200, 80), "down": (80, 160, 255), "up": (150, 200, 255), "mob": (230, 60, 60),
            "chest": (240, 190, 70), "rare": (230, 90, 230), "lever": (255, 255, 255), "gate": (255, 120, 80), "trap": (255, 140, 0),
            "lore": (170, 220, 255), "bonus": (120, 255, 200), "ladder": (200, 200, 255), "trapdoor": (180, 120, 255),
            "shortcut": (120, 255, 120), "secret_wall": (200, 120, 255), "hint": (160, 120, 230)}
    for m in data["marks"]:
        x, z = T(m["pos"][0], m["pos"][2])
        c = cols.get(m["kind"], (255, 255, 255))
        d.ellipse((x - 5, z - 5, x + 5, z + 5), fill=c + (255,), outline=(0, 0, 0, 255))
        for q in m.get("patrol", []):
            qx, qz = T(q[0], q[2])
            d.line((x, z, qx, qz), fill=c + (160,), width=1)
    from PIL import ImageFont
    try:
        font = ImageFont.truetype("DejaVuSans.ttf", 14)
    except OSError:
        font = None
    d.text((8, 8), f"Nivå {level.n}: navigasjonsnettet ({len(nav['polys'])} polygoner) over planen", fill=(240, 230, 210, 255), font=font)
    img.save(path)


# --- Hovedprogrammet ------------------------------------------------------------


def main():
    args = sys.argv[1:]
    n = int(args[0]) if args and args[0].isdigit() else 1
    export = "--export" in args
    L = Level(n)
    L.build()
    pieces = chunks(L.mb, n)
    data = {
        "level": n,
        "name": L.plan.name,
        "theme": "grav",
        "pieces": sorted(pieces),
        "rooms": L.rooms_json(),
        "colliders": L.colliders,
        "nav": L.nav(),
        "lights": L.lights,
        "marks": L.marks,
    }
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, f"crypt_{n:02d}.json"), "w") as f:
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
    draw(L, data, os.path.join(DOCS, f"nav_{n:02d}.png"))
    tris = L.mb.tris()
    print(f"nivå {n}: {tris} trekanter i {len(pieces)} biter, {len(data['colliders'])} kollidere, {len(data['nav']['polys'])} polygoner, {len(L.lights)} lys")
    for name, m in sorted(pieces.items()):
        print(f"  {name}: {m.tris()}")
    if "--no-check" not in args:
        # Tetthetssjekken: et nivå som slipper himmelen inn, eksporteres ikke.
        import leak_check
        leaks = leak_check.check(L.mb, data["nav"], data["rooms"], data["marks"], n, os.path.join(ROOT, "shots", "krypten", f"leaks_{n:02d}.txt"))
        if leaks:
            print(f"nivå {n} LEKKER ({len(leaks)} hull): rett årsaken i build_level.py/kit_grav.py (--no-check hopper over)")
            sys.exit(1)
    if export:
        only = [a for a in args[1:] if a.startswith("crypt_")]
        for name, m in sorted(pieces.items()):
            if only and name not in only:
                continue
            K.export_model(m, os.path.join(OUT, f"{name}.glb"), ao=True, samples=32)
        # Biter fra en eldre utgave som ikke finnes lenger.
        for fn in os.listdir(OUT):
            if fn.startswith(f"crypt_{n:02d}_") and fn.endswith(".glb") and fn[:-4] not in pieces:
                os.remove(os.path.join(OUT, fn))


if __name__ == "__main__":
    main()
