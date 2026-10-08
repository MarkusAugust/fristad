"""Bygningene på torget, modellert i Blender (bpy) og eksportert til
assets/models/town/. Kjør:

    python3 tools/models/buildings.py [forge tavern apothecary scriptorium stave_church gravestone]

Smia (`forge`): et lavt hus av bruddstein med sotet mur og spontak, og
foran et åpent verksted under et skråtak på stolper: esse av stein med
glør og pipe opp gjennom taket, blåsebelg av lær, ambolt på kubbe,
kjølekar med vann, slipestein, verktøy på veggen, vedstabel og et
smijernsskilt på arm.

Vertshuset (`tavern`): et stort bindingsverkshus i to etasjer med
langsida mot torget, framskutt overetasje og halmtak, og ved døra et
skilt på jernarm, en lykt, benker og bord, og tønner.

Apoteket (`apothecary`): et smalt bindingsverkshus med gavlen mot torget
og skifertak, med en salgsluke der skodda er slått ned til et bord med
krukker og glass, urtebunter til tørk under takskjegget, plantekrukker
og et skilt med morter og støter.

Skriptoriet (`scriptorium`): et hus av hogd stein med støttepilarer,
høye rundbuede vinduer med blyglass, rundbuet dør, skifertak, en liten
klokkestol på mønet og et skilt med en åpen bok.

Stavkirka (`stave_church`), bygd i lag som Borgund: svalgang rundt med
stolper og lavt tak, sideskip med skråtak, høyt midtskip med bratt
spontak og drakehoder på gavlene, takrytter med spir, kor og apsis med
eget lite tårn, rundbuet portal med søyler, alt i tjærebredde planker.
Gravsteinen (`gravestone`): en hogd steinhelle med rundet topp og kors.

Samme hjelpere og teksturlag som husene (town.py): origo midt på
grunnflaten, −Y fram (døra mot lokal +Z i motoren)."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402,F401
from common import clear  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import town  # noqa: E402
from town import M, PIECES, beam, block, collide, cut, ramp, door, finish, gable, layer, light, plain, roof, window  # noqa: E402
from walls import revolve  # noqa: E402
from props import barrel, crate  # noqa: E402

OUT = "assets/models/town"


def mats(**kw):
    m = M(**kw)
    inside(m)
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.soot = layer("soot", (0.8, 0.8, 0.8), 0.5)
    m.leather = layer("leather", (0.7, 0.62, 0.55), 0.5)
    m.bark = layer("bark", (0.8, 0.75, 0.7), 0.5)
    m.dark_iron = layer("metal", (0.28, 0.26, 0.25), 0.5)
    m.glow = plain("fire_coals", (1.0, 0.45, 0.12), 0.4)
    m.water = plain("glass", (0.03, 0.04, 0.04), 0.05)
    return m


def moved(before, mat):
    """Flytt/drei alle deler laget siden `before` med matrisen `mat`."""
    for ob in PIECES[before:]:
        ob.matrix_world = mat @ ob.matrix_world


def log_cyl(m, name, a, b, r, mat=None):
    """En stokk (sylinder) fra a til b."""
    a, b = Vector(a), Vector(b)
    d = b - a
    ob = revolve(name, [(r, 0.0), (r, d.length)], mat or m.bark, n=10)
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(d.normalized())
    ob.location = a
    return ob


def stone_wall(m, name, x0, x1, y0, y1, z0, z1, mat):
    """En murt vegg (bruddstein) som en kloss, med steiner som står ut."""
    rng = random.Random(hash(name) & 0xFFFF)
    ob = block(name, (x1 - x0, y1 - y0, z1 - z0), mat, at=((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2), bevel=0.04, segments=2)
    along_x = (x1 - x0) > (y1 - y0)
    before = len(PIECES)
    for _ in range(int(((x1 - x0) + (y1 - y0)) * (z1 - z0) * 0.9)):
        if along_x:
            x = rng.uniform(x0 + 0.2, x1 - 0.2)
            for y in (y0 - 0.04, y1 + 0.04):
                if rng.random() < 0.5:
                    block("stone", (rng.uniform(0.3, 0.6), 0.1, rng.uniform(0.2, 0.35)), mat, at=(x, y, rng.uniform(z0 + 0.2, z1 - 0.2)), rot=(0, rng.uniform(-0.06, 0.06), 0), bevel=0.04, segments=2)
        else:
            y = rng.uniform(y0 + 0.2, y1 - 0.2)
            for x in (x0 - 0.04, x1 + 0.04):
                if rng.random() < 0.5:
                    block("stone", (0.1, rng.uniform(0.3, 0.6), rng.uniform(0.2, 0.35)), mat, at=(x, y, rng.uniform(z0 + 0.2, z1 - 0.2)), rot=(rng.uniform(-0.06, 0.06), 0, 0), bevel=0.04, segments=2)
    town.RELIEF[ob.name] = PIECES[before:]
    return ob


# --- Innredning -------------------------------------------------------------


def inside(m):
    """Materialene til innredningen."""
    m.floor = layer("floor", (0.75, 0.72, 0.68), 0.5)
    m.straw = layer("straw", (0.8, 0.78, 0.7), 0.5)
    m.flags = layer("concrete", (0.75, 0.73, 0.7), 0.5)
    m.flame = plain("fire_flame", (1.0, 0.7, 0.35), 0.4)
    m.wax = layer("plaster", (0.75, 0.7, 0.55), 0.5)
    m.books = [layer("leather", c, 0.5) for c in ((0.55, 0.2, 0.12), (0.3, 0.22, 0.14), (0.18, 0.22, 0.3), (0.2, 0.3, 0.16), (0.6, 0.45, 0.3))]
    m.vellum = layer("plaster", (0.9, 0.85, 0.72), 0.5)
    m.wood = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    if not hasattr(m, "leather"):
        m.leather = layer("leather", (0.7, 0.62, 0.55), 0.5)
    return m


def wall_box(x0, y0, x1, y1, z0, z1):
    """Kollider for et veggstykke mellom to hjørner (Blender-akser)."""
    collide(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2), (abs(x1 - x0), abs(y1 - y0), z1 - z0))


def walls_with_door(x0, x1, y0, y1, z0, z1, t, door_x, door_w, door_h, front_y):
    """Kollidere for fire vegger rundt et rom; fronten (y = front_y) har en
    døråpning med overligger."""
    yf0, yf1 = front_y - t / 2, front_y + t / 2
    wall_box(x0, yf0, door_x - door_w / 2, yf1, z0, z1)
    wall_box(door_x + door_w / 2, yf0, x1, yf1, z0, z1)
    wall_box(door_x - door_w / 2, yf0, door_x + door_w / 2, yf1, z0 + door_h, z1)
    back = y1 if abs(y1 - front_y) > abs(y0 - front_y) else y0
    wall_box(x0, back - t / 2, x1, back + t / 2, z0, z1)
    for x in (x0, x1):
        wall_box(x - t / 2, min(y0, y1), x + t / 2, max(y0, y1), z0, z1)


def candle(m, x, y, z, h=0.18, lit=True, r=0.025):
    revolve("candle", [(0.0, z), (r, z), (r * 0.9, z + h), (0.0, z + h)], m.wax, n=8)
    PIECES[-1].location = (x, y, 0)
    beam("wick", (x, y, z + h), (x, y, z + h + 0.02), 0.004, 0.004, m.dark_iron if hasattr(m, "dark_iron") else m.iron)
    if lit:
        revolve("flame", [(0.0, z + h + 0.01), (0.012, z + h + 0.03), (0.0, z + h + 0.07)], m.flame, n=6)
        PIECES[-1].location = (x, y, 0)


def holder(m, x, y, z):
    """Et lysestake av jern med et tent lys."""
    revolve("holder", [(0.0, z), (0.06, z), (0.05, z + 0.02), (0.012, z + 0.03), (0.012, z + 0.12), (0.04, z + 0.13), (0.0, z + 0.14)], m.iron, n=10)
    PIECES[-1].location = (x, y, 0)
    candle(m, x, y, z + 0.14)


def table(m, x, y, z0, along_x=True, ln=1.8, benches=True, rng=None):
    """Et langbord på bukker med benker, med kollider rundt det hele."""
    rng = rng or random.Random(int(x * 10 + y))
    w = 0.8
    sx, sy = (ln, w) if along_x else (w, ln)
    block("table_top", (sx, sy, 0.07), m.wood, at=(x, y, z0 + 0.76), bevel=0.01)
    for k in (-1, 1):
        off = (k * (ln / 2 - 0.25), 0) if along_x else (0, k * (ln / 2 - 0.25))
        for j in (-1, 1):
            a = Vector((x + off[0], y + off[1], z0 + 0.73))
            sp = Vector((0, j * 0.3, 0)) if along_x else Vector((j * 0.3, 0, 0))
            beam("trestle", a, a + sp + Vector((0, 0, -0.73)), 0.07, 0.07, m.timber)
    if benches:
        for j in (-1, 1):
            off = Vector((0, j * 0.65, 0)) if along_x else Vector((j * 0.65, 0, 0))
            c = Vector((x, y, z0)) + off
            block("bench", (sx, 0.3, 0.06) if along_x else (0.3, sy, 0.06), m.wood, at=c + Vector((0, 0, 0.45)), bevel=0.01)
            for k in (-1, 1):
                o = Vector((k * (ln / 2 - 0.15), 0, 0)) if along_x else Vector((0, k * (ln / 2 - 0.15), 0))
                block("bench_leg", (0.06, 0.26, 0.42) if along_x else (0.26, 0.06, 0.42), m.timber, at=c + o + Vector((0, 0, 0.21)), bevel=0.01)
    bw = sy + (1.6 if benches and along_x else 0.0)
    bl = sx + (1.6 if benches and not along_x else 0.0)
    collide((x, y, z0 + 0.42), (bl if not along_x else sx, bw if along_x else sy, 0.84))
    return z0 + 0.795


def books_row(m, x0, x1, y, z, depth=0.22, rng=None, axis_x=True):
    """En rad bøker på en hylle, ulike høyder og farger, noen lent."""
    rng = rng or random.Random(int(x0 * 7 + z * 13))
    x = x0
    while x < x1 - 0.05:
        t = rng.uniform(0.035, 0.08)
        h = rng.uniform(0.22, 0.34)
        lean = rng.uniform(-0.12, 0.12) if rng.random() < 0.15 else 0.0
        mat = rng.choice(m.books)
        c = (x + t / 2, y, z + h / 2) if axis_x else (y, x + t / 2, z + h / 2)
        size = (t, depth, h) if axis_x else (depth, t, h)
        block("book", size, mat, at=c, rot=(0, lean, 0) if axis_x else (lean, 0, 0), bevel=0.006)
        x += t + rng.uniform(0.0, 0.01)
        if rng.random() < 0.06:
            x += rng.uniform(0.08, 0.2)


def shelf(m, x, y, z0, w, h, along_x=True, rows=4, books=True, rng=None):
    """En bokhylle mot veggen: sider, hyller og bøker."""
    d = 0.32
    sx, sy = (w, d) if along_x else (d, w)
    for k in (-1, 1):
        if along_x:
            block("shelf_side", (0.05, d, h), m.wood, at=(x + k * w / 2, y, z0 + h / 2), bevel=0.006)
        else:
            block("shelf_side", (d, 0.05, h), m.wood, at=(x, y + k * w / 2, z0 + h / 2), bevel=0.006)
    for r in range(rows + 1):
        z = z0 + 0.05 + (h - 0.1) * r / rows
        block("shelf_board", (sx, sy, 0.03), m.wood, at=(x, y, z), bevel=0.004)
        if books and r < rows:
            if along_x:
                books_row(m, x - w / 2 + 0.04, x + w / 2 - 0.04, y, z + 0.015, rng=rng)
            else:
                books_row(m, y - w / 2 + 0.04, y + w / 2 - 0.04, x, z + 0.015, rng=rng, axis_x=False)
    collide((x, y, z0 + h / 2), (sx + 0.05, sy + 0.05, h))


def open_book(m, x, y, z, yaw=0.0):
    """En åpen bok: to sider pergament på et skinnbind."""
    for k in (-1, 1):
        block("page", (0.2, 0.28, 0.025), m.vellum, at=(x + k * 0.1 * math.cos(yaw), y + k * 0.1 * math.sin(yaw), z + 0.025), rot=(0, -k * 0.08, yaw), bevel=0.004)
    block("cover", (0.44, 0.3, 0.012), m.books[0], at=(x, y, z + 0.006), rot=(0, 0, yaw), bevel=0.004)


def horseshoe(m, x, y, z, yaw, mat, upright=False):
    """En hestesko: en bue av korte jernbiter, liggende eller hengende på
    veggen (åpningen ned)."""
    r = 0.065
    pts = []
    for k in range(9):
        a = math.radians(-40 + 260 * k / 8)
        pts.append((math.cos(a) * r, math.sin(a) * r))
    rot = Matrix.Rotation(yaw, 3, "Z")
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        if upright:
            a = Vector((x + ax, y, z + ay))
            b = Vector((x + bx, y, z + by))
        else:
            a = Vector((x, y, z)) + rot @ Vector((ax, ay, 0))
            b = Vector((x, y, z)) + rot @ Vector((bx, by, 0))
        beam("horseshoe", a, b, 0.022, 0.012, mat)


def helmet(m, x, y, z, mat):
    """En spangenhjelm: kuppel med kant og nesebeskytter."""
    revolve("helmet", [(0.12, 0.0), (0.125, 0.04), (0.118, 0.1), (0.095, 0.16), (0.055, 0.2), (0.0, 0.215)], mat, n=18, at=(x, y, z), cap=False)
    revolve("helmet_rim", [(0.122, 0.0), (0.132, 0.0), (0.132, 0.03), (0.122, 0.03)], m.dark_iron, n=18, at=(x, y, z), cap=False)
    beam("nasal", (x + 0.125, y, z - 0.06), (x + 0.125, y, z + 0.06), 0.03, 0.01, m.dark_iron)
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        beam("helmet_band", (x + math.cos(a) * 0.12, y + math.sin(a) * 0.12, z + 0.04), (x, y, z + 0.215), 0.02, 0.008, m.dark_iron)


def axe_head(m, x, y, z, mat):
    """Et skjeggøksehode stående på hylla med eggen mot rommet (+x)."""
    from town import prism
    ob = prism("axe_blade", [(-0.03, 0.03), (0.03, 0.03), (0.09, -0.0), (0.1, 0.17), (0.04, 0.11), (-0.03, 0.08)], 0.014, mat, at=(x, y, z))
    ob.rotation_euler = (0, 0, math.pi / 2)
    ob.location = (x, y, z)
    revolve("axe_eye", [(0.0, z + 0.06), (0.03, z + 0.06), (0.03, z + 0.12), (0.0, z + 0.12)], m.dark_iron, n=8, at=(x, y, 0))


def link_ring(m, x, y, z, yaw):
    """Et kjettingledd, liggende."""
    bpy.ops.mesh.primitive_torus_add(major_radius=0.03, minor_radius=0.007, major_segments=10, minor_segments=5, location=(x, y, z + 0.007))
    ob = bpy.context.active_object
    ob.scale = (1.4, 1.0, 1.0)
    ob.rotation_euler = (0, 0, yaw)
    ob.data.materials.append(m.dark_iron)
    town.ensure_uv(ob.data)
    town.uv_box(ob)
    PIECES.append(ob)


def stairs(m, x0, x1, y_end, z0, z1, n, depth, open_x=()):
    """En rett trapp langs +y mellom x0 og x1 som ender i y_end, fra z0 til
    z1: trinn, opptrinn, vanger, og rekkverk på de åpne sidene (`open_x`).
    Kolliderne er en rampe over trinnene og en vegg langs de åpne sidene
    fra tredje trinn, så man kan gå på fra siden nederst."""
    rise = (z1 - z0) / (n + 1)
    y_start = y_end - n * depth
    cx, w = (x0 + x1) / 2, x1 - x0
    for k in range(n):
        y = y_start + (k + 0.5) * depth
        hz = z0 + (k + 1) * rise
        block("stair", (w, depth, 0.06), m.wood, at=(cx, y, hz), bevel=0.008)
        block("riser", (w, 0.03, rise), m.wood, at=(cx, y - depth / 2 + 0.015, hz - rise / 2), bevel=0.004)
    for x in (x0, x1):
        beam("stringer", (x, y_start - 0.15, z0), (x, y_end, z1 - 0.1), 0.06, 0.25, m.timber)
    for x in open_x:
        beam("rail", (x, y_start - 0.15, z0 + 0.95), (x, y_end - 0.1, z1 + 0.95), 0.05, 0.06, m.timber)
        for k in range(0, n, 3):
            y = y_start + (k + 0.5) * depth
            beam("baluster", (x, y, z0 + (k + 1) * rise), (x, y, z0 + (k + 1) * rise + 0.95), 0.04, 0.04, m.timber)
        beam("newel", (x, y_start - 0.12, z0), (x, y_start - 0.12, z0 + 1.05), 0.09, 0.09, m.timber)
        beam("newel", (x, y_end - 0.1, z1 - 0.2), (x, y_end - 0.1, z1 + 1.0), 0.09, 0.09, m.timber)
        ya = y_start + 2.5 * depth
        collide((x, (ya + y_end) / 2, (z0 + z1) / 2 + 0.5), (0.08, y_end - ya, z1 - z0 + 1.0))
    ramp(x0, x1, y_start - 0.2, y_end, z0, z1 + 0.02)
    return y_start


def rail_run(m, a, b, fz, h=0.95):
    """Et rekkverk langs en rett linje fra a til b (x, y) på gulvet i fz:
    stolper i endene, håndlist og spiler, og en kollider."""
    a, b = Vector((a[0], a[1], fz)), Vector((b[0], b[1], fz))
    up = Vector((0, 0, h))
    beam("rail", a + up, b + up, 0.06, 0.07, m.timber)
    for p in (a, b):
        beam("rail_post", p, p + up + Vector((0, 0, 0.05)), 0.08, 0.08, m.timber)
    n = max(1, int((b - a).length / 0.35))
    for k in range(1, n):
        p = a + (b - a) * (k / n)
        beam("rail_spindle", p, p + up, 0.035, 0.035, m.timber)
    c = (a + b) / 2
    collide((c.x, c.y, fz + 0.5), (max(abs(b.x - a.x), 0.1), max(abs(b.y - a.y), 0.1), 1.0))


def stairwell(m, x0, x1, y0, y1, top, hole, n_joists, upper_y0=None):
    """Bjelkelaget mellom etasjene med trapphull: tak av bord under,
    bjelker langs y (delt ved hullet) med vekslere, og gulvet oppe med
    kollidere rundt hullet. Rommet er x0..x1, y0..y1 innvendig; gulvet oppe
    kan gå lenger fram (`upper_y0`, framskutt etasje)."""
    hx0, hx1, hy0, hy1 = hole
    ceil = block("ceiling", (x1 - x0, y1 - y0, 0.05), m.floor, at=((x0 + x1) / 2, (y0 + y1) / 2, top - 0.03), bevel=0)
    cut(ceil, (hx1 - hx0, hy1 - hy0, 0.4), ((hx0 + hx1) / 2, (hy0 + hy1) / 2, top))
    town.uv_box(ceil)
    for k in range(n_joists):
        x = x0 + 0.25 + (x1 - x0 - 0.5) * k / (n_joists - 1)
        if hx0 - 0.08 < x < hx1 + 0.08:
            if hy0 - 0.08 > y0:
                beam("joist", (x, y0, top - 0.15), (x, hy0 - 0.08, top - 0.15), 0.16, 0.2, m.timber)
            if hy1 + 0.08 < y1:
                beam("joist", (x, hy1 + 0.08, top - 0.15), (x, y1, top - 0.15), 0.16, 0.2, m.timber)
        else:
            beam("joist", (x, y0, top - 0.15), (x, y1, top - 0.15), 0.16, 0.2, m.timber)
    for y in (hy0 - 0.08, hy1 + 0.08):
        if y0 < y < y1:
            beam("trimmer", (max(x0, hx0 - 0.1), y, top - 0.15), (min(x1, hx1 + 0.1), y, top - 0.15), 0.16, 0.2, m.timber)
    uy0 = y0 if upper_y0 is None else upper_y0
    fl = block("upper_floor", (x1 - x0, y1 - uy0, 0.04), m.floor, at=((x0 + x1) / 2, (uy0 + y1) / 2, top), bevel=0)
    cut(fl, (hx1 - hx0, hy1 - hy0, 0.4), ((hx0 + hx1) / 2, (hy0 + hy1) / 2, top))
    town.uv_box(fl)
    # Kolliderne: gulvet oppe i fire biter rundt hullet.
    e = 0.15
    wall_box(x0 - e, uy0 - e, hx0, y1 + e, top - 0.2, top + 0.02)
    wall_box(hx1, uy0 - e, x1 + e, y1 + e, top - 0.2, top + 0.02)
    wall_box(hx0, uy0 - e, hx1, hy0, top - 0.2, top + 0.02)
    wall_box(hx0, hy1, hx1, y1 + e, top - 0.2, top + 0.02)


# --- Smia -------------------------------------------------------------------


def forge():
    m = mats(roof="tiles")
    rng = random.Random(21)
    W = 9.0
    yb0, yb1 = -0.4, 4.0  # steinhuset bak
    yf = -4.0  # forkant av verkstedet
    wh = 3.0
    # Steinhuset: fire murer, døra inn til lageret og et lite vindu.
    front = stone_wall(m, "front", -W / 2, W / 2, yb0, yb0 + 0.55, -0.24, wh, m.soot)
    cut(front, (1.0, 2.0, 2.0), (1.6, yb0 + 0.27, 1.0))
    cut(front, (0.6, 2.0, 0.6), (3.4, yb0 + 0.27, 1.8))
    town.uv_box(front)
    stone_wall(m, "back", -W / 2, W / 2, yb1 - 0.55, yb1, -0.24, wh, m.stone)
    for sx in (-1, 1):
        stone_wall(m, f"side{sx}", sx * W / 2 - 0.275 - 0.275 * sx, sx * W / 2 + 0.275 - 0.275 * sx, yb0 + 0.55, yb1 - 0.55, -0.24, wh, m.stone)
    door(m, 1.6, yb0 - 0.0, -1.0, 1.0, 2.0, 0.0, open=True)
    window(m, 3.4, 1.8, yb0 - 0.02, 0.6, 0.6, -1.0, shutters=True, rng=rng)
    # Taket over steinhuset: saltak med mønet langs x, flyttet bakover.
    before = len(PIECES)
    d = yb1 - yb0
    rise = roof(m, W + 0.1, d, wh, 42, gable_to_street=False)
    for sx in (-1, 1):
        b2 = len(PIECES)
        gable(m, d, wh, rise, 0.0, -1.0, rng)
        rot = Matrix.Rotation(math.radians(90 if sx > 0 else -90), 4, "Z")
        moved(b2, Matrix.Translation((sx * W / 2, 0, 0)) @ rot)
    moved(before, Matrix.Translation((0, (yb0 + yb1) / 2, 0)))
    # Verkstedet foran: stolper, rem, skråstag og skråtak med spon.
    eave_back, eave_front = wh - 0.1, 2.45
    posts = (-W / 2 + 0.2, -1.3, 1.3, W / 2 - 0.2)
    for x in posts:
        beam("post", (x, yf + 0.2, -0.05), (x, yf + 0.2, eave_front), 0.2, 0.2, m.timber, roll=math.pi / 2)
        block("post_stone", (0.4, 0.4, 0.2), m.stone, at=(x, yf + 0.2, 0.0), bevel=0.04, segments=2)
        beam("brace", (x, yf + 0.2, eave_front - 0.7), (x + (0.6 if x < 0 else -0.6), yf + 0.2, eave_front - 0.05), 0.12, 0.12, m.timber)
    beam("plate", (-W / 2 - 0.1, yf + 0.2, eave_front + 0.1), (W / 2 + 0.1, yf + 0.2, eave_front + 0.1), 0.22, 0.2, m.timber)
    run = yb0 - yf + 0.6
    slope_len = math.hypot(run, eave_back - eave_front)
    tilt = math.atan2(eave_back - eave_front, run)
    for k in range(7):
        x = -W / 2 + W * k / 6
        beam("rafter", (x, yf - 0.4, eave_front + 0.05), (x, yb0 + 0.1, eave_back + 0.05), 0.1, 0.14, m.timber)
    lean = block("lean_roof", (W + 0.6, slope_len, 0.14), m.roof, bevel=0.01)
    town._roof_uv(lean, along_y=False)
    lean.rotation_euler = (tilt, 0, 0)
    lean.location = (0, (yf - 0.4 + yb0 + 0.1) / 2, (eave_front + eave_back) / 2 + 0.2)
    # Essa: murt av stein, med glør i fordypningen og en hette som går
    # over i pipa opp gjennom taket.
    hx, hy = -2.4, yb0 - 0.75
    block("hearth", (1.8, 1.4, 0.85), m.soot, at=(hx, hy, 0.425 - 0.12), bevel=0.05, segments=2)
    block("hearth_rim", (1.9, 1.5, 0.1), m.stone, at=(hx, hy, 0.78), bevel=0.03, segments=2)
    block("coals", (0.9, 0.7, 0.06), m.glow, at=(hx, hy - 0.05, 0.82), bevel=0.02)
    rng2 = random.Random(5)
    for _ in range(14):
        block("coal", (rng2.uniform(0.06, 0.12), rng2.uniform(0.06, 0.12), 0.05), m.dark_iron, at=(hx + rng2.uniform(-0.4, 0.4), hy - 0.05 + rng2.uniform(-0.3, 0.3), 0.86), rot=(0, 0, rng2.uniform(0, 3)), bevel=0.015)
    # Hetta: smalner fra 1,6 til 0,8 m mellom 1,9 og 2,9 m.
    for k in range(5):
        t = k / 4
        w = 1.7 - 0.8 * t
        block("hood", (w, w * 0.85, 0.24), m.soot, at=(hx, hy + 0.1 * t, 1.9 + t * 1.0), bevel=0.03, segments=2)
    town.chimney(m, hx, hy + 0.1, 2.9, 6.6)
    # Blåsebelgen: to bord med lær mellom, og en stang til å trå.
    bx, by = hx - 1.25, hy + 0.2
    block("bellows_top", (0.55, 1.0, 0.05), m.planks, at=(bx, by, 0.9), rot=(0.12, 0, 0), bevel=0.01)
    block("bellows_bottom", (0.55, 1.0, 0.05), m.planks, at=(bx, by, 0.55), bevel=0.01)
    block("bellows_leather", (0.5, 0.85, 0.3), m.leather, at=(bx, by + 0.05, 0.72), bevel=0.08, segments=3)
    beam("bellows_nozzle", (bx, by - 0.55, 0.7), (hx - 0.7, hy - 0.2, 0.7), 0.06, 0.06, m.dark_iron)
    beam("bellows_legs", (bx, by, 0.0), (bx, by, 0.55), 0.12, 0.5, m.timber)
    # Ambolten på en kubbe.
    ax, ay = 0.2, yf + 1.7
    log_cyl(m, "stump", (ax, ay, -0.1), (ax, ay, 0.5), 0.32, m.bark)
    block("stump_top", (0.62, 0.62, 0.02), m.planks, at=(ax, ay, 0.5), bevel=0.0)
    block("anvil_foot", (0.34, 0.24, 0.12), m.dark_iron, at=(ax, ay, 0.57), bevel=0.02)
    block("anvil_waist", (0.18, 0.14, 0.16), m.dark_iron, at=(ax, ay, 0.71), bevel=0.02)
    block("anvil_face", (0.55, 0.16, 0.1), m.dark_iron, at=(ax, ay, 0.84), bevel=0.015)
    horn = revolve("anvil_horn", [(0.06, 0.0), (0.0, 0.28)], m.dark_iron, n=12)
    horn.rotation_euler = (0, math.pi / 2, 0)
    horn.location = (ax + 0.27, ay, 0.84)
    # Hammer og tang på ambolten.
    beam("hammer_handle", (ax - 0.1, ay - 0.05, 0.9), (ax - 0.45, ay - 0.25, 0.9), 0.03, 0.03, m.timber)
    block("hammer_head", (0.06, 0.14, 0.06), m.dark_iron, at=(ax - 0.1, ay - 0.05, 0.9), bevel=0.01)
    for s in (-1, 1):
        beam("tongs", (ax + 0.05, ay + 0.03, 0.9), (ax + 0.5, ay + 0.03 + s * 0.04, 0.9), 0.02, 0.015, m.dark_iron)
    # Kjølekaret: et trau av bord med vann.
    qx, qy = 1.9, yf + 1.4
    block("trough_bottom", (1.2, 0.55, 0.05), m.planks, at=(qx, qy, 0.12), bevel=0.005)
    for sy in (-1, 1):
        block("trough_side", (1.2, 0.05, 0.5), m.planks, at=(qx, qy + sy * 0.27, 0.35), bevel=0.005)
    for sx in (-1, 1):
        block("trough_end", (0.05, 0.6, 0.55), m.planks, at=(qx + sx * 0.6, qy, 0.37), bevel=0.005)
    block("water", (1.12, 0.48, 0.02), m.water, at=(qx, qy, 0.52), bevel=0)
    for sx in (-1, 1):
        block("trough_leg", (0.1, 0.6, 0.12), m.timber, at=(qx + sx * 0.45, qy, 0.06), bevel=0.01)
    # Slipesteinen i en ramme.
    gx, gy = 3.4, yf + 1.2
    wheel = revolve("grindstone", [(0.0, -0.06), (0.32, -0.06), (0.32, 0.06), (0.0, 0.06)], m.stone, n=24)
    wheel.rotation_euler = (math.pi / 2, 0, 0)
    wheel.location = (gx, gy, 0.6)
    for sy in (-1, 1):
        beam("grind_frame", (gx - 0.4, gy + sy * 0.14, 0.0), (gx, gy + sy * 0.14, 0.62), 0.07, 0.07, m.timber)
        beam("grind_frame", (gx + 0.4, gy + sy * 0.14, 0.0), (gx, gy + sy * 0.14, 0.62), 0.07, 0.07, m.timber)
    beam("grind_axle", (gx, gy - 0.25, 0.6), (gx, gy + 0.3, 0.6), 0.03, 0.03, m.dark_iron)
    beam("grind_crank", (gx, gy + 0.3, 0.6), (gx, gy + 0.3, 0.38), 0.025, 0.025, m.dark_iron)
    # Verktøy på murveggen: en list med tenger og hammere.
    beam("tool_rail", (-0.4, yb0 - 0.04, 1.75), (1.0, yb0 - 0.04, 1.75), 0.08, 0.08, m.timber)
    for k in range(6):
        x = -0.3 + k * 0.24
        if k % 2:
            beam("tool", (x, yb0 - 0.1, 1.72), (x, yb0 - 0.1, 1.2), 0.025, 0.025, m.timber)
            block("tool_head", (0.12, 0.05, 0.06), m.dark_iron, at=(x, yb0 - 0.1, 1.2), bevel=0.01)
        else:
            for s in (-1, 1):
                beam("tongs", (x, yb0 - 0.1, 1.72), (x + s * 0.03, yb0 - 0.1, 1.15), 0.015, 0.015, m.dark_iron)
    # Jernstenger og et ferdig sverd lent mot muren.
    for k in range(4):
        beam("iron_bar", (-0.6 + k * 0.06, yb0 - 0.12, 0.0), (-0.5 + k * 0.06, yb0 - 0.05, 1.3), 0.03, 0.03, m.dark_iron)
    # Kulltønne og kasser i hjørnet.
    barrel(m, at=(-4.0, yf + 1.0, 0.0), h=0.8, r=0.28, staves=14, rng=rng)
    crate(m, at=(-3.6, yb0 - 0.6, 0.0), s=0.5, rng=rng)
    # Vedstabelen langs gavlen ute, under takskjegget.
    for row in range(5):
        for k in range(9):
            y = yb0 + 0.5 + k * 0.38
            z = 0.14 + row * 0.24
            log_cyl(m, "firewood", (W / 2 + 0.25, y, z), (W / 2 + 0.95, y + rng.uniform(-0.03, 0.03), z), 0.1 + rng.uniform(-0.015, 0.015))
    # Smijernsskiltet på en arm ut fra stolpen: en ambolt i silhuett.
    sx = W / 2 - 0.2
    beam("sign_arm", (sx, yf + 0.2, 2.2), (sx + 1.1, yf + 0.2, 2.2), 0.05, 0.05, m.dark_iron)
    beam("sign_strut", (sx, yf + 0.2, 1.8), (sx + 0.7, yf + 0.2, 2.18), 0.03, 0.03, m.dark_iron)
    for k in range(2):
        beam("sign_chain", (sx + 0.35 + k * 0.6, yf + 0.2, 2.18), (sx + 0.35 + k * 0.6, yf + 0.2, 1.95), 0.015, 0.015, m.dark_iron)
    block("sign_anvil", (0.75, 0.04, 0.18), m.dark_iron, at=(sx + 0.65, yf + 0.2, 1.88), bevel=0.01)
    block("sign_anvil_waist", (0.25, 0.04, 0.16), m.dark_iron, at=(sx + 0.62, yf + 0.2, 1.72), bevel=0.01)
    block("sign_anvil_foot", (0.5, 0.04, 0.08), m.dark_iron, at=(sx + 0.62, yf + 0.2, 1.6), bevel=0.01)
    # --- Inne i steinhuset: lageret og smedens arbeidsbenk.
    iy0, iy1 = yb0 + 0.55, yb1 - 0.55
    block("floor", (W - 1.1, iy1 - iy0, 0.04), m.flags, at=(0, (iy0 + iy1) / 2, 0.0), bevel=0)
    # Arbeidsbenken langs bakveggen, med skrustikke, tenger og et sverd under arbeid.
    bx = -1.4
    block("bench_top", (2.2, 0.7, 0.08), m.wood, at=(bx, iy1 - 0.4, 0.86), bevel=0.01)
    for sx_ in (-1, 1):
        for sy_ in (-1, 1):
            block("bench_leg", (0.09, 0.09, 0.82), m.timber, at=(bx + sx_ * 0.98, iy1 - 0.4 + sy_ * 0.27, 0.41), bevel=0.01)
    block("bench_shelf", (2.0, 0.6, 0.04), m.wood, at=(bx, iy1 - 0.4, 0.25), bevel=0.005)
    block("vise", (0.16, 0.22, 0.2), m.dark_iron, at=(bx - 0.8, iy1 - 0.65, 0.98), bevel=0.01)
    beam("blade", (bx - 0.2, iy1 - 0.45, 0.91), (bx + 0.6, iy1 - 0.4, 0.91), 0.05, 0.006, m.iron)
    for k in range(3):
        beam("tool", (bx + 0.2 + k * 0.15, iy1 - 0.25, 0.91), (bx + 0.25 + k * 0.15, iy1 - 0.6, 0.91), 0.02, 0.02, m.dark_iron)
    collide((bx, iy1 - 0.4, 0.45), (2.3, 0.8, 0.9))
    # Hyller på gavlveggen med smedens varer: jernstenger, hestesko, hjelmer,
    # skjoldbuler, øksehoder, spydspisser, kjetting og en spikerbøtte.
    sx_, sy_ = -W / 2 + 0.75, (iy0 + iy1) / 2
    shelf(m, sx_, sy_, 0.0, 2.4, 1.9, along_x=False, rows=3, books=False, rng=rng)
    shelf_z = [0.05 + 1.8 * r / 3 + 0.015 for r in range(4)]
    m.bright_iron = layer("metal", (1.0, 0.97, 0.92), 0.5)
    fx = sx_ + 0.02  # midt på hylla
    # Nederst: en bunt jernstenger, en spikerbøtte og en haug hestesko.
    for lay in range(2):
        for i in range(3 - lay):
            bx_ = fx - 0.07 + i * 0.07 + lay * 0.035
            bz = shelf_z[0] + 0.04 + lay * 0.05
            beam("iron_bar", (bx_, sy_ - 1.1, bz), (bx_, sy_ - 0.35, bz), 0.05, 0.05, m.dark_iron)
    revolve("nail_bucket", [(0.0, 0.0), (0.12, 0.0), (0.14, 0.24), (0.0, 0.24)], m.planks, n=14, at=(fx, sy_ + 0.05, shelf_z[0]))
    revolve("nails", [(0.0, 0.2), (0.12, 0.2), (0.08, 0.27), (0.0, 0.29)], m.dark_iron, n=10, at=(fx, sy_ + 0.05, shelf_z[0]))
    for k in range(5):
        horseshoe(m, fx + rng.uniform(-0.05, 0.05), sy_ + 0.55 + rng.uniform(-0.12, 0.12), shelf_z[0] + 0.02 + k * 0.025, rng.uniform(0, 6.28), m.bright_iron)
    # Midten: hjelmer og skjoldbuler.
    for k, y in enumerate((-0.85, -0.45, 0.05)):
        helmet(m, fx, sy_ + y, shelf_z[1], m.bright_iron if k != 1 else m.dark_iron)
    for k, y in enumerate((0.45, 0.85)):
        revolve("boss", [(0.0, 0.0), (0.12, 0.0), (0.12, 0.015), (0.08, 0.03), (0.075, 0.08), (0.0, 0.11)], m.bright_iron, n=16, at=(fx, sy_ + y, shelf_z[1]))
    # Tredje hylle: øksehoder og spydspisser reist opp mot rommet, og
    # ferdige klinger lent mot veggen.
    for k in range(3):
        axe_head(m, fx, sy_ - 0.95 + k * 0.26, shelf_z[2], m.bright_iron)
    for k in range(3):
        y = sy_ - 0.12 + k * 0.13
        revolve("spear_socket", [(0.0, 0.0), (0.022, 0.0), (0.016, 0.09), (0.0, 0.09)], m.dark_iron, n=8, at=(fx, y, shelf_z[2]))
        tip = revolve("spear_tip", [(0.0, 0.0), (0.03, 0.06), (0.0, 0.26)], m.bright_iron, n=4, at=(0, 0, 0))
        tip.scale = (0.3, 1.0, 1.0)
        tip.location = (fx, y, shelf_z[2] + 0.09)
    for k in range(3):
        y = sy_ + 0.4 + k * 0.22
        block("blade", (0.008, 0.05, 0.5), m.bright_iron, at=(fx - 0.04, y, shelf_z[2] + 0.27), rot=(0, -0.12, 0), bevel=0.002)
        block("crossguard", (0.03, 0.18, 0.025), m.dark_iron, at=(fx - 0.045 + 0.03, y, shelf_z[2] + 0.53), rot=(0, -0.12, 0), bevel=0.004)
    # På toppen: en kveil kjetting og en lykt.
    for k in range(10):
        a = k * 2 * math.pi / 10
        link_ring(m, fx + math.cos(a) * 0.1, sy_ - 0.6 + math.sin(a) * 0.1, shelf_z[3] + 0.02, a + math.pi / 2)
    revolve("pail", [(0.0, 0.0), (0.11, 0.0), (0.13, 0.22), (0.0, 0.22)], m.planks, n=14, at=(fx, sy_ + 0.6, shelf_z[3]))
    # Hestesko på spiker over arbeidsbenken, og en hjelm på en knagg.
    for k in range(6):
        hz = 1.65 + (k % 2) * 0.18
        horseshoe(m, -2.3 + k * 0.32, iy1 - 0.04, hz, 0.0, m.bright_iron, upright=True)
    light((sx_ + 1.0, sy_, 1.3), (1.0, 0.7, 0.4), 2.2, 4.0)
    # Kasser, en kulltønne og sekker ved døra.
    crate(m, at=(2.9, iy1 - 0.45, 0.0), s=0.6, rng=rng)
    crate(m, at=(2.85, iy1 - 0.45, 0.62), s=0.5, rng=rng)
    collide((2.9, iy1 - 0.45, 0.55), (0.7, 0.7, 1.1))
    barrel(m, at=(3.4, iy0 + 1.3, 0.0), h=0.8, r=0.28, staves=14, rng=rng)
    collide((3.4, iy0 + 1.3, 0.4), (0.6, 0.6, 0.8))
    for k, (x, y) in enumerate([(-2.6, iy0 + 0.4), (-2.1, iy0 + 0.35)]):
        revolve("sack", [(0.0, 0.0), (0.2, 0.02), (0.24, 0.2), (0.18, 0.45), (0.06, 0.55), (0.0, 0.56)], m.leather, n=12, at=(x, y, 0))
    collide((-2.35, iy0 + 0.38, 0.3), (1.0, 0.5, 0.6))
    # En lykt under takbjelken.
    beam("tie_beam", (-W / 2 + 0.5, (iy0 + iy1) / 2, 2.85), (W / 2 - 0.5, (iy0 + iy1) / 2, 2.85), 0.2, 0.2, m.timber)
    beam("lamp_chain", (0.0, (iy0 + iy1) / 2, 2.75), (0.0, (iy0 + iy1) / 2, 2.3), 0.015, 0.015, m.dark_iron)
    block("lamp_cage", (0.2, 0.2, 0.28), m.dark_iron, at=(0.0, (iy0 + iy1) / 2, 2.15), bevel=0.01)
    candle(m, 0.0, (iy0 + iy1) / 2, 2.03, h=0.1)
    light((0.0, (iy0 + iy1) / 2, 2.15), (1.0, 0.65, 0.35), 1.4, 6.0)
    # Kolliderne: steinhuset med døråpning, og verkstedet foran.
    walls_with_door(-W / 2 + 0.275, W / 2 - 0.275, yb0 + 0.275, yb1 - 0.275, -0.24, wh, 0.55, 1.6, 1.0, 2.24, yb0 + 0.275)
    collide((hx, hy, 1.4), (1.9, 1.5, 2.8))
    collide((bx_ := hx - 1.25, hy + 0.2, 0.5), (0.6, 1.1, 1.0))
    collide((0.2, yf + 1.7, 0.45), (0.7, 0.7, 0.9))
    collide((1.9, yf + 1.4, 0.3), (1.3, 0.65, 0.6))
    collide((3.4, yf + 1.2, 0.47), (0.9, 0.5, 0.95))
    collide((-4.0, yf + 1.0, 0.4), (0.6, 0.6, 0.8))
    collide((-3.6, yb0 - 0.6, 0.25), (0.55, 0.55, 0.5))
    for x in posts:
        collide((x, yf + 0.2, 1.3), (0.3, 0.3, 2.6))
    collide((W / 2 + 0.6, (yb0 + yb1) / 2 + 0.1, 0.65), (0.8, 3.6, 1.3))
    light((hx, hy - 0.05, 1.3), (1.0, 0.45, 0.15), 3.0, 7.0)
    return m


# --- Vertshuset -------------------------------------------------------------


def tavern():
    spec = dict(seed=31, w=12.0, d=9.0, floors=2, jetty=0.45, pitch=50, gable=False, roof="thatch",
                plaster=(1.0, 0.93, 0.8), door_x=-1.2, windows0=[-4.4, -2.9, 1.6, 3.6], windows_up=5, plinth=0.6, open_door=True, lantern=False)
    m = town.half_timbered(spec)
    inside(m)
    m.soot = layer("soot", (0.8, 0.8, 0.8), 0.5)
    m.bark = layer("bark", (0.8, 0.75, 0.7), 0.5)
    m.leather = layer("leather", (0.7, 0.62, 0.55), 0.5)
    m.ember = plain("fire_ember", (1.0, 0.45, 0.12), 0.4)
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.dark_iron = layer("metal", (0.28, 0.26, 0.25), 0.5)
    m.glow = plain("glow", (1.0, 0.75, 0.4), 0.4)
    rng = random.Random(32)
    fy = -spec["d"] / 2
    # Skiltet: et bord på en jernarm over døra, med et krus skåret ut.
    ax = -1.2 + 1.2
    beam("sign_arm", (ax, fy - 0.05, 3.25), (ax, fy - 1.3, 3.25), 0.06, 0.06, m.dark_iron)
    beam("sign_strut", (ax, fy - 0.05, 2.75), (ax, fy - 0.85, 3.22), 0.04, 0.04, m.dark_iron)
    for k in (-1, 1):
        beam("sign_ring", (ax, fy - 0.75 + k * 0.3, 3.22), (ax, fy - 0.75 + k * 0.3, 3.05), 0.02, 0.02, m.dark_iron)
    block("sign_board", (0.05, 0.85, 0.6), m.planks, at=(ax, fy - 0.75, 2.75), bevel=0.01)
    mug = revolve("sign_mug", [(0.0, -0.18), (0.13, -0.18), (0.15, 0.16), (0.0, 0.16)], m.dark_iron, n=12)
    mug.rotation_euler = (0, math.pi / 2, 0)
    mug.scale = (1, 1, 0.25)
    mug.location = (ax, fy - 0.75, 2.75)
    # Lykta ved døra, på en jernarm.
    lx = -1.2 - 0.9
    beam("lamp_arm", (lx, fy - 0.05, 2.5), (lx, fy - 0.45, 2.5), 0.03, 0.03, m.dark_iron)
    block("lamp_cage", (0.2, 0.2, 0.28), m.dark_iron, at=(lx, fy - 0.45, 2.3), bevel=0.01)
    block("lamp_glass", (0.16, 0.16, 0.22), plain("glow_horn", (0.3, 0.19, 0.08), 0.5), at=(lx, fy - 0.45, 2.3), bevel=0)
    block("lamp_flame", (0.04, 0.04, 0.08), m.glow, at=(lx, fy - 0.45, 2.3), bevel=0)
    town.lamp((lx, fy - 0.45, 2.3), intensity=2.0, radius=9.0)
    # Benker og bord foran vinduene.
    for cx in (-3.6, 2.6):
        y = fy - 1.1
        block("table_top", (1.8, 0.75, 0.06), m.planks, at=(cx, y, 0.76), bevel=0.01)
        for sx in (-1, 1):
            block("table_leg", (0.08, 0.6, 0.7), m.timber, at=(cx + sx * 0.7, y, 0.38), bevel=0.01)
        for sy in (-1, 1):
            block("bench", (1.8, 0.3, 0.05), m.planks, at=(cx, y + sy * 0.62, 0.45), bevel=0.01)
            for sx in (-1, 1):
                block("bench_leg", (0.06, 0.26, 0.42), m.timber, at=(cx + sx * 0.75, y + sy * 0.62, 0.21), bevel=0.01)
        # Et krus og en tallerken på bordet.
        revolve("mug", [(0.0, 0.79), (0.05, 0.79), (0.055, 0.92), (0.0, 0.92)], m.planks, n=10, at=(cx - 0.4, y + 0.1, 0))
        revolve("plate", [(0.0, 0.79), (0.12, 0.79), (0.13, 0.81), (0.0, 0.81)], m.planks, n=14, at=(cx + 0.3, y - 0.05, 0))
    # Tønner og kasser ved hjørnet.
    for k, (x, y) in enumerate([(5.3, fy - 0.6), (5.0, fy - 1.25), (5.85, fy - 1.2)]):
        barrel(m, at=(x, y, 0.0), h=0.86, r=0.3, staves=16, rng=rng)
    barrel(m, at=(5.4, fy - 0.9, 0.86), h=0.7, r=0.25, staves=14, rng=rng)
    crate(m, at=(-5.4, fy - 0.6, 0.0), s=0.6, rng=rng)
    collide((5.4, fy - 0.95, 0.75), (1.5, 1.4, 1.5))
    collide((-5.4, fy - 0.6, 0.3), (0.65, 0.65, 0.6))
    for cx in (-3.6, 2.6):
        collide((cx, fy - 1.1, 0.42), (1.9, 1.6, 0.84))
    # --- Inne: skjenkestua.
    w, d, z = spec["w"], spec["d"], spec["plinth"]
    fh = 2.7
    top = z + fh
    block("floor", (w - 0.3, d - 0.3, 0.03), m.floor, at=(0, 0, z + 0.005), bevel=0)
    for k in range(9):
        block("straw", (rng.uniform(0.6, 1.4), rng.uniform(0.5, 1.1), 0.012), m.straw, at=(rng.uniform(-4, 4), rng.uniform(-3.5, 3.5), z + 0.025), rot=(0, 0, rng.uniform(0, 3)), bevel=0)
    # Bjelkelaget under overetasjen, med tak av bord og et trapphull langs
    # østveggen (hx0..hx1, hy0..hy1).
    hx0, hx1, hy0, hy1 = w / 2 - 1.1, w / 2 - 0.15, -2.9, 0.65
    stairwell(m, -w / 2 + 0.15, w / 2 - 0.15, -d / 2 + 0.15, d / 2 - 0.15, top, (hx0, hx1, hy0, hy1), 15, upper_y0=-d / 2 - spec["jetty"] + 0.15)
    # Drageren stopper i en veksel ved trapphullet.
    beam("summer", (-w / 2 + 0.15, 0.0, top - 0.32), (hx0 - 0.1, 0.0, top - 0.32), 0.26, 0.26, m.timber)
    beam("header", (hx0 - 0.1, hy0 - 0.16, top - 0.2), (hx0 - 0.1, hy1 + 0.16, top - 0.2), 0.2, 0.3, m.timber, roll=math.pi / 2)
    for x in (-2.0, 2.0):
        beam("post", (x, 0.0, z), (x, 0.0, top - 0.45), 0.24, 0.24, m.timber, roll=math.pi / 2)
        collide((x, 0.0, z + 1.2), (0.3, 0.3, 2.4))
    # Peisen mot bakveggen: mur av stein med åpning, glør og vedkubber, og
    # en hette opp til taket.
    px, py = 3.2, d / 2 - 0.62
    hearth = block("hearth", (2.4, 0.95, 1.35), m.soot, at=(px, py, z + 0.675), bevel=0.04, segments=2)
    cut(hearth, (1.4, 0.8, 0.85), (px, py - 0.35, z + 0.45))
    town.uv_box(hearth)
    for k in range(4):
        t = k / 3
        ww = 2.3 - 0.9 * t
        block("hood", (ww, 0.85 - 0.25 * t, 0.36), m.soot, at=(px, py + 0.05 * t, z + 1.53 + t * 0.36), bevel=0.03, segments=2)
    block("mantel", (2.6, 0.3, 0.14), m.timber, at=(px, py - 0.55, z + 1.42), bevel=0.02)
    for k in range(3):
        beam("log", (px - 0.45, py - 0.35 + k * 0.08, z + 0.12 + k * 0.08), (px + 0.45, py - 0.4 + k * 0.08, z + 0.12 + k * 0.08), 0.12, 0.12, m.bark)
    block("embers", (1.1, 0.5, 0.05), m.ember, at=(px, py - 0.3, z + 0.05), bevel=0.02)
    revolve("pot", [(0.0, z + 0.45), (0.16, z + 0.45), (0.2, z + 0.62), (0.18, z + 0.75), (0.0, z + 0.75)], m.dark_iron, n=14, at=(px + 0.2, py - 0.35, 0))
    beam("pot_hook", (px + 0.2, py - 0.35, z + 0.75), (px + 0.2, py - 0.35, z + 1.0), 0.015, 0.015, m.dark_iron)
    for k in range(3):
        revolve("plate", [(0.0, z + 1.49), (0.12, z + 1.49), (0.13, z + 1.51), (0.0, z + 1.51)], m.dark_iron, n=12, at=(px - 0.8 + k * 0.35, py - 0.55, 0))
    collide((px, py, z + 1.0), (2.5, 1.0, 2.0))
    light((px, py - 0.5, z + 0.5), (1.0, 0.5, 0.2), 3.2, 9.0)
    # Disken langs vestveggen, med tønner bak og en hylle med krus.
    bxr = -w / 2 + 1.35
    block("bar_top", (0.7, 4.4, 0.08), m.wood, at=(bxr, 1.1, z + 1.08), bevel=0.01)
    block("bar_front", (0.06, 4.4, 1.04), m.wood, at=(bxr + 0.3, 1.1, z + 0.52), bevel=0.006)
    for k in range(9):
        beam("bar_plank", (bxr + 0.34, -1.1 + k * 0.55, z), (bxr + 0.34, -1.1 + k * 0.55, z + 1.04), 0.03, 0.08, m.timber, roll=math.pi / 2)
    for k, y in enumerate((-0.6, 0.4, 1.4, 2.4)):
        barrel(m, at=(-w / 2 + 0.45, y, z), h=0.86, r=0.3, staves=16, rng=rng)
        beam("tap", (-w / 2 + 0.75, y, z + 0.25), (-w / 2 + 0.85, y, z + 0.25), 0.03, 0.03, m.wood)
    beam("mug_shelf", (-w / 2 + 0.2, -0.9, z + 1.75), (-w / 2 + 0.2, 3.1, z + 1.75), 0.3, 0.04, m.wood)
    for k in range(10):
        y = -0.7 + k * 0.38
        if k % 3 == 2:
            jar_r = 0.05
            revolve("bottle", [(0.0, 0.0), (jar_r, 0.0), (jar_r, 0.18), (0.02, 0.25), (0.02, 0.3), (0.0, 0.3)], m.glass, n=10, at=(-w / 2 + 0.22, y, z + 1.77))
        else:
            revolve("mug", [(0.0, 0.0), (0.05, 0.0), (0.055, 0.13), (0.0, 0.13)], m.wood, n=10, at=(-w / 2 + 0.22, y, z + 1.77))
    for k in range(3):
        revolve("mug", [(0.0, 0.0), (0.05, 0.0), (0.055, 0.13), (0.0, 0.13)], m.wood, n=10, at=(bxr, 0.2 + k * 0.6, z + 1.12))
    collide((-w / 2 + 0.85, 1.1, z + 0.55), (1.4, 4.5, 1.1))
    # Skinker og urter fra bjelkene over disken.
    for k, y in enumerate((-0.2, 0.9, 2.2)):
        beam("ham_string", (bxr - 0.3, y, top - 0.25), (bxr - 0.3, y, top - 0.55), 0.01, 0.01, m.wood)
        revolve("ham", [(0.0, top - 1.05), (0.1, top - 0.95), (0.13, top - 0.75), (0.06, top - 0.58), (0.0, top - 0.55)], m.leather, n=10, at=(bxr - 0.3, y, 0))
    # Bordene med benker og lys.
    for (tx, ty, ax) in ((1.2, 1.3, True), (1.6, -2.1, True), (-2.0, 2.4, True)):
        tz = table(m, tx, ty, z, along_x=ax, ln=1.9, rng=rng)
        holder(m, tx + 0.3, ty, tz)
        light((tx + 0.3, ty, tz + 0.3), (1.0, 0.7, 0.4), 0.9, 4.0)
        revolve("mug", [(0.0, 0.0), (0.05, 0.0), (0.055, 0.13), (0.0, 0.13)], m.wood, n=10, at=(tx - 0.5, ty + 0.15, tz))
        revolve("plate", [(0.0, 0.0), (0.12, 0.0), (0.13, 0.02), (0.0, 0.02)], m.wood, n=12, at=(tx - 0.1, ty - 0.1, tz))
    # Trappa opp langs østveggen gjennom trapphullet, med en meter fri
    # gulvplass foran nederste trinn.
    stairs(m, hx0 + 0.03, hx1, hy1, z, top, 12, 0.33, open_x=(hx0 + 0.05,))
    # Lysekrona: en jernring med lys i kjetting fra takbjelken.
    cx, cy = -0.4, 0.0
    beam("chain", (cx, cy, top - 0.45), (cx, cy, top - 1.0), 0.015, 0.015, m.dark_iron)
    revolve("ring", [(0.42, top - 1.04), (0.46, top - 1.02), (0.42, top - 1.0), (0.38, top - 1.02)], m.dark_iron, n=24, at=(cx, cy, 0), cap=False)
    for k in range(6):
        a = k * math.pi / 3
        candle(m, cx + math.cos(a) * 0.42, cy + math.sin(a) * 0.42, top - 1.0, h=0.14)
    light((cx, cy, top - 0.85), (1.0, 0.72, 0.42), 1.8, 8.0)
    # Kolliderne: gulvet (sokkelen), veggene med døra, taket over stua og
    # trinnene ute.
    collide((0, 0, (z - 0.24) / 2), (w + 0.16, d + 0.16, z + 0.24))
    walls_with_door(-w / 2, w / 2, -d / 2, d / 2, z, top, 0.26, -1.2, 1.0, 2.05, -d / 2)
    upper(m, w, d, spec["jetty"], top, fh, (hx0, hx1, hy0, hy1), rng)
    block("step_low", (1.8, 0.5, 0.3), m.stone, at=(-1.2, -d / 2 - 0.75, 0.15 - 0.12), bevel=0.03, segments=2)
    ramp(-1.95, -0.45, -d / 2 - 1.3, -d / 2 - 0.05, 0.0, z)
    return m


def bed(m, x, y, z, along_y=True, rng=None, wool=None):
    """En seng: ramme av planker med gavl, halmmadrass, ullteppe og
    linpute. Hodet mot +y (eller +x)."""
    rng = rng or random.Random(int(x * 10 + y))
    wool = wool or m.wool
    L, B = 2.0, 0.95
    sx, sy = (B, L) if along_y else (L, B)
    head = Vector((0, L / 2, 0)) if along_y else Vector((L / 2, 0, 0))
    side = Vector((B / 2, 0, 0)) if along_y else Vector((0, B / 2, 0))
    c = Vector((x, y, z))
    for k in (-1, 1):
        block("bed_side", (0.06, L, 0.28) if along_y else (L, 0.06, 0.28), m.wood, at=c + side * k + Vector((0, 0, 0.32)), bevel=0.008)
        for j in (-1, 1):
            block("bed_leg", (0.09, 0.09, 0.46), m.timber, at=c + side * k + head * j * 0.97 + Vector((0, 0, 0.23)), bevel=0.01)
    block("bed_head", (B + 0.1, 0.06, 0.95) if along_y else (0.06, B + 0.1, 0.95), m.wood, at=c + head + Vector((0, 0, 0.47)), bevel=0.01)
    block("bed_foot", (B + 0.1, 0.06, 0.5) if along_y else (0.06, B + 0.1, 0.5), m.wood, at=c - head + Vector((0, 0, 0.25)), bevel=0.01)
    block("mattress", (sx - 0.08, sy - 0.1, 0.16), m.straw, at=c + Vector((0, 0, 0.5)), bevel=0.05, segments=3)
    # Teppet over to tredjedeler, litt skjevt og bølgete over kanten.
    block("blanket", (sx + 0.02, sy * 0.66, 0.05) if along_y else (sx * 0.66, sy + 0.02, 0.05), wool, at=c - head * 0.32 + Vector((0, 0, 0.6)), rot=(0, 0, rng.uniform(-0.05, 0.05)), bevel=0.02, segments=2)
    block("blanket_fold", (sx + 0.02, 0.12, 0.08) if along_y else (0.12, sy + 0.02, 0.08), wool, at=c + head * 0.02 + Vector((0, 0, 0.62)), bevel=0.03, segments=2)
    block("pillow", (0.55, 0.32, 0.12) if along_y else (0.32, 0.55, 0.12), m.linen, at=c + head * 0.8 + Vector((0, 0, 0.64)), rot=(0, 0, rng.uniform(-0.1, 0.1)), bevel=0.05, segments=3)
    collide((x, y, z + 0.35), (sx + 0.12, sy + 0.12, 0.7))


def chest(m, x, y, z, along_x=True):
    """En kiste med jernbånd og lokk."""
    sx, sy = (0.9, 0.5) if along_x else (0.5, 0.9)
    block("chest", (sx, sy, 0.45), m.planks, at=(x, y, z + 0.225), bevel=0.012)
    block("chest_lid", (sx + 0.03, sy + 0.03, 0.08), m.planks, at=(x, y, z + 0.49), bevel=0.02, segments=2)
    for k in (-1, 1):
        o = (k * sx * 0.32, 0) if along_x else (0, k * sy * 0.32)
        block("chest_band", (0.05, sy + 0.04, 0.5) if along_x else (sx + 0.04, 0.05, 0.5), m.dark_iron, at=(x + o[0], y + o[1], z + 0.27), bevel=0.004)
    block("chest_lock", (0.08, 0.03, 0.1) if along_x else (0.03, 0.08, 0.1), m.dark_iron, at=(x, y - sy / 2 - 0.02, z + 0.4) if along_x else (x - sx / 2 - 0.02, y, z + 0.4), bevel=0.005)
    collide((x, y, z + 0.27), (sx + 0.05, sy + 0.05, 0.54))


def upper(m, w, d, jetty, z, fh, hole, rng):
    """Overetasjen i Vertshuset: sovesal med senger, kister, vaskefat og et
    bord, under sperrene og halmtaket."""
    m.wool = layer("wool", (0.62, 0.55, 0.48), 0.5)
    m.wool2 = layer("wool", (0.45, 0.5, 0.58), 0.5)
    m.linen = layer("linen", (0.85, 0.82, 0.75), 0.5)
    m.clay = layer("brick", (0.75, 0.5, 0.36), 0.5)
    hx0, hx1, hy0, hy1 = hole
    fz = z + 0.02
    y0, y1 = -d / 2 - jetty + 0.15, d / 2 - 0.15
    x0, x1 = -w / 2 + 0.15, w / 2 - 0.15
    for k in range(6):
        block("straw", (rng.uniform(0.5, 1.1), rng.uniform(0.4, 0.9), 0.012), m.straw, at=(rng.uniform(-4.5, 3.5), rng.uniform(-3.5, 3.0), fz + 0.006), rot=(0, 0, rng.uniform(0, 3)), bevel=0)
    # Rekkverket rundt trapphullet.
    rail_run(m, (hx0 - 0.04, hy1 - 0.1), (hx0 - 0.04, hy0 - 0.04), fz)
    rail_run(m, (hx0 - 0.04, hy0 - 0.04), (hx1, hy0 - 0.04), fz)
    # Sperrene og hanebjelkene under taket (mønet langs x).
    pitch = math.radians(50)
    run = (d + jetty) / 2 + 0.45
    eave = z + fh
    yc = 0.0

    def under(y):
        return eave + (run - abs(y - yc)) * math.tan(pitch) - 0.22

    for k in range(9):
        x = -w / 2 + 0.6 + (w - 1.2) * k / 8
        for sy in (-1, 1):
            ya = yc + sy * (run - 0.7)
            beam("rafter", (x, ya, under(ya)), (x, yc, under(yc) - 0.05), 0.12, 0.16, m.timber)
        cz = eave + 1.6
        half = run - (cz + 0.22 - eave) / math.tan(pitch)
        beam("collar", (x, yc - half + 0.1, cz), (x, yc + half - 0.1, cz), 0.12, 0.14, m.timber)
    beam("purlin", (-w / 2 + 0.2, yc, under(yc) - 0.2), (w / 2 - 0.2, yc, under(yc) - 0.2), 0.18, 0.18, m.timber)
    # Sengene langs bakveggen og en ved gavlen, med en kiste ved foten.
    for k, bx in enumerate((-4.6, -2.4, -0.2)):
        bed(m, bx, y1 - 1.05, fz, along_y=True, rng=rng, wool=m.wool2 if k == 1 else None)
        chest(m, bx, y1 - 2.4, fz)
    bed(m, x1 - 1.05, 3.0, fz, along_y=False, rng=rng)
    # Bordet ved vinduene mot gata, med krakk, lys og et krus.
    tx, ty = -3.2, y0 + 0.9
    block("table_top", (1.3, 0.7, 0.05), m.planks, at=(tx, ty, fz + 0.74), bevel=0.01)
    for sx in (-1, 1):
        for sy in (-1, 1):
            block("table_leg", (0.06, 0.06, 0.72), m.timber, at=(tx + sx * 0.55, ty + sy * 0.28, fz + 0.36), bevel=0.008)
    for sx in (-1, 1):
        revolve("stool", [(0.0, fz + 0.45), (0.17, fz + 0.45), (0.17, fz + 0.5), (0.0, fz + 0.5)], m.wood, n=14, at=(tx + sx * 0.45, ty + 0.65, 0))
        for a in range(3):
            ang = a * 2 * math.pi / 3
            ox, oy = tx + sx * 0.45 + math.cos(ang) * 0.1, ty + 0.65 + math.sin(ang) * 0.1
            beam("stool_leg", (ox, oy, fz + 0.45), (ox + math.cos(ang) * 0.06, oy + math.sin(ang) * 0.06, fz), 0.03, 0.03, m.timber)
    holder(m, tx + 0.35, ty, fz + 0.765)
    candle(m, tx + 0.35, ty, fz + 0.9, h=0.12)
    revolve("mug", [(0.0, 0.0), (0.05, 0.0), (0.055, 0.13), (0.0, 0.13)], m.wood, n=10, at=(tx - 0.3, ty + 0.1, fz + 0.765))
    collide((tx, ty, fz + 0.38), (1.4, 0.8, 0.76))
    light((tx + 0.35, ty, fz + 1.2), (1.0, 0.7, 0.4), 1.0, 5.0)
    # Vaskefatet på en krakk ved vestgavlen, med mugge.
    vx, vy = x0 + 0.4, -0.6
    block("wash_stand", (0.5, 0.5, 0.7), m.wood, at=(vx, vy, fz + 0.35), bevel=0.01)
    revolve("basin", [(0.0, 0.0), (0.12, 0.0), (0.2, 0.1), (0.21, 0.11), (0.0, 0.11)], m.clay, n=16, at=(vx, vy, fz + 0.7))
    revolve("jug", [(0.0, 0.0), (0.07, 0.0), (0.09, 0.08), (0.06, 0.2), (0.05, 0.26), (0.0, 0.26)], m.clay, n=12, at=(vx + 0.05, vy + 0.18, fz + 0.7))
    collide((vx, vy, fz + 0.35), (0.55, 0.55, 0.7))
    # Knagger med en kappe og en veske på vestveggen.
    for k in range(4):
        py = 0.6 + k * 0.45
        beam("peg", (x0, py, fz + 1.65), (x0 + 0.14, py, fz + 1.67), 0.03, 0.03, m.timber)
    block("cloak", (0.06, 0.45, 1.1), m.wool2, at=(x0 + 0.1, 0.6, fz + 1.12), bevel=0.03, segments=2)
    block("bag", (0.1, 0.26, 0.3), m.leather, at=(x0 + 0.11, 1.95, fz + 1.4), bevel=0.04, segments=2)
    # Nattpotta under en seng.
    revolve("chamber_pot", [(0.0, 0.0), (0.1, 0.0), (0.13, 0.08), (0.12, 0.16), (0.0, 0.16)], m.clay, n=12, at=(-2.0, y1 - 1.9, fz))
    # Lykta i kjetting fra hanebjelken midt i salen.
    lx, ly = -1.0, -1.0
    beam("lamp_chain", (lx, ly, eave + 1.55), (lx, ly, fz + 2.2), 0.015, 0.015, m.dark_iron)
    block("lamp_cage", (0.2, 0.2, 0.28), m.dark_iron, at=(lx, ly, fz + 2.05), bevel=0.01)
    candle(m, lx, ly, fz + 1.93, h=0.1)
    light((lx, ly, fz + 2.0), (1.0, 0.68, 0.38), 1.4, 7.0)
    # Kolliderne: veggene oppe.
    t = 0.26
    yf = -d / 2 - jetty
    wall_box(-w / 2, yf - t / 2, w / 2, yf + t / 2, z, eave)
    wall_box(-w / 2, d / 2 - t / 2, w / 2, d / 2 + t / 2, z, eave)
    for x in (-w / 2, w / 2):
        wall_box(x - t / 2, yf, x + t / 2, d / 2, z, eave)


# --- Apoteket ---------------------------------------------------------------


def jar(m, x, y, z, r, h, mat):
    """En krukke eller flaske (dreid), med kork."""
    revolve("jar", [(0.0, z), (r * 0.8, z), (r, z + h * 0.15), (r, z + h * 0.7), (r * 0.45, z + h * 0.88), (r * 0.4, z + h), (0.0, z + h)], mat, n=12, at=(x, y, 0))
    revolve("cork", [(0.0, z + h), (r * 0.36, z + h), (r * 0.36, z + h + 0.03), (0.0, z + h + 0.03)], m.bark, n=8, at=(x, y, 0))


def apothecary():
    spec = dict(seed=41, w=7.4, d=7.0, floors=2, jetty=0.4, pitch=55, gable=True, roof="slate",
                plaster=(0.95, 0.97, 0.9), door_x=1.9, windows0=[-1.4], windows_up=2, plinth=0.7, open_door=True)
    m = town.half_timbered(spec)
    inside(m)
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.dark_iron = layer("metal", (0.28, 0.26, 0.25), 0.5)
    m.bark = layer("bark", (0.8, 0.75, 0.7), 0.5)
    m.clay = layer("brick", (0.75, 0.5, 0.36), 0.5)
    m.herb = layer("thatch", (0.45, 0.55, 0.3), 0.5)
    m.herb2 = layer("thatch", (0.6, 0.45, 0.4), 0.5)
    rng = random.Random(42)
    fy = -spec["d"] / 2 - 0.11
    # Salgsluka: skodda er slått ned til et bord på to knekter under vinduet.
    wx, wz = -1.4, 0.7 + 1.45
    block("counter", (1.25, 0.5, 0.05), m.planks, at=(wx, fy - 0.27, wz - 0.55), bevel=0.008)
    for sx in (-1, 1):
        beam("bracket", (wx + sx * 0.5, fy - 0.02, wz - 1.0), (wx + sx * 0.5, fy - 0.45, wz - 0.58), 0.05, 0.05, m.timber)
    for k in range(6):
        x = wx - 0.5 + k * 0.2
        if k % 2:
            jar(m, x, fy - 0.27 + rng.uniform(-0.08, 0.08), wz - 0.525, 0.06, rng.uniform(0.14, 0.2), m.clay)
        else:
            jar(m, x, fy - 0.27 + rng.uniform(-0.08, 0.08), wz - 0.525, 0.05, rng.uniform(0.16, 0.24), m.glass)
    # En vekt og en morter på bordet.
    revolve("mortar", [(0.0, 0.0), (0.09, 0.0), (0.11, 0.1), (0.08, 0.1), (0.06, 0.04), (0.0, 0.04)], m.stone, n=14, at=(wx + 0.45, fy - 0.35, wz - 0.525))
    beam("pestle", (wx + 0.45, fy - 0.35, wz - 0.48), (wx + 0.52, fy - 0.3, wz - 0.3), 0.025, 0.025, m.planks)
    # Urtebunter til tørk på en stang under den framskutte etasjen.
    bar_z = 0.7 + 2.7 - 0.25
    beam("herb_bar", (-3.4, fy - 0.55, bar_z), (0.6, fy - 0.55, bar_z), 0.04, 0.04, m.timber)
    for k in range(9):
        x = -3.2 + k * 0.42
        mat = m.herb if k % 3 else m.herb2
        ln = rng.uniform(0.3, 0.45)
        b = revolve("herbs", [(0.0, -ln), (0.07, -ln * 0.8), (0.05, -0.05), (0.015, 0.0)], mat, n=8, at=(x, fy - 0.55, bar_z - 0.02))
        b.rotation_euler = (rng.uniform(-0.1, 0.1), rng.uniform(-0.1, 0.1), 0)
        beam("herb_tie", (x, fy - 0.55, bar_z), (x, fy - 0.55, bar_z - 0.06), 0.012, 0.012, m.planks)
    # Plantekrukker ved døra.
    for k, (x, y) in enumerate([(3.1, fy - 0.5), (3.3, fy - 1.0), (0.7, fy - 0.45)]):
        r = rng.uniform(0.18, 0.24)
        revolve("pot", [(0.0, 0.0), (r * 0.8, 0.0), (r, r * 1.3), (r * 1.05, r * 1.4), (0.0, r * 1.4)], m.clay, n=14, at=(x, y, 0))
        for j in range(6):
            a = j * 1.05
            b = revolve("plant", [(0.0, 0.0), (0.04, 0.12), (0.0, 0.35)], m.herb, n=5, at=(x + math.cos(a) * r * 0.4, y + math.sin(a) * r * 0.4, r * 1.35))
            b.rotation_euler = (math.cos(a) * 0.4, math.sin(a) * 0.4, 0)
    # Skiltet: en morter med støter i smijern på en arm ved hjørnet.
    sx = spec["w"] / 2 - 0.1
    beam("sign_arm", (sx, fy, 3.6), (sx, fy - 1.0, 3.6), 0.05, 0.05, m.dark_iron)
    beam("sign_strut", (sx, fy, 3.1), (sx, fy - 0.7, 3.58), 0.03, 0.03, m.dark_iron)
    beam("sign_chain", (sx, fy - 0.75, 3.58), (sx, fy - 0.75, 3.35), 0.015, 0.015, m.dark_iron)
    revolve("sign_mortar", [(0.0, -0.24), (0.16, -0.24), (0.2, 0.0), (0.16, 0.0), (0.0, -0.08)], m.dark_iron, n=14, at=(sx, fy - 0.75, 3.35))
    beam("sign_pestle", (sx - 0.05, fy - 0.75, 3.2), (sx + 0.12, fy - 0.75, 3.5), 0.04, 0.04, m.dark_iron)
    # Kolliderne ute: salgsbordet og krukkene.
    collide((wx, fy - 0.3, wz - 0.75), (1.4, 0.6, 1.0))
    collide((3.2, fy - 0.75, 0.25), (0.8, 1.1, 0.5))
    collide((0.7, fy - 0.45, 0.25), (0.5, 0.5, 0.5))
    # --- Inne: apoteket med disk, hyller fulle av krukker, destillerbenk
    # og urter til tørk.
    w, d, z = spec["w"], spec["d"], spec["plinth"]
    fh = 2.7
    top = z + fh
    m.copper = layer("metal", (0.75, 0.42, 0.25), 0.5)
    m.glow = plain("fire_coals", (1.0, 0.45, 0.12), 0.4)
    block("floor", (w - 0.3, d - 0.3, 0.03), m.floor, at=(0, 0, z + 0.005), bevel=0)
    # Bjelkelaget med trapphull langs østveggen.
    hole = (w / 2 - 1.1, w / 2 - 0.15, -2.35, 1.1)
    stairwell(m, -w / 2 + 0.15, w / 2 - 0.15, -d / 2 + 0.15, d / 2 - 0.15, top, hole, 9, upper_y0=-d / 2 - spec["jetty"] + 0.15)
    stairs(m, hole[0] + 0.05, hole[1], hole[3], z, top, 13, 0.27, open_x=(hole[0] + 0.07,))
    # Hyllene langs bakveggen og vestveggen, med krukker, flasker og bøker.
    jar_mats = [m.clay, m.glass, m.clay, m.books[2]]
    for (sx, sy, ax, ln) in ((-1.4, d / 2 - 0.3, True, 2.6), (1.5, d / 2 - 0.3, True, 1.4), (-w / 2 + 0.3, 0.2, False, 3.0)):
        shelf(m, sx, sy, z, ln, 2.3, along_x=ax, rows=5, books=False, rng=rng)
        for r in range(5):
            zz = z + 0.05 + 2.2 * r / 5 + 0.015
            u = -ln / 2 + 0.12
            while u < ln / 2 - 0.1:
                rr = rng.uniform(0.04, 0.075)
                hh = rng.uniform(0.12, 0.3)
                x, y = (sx + u, sy) if ax else (sx, sy + u)
                if rng.random() < 0.15 and r > 0:
                    books_row(m, u + (sx if ax else sy), u + (sx if ax else sy) + 0.25, sy if ax else sx, zz, rng=rng, axis_x=ax)
                    u += 0.3
                    continue
                jar(m, x, y, zz, rr, min(hh, 2.2 / 5 - 0.08), rng.choice(jar_mats))
                u += rr * 2 + rng.uniform(0.02, 0.06)
    # Disken: på tvers av rommet, med vekt, morter, papirer og ei pengekiste.
    cy = -0.4
    block("counter_top", (3.4, 0.7, 0.08), m.wood, at=(-1.7, cy, z + 1.0), bevel=0.01)
    block("counter_front", (3.4, 0.06, 0.96), m.wood, at=(-1.7, cy - 0.32, z + 0.48), bevel=0.006)
    for k in range(7):
        beam("counter_plank", (-3.3 + k * 0.53, cy - 0.36, z), (-3.3 + k * 0.53, cy - 0.36, z + 0.96), 0.03, 0.08, m.timber, roll=math.pi / 2)
    # Skålvekta.
    beam("scale_post", (-2.6, cy, z + 1.04), (-2.6, cy, z + 1.45), 0.03, 0.03, m.copper)
    beam("scale_beam", (-2.85, cy, z + 1.45), (-2.35, cy, z + 1.45), 0.02, 0.02, m.copper)
    for sx_ in (-1, 1):
        beam("scale_chain", (-2.6 + sx_ * 0.25, cy, z + 1.45), (-2.6 + sx_ * 0.25, cy, z + 1.2), 0.006, 0.006, m.copper)
        revolve("scale_pan", [(0.0, z + 1.18), (0.09, z + 1.2), (0.0, z + 1.2)], m.copper, n=12, at=(-2.6 + sx_ * 0.25, cy, 0))
    revolve("mortar", [(0.0, 0.0), (0.1, 0.0), (0.12, 0.11), (0.09, 0.11), (0.07, 0.04), (0.0, 0.04)], m.stone, n=14, at=(-1.6, cy + 0.1, z + 1.04))
    open_book(m, -0.8, cy + 0.05, z + 1.04)
    holder(m, -0.3, cy + 0.2, z + 1.04)
    light((-0.3, cy + 0.2, z + 1.4), (1.0, 0.72, 0.42), 0.9, 4.0)
    collide((-1.7, cy, z + 0.5), (3.5, 0.8, 1.0))
    # Destillerbenken i hjørnet: kobberkjele over en glohaug, rør og glass.
    bx, by = 2.4, d / 2 - 1.2
    block("still_bench", (1.5, 0.8, 0.07), m.wood, at=(bx, by, z + 0.85), bevel=0.01)
    for sx_ in (-1, 1):
        for sy_ in (-1, 1):
            block("still_leg", (0.08, 0.08, 0.82), m.timber, at=(bx + sx_ * 0.65, by + sy_ * 0.32, z + 0.41), bevel=0.01)
    block("brazier", (0.4, 0.4, 0.12), m.dark_iron, at=(bx - 0.35, by, z + 0.95), bevel=0.02)
    block("coals", (0.3, 0.3, 0.04), m.glow, at=(bx - 0.35, by, z + 1.02), bevel=0.01)
    revolve("alembic", [(0.0, z + 1.04), (0.16, z + 1.06), (0.2, z + 1.2), (0.16, z + 1.34), (0.05, z + 1.44), (0.04, z + 1.55), (0.0, z + 1.56)], m.copper, n=16, at=(bx - 0.35, by, 0))
    beam("still_pipe", (bx - 0.33, by, z + 1.52), (bx + 0.35, by, z + 1.12), 0.025, 0.025, m.copper)
    jar(m, bx + 0.4, by, z + 0.89, 0.09, 0.24, m.glass)
    for k in range(4):
        jar(m, bx + 0.1 + k * 0.12, by + 0.28, z + 0.89, 0.04, 0.14, m.glass)
    light((bx - 0.35, by, z + 1.2), (1.0, 0.5, 0.2), 1.4, 5.0)
    collide((bx, by, z + 0.5), (1.6, 0.9, 1.0))
    # Urtebunter fra bjelkene.
    for k in range(10):
        x = -w / 2 + 0.4 + (w - 0.8) * (k % 8) / 8
        y = rng.uniform(-2.5, 1.0)
        ln = rng.uniform(0.25, 0.4)
        revolve("herbs", [(0.0, top - 0.25 - ln), (0.06, top - 0.25 - ln * 0.8), (0.045, top - 0.3), (0.012, top - 0.25)], m.herb if k % 3 else m.herb2, n=8, at=(x, y, 0))
    # Sekker og en krakk.
    for x, y in ((-3.2, -2.9), (-2.75, -3.0)):
        revolve("sack", [(0.0, z), (0.2, z + 0.02), (0.24, z + 0.2), (0.18, z + 0.45), (0.06, z + 0.55), (0.0, z + 0.56)], m.leather, n=12, at=(x, y, 0))
    collide((-3.0, -2.95, z + 0.3), (1.0, 0.55, 0.6))
    # Kolliderne: gulvet, veggene med døra, taket og rampa opp.
    collide((0, 0, (z - 0.24) / 2), (w + 0.16, d + 0.16, z + 0.24))
    walls_with_door(-w / 2, w / 2, -d / 2, d / 2, z, top, 0.26, 1.9, 1.0, 2.05, -d / 2)
    herb_loft(m, w, d, spec["jetty"], top, fh, hole, rng)
    ramp(1.15, 2.65, -d / 2 - 1.4, -d / 2 - 0.05, 0.0, z)
    block("step_low", (1.6, 0.5, 0.36), m.stone, at=(1.9, -d / 2 - 0.8, 0.06), bevel=0.03, segments=2)
    return m


def herb_loft(m, w, d, jetty, z, fh, hole, rng):
    """Overetasjen i Apoteket: apotekerens kammer og urteloft under
    sperrene, med seng, skrivepult, tørkestativ, kurver og urter i taket."""
    m.wool = layer("wool", (0.5, 0.42, 0.36), 0.5)
    m.linen = layer("linen", (0.85, 0.82, 0.75), 0.5)
    hx0, hx1, hy0, hy1 = hole
    fz = z + 0.02
    y0, y1 = -d / 2 - jetty + 0.15, d / 2 - 0.15
    x0, x1 = -w / 2 + 0.15, w / 2 - 0.15
    rail_run(m, (hx0 - 0.04, hy1 - 0.1), (hx0 - 0.04, hy0 - 0.04), fz)
    rail_run(m, (hx0 - 0.04, hy0 - 0.04), (hx1, hy0 - 0.04), fz)
    for k in range(5):
        block("straw", (rng.uniform(0.4, 0.9), rng.uniform(0.4, 0.8), 0.012), m.straw, at=(rng.uniform(-2.5, 1.5), rng.uniform(-2.5, 2.5), fz + 0.006), rot=(0, 0, rng.uniform(0, 3)), bevel=0)
    # Sperrene (mønet langs y) med hanebjelker, og stenger med urter.
    pitch = math.radians(55)
    run = (w + 0.1) / 2 + 0.45
    eave = z + fh

    def under(x):
        return eave + (run - abs(x)) * math.tan(pitch) - 0.22

    for k in range(9):
        y = y0 + 0.1 + (y1 - y0 - 0.2) * k / 8
        for sx in (-1, 1):
            xa = sx * (run - 0.75)
            beam("rafter", (xa, y, under(xa)), (0, y, under(0) - 0.05), 0.12, 0.16, m.timber)
        cz = eave + 1.7
        half = run - (cz + 0.22 - eave) / math.tan(pitch)
        beam("collar", (-half + 0.1, y, cz), (half - 0.1, y, cz), 0.12, 0.14, m.timber)
    beam("purlin", (0, y0, under(0) - 0.2), (0, y1, under(0) - 0.2), 0.18, 0.18, m.timber)
    for x in (-1.2, 0.0, 1.2):
        bar_z = eave + 1.55
        beam("herb_pole", (x, y0 + 0.3, bar_z), (x, y1 - 0.3, bar_z), 0.04, 0.04, m.timber)
        y = y0 + 0.5
        while y < y1 - 0.4:
            ln = rng.uniform(0.3, 0.5)
            revolve("herbs", [(0.0, bar_z - ln), (0.07, bar_z - ln * 0.8), (0.05, bar_z - 0.06), (0.015, bar_z - 0.01)], m.herb if rng.random() < 0.65 else m.herb2, n=8, at=(x + rng.uniform(-0.03, 0.03), y, 0))
            y += rng.uniform(0.28, 0.45)
    # Senga langs vestveggen med kiste ved foten.
    bed(m, x0 + 0.55, y1 - 1.05, fz, along_y=True, rng=rng)
    chest(m, x0 + 0.55, y1 - 2.4, fz)
    # Skrivepulten under det vestre vinduet mot gata, med urteboka.
    tx, ty = -1.85, y0 + 0.55
    block("desk_top", (1.2, 0.6, 0.05), m.planks, at=(tx, ty, fz + 0.76), bevel=0.01)
    for sx in (-1, 1):
        block("desk_side", (0.05, 0.55, 0.74), m.planks, at=(tx + sx * 0.55, ty, fz + 0.37), bevel=0.006)
    open_book(m, tx - 0.1, ty, fz + 0.785)
    revolve("inkhorn", [(0.0, 0.0), (0.03, 0.0), (0.035, 0.08), (0.0, 0.08)], m.books[1], n=8, at=(tx + 0.3, ty + 0.12, fz + 0.785))
    beam("quill", (tx + 0.3, ty + 0.12, fz + 0.86), (tx + 0.36, ty + 0.05, fz + 1.08), 0.008, 0.02, m.vellum)
    holder(m, tx + 0.45, ty - 0.1, fz + 0.785)
    candle(m, tx + 0.45, ty - 0.1, fz + 0.925, h=0.12)
    for k in range(3):
        jar(m, tx - 0.45 + k * 0.1, ty + 0.15, fz + 0.785, 0.035, 0.1, m.glass if k % 2 else m.clay)
    revolve("stool", [(0.0, fz + 0.45), (0.17, fz + 0.45), (0.17, fz + 0.5), (0.0, fz + 0.5)], m.wood, n=14, at=(tx, ty + 0.65, 0))
    for a in range(3):
        ang = a * 2 * math.pi / 3
        ox, oy = tx + math.cos(ang) * 0.1, ty + 0.65 + math.sin(ang) * 0.1
        beam("stool_leg", (ox, oy, fz + 0.45), (ox + math.cos(ang) * 0.06, oy + math.sin(ang) * 0.06, fz), 0.03, 0.03, m.timber)
    collide((tx, ty, fz + 0.4), (1.25, 0.65, 0.8))
    light((tx + 0.45, ty - 0.1, fz + 1.2), (1.0, 0.7, 0.4), 1.0, 4.5)
    # Tørkestativet: en ramme med fire brett urter, bak trapphullet.
    rx, ry = 2.6, y1 - 0.45
    for sx in (-1, 1):
        for sy in (-1, 1):
            beam("rack_post", (rx + sx * 0.6, ry + sy * 0.25, fz), (rx + sx * 0.6, ry + sy * 0.25, fz + 1.7), 0.05, 0.05, m.timber)
    for k in range(4):
        hz = fz + 0.3 + k * 0.4
        block("tray", (1.2, 0.5, 0.03), m.planks, at=(rx, ry, hz), bevel=0.004)
        for j in range(5):
            block("drying_herbs", (rng.uniform(0.15, 0.25), rng.uniform(0.1, 0.2), 0.05), m.herb if (j + k) % 3 else m.herb2, at=(rx - 0.45 + j * 0.22, ry + rng.uniform(-0.1, 0.1), hz + 0.035), rot=(0, 0, rng.uniform(0, 3)), bevel=0.02)
    collide((rx, ry, fz + 0.85), (1.3, 0.6, 1.7))
    # Kurver med røtter i hjørnet ved vestveggen.
    for k, (bx, by) in enumerate([(x0 + 0.35, -2.55), (x0 + 0.35, -2.0), (x0 + 0.9, -2.35)]):
        revolve("basket", [(0.0, 0.0), (0.18, 0.0), (0.24, 0.28), (0.22, 0.29), (0.0, 0.05)], m.straw, n=14, at=(bx, by, fz))
        revolve("roots", [(0.0, 0.2), (0.2, 0.22), (0.12, 0.32), (0.0, 0.34)], m.bark, n=10, at=(bx, by, fz))
    collide((x0 + 0.6, -2.3, fz + 0.15), (0.9, 1.1, 0.3))
    # Ei hylle med krukker på bakveggen.
    shelf(m, 0.6, y1 - 0.18, fz, 1.4, 1.6, along_x=True, rows=3, books=False, rng=rng)
    for r in range(3):
        zz = fz + 0.05 + 1.5 * r / 3 + 0.015
        for k in range(6):
            jar(m, 0.0 + k * 0.22, y1 - 0.18, zz, rng.uniform(0.04, 0.06), rng.uniform(0.12, 0.3), rng.choice([m.clay, m.glass, m.clay]))
    # Lykta midt i kammeret.
    lx, ly = -0.6, 0.0
    beam("lamp_chain", (lx, ly, eave + 1.62), (lx, ly, fz + 2.2), 0.015, 0.015, m.dark_iron)
    block("lamp_cage", (0.2, 0.2, 0.28), m.dark_iron, at=(lx, ly, fz + 2.05), bevel=0.01)
    candle(m, lx, ly, fz + 1.93, h=0.1)
    light((lx, ly, fz + 2.0), (1.0, 0.68, 0.38), 1.3, 6.5)
    # Veggene oppe.
    t = 0.26
    yf = -d / 2 - jetty
    wall_box(-w / 2, yf - t / 2, w / 2, yf + t / 2, z, eave)
    wall_box(-w / 2, d / 2 - t / 2, w / 2, d / 2 + t / 2, z, eave)
    for x in (-w / 2, w / 2):
        wall_box(x - t / 2, yf, x + t / 2, d / 2, z, eave)


# --- Skriptoriet ------------------------------------------------------------


def arched_window(m, target, u, face, normal, z0, w, h, thick, side=False):
    """Et høyt rundbuet vindu i en mur: hull, buestein, sålbenk og blyglass
    innfelt. Muren har yta i `face` med normalen `normal` (±1) langs y, eller
    langs x når `side`; `u` er plassen langs muren."""
    from walls import cutter
    s = normal

    def P(along, out, z):
        return Vector((face + s * out, along, z)) if side else Vector((along, face + s * out, z))

    def size(a, b, c):
        return (b, a, c) if side else (a, b, c)

    straight = h - w / 2
    cut(target, size(w, thick + 0.4, straight), P(u, -thick / 2, z0 + straight / 2))
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=w / 2, depth=thick + 0.4, location=P(u, -thick / 2, z0 + straight), rotation=(0, math.pi / 2, 0) if side else (math.pi / 2, 0, 0))
    cutter(bpy.context.active_object, target)
    town.clear_relief(target, size(w, thick + 0.4, w / 2), P(u, -thick / 2, z0 + straight + w / 4))
    town.uv_box(target)
    for k in range(9):
        a = math.pi * k / 8
        c = P(u + math.cos(a) * (w / 2 + 0.12), 0.04, z0 + straight + math.sin(a) * (w / 2 + 0.12))
        rot = (-(math.pi / 2 - a), 0, 0) if side else (0, math.pi / 2 - a, 0)
        block("voussoir", size(0.16, 0.14, 0.3), m.stone, at=c, rot=rot, bevel=0.02, segments=2)
    block("sill", size(w + 0.3, 0.22, 0.1), m.stone, at=P(u, 0.06, z0 - 0.05), bevel=0.02, segments=2)
    g = -thick * 0.6
    inward = (-s, 0, 0) if side else (0, -s, 0)
    town.daylight(block("glass", size(w, 0.02, straight), m.glass, at=P(u, g, z0 + straight / 2), bevel=0), inward)
    disc = revolve("glass_top", [(0.0, -0.01), (w / 2, -0.01), (w / 2, 0.01), (0.0, 0.01)], m.glass, n=16)
    disc.rotation_euler = (0, math.pi / 2, 0) if side else (math.pi / 2, 0, 0)
    disc.location = P(u, g, z0 + straight)
    town.daylight(disc, inward)
    for o in (0.015, -0.015):
        for k in range(1, 3):
            block("lead", size(0.012, 0.012, h - 0.05), m.iron, at=P(u - w / 2 + w * k / 3, g + o, z0 + (h - 0.05) / 2), bevel=0)
        for k in range(1, 6):
            block("lead", size(w, 0.012, 0.012), m.iron, at=P(u, g + o, z0 + straight * k / 5), bevel=0)


def scriptorium():
    m = mats(roof="slate")
    m.ashlar = layer("brick", (0.62, 0.6, 0.57), 0.5)
    W, D, H = 10.0, 8.0, 6.2
    t = 0.65
    front = stone_wall(m, "front", -W / 2, W / 2, -D / 2, -D / 2 + t, -0.24, H, m.ashlar)
    back = stone_wall(m, "back", -W / 2, W / 2, D / 2 - t, D / 2, -0.24, H, m.ashlar)
    sides = [stone_wall(m, f"side{sx}", (sx * W / 2) - (t if sx > 0 else 0), (sx * W / 2) + (0 if sx > 0 else t), -D / 2 + t, D / 2 - t, -0.24, H, m.ashlar) for sx in (-1, 1)]
    # Sokkelen: en bredere steinrad nederst.
    block("plinth", (W + 0.3, D + 0.3, 0.6), m.stone, at=(0, 0, 0.06), bevel=0.04, segments=2)
    # Rundbuet dør midt på, med buestein og trinn.
    dw, dh = 1.3, 2.6
    cut(front, (dw, t + 0.4, dh - dw / 2), (0, -D / 2 + t / 2, 0.35 + (dh - dw / 2) / 2))
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=dw / 2, depth=t + 0.4, location=(0, -D / 2 + t / 2, 0.35 + dh - dw / 2), rotation=(math.pi / 2, 0, 0))
    from walls import cutter
    cutter(bpy.context.active_object, front)
    town.clear_relief(front, (dw, t + 0.4, dw / 2), (0, -D / 2 + t / 2, 0.35 + dh - dw / 4))
    for k in range(11):
        a = math.pi * k / 10
        c = Vector((math.cos(a) * (dw / 2 + 0.15), -D / 2 - 0.04, 0.35 + dh - dw / 2 + math.sin(a) * (dw / 2 + 0.15)))
        block("door_voussoir", (0.2, 0.16, 0.34), m.stone, at=c, rot=(0, math.pi / 2 - a, 0), bevel=0.02, segments=2)
    # Døra står slått inn mot veggen, hengslet på venstre side.
    n = 6
    a = math.radians(95)
    dirv = Vector((math.cos(a), math.sin(a), 0))
    hinge = Vector((-dw / 2, -D / 2 + t, 0))
    for k in range(n):
        px = -dw / 2 + dw * (k + 0.5) / n
        top = 0.35 + dh - dw / 2 + math.sqrt(max(0.0, (dw / 2) ** 2 - px * px)) - 0.02
        block("door_plank", (dw / n - 0.008, 0.06, top - 0.35), m.boards, at=hinge + dirv * (dw * (k + 0.5) / n) + Vector((0, 0, 0.35 + (top - 0.35) / 2)), rot=(0, 0, a), bevel=0.005)
    for z in (0.8, 2.0):
        block("door_strap", (dw * 0.9, 0.03, 0.07), m.iron, at=hinge + dirv * (dw * 0.47) + Vector((0.045, 0, z)), rot=(0, 0, a), bevel=0.005)
    block("step", (dw + 0.8, 0.6, 0.2), m.stone, at=(0, -D / 2 - 0.3, 0.1), bevel=0.03, segments=2)
    block("step2", (dw + 1.2, 0.5, 0.15), m.stone, at=(0, -D / 2 - 0.65, -0.02), bevel=0.03, segments=2)
    # Høye rundbuede vinduer: to på fronten, to på hver langside.
    for x in (-3.0, 3.0):
        arched_window(m, front, x, -D / 2, -1.0, 1.4, 0.9, 2.6, t)
    for sx, side in zip((-1, 1), sides):
        for y in (-1.9, 1.9):
            arched_window(m, side, y, sx * W / 2, float(sx), 1.4, 0.8, 2.4, t, side=True)
    # Støttepilarer på hjørnene og midt på langsidene, trappet inn.
    for x, y, ax in [(-W / 2, -D / 2, 0), (W / 2, -D / 2, 0), (-W / 2, D / 2, 0), (W / 2, D / 2, 0), (-W / 2, 0, 1), (W / 2, 0, 1)]:
        for k, (hgt, depth) in enumerate([(2.2, 0.75), (1.6, 0.5), (1.0, 0.3)]):
            z0 = [0, 2.2, 3.8][k]
            if ax:
                block("buttress", (depth, 0.7, hgt), m.ashlar, at=(x + math.copysign(depth / 2, x), y, z0 + hgt / 2 - (0.24 if k == 0 else 0)), bevel=0.04, segments=2)
            else:
                block("buttress", (0.7, depth, hgt), m.ashlar, at=(x, y + math.copysign(depth / 2, y), z0 + hgt / 2 - (0.24 if k == 0 else 0)), bevel=0.04, segments=2)
    # Gesimsen under taket.
    cornice = block("cornice", (W + 0.3, D + 0.3, 0.2), m.stone, at=(0, 0, H + 0.05), bevel=0.03, segments=2)
    cut(cornice, (W - 2 * t, D - 2 * t, 0.6), (0, 0, H + 0.05))
    town.uv_box(cornice)
    # Himlingen inne: bord på bjelker langs salen.
    inner = inside(m)
    block("ceiling", (W - 2 * t, D - 2 * t, 0.05), inner.floor, at=(0, 0, H - 0.03), bevel=0)
    for k in range(13):
        x = -W / 2 + t + 0.2 + (W - 2 * t - 0.4) * k / 12
        beam("ceiling_joist", (x, -D / 2 + t, H - 0.13), (x, D / 2 - t, H - 0.13), 0.14, 0.18, m.timber)
    # Taket: bratt saltak med gavlen mot torget, gavler av hogd stein.
    rise = roof(m, W + 0.2, D, H + 0.1, 52, gable_to_street=True)
    from town import prism
    for sy in (-1, 1):
        prism("gable_stone", [(-W / 2, 0.0), (W / 2, 0.0), (0.0, rise)], t, m.ashlar, at=(0, sy * (D / 2 - t / 2), H + 0.1))
        # Et lite rundvindu i gavlen.
        g = revolve("oculus", [(0.0, -0.02), (0.35, -0.02), (0.35, 0.02), (0.0, 0.02)], m.glass, n=16)
        g.rotation_euler = (math.pi / 2, 0, 0)
        g.location = (0, sy * (D / 2 + 0.01), H + 0.1 + rise * 0.35)
        rim = revolve("oculus_rim", [(0.35, -0.08), (0.48, -0.08), (0.48, 0.08), (0.35, 0.08)], m.stone, n=16, cap=False)
        rim.rotation_euler = (math.pi / 2, 0, 0)
        rim.location = (0, sy * (D / 2 + 0.02), H + 0.1 + rise * 0.35)
    # Klokkestolen på mønet, foran: fire stolper, et lite tak og ei klokke.
    bz = H + 0.1 + rise
    for sx in (-1, 1):
        for sy in (-1, 1):
            beam("bell_post", (sx * 0.35, -D / 2 + 1.2 + sy * 0.35, bz - 0.2), (sx * 0.35, -D / 2 + 1.2 + sy * 0.35, bz + 1.1), 0.1, 0.1, m.timber)
    from walls import pyramid
    pyramid("bell_roof", 0.6, 0.6, bz + 1.1, bz + 1.8, m.roof)
    revolve("bell", [(0.0, bz + 0.85), (0.08, bz + 0.85), (0.14, bz + 0.6), (0.2, bz + 0.4), (0.0, bz + 0.42)], m.iron, n=16, at=(0, -D / 2 + 1.2, 0))
    revolve("finial", [(0.05, bz + 1.75), (0.03, bz + 2.2), (0.0, bz + 2.3)], m.iron, n=8)
    PIECES[-1].location = (0, -D / 2 + 1.2, 0)
    # Skiltet: en åpen bok i smijern på en arm ved døra.
    sx = 1.6
    beam("sign_arm", (sx, -D / 2 - 0.05, 3.3), (sx, -D / 2 - 1.0, 3.3), 0.05, 0.05, m.dark_iron)
    beam("sign_strut", (sx, -D / 2 - 0.05, 2.85), (sx, -D / 2 - 0.7, 3.28), 0.03, 0.03, m.dark_iron)
    beam("sign_chain", (sx, -D / 2 - 0.7, 3.28), (sx, -D / 2 - 0.7, 3.05), 0.015, 0.015, m.dark_iron)
    for s in (-1, 1):
        block("sign_page", (0.04, 0.32, 0.4), m.planks, at=(sx, -D / 2 - 0.7 + s * 0.17, 2.85), rot=(s * 0.25, 0, 0), bevel=0.01)
    block("sign_spine", (0.06, 0.04, 0.42), m.dark_iron, at=(sx, -D / 2 - 0.7, 2.83), bevel=0.01)
    # --- Inne: skrivesalen.
    rng = random.Random(51)
    z = 0.36
    ix, iy = W / 2 - t, D / 2 - t
    block("floor", (2 * ix, 2 * iy, 0.03), m.flags, at=(0, 0, z + 0.005), bevel=0)
    for y in (-1.6, 1.6):
        beam("tie_beam", (-ix, y, H - 0.2), (ix, y, H - 0.2), 0.25, 0.28, m.timber)
    # Bokhyllene langs bakveggen og mellom vinduene på langsidene.
    for x in (-2.6, -0.9, 0.9, 2.6):
        shelf(m, x, iy - 0.18, z, 1.6, 2.3, along_x=True, rows=5, rng=rng)
    for sx in (-1, 1):
        shelf(m, sx * (ix - 0.18), 0.0, z, 1.6, 2.3, along_x=False, rows=5, rng=rng)
    # Skrivepultene under vinduene: skrå pult, krakk, bok, blekkhorn og lys.
    for sx in (-1, 1):
        for y in (-1.9, 1.9):
            x = sx * (ix - 0.55)
            block("desk_top", (0.6, 0.9, 0.04), m.wood, at=(x, y, z + 0.95), rot=(0, sx * 0.3, 0), bevel=0.006)
            for k in (-1, 1):
                block("desk_side", (0.55, 0.05, 0.95), m.wood, at=(x, y + k * 0.42, z + 0.47), bevel=0.006)
            block("desk_shelf", (0.5, 0.85, 0.03), m.wood, at=(x, y, z + 0.35), bevel=0.004)
            books_row(m, y - 0.38, y + 0.3, x, z + 0.365, rng=rng, axis_x=False)
            open_book(m, x, y, z + 0.99, yaw=math.pi / 2)
            revolve("inkhorn", [(0.0, 0.0), (0.03, 0.0), (0.035, 0.08), (0.0, 0.08)], m.books[1], n=8, at=(x - sx * 0.2, y + 0.38, z + 1.05))
            beam("quill", (x - sx * 0.2, y + 0.38, z + 1.12), (x - sx * 0.25, y + 0.3, z + 1.35), 0.008, 0.02, m.vellum)
            holder(m, x - sx * 0.22, y - 0.38, z + 1.02)
            light((x - sx * 0.22, y - 0.38, z + 1.35), (1.0, 0.72, 0.42), 0.8, 3.5)
            sx2 = x - sx * 0.7
            revolve("stool", [(0.0, z + 0.45), (0.2, z + 0.45), (0.2, z + 0.5), (0.0, z + 0.5)], m.wood, n=12, at=(sx2, y, 0))
            for j in range(3):
                aa = j * 2.1
                beam("stool_leg", (sx2 + math.cos(aa) * 0.13, y + math.sin(aa) * 0.13, z + 0.45), (sx2 + math.cos(aa) * 0.18, y + math.sin(aa) * 0.18, z), 0.04, 0.04, m.timber)
            collide((x - sx * 0.25, y, z + 0.5), (1.2, 1.0, 1.0))
    # Lesepulten midt i salen med en stor bok i kjetting.
    beam("lectern_post", (0, 0.3, z), (0, 0.3, z + 1.05), 0.14, 0.14, m.timber, roll=math.pi / 2)
    for k in range(4):
        aa = k * math.pi / 2 + math.pi / 4
        beam("lectern_foot", (0, 0.3, z + 0.08), (math.cos(aa) * 0.35, 0.3 + math.sin(aa) * 0.35, z), 0.08, 0.08, m.timber)
    block("lectern_top", (0.7, 0.55, 0.05), m.wood, at=(0, 0.3, z + 1.12), rot=(0.35, 0, 0), bevel=0.01)
    open_book(m, 0, 0.28, z + 1.16, yaw=0.0)
    PIECES[-1].rotation_euler = (0.35, 0, 0)
    for ob in PIECES[-3:-1]:
        ob.rotation_euler = (0.35, ob.rotation_euler[1], 0)
    for k in range(6):
        beam("chain", (0.15, 0.1 + k * 0.03, z + 1.05 - k * 0.12), (0.15, 0.13 + k * 0.03, z + 0.93 - k * 0.12), 0.02, 0.02, m.dark_iron)
    collide((0, 0.3, z + 0.6), (0.8, 0.7, 1.2))
    # Kista med skjemaene, og høye lysestaker ved døra.
    from props import crate as _crate  # noqa: F401
    block("chest", (1.1, 0.6, 0.55), m.wood, at=(-ix + 0.8, -iy + 0.5, z + 0.28), bevel=0.02)
    for k in (-1, 0, 1):
        block("chest_band", (0.06, 0.62, 0.57), m.dark_iron, at=(-ix + 0.8 + k * 0.4, -iy + 0.5, z + 0.28), bevel=0.005)
    collide((-ix + 0.8, -iy + 0.5, z + 0.3), (1.2, 0.7, 0.6))
    for sx in (-1, 1):
        x, y = sx * 1.4, -iy + 0.45
        revolve("candlestand", [(0.0, z), (0.2, z), (0.15, z + 0.06), (0.03, z + 0.1), (0.03, z + 1.4), (0.1, z + 1.44), (0.0, z + 1.46)], m.dark_iron, n=10, at=(x, y, 0))
        candle(m, x, y, z + 1.46, h=0.3, r=0.04)
        light((x, y, z + 1.85), (1.0, 0.72, 0.42), 1.2, 6.0)
        collide((x, y, z + 0.75), (0.4, 0.4, 1.5))
    # Lysekroner under bindbjelkene.
    for y in (-1.6, 0.0):
        beam("chain", (0, y, H - 0.35), (0, y, H - 1.4), 0.015, 0.015, m.dark_iron)
        revolve("ring", [(0.5, H - 1.44), (0.54, H - 1.42), (0.5, H - 1.4), (0.46, H - 1.42)], m.dark_iron, n=24, at=(0, y, 0), cap=False)
        for k in range(6):
            aa = k * math.pi / 3
            candle(m, math.cos(aa) * 0.5, y + math.sin(aa) * 0.5, H - 1.4, h=0.14)
        light((0, y, H - 1.25), (1.0, 0.72, 0.42), 1.8, 8.0)
    library_gallery(m, back, ix, iy, z, D, t, rng)
    # Kolliderne: sokkelen (gulvet), murene med døra, støttepilarene og trinnene.
    collide((0, 0, 0.06), (W + 0.3, D + 0.3, 0.6))
    walls_with_door(-W / 2 + t / 2, W / 2 - t / 2, -D / 2 + t / 2, D / 2 - t / 2, -0.24, H, t, 0.0, dw, dh - 0.1, -D / 2 + t / 2)
    for x, y, ax in [(-W / 2, -D / 2, 0), (W / 2, -D / 2, 0), (-W / 2, D / 2, 0), (W / 2, D / 2, 0), (-W / 2, 0, 1), (W / 2, 0, 1)]:
        if ax:
            collide((x + math.copysign(0.38, x), y, 1.9), (0.76, 0.7, 4.2))
        else:
            collide((x, y + math.copysign(0.38, y), 1.9), (0.7, 0.76, 4.2))
    ramp(-dw / 2 - 0.5, dw / 2 + 0.5, -D / 2 - 0.95, -D / 2 - 0.12, 0.0, 0.36)
    collide((0, 0, H + 0.1), (W, D, 0.2))
    return m


def library_gallery(m, back, ix, iy, z, D, t, rng):
    """Biblioteksgalleriet i Skriptoriet: et loft langs bakveggen på
    stolper, med frittstående trapp, rekkverk, bokhyller, et høyt vindu i
    bakmuren, lesepult og ei kiste med skriftruller."""
    gx0, gx1, gy0, gy1 = -2.9, 2.9, 1.0, iy
    gz = z + 2.84
    # Gulvet på bjelker, med en kantbjelke foran og stolper med knekter.
    fl = block("gallery_floor", (gx1 - gx0, gy1 - gy0, 0.05), m.floor, at=((gx0 + gx1) / 2, (gy0 + gy1) / 2, gz - 0.025), bevel=0)
    town.uv_box(fl)
    for k in range(11):
        x = gx0 + 0.1 + (gx1 - gx0 - 0.2) * k / 10
        beam("gallery_joist", (x, gy0, gz - 0.15), (x, gy1, gz - 0.15), 0.14, 0.2, m.timber)
    beam("gallery_edge", (gx0 + 1.0, gy0 + 0.1, gz - 0.2), (gx1, gy0 + 0.1, gz - 0.2), 0.22, 0.3, m.timber)
    for x in (-1.6, 0.6, gx1 - 0.12):
        beam("gallery_post", (x, gy0 + 0.1, z), (x, gy0 + 0.1, gz - 0.35), 0.22, 0.22, m.timber, roll=math.pi / 2)
        block("post_base", (0.32, 0.32, 0.12), m.stone, at=(x, gy0 + 0.1, z + 0.06), bevel=0.02)
        for sx in (-1, 1):
            if gx0 < x + sx * 0.6 < gx1:
                beam("gallery_brace", (x, gy0 + 0.1, gz - 0.95), (x + sx * 0.55, gy0 + 0.1, gz - 0.38), 0.12, 0.12, m.timber)
        collide((x, gy0 + 0.1, z + 1.3), (0.3, 0.3, 2.6))
    wall_box(gx0, gy0, gx1, gy1, gz - 0.25, gz)
    # Trappa opp til vestenden, åpen på begge sider.
    stairs(m, gx0, gx0 + 0.95, gy0, z, gz, 13, 0.27, open_x=(gx0 - 0.02, gx0 + 0.97))
    # Rekkverket: foran fra trappa, på østenden og langs vestenden.
    rail_run(m, (gx0 + 1.0, gy0 + 0.05), (gx1, gy0 + 0.05), gz)
    rail_run(m, (gx1, gy0 + 0.05), (gx1, gy1), gz)
    rail_run(m, (gx0, gy0 + 0.05), (gx0, gy1), gz)
    # Et høyt rundbuet vindu i bakmuren over galleriet.
    arched_window(m, back, 0.0, D / 2, 1.0, gz + 0.9, 0.7, 1.7, t)
    # Bokhyllene langs bakmuren på hver side av vinduet.
    for x in (-1.9, 1.9):
        shelf(m, x, gy1 - 0.18, gz, 1.6, 2.2, along_x=True, rows=5, rng=rng)
    # Lesepulten foran vinduet, med krakk og lys.
    dx, dy = 0.0, gy1 - 0.75
    block("desk_top", (0.9, 0.55, 0.04), m.wood, at=(dx, dy, gz + 0.95), rot=(-0.3, 0, 0), bevel=0.006)
    for k in (-1, 1):
        block("desk_side", (0.05, 0.5, 0.95), m.wood, at=(dx + k * 0.42, dy, gz + 0.47), bevel=0.006)
    block("desk_shelf", (0.85, 0.45, 0.03), m.wood, at=(dx, dy, gz + 0.35), bevel=0.004)
    books_row(m, dx - 0.38, dx + 0.35, dy, gz + 0.365, rng=rng)
    open_book(m, dx, dy - 0.02, gz + 0.99)
    for ob in PIECES[-3:]:
        ob.rotation_euler = (-0.3, ob.rotation_euler[1], ob.rotation_euler[2])
    holder(m, dx + 0.35, dy - 0.35, gz)
    candle(m, dx + 0.35, dy - 0.35, gz + 0.14, h=0.25, r=0.035)
    light((dx + 0.35, dy - 0.35, gz + 0.6), (1.0, 0.72, 0.42), 1.0, 5.0)
    revolve("stool", [(0.0, gz + 0.45), (0.2, gz + 0.45), (0.2, gz + 0.5), (0.0, gz + 0.5)], m.wood, n=12, at=(dx, dy - 0.65, 0))
    for j in range(3):
        aa = j * 2.1
        beam("stool_leg", (dx + math.cos(aa) * 0.13, dy - 0.65 + math.sin(aa) * 0.13, gz + 0.45), (dx + math.cos(aa) * 0.18, dy - 0.65 + math.sin(aa) * 0.18, gz), 0.04, 0.04, m.timber)
    collide((dx, dy - 0.2, gz + 0.5), (1.0, 1.2, 1.0))
    # Kista med skriftruller ved østenden.
    cx, cy = 2.3, gy0 + 0.75
    block("scroll_chest", (0.9, 0.55, 0.45), m.wood, at=(cx, cy, gz + 0.225), bevel=0.015)
    for k in (-1, 1):
        block("chest_band", (0.05, 0.57, 0.47), m.dark_iron, at=(cx + k * 0.3, cy, gz + 0.225), bevel=0.004)
    for k in range(9):
        a = rng.uniform(-0.3, 0.3)
        sx_ = cx - 0.32 + (k % 5) * 0.16
        sy_ = cy - 0.12 + (k // 5) * 0.2
        ob = revolve("scroll", [(0.0, -0.22), (0.035, -0.22), (0.035, 0.22), (0.0, 0.22)], m.vellum, n=10, at=(0, 0, 0))
        ob.rotation_euler = (math.pi / 2 - 0.25, a, 0)
        ob.location = (sx_, sy_, gz + 0.5 + 0.1)
    collide((cx, cy, gz + 0.3), (0.95, 0.6, 0.6))


# --- Stavkirka --------------------------------------------------------------


def plank_walls(m, name, w, d, z0, z1, mat):
    """Fire vegger av stående planker rundt et rektangel, med svill og
    stavlegjer (bjelke øverst)."""
    t = 0.14
    walls = []
    for sy in (-1, 1):
        walls.append(block(f"{name}_wall", (w, t, z1 - z0), mat, at=(0, sy * (d / 2 - t / 2), (z0 + z1) / 2), bevel=0.01))
    for sx in (-1, 1):
        walls.append(block(f"{name}_wall", (t, d - 2 * t, z1 - z0), mat, at=(sx * (w / 2 - t / 2), 0, (z0 + z1) / 2), bevel=0.01))
    for z in (z0 + 0.1, z1 - 0.1):
        for sy in (-1, 1):
            beam(f"{name}_sill", (-w / 2 - 0.05, sy * (d / 2 + 0.02), z), (w / 2 + 0.05, sy * (d / 2 + 0.02), z), 0.12, 0.2, mat)
        for sx in (-1, 1):
            beam(f"{name}_sill", (sx * (w / 2 + 0.02), -d / 2 - 0.05, z), (sx * (w / 2 + 0.02), d / 2 + 0.05, z), 0.12, 0.2, mat)
    # Hjørnestavene.
    for sx in (-1, 1):
        for sy in (-1, 1):
            revolve(f"{name}_stave", [(0.16, z0), (0.16, z1)], mat, n=10, at=(sx * w / 2, sy * d / 2, 0))
    return walls  # foran (−y), bak, vest (−x), øst


def skirt_roof(m, name, w_in, d_in, w_out, d_out, z_in, z_out, mat):
    """Et lavt tak rundt et rektangel (svalgang, sideskip): fire skrå flater
    fra den indre veggen ned til den ytre kanten."""
    for sy in (-1, 1):
        run = (d_out - d_in) / 2
        tilt = math.atan2(z_in - z_out, run)
        ln = math.hypot(run, z_in - z_out)
        ob = block(f"{name}_roof", (w_out, ln, 0.12), mat, bevel=0.01)
        town._roof_uv(ob, along_y=False)
        ob.rotation_euler = (sy * tilt, 0, 0)
        ob.location = (0, sy * (d_in / 2 + run / 2), (z_in + z_out) / 2 + 0.08)
    for sx in (-1, 1):
        run = (w_out - w_in) / 2
        tilt = math.atan2(z_in - z_out, run)
        ln = math.hypot(run, z_in - z_out)
        ob = block(f"{name}_roof", (ln, d_in, 0.12), mat, bevel=0.01)
        town._roof_uv(ob, along_y=True)
        ob.rotation_euler = (0, -sx * tilt, 0)
        ob.location = (sx * (w_in / 2 + run / 2), 0, (z_in + z_out) / 2 + 0.08)


def dragon(m, x, y, z, out, mat):
    """Et drakehode på gavlspissen: en buet hals ut og opp, og et hode med
    gapende kjeft."""
    pts = [Vector((x, y, z))]
    for k in range(1, 5):
        t = k / 4
        pts.append(Vector((x, y + out * 0.9 * t, z + 0.55 * math.sin(t * math.pi * 0.7))))
    for a, b in zip(pts, pts[1:]):
        beam("dragon_neck", a, b, 0.12, 0.16, mat)
    head = pts[-1]
    block("dragon_head", (0.12, 0.38, 0.2), mat, at=head + Vector((0, out * 0.15, 0.05)), rot=(out * -0.35, 0, 0), bevel=0.03)
    block("dragon_jaw", (0.1, 0.32, 0.08), mat, at=head + Vector((0, out * 0.2, -0.1)), rot=(out * 0.3, 0, 0), bevel=0.02)
    for s in (-1, 1):
        beam("dragon_horn", head + Vector((s * 0.04, 0, 0.12)), head + Vector((s * 0.08, -out * 0.15, 0.4)), 0.04, 0.04, mat)


def church_inside(m, W, D, NW, ND, cy, ay, aisle, nave, choir, gw, gd):
    """Inne i Stavkirka: plankegulv, stavene rundt midtskipet med andreaskors
    og knær, glugger høyt oppe, korbuen med triumfkrusifiks, alteret i
    koret, døpefonten ved døra og benker langs veggene."""
    from walls import cutter
    inside(m)
    m.red = layer("wool", (0.55, 0.12, 0.08), 0.5)
    m.linen = layer("linen", (0.88, 0.85, 0.78), 0.5)
    m.gilt = layer("metal", (0.85, 0.62, 0.25), 0.5)
    z = 0.25
    fz = z + 0.03
    t = 0.14
    block("floor", (W - 2 * t, D - 2 * t, 0.03), m.floor, at=(0, 0, z + 0.015), bevel=0)
    # Stavene: åtte rundt midtskipet, fra gulvet til toppen av midtskipet.
    staves = [(sx * NW / 2, y) for sx in (-1, 1) for y in (-ND / 2, -ND / 6, ND / 6, ND / 2)]
    for x, y in staves:
        revolve("nave_stave", [(0.17, fz), (0.15, fz + 0.3), (0.15, 7.1)], m.tar, n=12, at=(x, y, 0))
        revolve("stave_base", [(0.24, fz), (0.24, fz + 0.12), (0.17, fz + 0.2), (0.0, fz + 0.2)], m.tar, n=12, at=(x, y, 0))
        collide((x, y, 2.0), (0.34, 0.34, 3.6))
    # Bindbjelker og andreaskors mellom stavene, og buede knær under.
    for sx in (-1, 1):
        x = sx * NW / 2
        beam("girt", (x, -ND / 2, 3.55), (x, ND / 2, 3.55), 0.16, 0.22, m.tar)
        for y0, y1 in ((-ND / 2, -ND / 6), (-ND / 6, ND / 6), (ND / 6, ND / 2)):
            beam("andrew", (x, y0, 3.7), (x, y1, 5.3), 0.1, 0.12, m.tar)
            beam("andrew", (x, y1, 3.7), (x, y0, 5.3), 0.1, 0.12, m.tar)
            for k in range(6):
                a0, a1 = math.pi * k / 6, math.pi * (k + 1) / 6
                r = (y1 - y0) / 2
                mid = (y0 + y1) / 2
                p0 = Vector((x, mid - math.cos(a0) * r, 3.45 - math.sin(a0) * 0.55))
                p1 = Vector((x, mid - math.cos(a1) * r, 3.45 - math.sin(a1) * 0.55))
                beam("arch", p0, p1, 0.12, 0.1, m.tar)
    for sy in (-1, 1):
        beam("girt", (-NW / 2, sy * ND / 2, 3.55), (NW / 2, sy * ND / 2, 3.55), 0.16, 0.22, m.tar)
    # Himlingen over sideskipet: bord på bjelker i ring rundt midtskipet,
    # under det skrå sideskipstaket.
    hz = 3.5
    for sy in (-1, 1):
        dd = D / 2 - t - ND / 2
        block("aisle_ceiling", (W - 2 * t, dd, 0.04), m.floor, at=(0, sy * (ND / 2 + dd / 2), hz), bevel=0)
    for sx in (-1, 1):
        ww = W / 2 - t - NW / 2
        block("aisle_ceiling", (ww, ND, 0.04), m.floor, at=(sx * (NW / 2 + ww / 2), 0, hz), bevel=0)
    for k in range(9):
        y = -D / 2 + t + 0.3 + (D - 2 * t - 0.6) * k / 8
        for sx in (-1, 1):
            if abs(y) < ND / 2 - 0.1:
                beam("aisle_joist", (sx * (W / 2 - t), y, hz - 0.1), (sx * NW / 2, y, hz - 0.1), 0.12, 0.16, m.tar)
            else:
                beam("aisle_joist", (sx * (W / 2 - t), y, hz - 0.1), (0, y, hz - 0.1), 0.12, 0.16, m.tar)
    # Bjelker over midtskipet, og sperrer synlige under taket.
    for y in (-ND / 2 + 0.3, 0.0, ND / 2 - 0.3):
        beam("tie", (-NW / 2, y, 7.0), (NW / 2, y, 7.0), 0.16, 0.2, m.tar)
    # Gluggene: små runde lysåpninger høyt oppe i midtskipet.
    for wall, sx in ((nave[2], -1), (nave[3], 1)):
        for y in (-ND / 4, ND / 4):
            x = sx * NW / 2
            bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.17, depth=0.6, location=(x, y, 6.1), rotation=(0, math.pi / 2, 0))
            cutter(bpy.context.active_object, wall)
            g = revolve("glugg", [(0.0, -0.01), (0.17, -0.01), (0.17, 0.01), (0.0, 0.01)], m.glass, n=16)
            g.rotation_euler = (0, math.pi / 2, 0)
            g.location = (x - sx * 0.02, y, 6.1)
            town.daylight(g, (-sx, 0, 0))
            ring = revolve("glugg_rim", [(0.17, -0.1), (0.24, -0.1), (0.24, 0.1), (0.17, 0.1)], m.tar, n=16, cap=False)
            ring.rotation_euler = (0, math.pi / 2, 0)
            ring.location = (x, y, 6.1)
        town.uv_box(wall)
    # Korbuen: åpning gjennom bakveggen inn i koret, og koret inn i apsiden.
    for wall in (aisle[1], choir[0]):
        cut(wall, (2.2, 0.6, 2.8), (0, wall.location.y, 0.25 + 1.4))
        town.uv_box(wall)
    cut(choir[1], (2.2, 0.6, 2.4), (0, cy + 1.4 - 0.07, 0.25 + 1.2))
    town.uv_box(choir[1])
    # Triumfbjelken med krusifikset over korbuen.
    by = ND / 2 - 0.2
    beam("rood_beam", (-NW / 2, by, 3.25), (NW / 2, by, 3.25), 0.2, 0.24, m.tar)
    cz = 3.4
    beam("cross_v", (0, by, cz), (0, by, cz + 1.5), 0.12, 0.08, m.wood)
    beam("cross_h", (-0.55, by, cz + 1.05), (0.55, by, cz + 1.05), 0.12, 0.08, m.wood)
    # Kristus: en enkel, skåret figur av tre, malt.
    body = layer("plaster", (0.75, 0.62, 0.5), 0.5)
    block("figure_torso", (0.2, 0.08, 0.42), body, at=(0, by - 0.07, cz + 0.82), bevel=0.04, segments=2)
    block("figure_cloth", (0.24, 0.09, 0.2), m.linen, at=(0, by - 0.075, cz + 0.55), bevel=0.04, segments=2)
    for sx in (-1, 1):
        beam("figure_arm", (sx * 0.1, by - 0.07, cz + 1.0), (sx * 0.5, by - 0.07, cz + 1.08), 0.06, 0.06, body)
        beam("figure_leg", (sx * 0.05, by - 0.07, cz + 0.48), (sx * 0.02, by - 0.07, cz + 0.12), 0.07, 0.07, body)
    revolve("figure_head", [(0.0, -0.08), (0.075, -0.06), (0.08, 0.02), (0.06, 0.08), (0.0, 0.1)], body, n=12, at=(0, by - 0.08, cz + 1.17))
    revolve("figure_crown", [(0.085, 0.0), (0.09, 0.04), (0.08, 0.06)], m.gilt, n=12, at=(0, by - 0.08, cz + 1.23), cap=False)
    # Himling i koret og under apsidetaket, så taket ikke står åpent inn.
    block("choir_ceiling", (3.4 - 2 * t, 2.8 - 2 * t, 0.04), m.floor, at=(0, cy, 3.36), bevel=0)
    for k in range(4):
        beam("choir_joist", (-1.7 + t, cy - 1.0 + k * 0.66, 3.28), (1.7 - t, cy - 1.0 + k * 0.66, 3.28), 0.1, 0.14, m.tar)
    dome = revolve("apse_ceiling", [(0.0, 2.92), (1.3, 2.92), (1.3, 2.96), (0.0, 2.96)], m.floor, n=16, at=(0, ay, 0))
    cut(dome, (3.0, 1.4, 1.0), (0, ay - 0.7, 2.94))
    # Koret: et trinn opp, alteret med duk og forside, lys og et lite kors.
    block("choir_floor", (3.4 - 2 * t, 2.8 - 2 * t, 0.18), m.floor, at=(0, cy, z + 0.09), bevel=0.01)
    block("choir_step", (2.2, 0.35, 0.09), m.floor, at=(0, cy - 1.4 - 0.1, z + 0.045), bevel=0.01)
    az = z + 0.18
    ax, a_y = 0.0, ay - 0.45
    block("altar", (1.3, 0.75, 0.95), m.stone, at=(ax, a_y, az + 0.475), bevel=0.03, segments=2)
    block("altar_cloth", (1.36, 0.8, 0.02), m.linen, at=(ax, a_y, az + 0.96), bevel=0.005)
    block("altar_frontal", (1.32, 0.02, 0.8), m.red, at=(ax, a_y - 0.39, az + 0.5), bevel=0.004)
    for k in range(3):
        block("frontal_band", (0.06, 0.025, 0.8), m.gilt, at=(ax - 0.45 + k * 0.45, a_y - 0.4, az + 0.5), bevel=0.003)
    for sx in (-1, 1):
        revolve("altar_candlestick", [(0.0, 0.0), (0.09, 0.0), (0.06, 0.03), (0.02, 0.05), (0.02, 0.3), (0.06, 0.32), (0.0, 0.33)], m.gilt, n=12, at=(ax + sx * 0.45, a_y, az + 0.97))
        candle(m, ax + sx * 0.45, a_y, az + 1.3, h=0.26, r=0.03)
        light((ax + sx * 0.45, a_y, az + 1.7), (1.0, 0.7, 0.4), 1.2, 5.0)
    beam("altar_cross_v", (ax, a_y + 0.2, az + 0.97), (ax, a_y + 0.2, az + 1.55), 0.04, 0.04, m.gilt)
    beam("altar_cross_h", (ax - 0.15, a_y + 0.2, az + 1.38), (ax + 0.15, a_y + 0.2, az + 1.38), 0.04, 0.04, m.gilt)
    collide((ax, a_y, az + 0.5), (1.4, 0.85, 1.0))
    # Døpefonten av kleberstein ved døra.
    fx, fy_ = 1.6, -D / 2 + 1.3
    revolve("font", [(0.0, fz), (0.3, fz), (0.22, fz + 0.12), (0.16, fz + 0.5), (0.36, fz + 0.72), (0.4, fz + 0.9), (0.34, fz + 0.9), (0.3, fz + 0.75), (0.0, fz + 0.7)], m.stone, n=20, at=(fx, fy_, 0))
    revolve("font_water", [(0.0, fz + 0.82), (0.3, fz + 0.82), (0.0, fz + 0.821)], m.glass, n=16, at=(fx, fy_, 0))
    collide((fx, fy_, fz + 0.45), (0.85, 0.85, 0.9))
    # Benkene langs veggene i sideskipet.
    for sx in (-1, 1):
        x = sx * (W / 2 - t - 0.22)
        block("wall_bench", (0.4, D - 1.6, 0.06), m.wood, at=(x, 0.3, fz + 0.45), bevel=0.01)
        for k in range(5):
            y = -D / 2 + 1.2 + k * (D - 2.0) / 4
            block("bench_leg", (0.36, 0.06, 0.42), m.timber, at=(x, y, fz + 0.21), bevel=0.01)
        collide((x, 0.3, fz + 0.24), (0.42, D - 1.6, 0.48))
    # Lysestaker av jern ved stavene og ved korbuen.
    for x, y in ((-NW / 2 + 0.45, -ND / 6), (NW / 2 - 0.45, ND / 6), (-0.95, D / 2 - 0.55), (0.95, D / 2 - 0.55)):
        revolve("candlestand", [(0.0, fz), (0.18, fz), (0.13, fz + 0.06), (0.025, fz + 0.1), (0.025, fz + 1.3), (0.09, fz + 1.34), (0.0, fz + 1.36)], m.dark_iron, n=10, at=(x, y, 0))
        candle(m, x, y, fz + 1.36, h=0.24, r=0.035)
        light((x, y, fz + 1.75), (1.0, 0.7, 0.4), 1.0, 5.5)
        collide((x, y, fz + 0.7), (0.35, 0.35, 1.4))
    # Kolliderne: fundamentet, veggene med døra og korbuen, koret og trinnene.
    collide((0, 0, 0.0), (W + 2.9, D + 2.9, 0.5))
    e = t
    wall_box(-W / 2, -D / 2, -0.55, -D / 2 + e, z, 3.6)
    wall_box(0.55, -D / 2, W / 2, -D / 2 + e, z, 3.6)
    wall_box(-0.55, -D / 2, 0.55, -D / 2 + e, z + 2.3, 3.6)
    wall_box(-W / 2, D / 2 - e, -1.1, D / 2, z, 3.6)
    wall_box(1.1, D / 2 - e, W / 2, D / 2, z, 3.6)
    for sx in (-1, 1):
        wall_box(sx * W / 2 - e / 2, -D / 2, sx * W / 2 + e / 2, D / 2, z, 3.6)
        wall_box(sx * 1.7 - e / 2, cy - 1.4, sx * 1.7 + e / 2, cy + 1.4, z, 3.4)
    wall_box(-1.7, cy + 1.4 - e, -1.1, cy + 1.4, z, 3.4)
    wall_box(1.1, cy + 1.4 - e, 1.7, cy + 1.4, z, 3.4)
    collide((0, ay + 0.6, 1.6), (2.6, 1.2, 2.8))
    wall_box(-1.7, cy - 1.4, 1.7, cy + 1.4, z, z + 0.18)
    # Svalgangens lave vegger, med åpning foran portalen.
    for sx in (-1, 1):
        wall_box(sx * gw / 2 - 0.05, -gd / 2, sx * gw / 2 + 0.05, gd / 2, z, 1.15)
    wall_box(-gw / 2, gd / 2 - 0.05, -1.75, gd / 2 + 0.05, z, 1.15)
    wall_box(1.75, gd / 2 - 0.05, gw / 2, gd / 2 + 0.05, z, 1.15)
    wall_box(-gw / 2, -gd / 2 - 0.05, -0.9, -gd / 2 + 0.05, z, 1.15)
    wall_box(0.9, -gd / 2 - 0.05, gw / 2, -gd / 2 + 0.05, z, 1.15)
    ramp(-0.9, 0.9, -gd / 2 - 1.1, -(D + 2.9) / 2 + 0.05, 0.0, z)


def stave_church():
    m = mats(roof="tiles")
    m.tar = layer("timber", (0.85, 0.85, 0.85), 0.5)
    m.shingle = layer("tiles", (0.42, 0.36, 0.3), 0.5)
    W, D = 6.0, 8.6  # sideskipet
    # Steinfundamentet.
    block("plinth", (W + 2.9, D + 2.9, 0.5), m.stone, at=(0, 0, 0.0), bevel=0.04, segments=2)
    # Svalgangen: lav plankevegg med åpne buer mellom stolpene, og tak.
    gw, gd = W + 2.6, D + 2.6
    for sy in (-1, 1):
        # Bak går koret gjennom svalgangen: der står ingen stolper eller vegg.
        gap = 0.9 if sy < 0 else 1.75
        for k in range(9):
            x = -gw / 2 + gw * k / 8
            if sy > 0 and abs(x) < gap + 0.1:
                continue
            beam("gallery_post", (x, sy * gd / 2, 0.25), (x, sy * gd / 2, 2.0), 0.12, 0.12, m.tar, roll=math.pi / 2)
        for x0, x1 in ((-gw / 2, -gap), (gap, gw / 2)):
            block("gallery_wall", (x1 - x0, 0.08, 0.9), m.tar, at=((x0 + x1) / 2, sy * gd / 2, 0.7), bevel=0.01)
            if sy > 0:
                beam("gallery_plate", (x0, sy * gd / 2, 2.0), (x1, sy * gd / 2, 2.0), 0.14, 0.14, m.tar)
        if sy < 0:
            beam("gallery_plate", (-gw / 2, sy * gd / 2, 2.0), (gw / 2, sy * gd / 2, 2.0), 0.14, 0.14, m.tar)
    for sx in (-1, 1):
        for k in range(11):
            y = -gd / 2 + gd * k / 10
            beam("gallery_post", (sx * gw / 2, y, 0.25), (sx * gw / 2, y, 2.0), 0.12, 0.12, m.tar, roll=math.pi / 2)
        block("gallery_wall", (0.08, gd, 0.9), m.tar, at=(sx * gw / 2, 0, 0.7), bevel=0.01)
        beam("gallery_plate", (sx * gw / 2, -gd / 2, 2.0), (sx * gw / 2, gd / 2, 2.0), 0.14, 0.14, m.tar)
    before = len(PIECES)
    skirt_roof(m, "gallery", W, D, gw + 0.5, gd + 0.5, 3.0, 2.0, m.shingle)
    for ob in PIECES[before:]:
        if ob.location.y > D / 2:
            cut(ob, (3.5, 3.6, 3.0), (0, D / 2 + 1.5, 2.5))
            town._roof_uv(ob, along_y=False)
    # Sideskipet og taket over det.
    aisle = plank_walls(m, "aisle", W, D, 0.25, 3.6, m.tar)
    NW, ND = 3.8, 6.6
    skirt_roof(m, "aisle", NW, ND, W + 0.5, D + 0.5, 4.8, 3.55, m.shingle)
    # Midtskipet: høye vegger over sideskipet, bratt tak med drakehoder.
    nave = plank_walls(m, "nave", NW, ND, 3.6, 7.2, m.tar)
    before = len(PIECES)
    rise = roof(m, NW + 0.1, ND, 7.2, 62, overhang=0.35, gable_to_street=True)
    for ob in PIECES[before:]:
        if ob.name.startswith("roof"):
            ob.data.materials[0] = m.shingle
    for sy, ny in ((-1, -1.0), (1, 1.0)):
        gable(m, NW, 7.2, rise, sy * ND / 2, ny, random.Random(7), boards=True)
        dragon(m, 0.0, sy * (ND / 2 + 0.35), 7.2 + rise + 0.15, sy, m.tar)
    # Takrytteren midt på mønet: et lite tårn med svalganger og et spir.
    tz = 7.2 + rise - 0.6
    block("turret", (1.3, 1.3, 1.8), m.tar, at=(0, 0, tz + 0.9), bevel=0.02)
    for sy in (-1, 1):
        for k in range(3):
            block("turret_arch", (0.25, 0.04, 0.5), m.glass, at=(-0.4 + k * 0.4, sy * 0.66, tz + 1.3), bevel=0)
    from walls import pyramid
    pyramid("turret_roof", 0.9, 0.9, tz + 1.8, tz + 2.3, m.shingle)
    spire = revolve("spire", [(0.62, tz + 2.2), (0.0, tz + 5.0)], m.shingle, n=8, cap=False, uv="planar")
    spire.rotation_euler = (0, 0, math.pi / 8)
    revolve("spire_top", [(0.05, tz + 4.9), (0.04, tz + 5.6), (0.0, tz + 5.7)], m.iron, n=8)
    # Koret og apsiden bak.
    cy = D / 2 + 1.4
    before = len(PIECES)
    choir = plank_walls(m, "choir", 3.4, 2.8, 0.25, 3.4, m.tar)
    rr = roof(m, 3.5, 2.8, 3.4, 58, overhang=0.3, gable_to_street=True)
    for ob in PIECES[before:]:
        if ob.name.startswith("roof"):
            ob.data.materials[0] = m.shingle
    moved(before, Matrix.Translation((0, cy, 0)))
    ay = cy + 1.4
    apse = revolve("apse", [(1.3, 0.25), (1.3, 3.0)], m.tar, n=16, cap=False)
    apse.location = (0, ay, 0)
    # Halvdelen av apsiden som ville stått inne i koret skjæres bort.
    cut(apse, (3.0, 1.4, 4.0), (0, ay - 0.7, 1.6))
    revolve("apse_roof", [(1.6, 2.9), (0.0, 4.6)], m.shingle, n=16, cap=False, uv="planar").location = (0, ay, 0)
    revolve("apse_tower", [(0.35, 4.3), (0.35, 5.2), (0.0, 6.4)], m.shingle, n=8, uv="planar").location = (0, ay, 0)
    # Portalen: rundbuet dør i vestgavlen, med søyler og en buet karm.
    py = -D / 2 - 0.08
    n = 5
    # Døra står slått inn mot veggen, hengslet på venstre side.
    a = math.radians(100)
    dirv = Vector((math.cos(a), math.sin(a), 0))
    hinge = Vector((-0.55, -D / 2 + 0.16, 0))
    for k in range(n):
        px = -0.55 + 1.1 * (k + 0.5) / n
        top = 0.3 + 2.0 + math.sqrt(max(0.0, 0.55 ** 2 - px * px)) * 0.9
        block("portal_plank", (1.1 / n - 0.01, 0.06, top - 0.3), m.boards, at=hinge + dirv * (1.1 * (k + 0.5) / n) + Vector((0, 0, 0.3 + (top - 0.3) / 2)), rot=(0, 0, a), bevel=0.005)
    for z in (0.8, 1.8):
        block("door_strap", (1.0, 0.03, 0.06), m.iron, at=hinge + dirv * 0.5 + Vector((0.04, 0, z)), rot=(0, 0, a), bevel=0.005)
    # Døråpningen gjennom plankeveggen, med rund bue.
    cut(aisle[0], (1.1, 0.6, 2.05), (0, -D / 2 + 0.07, 0.25 + 1.025))
    from walls import cutter
    bpy.ops.mesh.primitive_cylinder_add(vertices=20, radius=0.55, depth=0.6, location=(0, -D / 2 + 0.07, 2.3), rotation=(math.pi / 2, 0, 0))
    cutter(bpy.context.active_object, aisle[0])
    town.uv_box(aisle[0])
    church_inside(m, W, D, NW, ND, cy, ay, aisle, nave, choir, gw, gd)
    for sx in (-1, 1):
        revolve("portal_column", [(0.1, 0.25), (0.08, 0.4), (0.08, 2.2), (0.12, 2.35), (0.0, 2.35)], m.tar, n=10, at=(sx * 0.75, py - 0.08, 0))
        for k in range(6):
            block("carving", (0.1, 0.05, 0.12), m.tar, at=(sx * 0.95, py - 0.06, 0.6 + k * 0.3), rot=(0, sx * 0.6 * (1 if k % 2 else -1), 0), bevel=0.02)
    for k in range(9):
        a = math.pi * k / 8
        c = Vector((math.cos(a) * 0.72, py - 0.06, 2.3 + math.sin(a) * 0.6))
        block("portal_arch", (0.18, 0.1, 0.22), m.tar, at=c, rot=(0, math.pi / 2 - a, 0), bevel=0.02)
    # Trinnet opp til svalgangen foran portalen.
    block("step", (1.8, 0.8, 0.25), m.stone, at=(0, -gd / 2 - 0.5, 0.0), bevel=0.03, segments=2)
    return m


def gravestone():
    m = mats()
    m.moss = layer("sod", (0.6, 0.7, 0.55), 0.5)
    # Hella med rundet topp, litt skjev, og et kors hogd ut i relieff.
    block("slab", (0.55, 0.14, 0.75), m.stone, at=(0, 0, 0.25), rot=(0.04, 0.03, 0), bevel=0.03, segments=2)
    top = revolve("slab_top", [(0.0, -0.07), (0.275, -0.07), (0.275, 0.07), (0.0, 0.07)], m.stone, n=16)
    top.rotation_euler = (math.pi / 2 + 0.04, 0, 0)
    top.location = (0, 0.03, 0.62)
    block("cross_v", (0.06, 0.03, 0.42), m.stone, at=(0, -0.08, 0.5), rot=(0.04, 0, 0), bevel=0.01)
    block("cross_h", (0.26, 0.03, 0.06), m.stone, at=(0, -0.08, 0.58), rot=(0.04, 0, 0), bevel=0.01)
    # Mose ved foten og en liten haug foran.
    block("moss", (0.6, 0.2, 0.1), m.moss, at=(0, 0, -0.07), bevel=0.04, segments=2)
    block("mound", (0.7, 1.6, 0.18), m.moss, at=(0, -0.95, -0.05), bevel=0.08, segments=3)
    return m


MODELS = {"forge": forge, "tavern": tavern, "apothecary": apothecary, "scriptorium": scriptorium, "stave_church": stave_church, "gravestone": gravestone}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb", max_edge=1.0 if name in ("tavern", "scriptorium", "apothecary") else 0.6)
