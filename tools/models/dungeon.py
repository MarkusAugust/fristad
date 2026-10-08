"""Kjelleren under borgen som modulsett, modellert i Blender (bpy) og
eksportert til assets/models/dungeon/. Kjør:

    python3 tools/models/dungeon.py [dng_hall_i dng_start ...]

Delene (se docs/kjelleren.md):

- Ganger på én celle (4 × 4 m): `dng_hall_i` (rett, med gurtbue), `_i2`
  (med fakkel), `_i3` (delvis rast), `_l` (hjørne), `_t`, `_x` (kryss med
  krysshvelv og diagonalribber) og `_end`. Gangen er 2,8 m bred med
  tønnehvelv; massen rundt er fjell.
- `dng_seal`: en tilmurt dørbue (dører i et rom som ingen gang går til).
- Rom: `dng_start` (trappa opp til krypten), `dng_gate` (portgitteret foran
  vindeltrappa ned), `dng_landing` (foten av vindeltrappa), `dng_hall`
  (søylehall med krysshvelv), `dng_long` (lang hall med sarkofager),
  `dng_cistern` (cisterne med vann og søyler), `dng_gallery` (forhøyning
  med trapp opp), `dng_chapel` (kapell med apsis og alter), `dng_treasury`
  (skattkammer) og `dng_arena` (rotunde der Rammeverket venter).
- `dng_spiral`: vindeltrappa som går 12 m ned til neste etasje.

Alt bygges i motorens akser (x øst, y opp, z sør) og gjøres om til
Blenders med `B`. Origo er hjørnet av fotavtrykket (minste x og z) i
gulvhøyde; vindeltrappa har origo midt i tårncella. Ved siden av hver
modell skrives `<navn>.json` med fotavtrykket i celler, gulv- og
takhøyde og halv gangbredde per celle (til fiendene og takprøven),
ramper, dører (celle og retning ut), stengte kanter, kollidere, lys og
steder (fiender, kister, eliten, start, port, trappa opp, tårnet)."""

import json
import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import town  # noqa: E402
from town import PIECES, block, layer, plain  # noqa: E402
from walls import link, revolve  # noqa: E402

OUT = "assets/models/dungeon"

CELL = 4.0
# Murtykkelsen ut mot fjellet, halv gangbredde, og gangens hvelv.
T = 0.6
HW = 1.4
SPRING = 2.4
# Vindeltrappa: radius i sjakta, høyden ned og antall trinn.
RS = 1.75
DROP = 12.0
STEPS = 72

# --- Motorens akser ---------------------------------------------------------

# Motoren (x, y, z) er Blender (x, −z, y).
C = Matrix(((1, 0, 0), (0, 0, -1), (0, 1, 0)))


def B(p):
    return C @ Vector(p)


def orient(ob, r):
    """Drei en del med en rotasjon i motorens rom."""
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = (C @ r @ C.transposed()).to_quaternion()


def yaw_m(a):
    return Matrix.Rotation(a, 3, "Y")


def eblock(name, size, mat, at, r=None, bevel=0.012, segments=1):
    """En avfaset kloss med mål (x, y, z) i motoren, sentrert i `at`."""
    sx, sy, sz = size
    ob = block(name, (sx, sz, sy), mat, at=B(at), bevel=bevel, segments=segments)
    if r is not None:
        orient(ob, r)
    return ob


def basis(x, y):
    """Rotasjonen der lokal x peker langs `x` og lokal y så nær `y` som mulig."""
    x = Vector(x).normalized()
    z = x.cross(Vector(y)).normalized()
    y = z.cross(x).normalized()
    return Matrix((x, y, z)).transposed()


def ebeam(name, a, b, w, h, mat, up=(0, 1, 0)):
    """En bjelke fra a til b (motorens akser), w bred og h høy."""
    a, b = Vector(a), Vector(b)
    d = b - a
    up = Vector(up)
    if abs(d.normalized().dot(up)) > 0.95:
        up = Vector((1, 0, 0))
    return eblock(name, (d.length, h, w), mat, at=(a + b) / 2, r=basis(d, up), bevel=min(w, h) * 0.2)


def erevolve(name, profile, mat, at, n=16, cap=True):
    """Et omdreiningslegeme om loddlinja gjennom `at` (profil [(r, y)])."""
    return revolve(name, profile, mat, n=n, at=B(at), cap=cap)


# --- Registrene til json-fila -----------------------------------------------

DATA = {}
# Flater på gulvet som er opptatt utenom kolliderne (fakler, trapper), til
# stedene for pynt.
BUSY = []


def reset(w, d):
    DATA.clear()
    BUSY.clear()
    DATA.update(
        w=w,
        d=d,
        floor=[[0.0] * w for _ in range(d)],
        ceil=[[3.8] * w for _ in range(d)],
        hw=[[2.0] * w for _ in range(d)],
        ramps=[],
        doors=[],
        blocked=[],
        boxes=[],
        hulls=[],
        lights=[],
        spots={},
    )


def collide(x0, x1, y0, y1, z0, z1, yaw=0.0):
    DATA["boxes"].append([[round((x0 + x1) / 2, 3), round((y0 + y1) / 2, 3), round((z0 + z1) / 2, 3)], [round(x1 - x0, 3), round(y1 - y0, 3), round(z1 - z0, 3)], yaw])


def collide_at(c, size, yaw=0.0):
    DATA["boxes"].append([[round(v, 3) for v in c], [round(v, 3) for v in size], round(yaw, 4)])


def hull(points):
    DATA["hulls"].append([[round(v, 3) for v in p] for p in points])


def light(p, color=(1.0, 0.55, 0.22), intensity=2.3, radius=9.0):
    DATA["lights"].append([[round(v, 3) for v in p], list(color), intensity, radius])


def spot(kind, value):
    if kind in ("spawn", "chest", "loot", "wall", "floor"):
        DATA["spots"].setdefault(kind, []).append([round(v, 3) for v in value])
        # Foran kista skal det være fritt (pynten holder seg unna).
        if kind == "chest":
            BUSY.append((value[0] - 1.0, value[0] + 1.0, value[2] - 1.0, value[2] + 1.0))
    else:
        DATA["spots"][kind] = [round(v, 3) if isinstance(v, float) else v for v in value]


def write_json(path):
    with open(path.replace(".glb", ".json"), "w") as f:
        json.dump(DATA, f)
    print(f"  {len(DATA['boxes'])} kollidere, {len(DATA['hulls'])} skall, {len(DATA['lights'])} lys, {len(DATA['doors'])} dører")


# --- Materialene ------------------------------------------------------------


class Style:
    """Materialene i kjelleren: mur av bruddstein, hvelv og buer av hogd
    stein, steinheller i gulvet, smijern og eik."""

    def __init__(self):
        self.wall = layer("stone", (0.62, 0.59, 0.54), 0.5)
        self.vault = layer("brick", (0.6, 0.56, 0.5), 0.5)
        self.ashlar = layer("concrete", (0.86, 0.8, 0.72), 0.5)
        self.dark = layer("stone", (0.42, 0.4, 0.37), 0.5)
        # Gulvet er bygd med «tiles» som rolle (motoren gir hver etasje sitt lag).
        self.floor = layer("tiles", (0.62, 0.6, 0.56), 0.5)
        self.iron = layer("metal", (0.34, 0.31, 0.29), 0.5)
        self.gold = layer("metal", (1.0, 0.72, 0.3), 0.3)
        self.wood = layer("oak", (0.95, 0.85, 0.72), 0.5)
        self.rope = layer("straw", (0.7, 0.6, 0.45), 0.5)
        self.cloth = layer("linen", (0.55, 0.12, 0.1), 0.5)
        self.bone = layer("plaster", (0.86, 0.8, 0.66), 0.5)
        self.wax = layer("plaster", (0.95, 0.9, 0.75), 0.5)
        self.moss = layer("sod", (0.5, 0.6, 0.42), 0.5)
        self.flame = plain("fire_flame", (1.0, 0.62, 0.26), 0.4)
        self.ember = plain("fire_ember", (1.0, 0.32, 0.06), 0.6)
        self.water = plain("water", (0.04, 0.07, 0.08), 0.05)


# --- Hogd fjell: masse med hulrom ------------------------------------------

CUTS = []


def mass(x0, x1, y0, y1, z0, z1, s):
    """En blokk av fjell som rommet hogges ut av: murflatene får kryptmur,
    flatene som vender ned (hvelvet) hogd stein."""
    ob = eblock("mass", (x1 - x0, y1 - y0, z1 - z0), s.wall, at=((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2), bevel=0)
    ob.data.materials.append(s.vault)
    return ob


def cbox(x0, x1, y0, y1, z0, z1):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=B(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2)))
    ob = bpy.context.active_object
    ob.scale = (x1 - x0, z1 - z0, y1 - y0)
    CUTS.append(ob)
    return ob


def cbarrel(axis, a0, a1, c, spring, r, verts=36):
    """Et tønnehvelv langs motorens x eller z fra a0 til a1, med midtlinja i
    `c` (z for x-hvelv, x for z-hvelv) og vederlaget i `spring`."""
    if axis == "x":
        loc, rot = B(((a0 + a1) / 2, spring, c)), (0, math.pi / 2, 0)
    else:
        loc, rot = B((c, spring, (a0 + a1) / 2)), (math.pi / 2, 0, 0)
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=r, depth=a1 - a0, location=loc, rotation=rot)
    ob = bpy.context.active_object
    CUTS.append(ob)
    return ob


def ccyl(x, z, y0, y1, r, verts=48):
    """En loddrett sylinder (rotunde, apsis, trappesjakt)."""
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=r, depth=y1 - y0, location=B((x, (y0 + y1) / 2, z)))
    ob = bpy.context.active_object
    CUTS.append(ob)
    return ob


def csphere(x, y, z, r, segs=32):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segs, ring_count=segs // 2, radius=r, location=B((x, y, z)))
    ob = bpy.context.active_object
    CUTS.append(ob)
    return ob


