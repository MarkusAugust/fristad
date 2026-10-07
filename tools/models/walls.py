"""Bymuren: mursegmentet, hjørnetårnet og porthuset, modellert i Blender (bpy)
og eksportert til assets/models/town/. Kjør:

    python3 tools/models/walls.py [town_wall wall_tower gatehouse]

Bruddstein (`layer_stone`) med skrå fot, gesims, brystvern med skyteskår og
murtinder, kragsteiner, bjelkehull og vannspyere. Tårnet er rundt med
skyteskår og spontekt kjegletak; porthuset har to flankerende tårn med
pyramidetak, buet portrom med bjelketak, hevet portgitter og to
portfløyer slått inn mot veggene.

Akser i Blender som i town.py: −Y fram, Z opp, origo midt på grunnflaten.
Mursegmentet har utsida mot +Y (byen snur det så utsida vender ut), og
porthuset har utsida mot −Y (døra mot lokal +Z i motoren)."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402

import town  # noqa: E402
from town import M, PIECES, beam, block, cut, finish, prism  # noqa: F401  # noqa: E402

OUT = "assets/models/town"


# --- Hjelpere ---------------------------------------------------------------


def link(name, bm, mat):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    PIECES.append(ob)
    return ob


def planar_uv(ob):
    """Uv i meter i flatens eget plan: u vannrett langs flaten, v opp langs
    den (eller langs y for flater som ligger flatt). Takflater får radene
    vannrette og v ned fallet."""
    town.ensure_uv(ob.data)
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    uvl = bm.loops.layers.uv.verify()
    up = Vector((0, 0, 1))
    for f in bm.faces:
        n = f.normal
        if abs(n.z) > 0.95:
            e1, e2 = Vector((1, 0, 0)), Vector((0, 1, 0))
        else:
            e1 = up.cross(n).normalized()
            e2 = n.cross(e1).normalized()
        for loop in f.loops:
            p = loop.vert.co
            loop[uvl].uv = (p.dot(e1), p.dot(e2))
    bm.to_mesh(ob.data)
    bm.free()


def cyl_uv(ob, r_ref):
    """Uv rundt en sylinder: u = vinkel × radius (meter langs omkretsen),
    v = høyden. Topp og bunn får plan uv. Sømmen rettes per flate."""
    town.ensure_uv(ob.data)
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    uvl = bm.loops.layers.uv.verify()
    for f in bm.faces:
        if abs(f.normal.z) > 0.9:
            for loop in f.loops:
                p = loop.vert.co
                loop[uvl].uv = (p.x, p.y)
            continue
        c = f.calc_center_median()
        ac = math.atan2(c.y, c.x)
        for loop in f.loops:
            p = loop.vert.co
            a = math.atan2(p.y, p.x)
            while a - ac > math.pi:
                a -= 2 * math.pi
            while a - ac < -math.pi:
                a += 2 * math.pi
            loop[uvl].uv = (a * r_ref, p.z)
    bm.to_mesh(ob.data)
    bm.free()


def revolve(name, profile, mat, n=40, at=(0, 0, 0), cap=True, uv="cyl"):
    """Et omdreiningslegeme fra profilen [(r, z), …] nedenfra og opp. En
    radius 0 øverst blir en spiss."""
    bm = bmesh.new()
    rings = []
    for r, z in profile:
        if r <= 1e-4:
            rings.append([bm.verts.new((0, 0, z))])
        else:
            rings.append([bm.verts.new((r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), z)) for i in range(n)])
    for lo, hi in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            if len(lo) == 1:
                bm.faces.new((lo[0], hi[j], hi[i]))
            elif len(hi) == 1:
                bm.faces.new((lo[i], lo[j], hi[0]))
            else:
                bm.faces.new((lo[i], lo[j], hi[j], hi[i]))
    if cap and len(rings[-1]) > 1:
        bm.faces.new(rings[-1])
    if len(rings[0]) > 1:
        bm.faces.new(rings[0][::-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = link(name, bm, mat)
    if uv == "cyl":
        cyl_uv(ob, max(r for r, _ in profile))
    else:
        planar_uv(ob)
    ob.location = at
    return ob


def ring(name, r_out, r_in, z0, z1, mat, n=40):
    """En ring (brystvern på tårnet): ytre og indre vegg, topp og bunn."""
    bm = bmesh.new()
    vs = {}
    for k, (r, z) in enumerate([(r_out, z0), (r_out, z1), (r_in, z1), (r_in, z0)]):
        vs[k] = [bm.verts.new((r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), z)) for i in range(n)]
    for a, b in [(0, 1), (1, 2), (2, 3), (3, 0)]:
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((vs[a][i], vs[a][j], vs[b][j], vs[b][i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = link(name, bm, mat)
    cyl_uv(ob, r_out)
    return ob


def pyramid(name, hw, hd, z0, z1, mat, thick=0.14):
    """Et pyramidetak med tykkelse (spon som egne flater over takbordene)."""
    bm = bmesh.new()
    base = [bm.verts.new(v) for v in [(-hw, -hd, z0), (hw, -hd, z0), (hw, hd, z0), (-hw, hd, z0)]]
    apex = bm.verts.new((0, 0, z1))
    for i in range(4):
        bm.faces.new((base[i], base[(i + 1) % 4], apex))
    bm.faces.new(base[::-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = link(name, bm, mat)
    planar_uv(ob)
    return ob


def cutter(ob, target):
    """Skjær `ob` bort fra `target` og fjern `ob`."""
    PIECES.remove(ob) if ob in PIECES else None
    names = {l.name for l in target.data.uv_layers}
    mod = target.modifiers.new("cut", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.object = ob
    mod.solver = "EXACT"
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(ob)
    town.keep_uv(target, names)


def merlons(target, length, z_top, depth, y, n, gap=0.62, h=0.85, slits=True):
    """Skjær skyteskår (crenels) i toppen av et brystvern langs x, og en
    smal skytespalte midt i hver murtind."""
    step = length / n
    for k in range(n):
        x = -length / 2 + step * (k + 0.5)
        cut(target, (gap, depth + 0.4, h + 0.1), (x, y, z_top - h / 2 + 0.05))
        if slits and k < n - 1:
            xm = x + step / 2
            cut(target, (0.08, depth + 0.4, 0.75), (xm, y, z_top - h - 0.55))


def corbels(x0, x1, y, z, mat, out_sign, spacing=0.9):
    """Kragsteiner under brystvernet: tre steiner trappet utover."""
    n = max(2, int((x1 - x0) / spacing))
    for k in range(n + 1):
        x = x0 + (x1 - x0) * k / n
        for i in range(3):
            block("corbel", (0.32, 0.22 + i * 0.12, 0.18), mat, at=(x, y + out_sign * (0.11 + i * 0.06), z - 0.54 + i * 0.18), bevel=0.02, segments=2)


# --- Mursegmentet -----------------------------------------------------------


def town_wall():
    m = M()
    rng = random.Random(11)
    L = 6.24
    # Fot og murkjerne: utsida (+Y) skrår innover oppover.
    body = prism("wall_body", [(-0.82, -0.8), (1.0, -0.8), (0.8, 1.2), (0.8, 5.4), (-0.8, 5.4), (-0.82, 1.2)], L, m.stone, at=(0, 0, 0), rot=(0, 0, math.pi / 2))
    # Bjelkehull (stillaset da muren ble murt) og en vannspyer.
    for k in range(3):
        x = -L / 2 + L * (k + 0.5) / 3 + rng.uniform(-0.3, 0.3)
        for z in (2.2, 4.0):
            cut(body, (0.2, 0.5, 0.2), (x, 0.82, z))
    # Gesimsen: en steinrad som står litt ut, med kragsteiner under brystvernet.
    block("course", (L, 1.85, 0.22), m.stone, at=(0, 0.1, 5.5), bevel=0.03, segments=2)
    corbels(-L / 2 + 0.3, L / 2 - 0.3, 0.92, 5.95, m.stone, 1)
    # Brystvernet på utsida, med murtinder og skytespalter.
    para = block("parapet", (L, 0.55, 1.75), m.stone, at=(0, 0.72, 6.48), bevel=0.03, segments=2)
    merlons(para, L, 7.35, 0.55, 0.72, 4)
    # Dekksteiner på murtindene.
    step = L / 4
    for k in range(4):
        x0 = -L / 2 + step * k + 0.31
        x1 = -L / 2 + step * (k + 1) - 0.31
        block("cap", (x1 - x0 - 0.04, 0.66, 0.1), m.stone, at=((x0 + x1) / 2, 0.72, 7.4), bevel=0.025, segments=2)
    # Et lavt rekkverk av stein på innsida av murgangen.
    block("inner_rail", (L, 0.3, 0.5), m.stone, at=(0, -0.68, 5.85), bevel=0.03, segments=2)
    # Vannspyer gjennom brystvernet.
    block("spout", (0.18, 0.7, 0.14), m.stone, at=(rng.uniform(-1, 1), 1.15, 5.68), bevel=0.02)
    # Løse, framstikkende steiner i foten og i murflata.
    for _ in range(14):
        x = rng.uniform(-L / 2 + 0.3, L / 2 - 0.3)
        z = rng.uniform(0.0, 5.0)
        y_face = 1.0 - (min(z, 1.2) + 0.25) / 1.45 * 0.2 if z < 1.2 else 0.8
        block("stone", (rng.uniform(0.35, 0.7), 0.16, rng.uniform(0.22, 0.4)), m.stone, at=(x, y_face + 0.02, z), rot=(rng.uniform(-0.05, 0.05), 0, rng.uniform(-0.08, 0.08)), bevel=0.05, segments=2)
    for _ in range(8):
        x = rng.uniform(-L / 2 + 0.3, L / 2 - 0.3)
        z = rng.uniform(0.2, 5.0)
        block("stone", (rng.uniform(0.35, 0.7), 0.16, rng.uniform(0.22, 0.4)), m.stone, at=(x, -0.82, z), rot=(rng.uniform(-0.05, 0.05), 0, rng.uniform(-0.08, 0.08)), bevel=0.05, segments=2)
    return m


# --- Hjørnetårnet -----------------------------------------------------------


def wall_tower():
    m = M()
    rng = random.Random(12)
    R = 3.4
    body = revolve("tower_body", [(R + 0.45, -0.8), (R + 0.3, 1.0), (R, 2.2), (R, 10.2), (R + 0.05, 10.2)], m.stone, n=48)
    # Skyteskår i tre høyder, forskjøvet rundt.
    for z, k0 in ((3.0, 0.0), (5.6, 0.5), (8.2, 0.25)):
        for k in range(5):
            a = 2 * math.pi * (k + k0) / 5
            cut(body, (1.2, 0.1, 1.1), (R * math.cos(a), R * math.sin(a), z), rot=(0, 0, a))
            # Avfaset nisje rundt spalten.
            cut(body, (0.5, 0.42, 1.35), ((R + 0.05) * math.cos(a), (R + 0.05) * math.sin(a), z), rot=(0, 0, a))
    # Gesims og en ring av kragsteiner under takskjegget.
    revolve("tower_course", [(R + 0.18, 9.8), (R + 0.18, 10.05), (R + 0.02, 10.05)], m.stone, n=48, cap=False)
    for k in range(24):
        a = 2 * math.pi * k / 24
        ob = block("corbel", (0.4, 0.26, 0.22), m.stone, at=((R + 0.12) * math.cos(a), (R + 0.12) * math.sin(a), 9.6), rot=(0, 0, a), bevel=0.03, segments=2)
    # Takfoten: et lavt bindingsverksbelte med bjelker før taket.
    revolve("tower_eave", [(R + 0.02, 10.2), (R + 0.02, 10.9), (0.01, 10.9)], m.plaster, n=48)
    for k in range(16):
        a = 2 * math.pi * k / 16
        p0 = Vector(((R + 0.07) * math.cos(a), (R + 0.07) * math.sin(a), 10.2))
        beam("post", p0, p0 + Vector((0, 0, 0.7)), 0.14, 0.1, m.timber)
    revolve("tower_plate", [(R + 0.12, 10.85), (R + 0.12, 11.0), (R - 0.3, 11.0)], m.timber, n=48, cap=False)
    # Kjegletaket med svai nederst, spon, og et smijernsspir.
    prof = []
    for i in range(9):
        t = i / 8
        r = (R + 0.85) * (1 - t) ** 1.15
        z = 10.75 + 6.6 * t + 0.25 * (1 - (1 - t) ** 3) - 0.25 * t
        prof.append((max(r, 0.0), z))
    roof = revolve("tower_roof", prof, m.roof, n=48, cap=False, uv="planar")
    sol = roof.modifiers.new("thick", "SOLIDIFY")
    sol.thickness = 0.12
    bpy.context.view_layer.objects.active = roof
    bpy.ops.object.modifier_apply(modifier=sol.name)
    revolve("finial", [(0.06, 17.2), (0.05, 18.4), (0.0, 18.6)], m.iron, n=8)
    revolve("ball", [(0.0, 17.75), (0.11, 17.85), (0.11, 17.95), (0.0, 18.05)], m.iron, n=12)
    # Steiner som står ut i foten.
    for _ in range(18):
        a = rng.uniform(0, 2 * math.pi)
        z = rng.uniform(2.4, 9.4)
        block("stone", (0.14, rng.uniform(0.35, 0.65), rng.uniform(0.22, 0.38)), m.stone, at=((R + 0.04) * math.cos(a), (R + 0.04) * math.sin(a), z), rot=(0, 0, a), bevel=0.05, segments=2)
    return m


# --- Porthuset --------------------------------------------------------------


def gatehouse():
    m = M()
    rng = random.Random(13)
    W, D = 13.2, 5.0
    pw, ph = 5.2, 3.3
    # Midtblokken over portrommet.
    mid = block("gate_mid", (pw + 0.4, D - 0.6, 9.15), m.stone, at=(0, 0, 3.775), bevel=0.04, segments=2)
    # Flanketårnene, litt framskutt og høyere, med skrå fot.
    towers = []
    for sx in (-1, 1):
        x = sx * (pw / 2 + 2.0)
        bm = bmesh.new()
        prof = [(-0.8, 2.25), (1.2, 2.05), (2.4, 2.0), (11.2, 2.0)]
        rings = []
        for z, hw in prof:
            rings.append([bm.verts.new((x + px * hw, py * D / 2 * hw / 2.0, z)) for px, py in ((-1, -1), (1, -1), (1, 1), (-1, 1))])
        for lo, hi in zip(rings, rings[1:]):
            for i in range(4):
                j = (i + 1) % 4
                bm.faces.new((lo[i], lo[j], hi[j], hi[i]))
        bm.faces.new(rings[-1])
        bm.faces.new(rings[0][::-1])
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        t = link("gate_tower", bm, m.stone)
        town.uv_box(t)
        towers.append(t)
        # Skyteskår mot utsida og innsida.
        for z in (3.2, 6.6, 9.4):
            cut(t, (0.1, 1.2, 1.0), (x, -D / 2, z))
            cut(t, (0.45, 0.5, 1.3), (x, -D / 2, z))
        for z in (6.6,):
            cut(t, (0.1, 1.2, 1.0), (x, D / 2, z))
        # Gesims og pyramidetak med spon.
        block("course", (4.25, D + 0.25, 0.2), m.stone, at=(x, 0, 11.15), bevel=0.03, segments=2)
        roof = pyramid("gate_roof", 2.45, D / 2 + 0.45, 11.25, 16.2, m.roof)
        sol = roof.modifiers.new("thick", "SOLIDIFY")
        sol.thickness = 0.12
        bpy.context.view_layer.objects.active = roof
        bpy.ops.object.modifier_apply(modifier=sol.name)
        roof.location.x = x
        revolve("finial", [(0.06, 16.1), (0.05, 17.2), (0.0, 17.4)], m.iron, n=8, at=(x, 0, 0))
    # Portrommet: rett nede, rundbue oppe, gjennom hele midtblokken og
    # inn i tårnenes innerkanter.
    for target in [mid] + towers:
        cut(target, (pw, D + 2, ph), (0, 0, ph / 2 - 0.3))
        bpy.ops.mesh.primitive_cylinder_add(vertices=40, radius=pw / 2, depth=D + 2, location=(0, 0, ph - 0.3), rotation=(math.pi / 2, 0, 0))
        cutter(bpy.context.active_object, target)
    town.uv_box(mid)
    for t in towers:
        town.uv_box(t)
    # Buesteiner rundt åpningen på begge sider.
    for sy in (-1, 1):
        for k in range(15):
            a = math.pi * k / 14
            c = Vector((math.cos(a) * (pw / 2 + 0.2), sy * (D / 2 - 0.25), ph - 0.3 + math.sin(a) * (pw / 2 + 0.2)))
            block("voussoir", (0.26, 0.2, 0.5), m.stone, at=c, rot=(0, math.pi / 2 - a, 0), bevel=0.03, segments=2)
    # Bjelketak i portrommet.
    for k in range(7):
        y = -D / 2 + 0.5 + (D - 1.0) * k / 6
        beam("ceiling", (-pw / 2, y, ph - 0.35), (pw / 2, y, ph - 0.35), 0.22, 0.26, m.timber)
    # Brystvern med murtinder og kragsteiner over midtblokken, foran og bak.
    for sy in (-1, 1):
        corbels(-pw / 2, pw / 2, sy * (D / 2 - 0.3), 8.35, m.stone, sy)
        para = block("parapet", (pw + 0.4, 0.5, 1.6), m.stone, at=(0, sy * (D / 2 - 0.05), 9.1), bevel=0.03, segments=2)
        merlons(para, pw + 0.4, 9.9, 0.5, sy * (D / 2 - 0.05), 4)
    # Portgitteret, heist opp: eik med jernspisser, synlig i buen.
    for k in range(13):
        x = -pw / 2 + 0.2 + (pw - 0.4) * k / 12
        top = ph - 0.3 + math.sqrt(max(0.0, (pw / 2) ** 2 - x * x)) + 0.4
        beam("gate_bar", (x, -D / 2 + 0.55, ph + 0.75), (x, -D / 2 + 0.55, top), 0.1, 0.1, m.timber)
        revolve("spike", [(0.05, 0.0), (0.0, -0.22)], m.iron, n=6, at=(x, -D / 2 + 0.55, ph + 0.75))
    for z in (ph + 0.95, ph + 1.5, ph + 2.05):
        half = math.sqrt(max(0.0, (pw / 2) ** 2 - (z - (ph - 0.3)) ** 2))
        beam("gate_rail", (-half + 0.05, -D / 2 + 0.47, z), (half - 0.05, -D / 2 + 0.47, z), 0.08, 0.1, m.timber)
    # Portfløyene slått inn mot veggene i portrommet, med jernbånd og nagler.
    for sx in (-1, 1):
        x = sx * (pw / 2 - 0.1)
        block("leaf", (0.12, 2.5, ph - 0.1), m.boards, at=(x, 0.6, (ph - 0.1) / 2 - 0.25), bevel=0.01)
        for z in (0.2, 1.4, 2.5):
            block("band", (0.14, 2.3, 0.08), m.iron, at=(x, 0.6, z), bevel=0.005)
            for i in range(9):
                block("nail", (0.16, 0.035, 0.035), m.iron, at=(x, 0.6 - 1.0 + 2.0 * i / 8, z), bevel=0.008)
    # Våpenskjoldet over buen: en innfelt steinplate.
    for sy in (-1,):
        block("panel", (1.1, 0.12, 1.3), m.stone, at=(0, sy * (D / 2 - 0.25) + sy * 0.06, 6.6), bevel=0.05, segments=2)
        block("panel_frame", (1.35, 0.08, 1.55), m.stone, at=(0, sy * (D / 2 - 0.25) + sy * 0.02, 6.6), bevel=0.03, segments=2)
    # Steiner som står ut.
    for _ in range(16):
        sx = rng.choice((-1, 1))
        x = sx * rng.uniform(pw / 2 + 0.5, pw / 2 + 3.6)
        z = rng.uniform(2.6, 10.6)
        block("stone", (rng.uniform(0.35, 0.65), 0.14, rng.uniform(0.22, 0.38)), m.stone, at=(x, -D / 2 - 0.04, z), bevel=0.05, segments=2)
    return m


MODELS = {"town_wall": town_wall, "wall_tower": wall_tower, "gatehouse": gatehouse}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb")
