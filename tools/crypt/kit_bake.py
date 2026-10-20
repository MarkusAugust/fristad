"""Eksporten av byggesettet for nivåredigeringen (`kit_cells.py`) med
Blender: okklusjonen bakt i sammenheng, fjernversjonen, festene og
manifestet i `extras.kit`, og tetthetssjekken per bit
(docs/nivaaredigering.md, del 4.3).

- **Okklusjonen i sammenheng:** hver bit bakes stående blant naboene den
  vanligvis har (`context`): romfelt, ganger, murer og hjørner fra det samme
  settet. Mot skjøtene (`SEAMS`: der biten møter en annen bit) går den over i
  det nøytrale, som er den samme for alle bitene i samme høyde (`profile`: en
  rett mur bakt i sin sammenheng), så to biter som møtes har samme mørke i
  skjøten og ingen kant.
- **Fjernversjonen** er noden `far` i samme glb-fil (uten sokkel, karmer,
  ribber og nisjer); `gltf.rs` legger den i det grove detaljnivået.
- **Tetthetssjekken** (`check`) skyter stråler mot flatene en bit sier er
  lukket: murene og hjørnene rett gjennom (der det ikke er en åpning skal
  strålen treffe, gjennom åpningen skal den ikke), gulvet ovenfra og hvelvet
  nedenfra. Eksporten stopper på en bit som lekker, og på en bit med
  strukket eller forstørret tekstur (`kit_grav.uv_stretch`: uv i meter,
  skala og skjevhet innenfor `UV_SCALE` og `UV_ANISO`).
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "tools", "models"))
import kit_grav as K  # noqa: E402
from kit_common import FACE, HALF, WALL_FOOT, WALL_T, WALL_TOP  # noqa: E402

FADE = 0.35  # okklusjonen går over i det nøytrale over så mange meter mot skjøten
AO_DISTANCE = 0.5
SAMPLES = 64
SIDE = [(0, -1), (1, 0), (0, 1), (-1, 0)]  # N, Ø, S, V


def side_yaw(w):
    return -w * math.pi / 2


def placed(mb, yaw=0.0, at=(0.0, 0.0, 0.0)):
    """En kopi av flatene dreid og flyttet (uv beholdes, de spiller ingen
    rolle for okklusjonen)."""
    out = K.MB()
    out.extend(mb, yaw, at, keep_uv=True)
    return out


# --- Omgivelsene -----------------------------------------------------------------


def context(md, piece, models):
    """Naboene biten bakes blant (flater i bitens rom)."""
    near = lambda n: models[n].near
    room, wall, corner = near("grav_romfelt"), near("grav_vegg"), near("grav_hjorne")
    out = []
    ctx = md.ctx
    if ctx in ("cell", "gang", "gangkryss"):
        faces = [f["kind"] for f in piece["faces"]]
        floors = piece["floors"]
        for w, (ox, oz) in enumerate(SIDE):
            at = (ox * 2 * HALF, floors[w], oz * 2 * HALF)
            if faces[w] == "open":
                out.append(placed(room, 0.0, at))
            elif faces[w] == "gang":
                out.append(placed(near("grav_gang"), 0.0 if ox == 0 else math.pi / 2, at))
            else:
                out.append(placed(wall, side_yaw(w), (ox * HALF, floors[w], oz * HALF)))
        for k, (vx, vz) in enumerate(((-1, -1), (1, -1), (1, 1), (-1, 1))):
            a, b = k, (k + 3) % 4
            if faces[a] != "open" or faces[b] != "open":
                out.append(placed(corner, -k * math.pi / 2, (vx * HALF, 0.0, vz * HALF)))
        if ctx == "cell" and all(f == "open" for f in faces):
            for vx in (-1, 1):
                for vz in (-1, 1):
                    out.append(placed(room, 0.0, (vx * 2 * HALF, 0.0, vz * 2 * HALF)))
    elif ctx in ("edge", "arch", "rail"):
        # Halvkanten på nordsida av cella (0, 2); naboen bak den.
        out.append(placed(room, 0.0, (0.0, 0.0, HALF)))
        if ctx != "rail":
            out.append(placed(md.near, math.pi, (0.0, 0.0, 0.0)))
            out.append(placed(room, 0.0, (0.0, 0.0, -HALF)))
        if ctx == "edge":
            for sx in (-1, 1):
                out.append(placed(wall, 0.0, (sx * 2 * HALF, 0.0, 0.0)))
                out.append(placed(corner, 0.0 if sx < 0 else -math.pi / 2, (sx * HALF, 0.0, 0.0)))
    elif ctx == "corner":
        # Innvendig hjørne i cella (2, 2): murene mot nord og vest.
        out.append(placed(room, 0.0, (HALF, 0.0, HALF)))
        out.append(placed(wall, 0.0, (HALF, 0.0, 0.0)))
        out.append(placed(wall, side_yaw(3), (0.0, 0.0, HALF)))
    elif ctx == "pillar":
        for vx in (-1, 1):
            for vz in (-1, 1):
                out.append(placed(room, 0.0, (vx * HALF, 0.0, vz * HALF)))
    elif ctx == "torch":
        out.append(placed(wall, 0.0, (0.0, -2.3, -WALL_T)))
        out.append(placed(room, 0.0, (0.0, -2.3, HALF - WALL_T)))
    elif ctx == "leaf":
        out.append(placed(near("grav_vegg_falsk"), 0.0, (0.0, 0.0, -WALL_T)))
        out.append(placed(room, 0.0, (0.0, 0.0, HALF - WALL_T)))
    return out


def seam_distance(seams, p):
    """Avstanden fra punktet (bitens rom) til nærmeste skjøt."""
    x, y, z = p
    if seams == "cell":
        return min(HALF - abs(x), HALF - abs(z))
    if seams == "edge":
        return min(abs(FACE - abs(x)), abs(HALF - abs(x)), abs(z))
    if seams == "corner":
        return min(abs(x), abs(z), abs(x - WALL_T), abs(z - WALL_T))
    if seams == "rotunde":
        return min(3 * HALF - abs(x), 3 * HALF - abs(z))
    return 1e9


# --- Blender ---------------------------------------------------------------------


def ours(co):
    """Blenders akser (x, −z, y) tilbake til spillets (x, y, z)."""
    return (co[0], co[2], -co[1])


def ao_values(ob):
    import numpy as np

    attr = ob.data.color_attributes["ao"]
    col = np.empty(len(attr.data) * 4, dtype=np.float32)
    attr.data.foreach_get("color", col)
    return col.reshape(-1, 4)


def set_ao(ob, col):
    ob.data.color_attributes["ao"].data.foreach_set("color", col.ravel())


def vertex_normals(ob):
    return [ours(v.normal) for v in ob.data.vertices]


def wall_profile(ob):
    """Det nøytrale mørket i en mur etter høyden: snittet av okklusjonen på
    murflaten midt på en rett mur (z ≈ 0,4 eller sokkelen foran), i trinn på
    5 cm."""
    col = ao_values(ob)
    bins = {}
    for v, n in zip(ob.data.vertices, vertex_normals(ob)):
        p = ours(v.co)
        if abs(p[0]) < 1.0 and n[2] > 0.7 and p[2] > WALL_T - 0.01:
            bins.setdefault(round(p[1] / 0.05), []).append(col[v.index, 0])
    pts = sorted((k * 0.05, sum(vs) / len(vs)) for k, vs in bins.items())
    return pts


def at_height(profile, y):
    if not profile:
        return 1.0
    if y <= profile[0][0]:
        return profile[0][1]
    for (y0, a), (y1, b) in zip(profile, profile[1:]):
        if y <= y1:
            t = (y - y0) / max(y1 - y0, 1e-6)
            return a + (b - a) * t
    return profile[-1][1]


def neutralize(ob, seams, profile):
    """Okklusjonen går over i det nøytrale mot skjøtene: flater som vender
    opp eller ned får 1, loddrette flater mørket en rett mur har i samme
    høyde."""
    if seams == "none":
        return
    col = ao_values(ob)
    for v, n in zip(ob.data.vertices, vertex_normals(ob)):
        p = ours(v.co)
        d = seam_distance(seams, p)
        if d >= FADE:
            continue
        t = max(0.0, d / FADE)
        w = t * t * (3 - 2 * t)
        neutral = 1.0 if abs(n[1]) > 0.6 else at_height(profile, p[1])
        a = neutral + (col[v.index, 0] - neutral) * w
        col[v.index, 0:3] = a
    set_ao(ob, col)


def bake(md, piece, models):
    """Nær- og fjernversjonen som Blender-objekter med okklusjonen bakt i
    sammenheng (ikke gjort nøytral ennå)."""
    import bpy

    K.clear()
    near = K.to_object(md.near, md.name)
    K.sharpen(near)
    ctx = [K.to_object(mb, f"ctx{k}") for k, mb in enumerate(context(md, piece, models))]
    K.bake_ao(near, samples=SAMPLES, distance=AO_DISTANCE)
    far = K.to_object(md.far, "far")
    K.sharpen(far)
    near.hide_render = True
    K.bake_ao(far, samples=SAMPLES // 2, distance=AO_DISTANCE)
    near.hide_render = False
    for o in ctx:
        bpy.data.objects.remove(o, do_unlink=True)
    return near, far


def write_kit(path, entries):
    import slim

    j, bin_ = slim.read(path)
    j.setdefault("extras", {})["kit"] = entries
    slim.write(path, j, bin_)


def export_all(pieces, models, out, only=()):
    from kit_cells import kit_extras

    bad = check_all(pieces, models, only)
    if bad:
        raise SystemExit(f"tetthetssjekken stopper eksporten: {', '.join(bad)}")
    first = {}
    for p in pieces:
        first.setdefault(p["model"], p)
        for v in p.get("variants", []):
            first.setdefault(v["model"], p)
    # Den rette muren først: den gir det nøytrale mørket etter høyden.
    order = ["grav_vegg"] + [n for n in models if n != "grav_vegg"]
    profile = None
    for name in order:
        md = models[name]
        near, far = bake(md, first[name], models)
        if profile is None:
            profile = wall_profile(near)
            print("  det nøytrale mørket i muren:", ", ".join(f"{y:.2f}:{a:.2f}" for y, a in profile[:: max(1, len(profile) // 8)]))
        neutralize(near, md.seams, profile)
        neutralize(far, md.seams, profile)
        if only and name not in only:
            continue
        path = os.path.join(out, f"{name}.glb")
        K.export_glb([near, far], path)
        write_kit(path, kit_extras(pieces, name))


# --- Tetthetssjekken -------------------------------------------------------------


def bvh(mb):
    import bpy  # noqa: F401 (mathutils kommer med bpy)
    from mathutils.bvhtree import BVHTree

    verts, polys, index = [], [], {}
    for pts, _, _, _ in mb.faces:
        ids = []
        for p in pts:
            k = (round(p[0], 5), round(p[1], 5), round(p[2], 5))
            if k not in index:
                index[k] = len(verts)
                verts.append(k)
            ids.append(index[k])
        polys.append(ids)
    return BVHTree.FromPolygons(verts, polys, all_triangles=False)


def hits(tree, o, d, dist):
    from mathutils import Vector

    loc, _, _, _ = tree.ray_cast(Vector(o), Vector(d), dist)
    return loc is not None


def inside(poly, u, v, margin):
    """Om (u, v) er inne i omrisset trukket `margin` inn (negativ: ut)."""
    from shapely.geometry import Point, Polygon

    return Polygon(poly).buffer(-margin).contains(Point(u, v))


def check(piece, mb, name):
    """Brudd på tettheten for biten (tom liste: tett)."""
    from kit_cells import opening

    tree = bvh(mb)
    out = []
    kind = piece["kind"]
    step = 0.1
    if kind == "edge":
        op = piece.get("opening")
        if op == "arch":
            return out
        if op in ("overmur", "rail"):
            lo, hi = (WALL_TOP, WALL_TOP + 1.95) if op == "overmur" else (0.05, 0.9)
            for i in range(1, 32):
                u = -FACE + 2 * FACE * i / 32
                for v in [lo + (hi - lo) * k / 8 for k in range(9)]:
                    if not hits(tree, (u, v, 1.0), (0, 0, -1), 1.0):
                        out.append(f"{name}: hull i {op} ved u={u:.2f} v={v:.2f}")
            return out
        prof, _ = opening(op)
        u = -FACE + 0.05
        while u < FACE - 0.04:
            v = 0.05
            while v < WALL_TOP - 0.04:
                if prof and inside(prof, u, v, 0.05):
                    if hits(tree, (u, v, 1.0), (0, 0, -1), 1.2):
                        out.append(f"{name}: åpningen er tett ved u={u:.2f} v={v:.2f}")
                elif not prof or not inside(prof, u, v, -0.05):
                    if not hits(tree, (u, v, 1.0), (0, 0, -1), 1.0):
                        out.append(f"{name}: hull i muren ved u={u:.2f} v={v:.2f}")
                v += step
            u += step
    elif kind == "corner":
        for a in [0.02 + 0.36 * k / 6 for k in range(7)]:
            for v in [0.05 + (WALL_TOP - 0.1) * k / 30 for k in range(31)]:
                if not hits(tree, (a, v, 1.0), (0, 0, -1), 1.0) or not hits(tree, (1.0, v, a), (-1, 0, 0), 1.0):
                    out.append(f"{name}: hull i hjørnet ved {a:.2f}, {v:.2f}")
    elif kind == "cell" and "footprint" not in piece:
        holes = []
        for m in piece.get("mounts", []):
            if m["kind"] == "down":
                holes.append((-1.2, -HALF - 0.1, 1.2, m["pos"][2]))
        top = piece["top"]
        for i in range(40):
            for j in range(40):
                x, z = -HALF + 0.05 + i * 0.1, -HALF + 0.05 + j * 0.1
                if any(h[0] - 0.05 < x < h[2] + 0.05 and h[1] - 0.05 < z < h[3] + 0.05 for h in holes):
                    continue
                hi = max(piece["floors"]) + 0.1
                if not hits(tree, (x, hi + 0.5, z), (0, -1, 0), hi + 1.0):
                    out.append(f"{name}: hull i gulvet ved {x:.2f}, {z:.2f}")
                if abs(x) < FACE - 0.05 and abs(z) < FACE - 0.05 and not hits(tree, (x, hi + 0.3, z), (0, 1, 0), top + 1.0):
                    out.append(f"{name}: hull i hvelvet ved {x:.2f}, {z:.2f}")
    elif kind == "cell":
        # Rotunden og bosshallen: hvelvet og muren rundt fra midten.
        from kit_cells import ROT_GANG, ROT_R

        gang = math.asin(ROT_GANG / ROT_R) + 0.03
        for k in range(72):
            a = 2 * math.pi * k / 72
            # Gangene ut går til kanten av kvadratet, der naboen tar over.
            if min(abs(((a - q * math.pi / 2 + math.pi) % (2 * math.pi)) - math.pi) for q in range(4)) < gang:
                continue
            for v in (0.5, 1.5, 2.5, 3.3):
                d = (math.cos(a), 0.0, math.sin(a))
                # Til hjørnet av kvadratet (bosshallen er firkantet).
                if not hits(tree, (0.0, v, 0.0), d, 3 * HALF * 1.5):
                    out.append(f"{name}: hull i muren mot {math.degrees(a):.0f}° i {v} m")
            for r in (1.0, 3.0, 4.6):
                if not hits(tree, (r * math.cos(a), 1.0, r * math.sin(a)), (0, 1, 0), 12.0):
                    out.append(f"{name}: hull i kuppelen ved r={r} {math.degrees(a):.0f}°")
    return out


def check_all(pieces, models, only=()):
    """Tetthetssjekken for alle bitene (nær og fjern). Gir modellene som lekker."""
    bad = []
    seen = set()
    for p in pieces:
        names = [p["model"]] + [v["model"] for v in p.get("variants", [])]
        for name in names:
            if name in seen or (only and name not in only):
                continue
            seen.add(name)
            for which, mb in (("nær", models[name].near), ("fjern", models[name].far)):
                errs = [f"strukket uv: {b}" for b in K.uv_stretch(mb)] + check(p, mb, f"{name} ({which})")
                if errs:
                    bad.append(name)
                    print(f"  {name} ({which}): {len(errs)} brudd, f.eks. {errs[0]}")
    print(f"tetthetssjekken: {len(seen)} modeller, {len(bad)} lekker eller har strukket uv")
    return bad