def carve(target):
    """Skjær alle `CUTS` ut av `target` på én gang, og gi flatene som vender
    ned (hvelvet) det andre materialet."""
    if not CUTS:
        return
    coll = bpy.data.collections.new("cutters")
    bpy.context.scene.collection.children.link(coll)
    for c in CUTS:
        for u in list(c.users_collection):
            u.objects.unlink(c)
        coll.objects.link(c)
    names = {lay.name for lay in target.data.uv_layers}
    box = [Vector(v) for v in target.bound_box]
    lo = Vector((min(v.x for v in box), min(v.y for v in box), min(v.z for v in box)))
    hi = Vector((max(v.x for v in box), max(v.y for v in box), max(v.z for v in box)))
    mod = target.modifiers.new("carve", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.operand_type = "COLLECTION"
    mod.collection = coll
    mod.solver = "EXACT"
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.modifier_apply(modifier=mod.name)
    for c in CUTS:
        bpy.data.objects.remove(c)
    bpy.data.collections.remove(coll)
    CUTS.clear()
    town.keep_uv(target, names)
    # Yttersidene av blokken vender ut i fjellet og synes aldri, men der
    # hvelvet og gangene går gjennom dem, trianguleres hullet feil og gir
    # takkete halvsirkler midt i gangen der to celler møtes. De fjernes.
    bm = bmesh.new()
    bm.from_mesh(target.data)
    eps = 1e-3
    outer = [f for f in bm.faces if any(all(abs(v.co[k] - b[k]) < eps for v in f.verts) for k in range(3) for b in (lo, hi))]
    bmesh.ops.delete(bm, geom=outer, context="FACES")
    bm.to_mesh(target.data)
    bm.free()
    # Uv i meter, og hvelvet (flater som vender ned) får hogd stein.
    rot = target.matrix_world.to_3x3()
    for p in target.data.polygons:
        n = rot @ p.normal
        p.material_index = 1 if n.z < -0.2 else 0
    town.uv_box(target)


# --- Bygningsdeler ------------------------------------------------------------


def slabs(rng, rects, s, size=(0.45, 0.85), y=0.0):
    """Steinheller i rader over rektanglene (x0, x1, z0, z1), med fuger og
    små skjevheter."""
    for x0, x1, z0, z1 in rects:
        z = z0
        while z < z1 - 0.05:
            d = rng.uniform(*size)
            if z1 - (z + d) < size[0] * 0.6:
                d = z1 - z
            x = x0
            while x < x1 - 0.05:
                w = rng.uniform(size[0], size[1] * 1.2)
                if x1 - (x + w) < size[0] * 0.6:
                    w = x1 - x
                tilt = Matrix.Rotation(rng.uniform(-0.006, 0.006), 3, "X") @ Matrix.Rotation(rng.uniform(-0.006, 0.006), 3, "Z")
                eblock("slab", (w - 0.016, 0.14, d - 0.016), s.floor, at=(x + w / 2, y - 0.07 + rng.uniform(-0.008, 0.004), z + d / 2), r=tilt, bevel=0.022)
                x += w
            z += d


def plinth(rng, a, b, n, s, h=0.32, out=0.07):
    """Et skift av store sokkelsteiner langs muren fra a til b (x, z) med
    innovernormalen n."""
    a, b, n = Vector((a[0], 0, a[1])), Vector((b[0], 0, b[1])), Vector((n[0], 0, n[1]))
    d = b - a
    length = d.length
    along = d.normalized()
    yaw = math.atan2(-along.z, along.x)
    t = 0.0
    while t < length - 0.05:
        w = rng.uniform(0.5, 0.95)
        if length - (t + w) < 0.35:
            w = length - t
        c = a + along * (t + w / 2) + n * (out / 2 - 0.03)
        eblock("plinth", (w - 0.012, h + rng.uniform(-0.02, 0.02), out + 0.06), s.ashlar, at=(c.x, h / 2, c.z), r=yaw_m(yaw), bevel=0.025)
        t += w


def arch_rib(c, axis, r, spring, s, width=0.36, depth=0.17, n=11, mat=None):
    """En gurtbue av buestein under et hvelv med radius r: halvsirkelen står
    i planet vinkelrett på `axis` (vannrett), med midten i c (x, z)."""
    axis = Vector((axis[0], 0, axis[1])).normalized()
    side = Vector((0, 1, 0)).cross(axis).normalized()
    up = Vector((0, 1, 0))
    centre = Vector((c[0], spring, c[1]))
    rr = r - depth / 2 + 0.02
    for k in range(n):
        a = math.pi * (k + 0.5) / n
        chord = 2 * (r + 0.02) * math.sin(math.pi / n / 2)
        p = centre + side * math.cos(a) * rr + up * math.sin(a) * rr
        inward = -(side * math.cos(a) + up * math.sin(a))
        tangent = -side * math.sin(a) + up * math.cos(a)
        eblock("voussoir", (chord - 0.012, depth, width), mat or s.ashlar, at=p, r=basis(tangent, inward), bevel=0.02)


def archivolt(c, normal, r, spring, s, depth=0.38, out=0.07, n=13):
    """Buestein rundt en døråpning i muren: halvsirkel med indre radius r i
    murflaten (c er midten av åpningen i flaten, normal inn i rommet)."""
    normal = Vector((normal[0], 0, normal[1])).normalized()
    side = Vector((0, 1, 0)).cross(normal).normalized()
    up = Vector((0, 1, 0))
    centre = Vector((c[0], spring, c[1])) + normal * (out / 2 - 0.02)
    rr = r + depth / 2
    for k in range(n):
        a = math.pi * (k + 0.5) / n
        chord = 2 * (r + depth) * math.sin(math.pi / n / 2)
        p = centre + side * math.cos(a) * rr + up * math.sin(a) * rr
        radial = side * math.cos(a) + up * math.sin(a)
        tangent = -side * math.sin(a) + up * math.cos(a)
        eblock("archivolt", (chord - 0.012, depth, out + 0.04), s.ashlar, at=p, r=basis(tangent, radial), bevel=0.02)
    # Vangene ned til gulvet.
    for sgn in (-1, 1):
        y = 0.0
        k = 0
        while y < spring - 0.05:
            h = min(0.42 + 0.06 * (k % 2), spring - y)
            w = depth + (0.08 if k % 2 == 0 else -0.04)
            p = centre + side * sgn * (r + w / 2)
            eblock("jamb", (w, h - 0.012, out + 0.04), s.ashlar, at=(p.x, y + h / 2, p.z), r=yaw_m(math.atan2(-side.z, side.x)), bevel=0.02)
            y += h
            k += 1


def pier(x, z, spring, s, w=0.46, cap=True):
    """En pilaster av kvaderstein (stablede blokker) med kapitél."""
    collide(x - w / 2 - 0.02, x + w / 2 + 0.02, 0, spring, z - w / 2 - 0.02, z + w / 2 + 0.02)
    y = 0.0
    k = 0
    while y < spring - 0.2:
        h = min(0.38 + 0.08 * (k % 2), spring - 0.16 - y)
        ww = w + (0.04 if k % 2 == 0 else 0.0)
        eblock("pier", (ww, h - 0.012, ww), s.ashlar, at=(x, y + h / 2, z), bevel=0.02)
        y += h
        k += 1
    if cap:
        eblock("impost", (w + 0.16, 0.16, w + 0.16), s.ashlar, at=(x, spring - 0.08, z), bevel=0.03)


def diagonal_rib(c, bx, bz, spring, s, width=0.24, depth=0.15, n=10):
    """Diagonalribbene i et krysshvelv over en bås med midten c og målene
    bx × bz: fra hjørne til hjørne, under gratlinja."""
    r = min(bx, bz) / 2
    for sgn in (1, -1):
        pts = []
        for k in range(n + 1):
            t = -1 + 2 * k / n
            dx, dz = t * bx / 2, sgn * t * bz / 2
            y = spring + math.sqrt(max(0.0, 1 - t * t)) * r
            pts.append(Vector((c[0] + dx, y, c[1] + dz)))
        horizontal = Vector((bx / 2, 0, sgn * bz / 2)).normalized()
        for p, q in zip(pts, pts[1:]):
            d = q - p
            # Ribba henger under flaten: forskjøvet langs normalen i
            # diagonalplanet, nedover.
            nrm = d.cross(horizontal.cross(Vector((0, 1, 0)))).normalized()
            if nrm.y > 0:
                nrm = -nrm
            mid = (p + q) / 2 + nrm * (depth / 2 - 0.02)
            eblock("rib", (d.length + 0.02, depth, width), s.ashlar, at=mid, r=basis(d, -nrm), bevel=0.02)


def column(x, z, y0, spring, s, r=0.26):
    """En søyle med base, skaft og terningkapitél."""
    eblock("col_plinth", (0.74, 0.18, 0.74), s.ashlar, at=(x, y0 + 0.09, z), bevel=0.03)
    erevolve("col_base", [(0.36, y0 + 0.18), (0.36, y0 + 0.24), (0.3, y0 + 0.32), (r + 0.02, y0 + 0.36)], s.ashlar, at=(x, 0, z), n=20)
    top = spring - 0.5
    erevolve("col_shaft", [(r + 0.02, y0 + 0.36), (r, y0 + 0.5), (r * 0.94, top - 0.06), (r + 0.03, top)], s.ashlar, at=(x, 0, z), n=20)
    # Terningkapitélet: en kloss med rundet underside, og abakus over.
    erevolve("col_neck", [(r + 0.03, top), (r + 0.06, top + 0.06)], s.ashlar, at=(x, 0, z), n=20, cap=False)
    eblock("col_cap", (0.66, 0.32, 0.66), s.ashlar, at=(x, top + 0.06 + 0.16, z), bevel=0.1, segments=3)
    eblock("col_abacus", (0.8, 0.12, 0.8), s.ashlar, at=(x, spring - 0.06, z), bevel=0.025)
    collide(x - 0.32, x + 0.32, y0, spring, z - 0.32, z + 0.32)


def torch(p, n, s, y=1.9, lit=True):
    """En fakkel i smijernsholder på muren. p er punktet i murflaten (x, z),
    n innovernormalen."""
    n = Vector((n[0], 0, n[1])).normalized()
    base = Vector((p[0], y, p[1]))
    side = Vector((n.z, 0, -n.x))
    a, b = Vector((p[0], 0, p[1])) - side * 0.6, Vector((p[0], 0, p[1])) + side * 0.6 + n * 0.7
    BUSY.append((min(a.x, b.x), max(a.x, b.x), min(a.z, b.z), max(a.z, b.z)))
    yaw = math.atan2(-n.z, n.x) - math.pi / 2
    eblock("torch_plate", (0.13, 0.3, 0.03), s.iron, at=base + n * 0.015, r=yaw_m(yaw), bevel=0.006)
    ebeam("torch_arm", base + n * 0.02, base + n * 0.24 + Vector((0, 0.05, 0)), 0.03, 0.03, s.iron)
    ebeam("torch_strut", base + n * 0.02 - Vector((0, 0.16, 0)), base + n * 0.2 + Vector((0, 0.03, 0)), 0.022, 0.022, s.iron)
    ring = base + n * 0.26 + Vector((0, 0.06, 0))
    erevolve("torch_ring", [(0.055, ring.y - 0.03), (0.06, ring.y), (0.055, ring.y + 0.03)], s.iron, at=(ring.x, 0, ring.z), n=10, cap=False)
    stick_lo = ring + n * -0.02 - Vector((0, 0.28, 0))
    stick_hi = ring + n * 0.06 + Vector((0, 0.24, 0))
    ebeam("torch_stick", stick_lo, stick_hi, 0.05, 0.05, s.wood)
    top = stick_hi
    erevolve("torch_head", [(0.03, top.y - 0.08), (0.055, top.y - 0.04), (0.06, top.y + 0.04), (0.035, top.y + 0.08)], s.rope, at=(top.x, 0, top.z), n=10)
    if lit:
        erevolve("torch_flame", [(0.05, top.y + 0.06), (0.07, top.y + 0.14), (0.04, top.y + 0.26), (0.0, top.y + 0.38)], s.flame, at=(top.x, 0, top.z), n=8)
        light((top.x + n.x * 0.15, top.y + 0.3, top.z + n.z * 0.15))


def brazier(x, z, s, y0=0.0):
    """Et glørfat på tre bein, med glør og flammer."""
    for k in range(3):
        a = k * math.tau / 3
        foot = Vector((x + math.cos(a) * 0.42, y0, z + math.sin(a) * 0.42))
        top = Vector((x + math.cos(a) * 0.3, y0 + 0.75, z + math.sin(a) * 0.3))
        ebeam("brazier_leg", foot, top, 0.04, 0.04, s.iron)
        eblock("brazier_foot", (0.1, 0.04, 0.1), s.iron, at=foot + Vector((0, 0.02, 0)), bevel=0.01)
    erevolve("brazier_bowl", [(0.12, y0 + 0.7), (0.38, y0 + 0.82), (0.46, y0 + 0.98), (0.44, y0 + 1.0)], s.iron, at=(x, 0, z), n=20, cap=False)
    erevolve("brazier_coals", [(0.0, y0 + 0.9), (0.42, y0 + 0.94), (0.2, y0 + 1.0), (0.0, y0 + 1.02)], s.ember, at=(x, 0, z), n=16)
    for k in range(4):
        a = k * 1.7
        erevolve("brazier_flame", [(0.09, y0 + 0.98), (0.11, y0 + 1.1), (0.05, y0 + 1.3), (0.0, y0 + 1.45 + 0.1 * (k % 2))], s.flame, at=(x + math.cos(a) * 0.16, 0, z + math.sin(a) * 0.16), n=8)
    light((x, y0 + 1.6, z), (1.0, 0.5, 0.18), 2.8, 11.0)
    collide(x - 0.45, x + 0.45, y0, y0 + 1.0, z - 0.45, z + 0.45)


def candles(rng, x, z, s, y0=0.0, n=5, glow=True):
    """En klynge stearinlys med rennende voks."""
    erevolve("wax_pool", [(0.0, y0), (0.2, y0), (0.18, y0 + 0.012), (0.0, y0 + 0.014)], s.wax, at=(x, 0, z), n=12)
    for k in range(n):
        a = k * 2.4 + rng.uniform(0, 0.5)
        rr = 0.04 + 0.06 * (k % 3)
        cx, cz = x + math.cos(a) * rr, z + math.sin(a) * rr
        h = rng.uniform(0.06, 0.24)
        erevolve("candle", [(0.024, y0), (0.022, y0 + h), (0.012, y0 + h + 0.01)], s.wax, at=(cx, 0, cz), n=8)
        erevolve("candle_flame", [(0.008, y0 + h + 0.012), (0.012, y0 + h + 0.03), (0.0, y0 + h + 0.065)], s.flame, at=(cx, 0, cz), n=6)
    if glow:
        light((x, y0 + 0.5, z), (1.0, 0.7, 0.38), 1.3, 5.5)


def barrel(x, z, s, y0=0.0, h=0.95, r=0.34):
    erevolve("barrel", [(r * 0.86, y0), (r, y0 + h * 0.5), (r * 0.86, y0 + h)], s.wood, at=(x, 0, z), n=18)
    for t in (0.1, 0.3, 0.7, 0.9):
        rr = r * (0.86 + 0.14 * math.sin(math.pi * t)) + 0.006
        erevolve("hoop", [(rr, y0 + h * t - 0.025), (rr, y0 + h * t + 0.025)], s.iron, at=(x, 0, z), n=18, cap=False)
    collide(x - r, x + r, y0, y0 + h, z - r, z + r)


def crate(x, z, s, yaw=0.0, size=0.7, y0=0.0):
    """En kasse med jernbånd rundt øverst og nederst."""
    eblock("crate", (size, size, size), s.wood, at=(x, y0 + size / 2, z), r=yaw_m(yaw), bevel=0.02)
    for dy in (-0.36, 0.36):
        eblock("crate_band", (size + 0.012, size * 0.1, size + 0.012), s.iron, at=(x, y0 + size / 2 + dy * size, z), r=yaw_m(yaw), bevel=0.004)
    collide_at((x, y0 + size / 2, z), (size, size, size), yaw)


def sarcophagus(rng, x, z, s, yaw=0.0, y0=0.0):
    """En steinkiste med lokk, profilert kant og et kors i lokket."""
    r = yaw_m(yaw)
    eblock("sarc_base", (0.95, 0.12, 2.25), s.ashlar, at=Vector((x, y0 + 0.06, z)), r=r, bevel=0.02)
    eblock("sarc_body", (0.85, 0.62, 2.1), s.ashlar, at=Vector((x, y0 + 0.43, z)), r=r, bevel=0.025)
    eblock("sarc_lid", (0.95, 0.14, 2.24), s.ashlar, at=Vector((x, y0 + 0.81, z)), r=r @ Matrix.Rotation(rng.uniform(-0.01, 0.01), 3, "Z"), bevel=0.04, segments=2)
    eblock("sarc_ridge", (0.5, 0.08, 2.0), s.ashlar, at=Vector((x, y0 + 0.92, z)), r=r, bevel=0.035, segments=2)
    eblock("sarc_cross_v", (0.08, 0.04, 1.2), s.ashlar, at=Vector((x, y0 + 0.97, z)), r=r, bevel=0.01)
    eblock("sarc_cross_h", (0.42, 0.04, 0.08), s.ashlar, at=Vector((x, y0 + 0.97, z)) + r @ Vector((0, 0, -0.3)), r=r, bevel=0.01)
    collide_at((x, y0 + 0.5, z), (0.95, 1.0, 2.25), yaw)


def skull(x, y, z, s, yaw=0.0):
    erevolve("skull", [(0.0, y), (0.06, y + 0.01), (0.085, y + 0.07), (0.075, y + 0.13), (0.0, y + 0.16)], s.bone, at=(x, 0, z), n=10)
    eblock("jaw", (0.1, 0.04, 0.06), s.bone, at=yaw_m(yaw) @ Vector((0, 0, 0.07)) + Vector((x, y + 0.03, z)), r=yaw_m(yaw), bevel=0.012)


def bones(rng, x, z, s, n=4, y0=0.0):
    for _ in range(n):
        a = rng.uniform(0, math.tau)
        L = rng.uniform(0.25, 0.45)
        c = Vector((x + rng.uniform(-0.3, 0.3), y0 + 0.022, z + rng.uniform(-0.3, 0.3)))
        d = Vector((math.cos(a), 0, math.sin(a))) * L / 2
        ebeam("bone", c - d, c + d, 0.035, 0.035, s.bone)
        for e in (c - d, c + d):
            erevolve("bone_end", [(0.0, e.y - 0.025), (0.03, e.y), (0.0, e.y + 0.025)], s.bone, at=(e.x, 0, e.z), n=6)
    if rng.random() < 0.7:
        skull(x + rng.uniform(-0.2, 0.2), y0, z + rng.uniform(-0.2, 0.2), s, rng.uniform(0, 6))


def rubble(rng, x, z, spread, n, s, size=(0.2, 0.5), y0=0.0):
    for _ in range(n):
        a = rng.uniform(0, math.tau)
        rr = spread * math.sqrt(rng.random())
        k = rng.uniform(*size)
        eblock("rubble", (k, k * rng.uniform(0.5, 0.8), k * rng.uniform(0.6, 1.0)), s.ashlar,
               at=(x + math.cos(a) * rr, y0 + k * 0.22, z + math.sin(a) * rr),
               r=Matrix.Rotation(rng.uniform(-0.4, 0.4), 3, "X") @ yaw_m(rng.uniform(0, 6)), bevel=0.04)


# --- Gangene ----------------------------------------------------------------

DIRS = {"N": (0, -1), "S": (0, 1), "E": (1, 0), "W": (-1, 0)}


def corridor(arms, variant=0, seed=1):
    """En gangcelle med armer mot `arms` (N, S, Ø=E, V=W): tønnehvelv langs
    armene, krysshvelv der de møtes, gurtbuer på pilastre, sokkelsteiner
    og heller."""
    rng = random.Random(seed)
    s = Style()
    reset(1, 1)
    DATA["hw"] = [[HW]]
    DATA["ceil"] = [[SPRING + HW]]
    c = CELL / 2
    lo, hi = c - HW, c + HW
    m = mass(0, CELL, -0.4, SPRING + HW + 0.8, 0, CELL, s)
    cbox(lo, hi, -0.12, SPRING, lo, hi)
    rects = [(lo, hi, lo, hi)]
    for a in arms:
        dx, dz = DIRS[a]
        # Armene overlapper midtfeltet: kuttere som møtes i samme plan gir
        # flisete rester i boolean-operasjonen.
        if dx:
            x0, x1 = (hi, CELL + 0.05) if dx > 0 else (-0.05, lo)
            cbox(c if dx > 0 else x0, x1 if dx > 0 else c, -0.12, SPRING, lo, hi)
            rects.append((max(x0, 0), min(x1, CELL), lo, hi))
        else:
            z0, z1 = (hi, CELL + 0.05) if dz > 0 else (-0.05, lo)
            cbox(lo, hi, -0.12, SPRING, c if dz > 0 else z0, z1 if dz > 0 else c)
            rects.append((lo, hi, max(z0, 0), min(z1, CELL)))
        DATA["doors"].append([0, 0, dx, dz])
    ns = [a for a in arms if a in "NS"]
    ew = [a for a in arms if a in "EW"]
    if ns or not ew:
        cbarrel("z", -0.05 if "N" in arms else lo, CELL + 0.05 if "S" in arms else hi, c, SPRING, HW)
    if ew:
        cbarrel("x", -0.05 if "W" in arms else lo, CELL + 0.05 if "E" in arms else hi, c, SPRING, HW)
    carve(m)
    slabs(rng, rects, s)
    # Murene: sidene av midtfeltet uten arm, og sidene langs armene.
    walls = []
    for a, (dx, dz) in DIRS.items():
        if a in arms:
            continue
        if dx:
            x = hi if dx > 0 else lo
            walls.append(((x, lo), (x, hi), (-dx, 0)))
        else:
            z = hi if dz > 0 else lo
            walls.append(((lo, z), (hi, z), (0, -dz)))
    for a in arms:
        dx, dz = DIRS[a]
        if dx:
            x0, x1 = (hi, CELL) if dx > 0 else (0, lo)
            walls.append(((x0, lo), (x1, lo), (0, 1)))
            walls.append(((x0, hi), (x1, hi), (0, -1)))
        else:
            z0, z1 = (hi, CELL) if dz > 0 else (0, lo)
            walls.append(((lo, z0), (lo, z1), (1, 0)))
            walls.append(((hi, z0), (hi, z1), (-1, 0)))
    for a, b, n in walls:
        plinth(rng, a, b, n, s)
        # Kollideren: fra murflaten og ut til celleveggen.
        if n[0]:
            x = a[0]
            x0, x1 = (x - 0.6, x) if n[0] > 0 else (x, x + 0.6)
            collide(x0, x1, -0.4, SPRING + HW + 0.8, min(a[1], b[1]), max(a[1], b[1]))
        else:
            z = a[1]
            z0, z1 = (z - 0.6, z) if n[1] > 0 else (z, z + 0.6)
            collide(min(a[0], b[0]), max(a[0], b[0]), -0.4, SPRING + HW + 0.8, z0, z1)
    collide(0, CELL, -0.5, 0.0, 0, CELL)
    straight = sorted(arms) in (["N", "S"], ["E", "W"])
    if straight:
        # Gurtbue midt i cella, på pilastre.
        axis = (0, 1) if "N" in arms else (1, 0)
        arch_rib((c, c), axis, HW, SPRING, s)
        side = Vector((axis[1], 0, -axis[0]))
        for sgn in (-1, 1):
            p = Vector((c, 0, c)) + side * sgn * (HW - 0.05)
            pier(p.x, p.z, SPRING, s, w=0.42)
    else:
        # Gurtbuer der armene går ut av midtfeltet, og diagonalribber i
        # krysshvelvet, på hjørnepilastre.
        for a in arms:
            dx, dz = DIRS[a]
            arch_rib((c + dx * HW, c + dz * HW), (dx, dz) if dx else (0, 1), HW, SPRING, s, width=0.3)
        if len(arms) >= 2:
            diagonal_rib((c, c), 2 * HW, 2 * HW, SPRING, s)
        for sx in (-1, 1):
            for sz in (-1, 1):
                pier(c + sx * (HW - 0.02), c + sz * (HW - 0.02), SPRING, s, w=0.34)
    if variant == 1:
        # Fakkel på muren, ring for lenker og et avløp i gulvet.
        wall_x = lo if "N" in arms else c + 1.1
        if "N" in arms:
            torch((lo, c + 1.1), (1, 0), s)
            eblock("chain_staple", (0.04, 0.12, 0.08), s.iron, at=(hi - 0.02, 1.3, c - 1.2), bevel=0.01)
            erevolve("chain_ring", [(0.07, 1.1), (0.075, 1.14), (0.07, 1.18)], s.iron, at=(hi - 0.06, 0, c - 1.2), n=10, cap=False)
        else:
            torch((wall_x, lo), (0, 1), s)
        eblock("drain", (0.5, 0.03, 0.5), s.iron, at=(c, 0.0, c + 0.9), bevel=0.01)
        for k in range(5):
            eblock("drain_bar", (0.04, 0.035, 0.46), s.iron, at=(c - 0.2 + k * 0.1, 0.012, c + 0.9), bevel=0.006)
        bones(rng, hi - 0.4, c + 1.4, s, n=3)
    if variant == 2:
        # Delvis rast: en røys inntil muren og nedfalne buesteiner.
        x = hi - 0.5 if "N" in arms else c
        z = c - 0.6 if "N" in arms else hi - 0.5
        rubble(rng, x, z, 0.7, 16, s, (0.18, 0.5))
        rubble(rng, x - 0.3, z + 0.9, 0.5, 6, s, (0.12, 0.3))
        collide(x - 0.6, x + 0.6, 0, 0.45, z - 0.7, z + 0.7)
        eblock("fallen_voussoir", (0.4, 0.18, 0.36), s.ashlar, at=(c - 0.5, 0.09, c + 1.0), r=yaw_m(0.7), bevel=0.02)
    return m


# --- Rom ----------------------------------------------------------------------


def free(x0, x1, z0, z1):
    """Om rektangelet (x, z) på gulvet er fritt for kollidere (over gulvet)
    og opptatte flater."""
    for c, size, yaw in DATA["boxes"]:
        if c[1] + size[1] / 2 <= 0.05 or c[1] - size[1] / 2 > 2.2:
            continue
        hx, hz = size[0] / 2, size[2] / 2
        if abs(yaw) > 1e-3:
            hx = hz = max(hx, hz)
        if c[0] - hx < x1 and c[0] + hx > x0 and c[2] - hz < z1 and c[2] + hz > z0:
            return False
    for a0, a1, b0, b1 in BUSY:
        if a0 < x1 and a1 > x0 and b0 < z1 and b1 > z0:
            return False
    return True


class Room:
    """Et hvelvet rom w × d celler: krysshvelv over båsene, søyler i de
    indre krysningene, pilastre langs murene, ribber, sokkel, heller og
    dører med buestein. `rows` begrenser rommet til de første radene, og
    `z0` flytter det sørover (resten av fotavtrykket er annet: tårnet,
    apsisen). Åpninger i murene (dører, apsis, tårnet) er `gaps`: murlinja
    → [(midten, halv bredde, høyden over gulvet)]."""

    def __init__(self, w, d, spring=3.2, rows=None, z0=0.0, seed=1, columns=True, mass_bottom=-0.4):
        self.rng = random.Random(seed)
        self.s = Style()
        self.w, self.d = w, d
        self.spring = spring
        rows = rows or d
        self.x0, self.x1 = T, w * CELL - T
        self.z0, self.z1 = z0 + T, z0 + rows * CELL - T
        self.nx, self.nz = w, rows
        self.bx = (self.x1 - self.x0) / self.nx
        self.bz = (self.z1 - self.z0) / self.nz
        self.crown = spring + max(self.bx, self.bz) / 2
        self.columns = columns
        self.doors = []
        self.gaps = {}
        self.holes = []
        self.passages = []
        reset(w, d)
        for j in range(d):
            for i in range(w):
                DATA["ceil"][j][i] = round(self.crown - 0.3, 2)
        self.mass = mass(0, w * CELL, mass_bottom, self.crown + 0.8, 0, d * CELL, self.s)
        cbox(self.x0, self.x1, -0.12, spring, self.z0, self.z1)
        for j in range(self.nz):
            cbarrel("x", self.x0, self.x1, self.z0 + (j + 0.5) * self.bz, spring, self.bz / 2)
        for i in range(self.nx):
            cbarrel("z", self.z0, self.z1, self.x0 + (i + 0.5) * self.bx, spring, self.bx / 2)

    def gap(self, axis, v, centre, half, top):
        self.gaps.setdefault((axis, v), []).append((centre, half, top))

    def door(self, i, j, dx, dz):
        """En dør i muren ut av celle (i, j) mot (dx, dz), med gangens profil."""
        self.doors.append((i, j, dx, dz))
        DATA["doors"].append([i, j, dx, dz])
        cx, cz = (i + 0.5) * CELL, (j + 0.5) * CELL
        if dx:
            x0, x1 = ((i + 1) * CELL - T - 0.3, (i + 1) * CELL + 0.05) if dx > 0 else (i * CELL - 0.05, i * CELL + T + 0.3)
            cbox(x0, x1, -0.12, SPRING, cz - HW, cz + HW)
            cbarrel("x", x0, x1, cz, SPRING, HW)
            self.gap("x", self.x1 if dx > 0 else self.x0, cz, HW, SPRING + HW)
            self.passages.append(((self.x1, (i + 1) * CELL) if dx > 0 else (i * CELL, self.x0)) + (cz - HW, cz + HW))
        else:
            z0, z1 = ((j + 1) * CELL - T - 0.3, (j + 1) * CELL + 0.05) if dz > 0 else (j * CELL - 0.05, j * CELL + T + 0.3)
            cbox(cx - HW, cx + HW, -0.12, SPRING, z0, z1)
            cbarrel("z", z0, z1, cx, SPRING, HW)
            self.gap("z", self.z1 if dz > 0 else self.z0, cx, HW, SPRING + HW)
            self.passages.append((cx - HW, cx + HW) + ((self.z1, (j + 1) * CELL) if dz > 0 else (j * CELL, self.z0)))

    def in_gap(self, axis, v, t, margin=0.45):
        return any(abs(t - c) < h + margin for c, h, _ in self.gaps.get((axis, v), []))

    def build(self):
        """Hogg ut rommet og sett inn det faste: søyler, pilastre, ribber,
        sokkel, heller, dørbuer og kollidere."""
        s, rng = self.s, self.rng
        carve(self.mass)
        xs = [self.x0 + i * self.bx for i in range(self.nx + 1)]
        zs = [self.z0 + j * self.bz for j in range(self.nz + 1)]
        # Søylene i de indre krysningene.
        if self.columns:
            for x in xs[1:-1]:
                for z in zs[1:-1]:
                    column(x, z, 0.0, self.spring, s)
        # Pilastre der ribbene møter murene, og i hjørnene (ikke i åpningene).
        for x in xs:
            for z in (self.z0, self.z1):
                if self.in_gap("z", z, x):
                    continue
                pier(x + (0.17 if x == xs[0] else -0.17 if x == xs[-1] else 0), z + (0.17 if z == self.z0 else -0.17), self.spring, s, w=0.4)
        for z in zs[1:-1]:
            for x in (self.x0, self.x1):
                if self.in_gap("x", x, z):
                    continue
                pier(x + (0.17 if x == self.x0 else -0.17), z, self.spring, s, w=0.4)
        # Gurtbuer langs linjene, diagonalribber i hver bås.
        for x in xs[1:-1]:
            for j in range(self.nz):
                arch_rib((x, self.z0 + (j + 0.5) * self.bz), (1, 0), self.bz / 2, self.spring, s, width=0.34)
        for z in zs[1:-1]:
            for i in range(self.nx):
                arch_rib((self.x0 + (i + 0.5) * self.bx, z), (0, 1), self.bx / 2, self.spring, s, width=0.34)
        for i in range(self.nx):
            for j in range(self.nz):
                diagonal_rib((self.x0 + (i + 0.5) * self.bx, self.z0 + (j + 0.5) * self.bz), self.bx, self.bz, self.spring, s)
        # Dørbuene.
        for i, j, dx, dz in self.doors:
            cx, cz = (i + 0.5) * CELL, (j + 0.5) * CELL
            if dx:
                archivolt((self.x1 if dx > 0 else self.x0, cz), (-dx, 0), HW, SPRING, s)
            else:
                archivolt((cx, self.z1 if dz > 0 else self.z0), (0, -dz), HW, SPRING, s)
        self.wall_line("z", self.z0, self.x0, self.x1, (0, 1))
        self.wall_line("z", self.z1, self.x0, self.x1, (0, -1))
        self.wall_line("x", self.x0, self.z0, self.z1, (1, 0))
        self.wall_line("x", self.x1, self.z0, self.z1, (-1, 0))
        # Hellene, utenom hullene (bassenget), og i dørgjennomgangene.
        rects = [(self.x0, self.x1, self.z0, self.z1)]
        for hx0, hx1, hz0, hz1 in self.holes:
            out = []
            for x0, x1, z0, z1 in rects:
                if hx1 <= x0 or hx0 >= x1 or hz1 <= z0 or hz0 >= z1:
                    out.append((x0, x1, z0, z1))
                    continue
                out += [r for r in [(x0, x1, z0, hz0), (x0, x1, hz1, z1), (x0, hx0, max(z0, hz0), min(z1, hz1)), (hx1, x1, max(z0, hz0), min(z1, hz1))] if r[1] - r[0] > 0.05 and r[3] - r[2] > 0.05]
            rects = out
        slabs(rng, rects, s, size=(0.55, 1.0))
        slabs(rng, self.passages, s, size=(0.45, 0.8))

    def wall_line(self, axis, v, a0, a1, n):
        """Sokkel og kollidere langs en mur, med gap for åpningene."""
        gaps = sorted(self.gaps.get((axis, v), []))
        top = self.crown + 0.8
        t = a0
        for c, h, _ in gaps + [(a1 + 0.4 + HW, 0.0, 0.0)]:
            lo, hi = t, c - h - 0.4
            if hi - lo > 0.1:
                if axis == "z":
                    plinth(self.rng, (lo, v), (hi, v), n, self.s)
                else:
                    plinth(self.rng, (v, lo), (v, hi), n, self.s)
            t = c + h + 0.4

        def wall_box(a, b, y0, y1):
            if axis == "z":
                z0, z1 = (v - T - 0.3, v) if n[1] > 0 else (v, v + T + 0.3)
                collide(a, b, y0, y1, z0, z1)
            else:
                x0, x1 = (v - T - 0.3, v) if n[0] > 0 else (v, v + T + 0.3)
                collide(x0, x1, y0, y1, a, b)

        t = a0 - T
        for c, h, ytop in gaps:
            if c - h > t:
                wall_box(t, c - h, -0.5, top)
            wall_box(c - h, c + h, ytop, top)
            t = c + h
        if a1 + T > t:
            wall_box(t, a1 + T, -0.5, top)

    def floor_collider(self):
        collide(0, self.w * CELL, -0.5, 0.0, 0, self.d * CELL)

    def decor(self):
        """Steder for pynt hver etasje setter ut: ledige murflater (1,7 m
        lange, 0,6 m dype, «wall»: x, z og normalen) og ledige flekker på
        gulvet («floor»), utenom dører, fakler, søyler og møbler."""
        lines = [("z", self.z0, self.x0, self.x1, (0, 1)), ("z", self.z1, self.x0, self.x1, (0, -1)), ("x", self.x0, self.z0, self.z1, (1, 0)), ("x", self.x1, self.z0, self.z1, (-1, 0))]
        for axis, v, a0, a1, n in lines:
            t = a0 + 1.0
            while t < a1 - 0.95:
                inward = (v + 0.02 * (n[0] + n[1]), v + 0.62 * (n[0] + n[1]))
                lo, hi = min(inward), max(inward)
                rect = (t - 0.85, t + 0.85, lo, hi) if axis == "z" else (lo, hi, t - 0.85, t + 0.85)
                if not self.in_gap(axis, v, t, margin=1.3) and free(*rect):
                    x, z = (t, (lo + hi) / 2 - 0.3 * (n[0] + n[1])) if axis == "z" else ((lo + hi) / 2 - 0.3 * (n[0] + n[1]), t)
                    spot("wall", (x, z, float(n[0]), float(n[1])))
                    BUSY.append(rect)
                    t += 2.4
                else:
                    t += 0.5
        for i in range(self.nx):
            for j in range(self.nz):
                x = self.x0 + (i + 0.5) * self.bx + self.rng.uniform(-0.6, 0.6)
                z = self.z0 + (j + 0.5) * self.bz + self.rng.uniform(-0.6, 0.6)
                rect = (x - 0.8, x + 0.8, z - 0.8, z + 0.8)
                if any(h[0] < rect[1] and h[1] > rect[0] and h[2] < rect[3] and h[3] > rect[2] for h in self.holes):
                    continue
                if free(*rect):
                    spot("floor", (x, 0.0, z))
                    BUSY.append(rect)


def tower_opening(room, i, j):
    """Åpningen fra rommet (celle i, j) inn til tårncella sør for den."""
    cx = (i + 0.5) * CELL
    zf = room.z1
    w, h = 3.4, 3.0
    cbox(cx - w / 2, cx + w / 2, -0.12, h, zf - 0.05, (j + 1) * CELL + 0.05)
    # Hele tårncella skjæres bort: der står vindeltrappa med sin egen sjakt
    # (ellers står rommets murmasse som en vegg bak gitteret og som et gulv
    # midt i trappa).
    cbox(i * CELL - 0.01, (i + 1) * CELL + 0.01, -1.0, 8.0, (j + 1) * CELL, (j + 2) * CELL + 0.05)
    room.gap("z", zf, cx, w / 2, h)
    room.passages.append((cx - w / 2, cx + w / 2, zf, (j + 1) * CELL))
    return (cx, zf, w, h)


def tower_trim(room, cx, zf, w, h, gate):
    """Overligger og vanger rundt tårnåpningen; i portrommet føringer for
    gitteret og vinsjen som sveiver det opp."""
    s = room.s
    eblock("lintel", (w + 1.0, 0.55, 0.3), s.ashlar, at=(cx, h + 0.275, zf - 0.12), bevel=0.03, segments=2)
    for k in range(5):
        eblock("lintel_course", (0.9 + 0.1 * (k % 2), 0.32, 0.22), s.ashlar, at=(cx - 1.9 + k * 0.95, h + 0.72, zf - 0.08), bevel=0.02)
    for sgn in (-1, 1):
        y = 0.0
        k = 0
        while y < h - 0.05:
            hh = min(0.45, h - y)
            eblock("gate_jamb", (0.45 + 0.08 * (k % 2), hh - 0.012, 0.3), s.ashlar, at=(cx + sgn * (w / 2 + 0.22), y + hh / 2, zf - 0.12), bevel=0.02)
            y += hh
            k += 1
    if gate:
        # Føringene til gitteret i vangene, og vinsjen ved siden av.
        for sgn in (-1, 1):
            eblock("gate_rail", (0.08, h + 0.4, 0.14), s.iron, at=(cx + sgn * (w / 2 - 0.02), h / 2 + 0.2, zf + 0.3), bevel=0.01)
        wx = cx + w / 2 + 1.1
        for sgn in (-1, 1):
            eblock("winch_post", (0.16, 1.2, 0.16), s.wood, at=(wx + sgn * 0.45, 0.6, zf - 0.45), bevel=0.02)
        bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.16, depth=0.8, location=B((wx, 1.0, zf - 0.45)), rotation=(0, math.pi / 2, 0))
        drum = bpy.context.active_object
        drum.data.materials.append(s.wood)
        town.uv_box(drum)
        PIECES.append(drum)
        for sgn in (-1, 1):
            ebeam("winch_spoke", (wx + sgn * 0.5, 1.0, zf - 0.45 - 0.5), (wx + sgn * 0.5, 1.0, zf - 0.45 + 0.5), 0.05, 0.05, s.wood)
            ebeam("winch_spoke", (wx + sgn * 0.5, 0.5, zf - 0.45), (wx + sgn * 0.5, 1.5, zf - 0.45), 0.05, 0.05, s.wood)
        ebeam("winch_chain", (wx, 1.12, zf - 0.45), (cx + w / 2 - 0.1, h + 0.1, zf + 0.2), 0.03, 0.03, s.iron)
        collide(wx - 0.6, wx + 0.6, 0, 1.4, zf - 0.75, zf - 0.15)


