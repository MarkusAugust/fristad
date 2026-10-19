"""Byggesettet for gravkjelleren (krypten, tema 1) i den malte stilen
(docs/ny-stil.md, docs/krypten-ny.md).

Detaljene (stein, fuger, knokler, planker) ligger i de malte teksturlagene
(`crypt_wall`, `crypt_floor`, `crypt_vault`, `grave_slab`, `ossuary`,
`plaster_old` som kalkpuss, `rubble`, `bronze_door`, `glitch_seam`). Bare silhuetten er
geometri: murene med døråpninger og nisjer, hvelvene med ribber, søylene,
trinnene, sarkofagene, alteret, lysestakene og faklene.

Modulene settes på et rutenett på 1 m: murene og gulvene deles i ruter på
1 × 1 m fra hjørnet (det gir også okklusjonen hjørner å bo i), og nivåskriptet
(`build_level.py`) setter rommene, gangene og trappene sammen etter planen.

Akser som i spillet og planene: x mot øst, y opp, z mot sør, i meter. Det er
de samme aksene som i glTF, så flatene vender riktig vei uten omregning;
`export` gjør dem om til Blenders akser (x, −z, y).

Geometrien bygges i ren Python (`MB`) uten Blender, så nivået, kolliderne og
navigasjonsnettet kan regnes ut og sjekkes uten bpy. Bare `export` trenger
Blender (`python3.13` med `bpy`).

    python3 tools/crypt/kit_grav.py            # eksporterer mekanismene (kit_*.glb)
    python3 tools/crypt/kit_grav.py --sheet ut.glb   # alle modulene ved siden av hverandre

Mekanismene (egne modeller, motoren setter dem ut etter `marks` og beveger
dem) har origo og front slik:

| Fil | Origo | Front (+z) |
|---|---|---|
| `kit_rest` / `kit_rest_flame` | midt under alteret på gulvet | mot den som tenner |
| `kit_lever` | midt under foten på gulvet | mot den som trekker |
| `kit_lever_handle` | i dreieaksen (x), 0,55 m over gulvet | står rett opp, trekkes mot +z |
| `kit_gate` | midt under gitteret | flatt i xy-planet |
| `kit_shortcut_door` | nederst i hengselet (venstre kant sett forfra) | slåsida er +z |
| `kit_shortcut_bar` | midt i slåen | ligger langs x |
| `kit_false_wall` | midt under muren | mot rommet den synes fra |
| `kit_trapdoor` | i hengselet midt på kanten mot −z | ligger flatt, åpnes opp om x |
| `kit_ladder` | midt under en bit på 1 m | trinnene mot +z (stables) |
| `kit_ladder_top` | i kanten der du går av, på vangenes midtlinje | krokene over kanten mot −z |
| `kit_plate` | midt under platen | flat |
| `kit_arrow_slots` | midt på panelet, i veggflaten | mot gangen |
"""

import math
import os
import sys

from shapely import constrained_delaunay_triangles
from shapely.geometry import GeometryCollection, MultiPolygon, Polygon
from shapely.geometry import box as sbox

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
OUT = os.path.join(ROOT, "assets", "models", "crypt")

# --- Vektorer -------------------------------------------------------------------


def add(a, b):
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def mul(a, s):
    return (a[0] * s, a[1] * s, a[2] * s)


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def norm(a):
    l = math.sqrt(dot(a, a))
    return (a[0] / l, a[1] / l, a[2] / l) if l > 1e-12 else (0.0, 0.0, 0.0)


