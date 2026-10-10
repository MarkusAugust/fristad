"""Småbyggene i bakgårdene, i den malte stilen (kit.py). Kjør:

    python3 tools/models/yard.py [shed_a ...]

- `shed_a`: bordkledd skur med pulttak av spon og en dør.
- `shed_b`: vedskjul: åpent foran, stolper og pulttak, full av ved.
- `privy`: utedo med hjerte i døra.
- `haystack`: høystakk rundt en stang, med en kjerrestige ved siden av.
- `garden`: kjøkkenhage: flettverksgjerde rundt bed med kål og løk.
- `coop`: hønsehus på stolper med rampe.

Origo midt på grunnflaten, fronten mot lokal +Z i motoren (−Y i Blender)."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import box, stick  # noqa: E402
from town import collide, layer  # noqa: E402

OUT = "assets/models/town"


def lean_roof(p, x0, x1, y0, y1, z_front, z_back, mat, over=0.35, thick=0.14):
    """Pulttak fra fronten (y0, lav) til bakkanten (y1, høy)."""
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    pts = [(x0 - over, y0 - over, z_front - 0.1), (x1 + over, y0 - over, z_front - 0.1), (x1 + over, y1 + over, z_back + 0.1), (x0 - over, y1 + over, z_back + 0.1)]
    top = [bm.verts.new(v) for v in pts]
    bot = [bm.verts.new((v[0], v[1], v[2] - thick)) for v in pts]
    bm.faces.new(top)
    bm.faces.new(bot[::-1])
    for i in range(4):
        j = (i + 1) % 4
        bm.faces.new((bot[i], bot[j], top[j], top[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (c.x, -c.y)
    ob = kit._object("lean_roof", bm, mat)
    kit._bevel(ob, 0.04)


def shed_a(p, rng):
    w, d, h = 3.2, 2.6, 2.2
    x0, x1, y0, y1 = -w / 2, w / 2, -d / 2, d / 2
    box("plinth", (x0 - 0.1, y0 - 0.1, -0.2), (x1 + 0.1, y1 + 0.1, 0.15), p.stone, bevel=0.04)
    for a, b, out in (((x0, y0), (x1, y0), -1), ((x0, y1), (x1, y1), 1), ((x0, y0), (x0, y1), 1), ((x1, y0), (x1, y1), -1)):
        frame, L = kit.wall_frame(a, b)
        holes = [(L / 2 - 0.5, L / 2 + 0.5, 0.15, 2.0)] if out == -1 and a[1] == y0 and b[1] == y0 else []
        top = h if a[1] == y0 and b[1] == y0 else (h + 0.5 if a[1] == y1 and b[1] == y1 else h + 0.25)
        kit.slab_grid("shed_wall", frame, L, 0.15, top, 0.12, holes, p.boards)
    for x in (x0, x1):
        for y in (y0, y1):
            stick("shed_post", (x, y, 0.15), (x, y, h + (0.5 if y == y1 else 0.0)), 0.16, 0.16, p.timber, ("timber", rng.randrange(4)))
    kit.door(p, kit.wall_frame((x0, y0), (x1, y0))[0], w / 2, 0.15, 1.0, 1.85, -1, thick=0.12, open_in=False, dark=True)
    lean_roof(p, x0, x1, y0, y1, h + 0.05, h + 0.55, layer("shingles"))
    collide((0, 0, h / 2), (w + 0.2, d + 0.2, h))


def shed_b(p, rng):
    w, d, h = 3.6, 2.0, 2.1
    x0, x1, y0, y1 = -w / 2, w / 2, -d / 2, d / 2
    for x in (x0, 0.0, x1):
        for y in (y0, y1):
            stick("post", (x, y, 0.0), (x, y, h + (0.45 if y == y1 else 0.0)), 0.16, 0.16, p.timber, ("timber", rng.randrange(4)))
    frame, L = kit.wall_frame((x0, y1), (x1, y1))
    kit.slab_grid("back", frame, L, 0.0, h + 0.45, 0.1, [], p.boards)
    lean_roof(p, x0, x1, y0, y1, h + 0.05, h + 0.5, layer("shingles"))
    # Veden: kubber stablet i rader, endene ut mot fronten.
    logs = layer("logs")
    z = 0.12
    row = 0
    while z < h - 0.2:
        x = x0 + 0.15 + (0.11 if row % 2 else 0.0)
        while x < x1 - 0.15:
            stick("log", (x, y0 + 0.25, z), (x, y1 - 0.15, z), 0.2, 0.2, logs, bevel=0.06)
            x += 0.22
        z += 0.2
        row += 1
    collide((0, 0, h / 2), (w + 0.2, d + 0.2, h))


def privy(p, rng):
    w, d, h = 1.3, 1.3, 2.1
    x0, x1, y0, y1 = -w / 2, w / 2, -d / 2, d / 2
    for a, b in (((x0, y0), (x1, y0)), ((x0, y1), (x1, y1)), ((x0, y0), (x0, y1)), ((x1, y0), (x1, y1))):
        frame, L = kit.wall_frame(a, b)
        holes = [(L / 2 - 0.35, L / 2 + 0.35, 0.05, 1.85)] if a[1] == y0 and b[1] == y0 else []
        kit.slab_grid("privy_wall", frame, L, 0.0, h + (0.3 if a[1] == y1 and b[1] == y1 else 0.15 if a[0] == b[0] else 0.0), 0.08, holes, p.cladding)
    kit.door(p, kit.wall_frame((x0, y0), (x1, y0))[0], w / 2, 0.05, 0.7, 1.8, -1, thick=0.08, open_in=False, dark=False)
    lean_roof(p, x0, x1, y0, y1, h + 0.02, h + 0.32, layer("shingles"), over=0.2)
    collide((0, 0, h / 2), (w + 0.1, d + 0.1, h))


def haystack(p, rng):
    hay = layer("thatch", (0.95, 0.9, 0.75))
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    n, rings = 12, 6
    prof = [(1.3, 0.0), (1.45, 0.5), (1.4, 1.2), (1.15, 1.9), (0.75, 2.5), (0.3, 2.9), (0.0, 3.05)]
    rows = []
    for k, (r, z) in enumerate(prof):
        if r == 0.0:
            rows.append([bm.verts.new((0, 0, z))])
            continue
        rows.append([bm.verts.new((r * math.cos(2 * math.pi * i / n) * (1 + 0.05 * math.sin(i * 2.3 + k)), r * math.sin(2 * math.pi * i / n) * (1 + 0.05 * math.cos(i * 1.7 + k)), z)) for i in range(n)])
    for a, b in zip(rows, rows[1:]):
        for i in range(n):
            j = (i + 1) % n
            if len(b) == 1:
                bm.faces.new((a[i], a[j], b[0]))
            else:
                bm.faces.new((a[i], a[j], b[j], b[i]))
    bm.faces.new(rows[0][::-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (math.atan2(c.y, c.x) * 1.4, -c.z)
    kit._object("haystack", bm, hay)
    stick("pole", (0, 0, 2.6), (0, 0, 3.6), 0.1, 0.1, p.timber, ("timber", 0))
    # Stigen inntil.
    for sx in (-0.25, 0.25):
        stick("ladder", (1.6 + sx, -0.2, 0.0), (1.35 + sx, -0.2, 2.6), 0.07, 0.07, p.timber, ("timber", 1))
    for k in range(1, 8):
        z = k * 0.32
        stick("rung", (1.6 - 0.25 - z * 0.1, -0.2, z), (1.6 + 0.25 - z * 0.1, -0.2, z), 0.05, 0.05, p.timber, ("timber", 2))
    collide((0, 0, 1.3), (2.8, 2.8, 2.6))


def garden(p, rng):
    w, d = 4.0, 3.0
    wattle = layer("wattle")
    # Gjerdet: fire sider av flettverk mellom stolper, med en grind foran.
    for a, b in (((-w / 2, -d / 2), (-0.5, -d / 2)), ((0.5, -d / 2), (w / 2, -d / 2)), ((-w / 2, d / 2), (w / 2, d / 2)), ((-w / 2, -d / 2), (-w / 2, d / 2)), ((w / 2, -d / 2), (w / 2, d / 2))):
        frame, L = kit.wall_frame(a, b)
        kit.slab_grid("wattle", frame, L, 0.05, 0.85, 0.08, [], wattle)
        for t in range(int(L / 1.0) + 1):
            q = kit.at_wall(frame, min(t * 1.0, L), 0, 0)
            stick("stake", (q.x, q.y, 0), (q.x, q.y, 1.0), 0.07, 0.07, p.timber, ("timber", 0))
        collide(tuple(kit.at_wall(frame, L / 2, 0, 0.5)), (L, 0.15, 1.0), math.atan2(b[1] - a[1], b[0] - a[0]))
    # Bedene tråkker ingen i: en lav kollider over hele hagen.
    collide((0, 0, 0.3), (w - 0.2, d - 0.2, 0.6))
    # Bedene: jord med rader av kål og løk.
    earth = layer("earth")
    leaf = layer("sod", (0.65, 0.95, 0.55))
    onion = layer("sod", (0.8, 1.0, 0.6))
    for k, y in enumerate((-0.8, 0.0, 0.8)):
        box("bed", (-w / 2 + 0.3, y - 0.3, 0.0), (w / 2 - 0.3, y + 0.3, 0.12), earth, bevel=0.05)
        x = -w / 2 + 0.55
        while x < w / 2 - 0.4:
            if k == 1:
                stick("onion", (x, y, 0.1), (x, y, 0.4), 0.06, 0.06, onion, bevel=0)
                x += 0.25
            else:
                box("cabbage", (x - 0.16, y - 0.16, 0.1), (x + 0.16, y + 0.16, 0.36), leaf, bevel=0.12)
                x += 0.45


def coop(p, rng):
    w, d, h = 1.8, 1.4, 1.2
    x0, x1, y0, y1 = -w / 2, w / 2, -d / 2, d / 2
    for x in (x0, x1):
        for y in (y0, y1):
            stick("leg", (x, y, 0), (x, y, 0.6), 0.1, 0.1, p.timber, ("timber", rng.randrange(4)))
    box("floor", (x0, y0, 0.6), (x1, y1, 0.68), p.floor)
    for a, b in (((x0, y0), (x1, y0)), ((x0, y1), (x1, y1)), ((x0, y0), (x0, y1)), ((x1, y0), (x1, y1))):
        frame, L = kit.wall_frame(a, b)
        holes = [(L / 2 - 0.2, L / 2 + 0.2, 0.68, 1.05)] if a[1] == y0 and b[1] == y0 else []
        kit.slab_grid("coop_wall", frame, L, 0.68, 0.68 + h + (0.25 if a[1] == y1 and b[1] == y1 else 0.12 if a[0] == b[0] else 0.0), 0.06, holes, p.boards)
    lean_roof(p, x0, x1, y0, y1, 0.7 + h, 0.95 + h, layer("shingles"), over=0.2)
    # Rampa ned med lister.
    stick("ramp", (0, y0 - 1.0, 0.02), (0, y0, 0.66), 0.4, 0.04, p.floor, bevel=0)
    collide((0, 0, 1.0), (w + 0.1, d + 0.1, 2.0))


MODELS = {"shed_a": shed_a, "shed_b": shed_b, "privy": privy, "haystack": haystack, "garden": garden, "coop": coop}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        town.PIECES.clear()
        town.LIT = False
        p = kit.Paint(plaster="plaster", timber="timber_grey", roof="shingles")
        MODELS[name](p, random.Random(hash(name) % 1000))
        town.finish(f"{OUT}/{name}.glb", max_edge=100.0)