def start_room():
    """Startrommet i første etasje: søylehall der trappa opp til krypten står
    i nordmuren, med tønner, kasser og et bord ved inngangen."""
    r = Room(3, 3, seed=11)
    r.door(2, 1, 1, 0)
    r.door(1, 2, 0, 1)
    r.door(0, 1, -1, 0)
    r.build()
    r.floor_collider()
    s, rng = r.s, r.rng
    spot("ascend", (6.0, 0.0, T, 0.0))
    spot("start", (6.0, 0.0, 5.0, 0.0))
    for x, z in [(1.4, 1.5), (1.4, 2.3), (2.2, 1.4)]:
        barrel(x, z, s)
    crate(10.3, 1.6, s, 0.3)
    crate(10.5, 2.4, s, -0.2, 0.55)
    crate(10.35, 1.65, s, 0.5, 0.45, y0=0.7)
    # Bord med en lykt og en åpen bok, benk ved siden av.
    eblock("table_top", (1.6, 0.07, 0.8), s.wood, at=(9.8, 0.82, 9.8), bevel=0.015)
    for sx in (-1, 1):
        for sz in (-1, 1):
            eblock("table_leg", (0.08, 0.78, 0.08), s.wood, at=(9.8 + sx * 0.68, 0.39, 9.8 + sz * 0.3), bevel=0.01)
    collide(9.0, 10.6, 0, 0.86, 9.4, 10.2)
    candles(rng, 9.4, 9.7, s, y0=0.86, n=3)
    eblock("book", (0.34, 0.04, 0.25), s.cloth, at=(10.1, 0.88, 9.8), r=yaw_m(0.2), bevel=0.01)
    torch((T, 2.0), (1, 0), s)
    torch((12 - T, 2.0), (-1, 0), s)
    torch((T, 10.0), (1, 0), s)
    torch((12 - T, 10.0), (-1, 0), s)
    for p in [(3.0, 0.0, 8.0), (9.0, 0.0, 6.0)]:
        spot("spawn", p)
    r.decor()
    return r