def lerp(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def newell(pts):
    n = [0.0, 0.0, 0.0]
    for i, p in enumerate(pts):
        q = pts[(i + 1) % len(pts)]
        n[0] += (p[1] - q[1]) * (p[2] + q[2])
        n[1] += (p[2] - q[2]) * (p[0] + q[0])
        n[2] += (p[0] - q[0]) * (p[1] + q[1])
    return tuple(n)


def rot_y(p, yaw):
    """Dreier (x, y, z) om +y (som glam: +z går mot +x ved 90°)."""
    c, s = math.cos(yaw), math.sin(yaw)
    return (p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c)


# --- Materialene ----------------------------------------------------------------

# Navnet på materialet i modellen → (teksturlag, tone, ruhet). Tonen er lineær
# og høyst 1. «layer_<lag>.magic» gløder (gltf.rs), «fire_» brenner hele
# døgnet, «magic_» gløder i sin egen farge, «water» er vann.
MATS = {
    "wall": ("crypt_wall", (0.9, 0.9, 0.93), 0.5),
    "dressed": ("crypt_wall", (1.0, 0.98, 0.95), 0.45),
    "wall_dark": ("crypt_wall", (0.62, 0.62, 0.68), 0.55),
    # Revnene i sprekken: nesten svart over muren, så de synes på avstand
    # (eieren i runde 8; med `wall_dark` var de like lyse som muren).
    "crack": ("crypt_wall", (0.16, 0.15, 0.15), 0.7),
    # Mørtelen bak fugene i en karm: ensfarget og mørk, så fugene ikke ser
    # ut som hull (eieren i runde 8: «kanten rundt ser hullete ut»).
    "mortar": ("plaster_old", (0.26, 0.25, 0.24), 0.75),
    "floor": ("crypt_floor", (1.0, 1.0, 1.0), 0.5),
    "slab": ("grave_slab", (1.0, 1.0, 1.0), 0.45),
    # Hvelvet mures av den samme steinen som veggene, litt mørkere (eieren
    # om nivå 1: «teksturer som ikke passer sammen»; crypt_vault var små,
    # mørke bruddsteiner over store lyse kvadre).
    "vault": ("crypt_wall", (0.72, 0.72, 0.76), 0.5),
    "rib": ("crypt_wall", (0.8, 0.79, 0.8), 0.5),
    "bones": ("ossuary", (1.0, 1.0, 1.0), 0.5),
    "fresco": ("plaster_old", (0.52, 0.55, 0.60), 0.62),  # gammel kalkpuss, mørk og kald (crypt_plaster så ut som et barnerom)
    "rubble": ("rubble", (1.0, 1.0, 1.0), 0.55),
    # Tre: planker (lemmer, pulter, skrin) og bjelker (stiger, håndtak,
    # slåer). Dørbildene (`bronze_door`, `door_b`) er hele bilder av en dør
    # og brukes bare på dørblad med uv 0–1 («leaf»; eierens dom i runde 7:
    # svart lem, hengsler malt på stigetrinnene).
    "oak": ("planks", (0.62, 0.5, 0.42), 0.55),
    "wood": ("timber", (0.85, 0.78, 0.72), 0.55),
    "leaf": ("bronze_door", (1.0, 1.0, 1.0), 0.5),
    "strap": ("trim", (1.0, 1.0, 1.0), 0.45),
    "gilt": ("plaster_white", (0.95, 0.68, 0.26), 0.35),
    "leather": ("leather", (0.62, 0.36, 0.24), 0.6),
    "iron": ("iron", (0.45, 0.43, 0.42), 0.45),
    "wax": ("plaster_white", (1.0, 0.93, 0.78), 0.4),
    "glitch": ("glitch_seam", (1.0, 1.0, 1.0), 0.4),
    "rock": ("rock", (0.8, 0.8, 0.82), 0.65),
    # Falmet ull til fanene i borgen (nivå 7).
    "cloth_red": ("wool", (0.55, 0.12, 0.1), 0.85),
    "cloth_blue": ("wool", (0.12, 0.18, 0.45), 0.85),
    "mud": ("mud", (1.0, 1.0, 1.0), 0.8),
    # Plasten i den korrupte kjernen (nivå 9–10): glatt og blek. Det malte
    # bildet `core_plastic` er ikke et teksturlag ennå (lagene er nesten
    # fulle), så plasten er kalkpuss med en kald, lys tone og lav ruhet.
    "plastic": ("plaster_white", (0.82, 0.83, 0.86), 0.22),
    "fire_torch": None,
    "fire_candle": None,
    "fire_lamp": None,
    "magic_crystal": None,
    "magic_white": None,
    "magic_pink": None,
    "magic_down": None,
    "magic_bonus": None,
    "magic_seam": None,  # sømmene som lyser fiolett i muren (nivå 9–10)
    "water": None,
}
# Fargene til de spesielle materialene (lineær RGB).
SPECIAL = {
    "fire_torch": ((1.0, 0.5, 0.16), 0.5),
    "fire_candle": ((1.0, 0.68, 0.32), 0.5),
    "fire_lamp": ((1.0, 0.62, 0.22), 0.45),
    "magic_crystal": ((0.3, 0.55, 1.0), 0.2),
    "magic_white": ((0.85, 0.92, 1.0), 0.15),
    "magic_pink": ((1.0, 0.45, 0.8), 0.2),
    "magic_down": ((0.25, 0.45, 1.0), 0.3),
    "magic_bonus": ((1.0, 0.72, 0.25), 0.3),
    "magic_seam": ((0.7, 0.35, 1.0), 0.2),
    "water": ((0.02, 0.04, 0.05), 0.05),
}


def material_name(key):
    """Navnet i glb-fila. Variantene av samme lag får et suffiks etter
    punktum (gltf.rs leser laget før punktumet)."""
    spec = MATS[key]
    if spec is None:
        return key
    layer = spec[0]
    base = {"wall": "", "floor": "", "slab": "", "vault": "", "bones": "", "fresco": "", "rubble": "", "oak": "", "wood": "", "iron": "", "leather": "", "rock": ""}
    if key == "glitch":
        return "layer_glitch_seam.magic"
    if key in base:
        return f"layer_{layer}"
    return f"layer_{layer}.{key}"


# --- Mesh i ren Python ----------------------------------------------------------


class MB:
    """Flatene til en modell: hjørner (x, y, z), uv i meter, materiale og
    om flaten skal glattes (hvelv, søyler). Hjørnene slås sammen ved
    eksporten."""

    def __init__(self):
        self.faces = []

    def poly(self, pts, uvs, mat, facing=None, smooth=False):
        pts = [tuple(float(c) for c in p) for p in pts]
        if len(pts) < 3:
            return
        n = newell(pts)
        if dot(n, n) < 1e-14:
            return
        if facing is not None and dot(n, facing) < 0:
            pts = pts[::-1]
            uvs = list(uvs)[::-1]
        self.faces.append((pts, [tuple(u) for u in uvs], mat, smooth))

    def quad(self, a, b, c, d, mat, facing=None, uv=None, smooth=False):
        pts = [a, b, c, d]
        self.poly(pts, uv or [auto_uv(p, newell(pts)) for p in pts], mat, facing, smooth)

    def tri(self, a, b, c, mat, facing=None, uv=None, smooth=False):
        pts = [a, b, c]
        self.poly(pts, uv or [auto_uv(p, newell(pts)) for p in pts], mat, facing, smooth)

    def extend(self, other, yaw=0.0, at=(0.0, 0.0, 0.0), keep_uv=False):
        """Legg til flatene fra en annen modul, dreid om y og flyttet. Uv-ene
        regnes på nytt fra verdenskoordinatene (så teksturen går videre
        mellom modulene), med mindre `keep_uv`."""
        for pts, uvs, mat, smooth in other.faces:
            q = [add(rot_y(p, yaw), at) for p in pts]
            if keep_uv:
                self.faces.append((q, uvs, mat, smooth))
            else:
                n = newell(q)
                self.faces.append((q, [auto_uv(p, n) for p in q], mat, smooth))

    def tris(self):
        return sum(len(p) - 2 for p, _, _, _ in self.faces)

    def bounds(self):
        xs = [p for f in self.faces for p in f[0]]
        lo = tuple(min(p[i] for p in xs) for i in range(3))
        hi = tuple(max(p[i] for p in xs) for i in range(3))
        return lo, hi


def auto_uv(p, n):
    """Uv i meter etter flatens hovedakse (som `town.uv_box`): vegger mot ±z
    får (x, y), vegger mot ±x (z, y), flater opp og ned (x, z)."""
    ax = max(range(3), key=lambda i: abs(n[i]))
    if ax == 1:
        return (p[0], p[2])
    if ax == 2:
        return (p[0], p[1])
    return (p[2], p[1])


def box(mb, lo, hi, mat, faces="all", smooth=False):
    """En boks fra hjørnet `lo` til `hi`. `faces` kan utelate flater som
    aldri synes ('-y' osv.)."""
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    P = lambda x, y, z: (x, y, z)
    sides = {
        "+y": ([P(x0, y1, z0), P(x0, y1, z1), P(x1, y1, z1), P(x1, y1, z0)], (0, 1, 0)),
        "-y": ([P(x0, y0, z0), P(x1, y0, z0), P(x1, y0, z1), P(x0, y0, z1)], (0, -1, 0)),
        "+x": ([P(x1, y0, z0), P(x1, y1, z0), P(x1, y1, z1), P(x1, y0, z1)], (1, 0, 0)),
        "-x": ([P(x0, y0, z0), P(x0, y0, z1), P(x0, y1, z1), P(x0, y1, z0)], (-1, 0, 0)),
        "+z": ([P(x0, y0, z1), P(x1, y0, z1), P(x1, y1, z1), P(x0, y1, z1)], (0, 0, 1)),
        "-z": ([P(x0, y0, z0), P(x0, y1, z0), P(x1, y1, z0), P(x1, y0, z0)], (0, 0, -1)),
    }
    for k, (pts, n) in sides.items():
        if faces != "all" and k not in faces:
            continue
        mb.quad(*pts, mat, facing=n, smooth=smooth)


def obox(mb, c, size, yaw, mat, faces="all"):
    """En dreid boks med midten `c` (nederst i y) og mål `size`."""
    m = MB()
    sx, sy, sz = size
    box(m, (-sx / 2, 0, -sz / 2), (sx / 2, sy, sz / 2), mat, faces)
    mb.extend(m, yaw, c)


def prism(mb, ring, y0, y1, mat, outward=True, cap_top=True, cap_bottom=False, top_mat=None, smooth=False):
    """Et loddrett prisme over et polygon i xz (rekkefølgen spiller ingen rolle)."""
    n = len(ring)
    cx = sum(p[0] for p in ring) / n
    cz = sum(p[1] for p in ring) / n
    for i in range(n):
        a, b = ring[i], ring[(i + 1) % n]
        mid = ((a[0] + b[0]) / 2 - cx, 0, (a[1] + b[1]) / 2 - cz)
        facing = mid if outward else mul(mid, -1)
        L = math.dist(a, b)
        u0 = sum(math.dist(ring[k], ring[k + 1]) for k in range(i))
        mb.poly([(a[0], y0, a[1]), (b[0], y0, b[1]), (b[0], y1, b[1]), (a[0], y1, a[1])],
                [(u0, y0), (u0 + L, y0), (u0 + L, y1), (u0, y1)], mat, facing=facing, smooth=smooth)
    if cap_top:
        mb.poly([(p[0], y1, p[1]) for p in ring], [(p[0], p[1]) for p in ring], top_mat or mat, facing=(0, 1, 0))
    if cap_bottom:
        mb.poly([(p[0], y0, p[1]) for p in ring], [(p[0], p[1]) for p in ring], mat, facing=(0, -1, 0))


def circle(c, r, n, a0=0.0, a1=2 * math.pi):
    full = abs(a1 - a0 - 2 * math.pi) < 1e-9
    k = n if full else n + 1
    return [(c[0] + r * math.cos(a0 + (a1 - a0) * i / n), c[1] + r * math.sin(a0 + (a1 - a0) * i / n)) for i in range(k)]


def cylinder(mb, c, r, y0, y1, mat, n=8, caps=(True, False), smooth=True):
    ring = circle(c, r, n)
    prism(mb, ring, y0, y1, mat, cap_top=caps[0], cap_bottom=caps[1], smooth=smooth)


# --- Flater: plan og sylinder ---------------------------------------------------


class Plane:
    """En loddrett mur: `p0` (x, z) i planen, `udir` langs muren, normalen
    inn i rommet. u er meter langs muren fra p0, v er høyden (absolutt y),
    d er avstanden ut fra flaten mot rommet."""

    def __init__(self, p0, udir, normal, u0=0.0):
        self.p0 = p0
        self.ud = udir
        self.n = normal
        self.u0 = u0  # u-en til p0 i teksturen

    def point(self, u, v, d=0.0):
        return (self.p0[0] + self.ud[0] * u + self.n[0] * d, v, self.p0[1] + self.ud[1] * u + self.n[1] * d)

    def normal(self, u=0.0):
        return (self.n[0], 0.0, self.n[1])

    def uv(self, u, v):
        return (self.u0 + u, v)


class Cyl:
    """Innsida (eller utsida) av en loddrett sylinder: u er buelengden fra
    vinkelen a0 (mot økende vinkel i xz), v er høyden."""

    def __init__(self, c, r, a0=0.0, inward=True):
        self.c = c
        self.r = r
        self.a0 = a0
        self.inward = inward

    def point(self, u, v, d=0.0):
        a = self.a0 + u / self.r
        rr = self.r - d if self.inward else self.r + d
        return (self.c[0] + rr * math.cos(a), v, self.c[1] + rr * math.sin(a))

    def normal(self, u=0.0):
        a = self.a0 + u / self.r
        s = -1.0 if self.inward else 1.0
        return (s * math.cos(a), 0.0, s * math.sin(a))

    def uv(self, u, v):
        return (self.a0 * self.r + u, v)


class Horizontal:
    """Et vannrett gulv (opp) eller tak (ned) i høyden y; u, v er x, z."""

    def __init__(self, y, up=True):
        self.y = y
        self.up = up

    def point(self, u, v, d=0.0):
        return (u, self.y + (d if self.up else -d), v)

    def normal(self, u=0.0):
        return (0.0, 1.0 if self.up else -1.0, 0.0)

    def uv(self, u, v):
        return (u, v)


def _pieces(g):
    if g.is_empty:
        return []
    if isinstance(g, Polygon):
        return [g]
    if isinstance(g, (MultiPolygon, GeometryCollection)):
        out = []
        for h in g.geoms:
            out += _pieces(h)
        return out
    return []


def grid_face(mb, surf, shape, mat, cell=1.0, origin=(0.0, 0.0), smooth=False, cell_v=None):
    """Legger en flate over `shape` (shapely, i flatens (u, v)) delt i ruter
    på `cell` meter fra `origin`. Hele ruter blir firkanter; ruter med kant
    eller hull trianguleres."""
    if shape.is_empty:
        return
    cv = cell_v or cell
    u0, v0, u1, v1 = shape.bounds
    i0 = math.floor((u0 - origin[0]) / cell + 1e-9)
    i1 = math.ceil((u1 - origin[0]) / cell - 1e-9)
    j0 = math.floor((v0 - origin[1]) / cv + 1e-9)
    j1 = math.ceil((v1 - origin[1]) / cv - 1e-9)
    for i in range(i0, i1):
        for j in range(j0, j1):
            a = (origin[0] + i * cell, origin[1] + j * cv)
            b = (a[0] + cell, a[1] + cv)
            cellbox = sbox(a[0], a[1], b[0], b[1])
            piece = shape.intersection(cellbox)
            for p in _pieces(piece):
                if p.area < 1e-5:
                    continue
                if abs(p.area - cell * cv) < 1e-6:
                    ring = [(a[0], a[1]), (b[0], a[1]), (b[0], b[1]), (a[0], b[1])]
                    _surf_poly(mb, surf, ring, mat, smooth)
                    continue
                coords = list(p.exterior.coords)[:-1]
                if not p.interiors and _convex(coords):
                    _surf_poly(mb, surf, coords, mat, smooth)
                    continue
                for t in _pieces(constrained_delaunay_triangles(p)):
                    _surf_poly(mb, surf, list(t.exterior.coords)[:-1], mat, smooth)


def _convex(c):
    n = len(c)
    if n < 3:
        return False
    s = 0
    for i in range(n):
        a, b, d = c[i], c[(i + 1) % n], c[(i + 2) % n]
        z = (b[0] - a[0]) * (d[1] - b[1]) - (b[1] - a[1]) * (d[0] - b[0])
        if abs(z) < 1e-12:
            continue
        if s == 0:
            s = 1 if z > 0 else -1
        elif (z > 0) != (s > 0):
            return False
    return True


def _surf_poly(mb, surf, ring, mat, smooth=False):
    pts = [surf.point(u, v) for u, v in ring]
    mu = sum(u for u, _ in ring) / len(ring)
    mb.poly(pts, [surf.uv(u, v) for u, v in ring], mat, facing=surf.normal(mu), smooth=smooth)


# --- Åpningene i murene ---------------------------------------------------------


def arch_profile(uc, w, v0, spring, n=12):
    """Omrisset av en rundbuet åpning (u, v): fra venstre fot opp, buen over
    og ned på høyre side. `spring` er der buen begynner, over v0."""
    r = w / 2
    pts = [(uc - r, v0), (uc - r, v0 + spring)]
    for k in range(1, n):
        a = math.pi - math.pi * k / n
        pts.append((uc + r * math.cos(a), v0 + spring + r * math.sin(a)))
    pts += [(uc + r, v0 + spring), (uc + r, v0)]
    return pts


def rect_profile(uc, w, v0, h):
    return [(uc - w / 2, v0), (uc - w / 2, v0 + h), (uc + w / 2, v0 + h), (uc + w / 2, v0)]


def opening_shape(profile):
    return Polygon(profile)


def wall(mb, surf, length, bottom, top, openings=(), mat="wall", cell=1.0, top_fn=None, top_samples=None, extra=0.05):
    """En mur (modulen «mur»): fra u = 0 til `length`, fra `bottom` til
    toppen (fast høyde `top`, eller `top_fn(u)` med prøvene `top_samples`),
    med åpningene skåret ut (polygoner i (u, v)). Ruter på 1 m."""
    if top_fn is None:
        outline = Polygon([(0, bottom), (length, bottom), (length, top + extra), (0, top + extra)])
    else:
        us = sorted(set([0.0, length] + [u for u in (top_samples or []) if 0 <= u <= length]))
        if len(us) < 3:
            us = [length * k / 16 for k in range(17)]
        outline = Polygon([(0, bottom), (length, bottom)] + [(u, top_fn(u) + extra) for u in reversed(us)])
    for o in openings:
        outline = outline.difference(o)
    grid_face(mb, surf, outline, mat, cell=cell, origin=(0.0, math.floor(bottom)))


def plinth(mb, surf, spans, h=0.34, proud=0.07, bevel=0.035, mat="dressed"):
    """Sokkelen langs veggfoten: et skift av hogd stein som står `proud` ut
    fra muren, med skrådd overkant. `spans` er (u0, u1, gulvhøyde) langs
    muren, mellom åpningene (eieren om nivå 1: «harde overganger»)."""
    up = (0.0, 1.0, 0.0)
    ud = (surf.ud[0], 0.0, surf.ud[1]) if hasattr(surf, "ud") else None
    P = surf.point
    for u0, u1, y in spans:
        if u1 - u0 < 0.05:
            continue
        n = surf.normal((u0 + u1) / 2)
        yb, yt = y - 0.03, y + h
        yc = yt - bevel
        mb.quad(P(u0, yb, proud), P(u1, yb, proud), P(u1, yc, proud), P(u0, yc, proud), mat, facing=n)
        mb.quad(P(u0, yc, proud), P(u1, yc, proud), P(u1, yt, proud - bevel), P(u0, yt, proud - bevel), mat, facing=(n[0] + up[0], n[1] + up[1], n[2] + up[2]))
        mb.quad(P(u0, yt, proud - bevel), P(u1, yt, proud - bevel), P(u1, yt, 0.0), P(u0, yt, 0.0), mat, facing=up)
        if ud is None:
            continue
        for u, sg in ((u0, -1.0), (u1, 1.0)):
            pts = [P(u, yb, 0.0), P(u, yb, proud), P(u, yc, proud), P(u, yt, proud - bevel), P(u, yt, 0.0)]
            mb.poly(pts, [auto_uv(q, newell(pts)) for q in pts], mat, facing=(ud[0] * sg, 0.0, ud[2] * sg))


def free_spans(length, openings, floor, h):
    """Strekkene langs en mur (0..length) som ingen åpning skjærer ned i
    under høyden `floor + h`."""
    cuts = []
    for o in openings:
        x0, y0, x1, _ = o.bounds
        if y0 < floor + h + 0.05:
            cuts.append((x0, x1))
    spans, at = [], 0.0
    for a, b in sorted(cuts):
        if a > at:
            spans.append((at, min(a, length), floor))
        at = max(at, b)
    if at < length:
        spans.append((at, length, floor))
    return spans


def portal(mb, surf, profile, mat="dressed", band=0.32, depth=0.14, rect=False):
    """Døråpningen med rundbue: et bånd av buestein rundt åpningen på
    romsida, `depth` ut fra muren, med lysning inn mot åpningen."""
    # Profilen går fra venstre fot, opp og over, ned til høyre fot.
    inner = profile
    uc = (profile[0][0] + profile[-1][0]) / 2
    outer = []
    for i, (u, v) in enumerate(inner):
        if i == 0:
            outer.append((u - band, v))
        elif i == len(inner) - 1:
            outer.append((u + band, v))
        else:
            # Ut fra midten av buen (eller rett ut på sidene).
            if rect:
                du = -band if u < uc else band
                outer.append((u + du, v + band))
            else:
                spring_v = inner[1][1]
                if v <= spring_v + 1e-6:
                    outer.append((u - band if u < uc else u + band, v))
                else:
                    r = math.hypot(u - uc, v - spring_v)
                    s = (r + band) / max(r, 1e-6)
                    outer.append((uc + (u - uc) * s, spring_v + (v - spring_v) * s))
    for i in range(len(inner) - 1):
        a, b = inner[i], inner[i + 1]
        c, d = outer[i], outer[i + 1]
        fa, fb, fc, fd = (surf.point(*a, depth), surf.point(*b, depth), surf.point(*d, depth), surf.point(*c, depth))
        n = surf.normal((a[0] + b[0]) / 2)
        mb.poly([fa, fb, fc, fd], [surf.uv(*a), surf.uv(*b), surf.uv(*d), surf.uv(*c)], mat, facing=n)
        # Lysningen (inn mot åpningen) og ytterkanten.
        L = math.dist(a, b)
        mb.poly([surf.point(*a, 0), surf.point(*b, 0), surf.point(*b, depth), surf.point(*a, depth)],
                [(0, 0), (L, 0), (L, depth), (0, depth)], mat, facing=_toward(surf, a, b, uc, inner[1][1], inward=True))
        L2 = math.dist(c, d)
        mb.poly([surf.point(*c, 0), surf.point(*d, 0), surf.point(*d, depth), surf.point(*c, depth)],
                [(0, 0), (L2, 0), (L2, depth), (0, depth)], mat, facing=_toward(surf, c, d, uc, inner[1][1], inward=False))


def _toward(surf, a, b, uc, spring_v, inward):
    """Retningen fra en kant i åpningsbåndet mot midten av åpningen (inward)
    eller bort fra den."""
    m = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
    cen = (uc, min(spring_v, m[1]) if m[1] < spring_v else spring_v)
    du, dv = cen[0] - m[0], cen[1] - m[1]
    if not inward:
        du, dv = -du, -dv
    p = surf.point(m[0], m[1])
    q = surf.point(m[0] + du * 0.01, m[1] + dv * 0.01)
    return sub(q, p)


def recess(mb, surf, profile, depth, mat_side="wall", mat_back="wall", back=True):
    """Et innhugg i muren bak en åpning (nisjer, pilhull): sidene langs
    omrisset og bakveggen i dybden `depth` (bak flaten)."""
    n = len(profile)
    uc = sum(p[0] for p in profile) / n
    vc = sum(p[1] for p in profile) / n
    for i in range(n):
        a, b = profile[i], profile[(i + 1) % n]
        pa0, pb0 = surf.point(*a, 0.0), surf.point(*b, 0.0)
        pa1, pb1 = surf.point(*a, -depth), surf.point(*b, -depth)
        # Sideflatene vender inn mot midten av hullet.
        mid = surf.point((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, -depth / 2)
        cen = surf.point(uc, vc, -depth / 2)
        L = math.dist(a, b)
        mb.poly([pa0, pb0, pb1, pa1], [(0, 0), (L, 0), (L, depth), (0, depth)], mat_side, facing=sub(cen, mid))
    if back:
        pts = [surf.point(u, v, -depth) for u, v in profile]
        mb.poly(pts, [surf.uv(u, v) for u, v in profile], mat_back, facing=surf.normal(uc))


# --- Hvelvene -------------------------------------------------------------------


def ellipse_arclen(a, R, ts):
    """Buelengden langs ellipsen x = a·t, y = R·√(1 − t²) for t i `ts`."""
    out = [0.0]
    for t0, t1 in zip(ts, ts[1:]):
        p0 = (a * t0, R * math.sqrt(max(0.0, 1 - t0 * t0)))
        p1 = (a * t1, R * math.sqrt(max(0.0, 1 - t1 * t1)))
        out.append(out[-1] + math.dist(p0, p1))
    return out


def tsamples(n):
    """Prøvene i [−1, 1] for hvelvene: tettere mot foten, der buen er bratt."""
    return [-math.cos(math.pi * k / n) for k in range(n + 1)]


def groin_bay(mb, x0, x1, z0, z1, S, R, mat="vault", n=12, k=3):
    """Krysshvelv over et fag: to tønnehvelv som krysser hverandre, med
    gratene langs diagonalene. Hver av de fire trekantene er en del av en
    sylinder, med linjene (generatrisene) inn mot gratene."""
    cx, cz = (x0 + x1) / 2, (z0 + z1) / 2
    a, b = (x1 - x0) / 2, (z1 - z0) / 2
    ts = tsamples(n)
    arc_x = ellipse_arclen(a, R, ts)
    arc_z = ellipse_arclen(b, R, ts)
    h = lambda t: S + R * math.sqrt(max(0.0, 1 - t * t))
    for side in ("n", "s", "w", "e"):
        for i in range(n):
            t0, t1 = ts[i], ts[i + 1]
            strips = []
            for t, arc in ((t0, (arc_x if side in "ns" else arc_z)[i]), (t1, (arc_x if side in "ns" else arc_z)[i + 1])):
                y = h(t)
                if side in "ns":
                    x = cx + a * t
                    zw = z1 if side == "s" else z0
                    zd = cz + b * abs(t) * (1 if side == "s" else -1)
                    strips.append([((x, y, zw + (zd - zw) * q / k), (zw + (zd - zw) * q / k, arc)) for q in range(k + 1)])
                else:
                    z = cz + b * t
                    xw = x1 if side == "e" else x0
                    xd = cx + a * abs(t) * (1 if side == "e" else -1)
                    strips.append([((xw + (xd - xw) * q / k, y, z), (xw + (xd - xw) * q / k, arc)) for q in range(k + 1)])
            A, B = strips
            for q in range(k):
                pts = [A[q][0], B[q][0], B[q + 1][0], A[q + 1][0]]
                uvs = [A[q][1], B[q][1], B[q + 1][1], A[q + 1][1]]
                facing = (0.0, -1.0, 0.0)
                # Flaten vender ned og inn mot midten av faget.
                c = lerp(lerp(pts[0], pts[1], 0.5), lerp(pts[2], pts[3], 0.5), 0.5)
                facing = norm(sub((cx, S - 2.0, cz), c))
                if q == k - 1 and math.dist(pts[2], pts[3]) < 1e-6:
                    mb.poly(pts[:3], uvs[:3], mat, facing=facing, smooth=True)
                else:
                    mb.poly(pts, uvs, mat, facing=facing, smooth=True)


def band(mb, path, width, depth, mat="rib", down=None, smooth=False):
    """En ribbe eller gjordbue under et hvelv: et rektangulært tverrsnitt langs
    `path` (punkter i 3D), `depth` ned fra flaten. Siden mot hvelvet lages
    ikke. `down(p)` er retningen ned fra flaten i punktet (standard: rett ned
    mot midten av buen)."""
    rings = []
    for i, p in enumerate(path):
        t = norm(sub(path[min(i + 1, len(path) - 1)], path[max(i - 1, 0)]))
        dn = norm(down(p)) if down else (0.0, -1.0, 0.0)
        side = norm(cross(t, dn))
        dn = norm(cross(side, t))
        if dot(dn, (0, -1, 0)) < 0 and down is None:
            dn = mul(dn, -1)
        hw = width / 2
        rings.append([add(p, mul(side, -hw)), add(p, mul(side, hw)), add(add(p, mul(side, hw)), mul(dn, depth)), add(add(p, mul(side, -hw)), mul(dn, depth))])
    s = 0.0
    for i in range(len(rings) - 1):
        A, B = rings[i], rings[i + 1]
        L = math.dist(path[i], path[i + 1])
        cen = lambda r: lerp(r[0], r[2], 0.5)
        for j0, j1 in ((1, 2), (2, 3), (3, 0)):
            pts = [A[j0], B[j0], B[j1], A[j1]]
            mid = lerp(lerp(A[j0], A[j1], 0.5), lerp(B[j0], B[j1], 0.5), 0.5)
            c = lerp(cen(A), cen(B), 0.5)
            mb.poly(pts, [(s, 0), (s + L, 0), (s + L, width), (s, width)], mat, facing=sub(mid, c), smooth=smooth)
        s += L
    # Endene.
    for R, other in ((rings[0], rings[1]), (rings[-1], rings[-2])):
        c = lerp(R[0], R[2], 0.5)
        o = lerp(other[0], other[2], 0.5)
        mb.poly(R, [(0, 0), (width, 0), (width, depth), (0, depth)], mat, facing=sub(c, o))


def groin_ribs(mb, x0, x1, z0, z1, S, R, n=12, width=0.24, depth=0.17):
    """Ribbene langs gratene i et krysshvelv (ribbehvelv)."""
    cx, cz = (x0 + x1) / 2, (z0 + z1) / 2
    a, b = (x1 - x0) / 2, (z1 - z0) / 2
    ts = tsamples(n)
    for sx, sz in ((1, 1), (1, -1)):
        path = [(cx + a * t * sx, S + R * math.sqrt(max(0.0, 1 - t * t)), cz + b * t * sz) for t in ts]
        band(mb, path, width, depth)
    # Ribbene står på en konsoll i hvert hjørne (ellers ender de i en spiss
    # mot muren; eieren om nivå 1: «harde overganger»).
    for sx in (1, -1):
        for sz in (1, -1):
            k = 0.7071
            c = (cx + a * sx - sx * k * 0.2, S - 0.3, cz + b * sz - sz * k * 0.2)
            obox(mb, c, (0.5, 1.0, 0.5), math.pi / 4, "dressed", faces=("+y", "-y", "+x", "-x", "+z", "-z"))
            obox(mb, (c[0] + sx * k * 0.05, S - 0.52, c[2] + sz * k * 0.05), (0.32, 0.22, 0.32), math.pi / 4, "dressed", faces=("-y", "+x", "-x", "+z", "-z"))


def barrel(mb, axis, c0, length, cw, S, R, a, mat="vault", n=12, step=1.0, lo=None):
    """Tønnehvelv langs `axis` ('x' eller 'z'): fra c0 (start langs aksen) og
    `length` langt, med midtlinja i `cw` på tvers, halv spennvidde `a` og
    pilhøyde `R` over vederlaget `S`."""
    ts = tsamples(n)
    arc = ellipse_arclen(a, R, ts)
    m = max(1, math.ceil(length / step - 1e-9))
    ss = [c0 + length * q / m for q in range(m + 1)]
    for i in range(n):
        for q in range(m):
            pts, uvs = [], []
            for t, ar, s in ((ts[i], arc[i], ss[q]), (ts[i + 1], arc[i + 1], ss[q]), (ts[i + 1], arc[i + 1], ss[q + 1]), (ts[i], arc[i], ss[q + 1])):
                y = S + R * math.sqrt(max(0.0, 1 - t * t))
                w = cw + a * t
                pts.append((s, y, w) if axis == "x" else (w, y, s))
                uvs.append((s, ar))
            c = lerp(pts[0], pts[2], 0.5)
            target = (c[0], S - 2.0, cw) if axis == "x" else (cw, S - 2.0, c[2])
            mb.poly(pts, uvs, mat, facing=sub(target, c), smooth=True)


def barrel_rib(mb, axis, s, cw, S, R, a, n=12, width=0.3, depth=0.16):
    ts = tsamples(n)
    path = [((s, S + R * math.sqrt(max(0.0, 1 - t * t)), cw + a * t) if axis == "x" else (cw + a * t, S + R * math.sqrt(max(0.0, 1 - t * t)), s)) for t in ts]
    cen = (s, S, cw) if axis == "x" else (cw, S, s)
    band(mb, path, width, depth, down=lambda p: sub(cen, p))


def dome(mb, c, r, S, crown, mat="vault", rings=8, seg=48):
    """Kuppel (flat kalott) over et rundt rom: fra radius r i høyden S til
    toppen i `crown`."""
    rise = crown - S
    Rs = (r * r + rise * rise) / (2 * rise)
    yc = crown - Rs
    h = lambda q: yc + math.sqrt(max(0.0, Rs * Rs - q * q))
    for i in range(rings):
        r0 = r * (1 - i / rings)
        r1 = r * (1 - (i + 1) / rings)
        for j in range(seg):
            a0 = 2 * math.pi * j / seg
            a1 = 2 * math.pi * (j + 1) / seg
            P = lambda q, a: (c[0] + q * math.cos(a), h(q), c[1] + q * math.sin(a))
            pts = [P(r0, a0), P(r0, a1), P(r1, a1), P(r1, a0)]
            uvs = [(r0 * a0, r - r0), (r0 * a1, r - r0), (r1 * a1, r - r1), (r1 * a0, r - r1)]
            if r1 < 1e-6:
                pts, uvs = pts[:3], uvs[:3]
            cen = lerp(pts[0], pts[2], 0.5)
            mb.poly(pts, uvs, mat, facing=sub((c[0], S - 3.0, c[1]), cen), smooth=True)
    return h


def dome_ribs(mb, c, r, h, count=8, width=0.3, depth=0.2, n=10, a0=0.0):
    for k in range(count):
        a = a0 + 2 * math.pi * k / count
        path = [(c[0] + q * math.cos(a), h(q), c[1] + q * math.sin(a)) for q in [r * (1 - i / n) for i in range(n + 1)]]
        cen = (c[0], h(0) - 30.0, c[1])
        band(mb, path, width, depth, down=lambda p: sub(cen, p))
    # Sluttsteinen i toppen.
    cylinder(mb, c, 0.55, h(0) - 0.25, h(0) + 0.02, "dressed", n=8, caps=(False, True), smooth=False)


# --- Søyler, trapper og møbler --------------------------------------------------


def chamfered(c, s, ch):
    """Et kvadrat med avfasede hjørner (åttekant) rundt c."""
    x, z = c
    h = s / 2
    return [(x - h + ch, z - h), (x + h - ch, z - h), (x + h, z - h + ch), (x + h, z + h - ch),
            (x + h - ch, z + h), (x - h + ch, z + h), (x - h, z + h - ch), (x - h, z - h + ch)]


def pillar(mb, x, z, y0, y1, size=0.7, mat="dressed"):
    """Søyle (modulen «søyle»): sokkel, skaft med avfasede hjørner og et
    kapitél under vederlaget."""
    prism(mb, chamfered((x, z), size + 0.34, 0.1), y0 - 0.05, y0 + 0.32, mat, top_mat=mat)
    prism(mb, chamfered((x, z), size + 0.18, 0.08), y0 + 0.32, y0 + 0.48, mat, top_mat=mat)
    prism(mb, chamfered((x, z), size, 0.12), y0 + 0.48, y1 - 0.42, mat, cap_top=False)
    # Kapitélet skrår ut og har en dekkplate.
    lo = chamfered((x, z), size, 0.12)
    hi = chamfered((x, z), size + 0.3, 0.12)
    for i in range(8):
        a, b = lo[i], lo[(i + 1) % 8]
        c_, d = hi[(i + 1) % 8], hi[i]
        pts = [(a[0], y1 - 0.42, a[1]), (b[0], y1 - 0.42, b[1]), (c_[0], y1 - 0.16, c_[1]), (d[0], y1 - 0.16, d[1])]
        mid = lerp(pts[0], pts[2], 0.5)
        mb.poly(pts, [(0, 0), (1, 0), (1, 0.3), (0, 0.3)], mat, facing=(mid[0] - x, -0.4, mid[2] - z))
    prism(mb, chamfered((x, z), size + 0.36, 0.1), y1 - 0.16, y1 + 0.02, mat, cap_top=False)
    mb.poly([(p[0], y1 - 0.16, p[1]) for p in chamfered((x, z), size + 0.36, 0.1)], [(p[0], p[1]) for p in chamfered((x, z), size + 0.36, 0.1)], mat, facing=(0, -1, 0))


def steps(mb, p0, d, lat, width, s0, s1, y0, y1, n, mat="dressed", side_walls=False, nosing=0.03):
    """Trapp (modulen «trapp»): `n` trinn fra s0 til s1 langs retningen `d`
    fra p0 (planen), fra høyden y0 til y1, `width` bred om midtlinja (`lat`
    er sideveis). Inntrinnene og opptrinnene; sidene står mot murene."""
    t = (s1 - s0) / n
    r = (y1 - y0) / n
    hw = width / 2
    P = lambda s, l, y: (p0[0] + d[0] * s + lat[0] * l, y, p0[1] + d[1] * s + lat[1] * l)
    up = r > 0
    for k in range(n):
        sa, sb = s0 + k * t, s0 + (k + 1) * t
        y_tread = y0 + (k + 1) * r if up else y0 + (k + 1) * r
        y_prev = y0 + k * r
        # Inntrinnet (delt sideveis i ruter på 1 m).
        m = max(1, math.ceil(width - 1e-6))
        for q in range(m):
            la, lb = -hw + width * q / m, -hw + width * (q + 1) / m
            if up:
                mb.quad(P(sa, la, y_tread), P(sb, la, y_tread), P(sb, lb, y_tread), P(sa, lb, y_tread), mat, facing=(0, 1, 0))
            else:
                mb.quad(P(sa, la, y_tread), P(sb, la, y_tread), P(sb, lb, y_tread), P(sa, lb, y_tread), mat, facing=(0, 1, 0))
            # Opptrinnet: ved sa for trapper opp (mot den som kommer), ved sa
            # også for trapper ned (synlig fra den lave sida, bakover).
            lo_y, hi_y = min(y_prev, y_tread), max(y_prev, y_tread)
            facing = (-d[0], 0, -d[1]) if up else (d[0], 0, d[1])
            mb.quad(P(sa, la, lo_y), P(sa, lb, lo_y), P(sa, lb, hi_y), P(sa, la, hi_y), mat, facing=facing)
        if side_walls:
            for l, sgn in ((-hw, -1), (hw, 1)):
                lo = min(y0, y1) - 0.05
                mb.quad(P(sa, l, lo), P(sb, l, lo), P(sb, l, y_tread), P(sa, l, y_tread), mat, facing=(lat[0] * sgn, 0, lat[1] * sgn))


def sarcophagus(mb, x, z, y, yaw=0.0, L=2.3, W=1.0, H=0.85):
    """Sarkofag: kiste av hogd stein på en sokkel, med et lokk som har
    rygg og overheng (gravhelle oppå)."""
    m = MB()
    box(m, (-W / 2 - 0.08, 0, -L / 2 - 0.08), (W / 2 + 0.08, 0.16, L / 2 + 0.08), "dressed", faces=("+y", "+x", "-x", "+z", "-z"))
    box(m, (-W / 2, 0.16, -L / 2), (W / 2, H, L / 2), "wall", faces=("+x", "-x", "+z", "-z"))
    # Lokket: overheng og en lav rygg langs midten.
    o = 0.07
    yl, yr = H + 0.12, H + 0.26
    box(m, (-W / 2 - o, H, -L / 2 - o), (W / 2 + o, yl, L / 2 + o), "dressed", faces=("+x", "-x", "+z", "-z", "-y"))
    for sgn in (-1, 1):
        a = (sgn * (W / 2 + o), yl, -L / 2 - o)
        b = (sgn * (W / 2 + o), yl, L / 2 + o)
        c = (0.0, yr, L / 2 + o)
        d = (0.0, yr, -L / 2 - o)
        m.poly([a, b, c, d], [(a[2], 0), (b[2], 0), (c[2], W / 2), (d[2], W / 2)], "slab", facing=(sgn * 0.3, 1, 0))
    for sgn in (-1, 1):
        zz = sgn * (L / 2 + o)
        m.poly([(-(W / 2 + o), yl, zz), ((W / 2 + o), yl, zz), (0.0, yr, zz)], [(0, 0), (1, 0), (0.5, 0.2)], "dressed", facing=(0, 0, sgn))
    mb.extend(m, yaw, (x, y, z))


def coffin(mb, x, z, y, yaw=0.0, L=1.9, W=0.6, H=0.42):
    """Likkiste av mørk eik, smalere ved føttene, med lokk."""
    m = MB()
    hw_h, hw_f = W / 2, W * 0.36
    ring = [(-hw_f, L / 2), (hw_f, L / 2), (W / 2, -L / 2 + 0.45), (hw_h * 0.8, -L / 2), (-hw_h * 0.8, -L / 2), (-W / 2, -L / 2 + 0.45)]
    prism(m, ring, 0.0, H, "wood", top_mat="wood")
    mb.extend(m, yaw, (x, y, z))


def altar(mb, x, z, y, yaw=0.0, w=2.6, d=1.1, h=1.05):
    """Alteret: et trinn, en blokk med innfelt felt foran og en
    alterplate med overheng."""
    m = MB()
    box(m, (-w / 2 - 0.5, 0, -d / 2 - 0.6), (w / 2 + 0.5, 0.18, d / 2 + 0.5), "dressed", faces=("+y", "+x", "-x", "+z", "-z"))
    box(m, (-w / 2, 0.18, -d / 2), (w / 2, h - 0.12, d / 2), "wall", faces=("+x", "-x", "-z"))
    # Front med innfelt felt (+z er fronten).
    fz = d / 2
    surf = Plane((-w / 2, fz), (1.0, 0.0), (0.0, 1.0))
    panel = rect_profile(w / 2, w - 0.6, 0.38, h - 0.12 - 0.62)
    wall(m, surf, w, 0.18, h - 0.12, [opening_shape(panel)], mat="wall", cell=1.0)
    recess(m, surf, panel, 0.06, "dressed", "fresco")
    box(m, (-w / 2 - 0.12, h - 0.12, -d / 2 - 0.12), (w / 2 + 0.12, h + 0.02, d / 2 + 0.12), "slab", faces=("+y", "+x", "-x", "+z", "-z", "-y"))
    mb.extend(m, yaw, (x, y, z))


def candle(mb, x, z, y, h=0.22, r=0.035, flame=0.06):
    """Stearinlys (modulen «stearinlys»): voks og flamme."""
    cylinder(mb, (x, z), r, y, y + h, "wax", n=6, caps=(True, False))
    fy = y + h + 0.01
    f = flame
    tip = (x, fy + f * 1.7, z)
    base = [(x + f * 0.35 * math.cos(a), fy + f * 0.5, z + f * 0.35 * math.sin(a)) for a in (0, 2.09, 4.19)]
    for i in range(3):
        mb.tri(base[i], base[(i + 1) % 3], tip, "fire_candle", facing=sub(lerp(base[i], base[(i + 1) % 3], 0.5), (x, fy, z)))
        mb.tri(base[i], base[(i + 1) % 3], (x, fy, z), "fire_candle", facing=add(sub(lerp(base[i], base[(i + 1) % 3], 0.5), (x, fy, z)), (0, -0.05, 0)))


def candlestick(mb, x, z, y, h=1.35, arms=0):
    """Lysestake av smijern på tre føtter, med dryppskål og lys. Med `arms`
    blir det en kandelaber med flere lys."""
    for k in range(3):
        a = k * 2.094 + 0.5
        foot = (x + 0.26 * math.cos(a), y, z + 0.26 * math.sin(a))
        top = (x + 0.05 * math.cos(a), y + 0.3, z + 0.05 * math.sin(a))
        band(mb, [foot, top], 0.035, 0.035, mat="iron", down=lambda p: (0, -1, 0))
    cylinder(mb, (x, z), 0.03, y + 0.25, y + h - 0.04, "iron", n=6, caps=(False, False))
    cylinder(mb, (x, z), 0.11, y + h - 0.04, y + h, "iron", n=8, caps=(True, True), smooth=False)
    candle(mb, x, z, y + h, h=0.26, r=0.04)
    tips = [(x, y + h + 0.26 + 0.07, z)]
    for k in range(arms):
        a = k * 2 * math.pi / arms
        ax, az = x + 0.32 * math.cos(a), z + 0.32 * math.sin(a)
        band(mb, [(x, y + h - 0.25, z), (ax, y + h - 0.12, az)], 0.03, 0.03, mat="iron", down=lambda p: (0, -1, 0))
        cylinder(mb, (ax, az), 0.08, y + h - 0.14, y + h - 0.1, "iron", n=6, caps=(True, True), smooth=False)
        candle(mb, ax, az, y + h - 0.1, h=0.2, r=0.035)
        tips.append((ax, y + h - 0.1 + 0.2 + 0.06, az))
    return tips


def torch(mb, p, n):
    """Fakkel i et jernfeste på muren: `p` er punktet i muren (x, y, z),
    `n` normalen ut fra muren (x, z). Gir punktet der flammen er."""
    nx, nz = n
    out = lambda d, up=0.0: (p[0] + nx * d, p[1] + up, p[2] + nz * d)
    # Veggplata og armen.
    side = (-nz, nx)
    plate = [(p[0] + side[0] * s + nx * 0.01, p[1] + v, p[2] + side[1] * s + nz * 0.01) for s, v in ((-0.06, -0.12), (0.06, -0.12), (0.06, 0.12), (-0.06, 0.12))]
    mb.poly(plate, [(0, 0), (0.12, 0), (0.12, 0.24), (0, 0.24)], "iron", facing=(nx, 0, nz))
    band(mb, [out(0.01, -0.05), out(0.22, 0.0)], 0.04, 0.03, mat="iron", down=lambda q: (0, -1, 0))
    # Ringen fakkelen står i.
    ring_c = out(0.24, 0.0)
    cylinder(mb, (ring_c[0], ring_c[2]), 0.055, ring_c[1] - 0.03, ring_c[1] + 0.03, "iron", n=6, caps=(False, False), smooth=False)
    # Skaftet heller litt utover.
    bot = out(0.21, -0.32)
    top = out(0.27, 0.22)
    band(mb, [bot, top], 0.06, 0.06, mat="wood", down=lambda q: (nx, 0, nz))
    # Hodet (tjærebeget) og flammen.
    head = out(0.275, 0.3)
    cylinder(mb, (head[0], head[2]), 0.06, head[1] - 0.09, head[1] + 0.04, "wood", n=6, caps=(True, False))
    f = (head[0], head[1] + 0.05, head[2])
    tip = (f[0], f[1] + 0.32, f[2])
    base = [(f[0] + 0.08 * math.cos(a), f[1] + 0.06, f[2] + 0.08 * math.sin(a)) for a in (0, 1.257, 2.513, 3.77, 5.03)]
    for i in range(5):
        a, b = base[i], base[(i + 1) % 5]
        mb.tri(a, b, tip, "fire_torch", facing=sub(lerp(a, b, 0.5), f))
        mb.tri(a, b, f, "fire_torch", facing=add(sub(lerp(a, b, 0.5), f), (0, -0.1, 0)))
    return (f[0], f[1] + 0.15, f[2])


def rubble(mb, x, z, y, r=1.4, h=0.6, seed=1, lean=None):
    """Rasmasse: en lav haug med ujevn kant og noen store steiner. `lean`
    (nx, nz) heller haugen opp mot en mur."""
    import random

    rng = random.Random(seed)
    rings, seg = 3, 10
    jit = [[0.75 + 0.5 * rng.random() for _ in range(seg)] for _ in range(rings + 1)]
    pts = {}
    for i in range(rings + 1):
        q = 1 - i / rings
        for j in range(seg):
            a = 2 * math.pi * j / seg
            rr = r * q * jit[i][j] if i < rings else 0.0
            px, pz = x + rr * math.cos(a), z + rr * math.sin(a)
            hh = h * (1 - q * q) * (0.8 + 0.4 * rng.random()) if i > 0 else -0.04
            if lean and i > 0:
                hh += max(0.0, (math.cos(a) * lean[0] + math.sin(a) * lean[1])) * 0.25 * q
            pts[(i, j)] = (px, y + hh, pz)
    top = (x, y + h, z)
    for i in range(rings):
        for j in range(seg):
            a, b = pts[(i, j)], pts[(i, (j + 1) % seg)]
            if i + 1 < rings:
                c, d = pts[(i + 1, (j + 1) % seg)], pts[(i + 1, j)]
                mb.quad(a, b, c, d, "rubble", facing=(0, 1, 0), uv=[(p[0], p[2]) for p in (a, b, c, d)], smooth=True)
            else:
                mb.tri(a, b, top, "rubble", facing=(0, 1, 0), uv=[(p[0], p[2]) for p in (a, b, top)], smooth=True)
    for k in range(3):
        a = rng.random() * 6.28
        rr = r * (0.5 + 0.5 * rng.random())
        s = 0.22 + 0.2 * rng.random()
        obox(mb, (x + rr * math.cos(a), y - 0.02, z + rr * math.sin(a)), (s * 1.3, s, s), rng.random() * 3, "wall")


def well(mb, x, z, y, r_in=0.85, r_out=1.3, curb=0.5, depth=7.0, n=16):
    """Brønnsjakta: en lav brønnkarm av hogd stein med dekkheller, og sjakta
    ned (innsida av en sylinder) til vannet. Gir høyden på vannet."""
    # Karmen.
    outer = circle((x, z), r_out, n)
    inner = circle((x, z), r_in, n)
    for i in range(n):
        a, b = outer[i], outer[(i + 1) % n]
        mb.quad((a[0], y - 0.05, a[1]), (b[0], y - 0.05, b[1]), (b[0], y + curb, b[1]), (a[0], y + curb, a[1]), "dressed", facing=((a[0] + b[0]) / 2 - x, 0, (a[1] + b[1]) / 2 - z))
        c, d = inner[(i + 1) % n], inner[i]
        mb.quad((a[0], y + curb, a[1]), (b[0], y + curb, b[1]), (c[0], y + curb, c[1]), (d[0], y + curb, d[1]), "slab", facing=(0, 1, 0))
    # Sjakta: innsida, ned til vannet.
    for i in range(n):
        a, b = inner[i], inner[(i + 1) % n]
        mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        f = (x - mid[0], 0, z - mid[1])
        u0, u1 = r_in * 2 * math.pi * i / n, r_in * 2 * math.pi * (i + 1) / n
        ys = [y + curb] + [y - k for k in range(1, int(depth) + 1)] + [y - depth]
        ys = sorted(set(ys), reverse=True)
        for ya, yb in zip(ys, ys[1:]):
            mb.poly([(a[0], ya, a[1]), (b[0], ya, b[1]), (b[0], yb, b[1]), (a[0], yb, a[1])], [(u0, ya), (u1, ya), (u1, yb), (u0, yb)], "wall_dark", facing=f)
    return y - depth


def parapet(mb, path, y, h=0.95, t=0.26, mat="dressed", cap="slab", closed=False, ends=True, base_y=None):
    """Rekkverk (modulen «galleri med rekkverk»): en brystning av hogd stein
    langs `path` (punkter i xz), med dekkhelle. `base_y` gir en egen
    bunnhøyde per punkt (trapper)."""
    pts = list(path)
    n = len(pts)
    segs = range(n if closed else n - 1)

    def offs(i, s):
        p = pts[i]
        a = pts[i - 1] if (i > 0 or closed) else None
        b = pts[(i + 1) % n] if (i < n - 1 or closed) else None
        dirs = []
        for q0, q1 in ((a, p), (p, b)):
            if q0 is not None and q1 is not None:
                dx, dz = q1[0] - q0[0], q1[1] - q0[1]
                L = math.hypot(dx, dz)
                dirs.append((dx / L, dz / L))
        dx = sum(d[0] for d in dirs)
        dz = sum(d[1] for d in dirs)
        L = math.hypot(dx, dz)
        dx, dz = dx / L, dz / L
        nx, nz = -dz, dx
        k = 1.0
        if len(dirs) == 2:
            # Gjæringen: forskyvningen deles på cos av halve vinkelen.
            k = 1.0 / max(0.3, math.sqrt((1 + dirs[0][0] * dirs[1][0] + dirs[0][1] * dirs[1][1]) / 2))
        return (p[0] + nx * s * k, p[1] + nz * s * k)

    by = base_y or [y] * n
    L = [offs(i, -t / 2) for i in range(n)]
    R = [offs(i, t / 2) for i in range(n)]
    Lc = [offs(i, -t / 2 - 0.05) for i in range(n)]
    Rc = [offs(i, t / 2 + 0.05) for i in range(n)]
    s = 0.0
    for i in segs:
        j = (i + 1) % n
        seg = math.dist(pts[i], pts[j])
        for A, B, sgn in ((L, None, -1), (R, None, 1)):
            a, b = A[i], A[j]
            mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
            cen = ((pts[i][0] + pts[j][0]) / 2, (pts[i][1] + pts[j][1]) / 2)
            f = (mid[0] - cen[0], 0, mid[1] - cen[1])
            mb.poly([(a[0], by[i] - 0.05, a[1]), (b[0], by[j] - 0.05, b[1]), (b[0], by[j] + h - 0.08, b[1]), (a[0], by[i] + h - 0.08, a[1])],
                    [(s, by[i]), (s + seg, by[j]), (s + seg, by[j] + h), (s, by[i] + h)], mat, facing=f)
        # Dekkhella.
        a, b, c, d = Lc[i], Lc[j], Rc[j], Rc[i]
        mb.poly([(a[0], by[i] + h, a[1]), (b[0], by[j] + h, b[1]), (c[0], by[j] + h, c[1]), (d[0], by[i] + h, d[1])],
                [(a[0], a[1]), (b[0], b[1]), (c[0], c[1]), (d[0], d[1])], cap, facing=(0, 1, 0))
        for A in (Lc, Rc):
            a, b = A[i], A[j]
            mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
            cen = ((pts[i][0] + pts[j][0]) / 2, (pts[i][1] + pts[j][1]) / 2)
            mb.poly([(a[0], by[i] + h - 0.08, a[1]), (b[0], by[j] + h - 0.08, b[1]), (b[0], by[j] + h, b[1]), (a[0], by[i] + h, a[1])],
                    [(s, 0), (s + seg, 0), (s + seg, 0.08), (s, 0.08)], cap, facing=(mid[0] - cen[0], 0, mid[1] - cen[1]))
            W = L if A is Lc else R
            c2, d2 = W[j], W[i]
            mb.poly([(a[0], by[i] + h - 0.08, a[1]), (b[0], by[j] + h - 0.08, b[1]), (c2[0], by[j] + h - 0.08, c2[1]), (d2[0], by[i] + h - 0.08, d2[1])],
                    [(0, 0), (1, 0), (1, 0.05), (0, 0.05)], cap, facing=(0, -1, 0))
        s += seg
    if ends and not closed:
        for i, o in ((0, 1), (n - 1, n - 2)):
            a, b = L[i], R[i]
            f = (pts[i][0] - pts[o][0], 0, pts[i][1] - pts[o][1])
            mb.poly([(a[0], by[i] - 0.05, a[1]), (b[0], by[i] - 0.05, b[1]), (b[0], by[i] + h - 0.08, b[1]), (a[0], by[i] + h - 0.08, a[1])],
                    [(0, 0), (t, 0), (t, h), (0, h)], mat, facing=f)
            a, b = Lc[i], Rc[i]
            mb.poly([(a[0], by[i] + h - 0.08, a[1]), (b[0], by[i] + h - 0.08, b[1]), (b[0], by[i] + h, b[1]), (a[0], by[i] + h, a[1])],
                    [(0, 0), (t, 0), (t, 0.08), (0, 0.08)], cap, facing=f)


def spiral_stair(mb, c, r_in, r_out, a0, a1, y0, y1, n, mat="dressed", inner_wall=True, outer_wall=False, newel=0.0, seg_per_step=2, outer_full=False):
    """Vindeltrapp (modulen «vindeltrapp»): `n` trinn rundt `c` fra vinkelen
    a0 til a1 (radianer i xz), fra y0 til y1, mellom r_in og r_out. Med
    `newel` > 0 står trappa rundt en spindel; ellers er innsida en mur ned
    til gulvet (en trapp langs veggen i et rundt rom). Gir høyden y(a)."""
    da = (a1 - a0) / n
    r = (y1 - y0) / n
    P = lambda q, a, y: (c[0] + q * math.cos(a), y, c[1] + q * math.sin(a))
    sgn = 1 if a1 > a0 else -1
    for k in range(n):
        aa, ab = a0 + k * da, a0 + (k + 1) * da
        yt = y0 + (k + 1) * r
        yp = y0 + k * r
        # Inntrinnet, delt langs buen.
        for q in range(seg_per_step):
            u0 = aa + (ab - aa) * q / seg_per_step
            u1 = aa + (ab - aa) * (q + 1) / seg_per_step
            pts = [P(r_in, u0, yt), P(r_out, u0, yt), P(r_out, u1, yt), P(r_in, u1, yt)]
            mb.poly(pts, [(p[0], p[2]) for p in pts], mat, facing=(0, 1, 0))
        # Opptrinnet ved aa, synlig nedenfra (mot lavere vinkel).
        tang = (-math.sin(aa) * sgn, 0, math.cos(aa) * sgn)
        lo, hi = min(yp, yt), max(yp, yt)
        pts = [P(r_in, aa, lo), P(r_out, aa, lo), P(r_out, aa, hi), P(r_in, aa, hi)]
        mb.poly(pts, [(0, lo), (r_out - r_in, lo), (r_out - r_in, hi), (0, hi)], mat, facing=mul(tang, -1 if r > 0 else 1))
        if inner_wall:
            # Muren under trinnet på innsida (mot rommet), ned til gulvet.
            for q in range(seg_per_step):
                u0 = aa + (ab - aa) * q / seg_per_step
                u1 = aa + (ab - aa) * (q + 1) / seg_per_step
                ylo = min(y0, y1) - 0.05
                pts = [P(r_in, u0, ylo), P(r_in, u1, ylo), P(r_in, u1, yt), P(r_in, u0, yt)]
                mid = lerp(pts[0], pts[2], 0.5)
                mb.poly(pts, [(r_in * u0, ylo), (r_in * u1, ylo), (r_in * u1, yt), (r_in * u0, yt)], "wall", facing=sub((c[0], mid[1], c[1]), mid))
        if outer_wall:
            for q in range(seg_per_step):
                u0 = aa + (ab - aa) * q / seg_per_step
                u1 = aa + (ab - aa) * (q + 1) / seg_per_step
                pts = [P(r_out, u0, yt - 2.0), P(r_out, u1, yt - 2.0), P(r_out, u1, yt), P(r_out, u0, yt)]
                mid = lerp(pts[0], pts[2], 0.5)
                mb.poly(pts, [(r_out * u0, yt - 2), (r_out * u1, yt - 2), (r_out * u1, yt), (r_out * u0, yt)], "wall", facing=sub(mid, (c[0], mid[1], c[1])))
        if outer_full:
            # Trappa rundt en kjerne midt i salen: utsida er murt ned til gulvet.
            for q in range(seg_per_step):
                u0 = aa + (ab - aa) * q / seg_per_step
                u1 = aa + (ab - aa) * (q + 1) / seg_per_step
                ylo = min(y0, y1) - 0.05
                pts = [P(r_out, u0, ylo), P(r_out, u1, ylo), P(r_out, u1, yt), P(r_out, u0, yt)]
                mid = lerp(pts[0], pts[2], 0.5)
                mb.poly(pts, [(r_out * u0, ylo), (r_out * u1, ylo), (r_out * u1, yt), (r_out * u0, yt)], mat, facing=sub(mid, (c[0], mid[1], c[1])))
    if newel > 0:
        cylinder(mb, c, newel, min(y0, y1), max(y0, y1) + 2.5, mat, n=8, caps=(False, False))
    return lambda a: y0 + (y1 - y0) * min(1.0, max(0.0, (a - a0) / (a1 - a0)))


def niche(mb, surf, u, v0, w=1.7, h=0.52, depth=0.62, sealed=False, rng=None):
    """Gravnisje (modulen «gravnisje»): en lang, lav nisje i muren for en
    kropp, med en liten bue øverst. Åpne nisjer har knokler og en
    hodeskalle (beinhusteksturen) på bunnen; lukkede har en gravhelle litt
    inn i nisjen. Gir omrisset (som må skjæres ut av muren)."""
    prof = [(u - w / 2, v0), (u + w / 2, v0), (u + w / 2, v0 + h)]
    for k in range(1, 6):
        a = k / 6
        prof.append((u + w / 2 - w * a, v0 + h + 0.1 * math.sin(math.pi * a)))
    prof.append((u - w / 2, v0 + h))
    if sealed:
        recess(mb, surf, prof, 0.07, "dressed", "slab")
    else:
        recess(mb, surf, prof, depth, "wall_dark", "wall_dark")
        # Knoklene: en lav haug langs bunnen.
        m = 0.08
        pts = [surf.point(u - w / 2 + m, v0, -depth + 0.05), surf.point(u + w / 2 - m, v0, -depth + 0.05),
               surf.point(u + w / 2 - m, v0 + 0.16, -0.12), surf.point(u - w / 2 + m, v0 + 0.16, -0.12)]
        mb.poly(pts, [(0, 0), (w, 0), (w, 0.6), (0, 0.6)], "bones", facing=add(surf.normal(u), (0, 1, 0)))
        for sgn in (-1, 1):
            e = [surf.point(u + sgn * (w / 2 - m), v0, -depth + 0.05), surf.point(u + sgn * (w / 2 - m), v0 + 0.16, -0.12), surf.point(u + sgn * (w / 2 - m), v0, -0.12)]
            mb.poly(e, [(0, 0), (0.5, 0.2), (0.5, 0)], "bones", facing=mul(add(surf.point(u + sgn, v0, 0), mul(surf.point(u, v0, 0), -1)), 1))
        # En hodeskalle.
        sk = surf.point(u + (w / 4 if rng is None else (rng.random() - 0.5) * w * 0.5), v0 + 0.12, -0.35)
        skull(mb, sk, surf.normal(u))
    return prof


def skull(mb, p, n, s=0.1):
    """En liten hodeskalle: en avrundet kloss med beinteksturen."""
    ring = circle((p[0], p[2]), s, 6)
    prism(mb, ring, p[1], p[1] + s * 1.5, "bones", smooth=True)


def ossuary_shelves(mb, surf, length, y0, y1, levels, mat_shelf="dressed", depth=0.18, thick=0.08, step=1.0):
    """Beinhushylle (modulen «beinhushylle»): hyllekanter av stein ut fra en
    mur med beinhusteksturen (knokler og hodeskaller stablet). Muren selv
    lages med `wall(..., mat='bones')`; her legges hyllene på."""
    m = max(1, math.ceil(length / step - 1e-9))
    us = [length * k / m for k in range(m + 1)]
    for lv in levels:
        for u0, u1 in zip(us, us[1:]):
            a, b = surf.point(u0, lv, 0), surf.point(u1, lv, 0)
            c, d = surf.point(u1, lv, depth), surf.point(u0, lv, depth)
            mb.poly([a, b, c, d], [surf.uv(u0, 0), surf.uv(u1, 0), surf.uv(u1, depth), surf.uv(u0, depth)], mat_shelf, facing=(0, 1, 0))
            a2, b2 = surf.point(u0, lv - thick, depth), surf.point(u1, lv - thick, depth)
            mb.poly([d, c, b2, a2], [surf.uv(u0, lv), surf.uv(u1, lv), surf.uv(u1, lv - thick), surf.uv(u0, lv - thick)], mat_shelf, facing=surf.normal((u0 + u1) / 2))
            a3, b3 = surf.point(u0, lv - thick, 0), surf.point(u1, lv - thick, 0)
            mb.poly([a3, b3, b2, a2], [surf.uv(u0, 0), surf.uv(u1, 0), surf.uv(u1, depth), surf.uv(u0, depth)], mat_shelf, facing=(0, -1, 0))


def glitch_seam(mb, surf, u, v0, v1, w=0.5, seed=3):
    """Fristil-korrupsjonen: en lysende søm i muren, et taggete bånd litt
    ut fra flaten med `glitch_seam`-laget som gløder."""
    import random

    rng = random.Random(seed)
    n = 9
    left, right = [], []
    for k in range(n + 1):
        v = v0 + (v1 - v0) * k / n
        c = u + (rng.random() - 0.5) * w * 0.6
        hw = w * (0.15 + 0.35 * math.sin(math.pi * k / n)) * (0.7 + 0.6 * rng.random())
        left.append((c - hw, v))
        right.append((c + hw, v))
    for k in range(n):
        pts = [left[k], right[k], right[k + 1], left[k + 1]]
        mb.poly([surf.point(a, b, 0.012) for a, b in pts], [(a * 1.0, b) for a, b in pts], "glitch", facing=surf.normal(u))


def glitch_stone(mb, surf, u, v, w=0.42, h=0.26):
    """En stein som lyser: en kloss i muren med korrupsjonslaget."""
    p = [(u - w / 2, v), (u + w / 2, v), (u + w / 2, v + h), (u - w / 2, v + h)]
    d = 0.03
    mb.poly([surf.point(a, b, d) for a, b in p], [(a, b) for a, b in p], "glitch", facing=surf.normal(u))
    for i in range(4):
        a, b = p[i], p[(i + 1) % 4]
        mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        mb.poly([surf.point(*a, 0), surf.point(*b, 0), surf.point(*b, d), surf.point(*a, d)], [(0, 0), (1, 0), (1, d), (0, d)], "glitch",
                facing=sub(surf.point(*mid, d / 2), surf.point(u, v + h / 2, d / 2)))


def arrow_slots(mb, surf, u, v, n=5, spacing=0.2, w=0.05, h=0.32, panel=True):
    """Pilhull: smale, dype spalter i muren (og et panel av hogd stein rundt)."""
    width = (n - 1) * spacing + 0.4
    if panel:
        pr = rect_profile(u, width, v - 0.12, h + 0.24)
        for i in range(4):
            a, b = pr[i], pr[(i + 1) % 4]
            mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
            mb.poly([surf.point(*a, 0), surf.point(*b, 0), surf.point(*b, 0.05), surf.point(*a, 0.05)], [(0, 0), (1, 0), (1, 0.05), (0, 0.05)], "dressed",
                    facing=sub(surf.point(*mid, 0), surf.point(u, v + h / 2, 0)))
        face = Polygon(pr)
        holes = []
        for k in range(n):
            uk = u - (n - 1) * spacing / 2 + k * spacing
            holes.append(Polygon(rect_profile(uk, w, v, h)))
        for hpoly in holes:
            face = face.difference(hpoly)
        grid_face(mb, _Offset(surf, 0.05), face, "dressed", cell=1.0)
    for k in range(n):
        uk = u - (n - 1) * spacing / 2 + k * spacing
        recess(mb, _Offset(surf, 0.05 if panel else 0.0), rect_profile(uk, w, v, h), 0.4, "wall_dark", "wall_dark")


class _Offset:
    """En flate flyttet `d` ut mot rommet."""

    def __init__(self, s, d):
        self.s, self.d = s, d

    def point(self, u, v, d=0.0):
        return self.s.point(u, v, d + self.d)

    def normal(self, u=0.0):
        return self.s.normal(u)

    def uv(self, u, v):
        return self.s.uv(u, v)


# --- Bjelker, jernbånd og karmer --------------------------------------------------


def beam(mb, a, b, w, h, mat, up=(0.0, 1.0, 0.0), strip=None):
    """En firkantet bjelke fra a til b (w bred, h høy, «høyden» langs `up`).
    Uv: u langs bjelken i meter; v på tvers i meter, eller en stripe av et
    ark: ("timber", k) rad k av fire, ("trim", k) stripe k av åtte (jernbånd
    med nagler er `IRON_BAND`)."""
    d = norm(sub(b, a))
    upv = norm(sub(up, mul(d, dot(up, d))))
    if dot(upv, upv) < 1e-9:
        upv = norm(cross(d, (1.0, 0.0, 0.0)))
    side = norm(cross(upv, d))
    L = math.dist(a, b)
    corners = [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2)]
    P = lambda p0, cx, cy: add(add(p0, mul(side, cx)), mul(upv, cy))

    def v_of(t, span):
        if strip is None:
            return t * span
        kind, k = strip
        if kind == "timber":
            return k * 0.25 + 0.02 + t * 0.21
        v0, v1 = 1 - (k + 1) / 8, 1 - k / 8
        return v0 + 0.006 + t * (v1 - v0 - 0.012)

    def u_of(x, span):
        return x * 0.125 / span if strip and strip[0] == "trim" else x

    for i in range(4):
        (ax, ay), (bx, by) = corners[i], corners[(i + 1) % 4]
        span = w if i % 2 == 0 else h
        pts = [P(a, ax, ay), P(a, bx, by), P(b, bx, by), P(b, ax, ay)]
        uvs = [(u_of(0, span), v_of(0, span)), (u_of(0, span), v_of(1, span)), (u_of(L, span), v_of(1, span)), (u_of(L, span), v_of(0, span))]
        mid = add(mul(side, (ax + bx) / 2), mul(upv, (ay + by) / 2))
        mb.poly(pts, uvs, mat, facing=mid)
    for p0, f in ((a, mul(d, -1)), (b, d)):
        pts = [P(p0, cx, cy) for cx, cy in corners]
        mb.poly(pts, [(c[0] + w / 2, c[1] + h / 2) for c in corners], mat, facing=f)


IRON_BAND = 5


def strap(mb, a, b, w, t, up):
    """Et smijernsbånd med nagler (trim-arket) fra a til b, `t` tykt ut
    langs `up`."""
    beam(mb, a, b, w, t, "strap", up=up, strip=("trim", IRON_BAND))


# Enkeltsteinene i kryptmurens bilde (`crypt_wall`, 1024 px på 2 m), som
# piksler (x0, y0, x1, y1) innenfor fugene. En stein i en karm får én av dem
# over hele forsida, med steinens lange akse langs steinen, så bildet følger
# buen i stedet for vannrette skift som kuttes av steinens kant (eieren i
# runde 8: «steinene legges slik at de følger buen»).
STONE_PX = [(32, 60, 205, 174), (226, 60, 389, 174), (408, 60, 560, 174), (578, 60, 743, 174), (133, 196, 296, 312), (672, 196, 827, 312),
            (24, 334, 204, 450), (753, 334, 910, 450), (95, 474, 246, 588), (264, 474, 397, 588), (404, 474, 583, 588), (600, 474, 790, 588),
            (810, 474, 968, 588), (697, 610, 876, 729), (32, 750, 239, 864), (260, 750, 389, 864), (401, 750, 563, 864)]
STONE_TEX_M = 2.0  # crypt_wall: size_m i meta.json
STONE_TEX_PX = 1024


def stone_uv(k, pts):
    """Uv (i meter, som laget) som legger enkeltstein `k` over firkanten
    `pts` (i rekkefølge rundt), med steinbildets lange akse langs den lengste
    sida av firkanten."""
    x0, y0, x1, y1 = STONE_PX[k % len(STONE_PX)]
    inset = 4
    f = STONE_TEX_M / STONE_TEX_PX
    u0, u1, v0, v1 = (x0 + inset) * f, (x1 - inset) * f, (y0 + inset) * f, (y1 - inset) * f
    corners = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
    if math.dist(pts[0], pts[1]) < math.dist(pts[1], pts[2]):
        corners = [(u0, v0), (u0, v1), (u1, v1), (u1, v0)]
    return corners


def arch_trim(mb, surf, w, spring, v0=0.0, band=0.3, proud=0.1, n=9, mat="dressed", seed=0):
    """Karmen rundt en rundbuet åpning i en mur (`surf`, u = 0 midt i
    åpningen): vangestein annenhver bred og smal opp til buen, og
    kilesteiner som peker inn mot buens sentrum med sluttstein øverst, hver
    for seg med en smal fuge og `proud` ut fra muren (eierens dom i runde 7:
    en trim langs kanten). Hver stein viser én hel stein fra murbildet langs
    seg (`stone_uv`), og bak fugene ligger mørtel, så kanten ikke ser
    hullete ut (runde 8)."""
    r = w / 2
    g = 0.008
    k = seed
    # Mørtelen bak steinene: vangene og buen, litt inn fra forsida.
    back = proud * 0.35
    # Bare bak steinene: vangene så brede som de smale vangesteinene, og
    # buen innenfor kilesteinene (ellers synes mørtelen som en brun kant).
    for sx in (-1, 1):
        u0, u1 = sorted((sx * (r + 0.005), sx * (r + band * 0.7 - 0.01)))
        q = [(u0, v0), (u1, v0), (u1, v0 + spring), (u0, v0 + spring)]
        mb.poly([surf.point(u, v, back) for u, v in q], [surf.uv(u, v) for u, v in q], "mortar", facing=surf.normal(0.0))
    zs = v0 + spring
    m_ = 24
    for i in range(m_):
        a0, a1 = math.pi * i / m_, math.pi * (i + 1) / m_
        ri, ro = r + 0.005, r + band - 0.01
        q = [(ri * math.cos(a0), zs + ri * math.sin(a0)), (ro * math.cos(a0), zs + ro * math.sin(a0)), (ro * math.cos(a1), zs + ro * math.sin(a1)), (ri * math.cos(a1), zs + ri * math.sin(a1))]
        mb.poly([surf.point(u, v, back) for u, v in q], [surf.uv(u, v) for u, v in q], "mortar", facing=surf.normal(0.0))
    z = v0
    j = 0
    while z < v0 + spring - 0.05:
        h = min(0.42 if j % 2 == 0 else 0.3, v0 + spring - z)
        bw = band if j % 2 == 0 else band * 0.7
        for sx in (-1, 1):
            u0, u1 = sorted((sx * r, sx * (r + bw)))
            pts = [(u0, z + g), (u1, z + g), (u1, z + h - g), (u0, z + h - g)]
            _trim_block(mb, surf, pts, proud, mat, stone_uv(k, pts))
            k += 3
        z += h
        j += 1
    for i in range(n):
        a0 = math.pi * i / n + g / (r + band / 2)
        a1 = math.pi * (i + 1) / n - g / (r + band / 2)
        ro = r + band + (0.06 if i == n // 2 else 0.0)
        pts = [(r * math.cos(a0), zs + r * math.sin(a0)), (ro * math.cos(a0), zs + ro * math.sin(a0)), (ro * math.cos(a1), zs + ro * math.sin(a1)), (r * math.cos(a1), zs + r * math.sin(a1))]
        _trim_block(mb, surf, pts, proud, mat, stone_uv(k, pts))
        k += 5


def _trim_block(mb, surf, pts, proud, mat, uvs=None):
    """En stein i karmen: forsiden `proud` ut fra muren (med `uvs` på
    forsida, ellers murens uv), og kantene."""
    front = [surf.point(u, v, proud) for u, v in pts]
    mb.poly(front, uvs or [surf.uv(u, v) for u, v in pts], mat, facing=surf.normal(0.0))
    cu = sum(u for u, _ in pts) / len(pts)
    cv = sum(v for _, v in pts) / len(pts)
    for i in range(len(pts)):
        a, b = pts[i], pts[(i + 1) % len(pts)]
        L = math.dist(a, b)
        quad = [surf.point(*a, 0.0), surf.point(*b, 0.0), surf.point(*b, proud), surf.point(*a, proud)]
        mid = surf.point((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, proud / 2)
        mb.poly(quad, [(0, 0), (L, 0), (L, proud), (0, proud)], mat, facing=sub(mid, surf.point(cu, cv, proud / 2)))


def kit_rest():
    """Hvilestedet: et lite alter med en stor lysestake (sju lys) som tennes.
    Flammene er en egen modell (`kit_rest_flame`), som motoren viser når
    hvilestedet er tent."""
    m, fl = MB(), MB()
    altar(m, 0, 0, 0, 0.0, w=1.5, d=0.8, h=1.0)
    # Den store lysestaken står på alterplata.
    y = 1.02
    cylinder(m, (0, 0), 0.16, y, y + 0.06, "iron", n=8, caps=(True, False), smooth=False)
    cylinder(m, (0, 0), 0.035, y + 0.06, y + 0.62, "iron", n=6, caps=(False, False))
    tips = []
    for k, (ox, hh) in enumerate([(-0.45, 0.42), (-0.3, 0.5), (-0.15, 0.56), (0.0, 0.62), (0.15, 0.56), (0.3, 0.5), (0.45, 0.42)]):
        if ox != 0.0:
            band(m, [(0.0, y + 0.3, 0.0), (ox * 0.5, y + 0.3, 0.0), (ox, y + hh, 0.0)], 0.03, 0.03, mat="iron", down=lambda p: (0, -1, 0))
        cylinder(m, (ox, 0), 0.06, y + hh, y + hh + 0.03, "iron", n=6, caps=(True, True), smooth=False)
        ch = 0.16 + 0.06 * (k % 3)
        cylinder(m, (ox, 0), 0.03, y + hh + 0.03, y + hh + 0.03 + ch, "wax", n=6, caps=(True, False))
        tips.append((ox, y + hh + 0.03 + ch, 0.0))
    for t in tips:
        f = 0.055
        tip = (t[0], t[1] + 0.01 + f * 1.8, t[2])
        base = [(t[0] + f * 0.4 * math.cos(a), t[1] + 0.01 + f * 0.5, t[2] + f * 0.4 * math.sin(a)) for a in (0, 2.09, 4.19)]
        c = (t[0], t[1] + 0.01, t[2])
        for i in range(3):
            fl.tri(base[i], base[(i + 1) % 3], tip, "fire_candle", facing=sub(lerp(base[i], base[(i + 1) % 3], 0.5), c))
            fl.tri(base[i], base[(i + 1) % 3], c, "fire_candle", facing=add(sub(lerp(base[i], base[(i + 1) % 3], 0.5), c), (0, -0.05, 0)))
    return m, fl, tips


def kit_lever():
    """Spaken: en sokkel av hogd stein med en helle på toppen, to jernkinn
    med akselen mellom seg (dreiepunktet `LEVER_PIVOT`, 0,56 m oppe), og
    håndtaket for seg: et nav på akselen, en jernstang og et grep av tre
    (dreies om x i origo, eierens dom i runde 7: «modellen ser rar ut»)."""
    base, handle = MB(), MB()
    box(base, (-0.32, 0, -0.26), (0.32, 0.1, 0.26), "dressed", faces=("+y", "+x", "-x", "+z", "-z"))
    box(base, (-0.24, 0.1, -0.19), (0.24, 0.42, 0.19), "dressed", faces=("+x", "-x", "+z", "-z"))
    box(base, (-0.27, 0.42, -0.22), (0.27, 0.47, 0.22), "slab", faces=("+y", "-y", "+x", "-x", "+z", "-z"))
    box(base, (-0.15, 0.47, -0.1), (0.15, 0.49, 0.1), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    for x0, x1 in ((-0.11, -0.065), (0.065, 0.11)):
        box(base, (x0, 0.49, -0.07), (x1, 0.64, 0.07), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    beam(base, (-0.13, 0.56, 0.0), (0.13, 0.56, 0.0), 0.035, 0.035, "iron")
    # Håndtaket (origo i dreiepunktet): navet går rundt akselen mellom kinnene.
    beam(handle, (-0.055, 0.0, 0.0), (0.055, 0.0, 0.0), 0.075, 0.075, "iron")
    beam(handle, (0.0, 0.035, 0.0), (0.0, 0.66, 0.0), 0.04, 0.04, "iron", up=(0.0, 0.0, 1.0))
    beam(handle, (0.0, 0.62, 0.0), (0.0, 0.9, 0.0), 0.065, 0.065, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 1))
    beam(handle, (0.0, 0.9, 0.0), (0.0, 0.93, 0.0), 0.08, 0.08, "iron", up=(0.0, 0.0, 1.0))
    return base, handle


def kit_gate(w=2.8, h=3.0):
    """Gitteret: smijernsstenger i rutenett med spisser nederst."""
    m = MB()
    nv = 9
    t = 0.06
    for k in range(nv):
        x = -w / 2 + 0.1 + (w - 0.2) * k / (nv - 1)
        box(m, (x - t / 2, 0.12, -t / 2), (x + t / 2, h, t / 2), "iron", faces=("+x", "-x", "+z", "-z"))
        tip = (x, 0.0, 0.0)
        ring = [(x - t / 2, 0.12, -t / 2), (x + t / 2, 0.12, -t / 2), (x + t / 2, 0.12, t / 2), (x - t / 2, 0.12, t / 2)]
        for i in range(4):
            a, b = ring[i], ring[(i + 1) % 4]
            m.tri(a, b, tip, "iron", facing=sub(lerp(a, b, 0.5), (x, 0.12, 0)))
    for y in (0.55, 1.35, 2.15, 2.85):
        box(m, (-w / 2, y - 0.05, -t / 2 - 0.02), (w / 2, y + 0.05, t / 2 + 0.02), "iron")
    return m


def shortcut_leaf(w=1.6, spring=1.5, t=0.12):
    """Et rundbuet dørblad (hengslet i origo, bladet går mot +x): hele
    dørbildet over bladet foran og bak (uv 0–1), kantene av planker."""
    leaf = MB()
    prof = arch_profile(w / 2, w - 0.04, 0.0, spring, n=10)
    H = spring + (w - 0.04) / 2
    for z, f in ((t / 2, 1.0), (-t / 2, -1.0)):
        pts = [(u, v, z) for u, v in prof]
        uvs = [((u / w) if f > 0 else 1 - u / w, v / H) for u, v in prof]
        leaf.poly(pts, uvs, "leaf", facing=(0, 0, f))
    for i in range(len(prof)):
        a, b = prof[i], prof[(i + 1) % len(prof)]
        mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        L = math.dist(a, b)
        leaf.poly([(a[0], a[1], -t / 2), (b[0], b[1], -t / 2), (b[0], b[1], t / 2), (a[0], a[1], t / 2)], [(0, 0), (L, 0), (L, t), (0, t)], "oak",
                  facing=(mid[0] - w / 2, mid[1] - 1.0, 0))
    return leaf


def kit_shortcut(w=1.6, spring=1.5):
    """Snarveidøra: et rundbuet blad med dørbildet (hengslet i origo, bladet
    går mot +x, toppen 2,3 m), og slåen på slåsida (+z): en bjelke som
    ligger fritt i to U-kramper av jern, med luft rundt, så ingenting går
    inn i hverandre (eierens dom i runde 7). Høyden må stemme med
    `SHORTCUT_H` i motoren."""
    t = 0.12
    leaf = shortcut_leaf(w, spring, t)
    bar = MB()
    f = t / 2
    for x in (0.24, w - 0.24):
        box(leaf, (x - 0.035, 0.965, f), (x + 0.035, 0.995, f + 0.14), "iron", faces=("+y", "-y", "+x", "-x", "+z"))
        box(leaf, (x - 0.035, 1.105, f), (x + 0.035, 1.135, f + 0.14), "iron", faces=("+y", "-y", "+x", "-x", "+z"))
        box(leaf, (x - 0.035, 0.965, f + 0.115), (x + 0.035, 1.135, f + 0.14), "iron", faces=("+y", "-y", "+x", "-x", "+z"))
    beam(leaf, (0.1, 1.05, f + 0.065), (w - 0.1, 1.05, f + 0.065), 0.09, 0.1, "wood", strip=("timber", 2))
    beam(bar, (-w / 2 - 0.25, 0.0, 0.0), (w / 2 + 0.25, 0.0, 0.0), 0.12, 0.14, "wood", strip=("timber", 2))
    return leaf, bar


def kit_shortcut_frame(w=1.6, spring=1.5, depth=0.5):
    """Karmen rundt snarveidøra: vangestein og buestein som står ut av muren
    på begge sider (`arch_trim`), lysningen gjennom muren og terskelen
    (origo midt nederst i åpningen, muren går langs x)."""
    m = MB()
    prof = arch_profile(0.0, w, 0.0, spring, n=10)
    arch_trim(m, Plane((0.0, depth / 2), (1.0, 0.0), (0.0, 1.0)), w, spring)
    arch_trim(m, Plane((0.0, -depth / 2), (-1.0, 0.0), (0.0, -1.0)), w, spring)
    # Lysningen gjennom muren.
    for i in range(len(prof) - 1):
        a, b = prof[i], prof[i + 1]
        L = math.dist(a, b)
        mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        m.poly([(a[0], a[1], -depth / 2), (b[0], b[1], -depth / 2), (b[0], b[1], depth / 2), (a[0], a[1], depth / 2)],
               [(0, 0), (L, 0), (L, depth), (0, depth)], "dressed", facing=(-mid[0], spring * 0.6 - mid[1], 0))
    # Terskelen.
    box(m, (-w / 2, -0.02, -depth / 2), (w / 2, 0.03, depth / 2), "slab", faces=("+y",))
    return m


def kit_false_wall(w=2.0, h=2.6, t=0.4):
    """Den falske veggen (reserven når nivået ikke har sin egen bit,
    `secret_pieces` i build_level.py): én hel murbit med fronten i flukt med
    muren (z = 0, mot +z) og resten inne i muren (eieren 18. oktober: den
    stakk ut og røpet seg), med murens tekstur og sprekken i fugene på seg."""
    m = MB()
    box(m, (-w / 2, 0.0, -t), (w / 2, h, 0.0), "wall", faces=("+z", "-z", "+x", "-x", "+y"))
    mortar_crack(m, 0.0, 11, x0=0.2, half=w / 2, top=1.9)
    return m


def surface_crack(m, surf, u0, u1, y0, y1, seed, d=0.0, width=0.015):
    """Sprekken i fugene på en mur i nivået (`surf`, u langs muren, v
    absolutt høyde) mellom u0 og u1 og fra y0 til y1: samme trapp som
    `mortar_crack`, men i murens egen tekstur, så den følger fugene der
    den falske veggen står (`secret_pieces` i build_level.py)."""
    import random

    rng = random.Random(seed)
    u0t, u1t = surf.uv(u0, 0.0)[0], surf.uv(u1, 0.0)[0]
    shift = STONE_TEX_M * math.floor(y0 / STONE_TEX_M)
    courses = [(a + shift, b + shift, heads) for a, b, heads in wall_courses(y1 - shift + STONE_TEX_M) if b + shift > y0 + 0.05 and a + shift < y1]
    tile = lambda h: [h + STONE_TEX_M * j for j in range(math.floor((u0t - 2) / STONE_TEX_M), math.ceil((u1t + 2) / STONE_TEX_M) + 1)]
    pts = []
    x = (u0t + u1t) / 2 + rng.uniform(-0.3, 0.3)
    for a, b, heads in courses:
        hs = sorted({hh for h in heads for hh in tile(h - STONE_TEX_M * math.floor(h / STONE_TEX_M)) if u0t + 0.25 < hh < u1t - 0.25})
        if not hs:
            break
        near = sorted(hs, key=lambda h: abs(h - x))
        h = near[0] if rng.random() < 0.65 or len(near) < 2 else near[1]
        pts.append((h, max(a, y0 + 0.02)))
        pts.append((h, min(b, y1)))
        x = h
    n = len(pts) - 1
    P = lambda ut, v, dd: surf.point(ut - (u0t - u0), v, dd)
    for i in range(n):
        (ax, ay), (bx, by) = pts[i], pts[i + 1]
        w0 = width * (1.0 - 0.6 * i / max(n, 1))
        w1 = width * (1.0 - 0.6 * (i + 1) / max(n, 1))
        dx, dy = bx - ax, by - ay
        ln = math.hypot(dx, dy) or 1.0
        nx, ny = -dy / ln, dx / ln
        ex, ey = dx / ln * w1, dy / ln * w1
        q = [(ax - nx * w0 - ex, ay - ny * w0 - ey), (bx - nx * w1 + ex, by - ny * w1 + ey), (bx + nx * w1 + ex, by + ny * w1 + ey), (ax + nx * w0 - ex, ay + ny * w0 - ey)]
        m.quad(*[P(qx, qy, d + 0.004) for qx, qy in q], "crack", facing=surf.normal((u0 + u1) / 2))


def kit_trapdoor(w=1.2, d=1.2, t=0.08):
    """Fallemmen: planker (2 cm over gulvet, så den ikke ligger i samme plan
    som gulvet), to hengselbånd av smijern fra hengslet, og en jernring på
    en plate foran, så du ser at den kan åpnes (eierens dom i runde 7).
    Hengslet langs kanten mot −z."""
    m = MB()
    top = 0.02
    box(m, (-w / 2, top - t, 0.0), (w / 2, top, d), "oak")
    for x in (-0.35, 0.35):
        strap(m, (x, top + 0.006, 0.02), (x, top + 0.006, d - 0.12), 0.07, 0.012, (0.0, 1.0, 0.0))
    cz = d - 0.24
    box(m, (-0.07, top, cz - 0.05), (0.07, top + 0.012, cz + 0.05), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
    # Ringen ligger flatt foran plata.
    r = 0.075
    c = (0.0, top + 0.024, cz + 0.07)
    pts = [(c[0] + r * math.cos(a), c[1], c[2] + r * math.sin(a)) for a in [2 * math.pi * k / 10 for k in range(11)]]
    for a, b in zip(pts, pts[1:]):
        beam(m, a, b, 0.022, 0.022, "iron")
    return m


def kit_bridge(w=3.6, L=8.0):
    """Broa over avgrunnen (motoren: `KIT_BRIDGE`, `BRIDGE_W` × `BRIDGE_L`):
    et dekke av eikeplanker på tvers med origo midt på oversida, to
    langbjelker og tverrbjelker under, lave kantbjelker oppå, og jernøyer
    i hjørnene der kjettingene er festet. Lengden langs z."""
    m = MB()
    box(m, (-w / 2, -0.12, -L / 2), (w / 2, 0.0, L / 2), "oak")
    for x in (-w / 2 + 0.25, w / 2 - 0.25):
        beam(m, (x, -0.27, -L / 2), (x, -0.27, L / 2), 0.22, 0.3, "wood", strip=("timber", 0))
    for k in range(5):
        z = -L / 2 + 0.3 + (L - 0.6) * k / 4
        beam(m, (-w / 2, -0.21, z), (w / 2, -0.21, z), 0.18, 0.18, "wood", up=(0.0, 1.0, 0.0), strip=("timber", 2))
    for x in (-w / 2 + 0.08, w / 2 - 0.08):
        beam(m, (x, 0.07, -L / 2), (x, 0.07, L / 2), 0.16, 0.14, "wood", strip=("timber", 1))
    for sx in (-1, 1):
        for sz in (-1, 1):
            x, z = sx * (w / 2 - 0.2), sz * (L / 2 - 0.35)
            box(m, (x - 0.09, 0.14, z - 0.09), (x + 0.09, 0.2, z + 0.09), "iron", faces=("+y", "+x", "-x", "+z", "-z"))
            strap(m, (x, 0.145, z - sz * 0.6), (x, 0.145, z), 0.08, 0.012, (0.0, 1.0, 0.0))
    return m


def kit_water(n=4):
    """Vannflaten (`KIT_WATER`): 1 × 1 m med origo midt på, delt i ruter så
    den kan skaleres til kanalen; økta senker den når slusa dreies."""
    m = MB()
    for i in range(n):
        for j in range(n):
            x0, z0 = -0.5 + i / n, -0.5 + j / n
            x1, z1 = x0 + 1 / n, z0 + 1 / n
            m.quad((x0, 0.0, z0), (x1, 0.0, z0), (x1, 0.0, z1), (x0, 0.0, z1), "water", facing=(0, 1, 0), uv=[(x0, z0), (x1, z0), (x1, z1), (x0, z1)])
    return m


def kit_chain(length=12.0, link=0.2):
    """Kjettingen broa henger i (`KIT_CHAIN`): ledd av smijern fra origo
    (øyet på dekket) og 12 m opp, så den går inn i hvelvet over avgrunnen
    uansett hvor broa henger."""
    m = MB()
    n = int(length / link)
    for k in range(n):
        y0 = k * link
        a, b = (0.05, 0.016) if k % 2 == 0 else (0.016, 0.05)
        box(m, (-a, y0, -b), (a, y0 + link + 0.03, b), "iron", faces=("+x", "-x", "+z", "-z"))
    return m


def rod(mb, a, b, r, mat, n=6, row=1):
    """En rund stang (trinn) fra a til b med `n` sider, uv fra timber-arket
    (rad `row`: u langs stanga i meter, v rundt)."""
    d = norm(sub(b, a))
    up = (0.0, 1.0, 0.0) if abs(d[1]) < 0.9 else (1.0, 0.0, 0.0)
    e1 = norm(cross(up, d))
    e2 = cross(d, e1)
    L = math.dist(a, b)
    ring = [add(mul(e1, r * math.cos(2 * math.pi * k / n)), mul(e2, r * math.sin(2 * math.pi * k / n))) for k in range(n)]
    for k in range(n):
        p, q = ring[k], ring[(k + 1) % n]
        v0, v1 = row * 0.25 + 0.02 + 0.21 * k / n, row * 0.25 + 0.02 + 0.21 * (k + 1) / n
        mb.poly([add(a, p), add(a, q), add(b, q), add(b, p)], [(0.0, v0), (0.0, v1), (L, v1), (L, v0)], mat, facing=add(p, q))
    for c, f in ((a, mul(d, -1)), (b, d)):
        mb.poly([add(c, p) for p in ring], [(0.5 + p[0], 0.5 + p[1]) for p in ring], mat, facing=f)


LADDER_RUNGS = 3


def kit_ladder(w=0.48, sw=0.075, sd=0.06):
    """Stigen (eieren 18. oktober: «ser ikke ut som en stige»; før var
    biten på 1 m strukket til hele høyden, så fire trinn ble to meter fra
    hverandre): en bit på 1 m som motoren stabler (`KIT_LADDER`, skalert
    bare litt så bitene går opp i høyden). To kløyvde vanger av eik og tre
    runde trinn som går gjennom vangene og stikker litt ut på hver side,
    33 cm mellom trinnene som på en ekte stige. Origo midt under biten,
    trinnene mot +z; vangene står inntil muren bak (−z)."""
    m = MB()
    for k, x in enumerate((-w / 2, w / 2)):
        beam(m, (x, 0.0, 0.0), (x, 1.0, 0.0), sw, sd, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 0 if k == 0 else 3))
    for k in range(LADDER_RUNGS):
        y = (k + 0.5) / LADDER_RUNGS
        rod(m, (-w / 2 - sw / 2 - 0.018, y, 0.004), (w / 2 + sw / 2 + 0.018, y, 0.004), 0.021, "wood", row=1 + k % 2)
    return m


def kit_ladder_top(w=0.48, sw=0.075, sd=0.06):
    """Toppen av stigen (`KIT_LADDER_TOP`, origo i kanten der du går av,
    på vangenes midtlinje): vangene ender i avrundede hoder litt over kanten,
    og to kroker av smijern, naglet til vangene, er bøyd over kanten og
    ligger på gulvet oppe (mot −z), så du ser at stigen henger fast i
    kanten av lemmen."""
    m = MB()
    for k, x in enumerate((-w / 2, w / 2)):
        # Hodet på vangen: et kort stykke med skrå topp.
        beam(m, (x, 0.0, 0.0), (x, 0.06, 0.0), sw, sd, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 0 if k == 0 else 3))
        m.quad((x - sw / 2, 0.06, sd / 2), (x + sw / 2, 0.06, sd / 2), (x + sw / 2, 0.1, -sd / 2), (x - sw / 2, 0.1, -sd / 2), "wood", facing=(0, 1, 1))
        for sx in (-1, 1):
            m.tri((x + sx * sw / 2, 0.06, sd / 2), (x + sx * sw / 2, 0.06, -sd / 2), (x + sx * sw / 2, 0.1, -sd / 2), "wood", facing=(sx, 0, 0))
        # Kroken: opp langs baksida av vangen, over kanten og ned på gulvet.
        z = -sd / 2 - 0.006
        hw, ht = 0.035, 0.012
        beam(m, (x, -0.32, z), (x, 0.05, z), hw, ht, "iron", up=(0.0, 0.0, -1.0))
        beam(m, (x, 0.05 - ht / 2, z + ht / 2), (x, 0.05 - ht / 2, -0.22), hw, ht, "iron", up=(0.0, 1.0, 0.0))
        beam(m, (x, 0.05, -0.22 + ht / 2), (x, 0.0, -0.22 + ht / 2), hw, ht, "iron", up=(0.0, 0.0, -1.0))
        # Naglene gjennom vangen.
        for y in (-0.27, -0.12):
            box(m, (x - 0.011, y - 0.011, -sd / 2 - 0.016), (x + 0.011, y + 0.011, -sd / 2 - 0.012), "iron", faces=("-z", "+x", "-x", "+y", "-y"))
            box(m, (x - 0.011, y - 0.011, sd / 2), (x + 0.011, y + 0.011, sd / 2 + 0.006), "iron", faces=("+z", "+x", "-x", "+y", "-y"))
    return m


def kit_plate(w=1.0, d=1.0, h=0.05):
    """Trykkplaten: en gravhelle som står litt opp fra gulvet og synker."""
    m = MB()
    box(m, (-w / 2, -0.15, -d / 2), (w / 2, h, d / 2), "slab", faces=("+y", "+x", "-x", "+z", "-z"))
    return m


def kit_arrow_slots():
    """Pilhullene: et panel med fem spalter, i veggflaten (z = 0), mot +z."""
    m = MB()
    surf = Plane((0.0, 0.0), (1.0, 0.0), (0.0, 1.0))
    arrow_slots(m, surf, 0.0, -0.16)
    return m


def kit_arrow(L=0.62):
    """Pila: et skaft med jernspiss foran (+z) og fjær bak, origo midt på."""
    m = MB()
    r = 0.012
    box(m, (-r, -r, -L / 2), (r, r, L / 2 - 0.08), "wood", faces=("+x", "-x", "+y", "-y"))
    tip = (0.0, 0.0, L / 2 + 0.04)
    ring = [(-0.03, 0.0, L / 2 - 0.08), (0.0, 0.03, L / 2 - 0.08), (0.03, 0.0, L / 2 - 0.08), (0.0, -0.03, L / 2 - 0.08)]
    for i in range(4):
        a, b = ring[i], ring[(i + 1) % 4]
        m.tri(a, b, tip, "iron", facing=lerp(a, b, 0.5))
        m.tri(a, b, (0.0, 0.0, L / 2 - 0.1), "iron", facing=(0, 0, -1))
    for a in (0.0, 2.09, 4.19):
        d = (math.cos(a) * 0.04, math.sin(a) * 0.04, 0.0)
        p0, p1 = (0.0, 0.0, -L / 2), (0.0, 0.0, -L / 2 + 0.14)
        m.quad(p0, p1, add(p1, d), add(p0, (d[0] * 1.3, d[1] * 1.3, 0.0)), "wax", facing=(-d[1], d[0], 0.0))
        m.quad(p0, add(p0, (d[0] * 1.3, d[1] * 1.3, 0.0)), add(p1, d), p1, "wax", facing=(d[1], -d[0], 0.0))
    return m


def kit_lore():
    """Lappen eller boka: en lesepult av tre (kryssføtter, stolpe, knekter
    og en skrå pultplate med list), med en åpen bok i skinnperm og et lys
    på en jernarm (origo på gulvet, leseren står på +z; eierens dom i
    runde 7: av treverk, ikke stein)."""
    m = MB()
    beam(m, (-0.3, 0.04, 0.0), (0.3, 0.04, 0.0), 0.08, 0.08, "wood", strip=("timber", 1))
    for z0, z1 in ((-0.25, -0.04), (0.04, 0.25)):
        beam(m, (0.0, 0.04, z0), (0.0, 0.04, z1), 0.08, 0.08, "wood", strip=("timber", 1))
    # Pulten heller mot leseren: bak høyt, foran lavt.
    yb, yf, zb, zf, th = 1.02, 0.86, -0.2, 0.2, 0.04
    beam(m, (0.0, 0.08, 0.0), (0.0, (yb + yf) / 2 - th, 0.0), 0.1, 0.1, "wood", up=(0.0, 0.0, 1.0), strip=("timber", 2))
    for sx in (-1, 1):
        beam(m, (sx * 0.04, 0.72, 0.0), (sx * 0.22, (yb + yf) / 2 - th - 0.005, 0.0), 0.05, 0.05, "wood", strip=("timber", 3))
    ez = norm((0.0, yf - yb, zf - zb))
    n = norm(cross((1.0, 0.0, 0.0), ez))
    if n[1] < 0:
        n = mul(n, -1)
    c = (0.0, (yb + yf) / 2, 0.0)
    half = math.dist((yb, zb), (yf, zf)) / 2
    at = lambda u, s_, h: add(add(add(c, (u, 0.0, 0.0)), mul(ez, s_)), mul(n, h))

    def slab_(u0, u1, s0, s1, h0, h1, mat):
        corners = [(u0, s0), (u1, s0), (u1, s1), (u0, s1)]
        top = [at(u, s_, h1) for u, s_ in corners]
        bot = [at(u, s_, h0) for u, s_ in corners]
        m.poly(top, [(u, s_) for u, s_ in corners], mat, facing=n)
        m.poly(bot, [(u, s_) for u, s_ in corners], mat, facing=mul(n, -1))
        for i in range(4):
            a, b = corners[i], corners[(i + 1) % 4]
            L = math.dist(a, b)
            quad = [at(*a, h0), at(*b, h0), at(*b, h1), at(*a, h1)]
            mid = at((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (h0 + h1) / 2)
            m.poly(quad, [(0, 0), (L, 0), (L, h1 - h0), (0, h1 - h0)], mat, facing=sub(mid, at((u0 + u1) / 2, (s0 + s1) / 2, (h0 + h1) / 2)))

    slab_(-0.3, 0.3, -half, half, -th, 0.0, "oak")
    slab_(-0.3, 0.3, half - 0.03, half, 0.0, 0.045, "wood")
    # Boka: skinnpermen og to sideblokker med ark.
    slab_(-0.24, 0.24, -0.16, 0.14, 0.0, 0.012, "leather")
    slab_(-0.225, -0.008, -0.145, 0.125, 0.012, 0.034, "wax")
    slab_(0.008, 0.225, -0.145, 0.125, 0.012, 0.034, "wax")
    # Lyset på en jernarm ut fra pulten bak til høyre.
    y = yb - 0.02
    beam(m, (0.28, y, -0.12), (0.42, y, -0.12), 0.025, 0.025, "iron")
    cylinder(m, (0.44, -0.12), 0.05, y - 0.01, y + 0.01, "iron", n=8, caps=(True, True), smooth=False)
    candle(m, 0.44, -0.12, y + 0.01, h=0.12)
    return m


def kit_bonus():
    """Den varige styrken: et relikvieskrin, en liten kiste av eik formet
    som et hus med forgylt saltak, forgylte hjørner og føtter, et kors på
    mønet og glødende steiner foran, på en sokkel av hogd stein (origo på
    gulvet, front mot +z; eierens dom i runde 7: «ser ikke ut som et skrin»)."""
    m = MB()
    box(m, (-0.3, 0.0, -0.3), (0.3, 0.1, 0.3), "dressed", faces=("+y", "+x", "-x", "+z", "-z"))
    cylinder(m, (0.0, 0.0), 0.16, 0.1, 0.7, "dressed", n=8, caps=(False, False), smooth=False)
    box(m, (-0.3, 0.7, -0.22), (0.3, 0.78, 0.22), "slab", faces=("+y", "-y", "+x", "-x", "+z", "-z"))
    W, D, y0, y1 = 0.24, 0.13, 0.8, 1.0
    for sx in (-1, 1):
        for sz in (-1, 1):
            box(m, (sx * (W - 0.03) - 0.025, 0.78, sz * (D - 0.03) - 0.025), (sx * (W - 0.03) + 0.025, y0, sz * (D - 0.03) + 0.025), "gilt", faces=("+x", "-x", "+z", "-z"))
            box(m, (sx * W - 0.012, y0, sz * D - 0.012), (sx * W + 0.012, y1, sz * D + 0.012), "gilt", faces=("+x", "-x", "+z", "-z", "+y"))
    box(m, (-W, y0, -D), (W, y1, D), "oak", faces=("+x", "-x", "+z", "-z", "-y"))
    # Saltaket: forgylte takflater med mønet langs x, og gavler av eik.
    o, yr = 0.02, y1 + 0.2
    for sz in (-1, 1):
        m.quad((-W - o, y1, sz * (D + o)), (W + o, y1, sz * (D + o)), (W + o, yr, 0.0), (-W - o, yr, 0.0), "gilt", facing=(0, 1, sz))
        m.quad((-W - o, y1 - 0.012, sz * (D + o)), (W + o, y1 - 0.012, sz * (D + o)), (W + o, y1, sz * (D + o)), (-W - o, y1, sz * (D + o)), "gilt", facing=(0, 0, sz))
    for sx in (-1, 1):
        m.tri((sx * W, y1, -D), (sx * W, y1, D), (sx * W, yr - 0.01, 0.0), "oak", facing=(sx, 0, 0))
        x = sx * (W + o)
        m.quad((x, y1 - 0.012, -D - o), (x, y1 - 0.012, D + o), (x, y1, D + o), (x, y1, -D - o), "gilt", facing=(sx, 0, 0))
        cylinder(m, (sx * (W + 0.01), 0.0), 0.022, yr - 0.01, yr + 0.05, "gilt", n=6, caps=(True, False), smooth=False)
    beam(m, (0.0, yr - 0.01, 0.0), (0.0, yr + 0.12, 0.0), 0.022, 0.022, "gilt", up=(0.0, 0.0, 1.0))
    for x0, x1 in ((-0.05, -0.011), (0.011, 0.05)):
        beam(m, (x0, yr + 0.08, 0.0), (x1, yr + 0.08, 0.0), 0.022, 0.022, "gilt")
    # De glødende steinene foran.
    for x in (-0.13, 0.0, 0.13):
        box(m, (x - 0.017, 0.885, D), (x + 0.017, 0.919, D + 0.016), "magic_bonus", faces=("+z", "+y", "-y", "+x", "-x"))
    # Et forgylt bånd rundt midten av skrinet, steinene sitter i det.
    for z0, z1, f in ((D, D + 0.008, "+z"), (-D - 0.008, -D, "-z")):
        box(m, (-W, 0.87, z0), (W, 0.935, z1), "gilt", faces=(f, "+y", "-y"))
    return m


def crack_web(m, d, seed, scale=1.0, x0=0.0, half=0.45, top=1.9):
    """Sprekken som avslører en hemmelighet: én sort strek per revne, jevn
    og mørk over muren, som smalner av mot spissen og greiner seg opp fra
    gulvet, en løs stein som står ut og litt smuldret stein ved foten.
    Ingen lyse kanter (eieren i runde 9: to typer streker så ut som en hard
    strektegning; i runde 8 synes den ikke med lys tekstur). Flaten er
    z = `d` (fronten mot +z), origo nede på midten; revnene holder seg
    innenfor |x| ≤ `half` og under `top`."""
    import random

    rng = random.Random(seed)

    def crack(p, ang, n, width):
        x, y = p
        pts = [(x, y)]
        for _ in range(n):
            a = ang + rng.uniform(-0.4, 0.4)
            L = rng.uniform(0.12, 0.22) * scale
            x2, y2 = x + math.cos(a) * L, y + math.sin(a) * L
            if abs(x2) > half - 0.05 or not 0.02 < y2 < top:
                break
            pts.append((x2, y2))
            x, y = x2, y2
        # Bredden går jevnt fra `width` til en spiss, og hjørnene deles
        # mellom bitene, så streken er sammenhengende.
        k = len(pts) - 1
        if k < 1:
            return pts
        side = []
        for i, (px, py) in enumerate(pts):
            ax, ay = pts[max(i - 1, 0)]
            bx, by = pts[min(i + 1, k)]
            tx, ty = bx - ax, by - ay
            ln = math.hypot(tx, ty) or 1.0
            w_ = width * (1.0 - i / k) + 0.002
            side.append(((px - ty / ln * w_, py + tx / ln * w_), (px + ty / ln * w_, py - tx / ln * w_)))
        for i in range(k):
            (l0, r0), (l1, r1) = side[i], side[i + 1]
            m.quad((*r0, d + 0.004), (*r1, d + 0.004), (*l1, d + 0.004), (*l0, d + 0.004), "crack", facing=(0, 0, 1))
        return pts

    # Hovedrevnen, og greinene ut fra punkter på den (så ingen strek står løs).
    main = crack((x0, 0.05), math.pi / 2 + rng.uniform(-0.1, 0.1), 10, 0.016 * scale)
    n_ = len(main)
    for at, ang, n, wd in ((0.35, math.pi / 2 + 0.7, 5, 0.01), (0.55, 0.4, 4, 0.009), (0.8, math.pi - 0.4, 4, 0.008)):
        if n_ > 2:
            crack(main[min(int(at * (n_ - 1)), n_ - 2)], ang, n, wd * scale)
    # Den løse steinen, som står ut av muren.
    box(m, (x0 - 0.36, 0.48, d), (x0 - 0.1, 0.66, d + 0.05), "wall", faces=("+z", "+x", "-x", "+y", "-y"))
    # Litt smuldret stein ved foten.
    for k in range(4):
        sx, sz, sy = rng.uniform(0.04, 0.09), rng.uniform(0.03, 0.07), rng.uniform(0.02, 0.045)
        cx, cz = x0 + rng.uniform(-0.25, 0.25), d + rng.uniform(0.05, 0.25)
        box(m, (cx - sx / 2, 0.0, cz - sz / 2), (cx + sx / 2, sy, cz + sz / 2), "wall", faces=("+y", "+x", "-x", "+z", "-z"))


# Fugene i kryptmurens bilde (`crypt_wall`, 1024 px på 2 m, åtte skift; målt
# på det avledede bildet 13. oktober): de vannrette (piksel-y midt i fugen) og
# de loddrette i hvert skift mellom dem (mørke minimum midt i skiftet).
WALL_BEDS = [127.0, 254.0, 383.0, 511.0, 639.0, 767.0, 896.0, 1022.0]
WALL_HEADS = [[130, 266, 489, 684, 886, 1007], [55, 190, 408, 611, 746, 964], [143, 331, 514, 699, 871], [42, 246, 406, 598, 802, 961],
              [73, 141, 355, 460, 628, 696, 911], [191, 331, 533, 747, 886], [63, 246, 408, 618, 801, 963], [53, 246, 406, 609, 802, 962]]
WALL_FLIP = True  # glTF har v nedover: bildet står opp ned i meter (målt i runde 10)


def wall_courses(top):
    """Skiftene i muren nedenfra og opp til `top` m: (fra, til, fugene i x
    i meter), med bildet gjentatt hver 2. m. `WALL_FLIP`: bildet står opp ned
    på flaten (glTF har v nedover)."""
    f = STONE_TEX_M / STONE_TEX_PX
    out = []
    for tile in range(int(top / STONE_TEX_M) + 2):
        for k in range(len(WALL_BEDS)):
            if WALL_FLIP:
                a = STONE_TEX_M * (tile + 1) - WALL_BEDS[-1 - k] * f
                b = STONE_TEX_M * (tile + 1) - (WALL_BEDS[-2 - k] if k + 1 < len(WALL_BEDS) else WALL_BEDS[-1] - STONE_TEX_PX) * f
                heads = WALL_HEADS[-2 - k] if k + 1 < len(WALL_BEDS) else WALL_HEADS[-1]
            else:
                a = STONE_TEX_M * tile + WALL_BEDS[k] * f
                b = STONE_TEX_M * tile + (WALL_BEDS[k + 1] if k + 1 < len(WALL_BEDS) else WALL_BEDS[0] + STONE_TEX_PX) * f
                heads = WALL_HEADS[k]
            out.append((a, b, [h * f + j * STONE_TEX_M for j in (-2, -1, 0, 1) for h in heads]))
    return sorted([c for c in out if c[0] < top], key=lambda c: c[0])


def mortar_crack(m, d, seed, x0=0.0, half=0.9, top=2.2, width=0.015):
    """Sprekken i en mur av kvaderstein: den går i fugene, opp en loddrett
    fuge, langs den vannrette til nærmeste loddrette fuge i skiftet over, og
    opp igjen, i trapp (eieren i runde 10: «mursteiner sprekker ikke slik på
    tvers av flere»). Én mørk strek som smalner av mot toppen, og en løs
    stein ved foten. Flaten er z = `d` (fronten mot +z)."""
    import random

    rng = random.Random(seed)
    courses = [c for c in wall_courses(top) if c[1] > 0.0]
    pts = []
    x = x0
    for a, b, heads in courses:
        hs = [h for h in heads if -half + 0.1 < h < half - 0.1]
        if not hs:
            break
        near = sorted(hs, key=lambda h: abs(h - x))
        h = near[0] if rng.random() < 0.65 or len(near) < 2 else near[1]
        y0 = max(a, 0.02)
        if pts:
            pts.append((h, y0))
        else:
            pts.append((h, y0))
        pts.append((h, min(b, top)))
        x = h
    n = len(pts) - 1
    for i in range(n):
        (ax, ay), (bx, by) = pts[i], pts[i + 1]
        w0 = width * (1.0 - 0.6 * i / max(n, 1))
        w1 = width * (1.0 - 0.6 * (i + 1) / max(n, 1))
        dx, dy = bx - ax, by - ay
        ln = math.hypot(dx, dy) or 1.0
        nx, ny = -dy / ln, dx / ln
        # Litt forbi hjørnet, så trappetrinnene henger sammen.
        ex, ey = dx / ln * w1, dy / ln * w1
        m.quad((ax - nx * w0 - ex, ay - ny * w0 - ey, d + 0.004), (bx - nx * w1 + ex, by - ny * w1 + ey, d + 0.004), (bx + nx * w1 + ex, by + ny * w1 + ey, d + 0.004), (ax + nx * w0 - ex, ay + ny * w0 - ey, d + 0.004), "crack", facing=(0, 0, 1))
    for k in range(3):
        sx, sz, sy = rng.uniform(0.04, 0.08), rng.uniform(0.03, 0.06), rng.uniform(0.02, 0.04)
        cx, cz = (pts[0][0] if pts else x0) + rng.uniform(-0.2, 0.2), d + rng.uniform(0.05, 0.2)
        box(m, (cx - sx / 2, 0.0, cz - sz / 2), (cx + sx / 2, sy, cz + sz / 2), "wall", faces=("+y", "+x", "-x", "+z", "-z"))


def kit_crack(w=0.9, h=1.6, seed=5):
    """Sprekk-hintet: sprekken fra `crack_web` på muren, den samme som på
    den falske veggen (origo på muren nede på midten, front mot +z)."""
    m = MB()
    crack_web(m, 0.012, seed)
    return m


def kit_sarcophagus(L=2.0, W=0.9, H=0.75):
    """Sarkofagen (pynt, merket `prop`): en kiste av hogd stein på en sokkel,
    med blinde rundbuer på langsidene, en kant øverst og et valmet lokk med
    avfaset kant og et kors i relieff på hver takside (origo midt under på
    gulvet, lengden langs z; eierens dom i runde 7: «ligner overhodet ikke
    på en sarkofag»; runde 8: hjørneørene, de firkantede klossene, er
    borte)."""
    m = MB()
    box(m, (-W / 2 - 0.06, 0.0, -L / 2 - 0.06), (W / 2 + 0.06, 0.12, L / 2 + 0.06), "dressed", faces=("+y", "+x", "-x", "+z", "-z"))
    for sz in (-1, 1):
        surf = Plane((-W / 2 * sz, sz * L / 2), (sz * 1.0, 0.0), (0.0, sz * 1.0))
        wall(m, surf, W, 0.12, H, [], mat="wall")
    for sx in (-1, 1):
        surf = Plane((sx * W / 2, -sx * L / 2), (0.0, sx * 1.0), (sx * 1.0, 0.0))
        arches = [arch_profile(L * (k + 0.5) / 3, 0.42, 0.22, 0.2, n=8) for k in range(3)]
        wall(m, surf, L, 0.12, H - 0.04, [Polygon(a) for a in arches], mat="wall")
        for a in arches:
            recess(m, surf, a, 0.08, "dressed", "wall_dark")
    box(m, (-W / 2 - 0.03, H - 0.06, -L / 2 - 0.03), (W / 2 + 0.03, H, L / 2 + 0.03), "dressed", faces=("+x", "-x", "+z", "-z", "-y"))
    # Lokket: en helle med overheng og avfaset overkant, og et valmet tak
    # over (mønet langs z) med et kors i relieff på hver takside. Ingen
    # hjørneører (eieren i runde 8).
    o = 0.07
    ow, ol, ys = W / 2 + o, L / 2 + o, H + 0.05
    box(m, (-ow, H, -ol), (ow, ys, ol), "slab", faces=("+x", "-x", "+z", "-z", "-y"))
    yb, yr, hw, hl, rl = H + 0.09, H + 0.36, ow - 0.045, ol - 0.045, L / 2 - 0.25
    for sx in (-1, 1):
        m.poly([(sx * ow, ys, -ol), (sx * ow, ys, ol), (sx * hw, yb, hl), (sx * hw, yb, -hl)], [(-ol, 0), (ol, 0), (hl, 0.06), (-hl, 0.06)], "slab", facing=(sx, 1.0, 0))
    for sz in (-1, 1):
        m.poly([(-ow, ys, sz * ol), (ow, ys, sz * ol), (hw, yb, sz * hl), (-hw, yb, sz * hl)], [(-ow, 0), (ow, 0), (hw, 0.06), (-hw, 0.06)], "slab", facing=(0, 1.0, sz))
    for sx in (-1, 1):
        m.poly([(sx * hw, yb, -hl), (sx * hw, yb, hl), (0.0, yr, rl), (0.0, yr, -rl)], [(-hl, 0), (hl, 0), (rl, hw * 1.2), (-rl, hw * 1.2)], "slab", facing=(sx, 0.8, 0))
    for sz in (-1, 1):
        m.poly([(-hw, yb, sz * hl), (hw, yb, sz * hl), (0.0, yr, sz * rl)], [(-hw, 0), (hw, 0), (0, 0.4)], "slab", facing=(0, 0.6, sz))
    # Korset på hver takside: en stav langs mønet og en tverrstav ned i
    # taket, litt ut fra flaten.
    k = (yr - yb) / hw
    for sx in (-1, 1):
        lift = lambda t: (sx * t, yr - k * t + 0.035, 0.0)
        a, b = lift(0.12), lift(0.12)
        beam(m, (a[0], a[1], -0.6), (b[0], b[1], 0.55), 0.08, 0.035, "dressed", up=(sx * k, 1.0, 0.0))
        p0, p1 = lift(0.04), lift(0.3)
        beam(m, (p0[0], p0[1], -0.3), (p1[0], p1[1], -0.3), 0.08, 0.035, "dressed", up=(sx * k, 1.0, 0.0))
    return m


def kit_rest_flame():
    """Flammen på hvilestedet (eierens ark, `kit_rest`): et lite bål av
    tunger i skåla på toppen av stanga, 1,72 m oppe. Motoren viser den når
    hvilestedet er tent (`REST_FLAME`)."""
    m = MB()
    y = 1.72
    for k, (ox, oz, h) in enumerate(((0.0, 0.0, 0.26), (0.035, 0.02, 0.17), (-0.03, 0.025, 0.15), (0.0, -0.035, 0.18), (0.025, -0.02, 0.12))):
        f = 0.05 if k == 0 else 0.035
        tip = (ox * 1.4, y + h, oz * 1.4)
        base = [(ox + f * math.cos(a), y + 0.02, oz + f * math.sin(a)) for a in (0.3, 2.4, 4.5)]
        cc = (ox, y, oz)
        for i in range(3):
            m.tri(base[i], base[(i + 1) % 3], tip, "fire_torch", facing=sub(lerp(base[i], base[(i + 1) % 3], 0.5), cc))
            m.tri(base[i], base[(i + 1) % 3], cc, "fire_torch", facing=add(sub(lerp(base[i], base[(i + 1) % 3], 0.5), cc), (0, -0.05, 0)))
    return m


def kit_up(w=1.6, spring=1.6):
    """Døra opp: en lukket rundbuet dør (dørbildet) inntil muren, med en
    karm av vangestein og buestein rundt (`arch_trim`), en terskel og tre
    trinn foran (origo midt foran nederste trinn på gulvet; murflata i
    z = −1, døra 0,45 m opp, fronten mot +z). Ingen mur i modellen: muren
    er nivåets, så teksturen ikke krasjer (eierens dom i runde 7)."""
    m = MB()
    rise, run = 0.15, 0.3
    for k in range(3):
        box(m, (-w / 2 - 0.3, k * rise, -0.9 + k * run), (w / 2 + 0.3, (k + 1) * rise, -0.9 + 3 * run if k < 2 else 0.0), "dressed",
            faces=("+y", "+z", "+x", "-x"))
    # Trinnene går fra z = 0 inn mot døra; snu dem riktig vei.
    m.faces = [([(p[0], p[1], -0.9 - p[2]) for p in pts], uvs, mat, sm) for pts, uvs, mat, sm in m.faces]
    m.faces = [(pts[::-1], uvs[::-1], mat, sm) for pts, uvs, mat, sm in m.faces]
    y0 = 3 * rise
    box(m, (-w / 2 - 0.3, 0.0, -1.0), (w / 2 + 0.3, y0, -0.9), "dressed", faces=("+y", "+x", "-x"))
    leaf = shortcut_leaf(w=w - 0.04, spring=spring, t=0.1)
    m.extend(leaf, at=(-w / 2 + 0.02, y0, -0.95), keep_uv=True)
    arch_trim(m, Plane((0.0, -1.0), (1.0, 0.0), (0.0, 1.0)), w, spring, v0=y0, band=0.3, proud=0.12)
    box(m, (-w / 2 - 0.05, y0 - 0.01, -1.0), (w / 2 + 0.05, y0 + 0.02, -0.85), "slab", faces=("+y", "+z", "+x", "-x"))
    return m


def kit_down(w=2.4, depth=3.2, steps=10):
    """Trappa ned, lik i alle nivåer: en trapp som går ned under gulvet mot
    −z, med brystvern på tre sider, en rundbue over nedgangen og et kaldt
    blått lys som stiger opp fra dypet. Origo på gulvet midt i
    trappeåpningen (der du går inn), fronten mot +z."""
    m = MB()
    run = depth / steps
    rise = 2.4 / steps
    # Massive trinn ned til bunnen av sjakta (tynne plater med to sider lot
    # deg se rett gjennom mellom trinnene; eieren om nivå 1: «hull»).
    for k in range(steps):
        z0, z1 = -k * run - run, -k * run
        y = -(k + 1) * rise
        box(m, (-w / 2, -2.8, z0), (w / 2, y, z1), "dressed", faces=("+y", "+z", "-z"))
    # Terskelen foran øverste trinn fyller kanten av hullet i gulvet.
    box(m, (-w / 2, -2.8, 0.0), (w / 2, -0.015, 0.35), "dressed", faces=("+y", "-z"))
    # Sidene i trappesjakta og bakveggen.
    for sx in (-1, 1):
        surf = Plane((sx * w / 2, 0.0), (0.0, -1.0), (-sx, 0.0))
        grid_face(m, surf, sbox(0.0, -2.8, depth + 0.6, 0.0), "wall")
    back = Plane((-w / 2, -depth - 0.6), (1.0, 0.0), (0.0, 1.0))
    grid_face(m, back, sbox(0.0, -2.8, w, 0.0), "wall_dark")
    # Det blå lyset nede.
    m.quad((-w / 2, -2.75, -depth - 0.6), (w / 2, -2.75, -depth - 0.6), (w / 2, -2.75, -depth + 0.4), (-w / 2, -2.75, -depth + 0.4), "magic_down", facing=(0, 1, 0))
    # Brystvernet rundt hullet, åpent mot +z.
    t = 0.3
    # Det begynner bak portalmuren (T = 0,5 m, forsida 12 cm foran origo),
    # så innsida ikke ligger i samme plan som lysningen i buen.
    z_p = 0.12 - 0.5 + 0.03  # enden står 3 cm inne i muren, ikke i samme plan
    path = [(-w / 2 - t / 2, z_p), (-w / 2 - t / 2, -depth - 0.6 - t / 2), (w / 2 + t / 2, -depth - 0.6 - t / 2), (w / 2 + t / 2, z_p)]
    parapet(m, path, 0.0, h=0.95, t=t)
    # Portalen over nedgangen: en kort mur med rundbuet åpning så bred som
    # trappa, med vangestein og kilestein på begge sider (som døra opp og
    # snarveien; eieren var ikke imponert av søylene med buen oppå).
    T, side, spring = 0.5, 0.75, 2.2
    hw = w / 2 + side
    top = spring + w / 2 + 0.75
    hole = Polygon(arch_profile(0.0, w, -0.05, spring + 0.05, n=16))
    face = Polygon([(-hw, -0.05), (hw, -0.05), (hw, top), (-hw, top)]).difference(hole)
    # Muren står 12 cm foran brystvernet (og dekkhella), så endene ikke ligger i
    # samme plan som murflata.
    z0 = 0.12
    front = Plane((0.0, z0), (1.0, 0.0), (0.0, 1.0))
    back_s = Plane((0.0, z0 - T), (-1.0, 0.0), (0.0, -1.0))
    grid_face(m, front, face, "wall")
    grid_face(m, back_s, Polygon([(-u, v) for u, v in face.exterior.coords]).difference(Polygon([(-u, v) for u, v in hole.exterior.coords])), "wall")
    # Lysningen i åpningen (vangene og buens underside) mellom forsida og baksida.
    prof = arch_profile(0.0, w, -0.05, spring + 0.05, n=16)
    for (ua, va), (ub, vb) in zip(prof, prof[1:]):
        mid = ((ua + ub) / 2, (va + vb) / 2)
        n_in = (-mid[0], 0.0, 0.0) if va <= spring + 0.01 and vb <= spring + 0.01 else (-mid[0], spring - mid[1], 0.0)
        m.quad((ua, va, z0), (ub, vb, z0), (ub, vb, z0 - T), (ua, va, z0 - T), "dressed", facing=n_in)
    # Toppen og endene av muren.
    box(m, (-hw, -0.05, z0 - T), (hw, top, z0), "wall", faces=("+y", "+x", "-x"))
    box(m, (-hw - 0.06, top, z0 - T - 0.06), (hw + 0.06, top + 0.14, z0 + 0.06), "slab", faces=("+y", "+x", "-x", "+z", "-z", "-y"))
    arch_trim(m, front, w, spring, v0=0.0, band=0.3, proud=0.1, seed=3)
    # Smalere bak: vangene ville stått i samme plan som utsida av brystvernet.
    arch_trim(m, back_s, w, spring, v0=0.0, band=0.26, proud=0.1, seed=5)
    return m


# --- Eksport (Blender) ------------------------------------------------------------


def _bpy_material(bpy, key):
    name = material_name(key)
    m = bpy.data.materials.get(name)
    if m:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    if MATS[key] is None:
        rgb, rough = SPECIAL[key]
    else:
        _, rgb, rough = MATS[key]
    b.inputs["Base Color"].default_value = (*rgb, 1.0)
    b.inputs["Roughness"].default_value = rough
    if key == "iron":
        b.inputs["Metallic"].default_value = 0.0
    return m


def to_object(mbs, name="part"):
    """Lager et Blender-objekt av flatene (hjørnene slås sammen, flatene
    glattes etter vinkelen)."""
    import bmesh
    import bpy

    if hasattr(mbs, "faces"):  # én MB (også fra en annen modul, der klassen er en annen)
        mbs = [mbs]
    keys = []
    for m in mbs:
        for f in m.faces:
            if f[2] not in keys:
                keys.append(f[2])
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new("UVMap")
    index = {}

    def vert(p):
        k = (round(p[0], 4), round(p[1], 4), round(p[2], 4))
        v = index.get(k)
        if v is None:
            v = bm.verts.new((p[0], -p[2], p[1]))
            index[k] = v
        return v

    for m in mbs:
        for pts, uvs, mat, smooth in m.faces:
            vs = [vert(p) for p in pts]
            if len(set(vs)) < len(vs):
                # Hjørner som falt sammen: ta bort dem og hopp over det som blir for lite.
                seen, keep = set(), []
                for v, uv in zip(vs, uvs):
                    if v not in seen:
                        seen.add(v)
                        keep.append((v, uv))
                if len(keep) < 3:
                    continue
                vs, uvs = [k[0] for k in keep], [k[1] for k in keep]
            try:
                f = bm.faces.new(vs)
            except ValueError:
                # Flaten finnes fra før (to like flater): lag nye hjørner.
                f = bm.faces.new([bm.verts.new(v.co) for v in vs])
            f.material_index = keys.index(mat)
            f.smooth = smooth
            for loop, uv in zip(f.loops, uvs):
                loop[uvl].uv = uv
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    for k in keys:
        me.materials.append(_bpy_material(bpy, k))
    return ob


def sharpen(ob, angle=38.0):
    """Glatte flater får skarpe kanter der vinkelen er større enn `angle`."""
    import bpy

    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.shade_smooth_by_angle(angle=math.radians(angle), keep_sharp_edges=True)
    # Flatene som ikke skal glattes (murer, gulv) er allerede flate: merk
    # kantene mellom flate og glatte som skarpe ved å bruke flatenes flagg.


def bake_ao(ob, samples=96, distance=0.6):
    """Okklusjonen til hjørnefargen (som `town.finish`): Cycles AO, jevnet ut
    over naboene og løftet så bare fugene og hjørnene mørkner."""
    import bpy
    import numpy as np

    attr = ob.data.color_attributes.new("ao", "BYTE_COLOR", "POINT")
    ob.data.color_attributes.active_color = attr
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = samples
    scene.render.bake.target = "VERTEX_COLORS"
    world = scene.world or bpy.data.worlds.new("w")
    scene.world = world
    world.light_settings.distance = distance
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.bake(type="AO")
    col = np.empty(len(attr.data) * 4, dtype=np.float32)
    attr.data.foreach_get("color", col)
    col = col.reshape(-1, 4)
    edges = np.empty(len(ob.data.edges) * 2, dtype=np.int64)
    ob.data.edges.foreach_get("vertices", edges)
    edges = edges.reshape(-1, 2)
    ao = col[:, 0].copy()
    for _ in range(2):
        acc = ao.copy()
        cnt = np.ones_like(ao)
        np.add.at(acc, edges[:, 0], ao[edges[:, 1]])
        np.add.at(acc, edges[:, 1], ao[edges[:, 0]])
        np.add.at(cnt, edges[:, 0], 1)
        np.add.at(cnt, edges[:, 1], 1)
        ao = acc / cnt
    t = np.clip((ao - 0.25) / 0.6, 0.0, 1.0)
    v = 0.4 + 0.6 * t * t * (3.0 - 2.0 * t)
    col[:, 0] = col[:, 1] = col[:, 2] = v
    col[:, 3] = 1.0
    attr.data.foreach_set("color", col.ravel())


def export_glb(obs, path):
    """Eksporterer objektene (valgte) til glb og slanker fila."""
    import bpy

    sys.path.insert(0, os.path.join(ROOT, "tools", "models"))
    import slim

    bpy.ops.object.select_all(action="DESELECT")
    for o in obs:
        o.select_set(True)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    has_color = any(o.data.color_attributes for o in obs)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True, export_apply=True, export_materials="EXPORT",
                              export_yup=True, export_normals=True, export_texcoords=True,
                              export_vertex_color="ACTIVE" if has_color else "NONE", export_all_vertex_colors=False,
                              export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    slim.slim(path)
    # Kolliderne som følger formen, for modellene motoren setter ut etter
    # merkene og for de falske veggene (`solids.py`, `extras.solids`).
    import solids

    name = os.path.splitext(os.path.basename(path))[0]
    if name in solids.KIT or "_secret_" in name:
        solids.write_solids(path, solids.model_solids(path))
    tris = sum(len(p.vertices) - 2 for o in obs for p in o.data.polygons)
    print(f"{os.path.relpath(path, ROOT)}: {tris} trekanter, {os.path.getsize(path) // 1024} kB")
    return tris


def clear():
    import bpy

    bpy.ops.wm.read_factory_settings(use_empty=True)


# Teksturlag som er ett helt bilde av en ting (en dør), ikke en flis: de
# kan bare legges med uv i 0..1 over et blad, ellers gjentas dørbildet i
# meter (hengsler og nøkkelhull malt på stigetrinn).
WHOLE_IMAGES = {"door", "door_b", "bronze_door"}


def check_whole_images(mb, name):
    for pts, uvs, mat, _ in mb.faces:
        spec = MATS.get(mat)
        if spec and spec[0] in WHOLE_IMAGES and any(not (-0.01 <= c <= 1.01) for uv in uvs for c in uv):
            raise SystemExit(f"{name}: {mat} ({spec[0]}) er et helt bilde og må ha uv i 0..1, ikke {uvs[:2]} …")


def export_model(mb, path, ao=True, samples=64):
    check_whole_images(mb, os.path.basename(path))
    clear()
    ob = to_object(mb, os.path.basename(path).replace(".glb", ""))
    sharpen(ob)
    if ao:
        bake_ao(ob, samples=samples, distance=0.3)
    return export_glb([ob], path)


MECHANISMS = {}


def mechanisms():
    """Modellene motoren setter ut etter merkene (navnene i
    `crates/fristad-domain/src/domain/world/crypt/mod.rs`)."""
    rest, flame, _ = kit_rest()
    rest.faces += flame.faces
    base, handle = kit_lever()
    leaf, _bar = kit_shortcut()
    return {
        "kit_rest": rest,
        "kit_lever": base,
        "kit_lever_arm": handle,
        "kit_gate": kit_gate(),
        "kit_shortcut": kit_shortcut_frame(),
        "kit_shortcut_leaf": leaf,
        "kit_secret_wall": kit_false_wall(),
        "kit_trapdoor": kit_trapdoor(),
        "kit_ladder": kit_ladder(),
        "kit_ladder_top": kit_ladder_top(),
        "kit_plate": kit_plate(),
        "kit_arrow": kit_arrow(),
        "kit_lore": kit_lore(),
        "kit_bonus": kit_bonus(),
        "kit_crack": kit_crack(),
        "kit_up": kit_up(),
        "kit_down": kit_down(),
        "kit_sarcophagus": kit_sarcophagus(),
        "kit_rest_flame": kit_rest_flame(),
        "kit_bridge": kit_bridge(),
        "kit_chain": kit_chain(),
        "kit_water": kit_water(),
        **__import__("kit_mine").mine(),
        **__import__("kit_castle").castle(),
    }


def sheet():
    """Alle modulene i byggesettet ved siden av hverandre (til
    forhåndsvisning)."""
    m = MB()
    # Mur med døråpning, nisjer og pilhull.
    surf = Plane((0.0, 0.0), (1.0, 0.0), (0.0, 1.0))
    door = arch_profile(2.0, 2.0, 0.0, 2.0)
    tmp = MB()
    n1 = niche(tmp, surf, 5.0, 0.35, rng=None)
    n2 = niche(tmp, surf, 5.0, 1.15, sealed=True)
    wall(m, surf, 7.0, 0.0, 3.2, [Polygon(door), Polygon(n1), Polygon(n2)], mat="wall")
    m.faces += tmp.faces
    portal(m, surf, door)
    torch(m, (3.5, 2.0, 0.0), (0.0, 1.0))
    glitch_seam(m, surf, 6.4, 0.4, 2.8)
    # Gulv.
    grid_face(m, Horizontal(0.0), sbox(0, 0, 7, 6), "floor")
    # Søyle, sarkofag, alter, lysestake, kiste, rasmasse og brønn.
    pillar(m, 1.0, 4.5, 0.0, 3.2)
    sarcophagus(m, 3.0, 3.5, 0.0, 0.0)
    altar(m, 5.5, 4.0, 0.0, math.pi)
    candlestick(m, 4.2, 5.0, 0.0, arms=3)
    rubble(m, 6.3, 1.5, 0.0, r=0.7, h=0.4)
    # Krysshvelv med ribber over et fag ved siden av.
    groin_bay(m, 8.0, 13.0, 0.0, 5.0, 3.0, 1.6)
    groin_ribs(m, 8.0, 13.0, 0.0, 5.0, 3.0, 1.6)
    barrel(m, "z", 0.0, 4.0, 15.5, 2.6, 1.5, 1.5)
    barrel_rib(m, "z", 2.0, 15.5, 2.6, 1.5, 1.5)
    steps(m, (8.0, 7.5), (1.0, 0.0), (0.0, 1.0), 2.0, 0.0, 2.4, 0.0, 1.0, 4)
    well(m, 12.0, 8.0, 0.0, depth=1.0)
    parapet(m, [(14.0, 6.0), (16.0, 6.5), (17.0, 8.0)], 0.0)
    spiral_stair(m, (19.0, 3.0), 0.3, 1.5, 0.0, math.pi * 1.5, 0.0, 3.0, 14, newel=0.3, inner_wall=False)
    ossuary_shelves(m, Plane((20.5, 6.0), (1.0, 0.0), (0.0, 1.0)), 3.0, 0.0, 2.4, [0.8, 1.6])
    wall(m, Plane((20.5, 6.0), (1.0, 0.0), (0.0, 1.0)), 3.0, 0.0, 2.4, mat="bones")
    return m


def main():
    args = sys.argv[1:]
    if "--sheet" in args:
        out = args[args.index("--sheet") + 1]
        m = sheet()
        print(f"modulene: {m.tris()} trekanter")
        export_model(m, out, ao=True, samples=32)
        return
    only = [a for a in args if not a.startswith("-")]
    # kit_rest er bygd fra eierens ark med sheet2model (docs/wip/gjenstander.md);
    # den skrives bare over når den nevnes uttrykkelig.
    from_sheets = {"kit_rest"}
    for name, m in mechanisms().items():
        if only and name not in only:
            continue
        if not only and name in from_sheets:
            continue
        export_model(m, os.path.join(OUT, f"{name}.glb"), ao=True)


if __name__ == "__main__":
    main()
