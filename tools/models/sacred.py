"""Skriptoriet, stavkirka og krypten i den malte stilen (docs/ny-stil.md),
bygd med byggesettet (kit.py). Kjør:

    python3 tools/models/sacred.py [scriptorium stave_church crypt]

Gir utsida og innsida (`<navn>_interior.glb`, tegnes bare nær) til
Skriptoriet og stavkirka, med `<navn>_inside.json` (kolliderne, lysene og
lykta for begge), og krypten (`crypt.glb`, uten innside: døra er stengt, og
byen setter kollideren, siktemålet og fakkellyset i `town::crypt`).

Skriptoriet: tykke murer av hogd stein på en sokkel, rundbuede vinduer med
buestein, trappede støttepilarer, gavler som står opp over skifertaket med
dekkheller, et rundvindu og klokkestolen på mønet foran. Inne: flatt
bjelketak, skrivepulter under vinduene, bokhyller, lesepult med lenkebok,
lysestaker og lysekroner, og biblioteksgalleriet langs bakmuren med trapp
opp i vestenden.

Stavkirka, i lag som Borgund: steinfundament, svalgang med buer mellom
stolpene, sideskip, høyt midtskip med bratt tak og drakehoder, takrytter
med spir, kor og apsis. Alle takene er tjærede skjellspon som henger litt.
Døra står slått inn. Inne: staver med andreaskors og knær, himling over
sideskipet, triumfbjelke med krusifiks, alter i koret, døpefont og
veggbenker.

Krypten: et lite, lavt gravkapell av forvitret stein med mose, bratt
steinhelletak som har sunket, gavl med kors, jernbeslått dør (stengt) i en
rundbue, støttepilarer og fakkelholdere ved døra.

Akser som i town.py: X til høyre, −Y fram (døra), Z opp, origo midt på
grunnflaten (krypten: fronten i y = −1, som før)."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import CARVED, at_wall, box, slab_grid, stick, wall_colliders, wall_frame  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from town import collide, layer, light, plain  # noqa: E402

OUT = "assets/models/town"
PIECES = town.PIECES


# --- Felles former ------------------------------------------------------------


def lin(a, b, n):
    return [a + (b - a) * k / (n - 1) for k in range(n)]


def prism(name, frame, pts, t0, t1, mat, bevel=0.0):
    """Et polygon i veggens plan (s langs veggen, z opp) trukket ut på tvers
    fra t0 til t1, med uv i meter (som veggen rundt)."""
    bm = bmesh.new()
    a = [bm.verts.new(at_wall(frame, s, t0, z)) for s, z in pts]
    b = [bm.verts.new(at_wall(frame, s, t1, z)) for s, z in pts]
    bm.faces.new(a[::-1])
    bm.faces.new(b)
    n = len(pts)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((a[i], a[j], b[j], b[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = kit._object(name, bm, mat)
    kit._bevel(ob, bevel)
    town.uv_box(ob)
    return ob


def lathe(name, prof, n, at, mat, cap=True, a0=0.0, arc=2 * math.pi):
    """Dreid form: profilet (r, z) rundt en loddrett akse i `at`. Med `arc`
    under en hel runde blir det en åpen bue (apsis, halvkjegle)."""
    bm = bmesh.new()
    full = arc >= 2 * math.pi - 1e-6
    m = n if full else n + 1
    rings = []
    for r, z in prof:
        if r < 1e-6:
            rings.append([bm.verts.new((at[0], at[1], at[2] + z))])
        else:
            rings.append([bm.verts.new((at[0] + r * math.cos(a0 + arc * i / n), at[1] + r * math.sin(a0 + arc * i / n), at[2] + z)) for i in range(m)])
    for A, B in zip(rings, rings[1:]):
        if len(A) == 1 and len(B) == 1:
            continue
        for i in range(n):
            j = (i + 1) % m
            if len(A) == 1:
                bm.faces.new((A[0], B[i], B[j]))
            elif len(B) == 1:
                bm.faces.new((A[i], A[j], B[0]))
            else:
                bm.faces.new((A[i], A[j], B[j], B[i]))
    if cap and full:
        if len(rings[0]) > 1:
            bm.faces.new(rings[0][::-1])
        if len(rings[-1]) > 1:
            bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = kit._object(name, bm, mat)
    town.uv_box(ob)
    return ob


def ring(name, c, r, minor, mat, segs=12):
    """En jernring (lysekrone, dørring) liggende i xy-planet."""
    bpy.ops.mesh.primitive_torus_add(major_radius=r, minor_radius=minor, major_segments=segs, minor_segments=4, location=c)
    ob = bpy.context.active_object
    ob.name = name
    ob.data.materials.append(mat)
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def transform(obs, M):
    """Flytt og drei ferdige deler (matrisene oppdateres først)."""
    bpy.context.view_layer.update()
    for o in obs:
        o.matrix_world = M @ o.matrix_world


def turned(before):
    """Delene laget etter `before` dreies 90° om z: (x, y) → (−y, x). Tak
    bygd med mønet langs x får mønet langs y."""
    transform(PIECES[before:], Matrix.Rotation(math.pi / 2, 4, "Z"))


def ridge_roof(p, y0, y1, half, z_eave, pitch, barges=True, **kw):
    """Saltak med mønet langs y (gavlene fram og bak) fra kit.gable_roof.
    Uten `barges` tas vindskiene bort (taket ligger inne mellom gavler av
    stein). Gir mønehøyden."""
    before = len(PIECES)
    ridge = kit.gable_roof(p, y0, y1, -half, half, z_eave, pitch, **kw)
    if not barges:
        for ob in [o for o in PIECES[before:] if o.name.startswith("barge")]:
            PIECES.remove(ob)
            bpy.data.objects.remove(ob, do_unlink=True)
    turned(before)
    return ridge


def skirt(name, inner, outer, z_in, z_out, thick, mat, n=4, sag=0.1, kick=0.14):
    """Én side av et pulttak rundt en kropp (svalgang, sideskip): fra den
    indre kanten (to punkter, høyt) ned til den ytre (lavt), med takfoten
    som henger litt i midten og bretter seg ut. Hjørnene møtes i gradene,
    så sidene deler kantpunktene."""
    ia, ib = Vector((*inner[0], 0)), Vector((*inner[1], 0))
    oa, ob_ = Vector((*outer[0], 0)), Vector((*outer[1], 0))
    d = (ib - ia).normalized()
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    rows = ((0.0, 0.0, 0.0), (0.7, 0.45, 0.0), (1.0, 1.0, 1.0))
    g = {}
    for i in range(n + 1):
        u = i / n
        pi_, po = ia.lerp(ib, u), oa.lerp(ob_, u)
        dip = sag * math.sin(math.pi * u)
        for j, (f, sg, kk) in enumerate(rows):
            q = pi_.lerp(po, f)
            z = z_in + (z_out - z_in) * f - dip * sg + kick * kk
            g[(i, j, 0)] = bm.verts.new((q.x, q.y, z))
            g[(i, j, 1)] = bm.verts.new((q.x, q.y, z - thick))
    last = len(rows) - 1
    for i in range(n):
        for j in range(last):
            bm.faces.new([g[(i, j, 0)], g[(i + 1, j, 0)], g[(i + 1, j + 1, 0)], g[(i, j + 1, 0)]])
            bm.faces.new([g[(i, j, 1)], g[(i, j + 1, 1)], g[(i + 1, j + 1, 1)], g[(i + 1, j, 1)]])
        for j in (0, last):
            bm.faces.new([g[(i, j, 0)], g[(i + 1, j, 0)], g[(i + 1, j, 1)], g[(i, j, 1)]])
    for i in (0, n):
        for j in range(last):
            bm.faces.new([g[(i, j, 0)], g[(i, j + 1, 0)], g[(i, j + 1, 1)], g[(i, j, 1)]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    # Uv: u langs kanten, v nedover takflaten (sponet renner ned).
    for f in bm.faces:
        for lp in f.loops:
            c = Vector((lp.vert.co.x, lp.vert.co.y, 0))
            along = (c - ia).dot(d)
            across = ((c - ia) - d * along).length
            lp[uvl].uv = (along, -across * 1.25)
    return kit._object(name, bm, mat)


def arch_pts(s, zs, r, a0, a1, n):
    return [(s + r * math.cos(a), zs + r * math.sin(a)) for a in lin(a0, a1, n)]


def arch_fill(frame, s, z0, w, h, thick, mat):
    """Svikkelen over en rundbue: muren mellom buen og den rette kanten øverst
    i hullet (`slab_grid` skjærer et rektangel)."""
    r = w / 2
    zs, top = z0 + h - r, z0 + h
    prism("spandrel", frame, [(s - r, top)] + arch_pts(s, zs, r, math.pi, math.pi / 2, 6), -thick / 2, thick / 2, mat)
    prism("spandrel", frame, [(s + r, top)] + arch_pts(s, zs, r, 0.0, math.pi / 2, 6), -thick / 2, thick / 2, mat)


def voussoirs(frame, s, zs, r, out, thick, mat, n=7, depth=0.28, proud=0.08, bevel=0.02):
    """Buesteinene rundt en rundbue, med sluttsteinen litt større."""
    for k in range(n):
        a0 = math.pi * k / n + 0.012
        a1 = math.pi * (k + 1) / n - 0.012
        ro = r + depth + (0.07 if k == n // 2 else 0.0)
        pts = [(s + r * math.cos(a0), zs + r * math.sin(a0)), (s + ro * math.cos(a0), zs + ro * math.sin(a0)), (s + ro * math.cos(a1), zs + ro * math.sin(a1)), (s + r * math.cos(a1), zs + r * math.sin(a1))]
        prism("voussoir", frame, pts, out * (thick / 2 - 0.05), out * (thick / 2 + proud), mat, bevel=bevel)


def arch_window(p, frame, s, z0, w, h, thick, out, wall, stone, lead=True, sill=True):
    """Rundbuet vindu i en mur som har hullet (s ± w/2, z0..z0+h): svikkelen,
    buesteinene, et sålbord og blyglass midt i muren (gløder om natta)."""
    r = w / 2
    zs = z0 + h - r
    arch_fill(frame, s, z0, w, h, thick, wall)
    voussoirs(frame, s, zs, r, out, thick, stone, n=7 if w > 0.7 else 5, depth=0.24 if w > 0.7 else 0.18)
    if sill:
        stick("sill", at_wall(frame, s - r - 0.16, out * (thick / 2 + 0.03), z0 - 0.07), at_wall(frame, s + r + 0.16, out * (thick / 2 + 0.03), z0 - 0.07), 0.24, 0.14, stone, bevel=0.025)
    gt = -out * thick * 0.12
    pane = prism("glass", frame, [(s - r, z0), (s + r, z0)] + arch_pts(s, zs, r, 0.0, math.pi, 9)[1:-1] + [(s - r, zs)], gt - 0.01, gt + 0.01, p.glass)
    (_, _), (dx, dy) = frame
    town.daylight(pane, (out * dy, -out * dx, 0))
    if lead:
        # Blysprosser: én loddrett og noen vannrette, gjennom ruta.
        stick("lead", at_wall(frame, s, gt, z0), at_wall(frame, s, gt, z0 + h - 0.02), 0.035, 0.03, p.iron, bevel=0)
        for k in range(1, 5):
            z = z0 + (h - 0.05) * k / 5
            hw = r if z <= zs else math.sqrt(max(r * r - (z - zs) ** 2, 0.0))
            if hw > 0.1:
                stick("lead", at_wall(frame, s - hw, gt, z), at_wall(frame, s + hw, gt, z), 0.035, 0.03, p.iron, bevel=0)


def arched_leaf(name, w, h, thick, mat):
    """Et dørblad med rund bue øverst (hele dørbildet over bladet): lokalt
    x 0..w fra hengslet, y ±thick/2, z 0..h."""
    r = w / 2
    zs = h - r
    pts = [(0.0, 0.0), (w, 0.0)] + [(r + r * math.cos(a), zs + r * math.sin(a)) for a in lin(0.0, math.pi, 10)]
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    a = [bm.verts.new((x, -thick / 2, z)) for x, z in pts]
    b = [bm.verts.new((x, thick / 2, z)) for x, z in pts]
    bm.faces.new(a[::-1])
    bm.faces.new(b)
    for i in range(len(pts)):
        j = (i + 1) % len(pts)
        bm.faces.new((a[i], a[j], b[j], b[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            u = c.x / w
            lp[uvl].uv = (u if f.normal.y <= 0 else 1 - u, c.z / h)
    return kit._object(name, bm, mat)


def buttress(name, base, out, along, w, prof, mat, bevel=0.03):
    """En trappet støttepilar: profilet (d ut fra muren, z) trukket ut langs
    muren, w bred. `base` er foten midt på murflata."""
    base, out, along = Vector(base), Vector(out), Vector(along)
    bm = bmesh.new()
    a = [bm.verts.new(base + out * d + along * (-w / 2) + Vector((0, 0, z))) for d, z in prof]
    b = [bm.verts.new(base + out * d + along * (w / 2) + Vector((0, 0, z))) for d, z in prof]
    bm.faces.new(a[::-1])
    bm.faces.new(b)
    for i in range(len(prof)):
        j = (i + 1) % len(prof)
        bm.faces.new((a[i], a[j], b[j], b[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = kit._object(name, bm, mat)
    kit._bevel(ob, bevel)
    town.uv_box(ob)
    return ob


def parapet(p, frame, L, z0, zk, apex, thick, mat, coping):
    """Gavl av stein som står opp over taket: rett opp i hjørnene (`zk`),
    skrå opp til mønet (`apex`), med dekkheller langs skråkantene."""
    prism("gable_stone", frame, [(0.0, z0), (L, z0), (L, zk), (L / 2, apex), (0.0, zk)], -thick / 2, thick / 2, mat)
    for s0, s1 in ((-0.12, L / 2 + 0.05), (L + 0.12, L / 2 - 0.05)):
        stick("coping", at_wall(frame, s0, 0, zk + 0.06), at_wall(frame, s1, 0, apex + 0.06), thick + 0.16, 0.2, coping, bevel=0.04)
    for s in (0.0, L):
        box("kneeler", tuple(at_wall(frame, s, 0, 0) + Vector((-(thick + 0.2) / 2, -(thick + 0.2) / 2, zk - 0.35))), tuple(at_wall(frame, s, 0, 0) + Vector(((thick + 0.2) / 2, (thick + 0.2) / 2, zk + 0.1))), coping, bevel=0.04)


def stone_cross(c, mat, s=1.0, tilt=0.0):
    """Et kors av stein på en gavlspiss."""
    obs = [box("cross_v", (-0.09 * s, -0.09 * s, 0), (0.09 * s, 0.09 * s, 0.95 * s), mat, bevel=0.025),
           box("cross_h", (-0.32 * s, -0.09 * s, 0.52 * s), (0.32 * s, 0.09 * s, 0.7 * s), mat, bevel=0.025)]
    transform(obs, Matrix.Translation(c) @ Matrix.Rotation(tilt, 4, "Y"))


def books(p, x0, x1, y, z, depth, rng, mats, max_h=0.3):
    """En rad bøker på en hylle (lokalt: langs x, ryggen mot −y), med
    hull og en og annen som lener seg."""
    x = x0
    obs = []
    while x < x1 - 0.06:
        if rng.random() < 0.08:
            x += rng.uniform(0.06, 0.18)
            continue
        bw = rng.uniform(0.04, 0.09)
        bh = rng.uniform(0.18, max_h)
        bd = depth * rng.uniform(0.75, 0.95)
        ob = box("book", (x, y - bd / 2, z), (x + bw, y + bd / 2, z + bh), rng.choice(mats), bevel=0)
        if rng.random() < 0.06 and x + bw + 0.12 < x1:
            ob.rotation_euler = (0, -0.3, 0)
        obs.append(ob)
        x += bw + 0.004
    return obs


def book_mats():
    return [layer("leather", c) for c in ((1.0, 0.38, 0.26), (0.4, 0.55, 0.95), (0.5, 0.8, 0.42), (1.0, 0.8, 0.42), (0.75, 0.42, 0.65), (0.95, 0.62, 0.36))]


def shelf(p, cx, cy, z, w, h, yaw, rng, rows=5, depth=0.36, fill=True):
    """Bokhylle med fronten mot lokal −y, dreid `yaw` og satt i (cx, cy, z):
    rygg, sider, hyllebord, krone og bøker."""
    before = len(PIECES)
    wood = layer("oak_planed", (0.8, 0.66, 0.56))
    box("shelf_back", (-w / 2, depth / 2 - 0.04, 0), (w / 2, depth / 2, h), wood, bevel=0)
    for sx in (-1, 1):
        box("shelf_side", (sx * w / 2 - 0.04, -depth / 2, 0), (sx * w / 2 + 0.04, depth / 2, h), wood, bevel=0.01)
    box("shelf_crown", (-w / 2 - 0.07, -depth / 2 - 0.05, h), (w / 2 + 0.07, depth / 2, h + 0.1), p.trim, bevel=0.02)
    mats = book_mats()
    for k in range(rows):
        zz = 0.1 + (h - 0.25) * k / (rows - 1) if rows > 1 else 0.1
        box("shelf_board", (-w / 2, -depth / 2, zz - 0.035), (w / 2, depth / 2, zz), wood, bevel=0)
        if fill and k < rows - 1:
            books(p, -w / 2 + 0.05, w / 2 - 0.05, 0.0, zz, depth - 0.06, rng, mats, max_h=min(0.32, (h - 0.25) / (rows - 1) - 0.06))
    transform(PIECES[before:], Matrix.Translation((cx, cy, z)) @ Matrix.Rotation(yaw, 4, "Z"))
    collide((cx, cy, z + h / 2), (w + 0.1, depth + 0.05, h), yaw)


def lectern_desk(p, cx, cy, z, yaw, rng, lit=True):
    """Skrivepult med skrå plate mot lokal −y, krakk foran, en åpen bok,
    blekkhorn, fjærpenn og lys."""
    before = len(PIECES)
    wood = p.cladding
    top = box("desk_top", (-0.45, -0.32, -0.025), (0.45, 0.32, 0.025), wood, bevel=0.01)
    top.rotation_euler = (-0.32, 0, 0)
    top.location = (0, 0, 1.0)
    for sx in (-1, 1):
        prism("desk_side", ((sx * 0.42, 0.0), (0.0, 1.0)), [(-0.3, 0.0), (0.3, 0.0), (0.3, 1.1), (-0.3, 0.9)], -0.03, 0.03, wood)
    box("desk_shelf", (-0.4, -0.25, 0.32), (0.4, 0.28, 0.35), wood, bevel=0)
    box("desk_ledge", (-0.45, -0.36, 0.86), (0.45, -0.31, 0.9), p.trim, bevel=0)
    mats = book_mats()
    books(p, -0.36, 0.34, 0.02, 0.35, 0.3, rng, mats, max_h=0.26)
    # Den åpne boka på plata og blekkhornet med fjærpenn oppe til høyre.
    page = layer("linen", (0.95, 0.9, 0.76))
    for sx in (-1, 1):
        pg = box("page", (0.0 if sx > 0 else -0.22, -0.15, 0.025), (0.22 if sx > 0 else 0.0, 0.15, 0.045), page, bevel=0.005)
        pg.rotation_euler = (-0.32, sx * -0.05, 0)
        pg.location = (0, 0, 1.0)
    lathe("inkhorn", [(0.0, 0.0), (0.035, 0.0), (0.04, 0.09), (0.0, 0.09)], 6, (0.36, 0.26, 1.08), layer("iron", (0.4, 0.35, 0.3)))
    stick("quill", (0.36, 0.26, 1.12), (0.32, 0.18, 1.36), 0.01, 0.03, page, bevel=0)
    # Krakken.
    lathe("stool", [(0.0, 0.44), (0.2, 0.44), (0.2, 0.5), (0.0, 0.5)], 8, (0, -0.7, 0), wood)
    for k in range(3):
        a = k * 2.1
        stick("stool_leg", (math.cos(a) * 0.12, -0.7 + math.sin(a) * 0.12, 0.45), (math.cos(a) * 0.17, -0.7 + math.sin(a) * 0.17, 0.0), 0.04, 0.04, p.timber, bevel=0)
    candle_holder(p, -0.38, 0.22, 1.1)
    transform(PIECES[before:], Matrix.Translation((cx, cy, z)) @ Matrix.Rotation(yaw, 4, "Z"))
    c, s_ = math.cos(yaw), math.sin(yaw)
    collide((cx + s_ * 0.25, cy - c * 0.25, z + 0.55), (1.0, 1.2, 1.1), yaw)
    if lit:
        lx, ly = -0.38 * c - 0.22 * s_, -0.38 * s_ + 0.22 * c
        light((cx + lx, cy + ly, z + 1.5), (1.0, 0.72, 0.42), 0.7, 3.5)


def candle_holder(p, x, y, z, h=0.16):
    """Et lys i en liten jernholder (tennes ikke her; lyset settes for seg)."""
    lathe("holder", [(0.0, 0.0), (0.06, 0.0), (0.05, 0.02), (0.015, 0.03), (0.015, 0.06), (0.05, 0.07), (0.0, 0.07)], 6, (x, y, z), p.iron)
    box("candle", (x - 0.025, y - 0.025, z + 0.07), (x + 0.025, y + 0.025, z + 0.07 + h), layer("plaster_white", (0.95, 0.9, 0.78)), bevel=0.006)
    box("candle_flame", (x - 0.012, y - 0.012, z + 0.07 + h), (x + 0.012, y + 0.012, z + 0.12 + h), p.fire, bevel=0.004)


def candle_stand(p, x, y, z, h=1.4, lit=True):
    """Høy lysestake av smijern med et tykt lys."""
    lathe("candlestand", [(0.0, 0.0), (0.2, 0.0), (0.14, 0.06), (0.03, 0.12), (0.03, h), (0.1, h + 0.04), (0.0, h + 0.05)], 8, (x, y, z), p.iron)
    box("candle", (x - 0.04, y - 0.04, z + h + 0.04), (x + 0.04, y + 0.04, z + h + 0.34), layer("plaster_white", (0.95, 0.9, 0.78)), bevel=0.01)
    box("candle_flame", (x - 0.018, y - 0.018, z + h + 0.34), (x + 0.018, y + 0.018, z + h + 0.42), p.fire, bevel=0.006)
    collide((x, y, z + h / 2), (0.4, 0.4, h))
    if lit:
        light((x, y, z + h + 0.6), (1.0, 0.7, 0.4), 1.0, 5.5)


def chandelier(p, x, y, z_top, drop, r=0.5, n=6, intensity=1.6, radius=8.0):
    """Lysekrone: en jernring i kjetting med lys rundt."""
    z = z_top - drop
    stick("chain", (x, y, z_top), (x, y, z + 0.02), 0.02, 0.02, p.iron, bevel=0)
    for k in range(3):
        a = k * 2 * math.pi / 3
        stick("chain", (x, y, z + 0.5), (x + math.cos(a) * r, y + math.sin(a) * r, z), 0.015, 0.015, p.iron, bevel=0)
    ring("crown", (x, y, z), r, 0.03, p.iron)
    for k in range(n):
        a = k * 2 * math.pi / n + 0.3
        cx, cy = x + math.cos(a) * r, y + math.sin(a) * r
        box("candle", (cx - 0.025, cy - 0.025, z + 0.02), (cx + 0.025, cy + 0.025, z + 0.16), layer("plaster_white", (0.95, 0.9, 0.78)), bevel=0.006)
        box("candle_flame", (cx - 0.012, cy - 0.012, z + 0.16), (cx + 0.012, cy + 0.012, z + 0.21), p.fire, bevel=0.004)
    light((x, y, z + 0.3), (1.0, 0.72, 0.42), intensity, radius)


def chest(p, x, y, z, w, d, h, yaw=0.0):
    before = len(PIECES)
    box("chest", (-w / 2, -d / 2, 0), (w / 2, d / 2, h), p.cladding, bevel=0.025)
    box("chest_lid", (-w / 2 - 0.02, -d / 2 - 0.02, h - 0.1), (w / 2 + 0.02, d / 2 + 0.02, h + 0.02), p.door_dark, bevel=0.02)
    for k in (-1, 1):
        box("chest_band", (k * w * 0.3 - 0.03, -d / 2 - 0.025, 0.02), (k * w * 0.3 + 0.03, d / 2 + 0.025, h + 0.03), p.iron, bevel=0.005)
    transform(PIECES[before:], Matrix.Translation((x, y, z)) @ Matrix.Rotation(yaw, 4, "Z"))
    collide((x, y, z + h / 2), (w + 0.05, d + 0.05, h), yaw)


def rail(p, a, b, z, posts=3):
    """Rekkverk fra a til b (x, y) på gulvet z: håndlist og stolper, og en
    kollider."""
    stick("handrail", (a[0], a[1], z + 0.95), (b[0], b[1], z + 0.95), 0.08, 0.08, p.timber, ("timber", 2))
    stick("midrail", (a[0], a[1], z + 0.5), (b[0], b[1], z + 0.5), 0.05, 0.06, p.timber, ("timber", 1))
    for t in lin(0.0, 1.0, posts):
        px, py = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
        stick("rail_post", (px, py, z), (px, py, z + 1.0), 0.09, 0.09, p.timber, ("timber", 0))
    L = math.hypot(b[0] - a[0], b[1] - a[1])
    collide(((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, z + 0.5), (L, 0.12, 1.0), math.atan2(b[1] - a[1], b[0] - a[0]))


def torch(p, x, y_face, z, out=-1):
    """Fakkelholder av smijern med en fakkel som brenner hele døgnet
    (materialet heter `fire…`); lyset settes av byen."""
    s = out
    stick("bracket_plate", (x, y_face + s * 0.02, z - 0.2), (x, y_face + s * 0.02, z + 0.12), 0.1, 0.03, p.iron, bevel=0)
    stick("bracket", (x, y_face + s * 0.02, z), (x, y_face + s * 0.3, z + 0.08), 0.04, 0.04, p.iron, bevel=0)
    stick("bracket_strut", (x, y_face + s * 0.02, z - 0.18), (x, y_face + s * 0.24, z + 0.02), 0.03, 0.03, p.iron, bevel=0)
    ring("bracket_ring", (x, y_face + s * 0.3, z + 0.1), 0.07, 0.015, p.iron, segs=8)
    stick("torch", (x, y_face + s * 0.28, z - 0.22), (x, y_face + s * 0.34, z + 0.32), 0.06, 0.06, layer("logs"), bevel=0.01)
    lathe("torch_head", [(0.0, 0.0), (0.06, 0.02), (0.065, 0.12), (0.0, 0.14)], 6, (x, y_face + s * 0.34, z + 0.26), layer("leather", (0.45, 0.32, 0.22)))
    flame = plain("fire_torch", (1.0, 0.55, 0.2), 0.5)
    lathe("flame", [(0.0, 0.0), (0.08, 0.08), (0.05, 0.22), (0.0, 0.4)], 5, (x, y_face + s * 0.35, z + 0.38), flame)


# --- Skriptoriet -------------------------------------------------------------

S_W, S_D, S_T = 10.0, 8.0, 0.6
S_F = 0.36  # gulvet inne
S_H = 6.0  # murkronen og himlingen
S_PITCH = 48.0
S_IX, S_IY = S_W / 2 - S_T, S_D / 2 - S_T
# Galleriet langs bakmuren og trappa opp i vestenden (langs y).
G_X0, G_X1, G_Y0 = -2.9, 2.9, 1.0
G_Z = S_F + 2.84
ST_X0, ST_X1, ST_Y0 = -2.9, -1.95, -2.6


def scriptorium_outside(p, rng):
    stone, cut = p.wall_stone, p.plaster
    sx, sy = S_W / 2 + 0.18, S_D / 2 + 0.18
    # Sokkelen: en bred steinkant under hele huset.
    box("plinth", (-sx, -sy, -0.25), (sx, sy, S_F - 0.02), stone, bevel=0.06)
    # Trinnene opp til døra.
    box("step", (-1.35, -sy - 0.55, -0.25), (1.35, -sy + 0.05, 0.16), stone, bevel=0.05)
    box("step", (-1.1, -sy - 0.2, 0.1), (1.1, -sy + 0.05, S_F - 0.02), stone, bevel=0.05)
    town.ramp(-1.1, 1.1, -sy - 0.75, -sy + 0.05, 0.0, S_F)
    # Murene: fronten og bakmuren går over hele bredden, sidene imellom.
    z0 = 0.1
    door_w, door_h = 1.5, 3.0
    win = (1.55, 2.9, 1.0)  # underkant, høyde, bredde
    fy, by = -S_D / 2 + S_T / 2, S_D / 2 - S_T / 2
    front, L = wall_frame((-S_W / 2, fy), (S_W / 2, fy))
    fopen = [(S_W / 2 - door_w / 2, S_W / 2 + door_w / 2, S_F, S_F + door_h)] + [(S_W / 2 + x - win[2] / 2, S_W / 2 + x + win[2] / 2, win[0], win[0] + win[1]) for x in (-3.0, 3.0)]
    slab_grid("wall_front", front, L, z0, S_H, S_T, fopen, cut)
    wall_colliders(front, L, z0, S_H, S_T, fopen)
    back, _ = wall_frame((-S_W / 2, by), (S_W / 2, by))
    bopen = [(S_W / 2 - 0.45, S_W / 2 + 0.45, G_Z + 0.75, G_Z + 2.55)] + [(S_W / 2 + x - 0.35, S_W / 2 + x + 0.35, 1.3, 2.9) for x in (-3.7, 3.7)]
    slab_grid("wall_back", back, L, z0, S_H, S_T, bopen, cut)
    wall_colliders(back, L, z0, S_H, S_T, bopen)
    sides = []
    for side in (-1, 1):
        x = side * (S_W / 2 - S_T / 2)
        fr, Ls = wall_frame((x, -S_D / 2 + S_T), (x, S_D / 2 - S_T))
        sopen = [(y + S_IY - win[2] / 2 + 0.03, y + S_IY + win[2] / 2 - 0.03, win[0], win[0] + win[1]) for y in (-1.9, 1.9)]
        slab_grid("wall_side", fr, Ls, z0, S_H, S_T, sopen, cut)
        wall_colliders(fr, Ls, z0, S_H, S_T, sopen)
        sides.append((fr, sopen, -side))
    # Vinduene: rundbuer med buestein og blyglass.
    for s0, s1, za, zb in fopen[1:]:
        arch_window(p, front, (s0 + s1) / 2, za, s1 - s0, zb - za, S_T, -1, cut, stone)
    for s0, s1, za, zb in bopen:
        arch_window(p, back, (s0 + s1) / 2, za, s1 - s0, zb - za, S_T, 1, cut, stone)
    for fr, sopen, out in sides:
        for s0, s1, za, zb in sopen:
            # Sidemurene går fra fronten bakover: utsida er mot −x i vest.
            arch_window(p, fr, (s0 + s1) / 2, za, s1 - s0, zb - za, S_T, out, cut, stone)
    # Døra: rundbue med buestein, kampestein i vangene og et overstykke.
    ds = S_W / 2
    arch_fill(front, ds, S_F, door_w, door_h, S_T, cut)
    voussoirs(front, ds, S_F + door_h - door_w / 2, door_w / 2, -1, S_T, stone, n=9, depth=0.34)
    for k, z in enumerate(lin(S_F, S_F + door_h - door_w / 2 - 0.4, 4)):
        for sx in (-1, 1):
            w = 0.36 if k % 2 == 0 else 0.24
            a = at_wall(front, ds + sx * (door_w / 2 + w / 2 - 0.02), -S_T / 2 - 0.05, z)
            box("jamb_stone", (a.x - w / 2, a.y - 0.07, z), (a.x + w / 2, a.y + 0.07, z + 0.38), stone, bevel=0.03)
    leaf = arched_leaf("door_leaf", door_w, door_h - 0.02, 0.08, p.door)
    straps = [stick("door_strap", (0.05, -0.06, z), (door_w * 0.8, -0.06, z), 0.03, 0.07, p.iron, bevel=0) for z in (0.6, 1.6, 2.4)]
    straps.append(ring("door_ring", (door_w - 0.25, -0.07, 1.25), 0.08, 0.015, p.iron, segs=8))
    straps[-1].rotation_euler = (math.pi / 2, 0, 0)
    # Slått inn mot rommet, hengslet i vest.
    transform([leaf] + straps, Matrix.Translation((ds - door_w / 2 - S_W / 2, fy + S_T / 2 - 0.06, S_F)) @ Matrix.Rotation(math.radians(95), 4, "Z"))
    # Støttepilarene: på hjørnene (fram og bak) og midt på langsidene.
    prof = [(0.0, -0.25), (0.85, -0.25), (0.85, 1.9), (0.6, 2.35), (0.6, 3.7), (0.34, 4.05), (0.34, S_H - 0.9), (0.0, S_H - 0.45)]
    for x in (-S_W / 2 + 0.4, S_W / 2 - 0.4):
        buttress("buttress", (x, -S_D / 2, 0), (0, -1, 0), (1, 0, 0), 0.75, prof, stone)
        buttress("buttress", (x, S_D / 2, 0), (0, 1, 0), (1, 0, 0), 0.75, prof, stone)
    for side in (-1, 1):
        buttress("buttress", (side * S_W / 2, 0, 0), (side, 0, 0), (0, 1, 0), 0.75, prof, stone)
        collide((side * (S_W / 2 + 0.42), 0, 1.9), (0.85, 0.75, 4.2))
    for x in (-S_W / 2 + 0.4, S_W / 2 - 0.4):
        for sy_ in (-1, 1):
            collide((x, sy_ * (S_D / 2 + 0.42), 1.9), (0.75, 0.85, 4.2))
    # Gesimsen langs langsidene under takfoten, og et bånd over sokkelen.
    for side in (-1, 1):
        stick("cornice", (side * (S_W / 2 + 0.06), -S_D / 2 - 0.05, S_H - 0.12), (side * (S_W / 2 + 0.06), S_D / 2 + 0.05, S_H - 0.12), 0.3, 0.26, stone, bevel=0.05)
    # Taket: skifer med mønet fra fronten til bakmuren, inne mellom gavlene.
    z_eave = S_H + 0.45
    ridge = ridge_roof(p, -S_D / 2 + 0.15, S_D / 2 - 0.15, S_W / 2, z_eave, S_PITCH, barges=False, overhang=0.55, gable_over=0.0, thick=0.42, sag=0.28, kick=0.3)
    rise = ridge - z_eave
    for y in (fy, by):
        fr, Lg = wall_frame((-S_W / 2 - 0.05, y), (S_W / 2 + 0.05, y))
        parapet(p, fr, Lg, S_H - 0.2, z_eave + 0.45, ridge + 0.35, S_T, cut, stone)
    # Rundvinduet i frontgavlen: blyglass i en krans av buestein.
    oz = S_H + rise * 0.42
    oy = -S_D / 2 - 0.02
    gl = lathe("oculus", [(0.0, -0.03), (0.42, -0.03), (0.42, 0.0), (0.0, 0.0)], 12, (0, 0, 0), p.glass)
    gl.rotation_euler = (math.pi / 2, 0, 0)
    gl.location = (0, oy, oz)
    for k in range(8):
        a0, a1 = 2 * math.pi * k / 8 + 0.02, 2 * math.pi * (k + 1) / 8 - 0.02
        pts = [(S_W / 2 + 0.42 * math.cos(a0), oz + 0.42 * math.sin(a0)), (S_W / 2 + 0.66 * math.cos(a0), oz + 0.66 * math.sin(a0)), (S_W / 2 + 0.66 * math.cos(a1), oz + 0.66 * math.sin(a1)), (S_W / 2 + 0.42 * math.cos(a1), oz + 0.42 * math.sin(a1))]
        prism("oculus_stone", front, pts, -S_T / 2 - 0.08, -S_T / 2 + 0.02, stone, bevel=0.02)
    stick("oculus_lead", (-0.42, oy - 0.01, oz), (0.42, oy - 0.01, oz), 0.03, 0.03, p.iron, bevel=0)
    stick("oculus_lead", (0, oy - 0.01, oz - 0.42), (0, oy - 0.01, oz + 0.42), 0.03, 0.03, p.iron, bevel=0)
    # Kors av stein på bakgavlen.
    stone_cross((0, by, ridge + 0.4), stone, 1.0)
    # Klokkestolen på mønet foran: fire stolper, kryssbånd, et lite
    # pyramidetak og klokka.
    bx, byy = 0.0, -S_D / 2 + 1.6
    bz = ridge - 0.25
    for px in (-0.45, 0.45):
        for py in (-0.45, 0.45):
            stick("bell_post", (bx + px, byy + py, bz - 0.5), (bx + px * 0.85, byy + py * 0.85, bz + 1.5), 0.14, 0.14, p.timber, ("timber", 1))
    for py in (-0.45, 0.45):
        stick("bell_beam", (bx - 0.55, byy + py * 0.88, bz + 1.35), (bx + 0.55, byy + py * 0.88, bz + 1.35), 0.12, 0.16, p.timber, ("timber", 2))
        stick("bell_brace", (bx - 0.42, byy + py * 0.95, bz + 0.1), (bx + 0.42, byy + py * 0.95, bz + 0.9), 0.08, 0.1, p.timber, ("timber", 3))
    stick("bell_yoke", (bx - 0.5, byy, bz + 1.3), (bx + 0.5, byy, bz + 1.3), 0.12, 0.14, p.timber, ("timber", 0))
    lathe("bell", [(0.0, 0.0), (0.26, 0.0), (0.22, 0.08), (0.15, 0.4), (0.09, 0.52), (0.0, 0.54)], 10, (bx, byy, bz + 0.68), layer("iron", (0.75, 0.55, 0.3), 0.4))
    lathe("bell_roof", [(0.9, 0.0), (0.0, 0.95)], 4, (bx, byy, bz + 1.45), p.roof, a0=math.pi / 4)
    lathe("finial", [(0.05, 0.0), (0.03, 0.5), (0.0, 0.6)], 5, (bx, byy, bz + 2.35), p.iron)
    # Skiltet: en åpen bok på en smijernsarm ved døra.
    ax, az = 1.75, 3.3
    stick("sign_arm", (ax, -S_D / 2, az), (ax, -S_D / 2 - 1.0, az), 0.06, 0.06, p.iron, bevel=0)
    stick("sign_strut", (ax, -S_D / 2, az - 0.45), (ax, -S_D / 2 - 0.7, az - 0.02), 0.04, 0.04, p.iron, bevel=0)
    stick("sign_chain", (ax, -S_D / 2 - 0.7, az), (ax, -S_D / 2 - 0.7, az - 0.2), 0.02, 0.02, p.iron, bevel=0)
    page = layer("linen", (0.95, 0.9, 0.76))
    for s in (-1, 1):
        b = box("sign_page", (-0.04, 0.0, -0.22), (0.04, 0.34, 0.22), page, bevel=0.02)
        b.rotation_euler = (0, 0, s * 0.35)
        b.location = (ax, -S_D / 2 - 0.7 + (0 if s > 0 else 0), az - 0.45)
        if s < 0:
            b.rotation_euler = (0, 0, math.pi - 0.35)
    box("sign_cover", (ax - 0.05, -S_D / 2 - 1.08, az - 0.7), (ax + 0.05, -S_D / 2 - 0.32, az - 0.2), layer("leather", (0.6, 0.2, 0.15)), bevel=0.02)
    # Lykta ved døra.
    town.wall_lantern(p, -1.55, -S_D / 2, S_F + 2.7, -1)
    # Kolliderne: sokkelen og taket.
    collide((0, 0, (S_F - 0.25) / 2 + 0.005), (S_W + 0.36, S_D + 0.36, S_F + 0.25))
    collide((0, 0, S_H + 0.1), (S_W, S_D, 0.2))


def scriptorium_inside(p, rng):
    ix, iy = S_IX, S_IY
    box("floor", (-ix, -iy, S_F - 0.05), (ix, iy, S_F + 0.02), p.flag, bevel=0)
    # Himlingen: bord på bjelker, og to svære bindbjelker på tvers.
    box("ceiling", (-ix, -iy, S_H - 0.05), (ix, iy, S_H), p.floor, bevel=0)
    x = -ix + 0.4
    while x < ix:
        stick("joist", (x, -iy, S_H - 0.15), (x, iy, S_H - 0.15), 0.16, 0.2, p.timber, ("timber", rng.randrange(4)), seed=x)
        x += 0.85
    for y in (-1.7, 0.8):
        stick("tie_beam", (-ix - 0.1, y, S_H - 0.42), (ix + 0.1, y, S_H - 0.42), 0.3, 0.34, p.timber, ("timber", rng.randrange(4)), seed=y)
        for side in (-1, 1):
            stick("corbel", (side * ix, y, S_H - 0.95), (side * (ix - 0.45), y, S_H - 0.6), 0.22, 0.24, p.timber, ("timber", 2))
    # Bokhyller: under galleriet langs bakmuren, og mellom vinduene.
    shelf(p, -0.5, iy - 0.2, S_F, 1.7, 2.3, 0.0, rng)
    shelf(p, 1.9, iy - 0.2, S_F, 1.7, 2.3, 0.0, rng)
    for side in (-1, 1):
        shelf(p, side * (ix - 0.2), 0.0, S_F, 1.7, 2.4, -side * math.pi / 2, rng)
    # Skrivepultene under sidevinduene, og to foran.
    for side in (-1, 1):
        for y in (-1.9, 1.9):
            if side < 0 and y > 0:
                continue  # der går trappa
            lectern_desk(p, side * (ix - 0.5), y, S_F, -side * math.pi / 2, rng)
    lectern_desk(p, 3.0, -iy + 0.55, S_F, math.pi, rng, lit=False)
    # Lesepulten midt i salen med en stor bok i kjetting.
    lx, ly = 0.6, -0.2
    stick("lectern_post", (lx, ly, S_F), (lx, ly, S_F + 1.05), 0.16, 0.16, p.timber, ("timber", 0))
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        stick("lectern_foot", (lx, ly, S_F + 0.12), (lx + math.cos(a) * 0.4, ly + math.sin(a) * 0.4, S_F), 0.09, 0.09, p.timber, ("timber", 1))
    top = box("lectern_top", (-0.42, -0.32, -0.03), (0.42, 0.32, 0.03), p.cladding, bevel=0.015)
    top.rotation_euler = (0.38, 0, 0)
    top.location = (lx, ly, S_F + 1.12)
    big = box("big_book", (-0.36, -0.26, 0.03), (0.36, 0.26, 0.12), layer("leather", (0.55, 0.18, 0.12)), bevel=0.02)
    big.rotation_euler = (0.38, 0, 0)
    big.location = (lx, ly, S_F + 1.12)
    pages = box("big_pages", (-0.34, -0.24, 0.12), (0.34, 0.24, 0.15), layer("linen", (0.95, 0.9, 0.76)), bevel=0.02)
    pages.rotation_euler = (0.38, 0, 0)
    pages.location = (lx, ly, S_F + 1.12)
    for k in range(7):
        stick("chain", (lx + 0.3, ly - 0.1 + k * 0.02, S_F + 1.05 - k * 0.12), (lx + 0.32, ly - 0.08 + k * 0.02, S_F + 0.93 - k * 0.12), 0.025, 0.025, p.iron, bevel=0)
    collide((lx, ly, S_F + 0.6), (0.85, 0.75, 1.2))
    # Høye lysestaker ved døra, og kista med skjemaene.
    for x in (-1.5, 1.5):
        candle_stand(p, x, -iy + 0.45, S_F)
    chest(p, -ix + 0.75, -iy + 0.5, S_F, 1.1, 0.6, 0.55)
    # Lysekronene.
    chandelier(p, 0.0, -1.7, S_H - 0.6, 1.6)
    chandelier(p, 1.2, 0.8, S_H - 0.6, 1.3, r=0.4, n=5, intensity=1.2)
    library_gallery(p, rng)


def library_gallery(p, rng):
    """Galleriet langs bakmuren: gulv på bjelker med stolper og knekter,
    trapp opp i vestenden, rekkverk, bokhyller, lesepult under det høye
    vinduet, og ei kiste med skriftruller."""
    iy = S_IY
    gz = G_Z
    box("gallery_floor", (G_X0, G_Y0, gz - 0.07), (G_X1, iy, gz), p.floor, bevel=0.01)
    collide(((G_X0 + G_X1) / 2, (G_Y0 + iy) / 2, gz - 0.12), (G_X1 - G_X0, iy - G_Y0, 0.25))
    x = G_X0 + 0.15
    while x < G_X1:
        stick("gallery_joist", (x, G_Y0, gz - 0.17), (x, iy, gz - 0.17), 0.14, 0.2, p.timber, ("timber", rng.randrange(4)), seed=x)
        x += 0.6
    stick("gallery_edge", (G_X0 - 0.05, G_Y0 + 0.08, gz - 0.2), (G_X1 + 0.05, G_Y0 + 0.08, gz - 0.2), 0.24, 0.32, p.timber, ("timber", 2))
    stick("gallery_edge", (G_X1 - 0.08, G_Y0, gz - 0.2), (G_X1 - 0.08, iy, gz - 0.2), 0.2, 0.3, p.timber, ("timber", 1))
    for x in (-1.6, 0.65, G_X1 - 0.12):
        stick("gallery_post", (x, G_Y0 + 0.1, S_F), (x, G_Y0 + 0.1, gz - 0.36), 0.24, 0.24, p.timber, ("timber", rng.randrange(4)), seed=x)
        box("post_base", (x - 0.18, G_Y0 + 0.1 - 0.18, S_F), (x + 0.18, G_Y0 + 0.1 + 0.18, S_F + 0.14), p.wall_stone, bevel=0.03)
        for sx in (-1, 1):
            if G_X0 + 0.2 < x + sx * 0.6 < G_X1:
                stick("gallery_brace", (x, G_Y0 + 0.1, gz - 1.0), (x + sx * 0.6, G_Y0 + 0.1, gz - 0.38), 0.12, 0.12, p.timber, ("timber", 3))
        collide((x, G_Y0 + 0.1, S_F + 1.4), (0.3, 0.3, 2.8))
    # Trappa opp langs y i vestenden, med rekkverk mot rommet.
    kit.stairs(p, ST_X0, ST_X1, ST_Y0, G_Y0, S_F, gz, 13, along="y", rails=(ST_X1,))
    stick("stair_post", (ST_X1, ST_Y0 - 0.05, S_F), (ST_X1, ST_Y0 - 0.05, S_F + 1.2), 0.12, 0.12, p.timber, ("timber", 0))
    collide((ST_X1 + 0.04, (ST_Y0 + G_Y0) / 2 + 0.3, S_F + 2.0), (0.1, G_Y0 - ST_Y0 - 0.8, 1.6))
    # Rekkverket: foran fra trappa, på østenden og langs vestenden.
    rail(p, (ST_X1 + 0.05, G_Y0 + 0.06), (G_X1, G_Y0 + 0.06), gz, posts=5)
    rail(p, (G_X1, G_Y0 + 0.06), (G_X1, iy), gz, posts=3)
    rail(p, (G_X0, G_Y0 + 0.9), (G_X0, iy), gz, posts=3)
    # Bokhyllene langs bakmuren på hver side av vinduet.
    for x in (-1.85, 1.85):
        shelf(p, x, iy - 0.2, gz, 1.6, 2.2, 0.0, rng)
    # Lesepulten foran vinduet, og kista med skriftruller ved østenden.
    lectern_desk(p, 0.0, iy - 0.85, gz, 0.0, rng)
    cx, cy = 2.25, G_Y0 + 0.75
    chest(p, cx, cy, gz, 0.9, 0.55, 0.45)
    vellum = layer("linen", (0.95, 0.88, 0.68))
    for k in range(7):
        ob = lathe("scroll", [(0.0, -0.22), (0.035, -0.22), (0.035, 0.22), (0.0, 0.22)], 6, (0, 0, 0), vellum)
        ob.rotation_euler = (math.pi / 2, 0, rng.uniform(-0.4, 0.4))
        ob.location = (cx - 0.3 + (k % 4) * 0.2, cy - 0.08 + (k // 4) * 0.16, gz + 0.5 + (k // 4) * 0.06)
    light((0.0, iy - 1.2, gz + 2.0), (1.0, 0.72, 0.42), 1.0, 6.0)


def scriptorium(rng):
    p = scriptorium_paint()
    town.LIT = True
    scriptorium_outside(p, rng)
    return p


# --- Stavkirka ---------------------------------------------------------------

C_W, C_D = 6.0, 8.6  # sideskipet
C_NW, C_ND = 3.8, 6.6  # midtskipet
C_GW, C_GD = C_W + 2.6, C_D + 2.6  # svalgangen
C_Z = 0.25  # gulvet
C_T = 0.16
C_CY0, C_CL, C_CW = C_D / 2, 2.8, 3.4  # koret: fra bakveggen, lengde, bredde
C_AY = C_D / 2 + 2.8  # apsiden: midten
C_AR = 1.3


def church_outside(p, rng):
    tar = p.timber
    boards = p.plaster
    shingle = p.roof
    stone = p.stone
    # Steinfundamentet under kirka og under koret.
    box("plinth", (-(C_W + 2.9) / 2, -(C_D + 2.9) / 2, -0.25), ((C_W + 2.9) / 2, (C_D + 2.9) / 2, C_Z), stone, bevel=0.06)
    box("plinth", (-C_CW / 2 - 0.35, C_D / 2, -0.25), (C_CW / 2 + 0.35, C_AY + C_AR + 0.35, C_Z), stone, bevel=0.06)
    collide((0, 0, (C_Z - 0.25) / 2), (C_W + 2.9, C_D + 2.9, C_Z + 0.25))
    collide((0, (C_D / 2 + C_AY + C_AR + 0.35) / 2, (C_Z - 0.25) / 2), (C_CW + 0.7, C_AY + C_AR + 0.35 - C_D / 2, C_Z + 0.25))
    box("step", (-0.95, -C_GD / 2 - 0.85, -0.25), (0.95, -(C_D + 2.9) / 2 + 0.05, C_Z - 0.12), stone, bevel=0.04)
    town.ramp(-0.9, 0.9, -C_GD / 2 - 1.1, -(C_D + 2.9) / 2 + 0.05, 0.0, C_Z)

    # Svalgangen: lav plankevegg, stolper med buer imellom, og pulttak.
    gx, gy = C_GW / 2, C_GD / 2
    top = 2.05
    sides = [((-gx, -gy), (gx, -gy), [(-0.9, 0.9)]), ((-gx, gy), (gx, gy), [(-1.85, 1.85)]), ((-gx, -gy), (-gx, gy), []), ((gx, -gy), (gx, gy), [])]
    for a, b, gaps in sides:
        fr, L = wall_frame(a, b)
        along_x = abs(a[1] - b[1]) < 1e-6
        c0 = a[0] if along_x else a[1]
        holes = [(g0 - c0, g1 - c0, C_Z, 1.15) for g0, g1 in gaps]
        slab_grid("gallery_wall", fr, L, C_Z, 1.15, 0.08, holes, boards)
        # Kolliderne: bitene mellom åpningene (åpningene går helt opp).
        cuts = [0.0] + [v for h in holes for v in (h[0], h[1])] + [L]
        for s0, s1 in zip(cuts[::2], cuts[1::2]):
            c = at_wall(fr, (s0 + s1) / 2, 0, (C_Z + 1.15) / 2)
            collide(tuple(c), (s1 - s0, 0.12, 1.15 - C_Z), math.atan2(fr[1][1], fr[1][0]))
        stick("gallery_rail", at_wall(fr, -0.05, 0, 1.15), at_wall(fr, L + 0.05, 0, 1.15), 0.16, 0.08, p.trim, ("trim", CARVED))
        # Stolpene, med en ved hver kant av åpningene.
        n = max(2, round(L / 1.1))
        posts = [L * k / n for k in range(n + 1)]
        for g0, g1 in gaps:
            posts = [s for s in posts if not (g0 - c0 - 0.3 < s < g1 - c0 + 0.3)] + [g0 - c0 - 0.07, g1 - c0 + 0.07]
        posts = sorted(posts)
        for s in posts:
            stick("gallery_post", at_wall(fr, s, 0, C_Z), at_wall(fr, s, 0, top), 0.14, 0.14, tar, ("timber", rng.randrange(4)), seed=s)
        for s0, s1 in zip(posts, posts[1:]):
            if any(g0 - c0 - 0.2 < (s0 + s1) / 2 < g1 - c0 + 0.2 for g0, g1 in gaps):
                continue  # inngangen, og bak der koret går gjennom
            m = (s0 + s1) / 2
            rx = (s1 - s0) / 2 - 0.07
            zs = top - 0.5
            pts = [(s0 + 0.07, top - 0.02), (s1 - 0.07, top - 0.02)] + [(m + rx * math.cos(a_), zs + 0.36 * math.sin(a_)) for a_ in lin(0.0, math.pi, 7)]
            prism("arcade", fr, pts, -0.03, 0.03, boards)
        # Stavlægja oppå stolpene (bak: to biter på hver side av koret).
        spans = [(0.0, L)] if not (along_x and a[1] > 0) else [(0.0, gaps[0][0] - c0), (gaps[0][1] - c0, L)]
        for s0, s1 in spans:
            stick("gallery_plate", at_wall(fr, s0 - 0.08, 0, top + 0.06), at_wall(fr, s1 + 0.08, 0, top + 0.06), 0.16, 0.16, tar, ("timber", rng.randrange(4)), seed=s0)
    # Takene over svalgangen: fire sider som møtes i gradene, bak delt av koret.
    ox, oy = gx + 0.35, gy + 0.35
    hw, hd = C_W / 2, C_D / 2
    zi, zo = 3.05, 1.95
    skirt("gallery_roof", ((-hw, -hd), (hw, -hd)), ((-ox, -oy), (ox, -oy)), zi, zo, 0.14, shingle)
    skirt("gallery_roof", ((-hw, -hd), (-hw, hd)), ((-ox, -oy), (-ox, oy)), zi, zo, 0.14, shingle)
    skirt("gallery_roof", ((hw, -hd), (hw, hd)), ((ox, -oy), (ox, oy)), zi, zo, 0.14, shingle)
    cg = C_CW / 2 + 0.12
    skirt("gallery_roof", ((-hw, hd), (-cg, hd)), ((-ox, oy), (-cg, oy)), zi, zo, 0.14, shingle, n=3)
    skirt("gallery_roof", ((cg, hd), (hw, hd)), ((cg, oy), (ox, oy)), zi, zo, 0.14, shingle, n=3)
    # Et lite gavltak over inngangen i svalgangen.
    ridge_roof(p, -gy - 0.6, -gy + 0.9, 1.05, 2.2, 52.0, overhang=0.3, gable_over=0.2, thick=0.2, sag=0.08, kick=0.15)
    fr, L = wall_frame((-1.05, -gy - 0.45), (1.05, -gy - 0.45))
    prism("porch_gable", fr, [(0.0, 2.15), (L, 2.15), (L / 2, 2.15 + 1.05 * math.tan(math.radians(52)) - 0.1)], -0.04, 0.04, boards)
    for sx in (-1, 1):
        stick("porch_post", (sx * 0.95, -gy - 0.45, C_Z - 0.15), (sx * 0.95, -gy - 0.45, 2.2), 0.16, 0.16, tar, ("timber", 2))
        collide((sx * 0.95, -gy - 0.45, 1.1), (0.2, 0.2, 2.2))

    # Sideskipet: plankevegger med portalen foran og korbuen bak.
    door_w, door_h = 1.1, 2.6
    walls = []
    fy, by = -C_D / 2 + C_T / 2, C_D / 2 - C_T / 2
    for y, holes in ((fy, [(C_W / 2 - door_w / 2, C_W / 2 + door_w / 2, C_Z, C_Z + door_h)]), (by, [(C_W / 2 - 1.1, C_W / 2 + 1.1, C_Z, C_Z + 2.8)])):
        fr, L = wall_frame((-C_W / 2, y), (C_W / 2, y))
        slab_grid("aisle_wall", fr, L, C_Z, 3.6, C_T, holes, boards)
        wall_colliders(fr, L, C_Z, 3.6, C_T, holes)
        walls.append(fr)
    for side in (-1, 1):
        fr, L = wall_frame((side * (C_W / 2 - C_T / 2), -C_D / 2 + C_T), (side * (C_W / 2 - C_T / 2), C_D / 2 - C_T))
        slab_grid("aisle_wall", fr, L, C_Z, 3.6, C_T, [], boards)
        wall_colliders(fr, L, C_Z, 3.6, C_T, [])
    arch_fill(walls[0], C_W / 2, C_Z, door_w, door_h, C_T, boards)
    # Svill og stavlægje rundt sideskipet, og hjørnestavene.
    for z in (C_Z + 0.12, 3.5):
        for a, b in (((-C_W / 2, -C_D / 2), (C_W / 2, -C_D / 2)), ((-C_W / 2, C_D / 2), (C_W / 2, C_D / 2)), ((-C_W / 2, -C_D / 2), (-C_W / 2, C_D / 2)), ((C_W / 2, -C_D / 2), (C_W / 2, C_D / 2))):
            fr, L = wall_frame(a, b)
            if z < 1 and a[1] == b[1] and a[1] < 0:
                for s0, s1 in ((-0.1, C_W / 2 - door_w / 2 - 0.15), (C_W / 2 + door_w / 2 + 0.15, L + 0.1)):
                    stick("aisle_sill", at_wall(fr, s0, 0, z) + _out(a, b) * 0.1, at_wall(fr, s1, 0, z) + _out(a, b) * 0.1, 0.12, 0.24, tar, ("timber", 1))
            else:
                stick("aisle_plate" if z > 1 else "aisle_sill", at_wall(fr, -0.1, 0, z) + _out(a, b) * 0.1, at_wall(fr, L + 0.1, 0, z) + _out(a, b) * 0.1, 0.12, 0.24, tar, ("timber", 1))
    for sx in (-1, 1):
        for sy in (-1, 1):
            lathe("corner_stave", [(0.2, C_Z), (0.18, C_Z + 0.3), (0.18, 3.55), (0.24, 3.7), (0.0, 3.72)], 8, (sx * C_W / 2, sy * C_D / 2, 0), tar)

    # Portalen: søyler med utskjæring og en buet karm rundt døra.
    py = -C_D / 2 - 0.06
    for sx in (-1, 1):
        lathe("portal_column", [(0.13, C_Z), (0.1, C_Z + 0.25), (0.1, C_Z + door_h - door_w / 2), (0.15, C_Z + door_h - door_w / 2 + 0.15), (0.0, C_Z + door_h - door_w / 2 + 0.16)], 8, (sx * (door_w / 2 + 0.12), py - 0.04, 0), tar)
        stick("portal_carving", (sx * (door_w / 2 + 0.38), py - 0.02, C_Z + 0.1), (sx * (door_w / 2 + 0.38), py - 0.02, C_Z + door_h - 0.2), 0.26, 0.06, p.trim, ("trim", CARVED), bevel=0.01)
    voussoirs(walls[0], C_W / 2, C_Z + door_h - door_w / 2, door_w / 2 + 0.05, -1, C_T, tar, n=7, depth=0.3, proud=0.08, bevel=0.015)
    leaf = arched_leaf("door_leaf", door_w, door_h - 0.02, 0.07, p.door_dark)
    straps = [stick("door_strap", (0.05, -0.05, z), (door_w * 0.85, -0.05, z), 0.03, 0.06, p.iron, bevel=0) for z in (0.6, 1.5, 2.1)]
    transform([leaf] + straps, Matrix.Translation((-door_w / 2, fy + C_T / 2 - 0.04, C_Z)) @ Matrix.Rotation(math.radians(100), 4, "Z"))

    # Taket over sideskipet: fra midtskipets vegger ned over plankeveggene.
    ax, ay = C_W / 2 + 0.45, C_D / 2 + 0.45
    nx, ny = C_NW / 2 + 0.02, C_ND / 2 + 0.02
    zi, zo = 4.75, 3.42
    skirt("aisle_roof", ((-nx, -ny), (nx, -ny)), ((-ax, -ay), (ax, -ay)), zi, zo, 0.16, shingle)
    skirt("aisle_roof", ((-nx, ny), (nx, ny)), ((-ax, ay), (ax, ay)), zi, zo, 0.16, shingle)
    skirt("aisle_roof", ((-nx, -ny), (-nx, ny)), ((-ax, -ay), (-ax, ay)), zi, zo, 0.16, shingle)
    skirt("aisle_roof", ((nx, -ny), (nx, ny)), ((ax, -ay), (ax, ay)), zi, zo, 0.16, shingle)

    # Midtskipet: høye plankevegger med glugger, bratt tak og gavler.
    nave = []
    for y in (-C_ND / 2 + 0.075, C_ND / 2 - 0.075):
        fr, L = wall_frame((-C_NW / 2, y), (C_NW / 2, y))
        slab_grid("nave_wall", fr, L, 3.5, 7.25, 0.15, [], boards)
    for side in (-1, 1):
        fr, L = wall_frame((side * (C_NW / 2 - 0.075), -C_ND / 2 + 0.15), (side * (C_NW / 2 - 0.075), C_ND / 2 - 0.15))
        holes = [(y + C_ND / 2 - 0.15 - 0.2, y + C_ND / 2 - 0.15 + 0.2, 5.85, 6.25) for y in (-C_ND / 4, C_ND / 4)]
        slab_grid("nave_wall", fr, L, 3.5, 7.25, 0.15, holes, boards)
        nave.append((fr, holes, side))
    for fr, holes, side in nave:
        for s0, s1, z0, z1 in holes:
            pane = town.block("glugg", (0.02, s1 - s0, z1 - z0), p.glass, at=tuple(at_wall(fr, (s0 + s1) / 2, 0, (z0 + z1) / 2)), bevel=0)
            town.daylight(pane, (-side, 0, 0))
            # Karmen: fire korte bjelker rundt gluggen.
            for a_, b_ in (((s0 - 0.08, z0 - 0.04), (s1 + 0.08, z0 - 0.04)), ((s0 - 0.08, z1 + 0.04), (s1 + 0.08, z1 + 0.04)), ((s0 - 0.04, z0), (s0 - 0.04, z1)), ((s1 + 0.04, z0), (s1 + 0.04, z1))):
                stick("glugg_frame", at_wall(fr, a_[0], side * 0.1, a_[1]), at_wall(fr, b_[0], side * 0.1, b_[1]), 0.08, 0.08, tar, ("timber", 0), bevel=0.015)
    for z in (3.65, 7.12):
        for a, b in (((-C_NW / 2, -C_ND / 2), (C_NW / 2, -C_ND / 2)), ((-C_NW / 2, C_ND / 2), (C_NW / 2, C_ND / 2)), ((-C_NW / 2, -C_ND / 2), (-C_NW / 2, C_ND / 2)), ((C_NW / 2, -C_ND / 2), (C_NW / 2, C_ND / 2))):
            fr, L = wall_frame(a, b)
            stick("nave_plate", at_wall(fr, -0.1, 0, z) + _out(a, b) * 0.09, at_wall(fr, L + 0.1, 0, z) + _out(a, b) * 0.09, 0.1, 0.22, tar, ("timber", 2))
    for sx in (-1, 1):
        for sy in (-1, 1):
            lathe("nave_stave", [(0.18, 3.5), (0.18, 7.2), (0.24, 7.35), (0.0, 7.37)], 8, (sx * C_NW / 2, sy * C_ND / 2, 0), tar)
    z_eave = 7.5
    ridge = ridge_roof(p, -C_ND / 2, C_ND / 2, C_NW / 2 + 0.08, z_eave, 60.0, overhang=0.45, gable_over=0.4, thick=0.3, sag=0.22, kick=0.28)
    for y in (-C_ND / 2, C_ND / 2):
        fr, L = wall_frame((-C_NW / 2 - 0.08, y), (C_NW / 2 + 0.08, y))
        prism("nave_gable", fr, [(0.0, 7.2), (L, 7.2), (L / 2, ridge - 0.3)], -0.08, 0.08, boards)
        # Bord på kant langs gavlen, og et kors på gavlspissen.
        stick("gable_board", at_wall(fr, L / 2, 0, 7.25), at_wall(fr, L / 2, 0, ridge - 0.35), 0.18, 0.06, tar, ("timber", 3))
    for sy in (-1, 1):
        dragon(p, sy * (C_ND / 2 + 0.45), ridge + 0.05, sy, tar)
        stick("cross_v", (0, sy * (C_ND / 2 + 0.2), ridge - 0.1), (0, sy * (C_ND / 2 + 0.2), ridge + 1.0), 0.1, 0.1, tar, ("timber", 1))
        stick("cross_h", (-0.3, sy * (C_ND / 2 + 0.2), ridge + 0.7), (0.3, sy * (C_ND / 2 + 0.2), ridge + 0.7), 0.1, 0.1, tar, ("timber", 2))

    # Takrytteren midt på mønet: et lite tårn, et skjørt, klokkestua med
    # åpninger og et høyt spir.
    tz = ridge - 0.7
    box("turret", (-0.6, -0.6, tz), (0.6, 0.6, tz + 1.5), boards, bevel=0.02)
    for a, b in (((-0.6, -0.6), (0.6, -0.6)), ((0.6, -0.6), (0.6, 0.6)), ((0.6, 0.6), (-0.6, 0.6)), ((-0.6, 0.6), (-0.6, -0.6))):
        stick("turret_plate", (a[0] * 1.08, a[1] * 1.08, tz + 1.45), (b[0] * 1.08, b[1] * 1.08, tz + 1.45), 0.1, 0.12, tar, ("timber", 0))
    sk = 1.0
    for inner, outer in ((((-0.6, -0.6), (0.6, -0.6)), ((-sk, -sk), (sk, -sk))), (((-0.6, 0.6), (0.6, 0.6)), ((-sk, sk), (sk, sk))), (((-0.6, -0.6), (-0.6, 0.6)), ((-sk, -sk), (-sk, sk))), (((0.6, -0.6), (0.6, 0.6)), ((sk, -sk), (sk, sk)))):
        skirt("turret_skirt", inner, outer, tz + 1.75, tz + 1.3, 0.1, shingle, n=2, sag=0.04, kick=0.08)
    box("belfry", (-0.48, -0.48, tz + 1.7), (0.48, 0.48, tz + 2.6), boards, bevel=0.02)
    dark = layer("stone", (0.35, 0.3, 0.28))
    for k in range(4):
        a = k * math.pi / 2
        c, s = math.cos(a), math.sin(a)
        for off in (-0.2, 0.2):
            px, py_ = c * 0.49 - s * off, s * 0.49 + c * off
            b = box("belfry_hole", (-0.06, -0.015, -0.22), (0.06, 0.015, 0.22), dark, bevel=0)
            b.rotation_euler = (0, 0, a + math.pi / 2)
            b.location = (px, py_, tz + 2.15)
    for inner, outer in ((((-0.48, -0.48), (0.48, -0.48)), ((-0.75, -0.75), (0.75, -0.75))), (((-0.48, 0.48), (0.48, 0.48)), ((-0.75, 0.75), (0.75, 0.75))), (((-0.48, -0.48), (-0.48, 0.48)), ((-0.75, -0.75), (-0.75, 0.75))), (((0.48, -0.48), (0.48, 0.48)), ((0.75, -0.75), (0.75, 0.75)))):
        skirt("belfry_skirt", inner, outer, tz + 2.75, tz + 2.45, 0.08, shingle, n=2, sag=0.03, kick=0.06)
    lathe("spire", [(0.5, 0.0), (0.0, 3.4)], 8, (0, 0, tz + 2.6), shingle, a0=math.pi / 8)
    lathe("spire_top", [(0.05, 0.0), (0.04, 0.7), (0.0, 0.8)], 5, (0, 0, tz + 5.85), p.iron)
    stick("spire_cross", (-0.22, 0, tz + 6.4), (0.22, 0, tz + 6.4), 0.04, 0.04, p.iron, bevel=0)

    # Koret: plankevegger, eget tak og gavl, og apsiden bak.
    cy1 = C_CY0 + C_CL
    for side in (-1, 1):
        fr, L = wall_frame((side * (C_CW / 2 - C_T / 2), C_CY0), (side * (C_CW / 2 - C_T / 2), cy1))
        slab_grid("choir_wall", fr, L, C_Z, 3.4, C_T, [], boards)
        wall_colliders(fr, L, C_Z, 3.4, C_T, [])
    fr, L = wall_frame((-C_CW / 2, cy1 - C_T / 2), (C_CW / 2, cy1 - C_T / 2))
    holes = [(L / 2 - 1.0, L / 2 + 1.0, C_Z, C_Z + 2.4)]
    slab_grid("choir_wall", fr, L, C_Z, 3.4, C_T, holes, boards)
    for side in (-1, 1):
        stick("choir_plate", (side * (C_CW / 2 + 0.06), C_CY0 - 0.1, 3.3), (side * (C_CW / 2 + 0.06), cy1 + 0.1, 3.3), 0.12, 0.22, tar, ("timber", 1))
        lathe("choir_stave", [(0.17, C_Z), (0.17, 3.35), (0.22, 3.48), (0.0, 3.5)], 8, (side * C_CW / 2, cy1, 0), tar)
    cr = ridge_roof(p, C_CY0 - 0.5, cy1, C_CW / 2 + 0.06, 3.65, 55.0, overhang=0.35, gable_over=0.3, thick=0.22, sag=0.12, kick=0.2)
    fr, L = wall_frame((-C_CW / 2 - 0.06, cy1), (C_CW / 2 + 0.06, cy1))
    prism("choir_gable", fr, [(0.0, 3.35), (L, 3.35), (L / 2, cr - 0.25)], -0.08, 0.08, boards)
    small_dragon(p, 0, cy1 + 0.6, cr - 0.05, 0, 1, tar)
    # Apsiden: en halv sylinder bak koret, med kjegletak og et lite tårn.
    lathe("apse", [(C_AR, C_Z), (C_AR, 3.0)], 8, (0, C_AY, 0), boards, cap=False, a0=0.0, arc=math.pi)
    lathe("apse_in", [(C_AR - 0.12, 3.0), (C_AR - 0.12, C_Z)], 8, (0, C_AY, 0), boards, cap=False, a0=0.0, arc=math.pi)
    lathe("apse_roof", [(C_AR + 0.35, 2.85), (0.0, 4.5)], 8, (0, C_AY, 0), shingle, cap=False, a0=0.0, arc=math.pi)
    lathe("apse_roof_under", [(0.0, 4.35), (C_AR + 0.35, 2.75)], 8, (0, C_AY, 0), shingle, cap=False, a0=0.0, arc=math.pi)
    lathe("apse_tower", [(0.3, 0.0), (0.3, 0.9), (0.0, 2.0)], 6, (0, C_AY + 0.25, 4.1), shingle)
    stick("apse_plate", (-C_AR - 0.05, C_AY, 2.95), (C_AR + 0.05, C_AY, 2.95), 0.1, 0.1, tar, ("timber", 0))
    collide((0, C_AY + 0.6, 1.6), (2.6, 1.2, 2.8))


def _out(a, b):
    """Normalen ut fra en side av et rektangel rundt origo."""
    fr, _ = wall_frame(a, b)
    n = Vector((-fr[1][1], fr[1][0], 0))
    mid = Vector(((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, 0))
    return n if n.dot(mid) > 0 else -n


def dragon(p, y, z, out, mat):
    """Et drakehode på mønet: buet hals ut og opp, hode med gapende kjeft,
    horn og en krøllet tunge."""
    pts = [Vector((0, y - out * 0.3, z))]
    for k in range(1, 6):
        t = k / 5
        pts.append(Vector((0, y + out * 1.1 * t, z + 0.75 * math.sin(t * math.pi * 0.75))))
    for k, (a, b) in enumerate(zip(pts, pts[1:])):
        stick("dragon_neck", a, b, 0.2 - k * 0.02, 0.26 - k * 0.025, mat, ("timber", 1), bevel=0.03)
    head = pts[-1]
    ob = box("dragon_head", (-0.1, 0.0, -0.1), (0.1, 0.5, 0.12), mat, bevel=0.04)
    ob2 = box("dragon_jaw", (-0.08, 0.05, -0.08), (0.08, 0.45, 0.0), mat, bevel=0.03)
    horns = [stick("dragon_horn", (s * 0.06, 0.08, 0.1), (s * 0.12, -0.25, 0.45), 0.05, 0.05, mat, bevel=0.01) for s in (-1, 1)]
    comb = [stick("dragon_comb", (0, 0.05 + k * 0.12, 0.12), (0, -0.05 + k * 0.12, 0.3), 0.03, 0.08, mat, bevel=0.01) for k in range(3)]
    M = Matrix.Translation(head) @ Matrix.Rotation(0 if out > 0 else math.pi, 4, "Z")
    transform([ob] + horns + comb, M @ Matrix.Rotation(-0.25, 4, "X"))
    transform([ob2], M @ Matrix.Rotation(0.35, 4, "X"))


def small_dragon(p, x, y, z, sx, sy, mat):
    """Et lite drakehode på hjørnet av et lavt tak, vendt ut på skrå."""
    d = Vector((sx, sy, 0)).normalized() if (sx or sy) else Vector((0, 1, 0))
    a = Vector((x, y, z))
    b = a + d * 0.45 + Vector((0, 0, 0.3))
    c = b + d * 0.3 + Vector((0, 0, 0.15))
    stick("dragon_neck", a, b, 0.14, 0.16, mat, ("timber", 1), bevel=0.02)
    stick("dragon_head", b, c, 0.12, 0.18, mat, ("timber", 2), bevel=0.03)
    stick("dragon_horn", b + Vector((0, 0, 0.05)), b - d * 0.15 + Vector((0, 0, 0.32)), 0.04, 0.04, mat, bevel=0.01)


def church_inside(p, rng):
    tar = p.timber
    boards = p.plaster
    wood = p.cladding
    ix, iy = C_W / 2 - C_T, C_D / 2 - C_T
    fz = C_Z + 0.03
    box("floor", (-ix, -iy, C_Z - 0.04), (ix, iy, fz), p.floor, bevel=0)
    # Stavene rundt midtskipet, fra gulvet til toppen av midtskipet.
    staves = [(sx * C_NW / 2, y) for sx in (-1, 1) for y in (-C_ND / 2, -C_ND / 6, C_ND / 6, C_ND / 2)]
    for x, y in staves:
        lathe("stave", [(0.0, fz + 0.24), (0.27, fz + 0.24), (0.27, fz), (0.2, fz), (0.17, fz + 0.36), (0.16, 7.2)], 10, (x, y, 0), tar, cap=False)
        collide((x, y, 2.0), (0.38, 0.38, 3.6))
    # Bindbjelker, andreaskors og buede knær mellom stavene.
    gz = 3.55
    for sx in (-1, 1):
        x = sx * (C_NW / 2 - 0.02)
        stick("girt", (x, -C_ND / 2, gz), (x, C_ND / 2, gz), 0.18, 0.24, tar, ("timber", 1))
        xi = sx * (C_NW / 2 - 0.16)
        for y0, y1 in ((-C_ND / 2, -C_ND / 6), (-C_ND / 6, C_ND / 6), (C_ND / 6, C_ND / 2)):
            stick("andrew", (xi, y0 + 0.15, gz + 0.15), (xi, y1 - 0.15, 5.4), 0.12, 0.14, tar, ("timber", rng.randrange(4)))
            stick("andrew", (xi, y1 - 0.15, gz + 0.15), (xi, y0 + 0.15, 5.4), 0.12, 0.14, tar, ("timber", rng.randrange(4)))
            stick("girt_upper", (xi, y0, 5.5), (xi, y1, 5.5), 0.14, 0.18, tar, ("timber", 2))
            mid, r = (y0 + y1) / 2, (y1 - y0) / 2 - 0.15
            arc = [Vector((x, mid + r * math.cos(a), gz - 0.12 - 0.55 * math.sin(a))) for a in lin(0.0, math.pi, 6)]
            for a_, b_ in zip(arc, arc[1:]):
                stick("knee", a_, b_, 0.14, 0.12, tar, ("timber", 3), bevel=0.02)
    for sy in (-1, 1):
        stick("girt", (-C_NW / 2, sy * (C_ND / 2 - 0.02), gz), (C_NW / 2, sy * (C_ND / 2 - 0.02), gz), 0.18, 0.24, tar, ("timber", 1))
    # Himlingen over sideskipet, i ring rundt midtskipet.
    hz = 3.5
    for sy in (-1, 1):
        box("aisle_ceiling", (-ix, min(sy * C_ND / 2, sy * iy), hz), (ix, max(sy * C_ND / 2, sy * iy), hz + 0.05), p.floor, bevel=0)
    for sx in (-1, 1):
        box("aisle_ceiling", (min(sx * C_NW / 2, sx * ix), -C_ND / 2, hz), (max(sx * C_NW / 2, sx * ix), C_ND / 2, hz + 0.05), p.floor, bevel=0)
    for k in range(7):
        y = -iy + 0.3 + (2 * iy - 0.6) * k / 6
        for sx in (-1, 1):
            x1 = sx * C_NW / 2 if abs(y) < C_ND / 2 - 0.1 else 0.0
            if x1 == 0.0 and sx > 0:
                x1 = 0.0
            stick("aisle_joist", (sx * ix, y, hz - 0.08), (x1, y, hz - 0.08), 0.12, 0.16, tar, ("timber", rng.randrange(4)))
    # Bjelker på tvers høyt oppe i midtskipet.
    for y in (-C_ND / 2 + 0.4, 0.0, C_ND / 2 - 0.4):
        stick("tie", (-C_NW / 2, y, 7.05), (C_NW / 2, y, 7.05), 0.18, 0.22, tar, ("timber", rng.randrange(4)))
    # Triumfbjelken med krusifikset foran korbuen.
    by = C_ND / 2 - 0.25
    stick("rood_beam", (-C_NW / 2, by, 3.15), (C_NW / 2, by, 3.15), 0.22, 0.26, tar, ("timber", 2))
    cz = 3.28
    gilt = layer("iron", (0.95, 0.72, 0.32), 0.35)
    rood = layer("planks", (0.9, 0.7, 0.5))
    stick("cross_v", (0, by, cz), (0, by, cz + 1.7), 0.14, 0.1, rood, ("timber", 0))
    stick("cross_h", (-0.62, by, cz + 1.2), (0.62, by, cz + 1.2), 0.14, 0.1, rood, ("timber", 0))
    for e in ((0, by, cz + 1.7), (-0.62, by, cz + 1.2), (0.62, by, cz + 1.2)):
        box("cross_end", (e[0] - 0.1, e[1] - 0.07, e[2] - 0.1), (e[0] + 0.1, e[1] + 0.07, e[2] + 0.1), gilt, bevel=0.03)
    body = layer("plaster", (0.86, 0.7, 0.55))
    linen = layer("linen", (0.92, 0.88, 0.8))
    fy = by - 0.1
    box("figure_torso", (-0.12, fy - 0.05, cz + 0.72), (0.12, fy + 0.05, cz + 1.12), body, bevel=0.04)
    box("figure_cloth", (-0.15, fy - 0.06, cz + 0.55), (0.15, fy + 0.06, cz + 0.76), linen, bevel=0.04)
    for sx in (-1, 1):
        stick("figure_arm", (sx * 0.1, fy, cz + 1.08), (sx * 0.56, fy, cz + 1.2), 0.07, 0.07, body, bevel=0.02)
        stick("figure_leg", (sx * 0.06, fy, cz + 0.58), (sx * 0.02, fy, cz + 0.18), 0.08, 0.08, body, bevel=0.02)
    lathe("figure_head", [(0.0, -0.09), (0.08, -0.06), (0.085, 0.03), (0.06, 0.09), (0.0, 0.11)], 8, (0, fy, cz + 1.25), body)
    ring("figure_halo", (0, fy + 0.04, cz + 1.3), 0.17, 0.02, gilt, segs=12).rotation_euler = (math.pi / 2, 0, 0)
    # Koret: et trinn opp, himling, alteret med duk og forside, lys og kors.
    cy0, cy1 = C_CY0, C_CY0 + C_CL
    ci = C_CW / 2 - C_T
    box("choir_floor", (-ci, cy0 - 0.05, C_Z - 0.04), (ci, cy1, C_Z + 0.2), p.floor, bevel=0.01)
    lathe("apse_floor", [(0.0, C_Z + 0.2), (C_AR - 0.1, C_Z + 0.2), (C_AR - 0.1, C_Z - 0.04)], 8, (0, C_AY, 0), p.floor, cap=False, arc=math.pi)
    box("choir_step", (-1.1, cy0 - 0.45, C_Z - 0.04), (1.1, cy0 - 0.05, C_Z + 0.1), p.floor, bevel=0.01)
    collide((0, (cy0 + cy1) / 2, C_Z + 0.08), (C_CW - 0.3, C_CL, 0.24))
    box("choir_ceiling", (-ci, cy0 - 0.1, 3.3), (ci, cy1, 3.35), p.floor, bevel=0)
    lathe("apse_ceiling", [(0.0, 2.95), (C_AR - 0.1, 2.95), (C_AR - 0.1, 3.0), (0.0, 3.0)], 8, (0, C_AY, 0), p.floor, cap=False, arc=math.pi)
    for k in range(4):
        stick("choir_joist", (-ci, cy0 + 0.3 + k * 0.7, 3.22), (ci, cy0 + 0.3 + k * 0.7, 3.22), 0.12, 0.16, tar, ("timber", k % 4))
    az = C_Z + 0.2
    ax, ay = 0.0, C_AY - 0.3
    stone = p.stone
    box("altar", (ax - 0.7, ay - 0.42, az), (ax + 0.7, ay + 0.42, az + 0.98), stone, bevel=0.04)
    box("altar_cloth", (ax - 0.74, ay - 0.46, az + 0.98), (ax + 0.74, ay + 0.46, az + 1.02), linen, bevel=0.01)
    red = layer("wool", (0.62, 0.14, 0.1))
    box("altar_frontal", (ax - 0.68, ay - 0.47, az + 0.15), (ax + 0.68, ay - 0.44, az + 0.98), red, bevel=0.005)
    for k in range(3):
        x = ax - 0.45 + k * 0.45
        box("frontal_band", (x - 0.04, ay - 0.49, az + 0.15), (x + 0.04, ay - 0.46, az + 0.98), gilt, bevel=0.005)
    for sx in (-1, 1):
        lathe("altar_candlestick", [(0.0, 0.0), (0.1, 0.0), (0.06, 0.04), (0.02, 0.06), (0.02, 0.32), (0.07, 0.34), (0.0, 0.35)], 8, (ax + sx * 0.5, ay, az + 1.02), gilt)
        box("candle", (ax + sx * 0.5 - 0.03, ay - 0.03, az + 1.37), (ax + sx * 0.5 + 0.03, ay + 0.03, az + 1.62), layer("plaster_white", (0.95, 0.9, 0.78)), bevel=0.008)
        box("candle_flame", (ax + sx * 0.5 - 0.014, ay - 0.014, az + 1.62), (ax + sx * 0.5 + 0.014, ay + 0.014, az + 1.68), p.fire, bevel=0.005)
        light((ax + sx * 0.5, ay, az + 1.9), (1.0, 0.7, 0.4), 1.2, 5.0)
    stick("altar_cross_v", (ax, ay + 0.2, az + 1.02), (ax, ay + 0.2, az + 1.7), 0.05, 0.05, gilt, bevel=0)
    stick("altar_cross_h", (ax - 0.18, ay + 0.2, az + 1.5), (ax + 0.18, ay + 0.2, az + 1.5), 0.05, 0.05, gilt, bevel=0)
    collide((ax, ay, az + 0.5), (1.5, 0.95, 1.0))
    # Døpefonten av kleberstein ved døra.
    fx, fy_ = 1.6, -C_D / 2 + 1.3
    soap = layer("stone", (0.75, 0.78, 0.72))
    lathe("font", [(0.0, 0.0), (0.32, 0.0), (0.24, 0.12), (0.17, 0.5), (0.38, 0.72), (0.42, 0.92), (0.35, 0.92), (0.3, 0.78), (0.0, 0.74)], 12, (fx, fy_, fz), soap)
    lathe("font_water", [(0.0, 0.84), (0.31, 0.84), (0.0, 0.845)], 10, (fx, fy_, fz), p.glass)
    collide((fx, fy_, fz + 0.45), (0.85, 0.85, 0.9))
    # Benkene langs veggene i sideskipet.
    for sx in (-1, 1):
        x = sx * (ix - 0.23)
        y0, y1 = -iy + 0.7, iy - 0.4
        box("wall_bench", (x - 0.22, y0, fz + 0.42), (x + 0.22, y1, fz + 0.48), wood, bevel=0.015)
        for k in range(5):
            y = y0 + 0.2 + k * (y1 - y0 - 0.4) / 4
            box("bench_leg", (x - 0.2, y - 0.04, fz), (x + 0.2, y + 0.04, fz + 0.42), tar, bevel=0.01)
        collide((x, (y0 + y1) / 2, fz + 0.24), (0.44, y1 - y0, 0.48))
    # Lysestaker ved stavene og ved korbuen.
    for x, y in ((-C_NW / 2 + 0.5, -C_ND / 6), (C_NW / 2 - 0.5, C_ND / 6), (-1.0, iy - 0.45), (1.0, iy - 0.45)):
        candle_stand(p, x, y, fz, h=1.3)


def church(rng):
    p = church_paint()
    town.LIT = True
    church_outside(p, rng)
    return p


# --- Krypten -----------------------------------------------------------------


def crypt_model(rng):
    p = kit.Paint(plaster="ruin", timber="timber", roof="slate", plinth="ruin")
    town.LIT = False
    wall = layer("ruin", (0.92, 0.95, 0.9))
    stone = layer("wall_stone", (0.85, 0.88, 0.82))
    roof = layer("slate", (0.75, 0.82, 0.78))
    moss = layer("sod", (0.62, 0.82, 0.42))
    p.roof = roof
    W, D, T, H = 5.6, 6.0, 0.55, 2.75
    y0, y1 = -1.0, 5.0
    yc = (y0 + y1) / 2
    z0 = -0.5
    door_w, door_h, dz = 1.3, 2.35, 0.12
    fy, by = y0 + T / 2, y1 - T / 2
    front, L = wall_frame((-W / 2, fy), (W / 2, fy))
    slab_grid("crypt_front", front, L, z0, H, T, [(W / 2 - door_w / 2, W / 2 + door_w / 2, dz, dz + door_h)], wall)
    back, _ = wall_frame((-W / 2, by), (W / 2, by))
    slab_grid("crypt_back", back, L, z0, H, T, [], wall)
    for side in (-1, 1):
        fr, Ls = wall_frame((side * (W / 2 - T / 2), y0 + T), (side * (W / 2 - T / 2), y1 - T))
        slab_grid("crypt_side", fr, Ls, z0, H, T, [], wall)
        # En smal, mørk glugg midt på langsida.
        out = side
        sx = side * (W / 2 + 0.005)
        box("slit", (sx - 0.04, yc - 0.13, 1.0), (sx + 0.04, yc + 0.13, 1.7), layer("stone", (0.18, 0.16, 0.15)), bevel=0)
        sfr = ((sx, yc - 1), (0, 1)) if side > 0 else ((sx, yc + 1), (0, -1))
        voussoirs(sfr, 1.0, 1.7, 0.13, 1, 0.0, stone, n=3, depth=0.16, proud=0.07, bevel=0.015)
        stick("slit_sill", (sx + out * 0.05, yc - 0.25, 0.96), (sx + out * 0.05, yc + 0.25, 0.96), 0.14, 0.1, stone, bevel=0.02)
    box("crypt_floor", (-W / 2 + T, y0 + T, dz - 0.1), (W / 2 - T, y1 - T, dz), wall, bevel=0)
    # Hjørnesteinene: annenhver lang og kort, litt ujevne.
    for cx in (-W / 2, W / 2):
        for cy, sy in ((y0, -1), (y1, 1)):
            z = -0.1
            k = 0
            while z < H - 0.3:
                h = rng.uniform(0.34, 0.46)
                lx, ly = (0.55, 0.32) if k % 2 == 0 else (0.32, 0.55)
                ex = 0.04 + rng.uniform(0, 0.03)
                lo = (min(cx - math.copysign(lx, cx), cx + math.copysign(ex, cx)), min(cy - sy * ly, cy + sy * ex), z)
                hi = (max(cx - math.copysign(lx, cx), cx + math.copysign(ex, cx)), max(cy - sy * ly, cy + sy * ex), z + h - 0.03)
                box("quoin", lo, hi, stone, bevel=0.035)
                z += h
                k += 1
    # Døra: rundbue med buestein og vangestein, og den jernbeslåtte døra
    # stengt, litt innfelt.
    s = W / 2
    zs = dz + door_h - door_w / 2
    arch_fill(front, s, dz, door_w, door_h, T, wall)
    voussoirs(front, s, zs, door_w / 2, -1, T, stone, n=7, depth=0.3, proud=0.1)
    for k, z in enumerate(lin(dz, zs - 0.4, 4)):
        for sx in (-1, 1):
            w = 0.34 if k % 2 == 0 else 0.22
            a = at_wall(front, s + sx * (door_w / 2 + w / 2 - 0.02), -T / 2 - 0.05, z)
            box("jamb_stone", (a.x - w / 2, a.y - 0.08, z), (a.x + w / 2, a.y + 0.08, z + 0.38), stone, bevel=0.03)
    leaf = arched_leaf("door_leaf", door_w + 0.02, door_h + 0.01, 0.1, p.door_dark)
    iron = layer("iron", (0.55, 0.52, 0.5), 0.45)
    parts = [leaf]
    for z in (0.35, 1.05, 1.75):
        parts.append(stick("door_band", (0.02, -0.07, z), (door_w - 0.0, -0.07, z), 0.03, 0.09, iron, bevel=0))
        # Hengslene: bøyde ender mot hengselsida.
        parts.append(stick("door_hinge", (-0.04, -0.07, z), (0.18, -0.07, z + 0.12), 0.03, 0.07, iron, bevel=0))
    parts.append(stick("door_band", (door_w / 2, -0.07, 0.1), (door_w / 2, -0.07, door_h - 0.1), 0.03, 0.09, iron, bevel=0))
    for k in range(6):
        for z in (0.35, 1.05, 1.75):
            x = 0.15 + k * (door_w - 0.3) / 5
            parts.append(box("door_stud", (x - 0.025, -0.11, z - 0.025), (x + 0.025, -0.06, z + 0.025), iron, bevel=0.01))
    r = ring("door_ring", (door_w - 0.3, -0.12, 1.25), 0.1, 0.018, iron, segs=8)
    r.rotation_euler = (math.pi / 2, 0, 0)
    parts.append(r)
    parts.append(box("door_lock", (door_w - 0.38, -0.1, 1.33), (door_w - 0.22, -0.06, 1.5), iron, bevel=0.01))
    transform(parts, Matrix.Translation((-door_w / 2 - 0.01, y0 + 0.17, dz)))
    box("threshold", (-door_w / 2 - 0.1, y0 - 0.05, dz - 0.15), (door_w / 2 + 0.1, y0 + 0.3, dz + 0.01), stone, bevel=0.03)
    # Trinnene ned foran døra.
    box("step", (-1.15, y0 - 0.6, -0.4), (1.15, y0 + 0.02, dz - 0.05), stone, bevel=0.05)
    box("step", (-1.3, y0 - 1.15, -0.45), (1.3, y0 - 0.55, -0.04), stone, bevel=0.05)
    # Støttepilarene på langsidene.
    prof = [(0.0, z0), (0.7, z0), (0.7, 0.9), (0.45, 1.35), (0.45, 2.0), (0.0, H - 0.2)]
    for side in (-1, 1):
        for y in (y0 + 0.65, y1 - 0.65):
            buttress("buttress", (side * W / 2, y, 0), (side, 0, 0), (0, 1, 0), 0.6, prof, wall)
            box("moss", (side * (W / 2 + 0.47) - 0.08, y - 0.28, 1.27), (side * (W / 2 + 0.47) + 0.12, y + 0.28, 1.42), moss, bevel=0.05)
    # Taket: tunge steinheller med mønet fra døra og bakover, som har
    # sunket i midten; gavler av stein med dekkheller, og kors foran.
    z_eave = H + 0.35
    ridge = ridge_roof(p, y0 + 0.12, y1 - 0.12, W / 2, z_eave, 42.0, barges=False, overhang=0.4, gable_over=0.0, thick=0.32, sag=0.32, kick=0.16)
    for ob in PIECES:
        if ob.name.startswith("ridge_roll"):
            ob.data.materials[0] = moss
    for y in (fy, by):
        fr, Lg = wall_frame((-W / 2 - 0.05, y), (W / 2 + 0.05, y))
        parapet(p, fr, Lg, H - 0.2, z_eave + 0.35, ridge + 0.3, T, wall, stone)
    stone_cross((0, fy, ridge + 0.4), stone, 0.9, tilt=0.06)
    # Mosen: langs foten, på dekkhellene og i flekker på taket.
    for side in (-1, 1):
        box("moss", (side * (W / 2 + 0.08) - 0.1, y0 + 0.2, -0.08), (side * (W / 2 + 0.08) + 0.1, y1 - 0.2, 0.12), moss, bevel=0.06)
    box("moss", (-W / 2 + 0.1, y1 - 0.04, -0.08), (W / 2 - 0.1, y1 + 0.12, 0.14), moss, bevel=0.06)
    tan = math.tan(math.radians(42))
    for k in range(9):
        side = 1 if k % 2 else -1
        x = side * rng.uniform(0.6, W / 2 + 0.1)
        y = rng.uniform(y0 + 0.6, y1 - 0.6)
        z = z_eave + (W / 2 - abs(x)) * tan - 0.02 - 0.32 * 0.5 * (1 - abs(x) / (W / 2)) * math.sin(math.pi * (y - y0) / (y1 - y0))
        w_, d_ = rng.uniform(0.35, 0.8), rng.uniform(0.4, 0.9)
        m = box("moss", (-w_ / 2, -d_ / 2, -0.03), (w_ / 2, d_ / 2, 0.06), moss, bevel=0.03)
        transform([m], Matrix.Translation((x, y, z)) @ Matrix.Rotation(side * math.radians(42), 4, "Y"))
    # Nedfalne steiner ved foten.
    for k in range(7):
        x = rng.choice((-1, 1)) * rng.uniform(W / 2 + 0.6, W / 2 + 1.2)
        y = rng.uniform(y0 + 0.2, y1 + 0.5)
        sz = rng.uniform(0.18, 0.35)
        b = box("rubble", (-sz, -sz * 0.8, -sz * 0.5), (sz, sz * 0.8, sz * 0.5), stone, bevel=0.05)
        b.rotation_euler = (rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3), rng.uniform(0, 3))
        b.location = (x, y, sz * 0.2)
    # Fakkelholderne ved døra (lyset settes av byen).
    for x in (-1.3, 1.3):
        torch(p, x, y0, 1.95)


def crypt():
    rng = random.Random(62)
    clear()
    PIECES.clear()
    crypt_model(rng)
    town.finish(f"{OUT}/crypt.glb", max_edge=100.0)


# --- Bygging -----------------------------------------------------------------


def two_part(name, outside_fn, inside_fn, paint):
    """Utsida og innsida som to modeller, og kolliderne, lysene og lykta
    for begge i `<navn>_inside.json`."""
    rng = random.Random(1350)
    real_write = town.write_inside
    town.write_inside = lambda path: None
    clear()
    PIECES.clear()
    outside_fn(rng)
    town.finish(f"{OUT}/{name}.glb", max_edge=100.0)
    clear()
    PIECES.clear()
    town.LIT = False
    p = paint()
    inside_fn(p, rng)
    town.finish(f"{OUT}/{name}_interior.glb", max_edge=100.0)
    town.write_inside = real_write
    real_write(f"{OUT}/{name}.glb")


def scriptorium_paint():
    p = kit.Paint(plaster="ashlar", timber="timber", roof="slate", plinth="wall_stone")
    p.plaster = layer("ashlar", (1.0, 0.95, 0.88))
    p.roof = layer("slate", (0.92, 0.95, 1.0))
    return p


def church_paint():
    p = kit.Paint(plaster="church_boards", timber="timber", roof="shingles_tar", plinth="wall_stone")
    p.timber = layer("timber", (0.78, 0.7, 0.62))
    return p


MODELS = {
    "scriptorium": lambda: two_part("scriptorium", scriptorium, scriptorium_inside, scriptorium_paint),
    "stave_church": lambda: two_part("stave_church", church, church_inside, church_paint),
    "crypt": crypt,
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        MODELS[name]()