def gate_room(gate=True):
    """Portrommet (og trappefoten): søylehall 3 × 3 med åpningen mot
    vindeltrappa i sør (cella (1, 3)). I portrommet henger gitteret i
    åpningen og eliten vokter nøkkelen."""
    r = Room(3, 4, rows=3, seed=12 if gate else 13)
    r.door(1, 0, 0, -1)
    r.door(2, 1, 1, 0)
    r.door(0, 1, -1, 0)
    opening = tower_opening(r, 1, 2)
    r.build()
    tower_trim(r, *opening, gate)
    s, rng = r.s, r.rng
    collide(0, 12, -0.5, 0, 0, 12)
    # Tårncella: ingen gulv for fiendene (trappa er egen modell).
    for i in range(3):
        DATA["floor"][3][i] = None
    spot("tower", [1, 3])
    if gate:
        spot("gate", (6.0, 0.0, 11.75, 0.0))
        spot("elite", (6.0, 0.0, 6.0))
        # Nøkkelvokterens hjørne: bein og et skrin.
        bones(rng, 2.0, 10.5, s, 5)
        bones(rng, 10.0, 10.8, s, 3)
        brazier(3.4, 9.6, s)
        brazier(8.6, 9.6, s)
    else:
        spot("start", (6.0, 0.0, 9.6, 0.0))
        for x, z in [(1.5, 1.4), (10.5, 1.4)]:
            barrel(x, z, s)
        torch((6.0 + 1.9, 11.4), (0, -1), s)
        torch((6.0 - 1.9, 11.4), (0, -1), s)
    torch((T, 6.0 + 1.0), (1, 0), s)
    torch((12 - T, 6.0 + 1.0), (-1, 0), s)
    spot("spawn", (3.0, 0.0, 5.0))
    spot("spawn", (9.0, 0.0, 5.0))
    r.decor()
    return r


def hall_room():
    """Søylehallen: ni båser med krysshvelv på fire søyler, glørfat i midten,
    tønner og sekker langs murene."""
    r = Room(3, 3, seed=21)
    r.door(1, 0, 0, -1)
    r.door(2, 1, 1, 0)
    r.door(1, 2, 0, 1)
    r.door(0, 1, -1, 0)
    r.build()
    r.floor_collider()
    s, rng = r.s, r.rng
    brazier(6.0, 6.0, s)
    for x, z in [(1.4, 1.4), (2.15, 1.35), (1.35, 2.2)]:
        barrel(x, z, s)
    crate(10.5, 10.5, s, 0.4)
    crate(10.4, 9.7, s, -0.1, 0.55)
    bones(rng, 10.5, 1.6, s, 4)
    rubble(rng, 1.5, 10.5, 0.6, 9, s, (0.15, 0.4))
    torch((T, 9.6), (1, 0), s)
    torch((12 - T, 2.4), (-1, 0), s)
    for p in [(3.0, 0.0, 3.0), (9.0, 0.0, 9.0), (9.0, 0.0, 3.0)]:
        spot("spawn", p)
    spot("chest", (1.4, 0.0, 6.0 + 2.2, math.pi / 2))
    r.decor()
    return r


def long_room():
    """Den lange hallen: 2 × 5 celler med søylerad på midten og sarkofager
    langs murene, stearinlys på lokkene."""
    r = Room(2, 5, spring=3.0, seed=31)
    r.door(1, 0, 0, -1)
    r.door(0, 4, 0, 1)
    r.door(0, 2, -1, 0)
    r.door(1, 2, 1, 0)
    r.build()
    r.floor_collider()
    s, rng = r.s, r.rng
    for z in (3.0, 6.4, 13.6, 17.0):
        sarcophagus(rng, 1.35, z, s)
        sarcophagus(rng, 6.65, z, s)
    for z in (3.0, 13.6):
        candles(rng, 1.35, z + 0.6, s, y0=0.97, n=4)
        candles(rng, 6.65, z - 0.6, s, y0=0.97, n=3, glow=z > 10)
    torch((T, 9.0 - 2.2), (1, 0), s, lit=False)
    torch((8 - T, 11.2), (-1, 0), s)
    bones(rng, 4.0, 10.0, s, 3)
    for p in [(4.0, 0.0, 4.0), (4.0, 0.0, 16.0), (2.0, 0.0, 10.0)]:
        spot("spawn", p)
    spot("chest", (6.8, 0.0, 9.8, -math.pi / 2))
    r.decor()
    return r


def cistern_room():
    """Cisternen: 4 × 4 celler, et sunket basseng med vann midt i (2 × 2
    celler) mellom søylene, trinn ned i vannet og gangbane rundt."""
    r = Room(4, 4, spring=3.4, seed=41, mass_bottom=-2.2)
    r.door(1, 0, 0, -1)
    r.door(3, 2, 1, 0)
    r.door(2, 3, 0, 1)
    r.door(0, 1, -1, 0)
    xs = [r.x0 + i * r.bx for i in range(5)]
    p0, p1 = xs[1], xs[3]
    deep = -1.6
    cbox(p0, p1, deep - 0.12, 0.1, p0, p1)
    r.holes.append((p0, p1, p0, p1))
    r.build()
    s, rng = r.s, r.rng
    # Bassenget: bunn, foring av kvaderstein, kantstein og vann.
    slabs(rng, [(p0, p1, p0, p1)], s, size=(0.7, 1.2), y=deep)
    for a, b, n in [((p0, p0), (p1, p0), (0, 1)), ((p0, p1), (p1, p1), (0, -1)), ((p0, p0), (p0, p1), (1, 0)), ((p1, p0), (p1, p1), (-1, 0))]:
        av, bv = Vector((a[0], 0, a[1])), Vector((b[0], 0, b[1]))
        along = (bv - av).normalized()
        nv = Vector((n[0], 0, n[1]))
        yaw = math.atan2(-along.z, along.x)
        y = deep
        k = 0
        while y < -0.05:
            h = min(0.4, -y)
            t = 0.25 * (k % 2)
            while t < (bv - av).length - 0.05:
                w = min(rng.uniform(0.6, 1.0), (bv - av).length - t)
                c = av + along * (t + w / 2) + nv * 0.04
                eblock("lining", (w - 0.012, h - 0.012, 0.1), s.ashlar, at=(c.x, y + h / 2, c.z), r=yaw_m(yaw), bevel=0.015)
                t += w
            y += h
            k += 1
        t = 0.0
        while t < (bv - av).length - 0.05:
            w = min(rng.uniform(0.7, 1.1), (bv - av).length - t)
            c = av + along * (t + w / 2) - nv * 0.18
            eblock("coping", (w - 0.012, 0.16, 0.5), s.ashlar, at=(c.x, -0.06, c.z), r=yaw_m(yaw), bevel=0.03)
            t += w
    # Søylene i vannet og på kanten står fra bunnen.
    for i in (1, 2, 3):
        for j in (1, 2, 3):
            column(xs[i], xs[j], deep, r.spring, s)
    bpy.ops.mesh.primitive_plane_add(size=1.0, location=B(((p0 + p1) / 2, deep + 0.7, (p0 + p1) / 2)))
    water = bpy.context.active_object
    water.scale = (p1 - p0, p1 - p0, 1)
    water.data.materials.append(s.water)
    town.uv_box(water)
    PIECES.append(water)
    # Trinn ned i vannet fra sør.
    sx = 8.0
    for k in range(7):
        y = -0.2 * (k + 1)
        eblock("cistern_step", (2.0, y - deep, 0.34), s.ashlar, at=(sx, (y + deep) / 2, p1 - 0.17 - k * 0.32), bevel=0.02)
    hull([(sx - 1.0, 0.0, p1), (sx + 1.0, 0.0, p1), (sx - 1.0, deep, p1 - 2.6), (sx + 1.0, deep, p1 - 2.6), (sx - 1.0, deep - 0.3, p1), (sx + 1.0, deep - 0.3, p1), (sx - 1.0, deep - 0.3, p1 - 2.6), (sx + 1.0, deep - 0.3, p1 - 2.6)])
    # Gulvet rundt og bunnen av bassenget.
    collide(0, 16, -2.2, 0, 0, p0)
    collide(0, 16, -2.2, 0, p1, 16)
    collide(0, p0, -2.2, 0, p0, p1)
    collide(p1, 16, -2.2, 0, p0, p1)
    collide(p0, p1, deep - 0.5, deep, p0, p1)
    for i in (1, 2):
        for j in (1, 2):
            DATA["floor"][j][i] = deep
    # Lykter på stolper i hjørnene av bassenget.
    for x, z in [(p0 - 0.6, p0 - 0.6), (p1 + 0.6, p1 + 0.6)]:
        eblock("lamp_post", (0.12, 2.2, 0.12), s.wood, at=(x, 1.1, z), bevel=0.02)
        eblock("lamp_box", (0.26, 0.34, 0.26), s.iron, at=(x, 2.35, z), bevel=0.02)
        erevolve("lamp_flame", [(0.03, 2.28), (0.05, 2.34), (0.0, 2.46)], s.flame, at=(x, 0, z), n=8)
        light((x, 2.4, z), (1.0, 0.68, 0.36), 2.0, 10.0)
        collide(x - 0.1, x + 0.1, 0, 2.5, z - 0.1, z + 0.1)
    torch((T, 2.2), (1, 0), s)
    torch((16 - T, 13.8), (-1, 0), s)
    for p in [(2.0, 0.0, 2.0), (14.0, 0.0, 14.0), (14.0, 0.0, 2.0), (2.0, 0.0, 14.0)]:
        spot("spawn", p)
    spot("chest", (14.4, 0.0, 1.6, 0.0))
    r.decor()
    return r


def gallery_room():
    """Galleriet: 3 × 4 celler, høyt hvelv, en forhøyning langs nordmuren
    (3,2 m) på blindarkader med brystvern, og en bred trapp opp langs
    vestmuren. Kista står oppe."""
    r = Room(3, 4, spring=5.6, seed=51)
    r.door(2, 2, 1, 0)
    r.door(1, 3, 0, 1)
    r.door(0, 3, -1, 0)
    r.build()
    s, rng = r.s, r.rng
    up = 3.2
    zf = CELL
    # Forhøyningen: massiv, med blindarkader mot sør og heller oppå.
    eblock("dais", (12 - 2 * T, up, zf - T), s.wall, at=(6.0, up / 2, (T + zf) / 2), bevel=0.02)
    slabs(rng, [(T, 12 - T, T, zf)], s, size=(0.55, 0.95), y=up)
    for k in range(3):
        cx = 4.0 + 1.13 + k * 2.27
        if cx > 12 - T - 0.5:
            break
        archivolt((cx, zf), (0, 1), 0.75, 1.9, s, depth=0.26, out=0.05, n=9)
    for x in (4.2, 11.2):
        pier(x, zf + 0.12, up - 0.15, s, w=0.4)
    # Brystvern med kapping, åpent mot trappa.
    eblock("parapet", (12 - T - 4.4, 0.9, 0.3), s.ashlar, at=((4.4 + 12 - T) / 2, up + 0.45, zf - 0.15), bevel=0.03)
    eblock("parapet_cap", (12 - T - 4.4 + 0.1, 0.1, 0.4), s.ashlar, at=((4.4 + 12 - T) / 2, up + 0.95, zf - 0.15), bevel=0.02)
    collide(4.4, 12 - T, up, up + 1.0, zf - 0.3, zf)
    collide(0, 12, -0.5, up, 0, zf)
    # Trappa langs vestmuren: fra z = 12 (gulvet) til z = 4 (forhøyningen).
    n = 16
    run = 8.0 / n
    for k in range(n):
        y = up * (k + 1) / n
        z = 12.0 - (k + 1) * run
        eblock("stair", (4.0 - T, y, run + 0.02), s.ashlar, at=((T + 4.0) / 2, y / 2, z + run / 2), bevel=0.02)
    eblock("stair_rail", (0.3, 0.9, 8.0), s.ashlar, at=(4.0 - 0.15, 0.45 + up / 2, 8.0), r=Matrix.Rotation(math.atan2(up, 8.0), 3, "X"), bevel=0.03)
    BUSY.append((T, 4.2, 4.0, 12.6))
    hull([(T, 0, 12.0), (4.0, 0, 12.0), (T, up, 4.0), (4.0, up, 4.0), (T, -0.3, 12.0), (4.0, -0.3, 12.0), (T, -0.3, 4.0), (4.0, -0.3, 4.0)])
    collide(3.85, 4.15, 0, up + 1.0, 4.0, 12.0)
    collide(0, 12, -0.5, 0, zf, 16)
    DATA["floor"][0] = [up, up, up]
    DATA["ramps"].append([0, 1, 0, -1, round(up / 2, 2), up])
    DATA["ramps"].append([0, 2, 0, -1, 0.0, round(up / 2, 2)])
    DATA["floor"][1][0] = None
    DATA["floor"][2][0] = None
    DATA["blocked"] += [[0, 1, 1, 0], [0, 2, 1, 0]]
    # Lys: et glørfat oppe, fakler under.
    brazier(8.0, 2.2, s, y0=up)
    torch((12 - T, 10.0), (-1, 0), s)
    torch((6.0, 16 - T), (0, -1), s, y=2.2)
    spot("chest", (10.6, up, 1.4, 0.0))
    for p in [(8.0, 0.0, 9.0), (6.0, up, 2.2), (10.0, 0.0, 13.0)]:
        spot("spawn", p)
    bones(rng, 10.6, 13.8, s, 3)
    r.decor()
    return r


def chapel_room():
    """Kapellet: skip på 3 × 3 båser med apsis i nord, alter med kors og
    lys, sarkofager langs murene og et gravmæle midt i skipet."""
    r = Room(3, 4, rows=3, z0=CELL, seed=61)
    r.door(1, 3, 0, 1)
    r.door(0, 2, -1, 0)
    r.door(2, 2, 1, 0)
    # Apsisen: en halvsirkel inn i nordmuren med halvkuppel.
    ar = r.bx / 2
    ccyl(6.0, CELL + T, -0.12, r.spring, ar)
    csphere(6.0, r.spring, CELL + T, ar)
    r.gap("z", r.z0, 6.0, ar, r.spring + ar)
    r.build()
    s, rng = r.s, r.rng
    collide(0, 12, -0.5, 0, CELL, 16)
    collide(0, 12, -0.5, 0, 0, CELL)
    slabs(rng, [(6.0 - ar, 6.0 + ar, CELL + T - ar, CELL + T)], s, size=(0.5, 0.8))
    # Apsismuren som kollider (åtte bokser rundt).
    for k in range(9):
        a = math.pi + math.pi * k / 8
        x, z = 6.0 + math.cos(a) * (ar + 0.3), CELL + T + math.sin(a) * (ar + 0.3)
        collide_at((x, 3.0, z), (1.1, 6.0, 0.6), -a + math.pi / 2)
    for i in (0, 2):
        DATA["floor"][0][i] = None
    DATA["floor"][0][1] = 0.0
    # Alteret med alterduk, kors og lysestaker.
    az = CELL + T - 0.8
    eblock("altar_step", (2.2, 0.18, 1.3), s.ashlar, at=(6.0, 0.09, az + 0.1), bevel=0.03)
    eblock("altar", (1.5, 0.95, 0.7), s.ashlar, at=(6.0, 0.18 + 0.475, az - 0.1), bevel=0.03, segments=2)
    eblock("altar_top", (1.7, 0.1, 0.85), s.ashlar, at=(6.0, 1.18, az - 0.1), bevel=0.02)
    eblock("altar_cloth", (1.2, 0.012, 0.88), s.cloth, at=(6.0, 1.236, az - 0.1), bevel=0.003)
    eblock("altar_cloth_hang", (1.2, 0.4, 0.012), s.cloth, at=(6.0, 1.04, az - 0.1 + 0.44), bevel=0.003)
    eblock("cross_v", (0.08, 0.9, 0.06), s.gold, at=(6.0, 1.68, az - 0.35), bevel=0.01)
    eblock("cross_h", (0.5, 0.08, 0.06), s.gold, at=(6.0, 1.86, az - 0.35), bevel=0.01)
    collide(4.9, 7.1, 0, 1.25, az - 0.6, az + 0.75)
    for sx in (-1, 1):
        x = 6.0 + sx * 0.75
        erevolve("candlestick", [(0.08, 1.23), (0.03, 1.28), (0.025, 1.6), (0.06, 1.62), (0.05, 1.64)], s.gold, at=(x, 0, az - 0.2), n=10)
        erevolve("altar_candle", [(0.03, 1.64), (0.028, 1.86)], s.wax, at=(x, 0, az - 0.2), n=8)
        erevolve("altar_flame", [(0.012, 1.87), (0.02, 1.9), (0.0, 1.97)], s.flame, at=(x, 0, az - 0.2), n=6)
    light((6.0, 2.2, az + 0.3), (1.0, 0.72, 0.42), 2.2, 9.0)
    # Gravmælet midt i skipet, sarkofager langs murene.
    sarcophagus(rng, 6.0, 10.0, s)
    for z in (8.0, 13.0):
        sarcophagus(rng, 1.4, z, s)
        sarcophagus(rng, 10.6, z, s)
    candles(rng, 6.0, 11.0, s, y0=0.97, n=5)
    candles(rng, 1.4, 13.6, s, y0=0.97, n=3, glow=False)
    torch((T, 6.0), (1, 0), s)
    torch((12 - T, 6.0), (-1, 0), s)
    for p in [(3.5, 0.0, 10.5), (8.5, 0.0, 14.0)]:
        spot("spawn", p)
    spot("chest", (4.6, 0.0, 6.0, 0.0))
    r.decor()
    return r


def treasury_room():
    """Skattkammeret: 2 × 2 celler med én søyle, kister langs murene, mynt
    i hauger og lys i nisjene."""
    r = Room(2, 2, spring=2.8, seed=71)
    r.door(0, 1, -1, 0)
    r.door(1, 1, 0, 1)
    r.build()
    r.floor_collider()
    s, rng = r.s, r.rng
    for k in range(5):
        x, z = 6.0 + rng.uniform(-0.5, 0.5), 2.0 + rng.uniform(-0.6, 0.6)
        h = rng.uniform(0.05, 0.14)
        erevolve("coins", [(0.0, 0.0), (h * 2.6, 0.0), (h * 1.4, h * 0.6), (0.0, h)], s.gold, at=(x, 0, z), n=14)
    candles(rng, 1.2, 1.2, s, n=6)
    candles(rng, 6.8, 6.8, s, n=4)
    spot("chest", (1.3, 0.0, 4.0, math.pi / 2))
    spot("chest", (4.0, 0.0, 1.3, 0.0))
    spot("chest", (6.7, 0.0, 3.0, -math.pi / 2))
    spot("spawn", (4.0, 0.0, 5.5))
    r.decor()
    return r


def arena_room():
    """Rotunden der Rammeverket venter: en rund hall, 18 m over, med
    halvkuppel og ribber, ring av søyler, glørfat og et innlagt rosevindu
    av stein i gulvet. Tokens ligger rundt midten."""
    s = Style()
    rng = random.Random(81)
    w = d = 5
    reset(w, d)
    R = 9.2
    c = 10.0
    spring = 5.0
    m = mass(0, 20, -0.4, spring + R + 0.6, 0, 20, s)
    ccyl(c, c, -0.12, spring, R, verts=64)
    csphere(c, spring, c, R, segs=48)
    doors = [(2, 0, 0, -1), (2, 4, 0, 1)]
    for i, j, dx, dz in doors:
        DATA["doors"].append([i, j, dx, dz])
        cx = (i + 0.5) * CELL
        z0, z1 = (-0.05, 1.8) if dz < 0 else (18.2, 20.05)
        cbox(cx - HW, cx + HW, -0.12, SPRING, z0, z1)
        cbarrel("z", z0, z1, cx, SPRING, HW)
    carve(m)
    for i, j, dx, dz in doors:
        z = c - dz * -R * 0.995 if False else (c + dz * R * 0.99)
        archivolt((10.0, z), (0, -dz), HW, SPRING, s)
    # Gulvet: heller i ringer ut fra midten, og en rose av mørk stein.
    for k in range(6):
        r0, r1 = k * 1.55, (k + 1) * 1.55
        if r0 > R:
            break
        r1 = min(r1, R)
        n = max(6, int(2 * math.pi * (r0 + r1) / 2 / 1.0))
        for q in range(n):
            a0, a1 = 2 * math.pi * q / n, 2 * math.pi * (q + 1) / n
            bm = bmesh.new()
            pts = []
            for rr, aa in [(r0, a0), (r1, a0), (r1, a1), (r0, a1)]:
                pts.append((c + math.cos(aa) * rr, c + math.sin(aa) * rr))
            # Fuger: trekk hjørnene litt inn mot midten av hella.
            mx = sum(p[0] for p in pts) / 4
            mz = sum(p[1] for p in pts) / 4
            top = [bm.verts.new(B((mx + (p[0] - mx) * 0.985, 0.0 + rng.uniform(-0.006, 0.004), mz + (p[1] - mz) * 0.985))) for p in pts]
            bot = [bm.verts.new(B((mx + (p[0] - mx) * 0.985, -0.14, mz + (p[1] - mz) * 0.985))) for p in pts]
            bm.faces.new(top)
            bm.faces.new(bot[::-1])
            for i in range(4):
                j = (i + 1) % 4
                bm.faces.new((top[i], bot[i], bot[j], top[j]))
            bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
            ob = link("ring_slab", bm, s.dark if k == 0 or (k == 1 and q % 2 == 0) else s.floor)
            town.uv_box(ob)
    # Ribber langs kuppelen og pilastre under dem.
    nrib = 12
    for k in range(nrib):
        a = 2 * math.pi * (k + 0.5) / nrib
        dirv = Vector((math.cos(a), 0, math.sin(a)))
        if abs(dirv.z) > 0.97:
            continue
        base = Vector((c, 0, c)) + dirv * (R - 0.2)
        pier(base.x, base.z, spring, s, w=0.48)
        pts = []
        for q in range(15):
            t = q / 14 * math.pi / 2
            p = Vector((c, spring, c)) + dirv * math.cos(t) * (R - 0.05) + Vector((0, 1, 0)) * math.sin(t) * (R - 0.05)
            pts.append(p)
        for p, q in zip(pts, pts[1:]):
            d = q - p
            nrm = -(((p + q) / 2) - Vector((c, spring, c))).normalized()
            eblock("dome_rib", (d.length + 0.02, 0.2, 0.32), s.ashlar, at=(p + q) / 2 + nrm * 0.08, r=basis(d, nrm), bevel=0.02)
        collide_at((base.x, spring / 2, base.z), (0.6, spring, 0.6), -a)
    # Slutten i toppen av kuppelen.
    erevolve("boss_keystone", [(0.0, spring + R - 0.5), (0.7, spring + R - 0.3), (0.5, spring + R + 0.1)], s.ashlar, at=(c, 0, c), n=16)
    plinth_ring = 40
    for k in range(plinth_ring):
        a0 = 2 * math.pi * k / plinth_ring
        dirv = Vector((math.cos(a0 + math.pi / plinth_ring), 0, math.sin(a0 + math.pi / plinth_ring)))
        if abs(dirv.z) > 0.972:
            continue
        p = Vector((c, 0.16, c)) + dirv * (R - 0.04)
        eblock("plinth", (2 * math.pi * R / plinth_ring - 0.02, 0.32, 0.14), s.ashlar, at=p, r=yaw_m(math.atan2(-dirv.x, -dirv.z) + math.pi / 2 + math.pi / 2), bevel=0.025)
    # Murene rundt som kollidere (24 bokser), med åpninger i dørene.
    for k in range(24):
        a = 2 * math.pi * (k + 0.5) / 24
        dirv = Vector((math.cos(a), 0, math.sin(a)))
        if abs(dirv.z) > 0.96:
            continue
        p = Vector((c, 0, c)) + dirv * (R + 0.35)
        collide_at((p.x, 5.0, p.z), (2.6, 10.0, 0.7), -a + math.pi / 2)
    collide(0, 20, -0.5, 0, 0, 20)
    for z0, z1 in ((0.0, 2.4), (17.6, 20.0)):
        collide(7.0, 8.6, -0.5, 6.0, z0, z1)
        collide(11.4, 13.0, -0.5, 6.0, z0, z1)
        slabs(rng, [(8.6, 11.4, z0 if z0 == 0 else 19.05, 0.95 if z0 == 0 else z1)], s, size=(0.45, 0.8))
    for x, z in [(c - 5.6, c - 5.6), (c + 5.6, c - 5.6), (c - 5.6, c + 5.6), (c + 5.6, c + 5.6)]:
        brazier(x, z, s)
    for j in range(5):
        for i in range(5):
            cx, cz = (i + 0.5) * CELL, (j + 0.5) * CELL
            if math.hypot(cx - c, cz - c) > 8.2:
                DATA["floor"][j][i] = None
            DATA["ceil"][j][i] = spring + 3.0
    spot("boss", (c, 0.0, c))
    for k in range(8):
        a = 2 * math.pi * (k + 0.5) / 8 + 0.2
        dirv = Vector((math.cos(a), 0, math.sin(a)))
        if abs(dirv.z) > 0.9:
            continue
        p = Vector((c, 0, c)) + dirv * (R - 0.4)
        spot("wall", (p.x, p.z, -dirv.x, -dirv.z))
    for x, z in [(c + 3.5, c), (c - 3.5, c), (c, c + 3.5), (c, c - 3.5)]:
        spot("floor", (x, 0.0, z))
    for k in range(12):
        a = k * 0.52
        spot("loot", (c + math.cos(a) * 1.6, 0.0, c + math.sin(a) * 1.6))
    bones(rng, c + 7.0, c + 2.0, s, 6)
    bones(rng, c - 6.5, c - 3.0, s, 5)
    bones(rng, c - 2.0, c + 7.2, s, 4)
    return m


# --- Tilmurt dør og vindeltrappa ------------------------------------------


def seal():
    """En tilmurt dørbue: skift av hogd stein satt inn i åpningen (origo i
    celleveggen, lokal +z inn i rommet), og løse steiner foran."""
    s = Style()
    rng = random.Random(91)
    reset(1, 1)
    DATA["floor"] = [[None]]
    y = 0.0
    k = 0
    face = T - 0.06
    while y < SPRING + HW - 0.05:
        h = min(rng.uniform(0.26, 0.34), SPRING + HW - y)
        ym = y + h / 2
        if ym > SPRING:
            half = math.sqrt(max(0.0, HW * HW - (ym - SPRING) ** 2))
        else:
            half = HW
        x = -half - 0.02
        first = True
        while x < half + 0.02 - 0.05:
            w = rng.uniform(0.38, 0.7) * (0.6 if first and k % 2 else 1.0)
            first = False
            w = min(w, half + 0.04 - x)
            if w < 0.08:
                break
            out = rng.uniform(-0.015, 0.02)
            eblock("seal_stone", (w - 0.014, h - 0.014, 0.4), s.ashlar, at=(x + w / 2, ym, face - 0.2 + out), bevel=0.02)
            x += w
        y += h
        k += 1
    rubble(rng, 0.3, face + 0.5, 0.5, 7, s, (0.12, 0.32))
    collide(-HW, HW, 0, SPRING + HW, 0.0, face)
    return None


def spiral():
    """Vindeltrappa ned til neste etasje: en sylindrisk sjakt med kuppel,
    spindel i midten og 72 trinn i tre omganger; inngangen øverst og
    utgangen nederst vender begge mot −z. Origo midt i sjakta på det øverste
    gulvet."""
    s = Style()
    rng = random.Random(101)
    reset(1, 1)
    DATA["floor"] = [[None]]
    top_h = 4.0
    m = mass(-2.0, 2.0, -DROP - 0.4, top_h + RS + 0.4, -2.0, 2.0, s)
    ccyl(0, 0, -DROP - 0.12, top_h, RS)
    csphere(0, top_h, 0, RS)
    for y0 in (0.0, -DROP):
        cbox(-1.2, 1.2, y0 - 0.12, y0 + 1.3, -2.05, -RS + 0.1)
        cbarrel("z", -2.05, -RS + 0.1, 0.0, y0 + 2.5 - 1.2, 1.2)
    carve(m)
    # Spindelen.
    erevolve("newel", [(0.3, -DROP), (0.28, -DROP + 0.3), (0.28, top_h - 0.4), (0.36, top_h)], s.ashlar, at=(0, 0, 0), n=16)
    collide(-0.28, 0.28, -DROP, top_h, -0.28, 0.28)
    # Avsatsene: øverst ved inngangen (vinkel −130°..−50°), nederst hele bunnen.
    a_start, a_end = math.radians(-50.0), math.radians(-130.0 + 1080.0)
    total = a_end - a_start
    da = total / STEPS
    rise = DROP / STEPS

    def wedge(name, a0, a1, y_top, thick, r0=0.26, r1=RS + 0.02, mat=None):
        bm = bmesh.new()
        n = max(2, int(abs(a1 - a0) / math.radians(8)) + 1)
        inner_t, outer_t, inner_b, outer_b = [], [], [], []
        for q in range(n + 1):
            a = a0 + (a1 - a0) * q / n
            for lst, rr, yy in [(inner_t, r0, y_top), (outer_t, r1, y_top), (inner_b, r0, y_top - thick), (outer_b, r1, y_top - thick)]:
                lst.append(bm.verts.new(B((math.cos(a) * rr, yy, math.sin(a) * rr))))
        for q in range(n):
            bm.faces.new((inner_t[q], outer_t[q], outer_t[q + 1], inner_t[q + 1]))
            bm.faces.new((inner_b[q + 1], outer_b[q + 1], outer_b[q], inner_b[q]))
            bm.faces.new((outer_t[q], outer_b[q], outer_b[q + 1], outer_t[q + 1]))
        bm.faces.new((inner_t[0], inner_b[0], outer_b[0], outer_t[0]))
        bm.faces.new((outer_t[n], outer_b[n], inner_b[n], inner_t[n]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        ob = link(name, bm, mat or s.ashlar)
        town.uv_box(ob)
        return ob

    wedge("landing_top", math.radians(-130), a_start + 0.02, 0.0, 0.3, mat=s.floor)
    hull([(math.cos(a) * rr, y, math.sin(a) * rr) for a in (math.radians(-130), math.radians(-90), a_start) for rr in (0.2, RS + 0.1) for y in (0.0, -0.4)])
    slabs(rng, [(-RS, RS, -RS, RS)], s, size=(0.5, 0.8), y=-DROP)
    for y0 in (0.0, -DROP):
        slabs(rng, [(-1.2, 1.2, -2.0, -RS + 0.1)], s, size=(0.4, 0.7), y=y0)
        collide(-1.2, 1.2, y0 - 0.5, y0, -2.05, -1.5)
    collide(-2.0, 2.0, -DROP - 0.5, -DROP, -2.0, 2.0)
    for k in range(STEPS):
        a0 = a_start + k * da
        y = -(k + 1) * rise
        wedge("step", a0, a0 + da * 1.04, y, 0.32 + rise)
        # Nesen på trinnet: en avfaset list.
        mid = a0 + 0.01
        p = Vector((math.cos(mid), 0, math.sin(mid)))
        ebeam("nosing", Vector((p.x * 0.3, y - 0.02, p.z * 0.3)), Vector((p.x * (RS - 0.02), y - 0.02, p.z * (RS - 0.02))), 0.06, 0.05, s.ashlar)
    # Kolliderne: skrå skall over tre og tre trinn.
    for k in range(0, STEPS, 3):
        a0 = a_start + k * da
        a1 = a_start + min(k + 3, STEPS) * da
        y0 = -k * rise
        y1 = -min(k + 3, STEPS) * rise
        pts = []
        for a, y in [(a0, y0), (a1, y1)]:
            for rr in (0.25, RS + 0.05):
                pts.append((math.cos(a) * rr, y, math.sin(a) * rr))
                pts.append((math.cos(a) * rr, y - 0.6, math.sin(a) * rr))
        hull(pts)
    # Sjaktmuren: 20 bokser rundt, med åpninger for inngangen og utgangen.
    for k in range(20):
        a = 2 * math.pi * (k + 0.5) / 20
        p = (math.cos(a) * (RS + 0.3), math.sin(a) * (RS + 0.3))
        yaw = -a + math.pi / 2
        near_door = abs(math.atan2(math.sin(a + math.pi / 2), math.cos(a + math.pi / 2))) < math.radians(42)
        if near_door:
            collide_at((p[0], (-DROP + 2.6) / 2, p[1]), (0.95, DROP - 2.6, 0.6), yaw)
            collide_at((p[0], (2.6 + top_h + 1.0) / 2, p[1]), (0.95, top_h + 1.0 - 2.6, 0.6), yaw)
        else:
            collide_at((p[0], (-DROP + top_h + 1.0) / 2, p[1]), (0.95, DROP + top_h + 1.0, 0.6), yaw)
    # Fakler hver halve omgang på ytterveggen, over trinnene.
    for k in range(6, STEPS, 12):
        a = a_start + (k + 0.5) * da
        y = -(k + 1) * rise
        p = (math.cos(a) * RS, math.sin(a) * RS)
        torch(p, (-math.cos(a), -math.sin(a)), s, y=y + 1.9)
    torch((math.cos(math.radians(-150)) * RS, math.sin(math.radians(-150)) * RS), (-math.cos(math.radians(-150)), -math.sin(math.radians(-150))), s, y=-DROP + 1.9)
    return m


# --- Pynten per etasje ------------------------------------------------------
#
# Modeller hver etasje setter ut på de ledige stedene i rommene og langs
# gangene (`dungeon::theme`). Pynt på muren har origo ved foten av muren
# og lokal +z inn i rommet; pynt på gulvet har origo midt på.


def deco_shelf():
    """Arkivhylle (forretningslogikken, klassekomponentene): eikereol med
    fire hyller, protokoller, ruller og skrin."""
    s = Style()
    rng = random.Random(201)
    reset(1, 1)
    w, h, d = 1.6, 2.05, 0.42
    for sx in (-1, 1):
        for sz in (0.03, d - 0.03):
            eblock("shelf_post", (0.06, h, 0.06), s.wood, at=(sx * (w / 2 - 0.03), h / 2, sz), bevel=0.008)
    eblock("shelf_back", (w - 0.04, h - 0.1, 0.02), s.wood, at=(0, h / 2, 0.012), bevel=0.004)
    books = [layer("leather", (0.55, 0.25, 0.15)), layer("leather", (0.3, 0.32, 0.4)), layer("linen", (0.7, 0.6, 0.4)), layer("leather", (0.25, 0.3, 0.18))]
    for k, y in enumerate((0.08, 0.58, 1.08, 1.58)):
        eblock("shelf_board", (w - 0.08, 0.035, d - 0.02), s.wood, at=(0, y, d / 2), bevel=0.006)
        x = -w / 2 + 0.08
        while x < w / 2 - 0.12:
            kind = rng.random()
            if kind < 0.65:
                bw = rng.uniform(0.04, 0.09)
                bh = rng.uniform(0.24, 0.38)
                tilt = rng.uniform(-0.12, 0.12) if rng.random() < 0.2 else 0.0
                eblock("book", (bw, bh, rng.uniform(0.2, 0.3)), rng.choice(books), at=(x + bw / 2, y + 0.02 + bh / 2, d / 2), r=Matrix.Rotation(tilt, 3, "Z"), bevel=0.006)
                x += bw + 0.004
            elif kind < 0.85:
                # Ruller i en haug.
                for q in range(3):
                    bpy.ops.mesh.primitive_cylinder_add(vertices=10, radius=0.035, depth=0.32, location=B((x + 0.05 + q * 0.03, y + 0.055 + 0.06 * (q == 2), d / 2)), rotation=(math.pi / 2, 0, 0))
                    ob = bpy.context.active_object
                    ob.data.materials.append(layer("linen", (0.85, 0.78, 0.6)))
                    town.uv_box(ob)
                    PIECES.append(ob)
                x += 0.16
            else:
                eblock("casket", (0.22, 0.14, 0.2), s.wood, at=(x + 0.11, y + 0.09, d / 2), bevel=0.012)
                eblock("casket_band", (0.226, 0.03, 0.206), s.iron, at=(x + 0.11, y + 0.12, d / 2), bevel=0.004)
                x += 0.25
    eblock("shelf_top", (w + 0.04, 0.05, d + 0.02), s.wood, at=(0, h + 0.025, d / 2), bevel=0.01)
    collide(-w / 2, w / 2, 0, h, 0, d)


def deco_sacks():
    """Sekker og en kasse (forretningslogikken): kornsekker som har sunket
    sammen, bundet med tau."""
    s = Style()
    rng = random.Random(202)
    reset(1, 1)
    sack = layer("linen", (0.72, 0.62, 0.45))
    for k, (x, z) in enumerate([(-0.45, 0.35), (0.1, 0.32), (-0.15, 0.75)]):
        h = rng.uniform(0.55, 0.7)
        erevolve("sack", [(0.22, 0.0), (0.3, 0.08), (0.31, h * 0.45), (0.24, h * 0.85), (0.08, h), (0.05, h + 0.06), (0.09, h + 0.12)], sack, at=(x, 0, z), n=14)
        erevolve("sack_tie", [(0.07, h + 0.0), (0.075, h + 0.03), (0.07, h + 0.06)], s.rope, at=(x, 0, z), n=8, cap=False)
    crate(0.55, 0.45, s, 0.2, 0.6)
    collide(-0.8, 0.9, 0, 0.75, 0.0, 1.05)


def deco_banner():
    """Et banner på muren (klassekomponentene): smijernsstang, tøy med kant
    og et merke av messing, og dusker nederst."""
    s = Style()
    reset(1, 1)
    cloth = layer("wool", (0.16, 0.24, 0.5))
    hem = layer("wool", (0.75, 0.6, 0.25))
    y0, y1 = 0.85, 2.75
    ebeam("banner_rod", (-0.62, y1 + 0.05, 0.16), (0.62, y1 + 0.05, 0.16), 0.03, 0.03, s.iron)
    for sx in (-1, 1):
        ebeam("banner_arm", (sx * 0.5, y1 + 0.05, 0.0), (sx * 0.5, y1 + 0.05, 0.18), 0.03, 0.03, s.iron)
    # Tøyet: litt bølget, med tykkelse.
    n = 8
    for k in range(n):
        x0 = -0.5 + k / n
        x1 = -0.5 + (k + 1) / n
        z = 0.16 + 0.025 * math.sin(k * 1.3)
        eblock("banner_cloth", (x1 - x0 + 0.004, y1 - y0, 0.02), cloth, at=((x0 + x1) / 2, (y0 + y1) / 2, z), r=yaw_m(0.08 * math.cos(k * 1.3)), bevel=0.003)
    for x in (-0.47, 0.47):
        eblock("banner_hem", (0.05, y1 - y0, 0.026), hem, at=(x, (y0 + y1) / 2, 0.16), bevel=0.003)
    eblock("banner_hem_low", (1.0, 0.06, 0.026), hem, at=(0, y0 + 0.03, 0.16), bevel=0.003)
    # Merket: en ring og et kors av messing.
    erevolve("banner_ring", [(0.2, 0.0), (0.2, 0.02)], s.gold, at=(0, 0, 0), n=20, cap=False)
    PIECES[-1].location = B((0, 2.0, 0.19))
    PIECES[-1].rotation_euler = (math.pi / 2, 0, 0)
    eblock("banner_mark_v", (0.06, 0.5, 0.02), s.gold, at=(0, 2.0, 0.19), bevel=0.004)
    eblock("banner_mark_h", (0.34, 0.06, 0.02), s.gold, at=(0, 2.05, 0.19), bevel=0.004)
    for x in (-0.35, -0.12, 0.12, 0.35):
        erevolve("tassel", [(0.0, y0 - 0.16), (0.03, y0 - 0.1), (0.015, y0 - 0.02), (0.01, y0)], hem, at=(x, 0, 0.16), n=6)


def deco_moss():
    """Mose og sopp ved foten av muren (kloakken), med salt og fukt."""
    s = Style()
    rng = random.Random(204)
    reset(1, 1)
    moss = layer("sod", (0.45, 0.62, 0.35))
    for k in range(9):
        x = rng.uniform(-0.8, 0.8)
        r = rng.uniform(0.12, 0.3)
        erevolve("moss", [(0.0, 0.0), (r, 0.0), (r * 0.8, r * 0.35), (0.0, r * 0.45)], moss, at=(x, 0, r * 0.5), n=10)
    for k in range(6):
        x = rng.uniform(-0.7, 0.7)
        h = rng.uniform(0.05, 0.14)
        z = rng.uniform(0.08, 0.35)
        erevolve("mushroom_stem", [(0.012, 0.0), (0.01, h)], s.bone, at=(x, 0, z), n=6)
        erevolve("mushroom_cap", [(0.0, h - 0.01), (0.045, h), (0.03, h + 0.02), (0.0, h + 0.03)], layer("leather", (0.6, 0.4, 0.2)), at=(x, 0, z), n=8)


def deco_puddle():
    """En pytt med gjørme rundt (kloakken)."""
    rng = random.Random(205)
    reset(1, 1)
    mud = layer("gravel", (0.35, 0.3, 0.24))
    water = plain("water", (0.04, 0.07, 0.08), 0.05)
    for name, mat, scale, y in [("mud", mud, 1.0, 0.004), ("puddle", water, 0.78, 0.012)]:
        bm = bmesh.new()
        n = 14
        c = bm.verts.new(B((0, y, 0)))
        ring = []
        for k in range(n):
            a = 2 * math.pi * k / n
            r = scale * (0.75 + 0.3 * math.sin(a * 3 + 1.0) + rng.uniform(-0.05, 0.05))
            ring.append(bm.verts.new(B((math.cos(a) * r, y, math.sin(a) * r * 0.75))))
        for k in range(n):
            bm.faces.new((c, ring[(k + 1) % n], ring[k]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        ob = link(name, bm, mat)
        town.uv_box(ob)
        for f in ob.data.polygons:
            if f.normal.z < 0:
                f.flip()


def deco_roots():
    """Røtter som har vokst ned gjennom hvelvet (kloakken). Origo i
    hvelvet; røttene henger 0,8–1,5 m ned."""
    rng = random.Random(206)
    reset(1, 1)
    bark = layer("bark", (0.7, 0.6, 0.5))
    for k in range(7):
        p = Vector((rng.uniform(-0.4, 0.4), 0.1, rng.uniform(-0.4, 0.4)))
        length = rng.uniform(0.8, 1.5)
        r = rng.uniform(0.02, 0.05)
        steps = 5
        for q in range(steps):
            d = Vector((rng.uniform(-0.12, 0.12), -length / steps, rng.uniform(-0.12, 0.12)))
            ebeam("root", p, p + d, r, r, bark)
            p = p + d
            r *= 0.75
    # Over hodet: røttene henger ned til 2,3 m over gulvet.
    collide(-0.5, 0.5, -1.2, 0.3, -0.5, 0.5)


def deco_skulls():
    """Beinhylle (tabellayout-katakombene): et rutenett av nisjer i hogd
    stein, fire og fire, med en hodeskalle i hver, og lys på toppen."""
    s = Style()
    rng = random.Random(207)
    reset(1, 1)
    stone = layer("rock", (0.5, 0.48, 0.46))
    w, h, d = 1.6, 1.9, 0.4
    cols, rows = 4, 4
    cw, ch = (w - 0.1) / cols, (h - 0.2) / rows
    eblock("ossuary_back", (w, h, 0.06), stone, at=(0, h / 2, 0.03), bevel=0.01)
    for i in range(cols + 1):
        x = -w / 2 + 0.05 + i * cw
        eblock("ossuary_post", (0.06, h, d), stone, at=(x, h / 2, d / 2), bevel=0.012)
    for j in range(rows + 1):
        y = 0.1 + j * ch
        eblock("ossuary_shelf", (w, 0.06, d), stone, at=(0, y, d / 2), bevel=0.012)
    for i in range(cols):
        for j in range(rows):
            x = -w / 2 + 0.05 + (i + 0.5) * cw
            y = 0.13 + j * ch
            if rng.random() < 0.85:
                skull(x, y, d * 0.55, s, rng.uniform(-0.4, 0.4))
            if rng.random() < 0.5:
                ebeam("ossuary_bone", (x - 0.12, y + 0.02, d * 0.25), (x + 0.12, y + 0.025, d * 0.3), 0.03, 0.03, s.bone)
    candles(rng, 0.4, d / 2, s, y0=h + 0.03, n=3, glow=True)
    collide(-w / 2, w / 2, 0, h, 0, d)


def deco_bonepile():
    """En haug av knokler og hodeskaller (katakombene)."""
    s = Style()
    rng = random.Random(208)
    reset(1, 1)
    for layer_y in (0.0, 0.07, 0.14):
        bones(rng, 0.0, 0.0, s, n=6, y0=layer_y)
    for k in range(3):
        skull(rng.uniform(-0.3, 0.3), 0.18, rng.uniform(-0.3, 0.3), s, rng.uniform(0, 6))


def deco_lava():
    """Sprekker med glødende lava i gulvet (blink-dypet): mørke heller av
    berg med glød imellom."""
    rng = random.Random(209)
    reset(1, 1)
    rock = layer("rock", (0.22, 0.18, 0.17))
    glow = plain("fire_lava", (1.0, 0.36, 0.08), 0.6)
    bpy.ops.mesh.primitive_plane_add(size=2.2, location=B((0, 0.004, 0)))
    ob = bpy.context.active_object
    ob.data.materials.append(glow)
    town.uv_box(ob)
    PIECES.append(ob)
    # Hellene: et skjevt rutenett av flak med fuger der lavaen lyser.
    for i in range(4):
        for j in range(4):
            x = -0.83 + i * 0.55 + rng.uniform(-0.05, 0.05)
            z = -0.83 + j * 0.55 + rng.uniform(-0.05, 0.05)
            sz = rng.uniform(0.38, 0.48)
            eblock("lava_crust", (sz, 0.05, sz * rng.uniform(0.8, 1.0)), rock, at=(x, 0.02, z), r=yaw_m(rng.uniform(-0.4, 0.4)), bevel=0.02)
    light((0, 0.4, 0), (1.0, 0.32, 0.08), 1.6, 6.0)


def deco_embers():
    """En haug glødende slagg og glør (blink-dypet), med små flammer."""
    s = Style()
    rng = random.Random(210)
    reset(1, 1)
    slag = layer("rock", (0.2, 0.17, 0.16))
    for k in range(10):
        a = rng.uniform(0, math.tau)
        r = rng.uniform(0, 0.45)
        k_ = rng.uniform(0.12, 0.3)
        eblock("slag", (k_, k_ * 0.6, k_ * 0.8), slag if k % 3 else s.ember, at=(math.cos(a) * r, k_ * 0.25, 0.5 + math.sin(a) * r), r=yaw_m(rng.uniform(0, 6)), bevel=0.03)
    for k in range(3):
        erevolve("ember_flame", [(0.06, 0.05), (0.07, 0.15), (0.03, 0.3), (0.0, 0.42)], s.flame, at=(rng.uniform(-0.25, 0.25), 0, 0.5 + rng.uniform(-0.2, 0.2)), n=8)
    light((0, 0.6, 0.5), (1.0, 0.35, 0.1), 1.8, 7.0)
    collide(-0.5, 0.5, 0, 0.4, 0.05, 0.95)


# --- Eksport ------------------------------------------------------------------

MODELS = {
    "dng_hall_i": lambda: corridor("NS", 0, 1),
    "dng_hall_i2": lambda: corridor("NS", 1, 2),
    "dng_hall_i3": lambda: corridor("NS", 2, 3),
    "dng_hall_l": lambda: corridor("NE", 0, 4),
    "dng_hall_t": lambda: corridor("NES", 0, 5),
    "dng_hall_x": lambda: corridor("NESW", 0, 6),
    "dng_hall_end": lambda: corridor("S", 0, 7),
    "dng_seal": seal,
    "dng_spiral": spiral,
    "dng_start": start_room,
    "dng_gate": lambda: gate_room(True),
    "dng_landing": lambda: gate_room(False),
    "dng_hall": hall_room,
    "dng_long": long_room,
    "dng_cistern": cistern_room,
    "dng_gallery": gallery_room,
    "dng_chapel": chapel_room,
    "dng_treasury": treasury_room,
    "dng_arena": arena_room,
    "dngp_shelf": deco_shelf,
    "dngp_sacks": deco_sacks,
    "dngp_banner": deco_banner,
    "dngp_moss": deco_moss,
    "dngp_puddle": deco_puddle,
    "dngp_roots": deco_roots,
    "dngp_skulls": deco_skulls,
    "dngp_bonepile": deco_bonepile,
    "dngp_lava": deco_lava,
    "dngp_embers": deco_embers,
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    os.makedirs(OUT, exist_ok=True)
    for name in names:
        clear()
        PIECES.clear()
        CUTS.clear()
        MODELS[name]()
        path = f"{OUT}/{name}.glb"
        if os.environ.get("FRISTAD_JSON_ONLY"):
            write_json(path)
            continue
        town.finish(path, ao_samples=96, max_edge=1.2)
        write_json(path)
